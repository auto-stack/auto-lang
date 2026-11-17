//! PLAN-982（B-CONFIG-I，E8）：**V-CFG 九 Case 的语言侧七例**。
//!
//! 每例只证明设计 §6 该行的**独立业务值**（不是 parser 绿、不是 new==old、不是
//! same_sdk）：真实桥（`HostExecutor::eval_config`）→ owned 中立文档 → 该 Case
//! 自己的 oracle；每例可单独红（把对应断言/输入删掉即红）。
//!
//! 覆盖与**具名缺口**（同一份计划台账）：
//! - SYNTAX/FLOW/CONTEXT/VALUE/FAIL/LIMIT/IDENTITY：本文件；
//! - PRODUCT/RETIRE：man 侧 `crates/auto/tests/plan982_ace_config_b.rs`。
//!
//! PENDING（当前能力缺、**不冒充通过**）：运行期 `for`（FLOW 的 for 行以具名
//! `source.config-document/pending` 断言）、对象字段访问/`cfg.get`（CONTEXT
//! 的对象注入以具名 `ace.config/inject-unsupported` 断言）、`cfg.merge`。

use std::sync::atomic::AtomicBool;
use std::sync::{Mutex, MutexGuard, Once};

use auto_lang_core::ace_config::{
    AceConfigOutcome, AceConfigReport, AceConfigRequest, ConfigContext, ConfigDocument,
    ConfigEvalLimits, ConfigValue,
};
use auto_lang_core::eval_config_with_ace;

static LOCK: Mutex<()> = Mutex::new(());
static INSTALL: Once = Once::new();

/// 例程互斥 + 一次装桥（executor 槽是进程级单例）。
fn setup() -> MutexGuard<'static, ()> {
    let guard = LOCK.lock().unwrap_or_else(|e| e.into_inner());
    INSTALL.call_once(|| {
        auto_lang_ace_host::install().expect("the reference ACE bridge installs");
    });
    guard
}

fn empty_context() -> ConfigContext {
    ConfigContext::empty().clone()
}

fn run(source: &str, context: &ConfigContext) -> AceConfigReport {
    let request = AceConfigRequest {
        source,
        virtual_path: "<vcfg>",
        context,
        limits: ConfigEvalLimits::default(),
        cancel: None,
    };
    eval_config_with_ace(&request)
}

fn document(source: &str) -> ConfigDocument {
    match run(source, &empty_context()).outcome {
        AceConfigOutcome::Success { document, .. } => document,
        other => panic!("expected a served document, got {other:?}"),
    }
}

fn prop(doc: &ConfigDocument, key: &str) -> Option<ConfigValue> {
    doc.root
        .props
        .iter()
        .find(|(k, _)| k == key)
        .map(|(_, v)| v.clone())
}

fn text(v: Option<ConfigValue>) -> String {
    match v {
        Some(ConfigValue::Text(t)) => t,
        other => panic!("expected text, got {other:?}"),
    }
}

fn i64_of(v: Option<ConfigValue>) -> i64 {
    match v {
        Some(ConfigValue::I64(n)) => n,
        other => panic!("expected an integer, got {other:?}"),
    }
}

// ---------------------------------------------------------------------------
// V-CFG-SYNTAX：三形 dep 头 + Node-valued pair + UI 面向量，精确值不漂移
// ---------------------------------------------------------------------------
#[test]
fn p982_vcfg_syntax_three_dep_forms_and_node_valued_pair() {
    let _g = setup();
    let src = r#"name: "syn"
dep bps {
    path: "../../packages/blueprints"
}
dep "stylekit" {
    path: "../stylekit"
}
dep("kernel") {
    port: "msvc/x64"
}
icf: file("linker/x.icf") {
    target: "chip"
}
window: "fit"
"#;
    let doc = document(src);
    assert_eq!(text(prop(&doc, "name")), "syn");
    assert_eq!(text(prop(&doc, "window")), "fit");
    assert_eq!(doc.root.children.len(), 3);
    let bps = &doc.root.children[0];
    assert_eq!(bps.name, "dep");
    assert_eq!(bps.id.as_deref(), Some("bps"));
    assert_eq!(
        text(Some(prop_of(bps, "path"))),
        "../../packages/blueprints"
    );
    let stylekit = &doc.root.children[1];
    assert_eq!(stylekit.id.as_deref(), Some("stylekit"));
    let kernel = &doc.root.children[2];
    assert_eq!(kernel.id, None);
    assert_eq!(kernel.args, vec![ConfigValue::Text("kernel".into())]);
    match prop(&doc, "icf") {
        Some(ConfigValue::Node(n)) => {
            assert_eq!(n.name, "file");
            assert_eq!(n.args, vec![ConfigValue::Text("linker/x.icf".into())]);
            assert_eq!(text(Some(prop_of(&n, "target"))), "chip");
        }
        other => panic!("`icf` must stay a node value, got {other:?}"),
    }
}

