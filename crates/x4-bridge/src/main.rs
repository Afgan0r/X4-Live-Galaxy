use std::io::Write as _;

fn main() -> Result<(), x4_bridge::StartupError> {
    if let Some(output) = x4_bridge::run_production(std::env::args_os().skip(1))? {
        let _ = writeln!(std::io::stdout().lock(), "{output}");
    }
    Ok(())
}
