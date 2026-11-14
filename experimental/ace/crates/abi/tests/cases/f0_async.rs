//! PLAN-967 T-08 (AC-06/V-08): the async completion PORTS driven by a
//! CONTROLLED REAL provider — a real cdylib whose entry completes on a
//! real OS thread through a C-ABI completion callback. The host registry
//! (av-core CompletionPorts, the frozen foreign-contract rule table)
//! publishes, cancels and drops late exactly as the contract freezes;
//! 968's language `await` will consume these ports (967 never depends on
//! 968). The V-08 export/resource/callback-pin/stale-generation rows on
//! the C side are the delivered 939 permanent fixtures (ce-*/callback-*/
//! handle-* decks); this file owns the port leg.
#![cfg(windows)]
use std::path::PathBuf;

use ace_abi::rust_provider::{build_raw_cdylib, f0_async_provider_source, RawLibrary};
use ace_av_core::foreign_exec::CompletionPorts;
use ace_foreign_contract::{CompletionDisposition, PortEvent, PortValue};

fn scope(tag: &str) -> PathBuf {
    let root = PathBuf::from("D:/autostack/plan-artifacts/plan-967/tests")
        .join(format!("{tag}-{}", std::process::id()));
    std::fs::create_dir_all(&root).unwrap();
    root
}

type DoneFn = extern "C" fn(ctx: *mut core::ffi::c_void, status: i32, value: i32);
type AsyncEntry = unsafe extern "C" fn(
    data: *const u8,
    len: u64,
    done: DoneFn,
    ctx: *mut core::ffi::c_void,
) -> i32;

/// The completion trampoline: the provider's OS thread re-enters the
/// host here; the ctx carries (registry, port) so the completion lands
/// in the port registry exactly once.
struct TrampolineCtx {
    registry: *const CompletionPorts,
    port: u64,
}

extern "C" fn trampoline(ctx: *mut core::ffi::c_void, status: i32, value: i32) {
    let ctx = unsafe { &*(ctx as *const TrampolineCtx) };
    let registry = unsafe { &*ctx.registry };
    let port_value = if status == 0 {
        PortValue::I32(value)
    } else {
        PortValue::Status(status)
    };
    registry.complete(ctx.port, port_value);
}

fn wait_for_event(ports: &CompletionPorts, needle: &str, deadline_ms: u128) -> Option<Vec<String>> {
    let started = std::time::Instant::now();
    let mut seen: Vec<String> = Vec::new();
    loop {
        let events = ports.take_events();
        let spells: Vec<String> = events
            .iter()
            .map(|e| match e {
                PortEvent::Registered { port, .. } => format!("reg:{port}"),
                PortEvent::Published { port, value } => format!("pub:{port}:{}", value.spell()),
                PortEvent::Cancelled { port } => format!("cancel:{port}"),
                PortEvent::LateDropped { port, value } => format!("late:{port}:{}", value.spell()),
                PortEvent::Released { port } => format!("rel:{port}"),
            })
            .collect();
        if spells.iter().any(|s| s == needle) {
            seen.extend(spells);
            return Some(seen);
        }
        seen.extend(spells);
        if started.elapsed().as_millis() > deadline_ms {
            return None;
        }
        std::thread::sleep(std::time::Duration::from_millis(5));
    }
}

#[test]
fn f0_async_ports_publish_with_a_real_threaded_provider() {
    let root = scope("async");
    let dll = build_raw_cdylib(&root, f0_async_provider_source(), "f0_async_provider")
        .expect("the controlled async provider builds");
    let library = RawLibrary::load(&dll).expect("loads");
    let entry: AsyncEntry =
        unsafe { std::mem::transmute(library.symbol("f0_async_double").expect("export present")) };

    // publish: register -> real off-thread completion -> published+released
    let mut ports = CompletionPorts::new(1);
    let port = ports.register("f0.async.double");
    let ctx = TrampolineCtx {
        registry: &ports,
        port,
    };
    let input = b"hello";
    let rc = unsafe {
        entry(
            input.as_ptr(),
            input.len() as u64,
            trampoline,
            &ctx as *const _ as *mut _,
        )
    };
    assert_eq!(rc, 0, "the provider accepted the call");
    let spells = wait_for_event(&ports, &format!("pub:{port}:i32:10"), 5_000)
        .expect("the real thread completed through the C callback");
    assert!(spells.contains(&format!("reg:{port}")));
    assert!(
        spells.contains(&format!("rel:{port}")),
        "the pin released once"
    );
}

#[test]
fn f0_async_cancel_and_teardown_drop_late_completions_from_the_real_provider() {
    let root = scope("async-cancel");
    let dll =
        build_raw_cdylib(&root, f0_async_provider_source(), "f0_async_provider").expect("build");
    let library = RawLibrary::load(&dll).expect("load");
    let entry: AsyncEntry =
        unsafe { std::mem::transmute(library.symbol("f0_async_double").expect("export")) };

    // cancel: the provider declines to complete on the literal input;
    // the host cancels and a LATE completion (forced through the same
    // trampoline afterwards) drops, never publishes.
    let mut ports = CompletionPorts::new(1);
    let port = ports.register("f0.async.double");
    let ctx = TrampolineCtx {
        registry: &ports,
        port,
    };
    let rc = unsafe {
        entry(
            b"cancel-me".as_ptr(),
            b"cancel-me".len() as u64,
            trampoline,
            &ctx as *const _ as *mut _,
        )
    };
    assert_eq!(rc, -1, "the controlled provider declined (no completion)");
    assert!(ports.cancel(port));
    // a racing late completion arrives AFTER the cancel
    trampoline(&ctx as *const _ as *mut core::ffi::c_void, 0, 42);
    let spells = wait_for_event(&ports, &format!("late:{port}:i32:42"), 1_000)
        .expect("the late completion was dropped observably");
    assert!(
        !spells.iter().any(|s| s.starts_with("pub:")),
        "nothing published"
    );

    // teardown: the session moves on while the real thread is in flight;
    // its completion drops late.
    let mut ports2 = CompletionPorts::new(1);
    let port2 = ports2.register("f0.async.double");
    let ctx2 = TrampolineCtx {
        registry: &ports2,
        port: port2,
    };
    let rc = unsafe { entry(b"abc".as_ptr(), 3, trampoline, &ctx2 as *const _ as *mut _) };
    assert_eq!(rc, 0);
    ports2.teardown();
    let spells = wait_for_event(&ports2, &format!("late:{port2}:i32:6"), 5_000)
        .expect("the in-flight completion observed the moved generation");
    assert!(!spells.iter().any(|s| s.starts_with("pub:")));
    // the unknown-port completion stays a named rejection end to end
    assert_eq!(
        ports2.complete(9999, PortValue::I32(1)),
        CompletionDisposition::UnknownPort
    );
}