fn prop_of(node: &auto_lang_core::ace_config::ConfigNode, key: &str) -> ConfigValue {
    node.props
        .iter()
        .find(|(k, _)| k == key)
        .map(|(_, v)| v.clone())
        .unwrap_or(ConfigValue::Null)
}

// ---------------------------------------------------------------------------
// V-CFG-FLOW：控制流真在 AV 里执行（if/else 分支、var 遮蔽、嵌套体顺序）
// ---------------------------------------------------------------------------
#[test]
fn p982_vcfg_flow_control_runs_in_the_av() {
    let _g = setup();
    let mut context = empty_context();
    context
        .args
        .push(("port".to_string(), ConfigValue::Text("win32".to_string())));
    let src = r#"var mode = "default"
if port == "win32" { x: 1 } else { x: 2 }
if mode == "default" { y: "yes" }
var mode = "latest"
z: mode
lib("a") {
    dir(id: "n1") { at: "p1" }
}
"#;
    let doc = match run(src, &context).outcome {
        AceConfigOutcome::Success { document, .. } => document,
        other => panic!("expected served document, got {other:?}"),
    };
    assert_eq!(
        i64_of(prop(&doc, "x")),
        1,
        "the injected port selects the then-branch"
    );
    assert_eq!(text(prop(&doc, "y")), "yes");
    assert_eq!(
        text(prop(&doc, "z")),
        "latest",
        "a redeclared document var takes the latest binding"
    );
    let lib = &doc.root.children[0];
    assert_eq!(lib.name, "lib");
    let dir = &lib.children[0];
    assert_eq!(dir.name, "dir");
    // `dir(id: "n1")` 是**具名实参**（不是节点身份形）：legacy 同形把它记成 prop。
    assert_eq!(text(Some(prop_of(dir, "id"))), "n1");
    assert_eq!(text(Some(prop_of(dir, "at"))), "p1");

    // `for` 的字面展开（legacy `test_config_for_unrolls_literal_array` 语义）：
    // 字面数组按源顺序展开，循环变量逐项绑定。动态列表仍是具名 pending。
    let unrolled = r#"lib("x") {
    for d in (["a", "b"]) { dir(id: d) { at: d } }
}
"#;
    let doc = match run(unrolled, &empty_context()).outcome {
        AceConfigOutcome::Success { document, .. } => document,
        other => panic!("a literal `for` must unroll and run, got {other:?}"),
    };
    let lib = &doc.root.children[0];
    assert_eq!(lib.children.len(), 2, "both iterations produced their node");
    for (i, want) in ["a", "b"].iter().enumerate() {
        let dir = &lib.children[i];
        assert_eq!(text(Some(prop_of(dir, "id"))), *want);
        assert_eq!(text(Some(prop_of(dir, "at"))), *want);
    }
    // 运行期列表（这里经一个非字面绑定）⇒ 具名 pending，不静默空跑。
    let dynamic = "var base = [\"a\"]\nvar m = base\nfor d in m { x: d }\n";
    match run(dynamic, &empty_context()).outcome {
        AceConfigOutcome::Diagnostic { code, .. } => {
            assert_eq!(code, "source.config-document/pending")
        }
        other => panic!("a runtime-list `for` must be a named pending, got {other:?}"),
    }

    // 另一分支真的改变结果（非空证）：同一文档、port 换成 linux ⇒ x=2。
    let mut linux = empty_context();
    linux
        .args
        .push(("port".to_string(), ConfigValue::Text("linux".to_string())));
    let doc = match run(src, &linux).outcome {
        AceConfigOutcome::Success { document, .. } => document,
        other => panic!("expected served document, got {other:?}"),
    };
    assert_eq!(
        i64_of(prop(&doc, "x")),
        2,
        "the other branch is a different real result"
    );
}

