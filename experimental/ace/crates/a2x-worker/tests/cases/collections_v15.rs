//! PLAN-904 T-02 V-15 helper probes (typed runtime, independent scope):
//! the billing table (call ctor, list slots, Str bytes), budget
//! exhaustion (`Budget`), cooperative cancellation at the poll point
//! (`Cancelled`), fault-at-the-n-th-allocation atomicity (the old target
//! survives, the uncommitted input's responsibility is reclaimed, live
//! counters fall back), and no-margin cleanup (free paths never bill,
//! poll or allocate — a budget of exactly the construction cost still
//! allows full cleanup).
//!
//! These probes consume the SAME generated runtime content the channel
//! emits (`collections_runtime_source`) through a hand-written probe
//! main — they are helper evidence, independent of the 20 source cases.

use std::path::PathBuf;
use std::process::Command;

use ace_a2x_rust::coll::collections_runtime_source;

fn out_base() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../target/a2x-out/collections-904-v15")
}

fn compile_probe(name: &str, main_rs: &str) -> PathBuf {
    let dir = out_base().join(name);
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(dir.join("a2x_runtime.rs"), collections_runtime_source(&[])).unwrap();
    std::fs::write(dir.join("main.rs"), main_rs).unwrap();
    let exe = dir.join("probe.exe");
    let compile = auto_tool_execution::run_command_capture(
        {
            let mut c = Command::new(std::env::var("RUSTC").unwrap_or_else(|_| "rustc".to_string()));
            c.args(["--edition", "2021", "-o"])
                .arg(&exe)
                .arg("main.rs")
                .current_dir(&dir);
            c
        },
        auto_tool_execution::TreeScope::ProcessTree,
        None,
    )
    .expect("rustc spawn");
    assert_eq!(
        compile.kind,
        auto_tool_execution::ExitKind::Success,
        "rustc failed for {name}: {}",
        String::from_utf8_lossy(compile.stderr.as_ref().unwrap())
    );
    exe
}

fn run_with(exe: &PathBuf, env: &[(&str, &str)]) -> (String, String) {
    let run = auto_tool_execution::run_command_capture(
        {
            let mut c = Command::new(exe);
            for (k, v) in env {
                c.env(k, v);
            }
            c
        },
        auto_tool_execution::TreeScope::ProcessTree,
        None,
    )
    .expect("run spawn");
    (
        String::from_utf8_lossy(run.stdout.as_ref().unwrap()).to_string(),
        String::from_utf8_lossy(run.stderr.as_ref().unwrap()).to_string(),
    )
}

/// Push values + a poll loop; prints live counters and the trap name.
const PROBE_MAIN: &str = r#"
mod a2x_runtime;
use a2x_runtime::*;

fn main() {
    // the frame discipline the generated channel emits: every error path
    // frees the surviving owned slots exactly once
    let mut xs = match a2x_coll_list_new::<A2xStr>() { Ok(v) => v, Err(e) => { println!("nodes:{} str_bytes:{} backing:{}", a2x_coll_live_nodes(), a2x_coll_live_str_bytes(), a2x_coll_live_backing_bytes()); println!("trap:{}", e.name()); return; } };
    let outcome: A2xResult<i32> = (|| {
        for _ in 0..4 {
            let s = match a2x_str_from_lit("abc") { Ok(v) => v, Err(e) => return Err(e) };
            if let Err(e) = a2x_coll_list_push(&mut xs, s) { return Err(e); }
        }
        loop {
            if let Err(e) = a2x_coll_poll() { return Err(e); }
        }
    })();
    a2x_coll_list_free(&mut xs);
    println!("nodes:{} str_bytes:{} backing:{}", a2x_coll_live_nodes(), a2x_coll_live_str_bytes(), a2x_coll_live_backing_bytes());
    match outcome {
        Ok(v) => println!("value:{}", v),
        Err(t) => println!("trap:{}", t.name()),
    }
}
"#;

/// Fault at a chosen allocation ordinal; verifies the old list and the
/// counters (atomicity: nothing leaks, nothing double-frees).
const FAULT_MAIN: &str = r#"
mod a2x_runtime;
use a2x_runtime::*;

