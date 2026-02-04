fn check_value() -> MayBool {
    let x: i32 = 42;
    let is_positive: i32 = x > 0;
    May.val(is_positive)
}

fn main() {
    let result: MayBool = check_value();
}
