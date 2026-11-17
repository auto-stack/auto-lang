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
use crate::trans::emit::EmitDialect;
use crate::trans::SourceMapEntry;

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
        // Plan 567 T09（P560-D2）: 隐式 !T 传播（桥调用→may+.?、用户
        // 函数调用点+.?；无 use.py 文件零改写）。链尾注册——看见前序
        // 规则的全部桥改写产物。
        LoweringRule {
            id: "E1",
            transform: crate::trans::s2s_rules::rule_err_propagate,
        },
    ]
}

/// 对源码应用规则表（parse → 链式规则 → AST 发射）。
///
/// - 输入必须可解析（改写器不吃非法源）；
/// - 空表 = 规范化 identity（AST 发射产物，语义等价、布局规范化——
///   幂等性由单测钉死：lower(lower(x)) == lower(x)）。
pub fn lower_source(src: &str) -> AutoResult<String> {
    lower_in(src, EmitDialect::Legacy).map(|(text, _)| text)
}

/// PLAN-977 B-REVERSE-I: 改写 + 行级 source map（历史方言，产物逐字节
/// 等于 [`lower_source`]）。
///
/// map 粒度 = 顶层语句起点行（`Code::source_lines` 口径）；同一语句内
/// 与原文逐行同形的产物行细化为该原行（见 [`refine_map_with_source`]），
/// 故未改写行的映射是精确行、被改写行回落到语句起点行。
pub fn lower_source_with_map(src: &str) -> AutoResult<(String, Vec<SourceMapEntry>)> {
    lower_in(src, EmitDialect::Legacy)
}

/// PLAN-977 B-REVERSE-I: 改写 + ACE 规范形 + source map。
///
/// 产物面 = ACE 执行器（M2 唯一执行器）frontend 接受的规范 Auto 源：
/// fn 返回类型不带历史 `->`（见 [`EmitDialect::AceCanonical`]）。调用方
/// 把产物连同 map 交给 ACE 门（`auto_lang_core::run_ace`），诊断位置用
/// [`crate::trans::emit::source_line_for_output`] 回到原源行。
pub fn lower_source_for_ace(src: &str) -> AutoResult<(String, Vec<SourceMapEntry>)> {
    lower_in(src, EmitDialect::AceCanonical)
}

fn lower_in(src: &str, dialect: EmitDialect) -> AutoResult<(String, Vec<SourceMapEntry>)> {
    let mut parser = crate::parser::Parser::new(src);
    let mut code = parser.parse()?;
    for rule in builtin_rules() {
        (rule.transform)(&mut code)?;
    }
    let (text, map) = crate::trans::emit::emit_code_with_map(&code, dialect)?;
    let map = refine_map_with_source(src, &text, &map);
    Ok((text, map))
}

/// 把语句粒度的 map 细化为"逐行同形即精确行"。
///
/// 每条顶层语句的产物行原本都记语句起点行；同一语句内，若某产物行的
/// trim 文本在原源的该语句行区间内（起点行到下一语句起点行之前）按序
/// 找到完全相同的行，或（≥6 字符时）作为该行的子串，则把该产物行改记到
/// 那一原行。结构行（以 `{` 收尾的块头、`}`/`else` 片段）与空行不参与；
/// 被规则改写过的行（文本变了且不再是任何原行的子串）保持语句起点行——
/// **不猜**近似位置。
fn refine_map_with_source(src: &str, emitted: &str, map: &[SourceMapEntry]) -> Vec<SourceMapEntry> {
    if map.is_empty() {
        return Vec::new();
    }
    let original: Vec<&str> = src.lines().map(|l| l.trim()).collect();
    let emitted_lines: Vec<&str> = emitted.lines().collect();
    let mut refined = map.to_vec();
    // 连续同 source_line 的一段产物行 = 一条顶层语句的产物。
    let mut start = 0usize;
    while start < refined.len() {
        let stmt_line = refined[start].source_line;
        let mut end = start + 1;
        while end < refined.len() && refined[end].source_line == stmt_line {
            end += 1;
        }
        // 原源区间：本语句起点行 .. 下一语句起点行（不含），1-based。
        let range_start = stmt_line.saturating_sub(1).min(original.len());
        let range_end = refined
            .get(end)
            .map(|e| e.source_line.saturating_sub(1))
            .unwrap_or(original.len())
            .min(original.len())
            .max(range_start);
        let mut cursor = range_start;
        for entry in refined.iter_mut().take(end).skip(start) {
            let output_line = entry.output_line;
            let trimmed = emitted_lines
                .get(output_line.saturating_sub(1))
                .map(|l| l.trim())
                .unwrap_or("");
            if trimmed.is_empty() || is_structural_line(trimmed) {
                continue;
            }
            let window = &original[cursor..range_end];
            if let Some(offset) = window.iter().position(|line| *line == trimmed) {
                entry.source_line = cursor + offset + 1;
                cursor = cursor + offset + 1;
            } else if trimmed.len() >= 6 {
                if let Some(offset) = window.iter().position(|line| line.contains(trimmed)) {
                    entry.source_line = cursor + offset + 1;
                    cursor += offset;
                }
            }
        }
        start = end;
    }
    refined
}

