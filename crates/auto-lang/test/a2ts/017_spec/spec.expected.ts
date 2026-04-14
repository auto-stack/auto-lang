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
