// Independent PLAN-941 query/instance reference; no emitted source.
fn choose<T>(a:T,b:T,flag:bool)->T{if flag{return a} b}
fn wrap<T>(a:T,b:T)->T{choose(a,b,true)}
fn main(){let value=if wrap(false,true){1}else{wrap(40,42)};println!("value:{}",value);}
