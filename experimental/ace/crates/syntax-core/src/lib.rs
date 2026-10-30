//! ac-syntax-core-adapter：CST → ac-core `Program` 严格投影（PLAN-787 T-05）。
//!
//! - strict：树含 error/missing/不支持构造 → 受控拒绝，不产生 Checked 凭证；
//! - 从 CST **真实读取**并构造现有 `ace_core::source::ast::Program`
//!   （禁止调用旧 parser 冒充投影；旧 parser 仅在测试层作对照 oracle）；
//! - 名字驱动：kind 通过 auto-core grammar 的 token/rule 名解析（非硬编码 id）；
//! - 有符号字面量折叠与 i32 域检查在投影层（source.literal-range 语义拒绝）；
//! - 本投影仅用于对照与未来接口验证，不替换 ac-core 编译入口。

use ace_core::atom_text::Span;
use ace_core::source::ast::*;
use ace_syntax_runtime::cst::CstTree;
use ace_syntax_runtime::program::{GrammarProgram, KIND_CALL, RULE_KIND_BASE, TOKEN_KIND_BASE};
use ace_syntax_runtime::snapshot::Snapshot;

#[derive(Debug, PartialEq)]
pub enum ProjectionError {
    /// strict 拒绝：树含 error/missing。
    HasErrors(String),
    /// 字面量域违规（source.literal-range 语义）。
    LiteralRange { start: u64, end: u64 },
    /// 不支持构造/结构意外（受控拒绝，不 panic）。
    Unsupported(String),
    /// 投影递归预算（对照语料面之外的超深表达式）。
    TooDeep,
}

type PResult<T> = Result<T, ProjectionError>;

pub struct CoreKinds {
    fn_item: u32,
    block: u32,
    let_stmt: u32,
    var_stmt: u32,
    assign_stmt: u32,
    return_stmt: u32,
    if_stmt: u32,
    /// core 语法独有（scalar 拒绝 `while`）。
    while_stmt: Option<u32>,
    /// scalar 语法独有（条件 `for`，投影为 While AST 形状）。
    for_stmt: Option<u32>,
    loop_stmt: u32,
    else_block: u32,
    type_ann: u32,
    type_rule: u32,
    int_literal: u32,
    atom_expr: u32,
    /// scalar 语法独有（一元前缀 `- !`）。
    unary_expr: Option<u32>,
    expr_rule: u32,
    args: u32,
    arg: u32,
    arg_tail: u32,
    named_arg: u32,
    param: u32,
    stmt: u32,
    tok_ident: u32,
    tok_int: u32,
    tok_minus: u32,
    tok_true: u32,
    tok_false: u32,
    tok_int_kw: u32,
    tok_i32: u32,
    tok_bool: u32,
    op_plus: u32,
    op_star: u32,
    op_lt: u32,
    /// scalar 扩展二元算符（grammar 缺失 = None）。
    op_minus: Option<u32>,
    op_slash: Option<u32>,
    op_percent: Option<u32>,
    op_gt: Option<u32>,
    op_le: Option<u32>,
    op_ge: Option<u32>,
    op_eq: Option<u32>,
    op_ne: Option<u32>,
    op_andand: Option<u32>,
    op_oror: Option<u32>,
    tok_bang: Option<u32>,
    tok_let: u32,
    tok_var: u32,
}

fn rk(p: &GrammarProgram, name: &str) -> u32 {
    RULE_KIND_BASE
        + p.rules
            .iter()
            .position(|r| r.name == name)
            .expect("rule in grammar") as u32
}
fn tk(p: &GrammarProgram, name: &str) -> u32 {
    TOKEN_KIND_BASE
        + p.tokens
            .iter()
            .position(|t| t.name == name)
            .expect("token in grammar") as u32
}
fn rk_opt(p: &GrammarProgram, name: &str) -> Option<u32> {
    p.rules
        .iter()
        .position(|r| r.name == name)
        .map(|i| RULE_KIND_BASE + i as u32)
}
fn tk_opt(p: &GrammarProgram, name: &str) -> Option<u32> {
    p.tokens
        .iter()
        .position(|t| t.name == name)
        .map(|i| TOKEN_KIND_BASE + i as u32)
}

