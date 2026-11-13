//! PLAN-960 完整模块包语法层。与 950 有限模块（`modules.rs`）分立：
//! 本面消费 `ac.source.modules.packages/v1` 能力与 `packages-modules`
//! 裁剪，记录源事实（可见性/导入/重导出/常量/store/类型声明）；绑定、
//! Checked 构造与装配仍由各自权威负责。旧 `scalar/v1` 面逐字节不变。
use super::ast::Program;
use super::cst::FrontBudget;
use super::modules::SyntaxPiece;
use crate::atom_text::{Diagnostic, Span, Stage};
use std::collections::{BTreeMap, BTreeSet};

pub const PACKAGES_CAPABILITY: &str = "ac.source.modules.packages/v1";
pub const PACKAGES_BINDING_VERSION: &str = "source-module-binding/2";
/// PLAN-1043（T-03）：packages-ct 面显式能力（`#in` 编译期命名空间上下文
/// + `pub spec fn` 需求声明 + provider 装配）。携带本能力时 packages 前门
/// 以 `packages-ct` 裁剪编译；仅 `packages/v1` → plain packages-modules
/// 裁剪逐字节不变，`#`/`spec` 保持原拒绝面。
pub const IN_CAPABILITY: &str = "ac.source.cte.in/v1";
/// PLAN-1043（T-03）：唯一注册的编译期命名空间（事实表由统一声明/显式
/// target metadata 供给；未注册 namespace 具名拒）。
pub const SYS_NAMESPACE: &str = "sys";
/// PLAN-1043（T-03）：sys 事实表 schema（进入装配身份；事实域变更即失效）。
pub const SYS_FACTS_SCHEMA: &str = "ac.source.cte.in.sys/v1";

/// 一条多段路径导入 `use a.b[.c] [as x]` 或重导出 `pub use a.b[.c] [as x]`。
/// 2 段 = 本包 `mod.item`；3 段 = 跨包 `pkg.mod.item`；>3 段在绑定层拒绝。
#[derive(Clone, Debug)]
pub struct PackageImport {
    pub path: Vec<String>,
    pub alias: Option<String>,
    pub reexport: bool,
    pub span: Span,
}

impl PackageImport {
    /// 绑定到本模块命名空间的裸名（`as` 别名或路径末段）。
    pub fn bound_name(&self) -> &str {
        self.alias
            .as_deref()
            .unwrap_or_else(|| self.path.last().map(|s| s.as_str()).unwrap_or_default())
    }
}

