// PLAN-997 independent trap reference for corpus B14
// (source: `fn main() int { return 10 / 0 }`).
//
// Derived ONLY from the frozen binding expectation (outcome `trap`,
// expected `div-by-zero`) plus the trap-domain observable contract
// (stderr line `auto.trap:div-by-zero:div`, controlled exit 70, clean
// stdout) — NOT from the a2r emission. Independent mechanism:
// i32::checked_div (None on zero divisor), an i32 carrier.
fn main() {
    let a: i32 = 10;
    let b: i32 = 0;
    match a.checked_div(b) {
        Some(_) => std::process::exit(1), // wrong phase: a value result must not pass
        None => {
            eprintln!("auto.trap:div-by-zero:div");
            std::process::exit(70);
        }
    }
}
