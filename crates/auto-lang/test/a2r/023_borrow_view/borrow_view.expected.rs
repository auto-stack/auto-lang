fn main() {
    let s: String = "hello";
    let slice = &s;
    println!("{}", str_len(slice));
    println!("{}", s);
}
