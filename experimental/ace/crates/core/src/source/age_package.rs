//! PLAN-981（SOURCE 批，IF-981-MODULE）：**AGE 多模块包前门**。
//!
//! 职责（§5 交接合同）：把"普通 Auto 模块图"与"AGE 领域候选"放进**同一次**
//! 解析、**同一**绑定图、**唯一** verify，产出可被 AV 与新 A2R 共同消费的
//! 结果；不新建第二解析器、第二绑定器或第二类型检查器。
//!
//! 数据流（与 979 单模块前门同构，只是驱动换成共享包绑定器）：
//! `每模块 CST → 普通包项 + AGE 候选（同一次解析）` →
//! `共享 bind_packages`（可见性/导入/初始化序/mangle）→
//! `共享 resolved→typecheck→emit→verify_text_record`（唯一 verify）→
//! `按模块读回 AGE 行的 Checked 结论 + 原文件 source map`。
//!
//! 语义边界（本切片**具名**）：
//! - 领域结构只在 `age-package` 裁剪里成立（`age-node-c-item` 规则存在）；
//! - 文本叶表达式脱糖为普通隐藏函数后由**共享** verify 检查（不二次解释）；
//! - 模块级 `const`/`var` 照 960 规则进 init 链；它们在 A2R 端的缺口由
//!   RUST 批具名关闭（本模块不假装已支持）；
//! - 任一模块拒绝即整包拒绝，且诊断归位到**原文件** span（不是合成偏移）。

use super::age_component::{ComponentNode, ComponentText, ComponentViewOut};
use super::age_h0::{text_diag, LegacyWidgetAlias, TextKind};
use super::ast::{
    Block, Expr, ExprKind, FnItem, Ident, Item, Receiver, Stmt, TypeAnn, TypeRef, TypeWord,
};
use super::packages::{bind_packages_age, PackagedAgeComponent, PackageModuleInput};
use super::cst::FrontBudget;
use super::SourceProfile;
use crate::atom_text::{Diagnostic, Span};
use crate::verify::CheckedModule;

/// PLAN-981：包面的**消费者配置**（不是源语言的一部分）——实例数与定向事件
/// 序列。与 979 组件场景是**同一个类型**（同一套脱糖口径，不复制一份）。
pub use super::age_component::ComponentScenario as AgePackageScenario;

/// 包面领域身份（进入描述与证据身份；与 grammar id 一起构成交接件身份）。
pub const AGE_PACKAGE_DOMAIN: &str = "ace.age.package/v1";
/// 隐藏求值函数前缀（用户顶层名含此前缀 = 具名冲突）。
pub const AGE_PACKAGE_HIDDEN_PREFIX: &str = "__age_p_hidden_";
/// 脱糖生成的保留名族（隐藏求值函数；用户顶层名不得占用）。
const RESERVED: [&str; 1] = [AGE_PACKAGE_HIDDEN_PREFIX];

/// 一个模块在包图里的**源坐标**：诊断归位与 source map 的唯一依据。
#[derive(Clone, Debug)]
pub struct AgeModuleSource {
    /// `pkg::mod` 键（与共享绑定器同一身份）。
    pub key: String,
    /// 该模块源文本的原字节长度。
    pub source_len: usize,
    /// 该模块在扁平坐标里的起点（绑定器给出）。
    pub flat_start: usize,
}

/// 受检的一段 AGE 文本叶（描述里的表达式标识 + 共享检查结论）。
#[derive(Clone, Debug)]
pub struct AgePackageText {
    pub role: String,
    pub span: Span,
    /// 共享 resolver/verify 判定的文本种类。
    pub kind: TextKind,
    /// 隐藏求值函数名与 def id（AV/A2R consumer 按它走共享后端）。
    pub fn_name: String,
    pub def_id: String,
    /// 所属模块键（跨模块 domain 行的归属事实）。
    pub module: String,
}

/// 受检的一个 AGE 组件（一个 node 声明）。
#[derive(Clone, Debug)]
pub struct AgePackageComponent {
    pub name: String,
    pub head_span: Span,
    pub legacy_widget: bool,
    /// model 字段（名字 + 类型拼写，按声明序）。
    pub fields: Vec<(String, String)>,
    /// `on <msg>` 消息名（按声明序；可能带 `.`，如 `Life.Tick`）。
    pub handler_msgs: Vec<String>,
    /// PLAN-981（IF-981-REF）：`pub node` 声明的可见性位。
    pub public: bool,
    /// PLAN-981（IF-981-FIELDS）：本组件的 `model` 初值是否真的进了求值链
    /// （即每个字段都有实例状态函数）。没有它，"model.init" 不得算已覆盖。
    pub initialized: bool,
    /// PLAN-981（IF-981-ORIGINAL）：作者 typed 参数（名字 + 类型拼写）。
    pub params: Vec<(String, String)>,
    /// PLAN-981（IF-981-REF）：本组件 view 里**已解析**的实例引用。
    pub refs: Vec<AgeResolvedRef>,
    pub texts: Vec<AgePackageText>,
    /// 声明所在模块键。
    pub module: String,
}

