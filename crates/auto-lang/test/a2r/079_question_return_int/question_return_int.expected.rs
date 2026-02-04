fn get_value() -> MayInt {
    let x: i32 = 42;
    x
}

fn get_nil() -> MayInt {
    None;
}

fn main() {
    let a: MayInt = get_value();
    let b: MayInt = get_nil();
}
