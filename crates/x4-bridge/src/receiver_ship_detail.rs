use crate::ProductionError;
use crate::receiver_ship_detail_record::record_dependency;
pub use crate::receiver_ship_detail_record::validate_batch;
use observation_application::{LifecycleError, ObservationLifecycle};
use observation_domain::{CompleteMessage, ImmutableBatchEnvelope, SectionKey, SectionRevisionId};
use observation_persistence::{CurrentRevision, ObservationRepository};
use std::collections::BTreeMap;

const fn rejected() -> ProductionError {
    ProductionError::Lifecycle(LifecycleError::AuthorityRejected)
}

pub fn is_key(key: &str) -> bool {
    ["ship_cargo:g", "ship_crew:g", "ship_loadout:g"]
        .iter()
        .any(|prefix| {
            key.strip_prefix(prefix)
                .is_some_and(|v| v.parse::<u16>().is_ok_and(|n| n.to_string() == v))
        })
}

fn core<R: ObservationRepository>(
    lifecycle: &ObservationLifecycle<R>,
) -> Result<CurrentRevision, ProductionError> {
    let key = SectionKey::new("ship_core").ok_or_else(rejected)?;
    lifecycle
        .current_revision(&key)
        .map_err(|_| ProductionError::Storage)?
        .ok_or_else(rejected)
}

fn group(
    parent: &observation_persistence::RevisionRecord,
    key: &str,
    group_members: usize,
) -> Result<observation_domain::ShipDetailGroup, ProductionError> {
    use observation_domain::{
        ObservationPolicyVersion, ShipDetailGroup, ShipGroupDescriptor, ShipIdentity,
    };
    let ordinal = key
        .split_once(":g")
        .map(|(_, ordinal)| ordinal)
        .and_then(|v| v.parse::<u16>().ok())
        .ok_or_else(rejected)?;
    let members = parent
        .records
        .iter()
        .map(|v| {
            v.entity_id
                .as_str()
                .strip_prefix("x4:ship:")
                .ok_or_else(rejected)
                .and_then(|id| ShipIdentity::new(id).map_err(|_| rejected()))
        })
        .collect::<Result<Vec<_>, _>>()?;
    let descriptor = ShipGroupDescriptor::new(
        parent.source_scope.clone(),
        parent.revision,
        ObservationPolicyVersion::new(2).ok_or_else(rejected)?,
        members,
    )
    .map_err(|_| rejected())?;
    ShipDetailGroup::new(descriptor, ordinal, group_members).map_err(|_| rejected())
}

pub fn dependencies<R: ObservationRepository>(
    lifecycle: &ObservationLifecycle<R>,
    key: &str,
) -> Result<BTreeMap<SectionKey, SectionRevisionId>, ProductionError> {
    if !is_key(key) {
        return Ok(BTreeMap::new());
    }
    let current = core(lifecycle)?;
    Ok(BTreeMap::from([(
        current.revision().section_key.clone(),
        current.receipt().revision,
    )]))
}

pub fn validate_dependency<R: ObservationRepository>(
    lifecycle: &ObservationLifecycle<R>,
    message: &CompleteMessage,
    group_members: usize,
    inner_limit: usize,
    freshness: Option<(u64, u64)>,
) -> Result<(), ProductionError> {
    let (key, scope, incarnation, epoch) = match message {
        CompleteMessage::SectionStart(v) => (
            &v.section_key,
            &v.source_scope,
            &v.producer_incarnation,
            v.transport_epoch,
        ),
        CompleteMessage::ImmutableBatch(v) => (
            &v.section_key,
            &v.source_scope,
            &v.producer_incarnation,
            v.transport_epoch,
        ),
        CompleteMessage::SectionCompletion(v) => (
            &v.section_key,
            &v.source_scope,
            &v.producer_incarnation,
            v.transport_epoch,
        ),
        CompleteMessage::Control(_) => return Ok(()),
    };
    if !is_key(key.as_str()) {
        return Ok(());
    }
    let current = core(lifecycle)?;
    if freshness.is_some_and(|(now, max_age)| {
        now < current.receipt().accepted_at || now - current.receipt().accepted_at > max_age
    }) {
        return Err(ProductionError::StaleShipParent);
    }
    let parent = current.revision();
    let expected_group = group(parent, key.as_str(), group_members)?;
    if &parent.source_scope != scope
        || parent.source_session.producer_incarnation() != incarnation
        || parent.source_session.transport_epoch() != epoch
    {
        return Err(rejected());
    }
    if let CompleteMessage::ImmutableBatch(batch) = message {
        for record in &batch.records {
            crate::receiver_ship_detail_record::record_dependency_with_limit(
                &record.content,
                inner_limit,
            )?;
        }
        validate_members(batch, parent)?;
        let member = expected_group
            .members()
            .get(batch.section_ordinal.saturating_sub(1))
            .ok_or_else(rejected)?;
        if batch
            .records
            .first()
            .is_none_or(|v| v.entity_id.as_str() != format!("x4:ship:{}", member.as_str()))
        {
            return Err(rejected());
        }
    }
    validate_group_count(message, expected_group.members().len())?;
    Ok(())
}

fn validate_group_count(message: &CompleteMessage, expected: usize) -> Result<(), ProductionError> {
    let count = match message {
        CompleteMessage::SectionStart(v) => Some(v.expected_records),
        CompleteMessage::SectionCompletion(v) => Some(v.record_count),
        _ => None,
    };
    if count.is_some_and(|v| v != expected) {
        return Err(rejected());
    }
    Ok(())
}

fn validate_members(
    batch: &ImmutableBatchEnvelope,
    parent: &observation_persistence::RevisionRecord,
) -> Result<(), ProductionError> {
    for record in &batch.records {
        let dependency = record_dependency(&record.content)?;
        if dependency.core_revision != parent.revision
            || dependency.member_revision != parent.revision
            || dependency.capture.start_millis()
                < parent
                    .context
                    .candidate(parent.dependencies.clone(), parent.expected_current)
                    .state()
                    .capture_window()
                    .end_millis()
            || !parent
                .records
                .iter()
                .any(|v| v.entity_id == record.entity_id)
        {
            return Err(rejected());
        }
    }
    Ok(())
}
