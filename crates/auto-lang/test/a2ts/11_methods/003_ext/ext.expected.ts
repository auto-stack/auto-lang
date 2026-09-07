export class Point {
    x: number;
    y: number;

    constructor(x: number, y: number) {
        this.x = x;
        this.y = y;
    }

    distance(other: Point): number {
        const dx: number = x - other.x;
        const dy: number = y - other.y;
        return dx * dx + dy * dy;
    }
}

function main(): void {
    const p1 = new Point(1, 2);
    const p2 = new Point(4, 6);
    const d = p1.distance(p2);
    console.log("Distance:", d);
}

main();
