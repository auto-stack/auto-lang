//! PLAN-982（B-CONFIG-I，E1/E2）：**配置文档 source 前门**。
//!
//! 本文件证明两件事，且只证明这两件：
//! - **E1**：profile 身份/名字映射存在；语句形态的文档行物化为**一个**顺序入口
//!   的 `CompiledSource`（入口在共享 Checked HIR 里真实存在；既有 14 个 profile
//!   的裁剪身份由 `plan981_grammar_scope` 的冻结表承担）。
//! - **E2**：真实 `pac.at` 三形 `dep` 头（`dep bps{}`／`dep "stylekit"{}`／
//!   `dep("kernel"){}`）＋顶层 prop＋嵌套体＋`if`/`for`＋对象/数组值**从 CST
//!   投影为文档候选**（不是字符串扫描，span 保原源）；坏邻例给原 span 的
//!   `source.*` 诊断；通用入口对该 profile 具名拒绝。
//!
//! 它**不**证明配置文档值语义已实现：`key: value`/节点行的执行与 owned 导出是
//! E3/E4/E5（`compile_config_document` 现在对它们返回具名 `pending`）。

use ace_core::source::ast::ExprKind;
use ace_core::source::config_document::{
    compile_config_document, parse_config_document, ConfigLine, ConfigNodeId, ConfigPairValue,
};
use ace_core::source::cst::{profile_from_slug, profile_grammar, profile_slug};
use ace_core::source::{
    parse_program_with, SourceProfile, CONFIG_DOCUMENT_DOMAIN, CONFIG_DOCUMENT_ENTRY_NAME,
    CONFIG_DOCUMENT_SOURCE_PROFILE, CONFIG_DOCUMENT_VALUE_SCHEMA,
};

fn diag_codes(src: &str) -> Vec<String> {
    match parse_config_document(src) {
        Ok(_) => Vec::new(),
        Err(ds) => ds
            .into_iter()
            .map(|d| format!("{}@{}..{}", d.code, d.span.start, d.span.end))
            .collect(),
    }
}

/// profile 身份：稳定版本串 ↔ 枚举 ↔ grammar slug 三向一致，且与 text-record
/// 的 grammar 身份隔离；通用解析入口具名拒绝该 profile。
#[test]
fn p982_config_document_profile_identity() {
    assert_eq!(
        SourceProfile::of(CONFIG_DOCUMENT_SOURCE_PROFILE),
        Some(SourceProfile::ConfigDocument)
    );
    assert_eq!(
        SourceProfile::ConfigDocument.as_str(),
        CONFIG_DOCUMENT_SOURCE_PROFILE
    );
    assert_eq!(
        profile_slug(SourceProfile::ConfigDocument),
        "config-document"
    );
    assert_eq!(
        profile_from_slug("config-document"),
        Some(SourceProfile::ConfigDocument)
    );
    assert_eq!(CONFIG_DOCUMENT_DOMAIN, "ace.config.document/v1");
    assert_eq!(CONFIG_DOCUMENT_VALUE_SCHEMA, "auto.config.value/1");
    assert!(CONFIG_DOCUMENT_SOURCE_PROFILE.starts_with("auto.source."));
    let cd = profile_grammar(SourceProfile::ConfigDocument).expect("config-document grammar");
    let tr = profile_grammar(SourceProfile::TextRecord).expect("text-record grammar");
    assert_ne!(
        format!("{}", cd.program().identity.grammar_id.0),
        format!("{}", tr.program().identity.grammar_id.0),
        "a new profile trim must not share the text-record GrammarId"
    );
    // 通用入口拒绝：文档行不会被普通投影静默丢弃。
    let err = parse_program_with("name: \"x\"\n", SourceProfile::ConfigDocument)
        .expect_err("generic parse must refuse the config-document profile");
    assert_eq!(err.code, "source.config-document/front-door");
}

