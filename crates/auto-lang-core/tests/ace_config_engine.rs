//! PLAN-982 B-CONFIG-I: the **explicit-ACE configuration path** on the SDK's
//! `AutoConfig`, end to end through the installed-executor door.
//!
//! Proven here (own process, so the executor slot is isolated):
//! 1. `LegacyVm` stays exactly the legacy engine (unchanged behaviour).
//! 2. With NO executor installed, `AceAv` is a NAMED error while the legacy
//!    engine still parses the same source — the no-fallback oracle.
//! 3. With an executor that serves configuration, `AceAv` returns the real
//!    document projected onto the consumer shape: exact values, ordered props,
//!    node arguments/children and duplicates preserved.
//! 4. The injected arguments reach the engine as explicit context (not argv).
//! 5. An executor without the face / a denial / a projection overflow is a
//!    NAMED error — never a silent fallback and never a silent value change.

use std::sync::Arc;

use auto_lang_core::ace_config::{
    AceConfigOutcome, AceConfigReport, AceConfigRequest, ConfigCost, ConfigDocument,
    ConfigIdentity, ConfigNode, ConfigValue, CONFIG_ENGINE_ID,
};
use auto_lang_core::config::AutoConfig;
use auto_lang_core::{install_ace_executor, shutdown_ace_executor};
use auto_lang_core::{AceExecutor, AceOutcome, AceRequest, AceRunReport};
use auto_val::{Obj, Value};

struct ConfigExecutor {
    /// `None` = the executor does not implement the configuration face.
    behaviour: Option<Behaviour>,
}

enum Behaviour {
    /// Echo the request context back as document props and include a node tree.
    Echo,
    /// A document whose U64 value cannot be projected exactly.
    Overflow,
    Deny,
}

impl AceExecutor for ConfigExecutor {
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
        let Some(behaviour) = &self.behaviour else {
            return AceConfigReport::unsupported();
        };
        match behaviour {
            Behaviour::Deny => {
                AceConfigReport::denied("ace.config.schema/denied", "schema not admitted")
            }
            Behaviour::Overflow => {
                let mut root = ConfigNode::new("root");
                root.props
                    .push(("big".to_string(), ConfigValue::U64(u64::MAX)));
                AceConfigReport::success(
                    ConfigDocument { root },
                    ConfigIdentity::of(request, CONFIG_ENGINE_ID),
                    ConfigCost::default(),
                )
            }
            Behaviour::Echo => {
                let mut root = ConfigNode::new("root");
                // Echo the injected context args so the test can prove they
                // reached the engine as explicit configuration context.
                root.props.push((
                    "context_args".to_string(),
                    ConfigValue::U64(request.context.args.len() as u64),
                ));
                for (key, value) in &request.context.args {
                    root.props.push((key.clone(), value.clone()));
                }
                // A realistic slice of document shape: values + ordered props,
                // a node child with an identity argument and a prop, and a
                // duplicate child.
                root.props
                    .push(("name".to_string(), ConfigValue::Text("p982b".to_string())));
                root.props
                    .push(("nul".to_string(), ConfigValue::Text("a\0b".to_string())));
                let mut dep = ConfigNode::new("dep");
                dep.id = Some("kernel".to_string());
                dep.props.push((
                    "port".to_string(),
                    ConfigValue::Text("msvc/x64".to_string()),
                ));
                root.children.push(dep);
                root.children.push(ConfigNode::new("link"));
                root.children.push(ConfigNode::new("link"));
                AceConfigReport::success(
                    ConfigDocument { root },
                    ConfigIdentity::of(request, CONFIG_ENGINE_ID),
                    ConfigCost::default(),
                )
            }
        }
    }
}

fn install(behaviour: Option<Behaviour>) {
    install_ace_executor(Arc::new(ConfigExecutor { behaviour }))
        .expect("a clean executor slot accepts the executor");
}

fn args_with_context() -> Obj {
    let mut args = Obj::new();
    args.set("port", Value::str("win32"));
    let mut kernel = Obj::new();
    kernel.set("port", Value::Int(7));
    args.set("kernel", Value::Obj(Box::new(kernel)));
    args
}

