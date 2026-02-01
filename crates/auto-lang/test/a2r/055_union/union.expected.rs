union MyUnion {
    i: i32,
    f: f64,
    c: char,
}

fn main() {
    let my_union: MyUnion = MyUnion { i: 42 };
    println!("int value: {}", my_union.i);
}
