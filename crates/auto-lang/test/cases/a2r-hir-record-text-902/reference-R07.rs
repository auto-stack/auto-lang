// PLAN-902 hand-written independent reference — R07 (i32 form): the mut
// param's field write is visible to the caller.
fn main() {
    let mut x: i32 = 20;
    x = x + 1;
    let now = x;
    println!("value:{}", now + x);
}
