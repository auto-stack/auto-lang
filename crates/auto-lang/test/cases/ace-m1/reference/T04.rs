// Independent hand-written behavior reference; not an a2r golden.
fn main() {
    let observed: i32 = { '😀' as u32 as i32 };
    assert_eq!(observed, 128512);
    println!("i32:{}", observed);
}