/// 产物里的结构行：块头（以 `{` 收尾）、闭括号行、`else`/`catch`/`finally`
/// 片段。这些行不与原源行做同形匹配（一对多/多对一，匹配会误导向）。
fn is_structural_line(trimmed: &str) -> bool {
    trimmed.ends_with('{')
        || trimmed.starts_with('}')
        || trimmed == "} else"
        || trimmed.starts_with("else")
        || trimmed.starts_with("catch")
        || trimmed.starts_with("finally")
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
        // A1 方法调用（py-known 接收者）→ py_call；Plan 567 T09 后桥调用
        // 隐式传播为 may 变体 + `.?`。
        assert!(out.contains("py_call_may(t, \"sum\").?"), "{}", out);
        // B1 属性（arange 结果 py-known）→ py_getattr_may + `.?`
        assert!(out.contains("py_getattr_may(t, \"shape\").?"), "{}", out);
        // B3 索引 → py_getitem_may + `.?`
        assert!(out.contains("py_getitem_may(t, 0).?"), "{}", out);
        // B2 属性赋值 → py_setattr（无 may 变体，不传播）
        assert!(out.contains("py_setattr(t, \"note\", \"hi\")"), "{}", out);
        // B4 索引赋值 → py_setitem（同上）
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
        assert!(
            out.contains("py_getitem_may(x, py_slice(2, 5)).?"),
            "{}",
            out
        );
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

    /// Plan 567 T09（P560-D2）: 隐式 !T 传播——桥调用 may 化 + `.?`、
    /// 用户函数调用点 `.?`、kwargs 形态保持 Pair 由 codegen 路由 478。
    #[test]
    fn test_s2s_rule_err_propagate() {
        let src = r#"use.py torch: arange
fn double(n int) -> int {
    return n * 2
}
fn main() {
    var t = arange(6)
    var s = t.sum()
    var w = t.shape
    var v = t[0]
    var k = t.sum(dim: 0)
    var d = double(3)
    t.note = "hi"
}
"#;
        let out = lower_source(src).unwrap();
        // 桥调用 → may + `.?`（kwargs 形态保持 Pair 实参）
        assert!(out.contains("py_call_may(t, \"sum\").?"), "{}", out);
        assert!(out.contains("py_getattr_may(t, \"shape\").?"), "{}", out);
        assert!(out.contains("py_getitem_may(t, 0).?"), "{}", out);
        assert!(out.contains("dim: 0"), "kwargs 形态保留: {}", out);
        // 用户函数调用点 → `.?`
        assert!(out.contains("double(3).?"), "{}", out);
        // setter 无 may 变体不传播
        assert!(out.contains("py_setattr(t, \"note\", \"hi\")"), "{}", out);
        // 产物可再解析 + 幂等（corpus 测试全局钉）
        let mut p = crate::parser::Parser::new(&out);
        assert!(p.parse().is_ok(), "{}", out);
    }

    /// Plan 567 T09 反例：无 use.py 的源零传播改写（纯 Auto `.as` 不受扰
    /// ——legacy -1 哨兵语义保护）。
    #[test]
    fn test_s2s_err_propagate_no_py_noop() {
        let src = r#"fn double(n int) -> int {
    return n * 2
}
fn main() {
    var d = double(3)
    var arr = [1, 2]
    var e0 = arr[0]
}
"#;
        let out = lower_source(src).unwrap();
        assert!(!out.contains(".?"), "无 py 导入零传播改写: {}", out);
        assert!(out.contains("double(3)"), "{}", out);
    }

    /// 真实语料 round-trip：py parity 套件源（T12 迁移目标面）逐文件
    /// parse→emit→re-parse 幂等。发射器覆盖面的实弹靶场。
    #[test]
    fn test_s2s_py_suite_corpus_roundtrip() {
        let corpus = [
            "test/parity/libs/python/py_math/tests/auto/math.as",
            "test/parity/libs/python/py_torch_infer/tests/auto/infer.as",
            "test/parity/libs/python/py_torch_train/tests/auto/train.as",
            "test/parity/libs/python/py_numpy/tests/auto/numpy.as",
            "test/parity/libs/python/py_torch/tests/auto/torch.as",
        ];
        let manifest = env!("CARGO_MANIFEST_DIR");
        for rel in corpus {
            let path = std::path::Path::new(manifest).join("../../").join(rel);
            let Ok(src) = std::fs::read_to_string(&path) else {
                panic!("corpus missing: {}", path.display());
            };
            let once = lower_source(&src).unwrap_or_else(|e| panic!("{}: lower 失败: {}", rel, e));
            let mut p = crate::parser::Parser::new(&once);
            assert!(p.parse().is_ok(), "{}: 产物不可再解析:\n{}", rel, once);
            let twice =
                lower_source(&once).unwrap_or_else(|e| panic!("{}: 二次 lower 失败: {}", rel, e));
            assert_eq!(twice, once, "{}: 不幂等", rel);
        }
    }

    // ── PLAN-977 B-REVERSE-I: ACE 规范形 + 行级 source map ──

    /// ACE 执行器只能解析规范返回类型（`fn f(...) T`，无 `->`）；s2s 的
    /// ACE 产物面就是规范方言，历史方言逐字节不变。行映射：每条产物行
    /// 都有条目，未改写行精确回到原行，结构/规范化行回落到语句起点行。
    #[test]
    fn p977_s2s_ace_dialect_is_canonical_and_mapped() {
        let src = "fn fib(n int) -> int {\n    if n < 2 { return n }\n    return fib(n - 1) + fib(n - 2)\n}\nfn main() -> int {\n    return fib(10)\n}\n";

        // 历史方言（公开命令面）逐字节不变。
        let legacy = lower_source(src).unwrap();
        let (mapped_legacy, legacy_map) = lower_source_with_map(src).unwrap();
        assert_eq!(
            legacy, mapped_legacy,
            "带 map 入口在历史方言下必须逐字节同产物"
        );
        assert!(legacy.contains("fn fib(n int) -> int {"), "{}", legacy);
        assert_eq!(legacy_map.len(), legacy.lines().count());

        // ACE 方言：签名无箭头，产物仍可被原 parser 再解析（同一语言）。
        let (canonical, map) = lower_source_for_ace(src).unwrap();
        assert!(
            !canonical.contains("->"),
            "ACE 规范形不得含签名箭头:\n{}",
            canonical
        );
        assert!(
            canonical.contains("fn fib(n int) int {"),
            "规范形返回类型是裸 type word:\n{}",
            canonical
        );
        assert!(crate::parser::Parser::new(&canonical).parse().is_ok());
        assert_eq!(map.len(), canonical.lines().count(), "每行都要有条目");

        let at = |line: usize| {
            crate::trans::emit::source_line_for_output(&map, line)
                .unwrap_or_else(|| panic!("产物行 {} 无映射", line))
        };
        // 产物行：1 fn fib / 2 if / 3 return n / 4 } / 5 return fib(..) /
        // 6 } / 7 fn main / 8 return fib(10) / 9 }
        assert_eq!(at(1), 1, "fn fib 声明行");
        assert_eq!(at(3), 2, "单行 if 内的 return n 细化到 if 原行");
        assert_eq!(at(5), 3, "未改写的 return 行精确回到原行");
        assert_eq!(at(7), 5, "fn main 声明行");
        assert_eq!(at(8), 6, "return fib(10) 精确回到原行");
        // 规范化行（声明）回落到语句起点行——文档化粒度，不是错误映射。
        assert_eq!(at(2), 1);
        assert_eq!(at(6), 1);
    }

    /// 规则改写过的那一行没有原行同形文本：map 回落到语句起点行（不猜），
    /// 未改写行仍精确。
    #[test]
    fn p977_s2s_map_falls_back_to_statement_start_on_rewrite() {
        let src =
            "use.py torch: arange\nfn main() {\n    var t = arange(6)\n    let s = t.sum()\n}\n";
        let (out, map) = lower_source_with_map(src).unwrap();
        assert!(out.contains("py_call_may(t, \"sum\").?"), "{}", out);
        assert_eq!(map.len(), out.lines().count());
        let at = |line: usize| crate::trans::emit::source_line_for_output(&map, line).unwrap();
        assert_eq!(at(1), 1, "use 行");
        assert_eq!(at(3), 3, "未改写的 var 行精确");
        assert_eq!(at(4), 2, "改写行回落到 fn main 起点行");
    }

    /// 规范形只动签名位：字符串/字符字面量里的 `) -> ` 逐字保留。
    #[test]
    fn p977_s2s_canonical_keeps_arrow_inside_literals() {
        let src = "fn main() -> str {\n    return \") -> \"\n}\n";
        let (canonical, _) = lower_source_for_ace(src).unwrap();
        assert!(
            canonical.contains("fn main() str {"),
            "签名归一:\n{}",
            canonical
        );
        assert!(
            canonical.contains("\") -> \""),
            "字面量内的箭头必须原样:\n{}",
            canonical
        );
    }
}
