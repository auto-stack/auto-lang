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


class Calculator {
    value: number;

    constructor(value: number) {
        this.value = value;
    }

    add(a: number, b: number): number {
    a + b;
}

    multiply(a: number, b: number): number {
    a * b;
}
}

function main(): void {
    const calc = Calculator(0);
    const sum = calc.add(5, 3);
    const product = calc.multiply(4, 7);
    console.log("Sum:", sum, "Product:", product);
}

main();
