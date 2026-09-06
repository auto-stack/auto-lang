export interface Engine { {
    start(): void;
}


export class WarpDrive {

    start(): void {
        console.log("WarpDrive engaging");
    }
}

export class Starship {
}

function main(): void {
    const ship = new Starship();
    ship.start();
}

main();
