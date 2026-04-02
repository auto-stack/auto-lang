/**
 * AutoLang TypeScript Runtime
 */
const print = console.log.bind(console);

function range(start: number, end: number, eq: boolean = false): number[] {
    const res: number[] = [];
    if (eq) {
        for (let i = start; i <= end; i++) res.push(i);
    } else {
        for (let i = start; i < end; i++) res.push(i);
    }
    return res;
}


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
