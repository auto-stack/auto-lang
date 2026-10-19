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

pub fn abs_f(n: f64) -> f64 {
    n.abs()
}

pub fn powf(x: f64, y: f64) -> f64 {
    x.powf(y)
}

pub fn sin(x: f64) -> f64 {
    x.sin()
}

pub fn cos(x: f64) -> f64 {
    x.cos()
}

#[cfg(test)]
mod tests {
    use super::{abs_f, cos, powf, sin};

    #[test]
    fn floating_point_helpers_match_rust_math() {
        assert_eq!(abs_f(-1.5), 1.5);
        assert_eq!(powf(2.0, 3.0), 8.0);
        assert_eq!(sin(0.0), 0.0);
        assert_eq!(cos(0.0), 1.0);
    }
}

pub fn square(x: i32) -> i32 {
    x * x
}

pub fn cube(x: i32) -> i32 {
    x * x * x
}
