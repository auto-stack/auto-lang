//! PLAN-984 B-EMBED (A batch): the permanent embedding-contract fixture.
//!
//! The out-of-repo consumer (plan-984 artifacts scope, `embed-consumer`)
//! is the one-shot external proof; THIS file pins the same embedding
//! contract as a daily-visible regression so the public surfaces a real
//! host composes cannot drift apart:
//!   * one host process installs ALL public faces (969 bridge, 970
//!     session host, 971-A project executor) and they stay installable
//!     together (registry discipline: single owner, idempotent reinstall);
//!   * an ORIGINAL multi-module script — the shipped
//!     `stdlib/ace/std/{math,text}.at` module sources read verbatim from
//!     the repository tree — runs through the explicit project gate with
//!     an independent value oracle (square(7)*100 + byte_len("Auto") =
//!     4904), and a broken original module is diagnosed under its own
//!     `std::math` identity;
//!   * two live sessions stay isolated and closing one is a named
//!     refusal while the survivor keeps evaluating;
//!   * the permission/compat doors refuse by NAME with zero dispatch
//!     (grants without a provider, argv/stdin lanes) and a legal
//!     neighbor still runs after every refusal.
//!
//! Platform: Windows x64 (this crate is Windows-only). The 978
//! candidate-SDK legs stay with the scope consumer (`run-sdk-legs.sh`) —
//! they drive the `ac-build` CLI and never belong in a daily test body
//! (2026-11-14 test ruling: build tools stay out of daily tests).

#![cfg(windows)]

use std::time::Duration;

use auto_lang_ace_host::install;
use auto_lang_ace_host::project::project_host;
use auto_lang_ace_host::session_bridge::install_session_host;
use auto_lang_core::{
    ace_session_create, ace_session_op, install_ace_project_executor, packages_capabilities,
    run_ace, run_ace_project, AceEngineChoice, AceModuleSpec, AceOutcome, AceProjectOutcome,
    AceProjectRequest, AceRequest, AceSessionOp, AceSessionOutcome, AceSessionValue, AceValue,
};

/// The ORIGINAL stdlib module sources (read verbatim — never re-typed
/// here, so a stdlib edit that breaks the embedding lane turns this
/// fixture red instead of silently re-baselining).
fn original_stdlib_module(id: &str) -> String {
    let path = format!("../../stdlib/ace/std/{id}.at");
    std::fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("read original stdlib module {path}: {e}"))
}

const ENTRY_MAIN: &str = concat!(
    "use std.math.square\n",
    "use std.text.byte_len\n",
    "fn main() int { return square(7) * 100 + byte_len(\"Auto\") }",
);

fn spec<'a>(package: &'a str, id: &'a str, source: &'a str) -> AceModuleSpec<'a> {
    AceModuleSpec {
        package,
        version: "1.0.0",
        id,
        source,
    }
}

/// One host process installs every public face; the same singletons
/// reinstall idempotently; a real project runs right after the boot.
#[test]
fn p984_embed_host_boots_all_public_faces() {
    install().expect("install the reference ACE host");
    install().expect("reinstall is idempotent");
    {
        let _guard = install_session_host(Default::default()).expect("session host");
    } // guard dropped: the registry slot is released again
    install_ace_project_executor(project_host()).expect("project executor");
    install_ace_project_executor(project_host()).expect("same-pointer reinstall is idempotent");

    let math = original_stdlib_module("math");
    let text = original_stdlib_module("text");
    let modules = [
        spec("std", "math", &math),
        spec("std", "text", &text),
        spec("app", "main", ENTRY_MAIN),
    ];
    let caps = packages_capabilities();
    let req = AceProjectRequest::simple_project(&modules, "app::main", &caps);
    let report = run_ace_project(AceEngineChoice::AceAV, &req);
    assert!(
        matches!(&report.outcome, AceProjectOutcome::Success { value: AceValue::I32(4904), steps } if *steps > 0),
        "the booted host must run the original multi-module script, got {:?}",
        report.outcome
    );
}

/// The ORIGINAL stdlib math+text modules run cross-package through the
/// explicit project gate; a broken original module is diagnosed under
/// its own module identity, and the intact project recovers.
#[test]
fn p984_embed_original_stdlib_modules_run_cross_package() {
    install().expect("install the reference ACE host");
    install_ace_project_executor(project_host()).expect("project executor");

    let math = original_stdlib_module("math");
    let text = original_stdlib_module("text");
    let caps = packages_capabilities();

    // Value leg: 49 * 100 + 4 = 4904 (independent oracle: the 943/945
    // math/text contracts, not the implementation's own echo).
    let modules = [
        spec("std", "math", &math),
        spec("std", "text", &text),
        spec("app", "main", ENTRY_MAIN),
    ];
    let req = AceProjectRequest::simple_project(&modules, "app::main", &caps);
    let report = run_ace_project(AceEngineChoice::AceAV, &req);
    assert!(
        matches!(
            &report.outcome,
            AceProjectOutcome::Success {
                value: AceValue::I32(4904),
                ..
            }
        ),
        "expected 4904, got {:?}",
        report.outcome
    );
    assert_eq!(report.exit_code, 0);

    // Negative neighbor INSIDE the multi-module run: the ORIGINAL module
    // slot breaks; the diagnosis carries that module's identity.
    let bad_math = math.replace("pub fn square", "pub fn nosuch");
    let bad_modules = [
        spec("std", "math", &bad_math),
        spec("std", "text", &text),
        spec("app", "main", ENTRY_MAIN),
    ];
    let bad_req = AceProjectRequest::simple_project(&bad_modules, "app::main", &caps);
    let bad_report = run_ace_project(AceEngineChoice::AceAV, &bad_req);
    match &bad_report.outcome {
        AceProjectOutcome::Diagnostic { module, .. } => {
            assert_eq!(module, "std::math", "the ORIGINAL module identity");
        }
        other => panic!("broken original module must diagnose, got {other:?}"),
    }
    assert_eq!(bad_report.exit_code, 1);

    // Recovery: the intact project runs again after the diagnostic.
    let fixed = run_ace_project(AceEngineChoice::AceAV, &req);
    assert!(matches!(
        fixed.outcome,
        AceProjectOutcome::Success {
            value: AceValue::I32(4904),
            ..
        }
    ));
}

