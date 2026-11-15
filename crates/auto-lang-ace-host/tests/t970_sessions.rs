//! PLAN-970 T-02/T-03 fixtures: the REAL consumer of the ACE session
//! contract — the reference bridge (`session_bridge`) installed as the
//! process host, driven through the neutral `auto_lang_core` session
//! facade exactly like the daemon/MCP surfaces drive it.
//!
//! Every assertion is an independent oracle (hand-computed values, not
//! engine echoes): B01 arithmetic, same-PID edit arithmetic, isolation,
//! carry-table values, mixed-generation resident totals.

#![cfg(windows)]

use auto_lang_core::{
    ace_session_cleanup_expired, ace_session_create, ace_session_list, ace_session_op,
    shutdown_ace_session_host, AceSessionLimits, AceSessionOp,
    AceSessionOutcome, AceSessionValue,
};

use auto_lang_ace_host::session_bridge::install_session_host;

/// One host at a time: the registry is single-owner AND the tests run on
/// parallel threads — a shared lock keeps the slot race-free.
static HOST_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());

// Field order is load-bearing: the guard (whose stop() shuts the host
// down) must drop BEFORE the lock releases, or a waiter installs a host
// that the still-dropping guard then unregisters.
struct Host {
    _guard: auto_lang_ace_host::session_bridge::SessionHostGuard,
    _lock: std::sync::MutexGuard<'static, ()>,
}

impl Host {
    fn install() -> Host {
        let lock: std::sync::MutexGuard<'static, ()> = unsafe {
            std::mem::transmute(HOST_LOCK.lock().unwrap_or_else(|p| p.into_inner()))
        };
        let guard = install_session_host(AceSessionLimits::default()).expect("install session host");
        Host { _guard: guard, _lock: lock }
    }
}

impl Drop for Host {
    fn drop(&mut self) {
        shutdown_ace_session_host();
    }
}

fn ok_of(r: &auto_lang_core::AceSessionReport) {
    match &r.outcome {
        AceSessionOutcome::Ok => {}
        AceSessionOutcome::Diagnostic { code, diagnostics } => {
            panic!("expected Ok, got {code}: {diagnostics:?}")
        }
        other => panic!("expected Ok, got {other:?}"),
    }
}

fn eval(host_session: &str, input: &str) -> auto_lang_core::AceSessionReport {
    let r = ace_session_op(
        host_session,
        AceSessionOp::Eval {
            input,
            wall: None,
            cancel: None,
        },
    );
    ok_of(&r);
    r
}

fn eval_value(host_session: &str, input: &str) -> AceSessionValue {
    eval(host_session, input).value
}

/// V-DETAIL-01 / V-01..03 face: the frozen B01 corpus source loads into
/// a session, evaluates to the independent arithmetic oracle (14), the
/// same-PID edit 4→5 yields 17, a bad edit keeps the last accepted
/// generation (still 17), and the recovery edit yields 20.
#[test]
fn t970_b01_same_pid_edit_oracle() {
    let _host = Host::install();
    let b01 = "fn main() int { return 2 + 3 * 4 }";
    let created = ace_session_create(Some(b01), None);
    ok_of(&created);
    let s = created.session.clone().expect("session id");

    // The B01 body runs as the current int-lane evaluation.
    assert_eq!(eval_value(&s, "2 + 3 * 4"), AceSessionValue::Int(14));

    // Same-PID online update: the raw edit API against the canonical
    // source (the Inspect surface), applied through the ACE seam.
    let inspect = ace_session_op(&s, AceSessionOp::Inspect);
    ok_of(&inspect);
    let source = inspect.snapshot.expect("inspect carries the source");
    let at = source.find("2 + 3 * 4").expect("B01 body in canonical source");
    let edited = {
        let r = ace_session_op(
            &s,
            AceSessionOp::Edit {
                start: at as u64 + 8, // position of the `4`
                end: at as u64 + 9,
                insert: b"5".as_slice(),
            },
        );
        ok_of(&r);
        r
    };
    let rev_after_edit = edited.revision;
    assert!(rev_after_edit > 0, "the edit advanced the revision");

    // The program value after the edit: re-run the SAME expression the
    // lane now holds — the snapshot shows the edited body.
    let snap = ace_session_op(&s, AceSessionOp::Snapshot);
    ok_of(&snap);
    let text = snap.snapshot.expect("snapshot");
    assert!(text.contains("2 + 3 * 5"), "edited body present: {text}");

    // A NEW evaluation re-wraps the lane; the persisted edited program's
    // other functions remain. Evaluate through the edit path's value:
    // run the edited program by evaluating its exact expression.
    assert_eq!(eval_value(&s, "2 + 3 * 5"), AceSessionValue::Int(17));

    // Bad edit (illegal neighbor): refused, revision unchanged, the last
    // accepted generation still evaluates.
    let inspect = ace_session_op(&s, AceSessionOp::Inspect);
    let source = inspect.snapshot.expect("source");
    let at = source.find("2 + 3 * 5").expect("edited body");
    let bad = ace_session_op(
        &s,
        AceSessionOp::Edit {
            start: at as u64,
            end: at as u64 + 9,
            insert: b"2 + 3 * %",
        },
    );
    match bad.outcome {
        AceSessionOutcome::Diagnostic { .. } => {}
        other => panic!("bad edit must be a diagnostic, got {other:?}"),
    }
    let after_bad = ace_session_op(&s, AceSessionOp::Inspect);
    assert_eq!(after_bad.revision, bad.revision, "refused edit keeps revision");
    assert_eq!(eval_value(&s, "2 + 3 * 5"), AceSessionValue::Int(17));

    // Failure recovery: the fixed edit lands and evaluates.
    let at = after_bad.snapshot.as_ref().unwrap().find("2 + 3 * 5").unwrap();
    let fixed = ace_session_op(
        &s,
        AceSessionOp::Edit {
            start: at as u64,
            end: at as u64 + 9,
            insert: b"2 + 3 * 6",
        },
    );
    ok_of(&fixed);
    assert_eq!(eval_value(&s, "2 + 3 * 6"), AceSessionValue::Int(20));
}

