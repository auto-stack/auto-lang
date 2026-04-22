function main(): void {
    const x: any | null = null;
    const value: any | null = x ?? 10;
    console.log(value);
}

main();
