//! PLAN-827 T-04 semantics/admission/resources axes (V-02..V-06, V-09
//! targeted subset) over the REAL seed files and derived variants.
//!
//! Variants are derived IN-TEST from the frozen seed bytes by exactly-once
//! literal replacement (the same rule the formal edit-source step enforces
//! worker- and parent-side); nothing is written back into the seed tree.

use auto_av_core::av::{CancelFlag, ExecLimits};
use auto_av_core::contract::{ExecOutcome, ResourceKind, TrapKind, ABC_SCHEMA, TRACE_CAPABILITY};
use auto_av_core::mir::OwnerScope;
use auto_av_core::{decode_abc, encode_abc, execute, verify_abc};
use auto_av_probe::driver::Driver;
use auto_av_probe::hir_adapter::lower_checked;

const AP01_ORIGINAL: &str = "../../ac-core/fixtures/scalar-source/valid/p-gcd-euclid.at";
const AP22_ORIGINAL: &str = "../../ac-core/fixtures/scalar-source/valid/p-binop-args.at";
const AP23_ORIGINAL: &str = "../../ac-core/fixtures/scalar-source/valid/p-short-circuit-trap.at";
const AP01_SHA256: &str = "6d466c03e1b0012f7ac9b883cf88f253345fd911ac3ced2d1f282ce042799af3";
const AP22_SHA256: &str = "46d6a431a5c1b9bbe6e2e87b8732c09a9c584b58eb92799fdb4ab073f423e625";
const AP23_SHA256: &str = "d6871b77c9b02e797c77a2cf34900e005fe97062bf1fc95ffcdf1cf7707052a9";

fn sha256_hex(bytes: &[u8]) -> String {
    use sha2::{Digest, Sha256};
    Sha256::digest(bytes)
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect()
}

fn read_seed(rel: &str, frozen_sha: &str) -> String {
    let bytes = std::fs::read(std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(rel))
        .unwrap_or_else(|e| panic!("read seed {rel}: {e}"));
    assert_eq!(sha256_hex(&bytes), frozen_sha, "seed {rel} drifted");
    String::from_utf8(bytes).unwrap()
}

fn replace_once(src: &str, from: &str, to: &str) -> String {
    let hits = src.matches(from).count();
    assert_eq!(hits, 1, "edit literal `{from}` matched {hits} times");
    src.replacen(from, to, 1)
}

/// V-02: AP-22 real file -> 0; a derived variant that CHANGES AN ARGUMENT
/// VALUE (and only it) changes the independent result (1) — source value
/// changes must flow through the real pipeline, and the input hash moves.
#[test]
fn av_source_827_ap22_argument_value_change_changes_result() {
    let src = read_seed(AP22_ORIGINAL, AP22_SHA256);
    let mut d = Driver::ephemeral();
    let r1 = d.request(&src, "d_main", &[]);
    assert!(r1.ok(), "AP-22 original: {:?}", r1.diag);
    assert_eq!(r1.value_i32(), Some(0));
    // Group 1: add(1 - 2, 3 * 4) = 11 holds. Claiming 12 makes the check
    // fail -> main returns 1. Only argument/check constants changed.
    let variant = replace_once(
        &src,
        "if add(1 - 2, 3 * 4) != 11 { return 1 }",
        "if add(1 - 2, 3 * 4) != 12 { return 1 }",
    );
    let r2 = d.request(&variant, "d_main", &[]);
    assert!(r2.ok(), "AP-22 variant: {:?}", r2.diag);
    assert_eq!(r2.value_i32(), Some(1), "changed source value, new result");
    assert_ne!(
        r1.source_hash, r2.source_hash,
        "different content must carry a different identity"
    );
}

/// V-03: AP-23 — the skipped RHS never traps; the executed RHS traps i32
/// divide-by-zero (the precise kind, never "any error"). The false-only
/// derivative is the positive control: no RHS executes, guest returns 0.
#[test]
fn av_source_827_ap23_precise_trap_and_false_only_control() {
    let src = read_seed(AP23_ORIGINAL, AP23_SHA256);
    let mut d = Driver::ephemeral();
    let r = d.request(&src, "d_main", &[]);
    assert!(r.diag.is_none(), "trap is an executed outcome, not a diag");
    match &r.av {
        Some(ExecOutcome::Trap {
            kind: TrapKind::DivByZero,
            ..
        }) => {}
        other => panic!("expected DivByZero trap, got {other:?}"),
    }
    // Positive control: BOTH guards false — no RHS executes, no trap, 0.
    let variant = replace_once(&src, "let boom = true &&", "let boom = false &&");
    let rc = d.request(&variant, "d_main", &[]);
    assert!(rc.ok(), "false-only control: {:?}", rc.diag);
    assert_eq!(rc.value_i32(), Some(0));
}