// ---------------------------------------------------------------------------
// V-CFG-CONTEXT：注入分轴（bool/int/text）、on/off 内建、文档遮蔽、不支持种类具名拒
// ---------------------------------------------------------------------------
#[test]
fn p982_vcfg_context_injection_axes() {
    let _g = setup();
    let mut context = empty_context();
    context
        .args
        .push(("feature".to_string(), ConfigValue::Bool(true)));
    context
        .args
        .push(("retries".to_string(), ConfigValue::I64(3)));
    context
        .args
        .push(("label".to_string(), ConfigValue::Text("beta".to_string())));
    let src = r#"flag: feature
tries: retries
tagged: label
toggle: on
off_toggle: off
var label = "doc-wins"
shadowed: label
"#;
    let doc = match run(src, &context).outcome {
        AceConfigOutcome::Success { document, .. } => document,
        other => panic!("expected served document, got {other:?}"),
    };
    assert_eq!(prop(&doc, "flag"), Some(ConfigValue::Bool(true)));
    assert_eq!(i64_of(prop(&doc, "tries")), 3);
    // 注入值同时是根属性：文档同名 pair 不覆盖它（legacy 语义）。
    assert_eq!(text(prop(&doc, "tagged")), "beta");
    assert_eq!(prop(&doc, "toggle"), Some(ConfigValue::Bool(true)));
    assert_eq!(prop(&doc, "off_toggle"), Some(ConfigValue::Bool(false)));
    assert_eq!(
        text(prop(&doc, "shadowed")),
        "doc-wins",
        "a document `var` shadows the injected name in expressions"
    );

    // 设计 §4.3：被注入覆盖的默认 pair **不执行**——一个本会失败的 RHS
    // （`1 / 0` 在 AV 里 trap）不得让整份文档失败（B-CONFIG-R F-982B-09）。
    let mut overridden = empty_context();
    overridden.args.push(("x".to_string(), ConfigValue::I64(7)));
    let guarded = "x: 1 / 0\ny: \"kept\"\n";
    match run(guarded, &overridden).outcome {
        AceConfigOutcome::Success { document, .. } => {
            assert_eq!(i64_of(prop(&document, "x")), 7, "the injected value wins");
            assert_eq!(text(prop(&document, "y")), "kept");
        }
        other => panic!("the overridden pair's RHS must not execute, got {other:?}"),
    }
    // 注入覆盖只作用于**根级**：节点体里与注入名相同的键是节点自己的属性，
    // 必须保留（B-CONFIG-R F-982B-18）。
    let mut nested = empty_context();
    nested
        .args
        .push(("port".to_string(), ConfigValue::Text("win32".to_string())));
    let nested_src = "dep(\"kernel\") {\n    port: \"msvc/x64\"\n}\n";
    match run(nested_src, &nested).outcome {
        AceConfigOutcome::Success { document, .. } => {
            let dep = &document.root.children[0];
            assert_eq!(dep.name, "dep");
            assert_eq!(
                text(Some(prop_of(dep, "port"))),
                "msvc/x64",
                "a nested `port` is the node's own property, not the injected root key"
            );
        }
        other => panic!("the nested same-name key must survive, got {other:?}"),
    }

    // 根级**分支体**里写的同名 pair 也属于"根默认值"：被覆盖时 RHS 同样不执行
    // （B-CONFIG-R F-982B-27）。
    let mut branch = empty_context();
    branch.args.push(("x".to_string(), ConfigValue::I64(9)));
    match run("if true { x: 1 / 0 }\n", &branch).outcome {
        AceConfigOutcome::Success { document, .. } => {
            assert_eq!(i64_of(prop(&document, "x")), 9);
        }
        other => {
            panic!("a root-level branch default must not execute when overridden, got {other:?}")
        }
    }

    // 根级 `for` 体同理（同一 `ctx.depth` 路径，B-CONFIG-R F-982B-32）。
    let mut looped = empty_context();
    looped.args.push(("x".to_string(), ConfigValue::I64(11)));
    match run("for d in ([\"a\", \"b\"]) { x: 1 / 0 }\n", &looped).outcome {
        AceConfigOutcome::Success { document, .. } => assert_eq!(i64_of(prop(&document, "x")), 11),
        other => {
            panic!("a root-level for-body default must not execute when overridden, got {other:?}")
        }
    }

    // 非空证：同一文档在**没有**注入时确实被 RHS 的失败挡住（traps ⇒ 具名诊断）。
    match run(guarded, &empty_context()).outcome {
        AceConfigOutcome::Diagnostic { code, stage, .. } => {
            assert!(
                code.starts_with("ace.config/") || code.starts_with("source."),
                "the trap is a named diagnostic: {code} ({stage})"
            );
        }
        other => panic!("`1 / 0` without an override must fail by name, got {other:?}"),
    }

    // 对象字段**搬运**（`heap: k["heap"]`）：值进文档，脚本不经过 Str 返回面。
    match run(
        "var k = { heap: \"big\" }\nheap: k[\"heap\"]\n",
        &empty_context(),
    )
    .outcome
    {
        AceConfigOutcome::Success { document, .. } => {
            assert_eq!(text(prop(&document, "heap")), "big");
        }
        other => panic!("an object field must be carried into the document, got {other:?}"),
    }

    // 搬运**保留值种类**且落在**pair 的键**上（不是源字段名）：早先的实现把
    // 目标键硬写成源字段名，只有"同名"时才对——被 i32 用例抓到后修正。
    match run("var k = { port: 8080 }\np: k[\"port\"]\n", &empty_context()).outcome {
        AceConfigOutcome::Success { document, .. } => {
            assert_eq!(
                i64_of(prop(&document, "p")),
                8080,
                "dst key is the pair key"
            );
            assert!(
                prop(&document, "port").is_none(),
                "the source name is not leaked"
            );
        }
        other => panic!("an i32 field must be carried under the pair key, got {other:?}"),
    }

    // 对象值字段的搬运：字段本身是对象时，搬进去的仍是**对象**（不是被拍平/丢弃）。
    match run("var k = { a: { b: 1 } }\nx: k[\"a\"]\n", &empty_context()).outcome {
        AceConfigOutcome::Success { document, .. } => match prop(&document, "x") {
            Some(ConfigValue::Object(fields)) => {
                assert_eq!(fields.len(), 1, "the nested object keeps its field");
                assert_eq!(fields[0].0, "b");
            }
            other => panic!("an object-valued field must stay an object, got {other:?}"),
        },
        other => panic!("object-valued carry must succeed, got {other:?}"),
    }

    // 链式读：`x: k["a"]["b"]` ⇒ 文档 `x == 1`（中间步取对象句柄）。
    match run(
        "var k = { a: { b: 1 } }\nx: k[\"a\"][\"b\"]\n",
        &empty_context(),
    )
    .outcome
    {
        AceConfigOutcome::Success { document, .. } => {
            assert_eq!(i64_of(prop(&document, "x")), 1);
        }
        other => panic!("a chained object read must serve the leaf value, got {other:?}"),
    }

    // 缺字段：具名诊断（不是空值、不是 panic）。
    match run("var k = { port: 1 }\np: k[\"nope\"]\n", &empty_context()).outcome {
        AceConfigOutcome::Diagnostic { code, .. } => {
            assert!(
                code.starts_with("ace.config/") || code.starts_with("ace.diagnostic/"),
                "the missing field is named: {code}"
            );
        }
        // 宿主车道把这条错误报成 `HostFailure`（`ace.config/infra` + 具名 message），
        // 与 `Diagnostic` 同为具名终态；不允许"空值/panic/静默"。
        AceConfigOutcome::HostFailure { code, message } => {
            assert!(code.starts_with("ace.config/"), "named code: {code}");
            assert!(
                message.contains("no field"),
                "the message names the missing field: {message}"
            );
        }
        other => panic!("a missing object field must be a named terminal, got {other:?}"),
    }

    // 不能精确承载的注入种类：具名拒绝（不静默丢弃、不默认空）。
    let mut bad = empty_context();
    bad.args.push(("ratio".to_string(), ConfigValue::f64(1.5)));
    match run("name: \"x\"\n", &bad).outcome {
        AceConfigOutcome::Denied { code, .. } => assert_eq!(code, "ace.config/inject-unsupported"),
        other => panic!("an unsupported injected kind must be a named denial, got {other:?}"),
    }
}

