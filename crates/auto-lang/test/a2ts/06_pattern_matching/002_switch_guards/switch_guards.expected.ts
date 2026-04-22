function classify(x: number): void {
    switch (x) {
        case 0:
            console.log("zero");
            break;
        case 1:
            console.log("one");
            break;
        case _:
            console.log("other");
            break;
    }
}

function main(): void {
    classify(0);
    classify(5);
}

main();
