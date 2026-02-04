fn negate_value() -> MayInt {
    let x: i32 = 10;
    let neg: i32 = -x;
    May.val(neg)
}

fn main() {
    let result: MayInt = negate_value();
}
