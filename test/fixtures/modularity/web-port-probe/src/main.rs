//! PLAN-890: external SDK consumer, deliberately no Web/UI/Man dependency.
use std::io::BufRead;

struct ExplicitProbeAdapter;

#[cfg(feature = "compile-fail-owner-send")]
fn reject_moving_vm_owner_to_network_thread() {
    fn requires_send<T: Send>() {}
    requires_send::<auto_lang_core::vm::host_capabilities::http_service::HttpVmOwner>();
}

impl auto_lang_core::vm::host_capabilities::http_service::HttpServiceHost for ExplicitProbeAdapter {
    fn serve(
        &self,
        owner: auto_lang_core::vm::host_capabilities::http_service::HttpVmOwner,
    ) -> auto_lang_core::vm::host_capabilities::http_service::HttpServiceFuture {
        eprintln!("explicit probe adapter: {} route(s)", owner.route_count());
        Box::pin(owner.run_existing_runtime_for_probe())
    }
}

fn main() {
    let arguments: Vec<_> = std::env::args().skip(1).collect();
    let source =
        std::fs::read_to_string(arguments.first().expect("source path")).expect("read source");
    if arguments
        .iter()
        .any(|argument| argument == "--install-adapter")
    {
        auto_lang_core::vm::host_capabilities::http_service::install_process_http_service_host_if_absent(
            std::sync::Arc::new(ExplicitProbeAdapter),
        );
    }
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
    // Only the explicit flag above installs a host. Missing-provider and plain
    // language controls use this exact public call with no installation.
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
