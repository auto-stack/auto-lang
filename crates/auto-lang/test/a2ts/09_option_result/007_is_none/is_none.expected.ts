function main(): void {
    const x: number | null = 10;
    const y: any | null = null;
    const a = x.is_none();
    const b = y.is_none();
    console.log(a);
    console.log(b);
}

main();
