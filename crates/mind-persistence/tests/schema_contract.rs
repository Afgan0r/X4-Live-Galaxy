use mind_persistence::{GAME_PROTOCOL_IDENTITY, SCHEMA_VERSION};
use serde_json::Value;
use std::path::PathBuf;

#[test]
fn rust_constants_match_the_md_checkpoint_manifest() {
    let manifest_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../extensions/live_galaxy/checkpoint_schema.json");
    let contents_result = std::fs::read_to_string(manifest_path);
    assert!(contents_result.is_ok());
    let Ok(contents) = contents_result else {
        return;
    };
    let manifest_result: Result<Value, _> = serde_json::from_str(&contents);
    assert!(manifest_result.is_ok());
    let Ok(manifest) = manifest_result else {
        return;
    };

    assert_eq!(manifest["schema_version"].as_str(), Some(SCHEMA_VERSION));
    assert_eq!(
        manifest["game_protocol_identity"].as_str(),
        Some(GAME_PROTOCOL_IDENTITY)
    );
    let fields = manifest["required_fields"].as_array();
    assert!(fields.is_some());
    let Some(fields) = fields else { return };
    for required in [
        "schema_version",
        "game_protocol_identity",
        "sequence",
        "integrity_hash",
        "compatibility_status",
        "x4_restart_required",
        "payload",
    ] {
        assert!(fields.iter().any(|value| value.as_str() == Some(required)));
    }
}
