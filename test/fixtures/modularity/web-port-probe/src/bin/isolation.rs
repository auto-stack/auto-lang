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
    addresses: Vec<String>,
    shutdowns: Mutex<Vec<tokio::sync::watch::Sender<bool>>>,
}

impl HttpServiceHost for Adapter {
    fn serve(&self, owner: HttpVmOwner) -> HttpServiceFuture {
        let mut controls = self.shutdowns.lock().unwrap();
        let address = self.addresses[controls.len()].clone();
        let (sender, receiver) = tokio::sync::watch::channel(false);
        controls.push(sender);
        Box::pin(async move {
            SERVICES.with(|services| {
                services.borrow_mut().push(tokio::task::spawn_local(
                    owner.run_existing_with_shutdown_for_probe(address, receiver),
                ))
            });
            tokio::task::yield_now().await;
            Ok(())
        })
    }
}

fn main() {
    let args: Vec<_> = std::env::args().skip(1).collect();
    let first = std::fs::read_to_string(&args[0]).unwrap();
    let second = std::fs::read_to_string(&args[1]).unwrap();
    let adapter = Arc::new(Adapter {
        addresses: vec![
            format!("127.0.0.1:{}", args[2]),
            format!("127.0.0.1:{}", args[3]),
        ],
        shutdowns: Mutex::new(Vec::new()),
    });
    http_service::set_process_default_http_service_host(adapter.clone());
    let (commands, mut receiver) = tokio::sync::mpsc::unbounded_channel();
    std::thread::spawn(move || {
        for line in std::io::stdin().lock().lines() {
            if commands.send(line.unwrap()).is_err() {
                break;
            }
        }
    });
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap();
    tokio::task::LocalSet::new().block_on(&runtime, async {
        auto_lang_core::execute_autovm(&first, true).await.unwrap();
        assert_eq!(receiver.recv().await.as_deref(), Some("second"));
        auto_lang_core::execute_autovm(&second, true).await.unwrap();
        let mut command = receiver.recv().await.unwrap();
        let first_stopped = command == "stop-first";
        if first_stopped {
            adapter.shutdowns.lock().unwrap()[0].send(true).unwrap();
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
        assert_eq!(auto_lang_core::vm::ffi::http_server::live_scope_count(), 0);
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
}