/// E2 主证：PLAN-780 的真实三形 dep 输入（`static-dep-forms.pac.at` 同形）被投影
/// 成文档候选——节点名/头 span/id 三形/体属性/顶层 prop 全部保真。
#[test]
fn p982_config_document_parses_the_three_dep_forms() {
    let src = r#"// PLAN-780 V-02 fixture: all three dep name forms.
name: "fixture-deps"
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
    let doc = parse_config_document(src).expect("real three-form document parses");
    assert_eq!(doc.lines.len(), 4, "one top-level pair + three dep nodes");
    assert_eq!(doc.total_lines(), 7, "nested body pairs are counted too");
    assert!(!doc.grammar_id.is_empty());
    assert!(!doc.input_hash.is_empty());

    // 顶层 prop。`name` 的值是真实字符串字面量（解码后的值，不是词面扫描）。
    let ConfigLine::Pair(name) = &doc.lines[0] else {
        panic!(
            "first line must be the top-level prop, got {:?}",
            doc.lines[0]
        );
    };
    assert_eq!(name.key.name, "name");
    assert_eq!(&src[name.key.span.start..name.key.span.end], "name");
    match &name.value {
        ConfigPairValue::Expr(e) => {
            assert!(matches!(&e.kind, ExprKind::StrLit(s) if s == "fixture-deps"))
        }
        other => panic!("`name` value must be an expression, got {other:?}"),
    }

    // 三形 dep：裸名 id / 字符串 id / 调用实参。
    let nodes: Vec<_> = doc
        .lines
        .iter()
        .filter_map(|l| match l {
            ConfigLine::Node(n) => Some(n),
            _ => None,
        })
        .collect();
    assert_eq!(nodes.len(), 3);

    assert_eq!(nodes[0].name, "dep");
    assert_eq!(
        &src[nodes[0].name_span.start..nodes[0].name_span.end],
        "dep"
    );
    match &nodes[0].id {
        Some(ConfigNodeId::Bare(id)) => {
            assert_eq!(id.name, "bps");
            assert_eq!(&src[id.span.start..id.span.end], "bps");
        }
        other => panic!("`dep bps` must carry a bare-name id, got {other:?}"),
    }
    let body0 = nodes[0].body.as_ref().expect("`dep bps` body");
    assert_eq!(body0.lines.len(), 1);
    let ConfigLine::Pair(path) = &body0.lines[0] else {
        panic!("body line must be a pair");
    };
    assert_eq!(path.key.name, "path");

    assert_eq!(nodes[1].name, "dep");
    match &nodes[1].id {
        Some(ConfigNodeId::Text { text, span }) => {
            assert_eq!(text, "stylekit");
            assert_eq!(&src[span.start..span.end], "\"stylekit\"");
        }
        other => panic!("`dep \"stylekit\"` must carry a string id, got {other:?}"),
    }

    assert_eq!(nodes[2].name, "dep");
    assert!(
        nodes[2].id.is_none(),
        "`dep(\"kernel\")` id comes from args"
    );
    assert_eq!(nodes[2].args.len(), 1);
    assert_eq!(
        &src[nodes[2].args[0].span().start..nodes[2].args[0].span().end],
        "\"kernel\""
    );
    let body2 = nodes[2].body.as_ref().expect("`dep(\"kernel\")` body");
    assert_eq!(body2.lines.len(), 1);
}

