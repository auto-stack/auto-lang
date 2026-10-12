use auto_ac_prototype::link;
use std::{env,fs,path::PathBuf,process::Command,time::{Duration,Instant}};
fn main() {
 let args:Vec<String>=env::args().collect();
 if args.len()==1 {
  let child=Command::new(env::current_exe().unwrap()).arg("pipe-child").spawn().unwrap();
  fs::write(env::var("P741_REVIEW_CHILD_PID").unwrap(),child.id().to_string()).unwrap();
  return;
 }
 match args[1].as_str() {
  "pipe-child"=>std::thread::sleep(Duration::from_secs(120)),
  "watchdog"=>{let started=Instant::now();let r=link::run_exe(&env::current_exe().unwrap());println!("elapsed={:?},result={r:?}",started.elapsed());},
  "receipt-prepare"=>{
   let dir=PathBuf::from(&args[2]);fs::create_dir_all(&dir).unwrap();
   let exe=dir.join("tx.exe");let obj=dir.join("tx.obj");let receipt=dir.join("tx.ac-link.txt");
   fs::copy(&args[3],&exe).unwrap();fs::copy(&args[4],&obj).unwrap();fs::write(&receipt,b"old receipt").unwrap();
   let before_exe=fs::read(&exe).unwrap();let before_obj=fs::read(&obj).unwrap();
   let tag=link::staging_tag();
   let tmp_exe=dir.join(format!("tx.exe.tmp-{tag}"));
   let tmp_obj=dir.join(format!("tx.obj.tmp-{tag}"));
   fs::copy(&exe,&tmp_exe).unwrap();fs::copy(&obj,&tmp_obj).unwrap();
   fs::create_dir(dir.join(format!("tx.ac-link.txt.tmp-{tag}"))).unwrap();
   let staged=link::StagedLink{tmp_exe:tmp_exe.clone(),exe:exe.clone(),linker:PathBuf::from("rust-lld"),entry_symbol:"ac_start".into()};
   let result=link::publish_artifacts(&staged,&tmp_obj,&obj);
   if result.is_err(){let _=fs::remove_file(&tmp_obj);} // actual main.rs caller cleanup
   println!("result={result:?}");
   println!("staged_exe_left={};old_exe_unchanged={};old_obj_unchanged={}",tmp_exe.is_file(),fs::read(&exe).unwrap()==before_exe,fs::read(&obj).unwrap()==before_obj);
  },
  _=>panic!("bad mode")
 }
}
