//! 恢复策略：同步点选择与前进性约束（PLAN-787 §5.3）。
//!
//! 恢复执行在 `parse` 执行器内（规则失败 → 吃进 error 叶到同步点 →
//! missing + 继续）；本模块承载共享判定与不变式测试锚点：
//! - 同步点 = `recovery.sync` ∪ first(rule)（规则可重新开始的位置）；
//! - 每次恢复必须：消耗 ≥1 token，或产出有限 missing 动作后停止，
//!   或到达 EOF——不允许静默零进展循环。

use crate::program::{GrammarProgram, RuleId, TokenId};

/// 恢复是否应在此 token 前停（同步命中）。
pub fn is_sync_point(
    p: &GrammarProgram,
    first: &crate::parse::FirstSets,
    rule: RuleId,
    tok: TokenId,
) -> bool {
    if tok == u32::MAX {
        return false; // 错误叶 token 不作为同步点
    }
    p.recovery.sync.contains(&tok) || first.starts(rule, tok)
}

/// 前进性审计：一次恢复动作的 (消耗 token 数, 产出 missing 数) 必须非全零。
pub fn recovery_makes_progress(consumed: usize, missing_emitted: usize, at_eof: bool) -> bool {
    consumed > 0 || missing_emitted > 0 || at_eof
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn recovery_progress_rules() {
        assert!(recovery_makes_progress(1, 0, false));
        assert!(
            recovery_makes_progress(0, 1, false),
            "finite missing action is progress"
        );
        assert!(
            recovery_makes_progress(0, 0, true),
            "termination at EOF is progress"
        );
        assert!(
            !recovery_makes_progress(0, 0, false),
            "zero-progress loop is forbidden"
        );
    }
}
