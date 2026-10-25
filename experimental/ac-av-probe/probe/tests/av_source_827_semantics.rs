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
/// channel from the AV guest outcome. The channel is REQUIRED (V-03
/// "native exit70独立匹配"): a missing tool FAILS the test — never a
/// silent pass (R4-F01).
static NATIVE_TOOL_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());

#[test]
#[cfg(windows)]
fn av_source_827_ap23_native_exit70_channel() {
    let _guard = NATIVE_TOOL_LOCK.lock().unwrap_or_else(|p| p.into_inner());
    let src = read_seed(AP23_ORIGINAL, AP23_SHA256);
    if let Err(e) = auto_ac_prototype::link::find_rust_lld() {
        panic!(
            "REQUIRED native oracle unavailable (rust-lld: {e:?}); \
             AP-23 trap/native-channel separation cannot pass without it"
        );
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

/// R4-F01 missing-tool control (AP-23 branch): the SUPPORTED
/// `AC741_RUST_LLD` override pointing at a nonexistent file makes tool
/// resolution ERROR without a silent fallback — exactly the branch the
/// required AP-23 native test above panics on.
#[test]
#[cfg(windows)]
fn av_source_827_native_missing_tool_is_red_ap23_control() {
    let _guard = NATIVE_TOOL_LOCK.lock().unwrap_or_else(|p| p.into_inner());
    let saved = std::env::var("AC741_RUST_LLD").ok();
    std::env::set_var("AC741_RUST_LLD", "plan827-absent-review-linker.exe");
    let resolved = auto_ac_prototype::link::find_rust_lld();
    match saved {
        Some(v) => std::env::set_var("AC741_RUST_LLD", v),
        None => std::env::remove_var("AC741_RUST_LLD"),
    }
    assert!(
        resolved.is_err(),
        "a broken override must make resolution fail — the branch the required AP-23 native test panics on"
    );
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

/// V-06 (R5 binding, thin source-entry replay): the consume gate on the
/// SOURCE path — a self-consistently corrupted ABC payload is rejected at
/// decode (before ANY dispatch), while a valid single-function FULL
/// module passes verify and executes. Bound at the inherited layers by
/// abc_semantics_tier/cache_tier/native_admission_tier (required members
/// of the admission scope); this replays the same invariant through this
/// plan's compile pipeline on the real AP-01 bytes.
#[test]
fn av_source_827_consume_gate_corrupt_abc_never_executes() {
    use auto_av_core::{decode_abc, encode_abc, verify_abc};
    let src = read_seed(AP01_ORIGINAL, AP01_SHA256);
    let compiled =
        auto_ac_prototype::source::compile_scalar_source(&src, "v06.at", &[]).expect("compile");
    let mir = lower_checked(&compiled.checked, "d_main", 0).expect("lower");
    let flat = encode_abc(&mir).expect("encode").to_flat();
    // Corrupt the payload interior (header intact) — a later re-encode is
    // impossible; every consumer must reject at decode.
    let mut corrupt = flat.clone();
    let mid = corrupt.len() / 2;
    corrupt[mid] ^= 0xff;
    corrupt[mid + 1] ^= 0xff;
    assert!(
        decode_abc(&corrupt).is_err(),
        "corrupted ABC bytes must fail decode (never executed)"
    );
    // Positive: the pristine bytes decode and a single-function FULL
    // module verifies and executes (the consume gate's positive control).
    let decoded = decode_abc(&flat).expect("pristine decode");
    let owner = OwnerScope {
        schema: ABC_SCHEMA.to_string(),
        module: "v06-one".into(),
    };
    let loaded = verify_abc(decoded, &owner).expect("verify");
    let (out, _) = execute(&loaded, &ExecLimits::default(), &CancelFlag::new(false));
    match out {
        ExecOutcome::Success { value, .. } => assert_eq!(value.as_i32(), Some(0)),
        other => panic!("AP-01 must execute: {other:?}"),
    }
    // One-function full module (main only): the minimal full-module shape
    // the gate must accept.
    let one = auto_ac_prototype::source::compile_scalar_source(
        "fn main() int { return 7 }\n",
        "one.at",
        &[],
    )
    .expect("one-fn compile");
    let one_mir = lower_checked(&one.checked, "d_main", 0).expect("one-fn lower");
    let one_dec =
        decode_abc(&encode_abc(&one_mir).expect("one-fn encode").to_flat()).expect("one-fn decode");
    let one_loaded = verify_abc(
        one_dec,
        &OwnerScope {
            schema: ABC_SCHEMA.to_string(),
            module: "v06-two".into(),
        },
    )
    .expect("one-fn verify");
    let (out, _) = execute(&one_loaded, &ExecLimits::default(), &CancelFlag::new(false));
    match out {
        ExecOutcome::Success { value, .. } => assert_eq!(value.as_i32(), Some(7)),
        other => panic!("one-fn module must execute: {other:?}"),
    }
}

/// V-08 (R5): revoking the CURRENT grant needed by cached code blocks
/// execution — the trace source compiled and cached under the trace grant
/// is REJECTED when the grant is absent on the next request (never a
/// stale dispatch of the cached trace-capable code).
#[test]
fn av_source_827_trace_grant_revocation_blocks_execution() {
    let src_path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../../crates/auto-lang/test/cases/ac-av/av/core/trace_order.at");
    let src = std::fs::read_to_string(&src_path).expect("trace seed");
    let root = std::env::temp_dir().join(format!("av827-revoke-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&root);
    let mut d = Driver::with_store(&root, "av827-revoke").expect("store");
    let granted = d.request(&src, "d_main", &["hir.test.trace.v1".to_string()]);
    assert!(granted.ok(), "granted run: {:?}", granted.diag);
    assert_eq!(granted.value_i32(), Some(1));
    // Same store, grant REVOKED: diagnosed rejection, zero dispatch.
    let revoked = d.request(&src, "d_main", &[]);
    let diag = revoked
        .diag
        .as_ref()
        .expect("revoked grant must reject, not execute cached code");
    assert_eq!(
        (diag.stage.as_str(), diag.code.as_str()),
        ("frontend", "source.undefined"),
        "the marks are undefined without the live grant"
    );
    assert_eq!(
        *revoked.counters.get("av_execute").unwrap_or(&0),
        0,
        "no dispatch of cached trace-capable code under a revoked grant"
    );
    let _ = std::fs::remove_dir_all(&root);
}

/// V-08 (R5): a position-only shift re-binds the trap span to the NEW
/// coordinates — the shifted run's trap span moves by exactly the prefix
/// length (a real coordinate change, not merely "spans non-empty").
#[test]
fn av_source_827_position_shift_rebinds_trap_span_coordinates() {
    let src = read_seed(AP23_ORIGINAL, AP23_SHA256);
    let prefix = "\n\n";
    let shifted = format!("{prefix}{src}");
    let mut d = Driver::ephemeral();
    let base = d.request(&src, "d_main", &[]);
    let shifted_run = d.request(&shifted, "d_main", &[]);
    let span_of = |r: &auto_av_probe::driver::RequestOutcome| match &r.av {
        Some(ExecOutcome::Trap { kind, span }) => {
            assert_eq!(*kind, TrapKind::DivByZero);
            span.clone().expect("trap carries a span")
        }
        other => panic!("expected the div-by-zero trap, got {other:?}"),
    };
    let s1 = span_of(&base);
    let s2 = span_of(&shifted_run);
    assert_eq!(
        s2.start as usize,
        s1.start as usize + prefix.len(),
        "the trap span start re-binds to the shifted coordinates (+{}): {:?} -> {:?}",
        prefix.len(),
        s1,
        s2
    );
    assert_eq!(
        s2.end - s2.start,
        s1.end - s1.start,
        "the span LENGTH is stable under a pure position shift"
    );
    assert_eq!(s2.func, s1.func, "the owning function is unchanged");
}

/// V-09 (R5): the input cap as an at/over-one PAIR — a source of EXACTLY
/// MAX_INPUT_BYTES passes admission, one byte more is rejected at
/// admission with the precise resource code (never a guest outcome).
#[test]
fn av_source_827_input_cap_at_and_over_one() {
    // Comment lines only ("//<x's>\n") so the padded source stays valid;
    // the final partial line is a comment too, landing EXACTLY on the cap.
    const CAP: usize = auto_av_core::contract::MAX_INPUT_BYTES;
    let mut at_cap = b"fn main() int { return 0 }\n".to_vec();
    let mut rem = CAP - at_cap.len();
    while rem >= 3 {
        // line = "//" + k x's + "\n", at most 14 bytes, at least 3.
        let take = rem.min(14);
        at_cap.extend_from_slice(b"//");
        at_cap.extend(std::iter::repeat(b'x').take(take - 3));
        at_cap.push(b'\n');
        rem -= take;
    }
    assert_eq!(rem, 0, "the construction consumes the remainder exactly");
    assert_eq!(
        at_cap.len(),
        CAP,
        "the at-cap source is exactly at the frozen boundary"
    );
    let mut d = Driver::ephemeral();
    let ok = d.request(std::str::from_utf8(&at_cap).unwrap(), "d_main", &[]);
    assert!(
        ok.ok(),
        "at-cap input passes the front door and compiles: {:?}",
        ok.diag
    );
    assert_eq!(ok.value_i32(), Some(0));
    let over = format!("{}x", std::str::from_utf8(&at_cap).unwrap());
    assert_eq!(over.len(), auto_av_core::contract::MAX_INPUT_BYTES + 1);
    let rejected = d.request(&over, "d_main", &[]);
    let diag = rejected
        .diag
        .as_ref()
        .expect("over-by-one must be rejected at admission");
    assert_eq!(
        (diag.stage.as_str(), diag.code.as_str()),
        ("admission", ResourceKind::InputTooLarge.code())
    );
    assert_eq!(
        *rejected.counters.get("av_execute").unwrap_or(&0),
        0,
        "rejection happens with zero dispatch"
    );
}
/// V-09/R6: actual frame-slot reserve failures, including failure after
/// earlier frames and trace events exist. No global allocator is replaced.
#[test]
fn av_source_827_nth_frame_allocation_failure_cleans_owned_slots() {
    use auto_av_core::frame_allocation::FrameAllocationProbe;
    let src = "fn leaf(x int) int {\n    let a = mark_a()\n    return x + 1\n}\nfn mid(x int) int {\n    let b = mark_b()\n    return leaf(x)\n}\nfn main() int {\n    let a = mark_a()\n    return mid(6)\n}\n";
    let c = auto_ac_prototype::source::compile_scalar_source(
        src,
        "allocation.at",
        &[TRACE_CAPABILITY.to_string()],
    )
    .unwrap();
    let m = lower_checked(&c.checked, "d_main", 0).unwrap();
    let bytes = encode_abc(&m).unwrap();
    let loaded = verify_abc(
        decode_abc(&bytes.to_flat()).unwrap(),
        &OwnerScope {
            schema: ABC_SCHEMA.into(),
            module: "av827-allocation".into(),
        },
    )
    .unwrap();
    for n in 1..=3 {
        let probe = FrameAllocationProbe::new(Some(n));
        let (out, trace) =
            probe.run(|| execute(&loaded, &ExecLimits::default(), &CancelFlag::new(false)));
        assert!(
            matches!(
                out,
                ExecOutcome::Resource {
                    kind: ResourceKind::SlotBytes,
                    ..
                }
            ),
            "failure {n}: {out:?}"
        );
        let s = probe.snapshot();
        eprintln!(
            "V09 allocation failure n={n}: {s:?}, prefix={:?}",
            trace.events
        );
        assert_eq!(s.attempts, n);
        assert_eq!(s.successful, n - 1);
        assert_eq!(s.released, s.successful);
        assert_eq!(
            s.live_bytes, 0,
            "all previously allocated buffers actually dropped"
        );
        assert_eq!(
            trace.events.len() as u64,
            n - 1,
            "retain legal prefix; failed callee never dispatches"
        );
        assert_eq!(s.peak_bytes > 0, n > 1);
    }
    // At n=4 no failure is reached. Correct source result/trace and every
    // buffer is released on successful returns, not only error unwinding.
    let probe = FrameAllocationProbe::new(Some(4));
    let (out, trace) =
        probe.run(|| execute(&loaded, &ExecLimits::default(), &CancelFlag::new(false)));
    assert!(matches!(
        out,
        ExecOutcome::Success {
            value: auto_av_core::contract::Value::I32(7),
            ..
        }
    ));
    assert_eq!(trace.events.len(), 3);
    let s = probe.snapshot();
    assert_eq!(
        (s.attempts, s.successful, s.released, s.live_bytes),
        (3, 3, 3, 0)
    );
    for limits in [
        ExecLimits {
            max_frames: 0,
            ..ExecLimits::default()
        },
        ExecLimits {
            max_slot_bytes: 0,
            ..ExecLimits::default()
        },
    ] {
        let probe = FrameAllocationProbe::new(Some(1));
        let (out, trace) = probe.run(|| execute(&loaded, &limits, &CancelFlag::new(false)));
        assert!(matches!(out, ExecOutcome::Resource { .. }));
        assert!(trace.events.is_empty());
        assert_eq!(
            probe.snapshot(),
            Default::default(),
            "budget rejection precedes reserve attempt"
        );
    }
    // Root succeeds, callee budget rejects before a second reserve.
    let probe = FrameAllocationProbe::new(Some(2));
    let (out, trace) = probe.run(|| {
        execute(
            &loaded,
            &ExecLimits {
                max_frames: 1,
                ..ExecLimits::default()
            },
            &CancelFlag::new(false),
        )
    });
    assert!(matches!(
        out,
        ExecOutcome::Resource {
            kind: ResourceKind::Frames,
            ..
        }
    ));
    assert_eq!(trace.events.len(), 1);
    let s = probe.snapshot();
    assert_eq!(
        (s.attempts, s.successful, s.released, s.live_bytes),
        (1, 1, 1, 0)
    );
    // The fault scope does not leak to the next uninstrumented run.
    assert!(matches!(
        execute(&loaded, &ExecLimits::default(), &CancelFlag::new(false)).0,
        ExecOutcome::Success { .. }
    ));
}

/// The same allocation ownership survives pauses and releases on all exits.
#[test]
fn av_source_827_frame_allocation_pause_cancel_trap_cleanup() {
    use auto_av_core::av::{execute_program_steps, resume_program, SteppedOutcome};
    use auto_av_core::frame_allocation::FrameAllocationProbe;
    use auto_av_probe::session::warmup;
    let call_source =
        "fn helper(x int) int {\n    return x + 1\n}\nfn main() int {\n    return helper(6)\n}\n";
    let call_session = warmup(call_source, "d_main", &[]).unwrap();
    let failure_probe = FrameAllocationProbe::new(Some(2));
    let parked = failure_probe.run(|| {
        execute_program_steps(
            &call_session.current(),
            &ExecLimits::default(),
            &CancelFlag::new(false),
            1,
        )
    });
    let parked = match parked {
        SteppedOutcome::Paused(p) => p,
        _ => panic!("must pause before call"),
    };
    let finished = failure_probe.run(|| resume_program(parked, &CancelFlag::new(false), None));
    assert!(matches!(
        finished,
        SteppedOutcome::Finished(
            ExecOutcome::Resource {
                kind: ResourceKind::SlotBytes,
                ..
            },
            _
        )
    ));
    let stats = failure_probe.snapshot();
    assert_eq!(
        (
            stats.attempts,
            stats.successful,
            stats.released,
            stats.live_bytes
        ),
        (2, 1, 1, 0),
        "nth failure across pause/resume releases old frame"
    );
    let src =
        "fn main() int {\n    var i = 0\n    loop {\n        i = i + 1\n    }\n    return i\n}\n";
    let mut session = warmup(src, "d_main", &[]).unwrap();
    let snapshot = session.current();
    let weak = std::sync::Arc::downgrade(&snapshot);
    let probe = FrameAllocationProbe::new(None);
    let paused = probe.run(|| {
        execute_program_steps(
            &snapshot,
            &ExecLimits::default(),
            &CancelFlag::new(false),
            1,
        )
    });
    assert_eq!(probe.snapshot().successful, 1);
    assert!(probe.snapshot().live_bytes > 0);
    assert_eq!(probe.snapshot().released, 0);
    session.close();
    drop(snapshot);
    assert!(weak.upgrade().is_some(), "paused chain pins the snapshot");
    let paused = match paused {
        SteppedOutcome::Paused(p) => p,
        _ => panic!("must pause"),
    };
    let done = resume_program(paused, &CancelFlag::new(true), None);
    assert!(matches!(
        done,
        SteppedOutcome::Finished(ExecOutcome::Cancelled, _)
    ));
    assert_eq!(probe.snapshot().released, 1);
    assert_eq!(probe.snapshot().live_bytes, 0);
    assert!(
        weak.upgrade().is_none(),
        "completed cancelled chain releases its snapshot pin"
    );

    // Dropping an unfinished chain also drops owned slot buffers.
    let session = warmup(src, "d_main", &[]).unwrap();
    let probe = FrameAllocationProbe::new(None);
    let paused = probe.run(|| {
        execute_program_steps(
            &session.current(),
            &ExecLimits::default(),
            &CancelFlag::new(false),
            1,
        )
    });
    assert!(probe.snapshot().live_bytes > 0);
    drop(paused);
    assert_eq!(probe.snapshot().live_bytes, 0);
    assert_eq!(probe.snapshot().successful, probe.snapshot().released);

    for (source, limits, trap) in [
        (
            "fn main() int {\n    return 1 / 0\n}\n",
            ExecLimits::default(),
            true,
        ),
        (
            src,
            ExecLimits {
                max_steps: 10,
                ..ExecLimits::default()
            },
            false,
        ),
    ] {
        let s = warmup(source, "d_main", &[]).unwrap();
        let probe = FrameAllocationProbe::new(None);
        let (out, _) = probe.run(|| {
            auto_av_core::av::execute_program(&s.current(), &limits, &CancelFlag::new(false))
        });
        if trap {
            assert!(matches!(out, ExecOutcome::Trap { .. }));
        } else {
            assert!(matches!(
                out,
                ExecOutcome::Resource {
                    kind: ResourceKind::InstrSteps,
                    ..
                }
            ));
        }
        assert!(probe.snapshot().successful > 0);
        assert_eq!(probe.snapshot().live_bytes, 0);
        assert_eq!(probe.snapshot().successful, probe.snapshot().released);
    }
}
