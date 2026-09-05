//! Plan 555 T07 / Plan 560 T02: s2s 改写器——Auto（脚本糖）→ Auto（正常模式桥）。
//!
//! 设计源 `docs/design/strategy/script-mode-interop.md` §1：改写器独立
//! source-to-source 工具先行（Babel 模式），产出直接跑现有门禁；稳定后
//! 折进编译器模式管线（multi_mode.rs），**不**塞进 codegen 热臂（§10
//! 裁决）。每条规则配 source-to-source 单测。
//!
//! 帧形（P555-D2 裁定，W2 落地）：**AST 帧形 + 链式规则**——
//! `lower_source` = parse → 规则表有序全过（后继规则看见前驱改写，
//! 单遍）→ `emit::emit_code` 发射正常模式源。W1 的 token 粒度帧退役
//! （无法表达 E2 语句展开与表达式重排）。

use crate::ast::Code;
use crate::error::AutoResult;

/// 一条 lowering 规则。设计 §3 边界目录按 `A1..F4` 编号键控。
///
/// 链式语义：`lower_source` 按注册序单遍全过，每条规则就地改写
/// `Code` 并返回是否命中（诊断/测试用）；后继规则看见前驱的产物
/// （如 B9 dotted 递归在 B1 属性改写之后仍能吃到模块句柄形态）。
pub struct LoweringRule {
    /// 边界目录编号（"A2"/"B5"/"E2"...），诊断与单测键控。
    pub id: &'static str,
    /// AST 改写：就地改写，返回是否命中。
    pub transform: fn(&mut Code) -> AutoResult<bool>,
}

/// 内置规则表。W2 各批任务（T05-T11）逐条注册；每条配
/// source-to-source 单测（§9 纪律）。
pub fn builtin_rules() -> Vec<LoweringRule> {
    vec![
        // Plan 560 T05: A/B 族访问糖（obj_call 双通道 / py-known B 族 /
        // kwargs→py_call）。链式序：后续 C/E 族规则在其后注册。
        LoweringRule {
            id: "A1/A2+B1-B4",
            transform: crate::trans::s2s_rules::rule_ab_family,
        },
        // Plan 560 T06: B5 切片族 + B6 len + A5 闭包包裹 + D7 句柄 print。
        LoweringRule {
            id: "B5/B6/A5/D7",
            transform: crate::trans::s2s_rules::rule_b5_b6_a5_d7,
        },
        // Plan 560 T08: C3/C4 句柄真值（if/while/and/or/not → py_truthy）。
        LoweringRule {
            id: "C3/C4",
            transform: crate::trans::s2s_rules::rule_c34_truthy,
        },
    ]
}

/// 对源码应用规则表（parse → 链式规则 → AST 发射）。
///
/// - 输入必须可解析（改写器不吃非法源）；
/// - 空表 = 规范化 identity（AST 发射产物，语义等价、布局规范化——
///   幂等性由单测钉死：lower(lower(x)) == lower(x)）。
pub fn lower_source(src: &str) -> AutoResult<String> {
    let mut parser = crate::parser::Parser::new(src);
    let mut code = parser.parse()?;
    for rule in builtin_rules() {
        (rule.transform)(&mut code)?;
    }
    crate::trans::emit::emit_code(&code)
}

#[cfg(test)]
mod tests_s2s {
    use super::*;

    const SAMPLE: &str = r#"use.py torch: arange
fn fib(n int) -> int {
    if n < 2 { return n }
    return fib(n - 1) + fib(n - 2)
}
fn main() {
    var xs = [1, 2, 3]
    for x in xs {
        print(obj_len(xs))
    }
}
"#;

