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
