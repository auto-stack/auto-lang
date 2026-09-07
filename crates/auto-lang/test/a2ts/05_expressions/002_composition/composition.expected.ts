export class Wing {

    fly(): void {
        console.log("flying");
    }
}

export class Duck {

    fly(): void {
        console.log("flying");
    }
}

function main(): void {
    const d = new Duck();
    d.fly();
}

main();
