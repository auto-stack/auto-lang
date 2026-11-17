//! PLAN-982（B-CONFIG-I，E6）：**真桥**（`HostExecutor::eval_config`）端到端。
//!
//! 一个进程内一次装桥，覆盖：桥面合同（Success/Diagnostic/Resource/无桥具名拒）、
//! SDK 消费面（`AutoConfig::from_code_with_engine(.., AceAv)` 投影出原值/顺序/
//! 重复），以及"每次请求后配置会话必须已关闭"的清理纪律。

use auto_lang_core::ace_config::{
    AceConfigOutcome, AceConfigRequest, ConfigEngineChoice, ConfigValue,
};
use auto_lang_core::config::AutoConfig;
use auto_lang_core::eval_config_with_ace;
use auto_val::Obj;

fn bridge_ready() {
    let _ = auto_lang_ace_host::install();
}

#[test]
fn p982_config_engine_through_the_real_bridge() {
    let _ = auto_lang_core::shutdown_ace_executor();
    bridge_ready();

    // ---- 1. 桥面：真实文档求值到 owned 文档 ---------------------------------
    let src = r#"name: "fixture-deps"
version: "1.0.0"
dep bps {
    path: "../../packages/blueprints"
}
dep "stylekit" {
    path: "../stylekit"
}
dep("kernel") {
    port: "msvc/x64"
}
"#;
    let request = AceConfigRequest::simple(src);
    let report = eval_config_with_ace(&request);
    let (document, identity, cost) = match &report.outcome {
        AceConfigOutcome::Success {
            document,
            identity,
            cost,
        } => (document, identity, cost),
        other => panic!("the real bridge must serve the document, got {other:?}"),
    };
    assert_eq!(identity.engine, "av", "the receipt names the ACE AV engine");
    assert!(identity.served_by_ace_av());
    assert_eq!(
        document.root.name, "",
        "the document root is a plain container"
    );
    match document.root.prop("name") {
        Some(ConfigValue::Text(name)) => assert_eq!(name, "fixture-deps"),
        other => panic!("root `name` must be text, got {other:?}"),
    }
    assert_eq!(document.root.children.len(), 3);
    assert_eq!(document.root.children[0].id.as_deref(), Some("bps"));
    assert_eq!(document.root.children[1].id.as_deref(), Some("stylekit"));
    assert_eq!(document.root.children[2].id, None);
    assert_eq!(
        document.root.children[2].args,
        vec![ConfigValue::Text("kernel".into())]
    );
    assert!(
        cost.callee_count.unwrap_or(0) >= 10,
        "real primitive dispatch count expected, got {:?}",
        cost.callee_count
    );
    assert!(cost.steps > 0, "AV step count is observed");
    // 清理纪律：请求结束后没有遗留的配置会话。
    assert!(
        !auto_lang_ace_host::config_document::session_open(),
        "the bridge must close the configuration session on success"
    );

    // ---- 2. SDK 消费面：同一桥经 `AutoConfig` 的显式 ACE 路径 ----------------
    let config = AutoConfig::from_code_with_engine(src, &Obj::new(), ConfigEngineChoice::AceAv)
        .expect("the SDK consumes the ACE-evaluated document");
    assert_eq!(
        config.root.get_prop("name").to_astr().as_str(),
        "fixture-deps"
    );
    assert_eq!(config.root.get_prop("version").to_astr().as_str(), "1.0.0");
    let deps: Vec<_> = config.root.get_kids("dep");
    assert_eq!(deps.len(), 3, "dep children stay in source order");
    // 旧消费面形状：身份在统一 args/props 槽的 `_arg0`（与 legacy 一致）。
    let stylekit = config
        .root
        .get_kids("dep")
        .into_iter()
        .find(|n| n.get_prop("_arg0").to_astr().as_str() == "stylekit")
        .expect("the string-id dep keeps its identity argument");
    assert_eq!(stylekit.get_prop("path").to_astr().as_str(), "../stylekit");

    // ---- 3. 坏文档：具名诊断（原 span），不静默成功、不回退 ----------------
    let bad = AceConfigRequest::simple("dep bps {\n    path:\n}\n");
    let bad_report = eval_config_with_ace(&bad);
    match &bad_report.outcome {
        AceConfigOutcome::Diagnostic {
            stage,
            code,
            span: Some(span),
            ..
        } => {
            assert!(!stage.is_empty());
            assert!(
                code.starts_with("source."),
                "expected a source diagnostic, got {code}"
            );
            assert!(
                span.start < span.end,
                "the original span must be kept: {span:?}"
            );
        }
        other => panic!("a broken document must be a named diagnostic, got {other:?}"),
    }
    let sdk_err = match AutoConfig::from_code_with_engine(
        "dep bps {\n    path:\n}\n",
        &Obj::new(),
        ConfigEngineChoice::AceAv,
    ) {
        Ok(_) => panic!("the SDK must not fall back to the legacy engine"),
        Err(e) => e,
    };
    let msg = format!("{sdk_err}");
    assert!(
        msg.contains("ace.diagnostic/"),
        "named SDK error expected, got {msg}"
    );
    assert!(
        !auto_lang_ace_host::config_document::session_open(),
        "the bridge must close the configuration session on the failure path too"
    );

    // ---- 4. 预算：字节预算极小 → 具名 ResourceDenied ------------------------
    let mut tight = AceConfigRequest::simple("name: \"abcdef\"\n");
    tight.limits.max_bytes = 2;
    match eval_config_with_ace(&tight).outcome {
        AceConfigOutcome::ResourceDenied { kind, .. } => {
            assert!(!kind.is_empty(), "the resource denial must name its axis");
        }
        other => panic!("a tiny byte budget must be a named resource denial, got {other:?}"),
    }

    // ---- 5. 无桥：具名拒（不是回退，也不是静默成功） ------------------------
    let _ = auto_lang_core::shutdown_ace_executor();
    let unregistered = eval_config_with_ace(&AceConfigRequest::simple("name: \"x\"\n"));
    match &unregistered.outcome {
        AceConfigOutcome::HostFailure { code, .. } => {
            assert_eq!(code, "ace.host/executor-unregistered");
        }
        other => {
            panic!("without an executor the config face must be a named refusal, got {other:?}")
        }
    }
    // 恢复桥，避免影响同进程后续用例。
    bridge_ready();
}
