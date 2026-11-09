// Independent standard checked division; MIN / -1 has no i32 quotient.
fn main(){let dividend=std::hint::black_box(i32::MIN);let divisor=std::hint::black_box(-1_i32);
match dividend.checked_div(divisor){Some(value)=>println!("value:{}",value),None=>println!("trap:DivOverflow")}}
