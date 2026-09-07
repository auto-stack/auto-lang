export class Counter {
    count: number;

    constructor(count: number) {
        this.count = count;
    }

    add(n: number): void {
        count = count + n;
    }
}

function main(): void {
    const c = new Counter(0);
    c.add(5);
    console.log(c.count);
}

main();
