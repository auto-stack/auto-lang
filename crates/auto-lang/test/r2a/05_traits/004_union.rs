union Value {
    int_val: i32,
    float_val: f32,
}

fn main() {
    let mut v = Value { int_val: 42 };
    println!("union");
}
