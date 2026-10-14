use auto_ac_prototype::{descriptor,record_descriptor,verify,hir::ExprKind};
fn main() {
 let text=std::fs::read_to_string(std::env::args().nth(1).unwrap()).unwrap();
 let control=descriptor::bind(record_descriptor(),&text).unwrap();
 assert!(verify::verify_record(control.clone()).is_ok());
 for mode in ["binding-arg", "expression-ref"] {
  let mut b=control.clone();
  let mut mutated=false;
  for body in &mut b.modules[0].bodies {
   for expr in &mut body.exprs {
    if let ExprKind::Call { bindings,eval_args,.. }=&mut expr.kind {
     if !bindings.is_empty() {
      if mode=="binding-arg" { bindings[0].arg=u32::MAX; }
      else { eval_args[0].0=u32::MAX; }
      mutated=true; break;
     }
    }
   }
   if mutated { break; }
  }
  assert!(mutated);
  let r=std::panic::catch_unwind(||verify::verify_record(b));
  println!("{mode}: {}",match r {Ok(Err(_))=>"controlled reject",Ok(Ok(_))=>"ACCEPTED",Err(_)=>"PANIC"});
 }
}