    /// identity 稳定性（AST 帧形）：规范化 identity + 幂等
    /// （lower(lower(x)) == lower(x)，产物再解析成功）。
    #[test]
    fn test_s2s_identity_roundtrip_stable() {
        let once = lower_source(SAMPLE).unwrap();
        // 产物必须可再解析
        let mut p = crate::parser::Parser::new(&once);
        assert!(p.parse().is_ok(), "产物必须可解析:\n{}", once);
        // 幂等
        let twice = lower_source(&once).unwrap();
        assert_eq!(twice, once, "lower 必须幂等");
        // 语义锚点抽查
        assert!(once.contains("use.py torch: arange"), "{}", once);
        assert!(once.contains("fn fib(n int) -> int"), "{}", once);
        assert!(once.contains("obj_len(xs)"), "{}", once);
    }

    /// 可扩展性：注入一条测试规则（调用名 obj_len → interop_len）即生效
    /// ——证明规则表注册点真的被消费（AST 帧形）。
    #[test]
    fn test_s2s_test_rule_injection_takes_effect() {
        let rule = LoweringRule {
            id: "TEST-1",
            transform: |code: &mut Code| {
                let mut hit = false;
                for stmt in code.stmts.iter_mut() {
                    if let crate::ast::Stmt::Fn(f) = stmt {
                        rename_calls(&mut f.body.stmts, &mut hit);
                    }
                }
                Ok(hit)
            },
        };
        let mut parser = crate::parser::Parser::new(SAMPLE);
        let mut code = parser.parse().unwrap();
        let changed = (rule.transform)(&mut code).unwrap();
        assert!(changed, "规则必须命中");
        let out = crate::trans::emit::emit_code(&code).unwrap();
        assert!(out.contains("interop_len("), "注入即生效: {}", out);
        assert!(!out.contains("obj_len"), "{}", out);
        // 产物必须仍可解析
        let mut p2 = crate::parser::Parser::new(&out);
        assert!(p2.parse().is_ok());
    }


    fn rename_in_expr(e: &mut crate::ast::Expr, hit: &mut bool) {
        use crate::ast::{Arg, Expr};
        match e {
            Expr::Call(c) => {
                if let Expr::Ident(n) = c.name.as_ref() {
                    if n.as_str() == "obj_len" {
                        c.name = Box::new(Expr::Ident("interop_len".into()));
                        *hit = true;
                    }
                }
                for a in c.args.args.iter_mut() {
                    if let Arg::Pos(inner) = a {
                        rename_in_expr(inner, hit);
                    }
                }
            }
            Expr::Bina(l, _, r) => {
                rename_in_expr(l, hit);
                rename_in_expr(r, hit);
            }
            Expr::Dot(inner, _) => rename_in_expr(inner, hit),
            _ => {}
        }
    }

    fn rename_calls(stmts: &mut [crate::ast::Stmt], hit: &mut bool) {
        use crate::ast::{Expr, Stmt};
        for stmt in stmts.iter_mut() {
            match stmt {
                Stmt::Expr(e) => rename_in_expr(e, hit),
                Stmt::If(i) => {
                    for b in i.branches.iter_mut() {
                        rename_calls(&mut b.body.stmts, hit);
                    }
                    if let Some(e) = i.else_.as_mut() {
                        rename_calls(&mut e.stmts, hit);
                    }
                }
                Stmt::For(f) => rename_calls(&mut f.body.stmts, hit),
                Stmt::Try(t) => {
                    rename_calls(&mut t.body.stmts, hit);
                    rename_calls(&mut t.catch_body.stmts, hit);
                }
                Stmt::Fn(f) => rename_calls(&mut f.body.stmts, hit),
                _ => {}
            }
        }
    }

