//! ac-syntax-core-adapter：CST → ac-core `Program` 严格投影（PLAN-787 T-05）。
//!
//! - strict：树含 error/missing/不支持构造 → 受控拒绝，不产生 Checked 凭证；
//! - 从 CST **真实读取**并构造现有 `auto_ac_prototype::source::ast::Program`
//!   （禁止调用旧 parser 冒充投影；旧 parser 仅在测试层作对照 oracle）；
//! - 名字驱动：kind 通过 auto-core grammar 的 token/rule 名解析（非硬编码 id）；
//! - 有符号字面量折叠与 i32 域检查在投影层（source.literal-range 语义拒绝）；
//! - 本投影仅用于对照与未来接口验证，不替换 ac-core 编译入口。

use ac_syntax_rt::cst::CstTree;
use ac_syntax_rt::program::{GrammarProgram, KIND_CALL, RULE_KIND_BASE, TOKEN_KIND_BASE};
use ac_syntax_rt::snapshot::Snapshot;
use auto_ac_prototype::atom_text::Span;
use auto_ac_prototype::source::ast::*;

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
    while_stmt: u32,
    loop_stmt: u32,
    else_block: u32,
    type_ann: u32,
    type_rule: u32,
    int_literal: u32,
    atom_expr: u32,
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
            while_stmt: rk(p, "while-stmt"),
            loop_stmt: rk(p, "loop-stmt"),
            else_block: rk(p, "else-block"),
            type_ann: rk(p, "type-ann"),
            type_rule: rk(p, "type"),
            int_literal: rk(p, "int-literal"),
            atom_expr: rk(p, "atom-expr"),
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
            tok_let: tk(p, "let"),
            tok_var: tk(p, "var"),
        }
    }
}

struct Cx<'a> {
    k: &'a CoreKinds,
    tree: &'a CstTree,
    snap: &'a Snapshot,
    /// 已访问节点数（F787-R2-08：无固定句法深度上限——资源预算受控）。
    visited: usize,
}

