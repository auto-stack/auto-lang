async function fetch_data(): Future<number> {
    const result = (await 42);
    return result;
}

function main(): void {
    const data = fetch_data();
    console.log(data);
}

main();
