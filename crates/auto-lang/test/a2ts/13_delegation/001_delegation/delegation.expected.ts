class WarpDrive {

    start(): void {
        console.log("WarpDrive engaging");
    }
}

class Starship {
}

function main(): void {
    const ship = Starship();
    ship.start();
}

main();