/// 投影工作预算（访问节点数）：约 4M 节点——合法 core 承诺面（含深链/深
/// 嵌套）远低于此；超出 = 受控 TooDeep（预算语义，非语法排除）。
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
    let mut items = Vec::new();
    for d in &tree.decls {
        if tree.kind(d.node) == k.fn_item {
            items.push(Item::Fn(project_fn(p, k, tree, snap, d.node)?));
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

fn project_fn(
    p: &GrammarProgram,
    k: &CoreKinds,
    tree: &CstTree,
    snap: &Snapshot,
    node: u32,
) -> PResult<FnItem> {
    let mut cx = Cx {
        k,
        tree,
        snap,
        visited: 0,
    };
    let name_f = p.field_id("name").unwrap();
    let params_f = p.field_id("params").unwrap();
    let ty_f = p.field_id("ty").unwrap();
    let body_f = p.field_id("body").unwrap();
    let name_n = tree
        .field_child(node, name_f)
        .ok_or_else(|| ProjectionError::Unsupported("fn name".into()))?;
    let ident = Ident {
        name: spelling(tree, snap, name_n)?,
        span: span_of(tree, name_n),
    };
    let mut params = Vec::new();
    if let Some(pn) = tree.field_child(node, params_f) {
        for pn2 in descendants_of_kind(&cx, pn, k.param) {
            params.push(project_param(p, &mut cx, pn2)?);
        }
    }
    let ty_n = tree
        .field_child(node, ty_f)
        .ok_or_else(|| ProjectionError::Unsupported("fn ret".into()))?;
    let ret = project_type(&mut cx, ty_n)?;
    let body_n = tree
        .field_child(node, body_f)
        .ok_or_else(|| ProjectionError::Unsupported("fn body".into()))?;
    let body = project_block(p, &mut cx, body_n)?;
    Ok(FnItem {
        name: ident,
        params,
        ret,
        body,
        span: span_of(tree, node),
        receiver: Receiver::Free,
        owner: None,
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
        for &c in cx.tree.children(n).iter().rev() {
            stack.push(c);
        }
    }
    found.sort_by_key(|&n| cx.tree.abs_range(n).0);
    found
}

fn project_type(cx: &mut Cx, node: u32) -> PResult<TypeAnn> {
    let tok_n = *cx
        .tree
        .children(node)
        .iter()
        .find(|&&c| matches!(cx.tree.kind(c), x if x == cx.k.tok_int_kw || x == cx.k.tok_i32 || x == cx.k.tok_bool))
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

fn project_param(p: &GrammarProgram, cx: &mut Cx, node: u32) -> PResult<Param> {
    let name_f = p.field_id("name").unwrap();
    let ty_f = p.field_id("ty").unwrap();
    let name_n = cx.tree.field_child(node, name_f).unwrap();
    let ty_n = cx.tree.field_child(node, ty_f).unwrap();
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

fn project_block(p: &GrammarProgram, cx: &mut Cx, node: u32) -> PResult<Block> {
    let mut stmts = Vec::new();
    for c in cx.tree.children(node) {
        if cx.tree.kind(c) == cx.k.stmt {
            // 分派节点 inner = 第一个非 trivia 孩子；break/continue 是叶子语句。
            let inner = *cx
                .tree
                .children(c)
                .iter()
                .find(|&&x| {
                    let nd = cx.tree.node(x);
                    nd.flags & ac_syntax_rt::cst::FLAG_TRIVIA == 0
                        && nd.flags & ac_syntax_rt::cst::FLAG_ERROR == 0
                })
                .ok_or_else(|| ProjectionError::Unsupported("empty stmt".into()))?;
            stmts.push(project_stmt(p, cx, inner)?);
        }
    }
    Ok(Block {
        stmts,
        span: span_of(cx.tree, node),
    })
}

fn project_stmt(p: &GrammarProgram, cx: &mut Cx, node: u32) -> PResult<Stmt> {
    let value_f = p.field_id("value").unwrap();
    let cond_f = p.field_id("cond").unwrap();
    let then_f = p.field_id("then").unwrap();
    let body_f = p.field_id("body").unwrap();
    let name_f = p.field_id("name").unwrap();
    match cx.tree.kind(node) {
        x if x == cx.k.let_stmt || x == cx.k.var_stmt => {
            let name_n = cx.tree.field_child(node, name_f).unwrap();
            let ann = match cx
                .tree
                .children(node)
                .iter()
                .find(|&&c| cx.tree.kind(c) == cx.k.type_ann)
            {
                Some(&ta) => {
                    let t = *cx
                        .tree
                        .children(ta)
                        .iter()
                        .find(|&&c| cx.tree.kind(c) == cx.k.type_rule)
                        .unwrap();
                    Some(project_type(cx, t)?)
                }
                None => None,
            };
            let val_n = cx.tree.field_child(node, value_f).unwrap();
            let kw_kind = if x == cx.k.let_stmt {
                cx.k.tok_let
            } else {
                cx.k.tok_var
            };
            let kw_n = *cx
                .tree
                .children(node)
                .iter()
                .find(|&&c| cx.tree.kind(c) == kw_kind)
                .ok_or_else(|| ProjectionError::Unsupported("let/var kw".into()))?;
            let value = project_expr(cx, val_n)?;
            let sp = Span {
                start: cx.tree.abs_range(node).0 as usize,
                end: value.span.end,
            };
            Ok(Stmt::Let {
                kw: span_of(cx.tree, kw_n),
                name: Ident {
                    name: spelling(cx.tree, cx.snap, name_n)?,
                    span: span_of(cx.tree, name_n),
                },
                ann,
                mutable: x == cx.k.var_stmt,
                value,
                span: sp,
            })
        }
        x if x == cx.k.assign_stmt => {
            let name_n = cx.tree.field_child(node, name_f).unwrap();
            let val_n = cx.tree.field_child(node, value_f).unwrap();
            let value = project_expr(cx, val_n)?;
            let sp = Span {
                start: cx.tree.abs_range(node).0 as usize,
                end: value.span.end,
            };
            Ok(Stmt::Assign {
                target: AssignTarget {
                    base: Ident {
                        name: spelling(cx.tree, cx.snap, name_n)?,
                        span: span_of(cx.tree, name_n),
                    },
                    path: Vec::new(),
                },
                value,
                span: sp,
            })
        }
        x if x == cx.k.return_stmt => {
            let val_n = cx.tree.field_child(node, value_f).unwrap();
            let value = project_expr(cx, val_n)?;
            let sp = Span {
                start: cx.tree.abs_range(node).0 as usize,
                end: value.span.end,
            };
            Ok(Stmt::Return { value, span: sp })
        }
        x if x == cx.k.if_stmt => {
            let cond_n = cx.tree.field_child(node, cond_f).unwrap();
            let then_n = cx.tree.field_child(node, then_f).unwrap();
            let els = match cx
                .tree
                .children(node)
                .iter()
                .find(|&&c| cx.tree.kind(c) == cx.k.else_block)
            {
                Some(&eb) => {
                    let b = *cx
                        .tree
                        .children(eb)
                        .iter()
                        .find(|&&c| cx.tree.kind(c) == cx.k.block)
                        .unwrap();
                    Some(project_block(p, cx, b)?)
                }
                None => None,
            };
            Ok(Stmt::If {
                cond: project_expr(cx, cond_n)?,
                then: project_block(p, cx, then_n)?,
                els,
                span: span_of(cx.tree, node),
            })
        }
        x if x == cx.k.while_stmt => {
            let cond_n = cx.tree.field_child(node, cond_f).unwrap();
            let body_n = cx.tree.field_child(node, body_f).unwrap();
            Ok(Stmt::While {
                cond: project_expr(cx, cond_n)?,
                body: project_block(p, cx, body_n)?,
                span: span_of(cx.tree, node),
            })
        }
        x if x == cx.k.loop_stmt => {
            let body_n = cx.tree.field_child(node, body_f).unwrap();
            Ok(Stmt::Loop {
                body: project_block(p, cx, body_n)?,
                span: span_of(cx.tree, node),
            })
        }
        _ => {
            // break/continue：叶语句——node 即关键字叶（child_count=0）或单孩子叶。
            let mut leaf = node;
            if cx.tree.children(node).len() == 1 {
                leaf = cx.tree.children(node)[0];
            }
            if cx.tree.children(leaf).is_empty() {
                let sp = spelling(cx.tree, cx.snap, leaf)?;
                if sp == "break" {
                    return Ok(Stmt::Break {
                        span: span_of(cx.tree, leaf),
                    });
                }
                if sp == "continue" {
                    return Ok(Stmt::Continue {
                        span: span_of(cx.tree, leaf),
                    });
                }
            }
            Err(ProjectionError::Unsupported(format!(
                "stmt kind {}",
                cx.tree.kind(node)
            )))
        }
    }
}

fn project_expr(cx: &mut Cx, node: u32) -> PResult<Expr> {
    // 访问预算受控（F787-R2-08）：合法 core 输入（任意深链/嵌套）不设句法
    // 上限——仅当访问量超过工作预算时受控拒绝（TooDeep 语义=预算耗尽）。
    cx.visited += 1;
    if cx.visited > PROJECTION_NODE_BUDGET {
        return Err(ProjectionError::TooDeep);
    }
    project_expr_inner(cx, node)
}

fn project_expr_inner(cx: &mut Cx, node: u32) -> PResult<Expr> {
    let kind = cx.tree.kind(node);
    if kind == cx.k.int_literal {
        let kids = cx.tree.children(node);
        let int_n = *kids
            .iter()
            .find(|&&c| cx.tree.kind(c) == cx.k.tok_int)
            .unwrap();
        let neg = kids.iter().any(|&c| cx.tree.kind(c) == cx.k.tok_minus);
        let text = spelling(cx.tree, cx.snap, int_n)?;
        let mag: i128 = text
            .parse()
            .map_err(|_| ProjectionError::Unsupported("int literal".into()))?;
        let v: i128 = if neg { -mag } else { mag };
        if v > i32::MAX as i128 || v < i32::MIN as i128 {
            let (s, e) = cx.tree.abs_range(node);
            return Err(ProjectionError::LiteralRange { start: s, end: e });
        }
        return Ok(Expr {
            kind: ExprKind::IntLit(v as i32),
            span: span_of(cx.tree, node),
        });
    }
    if kind == KIND_CALL {
        let kids = cx.tree.children(node);
        let callee_n = *kids
            .iter()
            .find(|&&c| cx.tree.kind(c) == cx.k.atom_expr)
            .unwrap();
        // atom-expr 孩子可能以 trivia 叶开头——按 token kind 定位 ident。
        let ident_n = *cx
            .tree
            .children(callee_n)
            .iter()
            .find(|&&c| cx.tree.kind(c) == cx.k.tok_ident)
            .ok_or_else(|| ProjectionError::Unsupported("call callee".into()))?;
        let name = spelling(cx.tree, cx.snap, ident_n)?;
        let mut args = Vec::new();
        if let Some(&args_n) = kids.iter().find(|&&c| cx.tree.kind(c) == cx.k.args) {
            // args := arg (, arg)*：只取本层 arg——嵌套调用的 args 子树不得混入。
            let mut direct: Vec<u32> = Vec::new();
            for c in cx.tree.children(args_n) {
                let ck = cx.tree.kind(c);
                if ck == cx.k.arg {
                    direct.push(c);
                } else if ck == cx.k.arg_tail {
                    for gc in cx.tree.children(c) {
                        if cx.tree.kind(gc) == cx.k.arg {
                            direct.push(gc);
                        }
                    }
                }
            }
            direct.sort_by_key(|&n| cx.tree.abs_range(n).0);
            for a in direct {
                if let Some(&na) = cx
                    .tree
                    .children(a)
                    .iter()
                    .find(|&&c| cx.tree.kind(c) == cx.k.named_arg)
                {
                    let label_n = *cx
                        .tree
                        .children(na)
                        .iter()
                        .find(|&&c| cx.tree.kind(c) == cx.k.tok_ident)
                        .ok_or_else(|| ProjectionError::Unsupported("named label".into()))?;
                    let val_n = *cx
                        .tree
                        .children(na)
                        .iter()
                        .find(|&&c| cx.tree.kind(c) == cx.k.expr_rule)
                        .unwrap();
                    args.push(Arg::Named {
                        name: Ident {
                            name: spelling(cx.tree, cx.snap, label_n)?,
                            span: span_of(cx.tree, label_n),
                        },
                        value: project_expr(cx, val_n)?,
                        span: span_of(cx.tree, a),
                    });
                } else {
                    let e_n = *cx
                        .tree
                        .children(a)
                        .iter()
                        .find(|&&c| cx.tree.kind(c) == cx.k.expr_rule)
                        .unwrap();
                    args.push(Arg::Pos(project_expr(cx, e_n)?));
                }
            }
        }
        return Ok(Expr {
            kind: ExprKind::Call {
                callee: Ident {
                    name,
                    span: span_of(cx.tree, ident_n),
                },
                args,
            },
            span: span_of(cx.tree, node),
        });
    }
    if kind == cx.k.op_plus || kind == cx.k.op_star || kind == cx.k.op_lt {
        let kids = cx.tree.children(node);
        let lhs_n = kids[0];
        let rhs_n = kids[kids.len() - 1];
        let op = match kind {
            x if x == cx.k.op_plus => BinOp::Add,
            x if x == cx.k.op_star => BinOp::Mul,
            _ => BinOp::Lt,
        };
        let lhs = project_expr(cx, lhs_n)?;
        let rhs = project_expr(cx, rhs_n)?;
        let sp = Span {
            start: lhs.span.start,
            end: rhs.span.end,
        };
        return Ok(Expr {
            kind: ExprKind::Binary {
                op,
                lhs: Box::new(lhs),
                rhs: Box::new(rhs),
            },
            span: sp,
        });
    }
    if kind == cx.k.atom_expr {
        for c in cx.tree.children(node) {
            let ck = cx.tree.kind(c);
            if ck == cx.k.int_literal {
                return project_expr(cx, c);
            }
            if ck == cx.k.tok_true || ck == cx.k.tok_false {
                let v = ck == cx.k.tok_true;
                return Ok(Expr {
                    kind: ExprKind::BoolLit(v),
                    span: span_of(cx.tree, node),
                });
            }
            if ck == cx.k.tok_ident {
                return Ok(Expr {
                    kind: ExprKind::Var(Ident {
                        name: spelling(cx.tree, cx.snap, c)?,
                        span: span_of(cx.tree, c),
                    }),
                    span: span_of(cx.tree, c),
                });
            }
            if ck == cx.k.expr_rule {
                // 括号组：ac-core Paren 帧弹出不改值——组值 = 内层表达式原样。
                return project_expr(cx, c);
            }
        }
        return Err(ProjectionError::Unsupported("atom-expr empty".into()));
    }
    if kind == cx.k.expr_rule {
        // expr 分派节点 → 内层 pratt 产物。
        for c in cx.tree.children(node) {
            if cx.tree.child_count(c) > 0 {
                return project_expr(cx, c);
            }
        }
        return Err(ProjectionError::Unsupported("empty expr".into()));
    }
    Err(ProjectionError::Unsupported(format!("expr kind {kind}")))
}
