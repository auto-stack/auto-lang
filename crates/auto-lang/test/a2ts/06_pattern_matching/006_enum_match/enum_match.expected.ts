const enum Color {
    Red,
    Green = 1,
    Blue = 2
}

function check_color(c: Color): void {
    switch (c) {
        case Color.Red():
            console.log("red");
            break;
        case Color.Green():
            console.log("green");
            break;
        case Color.Blue():
            console.log("blue");
            break;
    }
}

function main(): void {
    check_color(Color.Red);
}

main();
