// reference-V02.rs — hand-written std-only reference (PLAN-904); independent
// of the generated runtime/helpers.
fn main() {
    let b = 2i32;
    let r: Result<i32, String> = if b == 0 {
        Err("bad".to_string())
    } else {
        Ok(84 / b)
    };
    match r {
        Ok(v) => println!("value:{}", v),
        Err(e) => println!("value:{}", e.chars().count() as i32),
    }
}
