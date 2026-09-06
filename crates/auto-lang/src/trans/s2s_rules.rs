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

use crate::ast::{Arg, Args, Call, Code, Expr, Stmt, Type, UseKind};
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
                    // Plan 569 D2: `.len()` 特判——py 无 .len 方法（len 是
                    // 内置函数），py_call("len") 恒 AttributeError；改发
                    // obj_len 双通道组合子（GIL len / Auto 原生 len），
                    // 与 codegen 侧表路径（.at 直跑）殊途同归。
                    if method.as_str() == "len" && c.args.args.is_empty() {
                        *e = mk_call("obj_len", vec![Arg::Pos(*recv)]);
                        *changed = true;
                        return;
                    }
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
        // ---- 索引位（B3）：py-known 接收者（切片位形让位 B5 规则）----
        Expr::Index(recv, idx) => {
            rewrite_expr(recv, k, fname, changed);
            rewrite_expr(idx, k, fname, changed);
            let is_slice_shape = matches!(
                idx.as_ref(),
                Expr::Range(_) | Expr::Bina(_, _, _)
            );
            if recv_is_py(recv, k, fname) && !is_slice_shape {
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

/// Plan 560 T06: B5 切片族 + B6 len + A5 闭包包裹 + D7 句柄 print。
pub fn rule_b5_b6_a5_d7(code: &mut Code) -> AutoResult<bool> {
    let k = crate::trans::py_known::analyze(code);
    let mut changed = false;
    for stmt in code.stmts.iter_mut() {
        if let Stmt::Fn(f) = stmt {
            let fname = f.name.as_str().to_string();
            rewrite2_body(&mut f.body.stmts, &k, &fname, &mut changed);
        }
    }
    Ok(changed)
}

fn rewrite2_body(stmts: &mut [Stmt], k: &PyKnowledge, fname: &str, changed: &mut bool) {
    for stmt in stmts.iter_mut() {
        match stmt {
            Stmt::Store(s) => rewrite2_expr(&mut s.expr, k, fname, changed),
            Stmt::Expr(e) => rewrite2_expr(e, k, fname, changed),
            Stmt::Return(e) => rewrite2_expr(e, k, fname, changed),
            Stmt::If(i) => {
                for b in i.branches.iter_mut() {
                    rewrite2_expr(&mut b.cond, k, fname, changed);
                    rewrite2_body(&mut b.body.stmts, k, fname, changed);
                }
                if let Some(e) = i.else_.as_mut() {
                    rewrite2_body(&mut e.stmts, k, fname, changed);
                }
            }
            Stmt::For(f) => {
                rewrite2_expr(&mut f.range, k, fname, changed);
                rewrite2_body(&mut f.body.stmts, k, fname, changed);
            }
            Stmt::Try(t) => {
                rewrite2_body(&mut t.body.stmts, k, fname, changed);
                rewrite2_body(&mut t.catch_body.stmts, k, fname, changed);
            }
            Stmt::Block(b) => rewrite2_body(&mut b.stmts, k, fname, changed),
            _ => {}
        }
    }
}

fn rewrite2_expr(e: &mut Expr, k: &PyKnowledge, fname: &str, changed: &mut bool) {
    // 子节点先改（自底向上）——通用递归借 rule_ab_family 的容器面
    match e {
        Expr::Call(c) => {
            for a in c.args.args.iter_mut() {
                if let Arg::Pos(inner) = a {
                    rewrite2_expr(inner, k, fname, changed);
                }
            }
            match c.name.as_ref() {
                // B6：len(x) → obj_len(x)（双通道组合子；len 非保留字，
                // 用户自定义 fn len 的遮蔽面由链接序兜底——W2 罕见注记）
                Expr::Ident(n) if n.as_str() == "len" && c.args.args.len() == 1 => {
                    *e = mk_call("obj_len", vec![std::mem::replace(
                        &mut c.args.args[0],
                        Arg::Pos(Expr::Null),
                    )]);
                    *changed = true;
                }
                // D7：print(x) 中 x 为 py-known 裸名 → py_str 包裹。
                // 收敛边界：仅 Ident 形态（嵌套 handle 表达式由 print shim
                // 的运行期 PyObjectHandle→GIL str() 臂兜底——T06 实证：
                // 盲包会在二遍重入 obj_len/py_str 产物）。
                Expr::Ident(n) if n.as_str() == "print" => {
                    for a in c.args.args.iter_mut() {
                        if let Arg::Pos(inner) = a {
                            if matches!(inner, Expr::Ident(_)) && recv_is_py(inner, k, fname) {
                                let taken = std::mem::replace(inner, Expr::Null);
                                *inner = mk_call("py_str", vec![Arg::Pos(taken)]);
                                *changed = true;
                            }
                        }
                    }
                }
                // A5：py_* 调用的闭包实参自动包 py_callable
                // py_callable 自身排除（防二遍重入）；py_with 排除——
                // 其 codegen 内联形态要求**裸闭包**实参（539 inline
                // bracket），包裹会落 shim 的 closure-id 通道翻车
                //（T09 实证 Invalid closure ID）。
                Expr::Ident(n)
                    if n.as_str().starts_with("py_")
                        && n.as_str() != "py_callable"
                        && n.as_str() != "py_with" =>
                {
                    for a in c.args.args.iter_mut() {
                        if let Arg::Pos(inner) = a {
                            if matches!(inner, Expr::Closure(_)) {
                                let taken = std::mem::replace(inner, Expr::Null);
                                *inner = mk_call("py_callable", vec![Arg::Pos(taken)]);
                                *changed = true;
                            }
                        }
                    }
                }
                _ => {}
            }
        }
        // B5：切片族——py-known 接收者的 Range/步长索引 → py_getitem(x, py_slice(...))
        Expr::Index(recv, idx) => {
            rewrite2_expr(recv, k, fname, changed);
            rewrite2_expr(idx, k, fname, changed);
            let is_slice = matches!(
                idx.as_ref(),
                Expr::Range(_) | Expr::Bina(_, _, _)
            );
            if recv_is_py(recv, k, fname) && is_slice {
                // 先探形（借用），命中才取走
                if lower_slice_probe(idx) {
                    let recv_v = *std::mem::replace(recv, Box::new(Expr::Null));
                    let idx_v = *std::mem::replace(idx, Box::new(Expr::Null));
                    if let Some(slice_expr) = lower_slice(idx_v) {
                        *e = mk_call(
                            "py_getitem",
                            vec![Arg::Pos(recv_v), Arg::Pos(slice_expr)],
                        );
                        *changed = true;
                    }
                }
            }
        }
        _ => {
            // 借 A/B 族的递归骨架改写其余容器位（规则本体不再触发——
            // 已在链式前序处理过 A/B 面，此处仅为子表达式下钻）。
            rewrite_expr_shallow(e, k, fname, changed);
        }
    }
}

fn rewrite_expr_shallow(e: &mut Expr, k: &PyKnowledge, fname: &str, changed: &mut bool) {
    match e {
        Expr::Bina(l, _, r) => {
            rewrite2_expr(l, k, fname, changed);
            rewrite2_expr(r, k, fname, changed);
        }
        Expr::Array(elems) => {
            for el in elems.iter_mut() {
                rewrite2_expr(el, k, fname, changed);
            }
        }
        Expr::Some(i) | Expr::Ok(i) | Expr::Err(i) => rewrite2_expr(i, k, fname, changed),
        Expr::To { expr, .. } | Expr::Cast { expr, .. } => {
            rewrite2_expr(expr, k, fname, changed)
        }
        _ => {}
    }
}

/// 切片位形探针（借用判定，命中后由 lower_slice 取走构造）。
fn lower_slice_probe(idx: &Expr) -> bool {
    match idx {
        Expr::Range(_) => true,
        Expr::Bina(l, op, _)
            if std::mem::discriminant(op) == std::mem::discriminant(&auto_val::Op::Range)
                && matches!(l.as_ref(), Expr::Range(_)) =>
        {
            true
        }
        _ => false,
    }
}

/// `x[a..b]`/`x[..b]`/`x[a..]`/`x[a..=b]`/`x[a..b..c]` → `py_slice(a', b'[, c])`。
/// 开端 Nil/Null → null；`..=` → end+1；步长形三参。
fn lower_slice(idx: Expr) -> Option<Expr> {
    match idx {
        Expr::Range(r) => {
            let start = unwrap_range_arm(&r.start);
            // 步长位形（T06 实证）：a..b..c 解析为嵌套 Range(a, Range(b, c))
            if let Expr::Range(inner) = r.end.as_ref() {
                return Some(build_slice_call(
                    start,
                    unwrap_range_arm(&inner.start),
                    Some(unwrap_range_arm(&inner.end)),
                ));
            }
            let end = if r.eq {
                Expr::Bina(
                    Box::new(unwrap_range_arm(&r.end)),
                    auto_val::Op::Add,
                    Box::new(Expr::Int(1)),
                )
            } else {
                unwrap_range_arm(&r.end)
            };
            Some(build_slice_call(start, end, None))
        }
        Expr::Bina(l, op, r)
            if std::mem::discriminant(&op) == std::mem::discriminant(&auto_val::Op::Range) =>
        {
            // a..b..c：左 Range(a,b) + 步长 c
            match *l {
                Expr::Range(inner) => {
                    let start = unwrap_range_arm(&inner.start);
                    let end = unwrap_range_arm(&inner.end);
                    Some(build_slice_call(start, end, Some(*r)))
                }
                _ => None,
            }
        }
        _ => None,
    }
}

fn unwrap_range_arm(e: &Expr) -> Expr {
    match e {
        // 开端 Nil/Null → null 字面（py_slice 的 None 端）
        Expr::Nil | Expr::Null => Expr::Null,
        other => other.clone(),
    }
}

fn build_slice_call(start: Expr, end: Expr, step: Option<Expr>) -> Expr {
    let mut args = vec![Arg::Pos(start), Arg::Pos(end)];
    if let Some(s) = step {
        args.push(Arg::Pos(s));
    }
    mk_call("py_slice", args)
}

/// Plan 560 T08 (C3/C4)：py-known 句柄在真值位（if/while 条件、
/// `!`/`&&`/`||` 操作数）→ py_truthy 包裹（GIL bool——多元素张量按
/// Python 抛；Auto 值真值语义零打扰）。
pub fn rule_c34_truthy(code: &mut Code) -> AutoResult<bool> {
    let k = crate::trans::py_known::analyze(code);
    let mut changed = false;
    for stmt in code.stmts.iter_mut() {
        if let Stmt::Fn(f) = stmt {
            let fname = f.name.as_str().to_string();
            truthy_body(&mut f.body.stmts, &k, &fname, &mut changed);
        }
    }
    Ok(changed)
}

fn truthy_body(stmts: &mut [Stmt], k: &PyKnowledge, fname: &str, changed: &mut bool) {
    for stmt in stmts.iter_mut() {
        match stmt {
            Stmt::If(i) => {
                for b in i.branches.iter_mut() {
                    wrap_truthy(&mut b.cond, k, fname, changed);
                    truthy_body(&mut b.body.stmts, k, fname, changed);
                }
                if let Some(e) = i.else_.as_mut() {
                    truthy_body(&mut e.stmts, k, fname, changed);
                }
            }
            Stmt::For(f) => {
                if matches!(f.iter, crate::ast::Iter::Cond) {
                    wrap_truthy(&mut f.range, k, fname, changed);
                }
                truthy_body(&mut f.body.stmts, k, fname, changed);
            }
            Stmt::Try(t) => {
                truthy_body(&mut t.body.stmts, k, fname, changed);
                truthy_body(&mut t.catch_body.stmts, k, fname, changed);
            }
            Stmt::Block(b) => truthy_body(&mut b.stmts, k, fname, changed),
            Stmt::Store(s) => wrap_truthy_operands(&mut s.expr, k, fname, changed),
            Stmt::Expr(e) => wrap_truthy_operands(e, k, fname, changed),
            Stmt::Return(e) => wrap_truthy_operands(e, k, fname, changed),
            _ => {}
        }
    }
}

fn wrap_truthy(e: &mut Expr, k: &PyKnowledge, fname: &str, changed: &mut bool) {
    // 条件位整体是 py-known → 整体包裹（底层 Bool 结果比较式不包——
    // recv_is_py 只认 handle 形态，Call(py_==) 不在其中）。
    if recv_is_py(e, k, fname) {
        let taken = std::mem::replace(e, Expr::Null);
        *e = mk_call("py_truthy", vec![Arg::Pos(taken)]);
        *changed = true;
    }
}

fn wrap_truthy_operands(e: &mut Expr, k: &PyKnowledge, fname: &str, changed: &mut bool) {
    use auto_val::Op;
    match e {
        Expr::Unary(op, inner) if *op == Op::Not => {
            if recv_is_py(inner, k, fname) {
                let taken = *std::mem::replace(inner, Box::new(Expr::Null));
                *inner = Box::new(mk_call("py_truthy", vec![Arg::Pos(taken)]));
                *changed = true;
            } else {
                wrap_truthy_operands(inner, k, fname, changed);
            }
        }
        Expr::Bina(l, op, r) if *op == Op::And || *op == Op::Or => {
            for side in [l, r] {
                if recv_is_py(side, k, fname) {
                    let taken = *std::mem::replace(side, Box::new(Expr::Null));
                    **side = mk_call("py_truthy", vec![Arg::Pos(taken)]);
                    *changed = true;
                } else {
                    wrap_truthy_operands(side, k, fname, changed);
                }
            }
        }
        _ => {}
    }
}

// ============================================================================
// Plan 567 T09（P560-D2）: 隐式 !T 传播规则
// ============================================================================

/// 桥调用严格变体 → may 变体的改名表（kwargs 形态由 codegen
/// is_py_call_kw_form 路由 py_call_kw_may=478）。
const MAY_RENAMES: &[(&str, &str)] = &[
    ("py_call", "py_call_may"),
    ("py_getattr", "py_getattr_may"),
    ("py_getitem", "py_getitem_may"),
];

/// Plan 567 T09（P560-D2）: 隐式 `!T` 传播——含 use.py 的 `.as` 函数体内：
/// - 桥调用（py_call/py_getattr/py_getitem，含 kwargs 形态）→ may 变体
///   + `.?`（Err 落值通道，ERROR_PROPAGATE 帧展开或被 T08 值通道拦截进
///   catch）；
/// - 同文件用户函数调用点 → 追加 `.?`（裸值零开销透传——engine 裸值分支）。
///
/// 保守边界：已带 `.?` 或 `_may` 名不重包；`py_with`/`py_callable` 等
/// 全函数桥不涉；**无 use.py 的文件零改写**（纯 Auto `.as` 语料不受扰，
/// 含 legacy -1 哨兵返回值语义）。
pub fn rule_err_propagate(code: &mut Code) -> AutoResult<bool> {
    let has_py = code
        .stmts
        .iter()
        .any(|s| matches!(s, Stmt::Use(u) if matches!(u.kind, UseKind::Py)));
    if !has_py {
        return Ok(false);
    }
    let user_fns: std::collections::HashSet<String> = code
        .stmts
        .iter()
        .filter_map(|s| {
            if let Stmt::Fn(f) = s {
                Some(f.name.as_str().to_string())
            } else {
                None
            }
        })
        .collect();
    let mut changed = false;
    for stmt in code.stmts.iter_mut() {
        if let Stmt::Fn(f) = stmt {
            errprop_body(&mut f.body.stmts, &user_fns, &mut changed);
        }
    }
    Ok(changed)
}

fn errprop_body(stmts: &mut [Stmt], user_fns: &std::collections::HashSet<String>, changed: &mut bool) {
    for stmt in stmts.iter_mut() {
        match stmt {
            Stmt::Store(s) => errprop_expr(&mut s.expr, user_fns, changed),
            Stmt::Expr(e) => errprop_expr(e, user_fns, changed),
            Stmt::Return(e) => errprop_expr(e, user_fns, changed),
            Stmt::If(i) => {
                for b in i.branches.iter_mut() {
                    errprop_expr(&mut b.cond, user_fns, changed);
                    errprop_body(&mut b.body.stmts, user_fns, changed);
                }
                if let Some(e) = i.else_.as_mut() {
                    errprop_body(&mut e.stmts, user_fns, changed);
                }
            }
            Stmt::For(f) => {
                errprop_expr(&mut f.range, user_fns, changed);
                errprop_body(&mut f.body.stmts, user_fns, changed);
            }
            Stmt::Try(t) => {
                errprop_body(&mut t.body.stmts, user_fns, changed);
                errprop_body(&mut t.catch_body.stmts, user_fns, changed);
            }
            Stmt::Block(b) => errprop_body(&mut b.stmts, user_fns, changed),
            _ => {}
        }
    }
}

fn errprop_expr(e: &mut Expr, user_fns: &std::collections::HashSet<String>, changed: &mut bool) {
    match e {
        // 已传播形态：no-op（内层调用首遍已各自包裹；递归会二遍重包
        // `.?.?` ——幂等性由 corpus 测试钉死）。
        Expr::ErrorPropagate(_) => {}
        Expr::Call(c) => {
            // 实参自底向上先改。
            for a in c.args.args.iter_mut() {
                if let Arg::Pos(inner) = a {
                    errprop_expr(inner, user_fns, changed);
                }
            }
            if let Expr::Ident(n) = c.name.as_ref() {
                let name = n.as_str().to_string();
                if let Some((_, may)) = MAY_RENAMES.iter().find(|(from, _)| *from == name) {
                    *c.name = Expr::Ident((*may).into());
                    wrap_propagate(e, changed);
                } else if user_fns.contains(&name)
                    && !name.starts_with("py_")
                    && !name.starts_with("obj_")
                {
                    // 用户函数调用点：隐式可失败（调用约定值通道透传）。
                    wrap_propagate(e, changed);
                }
            } else if matches!(c.name.as_ref(), Expr::Dot(_, _)) {
                // Auto 方法糖保留态（ab 规则未动的非 py 接收者）：callee 位
                // 不包；实参已在上面的循环覆盖。
            }
        }
        // ---- 通用容器下钻 ----
        Expr::Bina(l, _, r) => {
            errprop_expr(l, user_fns, changed);
            errprop_expr(r, user_fns, changed);
        }
        Expr::Unary(_, inner) => errprop_expr(inner, user_fns, changed),
        Expr::Array(elems) => {
            for el in elems.iter_mut() {
                errprop_expr(el, user_fns, changed);
            }
        }
        Expr::Object(pairs) => {
            for p in pairs.iter_mut() {
                errprop_expr(&mut p.value, user_fns, changed);
            }
        }
        Expr::Some(inner) | Expr::Ok(inner) | Expr::Err(inner) => {
            errprop_expr(inner, user_fns, changed)
        }
        Expr::NullCoalesce(l, r) => {
            errprop_expr(l, user_fns, changed);
            errprop_expr(r, user_fns, changed);
        }
        Expr::To { expr, .. } | Expr::Cast { expr, .. } => {
            errprop_expr(expr, user_fns, changed)
        }
        Expr::Closure(cl) => errprop_expr(&mut cl.body, user_fns, changed),
        Expr::Block(b) => errprop_body(&mut b.stmts, user_fns, changed),
        Expr::If(i) => {
            for b in i.branches.iter_mut() {
                errprop_expr(&mut b.cond, user_fns, changed);
                errprop_body(&mut b.body.stmts, user_fns, changed);
            }
            if let Some(eb) = i.else_.as_mut() {
                errprop_body(&mut eb.stmts, user_fns, changed);
            }
        }
        Expr::Dot(recv, _) | Expr::Index(recv, _) => {
            errprop_expr(recv, user_fns, changed);
        }
        _ => {}
    }
}

fn wrap_propagate(e: &mut Expr, changed: &mut bool) {
    if matches!(e, Expr::ErrorPropagate(_)) {
        return;
    }
    let taken = std::mem::replace(e, Expr::Null);
    *e = Expr::ErrorPropagate(Box::new(taken));
    *changed = true;
}
