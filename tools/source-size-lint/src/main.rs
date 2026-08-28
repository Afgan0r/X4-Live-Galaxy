use std::env;
use std::ffi::OsStr;
use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::process::{Command, ExitCode};

const MAX_RUST_FILE_LINES: usize = 200;

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            let _ = writeln!(io::stderr().lock(), "{error}");
            ExitCode::FAILURE
        }
    }
}

fn run() -> Result<(), String> {
    let repository = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .ok_or_else(|| String::from("lint tool must live under tools/source-size-lint"))?;

    enforce_source_size(repository)?;

    for (program, arguments) in [
        ("cargo", vec!["fmt", "--check"]),
        (
            "cargo",
            vec![
                "clippy",
                "--workspace",
                "--lib",
                "--bins",
                "--all-features",
                "--",
                "-D",
                "warnings",
                "-D",
                "clippy::expect_used",
                "-D",
                "clippy::panic",
                "-D",
                "clippy::unwrap_used",
            ],
        ),
        (
            "cargo",
            vec![
                "clippy",
                "--workspace",
                "--all-targets",
                "--all-features",
                "--",
                "-D",
                "warnings",
                "-A",
                "clippy::expect_used",
                "-A",
                "clippy::panic",
                "-A",
                "clippy::unwrap_used",
            ],
        ),
    ] {
        let status = Command::new(program)
            .args(arguments)
            .current_dir(repository)
            .status()
            .map_err(|error| format!("failed to run {program}: {error}"))?;
        if !status.success() {
            return Err(format!("{program} failed with {status}"));
        }
    }

    Ok(())
}

fn enforce_source_size(repository: &Path) -> Result<(), String> {
    let mut files = Vec::new();
    collect_rust_files(&repository.join("crates"), &mut files)?;
    collect_rust_files(&repository.join("tools"), &mut files)?;
    files.sort();

    let mut violations = Vec::new();
    for path in files {
        let source = read_source(&path)?;
        let lines = source.lines().count();
        if lines > MAX_RUST_FILE_LINES {
            violations.push((path, lines));
        }
    }

    if violations.is_empty() {
        return Ok(());
    }

    let details = violations
        .into_iter()
        .map(|(path, lines)| {
            let relative = path.strip_prefix(repository).unwrap_or(&path);
            format!("{}: {lines} lines", relative.display())
        })
        .collect::<Vec<_>>()
        .join("\n");

    Err(format!(
        "Rust source file limit exceeded (max {MAX_RUST_FILE_LINES} lines):\n{details}"
    ))
}

fn read_source(path: &Path) -> Result<String, String> {
    fs::read_to_string(path)
        .map_err(|error| format!("failed to read Rust source {}: {error}", path.display()))
}

fn collect_rust_files(directory: &Path, files: &mut Vec<PathBuf>) -> Result<(), String> {
    if !directory.exists() {
        return Ok(());
    }

    for entry in fs::read_dir(directory).map_err(|error| error.to_string())? {
        let entry = entry.map_err(|error| error.to_string())?;
        collect_path(entry.path(), files)?;
    }

    Ok(())
}

fn collect_path(path: PathBuf, files: &mut Vec<PathBuf>) -> Result<(), String> {
    if path.is_dir() && path.file_name() != Some(OsStr::new("target")) {
        return collect_rust_files(&path, files);
    }
    if path.extension() == Some(OsStr::new("rs")) {
        files.push(path);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::fs;

    use super::read_source;

    #[test]
    fn unreadable_rust_source_fails_with_its_path() {
        let path = std::env::temp_dir().join("source-size-lint-non-utf8.rs");
        assert!(
            fs::write(&path, [0xff]).is_ok(),
            "test fixture must be writable"
        );

        let error = read_source(&path).expect_err("non-UTF-8 source must fail closed");

        assert!(error.contains(&path.display().to_string()));
        assert!(
            fs::remove_file(path).is_ok(),
            "test fixture must be removable"
        );
    }
}
