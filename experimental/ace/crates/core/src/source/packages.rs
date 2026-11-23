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
/// 本面，越域由绑定具名拒）。PLAN-966（T-02）：定义移入 `ast`（语句级 CT
/// 与项位 CT 共用同一谓词类型），此处保持原公开路径。
pub use super::ast::{CtBinOp, CtExpr};

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

/// PLAN-1043（T-03）/PLAN-966（T-02）：一个项位编译期选择项（投影保持结构；
/// 绑定期按 namespace 事实求值并落位）。`kind` 区分 `#in <ns> …` 与无
/// namespace 的 `#if/#is/#for`（谓词域=显式目标 sys 事实）。
#[derive(Clone, Debug)]
pub struct PackageCtItem {
    pub kind: CtItemKind,
    pub span: Span,
    /// 本 CT 项在模块 items 序中的落位槽（选中声明按此插回）。
    pub decl_slot: usize,
    /// 本 CT 项在模块 imports 序中的落位槽（选中 use 按此插回）。
    pub import_slot: usize,
}

/// PLAN-966（T-02）：项位编译期选择形态。`In` 建立 namespace 上下文
/// （1043）；`If`/`Is`/`For` 为无 namespace 的独立形，按 `sys` 显式目标
/// 事实求值（与 `#in sys` 块内同域同求值器）。
#[derive(Clone, Debug)]
pub enum CtItemKind {
    In {
        namespace: String,
        tail: CtInTail,
    },
    If {
        cond: CtExpr,
        then: Vec<CtInStmt>,
        els: Option<Vec<CtInStmt>>,
    },
    Is {
        scrutinee: CtExpr,
        arms: Vec<CtInArm>,
    },
    For {
        var: String,
        start: CtExpr,
        end: CtExpr,
        body: Vec<CtInStmt>,
    },
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
    pub source_metadata: super::cst::SourceMetadata,
    /// 公共面：pub fn / pub type / pub const 名 + 重导出别名。
    pub exports: BTreeSet<String>,
    /// pub type 名（exports 的子集，用于类型可见性裁定）。
    pub type_exports: BTreeSet<String>,
    /// pub const 名（exports 的子集）。
    pub const_exports: BTreeSet<String>,
    pub imports: Vec<PackageImport>,
    /// PLAN-980 ④-A：本模块源中**出现在导入项之外**的标识符拼写集合（注释/字符串/
    /// 导入项自身不计）。AGE 面用它做引用驱动（惰性）导入：未被引用的导入不解析、
    /// 不建模块边、不要求目标模块在场；其它面恒空并保持 eager 语义。
    pub referenced: BTreeSet<String>,
    /// Literal style declarations projected from the AGE package source.
    pub(crate) age_literal_styles: BTreeMap<String, String>,
    /// Public dependency recipes whose unsupported form is diagnosed when imported.
    pub(crate) age_deferred_styles: BTreeMap<String, Diagnostic>,
    /// Type-only references generated for a widget's inferred caller-side record.
    pub(crate) age_widget_record_types: BTreeMap<String, (String, super::ast::Ident)>,
    pub grammar_id: String,
    /// 存储准入的源绑定；不是地址或授权。
    pub source_hash: String,
    pub pieces: Vec<SyntaxPiece>,
    /// 声明（fn/type/const/store）名到含可见性关键字的完整 span。
    pub declarations: BTreeMap<String, Span>,
    /// PLAN-1043（T-02）：`pub spec fn` bodyless 需求声明（packages-ct 面；
    /// plain 面恒空）。
    pub requires: Vec<PackageRequire>,
    /// PLAN-1043（T-03）/PLAN-966（T-02）：项位编译期选择项（packages-ct
    /// 面；plain 面恒空）。绑定期按事实求值，未选分支不进任何绑定。
    pub ct_items: Vec<PackageCtItem>,
    /// PLAN-981（SOURCE 批，IF-981-MODULE）：本模块的 **AGE 领域候选**（未受信
    /// 结构 + 原 span + 表达式）与其文本叶的隐藏求值函数名（同序）。只在
    /// `age-package` 裁剪面有内容；其它面恒空——字段存在是为了让"领域行"与
    /// 普通包项**同一次解析**产出，而不是再拼一遍源。
    pub age_components: Vec<(
        crate::source::age_component::ComponentNode,
        Vec<(String, Span)>,
        Vec<crate::source::age_package::AgeSceneValueFn>,
    )>,
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
    if let Err(ds) = require_packages_capabilities(capabilities) {
        return Err(ds);
    }
    if capabilities.iter().any(|c| c == IN_CAPABILITY) {
        super::cst::parse_packages_front_ct(src, budget)
    } else {
        super::cst::parse_packages_front(src, budget)
    }
}

