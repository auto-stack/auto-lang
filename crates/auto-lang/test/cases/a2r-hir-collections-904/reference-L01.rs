// reference-L01.rs — hand-written std-only reference (PLAN-904); independent
// of the generated runtime/helpers.
fn main() {
    let mut xs: Vec<i32> = Vec::new();
    xs.push(20);
    xs.push(22);
    println!("value:{}", xs[0] + xs[1]);
}
