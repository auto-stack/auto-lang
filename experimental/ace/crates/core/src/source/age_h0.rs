//! PLAN-979（T-H0-I）：AGE Schema 驱动的最小领域前门（`node/view/text`）。
//!
//! 职责边界（§5 H0 合同）：
//! - **解析由 Schema 驱动**：领域结构名只在 `auto-source.at` 的 `age-h0`
//!   裁剪面声明为 `contextual` 槽位；词法面不变（`view` 沿用既有保留字，
//!   `node`/`text`/`widget` 仍是普通 ident）。
//! - **候选不受信**：本模块把 CST 投影成 [`AgeNodeItem`]（结构 + 原 span +
//!   表达式），不携带 Checked 信任、不缓存宿主授权。
//! - **唯一 verify**：文本叶表达式脱糖成一个普通隐藏函数
//!   `fn __age_h0_text_<n>() Str { return (<expr>).clone() }`，整模块（普通
//!   项 + 隐藏函数）经 `text-record` 域的唯一 verify（resolve/typecheck/bind）
//!   取得 Checked HIR；领域层只读共享 resolver/verify 的结论，不另建类型检查器。
//! - **值由共享后端求值**：CPU consumer（`ace-av`）按描述里的 def id 走共享
//!   lower→MIR→ABC→AV 执行得到文本值（见 `ace_av::age_h0`）。
//!
//! 本切片（H0）只冻结最小输入：一个 node、一个 view、一个 text 叶；属性/
//! 未知构造由语法结构拒绝，非文本表达式由共享类型规则拒绝。

use super::ast::{Expr, Ident};
use super::SourceProfile;
use crate::atom_text::{Diagnostic, Span, Stage};
use crate::verify::CheckedModule;

/// 领域包身份（进入描述与缓存键；换 schema/兼容配置 = 换身份）。
pub const AGE_H0_DOMAIN: &str = "ace.age.h0/v1";
/// 隐藏文本求值函数名前缀（编译器内部；用户同名声明是具名冲突）。
pub const AGE_H0_HIDDEN_PREFIX: &str = "__age_h0_text_";

/// 旧作者源 `widget` 声明头的兼容配置（显式；关闭 = 受控拒绝）。
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum LegacyWidgetAlias {
    /// 只接受规范 `node` 头；`widget` 头在头 span 受控拒绝。
    Reject,
    /// 同一 Schema 把 `widget` 头映射为同一种 Node 候选（原字节/span 保留）。
    Accept,
}

impl LegacyWidgetAlias {
    pub fn is_accept(self) -> bool {
        matches!(self, LegacyWidgetAlias::Accept)
    }
}

/// 文本值在语言里的种类（共享 resolver 的结论；不是领域层自造的类型）。
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum TextKind {
    /// `str`（借用视图）。
    View,
    /// `Str`（自有缓冲）。
    Owned,
}

/// 一个文本叶（未受信候选）。
#[derive(Clone, Debug)]
pub struct AgeTextLeaf {
    /// Schema 声明的角色（换拼写不改角色）。
    pub role: String,
    /// 实际拼写（诊断/迁移用）。
    pub spelling: String,
    /// 组件头词面 span。
    pub component_span: Span,
    /// 原始 Auto 文本表达式（未经领域层二次解释）。
    pub expr: Expr,
    /// 组件头到空 body 的 span。
    pub span: Span,
}

/// 一个 view 节（H0：一个 node 恰一个 view）。
#[derive(Clone, Debug)]
pub struct AgeViewNode {
    pub role: String,
    pub spelling: String,
    pub span: Span,
    pub texts: Vec<AgeTextLeaf>,
}

/// 一个 AGE node 声明（未受信候选：带原 span，不带 Checked 信任）。
#[derive(Clone, Debug)]
pub struct AgeNodeItem {
    pub head_role: String,
    pub head_spelling: String,
    pub head_span: Span,
    pub legacy_widget: bool,
    pub name: Ident,
    pub view: AgeViewNode,
    pub span: Span,
}

