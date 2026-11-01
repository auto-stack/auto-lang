// reference-L02.rs — hand-written std-only reference (PLAN-904); independent
// of the generated runtime/helpers.
fn main() {
    let mut xs: Vec<i32> = Vec::new();
    xs.push(10);
    xs.push(30);
    xs.insert(1, 20);
    xs.remove(1);
    println!("value:{}", xs[0] + xs[1] + xs.len() as i32);
}
