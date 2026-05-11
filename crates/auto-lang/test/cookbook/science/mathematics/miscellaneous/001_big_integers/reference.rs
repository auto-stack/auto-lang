use num::bigint::BigInt;

fn main() {
    let a = BigInt::from(100);
    let b = BigInt::from(200);
    let sum = &a + &b;
    let product = &a * &b;
    println!("Sum: {}", sum);
    println!("Product: {}", product);
}
