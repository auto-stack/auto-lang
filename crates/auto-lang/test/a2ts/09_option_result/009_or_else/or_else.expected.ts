function main(): void {
    const x: any | null = null;
    const val = x.or_else(0);
    console.log(val);
}

main();
