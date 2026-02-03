fn main() {
    let s: String = "hello";


    let v1: String = &s;
    let v2: String = &s;



    println!("{}", v1);
    println!("{}", v2);
}
