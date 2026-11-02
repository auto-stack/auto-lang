// PLAN-914 E02 (a2r axis) — INDEPENDENT hand-written reference.
// `type Pair { left int right int }`, fan(p) = p.left*101 + p.right, and the
// constructor writes `Pair { right: mark_b(), left: mark_a() }`: the field
// expressions evaluate in SOURCE order (right/b then left/a) and bind BY
// FIELD NAME, so left=1, right=2, value 103, trace bytes "ba".
fn main() {
    print!("b");
    print!("a");
    println!("i32:{}", 1 * 101 + 2);
}
