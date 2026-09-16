mod carrier_b_support;
mod ship_support;

use observation_application::LifecycleResult;
use observation_domain::{BatchId, SectionKey, TransportEpoch};
use observation_ingest::ReceiverDisposition;
use observation_persistence::{ObservationRepository, SqliteObservationRepository};
use x4_bridge::ProductionObservationSession;

fn session(path: &std::path::Path) -> ProductionObservationSession {
    ProductionObservationSession::open(
        path,
        carrier_b_support::generation_limits(),
        carrier_b_support::publication_limits(),
        carrier_b_support::lifecycle_limits(),
        4,
    )
    .unwrap()
}

fn submit(
    session: &mut ProductionObservationSession,
    bytes: &[u8],
    ordinal: usize,
) -> Result<LifecycleResult, x4_bridge::ProductionError> {
    session.submit_received(
        TransportEpoch::new(7).unwrap(),
        BatchId::new(format!("outer:{ordinal}")).unwrap(),
        bytes.to_vec(),
        1,
        ordinal as u64,
    )
}

#[test]
fn malformed_ship_owner_cannot_replace_durable_core() {
    let database = carrier_b_support::database("ship-owner");
    let mut receiver = session(database.path());
    let valid = ship_support::messages(1, "argon");
    for (ordinal, bytes) in valid.iter().enumerate() {
        assert_eq!(
            submit(&mut receiver, bytes, ordinal + 1),
            Ok(LifecycleResult::Disposition(if ordinal == 3 {
                ReceiverDisposition::Committed
            } else {
                ReceiverDisposition::Received
            }))
        );
    }
    let malformed = ship_support::messages(2, "argon\nowner=teladi");
    assert_eq!(
        submit(&mut receiver, &malformed[0], 5),
        Ok(LifecycleResult::Disposition(ReceiverDisposition::Received))
    );
    let rejected = submit(&mut receiver, &malformed[1], 6);
    assert!(
        rejected.is_err()
            || rejected
                == Ok(LifecycleResult::Disposition(
                    ReceiverDisposition::PermanentlyRejected
                )),
        "receiver must reject malformed ship owner before publication: {rejected:?}"
    );
    drop(receiver);
    let repository =
        SqliteObservationRepository::open(database.path(), carrier_b_support::publication_limits())
            .unwrap();
    let current = repository
        .current(&SectionKey::new("ship_core").unwrap())
        .unwrap()
        .unwrap();
    assert_eq!(current.receipt().revision.get(), 1);
    assert_eq!(current.revision().records.len(), 2);
}
