function main(): void {
    const x: number = 42;
    switch (x) {
        case 0:
            console.log("zero");
            break;
        case 42:
            console.log("the answer");
            break;
        case _:
            console.log("other");
            break;
    }
}

main();
