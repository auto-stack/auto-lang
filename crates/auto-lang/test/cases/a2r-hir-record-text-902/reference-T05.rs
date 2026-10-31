// PLAN-902 hand-written independent reference — T05 (std str form,
// independent of the generated a2x runtime): full-byte equality over an
// embedded NUL, no normalization.
fn main() {
    let a = "A\0B";
    if a == "A" {
        println!("value:1");
        return;
    }
    if a != "A\0B" {
        println!("value:2");
        return;
    }
    println!("value:42");
}
