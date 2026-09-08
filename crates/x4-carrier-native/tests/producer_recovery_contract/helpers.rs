use x4_carrier_native::{Producer, ProducerOutcome, ProducerSource};

use super::support;

pub fn retry_then_receive(
    producer: &mut Producer,
    source: &ProducerSource,
    ordinal: usize,
    now: u64,
) {
    let bytes = producer.pending_bytes().unwrap().to_vec();
    producer.mark_local_handoff(now).unwrap();
    assert_eq!(
        producer.apply_control(
            &support::disposition(source, &bytes, "capacity_unavailable", ordinal),
            now + 1,
        ),
        Ok(ProducerOutcome::CapacityUnavailable)
    );
    assert_eq!(producer.pending_bytes(), Some(bytes.as_slice()));
    producer.mark_local_handoff(now + 1).unwrap();
    producer
        .apply_control(
            &support::disposition(
                source,
                &bytes,
                if ordinal == 3 {
                    "committed"
                } else {
                    "received"
                },
                ordinal,
            ),
            now + 2,
        )
        .unwrap();
}
