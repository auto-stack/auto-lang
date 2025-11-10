typedef enum Atom {
    Int int
    Char char
    Bool bool
}

fn main() {
    let atom = Atom { Int: 11 }

    is atom {
        Int(i) => print("Got Int:", i)
        Char(c) => print("Got Char:", c)
        Bool(b) => print("Got Bool:", b)
    }
}
