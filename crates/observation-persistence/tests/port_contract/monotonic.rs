use super::*;

pub fn contract(repository: &mut dyn ObservationRepository) {
    assert!(matches!(
        repository.publish(publish_request(validated(
            "stations",
            5,
            None,
            BTreeMap::new()
        ))),
        PublishOutcome::CommittedNew(_)
    ));
    assert!(matches!(
        repository.publish(publish_request(validated(
            "stations",
            3,
            Some(revision(5)),
            BTreeMap::new()
        ))),
        PublishOutcome::StalePointer(_)
    ));
    assert!(matches!(
        repository.publish(publish_request(validated(
            "stations",
            6,
            Some(revision(5)),
            BTreeMap::new()
        ))),
        PublishOutcome::CommittedNew(_)
    ));
}
