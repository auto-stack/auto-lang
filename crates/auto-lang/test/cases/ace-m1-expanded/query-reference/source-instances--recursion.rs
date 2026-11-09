// Independent PLAN-941 query/instance reference; no emitted source.
fn pick<T>(n:i32,seed:T)->T{if n<=0{return seed} pick(n-1,seed)}
fn main(){println!("value:{}",pick(2,40));}
