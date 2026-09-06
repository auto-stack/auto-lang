export type Atom =
    { _tag: "Int", value: number }
    | { _tag: "Char", value: number }
    | { _tag: "Float", value: number };

export const Atom = {
    Int: (value: number) => ({ _tag: "Int" as const, value }),
    Char: (value: number) => ({ _tag: "Char" as const, value }),
    Float: (value: number) => ({ _tag: "Float" as const, value })
};


function main(): void {
    const atom = Atom.Int(11);
    

        const __auto_is_0 = atom;
    if (__auto_is_0._tag === "Int") {
        const i = __auto_is_0.value;
        console.log("Got Int:", i);
    }
     else if (__auto_is_0._tag === "Char") {
        const c = __auto_is_0.value;
        console.log("Got Char:", c);
    }
     else if (__auto_is_0._tag === "Float") {
        const f = __auto_is_0.value;
        console.log("Got Float:", f);
    }
}

main();
