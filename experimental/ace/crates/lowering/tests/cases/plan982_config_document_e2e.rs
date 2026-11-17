//! PLAN-982（B-CONFIG-I，E3+E4+E5 最小闭环）：配置文档 → 单入口 MIR →
//! ABC 准入 → 宿主会话执行 → owned 快照。
//!
//! 本用例证明的是**真实运行**：文档行编译成的 `cfg.*` 调用经 PlatformCall
//! 落到 av-core 的配置会话，执行完导出 owned 文档（节点/身份/参数/属性全部
//! 按源顺序与种类）。它不证明产品消费（Man 侧 E7）与九 Case（E8）。

use ace_av_core::config_document::{self, OwnedId, OwnedNode, OwnedValue};
use ace_av_core::contract::Value;
use ace_av_core::mir_verify::verify_mir;
use ace_av_core::{encode_abc, verify_abc_bytes, CancelFlag, ExecLimits};
use ace_core::source::{compile_config_document, CONFIG_DOCUMENT_ENTRY_NAME};

fn entry_def_id() -> String {
    format!("d_{CONFIG_DOCUMENT_ENTRY_NAME}")
}

fn run_document(src: &str) -> Result<config_document::OwnedDocument, String> {
    let front = compile_config_document(src, "pac.at", &[]).map_err(|ds| {
        format!(
            "{:?}",
            ds.into_iter()
                .map(|d| (d.code, d.message))
                .collect::<Vec<_>>()
        )
    })?;
    let mir = ace_lowering::lower_checked_owned(&front.compiled.checked, &entry_def_id(), 0x982)
        .map_err(|e| format!("lower: {e:?}"))?;
    verify_mir(&mir).map_err(|e| format!("verify_mir: {e:?}"))?;
    let bytes = encode_abc(&mir).map_err(|e| format!("encode: {e:?}"))?;
    let owner = ace_av_core::mir::OwnerScope {
        schema: mir.owner.schema.clone(),
        module: mir.owner.module.clone(),
    };
    let loaded = verify_abc_bytes(&bytes.to_flat(), &owner).map_err(|e| format!("admit: {e:?}"))?;
    config_document::open_session(config_document::DocumentLimits::default())
        .map_err(|e| format!("open: {} {}", e.code, e.message))?;
    let cancel = CancelFlag::new(false);
    let (outcome, _trace) = ace_av_core::execute(&loaded, &ExecLimits::default(), &cancel);
    let closed = config_document::close_session();
    match (outcome, closed) {
        (ace_av_core::ExecOutcome::Success { .. }, Ok(doc)) => Ok(doc),
        (ace_av_core::ExecOutcome::Exit { .. }, Ok(doc)) => Ok(doc),
        (outcome, _) => {
            config_document::abandon_session();
            Err(format!("execute outcome: {outcome:?}"))
        }
    }
}

fn props(node: &OwnedNode, key: &str) -> Option<OwnedValue> {
    node.props
        .iter()
        .find(|(k, _)| k == key)
        .map(|(_, v)| v.clone())
}

/// 真实三形 dep 文档：根 prop + 三个 dep 节点（裸名 id / 字符串 id / 调用实参），
/// 子节点属性按源顺序与种类导出。
#[test]
fn p982_config_document_runs_end_to_end_to_owned_snapshot() {
    let src = r#"name: "fixture-deps"
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
    let doc = run_document(src).expect("the document evaluates to an owned snapshot");
    assert_eq!(
        props(&doc.root, "name"),
        Some(OwnedValue::Text("fixture-deps".into()))
    );
    assert_eq!(
        doc.root.children.len(),
        3,
        "three dep nodes stay in source order"
    );

    let bps = &doc.root.children[0];
    assert_eq!(bps.name, "dep");
    assert_eq!(bps.id, Some(OwnedId::Name("bps".into())));
    assert_eq!(
        props(bps, "path"),
        Some(OwnedValue::Text("../../packages/blueprints".into()))
    );

    let stylekit = &doc.root.children[1];
    assert_eq!(stylekit.name, "dep");
    assert_eq!(stylekit.id, Some(OwnedId::Text("stylekit".into())));
    assert_eq!(
        props(stylekit, "path"),
        Some(OwnedValue::Text("../stylekit".into()))
    );

    let kernel = &doc.root.children[2];
    assert_eq!(kernel.name, "dep");
    assert_eq!(
        kernel.id, None,
        "`dep(\"kernel\")` carries the name as an argument"
    );
    assert_eq!(kernel.args, vec![OwnedValue::Text("kernel".into())]);
    assert_eq!(
        props(kernel, "port"),
        Some(OwnedValue::Text("msvc/x64".into()))
    );
    // 真实代价事实（不是自报布尔）。
    assert!(doc.cost.nodes >= 4, "root + three nodes: {:?}", doc.cost);
    assert!(
        doc.cost.values >= 5,
        "name/path/port/kernel values: {:?}",
        doc.cost
    );
}

