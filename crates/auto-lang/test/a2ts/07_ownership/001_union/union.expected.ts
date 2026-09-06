export interface MyUnion {
    i?: number;
    f?: number;
    c?: number;
}


function main(): void {
    const my_union = MyUnion(i: 42);
    console.log("int value:", my_union.i);
}

main();
