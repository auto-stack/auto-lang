// reference-N889-nested-variant.rs — hand-written std-only reference (PLAN-904); independent
// of the generated runtime/helpers.
fn main() {
    enum Inner { Data((String, i32)), End }
    enum Outer { Wrap(Inner), Empty }
    fn make() -> Outer {
        Outer::Wrap(Inner::Data(("中".to_string(), 41)))
    }
    let mut xs: Vec<Outer> = Vec::new();
    let created = make();
    xs.push(created);
    let o = xs.remove(0);
    match o {
        Outer::Wrap(inner) => match inner {
            Inner::Data((n, c)) => println!("value:{}", n.chars().count() as i32 + c),
            Inner::End => println!("value:0"),
        },
        Outer::Empty => println!("value:0"),
    }
}
