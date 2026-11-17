//! PLAN-982（B-CONFIG-I，E1）：**配置文档 source 前门的单顺序入口**。
//!
//! 本用例只证明 E1 台账的那一条：新 profile 身份/名字映射存在，配置文档经
//! `compile_config_document` 物化为**一个**顺序入口的 `CompiledSource`，且
//! 既有 profile 的裁剪身份不变（后者由 `plan981_grammar_scope` 的冻结表承担）。
//!
//! 它**不**证明配置文档能力已实现：顶层 `key: value`、`dep`/节点三形与
//! `cfg.*` 原语是 E2/E4 的增量；本文件对"非文档形状"只证明具名拒绝（不是
//! 静默丢弃）。

use ace_core::source::config_document::{
    compile_config_document, config_document_entry, CONFIG_DOCUMENT_DOMAIN,
    CONFIG_DOCUMENT_ENTRY_NAME, CONFIG_DOCUMENT_HIDDEN_PREFIX, CONFIG_DOCUMENT_VALUE_SCHEMA,
};
use ace_core::source::cst::{profile_from_slug, profile_grammar, profile_slug};
use ace_core::source::{parse_program_with, SourceProfile, CONFIG_DOCUMENT_SOURCE_PROFILE};

fn compile(src: &str) -> Result<ace_core::source::ConfigDocumentFront, Vec<String>> {
    compile_config_document(src, "pac.at", &[]).map_err(|ds| {
        ds.into_iter()
            .map(|d| format!("{}: {}", d.code, d.message))
            .collect()
    })
}

/// profile 身份：稳定版本串 ↔ 枚举 ↔ grammar slug 三向一致，且与 text-record
/// 的 grammar 身份隔离（新名字不寄生在旧裁剪上）。
#[test]
fn p982_config_document_profile_identity() {
    assert_eq!(
        SourceProfile::of(CONFIG_DOCUMENT_SOURCE_PROFILE),
        Some(SourceProfile::ConfigDocument),
        "the declared profile string must map back to the enum variant"
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
    // 稳定协议名（设计 §3.2）。
    assert_eq!(CONFIG_DOCUMENT_DOMAIN, "ace.config.document/v1");
    assert_eq!(CONFIG_DOCUMENT_VALUE_SCHEMA, "auto.config.value/1");
    assert!(CONFIG_DOCUMENT_SOURCE_PROFILE.starts_with("auto.source."));
    assert!(CONFIG_DOCUMENT_HIDDEN_PREFIX.starts_with("__cfg_"));
    // 新裁剪可编译，且与 text-record 的 GrammarId 不同（身份隔离）。
    let cd = profile_grammar(SourceProfile::ConfigDocument).expect("config-document grammar");
    let tr = profile_grammar(SourceProfile::TextRecord).expect("text-record grammar");
    assert_ne!(
        format!("{}", cd.program().identity.grammar_id.0),
        format!("{}", tr.program().identity.grammar_id.0),
        "a new profile trim must not share the text-record GrammarId"
    );
}

/// 文档语句序列物化为**一个**顺序入口：入口名稳定、语句保持源顺序、共享
/// verify 通过、profile 身份写进 `CompiledSource`。
#[test]
fn p982_config_document_materializes_one_sequential_entry() {
    let src = r#"
var port = "win32"
if port == "win32" { var x = 1 } else { var x = 2 }
"#;
    let compiled = compile(src).expect("statement-sequence document compiles");
    assert_eq!(compiled.entry, CONFIG_DOCUMENT_ENTRY_NAME);
    assert_eq!(compiled.compiled.profile, CONFIG_DOCUMENT_SOURCE_PROFILE);
    assert!(!compiled.compiled.input_hash.is_empty());
    assert!(!compiled.grammar_id.is_empty());
    // 顶层两条语句 → 一个顺序入口（不是两个隐藏函数）。
    assert_eq!(compiled.statements, 2, "both document statements live in the one entry");

    // 入口在 Checked HIR 里确实存在（共享 verify 的产物，不是仅 CST 绿）。
    let module = compiled
        .compiled
        .checked
        .bundle()
        .modules
        .first()
        .expect("one module");
    let found = module.defs.iter().any(|d| {
        matches!(
            &d.kind,
            ace_core::hir::DefKind::Function { name, .. } if name == &format!("sf_{CONFIG_DOCUMENT_ENTRY_NAME}")
        )
    });
    assert!(
        found,
        "the sequential entry must be a real checked function in the shared HIR"
    );

    // 精确 source map：语句 span 仍落在原源（不是重写后的坐标）。
    let program = parse_program_with(src, SourceProfile::ConfigDocument).expect("parse");
    let entry = config_document_entry(&program).expect("one entry");
    assert_eq!(entry.body.stmts.len(), 2);
    for stmt_span in statement_spans(entry) {
        assert!(
            stmt_span.start < src.len() && stmt_span.end <= src.len() && stmt_span.start < stmt_span.end,
            "statement span {stmt_span:?} must stay inside the original source"
        );
    }
    // 第一条语句的 span 起点 = `var` 关键字（源码顺序未被改写）。
    let first = statement_spans(entry)[0];
    assert_eq!(&src[first.start..first.start + 3], "var");
}

fn statement_spans(entry: &ace_core::source::ast::FnItem) -> Vec<ace_core::atom_text::Span> {
    use ace_core::source::ast::Stmt;
    entry
        .body
        .stmts
        .iter()
        .map(|s| match s {
            Stmt::Let { span, .. } => *span,
            Stmt::Assign { span, .. } => *span,
            Stmt::Return { span, .. } => *span,
            Stmt::If { span, .. } => *span,
            other => panic!("unexpected statement in the test document: {other:?}"),
        })
        .collect()
}

/// 非文档形状具名拒绝：声明项、多入口、空文档都不得静默成功。
#[test]
fn p982_config_document_rejects_non_document_shapes() {
    // 声明项（模块形态）不是文档。
    let errs = compile("fn helper() int { 1 }\nvar x = 1\n").expect_err("fn declarations are rejected");
    assert!(
        errs.iter().any(|e| e.starts_with("source.config-document/declaration")),
        "expected a named declaration rejection, got {errs:?}"
    );
    // 空文档不是成功求值。
    let errs = compile("").expect_err("an empty document is not a successful evaluation");
    assert!(
        errs.iter().any(|e| e.starts_with("source.config-document/empty")),
        "expected the named empty-document rejection, got {errs:?}"
    );
}
