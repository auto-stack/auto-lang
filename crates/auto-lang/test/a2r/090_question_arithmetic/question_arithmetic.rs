fn add_values() -> MayInt {
    let a: i32 = 10;
    let b: i32 = 20;
    let result: i32 = a + b;
    May.val(result)
}

fn main() {
    let sum: MayInt = add_values();
}
