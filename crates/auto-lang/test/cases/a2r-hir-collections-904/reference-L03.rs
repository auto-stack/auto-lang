// reference-L03.rs — hand-written std-only reference (PLAN-904); independent
// of the generated runtime/helpers.
fn main() {
    let mut xs: Vec<i32> = Vec::new();
    if xs.len() != 0 { println!("value:1"); return; }
    xs.push(42);
    xs.clear();
    println!("value:{}", xs.len() as i32);
}
