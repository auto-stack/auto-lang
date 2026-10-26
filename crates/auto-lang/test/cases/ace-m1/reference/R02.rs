// Independent hand-written behavior reference; not an a2r golden.
fn main() {
    let observed: i32 = { struct Inner { value: i32 } struct Outer { inner: Inner, extra: i32 } let mut p=Outer{inner:Inner{value:3},extra:2}; p.inner.value=7; p.inner.value+p.extra };
    assert_eq!(observed, 9);
    println!("i32:{}", observed);
}
