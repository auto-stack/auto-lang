fn check_value() -> MayBool {
    let result: bool = true;
    result
}

fn check_nil() -> MayBool {
    None;
}

fn main() {
    let a: MayBool = check_value();
    let b: MayBool = check_nil();
}
