fn main() {

    let x: i32 = 42;
    let simple: String = format!("Value: {}", x);
    println!("{}", simple);


    let a: i32 = 5;
    let b: i32 = 10;
    let expr: String = format!("Sum: {}", a + b);
    println!("{}", expr);


    let name: String = "World";
    println!("Hello {}", name);
}
