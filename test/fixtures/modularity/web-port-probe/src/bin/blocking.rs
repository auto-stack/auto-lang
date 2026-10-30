//! Borrowed VM owner test. No VM, task or AST is sent to the controller thread.
use auto_lang_core::vm::host_capabilities::http_service::{
    messages::HttpProgramMetadata, BlockingListenerKind, HttpServiceFuture, HttpServiceHost,
    HttpVmOwner,
};
use std::io::BufRead;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};
static STOP: AtomicBool = AtomicBool::new(false);
// Inspect actual retained credentials, distinguishing live ownership from the
// VM's bounded RC grace queue. This is a probe over existing public SDK data.
fn retained_credentials(vm: &auto_lang_core::vm::engine::AutoVM) -> Vec<(u64, Option<u32>)> {
    vm.heap_objects
        .iter()
        .filter_map(|entry| {
            let object = entry.value().read().unwrap();
            let resource = object
                .as_any()
                .downcast_ref::<auto_lang_core::vm::ffi::http_return_resource::HttpReturnResource>(
            )?;
            if vm.rc_count(*entry.key()) == 0 {
                return None;
            }
            assert_eq!(resource.owner, vm.instance_id);
            let iterator = match resource.kind {
                auto_lang_core::vm::ffi::http_return_resource::HttpReturnKind::Sse {
                    iterator_id,
                } => Some(iterator_id),
                _ => None,
            };
            Some((*entry.key(), iterator))
        })
        .collect()
}

