class Pair {
    first: number;
    second: number;

    constructor(first: number, second: number) {
        this.first = first;
        this.second = second;
    }
}

function main(): void {
    const p = Pair(1, 2);
    console.log(p.first);
}

main();
