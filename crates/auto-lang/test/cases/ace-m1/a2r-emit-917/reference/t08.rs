// Hand-written reference for corpus T08 (Plan 917).
// Independent Rust encoding of the frozen slice-oob semantics: slicing the
// one-char string "中" at char range [0, 2) is out of range and must fail
// loudly with the frozen marker (this is a behavior observation, not an
// Auto trap certification).
fn main() -> i64 {
    let s = "中";
    let (a, b) = (0usize, 2usize);
    let n = s.chars().count();
    assert!(a <= b && b <= n, "auto str slice out of range");
    s.chars().skip(a).take(b - a).collect::<String>().len() as i64
}