struct Adapter {
    address: String,
    legacy: bool,
    swap_legacy_grade: bool,
}
impl HttpServiceHost for Adapter {
    fn serve(&self, owner: HttpVmOwner) -> HttpServiceFuture {
        eprintln!("blocking adapter async_activation=true");
        let address = self.address.clone();
        let legacy = self.legacy;
        Box::pin(async move {
            let (vm, _) = owner.into_local_parts();
            if legacy {
                // Exercise the actual public compatibility SDK entry, not a
                // private runtime call or hand-built VM response.
                // Produce a real lazy generator through the existing VM SDK.
                // Its live iterator key intentionally collides with JSON zero.
                let seed_task_id = vm.spawn_task(0, 8192);
                let seed_task = vm.tasks.get(&seed_task_id).unwrap().value().clone();
                let mut seed_task = seed_task.try_lock().unwrap();
                vm.call_fn_by_name_http(&mut seed_task, "collision_seed", 0)
                    .map_err(|error| auto_lang_core::error::AutoError::Msg(format!("{error:?}")))?;
                let value = seed_task.ram.pop_nv();
                let slot = seed_task.ram.sp;
                seed_task.ram.raw_nv[slot] = 0;
                let stake = auto_lang_core::vm::native::StakeGuard::nv(&vm, value);
                let Some(auto_lang_core::vm::ffi::http_return_resource::ResourceClaim::Sse {
                    iterator_id,
                }) = auto_lang_core::vm::ffi::http_return_resource::resolve_resource(&vm, value)
                else {
                    panic!("real seed did not produce its Sse credential");
                };
                assert_eq!(
                    iterator_id, 0,
                    "collision must use the actual first live key"
                );
                drop(seed_task);
                vm.remove_task(seed_task_id);
                auto_lang_core::vm::ffi::http_server::serve_blocking_stdnet(&vm, &address);
                let (_, seed) = vm
                    .iterators
                    .remove(&iterator_id)
                    .expect("ordinary integer must leave caller iterator alive");
                match seed {
                    auto_lang_core::vm::engine::Iterator::Generator(state) => {
                        assert!(
                            !state.started && state.task_id.is_none(),
                            "ordinary integer must not pull caller generator"
                        );
                    }
                    _ => panic!("seed identity changed"),
                }
                drop(stake);
                assert!(
                    retained_credentials(&vm).is_empty(),
                    "caller and request credential ownership released before VM drop"
                );
                eprintln!("legacy collision actual_key=0 caller_generator_untouched=true");
                Ok(())
            } else {
                auto_lang_core::vm::host_capabilities::http_service::listen_registered(
                    &vm, &address,
                )
            }
        })
    }
    fn listen_blocking(
        &self,
        vm: &auto_lang_core::vm::engine::AutoVM,
        address: &str,
        metadata: Arc<HttpProgramMetadata>,
        kind: BlockingListenerKind,
    ) -> auto_lang_core::AutoResult<()> {
        // Fixture-only wrong-grade mutation: the wire oracle must reject it.
        let effective_kind = if self.swap_legacy_grade && kind == BlockingListenerKind::LegacyStdnet
        {
            BlockingListenerKind::ServerHandle
        } else {
            kind
        };
        #[cfg(feature = "external-runtime")]
        {
            eprintln!(
                "blocking runtime=external owner={:?}",
                std::thread::current().id()
            );
            let owner_thread = std::thread::current().id();
            let tasks_before = vm.tasks.len();
            let iterators_before = vm.iterators.len();
            let result = web_runtime_boundary_probe::listen_borrowed(
                vm,
                address,
                metadata,
                effective_kind,
                &STOP,
            );
            assert_eq!(std::thread::current().id(), owner_thread);
            assert_eq!(
                vm.tasks.len(),
                tasks_before,
                "all fresh handler tasks removed"
            );
            if kind == BlockingListenerKind::LegacyStdnet {
                assert_eq!(
                    vm.iterators.len(),
                    iterators_before,
                    "legacy iterator reclaimed before VM drop"
                );
                assert_eq!(
                    auto_lang_core::vm::ffi::http_server_file::vm_file_response_count(),
                    0
                );
                let retained = retained_credentials(vm);
                assert_eq!(
                    retained.len(),
                    1,
                    "only caller-owned seed remains; no retained request credentials"
                );
                assert_eq!(
                    retained[0].1,
                    Some(0),
                    "remaining credential is the unrelated seed"
                );
                eprintln!("legacy terminal iterators_delta=0 file_descriptors=0 request_credential_owners=0");
            }
            eprintln!("blocking terminal handler_tasks_delta=0 owner_thread_preserved=true");
            result
        }
        #[cfg(not(feature = "external-runtime"))]
        {
            let _ = (vm, address, metadata, kind, effective_kind);
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
            Arc::new(Adapter { address: address.clone(), legacy: args.iter().any(|arg| arg == "--legacy"), swap_legacy_grade: args.iter().any(|arg| arg == "--swap-legacy-grade") }));
    }
    if args.iter().any(|arg| arg == "--async-only") {
        auto_lang_core::vm::host_capabilities::http_service::set_process_default_http_service_host(
            Arc::new(AsyncOnly),
        );
    }
    let controller = std::thread::spawn(move || {
        for line in std::io::stdin().lock().lines() {
            if matches!(line.as_deref(), Ok("shutdown")) {
                STOP.store(true, Ordering::Release);
                let wake = std::net::TcpStream::connect(&address).is_ok();
                eprintln!("blocking shutdown wake={wake}");
                break;
            }
        }
    });
    if args.iter().any(|arg| arg == "--direct-legacy") {
        let vm = auto_lang_core::vm::engine::AutoVM::new_without_host_snapshot(
            auto_lang_core::vm::virt_memory::VirtualFlash::new(0),
            8192,
        );
        auto_lang_core::vm::ffi::http_server::serve_blocking_stdnet(&vm, &args[2]);
        controller
            .join()
            .expect("owned compatibility controller joined");
        println!("legacy compatibility returned=unit");
        return;
    }
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
        controller.join().expect("owned shutdown controller joined");
        eprintln!("blocking controller_joined=true");
        return;
    }
    match auto_lang_core::run_with_capture(&source) {
        Ok((value, _)) => {
            if args.iter().any(|arg| arg == "--legacy") {
                assert_eq!(
                    auto_lang_core::vm::ffi::http_return_resource::live_resource_count(),
                    0
                );
                assert_eq!(
                    auto_lang_core::vm::ffi::http_upload::vm_upload_counts(),
                    (0, 0, 0)
                );
                eprintln!("legacy owner returned credentials=0 uploads=0");
            }
            println!("blocking returned={value:?}");
        }
        Err(error) => {
            eprintln!("blocking SDK failed: {error}");
            std::process::exit(1);
        }
    }
    controller.join().expect("owned shutdown controller joined");
    eprintln!("blocking controller_joined=true");
}
