function main(): void {
    const val: number | Error = 42;
    const err: any | Error = new Error("something failed");
    console.log(val);
    console.log(err);
}

main();
