//! 迭代投影引擎（F787-R3-01）：**显式工作栈**替代全部相互递归——合法深链/
//! 嵌套块/调用在正常线程栈上不溢出（深度只体现在堆上的 Frame 栈）；访问
//! 预算超限受控 TooDeep。语义与原递归版逐构造对应（字段/求值顺序/错误面）。
//!
//! 帧协议：需要 N 个子值的帧先压"汇总帧"（栈底），再**逆序**压各子任务
//! （栈顶先完成）——子值按文档序留在 Val 栈上，汇总帧弹出 N 个组装。

use super::*;

pub(super) enum Val {
    Expr(Expr),
    Arg(Arg),
    Stmt(Stmt),
    Block(Block),
}

pub(super) enum Frame {
    /// 块：按孩子序消费 stmt 分派节点；耗尽后组装 Block。
    Block {
        node: u32,
        cursor: usize,
        stmts: Vec<Stmt>,
    },
    /// stmt 分派节点：内层 = 首个非 trivia/error 孩子（break/continue 叶）。
    StmtWrap(u32),
    /// 实语句节点（kind 分派）。
    Stmt(u32),
    /// let/var：value 表达式完成后组装（span 到 value 尾）。
    LetFinish {
        node: u32,
        kw: Span,
        name: Ident,
        ann: Option<TypeAnn>,
        mutable: bool,
    },
    AssignFinish {
        node: u32,
        name: Ident,
    },
    ReturnFinish {
        node: u32,
    },
    /// if 三段：cond → then → else 逐级传递已得值。
    IfCond {
        then_n: u32,
        els_blk: Option<u32>,
        node: u32,
    },
    IfThen {
        cond: Expr,
        els_blk: Option<u32>,
        node: u32,
    },
    IfElse {
        cond: Expr,
        then: Block,
        node: u32,
    },
    WhileCond {
        body_n: u32,
        node: u32,
    },
    WhileBody {
        cond: Expr,
        node: u32,
    },
    LoopBody {
        node: u32,
    },
    /// 表达式节点进入：按 kind 分派（叶构造直接产值，复合压子帧）。
    ExprEnter(u32),
    /// 二元运算：两个孩子表达式完成后组装。
    OpPost {
        op: BinOp,
    },
    /// 一元前缀（scalar）：操作数完成后组装（span = 操作数 span，与
    /// parser apply_unaries 一致）。
    UnaryPost {
        op: UnaryOp,
    },
    /// 调用实参汇总：全部实参 Arg 就绪后组装 Call；`unary_ops` 为 callee
    /// 包裹层的一元前缀链（外→内），按 parser apply_unaries 的施用序
    /// （内→外）包到 Call 结果上（`-g()` = Unary(Neg, Call)）。
    CallAssemble {
        callee: Ident,
        span: Span,
        argc: usize,
        unary_ops: Vec<UnaryOp>,
    },
    /// 单实参值 → Arg（命名/位置）。
    ArgBuild {
        named: Option<Ident>,
        span: Span,
    },
}

