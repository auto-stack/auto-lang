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
