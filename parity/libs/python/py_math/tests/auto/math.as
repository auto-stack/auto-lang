// py_math Auto test — TAP output format (Plan 369 Python parity).
// The parity runner sets the working directory to the library root
// (parity/libs/py_math), so `use.py math` resolves the Python stdlib module.
//
// Test names MUST match tests/python/test_math.py because the comparator joins
// backends by name. The function set is chosen so every case is deterministic
// and yields a plain integer, keeping the P0 skeleton a clean green baseline
// (see README for the known AutoVM divergences on gcd/fabs that motivated
// excluding those two).
//
// Plan 369 P1: expanded with log2/log10/isfinite/isinf/isnan/sin/cos. Float
// returns are coerced via .to(int); boolean predicates are compared against
// 0/1 (PyFFI marshals Python bool to int).
//
// Plan 369 P4 (Task 15): added gcd/lcm (multi-arg, fixed in P3) and the pi/e
// constants. Constants marshal to their string representation, so pi/e are
// asserted by exact string equality (deterministic for IEEE-754 doubles).
use.py math: ceil, floor, factorial, trunc, isqrt, sqrt, pow, log2, log10, isfinite, isinf, isnan, sin, cos, gcd, lcm, pi, e

fn tap_ok(n int, name str) {
    print("ok " + n.to(str) + " - " + name)
}

fn tap_not_ok(n int, name str, diag str) {
    print("not ok " + n.to(str) + " - " + name + " # " + diag)
}

fn main() {
    if ceil(3.2) == 4 { tap_ok(1, "test_ceil") } else { tap_not_ok(1, "test_ceil", "got " + ceil(3.2).to(str)) }
    if floor(3.8) == 3 { tap_ok(2, "test_floor") } else { tap_not_ok(2, "test_floor", "got " + floor(3.8).to(str)) }
    if factorial(5) == 120 { tap_ok(3, "test_factorial") } else { tap_not_ok(3, "test_factorial", "got " + factorial(5).to(str)) }
    if trunc(3.7) == 3 { tap_ok(4, "test_trunc") } else { tap_not_ok(4, "test_trunc", "got " + trunc(3.7).to(str)) }
    if isqrt(17) == 4 { tap_ok(5, "test_isqrt") } else { tap_not_ok(5, "test_isqrt", "got " + isqrt(17).to(str)) }
    if sqrt(16).to(int) == 4 { tap_ok(6, "test_sqrt_int") } else { tap_not_ok(6, "test_sqrt_int", "got " + sqrt(16).to(str)) }
    if pow(2, 10).to(int) == 1024 { tap_ok(7, "test_pow_int") } else { tap_not_ok(7, "test_pow_int", "got " + pow(2, 10).to(str)) }
    // Plan 369 P1: expanded coverage (see header comment).
    if log2(8).to(int) == 3 { tap_ok(8, "test_log2") } else { tap_not_ok(8, "test_log2", "got " + log2(8).to(str)) }
    if log10(1000).to(int) == 3 { tap_ok(9, "test_log10") } else { tap_not_ok(9, "test_log10", "got " + log10(1000).to(str)) }
    if isfinite(5) == 1 { tap_ok(10, "test_isfinite") } else { tap_not_ok(10, "test_isfinite", "got " + isfinite(5).to(str)) }
    if isinf(5) == 0 { tap_ok(11, "test_isinf") } else { tap_not_ok(11, "test_isinf", "got " + isinf(5).to(str)) }
    if isnan(5) == 0 { tap_ok(12, "test_isnan") } else { tap_not_ok(12, "test_isnan", "got " + isnan(5).to(str)) }
    if sin(0).to(int) == 0 { tap_ok(13, "test_sin_zero") } else { tap_not_ok(13, "test_sin_zero", "got " + sin(0).to(str)) }
    if cos(0).to(int) == 1 { tap_ok(14, "test_cos_zero") } else { tap_not_ok(14, "test_cos_zero", "got " + cos(0).to(str)) }
    // Plan 369 P4 (Task 15): gcd/lcm multi-arg (fixed in P3) and pi/e constants.
    if gcd(12, 8) == 4 { tap_ok(15, "test_gcd_two_arg") } else { tap_not_ok(15, "test_gcd_two_arg", "got " + gcd(12, 8).to(str)) }
    if gcd(48, 36) == 12 { tap_ok(16, "test_gcd_larger") } else { tap_not_ok(16, "test_gcd_larger", "got " + gcd(48, 36).to(str)) }
    if lcm(4, 6) == 12 { tap_ok(17, "test_lcm_two_arg") } else { tap_not_ok(17, "test_lcm_two_arg", "got " + lcm(4, 6).to(str)) }
    if lcm(3, 4) == 12 { tap_ok(18, "test_lcm_coprime") } else { tap_not_ok(18, "test_lcm_coprime", "got " + lcm(3, 4).to(str)) }
    // Constants marshal to their string form; assert by exact string equality.
    var ps = pi.to(str)
    if ps == "3.141592653589793" { tap_ok(19, "test_pi_constant") } else { tap_not_ok(19, "test_pi_constant", "got " + ps) }
    var es = e.to(str)
    if es == "2.718281828459045" { tap_ok(20, "test_e_constant") } else { tap_not_ok(20, "test_e_constant", "got " + es) }
}
