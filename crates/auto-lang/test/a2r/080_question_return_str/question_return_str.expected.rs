fn get_string() -> MayStr {
    let s: String = "hello";
    s
}

fn get_nil_string() -> MayStr {
    None;
}

fn main() {
    let a: MayStr = get_string();
    let b: MayStr = get_nil_string();
}
