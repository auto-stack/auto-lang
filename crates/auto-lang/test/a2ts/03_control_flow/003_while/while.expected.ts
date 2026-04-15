function main(): void {
    let count: number = 0;
    while (true) {
        if (count >= 10) {
        break;
    }
        console.log(count);
        count = count + 1;
    }
    console.log("done");
}

main();
