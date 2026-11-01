// reference-V06.rs — hand-written std-only reference (PLAN-904); independent
// of the generated runtime/helpers.
fn main() {
    let mut r: Result<i32, String> = Err("old".to_string());
    r = Ok(42);
    match r {
        Ok(v) => println!("value:{}", v),
        Err(e) => println!("value:{}", e.chars().count() as i32),
    }
}
