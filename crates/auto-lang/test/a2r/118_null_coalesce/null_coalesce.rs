fn test_coalesce_int() -> i32 {
    let x: i32 = 10;
    let y: i32 = x ?? 0;
    y
}

fn test_coalesce_with_nil() -> i32 {
    let x = None;
    let y: i32 = x ?? 42;
    y
}

fn main() {
    let a: i32 = test_coalesce_int();
    let b: i32 = test_coalesce_with_nil();
}