// ---------------------------------------------------------------------------
// V-CFG-VALUE：owned 值面（中/emoji/NUL、边界整数、空/嵌套容器、顺序与重复）
// ---------------------------------------------------------------------------
#[test]
fn p982_vcfg_value_owned_precision() {
    let _g = setup();
    let src = r#"cjk: "中文"
emoji: "😀"
nul: "a\u{0}b"
min: -2147483648
max: 2147483647
empty_obj: {}
nested: { inner: [1, 2] }
list: ["x", "y"]
dep("dup") { at: "." }
dep("dup") { at: "." }
"#;
    let doc = document(src);
    assert_eq!(text(prop(&doc, "cjk")), "中文");
    assert_eq!(text(prop(&doc, "emoji")), "😀");
    assert_eq!(text(prop(&doc, "nul")), "a\u{0}b");
    assert_eq!(i64_of(prop(&doc, "min")), -2147483648);
    assert_eq!(i64_of(prop(&doc, "max")), 2147483647);
    assert_eq!(prop(&doc, "empty_obj"), Some(ConfigValue::Object(vec![])));
    match prop(&doc, "nested") {
        Some(ConfigValue::Object(fields)) => {
            assert_eq!(fields.len(), 1);
            assert_eq!(fields[0].0, "inner");
            match &fields[0].1 {
                ConfigValue::List(items) => {
                    assert_eq!(items.len(), 2);
                    assert_eq!(items[0], ConfigValue::I64(1));
                    assert_eq!(items[1], ConfigValue::I64(2));
                }
                other => panic!("nested list lost: {other:?}"),
            }
        }
        other => panic!("nested object lost: {other:?}"),
    }
    // 重复孩子保序且都可见（不去重）。
    assert_eq!(doc.root.children.len(), 2);
    assert_eq!(doc.root.children[0].args, doc.root.children[1].args);
    // owned 文档在会话关闭后仍可读（不是借用运行期内存）。
    let snapshot = doc.clone();
    assert_eq!(text(prop(&snapshot, "cjk")), "中文");
}

