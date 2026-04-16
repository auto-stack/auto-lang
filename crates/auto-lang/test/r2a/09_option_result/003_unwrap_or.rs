fn test_coalesce() -> i32 {
    let x: i32 = 10;
    let y: i32 = x.unwrap_or(0);
    y
}

fn main() {
    let a = test_coalesce();
}
