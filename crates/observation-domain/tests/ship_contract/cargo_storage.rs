use observation_domain::{CargoObservation, CargoStorage, FieldOutcome, ShipDetailError};

#[test]
fn cargo_accepts_canonical_tags_for_one_pool_without_changing_its_volume() {
    let text = super::cargo_content().replace("storage=solid|", "storage=container liquid solid|");
    let cargo = CargoObservation::from_content(&text, 2).expect("canonical storage pool");
    assert_eq!(cargo.canonical_content(), text);
    assert_eq!(
        cargo.storage,
        FieldOutcome::Value(vec![CargoStorage {
            transport: "container liquid solid".to_owned(),
            capacity_cubic_metres: 1000,
            occupied_cubic_metres: 170,
        }])
    );
    let max_length = format!("a {}", "b".repeat(126));
    let text = text.replace("container liquid solid", &max_length);
    assert_eq!(
        CargoObservation::from_content(&text, 2)
            .expect("128-byte storage key")
            .canonical_content(),
        text
    );
}

#[test]
fn cargo_rejects_noncanonical_or_oversized_tags_and_unordered_pools() {
    let text = super::cargo_content();
    let too_long = format!("a {}", "b".repeat(127));
    for transport in [
        "",
        " solid",
        "solid ",
        "container  solid",
        "container\tsolid",
        "solid container",
        "solid solid",
        "container,solid",
        "container\0solid",
        "container\nsolid",
        &too_long,
    ] {
        let bytes = text.replace("storage=solid|", &format!("storage={transport}|"));
        assert_eq!(
            CargoObservation::from_content(&bytes, 2),
            Err(ShipDetailError::InvalidField),
            "must reject {transport:?}"
        );
    }
    let bytes = text.replace("storage=solid|", "storage=container|solid|");
    assert_eq!(
        CargoObservation::from_content(&bytes, 2),
        Err(ShipDetailError::InvalidNumber)
    );
    assert_eq!(
        CargoObservation::from_content(&format!("{text}\nstorage=container solid|250|0"), 2),
        Err(ShipDetailError::InvalidField)
    );
}
