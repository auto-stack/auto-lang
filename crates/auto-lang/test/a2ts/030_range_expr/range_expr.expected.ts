import { range } from "./runtime";

function main(): void {
    const r = range(0, 5);
    for (const i of r) {
        console.log(i);
    }
    

    const r_eq = range(0, 5, true);
    for (const j of r_eq) {
        console.log(j);
    }
}

main();
