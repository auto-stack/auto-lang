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
}

class Circle {
    radius: number;
    border: number;
    center: Point;

    constructor(radius: number, border: number, center: Point) {
        this.radius = radius;
        this.border = border;
        this.center = center;
    }
}

function main(): void {
    let p = Point(1, 2);
    p.x = 3;
    console.log(`P: ${p.x}, ${p.y}`);
    

    const circle = Circle(5, 1, new Point(50, 50));
    console.log(`C: ${circle.center.x}, ${circle.center.y}, ${circle.radius}`);
}

main();
