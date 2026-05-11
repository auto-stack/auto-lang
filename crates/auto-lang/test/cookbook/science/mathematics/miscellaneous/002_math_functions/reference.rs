fn main() {
    let x: f64 = 2.0;
    let y: f64 = 3.0;
    let power = x.powf(y);
    let sqrt_val = x.sqrt();
    let abs_val: f64 = -5.0;
    let abs_result = abs_val.abs();
    println!("2^3 = {}", power);
    println!("sqrt(2) = {}", sqrt_val);
    println!("abs(-5) = {}", abs_result);
}
