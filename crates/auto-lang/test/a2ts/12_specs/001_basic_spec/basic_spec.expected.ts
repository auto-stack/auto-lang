export interface Flyer { {
    fly(): void;
}


export class Pigeon implements Flyer {

    fly(): void {
        console.log("Flap");
    }
}

function main(): void {
    const p = new Pigeon();
    p.fly();
}

main();
