// Review-only formatter state adapter. No large allocations or native ABI claim.
// Appended to the exact current shim, removing only crate-level std scaffolding.
fn main() {
    let case = std::env::args().nth(1).expect("edge/wide/released");
    let counters = c();
    counters.attempts = 11;
    counters.allocs = 5;
    counters.frees = if case == "released" { 5 } else { 2 };
    counters.live = counters.allocs - counters.frees;
    counters.free_attempts = counters.frees;
    counters.invalid_frees = 0;
    match case.as_str() {
        "edge" => {
            counters.live_bytes = u32::MAX as u64;
            counters.peak = u32::MAX as u64;
        }
        "wide" => {
            counters.live_bytes = 4294967298;
            counters.peak = 4294967315;
        }
        "released" => {
            counters.live_bytes = 0;
            counters.peak = 4294967315;
        }
        _ => panic!("unknown formatter case"),
    }
    // The ORIGINAL formatter, line schema and WriteFile output path.
    emit_summary();
}
