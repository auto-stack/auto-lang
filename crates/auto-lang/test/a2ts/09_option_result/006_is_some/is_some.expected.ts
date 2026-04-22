function main(): void {
    const x: number | null = 10;
    const y: any | null = null;
    const a = x.is_some();
    const b = y.is_some();
    console.log(a);
    console.log(b);
}

main();