/// 受检文本叶（描述里的表达式标识 + 共享检查结论）。
#[derive(Clone, Debug)]
pub struct AgeH0Text {
    pub role: String,
    pub component_span: Span,
    pub span: Span,
    /// 共享 resolver/verify 判定的文本种类。
    pub kind: TextKind,
    /// 隐藏求值函数名与 def id（AV consumer 按它走共享后端）。
    pub fn_name: String,
    pub def_id: String,
}

/// 受检 view 节。
#[derive(Clone, Debug)]
pub struct AgeH0View {
    pub role: String,
    pub span: Span,
    pub texts: Vec<AgeH0Text>,
}

/// 受检 node 描述。
#[derive(Clone, Debug)]
pub struct AgeH0Node {
    pub name: String,
    pub head_role: String,
    pub head_span: Span,
    pub legacy_widget: bool,
    pub view: AgeH0View,
    pub span: Span,
}

/// 领域包版本化描述（投影/受检结论 + 资源要求）。
#[derive(Clone, Debug)]
pub struct AgeH0Description {
    pub domain: &'static str,
    pub grammar_id: String,
    pub alias: LegacyWidgetAlias,
    pub nodes: Vec<AgeH0Node>,
}

impl AgeH0Description {
    /// 兼容配置的稳定拼写（描述/收据用）。
    pub fn alias_name(&self) -> &'static str {
        match self.alias {
            LegacyWidgetAlias::Reject => "reject",
            LegacyWidgetAlias::Accept => "accept",
        }
    }

    /// 领域包槽位/组件角色清单（最小资源要求：本切片不申请运行时资源）。
    pub fn roles(&self) -> Vec<&str> {
        let mut out = vec!["node", "view", "text"];
        for n in &self.nodes {
            for t in &n.view.texts {
                if !out.contains(&t.role.as_str()) {
                    out.push(&t.role);
                }
            }
        }
        out
    }
}

/// 一次 AGE 前门编译的结果：受检 HIR + 领域描述 + 源身份。
#[derive(Debug)]
pub struct AgeH0Compilation {
    pub description: AgeH0Description,
    pub checked: CheckedModule,
    pub input_hash: String,
    /// 脱糖后模块的共享 Atom 文本：普通模块与 text-record 前门逐字节同形，
    /// 有领域叶时只追加隐藏求值函数（H0 证据：领域层不改普通语义）。
    pub atom_text: String,
}

/// 领域 profile 的通用入口拒绝（PLAN-979 F2）：领域声明不得被共享管线
/// 静默丢弃——通用解析/编译入口一律具名拒绝，只经本模块入口消费。
pub fn domain_profile_refusal(profile: SourceProfile, src_len: usize) -> Option<Diagnostic> {
    if profile != SourceProfile::AgeH0 {
        return None;
    }
    Some(text_diag(
        "source.domain-frontend",
        "the AGE domain profile is consumed only by the AGE domain front door \
         (ace_core::source::age_h0::compile_age_h0); generic parsing/compilation cannot \
         represent domain declarations",
        Span {
            start: 0,
            end: src_len,
        },
    ))
}

pub(crate) fn text_diag(code: &str, message: impl Into<String>, span: Span) -> Diagnostic {
    Diagnostic::new(Stage::Text, code, message, span)
}

pub(crate) fn infra_diag(message: impl Into<String>, len: usize) -> Diagnostic {
    Diagnostic::new(
        Stage::Text,
        "source.age.infra",
        message,
        Span { start: 0, end: len },
    )
}

/// AGE H0 领域前门：解析（Schema 驱动）→ 未受信候选 → 共享唯一 verify。
///
/// `alias` 是显式的兼容配置（参与描述身份，不改变 Schema 数据本身）。
/// AGE H0 的领域包注册：**第一个**经共享驱动 `domain::compile_domain` 的包。
/// 领域 owner（T-02 起可为 auto-ui）只需按同一接口提供 `project`。
pub static AGE_H0_PACKAGE: crate::source::domain::DomainPackage =
    crate::source::domain::DomainPackage {
        id: AGE_H0_DOMAIN,
        version: crate::source::domain::DomainVersion { major: 1, minor: 0 },
        profile: SourceProfile::AgeH0,
        hidden_prefix: AGE_H0_HIDDEN_PREFIX,
        compat: Some(crate::source::domain::CompatRule {
            spelling: "widget",
            disabled_code: "source.age.legacy-widget",
            disabled_message: "legacy `widget` declarations require the AGE legacy-widget \
                               compat config; write `node` or enable the compat config \
                               explicitly",
        }),
        project: project_age_h0,
    };

