fn main() {
    let args: Vec<_> = std::env::args().skip(1).collect();
    if shadow_harness::run_cli(&args, shadow_harness::CodexProcess).is_err() {
        std::process::exit(2);
    }
}
