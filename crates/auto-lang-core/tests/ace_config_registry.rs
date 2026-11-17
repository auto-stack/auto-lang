//! PLAN-982 B-CONFIG-I: the configuration face's **registry and dispatch
//! discipline** (own test process, so the process-wide executor slot is
//! isolated from the crate's unit tests).
//!
//! What is proven here:
//! 1. No installed executor ⇒ a NAMED host failure (`ace.host/executor-unregistered`),
//!    never a legacy evaluation.
//! 2. An executor that does not implement the configuration face ⇒ the default
//!    NAMED refusal (`ace.config/executor-unsupported`), never a legacy retry.
//! 3. An executor that implements it ⇒ the owned document, identity and cost
//!    come back through the public door, exact values preserved.
//! 4. Denials and cancellation propagate as their own terminal states.
//! 5. Closing the host returns the door to the unregistered state (no fallback).

use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;

use auto_lang_core::ace_config::{
    AceConfigOutcome, AceConfigReport, AceConfigRequest, ConfigCost, ConfigDocument,
    ConfigEngineChoice, ConfigIdentity, ConfigNode, ConfigValue, CONFIG_ENGINE_ID,
};
use auto_lang_core::{
    eval_config_with_ace, install_ace_executor, shutdown_ace_executor, AceExecutor, AceOutcome,
    AceRequest, AceRunReport,
};

/// An executor whose configuration behaviour is canned; `execute` is never
/// used by these cases and says so.
struct CannedExecutor {
    /// `None` = the executor does NOT implement the configuration face.
    behaviour: Option<Behaviour>,
    calls: Arc<AtomicU64>,
}

enum Behaviour {
    Success,
    Denied,
    CancelledWhenFlagged,
}

impl AceExecutor for CannedExecutor {
    fn execute(&self, _request: &AceRequest<'_>) -> AceRunReport {
        AceRunReport {
            engine: "av",
            outcome: AceOutcome::HostFailure {
                code: "ace.host/unused-in-this-test".to_string(),
                message: "the script face is not under test here".to_string(),
            },
            diag_count: 0,
            callee_count: None,
            rejected_count: None,
            wall_ms: 0.0,
            console: None,
        }
    }

    fn eval_config(&self, request: &AceConfigRequest<'_>) -> AceConfigReport {
        self.calls.fetch_add(1, Ordering::SeqCst);
        match &self.behaviour {
            None => AceConfigReport::unsupported(),
            Some(Behaviour::Success) => {
                let mut kernel = ConfigNode::new("dep");
                kernel.id = Some("kernel".to_string());
                kernel.props.push((
                    "port".to_string(),
                    ConfigValue::Text("msvc/x64".to_string()),
                ));
                let mut root = ConfigNode::new("root");
                root.props
                    .push(("u".to_string(), ConfigValue::U64(u64::MAX)));
                root.props
                    .push(("nul".to_string(), ConfigValue::Text("a\0b".to_string())));
                root.props.push((
                    "nan".to_string(),
                    ConfigValue::f64(f64::from_bits(0x7ff8_0000_0000_0001)),
                ));
                root.children.push(kernel);
                root.children.push(ConfigNode::new("link"));
                root.children.push(ConfigNode::new("link"));
                let document = ConfigDocument { root };
                let identity = ConfigIdentity::of(request, CONFIG_ENGINE_ID);
                let cost = ConfigCost {
                    steps: 42,
                    nodes: 4,
                    bytes: 64,
                    depth: 1,
                    callee_count: Some(3),
                };
                AceConfigReport::success(document, identity, cost)
            }
            Some(Behaviour::Denied) => AceConfigReport::denied(
                "ace.config.schema/denied",
                "the document schema is not admitted for this run",
            ),
            Some(Behaviour::CancelledWhenFlagged) => match request.cancel {
                Some(flag) if flag.load(Ordering::SeqCst) => AceConfigReport::cancelled(),
                _ => AceConfigReport::diagnostic(
                    "execute",
                    "ace.config/no-cancel",
                    "the cancel flag was not observed",
                    None,
                ),
            },
        }
    }
}

fn install(behaviour: Option<Behaviour>) -> Arc<AtomicU64> {
    let calls = Arc::new(AtomicU64::new(0));
    let executor = Arc::new(CannedExecutor {
        behaviour,
        calls: Arc::clone(&calls),
    });
    install_ace_executor(executor).expect("a clean registry accepts the executor");
    calls
}