impl CoreKinds {
    pub fn from_program(p: &GrammarProgram) -> CoreKinds {
        CoreKinds {
            fn_item: rk(p, "fn-item"),
            block: rk(p, "block"),
            let_stmt: rk(p, "let-stmt"),
            var_stmt: rk(p, "var-stmt"),
            assign_stmt: rk(p, "assign-stmt"),
            return_stmt: rk(p, "return-stmt"),
            if_stmt: rk(p, "if-stmt"),
            while_stmt: rk_opt(p, "while-stmt"),
            for_stmt: rk_opt(p, "for-stmt"),
            loop_stmt: rk(p, "loop-stmt"),
            else_block: rk(p, "else-block"),
            type_ann: rk(p, "type-ann"),
            type_rule: rk(p, "type"),
            int_literal: rk(p, "int-literal"),
            atom_expr: rk(p, "atom-expr"),
            unary_expr: rk_opt(p, "unary-expr"),
            expr_rule: rk(p, "expr"),
            args: rk(p, "args"),
            arg: rk(p, "arg"),
            arg_tail: rk(p, "arg-tail"),
            named_arg: rk(p, "named-arg"),
            param: rk(p, "param"),
            stmt: rk(p, "stmt"),
            tok_ident: tk(p, "ident"),
            tok_int: tk(p, "int-lit"),
            tok_minus: tk(p, "minus"),
            tok_true: tk(p, "true-kw"),
            tok_false: tk(p, "false-kw"),
            tok_int_kw: tk(p, "int-kw"),
            tok_i32: tk(p, "i32"),
            tok_bool: tk(p, "bool"),
            op_plus: tk(p, "plus"),
            op_star: tk(p, "star"),
            op_lt: tk(p, "lt"),
            op_minus: Some(tk(p, "minus")),
            op_slash: tk_opt(p, "slash"),
            op_percent: tk_opt(p, "percent"),
            op_gt: tk_opt(p, "gt"),
            op_le: tk_opt(p, "le"),
            op_ge: tk_opt(p, "ge"),
            op_eq: tk_opt(p, "eq"),
            op_ne: tk_opt(p, "ne"),
            op_andand: tk_opt(p, "andand"),
            op_oror: tk_opt(p, "oror"),
            tok_bang: tk_opt(p, "bang"),
            tok_let: tk(p, "let"),
            tok_var: tk(p, "var"),
        }
    }
}

mod projection_engine;

use projection_engine::{pop_block, run, Frame, Val};

struct Cx<'a> {
    k: &'a CoreKinds,
    tree: &'a CstTree,
    snap: &'a Snapshot,
    /// 已访问帧数（F787-R2/3-08：无固定句法深度上限——资源预算受控）。
    visited: usize,
}

/// 投影工作预算（帧访问数）：合法 core 承诺面远低于此；超出 = 受控
/// TooDeep（预算语义，非语法排除）。
const PROJECTION_NODE_BUDGET: usize = 1 << 22;

fn span_of(tree: &CstTree, n: u32) -> Span {
    let (s, e) = tree.abs_range(n);
    Span {
        start: s as usize,
        end: e as usize,
    }
}

fn spelling(tree: &CstTree, snap: &Snapshot, n: u32) -> PResult<String> {
    let (s, e) = tree.abs_range(n);
    snap.read_str(s, e)
        .map_err(|_| ProjectionError::Unsupported("span read".into()))
}

/// strict 投影入口。
pub fn project_strict(
    p: &GrammarProgram,
    k: &CoreKinds,
    tree: &CstTree,
    snap: &Snapshot,
) -> PResult<Program> {
    if tree.has_error_nodes() {
        return Err(ProjectionError::HasErrors(
            "tree contains error/missing nodes".into(),
        ));
    }
    let mut cx = Cx {
        k,
        tree,
        snap,
        visited: 0,
    };
    let mut items = Vec::new();
    for d in &tree.decls {
        if tree.kind(d.node) == k.fn_item {
            items.push(Item::Fn(project_fn(p, &mut cx, d.node)?));
        }
    }
    if items.is_empty() {
        // core：空文件/无顶层 fn = source.syntax（投影层拒绝，不产生 Checked）。
        return Err(ProjectionError::HasErrors(
            "empty program (expected at least one fn item)".into(),
        ));
    }
    Ok(Program { items })
}

/// strict 单声明投影（PLAN-828 T-02）：只投影 `decl_index` 指向的顶层 fn
/// 声明——增量编辑热路径的原语（未改声明不重新投影）。span 为快照绝对
/// 坐标；树含 error/missing 时受控拒绝。
pub fn project_decl_strict(
    p: &GrammarProgram,
    k: &CoreKinds,
    tree: &CstTree,
    snap: &Snapshot,
    decl_index: usize,
) -> PResult<FnItem> {
    if tree.has_error_nodes() {
        return Err(ProjectionError::HasErrors(
            "tree contains error/missing nodes".into(),
        ));
    }
    let d = tree
        .decls
        .get(decl_index)
        .ok_or_else(|| ProjectionError::Unsupported("decl index out of range".into()))?;
    if tree.kind(d.node) != k.fn_item {
        return Err(ProjectionError::Unsupported(format!(
            "decl {decl_index} is not an fn item"
        )));
    }
    let mut cx = Cx {
        k,
        tree,
        snap,
        visited: 0,
    };
    project_fn(p, &mut cx, d.node)
}

