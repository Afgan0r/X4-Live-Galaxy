#![expect(
    clippy::expect_used,
    reason = "contract-test setup and mismatches must fail immediately"
)]

mod support;

use observation_application::{LifecycleContext, LifecycleResult, ObservationLifecycle};
use observation_ingest::{DecisionRevisionIndex, ReceiverDisposition};
use support::flow::{input, limits, submit_start_and_batch};
use support::{completion_bytes, current, repository, stager};

#[test]
fn wire_completion_certificate_fields_are_sender_bound() {
    let replacements = [
        ("\"batch_count\":1", "\"batch_count\":2"),
        ("\"record_count\":1", "\"record_count\":2"),
        ("\"raw_bytes\":", "\"raw_bytes\":"),
        ("\"decoded_bytes\":", "\"decoded_bytes\":"),
        ("\"schema_version\":1", "\"schema_version\":9"),
        ("\"policy_version\":2", "\"policy_version\":9"),
        (
            "\"canonicalization_version\":3",
            "\"canonicalization_version\":9",
        ),
        ("\"digest_version\":1", "\"digest_version\":9"),
    ];
    for (index, (from, to)) in replacements.into_iter().enumerate() {
        let (_database, repository) = repository(&format!("certificate-{index}"));
        let mut lifecycle = ObservationLifecycle::new(
            stager(),
            DecisionRevisionIndex::new(4).expect("blocker limit is non-zero"),
            repository,
            limits(),
        );
        let records = [("record:1", "ship:1")];
        submit_start_and_batch(&mut lifecycle, "ships", &records);
        let valid = String::from_utf8(completion_bytes("ships", &records, "complete"))
            .expect("fixture JSON is UTF-8");
        let changed = if matches!(index, 2 | 3) {
            replace_number(&valid, from, 9)
        } else {
            valid.replacen(from, to, 1)
        };
        assert_eq!(
            lifecycle.submit(input(
                "outer:ships:complete",
                changed.into_bytes(),
                LifecycleContext::Completion(current()),
                3,
            )),
            Ok(LifecycleResult::Disposition(
                ReceiverDisposition::PermanentlyRejected
            ))
        );
    }
}

fn replace_number(value: &str, field: &str, replacement: usize) -> String {
    let start = value.find(field).expect("field exists") + field.len();
    let end = value[start..]
        .find(',')
        .map_or(value.len(), |relative| start + relative);
    format!("{}{replacement}{}", &value[..start], &value[end..])
}
