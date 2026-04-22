function main(): void {
    const x: number | null = 42;
    const val = x.unwrap();
    console.log(val);
}

main();
