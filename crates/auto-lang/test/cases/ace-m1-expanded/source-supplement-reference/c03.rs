// Independent reference: leaf triples, mid adds one after the call.
fn leaf(value:i32)->i32 { value * 3 }
fn mid(value:i32)->i32 { leaf(value) + 1 }
fn main(){println!("value:{}",mid(20));}
