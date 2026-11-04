//! PLAN-931 owned view-consumed temporaries must survive until their REAL
//! consumer (D-925-2 Rust-side closure): probe/V-case file (T-02 seeds the
//! reds; T-06 freezes the full set). True chain: source → checked HIR →
//! view → Rust lowering → REAL rustc → real run → parent-side oracle with
//! the runtime event ledger (`A2X_RT_EVENTS=1`) and the frozen fault hook.

use ace_a2x_core::target::{Family, FileSet, FilesetRequest, TargetDescriptor, TargetProvider};
use ace_a2x_core::{A2xReject, FnFacts, SemanticView};
use ace_a2x_rust::{lower_program, RustTargetProvider};
use ace_a2x_store::A2xSession;
use ace_core::source::{compile_parts_with, SourceProfile};
use ace_core::text_resources::seed_param_exclusions;
use ace_core::verify::verify_text_function;

use crate::rt_common::{assert_value, compile_case, run_chain, Prof, RUNTIME_TEXT_CAP};

/// V-01 first case: the byte-exact 925 Wrap source (a2c_rt_restore::SRC) —
/// two owned literals as view args of `mk`; independent oracle span=5
/// ("ab" + "cde" hand-audited, C925 peer only).
const WRAP: &str = "type Wrap {\n    head Str\n    tail Str\n    static fn mk(h Str, t Str) Wrap { return Wrap { head: h.clone(), tail: t.clone() } }\n    fn span() int { return .head.len() + .tail.len() }\n}\nfn main() int {\n    let w Wrap = Wrap.mk(\"ab\", \"cde\")\n    return w.span()\n}";

/// P1 value red: head/tail must be the full original bytes.
#[test]
fn p931_wrap_span_value() {
    let (out, _err) = run_chain("P931_WRAP", WRAP, Prof::TextRecord, "probe", &[], &[]);
    assert_value("P931_WRAP", &out, 5);
}

/// P1 event red: the first literal's free must come AFTER the `mk` call
/// consumed it. Current full-drain shape frees it at the second literal's
/// pin — the ledger dump (eprintln) captures the actual order.
#[test]
fn p931_wrap_event_order() {
    let (out, err) = run_chain(
        "P931_WRAP",
        WRAP,
        Prof::TextRecord,
        "probe-events",
        &[],
        &[("A2X_RT_EVENTS", "1")],
    );
    eprintln!("--- P931_WRAP events ---\n{err}");
    assert_value("P931_WRAP", &out, 5);
}

/// P2 statement-root red: a pinned view operand (the clone temp) must be
/// freed after the consuming `let n = ..` statement, not before it.
#[test]
fn p931_stmt_root_len_of_clone() {
    let src = r#"fn main() int {
    let a Str = "ab"
    let n = a.clone().len()
    return n
}"#;
    let (out, _err) = run_chain("P931_LEN", src, Prof::Text, "probe", &[], &[]);
    assert_value("P931_LEN", &out, 2);
}

/// Short-circuit isolation: an untaken RHS side allocates and frees
/// NOTHING (its pins live inside the side block); the value oracle is
/// hand-audited (false side → else branch).
#[test]
fn p931_short_circuit_untaken_side_zero_alloc() {
    let src = r#"fn main() int {
    let b Str = "cde"
    let ok bool = false && b.clone() == b
    if ok { return 1 } else { return 7 }
}"#;
    let (out, err) = run_chain(
        "P931_SC_UNTAKEN",
        src,
        Prof::Text,
        "probe-events",
        &[],
        &[("A2X_RT_EVENTS", "1")],
    );
    assert_value("P931_SC_UNTAKEN", &out, 7);
    let events: Vec<&str> = err.lines().filter(|l| l.starts_with("rt:")).collect();
    // exactly b's own lifecycle: the untaken side's clone temp never runs
    assert_eq!(
        events,
        vec!["rt:alloc:3", "rt:free:3"],
        "untaken side must not allocate/free beyond b's own lifecycle: {events:?}"
    );
}

