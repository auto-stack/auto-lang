class Inner {
    val: number;

    constructor(val: number) {
        this.val = val;
    }
}

class Outer {
    inner: Inner;
    name: string;

    constructor(inner: Inner, name: string) {
        this.inner = inner;
        this.name = name;
    }
}

function main(): void {
    const i = Inner(42);
    const o = Outer(i, "test");
    console.log(o.inner.val);
}

main();