/// 嵌套 + 节点值 + 对象/数组值 + 运行时 `if` 分支：顺序、种类与 id 全保真。
#[test]
fn p982_config_document_runs_nested_values_and_branches() {
    let src = r#"var kernel_config = { mode: "lockstep", mpu: true }
kernel: kernel_config
var port = "linux"
if port == "win32" { x: 1 } else { x: 2 }
modules: ["a", "b"]
icf: file("linker/x.icf") { target: "chip" }
lib("x") {
    dir(id: "d") { at: "a" }
}
"#;
    let doc = run_document(src).expect("nested document evaluates");
    // `var kernel_config` 只绑定局部，不进 root。
    assert!(props(&doc.root, "kernel_config").is_none());
    match props(&doc.root, "kernel") {
        Some(OwnedValue::Obj(fields)) => {
            assert_eq!(
                fields[0],
                ("mode".to_string(), OwnedValue::Text("lockstep".into()))
            );
            assert_eq!(fields[1], ("mpu".to_string(), OwnedValue::Bool(true)));
        }
        other => panic!("`kernel` must be an object value, got {other:?}"),
    }
    // `port = "linux"` → 运行时取 else 分支（x: 2）。这是 AV 真实执行，不是静态折叠。
    assert_eq!(props(&doc.root, "x"), Some(OwnedValue::I32(2)));
    match props(&doc.root, "modules") {
        Some(OwnedValue::List(items)) => {
            assert_eq!(
                items,
                vec![OwnedValue::Text("a".into()), OwnedValue::Text("b".into())]
            );
        }
        other => panic!("`modules` must be a list value, got {other:?}"),
    }
    // 节点值 pair：`icf` 是名 `file`、参数 `linker/x.icf`、带体节点。
    match props(&doc.root, "icf") {
        Some(OwnedValue::Node(n)) => {
            assert_eq!(n.name, "file");
            assert_eq!(n.args, vec![OwnedValue::Text("linker/x.icf".into())]);
            assert_eq!(props(&n, "target"), Some(OwnedValue::Text("chip".into())));
        }
        other => panic!("`icf` must be a node value, got {other:?}"),
    }
    // 顶层节点 + 具名实参。
    assert_eq!(doc.root.children.len(), 1);
    let lib = &doc.root.children[0];
    assert_eq!(lib.name, "lib");
    assert_eq!(lib.args, vec![OwnedValue::Text("x".into())]);
    assert_eq!(lib.children.len(), 1);
    let dir = &lib.children[0];
    assert_eq!(dir.name, "dir");
    // `dir(id: d)` 的具名实参进 args + props（旧实现同形）。
    assert_eq!(props(dir, "id"), Some(OwnedValue::Text("d".into())));
}

/// 没有打开会话时执行必须具名拒绝（不是空操作、不是静默 0）。
#[test]
fn p982_config_document_execution_requires_an_open_session() {
    let front = compile_config_document("name: \"x\"\n", "pac.at", &[]).expect("compile");
    let mir = ace_lowering::lower_checked_owned(&front.compiled.checked, &entry_def_id(), 0x982)
        .expect("lower");
    let bytes = encode_abc(&mir).expect("encode");
    let owner = ace_av_core::mir::OwnerScope {
        schema: mir.owner.schema.clone(),
        module: mir.owner.module.clone(),
    };
    let loaded = verify_abc_bytes(&bytes.to_flat(), &owner).expect("admit");
    let cancel = CancelFlag::new(false);
    let (outcome, _trace) = ace_av_core::execute(&loaded, &ExecLimits::default(), &cancel);
    match outcome {
        ace_av_core::ExecOutcome::Infra(msg) => {
            assert!(
                msg.contains("configuration document session"),
                "expected the named session refusal, got {msg}"
            );
        }
        other => panic!("execution without a session must be a named refusal, got {other:?}"),
    }
}

/// 原语本身的 kind/预算纪律：坏句柄与超预算都是具名错误（不是 0/静默成功）。
#[test]
fn p982_config_document_primitives_reject_bad_handles_and_budgets() {
    let cancel = CancelFlag::new(false);
    config_document::open_session(config_document::DocumentLimits::default()).expect("open");
    let bad = config_document::dispatch(
        "auto.config.document.put.i32.v1",
        &[Value::I32(999), Value::I32(0)],
        &cancel,
    );
    // 参数个数不对时先拒（arity 兜底）。
    assert!(bad.is_err());
    let bad_kind = config_document::dispatch(
        "auto.config.document.append.v1",
        &[Value::I32(999), Value::I32(1000)],
        &cancel,
    );
    assert!(bad_kind.is_err(), "invalid handles must be rejected");
    config_document::abandon_session();

    // 值节点预算：只允许 1 个，第二个 node 即 Resource。
    config_document::open_session(config_document::DocumentLimits {
        value_nodes: 1,
        owned_bytes: 1024,
    })
    .expect("open tight");
    let first = config_document::dispatch("auto.config.document.obj-new.v1", &[], &cancel);
    assert!(first.is_ok());
    let second = config_document::dispatch("auto.config.document.obj-new.v1", &[], &cancel);
    assert!(
        matches!(second, Err(ace_av_core::host::HostError::Resource)),
        "the value-node budget must be enforced, got {second:?}"
    );
    config_document::abandon_session();
}
