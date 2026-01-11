fn main() {
    let x: i32 = 10;
    let y: i32 = 20;

    match x {
        10 => println!("X is 10"),
        20 => println!("X is 20"),
        _ => println!("X is something else"),
    }

    match y {
        5 => println!("Y is 5"),
        10 => println!("Y is 10"),
        15 => println!("Y is 15"),
        _ => println!("Y is unknown"),
    }
}