/// Outer pending across a short-circuit: the LHS text-compare's pinned
/// operand temp survives INTO and past the RHS side block (the side frees
/// only its own temps inside itself), then frees after its real consumer
/// (the `let ok` statement). Hand-audited ledger.
#[test]
fn p931_short_circuit_outer_pending_survives() {
    let src = r#"fn main() int {
    let a Str = "ab"
    let b Str = "cde"
    let ok bool = a.clone() == a && b.clone() == b
    if ok { return 5 } else { return 0 }
}"#;
    let (out, err) = run_chain(
        "P931_SC_OUTER",
        src,
        Prof::Text,
        "probe-events",
        &[],
        &[("A2X_RT_EVENTS", "1")],
    );
    assert_value("P931_SC_OUTER", &out, 5);
    let events: Vec<&str> = err.lines().filter(|l| l.starts_with("rt:")).collect();
    assert_eq!(
        events,
        vec![
            "rt:alloc:2", // a = "ab" (owned local)
            "rt:alloc:3", // b = "cde" (owned local)
            "rt:alloc:2", // t0 = a.clone() (LHS compare operand)
            "rt:alloc:3", // side: t2 = b.clone() (taken side)
            "rt:free:3",  // side frees ITS OWN temp inside the block
            "rt:free:2",  // t0 frees after its real consumer (`let ok`)
            "rt:free:3",  // exit: b
            "rt:free:2",  // exit: a
        ],
        "outer pending must survive the side; the side frees only its own"
    );
}

/// Failure atomicity (frozen trap semantics): faulting the SECOND literal
/// allocation (t1) in the Wrap main traps Resource with NO free of the
/// already-allocated t0 — frees never run before the failed allocation
/// (no partial cleanup, no early release).
#[test]
fn p931_fault_second_arg_no_free() {
    let (out, err) = run_chain(
        "P931_WRAP",
        WRAP,
        Prof::TextRecord,
        "probe-fault2",
        &[],
        &[("A2X_RT_FAULT_ALLOC", "2"), ("A2X_RT_EVENTS", "1")],
    );
    assert_eq!(out, "trap:Resource\n");
    let frees: Vec<&str> = err.lines().filter(|l| l.starts_with("rt:free")).collect();
    assert!(
        frees.is_empty(),
        "a failed allocation must trigger no frees: {err}"
    );
}

/// External fragment modes (V-08 seed): a fragment body calling
/// move/mut-param helpers renders byte-identically to the whole-module
/// emission — the VERIFIED external modes (MoveOwn/MutBorrow) propagate,
/// never a default View; the value oracle is hand-audited (2*10+2).
#[test]
fn p931_external_modes_fragment_equivalence() {
    let src = "fn own_len(move s Str) int { return s.len() }\nfn mut_peek(mut s Str) int { return s.len() }\nfn main() int {\n    let a Str = \"ab\"\n    var b Str = \"xy\"\n    let n1 int = own_len(a.move)\n    let n2 int = mut_peek(b)\n    return n1 * 10 + n2\n}";
    // and the whole chain runs to the hand-audited oracle (move args move;
    // mut args borrow exclusively; view args pass content)
    let (out, _err) = run_chain("P931_MODES", src, Prof::TextRecord, "probe", &[], &[]);
    assert_value("P931_MODES", &out, 22);

    // whole-module reference
    let whole = compile_case("P931_MODES", src, Prof::TextRecord);
    let whole_view = SemanticView::from_checked(&whole.checked).expect("whole view");
    let whole_prog = lower_program(&whole_view, "main").expect("whole lower");
    let whole_text = ace_a2x_rust::print_program(&whole_prog);

    // single-body credential for main: take/peek become signature-only
    // externals whose VERIFIED modes the lowering must consume
    let parts = compile_parts_with(
        src,
        "P931_MODES.at",
        &[RUNTIME_TEXT_CAP.to_string()],
        SourceProfile::TextRecord,
    )
    .expect("parts");
    let main_def = parts.bundle.modules[0]
        .defs
        .iter()
        .position(|d| d.id_text == "d_main")
        .expect("main def");
    let excl = seed_param_exclusions(&parts.bundle);
    let cred = verify_text_function(&parts.bundle, main_def, &excl)
        .unwrap_or_else(|ds| panic!("P931_MODES credential: {ds:?}"));
    let frag_view = SemanticView::from_parts(&parts, &cred).expect("fragment view");
    let frag_prog = lower_program(&frag_view, "main").expect("fragment lower");
    let frag_text = ace_a2x_rust::print_program(&frag_prog);
    // compare ONLY main's function bytes: the fragment is single-body by
    // design (the whole emission also carries own_len/mut_peek bodies)
    let fn_bytes = |text: &str| -> String {
        let start = text.find("pub fn fn_main").expect("fn_main rendered");
        let end = text[start..]
            .find("}\n")
            .map(|i| start + i + 2)
            .expect("fn end");
        text[start..end].to_string()
    };
    assert_eq!(
        fn_bytes(&whole_text),
        fn_bytes(&frag_text),
        "fragment must render fn_main identically: verified external modes propagate"
    );
}

