// Hand-written reference for nb_slice_ok (Plan 917): in-range char slice.
fn main() -> i64 {
    let s = "中ab";
    let (a, b) = (1usize, 3usize);
    let n = s.chars().count();
    assert!(a <= b && b <= n, "auto str slice out of range");
    s.chars().skip(a).take(b - a).collect::<String>().len() as i64
}
