use std::process::ExitCode;

use x4_bridge::ProductionLimits;

fn main() -> ExitCode {
    let mut arguments = std::env::args_os().skip(1);
    let (Some(flag), Some(path), None) = (arguments.next(), arguments.next(), arguments.next())
    else {
        eprintln!("usage: validate_limits --limits-file <path>");
        return ExitCode::FAILURE;
    };
    if flag != "--limits-file" {
        eprintln!("usage: validate_limits --limits-file <path>");
        return ExitCode::FAILURE;
    }
    if ProductionLimits::read(&std::path::PathBuf::from(path)).is_none() {
        eprintln!("limits rejected by the production parser");
        return ExitCode::FAILURE;
    }
    ExitCode::SUCCESS
}