/// E2 结构面：嵌套节点、对象/数组值、`if`/`for` 文档行、具名实参。
#[test]
fn p982_config_document_parses_nested_bodies_and_control_lines() {
    let src = r#"var kernel_config = { mode: "lockstep", mpu: true }
kernel: kernel_config
if port == "win32" { x: 1 } else { x: 2 }
modules: ["a", "b"]
icf: file("linker/x.icf") { target: "chip" }
lib("x") {
    for d in modules { dir(id: d) { at: d } }
}
"#;
    let doc = parse_config_document(src).expect("design §4 sample parses");
    assert_eq!(doc.lines.len(), 6);

    // `var` 行：局部绑定 + 对象字面量值（有序字段）。
    let ConfigLine::Var(v) = &doc.lines[0] else {
        panic!("first line must be a var line");
    };
    assert_eq!(v.name.name, "kernel_config");
    let ExprKind::ObjectLit { fields } = &v.value.kind else {
        panic!("object literal value expected, got {:?}", v.value.kind);
    };
    assert_eq!(fields.len(), 2);
    assert_eq!(fields[0].key, ace_core::hir::ObjectKey::Text("mode".into()));
    assert!(matches!(&fields[0].value.kind, ExprKind::StrLit(s) if s == "lockstep"));
    assert_eq!(fields[1].key, ace_core::hir::ObjectKey::Text("mpu".into()));
    assert!(matches!(&fields[1].value.kind, ExprKind::BoolLit(true)));

    // `kernel: kernel_config`——pair 的值是普通表达式（不是节点）。
    let ConfigLine::Pair(kernel) = &doc.lines[1] else {
        panic!("second line must be the `kernel:` pair");
    };
    assert!(matches!(
        &kernel.value,
        ConfigPairValue::Expr(e) if matches!(&e.kind, ExprKind::Var(id) if id.name == "kernel_config")
    ));

    // `if`/`else` 的体是**文档行**（分支里是 pair，不是语句）。
    let ConfigLine::If(ifline) = &doc.lines[2] else {
        panic!("third line must be an if line");
    };
    assert_eq!(ifline.then.lines.len(), 1);
    assert!(matches!(ifline.then.lines[0], ConfigLine::Pair(_)));
    let els = ifline.els.as_ref().expect("else branch");
    assert_eq!(els.lines.len(), 1);
    assert!(matches!(els.lines[0], ConfigLine::Pair(_)));

    // 数组值。
    let ConfigLine::Pair(modules) = &doc.lines[3] else {
        panic!("fourth line must be the `modules:` pair");
    };
    match &modules.value {
        ConfigPairValue::Expr(e) => match &e.kind {
            ExprKind::ArrayLit(elems) => {
                assert_eq!(elems.len(), 2);
                assert!(matches!(&elems[0].kind, ExprKind::StrLit(s) if s == "a"));
                assert!(matches!(&elems[1].kind, ExprKind::StrLit(s) if s == "b"));
            }
            other => panic!("array literal expected, got {other:?}"),
        },
        other => panic!("`modules` value must be an expression, got {other:?}"),
    }

    // 节点值 pair：`icf: file("linker/x.icf") { target: "chip" }`——值是带体节点，
    // 不是普通调用表达式，也不是被丢弃的属性。
    let ConfigLine::Pair(icf) = &doc.lines[4] else {
        panic!("fifth line must be the `icf:` pair");
    };
    assert_eq!(icf.key.name, "icf");
    let ConfigPairValue::Node(icf_node) = &icf.value else {
        panic!("`icf:` value must be a node, got {:?}", icf.value);
    };
    assert_eq!(icf_node.name, "file");
    assert_eq!(icf_node.args.len(), 1);
    let icf_body = icf_node.body.as_ref().expect("icf node body");
    assert_eq!(icf_body.lines.len(), 1);
    assert!(matches!(&icf_body.lines[0], ConfigLine::Pair(_)));

    // 顶层节点 `lib("x") { for d in modules { dir(id: d) { at: d } } }`。
    let ConfigLine::Node(lib) = &doc.lines[5] else {
        panic!("fifth line must be the `lib` node");
    };
    assert_eq!(lib.name, "lib");
    assert_eq!(lib.args.len(), 1);
    assert_eq!(
        &src[lib.args[0].span().start..lib.args[0].span().end],
        "\"x\""
    );
    let lib_body = lib.body.as_ref().expect("lib body");
    let ConfigLine::For(forline) = &lib_body.lines[0] else {
        panic!("lib body must start with the for line");
    };
    assert_eq!(forline.var.name, "d");
    assert!(matches!(&forline.iter.kind, ExprKind::Var(id) if id.name == "modules"));
    assert_eq!(forline.body.lines.len(), 1);
    let ConfigLine::Node(dir) = &forline.body.lines[0] else {
        panic!("for body must hold the `dir(...)` node");
    };
    assert_eq!(dir.name, "dir");
    assert_eq!(dir.args.len(), 1);
    let ace_core::source::ConfigArg::Named { name, value } = &dir.args[0] else {
        panic!("`dir(id: d)` must carry a named argument");
    };
    assert_eq!(name.name, "id");
    assert!(matches!(&value.kind, ExprKind::Var(id) if id.name == "d"));
    let dir_body = dir.body.as_ref().expect("dir body");
    assert_eq!(dir_body.lines.len(), 1);
}

