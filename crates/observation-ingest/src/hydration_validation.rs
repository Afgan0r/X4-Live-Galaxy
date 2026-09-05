use std::collections::BTreeMap;

use observation_domain::{
    CanonicalizationVersion, CompletionCoverage, DigestAlgorithmVersion, ObservationPolicyVersion,
    ObservationSchemaVersion, SectionCoverage,
};

use crate::{
    ContractVersions, DurableRevisionError, DurableRevisionParts, completion_digest::content_digest,
};

pub fn validate(parts: &DurableRevisionParts) -> Result<(), DurableRevisionError> {
    validate_context(parts)?;
    validate_records(parts)?;
    if content_digest(&parts.records) != parts.content_digest {
        return Err(DurableRevisionError::ContentDigest);
    }
    if parts
        .context
        .expected_current()
        .is_some_and(|current| current >= parts.section_revision)
    {
        return Err(DurableRevisionError::RevisionOrder);
    }
    Ok(())
}

fn validate_context(parts: &DurableRevisionParts) -> Result<(), DurableRevisionError> {
    let expected = ContractVersions::new(
        ObservationSchemaVersion::new(1).ok_or(DurableRevisionError::ContractVersion)?,
        ObservationPolicyVersion::new(2).ok_or(DurableRevisionError::ContractVersion)?,
        CanonicalizationVersion::new(3).ok_or(DurableRevisionError::ContractVersion)?,
        DigestAlgorithmVersion::new(1).ok_or(DurableRevisionError::ContractVersion)?,
    );
    if parts.context.versions() != expected {
        return Err(DurableRevisionError::ContractVersion);
    }
    if parts.context.capture_window() != parts.context.state().capture_window() {
        return Err(DurableRevisionError::ContextEvidence);
    }
    let coverage = match parts.context.state().coverage() {
        SectionCoverage::Complete => CompletionCoverage::Complete,
        SectionCoverage::KnownEmpty => CompletionCoverage::KnownEmpty,
        SectionCoverage::Partial => CompletionCoverage::Partial,
        SectionCoverage::Unknown => CompletionCoverage::Unknown,
        SectionCoverage::Unsupported => CompletionCoverage::Unsupported,
    };
    if coverage != parts.coverage
        || (coverage == CompletionCoverage::KnownEmpty
            && (!parts.records.is_empty() || !parts.context.stable_identity()))
    {
        return Err(DurableRevisionError::Coverage);
    }
    Ok(())
}

fn validate_records(parts: &DurableRevisionParts) -> Result<(), DurableRevisionError> {
    if parts
        .records
        .windows(2)
        .any(|pair| pair[0].record_id >= pair[1].record_id)
    {
        return Err(DurableRevisionError::RecordOrder);
    }
    let mut entities = BTreeMap::new();
    for record in &parts.records {
        let value = (record.observation_version, record.content.as_str());
        if entities
            .insert(&record.entity_id, value)
            .is_some_and(|existing| existing != value)
        {
            return Err(DurableRevisionError::EntityVersion);
        }
    }
    Ok(())
}
