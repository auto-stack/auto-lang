class Record {
    id: number;
    name: string;
    active: boolean;
    score: number;

    constructor(id: number, name: string, active: boolean, score: number) {
        this.id = id;
        this.name = name;
        this.active = active;
        this.score = score;
    }
}

function main(): void {
    const r = Record(1, "test", true, 95.5);
    console.log(r.name);
}

main();
