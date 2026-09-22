use observation_domain::{CompleteMessage, ShipSectionIdentity, ShipSectionKind, SourceScopeId};
use observation_ingest::CarrierIdentity;

#[derive(Default)]
pub(super) struct ShipTiming {
    observed: [u64; 3],
    pending: Option<(CarrierIdentity, String, u64, u64, SourceScopeId)>,
}

fn family(key: &str) -> Option<usize> {
    match ShipSectionIdentity::parse(key)?.kind() {
        ShipSectionKind::Cargo => Some(0),
        ShipSectionKind::Crew => Some(1),
        ShipSectionKind::Loadout => Some(2),
        ShipSectionKind::Core => None,
    }
}

impl ShipTiming {
    pub(super) fn issued(
        &mut self,
        identity: &CarrierIdentity,
        key: &str,
        revision: u64,
        now: u64,
        scope: &SourceScopeId,
    ) {
        self.pending = family(key).map(|_| {
            (
                identity.clone(),
                key.to_owned(),
                revision,
                now,
                scope.clone(),
            )
        });
    }

    pub(super) fn clear(&mut self) {
        self.pending = None;
    }

    pub(super) fn observed(&self, key: &str) -> u64 {
        family(key).map_or(0, |index| self.observed[index])
    }

    pub(super) fn completed(&mut self, message: &CompleteMessage, now: u64) {
        let CompleteMessage::SectionCompletion(value) = message else {
            return;
        };
        let Some((identity, key, revision, started, scope)) = &self.pending else {
            return;
        };
        if value.producer_incarnation.as_str() != identity.producer_incarnation
            || value.transport_epoch != identity.epoch
            || value.section_key.as_str() != key
            || value.section_revision.get() != *revision
            || &value.source_scope != scope
        {
            return;
        }
        if let (Some(index), Some(duration)) = (family(key), now.checked_sub(*started)) {
            self.observed[index] = duration;
        }
        self.clear();
    }
}
