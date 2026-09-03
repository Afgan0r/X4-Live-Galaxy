use observation_domain::{DecisionSnapshotId, SectionKey};
use observation_ingest::DecisionRevisionSet;

use crate::{
    CurrentRevision, DecisionPinReceipt, DecisionRevisionPin, PublishOutcome, PublishRequest,
    RepositoryError, UnpinOutcome,
};

pub trait ObservationRepository {
    fn publish(&mut self, request: PublishRequest) -> PublishOutcome;
    fn current(&self, key: &SectionKey) -> Result<Option<CurrentRevision>, RepositoryError>;
    fn pin_decision(
        &mut self,
        set: &DecisionRevisionSet,
    ) -> Result<DecisionPinReceipt, RepositoryError>;
    fn load_decision_pin(
        &self,
        decision: &DecisionSnapshotId,
    ) -> Result<DecisionRevisionPin, RepositoryError>;
    fn unpin_decision(
        &mut self,
        receipt: &DecisionPinReceipt,
    ) -> Result<UnpinOutcome, RepositoryError>;
}

#[cfg(test)]
mod tests {
    use crate::test_support::{decision_set, key, validated};
    use crate::{
        FakeObservationRepository, ObservationRepository, PublicationLimits, PublishOutcome,
        PublishRequest, UnpinOutcome,
    };

    fn repository() -> FakeObservationRepository {
        FakeObservationRepository::new(PublicationLimits::new(4, 256).expect("limits are non-zero"))
    }

    #[test]
    fn publishes_genesis_and_replays_exact_receipt() {
        let mut repository = repository();
        let request = PublishRequest::from_revision(validated(1, None));
        assert!(matches!(
            repository.publish(request.clone()),
            PublishOutcome::CommittedNew(_)
        ));
        assert!(matches!(
            repository.publish(request),
            PublishOutcome::CommittedReplay(_)
        ));
        assert!(
            repository
                .current(&key("ships"))
                .is_ok_and(|value| value.is_some())
        );
    }

    #[test]
    fn pins_loads_and_unpins_without_removing_current_revision() {
        let mut repository = repository();
        let revision = validated(1, None);
        let set = decision_set(revision.clone());
        assert!(matches!(
            repository.publish(PublishRequest::from_revision(revision)),
            PublishOutcome::CommittedNew(_)
        ));
        let receipt = repository.pin_decision(&set).expect("stored revision pins");
        let pin = repository
            .load_decision_pin(&receipt.decision)
            .expect("pin loads");
        assert_eq!(pin.revisions, *set.revisions());
        assert_eq!(
            repository.unpin_decision(&receipt),
            Ok(UnpinOutcome::Unpinned)
        );
        assert!(
            repository
                .current(&key("ships"))
                .is_ok_and(|item| item.is_some())
        );
    }

    #[test]
    fn rejects_conflict_stale_pointer_and_inconsistent_request() {
        let mut repository = repository();
        assert!(matches!(
            repository.publish(PublishRequest::from_revision(validated(1, None))),
            PublishOutcome::CommittedNew(_)
        ));
        let first = crate::test_support::revision(1);
        assert!(matches!(
            repository.publish(PublishRequest::from_revision(validated(1, Some(first)))),
            PublishOutcome::Conflict(_)
        ));
        assert!(matches!(
            repository.publish(PublishRequest::from_revision(validated(2, None))),
            PublishOutcome::StalePointer(_)
        ));
        let mut invalid = PublishRequest::from_revision(validated(2, Some(first)));
        invalid
            .frozen_dependencies
            .insert(key("missing"), crate::test_support::revision(9));
        assert!(matches!(
            repository.publish(invalid),
            PublishOutcome::PermanentRejection(_)
        ));
        let current = repository
            .current(&key("ships"))
            .expect("fake read succeeds")
            .expect("current remains");
        assert_eq!(current.receipt.revision, first);
    }
}
