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


class Pigeon implements Flyer {

    fly(): void {
    console.log("Flap");
}
}

function main(): void {
    const p = Pigeon();
    p.fly();
}

main();