/// 包面版本化描述（AV/A2R 共同消费面）。
#[derive(Clone, Debug)]
pub struct AgePackageDescription {
    pub domain: &'static str,
    pub grammar_id: String,
    pub alias: LegacyWidgetAlias,
    pub entry: String,
    pub components: Vec<AgePackageComponent>,
    /// 参与本包的模块源事实（module graph identity 的可读面）。
    pub modules: Vec<AgeModuleSource>,
}

/// 一次 AGE 包编译结果。
#[derive(Debug)]
pub struct AgePackageCompilation {
    pub description: AgePackageDescription,
    pub checked: CheckedModule,
    /// 源函数名 → 绑定器 HIR 符号（脱糖函数与普通函数都在内）。
    pub symbols: std::collections::BTreeMap<String, String>,
    /// 包图身份（源 + 模块图 + grammar + 兼容配置）。
    pub input_hash: String,
    /// 脱糖后模块内容的身份（换领域行为即换它；源哈希不含脱糖）。
    pub desugar_hash: u64,
    pub atom_text: String,
}

impl AgePackageCompilation {
    /// 源名 → 绑定器给出的 HIR 符号（`sf_<mangled>`），供 AV/A2R 消费者按
    /// 同一事实取函数；查不到返回 None（不猜命名规则）。
    pub fn symbol_of(&self, source_name: &str) -> Option<String> {
        self.symbols.get(source_name).cloned()
    }

    /// 证据身份（与 979 领域接面同元组：id/version/grammar/源/兼容配置）。
    pub fn identity(&self) -> super::domain::DomainIdentity {
        super::domain::DomainIdentity {
            id: AGE_PACKAGE_DOMAIN,
            version: super::domain::DomainVersion { major: 1, minor: 0 },
            grammar_id: self.description.grammar_id.clone(),
            input_hash: self.input_hash.clone(),
            compat: matches!(self.description.alias, LegacyWidgetAlias::Accept),
        }
    }
}

/// AGE 包前门：多模块源 → 同一次解析 → 共享绑定 → 唯一 verify → 版本化描述。
///
/// `inputs` 是宿主提供的模块闭包（无文件系统发现/源拼接）；`entry` 是
/// `pkg::mod` 形式的入口模块键（与 960 同一身份格式）。
pub fn compile_age_package(
    inputs: &[PackageModuleInput],
    entry: &str,
    alias: LegacyWidgetAlias,
    capabilities: &[String],
) -> Result<AgePackageCompilation, Vec<Diagnostic>> {
    compile_age_package_with_scenario(
        inputs,
        entry,
        alias,
        capabilities,
        &crate::source::age_component::ComponentScenario {
            instances: 0,
            ops: Vec::new(),
        },
    )
}