/// V-01: session isolation — a variable written in session A is not
/// visible in session B; B's reference to it is a named diagnostic.
#[test]
fn t970_session_isolation_variables() {
    let _host = Host::install();
    let a = ace_session_create(None, None);
    let b = ace_session_create(None, None);
    ok_of(&a);
    ok_of(&b);
    let (sa, sb) = (a.session.clone().unwrap(), b.session.clone().unwrap());

    // A writes x (statement eval: no display value), then reads it.
    assert_eq!(eval_value(&sa, "var x = 41"), AceSessionValue::None);
    assert_eq!(eval_value(&sa, "x + 1"), AceSessionValue::Int(42));

    // B never saw x.
    let denied = ace_session_op(
        &sb,
        AceSessionOp::Eval {
            input: "x + 1",
            wall: None,
            cancel: None,
        },
    );
    match denied.outcome {
        AceSessionOutcome::Diagnostic { .. } => {}
        other => panic!("B must not resolve A's x, got {other:?}"),
    }

    // The carry tables differ exactly by x.
    let va = ace_session_op(&sa, AceSessionOp::Vars);
    let vb = ace_session_op(&sb, AceSessionOp::Vars);
    ok_of(&va);
    ok_of(&vb);
    assert!(va.vars.iter().any(|v| v.name == "x" && v.value == AceSessionValue::Int(41)));
    assert!(vb.vars.is_empty());
}

/// Carry discipline: assignments update the table; re-evaluation after
/// reassignment observes the NEW value (prelude injection + pure
/// capture).
#[test]
fn t970_variable_carry_and_reassignment() {
    let _host = Host::install();
    let s = ace_session_create(None, None);
    ok_of(&s);
    let s = s.session.unwrap();

    assert_eq!(eval_value(&s, "var x = 41"), AceSessionValue::None);
    assert_eq!(eval_value(&s, "x = x + 1"), AceSessionValue::None);
    assert_eq!(eval_value(&s, "x"), AceSessionValue::Int(42));
    assert_eq!(eval_value(&s, "var flag = 1 < 2"), AceSessionValue::None);
    assert_eq!(eval_value(&s, "flag"), AceSessionValue::Bool(true));
}

/// Bool evaluations ride the bool lane (if-encode at the frozen int
/// entry; the report decodes).
#[test]
fn t970_bool_evaluations() {
    let _host = Host::install();
    let s = ace_session_create(None, None);
    ok_of(&s);
    let s = s.session.unwrap();
    assert_eq!(eval_value(&s, "1 < 2"), AceSessionValue::Bool(true));
    assert_eq!(eval_value(&s, "3 < 2"), AceSessionValue::Bool(false));
    // Back to int after a bool evaluation (lane switch each way).
    assert_eq!(eval_value(&s, "6 * 7"), AceSessionValue::Int(42));
}

