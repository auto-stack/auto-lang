fn test_propagate() -> i32 {
    let x: i32 = 10;
    let y: i32 = x?;
    y
}

fn main() {
    let result: i32 = test_propagate();
}
