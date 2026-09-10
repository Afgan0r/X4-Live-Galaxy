use std::io::Write as _;
use std::process::ExitCode;

use x4_bridge::ProductionLimits;

fn main() -> ExitCode {
    let mut arguments = std::env::args_os().skip(1);
    let (Some(flag), Some(path), None) = (arguments.next(), arguments.next(), arguments.next())
    else {
        return reject(b"usage: validate_limits --limits-file <path>\n");
    };
    if flag != "--limits-file" {
        return reject(b"usage: validate_limits --limits-file <path>\n");
    }
    if ProductionLimits::read(&std::path::PathBuf::from(path)).is_none() {
        return reject(b"limits rejected by the production parser\n");
    }
    ExitCode::SUCCESS
}

fn reject(message: &[u8]) -> ExitCode {
    let _ = std::io::stderr().lock().write_all(message);
    ExitCode::FAILURE
}
