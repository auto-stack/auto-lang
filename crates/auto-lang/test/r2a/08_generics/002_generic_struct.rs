struct Box<T> {
    value: T,
}

struct Container {
    data: Box<i32>,
}

fn main() {
    let x: i32 = 42;
}