/// 同 [`compile_age_package`]，但显式给出**消费者场景**（实例数与定向事件）。
///
/// 场景不是源语言的一部分：它决定为哪些实例生成状态函数/求值函数，从而让
/// AV/A2R 消费者观察真实值（两实例 2:1 事件、N 次 Tick）。
pub fn compile_age_package_with_scenario(
    inputs: &[PackageModuleInput],
    entry: &str,
    alias: LegacyWidgetAlias,
    capabilities: &[String],
    scenario: &crate::source::age_component::ComponentScenario,
) -> Result<AgePackageCompilation, Vec<Diagnostic>> {
    // 1) 共享包绑定：包模块面 ∪ AGE 面**一次**解析；可见性/导入/初始化序/
    //    mangle 全部由 960 的绑定器裁定（本模块不重做任何一项）。
    let product = bind_packages_age(
        inputs,
        entry,
        capabilities,
        FrontBudget::WORK_OWNED_64MIB,
        scenario,
    )
    .map_err(|d| vec![d.diagnostic])?;

    // 2) 兼容闸：旧 `widget` 声明头只在显式开启时准入（同 979 口径）。
    if !alias.is_accept() {
        if let Some(c) = product
            .age_components
            .iter()
            .find(|c| c.node.legacy_widget)
        {
            return Err(vec![text_diag(
                "source.age.legacy-widget",
                "legacy `widget` declarations require the AGE legacy-widget compat config; \
                 write `node` or enable the compat config explicitly",
                flat_to_module(&product.sources, c.node.head_span),
            )]);
        }
    }

    // 3) 隐藏名冲突：任何用户顶层名含保留前缀即具名冲突（不静默遮蔽）。
    //    排除本前门自己注入的隐藏函数——判据是"**用户**是否占用了保留名"。
    let injected: Vec<&str> = product
        .age_components
        .iter()
        .flat_map(|c| c.text_fns.iter().map(|(n, _)| n.as_str()))
        .collect();
    if let Some(d) = hidden_name_conflict(&product.bound, &injected) {
        return Err(vec![d]);
    }

    // 4) 唯一 verify：几何域 = text-record（与 979/960 同一条链）。
    let resolved = crate::source::resolve_with(
        &product.bound.program,
        capabilities,
        SourceProfile::TextRecord,
    )
    .map_err(|ds| relabel_package_diags(locate_all(&product.bound, ds)))?;
    crate::source::typecheck_with(&resolved, SourceProfile::TextRecord)
        .map_err(|ds| relabel_package_diags(locate_all(&product.bound, ds)))?;
    // PLAN-981：AGE 作者面发射（f32 类型表等按本面发射；协议身份仍是包文档）。
    let parts = crate::source::adapter::emit_resolved_packages_parts_profile(
        &resolved,
        &product.bound,
        "packages",
        SourceProfile::AgePackage,
    )
    .map_err(|ds| relabel_package_diags(locate_all(&product.bound, ds)))?;
    let input_hash = parts.input_hash.clone();
    let desugar_hash = crate::source::cst::content_sha(parts.atom_text.as_bytes());
    let checked = crate::verify::verify_text_record(parts.bundle)
        .map_err(|ds| relabel_package_diags(locate_all(&product.bound, ds)))?;

    // 4b) PLAN-981（IF-981-REF）：解析 view 里的节点实例引用。可见性判据复用
    //     共享绑定器的 exports/imports 事实；失败即整包具名拒绝（带原模块 span）。
    let resolved_refs = resolve_refs(&product.bound, &product.age_components)?;

    // 5) 从 Checked HIR 读回每条行为的结论（共享 resolver 的判定，不是本层猜的）。
    //    隐藏函数的 HIR 名已被绑定器 mangle（`sf_p3_appm4_main___age_p_hidden_0`），
    //    故用共享 `origins` 表把源名映到真实符号再查——不猜命名规则。
    let components =
        read_back_components(&product.age_components, &checked, &product.bound, &resolved_refs)?;

    Ok(AgePackageCompilation {
        description: AgePackageDescription {
            domain: AGE_PACKAGE_DOMAIN,
            grammar_id: product.grammar_id.clone(),
            alias,
            entry: entry.to_string(),
            components,
            modules: product.sources.clone(),
        },
        checked,
        symbols: product
            .bound
            .origins
            .iter()
            .map(|(sym, (_, _, original))| (original.clone(), sym.clone()))
            .collect(),
        input_hash,
        desugar_hash,
        atom_text: parts.atom_text,
    })
}

/// 把扁平坐标的诊断归位到**原文件**（保模块内偏移，标出所属模块）。
fn relabel_package_diags(mut diags: Vec<Diagnostic>) -> Vec<Diagnostic> {
    for d in &mut diags {
        if d.code == "source.age.text-type" {
            d.message = format!(
                "an AGE value expression must be text (`str` or `Str`); the shared checker \
                 rejected it: {}",
                d.message
            );
        }
    }
    diags
}

/// 扁平坐标的诊断归位到所属模块（保模块内偏移）——跨模块错误不报合成偏移。
fn locate_all(
    bound: &super::packages::BoundPackages,
    ds: Vec<Diagnostic>,
) -> Vec<Diagnostic> {
    ds.into_iter().map(|d| bound.locate(d).diagnostic).collect()
}

/// 扁平坐标 → 原模块坐标（诊断/描述都按原文件给定位）。
fn flat_to_module(sources: &[AgeModuleSource], span: Span) -> Span {
    let src = sources
        .iter()
        .rev()
        .find(|s| span.start >= s.flat_start)
        .or_else(|| sources.first());
    match src {
        Some(s) => Span {
            start: span.start.saturating_sub(s.flat_start),
            end: span.end.saturating_sub(s.flat_start),
        },
        None => span,
    }
}

