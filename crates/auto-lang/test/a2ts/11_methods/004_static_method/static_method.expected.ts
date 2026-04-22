class Box {
    width: number;
    height: number;

    constructor(width: number, height: number) {
        this.width = width;
        this.height = height;
    }

    new(w: number, h: number): Box {
        new Box(w, h);
    }
}

function main(): void {
    const b = Box.new(10, 20);
    console.log(b.width);
}

main();