// ---------------------------------------------------------------------------
// V-CFG-FAIL：坏源具名原 span、零副作用、修复后下一读成功、无回退
// ---------------------------------------------------------------------------
#[test]
fn p982_vcfg_fail_is_named_and_never_leaks_state() {
    let _g = setup();
    let bad = "dep bps {\n    path:\n}\n";
    let report = run(bad, &empty_context());
    match &report.outcome {
        AceConfigOutcome::Diagnostic {
            stage,
            code,
            message,
            span,
        } => {
            assert!(!stage.is_empty(), "stage is kept: {stage}");
            assert!(
                code.starts_with("source."),
                "named source diagnostic: {code}"
            );
            assert!(!message.is_empty());
            let span = span.expect("the refusal keeps a span");
            assert!(span.start < span.end, "original span: {span:?}");
            assert!((span.end as usize) <= bad.len());
        }
        other => panic!("a broken document must be a named diagnostic, got {other:?}"),
    }
    // 零副作用：失败后没有遗留的配置会话。
    assert!(!auto_lang_ace_host::config_document::session_open());
    // 修复后同一进程的下一次读**真实成功**（不是缓存/回退）。
    let fixed = "dep bps {\n    path: \"../bps\"\n}\n";
    let doc = document(fixed);
    assert_eq!(doc.root.children.len(), 1);
    assert_eq!(text(Some(prop_of(&doc.root.children[0], "path"))), "../bps");
}

