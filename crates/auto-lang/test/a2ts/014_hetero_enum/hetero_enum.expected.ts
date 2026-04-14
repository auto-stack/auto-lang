type Atom =
    { _tag: "Int", value: number }
    | { _tag: "Char", value: number }
    | { _tag: "Float", value: number };

const Atom = {
    Int: (value: number) => ({ _tag: "Int", value }),
    Char: (value: number) => ({ _tag: "Char", value }),
    Float: (value: number) => ({ _tag: "Float", value })
};


function main(): void {
    const atom = Atom.Int(11);
    

    switch (atom) {
        case Atom.Int(i):
            console.log("Got Int:", i);
            break;
        case Atom.Char(c):
            console.log("Got Char:", c);
            break;
        case Atom.Float(f):
            console.log("Got Float:", f);
            break;
    }
}

main();