/// 领域 owner 的未受信投影：AGE 解析候选 → 通用结构行 + 行为。
fn project_age_h0(
    ok: &crate::source::cst::FrontOk,
    _src: &str,
) -> Result<crate::source::domain::DomainCandidate, Vec<Diagnostic>> {
    use crate::source::domain::{DomainBehavior, DomainCandidate, DomainItem};
    let mut structure = Vec::new();
    let mut behaviors = Vec::new();
    for n in &ok.age_nodes {
        let mut texts = Vec::new();
        for leaf in &n.view.texts {
            let idx = behaviors.len();
            behaviors.push(DomainBehavior {
                expr: leaf.expr.clone(),
                span: leaf.expr.span,
            });
            texts.push(DomainItem {
                role: leaf.role.clone(),
                name: None,
                span: leaf.span,
                aux_span: Some(leaf.component_span),
                spelling: Some(leaf.spelling.clone()),
                bindings: vec![idx],
                children: Vec::new(),
            });
        }
        let view = DomainItem {
            role: n.view.role.clone(),
            name: None,
            span: n.view.span,
            aux_span: None,
            spelling: Some(n.view.spelling.clone()),
            bindings: Vec::new(),
            children: texts,
        };
        structure.push(DomainItem {
            role: "node".to_string(),
            name: Some(n.name.name.clone()),
            span: n.span,
            // 次级 span = 声明头（兼容拒绝/诊断定位用）。
            aux_span: Some(n.head_span),
            spelling: Some(n.head_spelling.clone()),
            bindings: Vec::new(),
            children: vec![view],
        });
    }
    Ok(DomainCandidate {
        structure,
        behaviors,
        requirements: vec!["ac.runtime.text.v1".to_string()],
    })
}

pub fn compile_age_h0(
    src: &str,
    module_path: &str,
    alias: LegacyWidgetAlias,
    capabilities: &[String],
) -> Result<AgeH0Compilation, Vec<Diagnostic>> {
    // 通用驱动：前门 → 本包投影 → 兼容闸 → 隐藏名 → 脱糖 → 唯一 verify → 行描述。
    let compiled = crate::source::domain::compile_domain(
        &AGE_H0_PACKAGE,
        src,
        module_path,
        alias.is_accept(),
        capabilities,
    )?;
    // 通用行描述 → AGE 版本化描述（领域 owner 自有类型的映射；不改判据）。
    let mut nodes = Vec::new();
    for row in &compiled.description {
        let view = row.children.first();
        let texts = view
            .map(|v| {
                v.children
                    .iter()
                    .map(|t| {
                        let binding = t.bindings.first();
                        AgeH0Text {
                            role: t.role.clone(),
                            component_span: t.aux_span.unwrap_or(t.span),
                            span: t.span,
                            kind: binding.map(|b| b.kind).unwrap_or(TextKind::View),
                            fn_name: binding.map(|b| b.hidden_fn.clone()).unwrap_or_default(),
                            def_id: binding.map(|b| b.def_id.clone()).unwrap_or_default(),
                        }
                    })
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default();
        nodes.push(AgeH0Node {
            name: row.name.clone().unwrap_or_default(),
            head_role: row.role.clone(),
            head_span: row.aux_span.unwrap_or(row.span),
            legacy_widget: row.compat,
            view: AgeH0View {
                role: view.map(|v| v.role.clone()).unwrap_or_default(),
                span: view.map(|v| v.span).unwrap_or(row.span),
                texts,
            },
            span: row.span,
        });
    }
    Ok(AgeH0Compilation {
        description: AgeH0Description {
            domain: AGE_H0_DOMAIN,
            grammar_id: compiled.grammar_id,
            alias,
            nodes,
        },
        checked: compiled.checked,
        input_hash: compiled.input_hash,
        atom_text: compiled.atom_text,
    })
}
