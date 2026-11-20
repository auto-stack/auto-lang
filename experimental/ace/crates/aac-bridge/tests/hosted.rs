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
        runtime,
        ..
    } = response.outcome
    else {
        panic!("not a linked-program object bundle: {:?}", response.outcome);
    };
    assert_eq!(runtime, ace_aac_bridge::NativeRuntime::None);
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
    let observed = native_object_image(&module, 92, Mode::NativeScheduler, &runtime, stage.path());
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

fn native_object_image(
    mir: &ace_av_core::MirModule,
    id: u8,
    mode: Mode,
    library: &Path,
    out: &Path,
) -> auto_tool_execution::BoundedResult {
    let bridge = Client::new(Path::new(env!("CARGO_BIN_EXE_ace-aac-bridge"))).unwrap();
    let native = request(mode, mir, id);
    let abc = ace_av_core::encode_abc(mir).unwrap().to_flat();
    let (response, data) = bridge.run(&native, &abc).unwrap();
    let Outcome::Objects {
        units,
        wrapper,
        runtime,
        ..
    } = response.outcome
    else {
        panic!("expected collections objects: {:?}", response.outcome);
    };
    let expected_runtime = match mode {
        Mode::NativeCollections => ace_aac_bridge::NativeRuntime::Collections,
        Mode::NativeScheduler => ace_aac_bridge::NativeRuntime::Scheduler,
        _ => panic!("expected a runtime-bound native mode"),
    };
    assert_eq!(runtime, expected_runtime);
    let mut remaining = data.as_slice();
    let mut objects = Vec::new();
    for unit in units {
        let (bytes, next) = remaining.split_at(unit.len as usize);
        objects.push(ace_native::link_driver::ObjectInput {
            name: unit.symbol,
            bytes: bytes.to_vec(),
        });
        remaining = next;
    }
    let linked = ace_native::link_driver::link_only(
        &objects,
        remaining,
        &wrapper.symbol,
        out,
        &[library.to_path_buf()],
        "aac-collections",
    )
    .unwrap();
    let mut spec = ExecutionSpec::new([linked.exe_path.as_os_str()]);
    spec.tree_scope = TreeScope::ProcessTree;
    run_bounded(
        &spec,
        &BoundedPolicy {
            exec_deadline: Duration::from_secs(3),
            cleanup_deadline: Duration::from_secs(1),
            stdout_cap: Some(4096),
            stderr_cap: Some(64 * 1024),
        },
    )
    .unwrap()
}

