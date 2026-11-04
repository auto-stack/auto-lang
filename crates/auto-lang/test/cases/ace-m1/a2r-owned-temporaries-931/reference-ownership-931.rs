// PLAN-931 hand-written independent reference — ownership-931: the moved
// value dies inside the callee (Rust drop = the a2x planned free), the
// returned value transfers to the caller; 4*10 + 3 = 43.
fn eat(s: String) -> i32 {
    s.chars().count() as i32
}
fn make() -> String {
    let s = "abc".to_string();
    s // moves out; make's frame owns nothing after
}
fn main() {
    let a = "abcd".to_string();
    let n = eat(a);
    let b = make();
    println!("value:{}", n * 10 + b.chars().count() as i32);
}
