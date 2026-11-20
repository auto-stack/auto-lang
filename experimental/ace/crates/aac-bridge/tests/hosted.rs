//! Explicit compiler/linker integration lane. Prepare the runtime with
//! `cargo build -p ace-collections-runtime --profile runtime-dev` first.

#![cfg(all(windows, target_arch = "x86_64", target_env = "msvc"))]

mod support;

use ace_aac_bridge::{Client, Mode, Outcome};
use auto_tool_execution::{run_bounded, BoundedPolicy, ExecutionSpec, ExitKind, TreeScope};
use std::ffi::OsStr;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};
use support::{fixture, request, scalar_globals};

fn stage() -> tempfile::TempDir {
    match std::env::var_os("AUTO_PLAN_ARTIFACTS") {
        Some(root) => tempfile::tempdir_in(root).unwrap(),
        None => tempfile::tempdir().unwrap(),
    }
}

fn file_literal(path: &Path) -> String {
    format!(
        "{:?}",
        path.to_str()
            .expect("test stage UTF-8 path")
            .replace('\\', "/")
    )
}

#[test]
fn gen0_native_auto_client_executes_ct_and_releases_its_exchange() {
    let started = Instant::now();
    let runtime = PathBuf::from(
        std::env::var_os("ACE_AAC_COLLECTIONS_LIB").expect("prepare ACE_AAC_COLLECTIONS_LIB"),
    );
    let runtime_identity = ace_native::collections_runtime::file_hash(&runtime).unwrap();
    let stage = stage();
    let request_path = stage.path().join("输入 request.bin");
    let response_path = stage.path().join("输出 response.bin");
    let bridge = Path::new(env!("CARGO_BIN_EXE_ace-aac-bridge"));
    let mir = fixture(false);
    let abc = ace_av_core::encode_abc(&mir).unwrap().to_flat();
    let mut request = request(Mode::CompileTime, &mir, 1);
    request.options.remaining_ms = 5000;
    request.inputs.source = *blake3::hash(&abc).as_bytes();
    let metadata = serde_json::to_string(&request).unwrap();
    let mut main = String::from(
        r#"
use app.client.exchange
use app.digest.utf8_bytes

fn main() int {
    let abc = List<int>.new()
"#,
    );
    for octet in abc {
        main.push_str(&format!("    abc.push({octet})\n"));
    }
    main.push_str(&format!(
        "    let response = exchange({}, {}, {}, {metadata:?}, abc, 5000, 65536)\n",
        file_literal(bridge),
        file_literal(&request_path),
        file_literal(&response_path)
    ));
    main.push_str(
        r#"
    if (response.status != 0) { return response.status }
    let value = utf8_bytes("\"value\":{\"i32\":7}")
    let bytes = response.frame
    var i = 44
    for (i <= bytes.len() - value.len()) {
        var matched = true
        var j = 0
        for (j < value.len()) {
            if (bytes.get(i + j) != value.get(j)) { matched = false }
            j = j + 1
        }
        if (matched) { return 7 }
        i = i + 1
    }
    return -6
}
"#,
    );
    let modules = vec![
        (
            "digest.at".into(),
            ace_aac_host_conformance::DIGEST_SOURCE.into(),
        ),
        (
            "client.at".into(),
            include_str!("../../../aac/host/bridge/client.at").into(),
        ),
        ("main.at".into(), main),
    ];
    let native = ace_aac_host_conformance::sys_product_backend(
        "aac-native-bridge",
        &modules,
        ace_core::source::packages::AssemblyBackend::Ac,
    )
    .unwrap();
    let product = ace_native::driver::MnDriver::ephemeral()
        .build_hosted_collections_checked(
            &native.compiled.checked,
            "d_main",
            &stage.path().join("native"),
            &runtime,
            &runtime_identity,
            1051,
        )
        .unwrap();
    let mut spec = ExecutionSpec::new([OsStr::new(&product.exe_path)]);
    spec.tree_scope = TreeScope::ProcessTree;
    let observed = run_bounded(
        &spec,
        &BoundedPolicy {
            exec_deadline: Duration::from_secs(6),
            cleanup_deadline: Duration::from_secs(1),
            stdout_cap: Some(64 * 1024),
            stderr_cap: Some(128 * 1024),
        },
    )
    .unwrap();
    let stderr = String::from_utf8_lossy(observed.stderr.as_deref().unwrap_or_default());
    assert_eq!(observed.kind, ExitKind::Failure(7), "{stderr}");
    assert!(
        stderr.lines().any(|line| line == "MN-COLL live=0"),
        "{stderr}"
    );
    assert!(!request_path.exists() && !response_path.exists());
    stage.close().unwrap();
    eprintln!(
        "gen0_native_auto_client_executes_ct_and_releases_its_exchange {:?}",
        started.elapsed()
    );
}