#[test]
fn dynamic_box_distinguishes_equal_width_types_and_releases_values() {
    use ace_av_core::mir::{MirBlock, MirConst, MirInstr, MirSlot, MirType, SlotKind, Terminator};

    let started = Instant::now();
    let library = PathBuf::from(
        std::env::var_os("ACE_AAC_COLLECTIONS_LIB").expect("prepare ACE_AAC_COLLECTIONS_LIB"),
    );
    let mut mir = fixture(false);
    mir.owner.schema = ace_av_core::contract::MIR_DYNAMIC_SCHEMA.into();
    mir.profile = ace_av_core::contract::AV_DYNAMIC_PROFILE.into();
    mir.funcs[0].owner = mir.owner.clone();
    let tagged = mir.intern_type(MirType::Dyn);
    let signed = mir.intern_type(MirType::I64);
    let unsigned = mir.intern_type(MirType::U64);
    let boolean = mir.intern_type(MirType::Bool);
    mir.consts = vec![MirConst::I64(7), MirConst::I32(-1)];
    let function = &mut mir.funcs[0];
    function.slots = [0, signed, tagged, boolean, signed]
        .into_iter()
        .map(|ty| MirSlot {
            kind: SlotKind::Temp,
            ty,
            name: None,
        })
        .collect();
    function.blocks = vec![
        MirBlock {
            instrs: vec![
                MirInstr::Const { dst: 1, value: 0 },
                MirInstr::DynPack { dst: 2, src: 1 },
                MirInstr::DynIs {
                    dst: 3,
                    src: 2,
                    ty: unsigned,
                },
            ],
            term: Terminator::Branch {
                cond: 3,
                then: 2,
                else_: 1,
            },
        },
        MirBlock {
            instrs: vec![
                MirInstr::DynTake {
                    dst: 4,
                    src: 2,
                    ty: signed,
                },
                MirInstr::Conv { dst: 0, src: 4 },
            ],
            term: Terminator::Return { value: 0 },
        },
        MirBlock {
            instrs: vec![MirInstr::Const { dst: 0, value: 1 }],
            term: Terminator::Return { value: 0 },
        },
    ];

    let stage = stage();
    for (index, wrong_type) in [false, true].into_iter().enumerate() {
        if wrong_type {
            mir.funcs[0].slots[4].ty = unsigned;
            mir.funcs[0].blocks[1].instrs[0] = MirInstr::DynTake {
                dst: 4,
                src: 2,
                ty: unsigned,
            };
        }
        let run = native_object_image(
            &mir,
            50 + index as u8,
            Mode::NativeCollections,
            &library,
            &stage.path().join(format!("case-{index}")),
        );
        let stderr = String::from_utf8_lossy(run.stderr.as_deref().unwrap_or_default());
        assert_eq!(
            run.kind,
            ExitKind::Failure(if wrong_type { 70 } else { 7 }),
            "{stderr}"
        );
        assert!(
            stderr.lines().any(|line| line == "MN-COLL live=0"),
            "{stderr}"
        );
        assert_eq!(
            stderr.lines().any(|line| line == "MN-COLL-TRAP code=30"),
            wrong_type,
            "{stderr}"
        );
    }
    stage.close().unwrap();
    eprintln!(
        "dynamic_box_distinguishes_equal_width_types_and_releases_values {:?}",
        started.elapsed()
    );
}

#[test]
fn native_object_cells_keep_key_tags_and_share_updates() {
    use ace_av_core::mir::{BinOp, MirConst, MirInstr, MirSlot, MirType, SlotKind, Terminator};

    let started = Instant::now();
    let library = PathBuf::from(
        std::env::var_os("ACE_AAC_COLLECTIONS_LIB").expect("prepare ACE_AAC_COLLECTIONS_LIB"),
    );
    let mut mir = fixture(false);
    mir.owner.schema = ace_av_core::contract::MIR_CLOSURES_SCHEMA.into();
    mir.profile = ace_av_core::contract::AV_CLOSURES_PROFILE.into();
    mir.capabilities
        .push(ace_av_core::contract::LANGUAGE_VALUES_CAPABILITY.into());
    mir.funcs[0].owner = mir.owner.clone();
    let boolean = mir.intern_type(MirType::Bool);
    let object = mir.intern_type(MirType::Object { elem: 0 });
    mir.consts = vec![
        MirConst::I32(1),
        MirConst::Bool(true),
        MirConst::I32(7),
        MirConst::I32(4),
        MirConst::I32(5),
        MirConst::I32(9),
    ];
    let f = &mut mir.funcs[0];
    f.slots = [0, boolean, 0, 0, 0, object, object, 0, 0, 0, 0]
        .into_iter()
        .map(|ty| MirSlot {
            kind: SlotKind::Temp,
            ty,
            name: None,
        })
        .collect();
    f.blocks[0].instrs = vec![
        MirInstr::Const { dst: 0, value: 0 },
        MirInstr::Const { dst: 1, value: 1 },
        MirInstr::Const { dst: 2, value: 2 },
        MirInstr::Const { dst: 3, value: 3 },
        MirInstr::Const { dst: 4, value: 4 },
        MirInstr::ObjectNew {
            dst: 5,
            keys: vec![0, 1],
            vals: vec![3, 4],
        },
        MirInstr::Copy { dst: 6, src: 5 },
        MirInstr::ObjectSet {
            obj: 6,
            key: 0,
            value: 2,
        },
        MirInstr::ObjectGet {
            dst: 7,
            obj: 5,
            key: 0,
            elem: 0,
        },
        MirInstr::ObjectGet {
            dst: 8,
            obj: 5,
            key: 1,
            elem: 0,
        },
        MirInstr::Bin {
            op: BinOp::AddI32,
            dst: 9,
            lhs: 7,
            rhs: 8,
        },
        MirInstr::Const { dst: 10, value: 5 },
    ];
    f.blocks[0].term = Terminator::Return { value: 9 };
    let stage = stage();
    for (index, expected) in [(0, 12), (1, 70), (2, 70)] {
        if index == 1 {
            let f = &mut mir.funcs[0];
            f.blocks[0].instrs.swap(8, 11);
            f.blocks[0].instrs[11] = MirInstr::ObjectGet {
                dst: 7,
                obj: 5,
                key: 10,
                elem: 0,
            };
            f.blocks[0].term = Terminator::Return { value: 7 };
        } else if index == 2 {
            mir.funcs[0].blocks[0].instrs[5] = MirInstr::ObjectNew {
                dst: 5,
                keys: vec![0, 0],
                vals: vec![3, 4],
            };
        }
        let run = native_object_image(
            &mir,
            60 + index as u8,
            Mode::NativeCollections,
            &library,
            &stage.path().join(format!("case-{index}")),
        );
        let stderr = String::from_utf8_lossy(run.stderr.as_deref().unwrap_or_default());
        assert_eq!(run.kind, ExitKind::Failure(expected), "{stderr}");
        assert!(
            stderr.lines().any(|line| line == "MN-COLL live=0"),
            "{stderr}"
        );
        let code = if index == 1 {
            "MN-COLL-TRAP code=31"
        } else {
            "MN-COLL-TRAP code=33"
        };
        assert_eq!(
            stderr.lines().any(|line| line == code),
            index != 0,
            "{stderr}"
        );
    }
    stage.close().unwrap();
    eprintln!(
        "native_object_cells_keep_key_tags_and_share_updates {:?}",
        started.elapsed()
    );
}

