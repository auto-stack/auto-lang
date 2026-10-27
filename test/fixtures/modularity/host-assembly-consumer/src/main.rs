use auto_lang_core::vm::host_capabilities::{
    process_default_frame_provider, set_process_default_frame_provider, FrameProvider,
};
use std::sync::Arc;

struct Explicit;
impl FrameProvider for Explicit {
    fn begin_ms(&self) -> i64 {
        17
    }
    fn present_ms(&self) -> i64 {
        31
    }
}

fn main() {
    let mode = std::env::args().nth(1).expect("mode");
    #[cfg(feature = "lang")]
    std::hint::black_box(auto_lang::Parser::from(""));
    if mode == "explicit" {
        set_process_default_frame_provider(Arc::new(Explicit));
    }
    if mode == "thread-exit" {
        let worker = std::thread::spawn(|| {
            auto_lang_core::install_ui_host_capabilities();
            assert!(
                process_default_frame_provider().is_some(),
                "worker default missing"
            );
        });
        worker.join().expect("worker panicked");
        assert!(
            process_default_frame_provider().is_some(),
            "default lost after worker exits"
        );
        println!("thread-exit: ok");
        std::process::exit(23);
    }
    auto_lang_core::install_ui_host_capabilities();
    let provider = process_default_frame_provider();
    match mode.as_str() {
        "absent" => assert!(
            provider.is_none(),
            "pure SDK or no-UI consumer gained a provider"
        ),
        "default" => {
            let first = provider.expect("linked UI default missing from direct SDK entry");
            auto_lang_core::install_ui_host_capabilities();
            let second = process_default_frame_provider().expect("default missing on reinstall");
            assert!(
                Arc::ptr_eq(&first, &second),
                "default provider replaced on reinstall"
            );
        }
        "explicit" => {
            let provider = provider.expect("explicit provider missing");
            assert_eq!((provider.begin_ms(), provider.present_ms()), (17, 31));
        }
        _ => panic!("unknown mode"),
    }
    println!("{mode}: ok");
}
