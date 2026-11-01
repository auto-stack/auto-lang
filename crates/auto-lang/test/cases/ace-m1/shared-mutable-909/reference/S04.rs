// Independent hand-written behavior reference (PLAN-909).
// Not an a2r golden; verified by reasoning against the Auto semantics.
fn main() {
    use std::cell::Cell;
    let observed: i32 = { let x = Cell::new(1); let f = { let x = &x; move |n: i32| { x.set(x.get() + n); x.get() } }; let g = f.clone(); let a = f(20); let b = g(1); let c = f(0); a + b + c };
    assert_eq!(observed, 44);
    println!("i32:{}", observed);
}
