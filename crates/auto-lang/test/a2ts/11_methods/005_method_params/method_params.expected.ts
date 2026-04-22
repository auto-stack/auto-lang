class Counter {
    count: number;

    constructor(count: number) {
        this.count = count;
    }

    add(n: number): void {
        this.count = this.count + n;
    }
}

function main(): void {
    const c = Counter(0);
    c.add(5);
    console.log(c.count);
}

main();
