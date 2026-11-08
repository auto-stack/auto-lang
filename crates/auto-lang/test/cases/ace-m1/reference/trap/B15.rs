// PLAN-997 independent trap reference for corpus B15
// (source: `fn main() int { return -2147483648 % -1 }`).
//
// Derived ONLY from the frozen binding expectation (outcome `trap`,
// expected `rem-overflow`) plus the trap-domain observable contract
// (stderr line `auto.trap:rem-overflow:rem`, controlled exit 70, clean
// stdout) — NOT from the a2r emission. Independent mechanism:
// i32::checked_rem (None on i32::MIN % -1), an i32 carrier. The
// div-by-zero and rem-overflow traps are DISTINCT kinds by contract.
fn main() {
    let a: i32 = -2147483648;
    let b: i32 = -1;
    match a.checked_rem(b) {
        Some(_) => std::process::exit(1), // wrong phase: a value result must not pass
        None => {
            eprintln!("auto.trap:rem-overflow:rem");
            std::process::exit(70);
        }
    }
}