/// PLAN-1043（T-03）：`#in` 内的编译期标量表达式（值域与 source-cte
/// `CtValue` 对齐：i32/bool；仅比较/逻辑算子——算术、括号分组与调用不在
/// 本面，越域由绑定具名拒）。
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CtExpr {
    Int(i32),
    Bool(bool),
    /// `current` / `sys.Windows`（head[.member]；绑定期按 namespace 事实解析）。
    Ref {
        head: String,
        member: Option<String>,
        span: Span,
    },
    Bin {
        op: CtBinOp,
        lhs: Box<CtExpr>,
        rhs: Box<CtExpr>,
        span: Span,
    },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CtBinOp {
    Eq,
    Ne,
    Lt,
    Le,
    Gt,
    Ge,
    And,
    Or,
}

impl CtExpr {
    pub fn span(&self) -> Span {
        match self {
            CtExpr::Int(_) | CtExpr::Bool(_) => Span { start: 0, end: 0 },
            CtExpr::Ref { span, .. } | CtExpr::Bin { span, .. } => *span,
        }
    }
}

/// PLAN-1043（T-03）：#in 单行/块内 use（`use path [: syms] [as alias]`；
/// 符号表形 = 逐符号导入，落位为普通 PackageImport；`pub use` 重导出形
/// 经 in-stmt 的 public 壳携带 reexport 标记）。
#[derive(Clone, Debug)]
pub struct CtUse {
    pub path: Vec<String>,
    pub syms: Option<Vec<String>>,
    pub alias: Option<String>,
    pub reexport: bool,
    pub span: Span,
}

/// PLAN-1043（T-03）：#in 内编译期语句。声明（fn/type/const/pub..）按原
/// Emit 规则落回本模块；控制层（is/if/for）与 use 是编译期，未选分支只
/// 保语法检查（Text 阶段）、不符号绑定/类型检查/链接。
#[derive(Clone, Debug)]
pub enum CtInStmt {
    Is {
        scrutinee: CtExpr,
        arms: Vec<CtInArm>,
        span: Span,
    },
    If {
        cond: CtExpr,
        then: Vec<CtInStmt>,
        els: Option<Vec<CtInStmt>>,
        span: Span,
    },
    /// `for <var> in <start>..<end> { .. }`（编译期计数迭代；var 在体内
    /// 条件域可见）。
    For {
        var: String,
        start: CtExpr,
        end: CtExpr,
        body: Vec<CtInStmt>,
        span: Span,
    },
    Use(CtUse),
    /// 声明（fn/type/const/store）；`public` = `pub` 壳（落位后进公共面）。
    Decl {
        item: super::ast::Item,
        public: bool,
    },
    /// `pub spec fn` 需求声明（平台条件需求）。
    Require(PackageRequire),
}

#[derive(Clone, Debug)]
pub struct CtInArm {
    /// 限定模式前段（`sys.Windows` 的 `sys`；须等于 #in 的 namespace）。
    pub qual: Option<String>,
    pub pattern: String,
    pub span: Span,
    pub body: CtArmBody,
}

#[derive(Clone, Debug)]
pub enum CtArmBody {
    /// 单行 `Arm -> use ...`。
    Use(CtUse),
    /// 块式 `Arm -> { ... }`。
    Block(Vec<CtInStmt>),
}

/// PLAN-1043（T-03）：一个 `#in <ns> ...` 项（投影保持结构；绑定期按
/// namespace 事实求值并落位）。
#[derive(Clone, Debug)]
pub struct PackageCtIn {
    pub namespace: String,
    pub span: Span,
    pub tail: CtInTail,
    /// 本 #in 在模块 items 序中的落位槽（选中声明按此插回）。
    pub decl_slot: usize,
    /// 本 #in 在模块 imports 序中的落位槽（选中 use 按此插回）。
    pub import_slot: usize,
}

#[derive(Clone, Debug)]
pub enum CtInTail {
    /// 单结构式 `#in ns is <expr> { arms }`。
    Is {
        scrutinee: CtExpr,
        arms: Vec<CtInArm>,
    },
    /// 块式 `#in ns { stmts }`。
    Block(Vec<CtInStmt>),
}

/// PLAN-1043（T-02）：`pub spec fn name(params) Ret` 公共必需实现声明
/// （bodyless 需求标记；签名域=参数个数+逐位类型+mode+返回类型，参数名
/// 不入合同）。由所选 provider 实现文件的同签名 `fn` 填充。
#[derive(Clone, Debug)]
pub struct PackageRequire {
    pub name: String,
    pub params: Vec<super::ast::Param>,
    pub ret: super::ast::TypeAnn,
    pub span: Span,
}

/// 一个包内模块的源事实（含可见性/导入/重导出与 const/store/type 声明清单；
/// 无法用只含函数的扁平 Program 表达的元数据都住在这里）。
#[derive(Clone, Debug)]
pub struct PackageSyntax {
    pub program: Program,
    /// 公共面：pub fn / pub type / pub const 名 + 重导出别名。
    pub exports: BTreeSet<String>,
    /// pub type 名（exports 的子集，用于类型可见性裁定）。
    pub type_exports: BTreeSet<String>,
    /// pub const 名（exports 的子集）。
    pub const_exports: BTreeSet<String>,
    pub imports: Vec<PackageImport>,
    pub grammar_id: String,
    /// 存储准入的源绑定；不是地址或授权。
    pub source_hash: String,
    pub pieces: Vec<SyntaxPiece>,
    /// 声明（fn/type/const/store）名到含可见性关键字的完整 span。
    pub declarations: BTreeMap<String, Span>,
    /// PLAN-1043（T-02）：`pub spec fn` bodyless 需求声明（packages-ct 面；
    /// plain 面恒空）。
    pub requires: Vec<PackageRequire>,
    /// PLAN-1043（T-03）：`#in` 编译期命名空间上下文项（packages-ct 面；
    /// plain 面恒空）。绑定期按事实求值，未选分支不进任何绑定。
    pub ct_ins: Vec<PackageCtIn>,
}

/// The explicit packages-module entry. Requires the packages capability and
/// never widens the finite scalar module capability's surface.
/// PLAN-1043（T-03）：携带 `ac.source.cte.in/v1` 时以 packages-ct 裁剪解析
/// （`#in`/`pub spec fn` 面可用）；否则 plain packages-modules 裁剪逐字节
/// 不变，`#`/`spec` 保持原拒绝面。
pub fn parse_package_source(
    src: &str,
    capabilities: &[String],
    budget: FrontBudget,
) -> Result<PackageSyntax, Vec<Diagnostic>> {
    let reason = if !capabilities.iter().any(|c| c == PACKAGES_CAPABILITY) {
        Some("package modules require explicit ac.source.modules.packages/v1")
    } else if capabilities
        .iter()
        .any(|c| c == super::modules::MODULES_CAPABILITY)
    {
        Some("the finite scalar module capability and package modules are separate surfaces")
    } else {
        None
    };
    if let Some(reason) = reason {
        return Err(vec![Diagnostic::new(
            Stage::Text,
            "source.module-capability",
            reason,
            Span { start: 0, end: 0 },
        )]);
    }
    if capabilities.iter().any(|c| c == IN_CAPABILITY) {
        super::cst::parse_packages_front_ct(src, budget)
    } else {
        super::cst::parse_packages_front(src, budget)
    }
}

/// The packages grammar identity for store admission (single shared grammar,
/// second frontend entry — never compiled by a third projector).
pub fn packages_grammar() -> Result<&'static super::cst::CachedGrammar, String> {
    super::cst::packages_grammar()
}

