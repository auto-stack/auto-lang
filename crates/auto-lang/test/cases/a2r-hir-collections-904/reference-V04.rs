// reference-V04.rs — hand-written std-only reference (PLAN-904); independent
// of the generated runtime/helpers.
// enum Token { Number(int), End } — Number arm
fn main() {
    let t = 42i32;
    match t {
        n => println!("value:{}", n),
    }
}
