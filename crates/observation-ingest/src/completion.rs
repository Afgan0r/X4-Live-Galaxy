use std::collections::BTreeMap;

use observation_domain::{
    CanonicalObservationKey, CanonicalizationVersion, CaptureWindow, CompletionCoverage,
    DigestAlgorithmVersion, EnvelopeRecord, ObservationPolicyVersion, ObservationSchemaVersion,
    ObservationVersion, SectionCompletionEnvelope, SectionKey, SectionRevisionId, SectionState,
    SourceScopeId,
};
use sha2::{Digest, Sha256};

use crate::{GenerationStager, ReceiverDisposition, RejectionReason};

#[must_use]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CompletedScope {
    version: ObservationVersion,
    members: Vec<CanonicalObservationKey>,
}

impl CompletedScope {
    pub fn new(version: ObservationVersion, mut members: Vec<CanonicalObservationKey>) -> Self {
        members.sort();
        members.dedup();
        Self { version, members }
    }
    pub const fn version(&self) -> ObservationVersion {
        self.version
    }
    pub fn is_exact_replay(
        &self,
        version: ObservationVersion,
        members: &[CanonicalObservationKey],
    ) -> bool {
        Self::new(version, members.to_vec()) == *self
    }
}

#[must_use]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ContractVersions {
    schema: ObservationSchemaVersion,
    policy: ObservationPolicyVersion,
    canonicalization: CanonicalizationVersion,
    digest: DigestAlgorithmVersion,
}

impl ContractVersions {
    pub const fn new(
        schema: ObservationSchemaVersion,
        policy: ObservationPolicyVersion,
        canonicalization: CanonicalizationVersion,
        digest: DigestAlgorithmVersion,
    ) -> Self {
        Self {
            schema,
            policy,
            canonicalization,
            digest,
        }
    }
}

#[must_use]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CandidateContext {
    versions: ContractVersions,
    capture_window: CaptureWindow,
    state: SectionState,
    dependencies: BTreeMap<SectionKey, SectionRevisionId>,
    expected_current: Option<SectionRevisionId>,
    stable_identity: bool,
}

impl CandidateContext {
    pub const fn new(
        versions: ContractVersions,
        capture_window: CaptureWindow,
        state: SectionState,
        dependencies: BTreeMap<SectionKey, SectionRevisionId>,
        expected_current: Option<SectionRevisionId>,
        stable_identity: bool,
    ) -> Self {
        Self {
            versions,
            capture_window,
            state,
            dependencies,
            expected_current,
            stable_identity,
        }
    }
    pub const fn capture_window(&self) -> CaptureWindow {
        self.capture_window
    }
    pub const fn state(&self) -> SectionState {
        self.state
    }
    pub const fn versions(&self) -> ContractVersions {
        self.versions
    }
    pub const fn stable_identity(&self) -> bool {
        self.stable_identity
    }
}

#[must_use]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CompletionCurrent {
    dependencies: BTreeMap<SectionKey, SectionRevisionId>,
    current_pointer: Option<SectionRevisionId>,
}

impl CompletionCurrent {
    pub const fn new(
        dependencies: BTreeMap<SectionKey, SectionRevisionId>,
        current_pointer: Option<SectionRevisionId>,
    ) -> Self {
        Self {
            dependencies,
            current_pointer,
        }
    }
}

#[must_use]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CompletionCertificate {
    pub envelope: SectionCompletionEnvelope,
    pub batch_count: usize,
    pub record_count: usize,
    pub raw_bytes: usize,
    pub decoded_bytes: usize,
    pub ordered_batch_manifest_digest: [u8; 32],
    pub canonical_content_digest: [u8; 32],
    pub versions: ContractVersions,
}

#[must_use]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ValidatedSectionRevision {
    source_scope: SourceScopeId,
    section_key: SectionKey,
    section_revision: SectionRevisionId,
    records: Vec<EnvelopeRecord>,
    coverage: CompletionCoverage,
    context: CandidateContext,
    manifest_digest: [u8; 32],
    content_digest: [u8; 32],
}

impl ValidatedSectionRevision {
    pub const fn source_scope(&self) -> &SourceScopeId {
        &self.source_scope
    }
    pub const fn section_key(&self) -> &SectionKey {
        &self.section_key
    }
    pub const fn section_revision(&self) -> SectionRevisionId {
        self.section_revision
    }
    pub fn records(&self) -> &[EnvelopeRecord] {
        &self.records
    }
    pub const fn coverage(&self) -> CompletionCoverage {
        self.coverage
    }
    pub const fn context(&self) -> &CandidateContext {
        &self.context
    }
    pub const fn manifest_digest(&self) -> &[u8; 32] {
        &self.manifest_digest
    }
    pub const fn content_digest(&self) -> &[u8; 32] {
        &self.content_digest
    }
    pub const fn is_published(&self) -> bool {
        false
    }
}

#[must_use]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CompletionOutcome {
    Validated(ValidatedSectionRevision),
    Rejected(RejectionReason),
}

