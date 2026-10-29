//! PLAN-890: external SDK consumer, deliberately no Web/UI/Man dependency.
use std::io::BufRead;

fn main() {
    let source = std::fs::read_to_string(std::env::args().nth(1).expect("source path"))
        .expect("read source");
    // Only this probe's controller owns stdin. A command asks the actual
    // transport to drain; it does not turn a timeout into successful cleanup.
    std::thread::spawn(|| {
        for line in std::io::stdin().lock().lines() {
            if matches!(line.as_deref(), Ok("shutdown")) {
                let sent = auto_lang_core::vm::ffi::http_server::test_trigger_shutdown();
                eprintln!("probe shutdown delivered={sent}");
                break;
            }
        }
    });
    // No provider registration or auto-api-gen facade precedes this public call.
    match auto_lang_core::run_with_capture(&source) {
        Ok((value, stdout)) => {
            println!("value={value:?} stdout={stdout:?}");
        }
        Err(error) => {
            eprintln!("SDK execution failed: {error}");
            std::process::exit(1);
        }
    }
}
