//! Plan 560 T05: s2s 规则批——A/B 族访问糖 lowering。
//!
//! 设计 §3 边界目录：语义全在桥/组合子上（§1 单一真相源），糖=AST 改写。
//! 分派策略（§1）：方法调用默认走 **obj_call 组合子**（py 句柄/Auto 值
//! 双通道，T13 窥孔再对静态已知 py 直达 py_xxx）；kwargs 形态走 py_call
//! （Auto 方法无 kwargs，py-only 糖）。B 族属性/索引/赋值只对 py-known
//! 接收者改写——Auto 结构体/数组语义零打扰。
//!
//! 已由既有管线覆盖、**无需 s2s 规则**的形态（考古结论）：
//! - A4 item-kwargs 构造 `Linear(2, 3, bias: false)`——codegen
//!   is_py_item_kw_form（539 T17）已直落 py_item_kw；
//! - A3 裸名调用 py 可调用值——依赖 use.py 直接调用通道 + py_call0 显式
//!   形态；流型自动改写归 T13 静态 py 分析的窥孔批。

use crate::ast::{Arg, Args, Call, Code, Expr, Stmt, Type};
use crate::error::AutoResult;
use crate::trans::py_known::{expr_is_py, PyKnowledge};

/// A1/A2 + B1/B2/B3/B4 规则（一条 transform，链式注册于 builtin_rules）。
pub fn rule_ab_family(code: &mut Code) -> AutoResult<bool> {
    let k = crate::trans::py_known::analyze(code);
    let mut changed = false;
    for stmt in code.stmts.iter_mut() {
        if let Stmt::Fn(f) = stmt {
            let fname = f.name.as_str().to_string();
            rewrite_body(&mut f.body.stmts, &k, &fname, &mut changed);
        }
    }
    Ok(changed)
}

fn rewrite_body(stmts: &mut [Stmt], k: &PyKnowledge, fname: &str, changed: &mut bool) {
    for stmt in stmts.iter_mut() {
        match stmt {
            Stmt::Store(s) => rewrite_expr(&mut s.expr, k, fname, changed),
            Stmt::Expr(e) => rewrite_expr(e, k, fname, changed),
            Stmt::Return(e) => rewrite_expr(e, k, fname, changed),
            Stmt::If(i) => {
                for b in i.branches.iter_mut() {
                    rewrite_expr(&mut b.cond, k, fname, changed);
                    rewrite_body(&mut b.body.stmts, k, fname, changed);
                }
                if let Some(e) = i.else_.as_mut() {
                    rewrite_body(&mut e.stmts, k, fname, changed);
                }
            }
            Stmt::For(f) => {
                rewrite_expr(&mut f.range, k, fname, changed);
                rewrite_body(&mut f.body.stmts, k, fname, changed);
            }
            Stmt::Try(t) => {
                rewrite_body(&mut t.body.stmts, k, fname, changed);
                rewrite_body(&mut t.catch_body.stmts, k, fname, changed);
            }
            Stmt::Block(b) => rewrite_body(&mut b.stmts, k, fname, changed),
            _ => {}
        }
    }
}

/// 接收者表达式是否静态已知 py（改写态内联判定 + 名字查分析）。
fn recv_is_py(e: &Expr, k: &PyKnowledge, fname: &str) -> bool {
    if let Expr::Ident(n) = e {
        return k.is_known(fname, n.as_str());
    }
    expr_is_py(e)
}

