use std::collections::BTreeMap;

use observation_application::LifecycleContext;
use observation_domain::{
    CanonicalizationVersion, CompleteMessage, CompletionCoverage, DigestAlgorithmVersion,
    ImmutableBatchEnvelope, ObservationPolicyVersion, ObservationSchemaVersion,
    SectionCompletionEnvelope, SectionCoverage, SectionRevisionId,
};
use observation_ingest::{CompletionCurrent, decode_complete_message};

use super::{candidate_context, digest_hex, epoch, key, revision};

pub fn start(
    section_revision: u64,
    expected: Option<SectionRevisionId>,
) -> (Vec<u8>, LifecycleContext) {
    let mut context = candidate_context(SectionCoverage::Complete);
    context = observation_ingest::CandidateContext::new(
        context.versions(),
        context.capture_window(),
        context.state(),
        BTreeMap::new(),
        expected,
        true,
    );
    (format!("{{\"type\":\"section_start\",\"contract_version\":1,\"source_scope\":\"scope:x4\",\"producer_incarnation\":\"producer:1\",\"transport_epoch\":1,\"section_key\":\"ships\",\"section_revision\":{section_revision},\"expected_records\":1}}").into_bytes(), LifecycleContext::Start(context))
}

#[must_use]
pub fn batch(section_revision: u64, version: u64, content: &str) -> Vec<u8> {
    format!("{{\"type\":\"immutable_batch\",\"contract_version\":1,\"source_scope\":\"scope:x4\",\"producer_incarnation\":\"producer:1\",\"transport_epoch\":1,\"section_key\":\"ships\",\"section_revision\":{section_revision},\"batch_id\":\"inner:{section_revision}\",\"section_ordinal\":1,\"records\":[{{\"record_id\":\"record:{section_revision}\",\"entity_id\":\"ship:alpha\",\"observation_version\":{version},\"content\":\"{content}\"}}],\"optional_detail\":null}}").into_bytes()
}

#[must_use]
pub fn completion(section_revision: u64, version: u64, content: &str) -> Vec<u8> {
    let batch = decoded_batch(section_revision, version, content);
    let versions = candidate_context(SectionCoverage::Complete).versions();
    let envelope = SectionCompletionEnvelope {
        source_scope: batch.source_scope.clone(),
        producer_incarnation: batch.producer_incarnation.clone(),
        transport_epoch: epoch(),
        section_key: key("ships"),
        section_revision: revision(section_revision),
        batch_count: 0,
        record_count: 0,
        raw_bytes: 0,
        decoded_bytes: 0,
        ordered_batch_manifest_digest: [0; 32],
        canonical_content_digest: [0; 32],
        schema_version: ObservationSchemaVersion::new(1).expect("version is non-zero"),
        policy_version: ObservationPolicyVersion::new(2).expect("version is non-zero"),
        canonicalization_version: CanonicalizationVersion::new(3).expect("version is non-zero"),
        digest_version: DigestAlgorithmVersion::new(1).expect("version is non-zero"),
        coverage: CompletionCoverage::Complete,
    };
    let bound = observation_ingest::bind_completion_certificate(envelope, &[batch], versions)
        .expect("producer certificate binds");
    format!("{{\"type\":\"section_completion\",\"contract_version\":1,\"source_scope\":\"scope:x4\",\"producer_incarnation\":\"producer:1\",\"transport_epoch\":1,\"section_key\":\"ships\",\"section_revision\":{section_revision},\"batch_count\":{},\"record_count\":{},\"raw_bytes\":{},\"decoded_bytes\":{},\"ordered_batch_manifest_digest\":\"{}\",\"canonical_content_digest\":\"{}\",\"schema_version\":1,\"policy_version\":2,\"canonicalization_version\":3,\"digest_version\":1,\"coverage\":\"complete\"}}", bound.batch_count, bound.record_count, bound.raw_bytes, bound.decoded_bytes, digest_hex(bound.ordered_batch_manifest_digest), digest_hex(bound.canonical_content_digest)).into_bytes()
}

pub const fn current(revision: Option<SectionRevisionId>) -> LifecycleContext {
    LifecycleContext::Completion(CompletionCurrent::new(BTreeMap::new(), revision))
}

fn decoded_batch(section_revision: u64, version: u64, content: &str) -> ImmutableBatchEnvelope {
    match decode_complete_message(&batch(section_revision, version, content), 4_096)
        .expect("producer batch decodes")
    {
        CompleteMessage::ImmutableBatch(batch) => batch,
        _ => unreachable!("fixture bytes are an immutable batch"),
    }
}
