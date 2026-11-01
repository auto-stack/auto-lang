// reference-L06.rs — hand-written std-only reference (PLAN-904); independent
// of the generated runtime/helpers.
fn main() {
    let mut xs: Vec<i32> = Vec::new();
    xs.push(42);
    // the ONLY outcome: the named list-index trap (checked access, never
    // a panic, never value 70)
    match xs.get(1) {
        Some(v) => println!("value:{}", v),
        None => println!("trap:ListIndex"),
    }
}
