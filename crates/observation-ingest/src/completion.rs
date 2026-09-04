use observation_domain::{CanonicalObservationKey, ObservationVersion, SectionCompletionEnvelope};

use crate::completion_digest::{candidate_material, content_digest};
use crate::completion_types::{
    CandidateContext, CompletionCertificate, CompletionCurrent, CompletionOutcome,
    ValidatedSectionRevision,
};
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
        if disposition == ReceiverDisposition::Received
            && let Some(candidate) = self.candidates.get_mut(&key)
        {
            candidate.context = Some(context);
        }
        disposition
    }

    #[must_use]
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
        certificate: &CompletionCertificate,
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
        let dependencies_match = context.dependencies == current.dependencies;
        let pointer_matches = context.expected_current == current.current_pointer;
        let dependency_changed = !dependencies_match || !pointer_matches;
        let (manifest, records) = candidate_material(candidate);
        if dependency_changed || !completion_is_exact(candidate, certificate, &context, &records) {
            return self.reject_completion(key, dependency_changed, now);
        }
        let Some(candidate) = self.drop_candidate(&key) else {
            return CompletionOutcome::Rejected(RejectionReason::CompletionMismatch);
        };
        CompletionOutcome::Validated(Box::new(ValidatedSectionRevision {
            source_scope: candidate.start.source_scope,
            section_key: key,
            section_revision: candidate.start.section_revision,
            records,
            coverage: certificate.envelope.coverage,
            context,
            manifest_digest: manifest,
            content_digest: certificate.canonical_content_digest,
        }))
    }

    fn reject_completion(
        &mut self,
        key: observation_domain::SectionKey,
        dependency_changed: bool,
        now: u64,
    ) -> CompletionOutcome {
        self.drop_candidate(&key);
        let reason = if dependency_changed {
            self.cooldowns.insert(
                key,
                now.saturating_add(self.limits.candidate.inactivity_millis.get()),
            );
            RejectionReason::DependencyChanged
        } else {
            RejectionReason::CompletionMismatch
        };
        self.accepted = std::mem::take(&mut self.accepted).record_rejection(reason);
        CompletionOutcome::Rejected(reason)
    }
}

fn completion_is_exact(
    candidate: &crate::completion_types::Candidate,
    certificate: &CompletionCertificate,
    context: &CandidateContext,
    records: &[observation_domain::EnvelopeRecord],
) -> bool {
    candidate.start.source_scope == certificate.envelope.source_scope
        && candidate.start.producer_incarnation == certificate.envelope.producer_incarnation
        && candidate.start.transport_epoch == certificate.envelope.transport_epoch
        && candidate.start.section_revision == certificate.envelope.section_revision
        && candidate.start.expected_records == records.len()
        && certificate.envelope.record_count == records.len()
        && certificate.batch_count == candidate.batches.len()
        && certificate.record_count == records.len()
        && certificate.raw_bytes == candidate.usage.raw_bytes
        && certificate.decoded_bytes == candidate.usage.decoded_bytes
        && certificate.versions == context.versions
        && certificate.ordered_batch_manifest_digest == candidate_material(candidate).0
        && certificate.canonical_content_digest == content_digest(records)
}