/// E2 负控：坏邻例给**原 span** 的 `source.*` 诊断（不是静默吞掉，也不是
/// 无位置的通用失败）；空文档具名拒绝。
#[test]
fn p982_config_document_bad_neighbours_keep_original_spans() {
    // 有冒号无值：`name:` 不是合法 pair——首个错误落在原行且是 `source.*`。
    let codes = diag_codes("name:\n");
    assert!(!codes.is_empty(), "a value-less prop must be rejected");
    assert!(
        codes.iter().all(|c| c.starts_with("source.")),
        "diagnostics must be source.*: {codes:?}"
    );

    // 缺值：`path:` 后没有表达式——错误必须落在原源内（有具名 span，不是 0 宽
    // 的通用失败）。
    let src = "dep bps {\n    path:\n}\n";
    let codes = diag_codes(src);
    assert!(!codes.is_empty(), "a value-less pair must be rejected");
    let spans_are_original = codes.iter().all(|c| {
        let at = c.split('@').nth(1).unwrap_or("");
        let start: usize = at.split("..").next().unwrap_or("0").parse().unwrap_or(0);
        let end: usize = at.split("..").nth(1).unwrap_or("0").parse().unwrap_or(0);
        start < end && end <= src.len()
    });
    assert!(
        spans_are_original,
        "diagnostics must carry real spans inside the source: {codes:?}"
    );

    // 类型注解的文档局部：具名拒绝（不是静默丢注解）。
    let codes = diag_codes("var x int = 1\n");
    assert!(
        codes
            .iter()
            .any(|c| c.starts_with("source.config-document/annotation@")),
        "typed document locals must be named-rejected: {codes:?}"
    );

    // 空文档不是成功求值。
    let codes = diag_codes("   \n");
    assert_eq!(codes.len(), 1);
    assert!(codes[0].starts_with("source.config-document/empty@"));
}

/// E3 物化：文档行 → **一个**顺序入口，值行变成 `cfg_*` 受检原语调用；入口与
/// 原语声明都在共享 Checked HIR 里真实存在，能力键 `config.document/1` 进
/// `requires`。运行期求值（E4）尚未接线——这里只证明前端+检查链成立，不冒充
/// 可执行。
#[test]
fn p982_config_document_materializes_one_sequential_entry() {
    let src = "var port = \"win32\"\nif port == \"win32\" { var x = 1 } else { var x = 2 }\n";
    let front = compile_config_document(src, "pac.at", &[]).expect("statement document compiles");
    assert_eq!(front.entry, CONFIG_DOCUMENT_ENTRY_NAME);
    assert_eq!(front.compiled.profile, CONFIG_DOCUMENT_SOURCE_PROFILE);
    // root + var + if + finish。
    assert_eq!(
        front.statements, 4,
        "root/var/if/finish live in one entry body"
    );

    let module = front
        .compiled
        .checked
        .bundle()
        .modules
        .first()
        .expect("one module");
    let symbol = format!("sf_{CONFIG_DOCUMENT_ENTRY_NAME}");
    assert!(
        module.defs.iter().any(
            |d| matches!(&d.kind, ace_core::hir::DefKind::Function { name, .. } if name == &symbol)
        ),
        "the sequential entry must be a real checked function in the shared HIR"
    );
    let intrinsic_symbols: Vec<&str> = module
        .defs
        .iter()
        .filter_map(|d| match &d.kind {
            ace_core::hir::DefKind::Intrinsic { symbol, .. } => Some(symbol.as_str()),
            _ => None,
        })
        .collect();
    assert!(
        intrinsic_symbols.contains(&"auto.config.document.root.v1"),
        "the document root primitive must be a checked intrinsic: {intrinsic_symbols:?}"
    );
    assert!(
        intrinsic_symbols.contains(&"auto.config.document.finish.v1"),
        "the document finish primitive must be a checked intrinsic: {intrinsic_symbols:?}"
    );
    assert!(
        front
            .compiled
            .checked
            .required_capabilities()
            .iter()
            .any(|c| c == "config.document/1"),
        "the document capability must be required: {:?}",
        front.compiled.checked.required_capabilities()
    );

    // 精确 source map：文档行的 span 仍指向原源起点。
    let doc = parse_config_document(src).expect("parse");
    assert!(matches!(doc.lines[0], ConfigLine::Var(_)));
    assert!(matches!(doc.lines[1], ConfigLine::If(_)));
    assert_eq!(
        &src[doc.lines[0].span().start..doc.lines[0].span().start + 3],
        "var"
    );
}

