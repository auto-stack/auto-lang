function main(): void {
    const x: number | null = 10;
    

    switch (x) {
        case { _tag: "Some", value: v }:
            console.log("got value:", v);
            break;
        case null:
            console.log("got none");
            break;
    }
}

main();
