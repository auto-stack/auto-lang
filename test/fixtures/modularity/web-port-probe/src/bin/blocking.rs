//! Borrowed VM owner test. No VM, task or AST is sent to the controller thread.
use auto_lang_core::vm::host_capabilities::http_service::{
    messages::HttpProgramMetadata, HttpServiceFuture, HttpServiceHost, HttpVmOwner,
};
use std::io::BufRead;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};
static STOP: AtomicBool = AtomicBool::new(false);
struct Adapter {
    address: String,
}
impl HttpServiceHost for Adapter {
    fn serve(&self, owner: HttpVmOwner) -> HttpServiceFuture {
        let address = self.address.clone();
        Box::pin(async move {
            let (vm, _) = owner.into_local_parts();
            auto_lang_core::vm::host_capabilities::http_service::listen_registered(&vm, &address)
        })
    }
    fn listen_blocking(
        &self,
        vm: &auto_lang_core::vm::engine::AutoVM,
        address: &str,
        metadata: Arc<HttpProgramMetadata>,
    ) -> auto_lang_core::AutoResult<()> {
        #[cfg(feature = "external-runtime")]
        {
            eprintln!(
                "blocking runtime=external owner={:?}",
                std::thread::current().id()
            );
            let owner_thread = std::thread::current().id();
            let tasks_before = vm.tasks.len();
            let result = web_runtime_boundary_probe::listen_borrowed(vm, address, metadata, &STOP);
            assert_eq!(std::thread::current().id(), owner_thread);
            assert_eq!(
                vm.tasks.len(),
                tasks_before,
                "all fresh handler tasks removed"
            );
            eprintln!("blocking terminal handler_tasks_delta=0 owner_thread_preserved=true");
            result
        }
        #[cfg(not(feature = "external-runtime"))]
        {
            let _ = (vm, address, metadata);
            Err("blocking fixture requires external-runtime".into())
        }
    }
}
struct AsyncOnly;
impl HttpServiceHost for AsyncOnly {
    fn serve(&self, _owner: HttpVmOwner) -> HttpServiceFuture {
        Box::pin(async { Ok(()) })
    }
}

fn main() {
    let args: Vec<_> = std::env::args().collect();
    let source = std::fs::read_to_string(&args[1]).unwrap();
    let address = args[2].clone();
    if args.iter().any(|arg| arg == "--install-adapter") {
        auto_lang_core::vm::host_capabilities::http_service::install_process_http_service_host_if_absent(
            Arc::new(Adapter { address: address.clone() }));
    }
    if args.iter().any(|arg| arg == "--async-only") {
        auto_lang_core::vm::host_capabilities::http_service::set_process_default_http_service_host(
            Arc::new(AsyncOnly),
        );
    }
    std::thread::spawn(move || {
        for line in std::io::stdin().lock().lines() {
            if matches!(line.as_deref(), Ok("shutdown")) {
                STOP.store(true, Ordering::Release);
                let wake = std::net::TcpStream::connect(&address).is_ok();
                eprintln!("blocking shutdown wake={wake}");
                break;
            }
        }
    });
    if args.iter().any(|arg| arg == "--direct-native") {
        use auto_lang_core::vm::ffi::VMConvertible;
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();
        runtime.block_on(async {
            let vm = auto_lang_core::vm::engine::AutoVM::new_without_host_snapshot(
                auto_lang_core::vm::virt_memory::VirtualFlash::new(0),
                8192,
            );
            let id = vm.spawn_task(0, 8192);
            let task = vm.tasks.get(&id).unwrap().value().clone();
            let mut task = task.try_lock().unwrap();
            task.ram.push_i64(0);
            args[2].push_to_stack(&mut task, &vm).unwrap();
            let result = auto_lang_core::vm::ffi::stdlib::shim_http_server_listen(&mut task, &vm);
            drop(task);
            vm.remove_task(id);
            match result {
                Ok(()) => println!("direct native returned=true"),
                Err(error) => {
                    eprintln!("direct native failed: {error:?}");
                    std::process::exit(1);
                }
            }
        });
        return;
    }
    match auto_lang_core::run_with_capture(&source) {
        Ok((value, _)) => println!("blocking returned={value:?}"),
        Err(error) => {
            eprintln!("blocking SDK failed: {error}");
            std::process::exit(1);
        }
    }
}
