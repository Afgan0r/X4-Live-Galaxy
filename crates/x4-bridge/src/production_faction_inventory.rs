use std::path::Path;

use observation_domain::FactionOriginEvidence;

#[path = "production_faction_inventory_schema.rs"]
mod schema;

pub fn read(path: &Path) -> Option<Vec<FactionOriginEvidence>> {
    let contents = std::fs::read(path).ok()?;
    if contents.len() > schema::MAX_DOCUMENT_BYTES {
        return None;
    }
    schema::decode(&contents)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fixture(replacement: impl FnOnce(String) -> String) -> std::path::PathBuf {
        static NEXT: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
        let registered = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../config/faction-source-inventory.json");
        let contents = std::fs::read_to_string(registered).expect("registered inventory");
        let path = std::env::temp_dir().join(format!(
            "live-galaxy-faction-inventory-{}-{}.json",
            std::process::id(),
            NEXT.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
        ));
        std::fs::write(&path, replacement(contents)).expect("inventory fixture");
        path
    }

    #[test]
    fn reads_registered_inventory() {
        let path = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../config/faction-source-inventory.json");
        let inventory = read(&path).expect("registered inventory");
        assert!(inventory.iter().any(|entry| entry.id() == "argon"));
        assert!(inventory.iter().any(|entry| entry.id() == "visitor"));
    }

    #[test]
    fn rejects_unsupported_or_incomplete_provenance_schema() {
        for invalid in [
            fixture(|value| value.replacen("\"schema_version\": 1", "\"schema_version\": 2", 1)),
            fixture(|value| value.replacen("9.00-steam-23660954", "9.10-unsupported", 1)),
            fixture(|value| {
                value.replacen("\"sources\": {", "\"unknown\": true, \"sources\": {", 1)
            }),
            fixture(|value| {
                value.replacen(
                    "\"classification\": \"inferred\"",
                    "\"classification\": \"guessed\"",
                    1,
                )
            }),
            fixture(|value| {
                value.replacen(
                    "\"schema_version\": 1",
                    "\"schema_version\": 1, \"schema_version\": 1",
                    1,
                )
            }),
            fixture(|value| {
                value.replacen("\"sources\": {", "\"sources\": { \"base\": \"forged\",", 1)
            }),
        ] {
            assert!(read(&invalid).is_none(), "accepted {}", invalid.display());
            let _ = std::fs::remove_file(invalid);
        }
    }

    #[test]
    fn rejects_malformed_escaped_and_oversize_documents() {
        for invalid in [
            fixture(|value| value.replacen("\"id\": \"alliance\"", "\"id\": \"alli\\\"ance\"", 1)),
            fixture(|value| value.replacen("\"entries\": [", "\"entries\": [ malformed", 1)),
            fixture(|mut value| {
                value.push_str(&" ".repeat(64 * 1_024));
                value
            }),
        ] {
            assert!(read(&invalid).is_none(), "accepted {}", invalid.display());
            let _ = std::fs::remove_file(invalid);
        }
    }
}