pub(super) fn run(
    p: &GrammarProgram,
    cx: &mut Cx,
    stack: &mut Vec<Frame>,
    vals: &mut Vec<Val>,
) -> PResult<()> {
    while let Some(frame) = stack.pop() {
        cx.visited += 1;
        if cx.visited > PROJECTION_NODE_BUDGET {
            return Err(ProjectionError::TooDeep);
        }
        match frame {
            Frame::Block {
                node,
                cursor,
                stmts,
            } => {
                let kids = cx.tree.children(node);
                // cursor 起的下一个 stmt 分派节点（孩子序）。
                let mut next = None;
                let mut c2 = cursor;
                while c2 < kids.len() {
                    let c = kids[c2];
                    c2 += 1;
                    if cx.tree.kind(c) == cx.k.stmt {
                        next = Some(c);
                        break;
                    }
                }
                match next {
                    Some(c) => {
                        // 恢复帧（cursor>0）：取回上一个 stmt 分派产物并入列。
                        let mut stmts = stmts;
                        if cursor > 0 {
                            match vals.pop() {
                                Some(Val::Stmt(st)) => stmts.push(st),
                                _ => return Err(ProjectionError::Unsupported("block stmt".into())),
                            }
                        }
                        stack.push(Frame::Block {
                            node,
                            cursor: c2,
                            stmts,
                        });
                        stack.push(Frame::StmtWrap(c));
                    }
                    None => {
                        let mut stmts = stmts;
                        if cursor > 0 {
                            match vals.pop() {
                                Some(Val::Stmt(st)) => stmts.push(st),
                                _ => return Err(ProjectionError::Unsupported("block stmt".into())),
                            }
                        }
                        vals.push(Val::Block(Block {
                            stmts,
                            span: span_of(cx.tree, node),
                        }));
                    }
                }
            }
            Frame::StmtWrap(c) => {
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
                if cx.tree.children(inner).is_empty() {
                    let sp = spelling(cx.tree, cx.snap, inner)?;
                    let span = span_of(cx.tree, inner);
                    let stmt = if sp == "break" {
                        Stmt::Break { span }
                    } else if sp == "continue" {
                        Stmt::Continue { span }
                    } else {
                        return Err(ProjectionError::Unsupported(format!(
                            "stmt kind {}",
                            cx.tree.kind(inner)
                        )));
                    };
                    vals.push(Val::Stmt(stmt));
                } else {
                    stack.push(Frame::Stmt(inner));
                }
            }
            Frame::Stmt(node) => step_stmt(p, cx, stack, node)?,
            Frame::LetFinish {
                node,
                kw,
                name,
                ann,
                mutable,
            } => {
                let value = pop_expr(vals, "let value")?;
                let sp = Span {
                    start: span_of(cx.tree, node).start,
                    end: value.span.end,
                };
                vals.push(Val::Stmt(Stmt::Let {
                    kw,
                    name,
                    ann,
                    mutable,
                    value,
                    span: sp,
                }));
            }
            Frame::AssignFinish { node, name } => {
                let value = pop_expr(vals, "assign value")?;
                let sp = Span {
                    start: span_of(cx.tree, node).start,
                    end: value.span.end,
                };
                vals.push(Val::Stmt(Stmt::Assign {
                    target: AssignTarget {
                        base: name,
                        path: Vec::new(),
                    },
                    value,
                    span: sp,
                }));
            }
            Frame::ReturnFinish { node } => {
                let value = pop_expr(vals, "return value")?;
                let sp = Span {
                    start: span_of(cx.tree, node).start,
                    end: value.span.end,
                };
                vals.push(Val::Stmt(Stmt::Return { value, span: sp }));
            }
            Frame::IfCond {
                then_n,
                els_blk,
                node,
            } => {
                let cond = pop_expr(vals, "if cond")?;
                stack.push(Frame::IfThen {
                    cond,
                    els_blk,
                    node,
                });
                stack.push(Frame::Block {
                    node: then_n,
                    cursor: 0,
                    stmts: Vec::new(),
                });
            }
            Frame::IfThen {
                cond,
                els_blk,
                node,
            } => {
                let then = pop_block(vals, "if then")?;
                match els_blk {
                    Some(eb) => {
                        stack.push(Frame::IfElse { cond, then, node });
                        stack.push(Frame::Block {
                            node: eb,
                            cursor: 0,
                            stmts: Vec::new(),
                        });
                    }
                    None => {
                        let span = span_of(cx.tree, node);
                        vals.push(Val::Stmt(Stmt::If {
                            cond,
                            then,
                            els: None,
                            span,
                        }));
                    }
                }
            }
            Frame::IfElse { cond, then, node } => {
                let els = pop_block(vals, "if else")?;
                let span = span_of(cx.tree, node);
                vals.push(Val::Stmt(Stmt::If {
                    cond,
                    then,
                    els: Some(els),
                    span,
                }));
            }
            Frame::WhileCond { body_n, node } => {
                let cond = pop_expr(vals, "while cond")?;
                stack.push(Frame::WhileBody { cond, node });
                stack.push(Frame::Block {
                    node: body_n,
                    cursor: 0,
                    stmts: Vec::new(),
                });
            }
            Frame::WhileBody { cond, node } => {
                let body = pop_block(vals, "while body")?;
                let span = span_of(cx.tree, node);
                vals.push(Val::Stmt(Stmt::While { cond, body, span }));
            }
            Frame::LoopBody { node } => {
                let body = pop_block(vals, "loop body")?;
                let span = span_of(cx.tree, node);
                vals.push(Val::Stmt(Stmt::Loop { body, span }));
            }
            Frame::UnaryPost { op } => {
                let operand = pop_expr(vals, "unary operand")?;
                let span = operand.span;
                vals.push(Val::Expr(Expr {
                    kind: ExprKind::Unary {
                        op,
                        operand: Box::new(operand),
                    },
                    span,
                }));
            }
            Frame::ExprEnter(node) => step_expr(cx, stack, vals, node)?,
            Frame::OpPost { op } => {
                let rhs = pop_expr(vals, "binary rhs")?;
                let lhs = pop_expr(vals, "binary lhs")?;
                let sp = Span {
                    start: lhs.span.start,
                    end: rhs.span.end,
                };
                vals.push(Val::Expr(Expr {
                    kind: ExprKind::Binary {
                        op,
                        lhs: Box::new(lhs),
                        rhs: Box::new(rhs),
                    },
                    span: sp,
                }));
            }
            Frame::CallAssemble {
                callee,
                span,
                argc,
                unary_ops,
            } => {
                let mut args = Vec::with_capacity(argc);
                for _ in 0..argc {
                    match vals.pop() {
                        Some(Val::Arg(a)) => args.push(a),
                        _ => return Err(ProjectionError::Unsupported("call arg".into())),
                    }
                }
                args.reverse(); // 弹栈逆序 → 文档序。
                let mut e = Expr {
                    kind: ExprKind::Call { callee, args },
                    span,
                };
                for op in unary_ops.iter().rev() {
                    let sp = e.span;
                    e = Expr {
                        kind: ExprKind::Unary {
                            op: *op,
                            operand: Box::new(e),
                        },
                        span: sp,
                    };
                }
                vals.push(Val::Expr(e));
            }
            Frame::ArgBuild { named, span } => {
                let e = pop_expr(vals, "arg value")?;
                let arg = match named {
                    Some(name) => Arg::Named {
                        name,
                        value: e,
                        span,
                    },
                    None => Arg::Pos(e),
                };
                vals.push(Val::Arg(arg));
            }
        }
    }
    Ok(())
}