/// 用户顶层名与保留前缀的冲突（具名拒绝，不静默遮蔽）。
///
/// 按 `units` 里的**源名**判：绑定器已把程序项 mangle 成包/模块限定符号
/// （必然含前缀片段），拿 mangle 后的名字判会把自己的隐藏函数判成冲突。
fn hidden_name_conflict(
    bound: &super::packages::BoundPackages,
    injected: &[&str],
) -> Option<Diagnostic> {
    for unit in bound.units.values() {
        for item in &unit.program.items {
            let user = match item {
                Item::Fn(f) => Some(&f.name),
                Item::Type(t) => Some(&t.name),
                Item::Enum(e) => Some(&e.name),
                Item::Const(c) => Some(&c.name),
                Item::Store(s) => Some(&s.name),
                Item::Task(t) => Some(&t.name),
                Item::UseRs(_) => None,
            };
            if let Some(u) = user {
                if injected.contains(&u.name.as_str()) {
                    continue;
                }
                for reserved in RESERVED {
                    if u.name.contains(reserved) {
                        return Some(text_diag(
                            "source.age.hidden-name",
                            format!(
                                "`{}` contains `{reserved}` which is reserved for AGE package \
                                 desugaring (value evaluation)",
                                u.name
                            ),
                            u.span,
                        ));
                    }
                }
            }
        }
    }
    None
}

/// 从 Checked HIR 精确读回每个文本叶的 def id 与文本种类。
fn read_back_components(
    packaged: &[PackagedAgeComponent],
    checked: &CheckedModule,
    bound: &super::packages::BoundPackages,
    refs: &std::collections::BTreeMap<String, Vec<AgeResolvedRef>>,
) -> Result<Vec<AgePackageComponent>, Vec<Diagnostic>> {
    let mut out = Vec::new();
    for p in packaged {
        let mut texts = Vec::new();
        for (fn_name, orig_span) in &p.text_fns {
            // 源名 → 绑定器给出的真实 HIR 符号（962/960 的 origins 表是唯一事实源），
            // 再按 adapter 的唯一约定取 `sf_<symbol>` 的 HIR 名（同一个 helper，不猜）。
            let symbol = bound
                .origins
                .iter()
                .find(|(_, (pkg, module, original))| {
                    original == fn_name
                        && super::packages::unit_key(pkg, module) == p.module
                })
                .map(|(sym, _)| sym.clone());
            let conclusion = symbol.as_deref().and_then(|sym| {
                super::domain::hidden_conclusion(checked, sym)
            });
            let Some((def_id, kind)) = conclusion else {
                return Err(vec![text_diag(
                    "source.age.text-type",
                    "an AGE value expression must be text (`str` or `Str`); the shared type \
                     checker rejected this expression",
                    *orig_span,
                )]);
            };
            texts.push(AgePackageText {
                role: p.node.view.role.clone(),
                span: *orig_span,
                kind,
                fn_name: fn_name.clone(),
                def_id,
                module: p.module.clone(),
            });
        }
        out.push(AgePackageComponent {
            name: p.node.name.name.clone(),
            head_span: p.node.head_span,
            legacy_widget: p.node.legacy_widget,
            fields: p
                .node
                .fields
                .iter()
                .map(|f| (f.name.clone(), f.ty.clone()))
                .collect(),
            handler_msgs: p.node.handlers.iter().map(|h| h.msg.clone()).collect(),
            public: p.node.public,
            initialized: state_functions_present(bound, &p.module, &p.node.fields),
            params: p
                .node
                .params
                .iter()
                .map(|q| (q.name.clone(), q.ty.clone()))
                .collect(),
            refs: refs.get(&p.module).cloned().unwrap_or_default(),
            texts,
            module: p.module.clone(),
        });
    }
    Ok(out)
}

/// 隐藏求值函数的**源形状**：`fn <prefix><n>() Str { return (<expr>).clone() }`。
///
/// 这是普通 Auto 源，交给唯一 verify；本函数不改判据、不建第二套检查。
pub(crate) fn hidden_fn(name: &str, expr: &Expr, span: Span) -> FnItem {
    let clone = Expr {
        span,
        kind: ExprKind::Method {
            recv: Box::new(expr.clone()),
            name: Ident {
                name: "clone".to_string(),
                span,
            },
            args: Vec::new(),
        },
    };
    FnItem {
        name: Ident {
            name: name.to_string(),
            span,
        },
        type_params: Vec::new(),
        params: Vec::new(),
        ret: TypeAnn {
            kind: TypeRef::Builtin(TypeWord::StrOwned),
            span,
        },
        body: Block {
            stmts: vec![Stmt::Return { value: clone, span }],
            span,
        },
        span,
        receiver: Receiver::Free,
        owner: None,
        script: false,
    }
}

