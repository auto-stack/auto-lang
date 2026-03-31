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
