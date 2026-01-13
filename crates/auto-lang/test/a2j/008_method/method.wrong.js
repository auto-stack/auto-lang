class Point {
    constructor(x, y) {
        this.x = x;
        this.y = y;
    }

    modulus() {
        this.x * this.x + this.y * this.y;
    }
}

function main() {
    console.log("Method defined");
}

main();