#[test]
fn native_scalar_global_objects_link_and_run_initializers() {
    let started = Instant::now();
    let (mut request, mir) = scalar_globals(Mode::NativeObjects);
    let abc = ace_av_core::encode_abc(&mir).unwrap().to_flat();
    let bridge = Client::new(Path::new(env!("CARGO_BIN_EXE_ace-aac-bridge"))).unwrap();
    let (response, bytes) = bridge.run(&request, &abc).unwrap();
    let Outcome::Objects {
        units,
        wrapper,
        text_runtime,
        ..
    } = response.outcome
    else {
        panic!("not a linked-program object bundle: {:?}", response.outcome);
    };
    assert!(!text_runtime);
    assert_eq!(units.len(), 2);
    let mut remaining = bytes.as_slice();
    let mut objects = Vec::new();
    for object in units {
        let (unit, rest) = remaining.split_at(object.len as usize);
        objects.push(ace_native::link_driver::ObjectInput {
            name: object.symbol,
            bytes: unit.to_vec(),
        });
        remaining = rest;
    }
    assert_eq!(remaining.len(), wrapper.len as usize);
    let stage = stage();
    let image = ace_native::link_driver::link_only(
        &objects,
        remaining,
        &wrapper.symbol,
        stage.path(),
        &[],
        "aac-scalar-global",
    )
    .unwrap();
    let mut spec = ExecutionSpec::new([image.exe_path.as_os_str()]);
    spec.tree_scope = TreeScope::ProcessTree;
    let result = run_bounded(
        &spec,
        &BoundedPolicy {
            exec_deadline: Duration::from_secs(3),
            cleanup_deadline: Duration::from_secs(1),
            stdout_cap: Some(4096),
            stderr_cap: Some(4096),
        },
    )
    .unwrap();
    assert_eq!(result.kind, ExitKind::Failure(7));
    assert!(result.stderr.unwrap().is_empty());

    request.id = [43; 16];
    request.globals.clear();
    let (response, bytes) = bridge.run(&request, &abc).unwrap();
    assert!(
        matches!(response.outcome, Outcome::Failure { error } if error.code == "global.reference")
    );
    assert!(bytes.is_empty());
    stage.close().unwrap();
    eprintln!(
        "native_scalar_global_objects_link_and_run_initializers {:?}",
        started.elapsed()
    );
}

#[test]
fn scheduler_and_win32_page_probe_run_in_one_native_product() {
    use ace_av_core::mir::{
        FnSig, MirHostImport, MirInstr, MirSlot, MirType, SlotKind, Terminator,
    };
    use ace_av_core::platform_io::{lookup_symbol, PlatformIoBackend, Win32PlatformIo};

    let started = Instant::now();
    let runtime =
        PathBuf::from(std::env::var_os("ACE_AAC_SCHED_LIB").expect("prepare ACE_AAC_SCHED_LIB"));
    let runtime_identity = ace_native::sched_runtime::file_hash(&runtime).unwrap();
    let source = r#"
fn worker(n int) {
    print("go:")
    print(n)
}
fn main() int {
    go worker(7)
    return 0
}
"#;
    let mut module = ace_native::sched::sched_compile(source, "d_main").unwrap();
    let platform = lookup_symbol("auto.platform.mem.page_size.v1").unwrap();
    let scalar = module.intern_type(MirType::I32);
    let signature = module.intern_type(MirType::Fn(FnSig {
        params: Vec::new(),
        result: scalar,
    }));
    let host = module.host_imports.len() as u32;
    module.host_imports.push(MirHostImport {
        category: platform.category.into(),
        symbol: platform.symbol.into(),
        sig: signature,
        capability: platform.capability.into(),
        max_call_bytes: platform.max_call_bytes,
    });
    module.capabilities.push(platform.capability.into());
    let entry = &mut module.funcs[module.entry as usize];
    let slot = entry.slots.len() as u32;
    entry.slots.push(MirSlot {
        kind: SlotKind::Temp,
        ty: scalar,
        name: None,
    });
    for block in &mut entry.blocks {
        if matches!(block.term, Terminator::Return { .. }) {
            block.instrs.push(MirInstr::PlatformCall {
                dst: slot,
                host,
                args: vec![],
            });
            block.term = Terminator::Return { value: slot };
        }
    }

    let stage = stage();
    let image = ace_native::sched::sched_native_build_prepared(
        &module,
        "d_main",
        stage.path(),
        &runtime,
        &runtime_identity,
    )
    .unwrap();
    let mut spec = ExecutionSpec::new([image.exe_path.as_os_str()]);
    spec.tree_scope = TreeScope::ProcessTree;
    let observed = run_bounded(
        &spec,
        &BoundedPolicy {
            exec_deadline: Duration::from_secs(4),
            cleanup_deadline: Duration::from_secs(1),
            stdout_cap: Some(4096),
            stderr_cap: Some(4096),
        },
    )
    .unwrap();
    let (page, _) = Win32PlatformIo::default().system_info();
    assert!(page > 0);
    assert_eq!(observed.kind, ExitKind::Failure(page as i32));
    assert_eq!(observed.stdout.as_deref(), Some(&b"go:\n7\n"[..]));
    assert_eq!(observed.stderr.as_deref(), Some(&b""[..]));
    stage.close().unwrap();
    eprintln!(
        "scheduler_and_win32_page_probe_run_in_one_native_product {:?}",
        started.elapsed()
    );
}
