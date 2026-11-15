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
    /// 包图身份（源 + 模块图 + grammar + 兼容配置）。
    pub input_hash: String,
    /// 脱糖后模块内容的身份（换领域行为即换它；源哈希不含脱糖）。
    pub desugar_hash: u64,
    pub atom_text: String,
}

impl AgePackageCompilation {
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
    // 1) 共享包绑定：包模块面 ∪ AGE 面**一次**解析；可见性/导入/初始化序/
    //    mangle 全部由 960 的绑定器裁定（本模块不重做任何一项）。
    let product = bind_packages_age(
        inputs,
        entry,
        capabilities,
        FrontBudget::WORK_OWNED_64MIB,
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
    let parts = crate::source::adapter::emit_resolved_packages_parts(
        &resolved,
        &product.bound,
        "packages",
    )
    .map_err(|ds| relabel_package_diags(locate_all(&product.bound, ds)))?;
    let input_hash = parts.input_hash.clone();
    let desugar_hash = crate::source::cst::content_sha(parts.atom_text.as_bytes());
    let checked = crate::verify::verify_text_record(parts.bundle)
        .map_err(|ds| relabel_package_diags(locate_all(&product.bound, ds)))?;

    // 5) 从 Checked HIR 读回每条行为的结论（共享 resolver 的判定，不是本层猜的）。
    //    隐藏函数的 HIR 名已被绑定器 mangle（`sf_p3_appm4_main___age_p_hidden_0`），
    //    故用共享 `origins` 表把源名映到真实符号再查——不猜命名规则。
    let components = read_back_components(&product.age_components, &checked, &product.bound)?;

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
