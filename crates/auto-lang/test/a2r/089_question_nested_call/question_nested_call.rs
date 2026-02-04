fn get_value() -> MayInt {
    let x: i32 = 42;
    May.val(x)
}

fn use_value() -> MayInt {
    get_value()?;
}

fn main() {
    let a: MayInt = use_value();
}
