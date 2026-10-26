// Independent hand-written behavior reference; not an a2r golden.
fn main() {
    let observed: i32 = { enum Token{Number(i32),End} let t=Token::Number(42);let _end=Token::End;match t{Token::Number(v)=>v,Token::End=>1} };
    assert_eq!(observed, 42);
    println!("i32:{}", observed);
}
