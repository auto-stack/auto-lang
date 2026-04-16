fn maybe_value(x: i32) -> Option<i32> {
    if x > 0 {
        return Some(x);
    }
    return None;
}

fn divide(a: i32, b: i32) -> Result<i32, String> {
    if b == 0 {
        return Err("division by zero");
    }
    return Ok(a / b);
}

fn main() {
    let a: Option<i32> = Some(42);
    let b: Option<i32> = None;
    let c: Result<i32, String> = Ok(100);
    let d: Result<i32, String> = Err("something went wrong");
}
