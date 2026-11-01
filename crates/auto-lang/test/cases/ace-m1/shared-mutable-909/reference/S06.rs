// Independent hand-written behavior reference (PLAN-909).
// Not an a2r golden; verified by reasoning against the Auto semantics.
fn main() {
    use std::cell::Cell;
    let observed: i32 = { let x = Cell::new(1); let mut m = 1; { let mf = move |n: i32| { m += n; m }; let w = |n: i32| { x.set(x.get() + n); x.get() }; w(40) + mf(1) + x.get() } };
    assert_eq!(observed, 84);
    println!("i32:{}", observed);
}