fn project_fn(p: &GrammarProgram, cx: &mut Cx, node: u32) -> PResult<FnItem> {
    let name_f = p.field_id("name").unwrap();
    let params_f = p.field_id("params").unwrap();
    let ty_f = p.field_id("ty").unwrap();
    let body_f = p.field_id("body").unwrap();
    let name_n = tree_field_child(cx, node, name_f, "fn name")?;
    let ident = Ident {
        name: spelling(cx.tree, cx.snap, name_n)?,
        span: span_of(cx.tree, name_n),
    };
    let mut params = Vec::new();
    if let Some(pn) = tree_field_child_opt(cx, node, params_f) {
        for pn2 in descendants_of_kind(cx, pn, cx.k.param) {
            params.push(project_param(p, cx, pn2)?);
        }
    }
    let ty_n = tree_field_child(cx, node, ty_f, "fn ret")?;
    let ret = project_type(cx, ty_n)?;
    let body_n = tree_field_child(cx, node, body_f, "fn body")?;
    let body = project_block(p, cx, body_n)?;
    Ok(FnItem {
        name: ident,
        params,
        ret,
        body,
        span: span_of(cx.tree, node),
        receiver: Receiver::Free,
        owner: None,
        // a CST-projected declaration is never the PLAN-862 synthetic
        // script item (the script form has no n keyword to project).
        script: false,
    })
}

/// 按位置序收集子树内指定 kind 的节点（迭代）。
fn descendants_of_kind(cx: &Cx, root: u32, kind: u32) -> Vec<u32> {
    let mut found = Vec::new();
    let mut stack = vec![root];
    while let Some(n) = stack.pop() {
        if cx.tree.kind(n) == kind {
            found.push(n);
        }
        for c in cx.tree.children(n) {
            stack.push(c);
        }
    }
    found.sort_by_key(|&n| cx.tree.abs_range(n).0);
    found
}

fn tree_field_child_opt(cx: &Cx, node: u32, field: u32) -> Option<u32> {
    cx.tree.field_child(node, field)
}

fn tree_field_child(cx: &Cx, node: u32, field: u32, what: &str) -> PResult<u32> {
    tree_field_child_opt(cx, node, field).ok_or_else(|| ProjectionError::Unsupported(what.into()))
}

/// 类型注解（非递归：读单个类型词孩子）。
fn project_type(cx: &Cx, node: u32) -> PResult<TypeAnn> {
    let tok_n = *cx
        .tree
        .children(node)
        .iter()
        .find(|&&c| {
            matches!(
                cx.tree.kind(c),
                x if x == cx.k.tok_int_kw || x == cx.k.tok_i32 || x == cx.k.tok_bool
            )
        })
        .ok_or_else(|| ProjectionError::Unsupported("type word".into()))?;
    let tw = match cx.tree.kind(tok_n) {
        x if x == cx.k.tok_int_kw => TypeWord::Int,
        x if x == cx.k.tok_i32 => TypeWord::I32,
        x if x == cx.k.tok_bool => TypeWord::Bool,
        _ => return Err(ProjectionError::Unsupported("type word".into())),
    };
    Ok(TypeAnn {
        kind: TypeRef::Builtin(tw),
        span: span_of(cx.tree, node),
    })
}

fn project_param(p: &GrammarProgram, cx: &Cx, node: u32) -> PResult<Param> {
    let name_f = p.field_id("name").unwrap();
    let ty_f = p.field_id("ty").unwrap();
    let name_n = tree_field_child(cx, node, name_f, "param name")?;
    let ty_n = tree_field_child(cx, node, ty_f, "param ty")?;
    let ty = project_type(cx, ty_n)?;
    Ok(Param {
        name: Ident {
            name: spelling(cx.tree, cx.snap, name_n)?,
            span: span_of(cx.tree, name_n),
        },
        ty,
        mode: ParamMode::Default,
        span: span_of(cx.tree, node),
    })
}

/// 块投影（引擎入口）：显式帧栈驱动，正常线程栈不随嵌套深度递归。
fn project_block(p: &GrammarProgram, cx: &mut Cx, node: u32) -> PResult<Block> {
    let mut stack: Vec<Frame> = vec![Frame::Block {
        node,
        cursor: 0,
        stmts: Vec::new(),
    }];
    let mut vals: Vec<Val> = Vec::new();
    run(p, cx, &mut stack, &mut vals)?;
    pop_block(&mut vals, "block result")
}
