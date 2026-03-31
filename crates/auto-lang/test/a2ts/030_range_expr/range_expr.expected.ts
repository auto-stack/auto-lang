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
    const r = range(0, 5);
    for (const i of r) {
        console.log(i);
    }
    

    const r_eq = range(0, 5, true);
    for (const j of r_eq) {
        console.log(j);
    }
}

main();