impl GenerationStager {
    pub fn start_section_with_context(
        &mut self,
        start: observation_domain::SectionStartEnvelope,
        context: CandidateContext,
        now: u64,
    ) -> ReceiverDisposition {
        let key = start.section_key.clone();
        if self.cooldowns.get(&key).is_some_and(|until| now < *until) {
            return ReceiverDisposition::TimedOutOrSuperseded;
        }
        self.cooldowns.remove(&key);
        if self
            .candidates
            .get(&key)
            .and_then(|value| value.context.as_ref())
            .is_some_and(|value| value != &context)
        {
            self.drop_candidate(&key);
            return ReceiverDisposition::PermanentlyRejected;
        }
        let disposition = self.start_section(start, now);
        if disposition == ReceiverDisposition::Received {
            if let Some(candidate) = self.candidates.get_mut(&key) {
                candidate.context = Some(context);
            }
        }
        disposition
    }

    pub fn completion_certificate(
        &self,
        envelope: SectionCompletionEnvelope,
    ) -> Option<CompletionCertificate> {
        let candidate = self.candidates.get(&envelope.section_key)?;
        let context = candidate.context.as_ref()?;
        let (manifest, records) = candidate_material(candidate);
        Some(CompletionCertificate {
            envelope,
            batch_count: candidate.batches.len(),
            record_count: records.len(),
            raw_bytes: candidate.usage.raw_bytes,
            decoded_bytes: candidate.usage.decoded_bytes,
            ordered_batch_manifest_digest: manifest,
            canonical_content_digest: content_digest(&records),
            versions: context.versions,
        })
    }

    pub fn complete_section(
        &mut self,
        certificate: CompletionCertificate,
        current: &CompletionCurrent,
        now: u64,
    ) -> CompletionOutcome {
        let key = certificate.envelope.section_key.clone();
        let Some(candidate) = self.candidates.get(&key) else {
            return CompletionOutcome::Rejected(RejectionReason::CompletionMismatch);
        };
        let Some(context) = candidate.context.clone() else {
            self.drop_candidate(&key);
            return CompletionOutcome::Rejected(RejectionReason::CompletionMismatch);
        };
        let dependency_changed = context.dependencies != current.dependencies
            || context.expected_current != current.current_pointer;
        let (manifest, records) = candidate_material(candidate);
        let exact = candidate.source_scope == certificate.envelope.source_scope
            && candidate.revision == certificate.envelope.section_revision
            && candidate.expected_records == records.len()
            && certificate.envelope.record_count == records.len()
            && certificate.batch_count == candidate.batches.len()
            && certificate.record_count == records.len()
            && certificate.raw_bytes == candidate.usage.raw_bytes
            && certificate.decoded_bytes == candidate.usage.decoded_bytes
            && certificate.versions == context.versions
            && certificate.ordered_batch_manifest_digest == manifest
            && certificate.canonical_content_digest == content_digest(&records);
        if dependency_changed || !exact {
            self.drop_candidate(&key);
            let reason = if dependency_changed {
                RejectionReason::DependencyChanged
            } else {
                RejectionReason::CompletionMismatch
            };
            if dependency_changed {
                self.cooldowns.insert(
                    key,
                    now.saturating_add(self.limits.candidate.inactivity_millis.get()),
                );
            }
            self.accepted = std::mem::take(&mut self.accepted).record_rejection(reason);
            return CompletionOutcome::Rejected(reason);
        }
        let Some(candidate) = self.drop_candidate(&key) else {
            return CompletionOutcome::Rejected(RejectionReason::CompletionMismatch);
        };
        CompletionOutcome::Validated(ValidatedSectionRevision {
            source_scope: candidate.source_scope,
            section_key: key,
            section_revision: candidate.revision,
            records,
            coverage: certificate.envelope.coverage,
            context,
            manifest_digest: manifest,
            content_digest: certificate.canonical_content_digest,
        })
    }
}

fn candidate_material(candidate: &crate::generation::Candidate) -> ([u8; 32], Vec<EnvelopeRecord>) {
    let mut batches: Vec<_> = candidate.batches.values().collect();
    batches.sort_by_key(|batch| batch.ordinal);
    let mut manifest = Sha256::new();
    let mut records = Vec::new();
    for batch in batches {
        framed(&mut manifest, &batch.ordinal.to_be_bytes());
        framed(&mut manifest, batch.envelope.batch_id.as_str().as_bytes());
        framed(&mut manifest, &batch.digest);
        records.extend(batch.envelope.records.clone());
    }
    records.sort_by(|left, right| left.record_id.cmp(&right.record_id));
    (manifest.finalize().into(), records)
}

fn content_digest(records: &[EnvelopeRecord]) -> [u8; 32] {
    let mut digest = Sha256::new();
    for record in records {
        framed(&mut digest, record.record_id.as_str().as_bytes());
        framed(&mut digest, record.entity_id.as_str().as_bytes());
        framed(&mut digest, &record.observation_version.get().to_be_bytes());
        framed(&mut digest, record.content.as_bytes());
    }
    digest.finalize().into()
}

fn framed(digest: &mut Sha256, value: &[u8]) {
    digest.update((value.len() as u64).to_be_bytes());
    digest.update(value);
}
