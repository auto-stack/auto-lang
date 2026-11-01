// Independent hand-written behavior reference (PLAN-909).
// Not an a2r golden; verified by reasoning against the Auto semantics.
fn main() {
    let observed: i32 = { let mut base = 1; let mut f = move |n: i32| { base += n; base }; base + f(20) + f(0) };
    assert_eq!(observed, 43);
    println!("i32:{}", observed);
}
