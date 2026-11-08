// PLAN-997 independent trap reference for corpus B20
// (source: `let b bool = true && 10 / 0 > 0; if b { return 1 } return 42`).
//
// Derived ONLY from the frozen binding expectation (outcome `trap`,
// expected `div-by-zero`) plus the trap-domain observable contract
// (stderr line `auto.trap:div-by-zero:div`, controlled exit 70, clean
// stdout) — NOT from the a2r emission. Independent mechanism: native
// Rust `&&` short-circuit over an i32::checked_div RHS. The LHS is
// `true`, so the RHS MUST be evaluated and the division traps; a
// `false` LHS must never evaluate the RHS (the legal short-circuit
// neighbor is certified separately, not by this reference).
fn main() {
    let b = true
        && {
            match 10i32.checked_div(0) {
                Some(q) => q > 0,
                None => {
                    eprintln!("auto.trap:div-by-zero:div");
                    std::process::exit(70);
                }
            }
        };
    let _ = b;
    std::process::exit(1); // unreachable per the frozen oracle (wrong phase)
}
