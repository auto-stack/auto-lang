// reference-V07.rs — hand-written std-only reference (PLAN-904); independent
// of the generated runtime/helpers.
fn main() {
    let x: Option<i32> = None;
    match x {
        Some(_) => println!("value:1"),
        None => println!("value:42"),
    }
}