/// V-03 move responsibility: an argument moved into a `move Str` callee is
/// cleaned exactly once — BY THE CALLEE at its exit; the moved-out source
/// is never re-freed by the caller's scope (alloc/free counts balance at
/// 1/1, hand-audited).
#[test]
fn p931_move_transferred_freed_once_by_callee() {
    let src = "fn eat(move s Str) int { return s.len() }\nfn main() int {\n    let a Str = \"abcd\"\n    let n int = eat(a.move)\n    return n\n}";
    let (out, err) = run_chain(
        "P931_MOVE",
        src,
        Prof::TextRecord,
        "probe-events",
        &[],
        &[("A2X_RT_EVENTS", "1")],
    );
    assert_value("P931_MOVE", &out, 4);
    let events: Vec<&str> = err.lines().filter(|l| l.starts_with("rt:")).collect();
    assert_eq!(
        events,
        vec!["rt:alloc:4", "rt:free:4"],
        "moved value frees exactly once (callee exit), never at the caller"
    );
}

/// V-03 return responsibility: an owned value RETURNED transfers to the
/// caller — the producing function's exit does not free it; the caller's
/// scope owns and frees it (1/1 balance, hand-audited).
#[test]
fn p931_return_owned_freed_by_caller_scope() {
    let src = "fn make() Str { let s Str = \"abc\"\n    return s }\nfn main() int {\n    let b Str = make()\n    return b.len()\n}";
    let (out, err) = run_chain(
        "P931_RETURN",
        src,
        Prof::TextRecord,
        "probe-events",
        &[],
        &[("A2X_RT_EVENTS", "1")],
    );
    assert_value("P931_RETURN", &out, 3);
    let events: Vec<&str> = err.lines().filter(|l| l.starts_with("rt:")).collect();
    assert_eq!(
        events,
        vec!["rt:alloc:3", "rt:free:3"],
        "returned owned value frees exactly once at the CALLER's exit"
    );
}

/// V-03 each-exit cleanup: a taken branch's replacement frees the OLD
/// buffer only after the RHS materialized; every exit frees the current
/// buffer exactly once (hand-audited ledger).
#[test]
fn p931_branch_exit_replacement_order() {
    let src = "fn main() int {\n    var x Str = \"ab\"\n    if 1 < 2 { x = x + \"cd\" } else { x = x + \"ef\" }\n    return x.len()\n}";
    let (out, err) = run_chain(
        "P931_BRANCH",
        src,
        Prof::TextRecord,
        "probe-events",
        &[],
        &[("A2X_RT_EVENTS", "1")],
    );
    assert_value("P931_BRANCH", &out, 4);
    let events: Vec<&str> = err.lines().filter(|l| l.starts_with("rt:")).collect();
    assert_eq!(
        events,
        vec![
            "rt:alloc:2", // x = "ab"
            "rt:alloc:4", // RHS materializes FIRST (taken branch)
            "rt:free:2",  // old x frees only after the RHS
            "rt:free:4",  // exit frees the CURRENT buffer once
        ],
        "replacement frees after RHS; each exit frees exactly once"
    );
}

/// V-04 live-byte peak (an independent observation over the event ledger,
/// NOT a runtime quota): per-consumer frees keep the simultaneous view
/// temps at one consumer's footprint — deferring every temp to function
/// end would double the observed peak.
#[test]
fn p931_live_peak_bounded_by_per_consumer_frees() {
    let src = "fn pair(a Str, b Str) int { return a.len() + b.len() }\nfn main() int {\n    let n1 int = pair(\"aaaa\", \"bbbb\")\n    let n2 int = pair(\"aaaa\", \"bbbb\")\n    return n1 + n2\n}";
    let (out, err) = run_chain(
        "P931_PEAK",
        src,
        Prof::TextRecord,
        "probe-events",
        &[],
        &[("A2X_RT_EVENTS", "1")],
    );
    assert_value("P931_PEAK", &out, 16);
    // independent ledger walk: live bytes after each event
    let mut live = 0i64;
    let mut peak = 0i64;
    let events: Vec<&str> = err.lines().filter(|l| l.starts_with("rt:")).collect();
    for e in &events {
        let n: i64 = e.rsplit(':').next().unwrap().parse().unwrap();
        if e.starts_with("rt:alloc") {
            live += n;
            peak = peak.max(live);
        } else {
            live -= n;
        }
    }
    assert_eq!(live, 0, "no leak: ledger balances to zero");
    // per-consumer footprint: 4+4 = 8 view-temp bytes alive together
    assert_eq!(
        peak, 8,
        "peak must equal ONE consumer's operand set; deferring temps to function end would observe 16"
    );
}