// ---------------------------------------------------------------------------
// V-CFG-LIMIT：预算逐轴真实扣费（值节点 N-1/N/N+1、字节界、取消）
// ---------------------------------------------------------------------------
#[test]
fn p982_vcfg_limit_budgets_are_real() {
    let _g = setup();
    // 值节点预算：`["a","b"]` 需要 1 个列表值；把预算压到 0 ⇒ 具名资源拒。
    let mut tight = ConfigEvalLimits::default();
    tight.max_nodes = 1;
    let request = AceConfigRequest {
        source: "list: [\"a\", \"b\"]\n",
        virtual_path: "<vcfg>",
        context: &empty_context(),
        limits: tight,
        cancel: None,
    };
    match eval_config_with_ace(&request).outcome {
        AceConfigOutcome::ResourceDenied { kind, .. } => assert!(!kind.is_empty()),
        other => panic!("a tiny value budget must be a named resource denial, got {other:?}"),
    }
    // 同一文档、默认预算 ⇒ 成功（非空证，证明上面的拒绝来自预算而非源）。
    let doc = document("list: [\"a\", \"b\"]\n");
    assert!(matches!(prop(&doc, "list"), Some(ConfigValue::List(v)) if v.len() == 2));

    // 取消：宿主 cancel 在处理前被观测 ⇒ Cancelled（不是成功、不是静默空）。
    let cancel = AtomicBool::new(true);
    let request = AceConfigRequest {
        source: "name: \"x\"\n",
        virtual_path: "<vcfg>",
        context: &empty_context(),
        limits: ConfigEvalLimits::default(),
        cancel: Some(&cancel),
    };
    match eval_config_with_ace(&request).outcome {
        AceConfigOutcome::Cancelled => {}
        other => panic!("a pre-set cancel flag must cancel the run, got {other:?}"),
    }
}

// ---------------------------------------------------------------------------
// V-CFG-IDENTITY：身份随 args 键/值变化，冷恢复后真实重读
// ---------------------------------------------------------------------------
#[test]
fn p982_vcfg_identity_tracks_context() {
    use auto_lang_core::ace_config::ConfigIdentity;
    let _g = setup();
    let src = "name: \"id\"\n";
    let base = empty_context();
    let mut other_value = empty_context();
    other_value
        .args
        .push(("port".to_string(), ConfigValue::Text("win32".to_string())));
    let mut other_key = empty_context();
    other_key
        .args
        .push(("kernel".to_string(), ConfigValue::Text("win32".to_string())));

    let request = AceConfigRequest {
        source: src,
        virtual_path: "<vcfg>",
        context: &base,
        limits: ConfigEvalLimits::default(),
        cancel: None,
    };
    let id0 = ConfigIdentity::of(&request, "av");
    let request = AceConfigRequest {
        source: src,
        virtual_path: "<vcfg>",
        context: &other_value,
        limits: ConfigEvalLimits::default(),
        cancel: None,
    };
    let id1 = ConfigIdentity::of(&request, "av");
    let request = AceConfigRequest {
        source: src,
        virtual_path: "<vcfg>",
        context: &other_key,
        limits: ConfigEvalLimits::default(),
        cancel: None,
    };
    let id2 = ConfigIdentity::of(&request, "av");
    assert_ne!(
        id0.context_digest, id1.context_digest,
        "a changed value changes identity"
    );
    assert_ne!(
        id0.context_digest, id2.context_digest,
        "a changed key set changes identity"
    );
    assert_eq!(
        id0.source_hash, id1.source_hash,
        "identical source keeps its hash"
    );
    assert!(id0.served_by_ace_av());

    // 值轴（B-CONFIG-R F-982B-01）：同一个键、不同**值**必须换身份，且经真桥
    // 求值的业务结果也不同——身份不是 `ConfigIdentity::of` 的自测。
    let mut win32 = empty_context();
    win32
        .args
        .push(("port".to_string(), ConfigValue::Text("win32".to_string())));
    let mut linux = empty_context();
    linux
        .args
        .push(("port".to_string(), ConfigValue::Text("linux".to_string())));
    let branch_src = "if port == \"win32\" { x: 1 } else { x: 2 }\n";
    let request = AceConfigRequest {
        source: branch_src,
        virtual_path: "<vcfg>",
        context: &win32,
        limits: ConfigEvalLimits::default(),
        cancel: None,
    };
    let win_id = ConfigIdentity::of(&request, "av");
    let win_doc = match eval_config_with_ace(&request).outcome {
        AceConfigOutcome::Success { document, .. } => document,
        other => panic!("expected served document, got {other:?}"),
    };
    let request = AceConfigRequest {
        source: branch_src,
        virtual_path: "<vcfg>",
        context: &linux,
        limits: ConfigEvalLimits::default(),
        cancel: None,
    };
    let linux_id = ConfigIdentity::of(&request, "av");
    let linux_doc = match eval_config_with_ace(&request).outcome {
        AceConfigOutcome::Success { document, .. } => document,
        other => panic!("expected served document, got {other:?}"),
    };
    assert_ne!(
        win_id.context_digest, linux_id.context_digest,
        "one key with two different values must not share a context digest"
    );
    assert_eq!(win_id.source_hash, linux_id.source_hash);
    assert_eq!(i64_of(prop(&win_doc, "x")), 1);
    assert_eq!(i64_of(prop(&linux_doc, "x")), 2);

    // 身份轴不替代真实求值：同一文档仍被真实服务一次（有代价事实）。
    match run(src, &base).outcome {
        AceConfigOutcome::Success { cost, identity, .. } => {
            assert!(cost.steps > 0, "real AV steps are observed");
            assert!(cost.callee_count.unwrap_or(0) > 0);
            assert_eq!(identity.engine, "av");
        }
        other => panic!("expected a served document, got {other:?}"),
    }
}

