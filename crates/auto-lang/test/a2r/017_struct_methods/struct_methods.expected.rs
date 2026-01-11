struct Calculator {
    value: i32,
}

impl Calculator {
    fn add(&self, a: i32, b: i32) -> i32 {
        a + b
    }
    fn multiply(&self, a: i32, b: i32) -> i32 {
        a * b
    }
}

fn main() {
    let calc: Calculator = Calculator { value: 0 };
    let sum = calc.add(5, 3);
    let product = calc.multiply(4, 7);
    println!("Sum: {} {} {}", sum, "Product:", product);
}