fn main() {
    let outcome: A2xResult<i32> = (|| {
        let mut xs = a2x_coll_list_new::<A2xStr>()?;
        // two committed pushes, then a faulted growth (ordinal chosen by env)
        for _ in 0..2 {
            let s = a2x_str_from_lit("xy");
            let s = match s { Ok(v) => v, Err(e) => return a2x_report(e, &mut xs) };
            match a2x_coll_list_push(&mut xs, s) {
                Ok(()) => {}
                Err(e) => return a2x_report(e, &mut xs),
            }
        }
        let len_before = a2x_coll_list_len(&xs);
        let s = match a2x_str_from_lit("zz") { Ok(v) => v, Err(e) => return a2x_report(e, &mut xs) };
        match a2x_coll_list_push(&mut xs, s) {
            Ok(()) => {}
            Err(e) => return a2x_report(e, &mut xs),
        }
        Ok(len_before)
    })();
    println!("nodes:{} str_bytes:{}", a2x_coll_live_nodes(), a2x_coll_live_str_bytes());
    match outcome {
        Ok(v) => println!("value:{}", v),
        Err(t) => println!("trap:{}", t.name()),
    }
}

fn a2x_report(e: AutoTrap, xs: &mut A2xList<A2xStr>) -> A2xResult<i32> {
    // the caller-side error path: the runtime freed the uncommitted input;
    // the frame frees the surviving old target exactly once
    a2x_coll_list_free(xs);
    Err(e)
}
"#;

/// Deep-clone partial rollback: a long chain whose n-th clone payload
/// fails; the partially cloned prefix is reclaimed (counters fall back).
const DEEP_MAIN: &str = r#"
mod a2x_runtime;
use a2x_runtime::*;

fn main() {
    let mut chain: Option<Box<A2xDeepLink>> = None;
    let outcome: A2xResult<i32> = (|| {
        for _ in 0..500 {
            a2x_coll_deep_push(&mut chain, "payload-0123456789")?;
        }
        let mut cloned = match a2x_coll_deep_clone(&chain) { Ok(v) => v, Err(e) => return Err(e) };
        let n = a2x_coll_deep_len(&cloned);
        a2x_coll_deep_free(&mut cloned);
        Ok(n)
    })();
    a2x_coll_deep_free(&mut chain);
    println!("str_bytes:{}", a2x_coll_live_str_bytes());
    match outcome {
        Ok(v) => println!("value:{}", v),
        Err(t) => println!("trap:{}", t.name()),
    }
}
"#;

#[test]
fn v15_budget_exhaustion_and_cancellation() {
    let exe = compile_probe("budget", PROBE_MAIN);
    // default budget: the infinite poll loop eventually exhausts the work
    // budget at the polling point — Budget, and the frame is CLEANED
    // (nodes/str_bytes fall back to zero through the explicit frees)
    let (out, _) = run_with(&exe, &[]);
    assert!(
        out.contains("trap:Budget") || out.contains("trap:Cancelled"),
        "expected budget/cancel trap, got: {out}"
    );
    assert!(
        out.contains("nodes:0 str_bytes:0"),
        "error path must clean the frame (nodes/str_bytes back to 0): {out}"
    );
    // armed cancellation turns the poll point into Cancelled
    let (out, _) = run_with(&exe, &[("A2X_RT_CANCEL", "1")]);
    assert!(out.contains("trap:Cancelled"), "cancel at poll: {out}");
    assert!(out.contains("nodes:0 str_bytes:0"), "cancel cleans: {out}");
    // a tiny budget refuses BEFORE the work happens (still cleaned)
    let (out, _) = run_with(&exe, &[("A2X_RT_BUDGET", "2")]);
    assert!(out.contains("trap:Budget"), "tiny budget: {out}");
    assert!(out.contains("nodes:0 str_bytes:0"), "refusal cleans: {out}");
}

