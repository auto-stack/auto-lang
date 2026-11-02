// PLAN-914 E01 (a2r axis) — INDEPENDENT hand-written reference.
// The Auto source is `sub(b: mark_b(), a: mark_a())` with sub(a,b) = a*3+b*2,
// mark_a() = 1 ("a" on stdout), mark_b() = 2 ("b" on stdout). Named arguments
// evaluate in SOURCE order (b then a) and bind BY NAME, so the frozen
// observable is the trace bytes "ba" followed by the value 7.
// This file is written from the language semantics above, not from the
// transpiler output; the certification harness runs BOTH and requires the
// same stdout.
fn main() {
    print!("b");
    print!("a");
    println!("i32:{}", 1 * 3 + 2 * 2);
}
