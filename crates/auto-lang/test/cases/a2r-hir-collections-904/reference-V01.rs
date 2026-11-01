// reference-V01.rs — hand-written std-only reference (PLAN-904); independent
// of the generated runtime/helpers.
fn main() {
    let x: Option<i32> = Some(42);
    match x {
        Some(v) => println!("value:{}", v),
        None => println!("value:1"),
    }
}