/// E3 值面：真实三形 dep 文档编译成**一个**入口里的 `cfg_*` 调用序列（节点/
/// id/实参/属性/追加/收尾），能力键进 `requires`；隐藏局部名冲突具名拒绝；
/// 运行期 `for` 行仍具名 pending（下一增量）。
#[test]
fn p982_config_document_value_rows_become_cfg_calls() {
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
    let front = compile_config_document(src, "pac.at", &[]).expect("value rows compile");
    let module = front
        .compiled
        .checked
        .bundle()
        .modules
        .first()
        .expect("one module");
    let intrinsics: Vec<&str> = module
        .defs
        .iter()
        .filter_map(|d| match &d.kind {
            ace_core::hir::DefKind::Intrinsic { symbol, .. } => Some(symbol.as_str()),
            _ => None,
        })
        .collect();
    for want in [
        "auto.config.document.root.v1",
        "auto.config.document.node.v1",
        "auto.config.document.id.name.v1",
        "auto.config.document.id.text.v1",
        "auto.config.document.arg.str.v1",
        "auto.config.document.put.str.v1",
        "auto.config.document.append.v1",
        "auto.config.document.finish.v1",
    ] {
        assert!(
            intrinsics.contains(&want),
            "expected checked intrinsic `{want}`; got {intrinsics:?}"
        );
    }
    assert!(front
        .compiled
        .checked
        .required_capabilities()
        .iter()
        .any(|c| c == "config.document/1"));
    // 一个入口：整份文档只有这一个 checked 函数体（不是逐字段隐藏函数）。
    let fns = module
        .defs
        .iter()
        .filter(|d| matches!(&d.kind, ace_core::hir::DefKind::Function { .. }))
        .count();
    assert_eq!(fns, 1, "the whole document must materialize into ONE entry");

    // 隐藏前缀冲突：用户局部不得抢占编译器生成的 `__cfg_` 名字空间。
    let err = compile_config_document("var __cfg_x = 1\n", "pac.at", &[])
        .expect_err("hidden-prefix locals are rejected");
    assert_eq!(err[0].code, "source.config-document/hidden-name");

    // 运行期 `for` 行仍具名 pending（需要 list-count 原语）。
    let err = compile_config_document("for d in modules { x: 1 }\n", "pac.at", &[])
        .expect_err("`for` rows are the next increment");
    assert_eq!(err[0].code, "source.config-document/pending");
}
/// 语句形态的编译入口里只允许文档行的语句种类（负控：`return` 不是文档行）。
#[test]
fn p982_config_document_rejects_non_document_statements() {
    let codes = diag_codes("return 1\n");
    assert!(!codes.is_empty(), "`return` is not a document line");
    assert!(codes.iter().all(|c| c.starts_with("source.")));
}

/// E7 前置（真实语料）：`#` 行注释、JSON 形整份文档、块形对象值（含字符串键）
/// 都从 CST 投影成文档候选——这三类是 auto-man 语料里 131→149 的主要缺口。
#[test]
fn p982_config_document_accepts_hash_comments_json_and_block_objects() {
    // `#` 说明行（真实 pac.at 的常见写法）不产生文档行。
    let src = "# PLAN-683: explanation line\nname: \"x\"\n";
    let syntax = parse_config_document(src).expect("hash comments are trivia");
    assert_eq!(syntax.lines.len(), 1, "the comment must not become a line");

    // JSON 形整份文档：字符串键 + 数组值 + 尾逗号。
    let json = r#"{
  "scene": "workspace",
  "members": ["back"],
}
"#;
    let syntax = parse_config_document(json).expect("a JSON-shaped document parses");
    assert_eq!(syntax.lines.len(), 2, "one document line per JSON entry");
    match &syntax.lines[0] {
        ConfigLine::Pair(p) => {
            assert_eq!(p.key.name, "scene");
            match &p.value {
                ConfigPairValue::Expr(e) => {
                    assert!(matches!(e.kind, ExprKind::StrLit(ref s) if s == "workspace"))
                }
                other => panic!("expected an expression value, got {other:?}"),
            }
        }
        other => panic!("expected a pair line, got {other:?}"),
    }
    match &syntax.lines[1] {
        ConfigLine::Pair(p) => {
            assert_eq!(p.key.name, "members");
            match &p.value {
                ConfigPairValue::Expr(e) => match &e.kind {
                    ExprKind::ArrayLit(items) => assert_eq!(items.len(), 1),
                    other => panic!("expected an array value, got {other:?}"),
                },
                other => panic!("expected an expression value, got {other:?}"),
            }
        }
        other => panic!("expected a pair line, got {other:?}"),
    }

    // 块形对象值（换行分隔、无逗号）＋字符串键去引号。
    let block = "npm_deps: {\n  \"@scope/x\": \"link:/tmp/x\"\n}\n";
    let syntax = parse_config_document(block).expect("a block object value parses");
    match &syntax.lines[0] {
        ConfigLine::Pair(p) => match &p.value {
            ConfigPairValue::Expr(e) => match &e.kind {
                ExprKind::ObjectLit { fields } => {
                    assert_eq!(fields.len(), 1);
                    assert_eq!(
                        fields[0].key,
                        ace_core::hir::ObjectKey::Text("@scope/x".into()),
                        "string keys are decoded"
                    );
                }
                other => panic!("expected an object literal, got {other:?}"),
            },
            other => panic!("expected an expression value, got {other:?}"),
        },
        other => panic!("expected a pair line, got {other:?}"),
    }
}

