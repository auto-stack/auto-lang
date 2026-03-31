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


class Wing {

    fly(): void {
    console.log("flying");
}
}

class Duck {

    fly(): void {
    console.log("flying");
}
}

function main(): void {
    const d = Duck();
    d.fly();
}

main();
