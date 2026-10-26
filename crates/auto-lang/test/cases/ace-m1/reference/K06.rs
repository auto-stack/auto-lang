// Independent hand-written behavior reference; not an a2r golden.
fn main() {
    let observed: i32 = { let mut base=1;let mut f=move |n:i32|{base+=n;base};f(20)+f(0) };
    assert_eq!(observed, 42);
    println!("i32:{}", observed);
}