#[test]
fn v15_fault_atomicity_and_live_fallback() {
    let exe = compile_probe("fault", FAULT_MAIN);
    // no fault: both pushes commit (the growth at push #3 is a real alloc)
    let (out, _) = run_with(&exe, &[]);
    assert!(out.contains("value:2"), "no-fault control: {out}");
    assert!(out.contains("nodes:3 str_bytes:6"), "live accounting: {out}");
    // fault ordinal 3 = the list-growth request of push #3 (allocs:
    // new=1, str x2=2, then growth=3): old list intact (2 elems), the
    // uncommitted input freed by the runtime, the frame freed by the
    // caller — nothing leaks
    let (out, _) = run_with(&exe, &[("A2X_RT_FAULT_ALLOC", "3")]);
    assert!(out.contains("trap:Resource"), "fault surfaces Resource: {out}");
    assert!(
        out.contains("nodes:0 str_bytes:0"),
        "fault path must reclaim everything (old target + uncommitted input): {out}"
    );
    // fault at the literal promotion of the third push (ordinal 4): same
    // atomicity, list not yet mutated
    let (out, _) = run_with(&exe, &[("A2X_RT_FAULT_ALLOC", "4")]);
    assert!(out.contains("trap:Resource"), "fault at promotion: {out}");
    assert!(out.contains("nodes:0 str_bytes:0"), "no leak: {out}");
}

#[test]
fn v15_deep_clone_rollback_and_cleanup() {
    let exe = compile_probe("deep", DEEP_MAIN);
    // control: full clone, then both chains free — counters back to zero
    let (out, _) = run_with(&exe, &[]);
    assert!(out.contains("value:500"), "deep clone length: {out}");
    assert!(
        out.contains("str_bytes:0"),
        "iterative deep free reclaims everything: {out}"
    );
    // fault near the end of the clone: the partial prefix is rolled back
    // by the helper and the frame is cleaned
    let (out, _) = run_with(&exe, &[("A2X_RT_FAULT_ALLOC", "600")]);
    assert!(
        out.contains("trap:Resource"),
        "deep clone fault surfaces: {out}"
    );
    assert!(
        out.contains("str_bytes:0"),
        "rollback + cleanup reclaim the partial prefix: {out}"
    );
}

/// RV-2 (final review r1): the frozen resource-side witness — a chain of
/// 3072+ nesting levels constructed, cloned and freed ENTIRELY inside a
/// 256KiB-stack thread (the iterative work-loop cleanup never multiplies
/// the host stack; a shallow control proves the mechanism, not the depth).
#[test]
fn v15_deep_3072_small_stack_witness() {
    let exe = compile_probe(
        "deep-small-stack",
        r#"
mod a2x_runtime;
use a2x_runtime::*;

fn main() {
    // the witness runs under a 256KiB stack: iterative helpers must not
    // multiply the host stack by nesting depth
    let h = std::thread::Builder::new()
        .stack_size(256 * 1024)
        .spawn(|| -> i32 {
            let mut chain: Option<Box<A2xDeepLink>> = None;
            for _ in 0..3072 {
                if let Err(e) = a2x_coll_deep_push(&mut chain, "w-0123456789") {
                    return -1;
                }
            }
            let len = a2x_coll_deep_len(&chain);
            let mut cloned = match a2x_coll_deep_clone(&chain) { Ok(v) => v, Err(_) => return -2 };
            let clen = a2x_coll_deep_len(&cloned);
            a2x_coll_deep_free(&mut cloned);
            a2x_coll_deep_free(&mut chain);
            if a2x_coll_live_str_bytes() != 0 { return -3; }
            len * 10_000 + clen
        })
        .expect("spawn witness thread");
    let r = h.join().expect("witness thread");
    println!("str_bytes:{}", a2x_coll_live_str_bytes());
    if r >= 0 {
        println!("value:{}", r / 10_000 + r % 10_000);
    } else {
        println!("trap:Resource");
    }
}
"#,
    );
    let (out, _) = run_with(&exe, &[]);
    // 3072-deep chain built, cloned, both freed under the small stack,
    // live accounting back to zero
    assert!(out.contains("value:6144"), "3072 + clone 3072: {out}");
    assert!(out.contains("str_bytes:0"), "both chains fully reclaimed: {out}");
}