/// PLAN-981：把场景里的 `Tick` 解析成**作者实际声明**的生命周期消息拼写。
///
/// 979 的组件场景把 `Life.Tick` 建模为保留消息 `Tick`（"运行身份归 981"）——
/// 本条就是那个运行身份：包面接受 `Tick` 与 `Life.Tick` 两种拼写，并归一到本
/// node 真正声明的那一个；两者都没声明才保持原样交给共享检查具名拒绝。
pub(crate) fn resolve_tick_spelling(
    scenario: &crate::source::age_component::ComponentScenario,
    node: &ComponentNode,
) -> crate::source::age_component::ComponentScenario {
    use crate::source::age_component::{ScenarioOp, ComponentScenario};
    let declared = node
        .handlers
        .iter()
        .find(|h| h.msg == "Life.Tick")
        .or_else(|| node.handlers.iter().find(|h| h.msg == "Tick"))
        .map(|h| h.msg.clone());
    let Some(declared) = declared else {
        return scenario.clone();
    };
    let ops = scenario
        .ops
        .iter()
        .map(|op| match op {
            ScenarioOp::Tick { instance } => ScenarioOp::Send {
                instance: *instance,
                msg: declared.clone(),
            },
            other => other.clone(),
        })
        .collect();
    ComponentScenario {
        instances: scenario.instances,
        ops,
    }
}

/// PLAN-981（IF-981-REF）：一条**已解析**的节点实例引用。
#[derive(Clone, Debug)]
pub struct AgeResolvedRef {
    /// 作者写的类型拼写。
    pub ty_spelling: String,
    /// 解析到的模块键（`pkg::mod`）。
    pub module: String,
    /// 该模块里 node 的源名。
    pub node: String,
    /// 实例名。
    pub name: String,
    /// 引用处原 span（原模块内）。
    pub span: Span,
}

/// PLAN-981（IF-981-REF）：实例引用解析失败的具名原因。
///
/// 三种失败必须**互斥可辨**（979 V-10 要求"是引用/可见性诊断而非偶然类型错"）：
/// 类型不存在、类型存在但不可见、以及**不是 node**（声明了但不是 AGE 节点）。
fn ref_failure_code(kind: &RefFailure) -> &'static str {
    match kind {
        RefFailure::Undeclared => "source.age.ref-undeclared",
        RefFailure::Invisible => "source.age.ref-invisible",
        RefFailure::NotANode => "source.age.ref-not-a-node",
    }
}

#[derive(Clone, Copy, Debug)]
enum RefFailure {
    /// 本模块与已导入符号里都没有这个名字。
    Undeclared,
    /// 名字存在，但声明在别的模块且未 `pub`（或未导入）。
    Invisible,
    /// 名字是有可见性的类型，但它不是 AGE node 声明。
    NotANode,
}

