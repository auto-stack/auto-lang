fn get_value() -> MayInt {
    let x: i32 = 42;
    May.val(x)
}

fn use_value() -> MayInt {
    let result: MayInt = get_value();
    result?;
}

fn main() {
    let a: MayInt = use_value();
}