fn step_stmt(p: &GrammarProgram, cx: &mut Cx, stack: &mut Vec<Frame>, node: u32) -> PResult<()> {
    let kind = cx.tree.kind(node);
    if kind == cx.k.let_stmt || kind == cx.k.var_stmt {
        let name_f = p.field_id("name").unwrap();
        let value_f = p.field_id("value").unwrap();
        let name_n = tree_field_child(cx, node, name_f, "let/var name")?;
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
                    .ok_or_else(|| ProjectionError::Unsupported("let ann type".into()))?;
                Some(project_type(cx, t)?)
            }
            None => None,
        };
        let val_n = tree_field_child(cx, node, value_f, "let/var value")?;
        let kw_kind = if kind == cx.k.let_stmt {
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
        let kw = span_of(cx.tree, kw_n);
        let name = Ident {
            name: spelling(cx.tree, cx.snap, name_n)?,
            span: span_of(cx.tree, name_n),
        };
        let mutable = kind == cx.k.var_stmt;
        stack.push(Frame::LetFinish {
            node,
            kw,
            name,
            ann,
            mutable,
        });
        stack.push(Frame::ExprEnter(val_n));
    } else if kind == cx.k.assign_stmt {
        let name_f = p.field_id("name").unwrap();
        let value_f = p.field_id("value").unwrap();
        let name_n = tree_field_child(cx, node, name_f, "assign name")?;
        let val_n = tree_field_child(cx, node, value_f, "assign value")?;
        let name = Ident {
            name: spelling(cx.tree, cx.snap, name_n)?,
            span: span_of(cx.tree, name_n),
        };
        stack.push(Frame::AssignFinish { node, name });
        stack.push(Frame::ExprEnter(val_n));
    } else if kind == cx.k.return_stmt {
        let value_f = p.field_id("value").unwrap();
        let val_n = tree_field_child(cx, node, value_f, "return value")?;
        stack.push(Frame::ReturnFinish { node });
        stack.push(Frame::ExprEnter(val_n));
    } else if kind == cx.k.if_stmt {
        let cond_f = p.field_id("cond").unwrap();
        let then_f = p.field_id("then").unwrap();
        let cond_n = tree_field_child(cx, node, cond_f, "if cond")?;
        let then_n = tree_field_child(cx, node, then_f, "if then")?;
        let els_blk = match cx
            .tree
            .children(node)
            .iter()
            .find(|&&c| cx.tree.kind(c) == cx.k.else_block)
        {
            Some(&eb) => Some(
                *cx.tree
                    .children(eb)
                    .iter()
                    .find(|&&c| cx.tree.kind(c) == cx.k.block)
                    .ok_or_else(|| ProjectionError::Unsupported("else block".into()))?,
            ),
            None => None,
        };
        stack.push(Frame::IfCond {
            then_n,
            els_blk,
            node,
        });
        stack.push(Frame::ExprEnter(cond_n));
    } else if Some(kind) == cx.k.while_stmt || Some(kind) == cx.k.for_stmt {
        // scalar 条件 for 复用 While 的 AST 形状（PLAN-767 §5.2.1）；两语法
        // 的 cond/body 字段名一致，投影同构。
        let cond_f = p.field_id("cond").unwrap();
        let body_f = p.field_id("body").unwrap();
        let cond_n = tree_field_child(cx, node, cond_f, "while/for cond")?;
        let body_n = tree_field_child(cx, node, body_f, "while/for body")?;
        stack.push(Frame::WhileCond { body_n, node });
        stack.push(Frame::ExprEnter(cond_n));
    } else if kind == cx.k.loop_stmt {
        let body_f = p.field_id("body").unwrap();
        let body_n = tree_field_child(cx, node, body_f, "loop body")?;
        stack.push(Frame::LoopBody { node });
        stack.push(Frame::Block {
            node: body_n,
            cursor: 0,
            stmts: Vec::new(),
        });
    } else {
        return Err(ProjectionError::Unsupported(format!("stmt kind {kind}")));
    }
    Ok(())
}

