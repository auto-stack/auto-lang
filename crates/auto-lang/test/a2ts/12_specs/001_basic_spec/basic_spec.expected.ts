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
