function main(): void {
    const x: number | null = 10;
    

        const __auto_is_0 = x;
    if (__auto_is_0 !== null) {
        const v = __auto_is_0;
        console.log("got value:", v);
    }
    else if (__auto_is_0 === null) {
        console.log("got none");
    }
}

main();
