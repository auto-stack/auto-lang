// Independent hand-written behavior reference (PLAN-909).
// Not an a2r golden; verified by reasoning against the Auto semantics.
fn main() {
    let observed: i32 = { fn make() -> impl Fn(i32) -> i32 { let mut count = 1; move |n: i32| { count += n; count } } let f = make(); f(20) + f(0) };
    assert_eq!(observed, 42);
    println!("i32:{}", observed);
}
