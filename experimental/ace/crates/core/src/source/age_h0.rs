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

use super::ast::{
    Block, Expr, ExprKind, FnItem, Ident, Item, Program, Receiver, Stmt, TypeAnn, TypeRef, TypeWord,
};
use super::cst::{parse_front, FrontBudget, FrontKind, FrontOutcome, FrontRequest};
use super::SourceProfile;
use crate::atom_text::{Diagnostic, Span, Stage};
use crate::hir::{BuiltinType, DefKind, ExprKind as HirExprKind, Stmt as HirStmt, TypeUse};
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

fn text_diag(code: &str, message: impl Into<String>, span: Span) -> Diagnostic {
    Diagnostic::new(Stage::Text, code, message, span)
}

fn infra_diag(message: impl Into<String>, len: usize) -> Diagnostic {
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
pub fn compile_age_h0(
    src: &str,
    module_path: &str,
    alias: LegacyWidgetAlias,
    capabilities: &[String],
) -> Result<AgeH0Compilation, Vec<Diagnostic>> {
    let outcome = parse_front(FrontRequest {
        src,
        profile: SourceProfile::AgeH0,
        kind: FrontKind::WholeModule,
        budget: FrontBudget::WORK_OWNED_64MIB,
    });
    let ok = match outcome {
        FrontOutcome::Ok(ok) => ok,
        FrontOutcome::Diag(ds) => return Err(ds),
        FrontOutcome::ResourceExhausted { what, limit } => {
            return Err(vec![text_diag(
                "source.resource",
                format!("resource exhausted: {what} (limit {limit})"),
                Span {
                    start: 0,
                    end: src.len(),
                },
            )])
        }
        FrontOutcome::StaleForeign(m) | FrontOutcome::Infra(m) => {
            return Err(vec![infra_diag(format!("AGE front door: {m}"), src.len())])
        }
    };
    let candidates = ok.age_nodes;

    // 兼容配置：别名关闭时旧声明头受控拒绝（不自动回落旧引擎）。
    if !alias.is_accept() {
        if let Some(n) = candidates.iter().find(|n| n.legacy_widget) {
            return Err(vec![text_diag(
                "source.age.legacy-widget",
                "legacy `widget` declarations require the AGE legacy-widget compat config; \
                 write `node` or enable the compat config explicitly",
                n.head_span,
            )]);
        }
    }

    // 隐藏求值函数与用户声明重名 = 具名冲突（不静默遮蔽）。
    let mut hidden: Vec<(String, usize, usize)> = Vec::new();
    let mut idx = 0usize;
    for (ni, n) in candidates.iter().enumerate() {
        for (ti, _) in n.view.texts.iter().enumerate() {
            hidden.push((format!("{AGE_H0_HIDDEN_PREFIX}{idx}"), ni, ti));
            idx += 1;
        }
    }
    for (fname, _, _) in &hidden {
        for item in &ok.program.items {
            let user = match item {
                Item::Fn(f) => Some(&f.name),
                Item::Type(t) => Some(&t.name),
                Item::Enum(e) => Some(&e.name),
                Item::Const(c) => Some(&c.name),
                Item::Store(s) => Some(&s.name),
                Item::UseRs(_) => None,
            };
            if let Some(u) = user {
                if u.name == *fname {
                    return Err(vec![text_diag(
                        "source.age.hidden-name",
                        format!("`{fname}` is reserved for AGE text evaluation"),
                        u.span,
                    )]);
                }
            }
        }
    }

    // 脱糖：每个文本叶一个隐藏函数（普通 Auto 源，由共享 verify 检查）。
    let mut items = ok.program.items.clone();
    let mut def_names: Vec<String> = Vec::new();
    for (fname, ni, ti) in &hidden {
        let leaf = &candidates[*ni].view.texts[*ti];
        let span = leaf.expr.span;
        let clone = Expr {
            span,
            kind: ExprKind::Method {
                recv: Box::new(leaf.expr.clone()),
                name: Ident {
                    name: "clone".to_string(),
                    span,
                },
                args: Vec::new(),
            },
        };
        let f = FnItem {
            name: Ident {
                name: fname.clone(),
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
        };
        def_names.push(fname.clone());
        items.push(Item::Fn(f));
    }
    let desugared = Program { items };

    // 唯一 verify：整模块（普通项 + 隐藏求值函数）走 text-record 域的共享链。
    let parts = super::adapter::compile_program_parts(
        &desugared,
        src,
        module_path,
        capabilities,
        SourceProfile::TextRecord,
    )?;
    let input_hash = parts.input_hash.clone();
    let checked = crate::verify::verify_text_record(parts.bundle)?;

    // 描述：从 Checked HIR 读共享检查结论（文本种类 + def id）。
    let mut nodes = Vec::new();
    for (ni, n) in candidates.iter().enumerate() {
        let mut texts = Vec::new();
        for (ti, leaf) in n.view.texts.iter().enumerate() {
            let fn_name = hidden
                .iter()
                .find(|(_, hni, hti)| *hni == ni && *hti == ti)
                .map(|(f, _, _)| f.clone())
                .unwrap_or_default();
            let (def_id, kind) = text_conclusion(&checked, &fn_name).ok_or_else(|| {
                vec![text_diag(
                    "source.age.text-type",
                    "a `text` leaf expression must be text (`str` or `Str`); the shared \
                     type checker rejected this expression",
                    leaf.expr.span,
                )]
            })?;
            texts.push(AgeH0Text {
                role: leaf.role.clone(),
                component_span: leaf.component_span,
                span: leaf.span,
                kind,
                fn_name,
                def_id,
            });
        }
        nodes.push(AgeH0Node {
            name: n.name.name.clone(),
            head_role: n.head_role.clone(),
            head_span: n.head_span,
            legacy_widget: n.legacy_widget,
            view: AgeH0View {
                role: n.view.role.clone(),
                span: n.view.span,
                texts,
            },
            span: n.span,
        });
    }
    Ok(AgeH0Compilation {
        description: AgeH0Description {
            domain: AGE_H0_DOMAIN,
            grammar_id: ok.grammar_id,
            alias,
            nodes,
        },
        checked,
        input_hash,
        atom_text: parts.atom_text,
    })
}

/// 从 Checked HIR 读隐藏求值函数的 def id 与文本种类（共享检查结论）。
fn text_conclusion(checked: &CheckedModule, fn_name: &str) -> Option<(String, TextKind)> {
    let module = checked.bundle().modules.first()?;
    // HIR def 名可能带域前缀（如 `sf__`）——按隐藏名后缀匹配，def id 仍取
    // 共享 adapter 为它生成的真实 id（不是本地拼名）。
    let def = module.defs.iter().find(|d| match &d.kind {
        DefKind::Function { name, .. } => name == fn_name || name.ends_with(fn_name),
        _ => false,
    })?;
    let body_id = match &def.kind {
        DefKind::Function { body, .. } => *body,
        _ => return None,
    };
    let body = module.bodies.get(body_id.0 as usize)?;
    let block = body.blocks.get(body.entry.0 as usize)?;
    let value = block.stmts.iter().find_map(|s| match s {
        HirStmt::Return { value } => Some(*value),
        _ => None,
    })?;
    let expr = body.exprs.get(value.0 as usize)?;
    let HirExprKind::TextClone { operand } = &expr.kind else {
        return None;
    };
    let operand_expr = body.exprs.get(operand.0 as usize)?;
    let kind = match operand_expr.ty {
        // 文本种类在共享 HIR 里是命名类型条目（`TypeKind::{Str,StrOwned}`）。
        TypeUse::Ref(tid) => match module.types.get(tid.0 as usize).map(|t| &t.kind) {
            Some(crate::hir::TypeKind::Str) => TextKind::View,
            Some(crate::hir::TypeKind::StrOwned) => TextKind::Owned,
            _ => return None,
        },
        TypeUse::Builtin(BuiltinType::StrSlice) => TextKind::View,
        TypeUse::Builtin(BuiltinType::StrOwned) => TextKind::Owned,
        _ => return None,
    };
    Some((def.id_text.clone(), kind))
}
