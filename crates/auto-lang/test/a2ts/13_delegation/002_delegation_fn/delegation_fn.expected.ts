class Inner {
    value: number;

    constructor(value: number) {
        this.value = value;
    }

    get_value(): number {
        return this.value;
    }
}

class Outer {
    inner: Inner;

    constructor(inner: Inner) {
        this.inner = inner;
    }

    get_inner_value(): number {
        return this.inner.get_value();
    }
}

function main(): void {
    const i = Inner(42);
    const o = Outer(i);
    console.log(o.get_inner_value());
}

main();
