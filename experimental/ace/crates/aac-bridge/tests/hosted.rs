//! Explicit compiler/linker integration lane. Prepare the runtime with
//! `cargo build -p ace-collections-runtime --profile runtime-dev` first.

#![cfg(all(windows, target_arch = "x86_64", target_env = "msvc"))]

use ace_aac_bridge::{Domains, InputIdentity, Mode, Options, Request};
use ace_av_core::contract::MIR_SCHEMA;
use ace_av_core::mir::*;
use auto_tool_execution::{run_bounded, BoundedPolicy, ExecutionSpec, ExitKind, TreeScope};
use std::ffi::OsStr;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

fn input() -> (Request, Vec<u8>) {
    let owner = OwnerScope {
        schema: MIR_SCHEMA.into(),
        module: "aac-hosted".into(),
    };
    let mut mir = MirModule::new(owner.clone());
    mir.types = vec![
        MirType::I32,
        MirType::Fn(FnSig {
            params: Vec::new(),
            result: 0,
        }),
    ];
    mir.consts = vec![MirConst::I32(7)];
    mir.funcs = vec![MirFunction {
        owner: owner.clone(),
        name: "entry".into(),
        sig: 1,
        slots: vec![MirSlot {
            kind: SlotKind::Temp,
            ty: 0,
            name: None,
        }],
        blocks: vec![MirBlock {
            instrs: vec![MirInstr::Const { dst: 0, value: 0 }],
            term: Terminator::Return { value: 0 },
        }],
        entry: 0,
        source_snapshot: 0,
        spans: Vec::new(),
        env: None,
    }];
    let bytes = ace_av_core::encode_abc(&mir).unwrap().to_flat();
    let request = Request {
        id: [1; 16],
        mode: Mode::CompileTime,
        domains: Domains::windows(Mode::CompileTime),
        inputs: InputIdentity {
            source: *blake3::hash(&bytes).as_bytes(),
            grammar: [2; 32],
            passes: [3; 32],
            stdlib: [4; 32],
            dependencies: [5; 32],
        },
        owner: owner.module,
        mir_schema: owner.schema,
        options: Options { remaining_ms: 5000 },
        capabilities: vec![],
        globals: vec![],
        initializers: vec![],
    };
    (request, bytes)
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
    let stage = match std::env::var_os("AUTO_PLAN_ARTIFACTS") {
        Some(root) => tempfile::tempdir_in(root).unwrap(),
        None => tempfile::tempdir().unwrap(),
    };
    let request_path = stage.path().join("输入 request.bin");
    let response_path = stage.path().join("输出 response.bin");
    let bridge = Path::new(env!("CARGO_BIN_EXE_ace-aac-bridge"));
    let (request, abc) = input();
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
