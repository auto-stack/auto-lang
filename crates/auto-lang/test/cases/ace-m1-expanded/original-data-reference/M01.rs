// PLAN-941 independent Rust reference: original source semantics, no emitted code.
#[derive(Clone,Copy)] struct Pair{left:i32,right:bool}
impl Pair{fn new(left:i32,right:bool)->Self{Self{left,right}}fn left_is(&self,v:i32)->bool{self.left==v}}
struct Inner{_flag:bool,p:Pair}
fn calc()->i32{let p=Pair::new(1,true);let mut q=p;q.left=5;
if p.left!=1{return 1}if q.left!=5{return 2}if !p.left_is(1){return 3}
let i=Inner{p,_flag:false};if !i.p.right{return 4}if !i.p.left_is(1){return 5}
let n=Pair{right:true,left:9};if n.left!=9{return 6}0}
fn main(){println!("value:{}",calc());}
