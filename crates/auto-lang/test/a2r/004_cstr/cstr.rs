

fn println(msg: &str) {
    printf("%s\n", msg);
}

fn main() {
    let s: &str = "Hello!";
    println(s);
}
