function main(): void {
    const add: (number, number) => number = (a: number, b: number) => a + b;
    const result = add(5, 3);
    console.log(result);
}

main();