/// V-CFG-TEMPLATE：模板 desugar 的两处语义。
/// - 字符串里的 `${expr}` 变成 `__fstr(...)`（真插值）；
/// - 节点头里的裸 `${expr}` 变成本文本字符串字面量（legacy 在节点头**不插值**，
///   实测 id/`_arg0` 都是字面 `${name}`）。
#[test]
fn p982_config_template_desugar_shapes() {
    let out = ace_core::source::fstr::desugar_config_templates("name: \"${name}\"\n")
        .expect("string template desugars");
    assert_eq!(out, "name: __fstr(name)\n");

    // 节点头的裸占位按 legacy 留字面（不插值）。
    let out = ace_core::source::fstr::desugar_config_templates("lib ${name} {}\n")
        .expect("head placeholder desugars");
    assert_eq!(out, "lib \"${name}\" {}\n");

    // 取值位的裸占位按 legacy **插值**（`render: ${x}` 是表达式）。
    let out = ace_core::source::fstr::desugar_config_templates("render: ${mode}\n")
        .expect("value placeholder desugars");
    assert_eq!(out, "render: __fstr(mode)\n");

    let out = ace_core::source::fstr::desugar_config_templates(
        "# note ${not_a_template}\nname: \"ok\"\n",
    )
    .expect("comments are not interpolated");
    assert_eq!(out, "# note ${not_a_template}\nname: \"ok\"\n");
}

/// E8/V-CFG-FLOW：`for` 的字面数组在**物化侧展开**（legacy
/// `test_config_for_unrolls_literal_array` 语义），运行期列表仍是具名 pending。
#[test]
fn p982_config_for_unrolls_literal_lists() {
    let src = "var names = [\"a\", \"b\"]\nfor d in ([\"x\", \"y\"]) { label: d }\nfor n in names { label: n }\n";
    let front = compile_config_document(src, "pac.at", &[]).expect("literal for loops compile");
    // root + var + 4 个 put（两次展开各 2 行）+ finish。
    assert!(
        front.statements >= 8,
        "the literal loops must unroll into real statements, got {}",
        front.statements
    );

    // PLAN-982（③，2026-11-18 用户指示）：**运行期列表现在编译**——物化侧发射
    // `Stmt::ForIn` + `Range(0, cfg_list_len)`，体内 `let <d> = cfg_list_at_i32(...)`。
    // **元素种类**的限制移到求值期：文本元素由 `list-at.i32` 具名拒绝（无 Str
    // 返回面），i32/bool/char 元素可真正迭代（`p982_config_for_runtime` 钉住）。
    // 这条断言因此从"必须 pending"改为"必须编译出真实语句"。
    let dynamic = "var base = [1, 2]\nvar m = base\nfor d in m { label: d }\n";
    let front = compile_config_document(dynamic, "pac.at", &[])
        .expect("a runtime-list for now compiles into a counted loop");
    assert!(
        front.statements >= 6,
        "the counted loop must emit real statements, got {}",
        front.statements
    );
}

