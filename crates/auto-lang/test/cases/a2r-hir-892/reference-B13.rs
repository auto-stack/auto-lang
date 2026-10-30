// hand-written independent reference: MAX+1 overflows i32
fn main() {
    match 2147483647i32.checked_add(1) {
        Some(v) => println!("value:{}", v),
        None => println!("trap:Overflow"),
    }
}
