/// Math module - Mathematical functions
/// Transpiled from auto-lang/stdlib/auto/math.at + math.rs.at

/// Absolute value (i32 version for Auto int type)
pub fn abs(n: i32) -> i32 {
    n.abs()
}

pub fn abs_i64(n: i64) -> i64 {
    n.abs()
}

pub fn min(a: i32, b: i32) -> i32 {
    std::cmp::min(a, b)
}

pub fn min_i64(a: i64, b: i64) -> i64 {
    std::cmp::min(a, b)
}

pub fn max(a: i32, b: i32) -> i32 {
    std::cmp::max(a, b)
}

pub fn max_i64(a: i64, b: i64) -> i64 {
    std::cmp::max(a, b)
}

pub fn sqrt(d: f64) -> f64 {
    d.sqrt()
}

pub fn square(x: i32) -> i32 {
    x * x
}

pub fn cube(x: i32) -> i32 {
    x * x * x
}
