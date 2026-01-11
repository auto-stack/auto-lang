fn factorial(n: i32) -> i32 {
    if n <= 1 {
        1
    } else {
        n * factorial(n - 1)
    }

}

fn main() {
    let result: i32 = factorial(5);
    println!("Factorial of 5 is: {}", result);
}
