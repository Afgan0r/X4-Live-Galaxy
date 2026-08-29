use mind_orchestration::ProviderFailure;

pub(crate) fn schema_path() -> Result<String, ProviderFailure> {
    let path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../shadow-deliberation-evals/v1/schema.json");
    let canonical = path
        .canonicalize()
        .map_err(|_| ProviderFailure::Unavailable)?;
    if canonical.file_name().and_then(std::ffi::OsStr::to_str) != Some("schema.json") {
        return Err(ProviderFailure::Unavailable);
    }
    canonical
        .into_os_string()
        .into_string()
        .map_err(|_| ProviderFailure::Unavailable)
}
