export class Inner {
    value: number;

    constructor(value: number) {
        this.value = value;
    }

    get_value(): number {
        return value;
    }
}

export class Outer {
    inner: Inner;

    constructor(inner: Inner) {
        this.inner = inner;
    }

    get_inner_value(): number {
        return inner.get_value();
    }
}

function main(): void {
    const i = new Inner(42);
    const o = new Outer(i);
    console.log(o.get_inner_value());
}

main();
