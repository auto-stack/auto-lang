fn main() {
    let name: String = "AutoLang";
    let age: i32 = 1;


    let greeting: String = format!("Hello, {}", name);
    println!("{}", greeting);


    let info: String = format!("Name: {}, Age: {}", name, age);
    println!("{}", info);


    let x: i32 = 10;
    let y: i32 = 20;
    let result: String = format!("Result: {}", x + y);
    println!("{}", result);


    let msg: String = format!("The value is {} and {}", x, y * 2);
    println!("{}", msg);


    println!("Direct: {} is {} years old", name, age);


    let a: i32 = 5;
    let b: i32 = 3;
    println!("Sum: {}, Product: {}", a + b, a * b);
}