/// Cross-session isolation: independent carried state per session; a
/// closed session refuses later ops BY NAME; the survivor is unaffected.
#[test]
fn p984_embed_session_isolation_and_close() {
    install().expect("install the reference ACE host");
    let _guard = install_session_host(Default::default()).expect("session host");

    let a = ace_session_create(
        Some("fn main() int { return 0 }"),
        Some(Duration::from_secs(30)),
    );
    let id_a = match (&a.outcome, &a.session) {
        (AceSessionOutcome::Ok, Some(id)) => id.clone(),
        other => panic!("create A: {other:?}"),
    };
    let b = ace_session_create(
        Some("fn main() int { return 0 }"),
        Some(Duration::from_secs(30)),
    );
    let id_b = match (&b.outcome, &b.session) {
        (AceSessionOutcome::Ok, Some(id)) => id.clone(),
        other => panic!("create B: {other:?}"),
    };
    assert_ne!(id_a, id_b, "two live sessions must have distinct ids");

    for (id, input, want) in [(&id_a, "var x int = 11", 11), (&id_b, "var x int = 22", 22)] {
        let set = ace_session_op(
            id,
            AceSessionOp::Eval {
                input,
                wall: None,
                cancel: None,
            },
        );
        assert!(
            matches!(set.outcome, AceSessionOutcome::Ok),
            "set: {:?}",
            set.outcome
        );
        let get = ace_session_op(
            id,
            AceSessionOp::Eval {
                input: "x",
                wall: None,
                cancel: None,
            },
        );
        assert!(
            matches!(&get.outcome, AceSessionOutcome::Ok if get.value == AceSessionValue::Int(want)),
            "session {} must read its own x={want}, got {:?} / {:?}",
            id,
            get.outcome,
            get.value
        );
    }

    let close = ace_session_op(&id_a, AceSessionOp::Close);
    assert!(matches!(close.outcome, AceSessionOutcome::Ok));
    let late = ace_session_op(
        &id_a,
        AceSessionOp::Eval {
            input: "1 + 1",
            wall: None,
            cancel: None,
        },
    );
    match &late.outcome {
        AceSessionOutcome::Denied { code, .. } | AceSessionOutcome::HostFailure { code, .. } => {
            assert!(!code.is_empty(), "closed-session refusal must be named");
        }
        other => panic!("late op on a closed session must refuse, got {other:?}"),
    }
    let survivor = ace_session_op(
        &id_b,
        AceSessionOp::Eval {
            input: "x + 1",
            wall: None,
            cancel: None,
        },
    );
    assert!(
        matches!(&survivor.outcome, AceSessionOutcome::Ok if survivor.value == AceSessionValue::Int(23)),
        "the surviving session must be unaffected, got {:?} / {:?}",
        survivor.outcome,
        survivor.value
    );
    let _ = ace_session_op(&id_b, AceSessionOp::Close);
}

/// The embedding permission/compat doors refuse by NAME with zero
/// dispatch, and a legal neighbor still runs after every refusal.
#[test]
fn p984_embed_permission_refusals_are_named() {
    install().expect("install the reference ACE host");

    let caps: Vec<String> = vec![];
    let empty: Vec<String> = vec![];
    // One explicit lifetime for every borrow of the request.
    fn plain<'a>(
        args: &'a [&'a str],
        input: Option<&'a str>,
        grants: &'a [String],
        caps: &'a [String],
    ) -> AceRequest<'a> {
        AceRequest {
            source: "fn main() int { return 1 }",
            entry_def_id: "d_main",
            capabilities: caps,
            grants,
            provider: None,
            wall: None,
            cancel: None,
            args,
            input,
            console: false,
        }
    }
    let call = |args: &[&str], input: Option<&str>, grants: &[String]| {
        run_ace(AceEngineChoice::AceAV, &plain(args, input, grants, &caps))
    };

    // Grants without a provider: named refusal, never a silent grant.
    let grants = vec!["foreign.scalar".to_string()];
    let rep = call(&[], None, &grants);
    assert!(
        matches!(&rep.outcome, AceOutcome::Denied { code, .. } if code == "ace.grant/no-provider"),
        "grants without provider must refuse by name, got {:?}",
        rep.outcome
    );

    // argv lane: named refusal (no silent drop, no legacy fallback).
    let rep = call(&["--flag"], None, &empty);
    assert!(
        matches!(&rep.outcome, AceOutcome::Denied { code, .. } if code == "ace.args/unsupported"),
        "argv lane must refuse by name, got {:?}",
        rep.outcome
    );

    // stdin lane: named refusal.
    let rep = call(&[], Some("payload"), &empty);
    assert!(
        matches!(&rep.outcome, AceOutcome::Denied { code, .. } if code == "ace.input/unsupported"),
        "stdin lane must refuse by name, got {:?}",
        rep.outcome
    );

    // The legal neighbor runs after every refusal.
    let rep = call(&[], None, &empty);
    assert!(
        matches!(
            &rep.outcome,
            AceOutcome::Success {
                value: AceValue::I32(1),
                ..
            }
        ),
        "legal neighbor must run, got {:?}",
        rep.outcome
    );
}
