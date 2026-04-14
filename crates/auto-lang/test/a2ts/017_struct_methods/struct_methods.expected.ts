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
