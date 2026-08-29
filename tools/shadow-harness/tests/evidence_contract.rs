use shadow_harness::{EvidenceRecord, validate_corpus};
use std::{
    fs,
    path::{Path, PathBuf},
};

fn corpus_copy(name: &str) -> Option<PathBuf> {
    let root = std::env::temp_dir().join(format!("shadow-harness-{name}"));
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(root.join("fixtures")).ok()?;
    let source = Path::new("../../shadow-deliberation-evals/v1");
    fs::copy(source.join("manifest.json"), root.join("manifest.json")).ok()?;
    fs::copy(source.join("schema.json"), root.join("schema.json")).ok()?;
    for item in fs::read_dir(source.join("fixtures")).ok()? {
        let item = item.ok()?;
        fs::copy(item.path(), root.join("fixtures").join(item.file_name())).ok()?;
    }
    Some(root)
}

fn replace(root: &Path, from: &str, to: &str) -> bool {
    let path = root.join("manifest.json");
    let Ok(value) = fs::read_to_string(&path) else {
        return false;
    };
    fs::write(path, value.replacen(from, to, 1)).is_ok()
}

#[test]
fn rejects_missing_tampered_and_outside_artifacts() {
    let Some(root) = corpus_copy("artifacts") else {
        return;
    };
    assert!(validate_corpus(&root));
    assert!(fs::remove_file(root.join("fixtures/SD-001.json")).is_ok());
    assert!(!validate_corpus(&root));
    let Some(root) = corpus_copy("tamper") else {
        return;
    };
    assert!(fs::write(root.join("fixtures/SD-002.json"), b"tampered").is_ok());
    assert!(!validate_corpus(&root));
    let Some(root) = corpus_copy("outside") else {
        return;
    };
    assert!(replace(&root, "fixtures/SD-001.json", "../outside.json"));
    assert!(!validate_corpus(&root));
    let Some(root) = corpus_copy("schema") else {
        return;
    };
    assert!(fs::remove_file(root.join("schema.json")).is_ok());
    assert!(!validate_corpus(&root));
}

#[test]
fn rejects_wrong_closed_case_track_and_disposition() {
    let Some(root) = corpus_copy("track") else {
        return;
    };
    assert!(replace(
        &root,
        "\"evidence_class\":\"benchmark\"",
        "\"evidence_class\":\"ci\""
    ));
    assert!(!validate_corpus(&root));
    let Some(root) = corpus_copy("disposition") else {
        return;
    };
    assert!(replace(&root, "manual-benchmark", "unexpected"));
    assert!(!validate_corpus(&root));
    let duplicate = include_str!("../../../shadow-deliberation-evals/v1/manifest.json").replacen(
        "\"id\":\"SD-013\"",
        "\"id\":\"SD-012\"",
        1,
    );
    assert!(!EvidenceRecord::validates_manifest(&duplicate));
}
