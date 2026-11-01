// reference-V03.rs — hand-written std-only reference (PLAN-904); independent
// of the generated runtime/helpers.
fn main() {
    let r: Result<i32, String> = Err("bad".to_string());
    match r {
        Ok(v) => println!("value:{}", v),
        Err(e) => println!("value:{}", e.chars().count() as i32),
    }
}
