// PLAN-931 hand-written independent reference — modes-931: ownership
// moves in (callee consumes), exclusive borrow reads; 2*10 + 2 = 22.
fn own_len(s: String) -> i32 {
    s.chars().count() as i32
}
fn mut_peek(s: &str) -> i32 {
    s.chars().count() as i32
}
fn main() {
    let a = "ab".to_string();
    let mut b = "xy".to_string();
    let n1 = own_len(a);
    let n2 = mut_peek(&b);
    b.clear(); // the exclusive borrow ended; the owner may mutate again
    let _ = b;
    println!("value:{}", n1 * 10 + n2);
}
