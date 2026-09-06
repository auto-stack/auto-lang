export class Point {
    x: number;
    y: number;

    constructor(x: number, y: number) {
        this.x = x;
        this.y = y;
    }

    modulus(): number {
        return x * x + y * y;
    }
}

function main(): void {
    const p = new Point(3, 4);
    const m: number = p.modulus();
    console.log("Modulus:", m);
}

main();