/// 解析一个模块里的全部实例引用（IF-981-REF 的核心判据）。
///
/// 可见性规则**完全复用** 960 绑定器的事实：`unit.exports`（公共面）与本模块
/// `imports`（导入绑定）。本函数只做"按引用查符号"，不建第二套命名/可见性规则。
fn resolve_refs(
    bound: &super::packages::BoundPackages,
    packaged: &[PackagedAgeComponent],
) -> Result<std::collections::BTreeMap<String, Vec<AgeResolvedRef>>, Vec<Diagnostic>> {
    // 每个模块里"本模块声明的 node"与"公共导出的 node"。
    let mut declared: std::collections::BTreeMap<String, Vec<String>> =
        std::collections::BTreeMap::new();
    let mut public_nodes: std::collections::BTreeMap<String, Vec<String>> =
        std::collections::BTreeMap::new();
    let mut all_type_names: std::collections::BTreeMap<String, Vec<String>> =
        std::collections::BTreeMap::new();
    for p in packaged {
        declared
            .entry(p.module.clone())
            .or_default()
            .push(p.node.name.name.clone());
        if p.node.public {
            public_nodes
                .entry(p.module.clone())
                .or_default()
                .push(p.node.name.name.clone());
        }
    }
    for (key, unit) in &bound.units {
        for item in &unit.program.items {
            if let Item::Type(t) = item {
                all_type_names
                    .entry(key.clone())
                    .or_default()
                    .push(t.name.name.clone());
            }
        }
    }

    let mut out: std::collections::BTreeMap<String, Vec<AgeResolvedRef>> =
        std::collections::BTreeMap::new();
    for p in packaged {
        let mut resolved = Vec::new();
        let unit = bound.units.get(&p.module);
        for r in &p.node.view.node_refs {
            let want = &r.ty.name;
            // 1) 本模块声明优先（不需要导入；可见性仅约束**跨模块**）。
            let local = declared
                .get(&p.module)
                .map(|v| v.iter().any(|n| n == want))
                .unwrap_or(false);
            if local {
                resolved.push(AgeResolvedRef {
                    ty_spelling: want.clone(),
                    module: p.module.clone(),
                    node: want.clone(),
                    name: r.name.name.clone(),
                    span: r.span,
                });
                continue;
            }
            // 2) 别的模块里的 **可见** node 声明：按 node 名解析（AGE 领域引用
            //    走节点命名空间，不借用普通 use 导入的值/类型命名空间）。
            let public_hit = public_nodes
                .iter()
                .find(|(k, v)| *k != &p.module && v.iter().any(|n| n == want))
                .map(|(k, _)| k.clone());
            if let Some(target_key) = public_hit {
                resolved.push(AgeResolvedRef {
                    ty_spelling: want.clone(),
                    module: target_key,
                    node: want.clone(),
                    name: r.name.name.clone(),
                    span: r.span,
                });
                continue;
            }
            // 3) 导入表：`use <mod>.<X>` 存在但目标不是可见 node → 具名区分。
            let imported = unit.and_then(|u| {
                u.imports
                    .iter()
                    .find(|i| i.bound_name() == want)
                    .map(|i| i.path.clone())
            });
            match imported {
                Some(path) => {
                    let (target_key, item) = match path.len() {
                        2 => (
                            super::packages::unit_key(
                                p.module.split_once("::").map(|(pk, _)| pk).unwrap_or(""),
                                &path[0],
                            ),
                            path[1].clone(),
                        ),
                        3 => (super::packages::unit_key(&path[0], &path[1]), path[2].clone()),
                        _ => (String::new(), want.clone()),
                    };
                    let is_plain_type = all_type_names
                        .get(&target_key)
                        .map(|v| v.iter().any(|n| n == &item))
                        .unwrap_or(false);
                    let visible = public_nodes
                        .get(&target_key)
                        .map(|v| v.iter().any(|n| n == &item))
                        .unwrap_or(false);
                    if is_plain_type && !visible {
                        // 名字可见且是类型，但它不是 AGE node —— 与"不可见"分开报。
                        return Err(vec![ref_diag(
                            RefFailure::NotANode,
                            format!(
                                "`{want}` resolves to a declared type, not an AGE `node`; an \
                                 instance reference must name a node declaration"
                            ),
                            flat_to_module(&bound_sources(bound), r.span),
                        )]);
                    }
                    if visible {
                        resolved.push(AgeResolvedRef {
                            ty_spelling: want.clone(),
                            module: target_key,
                            node: item,
                            name: r.name.name.clone(),
                            span: r.span,
                        });
                    } else {
                        return Err(vec![ref_diag(
                            RefFailure::Invisible,
                            format!(
                                "`{want}` is imported from `{target_key}` but is not a `pub node` \
                                 declaration there; an instance reference needs a visible AGE node"
                            ),
                            flat_to_module(&bound_sources(bound), r.span),
                        )]);
                    }
                }
                None => {
                    // 3) 既非本模块声明、又无导入：区分"别处有但不可见"与"根本不存在"。
                    let seen_elsewhere = declared
                        .iter()
                        .any(|(k, v)| k != &p.module && v.iter().any(|n| n == want))
                        || public_nodes
                            .iter()
                            .any(|(k, v)| k != &p.module && v.iter().any(|n| n == want));
                    let is_type_elsewhere = all_type_names
                        .iter()
                        .any(|(k, v)| k != &p.module && v.iter().any(|n| n == want));
                    if is_type_elsewhere {
                        return Err(vec![ref_diag(
                            RefFailure::NotANode,
                            format!(
                                "`{want}` is a declared type but not an AGE `node`; an instance \
                                 reference must name a node declaration"
                            ),
                            flat_to_module(&bound_sources(bound), r.span),
                        )]);
                    }
                    let kind = if seen_elsewhere {
                        RefFailure::Invisible
                    } else {
                        RefFailure::Undeclared
                    };
                    return Err(vec![ref_diag(
                        kind,
                        format!(
                            "no `node {want}` is declared here or imported; an instance reference \
                             must name a visible node declaration"
                        ),
                        flat_to_module(&bound_sources(bound), r.span),
                    )]);
                }
            }
        }
        out.insert(p.module.clone(), resolved);
    }
    Ok(out)
}

