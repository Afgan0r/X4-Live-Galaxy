#![expect(
    clippy::expect_used,
    reason = "test fixtures fail immediately when their invariants are invalid"
)]

use observation_domain::{
    CanonicalObservationKey, CaptureWindow, CompleteMarker, DuplicateDecision, EntityId,
    ObservationRecord, ObservationSource, ObservationTime, ObservationVersion, RecordId,
    SectionAvailability, SectionCoverage, SectionFreshness, SectionQuality, SectionState,
    SourceScopeId, classify_duplicate, quality_for_empty_section,
};

fn entity(value: &str) -> EntityId {
    EntityId::new(value).expect("test identity is valid")
}

const fn version(value: u64) -> ObservationVersion {
    ObservationVersion::new(value).expect("test version is positive")
}

fn record(id: &str, record_version: u64, content: &str) -> ObservationRecord {
    ObservationRecord::new(
        entity(id),
        ObservationSource::x4_runtime(),
        ObservationTime::from_unix_millis(1_725_000_000_000),
        version(record_version),
        content,
    )
    .expect("test record is valid")
}

fn scoped_record(scope: &str, id: &str, record_version: u64, content: &str) -> ObservationRecord {
    ObservationRecord::scoped(
        SourceScopeId::new(scope).expect("test scope is valid"),
        RecordId::new(format!("record:{id}")).expect("test record id is valid"),
        entity(id),
        ObservationSource::x4_runtime(),
        ObservationTime::from_unix_millis(1_725_000_000_000),
        version(record_version),
        content,
    )
    .expect("test record is valid")
}

#[test]
fn equal_version_duplicate_is_idempotent_but_divergent_content_conflicts() {
    let accepted = record("sector:alpha", 7, "stable-content");

    assert_eq!(
        classify_duplicate(&accepted, &record("sector:alpha", 7, "stable-content")),
        DuplicateDecision::Idempotent
    );
    assert_eq!(
        classify_duplicate(&accepted, &record("sector:alpha", 7, "changed-content")),
        DuplicateDecision::Conflict
    );
}

#[test]
fn known_empty_requires_a_successful_marker_for_the_same_scope() {
    let scope = entity("scope:sectors");
    let other_scope = entity("scope:assets");
    let marker = CompleteMarker::successful(other_scope, version(3));

    assert_eq!(
        quality_for_empty_section(&scope, None),
        SectionState::new(SectionFreshness::Fresh, SectionCoverage::Unknown)
    );
    assert_eq!(
        quality_for_empty_section(&scope, Some(&marker)),
        SectionState::new(SectionFreshness::Fresh, SectionCoverage::Partial)
    );
    assert_eq!(
        quality_for_empty_section(
            &scope,
            Some(&CompleteMarker::successful(scope.clone(), version(3)))
        ),
        SectionState::new(SectionFreshness::Fresh, SectionCoverage::KnownEmpty)
    );
}

#[test]
fn freshness_is_independent_and_canonical_keys_are_stable() {
    let fresh = SectionState::new(SectionFreshness::Fresh, SectionCoverage::Complete);
    let stale = SectionState::new(SectionFreshness::Stale, SectionCoverage::Complete);
    let key_a = CanonicalObservationKey::new(entity("sector:alpha"), version(2));
    let key_b = CanonicalObservationKey::new(entity("sector:beta"), version(1));

    assert_ne!(fresh, stale);
    assert!(key_a < key_b);
}

#[test]
fn evidence_axes_remain_independent_and_never_promote_coverage() {
    let window = CaptureWindow::new(10, 20).expect("capture window is ordered");
    let partial = SectionState::with_evidence(
        window,
        SectionFreshness::Fresh,
        SectionQuality::Fresh,
        SectionAvailability::Available,
        SectionCoverage::Partial,
    );
    let unsupported = SectionState::with_evidence(
        window,
        SectionFreshness::Stale,
        SectionQuality::Fresh,
        SectionAvailability::Unavailable,
        SectionCoverage::Unsupported,
    );

    assert_eq!(partial.capture_window(), window);
    assert_eq!(partial.coverage(), SectionCoverage::Partial);
    assert_eq!(unsupported.quality(), SectionQuality::Fresh);
    assert_eq!(unsupported.coverage(), SectionCoverage::Unsupported);
    assert_ne!(partial.availability(), unsupported.availability());
}

#[test]
fn scoped_versions_reject_lower_replay_equal_and_admit_only_higher() {
    let accepted = scoped_record("scope:ships", "ship:1", 7, "stable");

    assert_eq!(
        classify_duplicate(
            &accepted,
            &scoped_record("scope:ships", "ship:1", 6, "stable")
        ),
        DuplicateDecision::Conflict
    );
    assert_eq!(
        classify_duplicate(
            &accepted,
            &scoped_record("scope:ships", "ship:1", 7, "stable")
        ),
        DuplicateDecision::Idempotent
    );
    assert_eq!(
        classify_duplicate(
            &accepted,
            &scoped_record("scope:ships", "ship:1", 7, "changed")
        ),
        DuplicateDecision::Conflict
    );
    assert_eq!(
        classify_duplicate(
            &accepted,
            &scoped_record("scope:ships", "ship:1", 8, "advanced")
        ),
        DuplicateDecision::Conflict
    );
    assert_eq!(
        classify_duplicate(
            &accepted,
            &scoped_record("scope:stations", "ship:1", 7, "stable")
        ),
        DuplicateDecision::DifferentIdentity
    );
}

#[test]
fn canonical_record_order_is_independent_of_input_permutation() {
    let first = scoped_record("scope:ships", "ship:2", 1, "beta");
    let second = scoped_record("scope:ships", "ship:1", 2, "alpha");
    let mut left = vec![first.clone(), second.clone()];
    let mut right = vec![second, first];

    left.sort();
    right.sort();

    assert_eq!(left, right);
}
