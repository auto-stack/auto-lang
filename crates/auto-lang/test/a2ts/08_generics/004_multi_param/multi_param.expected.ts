class Triple {
    a: number;
    b: number;
    c: number;

    constructor(a: number, b: number, c: number) {
        this.a = a;
        this.b = b;
        this.c = c;
    }
}

function main(): void {
    const t = Triple(1, 2, 3);
    console.log(t.a, t.b, t.c);
}

main();
