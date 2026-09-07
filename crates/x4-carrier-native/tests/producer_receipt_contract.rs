#![expect(
    clippy::unwrap_used,
    reason = "contract fixtures fail immediately when assumptions break"
)]

use observation_domain::CompleteMessage;
use observation_ingest::{ControlBody, DispositionBody, decode_complete_message};
use x4_carrier_native::{ProducerOutcome, SectionEvidence};

#[path = "producer_contract/support.rs"]
mod support;

#[test]
fn actual_encoded_batch_identity_accepts_exact_receipt() {
    let (mut producer, source) = support::ready(0);
    producer
        .begin_section(SectionEvidence::point_measurement(
            "x4:carrier_b_acceptance",
        ))
        .unwrap();
    producer.push_record(&support::sample("123.5")).unwrap();
    producer.finish_section(support::finish(1)).unwrap();
    producer.progress(1, 1).unwrap();
    support::take(&mut producer, &source, "received", 1);
    let bytes = producer.pending_bytes().unwrap().to_vec();
    let CompleteMessage::ImmutableBatch(batch) = decode_complete_message(&bytes, 2_048).unwrap()
    else {
        panic!("batch expected")
    };
    producer.mark_local_handoff(1).unwrap();
    let receipt = support::control(
        &source,
        ControlBody::Disposition(DispositionBody {
            message_id: batch.batch_id.as_str().to_owned(),
            section_key: batch.section_key.as_str().to_owned(),
            section_revision: batch.section_revision.get(),
            message_digest: support::digest(&bytes),
            disposition: "received".to_owned(),
        }),
    );
    assert_eq!(
        producer.apply_control(&receipt, 1),
        Ok(ProducerOutcome::Received)
    );
}