fn step_expr(cx: &mut Cx, stack: &mut Vec<Frame>, vals: &mut Vec<Val>, node: u32) -> PResult<()> {
    let kind = cx.tree.kind(node);
    if kind == cx.k.int_literal {
        let kids = cx.tree.children(node);
        let int_n = *kids
            .iter()
            .find(|&&c| cx.tree.kind(c) == cx.k.tok_int)
            .ok_or_else(|| ProjectionError::Unsupported("int literal token".into()))?;
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
        let span = span_of(cx.tree, node);
        vals.push(Val::Expr(Expr {
            kind: ExprKind::IntLit(v as i32),
            span,
        }));
    } else if kind == KIND_CALL {
        let kids = cx.tree.children(node);
        // callee 包裹层：core = atom-expr；scalar = unary-expr（primary 规则
        // 更名，可带一元前缀链 `- -g(...)`）。先剥前缀链（外→内），再在
        // 最内包裹层限深 DFS 找首个 ident 叶。
        let callee_n = *kids
            .iter()
            .find(|&&c| {
                cx.tree.kind(c) == cx.k.atom_expr || Some(cx.tree.kind(c)) == cx.k.unary_expr
            })
            .ok_or_else(|| ProjectionError::Unsupported("call callee wrap".into()))?;
        let mut unary_ops: Vec<UnaryOp> = Vec::new();
        let mut wrap = callee_n;
        while Some(cx.tree.kind(wrap)) == cx.k.unary_expr {
            let wk = cx.tree.children(wrap);
            let has_minus = wk.iter().any(|&c| cx.tree.kind(c) == cx.k.tok_minus);
            let has_bang = wk.iter().any(|&c| Some(cx.tree.kind(c)) == cx.k.tok_bang);
            if !has_minus && !has_bang {
                break;
            }
            unary_ops.push(if has_bang { UnaryOp::Not } else { UnaryOp::Neg });
            let inner = wk
                .iter()
                .find(|&&c| Some(cx.tree.kind(c)) == cx.k.unary_expr)
                .copied();
            match inner {
                Some(i) => wrap = i,
                None => break,
            }
        }
        let ident_n = {
            let mut found: Option<u32> = None;
            let mut work: Vec<(u32, u8)> = vec![(wrap, 0)];
            while let Some((n, depth)) = work.pop() {
                if depth > 2 {
                    continue;
                }
                for c in cx.tree.children(n) {
                    if cx.tree.kind(c) == cx.k.tok_ident {
                        found = Some(c);
                    } else {
                        work.push((c, depth + 1));
                    }
                }
                if found.is_some() {
                    break;
                }
            }
            found.ok_or_else(|| ProjectionError::Unsupported("call callee".into()))?
        };
        let callee = Ident {
            name: spelling(cx.tree, cx.snap, ident_n)?,
            span: span_of(cx.tree, ident_n),
        };
        // Call span：callee ident 起点到括号尾（不含一元前缀——与 parser 的
        // Call span 一致；core 下 ident 起点即节点起点）。
        let span = Span {
            start: callee.span.start,
            end: span_of(cx.tree, node).end,
        };
        if let Some(&args_n) = kids.iter().find(|&&c| cx.tree.kind(c) == cx.k.args) {
            // args := arg (, arg)*：本层 arg 直取、arg_tail 内的 arg 展开，
            // 按位置序（与原实现一致）。
            let mut direct: Vec<u32> = Vec::new();
            for c2 in cx.tree.children(args_n) {
                let ck = cx.tree.kind(c2);
                if ck == cx.k.arg {
                    direct.push(c2);
                } else if ck == cx.k.arg_tail {
                    for gc in cx.tree.children(c2) {
                        if cx.tree.kind(gc) == cx.k.arg {
                            direct.push(gc);
                        }
                    }
                }
            }
            direct.sort_by_key(|&n| cx.tree.abs_range(n).0);
            // 汇总帧压底，各实参 [ArgBuild, ExprEnter] 逆序压栈 → 文档序完成。
            let argc = direct.len();
            stack.push(Frame::CallAssemble {
                callee,
                span,
                argc,
                unary_ops,
            });
            for a in direct.into_iter().rev() {
                let named_n = cx
                    .tree
                    .children(a)
                    .iter()
                    .find(|&c| cx.tree.kind(*c) == cx.k.named_arg)
                    .copied();
                let arg_span = span_of(cx.tree, a);
                let named = match named_n {
                    Some(na) => {
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
                            .ok_or_else(|| ProjectionError::Unsupported("named value".into()))?;
                        let label = Ident {
                            name: spelling(cx.tree, cx.snap, label_n)?,
                            span: span_of(cx.tree, label_n),
                        };
                        stack.push(Frame::ArgBuild {
                            named: Some(label),
                            span: arg_span,
                        });
                        val_n
                    }
                    None => {
                        let e_n = *cx
                            .tree
                            .children(a)
                            .iter()
                            .find(|&&c| cx.tree.kind(c) == cx.k.expr_rule)
                            .ok_or_else(|| ProjectionError::Unsupported("pos arg".into()))?;
                        stack.push(Frame::ArgBuild {
                            named: None,
                            span: arg_span,
                        });
                        e_n
                    }
                };
                stack.push(Frame::ExprEnter(named));
            }
        } else {
            let mut e = Expr {
                kind: ExprKind::Call {
                    callee,
                    args: Vec::new(),
                },
                span,
            };
            for op in unary_ops.iter().rev() {
                let sp = e.span;
                e = Expr {
                    kind: ExprKind::Unary {
                        op: *op,
                        operand: Box::new(e),
                    },
                    span: sp,
                };
            }
            vals.push(Val::Expr(e));
        }
    } else if kind == cx.k.op_plus
        || kind == cx.k.op_star
        || kind == cx.k.op_lt
        || Some(kind) == cx.k.op_minus
        || Some(kind) == cx.k.op_slash
        || Some(kind) == cx.k.op_percent
        || Some(kind) == cx.k.op_gt
        || Some(kind) == cx.k.op_le
        || Some(kind) == cx.k.op_ge
        || Some(kind) == cx.k.op_eq
        || Some(kind) == cx.k.op_ne
        || Some(kind) == cx.k.op_andand
        || Some(kind) == cx.k.op_oror
    {
        let kids = cx.tree.children(node);
        if kids.len() < 2 {
            return Err(ProjectionError::Unsupported("binary operands".into()));
        }
        let lhs_n = kids[0];
        let rhs_n = kids[kids.len() - 1];
        let op = if kind == cx.k.op_plus {
            BinOp::Add
        } else if kind == cx.k.op_star {
            BinOp::Mul
        } else if Some(kind) == cx.k.op_minus {
            BinOp::Sub
        } else if Some(kind) == cx.k.op_slash {
            BinOp::Div
        } else if Some(kind) == cx.k.op_percent {
            BinOp::Rem
        } else if Some(kind) == cx.k.op_gt {
            BinOp::Gt
        } else if Some(kind) == cx.k.op_le {
            BinOp::Le
        } else if Some(kind) == cx.k.op_ge {
            BinOp::Ge
        } else if Some(kind) == cx.k.op_eq {
            BinOp::Eq
        } else if Some(kind) == cx.k.op_ne {
            BinOp::Ne
        } else if Some(kind) == cx.k.op_andand {
            BinOp::And
        } else if Some(kind) == cx.k.op_oror {
            BinOp::Or
        } else {
            BinOp::Lt
        };
        stack.push(Frame::OpPost { op });
        stack.push(Frame::ExprEnter(rhs_n));
        stack.push(Frame::ExprEnter(lhs_n));
    } else if Some(kind) == cx.k.unary_expr {
        // scalar 一元前缀（右结合）：`-x` / `!(...)`；字面量/原子分支直取
        // 唯一内层孩子。Unary 的 span = 操作数 span（与 parser apply_unaries
        // 一致——不含前缀符本身）。
        let kids = cx.tree.children(node);
        let has_minus = kids.iter().any(|&c| cx.tree.kind(c) == cx.k.tok_minus);
        let has_bang = kids
            .iter()
            .any(|&c| Some(cx.tree.kind(c)) == cx.k.tok_bang);
        if has_minus || has_bang {
            let operand_n = *kids
                .iter()
                .find(|&&c| Some(cx.tree.kind(c)) == cx.k.unary_expr)
                .ok_or_else(|| ProjectionError::Unsupported("unary operand".into()))?;
            let op = if has_bang { UnaryOp::Not } else { UnaryOp::Neg };
            stack.push(Frame::UnaryPost { op });
            stack.push(Frame::ExprEnter(operand_n));
        } else {
            let inner = *kids
                .iter()
                .find(|&&c| {
                    cx.tree.kind(c) == cx.k.int_literal || cx.tree.kind(c) == cx.k.atom_expr
                })
                .ok_or_else(|| ProjectionError::Unsupported("unary-expr empty".into()))?;
            stack.push(Frame::ExprEnter(inner));
        }
    } else if kind == cx.k.atom_expr {
        // 按孩子序取第一个可识别内层（int-literal/布尔/ident/括号组）。
        let mut handled = false;
        for c in cx.tree.children(node) {
            let ck = cx.tree.kind(c);
            if ck == cx.k.int_literal || ck == cx.k.expr_rule {
                stack.push(Frame::ExprEnter(c));
                handled = true;
                break;
            }
            if ck == cx.k.tok_true || ck == cx.k.tok_false {
                let v = ck == cx.k.tok_true;
                let span = span_of(cx.tree, node);
                vals.push(Val::Expr(Expr {
                    kind: ExprKind::BoolLit(v),
                    span,
                }));
                handled = true;
                break;
            }
            if ck == cx.k.tok_ident {
                let span = span_of(cx.tree, c);
                let name = spelling(cx.tree, cx.snap, c)?;
                vals.push(Val::Expr(Expr {
                    kind: ExprKind::Var(Ident { name, span }),
                    span,
                }));
                handled = true;
                break;
            }
        }
        if !handled {
            return Err(ProjectionError::Unsupported("atom-expr empty".into()));
        }
    } else if kind == cx.k.expr_rule {
        // expr 分派节点 → 内层 pratt 产物（首个有孩子的孩子）。
        let mut inner = None;
        for c in cx.tree.children(node) {
            if cx.tree.child_count(c) > 0 {
                inner = Some(c);
                break;
            }
        }
        match inner {
            Some(c) => stack.push(Frame::ExprEnter(c)),
            None => return Err(ProjectionError::Unsupported("empty expr".into())),
        }
    } else {
        return Err(ProjectionError::Unsupported(format!("expr kind {kind}")));
    }
    Ok(())
}

pub(super) fn pop_expr(vals: &mut Vec<Val>, what: &str) -> PResult<Expr> {
    match vals.pop() {
        Some(Val::Expr(e)) => Ok(e),
        _ => Err(ProjectionError::Unsupported(what.into())),
    }
}

pub(super) fn pop_block(vals: &mut Vec<Val>, what: &str) -> PResult<Block> {
    match vals.pop() {
        Some(Val::Block(b)) => Ok(b),
        _ => Err(ProjectionError::Unsupported(what.into())),
    }
}