/// `model` 初值事实是否成立：该模块里每个字段都存在实例 0 的状态函数。
///
/// 用绑定器自己的 `origins` 表查（源名 → 符号），不猜命名规则。
fn state_functions_present(
    bound: &super::packages::BoundPackages,
    module: &str,
    fields: &[crate::source::age_component::ComponentField],
) -> bool {
    if fields.is_empty() {
        return false;
    }
    fields.iter().all(|f| {
        let want = format!("__age_state_0_{}", f.name);
        bound
            .origins
            .iter()
            .any(|(_, (pkg, m, original))| {
                original == &want && super::packages::unit_key(pkg, m) == module
            })
    })
}

fn ref_diag(kind: RefFailure, message: String, span: Span) -> Diagnostic {
    text_diag(ref_failure_code(&kind), message, span)
}

/// 绑定器给出的模块源坐标（供诊断归位；与描述面同源）。
fn bound_sources(
    bound: &super::packages::BoundPackages,
) -> Vec<AgeModuleSource> {
    bound
        .segments
        .iter()
        .map(|s| AgeModuleSource {
            key: super::packages::unit_key(&s.package, &s.module),
            source_len: s.source.len(),
            flat_start: s.start,
        })
        .collect()
}

/// PLAN-981（IF-981-FIELDS）：AGE 作者语言的**构造清单**（required rows）。
///
/// 每一行是"作者可写的一个构造 → 它在受检描述里必须留下的 typed 事实 → 谁消费
/// 它"。清单是**数据**：真实的覆盖检查（[`age_construct_coverage`]）逐行核对，
/// 删掉一行或删掉对应的投影都会让检查变红（见 T-01 冻结的删行反控用例）。
///
/// 与 980 的 IF-980-FIELDS（UI 组件字段表）**取并集**，不互相替代；本表只覆盖
/// AGE node 这一侧的构造（`node/model/on/view` 及其成员与视图引用）。
pub struct AgeConstruct {
    /// 构造键（稳定、可追踪；不随拼写改名）。
    pub key: &'static str,
    /// 作者实际写的语法形状（文档/诊断用）。
    pub spelling: &'static str,
    /// 该构造在受检产物里必须留下的 typed 事实。
    pub fact: &'static str,
    /// 消费该事实的下游 owner（本批 = 包前门自己的描述面）。
    pub consumer: &'static str,
    /// 该构造由哪一族规范条款要求（AGE-L / AG-A / V-DETAIL）。
    pub requirement: &'static str,
}

/// 兼容变体：旧 `widget` 声明头。**不属于** canonical 源必须覆盖的构造集，
/// 但它是同一 Schema 的合法别名，故单独登记、单独验证（compat 开/关各一态）。
pub const AGE_COMPAT_CONSTRUCTS: &[AgeConstruct] = &[AgeConstruct {
    key: "node.legacy-widget",
    spelling: "widget <Name> { … }",
    fact: "the same node row, flagged as the legacy compat head",
    consumer: "age_package (compat gate)",
    requirement: "AC-NODE-CHAIN",
}];

/// AGE 构造清单（本批实际迁入的范围；canonical 源必须逐行覆盖）。
pub const AGE_CONSTRUCTS: &[AgeConstruct] = &[
    AgeConstruct {
        key: "node",
        spelling: "node <Name> { … }",
        fact: "a component row carrying the declared node name + original head span",
        consumer: "age_package::AgePackageDescription.components",
        requirement: "AGE-L01",
    },
    AgeConstruct {
        key: "model",
        spelling: "model { <field> <type> = <init> }",
        fact: "per-field (name, type) rows in declaration order",
        consumer: "age_package::AgePackageComponent.fields",
        requirement: "AGE-L01",
    },
    AgeConstruct {
        key: "model.init",
        spelling: "<field> <type> = <init>",
        fact: "a per-instance state function built from the initializer",
        consumer: "av::age_package (instance_states)",
        requirement: "AGE-L01",
    },
    AgeConstruct {
        key: "on",
        spelling: "on <Msg> { <field> = <expr> }",
        fact: "the handler message names in declaration order",
        consumer: "age_package::AgePackageComponent.handler_msgs",
        requirement: "AGE-L01",
    },
    AgeConstruct {
        key: "on.lifecycle",
        spelling: "on Life.Tick { … }",
        fact: "the qualified lifecycle message, kept in author spelling",
        consumer: "av::age_package (Tick resolution)",
        requirement: "AGE-L05",
    },
    AgeConstruct {
        key: "view",
        spelling: "view { … }",
        fact: "the view role + its member rows",
        consumer: "age_package::AgePackageComponent.texts",
        requirement: "AGE-L02",
    },
    AgeConstruct {
        key: "view.text",
        spelling: "text <expression> {}",
        fact: "a checked text value function (def id + kind)",
        consumer: "av::age_package / a2x (value functions)",
        requirement: "AGE-L02",
    },
    AgeConstruct {
        key: "view.node-ref",
        spelling: "<Type> <name> {}",
        fact: "a resolved instance-reference row (declaring module + node)",
        consumer: "age_package::AgePackageComponent.refs",
        requirement: "AGE-L03",
    },
];