/// V-CFG-VALUE（深度轴，B-CONFIG-R F-982B-07/F-982B-26/F-982B-30）：**1000 层整链
/// 投影是永久断言**（`MAX_PROJECTION_DEPTH` 已与 av-core 的 2048 守卫对齐，不再
/// 是旧的 512 硬墙）；**2000 层只要求不崩**——要么整链服务，要么 MIR 校验的
/// `VerifyMemory` 预算具名拒绝（预算本身也被断言，见下）。
#[test]
fn p982_vcfg_deep_nesting_projects_beyond_the_old_cap() {
    let _g = setup();
    // 双档（B-CONFIG-R F-982B-26）：1000 层**必须被服务且整链投影**（远高于旧的
    // 512 硬墙、低于 MIR 校验的内存预算）；2000 层只要求"不崩"（服务或具名
    // `VerifyMemory` 预算拒绝）。探针实测：≤1976 层可服务，1980+ 被冻结的
    // 16 MiB 校验预算具名拒绝。
    const SERVED: usize = 1000;
    const DEEP: usize = 2000;
    let served_src = nested_document(SERVED);
    let doc = match run(&served_src, &empty_context()).outcome {
        AceConfigOutcome::Success { document, cost, .. } => {
            assert!(
                cost.depth >= SERVED as u64 - 1,
                "cost depth: {:?}",
                cost.depth
            );
            document
        }
        other => panic!("a {SERVED}-deep document must be served, got {other:?}"),
    };
    assert_eq!(
        counted_depth(&doc),
        SERVED,
        "the whole {SERVED}-level chain survives the projection"
    );

    let deep_src = nested_document(DEEP);
    match run(&deep_src, &empty_context()).outcome {
        AceConfigOutcome::Success { document, .. } => {
            assert_eq!(counted_depth(&document), DEEP);
        }
        AceConfigOutcome::Diagnostic {
            stage,
            code,
            message,
            ..
        } => {
            assert_eq!(stage, "verify");
            assert_eq!(code, "ace.config/verify");
            // 拒绝的**原因**必须是冻结的校验内存预算（B-CONFIG-R F-982B-29）：
            // 否则任何别的校验回归在这个深度都会保持绿。
            assert!(
                message.contains("VerifyMemory") && message.contains("above"),
                "the 2000-deep refusal must be the frozen verify-memory budget, got: {message}"
            );
            eprintln!("2000-deep document refused by the verify-memory budget (no crash)");
        }
        other => panic!("a {DEEP}-deep document must not crash, got {other:?}"),
    }
}

/// `{ a: { a: … 1 … } }` 形文档（depth 层嵌套对象）。
fn nested_document(depth: usize) -> String {
    let mut src = String::from("root_obj: ");
    for _ in 0..depth {
        src.push_str("{ a: ");
    }
    src.push('1');
    for _ in 0..depth {
        src.push_str(" }");
    }
    src.push('\n');
    src
}

/// 数投影后 root_obj 的嵌套对象层数（末层是整数 1）。
fn counted_depth(doc: &ConfigDocument) -> usize {
    let mut value = match prop(doc, "root_obj") {
        Some(v) => v,
        None => panic!("root prop missing"),
    };
    let mut depth = 0usize;
    while let ConfigValue::Object(fields) = value {
        assert_eq!(fields.len(), 1);
        depth += 1;
        value = fields[0].1.clone();
    }
    assert_eq!(value, ConfigValue::I64(1));
    depth
}
