class Point {
    x: number;
    y: number;

    constructor(x: number, y: number) {
        this.x = x;
        this.y = y;
    }
}

function main(): void {
    const p = Point(3, 4);
    console.log(p.x);
    console.log(p.y);
}

main();