fn rewrite_expr(e: &mut Expr, k: &PyKnowledge, fname: &str, changed: &mut bool) {
    use auto_val::Op;
    match e {
        // ---- 调用位（A1/A2）：name 位置的 Dot 是方法糖 ----
        Expr::Call(c) => {
            // 实参先改（自底向上）
            for a in c.args.args.iter_mut() {
                if let Arg::Pos(inner) = a {
                    rewrite_expr(inner, k, fname, changed);
                }
            }
            let name_dot = matches!(c.name.as_ref(), Expr::Dot(_, _));
            if name_dot {
                let name = std::mem::replace(&mut c.name, Box::new(Expr::Null));
                let (recv, method) = match *name {
                    Expr::Dot(r, m) => (r, m),
                    other => {
                        c.name = Box::new(other);
                        return;
                    }
                };
                let has_kw = c
                    .args
                    .args
                    .iter()
                    .any(|a| matches!(a, Arg::Pair(_, _)));
                // A1/A2 只对 py-known 接收者改写（Auto 方法分派
                // `s.len()`/`xs.push()` 保持糖态走原生通道——T05 实证：
                // 盲改会让 obj_call 的 Auto 臂拒绝字符串方法）。
                if recv_is_py(&recv, k, fname) {
                    // py_call：kwargs 形态由 codegen is_py_call_kw_form
                    // 落 5-slot py_call_kw（539）。
                    let mut args = vec![Arg::Pos(*recv), Arg::Pos(Expr::Str(method))];
                    args.append(&mut c.args.args);
                    *e = mk_call("py_call", args);
                    *changed = true;
                } else {
                    c.name = Box::new(Expr::Dot(recv, method));
                }
            }
        }
        // ---- 属性位（B1/B9）：py-known 接收者 ----
        Expr::Dot(recv, field) => {
            rewrite_expr(recv, k, fname, changed);
            if recv_is_py(recv, k, fname) {
                let recv = *std::mem::replace(recv, Box::new(Expr::Null));
                *e = mk_call(
                    "py_getattr",
                    vec![Arg::Pos(recv), Arg::Pos(Expr::Str(field.clone()))],
                );
                *changed = true;
            }
        }
        // ---- 索引位（B3）：py-known 接收者 ----
        Expr::Index(recv, idx) => {
            rewrite_expr(recv, k, fname, changed);
            rewrite_expr(idx, k, fname, changed);
            if recv_is_py(recv, k, fname) {
                let recv = *std::mem::replace(recv, Box::new(Expr::Null));
                let idx = *std::mem::replace(idx, Box::new(Expr::Null));
                *e = mk_call("py_getitem", vec![Arg::Pos(recv), Arg::Pos(idx)]);
                *changed = true;
            }
        }
        // ---- 赋值位（B2/B4）：`m.weight = w` / `x[i] = v` ----
        Expr::Bina(l, op, r) if *op == Op::Asn => {
            rewrite_expr(r, k, fname, changed);
            match l.as_mut() {
                Expr::Dot(recv, field) => {
                    rewrite_expr(recv, k, fname, changed);
                    if recv_is_py(recv, k, fname) {
                        let recv = *std::mem::replace(recv, Box::new(Expr::Null));
                        let val = *std::mem::replace(r, Box::new(Expr::Null));
                        *e = mk_call(
                            "py_setattr",
                            vec![
                                Arg::Pos(recv),
                                Arg::Pos(Expr::Str(field.clone())),
                                Arg::Pos(val),
                            ],
                        );
                        *changed = true;
                    }
                }
                Expr::Index(recv, idx) => {
                    rewrite_expr(recv, k, fname, changed);
                    rewrite_expr(idx, k, fname, changed);
                    if recv_is_py(recv, k, fname) {
                        let recv = *std::mem::replace(recv, Box::new(Expr::Null));
                        let idx = *std::mem::replace(idx, Box::new(Expr::Null));
                        let val = *std::mem::replace(r, Box::new(Expr::Null));
                        *e = mk_call(
                            "py_setitem",
                            vec![Arg::Pos(recv), Arg::Pos(idx), Arg::Pos(val)],
                        );
                        *changed = true;
                    }
                }
                _ => rewrite_expr(l, k, fname, changed),
            }
        }
        // ---- 通用容器/包装递归 ----
        Expr::Bina(l, _, r) => {
            rewrite_expr(l, k, fname, changed);
            rewrite_expr(r, k, fname, changed);
        }
        Expr::Unary(_, inner) => rewrite_expr(inner, k, fname, changed),
        Expr::Array(elems) => {
            for el in elems.iter_mut() {
                rewrite_expr(el, k, fname, changed);
            }
        }
        Expr::Object(pairs) => {
            for p in pairs.iter_mut() {
                rewrite_expr(&mut p.value, k, fname, changed);
            }
        }
        Expr::Some(inner) | Expr::Ok(inner) | Expr::Err(inner) => {
            rewrite_expr(inner, k, fname, changed)
        }
        Expr::NullCoalesce(l, r) => {
            rewrite_expr(l, k, fname, changed);
            rewrite_expr(r, k, fname, changed);
        }
        Expr::ErrorPropagate(inner) => rewrite_expr(inner, k, fname, changed),
        Expr::To { expr, .. } | Expr::Cast { expr, .. } => {
            rewrite_expr(expr, k, fname, changed)
        }
        Expr::Closure(c) => rewrite_expr(&mut c.body, k, fname, changed),
        Expr::Block(b) => rewrite_body(&mut b.stmts, k, fname, changed),
        Expr::If(i) => {
            for b in i.branches.iter_mut() {
                rewrite_expr(&mut b.cond, k, fname, changed);
                rewrite_body(&mut b.body.stmts, k, fname, changed);
            }
            if let Some(eb) = i.else_.as_mut() {
                rewrite_body(&mut eb.stmts, k, fname, changed);
            }
        }
        _ => {}
    }
}

fn mk_call(name: &str, args: Vec<Arg>) -> Expr {
    Expr::Call(Call {
        name: Box::new(Expr::Ident(name.into())),
        args: Args { args },
        ret: Type::Unknown,
        type_args: Vec::new(),
        generic_args: Vec::new(),
        pos: None,
    })
}