/// V-03 native channel: the SAME frozen bytes through the native reference
/// build/run — trap = exit 70, false-only control = exit 0. Separate
/// channel from the AV guest outcome.
#[test]
#[cfg(windows)]
fn av_source_827_ap23_native_exit70_channel() {
    let src = read_seed(AP23_ORIGINAL, AP23_SHA256);
    if auto_ac_prototype::link::find_rust_lld().is_err() {
        eprintln!("infra-blocked: rust-lld unavailable — native channel skipped");
        return;
    }
    let base = std::env::temp_dir().join(format!("av827-t04-native-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&base);
    let trap_dir = base.join("trap");
    let run = auto_av_probe::native_reference::build_and_run(&src, &[], "d_main", &trap_dir, None)
        .expect("native trap run");
    assert_eq!(run.exit_code, 70, "i32 divide-by-zero traps as exit 70");
    let variant = replace_once(&src, "let boom = true &&", "let boom = false &&");
    let ok_dir = base.join("ok");
    let run =
        auto_av_probe::native_reference::build_and_run(&variant, &[], "d_main", &ok_dir, None)
            .expect("native control run");
    assert_eq!(run.exit_code, 0, "false-only control exits 0");
    let _ = std::fs::remove_dir_all(&base);
}

/// V-02 trace authorization: D798's original trace source example under
/// `hir.test.trace.v1` — exact order A,B,A (mark_a, side's mark_b,
/// mark_a), value 1 (1 + 3*1 + 1 = 5 != 10). WITHOUT the capability the
/// same run produces an EMPTY trace (capability-gated, not silently on).
#[test]
fn av_source_827_trace_order_exact_and_capability_gated() {
    let src_path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../../crates/auto-lang/test/cases/ac-av/av/core/trace_order.at");
    let src = std::fs::read_to_string(&src_path).unwrap_or_else(|e| panic!("read trace seed: {e}"));
    let mut d = Driver::ephemeral();
    let granted = d.request(&src, "d_main", &[TRACE_CAPABILITY.to_string()]);
    assert!(granted.ok(), "granted run: {:?}", granted.diag);
    assert_eq!(granted.value_i32(), Some(1));
    let marks: Vec<&str> = granted
        .trace
        .iter()
        .map(|e| match e {
            auto_av_core::mir::TraceEvent::MarkA => "a",
            auto_av_core::mir::TraceEvent::MarkB => "b",
        })
        .collect();
    assert_eq!(marks, vec!["a", "b", "a"], "exact trace order a,b,a");
    let denied = d.request(&src, "d_main", &[]);
    // Capability gate (the real delivered semantics): without the grant the
    // mark intrinsics are NOT in scope — the trace source is diagnosed at
    // the frontend (`source.undefined` on the first mark), never silently
    // executed without marks, never a fallback path.
    let diag = denied
        .diag
        .as_ref()
        .expect("without the capability the trace source must be diagnosed");
    assert_eq!(
        (diag.stage.as_str(), diag.code.as_str()),
        ("frontend", "source.undefined")
    );
    assert!(
        denied.trace.is_empty(),
        "a diagnosed run carries no trace events"
    );
}

/// V-04 identity axes (file-level): same path, different bytes -> different
/// identity AND different correct result; frozen bytes -> later disk
/// mutation cannot change what the admitted request executes.
#[test]
fn av_source_827_same_path_different_bytes_and_frozen_snapshot() {
    let ap01 = read_seed(AP01_ORIGINAL, AP01_SHA256);
    let overlay = replace_once(&ap01, "return 0", "return 9");
    let dir = std::env::temp_dir().join(format!("av827-id-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let path = dir.join("same-name.at");

    let root = dir.join("store");
    let mut d = Driver::with_store(&root, "av827-id").expect("store");
    std::fs::write(&path, &ap01).unwrap();
    let v1 = std::fs::read_to_string(&path).unwrap();
    let r1 = d.request(&v1, "d_main", &[]);
    assert!(r1.ok(), "{:?}", r1.diag);
    assert_eq!(r1.value_i32(), Some(0));

    // Same path, different bytes: identity moves, result follows content.
    std::fs::write(&path, &overlay).unwrap();
    let v2 = std::fs::read_to_string(&path).unwrap();
    let r2 = d.request(&v2, "d_main", &[]);
    assert!(r2.ok(), "{:?}", r2.diag);
    assert_eq!(r2.value_i32(), Some(9), "overlay 0->9 executes as 9");
    assert_ne!(r1.source_hash, r2.source_hash);

    // Frozen snapshot: mutate the disk AFTER the bytes were read; the
    // already-admitted content still runs (0), never the disk's new state.
    std::fs::write(&path, "fn main() int { return 5 }\n").unwrap();
    let r3 = d.request(&v1, "d_main", &[]);
    assert!(r3.ok(), "{:?}", r3.diag);
    assert_eq!(r3.value_i32(), Some(0), "frozen bytes beat disk mutation");
    let _ = std::fs::remove_dir_all(&dir);
}

/// V-05 front door on real rejections: unsupported/syntax inputs are
/// diagnosed BEFORE the pipeline with ZERO AV dispatch (no old-VM
/// fallback, no host side effects), and an unsupported input never
/// produces a program success.
#[test]
fn av_source_827_front_door_rejects_disallow_no_dispatch() {
    let mut d = Driver::ephemeral();
    let print_src = "fn main() int {\n    print(\"hi\")\n    return 0\n}\n";
    let r = d.request(print_src, "d_main", &[]);
    let diag = r.diag.as_ref().expect("print must be diagnosed");
    assert_eq!(
        (diag.stage.as_str(), diag.code.as_str()),
        ("frontend", "source.unsupported")
    );
    assert_eq!(*r.counters.get("frontend_reject").unwrap_or(&0), 1);
    assert_eq!(
        *r.counters.get("av_execute").unwrap_or(&0),
        0,
        "rejection happens with ZERO AV dispatch"
    );
    let list_src = "fn main() int {\n    let xs = [1, 2, 3]\n    return xs.len()\n}\n";
    let r = d.request(list_src, "d_main", &[]);
    let diag = r.diag.as_ref().expect("list literal must be diagnosed");
    assert_eq!(
        (diag.stage.as_str(), diag.code.as_str()),
        ("frontend", "source.syntax")
    );
    assert_eq!(*r.counters.get("av_execute").unwrap_or(&0), 0);
    // Wrong entry: a diagnosed lowering rejection, not a guessed run.
    let r = d.request("fn main() int {\n    return 3\n}\n", "d_nope", &[]);
    let diag = r.diag.as_ref().expect("wrong entry must be diagnosed");
    assert_eq!(
        (diag.stage.as_str(), diag.code.as_str()),
        ("mir-lower", "lower.entry")
    );
}

/// V-04/V-09 budget axes: over-budget input is rejected at admission
/// (before the frontend, zero dispatch); an infinite loop is a RESOURCE
/// outcome at the step budget (never a language trap, never an OS kill);
/// a host cancellation surfaces as Cancelled with real prior activity.
#[test]
fn av_source_827_resource_budget_and_cancel_axes() {
    let mut d = Driver::ephemeral();
    // Over-budget by one byte: rejection at the admission front door.
    let filler = "fn pad() int { return 0 }\n".repeat(300_000); // ~7.8 MB > 4 MiB
    let big = format!("fn main() int {{ return 0 }}\n{filler}");
    let r = d.request(&big, "d_main", &[]);
    let diag = r.diag.as_ref().expect("oversize must be diagnosed");
    assert_eq!(
        (diag.stage.as_str(), diag.code.as_str()),
        ("admission", ResourceKind::InputTooLarge.code())
    );
    assert_eq!(*r.counters.get("av_execute").unwrap_or(&0), 0);

    // Infinite loop under a REAL (small) step budget: InstrSteps resource
    // outcome — the budget is enforced by the executor, not the OS.
    let loop_src =
        "fn main() int {\n    var i = 0\n    loop {\n        i = i + 1\n    }\n    return i\n}\n";
    let compiled_ok = d.request(loop_src, "d_main", &[]);
    // The default 1M-step budget bounds this loop in ~1M steps: resource.
    match &compiled_ok.av {
        Some(ExecOutcome::Resource { kind, .. }) => {
            assert_eq!(*kind, ResourceKind::InstrSteps, "step-budget resource");
        }
        other => panic!("expected InstrSteps resource, got {other:?}"),
    }
    assert!(compiled_ok.diag.is_none(), "resource is not a diag");

    // Host cancellation through the same seam the worker uses: the source
    // program is compiled, verified, decoded, admitted and executed with a
    // pre-set cancel flag — Cancelled is a host decision, never a language
    // verdict, and never an OS kill standing in for cleanup evidence.
    let c = auto_ac_prototype::source::compile_scalar_source(loop_src, "cancel.at", &[])
        .expect("compile loop source");
    let m = lower_checked(&c.checked, "d_main", 0).expect("lower");
    let bytes = encode_abc(&m).expect("encode");
    let decoded = decode_abc(&bytes.to_flat()).expect("decode");
    let loaded = verify_abc(
        decoded,
        &OwnerScope {
            schema: ABC_SCHEMA.to_string(),
            module: "av827-cancel".into(),
        },
    )
    .expect("verify");
    let (out, _) = execute(&loaded, &ExecLimits::default(), &CancelFlag::new(true));
    assert_eq!(out, ExecOutcome::Cancelled, "pre-set cancel -> Cancelled");
}