/// 构造清单覆盖检查：清单里每一行的 `fact` 必须在**实际产物**里出现。
///
/// 这是 IF-981-FIELDS 的通用判据（与 979 的 `mapping_complete` 同口径）：
/// - 删掉清单里的一行 → 该行不再是 required，检查仍绿（清单是分母，不是产物）；
/// - 删掉**投影**（产物不再留该事实）→ 用 kept 清单检查时必须红；
/// - 合法近邻（产物多出别的事实）不受影响。
///
/// 所以真实反控的做法是：用**完整**清单核对**被削弱**的产物 → 必须报缺失。
pub fn age_construct_coverage(
    description: &AgePackageDescription,
    required: &[AgeConstruct],
) -> Result<(), Vec<String>> {
    let present = collect_facts(description);
    let missing: Vec<String> = required
        .iter()
        .filter(|c| !present.contains(&c.key))
        .map(|c| c.key.to_string())
        .collect();
    if missing.is_empty() {
        Ok(())
    } else {
        Err(missing)
    }
}

/// 产物里实际留下的构造键（没有任何行靠"名字像"混过去：每项都要有实体事实）。
pub fn collect_facts(description: &AgePackageDescription) -> Vec<&'static str> {
    let mut out: Vec<&'static str> = Vec::new();
    if description.components.is_empty() {
        return out;
    }
    out.push("node");
    for c in &description.components {
        if c.legacy_widget && !out.contains(&"node.legacy-widget") {
            out.push("node.legacy-widget");
        }
        if !c.fields.is_empty() {
            if !out.contains(&"model") {
                out.push("model");
            }
            // 字段初值真的参与了求值（有状态函数），才算 init 事实成立。
            if c.initialized && !out.contains(&"model.init") {
                out.push("model.init");
            }
        }
        if !c.handler_msgs.is_empty() && !out.contains(&"on") {
            out.push("on");
        }
        if c.handler_msgs.iter().any(|m| m.contains('.')) && !out.contains(&"on.lifecycle") {
            out.push("on.lifecycle");
        }
        if !c.texts.is_empty() && !out.contains(&"view") {
            out.push("view");
        }
        for t in &c.texts {
            if !t.fn_name.is_empty() && !t.def_id.is_empty() && !out.contains(&"view.text") {
                out.push("view.text");
            }
        }
        if !c.refs.is_empty() && !out.contains(&"view.node-ref") {
            out.push("view.node-ref");
        }
    }
    out
}

/// 包面身份拒绝：`age-package` 只由本前门消费（对称于 979 的通用入口拒绝）。
pub fn age_package_refusal(profile: SourceProfile, src_len: usize) -> Option<Diagnostic> {
    if profile != SourceProfile::AgePackage {
        return None;
    }
    Some(text_diag(
        "source.domain-frontend",
        "the age-package profile is consumed only by the AGE package front door \
         (ace_core::source::age_package::compile_age_package)",
        Span {
            start: 0,
            end: src_len,
        },
    ))
}

/// 描述里的叶角色汇总（最小资源要求面；本切片不申请运行时资源）。
pub fn component_roles(description: &AgePackageDescription) -> Vec<&str> {
    let mut out = vec!["node", "view", "text"];
    for c in &description.components {
        for t in &c.texts {
            if !out.contains(&t.role.as_str()) {
                out.push(&t.role);
            }
        }
    }
    out
}

/// 未使用但保留口径的类型别名（与 979 描述面同形，便于消费端统一处理）。
pub type AgePackageViewOut = ComponentViewOut;
/// 同上（组件候选面的共享类型）。
pub type AgePackageNode = ComponentNode;
/// 同上（受检文本面的共享类型）。
pub type AgePackageTextOut = ComponentText;
