// reference-N889-optional-variant.rs — hand-written std-only reference (PLAN-904); independent
// of the generated runtime/helpers.
fn main() {
    fn make() -> Option<(String, i32)> {
        Some(("中".to_string(), 41))
    }
    let mut xs: Vec<Option<(String, i32)>> = Vec::new();
    let created = make();
    xs.push(created);
    let o = xs.remove(0);
    match o {
        Some((n, c)) => println!("value:{}", n.chars().count() as i32 + c),
        None => println!("value:0"),
    }
}
