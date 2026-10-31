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
    // PLAN-900：UI 宿主装配扇入单点在 auto-ui-runtime（进程期 inventory
    // 收集点在独立消费方里不保证链接保留——MSVC linker GC；真实宿主
    // （auto-man 等）的规范形态=显式 bootstrap 调用，本 fixture 同款）。
    #[cfg(feature = "ui")]
    auto_ui_runtime::install_ui_host_capabilities();
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
