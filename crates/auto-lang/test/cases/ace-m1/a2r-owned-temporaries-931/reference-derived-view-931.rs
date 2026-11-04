// PLAN-931 hand-written independent reference — derived-view-931: a
// clone temp consumed by len (2), then a clone consumed by an equality
// that holds (true); result 2*10 + 2 = 22.
fn main() {
    let a: &str = "ab";
    let n = a.to_string().chars().count() as i32;
    let ok = a.to_string().as_str() == a;
    if ok {
        println!("value:{}", n * 10 + n);
    } else {
        println!("value:0");
    }
}