/// A provider minting under the PRE-931 print version (identical to
/// `RustTargetProvider` except the `print_version` key field) — the
/// stand-in for any store written by an older build.
struct OldPrintProvider(RustTargetProvider);

impl TargetProvider for OldPrintProvider {
    fn descriptor(&self) -> TargetDescriptor {
        self.0.descriptor()
    }
    fn fragment_key_inputs(&self, runtime_sha256: &str) -> Vec<(String, String)> {
        self.0
            .fragment_key_inputs(runtime_sha256)
            .into_iter()
            .map(|(k, v)| {
                if k == "print_version" {
                    (k, "a2x-print/1".to_string())
                } else {
                    (k, v)
                }
            })
            .collect()
    }
    fn fragment_layer_id(&self, family: Family) -> Result<&'static str, A2xReject> {
        self.0.fragment_layer_id(family)
    }
    fn runtime_source(&self, view: &SemanticView<'_>, family: Family) -> Result<String, A2xReject> {
        self.0.runtime_source(view, family)
    }
    fn produce_fragments(
        &self,
        view: &SemanticView<'_>,
        fns: &[&FnFacts<'_>],
        family: Family,
    ) -> Result<Vec<String>, A2xReject> {
        self.0.produce_fragments(view, fns, family)
    }
    fn capture_prelude_seed(
        &self,
        view: &SemanticView<'_>,
        family: Family,
    ) -> Result<Option<String>, A2xReject> {
        self.0.capture_prelude_seed(view, family)
    }
    fn assemble_fileset(&self, req: FilesetRequest<'_>) -> Result<FileSet, A2xReject> {
        self.0.assemble_fileset(req)
    }
}

/// V-05 cache/version axis: fragments minted under the OLD print version
/// can NEVER hit after the PLAN-931 bump — the same source re-opens with
/// fresh production (miss + re-lower), assembles to the CORRECT bytes and
/// value through real rustc, and a THIRD open over the migrated store is
/// a full hit (a one-time migration, then stable).
#[test]
fn p931_old_print_version_fragments_never_hit() {
    let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../target/a2x-stores/p931-version");
    let _ = std::fs::remove_dir_all(&root);
    std::fs::create_dir_all(&root).unwrap();
    let caps = [RUNTIME_TEXT_CAP.to_string()];

    // generation 1: mint every fragment under a2x-print/1
    {
        let mut s = A2xSession::open(
            &root,
            "a2x-probe",
            "probe",
            128 * 1024 * 1024,
            OldPrintProvider(RustTargetProvider::v1()),
        )
        .unwrap();
        let asm = s
            .cold_assemble_profile(WRAP, SourceProfile::TextRecord, &caps, "main", false)
            .unwrap();
        assert_eq!(s.stats().lower, 3, "mk/span/main minted under print/1");
        assert_eq!(run_generated(&asm, "p931-v1"), "value:5");
    }

    // generation 2: the REAL provider (a2x-print/2) over the same store —
    // every old-version fragment MISSES and re-lowers; correct value
    let mut s2 = A2xSession::open(
        &root,
        "a2x-probe",
        "probe",
        128 * 1024 * 1024,
        RustTargetProvider::v1(),
    )
    .unwrap();
    let asm2 = s2
        .cold_assemble_profile(WRAP, SourceProfile::TextRecord, &caps, "main", false)
        .unwrap();
    assert_eq!(
        s2.stats().lower,
        3,
        "print/1 fragments must never hit under print/2"
    );
    assert_eq!(run_generated(&asm2, "p931-v2"), "value:5");

    // generation 3: same store, same provider — a full hit now
    let mut s3 = A2xSession::open(
        &root,
        "a2x-probe",
        "probe",
        128 * 1024 * 1024,
        RustTargetProvider::v1(),
    )
    .unwrap();
    let _ = s3
        .cold_assemble_profile(WRAP, SourceProfile::TextRecord, &caps, "main", false)
        .unwrap();
    assert_eq!(s3.stats().lower, 0, "one-time migration, then stable hits");
}

