export function identity(x: number): number {
    return x;
}

function main(): void {
    const result = identity(42);
    console.log(result);
}

main();
