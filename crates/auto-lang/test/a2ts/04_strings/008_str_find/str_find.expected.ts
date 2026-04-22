function main(): void {
    const s: string = "hello world";
    const found = s.find("world");
    const contains_hello = s.contains("hello");
    console.log(found);
    console.log(contains_hello);
}

main();
