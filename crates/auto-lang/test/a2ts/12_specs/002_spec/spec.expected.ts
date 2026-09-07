export interface Flyer { {
    fly(): void;
}


export class Pigeon implements Flyer {

    fly(): void {
        console.log("Flap Flap");
    }
}

export class Hawk implements Flyer {

    fly(): void {
        console.log("Gawk! Gawk!");
    }
}

function main(): void {
    

    const b1 = new Pigeon();
    const b2 = new Hawk();
    



    const arr: Flyer[] = [b1, b2];
    for (const b of arr) {
        b.fly();
    }
}

main();
