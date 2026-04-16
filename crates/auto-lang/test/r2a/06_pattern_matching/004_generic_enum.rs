enum May<T> {
    val(T),
}

fn main() {
    let x = May::val(42);
    x;
}
