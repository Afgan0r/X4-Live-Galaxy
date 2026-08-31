use observation_ingest::{
    AcceptedProjection, AdmissionOutcome, GenerationLimits, GenerationProgress, GenerationStager,
    MAX_BATCH_BYTES, MAX_BATCH_FRAMES, MAX_BATCH_MARKERS, MAX_BATCH_SCOPES, admit_batch,
};

fn observation(scope: &str, entity: &str) -> String {
    format!(
        r#"{{"type":"observation","scope":"{scope}","entity_id":"{entity}","version":1,"quality":"fresh","runtime_facts":{{"r":"x4_runtime","g":42,"q":"fresh","a":"available","s":[{{"i":"{entity}"}}],"x":[{{"i":"asset:ship:1","p":"{entity}"}}],"c":[{{"i":"capacity:ship:storage","p":"asset:ship:1","v":42}}],"o":[{{"i":"ownership:ship:1","p":"asset:ship:1","n":"faction:argon"}}]}}}}"#
    )
}

fn observation_with_exact_size(scope: &str, entity: &str, size: usize) -> String {
    let frame = observation(scope, entity);
    assert!(frame.len() <= size, "v2 fixture must fit the frame budget");
    let padding = " ".repeat(size - frame.len());
    format!("{}{}", &frame[..frame.len() - 1], padding) + "}"
}

fn streamed_observation(entity: &str, sequence: u64) -> String {
    let asset = format!("asset:station:{entity}");
    format!(
        r#"{{"type":"observation","scope":"runtime:sectors","entity_id":"{entity}","version":2,"quality":"fresh","runtime_facts":{{"r":"x4_runtime","g":42,"q":"fresh","a":"available","s":[{{"i":"{entity}"}}],"x":[{{"i":"{asset}","p":"{entity}"}}],"c":[{{"i":"capacity:station:{entity}","p":"{asset}","v":42}}],"o":[{{"i":"ownership:station:{entity}","p":"{asset}","n":"faction:argon"}}]}},"generation":7,"sequence":{sequence}}}"#
    )
}

#[test]
fn exact_and_over_frame_limits_are_atomic() {
    let frame = observation("runtime:sectors", "sector:alpha");
    let exact = vec![frame.as_str(); MAX_BATCH_FRAMES];
    assert!(matches!(
        admit_batch(AcceptedProjection::empty(), &exact),
        AdmissionOutcome::Accepted(_)
    ));

    let over = vec![frame.as_str(); MAX_BATCH_FRAMES + 1];
    assert!(matches!(
        admit_batch(AcceptedProjection::empty(), &over),
        AdmissionOutcome::Rejected { .. }
    ));
}

#[test]
fn aggregate_byte_limit_is_exact_and_atomic() {
    let frame = observation_with_exact_size("runtime:sectors", "sector:alpha", 2_048);
    assert_eq!(frame.len(), 2_048);
    let exact = vec![frame.as_str(); MAX_BATCH_FRAMES];
    assert_eq!(
        exact.iter().map(|item| item.len()).sum::<usize>(),
        MAX_BATCH_BYTES
    );
    assert!(matches!(
        admit_batch(AcceptedProjection::empty(), &exact),
        AdmissionOutcome::Accepted(_)
    ));

    let over = vec![frame.as_str(); MAX_BATCH_FRAMES + 1];
    assert!(matches!(
        admit_batch(AcceptedProjection::empty(), &over),
        AdmissionOutcome::Rejected { .. }
    ));
}

#[test]
fn marker_and_scope_limits_are_atomic() {
    let marker = r#"{"type":"complete_marker","scope":"runtime:sectors","version":1}"#;
    let exact_markers = vec![marker; MAX_BATCH_MARKERS];
    assert!(matches!(
        admit_batch(AcceptedProjection::empty(), &exact_markers),
        AdmissionOutcome::Accepted(_)
    ));
    let over_markers = vec![marker; MAX_BATCH_MARKERS + 1];
    assert!(matches!(
        admit_batch(AcceptedProjection::empty(), &over_markers),
        AdmissionOutcome::Rejected { .. }
    ));

    let scopes = (0..=MAX_BATCH_SCOPES)
        .map(|index| observation(&format!("runtime:scope{index}"), &format!("sector:{index}")))
        .collect::<Vec<_>>();
    let exact_scopes = scopes[..MAX_BATCH_SCOPES]
        .iter()
        .map(String::as_str)
        .collect::<Vec<_>>();
    assert!(matches!(
        admit_batch(AcceptedProjection::empty(), &exact_scopes),
        AdmissionOutcome::Accepted(_)
    ));
    let over_scopes = scopes.iter().map(String::as_str).collect::<Vec<_>>();
    assert!(matches!(
        admit_batch(AcceptedProjection::empty(), &over_scopes),
        AdmissionOutcome::Rejected { .. }
    ));
}

#[test]
fn streamed_generation_bypasses_legacy_aggregate_counters() {
    let limits = GenerationLimits::new(MAX_BATCH_BYTES * 2, MAX_BATCH_FRAMES + 2);
    let mut stager = GenerationStager::new(AcceptedProjection::empty(), limits);
    for index in 0..=MAX_BATCH_FRAMES {
        let entity = format!("sector:streamed_{index:03}");
        let frame = streamed_observation(&entity, index as u64 + 1);
        assert_eq!(
            stager.stage_frame_at(&frame, 1_725_000_000_200),
            GenerationProgress::Staged
        );
    }
    let marker = format!(
        r#"{{"type":"complete_marker","scope":"runtime:sectors","version":2,"generation":7,"sequence":{}}}"#,
        MAX_BATCH_FRAMES + 2
    );
    assert_eq!(
        stager.stage_frame_at(&marker, 1_725_000_000_200),
        GenerationProgress::Admitted
    );
    assert_eq!(stager.accepted().snapshot().entity_ids().len(), 129);
}