    /// Plan 560 T05: A/B 族规则 source-to-source 断言（§9 纪律）。
    #[test]
    fn test_s2s_rule_ab_family() {
        let src = r#"use.py torch: arange
fn main() {
    var t = arange(6)
    print(t.sum())
    var w = t.shape
    var v = t[0]
    t.note = "hi"
    t[0] = 9
    var s = "plain"
    var n = s.len()
    var arr = [1, 2]
    var e0 = arr[0]
}
"#;
        let out = lower_source(src).unwrap();
        // A1 方法调用（py-known 接收者）→ py_call
        assert!(out.contains("py_call(t, \"sum\")"), "{}", out);
        // B1 属性（arange 结果 py-known）→ py_getattr
        assert!(out.contains("py_getattr(t, \"shape\")"), "{}", out);
        // B3 索引 → py_getitem
        assert!(out.contains("py_getitem(t, 0)"), "{}", out);
        // B2 属性赋值 → py_setattr
        assert!(out.contains("py_setattr(t, \"note\", \"hi\")"), "{}", out);
        // B4 索引赋值 → py_setitem
        assert!(out.contains("py_setitem(t, 0, 9)"), "{}", out);
        // Auto 接收者零打扰：字符串 .len() 与数组索引保持原样
        assert!(out.contains("s.len()"), "{}", out);
        assert!(out.contains("arr[0]"), "{}", out);
        // 产物可再解析（幂等性由 corpus 测试全局钉）
        let mut p = crate::parser::Parser::new(&out);
        assert!(p.parse().is_ok(), "{}", out);
    }

    /// Plan 560 T06: B5/B6/A5/D7 规则 source-to-source 断言。
    #[test]
    fn test_s2s_rule_b5_b6_a5_d7() {
        let src = r#"use.py torch: arange
fn main() {
    var x = arange(10)
    var a = x[2..5]
    var b = x[..5]
    var c = x[2..]
    var d = x[1..9..2]
    var n = len(x)
    print(x)
    py_map(x, () => { print(1) })
}
"#;
        let out = lower_source(src).unwrap();
        assert!(out.contains("py_getitem(x, py_slice(2, 5))"), "{}", out);
        assert!(out.contains("py_slice(null, 5)"), "{}", out);
        assert!(out.contains("py_slice(2, null)"), "{}", out);
        assert!(out.contains("py_slice(1, 9, 2)"), "{}", out);
        assert!(out.contains("obj_len(x)"), "{}", out);
        assert!(out.contains("print(py_str(x))"), "{}", out);
        assert!(out.contains("py_callable("), "A5 闭包包裹: {}", out);
        let mut p = crate::parser::Parser::new(&out);
        assert!(p.parse().is_ok(), "{}", out);
    }

    /// 非法源拒绝：语法验证兜底。
    #[test]
    fn test_s2s_rejects_invalid_source() {
        assert!(lower_source("fn broken {{{{").is_err());
    }

    /// 真实语料 round-trip：py parity 套件源（T12 迁移目标面）逐文件
    /// parse→emit→re-parse 幂等。发射器覆盖面的实弹靶场。
    #[test]
    fn test_s2s_py_suite_corpus_roundtrip() {
        let corpus = [
            "parity/libs/python/py_math/tests/auto/math.as",
            "parity/libs/python/py_torch_infer/tests/auto/infer.as",
            "parity/libs/python/py_torch_train/tests/auto/train.as",
            "parity/libs/python/py_numpy/tests/auto/numpy.as",
            "parity/libs/python/py_torch/tests/auto/torch.as",
        ];
        let manifest = env!("CARGO_MANIFEST_DIR");
        for rel in corpus {
            let path = std::path::Path::new(manifest).join("../../").join(rel);
            let Ok(src) = std::fs::read_to_string(&path) else {
                panic!("corpus missing: {}", path.display());
            };
            let once = lower_source(&src)
                .unwrap_or_else(|e| panic!("{}: lower 失败: {}", rel, e));
            let mut p = crate::parser::Parser::new(&once);
            assert!(p.parse().is_ok(), "{}: 产物不可再解析:\n{}", rel, once);
            let twice = lower_source(&once)
                .unwrap_or_else(|e| panic!("{}: 二次 lower 失败: {}", rel, e));
            assert_eq!(twice, once, "{}: 不幂等", rel);
        }
    }
}
