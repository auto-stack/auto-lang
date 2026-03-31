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
    for (let j = 0; j < 10; j++) {
        console.log(j);
    }
    

    const arr: number[] = [1, 2, 3];
    for (const n of arr) {
        console.log(n);
    }
}

main();
