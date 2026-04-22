async function get_value(): Future<number> {
    return 42;
}

async function main(): void {
    const val = (await get_value());
    console.log(val);
}

main();