/// PLAN-1043（T-03）：packages-ct 裁剪 grammar 身份（IN 能力线的存储准入）。
pub fn packages_ct_grammar() -> Result<&'static super::cst::CachedGrammar, String> {
    super::cst::packages_ct_grammar()
}

/// Re-project content-verified stored CST declarations (no scanning/parsing;
/// grants no type credentials). Mirrors `modules::project_stored_module`.
pub fn project_stored_package(
    src: &str,
    trees: Vec<ace_syntax_runtime::cst::CstTree>,
    grammar_id: &str,
    budget: FrontBudget,
) -> Result<PackageSyntax, Vec<Diagnostic>> {
    let cached = packages_grammar().map_err(|e| {
        vec![Diagnostic::new(
            Stage::Text,
            "source.infra",
            e,
            Span { start: 0, end: 0 },
        )]
    })?;
    let actual = format!("{}", cached.program().identity.grammar_id.0);
    if actual != grammar_id || src.len() > budget.work_bytes {
        return Err(vec![Diagnostic::new(
            Stage::Text,
            "source.module-stale",
            "stored grammar/source budget mismatch",
            Span { start: 0, end: 0 },
        )]);
    }
    super::cst::project_packages_front(src, trees, None, false)
}

/// PLAN-1043（T-03）：packages-ct 存档 CST 的再投影（grammar_id 对
/// packages-ct 裁剪核验；与 plain 存档互不互认）。
pub fn project_stored_package_ct(
    src: &str,
    trees: Vec<ace_syntax_runtime::cst::CstTree>,
    grammar_id: &str,
    budget: FrontBudget,
) -> Result<PackageSyntax, Vec<Diagnostic>> {
    let cached = packages_ct_grammar().map_err(|e| {
        vec![Diagnostic::new(
            Stage::Text,
            "source.infra",
            e,
            Span { start: 0, end: 0 },
        )]
    })?;
    let actual = format!("{}", cached.program().identity.grammar_id.0);
    if actual != grammar_id || src.len() > budget.work_bytes {
        return Err(vec![Diagnostic::new(
            Stage::Text,
            "source.module-stale",
            "stored grammar/source budget mismatch",
            Span { start: 0, end: 0 },
        )]);
    }
    super::cst::project_packages_front(src, trees, None, true)
}

