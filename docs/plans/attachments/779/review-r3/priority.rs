use auto_lang::vm::{engine::AutoVM,host_capabilities::*,virt_memory::VirtualFlash};
use std::sync::{Arc,Barrier};
struct Fixed(i64,i64);
impl FrameProvider for Fixed{fn begin_ms(&self)->i64{self.0} fn present_ms(&self)->i64{self.1}}
fn main(){
 let a:Arc<dyn FrameProvider>=Arc::new(Fixed(71,83));
 let b:Arc<dyn FrameProvider>=Arc::new(Fixed(97,101));
 let program="print(frame.begin_ms())\nprint(frame.present_ms())\n";
 clear_process_default_frame_provider();
 set_process_default_frame_provider(a.clone());
 let old=AutoVM::new(VirtualFlash::new(0),64);
 for _ in 0..3{auto_lang::ui::frame_provider::install_process_default();}
 assert!(Arc::ptr_eq(&a,&process_default_frame_provider().unwrap()));
 assert_eq!(auto_lang::run_with_capture(program).unwrap().1,"71\n83\n");
 println!("explicit-default-through-repeated-install-and-run: pass");
 set_process_default_frame_provider(b.clone());
 assert_eq!(auto_lang::run_with_capture(program).unwrap().1,"97\n101\n");
 assert!(Arc::ptr_eq(&a,&old.frame_provider().unwrap()));
 old.set_frame_provider(Arc::new(Fixed(103,107)));
 assert_eq!(old.frame_provider().unwrap().begin_ms(),103);
 assert!(Arc::ptr_eq(&b,&process_default_frame_provider().unwrap()));
 println!("explicit-replace-and-old-vm-snapshot-and-instance-override: pass");
 clear_process_default_frame_provider();
 let barrier=Arc::new(Barrier::new(2));let worker_barrier=barrier.clone();
 let worker=std::thread::spawn(move||{worker_barrier.wait();for _ in 0..64{auto_lang::ui::frame_provider::install_process_default();}});
 barrier.wait();set_process_default_frame_provider(a.clone());worker.join().unwrap();
 assert!(Arc::ptr_eq(&a,&process_default_frame_provider().unwrap()));
 println!("explicit-set-concurrent-with-compat-install: pass");
 clear_process_default_frame_provider();
 auto_lang::ui::frame_provider::install_process_default();
 let compat=process_default_frame_provider().unwrap();
 auto_lang::ui::frame_provider::install_process_default();
 assert!(Arc::ptr_eq(&compat,&process_default_frame_provider().unwrap()));
 assert_eq!(auto_lang::run_with_capture(program).unwrap().1,"0\n0\n");
 println!("empty-slot-fallback-is-stable: pass");
 clear_process_default_frame_provider();
}
