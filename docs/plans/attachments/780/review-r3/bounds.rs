use auto_project_model::{parse_document,SourceInfo};
use auto_man::project_model_adapter::old_normalized_from_code;
use std::io::Write;
fn main(){
 let case=std::env::args().nth(1).unwrap();
 let code=match case.as_str(){
  "valid_array"=>"members: [\"front\", \"back\"]\n",
  "valid_object"=>"back: { project: \"back\" }\n",
  "arithmetic_array"=>"members: [1 * 2]\n",
  "invalid_array"=>"members: [1 : 2]\n",
  "mismatched_bracket"=>"members: [1}\n",
  "invalid_object_key"=>"back: { @: 1 }\n",
  _=>panic!("unknown case")
 };
 println!("case={case} input={code:?}");
 if case=="arithmetic_array"{println!("old={:?}",old_normalized_from_code(code));}
 std::io::stdout().flush().unwrap();
 let result=parse_document(code,SourceInfo::inline());
 println!("new={result:?}");
 match result{
   Ok(d)=>{if case.starts_with("valid_"){assert!(d.is_static_equivalent())}else{assert!(!d.is_static_equivalent(),"unsupported expression/malformed input must not claim static");}},
   Err(_)=>assert!(!case.starts_with("valid_"))
 }
}