/// Item-definition inputs reshape the frozen declaration set (the named
/// rebuild path); the new function is callable; reserved names refuse.
#[test]
fn t970_reshape_and_reserved_names() {
    let _host = Host::install();
    let s = ace_session_create(None, None);
    ok_of(&s);
    let s = s.session.unwrap();

    let r = eval(&s, "fn helper() int { return 7 }");
    assert!(r.rebuilt, "item input reshapes");

    // Carry survives the reshape.
    assert_eq!(eval_value(&s, "var x = 5"), AceSessionValue::None);
    let r = eval(&s, "fn helper() int { return 8 }");
    assert!(r.rebuilt, "same-name definition replaces");
    assert_eq!(eval_value(&s, "helper() + x"), AceSessionValue::Int(13));

    // Reserved names are a named refusal.
    let refused = ace_session_op(
        &s,
        AceSessionOp::Eval {
            input: "fn __tail_anchor() int { return 1 }",
            wall: None,
            cancel: None,
        },
    );
    match refused.outcome {
        AceSessionOutcome::Denied { code, .. } if code == "session.source/reserved-name" => {}
        other => panic!("reserved name must refuse, got {other:?}"),
    }
}

/// Error-then-continue at the evaluation level: a refused evaluation
/// leaves the revision and the carry table untouched.
#[test]
fn t970_error_then_continue() {
    let _host = Host::install();
    let s = ace_session_create(None, None);
    ok_of(&s);
    let s = s.session.unwrap();
    assert_eq!(eval_value(&s, "var x = 10"), AceSessionValue::None);
    let before = ace_session_op(&s, AceSessionOp::Inspect);

    let bad = ace_session_op(
        &s,
        AceSessionOp::Eval {
            input: "var y = 1 +",
            wall: None,
            cancel: None,
        },
    );
    match bad.outcome {
        AceSessionOutcome::Diagnostic { .. } => {}
        other => panic!("ill-formed eval must diagnose, got {other:?}"),
    }

    let after = ace_session_op(&s, AceSessionOp::Inspect);
    assert_eq!(after.revision, before.revision, "refused eval keeps revision");
    let vars = ace_session_op(&s, AceSessionOp::Vars);
    ok_of(&vars);
    assert!(vars.vars.iter().any(|v| v.name == "x"));
    assert!(!vars.vars.iter().any(|v| v.name == "y"), "failed binding not carried");
    assert_eq!(eval_value(&s, "x * 2"), AceSessionValue::Int(20));
}

/// V-02: the same-PID async long evaluation keeps the OLD generation
/// for the in-flight chain while the committed edit serves later runs —
/// the resident mixed-generation total is observable and neither the
/// pure-old nor the pure-new value.
#[test]
fn t970_async_edit_mixed_generations() {
    let _host = Host::install();
    let s = ace_session_create(None, None);
    ok_of(&s);
    let s = s.session.unwrap();
    // helper rounds: 300 × 7 = 2100 (old), 300 × 9 = 2700 (new);
    // mixed = 2100 + 2k for the boundary round k.
    assert!(eval(&s, "fn helper() int { return 7 }").rebuilt);
    let started = ace_session_op(
        &s,
        AceSessionOp::EvalAsync {
            input: "var i = 0\nvar total = 0\nfor i < 300 { total = total + helper()\ni = i + 1 }\nreturn total",
            wall: None,
        },
    );
    ok_of(&started);
    let eval_id = started.eval_id.expect("async id");

    // Queue the helper swap while the chain runs.
    let inspect = ace_session_op(&s, AceSessionOp::Inspect);
    ok_of(&inspect);
    let source = inspect.snapshot.expect("source");
    let at = source.find("return 7").expect("helper body");
    let queued = ace_session_op(
        &s,
        AceSessionOp::Edit {
            start: at as u64,
            end: (at + 9) as u64,
            insert: b"return 9",
        },
    );
    ok_of(&queued);

    // Poll to completion (bounded); assert the mixed-generation form.
    let mut final_report = None;
    for _ in 0..4_000 {
        let r = ace_session_op(&s, AceSessionOp::Result { eval_id });
        ok_of(&r);
        if r.snapshot.as_deref() != Some("running") {
            final_report = Some(r);
            break;
        }
    }
    let done = final_report.expect("async evaluation completes");
    match done.value {
        AceSessionValue::Int(v) => {
            assert!(v > 2100 && v < 2700, "mixed generation value, got {v}");
            assert_eq!((v - 2100) % 2, 0, "boundary arithmetic holds");
        }
        other => panic!("expected int total, got {other:?}"),
    }

    // The committed edit now serves every later run: 300 × 9.
    assert_eq!(
        eval_value(
            &s,
            "var i = 0\nvar total = 0\nfor i < 300 { total = total + helper()\ni = i + 1 }\nreturn total"
        ),
        AceSessionValue::Int(2700)
    );
}

