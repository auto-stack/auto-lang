function add(a: number, b: number): number {
    a + b;
}

function multiply(a: number, b: number): number {
    a * b;
}

function main(): void {
    const sum = add(5, 3);
    const product = multiply(4, 7);
    const result = sum + product;
    console.log("Sum:", sum, "Product:", product, "Result:", result);
}

main();
