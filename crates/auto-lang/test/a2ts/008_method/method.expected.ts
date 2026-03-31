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


class Point {
    x: number;
    y: number;

    constructor(x: number, y: number) {
        this.x = x;
        this.y = y;
    }

    modulus(): number {
    this.x * this.x + this.y * this.y;
}
}

function main(): void {
    const p = Point(3, 4);
    const m: number = p.modulus();
    console.log("Modulus:", m);
}

main();
