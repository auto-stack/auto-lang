function main(): void {
    const x: number | null = 5;
    const doubled = x.map((v) => v * 2);
    console.log(doubled);
}

main();
