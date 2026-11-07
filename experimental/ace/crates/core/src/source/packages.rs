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
        self.alias.as_deref().unwrap_or_else(|| {
            self.path.last().map(|s| s.as_str()).unwrap_or_default()
        })
    }
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
}

/// The explicit packages-module entry. Requires the packages capability and
/// never widens the finite scalar module capability's surface.
pub fn parse_package_source(
    src: &str,
    capabilities: &[String],
    budget: FrontBudget,
) -> Result<PackageSyntax, Vec<Diagnostic>> {
    let reason = if !capabilities
        .iter()
        .any(|c| c == PACKAGES_CAPABILITY)
    {
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
    super::cst::parse_packages_front(src, budget)
}

/// The packages grammar identity for store admission (single shared grammar,
/// second frontend entry — never compiled by a third projector).
pub fn packages_grammar() -> Result<&'static super::cst::CachedGrammar, String> {
    super::cst::packages_grammar()
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
    super::cst::project_packages_front(src, trees, None)
}

mod binding;
pub use binding::{
    bind_package_syntax, bind_packages, fn_symbol, global_symbol, init_symbol, type_symbol,
    unit_key, BoundPackages, GlobalDecl, PackageDiagnostic, PackageModuleInput, PackageSegment,
    GLOBAL_GET_PREFIX, GLOBAL_SET_PREFIX,
};

#[cfg(test)]
mod tests {
    use super::*;

    fn parse(src: &str) -> Result<PackageSyntax, Vec<Diagnostic>> {
        parse_package_source(src, &[PACKAGES_CAPABILITY.into()], FrontBudget::WORK_OWNED_64MIB)
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
            BTreeSet::from([
                "main".into(),
                "Point".into(),
                "LIMIT".into(),
                "lim".into()
            ])
        );
        assert_eq!(m.type_exports, BTreeSet::from(["Point".into()]));
        assert_eq!(m.const_exports, BTreeSet::from(["LIMIT".into()]));
        assert_eq!(m.imports.len(), 3);
        assert_eq!(m.imports[1].path, vec!["math".to_string(), "util".into(), "clamp".into()]);
        assert_eq!(m.imports[1].alias.as_deref(), Some("cl"));
        assert!(m.imports[2].reexport);
        assert!(m.declarations.contains_key("count"));
        // const/var items project into the program
        assert_eq!(m.program.items.len(), 5);
    }

    #[test]
    fn package_syntax_keeps_capability_gate_and_rejects_bad_forms() {
        assert_eq!(
            parse_package_source("pub fn f() int { return 1 }", &[], FrontBudget::WORK_OWNED_64MIB)
                .unwrap_err()[0]
                .code,
            "source.module-capability"
        );
        // `pub var` has no public form (stores stay module-private)
        assert!(parse("pub var x int = 1").is_err());
        // imports must precede declarations
        assert_eq!(
            parse("fn main() int { return 1 }\nuse std.answer")
                .unwrap_err()[0]
                .code,
            "source.module-import-order"
        );
        // duplicate bound import names
        assert_eq!(
            parse("use a.f\nuse b.f\nfn main() int { return f() }")
                .unwrap_err()[0]
                .code,
            "source.module-duplicate-import"
        );
        // template seam: `<T>` parses under the packages face; the binder
        // owns the named `source.module-templates` rejection (PLAN-963).
        assert!(parse("pub fn f<T>(x T) T { return x }").is_ok());
    }
}