/// PLAN-946（T-02）：aggregates 域的包模块解析入口（packages-aggregates
/// 裁剪面 = packages-modules 全表面 + 集合族最小语句面 for-in/is + `?T`
/// 可选类型位 + `List<T>.member` 接收原子）。同一 packages 能力准入；由
/// aggregates packages 前门（PLAN-962 `av.aggregates/1` 准入线）调用。
pub fn parse_package_source_aggregates(
    src: &str,
    capabilities: &[String],
    budget: FrontBudget,
) -> Result<PackageSyntax, Vec<Diagnostic>> {
    if let Err(ds) = require_packages_capabilities(capabilities) {
        return Err(ds);
    }
    super::cst::parse_packages_front_aggregates(src, budget)
}

/// PLAN-972（T-02）：装配 aggregates 域的包模块解析入口
/// （packages-aggregates-ct 裁剪面 = packages-aggregates 全表面 +
/// packages-ct 装配项 `#in`/`pub spec fn` + `fn(T..) -> R` 闭包类型注解/
/// 闭包字面量实参——std 装配标准库与集合 HOF 公共算法的承载面）。同一
/// packages 能力准入；由装配 aggregates 前门（`ac.source.cte.in/v1` +
/// PLAN-962 `av.aggregates/1` 准入线）调用。
pub fn parse_package_source_aggregates_ct(
    src: &str,
    capabilities: &[String],
    budget: FrontBudget,
) -> Result<PackageSyntax, Vec<Diagnostic>> {
    if let Err(ds) = require_packages_capabilities(capabilities) {
        return Err(ds);
    }
    super::cst::parse_packages_front_aggregates_ct(src, budget)
}

fn require_packages_capabilities(capabilities: &[String]) -> Result<(), Vec<Diagnostic>> {
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
    match reason {
        Some(reason) => Err(vec![Diagnostic::new(
            Stage::Text,
            "source.module-capability",
            reason,
            Span { start: 0, end: 0 },
        )]),
        None => Ok(()),
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

/// PLAN-981（IF-981-MODULE）：`age-package` 裁剪 grammar 身份（AGE 包线的
/// 存储/恢复准入；与 plain/ct/aggregates 存档互不互认）。
pub fn packages_age_grammar() -> Result<&'static super::cst::CachedGrammar, String> {
    super::cst::packages_age_grammar()
}

/// PLAN-946（T-02）：packages-aggregates 裁剪 grammar 身份（aggregates
/// packages 线的存储准入；与 plain/ct 存档互不互认）。
pub fn packages_aggregates_grammar() -> Result<&'static super::cst::CachedGrammar, String> {
    super::cst::packages_aggregates_grammar()
}

/// PLAN-972（T-02）：packages-aggregates-ct 裁剪 grammar 身份（装配
/// aggregates 线的存储准入；与 plain/ct/aggregates 存档互不互认）。
pub fn packages_aggregates_ct_grammar() -> Result<&'static super::cst::CachedGrammar, String> {
    super::cst::packages_aggregates_ct_grammar()
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
    super::cst::project_packages_front(
        src,
        trees,
        None,
        super::cst::PackagesFace::Modules,
        &super::age_component::ComponentScenario {
            instances: 0,
            ops: Vec::new(),
        },
        0,
    )
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
    super::cst::project_packages_front(
        src,
        trees,
        None,
        super::cst::PackagesFace::Ct,
        &super::age_component::ComponentScenario {
            instances: 0,
            ops: Vec::new(),
        },
        0,
    )
}

