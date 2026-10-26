// Independent hand-written behavior reference; not an a2r golden.
fn main() {
    let observed: i32 = { let mut total = 0; while false { total += 1; } total };
    assert_eq!(observed, 0);
    println!("i32:{}", observed);
}
