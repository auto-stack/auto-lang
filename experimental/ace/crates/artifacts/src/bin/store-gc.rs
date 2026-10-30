//! PLAN-788 R3 (F02): one-shot unified-store GC runner. The turnaround
//! harness spawns this as a REAL concurrent GC process while worker
//! requests flow through the same store — overlap is proven by barrier
//! lines on stdout ("gc-started" once the store is OPEN, before any
//! scanning; "store-gc-ok ..." after the run), not by thread bookkeeping.
//! Uses the exact same `ManagedStore` + `run_gc` entry the 783 suites
//! exercise; no new GC semantics. Watermarks default to 0/0 so every run
//! is a REAL decide+delete scan (the default high watermark would no-op
//! on small stores — that would not exercise concurrent collection).

use ace_artifacts::store::gc::{run_gc, GcConfig};
use ace_artifacts::store::ManagedStore;

fn main() {
    let mut args = std::env::args();
    let root = args
        .nth(1)
        .unwrap_or_else(|| "usage: store-gc <store-root> [high] [low]".to_string());
    let high: u64 = args.next().and_then(|s| s.parse().ok()).unwrap_or(0);
    let low: u64 = args.next().and_then(|s| s.parse().ok()).unwrap_or(0);
    let store = ManagedStore::open(std::path::Path::new(&root))
        .unwrap_or_else(|e| panic!("store-gc: open {root}: {e:?}"));
    println!("gc-started");
    use std::io::Write;
    let _ = std::io::stdout().flush();
    let receipt = run_gc(&store, GcConfig { high, low })
        .unwrap_or_else(|e| panic!("store-gc: run failed on {root}: {e:?}"));
    println!(
        "store-gc-ok ran={} scanned={} live={} freed={}",
        receipt.ran, receipt.scanned_chunks, receipt.live_chunks, receipt.freed_bytes
    );
}
