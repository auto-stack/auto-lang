// PLAN-977 cover-977 A2R certify leg: the HAND-WRITTEN independent
// reference for the public source `fn main() int { return 2 + 3 * 4 }`
// (expected value 14, hand-computed). Compiled and run separately from
// the emitted program; both must print `value:14\n` (hex
// 76616c75653a31340a). Never generated from the transpiler output.
fn main() {
    println!("value:14");
}
