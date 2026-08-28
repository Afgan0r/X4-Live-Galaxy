use observation_ingest::{
    AcceptedProjection, AdmissionOutcome, MAX_BATCH_BYTES, MAX_BATCH_FRAMES, MAX_BATCH_MARKERS,
    MAX_BATCH_SCOPES, admit_batch,
};

fn observation(scope: &str, entity: &str, content: &str) -> String {
    format!(
        r#"{{"type":"observation","scope":"{scope}","entity_id":"{entity}","observed_at_unix_millis":1,"version":1,"quality":"fresh","content":"{content}"}}"#
    )
}

#[test]
fn exact_and_over_frame_limits_are_atomic() {
    let frame = observation("runtime:sectors", "sector:alpha", "ok");
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
    let prefix = observation("runtime:sectors", "sector:alpha", "");
    let content_len = 512 - prefix.len();
    let frame = observation("runtime:sectors", "sector:alpha", &"x".repeat(content_len));
    assert_eq!(frame.len(), 512);
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
        .map(|index| {
            observation(
                &format!("runtime:scope{index}"),
                &format!("sector:{index}"),
                "ok",
            )
        })
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
