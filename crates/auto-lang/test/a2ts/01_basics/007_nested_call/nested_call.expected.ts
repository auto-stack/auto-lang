export function double(x: number): number {
    return x * 2;
}

function main(): void {
    const result = double(double(5));
    console.log(result);
}

main();
