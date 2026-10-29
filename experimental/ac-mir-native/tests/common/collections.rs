use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::OnceLock;

fn memory_stage(tag: &str, stage: &str) {
    if tag != "frozen-load-first" {
        return;
    }
    #[cfg(windows)]
    unsafe {
        use windows_sys::Win32::System::{
            ProcessStatus::{GetProcessMemoryInfo, PROCESS_MEMORY_COUNTERS_EX},
            Threading::GetCurrentProcess,
        };
        let mut c: PROCESS_MEMORY_COUNTERS_EX = std::mem::zeroed();
        c.cb = std::mem::size_of_val(&c) as u32;
        assert_ne!(
            GetProcessMemoryInfo(GetCurrentProcess(), &mut c as *mut _ as *mut _, c.cb),
            0
        );
        eprintln!(
            "feasibility-memory {stage}: private={} os_peak_commit={}",
            c.PrivateUsage, c.PeakPagefileUsage
        );
    }
}

pub fn libraries() -> &'static [PathBuf] {
    static LIBRARIES: OnceLock<Vec<PathBuf>> = OnceLock::new();
    LIBRARIES.get_or_init(|| {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("runtime/collections");
        let output = Command::new("cargo")
            .args(["build", "--locked", "--release", "--manifest-path"])
            .arg(root.join("Cargo.toml"))
            .env_remove("CARGO_TARGET_DIR")
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        vec![root.join("target/release/ac_collections_runtime.lib")]
    })
}

pub fn run_source(source: &str, tag: &str) -> ac_mn::link_driver::PeRun {
    let memory = |stage| memory_stage(tag, stage);
    let compiled = auto_ac_prototype::source::compile_collections_source(source, "p889.at", &[])
        .unwrap_or_else(|e| panic!("{tag}: frontend {e:?}"));
    memory("frontend");
    let m = auto_mir_lower::lower_checked_collections(&compiled.checked, "d_main", 0)
        .unwrap_or_else(|e| panic!("{tag}: lower {e:?}"));
    memory("mir");
    drop(compiled);
    memory("release-checked");
    run_module(&m, tag, Some(source))
}

pub fn run_module(
    m: &auto_av_core::mir::MirModule,
    tag: &str,
    source: Option<&str>,
) -> ac_mn::link_driver::PeRun {
    run_module_with_units(m, tag, source, |_| {})
}

pub fn run_module_with_units(
    m: &auto_av_core::mir::MirModule,
    tag: &str,
    source: Option<&str>,
    mut rewrite: impl FnMut(&mut auto_av_core::mir::MirModule),
) -> ac_mn::link_driver::PeRun {
    let memory = |stage| memory_stage(tag, stage);
    auto_av_core::verify_mir(&m).unwrap();
    memory("verify");
    let entry = &m.funcs[m.entry as usize];
    let signature = m.sig_of(entry.sig).unwrap();
    assert!(signature.params.is_empty());
    assert!(matches!(
        m.types[signature.result as usize],
        auto_av_core::mir::MirType::I32 | auto_av_core::mir::MirType::Bool
    ));
    let nonce = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let work = super::repo_root().join(format!(
        "docs/plans/attachments/889/source-{tag}-{}-{nonce}",
        std::process::id()
    ));
    std::fs::create_dir_all(&work).unwrap();
    let mut units = auto_av_probe::cache::split_units(&m).unwrap();
    memory("split");
    for unit in &mut units {
        rewrite(unit);
        auto_av_core::verify_mir(unit).unwrap();
    }
    let mut objects = Vec::new();
    for unit in &units {
        let object = ac_mn::collections_lower::emit_unit_coff(unit)
            .unwrap_or_else(|e| panic!("{tag}/{}: emit {e}", unit.funcs[0].name));
        std::fs::write(
            work.join(format!("{}.coff", unit.funcs[0].name)),
            &object.bytes,
        )
        .unwrap();
        objects.push(ac_mn::link_driver::ObjectInput {
            name: unit.funcs[0].name.clone(),
            bytes: object.bytes,
        });
    }
    memory("emit");
    let symbol = ac_mn::symbols::function_symbol(&m.owner, &entry.name);
    let wrapper = ac_mn::collections_lower::emit_wrapper_coff(&symbol).unwrap();
    let run = ac_mn::link_driver::link_and_run(
        &objects,
        &wrapper,
        "ac_start",
        &work.join("pe"),
        libraries(),
        ac_mn::collections_lower::ABI,
    )
    .unwrap_or_else(|e| panic!("{tag}: link/run {e}"));
    std::fs::write(work.join("stderr.log"), &run.stderr).unwrap();
    std::fs::write(work.join("receipt.json"), serde_json::to_vec_pretty(&serde_json::json!({
        "source_blake3": source.map(|source| blake3::hash(source.as_bytes()).to_hex().to_string()),
        "units_emitted": units.len(), "exit": run.exit_code,
        "abi": ac_mn::collections_lower::ABI, "runtime_blake3": blake3::hash(&std::fs::read(&libraries()[0]).unwrap()).to_hex().to_string(),
        "start": if source.is_some() { "source/collections" } else { "typed-mir/untrusted-builder" }, "backend": "shared-mir/clif/coff/pe"
    })).unwrap()).unwrap();
    run
}

pub fn assert_balanced(run: &ac_mn::link_driver::PeRun) {
    // Balance admission is independent of the expected business result.
    ac_mn::collections_observe::decode(run.exit_code, &run.stderr).unwrap();
    let stderr = &run.stderr;
    let text = String::from_utf8_lossy(stderr);
    for item in [
        "MN-COLL live=0\n",
        "MN-COLL bytes=0\n",
        "MN-COLL guest=0\n",
        "MN-COLL invalid=0\n",
        "MN-COLL arena_live=0\n",
        "MN-COLL arena_commit=0\n",
        "MN-COLL arena_invalid=0\n",
        "MN-COLL text_metadata=0\n",
    ] {
        assert!(text.contains(item), "missing {item}: {text}");
    }
    let summaries: Vec<_> = text
        .lines()
        .filter_map(|s| s.strip_prefix("SUMMARY "))
        .collect();
    assert_eq!(summaries.len(), 1, "{text}");
    let fields: std::collections::BTreeMap<_, _> = summaries[0]
        .split_whitespace()
        .map(|s| {
            let (k, v) = s.split_once('=').unwrap();
            (k, v.parse::<u64>().unwrap())
        })
        .collect();
    assert!(fields.get("allocs").is_some(), "{text}");
    assert!(fields.get("frees").is_some(), "{text}");
    assert_eq!(fields.get("live"), Some(&0), "{text}");
    assert_eq!(fields.get("live_bytes"), Some(&0), "{text}");
    assert_eq!(fields.get("allocs"), fields.get("frees"), "{text}");
    assert!(
        text.lines()
            .any(|s| s.starts_with("FREESTAT ") && s.ends_with(" invalid=0")),
        "{text}"
    );
}
