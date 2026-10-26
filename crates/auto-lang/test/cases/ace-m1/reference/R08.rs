// Independent hand-written behavior reference; not an a2r golden.
fn main() {
    let observed: i32 = { struct Counter{value:i32} impl Counter{fn new(n:i32)->Self{Self{value:n}} fn read(&self)->i32{self.value}} Counter::new(42).read() };
    assert_eq!(observed, 42);
    println!("i32:{}", observed);
}
