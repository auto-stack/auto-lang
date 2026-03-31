/**
 * AutoLang TypeScript Runtime
 */
const print = console.log.bind(console);

function range(start: number, end: number, eq: boolean = false): number[] {
    const res: number[] = [];
    if (eq) {
        for (let i = start; i <= end; i++) res.push(i);
    } else {
        for (let i = start; i < end; i++) res.push(i);
    }
    return res;
}


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
