fn main() {
    let benchmark_requested = std::env::args().nth(1).as_deref() == Some("--benchmark");
    std::hint::black_box(benchmark_requested);
}
