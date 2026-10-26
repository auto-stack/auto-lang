// Independent hand-written behavior reference; not an a2r golden.
fn main() {
    let observed: i32 = { let a = "A\0B"; if a == "A" { 1 } else if a != "A\0B" { 2 } else { 42 } };
    assert_eq!(observed, 42);
    println!("i32:{}", observed);
}
