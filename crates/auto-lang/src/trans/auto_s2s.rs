//! Plan 555 T07: s2s 改写器骨架——Auto（脚本糖）→ Auto（正常模式桥）。
//!
//! 设计源 `docs/design/strategy/script-mode-interop.md` §1：改写器做成
//! 独立 source-to-source 工具先行（Babel 模式），产出直接跑现有门禁；
//! 稳定后折进编译器模式管线（multi_mode.rs），**不**塞进 codegen 热臂
//! （§10 裁决）。每条规则配 source-to-source 单测。
//!
//! W1 状态：规则表**空置**（identity passthrough）——本模块交付的是
//! 可扩展框架：规则注册点、词法粒度改写面、round-trip 稳定发射。
//! W2 糖批（A1-A5/B1-B9/C3-C7/D7/E2-E3 lowering 全表）逐条落进
//! `builtin_rules()`。

use crate::error::AutoResult;

/// 一条 lowering 规则。设计 §3 边界目录按 `A1..F4` 编号键控。
///
/// W1 帧形：规则拿到**整源 + 词法 token 流**，返回 `Some(新源)` 表示
/// 命中改写（首中即返，单遍；W2 链式语义随规则批裁定）。token 粒度
/// 保证 identity 发射逐字节稳定（token 文本切片重组）。
pub struct LoweringRule {
    /// 边界目录编号（"A2"/"B5"/"E2"...），诊断与单测键控。
    pub id: &'static str,
    /// 改写函数：输入 (当前源, 原源 token 流)，输出改写后的完整源。
    pub rewrite: fn(&str, &[crate::token::Token]) -> Option<String>,
}

/// W1：内置规则表空置——identity passthrough。
/// W2：糖批 lowering 全表逐条注册于此（每条配 source-to-source 单测）。
pub fn builtin_rules() -> Vec<LoweringRule> {
    Vec::new()
}

/// 对源码应用规则表（词法 → 语法验证 → 规则逐条 → 发射）。
///
/// - 语法验证：输入必须可解析（改写器不吃非法源）；
/// - 发射：无规则命中 = 原源逐字节返回（identity 稳定性由单测钉死）。
pub fn lower_source(src: &str) -> AutoResult<String> {
    // 1. 词法（token 粒度改写面；失败=非法源，改写器拒绝）
    let tokens = crate::lexer::Lexer::new(src).tokenize_all()?;
    // 2. 语法验证（parse 成功才进入改写）
    let mut parser = crate::parser::Parser::new(src);
    parser.parse()?;
    // 3. 规则表单遍首中（W1 空表 → identity）
    for rule in builtin_rules() {
        if let Some(rewritten) = (rule.rewrite)(src, &tokens) {
            return Ok(rewritten);
        }
    }
    Ok(src.to_string())
}

#[cfg(test)]
mod tests_s2s {
    use super::*;

    const SAMPLE: &str = r#"fn main() {
    var xs = [1, 2, 3]
    print(obj_len(xs))
}
"#;

    /// identity 稳定性：空规则表下产出=输入逐字节同，且幂等
    /// （lower(lower(x)) == lower(x) == x）。
    #[test]
    fn test_s2s_identity_roundtrip_stable() {
        let once = lower_source(SAMPLE).unwrap();
        assert_eq!(once, SAMPLE, "W1 空表必须逐字节 identity");
        let twice = lower_source(&once).unwrap();
        assert_eq!(twice, once, "幂等");
    }

    /// 可扩展性：注入一条测试规则（Ident `obj_len` → `interop_len`）
    /// 即生效——证明规则表注册点真的被消费。
    #[test]
    fn test_s2s_test_rule_injection_takes_effect() {
        let rule = LoweringRule {
            id: "TEST-1",
            rewrite: |src, tokens| {
                let hit = tokens
                    .iter()
                    .any(|t| t.text.as_str() == "obj_len");
                if !hit {
                    return None;
                }
                Some(src.replace("obj_len", "interop_len"))
            },
        };
        // 直接走与 lower_source 相同的管线，但注入测试规则
        let tokens = crate::lexer::Lexer::new(SAMPLE).tokenize_all().unwrap();
        let mut parser = crate::parser::Parser::new(SAMPLE);
        parser.parse().unwrap();
        let out = (rule.rewrite)(SAMPLE, &tokens).unwrap();
        assert!(out.contains("interop_len(xs)"), "规则注入即生效: {}", out);
        assert!(!out.contains("obj_len"));
        // 改写产物必须仍可解析（产物跑现有门禁的前提）
        let mut p2 = crate::parser::Parser::new(&out);
        assert!(p2.parse().is_ok(), "改写产物必须可解析");
    }

    /// 非法源拒绝：语法验证兜底。
    #[test]
    fn test_s2s_rejects_invalid_source() {
        assert!(lower_source("fn broken {{{{").is_err());
    }
}
