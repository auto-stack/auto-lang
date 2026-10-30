//! PLAN-890: external SDK consumer, deliberately no Web/UI/Man dependency.
use std::io::BufRead;

struct ExplicitProbeAdapter;

#[cfg(feature = "external-runtime")]
static EXTERNAL_SHUTDOWN: std::sync::Mutex<Option<tokio::sync::watch::Sender<bool>>> =
    std::sync::Mutex::new(None);

#[cfg(feature = "compile-fail-owner-send")]
fn reject_moving_vm_owner_to_network_thread() {
    fn requires_send<T: Send>() {}
    requires_send::<auto_lang_core::vm::host_capabilities::http_service::HttpVmOwner>();
    requires_send::<auto_lang_core::vm::host_capabilities::request_context::RequestContextGuard>();
}

impl auto_lang_core::vm::host_capabilities::http_service::HttpServiceHost for ExplicitProbeAdapter {
    fn serve(
        &self,
        owner: auto_lang_core::vm::host_capabilities::http_service::HttpVmOwner,
    ) -> auto_lang_core::vm::host_capabilities::http_service::HttpServiceFuture {
        eprintln!("explicit probe adapter: {} route(s)", owner.route_count());
        #[cfg(not(feature = "external-runtime"))]
        {
            Box::pin(owner.run_existing_runtime_for_probe())
        }
        #[cfg(feature = "external-runtime")]
        {
            eprintln!("probe runtime=external");
            let (sender, receiver) = tokio::sync::watch::channel(false);
            *EXTERNAL_SHUTDOWN.lock().unwrap() = Some(sender);
            let address = format!("127.0.0.1:{}", std::env::var("AUTO_HTTP_PORT").unwrap());
            Box::pin(web_runtime_boundary_probe::serve_owned(
                owner, address, receiver,
            ))
        }
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
                #[cfg(not(feature = "external-runtime"))]
                let sent = auto_lang_core::vm::ffi::http_server::test_trigger_shutdown();
                #[cfg(feature = "external-runtime")]
                let sent = EXTERNAL_SHUTDOWN
                    .lock()
                    .unwrap()
                    .as_ref()
                    .is_some_and(|sender| sender.send(true).is_ok());
                eprintln!("probe shutdown delivered={sent}");
                break;
            }
        }
    });
    // Only the explicit flag above installs a host. Missing-provider and plain
    // language controls use this exact public call with no installation.
    match auto_lang_core::run_with_capture(&source) {
        Ok((value, stdout)) => {
            // Inspect real registries after the owner has returned. A successful
            // process exit alone cannot establish resource termination.
            #[cfg(not(feature = "external-runtime"))]
            let scopes = auto_lang_core::vm::ffi::http_server::live_scope_count();
            #[cfg(feature = "external-runtime")]
            let scopes = web_runtime_boundary_probe::http_server::live_scope_count();
            let files = auto_lang_core::vm::ffi::http_server_file::vm_file_response_count();
            let uploads = auto_lang_core::vm::ffi::http_upload::vm_upload_counts();
            let streams = auto_lang_core::vm::ffi::http_stream::stream_live_count();
            let live_ops = auto_lang_core::vm::ffi::async_http::live_op_count();
            eprintln!("probe terminal scopes={scopes} files={files} uploads={uploads:?} streams={streams} live_ops={live_ops}");
            assert_eq!(
                (scopes, files, uploads, streams, live_ops),
                (0, 0, (0, 0, 0), 0, 0)
            );
            println!("value={value:?} stdout={stdout:?}");
        }
        Err(error) => {
            eprintln!("SDK execution failed: {error}");
            std::process::exit(1);
        }
    }
}