mod binding;
pub use binding::{
    bind_package_syntax, bind_package_syntax_assembled, bind_packages, bind_packages_assembled,
    fn_symbol, global_symbol, init_symbol, type_symbol, unit_key, BoundPackages, GlobalDecl,
    PackageDiagnostic, PackageImplLayer, PackageImplSyntax, PackageModuleInput, PackageSegment,
    GLOBAL_GET_PREFIX, GLOBAL_SET_PREFIX,
};
mod provider_sources;
pub use provider_sources::{
    classify_provider_file_name, select_provider_source, AssemblyBackend, PackageAssemblyContext,
    ProviderCandidates, ProviderChoice, ProviderFileKind, ProviderKind, ProviderSelection,
    ProviderSourceError, TargetOs, PROVIDER_SOURCE_SCHEMA,
};
mod stdlib;
pub use stdlib::{
    is_frozen_std_module, stdlib_inputs, unwired_reason, STDLIB_PACKAGE, STDLIB_RESERVED,
    STDLIB_UNWIRED, STDLIB_VERSION,
};

#[cfg(test)]
mod tests {
    use super::*;

    fn parse(src: &str) -> Result<PackageSyntax, Vec<Diagnostic>> {
        parse_package_source(
            src,
            &[PACKAGES_CAPABILITY.into()],
            FrontBudget::WORK_OWNED_64MIB,
        )
    }

    #[test]
    fn package_syntax_preserves_imports_aliases_reexports_and_items() {
        let src = "use std.answer\nuse math.util.clamp as cl\npub use lib.util.limits as lim\n\
                   pub const LIMIT int = 9\n\
                   pub fn main() int { return cl(1, 2) + std.answer() + LIMIT }\n\
                   pub type Point { x int\n y int\n fn norm() int { return .x } }\n\
                   var count int = 0\nfn helper() int { return 1 }";
        let m = parse(src).unwrap();
        assert_eq!(
            m.exports,
            BTreeSet::from(["main".into(), "Point".into(), "LIMIT".into(), "lim".into()])
        );
        assert_eq!(m.type_exports, BTreeSet::from(["Point".into()]));
        assert_eq!(m.const_exports, BTreeSet::from(["LIMIT".into()]));
        assert_eq!(m.imports.len(), 3);
        assert_eq!(
            m.imports[1].path,
            vec!["math".to_string(), "util".into(), "clamp".into()]
        );
        assert_eq!(m.imports[1].alias.as_deref(), Some("cl"));
        assert!(m.imports[2].reexport);
        assert!(m.declarations.contains_key("count"));
        // const/var items project into the program
        assert_eq!(m.program.items.len(), 5);
    }

    #[test]
    fn package_syntax_keeps_capability_gate_and_rejects_bad_forms() {
        assert_eq!(
            parse_package_source(
                "pub fn f() int { return 1 }",
                &[],
                FrontBudget::WORK_OWNED_64MIB
            )
            .unwrap_err()[0]
                .code,
            "source.module-capability"
        );
        // `pub var` has no public form (stores stay module-private)
        assert!(parse("pub var x int = 1").is_err());
        // imports must precede declarations
        assert_eq!(
            parse("fn main() int { return 1 }\nuse std.answer").unwrap_err()[0].code,
            "source.module-import-order"
        );
        // duplicate bound import names
        assert_eq!(
            parse("use a.f\nuse b.f\nfn main() int { return f() }").unwrap_err()[0].code,
            "source.module-duplicate-import"
        );
        // template seam: `<T>` parses under the packages face; the binder
        // owns the named `source.module-templates` rejection (PLAN-963).
        assert!(parse("pub fn f<T>(x T) T { return x }").is_ok());
    }
}
