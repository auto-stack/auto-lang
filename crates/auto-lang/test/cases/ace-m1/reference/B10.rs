// Independent hand-written behavior reference; not an a2r golden.
fn main() {
    let observed: i32 = { fn even(n: i32) -> bool { n == 0 || odd(n-1) } fn odd(n: i32) -> bool { n != 0 && even(n-1) } if even(6) { 42 } else { 1 } };
    assert_eq!(observed, 42);
    println!("i32:{}", observed);
}
