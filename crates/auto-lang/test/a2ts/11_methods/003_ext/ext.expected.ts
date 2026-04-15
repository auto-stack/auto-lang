class Point {
    x: number;
    y: number;

    constructor(x: number, y: number) {
        this.x = x;
        this.y = y;
    }

    distance(other: Point): number {
        const dx: number = this.x - other.x;
        const dy: number = this.y - other.y;
        return dx * dx + dy * dy;
    }
}

function main(): void {
    const p1 = Point(1, 2);
    const p2 = Point(4, 6);
    const d = p1.distance(p2);
    console.log("Distance:", d);
}

main();
