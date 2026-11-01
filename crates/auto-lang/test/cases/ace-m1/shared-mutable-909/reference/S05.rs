// Independent hand-written behavior reference (PLAN-909).
// Not an a2r golden; verified by reasoning against the Auto semantics.
fn main() {
    use std::cell::Cell;
    let observed: i32 = { let x = Cell::new(1); let r = || x.get(); let w = |n: i32| { x.set(x.get() + n); x.get() }; w(40) + r() };
    assert_eq!(observed, 82);
    println!("i32:{}", observed);
}
