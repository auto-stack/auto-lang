//! Failure followed by a configured service and actual future cancellation.
#[cfg(feature = "external-runtime")]
fn main() {
    use auto_lang_core::vm::host_capabilities::http_service::*;
    use std::{
        cell::RefCell,
        io::BufRead,
        sync::{
            atomic::{AtomicBool, Ordering},
            Arc, Mutex,
        },
    };
    thread_local! {
        static TASK: RefCell<Option<tokio::task::JoinHandle<auto_lang_core::AutoResult<()>>>> = const { RefCell::new(None) };
    }
    struct Host {
        detached: AtomicBool,
        stop: Mutex<Option<tokio::sync::watch::Sender<bool>>>,
    }
    impl HttpServiceHost for Host {
        fn serve(&self, owner: HttpVmOwner) -> HttpServiceFuture {
            let config = owner
                .launch()
                .unwrap()
                .settings::<auto_lang_core::http_service_config::HttpServiceConfig>()
                .unwrap();
            let address = format!("{}:{}", config.listen_addr, config.listen_port);
            let (sender, receiver) = tokio::sync::watch::channel(false);
            *self.stop.lock().unwrap() = Some(sender.clone());
            let future = web_runtime_boundary_probe::serve_service_owned(
                owner, address, config, sender, receiver,
            );
            if self.detached.load(Ordering::SeqCst) {
                TASK.with(|task| *task.borrow_mut() = Some(tokio::task::spawn_local(future)));
                Box::pin(async {
                    tokio::task::yield_now().await;
                    Ok(())
                })
            } else {
                Box::pin(future)
            }
        }
    }
    let args: Vec<_> = std::env::args().skip(1).collect();
    let sources = [
        std::fs::read_to_string(&args[0]).unwrap(),
        std::fs::read_to_string(&args[1]).unwrap(),
    ];
    let configs: Vec<_> = args[2..4]
        .iter()
        .map(|path| {
            Arc::new(
                auto_lang_core::http_service_config::resolve_service_config(
                    &std::fs::read_to_string(path).unwrap(),
                    None,
                )
                .unwrap(),
            )
        })
        .collect();
    let host = Arc::new(Host {
        detached: AtomicBool::new(false),
        stop: Mutex::new(None),
    });
    assert!(install_process_http_service_host_if_absent(host.clone()));
    let (commands, mut receiver) = tokio::sync::mpsc::unbounded_channel();
    let controller = std::thread::spawn(move || {
        for line in std::io::stdin().lock().lines() {
            let line = line.unwrap();
            let final_command = line == "cancel" || line == "shutdown";
            if commands.send(line).is_err() || final_command {
                break;
            }
        }
    });
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap();
    tokio::task::LocalSet::new().block_on(&runtime, async {
        let failure = with_http_service_launch_async(
            Some(HttpServiceLaunch::new(configs[0].clone())),
            auto_lang_core::execute_autovm(&sources[0], true),
        )
        .await
        .unwrap_err();
        assert!(failure.to_string().contains("bind failed"));
        let first = web_runtime_boundary_probe::observations().remove(0);
        assert_eq!(first.log_threads_active(), 0);
        assert!(
            first.service_identity().is_none(),
            "failed instance must never be ready"
        );
        host.detached.store(true, Ordering::SeqCst);
        with_http_service_launch_async(
            Some(HttpServiceLaunch::new(configs[1].clone())),
            auto_lang_core::execute_autovm(&sources[1], true),
        )
        .await
        .unwrap();
        assert!(current_http_service_launch().is_none());
        println!("failed A propagated; B launched independently");
        let command = receiver.recv().await.unwrap();
        let task = TASK.with(|task| task.borrow_mut().take().unwrap());
        if command == "cancel" {
            assert!(
                web_runtime_boundary_probe::http_server::live_scope_count() > 0,
                "cancel must reach a real in-flight request"
            );
            task.abort();
            assert!(task.await.unwrap_err().is_cancelled());
        } else {
            assert_eq!(command, "shutdown");
            host.stop
                .lock()
                .unwrap()
                .as_ref()
                .unwrap()
                .send(true)
                .unwrap();
            task.await.unwrap().unwrap();
        }
        let session = web_runtime_boundary_probe::sessions()[1].clone();
        tokio::time::timeout(std::time::Duration::from_secs(5), session.wait_cleanup())
            .await
            .unwrap()
            .unwrap();
        let deadline = tokio::time::Instant::now() + std::time::Duration::from_secs(5);
        loop {
            let observations = web_runtime_boundary_probe::observations();
            let second = &observations[1];
            if second.service_state() == auto_web::http_service_observability::ServiceState::Stopped
                && second.log_threads_active() == 0
                && web_runtime_boundary_probe::http_transport::live_network_tails() == 0
            {
                break;
            }
            assert!(
                tokio::time::Instant::now() < deadline,
                "actual session join did not finish"
            );
            tokio::time::sleep(std::time::Duration::from_millis(5)).await;
        }
        assert_eq!(
            web_runtime_boundary_probe::http_server::live_scope_count(),
            0
        );
        assert_eq!(auto_lang_core::vm::ffi::async_http::live_op_count(), 0);
        assert_eq!(auto_lang_core::vm::ffi::http_stream::stream_live_count(), 0);
        assert_eq!(
            auto_lang_core::vm::ffi::http_upload::vm_upload_counts(),
            (0, 0, 0)
        );
        assert_eq!(
            auto_lang_core::vm::ffi::http_server_file::vm_file_response_count(),
            0
        );
        println!("session actual network/log joins and resource registries clean");
    });
    // Closing the parent's stdin lets the controller finish; it owns no VM.
    controller.join().unwrap();
}

#[cfg(not(feature = "external-runtime"))]
fn main() {
    panic!("session probe requires external-runtime");
}
