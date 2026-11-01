// Independent hand-written behavior reference (PLAN-909).
// Not an a2r golden; verified by reasoning against the Auto semantics.
fn main() {
    use std::cell::Cell;
    let observed: i32 = { let x = Cell::new(1); let f = |n: i32| { x.set(x.get() + n); x.get() }; let g = |n: i32| { x.set(x.get() + n); x.get() }; f(20) + g(0) };
    assert_eq!(observed, 42);
    println!("i32:{}", observed);
}
