//! PLAN-843 结构拆分：math 域实现（原样搬移，算法/行为零变化）。
//!
//! PLAN-872 实测钉死：本文件 28 个纯 shim 经 `#[rust_fn]` inventory 在
//! `production()` 步骤 4 绑定（canonical `auto.math.*` 命中 catalog 同 ID
//! 1700-1733），**是 `Math.*` 的生效实现**——native.rs 的
//! `math_unary_shim!`/`math_binary_shim!` nv 宏版（Plan 437 tag 感知弹参）
//! 被同 ID 后写覆盖，稳态不在派发路径上（RETIRE 候选，单独签核）。
//! 生效语义（VMConvertible 提取）：f64 参数**不吞 i32 tag**——
//! `Math.sin(1)`（整型字面量）= NaN、`Math.sin(1.0)` 正常；域外值走 IEEE
//! （`sqrt(-1)=NaN`、`ln(0)=-inf`）。跨后端：a2r = a2r-std/src/math.rs
//! 静态类型直调；JS/Vue = Math.*（整型实参强制转换，与 VM 的 NaN-on-int
//! 不同——见 attachments/872/binding-table.md 族二）。
use super::*;

#[auto_macros::rust_fn("Math.abs", "auto.math.abs")]
pub fn shim_math_abs(n: i32) -> i32 {
    n.abs()
}

#[auto_macros::rust_fn("Math.min", "auto.math.min")]
pub fn shim_math_min(a: i32, b: i32) -> i32 {
    a.min(b)
}

#[auto_macros::rust_fn("Math.max", "auto.math.max")]
pub fn shim_math_max(a: i32, b: i32) -> i32 {
    a.max(b)
}

/// Square root of a number
#[auto_macros::rust_fn("Math.sqrt")]
pub fn shim_math_sqrt(n: f64) -> f64 {
    n.sqrt()
}

/// Floor of a float
#[auto_macros::rust_fn("Math.floor")]
pub fn shim_math_floor(n: f64) -> f64 {
    n.floor()
}

/// Ceiling of a float
#[auto_macros::rust_fn("Math.ceil")]
pub fn shim_math_ceil(n: f64) -> f64 {
    n.ceil()
}

/// Round a float to nearest integer
#[auto_macros::rust_fn("Math.round")]
pub fn shim_math_round(n: f64) -> f64 {
    n.round()
}

/// Power function: base^exp
#[auto_macros::rust_fn("Math.pow")]
pub fn shim_math_pow(base: f64, exp: f64) -> f64 {
    base.powf(exp)
}

/// Minimum of two floats
#[auto_macros::rust_fn("Math.min_f")]
pub fn shim_math_min_f(a: f64, b: f64) -> f64 {
    a.min(b)
}

/// Maximum of two floats
#[auto_macros::rust_fn("Math.max_f")]
pub fn shim_math_max_f(a: f64, b: f64) -> f64 {
    a.max(b)
}

/// Sine function (radians)
#[auto_macros::rust_fn("Math.sin")]
pub fn shim_math_sin(n: f64) -> f64 {
    n.sin()
}

/// Cosine function (radians)
#[auto_macros::rust_fn("Math.cos")]
pub fn shim_math_cos(n: f64) -> f64 {
    n.cos()
}

/// Tangent function (radians)
#[auto_macros::rust_fn("Math.tan")]
pub fn shim_math_tan(n: f64) -> f64 {
    n.tan()
}

/// Exponential function e^x
#[auto_macros::rust_fn("Math.exp")]
pub fn shim_math_exp(n: f64) -> f64 {
    n.exp()
}

/// Natural logarithm ln(x)
#[auto_macros::rust_fn("Math.ln")]
pub fn shim_math_ln(n: f64) -> f64 {
    n.ln()
}

/// Base-2 logarithm log2(x)
#[auto_macros::rust_fn("Math.log2")]
pub fn shim_math_log2(n: f64) -> f64 {
    n.log2()
}

/// Base-10 logarithm log10(x)
#[auto_macros::rust_fn("Math.log10")]
pub fn shim_math_log10(n: f64) -> f64 {
    n.log10()
}

/// Absolute value of a float
#[auto_macros::rust_fn("Math.abs_f")]
pub fn shim_math_abs_f(n: f64) -> f64 {
    n.abs()
}

/// Signum of a float (-1.0, 0.0, or 1.0)
#[auto_macros::rust_fn("Math.signum")]
pub fn shim_math_signum(n: f64) -> f64 {
    n.signum()
}

/// Clamp a value between min and max
#[auto_macros::rust_fn("Math.clamp")]
pub fn shim_math_clamp(n: f64, min: f64, max: f64) -> f64 {
    n.clamp(min, max)
}

/// Arc sine
#[auto_macros::rust_fn("Math.asin")]
pub fn shim_math_asin(n: f64) -> f64 {
    n.asin()
}

/// Arc cosine
#[auto_macros::rust_fn("Math.acos")]
pub fn shim_math_acos(n: f64) -> f64 {
    n.acos()
}

/// Arc tangent
#[auto_macros::rust_fn("Math.atan")]
pub fn shim_math_atan(n: f64) -> f64 {
    n.atan()
}

/// Arc tangent of y/x
#[auto_macros::rust_fn("Math.atan2")]
pub fn shim_math_atan2(y: f64, x: f64) -> f64 {
    y.atan2(x)
}

/// Integer power
#[auto_macros::rust_fn("Math.powi")]
pub fn shim_math_powi(n: f64, exp: i32) -> f64 {
    n.powi(exp)
}

/// Float power
#[auto_macros::rust_fn("Math.powf")]
pub fn shim_math_powf(n: f64, exp: f64) -> f64 {
    n.powf(exp)
}

/// Convert degrees to radians
#[auto_macros::rust_fn("Math.to_radians")]
pub fn shim_math_to_radians(n: f64) -> f64 {
    n.to_radians()
}

/// Convert radians to degrees
#[auto_macros::rust_fn("Math.to_degrees")]
pub fn shim_math_to_degrees(n: f64) -> f64 {
    n.to_degrees()
}
