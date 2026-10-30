// hand-written independent reference: division by zero
fn main() {
    match 10i32.checked_div(0) {
        Some(v) => println!("value:{}", v),
        None => println!("trap:DivByZero"),
    }
}
