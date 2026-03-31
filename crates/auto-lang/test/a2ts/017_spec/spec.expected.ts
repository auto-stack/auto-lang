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
    console.log("Flap Flap");
}
}

class Hawk implements Flyer {

    fly(): void {
    console.log("Gawk! Gawk!");
}
}

function main(): void {
    

    const b1 = Pigeon();
    const b2 = Hawk();
    



    const arr: Flyer[] = [b1, b2];
    for (const b of arr) {
        b.fly();
    }
}

main();