#[test]
fn native_object_cells_reject_transitive_cycles_and_reclaim_the_graph() {
    use ace_av_core::mir::{MirConst, MirInstr, MirSlot, MirType, SlotKind, Terminator};

    let started = Instant::now();
    let library = PathBuf::from(
        std::env::var_os("ACE_AAC_COLLECTIONS_LIB").expect("prepare ACE_AAC_COLLECTIONS_LIB"),
    );
    let mut mir = fixture(false);
    mir.owner.schema = ace_av_core::contract::MIR_CLOSURES_SCHEMA.into();
    mir.profile = ace_av_core::contract::AV_CLOSURES_PROFILE.into();
    mir.capabilities
        .push(ace_av_core::contract::LANGUAGE_VALUES_CAPABILITY.into());
    mir.funcs[0].owner = mir.owner.clone();
    let dynamic = mir.intern_type(MirType::Dyn);
    let object = mir.intern_type(MirType::Object { elem: dynamic });
    mir.consts = vec![MirConst::I32(7)];
    let f = &mut mir.funcs[0];
    f.slots = [0, object, object, dynamic, dynamic]
        .into_iter()
        .map(|ty| MirSlot {
            kind: SlotKind::Temp,
            ty,
            name: None,
        })
        .collect();
    f.blocks[0].instrs = vec![
        MirInstr::Const { dst: 0, value: 0 },
        MirInstr::ObjectNew {
            dst: 1,
            keys: vec![],
            vals: vec![],
        },
        MirInstr::ObjectNew {
            dst: 2,
            keys: vec![],
            vals: vec![],
        },
        MirInstr::DynPack { dst: 3, src: 1 },
        MirInstr::DynPack { dst: 4, src: 2 },
        MirInstr::ObjectSet {
            obj: 1,
            key: 0,
            value: 4,
        },
    ];
    f.blocks[0].term = Terminator::Return { value: 0 };
    let stage = stage();
    for (index, expected) in [(0, 7), (1, 70)] {
        if index == 1 {
            mir.funcs[0].blocks[0].instrs.push(MirInstr::ObjectSet {
                obj: 2,
                key: 0,
                value: 3,
            });
        }
        let run = native_object_image(
            &mir,
            70 + index as u8,
            Mode::NativeCollections,
            &library,
            &stage.path().join(format!("case-{index}")),
        );
        let stderr = String::from_utf8_lossy(run.stderr.as_deref().unwrap_or_default());
        assert_eq!(run.kind, ExitKind::Failure(expected), "{stderr}");
        assert!(
            stderr.lines().any(|line| line == "MN-COLL live=0"),
            "{stderr}"
        );
        assert_eq!(
            stderr.lines().any(|line| line == "MN-COLL-TRAP code=32"),
            index == 1,
            "{stderr}"
        );
    }
    stage.close().unwrap();
    eprintln!(
        "native_object_cells_reject_transitive_cycles_and_reclaim_the_graph {:?}",
        started.elapsed()
    );
}