/// V-05 Rust session lifecycle on the EXISTING restart/edit surface (the
/// pure source-bound restore→first-edit seam is 929's deliverable — its
/// absence is recorded as AC-05 pending, not waived): cold → warm body
/// edit of `span` re-lowers ONLY span and keeps the corrected mk fragment
/// byte-identical; a broken edit is a NAMED rejection leaving the session
/// usable; the fix edit assembles to the hand-audited value.
#[test]
fn p931_rust_session_warm_edit_lifecycle() {
    let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../target/a2x-stores/p931-lifecycle");
    let _ = std::fs::remove_dir_all(&root);
    std::fs::create_dir_all(&root).unwrap();
    let caps = [RUNTIME_TEXT_CAP.to_string()];
    let mut s = A2xSession::open(
        &root,
        "a2x-probe",
        "probe",
        128 * 1024 * 1024,
        RustTargetProvider::v1(),
    )
    .unwrap();
    let cold = s
        .cold_assemble_profile(WRAP, SourceProfile::TextRecord, &caps, "main", false)
        .unwrap();
    assert_eq!(s.stats().lower, 3);
    assert_eq!(run_generated(&cold, "p931-life-cold"), "value:5");
    let cold_mk = fragment_of(generated_text(&cold), "mt_Wrap_mk");

    // warm body edit of span ONLY (the Rust record-text fragment contract:
    // the edited METHOD's declaration text, owner-qualified)
    let span_b = "fn span() int { return .head.len() * 10 + .tail.len() }";
    let before = s.stats().clone();
    let warm = s
        .warm_edit_method("Wrap", "span", span_b, "main", false)
        .unwrap();
    assert_eq!(run_generated(&warm, "p931-life-warm"), "value:23"); // 2*10+3
    let after = s.stats();
    assert_eq!(after.lower, before.lower + 1, "ONLY span re-lowered");
    // the corrected mk fragment survives the edit byte-identically
    assert_eq!(fragment_of(generated_text(&warm), "mt_Wrap_mk"), cold_mk);

    // a broken edit is a NAMED rejection (session state intact)
    let bad = "fn span() int { return .head.len() + unknown_name }";
    let err = s
        .warm_edit_method("Wrap", "span", bad, "main", false)
        .unwrap_err();
    assert!(
        format!("{err:?}").contains("source.undefined"),
        "broken edit must be a named undefined-name rejection: {err:?}"
    );
    // and the fix edit (same span*10 shape) still assembles to its value
    let fixed = s
        .warm_edit_method("Wrap", "span", span_b, "main", false)
        .unwrap();
    assert_eq!(run_generated(&fixed, "p931-life-fixed"), "value:23");
}

/// Compile + run one session `Assembly` through real rustc; returns stdout.
fn run_generated(asm: &ace_a2x_store::Assembly, tag: &str) -> String {
    let dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../target/a2x-out")
        .join(tag);
    std::fs::create_dir_all(&dir).unwrap();
    for (path, content) in &asm.files {
        std::fs::write(dir.join(path), content).unwrap();
    }
    let exe = dir.join("a2x_case.exe");
    let mut compiler = std::process::Command::new(crate::rt_common::rustc());
    compiler
        .args(["--edition", "2021", "-o"])
        .arg(&exe)
        .arg("main.rs")
        .current_dir(&dir);
    let out = auto_tool_execution::run_command_capture(
        compiler,
        auto_tool_execution::TreeScope::ProcessTree,
        None,
    )
    .expect("spawn rustc");
    assert_eq!(
        out.kind,
        auto_tool_execution::ExitKind::Success,
        "rustc failed for {tag}"
    );
    let mut run = std::process::Command::new(&exe);
    run.current_dir(&dir);
    let run = auto_tool_execution::run_command_capture(
        run,
        auto_tool_execution::TreeScope::ProcessTree,
        None,
    )
    .expect("run exe");
    String::from_utf8_lossy(run.stdout.as_ref().expect("stdout"))
        .trim()
        .to_string()
}

fn generated_text(asm: &ace_a2x_store::Assembly) -> &str {
    &asm.files
        .iter()
        .find(|(p, _)| p == "generated.rs")
        .unwrap()
        .1
}

fn fragment_of(generated: &str, marker: &str) -> String {
    let start = generated.find(marker).expect("fragment marker present");
    let from = generated[start..].find("pub fn").expect("pub fn") + start;
    let end = generated[from..].find("}\n").expect("fragment end") + from + 2;
    generated[from..end].to_string()
}
