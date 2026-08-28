use std::env;
use std::ffi::OsStr;
use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::process::{Command, ExitCode};

const MAX_RUST_FILE_LINES: usize = 300;

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

    let violations = files
        .into_iter()
        .filter_map(|path| {
            let source = fs::read_to_string(&path).ok()?;
            let lines = source.lines().count();
            (lines > MAX_RUST_FILE_LINES).then_some((path, lines))
        })
        .collect::<Vec<_>>();

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
