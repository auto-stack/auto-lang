// PLAN-997 independent trap reference for corpus B13
// (source: `fn main() int { return 2147483647 + 1 }`).
//
// Derived ONLY from the frozen binding expectation (outcome `trap`,
// expected `overflow`) plus the trap-domain observable contract
// (oracle/trap/trap-oracle.json: stderr line `auto.trap:overflow:add`,
// controlled exit 70, clean stdout) — NOT from the a2r emission.
// Independent mechanism: i32::checked_add on an i32 carrier (the a2r
// actual uses an i64 carrier with explicit range checks and a shared
// helper; this reference shares no code with it).
fn main() {
    let a: i32 = 2147483647;
    let b: i32 = 1;
    match a.checked_add(b) {
        Some(_) => std::process::exit(1), // wrong phase: a value result must not pass
        None => {
            eprintln!("auto.trap:overflow:add");
            std::process::exit(70);
        }
    }
}
