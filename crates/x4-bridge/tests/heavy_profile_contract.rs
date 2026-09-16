#![expect(clippy::expect_used, reason = "contract fixtures fail immediately")]
use std::process::Command;

#[test]
fn shared_nonzero_experimental_profile_is_accepted_by_production_validator() {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../config/heavy-ship-experiment.json");
    let status = Command::new(env!("CARGO_BIN_EXE_validate_limits"))
        .arg("--limits-file")
        .arg(path)
        .status()
        .expect("validator runs");
    assert!(status.success(), "assertion failed: shared_nonzero_experimental_profile_is_accepted_by_production_validator: validated heavy profile rejected");
}
