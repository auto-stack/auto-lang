//! PLAN-927：七 profile 统一源码前门 —— 共享 CST/规则程序 → 严格 AST 投影。
//!
//! 架构（计划 §2，默认无环方向）：
//! `syntax-runtime ← syntax-grammar ← core::source::cst（本模块） ← 旧兼容面/消费者`。
//! Auto 专属投影（CST→AST、字面量解码、诊断映射）住进 core；`ace-syntax-core`
//! 保留薄重导出接缝（T-03）。语法事实单源于共享 `grammars/auto-source.at`
//! 的 profile 裁剪编译；解析只产语法事实，名称/类型/捕获/Checked 凭证仍由
//! resolve/typecheck 唯一决定。
//!
//! 诊断保真分层（§5.1：stage/code/span/message 顺序，不靠字符串重构原因）：
//! 1. **lex 层**——旧 lexer 全文件先行、首个词法事实致命。本层收集全部
//!    str/char token 的分类/解码事实 + 未知字符 error 叶，按 token 起点序
//!    取首个 = 与旧顺序逐字节一致。
//! 2. **结构层**——CST 文档序游走：error/missing 区域的模式映射 + 能力/
//!    形状拒绝（expr-stmt、脚本顶 return、entry-order、host-console 保留名、
//!    self/重复字段、`.mut` 接收者、文本方法括号等），首个按位置胜出。

mod frontdoor;
mod kinds;
mod lex;
mod project;
mod shift;
pub mod units;

// PLAN-963（V-G06）：包模块扁平程序的记录模板单态化由绑定层在收尾调用。
pub(crate) use project::monomorphize_package_program;

pub use frontdoor::{
    ct_grammar, instances_grammar, parse_closure_fragment_via_cst, parse_fragment_front,
    parse_front, parse_front_with_caps, parse_function_fragment_via_cst,
    parse_method_fragment_via_cst, parse_program_via_cst, profile_from_slug, profile_grammar,
    profile_slug, warm_profile, FrontBudget, FrontKind, FrontOk, FrontOutcome, FrontRequest,
    CT_CAPABILITY, FRONT_CAPS_VERSION, INSTANCES_CAPABILITY,
};
pub use kinds::SourceKinds;
pub use units::{
    content_sha, CatalogEvent, CatalogManifest, CatalogPin, EditError, EditReceipt, ManifestUnit,
    ProductionMeter, RestoreError, SealedDecl, SealedFn, SealedLambda, StagedCandidate,
    UnitCatalog, UnitKey, UnitKind, MANIFEST_SCHEMA,
};

use crate::atom_text::Diagnostic;

/// 旧合同兼容投影：typed 结果 → `Result<Program, Diagnostic>`（首个诊断）。
pub(crate) fn first_diag(diags: &[Diagnostic]) -> Diagnostic {
    diags
        .first()
        .cloned()
        .expect("diagnostic outcome carries at least one diagnostic")
}

// PLAN-950: metadata-preserving module projection is consumed only by the module binder.
pub use frontdoor::packages_ct_grammar;
pub use frontdoor::packages_grammar;
pub use frontdoor::CachedGrammar;
pub(crate) use frontdoor::{
    module_grammar, parse_module_fragment, parse_module_front, parse_packages_front,
    parse_packages_front_ct, project_module_front, project_packages_front,
};

pub(crate) use shift::{shift_fn_item, shift_fn_item_signed};
