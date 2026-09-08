mod carrier_b_support;

use observation_application::{LifecycleContext, LifecycleError};
use x4_bridge::{ProductionError, ProductionObservationSession};

use carrier_b_support::{
    context, database, generation_limits, input, lifecycle_limits, publication_limits, start_bytes,
};

#[test]
fn sender_evidence_cannot_be_replaced_by_forged_receiver_context() {
    let database = database("receiver-context-red");
    let mut session = ProductionObservationSession::open(
        database.path(),
        generation_limits(),
        publication_limits(),
        lifecycle_limits(),
        4,
    )
    .expect("production session opens");
    let bytes = String::from_utf8(start_bytes("runtime-clock"))
        .expect("fixture is UTF-8")
        .replacen(
            "\"coverage\":\"complete\"",
            "\"coverage\":\"point_measurement\"",
            1,
        )
        .into_bytes();

    assert_eq!(
        session.submit(input(
            "outer:forged",
            bytes,
            LifecycleContext::Start(context(observation_domain::SectionCoverage::KnownEmpty,)),
            1,
        )),
        Err(ProductionError::Lifecycle(LifecycleError::ContextMismatch))
    );
}
