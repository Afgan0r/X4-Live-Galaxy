use crate::completion_types::Candidate;
use crate::{AdmissionError, CandidateUsage};
use observation_domain::{
    ProducerIncarnationId, SectionKey, SectionRevisionId, SectionStartEnvelope, SourceScopeId,
    TransportEpoch,
};
use std::collections::BTreeMap;

pub fn build_legacy_candidate(
    key: &SectionKey,
    version: u64,
    generation: u64,
    receipt: u64,
) -> Result<Candidate, AdmissionError> {
    let source_scope =
        SourceScopeId::new(key.as_str().to_owned()).ok_or(AdmissionError::InvalidScope)?;
    let section_revision = SectionRevisionId::new(version).ok_or(AdmissionError::InvalidVersion)?;
    let producer_incarnation = ProducerIncarnationId::new(format!("legacy:{generation}"))
        .ok_or(AdmissionError::InvalidScope)?;
    let transport_epoch = TransportEpoch::new(generation).ok_or(AdmissionError::InvalidVersion)?;
    Ok(Candidate {
        start: SectionStartEnvelope {
            source_scope,
            producer_incarnation,
            transport_epoch,
            section_key: key.clone(),
            section_revision,
            expected_records: 0,
        },
        usage: CandidateUsage::default(),
        started_at: receipt,
        last_progress_at: receipt,
        batches: BTreeMap::new(),
        legacy_identity: Some((key.as_str().to_owned(), version, generation)),
        next_sequence: 1,
        legacy_frames: Vec::new(),
        context: None,
        provisional_versions: BTreeMap::new(),
    })
}