/// PLAN-946（T-02）：packages-aggregates 存档 CST 的再投影（grammar_id 对
/// packages-aggregates 裁剪核验；与 plain/ct 存档互不互认）。
pub fn project_stored_package_aggregates(
    src: &str,
    trees: Vec<ace_syntax_runtime::cst::CstTree>,
    grammar_id: &str,
    budget: FrontBudget,
) -> Result<PackageSyntax, Vec<Diagnostic>> {
    let cached = packages_aggregates_grammar().map_err(|e| {
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
    super::cst::project_packages_front(
        src,
        trees,
        None,
        super::cst::PackagesFace::Aggregates,
        &super::age_component::ComponentScenario {
            instances: 0,
            ops: Vec::new(),
        },
        0,
    )
}

/// PLAN-981（IF-981-MODULE）：`age-package` 面的模块解析入口 —— packages-modules
/// 全表面 ∪ AGE 组件面，一次解析同时产出普通包项与领域候选。同一 packages
/// 能力准入；由 AGE 包前门调用。
pub fn parse_package_source_age(
    src: &str,
    capabilities: &[String],
    budget: FrontBudget,
    scenario: &super::age_component::ComponentScenario,
    node_base: usize,
) -> Result<PackageSyntax, Vec<Diagnostic>> {
    parse_package_source_age_with_style_dependencies(src, capabilities, budget, scenario, node_base, false, None)
}

pub(crate) struct AgeStoreImports<'a> {
    pub package: &'a str,
    pub stores: &'a BTreeMap<String, Vec<super::age_component::ComponentStore>>,
}

pub(crate) fn parse_package_source_age_with_style_dependencies(
    src: &str,
    capabilities: &[String],
    budget: FrontBudget,
    scenario: &super::age_component::ComponentScenario,
    node_base: usize,
    defer_styles: bool,
    store_imports: Option<AgeStoreImports<'_>>,
) -> Result<PackageSyntax, Vec<Diagnostic>> {
    if let Err(ds) = require_packages_capabilities(capabilities) {
        return Err(ds);
    }
    super::cst::parse_packages_front_age(src, budget, scenario, node_base, defer_styles, store_imports)
}

mod binding;
pub(crate) use binding::{assemble_standalone_auto, bind_packages_age_mode};
pub use binding::{
    bind_package_syntax, bind_package_syntax_aggregates, bind_package_syntax_assembled,
    bind_package_syntax_assembled_aggregates, bind_packages, bind_packages_age,
    bind_packages_aggregates, bind_packages_assembled, bind_packages_assembled_aggregates,
    bind_packages_auto, bind_packages_auto_with_context, fn_symbol, global_symbol, init_symbol,
    type_symbol, unit_key, BoundPackages, GlobalDecl, PackageDiagnostic, PackageImplLayer,
    PackageImplSyntax, PackageModuleInput, PackageSegment, PackagedAgeComponent, GLOBAL_GET_PREFIX,
    GLOBAL_SET_PREFIX, GLOBAL_TAKE_PREFIX,
};
mod provider_sources;
pub use provider_sources::{
    classify_provider_file_name, select_provider_source, AssemblyBackend, PackageAssemblyContext,
    ProviderCandidates, ProviderChoice, ProviderFileKind, ProviderKind, ProviderSelection,
    ProviderSourceError, TargetOs, PROVIDER_SOURCE_SCHEMA,
};
pub(crate) mod stdlib;
pub use stdlib::{
    is_frozen_std_assembled_module, is_frozen_std_module, is_frozen_std_sys_module,
    stdlib_aggregates_inputs, stdlib_assembled_files, stdlib_inputs,
    stdlib_sys_argv_process_source, stdlib_sys_files, unwired_reason, STDLIB_PACKAGE,
    STDLIB_RESERVED, STDLIB_UNWIRED, STDLIB_VERSION,
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
    fn age_package_function_accepts_a_single_for_condition() {
        use crate::source::ast::{Item, Stmt};

        let source = "fn count(limit int) int {\n    var i int = 0\n    for i < limit {\n        i = i + 1\n    }\n    return i\n}\n";
        let syntax = parse_package_source_age(
            source,
            &[PACKAGES_CAPABILITY.into()],
            FrontBudget::WORK_OWNED_64MIB,
            &super::super::age_component::ComponentScenario {
                instances: 0,
                ops: Vec::new(),
            },
            0,
        )
        .expect("a package function takes one condition before its loop body");
        let [Item::Fn(function)] = syntax.program.items.as_slice() else {
            panic!("expected one package function");
        };
        assert!(matches!(
            function.body.stmts.as_slice(),
            [Stmt::Let { .. }, Stmt::While { .. }, Stmt::Return { .. }]
        ));
    }

    #[test]
    fn age_package_handler_for_condition_ends_before_the_body() {
        use crate::source::age_component::{ComponentScenario, ComponentStmt};

        let source = r#"
widget Counter {
    msg { Count }
    model { var value int = 0 }
    view { text "counter" {} }
    on {
        .Count -> {
            let limit int = 2
            for .value < limit {
                .value = .value + 1
            }
        }
    }
}
"#;
        let syntax = parse_package_source_age(
            source,
            &[PACKAGES_CAPABILITY.into()],
            FrontBudget::WORK_OWNED_64MIB,
            &ComponentScenario { instances: 0, ops: Vec::new() },
            0,
        )
        .expect("the loop's opening brace belongs to its body");
        let (node, _, _) = &syntax.age_components[0];
        assert!(matches!(
            node.handlers[0].stmts.as_slice(),
            [ComponentStmt::Let { .. }, ComponentStmt::While { body, .. }]
                if matches!(body.as_slice(), [ComponentStmt::Assign(_)])
        ));
    }

    #[test]
    fn age_package_distinguishes_message_sends_from_checked_free_calls() {
        use crate::source::age_component::{ComponentScenario, ComponentStmt};
        use crate::source::age_h0::LegacyWidgetAlias;
        use crate::source::age_package::compile_age_package;

        let source = r#"
widget Counter {
    msg { Dispatch, Save, Update(int) }
    model { var value int = 0 }
    view { text "counter" {} }
    on {
        .Dispatch -> { .Update(2) }
        .Update(next) -> { .value = next }
        .Save -> {
            if .value > 0 {
                save_value(.value)
            }
        }
    }
}
"#;
        let syntax = parse_package_source_age(
            source,
            &[PACKAGES_CAPABILITY.into()],
            FrontBudget::WORK_OWNED_64MIB,
            &ComponentScenario { instances: 0, ops: Vec::new() },
            0,
        )
        .expect("project the message send and the free call");
        let (node, _, _) = &syntax.age_components[0];
        let dispatch = node.handlers.iter().find(|handler| handler.msg == "Dispatch").unwrap();
        assert!(!dispatch.body_foreign);
        assert!(matches!(
            dispatch.stmts.as_slice(),
            [ComponentStmt::Send { message, args, .. }] if message == "Update" && args.len() == 1
        ));
        let save = node.handlers.iter().find(|handler| handler.msg == "Save").unwrap();
        assert!(!save.body_foreign);
        assert!(matches!(
            save.stmts.as_slice(),
            [ComponentStmt::If { then, .. }] if matches!(then.as_slice(), [ComponentStmt::Effect { .. }])
        ));
        let unit = PackageModuleInput {
            package: "app".into(),
            version: "1.0.0".into(),
            id: "app".into(),
            source: source.into(),
            provider: None,
        };
        let Err(diags) = compile_age_package(
            &[unit], "app::app", LegacyWidgetAlias::Accept, &[PACKAGES_CAPABILITY.into()],
        ) else {
            panic!("the undeclared free call must be rejected");
        };
        assert!(diags.iter().any(|diag| diag.message.contains("save_value")), "{diags:?}");
    }

    #[test]
    fn age_package_projects_list_assignments_in_handler_blocks() {
        use crate::source::age_component::ComponentScenario;
        use crate::source::age_h0::LegacyWidgetAlias;
        use crate::source::age_package::compile_age_package_widget_runtime;
        use crate::source::ast::ExprKind;

        let source = r#"
widget Probe {
    msg { Direct, Branch, Loop, Attempt }
    model {
        var rows List<int> = [7]
        var labels = [{ text: "old" }]
        var marker int = 0
    }
    on {
        .Direct -> { .rows = [] }
        .Branch -> {
            if .marker == 0 {
                .rows = []
            }
        }
        .Loop -> {
            while .marker == 0 {
                .rows = []
                .marker = 1
            }
        }
        .Attempt -> {
            let caption = "new"
            try {
                .marker = 2
            } catch {
                .rows = []
                .labels = [{ text: caption }]
            }
        }
    }
    view {
        for label in .labels {
            text label.text {}
        }
    }
}
"#;
        let syntax = parse_package_source_age(
            source,
            &[PACKAGES_CAPABILITY.into()],
            FrontBudget::WORK_OWNED_64MIB,
            &ComponentScenario { instances: 0, ops: Vec::new() },
            0,
        )
        .expect("project the handlers");
        let (node, _, _) = &syntax.age_components[0];
        for name in ["Direct", "Branch", "Loop", "Attempt"] {
            let handler = node.handlers.iter().find(|handler| handler.msg == name).unwrap();
            assert!(handler.assigns.iter().any(|assign| {
                assign.field == "rows"
                    && matches!(&assign.rhs.kind, ExprKind::ArrayLit(items) if items.is_empty())
            }), "{name} must retain its list reset");
        }
        let attempt = node.handlers.iter().find(|handler| handler.msg == "Attempt").unwrap();
        let values = &attempt.assigns.iter().find(|assign| assign.field == "labels").unwrap().rhs;
        let ExprKind::ArrayLit(items) = &values.kind else {
            panic!("expected a record array: {values:?}");
        };
        let [item] = items.as_slice() else {
            panic!("expected a single record: {items:?}");
        };
        let ExprKind::ObjectLit { fields } = &item.kind else {
            panic!("expected a record literal: {item:?}");
        };
        let [field] = fields.as_slice() else {
            panic!("expected a single field: {fields:?}");
        };
        assert!(matches!(&field.value.kind, ExprKind::Var(local) if local.name == "caption"));
        let unit = PackageModuleInput {
            package: "app".into(),
            version: "1.0.0".into(),
            id: "app".into(),
            source: source.into(),
            provider: None,
        };
        compile_age_package_widget_runtime(
            &[unit], "app::app", LegacyWidgetAlias::Accept, &[PACKAGES_CAPABILITY.into()],
        )
        .expect("bind and check the record list assigned in catch");
    }

    #[test]
    fn age_package_legacy_list_default_types_match_their_elements() {
        use crate::source::age_component::ComponentScenario;
        use crate::source::age_package::widget_prop_type;
        use crate::source::ast::{TypeRef, TypeWord};

        let parse_default = |default: &str| {
            let source = format!("widget Pager (sizes: List = {default}) {{ view {{ text \"pages\" {{}} }} }}");
            parse_package_source_age(
                &source,
                &[PACKAGES_CAPABILITY.into()],
                FrontBudget::WORK_OWNED_64MIB,
                &ComponentScenario { instances: 0, ops: Vec::new() },
                0,
            )
        };
        for (default, element) in [
            ("[10, 25, 50]", TypeWord::I32),
            (r#"["small", "large"]"#, TypeWord::StrOwned),
            ("[]", TypeWord::StrOwned),
        ] {
            let syntax = parse_default(default).expect("homogeneous list default");
            let (node, _, _) = &syntax.age_components[0];
            assert!(matches!(
                widget_prop_type(&node.params[0]).map(|ty| ty.kind),
                Some(TypeRef::Generic { args, .. })
                    if args.len() == 1 && args[0].kind == TypeRef::Builtin(element)
            ));
        }
        let diagnostics = parse_default(r#"[10, "large"]"#).expect_err("mixed list default");
        assert_eq!(diagnostics[0].code, "source.age.widget-param-default");
    }

    #[test]
    fn age_package_bare_list_signatures_carry_dynamic_elements() {
        use crate::source::{age_component::ComponentScenario, age_h0::LegacyWidgetAlias};

        let source = r#"
fn make_rows() List {
    var rows List = []
    rows.push({ id: "folder", selected: true })
    return rows
}
fn count_rows(rows List) int {
    return rows.len()
}
widget App {
    computed { count => count_rows(make_rows()) }
    view { if .count > 0 { text "rows" {} } }
}
"#;
        let unit = PackageModuleInput {
            package: "app".into(),
            id: "main".into(),
            version: "1.0.0".into(),
            source: source.into(),
            provider: None,
        };
        crate::source::age_package::compile_age_package_with_scenario(
            &[unit],
            "app::main",
            LegacyWidgetAlias::Accept,
            &[PACKAGES_CAPABILITY.into()],
            &ComponentScenario { instances: 1, ops: Vec::new() },
        ).expect("old package signatures carry dynamic collection values");
    }

    #[test]
    fn age_package_infers_computed_conditional_results() {
        use crate::source::{age_component::ComponentScenario, age_h0::LegacyWidgetAlias};

        let source = r#"
widget App {
    model {
        var failed bool = false
        var total i32 = 42
    }
    computed {
        warning => if .failed { true } else if !.failed { false } else { true }
        label => if .failed { "error" } else { "ready" }
        count => if .failed { 1 } else { .total }
    }
    view {
        if .warning { text "failed" {} }
        if .count > 1 { text "more" {} }
        text .label {}
    }
}
"#;
        let compile = |source: String| {
            crate::source::age_package::compile_age_package_with_scenario(
                &[PackageModuleInput {
                    package: "app".into(),
                    id: "main".into(),
                    version: "1.0.0".into(),
                    source,
                    provider: None,
                }],
                "app::main",
                LegacyWidgetAlias::Accept,
                &[PACKAGES_CAPABILITY.into()],
                &ComponentScenario { instances: 1, ops: Vec::new() },
            )
        };
        compile(source.into()).expect("infer the Boolean and text results of a computed conditional");

        let diagnostics = compile(source.replace("else if !.failed { false }", "else if !.failed { 1 }"))
            .err().expect("computed branches with different types are refused");
        assert!(diagnostics.iter().any(|diagnostic| diagnostic.code == "source.inference"));
    }

    #[test]
    fn age_package_legacy_event_prop_forwards_declared_child_payload() {
        use crate::source::age_component::ComponentScenario;
        use crate::source::age_h0::LegacyWidgetAlias;
        use crate::source::age_package::compile_age_package_with_scenario;

        let compile = |binding: &str| {
            let parent = format!(r#"
widget App {{
    use {{ package: controls from "../controls" }}
    msg {{ Selected(str) }}
    model {{ var selected str = "" }}
    on {{ .Selected(value) -> {{ .selected = value }} }}
    view {{ Picker ({binding}) {{}} }}
}}
"#);
            let child = r#"
widget Picker (on_select: msg, on_retry: msg) {
    msg { "select"(str), "retry"(str) }
    model { var value str = "" }
    view { text "picker" {} }
}
"#;
            compile_age_package_with_scenario(
                &[
                    PackageModuleInput { package: "app".into(), version: "1.0.0".into(), id: "main".into(), source: parent, provider: None },
                    PackageModuleInput { package: "controls".into(), version: "1.0.0".into(), id: "picker".into(), source: child.into(), provider: None },
                ],
                "app::main",
                LegacyWidgetAlias::Accept,
                &[PACKAGES_CAPABILITY.into()],
                &ComponentScenario { instances: 1, ops: Vec::new() },
            )
        };
        let result = compile("on_select: .Selected($event)")
            .expect("bind legacy alias to the checked outbound event");
        let events = &result.first_widget_ref().expect("mounted child").events;
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].name, "select");
        assert_eq!(events[0].target, "Selected");

        let errors = compile("on_select: .Selected($event), onselect: .Selected($event)")
            .err().expect("the same child event cannot be bound twice");
        assert_eq!(errors[0].code, "source.age.child-event-duplicate");
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
