//! PLAN-781 T-15：HTTP handler 有限执行预算 policy（合同 =
//! docs/plans/reports/781-budget-decision.md §1 D-1/D-4）。
//!
//! - 配置源 = 环境变量 `AUTO_HTTP_STEP_BUDGET`（**宿主可信配置**：进程
//!   启动期读取，.at 业务代码不可写；每进程一份 → 实例天然隔离）。
//! - 语义 = 单 HTTP 请求 dispatch 生命周期的**累计**指令上限（跨段
//!   park/resume、跨嵌套 fn 不重置——不变量 2）；未设置 = 默认
//!   10,000,000（既有 Legacy 语义逐字节不变）。
//! - 合法域：十进制 u64，`1 ..= 4_000_000_000`。非法值（0/负号/小数点/
//!   非数字/溢出/超上界）= 指名配置错误——宿主启动链 fail-fast 非零
//!   退出（先于任何业务 dispatch）。
//!
//! 消费面：engine `DriveBudget::LegacyHttp`（http 入口族 +
//! resume_fn_by_name_segment_http）；见决策报告 §1 D-2 入口矩阵。

/// 上界依据（决策报告 D-1）：10MiB 导入最坏 420M 步的 ≥9× 余量 +
/// u32 便携 + 防 u64 溢出类笔误。
pub const HTTP_STEP_BUDGET_MAX: u64 = 4_000_000_000;

/// 配置项环境变量名（错误信息指名用）。
pub const HTTP_STEP_BUDGET_ENV: &str = "AUTO_HTTP_STEP_BUDGET";

/// 解析失败原因（指名错误——宿主以非零退出拒绝启动）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BudgetConfigError {
    pub env: &'static str,
    pub raw: String,
    pub reason: &'static str,
}

impl std::fmt::Display for BudgetConfigError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "invalid {}={:?}: {} (legal: decimal u64 in 1..={})",
            self.env, self.raw, self.reason, HTTP_STEP_BUDGET_MAX
        )
    }
}

impl std::error::Error for BudgetConfigError {}

/// 解析 env 原始值。`None`（未设置/空）= `Ok(None)`——默认 10M 语义，
/// 不进 policy 面。`Some(raw)` 严格十进制 u64 + 域检查。
pub fn parse_http_step_budget(raw: Option<&str>) -> Result<Option<u64>, BudgetConfigError> {
    let Some(raw) = raw else { return Ok(None) };
    if raw.trim().is_empty() {
        return Ok(None);
    }
    let err = |reason: &'static str| BudgetConfigError {
        env: HTTP_STEP_BUDGET_ENV,
        raw: raw.to_string(),
        reason,
    };
    // 严格十进制：全 ASCII 数字（拒 `+`/`_`/空白/小数点/负号/十六进制）。
    if raw.is_empty() || !raw.bytes().all(|b| b.is_ascii_digit()) {
        return Err(err("not a decimal integer (digits only)"));
    }
    let value: u64 = raw.parse().map_err(|_| err("value overflows u64"))?;
    if value == 0 {
        return Err(err("zero is not a positive budget"));
    }
    if value > HTTP_STEP_BUDGET_MAX {
        return Err(err("exceeds the frozen upper bound"));
    }
    Ok(Some(value))
}

/// 从进程环境读取并解析（AutoVM 构造期消费；宿主启动链已先行严格
/// 验证——此处宽松兜底：构造期遇非法值直接落 None 并 stderr 告警，
/// 不 panic 嵌入方）。
pub fn http_step_budget_from_env() -> Option<u64> {
    match parse_http_step_budget(std::env::var(HTTP_STEP_BUDGET_ENV).ok().as_deref()) {
        Ok(v) => v,
        Err(e) => {
            eprintln!(
                "WARN[budget] {} — falling back to default 10M (host startup validation missed)",
                e
            );
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// V-15（决策报告 §2）：非法配置表——各形态指名错误、合法域边界。
    #[test]
    fn plan781_v15_budget_config_table() {
        // 未设置 / 空白 = 默认语义（None）。
        assert_eq!(parse_http_step_budget(None), Ok(None));
        assert_eq!(parse_http_step_budget(Some("")), Ok(None));
        assert_eq!(parse_http_step_budget(Some("   ")), Ok(None));
        // 合法域边界。
        assert_eq!(parse_http_step_budget(Some("1")), Ok(Some(1)));
        assert_eq!(
            parse_http_step_budget(Some("4000000000")),
            Ok(Some(HTTP_STEP_BUDGET_MAX))
        );
        // 非法表：0 / 负 / 小数 / 非数 / 溢出 u64 / 超上界 / 符号与空白。
        for raw in [
            "0",
            "-5",
            "1.5",
            "abc",
            "99999999999999999999",
            "5000000001",
        ] {
            let e = parse_http_step_budget(Some(raw)).unwrap_err();
            assert_eq!(e.env, "AUTO_HTTP_STEP_BUDGET");
            assert_eq!(e.raw, raw);
            assert!(
                e.to_string().contains("AUTO_HTTP_STEP_BUDGET"),
                "error must name the env: {e}"
            );
        }
        for raw in ["+5", " 5", "5 ", "0x10", "1_000", "١٢"] {
            assert!(
                parse_http_step_budget(Some(raw)).is_err(),
                "strict decimal only: {raw:?}"
            );
        }
        // 推荐导入档在域内。
        assert_eq!(
            parse_http_step_budget(Some("1000000000")),
            Ok(Some(1_000_000_000))
        );
    }
}
