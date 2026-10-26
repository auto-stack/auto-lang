// Independent hand-written behavior reference; not an a2r golden.
fn main() {
    let observed: i32 = { struct Item{name:String,code:i32} enum Token{Data(Item),End} let t=Token::Data(Item{name:"hello".into(),code:37});let _end=Token::End;match t{Token::Data(v)=>v.name.len() as i32+v.code,Token::End=>1} };
    assert_eq!(observed, 42);
    println!("i32:{}", observed);
}
