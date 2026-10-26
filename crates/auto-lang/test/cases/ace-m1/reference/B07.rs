// Independent hand-written behavior reference; not an a2r golden.
fn main() {
    let observed: i32 = { let mut total = 0; for _ in 0..3 { for j in 1..=3 { if j == 2 { continue; } if j == 3 { break; } total += 10; } } total };
    assert_eq!(observed, 30);
    println!("i32:{}", observed);
}
