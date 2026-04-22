function main(): void {
    const x: number | null = 10;
    const filtered = x.filter((v) => v > 5);
    console.log(filtered);
}

main();