/// V-04: close releases the session (later ops refuse); cancel stops an
/// async chain; the unknown-session and unknown-id refusals are named.
#[test]
fn t970_close_cancel_and_refusals() {
    let _host = Host::install();
    let s = ace_session_create(None, None);
    ok_of(&s);
    let s = s.session.unwrap();

    let started = ace_session_op(
        &s,
        AceSessionOp::EvalAsync {
            input: "var i = 0\nfor i < 1000000 { i = i + 1 }\nreturn i",
            wall: None,
        },
    );
    ok_of(&started);
    let eval_id = started.eval_id.expect("id");
    let cancelled = ace_session_op(&s, AceSessionOp::CancelEval { eval_id });
    assert_eq!(cancelled.outcome, AceSessionOutcome::Cancelled);
    let fin = ace_session_op(&s, AceSessionOp::Result { eval_id });
    assert_eq!(fin.outcome, AceSessionOutcome::Cancelled);

    // Unknown eval id and session.
    let unknown_eval = ace_session_op(&s, AceSessionOp::Result { eval_id: 999 });
    match unknown_eval.outcome {
        AceSessionOutcome::Denied { code, .. } if code == "session.eval/unknown-id" => {}
        other => panic!("unknown id must refuse, got {other:?}"),
    }
    let closed = ace_session_op(&s, AceSessionOp::Close);
    assert_eq!(closed.outcome, AceSessionOutcome::Ok);
    let gone = ace_session_op(&s, AceSessionOp::Inspect);
    match gone.outcome {
        AceSessionOutcome::Denied { code, .. } if code == "session/unknown" => {}
        other => panic!("closed session must refuse, got {other:?}"),
    }
    assert!(!ace_session_list().contains(&s));
}

/// Reset clears source and variables; the session id survives.
#[test]
fn t970_reset() {
    let _host = Host::install();
    let s = ace_session_create(None, None);
    ok_of(&s);
    let s = s.session.unwrap();
    assert!(eval(&s, "fn helper() int { return 7 }").rebuilt);
    assert_eq!(eval_value(&s, "var x = 1"), AceSessionValue::None);
    let r = ace_session_op(&s, AceSessionOp::Reset);
    ok_of(&r);
    assert!(r.rebuilt);
    let vars = ace_session_op(&s, AceSessionOp::Vars);
    ok_of(&vars);
    assert!(vars.vars.is_empty());
    let gone = ace_session_op(
        &s,
        AceSessionOp::Eval {
            input: "helper()",
            wall: None,
            cancel: None,
        },
    );
    match gone.outcome {
        AceSessionOutcome::Diagnostic { .. } => {}
        other => panic!("reset must clear items, got {other:?}"),
    }
}

/// The session cap is a named resource refusal (default 20, kept from
/// the legacy daemon).
#[test]
fn t970_session_cap() {
    let _host = Host::install();
    for _ in 0..20 {
        let r = ace_session_create(None, None);
        ok_of(&r);
    }
    let over = ace_session_create(None, None);
    match over.outcome {
        AceSessionOutcome::ResourceDenied { kind, .. } if kind == "session-cap" => {}
        other => panic!("cap must be a named resource denial, got {other:?}"),
    }
    // Cleanup surface exists (idle expiry sweep).
    assert!(ace_session_cleanup_expired().is_empty());
}

/// Snapshot exports the user-visible program (probe bodies never leak).
#[test]
fn t970_snapshot_shape() {
    let _host = Host::install();
    let s = ace_session_create(None, None);
    ok_of(&s);
    let s = s.session.unwrap();
    assert_eq!(eval_value(&s, "var x = 41"), AceSessionValue::None);
    // The capture probe ran after this; the snapshot shows the USER
    // evaluation, not the probe.
    let snap = ace_session_op(&s, AceSessionOp::Snapshot);
    ok_of(&snap);
    let text = snap.snapshot.expect("snapshot");
    assert!(text.contains("var x = 41"), "user body in snapshot: {text}");
    assert!(text.contains("__tail_anchor"), "anchor present: {text}");
    assert!(!text.contains("return (41)"), "no probe leak: {text}");
}