/// 设计 §4.1：**空文档 = 空 root 文档**（求值入口），不是"求值失败"。
/// 原始前门仍对"没有文档行"给具名 `source.config-document/empty`（事实描述）。
#[test]
fn p982_config_empty_document_is_an_empty_root() {
    let front = compile_config_document("   \n", "pac.at", &[]).expect("empty input evaluates");
    assert_eq!(front.statements, 2, "root + finish only");
    // 纯注释文档同样得到空 root（与 legacy 一致）。
    let front = compile_config_document("# only a comment\n", "pac.at", &[]).expect("comment-only");
    assert_eq!(front.statements, 2);
    // 原始前门的具名事实不变。
    let codes = diag_codes("   \n");
    assert!(codes
        .iter()
        .any(|c| c.starts_with("source.config-document/empty@")));
}
/// 982/CONFIG：**保留字也能作文档键**（真实 pac.at：`tag: "kept"`、
/// `type: "x"`）。键位置是文档语义，不是语句关键字（下一批 I 的语料缺口）。
#[test]
fn p982_config_reserved_words_work_as_document_keys() {
    let keys_of = |src: &str| -> Vec<String> {
        let syntax = parse_config_document(src).expect("reserved keys parse");
        syntax
            .lines
            .iter()
            .map(|line| match line {
                ConfigLine::Pair(p) => p.key.name.clone(),
                other => panic!("expected a pair line, got {other:?}"),
            })
            .collect()
    };
    assert_eq!(
        keys_of("tag: \"kept\"\ntype: \"kind\"\nname: \"n\"\n"),
        vec!["tag", "type", "name"]
    );
    // JSON 形（字符串键）仍解码为无引号键，且保留字键同样可用。
    assert_eq!(
        keys_of("{\n  \"tag\": \"kept\",\n  type: \"kind\",\n  \"name\": \"n\",\n}\n"),
        vec!["tag", "type", "name"]
    );
}
/// 982/CONFIG：**对象字段搬运**（`heap: k["heap"]`）——不需要 Str 返回面，
/// 物化层发射 `cfg_super_put(dst, k, "heap")`；缺字段是具名错误。
#[test]
fn p982_config_object_field_reads_are_carried_into_the_document() {
    let src = "var k = { heap: \"big\" }\nheap: k[\"heap\"]\n";
    let front = compile_config_document(src, "pac.at", &[]).expect("object field read compiles");
    let module = front
        .compiled
        .checked
        .bundle()
        .modules
        .first()
        .expect("one module");
    let intrinsics: Vec<&str> = module
        .defs
        .iter()
        .filter_map(|d| match &d.kind {
            ace_core::hir::DefKind::Intrinsic { symbol, .. } => Some(symbol.as_str()),
            _ => None,
        })
        .collect();
    assert!(
        intrinsics.contains(&"auto.config.document.super-put.v1"),
        "the carrier primitive is emitted: {intrinsics:?}"
    );
    // 未知字段：前端不静默（运行期 `cfg_super_put` 具名拒绝；这里断言工具面
    // 仍能编译，具体错误由 av-core 合同用例覆盖）。
}
/// 链式对象读：`x: k["a"]["b"]` —— 中间步 `obj-get-obj` 取句柄、末端 `super-put`
/// 搬到 pair 键上（i32-only，不经过 Str 返回面）。
#[test]
fn p982_config_chained_object_reads_emit_both_primitives() {
    let src = "var k = { a: { b: 1 } }\nx: k[\"a\"][\"b\"]\n";
    let front = compile_config_document(src, "pac.at", &[]).expect("chained read compiles");
    let module = front
        .compiled
        .checked
        .bundle()
        .modules
        .first()
        .expect("one module");
    let intrinsics: Vec<&str> = module
        .defs
        .iter()
        .filter_map(|d| match &d.kind {
            ace_core::hir::DefKind::Intrinsic { symbol, .. } => Some(symbol.as_str()),
            _ => None,
        })
        .collect();
    for want in [
        "auto.config.document.obj-get-obj.v1",
        "auto.config.document.super-put.v1",
    ] {
        assert!(intrinsics.contains(&want), "missing {want}: {intrinsics:?}");
    }
}