#[test]
fn p982b_explicit_ace_config_path() {
    let source = "name: \"p982b\"\n";
    let _ = shutdown_ace_executor();

    // ---- 1. The legacy engine is unchanged and still available explicitly.
    let legacy = AutoConfig::from_code_with_engine(
        source,
        &Obj::new(),
        auto_lang_core::ace_config::ConfigEngineChoice::LegacyVm,
    )
    .expect("the legacy engine still parses the document");
    assert_eq!(legacy.root.get_prop("name").to_astr().as_str(), "p982b");

    // ---- 2. No executor: the ACE path is a NAMED error, the legacy path is
    //         NOT consulted for it (no-fallback oracle).
    let err = AutoConfig::from_code_with_engine(
        source,
        &Obj::new(),
        auto_lang_core::ace_config::ConfigEngineChoice::AceAv,
    )
    .map(|_| ())
    .expect_err("without an executor the ACE path must refuse");
    let text = format!("{err}");
    assert!(
        text.contains("ace.host/executor-unregistered"),
        "the refusal is named: {text}"
    );

    // ---- 3. An executor without the configuration face: named refusal.
    install(None);
    let err = AutoConfig::from_code_with_engine(
        source,
        &Obj::new(),
        auto_lang_core::ace_config::ConfigEngineChoice::AceAv,
    )
    .map(|_| ())
    .expect_err("an executor without the face must refuse");
    assert!(
        format!("{err}").contains("ace.config/executor-unsupported"),
        "the refusal is named: {err}"
    );
    assert!(shutdown_ace_executor());

    // ---- 4. A serving executor: the document comes back through the consumer
    //         shape with exact values, order and duplicates preserved, and the
    //         injected context visible to the engine.
    install(Some(Behaviour::Echo));
    let args = args_with_context();
    let config = AutoConfig::from_code_with_engine(
        source,
        &args,
        auto_lang_core::ace_config::ConfigEngineChoice::AceAv,
    )
    .expect("the ACE path serves the document");
    assert_eq!(config.root.get_prop("name").to_astr().as_str(), "p982b");
    assert_eq!(
        config.root.get_prop("nul").to_astr().as_str(),
        "a\0b",
        "text keeps its exact bytes"
    );
    // The context args really reached the engine (count + tagged values).
    assert_eq!(config.root.get_prop("context_args").to_astr().as_str(), "2");
    assert_eq!(config.root.get_prop("port").to_astr().as_str(), "win32");
    match config.root.get_prop("kernel") {
        Value::Obj(obj) => assert_eq!(
            obj.get("port").map(|v| v.to_astr().as_str().to_string()),
            Some("7".to_string())
        ),
        other => panic!("the object argument keeps its shape: {other:?}"),
    }
    // Node children: order, duplicates and the identity argument.
    let mut names: Vec<String> = Vec::new();
    for (_key, kid) in config.root.kids_iter() {
        if let auto_val::Kid::Node(node) = kid {
            names.push(node.name.as_str().to_string());
        }
    }
    assert_eq!(names, ["dep", "link", "link"], "node order and duplicates");
    assert!(shutdown_ace_executor());

    // ---- 5. Denial propagates by name.
    install(Some(Behaviour::Deny));
    let err = AutoConfig::from_code_with_engine(
        source,
        &Obj::new(),
        auto_lang_core::ace_config::ConfigEngineChoice::AceAv,
    )
    .map(|_| ())
    .expect_err("a denied configuration must fail");
    assert!(format!("{err}").contains("ace.config.schema/denied"));
    assert!(shutdown_ace_executor());

    // ---- 6. A value with no exact consumer representation is refused, never
    //         silently changed.
    install(Some(Behaviour::Overflow));
    let err = AutoConfig::from_code_with_engine(
        source,
        &Obj::new(),
        auto_lang_core::ace_config::ConfigEngineChoice::AceAv,
    )
    .map(|_| ())
    .expect_err("an unrepresentable value must refuse");
    assert!(
        format!("{err}").contains("config.projection/u64-out-of-range"),
        "the projection refusal is named: {err}"
    );
    assert!(shutdown_ace_executor());

    // ---- 7. `from_file_with_engine` keeps the same discipline for a real file.
    let dir = std::env::temp_dir().join(format!("p982b-cfg-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let path = dir.join("pac.at");
    std::fs::write(&path, source).unwrap();
    let err = AutoConfig::from_file_with_engine(
        &path,
        &Obj::new(),
        auto_lang_core::ace_config::ConfigEngineChoice::AceAv,
    )
    .map(|_| ())
    .expect_err("no executor: the file path refuses too");
    assert!(format!("{err}").contains("ace.host/executor-unregistered"));
    let file_legacy = AutoConfig::from_file_with_engine(
        &path,
        &Obj::new(),
        auto_lang_core::ace_config::ConfigEngineChoice::LegacyVm,
    )
    .expect("the legacy file path still works");
    assert_eq!(
        file_legacy.root.get_prop("name").to_astr().as_str(),
        "p982b"
    );
    let _ = std::fs::remove_dir_all(&dir);

    // ---- 8. The success outcome is a real success (not a silent empty one).
    install(Some(Behaviour::Echo));
    let config = AutoConfig::from_code_with_engine(
        source,
        &Obj::new(),
        auto_lang_core::ace_config::ConfigEngineChoice::AceAv,
    )
    .expect("serving executor");
    assert!(matches!(config.root.get_prop("name"), Value::Str(_)));
    assert!(shutdown_ace_executor());
    // After close the door is unregistered again (no fallback).
    let err = AutoConfig::from_code_with_engine(
        source,
        &Obj::new(),
        auto_lang_core::ace_config::ConfigEngineChoice::AceAv,
    )
    .map(|_| ())
    .expect_err("closed host: the ACE path refuses");
    assert!(format!("{err}").contains("ace.host/executor-unregistered"));
}
