mod generated {
    include!("generated.rs");
    pub fn entry() -> i64 { main() }
}
fn main() {
    let observed = generated::entry() as i32;
    println!("i32:{}", observed);
}
