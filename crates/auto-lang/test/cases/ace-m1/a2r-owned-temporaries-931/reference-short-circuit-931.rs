// PLAN-931 hand-written independent reference — short-circuit-931: the
// taken side's comparison holds, the untaken side never evaluates;
// result 5.
fn main() {
    let a = "ab".to_string();
    let b = "cde".to_string();
    let ok = a.clone().as_str() == a && b.clone().as_str() == b;
    let skip = false && b.clone().as_str() == b;
    if ok {
        if skip {
            println!("value:0");
        } else {
            println!("value:5");
        }
    } else {
        println!("value:0");
    }
}
