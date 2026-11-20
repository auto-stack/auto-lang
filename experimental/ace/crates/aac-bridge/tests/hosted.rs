//! Explicit compiler/linker integration lane. Prepare the runtime with
//! `cargo build -p ace-collections-runtime --profile runtime-dev` first.

#![cfg(all(windows, target_arch = "x86_64", target_env = "msvc"))]

mod support;

use ace_aac_bridge::{Client, Mode, Outcome};
use ace_av_core::mir::{
    BinOp, FnSig, MirBlock, MirConst, MirHostImport, MirInstr, MirModule, MirSlot, MirType,
    SlotKind, Terminator,
};
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
use app.client.ct_i32

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
    let value = ct_i32(response.move)
    if (value.status != 0) { return value.status }
    return value.value
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
    run_object_bundle(units, wrapper, &data, &[library.to_path_buf()], out)
}

fn run_object_bundle(
    units: Vec<ace_aac_bridge::Object>,
    wrapper: ace_aac_bridge::Object,
    data: &[u8],
    libraries: &[PathBuf],
    out: &Path,
) -> auto_tool_execution::BoundedResult {
    let linked = link_object_bundle(units, wrapper, data, libraries, out);
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

fn link_object_bundle(
    units: Vec<ace_aac_bridge::Object>,
    wrapper: ace_aac_bridge::Object,
    data: &[u8],
    libraries: &[PathBuf],
    out: &Path,
) -> ace_native::link_driver::PeLink {
    let mut remaining = data;
    let mut objects = Vec::new();
    for unit in units {
        let (bytes, next) = remaining.split_at(unit.len as usize);
        objects.push(ace_native::link_driver::ObjectInput {
            name: unit.symbol,
            bytes: bytes.to_vec(),
        });
        remaining = next;
    }
    ace_native::link_driver::link_only(
        &objects,
        remaining,
        &wrapper.symbol,
        out,
        libraries,
        "aac-bridge",
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

fn owned_scheduler_fixture(mismatch: bool) -> ace_av_core::mir::MirModule {
    use ace_av_core::contract::{MIR_CLOSURES_SCHEMA, TEXT_RUNTIME_CAPABILITY};
    use ace_av_core::mir::*;

    let owner = OwnerScope {
        schema: MIR_CLOSURES_SCHEMA.into(),
        module: "aac1051_sched".into(),
    };
    let mut mir = MirModule::new_closures(owner.clone());
    let i32_ty = mir.intern_type(MirType::I32);
    let bool_ty = mir.intern_type(MirType::Bool);
    let str_ty = mir.intern_type(MirType::Str);
    let view_ty = mir.intern_type(MirType::StrView);
    let sig = mir.intern_type(MirType::Fn(FnSig {
        params: vec![],
        result: i32_ty,
    }));
    mir.capabilities.push(TEXT_RUNTIME_CAPABILITY.into());
    mir.consts = vec![
        MirConst::I32(0),
        MirConst::I32(1),
        MirConst::Str("a\0界🇨".as_bytes().into()),
        MirConst::Str(b"late".as_slice().into()),
    ];
    let slots = |types: &[u32]| {
        types
            .iter()
            .map(|ty| MirSlot {
                kind: SlotKind::Temp,
                ty: *ty,
                name: None,
            })
            .collect()
    };
    mir.funcs.push(MirFunction {
        owner: owner.clone(),
        name: "main".into(),
        sig,
        slots: slots(&[i32_ty, i32_ty, str_ty, i32_ty, str_ty, view_ty, i32_ty]),
        blocks: vec![MirBlock {
            instrs: vec![
                MirInstr::Const { dst: 0, value: 1 },
                MirInstr::SchedSpawn {
                    callee: 1,
                    capacity: 0,
                    args: vec![],
                    dst: 1,
                },
                MirInstr::Const { dst: 2, value: 2 },
                MirInstr::SchedSend {
                    handle: 1,
                    msg: if mismatch { 0 } else { 2 },
                },
                MirInstr::SchedAsk {
                    handle: 1,
                    msg: 2,
                    dst: 3,
                },
                MirInstr::Drop { slot: 2 },
                MirInstr::SchedAwait { future: 3, dst: 4 },
                MirInstr::View { dst: 5, src: 4 },
                MirInstr::StrLenBytes { dst: 6, view: 5 },
                MirInstr::BorrowEnd { token: 5 },
                MirInstr::Drop { slot: 4 },
            ],
            term: Terminator::Return { value: 6 },
        }],
        entry: 0,
        source_snapshot: 0,
        spans: vec![],
        env: None,
    });
    mir.funcs.push(MirFunction {
        owner,
        name: "driver".into(),
        sig,
        slots: slots(&[str_ty, i32_ty, i32_ty, bool_ty, str_ty]),
        blocks: vec![
            MirBlock {
                instrs: vec![MirInstr::Const { dst: 2, value: 0 }],
                term: Terminator::Jump { target: 1 },
            },
            MirBlock {
                instrs: vec![
                    MirInstr::SchedRecv { dst: 0 },
                    MirInstr::SchedPort { dst: 1 },
                    MirInstr::Bin {
                        dst: 3,
                        op: BinOp::EqI32,
                        lhs: 1,
                        rhs: 2,
                    },
                ],
                term: Terminator::Branch {
                    cond: 3,
                    then: 2,
                    else_: 3,
                },
            },
            MirBlock {
                instrs: vec![MirInstr::Drop { slot: 0 }],
                term: Terminator::Jump { target: 1 },
            },
            MirBlock {
                instrs: vec![
                    MirInstr::SchedReply { value: 0 },
                    MirInstr::Drop { slot: 0 },
                    MirInstr::Const { dst: 4, value: 3 },
                    MirInstr::SchedReply { value: 4 },
                    MirInstr::Drop { slot: 4 },
                ],
                term: Terminator::Return { value: 2 },
            },
        ],
        entry: 0,
        source_snapshot: 0,
        spans: vec![(
            1,
            ace_av_core::SpanInfo {
                func: 1,
                start: 54,
                end: 61,
            },
        )],
        env: None,
    });
    mir
}

#[test]
fn scheduler_owned_text_loop_awaits_one_reply_and_recovers_after_type_failure() {
    let started = Instant::now();
    let runtime =
        PathBuf::from(std::env::var_os("ACE_AAC_SCHED_LIB").expect("prepare ACE_AAC_SCHED_LIB"));
    for (id, mismatch) in [(100, false), (101, true), (102, true), (103, false)] {
        let module = owned_scheduler_fixture(mismatch);
        ace_av_core::verify_mir(&module).unwrap();
        let stage = stage();
        let observed =
            native_object_image(&module, id, Mode::NativeScheduler, &runtime, stage.path());
        assert_eq!(observed.stdout.as_deref(), Some(&b""[..]));
        if mismatch {
            assert_eq!(observed.kind, ExitKind::Failure(70));
            assert_eq!(
                observed.stderr.as_deref(),
                Some(&b"sched: unsupported scheduler message value\nsched: source function 1 bytes 54..61\n"[..])
            );
        } else {
            // The drain changes this exit to 72 if any mailbox, port or
            // owned allocation remains. The first reply contains 9 bytes.
            assert_eq!(observed.kind, ExitKind::Failure(9));
            assert_eq!(observed.stderr.as_deref(), Some(&b""[..]));
        }
        stage.close().unwrap();
    }
    eprintln!(
        "scheduler_owned_text_loop_awaits_one_reply_and_recovers_after_type_failure {:?}",
        started.elapsed()
    );
}

#[test]
fn gen0_scheduler_reclaims_interpolated_text_on_each_iteration() {
    let started = Instant::now();
    let runtime =
        PathBuf::from(std::env::var_os("ACE_AAC_SCHED_LIB").expect("prepare ACE_AAC_SCHED_LIB"));
    let source = r#"
task Counter {
    on { n int -> {
        print(f"a=${n}")
    } }
}
fn main() int {
    let h = Task.spawn("Counter", 4)
    h.send(1)
    h.send(2)
    return 0
}
"#;
    let module = ace_native::sched::sched_compile(source, "d_main").unwrap();
    let stage = stage();
    let observed = native_object_image(&module, 104, Mode::NativeScheduler, &runtime, stage.path());
    assert_eq!(observed.kind, ExitKind::Success);
    assert_eq!(observed.stdout.as_deref(), Some(&b"a=1\na=2\n"[..]));
    assert_eq!(observed.stderr.as_deref(), Some(&b""[..]));
    stage.close().unwrap();
    eprintln!(
        "gen0_scheduler_reclaims_interpolated_text_on_each_iteration {:?}",
        started.elapsed()
    );
}

#[test]
fn scheduler_and_file_io_share_one_native_product_and_audit_closed_handles() {
    let started = Instant::now();
    let runtime =
        PathBuf::from(std::env::var_os("ACE_AAC_SCHED_LIB").expect("prepare ACE_AAC_SCHED_LIB"));
    let stage = stage();
    let file = stage.path().join("文件.bin");
    std::fs::write(&file, [7, 0, 255]).unwrap();
    let utf16: Vec<u16> = file.to_str().unwrap().encode_utf16().collect();
    for (id, close) in [(105, true), (106, false)] {
        let module = scheduler_file_fixture(&utf16, close);
        let product_stage = tempfile::tempdir_in(stage.path()).unwrap();
        let result = native_object_image(
            &module,
            id,
            Mode::NativeScheduler,
            &runtime,
            product_stage.path(),
        );
        assert_eq!(result.kind, ExitKind::Failure(if close { 7 } else { 72 }));
        assert_eq!(result.stdout.as_deref(), Some(&b"go:\n7\n"[..]));
        assert_eq!(result.stderr.as_deref(), Some(&b""[..]));
        product_stage.close().unwrap();
    }
    assert_eq!(std::fs::read(&file).unwrap(), [7, 0, 255]);
    stage.close().unwrap();
    eprintln!(
        "scheduler_and_file_io_share_one_native_product_and_audit_closed_handles {:?}",
        started.elapsed()
    );
}

fn scheduler_file_fixture(path: &[u16], close: bool) -> MirModule {
    let mut module = platform_scheduler();
    let scalar = module.intern_type(MirType::I32);
    let zero = platform_literal(&mut module, scalar, 0);
    let one = platform_literal(&mut module, scalar, 1);
    let two = platform_literal(&mut module, scalar, 2);
    let three = platform_literal(&mut module, scalar, 3);
    let max_byte = platform_literal(&mut module, scalar, 255);
    let units = platform_literal(&mut module, scalar, path.len() as i32);
    let len_bytes = platform_literal(&mut module, scalar, path.len() as i32 * 2);
    let path_memory = platform_call(&mut module, scalar, "mem.alloc", &[len_bytes]);
    for (i, byte) in path.iter().flat_map(|ch| ch.to_le_bytes()).enumerate() {
        let offset = platform_literal(&mut module, scalar, i as i32);
        let byte = platform_literal(&mut module, scalar, i32::from(byte));
        platform_call(
            &mut module,
            scalar,
            "mem.write_u8",
            &[path_memory, offset, byte],
        );
    }

    let file = platform_call(
        &mut module,
        scalar,
        "file.open_w16",
        &[path_memory, zero, units, one],
    );
    let freed_path = platform_call(&mut module, scalar, "mem.free", &[path_memory]);
    let data = platform_call(&mut module, scalar, "mem.alloc", &[three]);
    let read = platform_call(
        &mut module,
        scalar,
        "file.read_some",
        &[file, data, zero, three],
    );
    let value = platform_call(&mut module, scalar, "mem.read_u8", &[data, zero]);
    let nul = platform_call(&mut module, scalar, "mem.read_u8", &[data, one]);
    let high = platform_call(&mut module, scalar, "mem.read_u8", &[data, two]);
    let freed_data = platform_call(&mut module, scalar, "mem.free", &[data]);
    let mut assertions = vec![
        (read, three),
        (nul, zero),
        (high, max_byte),
        (freed_path, zero),
        (freed_data, zero),
    ];
    if close {
        let closed = platform_call(&mut module, scalar, "file.close", &[file]);
        assertions.push((closed, zero));
    }
    finish_platform_case(module, scalar, value, assertions)
}

fn platform_scheduler() -> MirModule {
    ace_native::sched::sched_compile(
        r#"
fn worker(n int) {
    print("go:")
    print(n)
}
fn main() int {
    go worker(7)
    return 0
}
"#,
        "d_main",
    )
    .unwrap()
}
fn platform_slot(module: &mut MirModule, ty: u32) -> u32 {
    let entry = &mut module.funcs[module.entry as usize];
    let index = entry.slots.len() as u32;
    entry.slots.push(MirSlot {
        kind: SlotKind::Temp,
        ty,
        name: None,
    });
    index
}

fn platform_literal(module: &mut MirModule, scalar: u32, value: i32) -> u32 {
    let dst = platform_slot(module, scalar);
    let index = module.consts.len() as u32;
    module.consts.push(MirConst::I32(value));
    module.funcs[module.entry as usize].blocks[0]
        .instrs
        .push(MirInstr::Const { dst, value: index });
    dst
}

fn platform_call(module: &mut MirModule, scalar: u32, symbol: &str, args: &[u32]) -> u32 {
    let symbol = format!("auto.platform.{symbol}.v1");
    let spec = ace_av_core::platform_io::lookup_symbol(&symbol).unwrap();
    let host = if let Some(index) = module
        .host_imports
        .iter()
        .position(|decl| decl.symbol == symbol)
    {
        index as u32
    } else {
        let sig = module.intern_type(MirType::Fn(FnSig {
            params: args
                .iter()
                .map(|arg| module.funcs[module.entry as usize].slots[*arg as usize].ty)
                .collect(),
            result: scalar,
        }));
        let decl = MirHostImport {
            category: spec.category.into(),
            symbol,
            sig,
            capability: spec.capability.into(),
            max_call_bytes: spec.max_call_bytes,
        };
        if !module.capabilities.contains(&decl.capability) {
            module.capabilities.push(decl.capability.clone());
        }
        let index = module.host_imports.len() as u32;
        module.host_imports.push(decl);
        index
    };
    let dst = platform_slot(module, scalar);
    module.funcs[module.entry as usize].blocks[0]
        .instrs
        .push(MirInstr::PlatformCall {
            dst,
            host,
            args: args.to_vec(),
        });
    dst
}

fn finish_platform_case(
    mut module: MirModule,
    scalar: u32,
    value: u32,
    assertions: Vec<(u32, u32)>,
) -> MirModule {
    let failed = platform_literal(&mut module, scalar, -17);
    let boolean = module.intern_type(MirType::Bool);
    let entry = &mut module.funcs[module.entry as usize];
    let return_block = assertions.len() as u32;
    let failure_block = return_block + 1;
    for (index, (got, expected)) in assertions.into_iter().enumerate() {
        let condition = entry.slots.len() as u32;
        entry.slots.push(MirSlot {
            kind: SlotKind::Temp,
            ty: boolean,
            name: None,
        });
        if index > 0 {
            entry.blocks.push(MirBlock {
                instrs: vec![],
                term: Terminator::Return { value },
            });
        }
        let block = &mut entry.blocks[index];
        block.instrs.push(MirInstr::Bin {
            dst: condition,
            op: BinOp::EqI32,
            lhs: got,
            rhs: expected,
        });
        block.term = Terminator::Branch {
            cond: condition,
            then: index as u32 + 1,
            else_: failure_block,
        };
    }
    entry.blocks.push(MirBlock {
        instrs: vec![],
        term: Terminator::Return { value },
    });
    entry.blocks.push(MirBlock {
        instrs: vec![],
        term: Terminator::Return { value: failed },
    });
    ace_av_core::verify_mir(&module).unwrap();
    module
}

#[test]
fn native_objects_call_the_admitted_967_provider() {
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
    request.id = [111; 16];
    request.mode = Mode::NativeObjects;
    request.domains = ace_aac_bridge::Domains::windows(Mode::NativeObjects);
    let abc = std::fs::read(scope.join("module.abc")).unwrap();
    let bridge = Path::new(env!("CARGO_BIN_EXE_ace-aac-bridge"));
    let (response, data) = Client::new(bridge).unwrap().run(&request, &abc).unwrap();
    assert!(data.is_empty());
    assert!(
        matches!(response.outcome, Outcome::Failure { error } if error.code == "bridge.foreign-authority")
    );

    let client = Client::new(bridge)
        .unwrap()
        .with_foreign_provider(&config_path, config_hash)
        .unwrap();
    let (response, data) = client.run(&request, &abc).unwrap();
    let Outcome::Objects {
        units,
        wrapper,
        runtime,
        ..
    } = response.outcome
    else {
        panic!(
            "native foreign code generation refused the authorized request: {:?}",
            response.outcome
        );
    };
    assert!(units
        .iter()
        .flat_map(|unit| &unit.imports)
        .any(|(_, export)| export == "f0_text_byte_len_v1"));
    let stage = stage();
    let dll = config.library.file_name().unwrap();
    std::fs::copy(&config.library, stage.path().join(dll)).unwrap();
    let mut libraries = vec![config.library.with_extension("dll.lib")];
    if runtime == ace_aac_bridge::NativeRuntime::Text {
        libraries.push(PathBuf::from(
            std::env::var_os("ACE_AAC_TEXT_LIB").expect("prepare ACE_AAC_TEXT_LIB"),
        ));
    }
    let result = run_object_bundle(units, wrapper, &data, &libraries, stage.path());
    assert_eq!(result.kind, ExitKind::Failure(9));
    stage.close().unwrap();
    eprintln!(
        "native_objects_call_the_admitted_967_provider {:?}",
        started.elapsed()
    );
}

#[test]
fn native_sqlite_executes_the_canonical_api_and_preserves_rollback() {
    let started = Instant::now();
    let scope = PathBuf::from(
        std::env::var_os("ACE_AAC_SQLITE_FIXTURE").expect("prepare ACE_AAC_SQLITE_FIXTURE"),
    );
    let config_path = scope.join("provider.json");
    let config_bytes = std::fs::read(&config_path).unwrap();
    let config: ace_aac_bridge::ForeignConfig = serde_json::from_slice(&config_bytes).unwrap();
    let config_hash = *blake3::hash(&config_bytes).as_bytes();
    let bridge = Path::new(env!("CARGO_BIN_EXE_ace-aac-bridge"));
    let client = Client::new(bridge)
        .unwrap()
        .with_foreign_provider(&config_path, config_hash)
        .unwrap();
    for (request_name, abc_name, code) in [
        ("request.json", "module.abc", 9),
        ("reject.json", "reject.abc", 70),
    ] {
        let mut request: ace_aac_bridge::Request =
            serde_json::from_slice(&std::fs::read(scope.join(request_name)).unwrap()).unwrap();
        let abc = std::fs::read(scope.join(abc_name)).unwrap();
        let (response, data) = client.run(&request, &abc).unwrap();
        let Outcome::Objects {
            units,
            wrapper,
            runtime,
            ..
        } = response.outcome
        else {
            panic!(
                "expected native SQLite object bundle: {:?}",
                response.outcome
            );
        };
        assert!(units
            .iter()
            .flat_map(|unit| &unit.imports)
            .any(|(_, export)| export.starts_with("p967_")));
        let stage = stage();
        std::fs::copy(
            &config.library,
            stage.path().join(config.library.file_name().unwrap()),
        )
        .unwrap();
        let mut libraries = vec![config.library.with_extension("lib")];
        if runtime == ace_aac_bridge::NativeRuntime::Text {
            libraries.push(PathBuf::from(
                std::env::var_os("ACE_AAC_TEXT_LIB").expect("prepare ACE_AAC_TEXT_LIB"),
            ));
        }
        let result = run_object_bundle(units, wrapper, &data, &libraries, stage.path());
        assert_eq!(result.kind, ExitKind::Failure(code));
        stage.close().unwrap();

        request.mode = Mode::CompileTime;
        request.domains = ace_aac_bridge::Domains::windows(Mode::CompileTime);
        let (response, data) = client.run(&request, &abc).unwrap();
        assert!(data.is_empty());
        assert!(
            matches!(response.outcome, Outcome::Failure { error } if error.code == "bridge.foreign-call-frame")
        );
    }
    eprintln!(
        "native_sqlite_executes_the_canonical_api_and_preserves_rollback {:?}",
        started.elapsed()
    );
}

#[test]
fn scheduler_native_tcp_uses_real_loopback_and_closes_both_streams() {
    let started = Instant::now();
    let runtime =
        PathBuf::from(std::env::var_os("ACE_AAC_SCHED_LIB").expect("prepare ACE_AAC_SCHED_LIB"));
    let mut module = platform_scheduler();
    let scalar = module.intern_type(MirType::I32);
    let text = module.intern_type(MirType::StrView);
    let entry = module.entry as usize;
    let host = platform_slot(&mut module, text);
    let host_literal = module.consts.len() as u32;
    module
        .consts
        .push(MirConst::Str(b"127.0.0.1".as_slice().into()));
    module.funcs[entry].blocks[0].instrs.push(MirInstr::Const {
        dst: host,
        value: host_literal,
    });
    let payload = platform_slot(&mut module, text);
    let text_literal = module.consts.len() as u32;
    module
        .consts
        .push(MirConst::Str(b"\0\xe4\xb8\xad".as_slice().into()));
    module.funcs[entry].blocks[0].instrs.push(MirInstr::Const {
        dst: payload,
        value: text_literal,
    });
    let zero = platform_literal(&mut module, scalar, 0);
    let one = platform_literal(&mut module, scalar, 1);
    let two = platform_literal(&mut module, scalar, 2);
    let three = platform_literal(&mut module, scalar, 3);
    let four = platform_literal(&mut module, scalar, 4);
    let seven = platform_literal(&mut module, scalar, 7);
    let byte = platform_literal(&mut module, scalar, 255);
    let retry = platform_literal(&mut module, scalar, -10);
    let closed = platform_literal(&mut module, scalar, -6);
    let wrong_origin = platform_literal(&mut module, scalar, -3);

    let listener = platform_call(&mut module, scalar, "net.tcp_listen", &[zero]);
    let quiet = platform_call(&mut module, scalar, "net.tcp_accept", &[listener]);
    let port = platform_call(&mut module, scalar, "net.tcp_local_port", &[listener]);
    let client = platform_call(&mut module, scalar, "net.tcp_connect", &[host, port]);
    let server = platform_call(&mut module, scalar, "net.tcp_accept", &[listener]);
    let region = platform_call(&mut module, scalar, "mem.alloc", &[four]);
    let wrote = platform_call(
        &mut module,
        scalar,
        "net.tcp_write_some",
        &[client, payload],
    );
    let read = platform_call(
        &mut module,
        scalar,
        "net.tcp_read_into",
        &[server, region, zero, four],
    );
    let nul = platform_call(&mut module, scalar, "mem.read_u8", &[region, zero]);
    let tail = platform_call(&mut module, scalar, "mem.read_u8", &[region, three]);
    let expected_tail = platform_literal(&mut module, scalar, 173);
    let staged1 = platform_call(&mut module, scalar, "mem.write_u8", &[region, zero, seven]);
    let staged2 = platform_call(&mut module, scalar, "mem.write_u8", &[region, one, zero]);
    let staged3 = platform_call(&mut module, scalar, "mem.write_u8", &[region, two, byte]);
    let raw_wrote = platform_call(
        &mut module,
        scalar,
        "net.tcp_write_region",
        &[server, region, zero, three],
    );
    let cleared_first = platform_call(&mut module, scalar, "mem.write_u8", &[region, zero, zero]);
    let cleared_last = platform_call(&mut module, scalar, "mem.write_u8", &[region, two, zero]);
    let raw_read = platform_call(
        &mut module,
        scalar,
        "net.tcp_read_into",
        &[client, region, zero, three],
    );
    let received = platform_call(&mut module, scalar, "mem.read_u8", &[region, zero]);
    let high = platform_call(&mut module, scalar, "mem.read_u8", &[region, two]);
    let wrong_close = platform_call(&mut module, scalar, "net.tcp_close", &[listener]);
    let client_closed = platform_call(&mut module, scalar, "net.tcp_close", &[client]);
    let again = platform_call(&mut module, scalar, "net.tcp_close", &[client]);
    let server_closed = platform_call(&mut module, scalar, "net.tcp_close", &[server]);
    let freed = platform_call(&mut module, scalar, "mem.free", &[region]);
    let module = finish_platform_case(
        module,
        scalar,
        received,
        vec![
            (quiet, retry),
            (wrote, four),
            (read, four),
            (nul, zero),
            (tail, expected_tail),
            (staged1, zero),
            (staged2, zero),
            (staged3, zero),
            (raw_wrote, three),
            (cleared_first, zero),
            (cleared_last, zero),
            (raw_read, three),
            (high, byte),
            (wrong_close, wrong_origin),
            (client_closed, zero),
            (again, closed),
            (server_closed, zero),
            (freed, zero),
        ],
    );
    let stage = stage();
    let result = native_object_image(&module, 120, Mode::NativeScheduler, &runtime, stage.path());
    assert_eq!(result.kind, ExitKind::Failure(7), "{result:?}");
    assert_eq!(result.stdout.as_deref(), Some(&b"go:\n7\n"[..]));
    assert_eq!(result.stderr.as_deref(), Some(&b""[..]));
    stage.close().unwrap();
    eprintln!(
        "scheduler_native_tcp_uses_real_loopback_and_closes_both_streams {:?}",
        started.elapsed()
    );
}

#[test]
fn cancelling_a_live_scheduler_reaps_the_product_before_a_late_reply() {
    use std::io::BufRead;
    let started = Instant::now();
    let runtime =
        PathBuf::from(std::env::var_os("ACE_AAC_SCHED_LIB").expect("prepare ACE_AAC_SCHED_LIB"));
    let stage = stage();
    let module = ace_native::sched::sched_compile(
        r#"
task Delay {
    on { n int -> {
        print("handling ask")
        sleep(8000)
        print("publishing")
        reply(n)
    } }
}
fn main() int {
    let h = Task.spawn("Delay", 2)
    return await h.ask(7)
}
"#,
        "d_main",
    )
    .unwrap();
    let needs_cancellation = request(Mode::NativeScheduler, &module, 125);
    let abc = ace_av_core::encode_abc(&module).unwrap().to_flat();
    let client = Client::new(Path::new(env!("CARGO_BIN_EXE_ace-aac-bridge"))).unwrap();
    let (response, data) = client.run(&needs_cancellation, &abc).unwrap();
    let Outcome::Objects { units, wrapper, .. } = response.outcome else {
        panic!("expected native scheduler");
    };
    let linked = link_object_bundle(units, wrapper, &data, &[runtime], stage.path());
    ace_native::sched::audit_sched_pe_imports(&std::fs::read(&linked.exe_path).unwrap()).unwrap();
    let mut spec = ExecutionSpec::new([linked.exe_path.as_os_str()]);
    spec.tree_scope = TreeScope::ProcessTree;
    let mut product = auto_tool_execution::spawn(&spec).unwrap();
    let stdout = product.take_stdout().unwrap();
    let (observed, read) = std::sync::mpsc::channel();
    let reader = std::thread::spawn(move || {
        let mut stdout = std::io::BufReader::new(stdout);
        let mut line = Vec::new();
        stdout.read_until(b'\n', &mut line).unwrap();
        let _ = observed.send(line);
        let mut late = Vec::new();
        stdout.read_until(b'\n', &mut late).unwrap();
        late
    });
    let entered = read.recv_timeout(Duration::from_secs(2));
    assert!(product.try_wait().unwrap().is_none());
    assert!(!product
        .cancel_within(Duration::from_millis(750))
        .unwrap()
        .success());
    assert_eq!(entered.unwrap(), b"handling ask\n");
    assert!(reader.join().unwrap().is_empty());
    stage.close().unwrap();

    let next = fixture(false);
    let input = request(Mode::CompileTime, &next, 126);
    let abc = ace_av_core::encode_abc(&next).unwrap().to_flat();
    let (response, _) = client.run(&input, &abc).unwrap();
    assert!(matches!(
        response.outcome,
        Outcome::Value {
            value: ace_aac_bridge::Value::I32(7),
            ..
        }
    ));
    eprintln!(
        "cancelling_a_live_scheduler_reaps_the_product_before_a_late_reply {:?}",
        started.elapsed()
    );
}

#[test]
fn native_bridge_replaces_and_reclaims_the_canonical_global_object() {
    let started = Instant::now();
    let library = PathBuf::from(
        std::env::var_os("ACE_AAC_COLLECTIONS_LIB").expect("prepare ACE_AAC_COLLECTIONS_LIB"),
    );
    let corpus = include_str!(
        "../../../../../crates/auto-lang/test/cases/language/modules/imports-globals.case.ad"
    );
    let case = corpus.split_once("## Case: object-global\n").unwrap().1;
    let source = case
        .split_once("```auto\n")
        .unwrap()
        .1
        .split_once("```")
        .unwrap()
        .0;
    let caps = [
        "ac.source.language.av/v1".into(),
        "ac.source.modules.packages/v1".into(),
        "console.stdout".into(),
        "console.stderr".into(),
    ];
    let product = ace_source_query::modules::packages::compile_package_product_auto(
        &[ace_core::source::packages::PackageModuleInput {
            package: "auto".into(),
            version: "0".into(),
            id: "main".into(),
            source: source.into(),
            provider: None,
        }],
        &ace_core::source::packages::unit_key("auto", "main"),
        &caps,
        ace_core::source::cst::FrontBudget::WORK_OWNED_64MIB,
    )
    .expect("compile the canonical PLAN-1085 package");
    let init = product
        .bound
        .module_init_order
        .iter()
        .filter_map(|key| {
            product
                .bound
                .module_init_fns
                .get(key)
                .map(|name| format!("d_{name}"))
        })
        .collect::<Vec<_>>();
    let mut mir = ace_lowering::lower_checked_auto_language_with_init(
        &product.compiled.checked,
        "d_main",
        1051,
        &init,
    )
    .unwrap();
    assert_eq!(mir.globals.len(), 1);
    assert!(!mir.init.is_empty());
    let stage = stage();
    let result = native_object_image(
        &mir,
        128,
        Mode::NativeCollections,
        &library,
        &stage.path().join("object-global"),
    );
    assert_eq!(result.kind, ExitKind::Failure(42), "{result:?}");
    let summary = String::from_utf8_lossy(result.stderr.as_deref().unwrap_or_default());
    assert!(
        summary.lines().any(|line| line == "MN-COLL live=0"),
        "{summary}"
    );

    mir.capabilities
        .push(ace_av_core::contract::GLOBAL_TAKE_CAPABILITY.into());
    let entry = &mut mir.funcs[mir.entry as usize];
    let block = entry
        .blocks
        .iter_mut()
        .find(|block| {
            block
                .instrs
                .iter()
                .any(|i| matches!(i, MirInstr::GlobalGet { .. }))
        })
        .unwrap();
    let index = block
        .instrs
        .iter()
        .position(|i| matches!(i, MirInstr::GlobalGet { .. }))
        .unwrap();
    let MirInstr::GlobalGet { dst, global } = block.instrs[index] else {
        unreachable!()
    };
    block
        .instrs
        .insert(index, MirInstr::GlobalTake { dst, global });
    entry.spans.clear();
    let result = native_object_image(
        &mir,
        129,
        Mode::NativeCollections,
        &library,
        &stage.path().join("moved-global"),
    );
    assert_eq!(result.kind, ExitKind::Failure(70), "{result:?}");
    let summary = String::from_utf8_lossy(result.stderr.as_deref().unwrap_or_default());
    assert!(
        summary.lines().any(|line| line == "MN-COLL-TRAP code=34"),
        "{summary}"
    );
    assert!(
        summary.lines().any(|line| line == "MN-COLL live=0"),
        "{summary}"
    );
    stage.close().unwrap();
    eprintln!(
        "native_bridge_replaces_and_reclaims_the_canonical_global_object {:?}",
        started.elapsed()
    );
}
