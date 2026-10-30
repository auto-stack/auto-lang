//! Actual two-service metadata isolation oracle on one local VM owner thread.
use auto_lang_core::vm::host_capabilities::http_service::{
    self, HttpServiceFuture, HttpServiceHost, HttpVmOwner,
};
use std::{
    cell::RefCell,
    io::BufRead,
    sync::{Arc, Mutex},
};

thread_local! {
    static SERVICES: RefCell<Vec<tokio::task::JoinHandle<auto_lang_core::AutoResult<()>>>> = const { RefCell::new(Vec::new()) };
}

struct Adapter {
    label: &'static str,
    index: usize,
    addresses: Vec<String>,
    shutdowns: Arc<Mutex<Vec<tokio::sync::watch::Sender<bool>>>>,
    configuration: Option<Arc<auto_lang_core::http_service_config::HttpServiceConfig>>,
}

impl HttpServiceHost for Adapter {
    fn serve(&self, owner: HttpVmOwner) -> HttpServiceFuture {
        eprintln!("isolation host={} activated", self.label);
        #[cfg(feature = "external-runtime")]
        eprintln!("isolation runtime=external");
        let mut controls = self.shutdowns.lock().unwrap();
        let address = self.addresses[self.index].clone();
        let (sender, receiver) = tokio::sync::watch::channel(false);
        controls.push(sender.clone());
        let configuration = self.configuration.clone();
        Box::pin(async move {
            #[cfg(not(feature = "external-runtime"))]
            assert!(
                configuration.is_none(),
                "configured probe requires external-runtime"
            );
            #[cfg(not(feature = "external-runtime"))]
            let service = owner.run_existing_with_shutdown_for_probe(address, receiver);
            #[cfg(feature = "external-runtime")]
            let service: HttpServiceFuture = match configuration {
                Some(config) => Box::pin(web_runtime_boundary_probe::serve_service_owned(
                    owner, address, config, sender, receiver,
                )),
                None => Box::pin(web_runtime_boundary_probe::serve_owned(
                    owner, address, sender, receiver,
                )),
            };
            SERVICES.with(|services| {
                services
                    .borrow_mut()
                    .push(tokio::task::spawn_local(service))
            });
            tokio::task::yield_now().await;
            Ok(())
        })
    }
}

fn main() {
    if std::env::var("PLAN890_EMBEDDING_RATE_DEFAULT").as_deref() == Ok("1") {
        // Real compatibility SDK setter in Core, not this external source
        // crate's former private copy of the process configuration.
        auto_lang_core::vm::ffi::http_server::set_rate_limit(1, 60_000);
        eprintln!("isolation embedding rate default=1");
    }
    let args: Vec<_> = std::env::args().skip(1).collect();
    let first = std::fs::read_to_string(&args[0]).unwrap();
    let second = std::fs::read_to_string(&args[1]).unwrap();
    let adapter = Arc::new(Adapter {
        label: "first",
        index: 0,
        addresses: vec![
            format!("127.0.0.1:{}", args[2]),
            format!("127.0.0.1:{}", args[3]),
        ],
        shutdowns: Arc::new(Mutex::new(Vec::new())),
        configuration: args.get(4).map(|path| {
            Arc::new(
                auto_lang_core::http_service_config::resolve_service_config(
                    &std::fs::read_to_string(path).unwrap(),
                    None,
                )
                .unwrap(),
            )
        }),
    });
    assert!(http_service::install_process_http_service_host_if_absent(
        adapter.clone()
    ));
    let replacement = Arc::new(Adapter {
        label: "second",
        index: 1,
        addresses: adapter.addresses.clone(),
        shutdowns: adapter.shutdowns.clone(),
        configuration: args.get(5).map(|path| {
            Arc::new(
                auto_lang_core::http_service_config::resolve_service_config(
                    &std::fs::read_to_string(path).unwrap(),
                    None,
                )
                .unwrap(),
            )
        }),
    });
    assert!(!http_service::install_process_http_service_host_if_absent(
        replacement.clone()
    ));
    let (commands, mut receiver) = tokio::sync::mpsc::unbounded_channel();
    let controller = std::thread::spawn(move || {
        for line in std::io::stdin().lock().lines() {
            let line = line.unwrap();
            let shutdown = line == "shutdown";
            if commands.send(line).is_err() || shutdown {
                break;
            }
        }
    });
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap();
    tokio::task::LocalSet::new().block_on(&runtime, async {
        if std::env::var("PLAN890_METADATA_GAP").as_deref() == Ok("1") {
            use std::future::Future;
            let mut pending = Box::pin(auto_lang_core::execute_autovm(&first, true));
            std::future::poll_fn(|cx| {
                assert!(
                    pending.as_mut().poll(cx).is_pending(),
                    "first main must actually park"
                );
                std::task::Poll::Ready(())
            })
            .await;
            http_service::set_process_default_http_service_host(replacement);
            auto_lang_core::execute_autovm(&second, true).await.unwrap();
            if std::env::var("PLAN890_ORDINARY_FIRST").as_deref() == Ok("1") {
                http_service::clear_process_default_http_service_host();
            } else {
                http_service::set_process_default_http_service_host(adapter.clone());
            }
            let outcome = pending.await.unwrap();
            println!("gap first result={}", outcome.0);
        } else {
            auto_lang_core::execute_autovm(&first, true).await.unwrap();
            assert_eq!(receiver.recv().await.as_deref(), Some("second"));
            http_service::set_process_default_http_service_host(replacement);
            auto_lang_core::execute_autovm(&second, true).await.unwrap();
        }
        http_service::clear_process_default_http_service_host();
        let mut command = receiver.recv().await.unwrap();
        let first_stopped = command == "stop-first";
        if first_stopped {
            let _ = adapter.shutdowns.lock().unwrap()[0].send(true);
            let first_service = SERVICES.with(|services| services.borrow_mut().remove(0));
            first_service.await.unwrap().unwrap();
            println!("first owner drained independently");
            command = receiver.recv().await.unwrap();
        }
        assert_eq!(command, "shutdown");
        for shutdown in adapter
            .shutdowns
            .lock()
            .unwrap()
            .iter()
            .skip(usize::from(first_stopped))
        {
            shutdown.send(true).unwrap();
        }
        let services = SERVICES.with(|services| std::mem::take(&mut *services.borrow_mut()));
        for service in services {
            service.await.unwrap().unwrap();
        }
        #[cfg(not(feature = "external-runtime"))]
        assert_eq!(auto_lang_core::vm::ffi::http_server::live_scope_count(), 0);
        #[cfg(feature = "external-runtime")]
        assert_eq!(
            web_runtime_boundary_probe::http_server::live_scope_count(),
            0
        );
        assert_eq!(
            auto_lang_core::vm::ffi::http_server_file::vm_file_response_count(),
            0
        );
        assert_eq!(
            auto_lang_core::vm::ffi::http_upload::vm_upload_counts(),
            (0, 0, 0)
        );
        assert_eq!(auto_lang_core::vm::ffi::http_stream::stream_live_count(), 0);
        assert_eq!(auto_lang_core::vm::ffi::async_http::live_op_count(), 0);
        println!("two owners drained");
    });
    controller.join().unwrap();
}
