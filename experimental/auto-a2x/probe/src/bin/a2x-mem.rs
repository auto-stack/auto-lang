//! PLAN-892 T-04 resource probe binary: measure the A2X MANAGEMENT working
//! set (the emitter itself — frontend + view + lowering + session/store
//! bookkeeping; NO rustc, NO target execution) at load256 scale.
//!
//! Scale: 115 functions, ~2.4 KiB per function body → the frozen 273,575B /
//! 115-function baseline shape (the formal sampling window, T-06, keeps the
//! ORIGINAL load256 bytes; this probe reproduces the SHAPE for the budget
//! measurement). Runs one cold assembly + 30 pure body edits (fn_037-style
//! first-multiplier bumps) and reports the process PEAK working set plus
//! producer stats. Exit 0 = measurement performed; the budget verdict is
//! the test's assertion (probe reports, never grades itself).

use auto_a2x_store::A2xSession;

/// 115 caller functions; fn_037 is the edit target (load256 layout).
const FN_COUNT: usize = 115;

#[cfg(windows)]
fn current_working_set_bytes() -> u64 {
    use windows_sys::Win32::System::ProcessStatus::{
        GetProcessMemoryInfo, PROCESS_MEMORY_COUNTERS,
    };
    unsafe {
        let mut counters: PROCESS_MEMORY_COUNTERS = std::mem::zeroed();
        let handle = windows_sys::Win32::System::Threading::GetCurrentProcess();
        let ok = GetProcessMemoryInfo(
            handle,
            &mut counters,
            std::mem::size_of::<PROCESS_MEMORY_COUNTERS>() as u32,
        );
        assert_ne!(ok, 0);
        counters.WorkingSetSize as u64
    }
}

#[cfg(not(windows))]
fn current_working_set_bytes() -> u64 {
    std::fs::read_to_string("/proc/self/status")
        .ok()
        .and_then(|s| {
            s.lines()
                .find(|l| l.starts_with("VmRSS:"))
                .and_then(|l| {
                    l.split_whitespace()
                        .nth(1)
                        .and_then(|n| n.parse::<u64>().ok())
                })
                .map(|kb| kb * 1024)
        })
        .unwrap_or(0)
}

/// Only the edited function's text (the warm-path contract input).
fn generate_fragment(revision: u32) -> String {
    let i = 37usize;
    let mut src = String::with_capacity(2800);
    src.push_str("fn fn_037(a int) int {\n");
    src.push_str("    var s = a\n");
    for k in 0..104 {
        let m = 3 + ((i * 7 + k * 11 + revision as usize) % 97);
        src.push_str(&format!("    s = s + {} * {} + s\n", k, m));
    }
    src.push_str("    return s\n}\n");
    src
}

fn generate_source(revision: u32) -> String {
    let mut src = String::with_capacity(280_000);
    for i in 0..FN_COUNT {
        let bump = if i == 37 { revision } else { 0 };
        let name = if i == 37 {
            "fn_037".to_string()
        } else {
            format!("fn_{:03}", i)
        };
        src.push_str(&format!("fn {}(a int) int {{\n", name));
        src.push_str("    var s = a\n");
        // ~40 statements of chained adds → ~2.3 KiB per function
        for k in 0..104 {
            let m = 3 + ((i * 7 + k * 11 + bump as usize) % 97);
            src.push_str(&format!("    s = s + {} * {} + s\n", k, m));
        }
        src.push_str("    return s\n}\n");
    }
    src.push_str("fn main() int { return fn_000(1) + fn_037(2) }\n");
    src
}

fn main() {
    let out_dir = std::env::temp_dir().join(format!("a2x-mem-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&out_dir);
    std::fs::create_dir_all(&out_dir).unwrap();

    let bytes = generate_source(0).len();
    let mut session =
        A2xSession::open(&out_dir, "a2x-mem", "mem-probe", 128 * 1024 * 1024).unwrap();
    // ---- COLD phase (upstream frontend cost, informational) ----
    let cold = session
        .cold_assemble(&generate_source(0), false, &[], "main", false)
        .expect("cold assembly at load256 scale");
    println!("cold_peak_working_set_bytes={}", peak_working_set_bytes());
    println!(
        "post_cold_current_working_set_bytes={}",
        current_working_set_bytes()
    );
    let cold_generated = cold
        .files
        .iter()
        .find(|(p, _)| p == "generated.rs")
        .map(|(_, c)| c.len())
        .unwrap_or(0);

    // ---- WARM steady state: fragment-only edits; the A2X management
    // budget is the max CURRENT working set across the warm phase (the
    // cold frontend transient is gone by then). ----
    let mut warm_max_current = current_working_set_bytes();
    for r in 1..=30u32 {
        session
            .warm_edit("fn_037", &generate_fragment(r), "main", false)
            .unwrap_or_else(|e| panic!("warm edit r{} failed: {:?}", r, e));
        warm_max_current = warm_max_current.max(current_working_set_bytes());
    }

    let stats = session.stats().clone();
    let peak = peak_working_set_bytes();
    println!("source_bytes={}", bytes);
    println!("functions={}", FN_COUNT + 1);
    println!("cold_generated_bytes={}", cold_generated);
    println!("warm_edits=30");
    println!(
        "stats lower={} print={} persist={} bytes_persisted={} resident_reuse={} store_hits={}",
        stats.lower,
        stats.print,
        stats.persist,
        stats.bytes_persisted,
        stats.fragments_reused_resident,
        stats.store_hit_fragment
    );
    println!("process_peak_working_set_bytes={}", peak);
    println!("warm_max_current_working_set_bytes={}", warm_max_current);
    let _ = std::fs::remove_dir_all(&out_dir);
}

#[cfg(windows)]
fn peak_working_set_bytes() -> u64 {
    use windows_sys::Win32::System::ProcessStatus::{
        GetProcessMemoryInfo, PROCESS_MEMORY_COUNTERS,
    };
    unsafe {
        let mut counters: PROCESS_MEMORY_COUNTERS = std::mem::zeroed();
        let handle = windows_sys::Win32::System::Threading::GetCurrentProcess();
        let ok = GetProcessMemoryInfo(
            handle,
            &mut counters,
            std::mem::size_of::<PROCESS_MEMORY_COUNTERS>() as u32,
        );
        assert_ne!(ok, 0, "GetProcessMemoryInfo failed");
        counters.PeakWorkingSetSize as u64
    }
}

#[cfg(not(windows))]
fn peak_working_set_bytes() -> u64 {
    // /proc/self/status VmHWM (Linux fallback)
    std::fs::read_to_string("/proc/self/status")
        .ok()
        .and_then(|s| {
            s.lines()
                .find(|l| l.starts_with("VmHWM:"))
                .and_then(|l| {
                    l.split_whitespace()
                        .nth(1)
                        .and_then(|n| n.parse::<u64>().ok())
                })
                .map(|kb| kb * 1024)
        })
        .unwrap_or(0)
}
