function double(x: number): number {
    x * 2;
}

function main(): void {
    const result = double(double(5));
    console.log(result);
}

main();
