// Independent PLAN-941 query/instance reference; no emitted source.
fn choose<T>(a:T,b:T,flag:bool)->T{if flag{return b} a}
fn main(){let value=if choose(true,false,true){choose(40,42,false)}else{0};println!("value:{}",value);}
