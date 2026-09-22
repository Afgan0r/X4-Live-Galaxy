use std::path::Path;

use observation_domain::{FactionOrigin, FactionOriginEvidence};

pub fn read(path: &Path) -> Option<Vec<FactionOriginEvidence>> {
    let contents = std::fs::read_to_string(path).ok()?;
    if contents.len() > 64 * 1_024 {
        return None;
    }
    let entries = contents.split_once("\"entries\"")?.1;
    let body = entries.split_once('[')?.1.rsplit_once(']')?.0;
    split_objects(body).map(parse_entry).collect()
}

fn split_objects(body: &str) -> impl Iterator<Item = &str> {
    body.split("},").filter_map(|raw| {
        let trimmed = raw.trim().trim_start_matches(',').trim();
        (!trimmed.is_empty()).then(|| trimmed.trim_start_matches('{').trim_end_matches('}'))
    })
}

fn parse_entry(raw: &str) -> Option<FactionOriginEvidence> {
    let id = string_field(raw, "id")?;
    let origin = match string_field(raw, "origin")? {
        "vanilla" => FactionOrigin::Vanilla,
        "dlc" => FactionOrigin::Dlc,
        "player" => FactionOrigin::Player,
        "service" => FactionOrigin::Service,
        "modded" => FactionOrigin::Modded,
        _ => return None,
    };
    let independent = match independence_field(raw, "independent")? {
        Independence::Known(value) => Some(value),
        Independence::Unknown => None,
    };
    let mind = bool_field(raw, "mind_candidate")?;
    let source = string_field(raw, "evidence")?;
    FactionOriginEvidence::new(id, origin, independent, mind, source).ok()
}

fn value<'a>(raw: &'a str, key: &str) -> Option<&'a str> {
    let marker = format!("\"{key}\"");
    raw.split_once(&marker)?
        .1
        .split_once(':')?
        .1
        .split_once(',')
        .map_or_else(
            || raw.split_once(&marker)?.1.split_once(':').map(|v| v.1),
            |v| Some(v.0),
        )
        .map(str::trim)
}

fn string_field<'a>(raw: &'a str, key: &str) -> Option<&'a str> {
    value(raw, key)?
        .strip_prefix('"')?
        .split_once('"')
        .map(|v| v.0)
}

fn bool_field(raw: &str, key: &str) -> Option<bool> {
    match value(raw, key)? {
        value if value.starts_with("true") => Some(true),
        value if value.starts_with("false") => Some(false),
        _ => None,
    }
}

enum Independence {
    Known(bool),
    Unknown,
}

fn independence_field(raw: &str, key: &str) -> Option<Independence> {
    match value(raw, key)? {
        value if value.starts_with("true") => Some(Independence::Known(true)),
        value if value.starts_with("false") => Some(Independence::Known(false)),
        value if value.starts_with("null") => Some(Independence::Unknown),
        _ => None,
    }
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
            fixture(|value| value.replacen("\"sources\": {", "\"unknown\": true, \"sources\": {", 1)),
            fixture(|value| value.replacen("\"classification\": \"inferred\"", "\"classification\": \"guessed\"", 1)),
            fixture(|value| value.replacen("\"schema_version\": 1", "\"schema_version\": 1, \"schema_version\": 1", 1)),
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
