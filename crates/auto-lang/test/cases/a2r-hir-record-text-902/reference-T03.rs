// PLAN-902 hand-written independent reference — T03 (std String form,
// independent of the generated a2x runtime): clone, move, concat with
// replacement, Unicode-scalar lengths ("ab"=2, "ab中"=3).
fn main() {
    let a: &str = "ab";
    let b = a.to_string();
    let mut c = b; // move: b dies here
    c.push('中');
    let alen = a.chars().count() as i32;
    let clen = c.chars().count() as i32;
    println!("value:{}", alen * 10 + clen);
}
