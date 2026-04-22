function check(x: number): void {
    switch (x) {
        case 1:
            console.log("one");
            break;
        case 2:
            console.log("two");
            break;
        case 3:
            console.log("three");
            break;
        case _:
            console.log("other");
            break;
    }
}

function main(): void {
    check(1);
    check(2);
    check(99);
}

main();
