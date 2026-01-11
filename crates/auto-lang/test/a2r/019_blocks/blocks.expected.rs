fn add(a: i32, b: i32) -> i32 {
    a + b
}

fn multiply(a: i32, b: i32) -> i32 {
    a * b
}

fn main() {
    let sum: i32 = add(5, 3);
    let product: i32 = multiply(4, 7);
    let result: i32 = sum + product;
    println!("Sum: {} {} {} {} {}", sum, "Product:", product, "Result:", result);
}
