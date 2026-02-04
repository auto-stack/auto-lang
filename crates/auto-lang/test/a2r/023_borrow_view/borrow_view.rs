fn main() {
    let s: String = "hello";
    let slice: String = &s;
    println!("{}", str_len(slice));
    println!("{}", s);
}
