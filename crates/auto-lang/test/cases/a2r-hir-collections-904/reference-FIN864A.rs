// reference-FIN864A.rs — hand-written std-only reference (PLAN-904); independent
// of the generated runtime/helpers.
fn main() {
    let mut xs: Vec<i32> = Vec::new();
    xs.push(10);
    xs.push(20);
    xs.push(12);
    let mut total = 0;
    for x in &xs {
        total += x;
    }
    println!("value:{}", total);
}
