enum Atom {
    Int(i32),
    Char(char),
    Float(f64),
}

fn main() {
    let atom = Atom::Int(11);

    match atom {
        Atom::Int(i) => println!("Got Int: {}", i),
        Atom::Char(c) => println!("Got Char: {}", c),
        Atom::Float(f) => println!("Got Float: {}", f),
    }
}