#[test]
fn p982b_config_face_registry_discipline() {
    // ---- 1. No executor installed: NAMED host failure, no legacy evaluation.
    let _ = shutdown_ace_executor();
    let report = eval_config_with_ace(&AceConfigRequest::simple("name: \"p982\"\n"));
    match &report.outcome {
        AceConfigOutcome::HostFailure { code, .. } => {
            assert_eq!(code, "ace.host/executor-unregistered")
        }
        other => panic!("an unregistered registry must fail by name, got {other:?}"),
    }
    assert!(!report.is_success());
    assert!(report.wall_ms >= 0.0);

    // ---- 2. Executor without the configuration face: default NAMED refusal.
    let calls = install(None);
    let report = eval_config_with_ace(&AceConfigRequest::simple("name: \"p982\"\n"));
    match &report.outcome {
        AceConfigOutcome::HostFailure { code, message } => {
            assert_eq!(code, "ace.config/executor-unsupported");
            assert!(
                message.contains("legacy"),
                "the refusal states that no legacy engine is used: {message}"
            );
        }
        other => panic!("an unsupporting executor must refuse by name, got {other:?}"),
    }
    assert_eq!(calls.load(Ordering::SeqCst), 1, "the door dispatched once");
    assert!(shutdown_ace_executor());

    // ---- 3. Implementing executor: owned document + identity + cost.
    let source = "name: \"p982\"\n";
    let calls = install(Some(Behaviour::Success));
    let report = eval_config_with_ace(&AceConfigRequest::simple(source));
    assert_eq!(calls.load(Ordering::SeqCst), 1);
    match &report.outcome {
        AceConfigOutcome::Success {
            document,
            identity,
            cost,
        } => {
            // Exact values survived the door.
            assert_eq!(document.root.prop("u"), Some(&ConfigValue::U64(u64::MAX)));
            assert_eq!(
                document.root.prop("nul"),
                Some(&ConfigValue::Text("a\0b".to_string()))
            );
            let nan = document.root.prop("nan").unwrap().as_f64().unwrap();
            assert_eq!(nan.to_bits(), 0x7ff8_0000_0000_0001);
            // Node order + duplicates + node/object distinction.
            let names: Vec<&str> = document
                .root
                .children
                .iter()
                .map(|c| c.name.as_str())
                .collect();
            assert_eq!(names, ["dep", "link", "link"]);
            assert_eq!(document.root.children[0].id.as_deref(), Some("kernel"));
            // Identity is attributable, not a boolean.
            assert_eq!(identity.engine, CONFIG_ENGINE_ID);
            assert!(identity.served_by_ace_av());
            assert_eq!(
                identity.source_hash,
                auto_lang_core::ace_config::fnv1a(source.as_bytes())
            );
            assert_eq!(identity.schema, "auto.config/eval/1");
            // Cost carries the real dispatch facts.
            assert_eq!(cost.steps, 42);
            assert_eq!(cost.callee_count, Some(3));
            // The document's own accounting is stable.
            let stats = document.stats();
            assert_eq!(stats.nodes, 4);
            assert_eq!(
                stats.depth, 2,
                "children sit one level below the root and their props one below that"
            );
        }
        other => panic!("a supporting executor must succeed, got {other:?}"),
    }
    assert!(report.wall_ms >= 0.0);
    // Same executor reinstalled is idempotent; the registry stays single-owner.
    assert!(shutdown_ace_executor());

    // ---- 4. Denial propagates as its own terminal state.
    install(Some(Behaviour::Denied));
    let report = eval_config_with_ace(&AceConfigRequest::simple(source));
    match &report.outcome {
        AceConfigOutcome::Denied { code, .. } => assert_eq!(code, "ace.config.schema/denied"),
        other => panic!("denial must propagate, got {other:?}"),
    }
    assert!(shutdown_ace_executor());

    // ---- 5. Cancellation: the host-owned flag reaches the engine.
    install(Some(Behaviour::CancelledWhenFlagged));
    let flag = AtomicBool::new(true);
    let request = AceConfigRequest {
        cancel: Some(&flag),
        ..AceConfigRequest::simple(source)
    };
    let report = eval_config_with_ace(&request);
    match &report.outcome {
        AceConfigOutcome::Cancelled => {}
        other => panic!("a flagged cancel must be observed, got {other:?}"),
    }
    // The flag was genuinely consulted: with it clear, the same executor does
    // not claim cancellation (it reports its own diagnostic instead).
    let clear = AtomicBool::new(false);
    let request = AceConfigRequest {
        cancel: Some(&clear),
        ..AceConfigRequest::simple(source)
    };
    let report = eval_config_with_ace(&request);
    assert!(matches!(
        report.outcome,
        AceConfigOutcome::Diagnostic { .. }
    ));
    assert!(shutdown_ace_executor());

    // ---- 6. Closing the host returns the door to the unregistered state.
    let report = eval_config_with_ace(&AceConfigRequest::simple(source));
    assert!(matches!(
        report.outcome,
        AceConfigOutcome::HostFailure { .. }
    ));

    // ---- 7. Engine names are explicit; the legacy name is never implicit.
    assert_eq!(ConfigEngineChoice::AceAv.as_str(), "av");
    assert_eq!(ConfigEngineChoice::LegacyVm.as_str(), "legacy-vm");
}
