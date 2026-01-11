fn main() {
    let x: i32 = 10;
    let y: i32 = 20;
    let z: i32 = 30;

    let result1: i32 = x + y * z;
    let result2: i32 = x + y * z;
    let result3: i32 = x + y * z - x;

    println!("Result1: {}", result1);
    println!("Result2: {}", result2);
    println!("Result3: {}", result3);

    let a: i32 = 5;
    let b: i32 = a * 2 + a / 2 - 1;
    println!("b: {}", b);
}
