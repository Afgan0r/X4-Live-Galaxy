use std::collections::BTreeMap;

use observation_persistence::{FakeObservationRepository, ObservationRepository, PublishOutcome};

use crate::support::{key, publish_request, validated};

#[test]
fn hydration_revalidates_durable_authority() {
    let mut repository = FakeObservationRepository::new(super::limits());
    let request = publish_request(validated("ships", 1, None, BTreeMap::new()));
    assert!(matches!(
        repository.publish(request),
        PublishOutcome::CommittedNew(_)
    ));
    let current = repository
        .current(&key("ships"))
        .expect("read succeeds")
        .expect("current exists");
    assert!(current.hydrate().is_ok());
}