#[test]
fn native_bridge_runs_canonical_object_source_cases() {
    let started = Instant::now();
    let library = PathBuf::from(
        std::env::var_os("ACE_AAC_COLLECTIONS_LIB").expect("prepare ACE_AAC_COLLECTIONS_LIB"),
    );
    let cases = include_str!(
        "../../../../../crates/auto-lang/test/cases/language/dynamic/objects-null.case.ad"
    );
    let stage = stage();
    for (index, name) in ["bool-and-int-keys", "nested-object-alias"]
        .into_iter()
        .enumerate()
    {
        let snippet = cases.split_once(&format!("## Case: {name}\n")).unwrap().1;
        let source = snippet
            .split_once("```auto\n")
            .unwrap()
            .1
            .split_once("```")
            .unwrap()
            .0;
        let caps = [
            "ac.source.language.av/v1".into(),
            "console.stdout".into(),
            "console.stderr".into(),
        ];
        let checked = ace_core::source::compile_language_source(source, "main.at", &caps)
            .expect("compile canonical source via Gen0")
            .checked;
        let mir = ace_lowering::lower_checked_auto_language(&checked, "d_main", 1051)
            .expect("lower canonical source via Gen0");
        let run = native_object_image(
            &mir,
            80 + index as u8,
            Mode::NativeCollections,
            &library,
            &stage.path().join(name),
        );
        let stderr = String::from_utf8_lossy(run.stderr.as_deref().unwrap_or_default());
        assert_eq!(run.kind, ExitKind::Failure(42), "{name}: {stderr}");
        assert!(
            stderr.lines().any(|line| line == "MN-COLL live=0"),
            "{name}: {stderr}"
        );
    }
    stage.close().unwrap();
    eprintln!(
        "native_bridge_runs_canonical_object_source_cases {:?}",
        started.elapsed()
    );
}

