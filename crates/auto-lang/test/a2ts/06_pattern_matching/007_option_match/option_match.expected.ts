function main(): void {
    const val: number | null = 42;
    

    switch (val) {
        case { _tag: "Some", value: v }:
            console.log("value:", v);
            break;
        case null:
            console.log("none");
            break;
    }
}

main();