#[test]
fn native_dynamic_list_cannot_create_an_owner_cycle() {
    use ace_av_core::mir::{ListTypeRef, MirConst, MirInstr, MirSlot, MirType, SlotKind};

    let started = Instant::now();
    let library = PathBuf::from(
        std::env::var_os("ACE_AAC_COLLECTIONS_LIB").expect("prepare ACE_AAC_COLLECTIONS_LIB"),
    );
    let mut mir = fixture(false);
    mir.owner.schema = ace_av_core::contract::MIR_CLOSURES_SCHEMA.into();
    mir.profile = ace_av_core::contract::AV_CLOSURES_PROFILE.into();
    mir.capabilities
        .push(ace_av_core::contract::LANGUAGE_VALUES_CAPABILITY.into());
    mir.capabilities.push("av.aggregates/1".into());
    mir.funcs[0].owner = mir.owner.clone();
    let dynamic = mir.intern_type(MirType::Dyn);
    let list = mir.intern_type(MirType::List(ListTypeRef { elem: dynamic }));
    mir.consts = vec![MirConst::I32(7)];
    let f = &mut mir.funcs[0];
    f.slots = [0, list, dynamic]
        .into_iter()
        .map(|ty| MirSlot {
            kind: SlotKind::Temp,
            ty,
            name: None,
        })
        .collect();
    f.blocks[0].instrs = vec![
        MirInstr::Const { dst: 0, value: 0 },
        MirInstr::ListNew {
            dst: 1,
            list_ty: list,
            cap: 1,
        },
        MirInstr::DynPack { dst: 2, src: 1 },
        MirInstr::ListPush { root: 1, value: 2 },
    ];
    let stage = stage();
    let observed = native_object_image(&mir, 91, Mode::NativeCollections, &library, stage.path());
    let stderr = String::from_utf8_lossy(observed.stderr.as_deref().unwrap_or_default());
    assert_eq!(observed.kind, ExitKind::Failure(70), "{stderr}");
    assert!(
        stderr.lines().any(|line| line == "MN-COLL-TRAP code=32"),
        "{stderr}"
    );
    assert!(
        stderr.lines().any(|line| line == "MN-COLL live=0"),
        "{stderr}"
    );
    stage.close().unwrap();
    eprintln!(
        "native_dynamic_list_cannot_create_an_owner_cycle {:?}",
        started.elapsed()
    );
}

#[test]
fn ct_calls_a_prepared_967_provider_only_with_matching_host_authority() {
    let started = Instant::now();
    let scope = PathBuf::from(
        std::env::var_os("ACE_AAC_FOREIGN_FIXTURE").expect("prepare ACE_AAC_FOREIGN_FIXTURE"),
    );
    let config_path = scope.join("provider.json");
    let config_bytes = std::fs::read(&config_path).unwrap();
    let config: ace_aac_bridge::ForeignConfig = serde_json::from_slice(&config_bytes).unwrap();
    let config_hash = *blake3::hash(&config_bytes).as_bytes();
    let mut request: ace_aac_bridge::Request =
        serde_json::from_slice(&std::fs::read(scope.join("request.json")).unwrap()).unwrap();
    let abc = std::fs::read(scope.join("module.abc")).unwrap();
    let bridge = Path::new(env!("CARGO_BIN_EXE_ace-aac-bridge"));
    let default = Client::new(bridge).unwrap();
    let (refused, _) = default.run(&request, &abc).unwrap();
    assert!(
        matches!(refused.outcome, Outcome::Failure { error } if error.code == "bridge.foreign-authority")
    );

    let installed = Client::new(bridge)
        .unwrap()
        .with_foreign_provider(&config_path, config_hash)
        .unwrap();
    let correct = request.domains.clone();
    request.domains.host = "x86_64-unknown-linux-gnu".into();
    request.id = [108; 16];
    let (refused, _) = installed.run(&request, &abc).unwrap();
    assert!(
        matches!(refused.outcome, Outcome::Failure { error } if error.code == "bridge.domains")
    );

    request.domains = correct;
    request.id = [109; 16];
    request.external_provider = Some([0; 32]);
    let (refused, _) = installed.run(&request, &abc).unwrap();
    assert!(
        matches!(refused.outcome, Outcome::Failure { error } if error.code == "bridge.foreign-authority")
    );

    request.external_provider = Some(config_hash);
    request.id = [110; 16];
    let (accepted, bytes) = installed.run(&request, &abc).unwrap();
    let Outcome::Value {
        value: ace_aac_bridge::Value::I32(9),
        facts,
    } = accepted.outcome
    else {
        panic!("prepared ABI call failed: {:?}", accepted.outcome);
    };
    assert!(bytes.is_empty());
    assert_eq!(facts.foreign_calls, 1);
    assert_eq!(facts.live_text, 0);
    let library_hash = blake3::Hash::from(config.library_hash);
    assert_eq!(
        facts.read_set,
        [format!(
            "foreign:{}:{library_hash}",
            config.manifest.provider
        )]
    );
    eprintln!(
        "ct_calls_a_prepared_967_provider_only_with_matching_host_authority {:?}",
        started.elapsed()
    );
}
