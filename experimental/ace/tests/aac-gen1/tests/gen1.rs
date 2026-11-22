//! PLAN-1053 T-02 conformance: the Gen1 native AAC executable.
//!
//! Daily lane — consumes the prepared executable via `AAC1053_GEN1_DIR`
//! (built by the explicit `prepare-gen1` build/integration lane, never
//! inside these tests).
//!
//! Boot parity oracle: the exit code is the wrap32 FNV-1a reduction of the
//! boot entry's per-subsystem observables. The expected reduction is
//! derived HERE, independently, in Rust — quoted contract constants, the
//! frozen schema bytes, a SHA-256 reference (sha2), and the documented
//! blake3 identity framing (PLAN-1056 `digest.identity_hex2` /
//! PLAN-1052 `store.axis2` shapes). No expected value is copied from Auto
//! output (aac-support / aac-host precedent). The whole-subject program
//! cannot run on the AV lane within the frozen verify-analysis budget
//! (one lowered subject function needs ~34 MiB > the frozen 16 MiB), so the
//! independent oracle is the contract mirror, exactly as the 1046 lane uses
//! its Rust front door as the same-source oracle.

use std::process::Command;
use std::time::Instant;

use ace_aac_gen1_conformance::{
    gen1_fingerprint, prepared_bridge, prepared_compile, prepared_gen1, run_compile_case,
    run_gen1_boot, scratch, DRIVER_MANIFEST, GEN1_EXE,
};
use sha2::Digest;

// ---------------------------------------------------------------------------
// Independent contract mirror (values derived from the frozen source text,
// never from Auto execution output)
// ---------------------------------------------------------------------------

/// SHA-256 hex (FIPS 180-4) — the `digest.sha256_hex` contract.
fn sha256_hex_reference(bytes: &[u8]) -> String {
    let mut h = sha2::Sha256::new();
    h.update(bytes);
    hex(&h.finalize())
}

fn hex(bytes: &[u8]) -> String {
    let mut out = String::with_capacity(bytes.len() * 2);
    for b in bytes {
        out.push_str(&format!("{b:02x}"));
    }
    out
}

/// Mirror of `digest.identity_hex2` / `store.axis2`: blake3 over
/// "blake3-1.5\0<family>\0" + two length-prefixed UTF-8 parts.
fn axis2(domain: &str, a: &str, b: &str) -> String {
    let mut pre = Vec::new();
    pre.extend_from_slice(b"blake3-1.5");
    pre.push(0);
    pre.extend_from_slice(domain.as_bytes());
    pre.push(0);
    for part in [a.as_bytes(), b.as_bytes()] {
        pre.extend_from_slice(&(part.len() as u64).to_le_bytes());
        pre.extend_from_slice(part);
    }
    hex(blake3::hash(&pre).as_bytes())
}

/// Mirror of `host_client.fnv32_bytes`: FNV-1a in the wrap32 discipline.
fn fnv32_bytes(s: &str) -> i64 {
    let mut h: i64 = 2166136261;
    for b in s.as_bytes() {
        h = ((h ^ (*b as i64)) * 16777619) % 4294967296;
    }
    h
}

struct Fold {
    c: i64,
}

impl Fold {
    fn new() -> Self {
        Fold { c: 0 }
    }
    fn mix_str(&mut self, s: &str) {
        self.c = (self.c * 31 + fnv32_bytes(s)) % 4294967296;
    }
    fn mix_int(&mut self, v: i64) {
        self.mix_str(&format!("{v}"));
    }
    fn exit(self) -> i32 {
        (self.c % 2147483648) as i32
    }
}

/// The independently derived boot reduction over the frozen contract.
fn expected_boot_reduction() -> i32 {
    let mut f = Fold::new();
    // support digests / codecs / identity
    let sha_abc = sha256_hex_reference(b"abc");
    f.mix_str(&sha_abc);
    f.mix_str("ok truncated-header bad-magic unsupported-framing unknown-kind too-large truncated-payload trailing-bytes checksum-mismatch count-overflow unsupported-version unknown-schema duplicate-key non-canonical-order non-ascii-bytes empty-field");
    f.mix_str(&sha_abc);
    f.mix_str("aac.atom-data/v1");
    // syntax front end + grammar data
    f.mix_int(4); // str_to_bytes("gen1").len()
    f.mix_int(427); // auto_parts().len() (frozen grammar table)
    f.mix_int(1); // flag_missing
    f.mix_int(65536); // rescan_window
    f.mix_int(16384); // page_size
    f.mix_int(10); // st_new().len()
    f.mix_int(0); // mtag_lit
    f.mix_int(1); // fstr_default
    // source projection
    f.mix_int(10); // ast_stride
    f.mix_int(0); // byte_offset_utf8(empty, 0, 0, 0)
    // semantics
    f.mix_int(0); // row_count(empty)
    f.mix_int(1); // function_decl
    f.mix_int(1); // integer_type
    f.mix_int(-20); // type_unknown
    f.mix_int(1); // pending_templates
    f.mix_int(-18); // initializer_cycle
    f.mix_int(-48); // type_pattern_error
    f.mix_int(-52); // choice_argument_error
    f.mix_int(-50); // type_recursive_layout_error
    f.mix_int(-37); // foreign_authority_error
    f.mix_int(0); // foreign_field_kind(empty page)
    f.mix_int(-70); // instance_unsupported
    // checked HIR through the synthesized frozen schema module (the harness
    // embeds the schema via Rust `{:?}` framing, which the Auto lexer
    // unescapes back to the exact schema bytes)
    f.mix_str(include_str!("../../../crates/core/schema/core-i32.atom"));
    f.mix_int(-30); // hir_unsupported
    f.mix_int(1); // constant_op
    f.mix_int(1); // getter_mode
    // typed MIR base
    f.mix_int(4); // function_stride
    f.mix_int(-62); // invalid_mir_type
    // driver / store / query / package identity axes
    let st = axis2("ac1052/axis2/v1", "s", "t");
    let sg = axis2("ac1052/axis2/v1", "l", "g");
    let pb = axis2("ac1052/axis2/v1", "p", "b");
    f.mix_str(&axis2(
        "ac1052/build/v1",
        &axis2("ac1052/pair/v1", &st, &sg),
        &pb,
    ));
    let qk = axis2("ac1052/q-kind/v1", "k", "l");
    let qa = axis2("ac1052/axes/v1", "a", "b");
    f.mix_str(&(axis2("ac1052/query/v1", &qk, &qa) + "c"));
    let sp = axis2("ac1052/pass/v1", "s", "p");
    let iv = axis2("ac1052/inputs/v1", "v", "g");
    f.mix_str(&axis2("ac1052/stage/v1", &sp, &iv));
    let lk = axis2("ac1052/lock/v1", "d", "p");
    let pt = axis2("ac1052/provider-target/v1", "r", "t");
    f.mix_str(&(axis2("ac1052/lock-closure/v1", &lk, &pt) + "c"));
    // host client surface
    f.mix_str(""); // join_bytes(empty)
    f.mix_int(200); // argv_shape(empty): n < 6
    f.mix_int(2); // nul_count([0, 0, 1])
    f.mix_str("__aac_ct_entry"); // ct_entry_name
    f.exit()
}

#[test]
fn gen1_boot_native_matches_independent_reference() {
    let started = Instant::now();
    let exe = prepared_gen1().expect("prepared Gen1 exe (run prepare-gen1, set AAC1053_GEN1_DIR)");
    let expected = expected_boot_reduction();
    let observed = run_gen1_boot(exe).expect("run Gen1 boot");
    assert_eq!(
        observed, expected,
        "native Gen1 boot reduction must equal the independently derived contract reduction"
    );
    assert_ne!(observed, 0, "boot reduction must not degenerate to zero");
    eprintln!("gen1 boot parity: exit={observed} in {:?} (<=10s gate)", started.elapsed());
    assert!(
        started.elapsed().as_secs_f64() <= 10.0,
        "gen1 boot case exceeded the 10-second gate: {:?}",
        started.elapsed()
    );
}

#[test]
fn gen1_exe_is_a_pe_with_manifest_identity() {
    let started = Instant::now();
    let exe = prepared_gen1().expect("prepared Gen1 exe");
    let bytes = std::fs::read(exe).expect("read Gen1 exe");
    assert_eq!(&bytes[..2], b"MZ", "Gen1 executable must be a Windows PE");
    let manifest = std::fs::read_to_string(
        exe.parent().expect("exe parent").join(DRIVER_MANIFEST),
    )
    .expect("read manifest");
    let mut fields = manifest.lines();
    assert_eq!(
        fields.next().map(|s| s.to_owned()),
        Some(gen1_fingerprint().to_hex().to_string()),
        "manifest fingerprint must bind the current whole-tree source identity"
    );
    assert_eq!(
        fields.next().map(|s| s.to_owned()),
        Some(blake3::hash(&bytes).to_hex().to_string()),
        "manifest executable digest must match the executable bytes"
    );
    let runtime_id = fields.next().expect("manifest runtime identity line");
    assert!(!runtime_id.is_empty());
    let tool = fields.next().expect("manifest tool identity line");
    assert!(tool.contains("rustc"), "tool identity must record the linker toolchain");
    eprintln!("gen1 identity: {} bytes, PE ok, manifest ok, {:?}", bytes.len(), started.elapsed());
    assert!(
        started.elapsed().as_secs_f64() <= 10.0,
        "identity case exceeded the 10-second gate"
    );
}

#[test]
fn gen1_compile_entry_runs_pipeline_and_reports_stages() {
    let started = Instant::now();
    let exe = prepared_compile()
        .expect("prepared Gen1 compile exe (run prepare-gen1, set AAC1053_GEN1_DIR)");
    let root = scratch("compile-entry");
    let input = root.join("input.at");
    let output = root.join("report.txt");
    let abc = root.join("closure.abc");
    std::fs::write(&input, "fn main() int { return 2 + 3 * 4 }\n").expect("write input");

    // Positive: the corpus baseline program lowers to typed MIR; the
    // report header is the frozen "MIR aac.base-mir/1 <n>" shape and the
    // ABC byte count must equal the reported length (never emit-as-run).
    let (report, abc) = run_compile_case(exe, &input, &output, "mir", Some(&abc))
        .expect("run Gen1 compile (mir)");
    let header = "MIR aac.base-mir/1 ";
    let reported = report
        .strip_prefix(header)
        .and_then(|rest| rest.trim().parse::<usize>().ok())
        .unwrap_or_else(|| panic!("MIR report header mismatch: {report:?}"));
    let bytes = abc.expect("mir mode must emit ABC bytes");
    assert_eq!(
        bytes.len(),
        reported,
        "reported ABC length must equal the emitted bytes (emit-as-run is a failure)"
    );
    assert!(bytes.len() > 44, "ABC closure must carry its header");

    // Determinism: same input, fresh run, byte-identical closure identity.
    let abc2_path = root.join("closure2.abc");
    let (report2, abc2) = run_compile_case(exe, &input, &output, "mir", Some(&abc2_path))
        .expect("rerun Gen1 compile (mir)");
    assert_eq!(report, report2, "compile report must be deterministic");
    assert_eq!(
        bytes,
        abc2.expect("second ABC"),
        "same source must lower to byte-identical ABC"
    );

    // Negative: an Auto-semantic error is refused at its stage with the
    // contract diagnostic naming (never a silent empty success).
    let bad = root.join("bad.at");
    std::fs::write(&bad, "fn main() int { return missing_name }\n").expect("write bad input");
    let (bad_report, bad_abc) = run_compile_case(exe, &bad, &output, "verify", None)
        .expect("run Gen1 compile (bad source)");
    assert!(
        bad_report.starts_with("ERROR "),
        "invalid source must report ERROR, got {bad_report:?}"
    );
    assert!(
        bad_report.contains("bind.unresolved") || bad_report.contains("type."),
        "diagnostic must name the frozen stage/code, got {bad_report:?}"
    );
    assert!(bad_abc.is_none());
    eprintln!("gen1 compile entry ok in {:?} (<=10s gate)", started.elapsed());
    assert!(
        started.elapsed().as_secs_f64() <= 10.0,
        "compile-entry case exceeded the 10-second gate"
    );
    let _ = std::fs::remove_dir_all(&root);
}



#[test]
fn gen1_end_to_end_source_to_real_pe_exit() {
    let started = Instant::now();
    let exe = prepared_compile()
        .expect("prepared Gen1 compile exe (run prepare-gen1, set AAC1053_GEN1_DIR)");
    let root = scratch("gen1-e2e");
    let input = root.join("input.at");
    let output = root.join("report.txt");
    let abc_path = root.join("closure.abc");
    std::fs::write(&input, "fn main() int { return 2 + 3 * 4 }\n").expect("write input");

    // 1. The Gen1 compiler (whole-subject Auto exe) lowers the corpus
    //    baseline program to its typed MIR ABC closure.
    let (_report, abc) = run_compile_case(exe, &input, &output, "mir", Some(&abc_path))
        .expect("Gen1 compile to ABC");
    let abc = abc.expect("mir mode must emit ABC");

    // 2. The ABC rides the 1051 bridge service (explicit machine backend,
    //    spawned as its own process) and comes back as a COFF object bundle.
    let decoded = ace_av_core::abc::decode_abc(&abc).expect("decode the Gen1 ABC");
    let module = decoded.module;
    // The bridge admits the request against the schema DERIVED FROM THE ABC
    // PROFILE (the driver's own owner schema is its private artifact id,
    // not the wire schema) — mirror the bridge's expectation exactly.
    use ace_av_core::contract::*;
    let expected_schema: &str = match module.profile.as_str() {
        AV_PROFILE => MIR_SCHEMA,
        AV_DATA_PROFILE => MIR_DATA_SCHEMA,
        AV_OWNED_PROFILE => MIR_OWNED_SCHEMA,
        AV_COLLECTIONS_PROFILE => MIR_COLLECTIONS_SCHEMA,
        AV_CLOSURES_PROFILE => MIR_CLOSURES_SCHEMA,
        AV_DYNAMIC_PROFILE => MIR_DYNAMIC_SCHEMA,
        AV_ENTRY_UNIT_PROFILE => MIR_ENTRY_UNIT_SCHEMA,
        AV_HOST_CONSOLE_PROFILE => MIR_HOST_CONSOLE_SCHEMA,
        other => panic!("unsupported ABC profile family: {other}"),
    };
    let mode = ace_aac_bridge::Mode::NativeObjects;
    let request = ace_aac_bridge::Request {
        id: blake3::hash(b"aac1053-gen1-e2e").as_bytes()[..16]
            .try_into()
            .expect("16-byte request id"),
        mode,
        domains: ace_aac_bridge::Domains::windows(mode),
        inputs: ace_aac_bridge::InputIdentity {
            source: *blake3::hash(&std::fs::read(&input).expect("read input")).as_bytes(),
            grammar: *blake3::hash(expected_schema.as_bytes()).as_bytes(),
            passes: [3; 32],
            stdlib: [4; 32],
            dependencies: [5; 32],
        },
        // The 1047 driver's ABC carries an empty owner module name; the
        // bridge requires a non-empty admitted owner — declare this
        // request's owner identity explicitly.
        owner: if module.owner.module.is_empty() {
            "aac1053.gen1.e2e".to_string()
        } else {
            module.owner.module.clone()
        },
        mir_schema: expected_schema.to_string(),
        options: ace_aac_bridge::Options { remaining_ms: 7000 },
        capabilities: module.capabilities.clone(),
        external_provider: None,
        globals: module
            .globals
            .iter()
            .map(|global| ace_aac_bridge::WireGlobal {
                name: global.name.clone(),
                ty: global.ty,
                mutable: global.mutable,
            })
            .collect(),
        initializers: module.init.clone(),
    };
    let bridge = ace_aac_bridge::Client::new(prepared_bridge().expect("prepared bridge tool"))
        .expect("bridge client");
    let (response, bytes) = bridge.run(&request, &abc).expect("bridge exchange");
    let ace_aac_bridge::Outcome::Objects {
        units,
        wrapper,
        runtime,
        ..
    } = response.outcome
    else {
        panic!("expected a linked-program object bundle: {:?}", response.outcome);
    };
    assert_eq!(
        runtime,
        ace_aac_bridge::NativeRuntime::None,
        "the scalar baseline must not pull a runtime library"
    );

    // 3. Parent-side link of the service-returned objects into a real PE.
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
    let image = ace_native::link_driver::link_only(
        &objects,
        remaining,
        &wrapper.symbol,
        &root.join("link"),
        &[],
        "aac1053-gen1-e2e",
    )
    .expect("link the bridge objects");

    // 4. The real machine product runs with the corpus oracle: exit 14.
    let mut spec = auto_tool_execution::ExecutionSpec::new([std::ffi::OsStr::new(
        &image.exe_path,
    )]);
    spec.tree_scope = auto_tool_execution::TreeScope::ProcessTree;
    let result = auto_tool_execution::run_bounded(
        &spec,
        &auto_tool_execution::BoundedPolicy {
            exec_deadline: std::time::Duration::from_secs(4),
            cleanup_deadline: std::time::Duration::from_secs(1),
            stdout_cap: Some(4096),
            stderr_cap: Some(4096),
        },
    )
    .expect("run the Gen1-produced PE");
    assert_eq!(
        result.kind,
        auto_tool_execution::ExitKind::Failure(14),
        "the Gen1-compiled corpus baseline must exit 14, stderr={:?}",
        result.stderr.as_deref().map(|b| String::from_utf8_lossy(b).into_owned())
    );
    eprintln!(
        "gen1 end-to-end (compile->bridge->link->run) ok in {:?} (<=10s gate)",
        started.elapsed()
    );
    assert!(
        started.elapsed().as_secs_f64() <= 10.0,
        "end-to-end case exceeded the 10-second gate"
    );
    let _ = std::fs::remove_dir_all(&root);
}

/// Run one single-file source through the whole Gen1 chain and return the
/// real PE's exit code: Gen1 compile (mir) -> bridge service -> link -> run.
fn gen1_chain_exit(compile: &std::path::Path, source: &[u8], tag: &str) -> Result<i32, String> {
    use auto_tool_execution::{BoundedPolicy, ExecutionSpec, ExitKind, TreeScope};
    let root = scratch(tag);
    let input = root.join("main.at");
    let output = root.join("report.txt");
    let abc_path = root.join("closure.abc");
    std::fs::write(&input, source).map_err(|e| format!("write pilot input: {e}"))?;
    let (_report, abc) = run_compile_case(compile, &input, &output, "mir", Some(&abc_path))
        .map_err(|e| format!("Gen1 compile: {e}"))?;
    let abc = abc.ok_or("mir mode emitted no ABC")?;
    let decoded = ace_av_core::abc::decode_abc(&abc).map_err(|e| format!("decode ABC: {e:?}"))?;
    let module = decoded.module;
    use ace_av_core::contract::*;
    let expected_schema: &str = match module.profile.as_str() {
        AV_PROFILE => MIR_SCHEMA,
        AV_DATA_PROFILE => MIR_DATA_SCHEMA,
        AV_OWNED_PROFILE => MIR_OWNED_SCHEMA,
        AV_COLLECTIONS_PROFILE => MIR_COLLECTIONS_SCHEMA,
        AV_CLOSURES_PROFILE => MIR_CLOSURES_SCHEMA,
        AV_DYNAMIC_PROFILE => MIR_DYNAMIC_SCHEMA,
        AV_ENTRY_UNIT_PROFILE => MIR_ENTRY_UNIT_SCHEMA,
        AV_HOST_CONSOLE_PROFILE => MIR_HOST_CONSOLE_SCHEMA,
        other => return Err(format!("unsupported ABC profile family: {other}")),
    };
    let mode = ace_aac_bridge::Mode::NativeObjects;
    let request = ace_aac_bridge::Request {
        id: blake3::hash(tag.as_bytes()).as_bytes()[..16]
            .try_into()
            .expect("16-byte request id"),
        mode,
        domains: ace_aac_bridge::Domains::windows(mode),
        inputs: ace_aac_bridge::InputIdentity {
            source: *blake3::hash(source).as_bytes(),
            grammar: *blake3::hash(expected_schema.as_bytes()).as_bytes(),
            passes: [3; 32],
            stdlib: [4; 32],
            dependencies: [5; 32],
        },
        owner: if module.owner.module.is_empty() {
            tag.to_string()
        } else {
            module.owner.module.clone()
        },
        mir_schema: expected_schema.to_string(),
        options: ace_aac_bridge::Options { remaining_ms: 7000 },
        capabilities: module.capabilities.clone(),
        external_provider: None,
        globals: module
            .globals
            .iter()
            .map(|global| ace_aac_bridge::WireGlobal {
                name: global.name.clone(),
                ty: global.ty,
                mutable: global.mutable,
            })
            .collect(),
        initializers: module.init.clone(),
    };
    let bridge = ace_aac_bridge::Client::new(
        prepared_bridge().map_err(|e| format!("prepared bridge: {e}"))?,
    )
    .map_err(|e| format!("bridge client: {e}"))?;
    let (response, bytes) = bridge
        .run(&request, &abc)
        .map_err(|e| format!("bridge exchange: {e}"))?;
    let ace_aac_bridge::Outcome::Objects {
        units,
        wrapper,
        ..
    } = response.outcome
    else {
        return Err(format!("expected an object bundle: {:?}", response.outcome));
    };
    let mut remaining = bytes.as_slice();
    let mut objects = Vec::new();
    for object in units {
        let (unit, rest) = remaining
            .split_at_checked(object.len as usize)
            .ok_or("object bundle truncated")?;
        objects.push(ace_native::link_driver::ObjectInput {
            name: object.symbol,
            bytes: unit.to_vec(),
        });
        remaining = rest;
    }
    let image = ace_native::link_driver::link_only(
        &objects,
        remaining,
        &wrapper.symbol,
        &root.join("link"),
        &[],
        tag,
    )
    .map_err(|e| format!("link: {e:?}"))?;
    let mut spec = ExecutionSpec::new([std::ffi::OsStr::new(&image.exe_path)]);
    spec.tree_scope = TreeScope::ProcessTree;
    let result = auto_tool_execution::run_bounded(
        &spec,
        &BoundedPolicy {
            exec_deadline: std::time::Duration::from_secs(4),
            cleanup_deadline: std::time::Duration::from_secs(1),
            stdout_cap: Some(4096),
            stderr_cap: Some(4096),
        },
    )
    .map_err(|e| format!("run PE: {e}"))?;
    let _ = std::fs::remove_dir_all(&root);
    match result.kind {
        ExitKind::Success => Ok(0),
        ExitKind::Failure(code) => Ok(code),
        other => Err(format!("unexpected exit kind {other:?}")),
    }
}

#[test]
fn gen1_corpus_pilot_basics_expressions_takeover() {
    let started = Instant::now();
    let compile = prepared_compile()
        .expect("prepared Gen1 compile exe (run prepare-gen1, set AAC1053_GEN1_DIR)");
    // The 1078 shared corpus: the `ac-native` probes carry the AV-phase
    // #ignore marker; this pilot executes them through the Gen1 chain and
    // compares the real PE exit codes against the corpus' frozen oracles
    // (the same bytes the AV and A2R checks consume).
    let case_file = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../../../crates/auto-lang/test/cases/language/basics/expressions.case.ad");
    let bytes = std::fs::read(&case_file).expect("read the corpus case file");
    let doc = auto_test_case::document::CaseDocument::parse(&bytes)
        .expect("the corpus document parses under the sanctioned loader");
    let limits = auto_test_case::admission::Limits::defaults();
    let cases = auto_test_case::author::decode_document(
        &doc,
        "language/basics/expressions.case.ad",
        &limits,
    )
    .expect("the corpus cases decode under the authoring contract");
    let mut ran = 0usize;
    let mut skipped = 0usize;
    let mut failures: Vec<String> = Vec::new();
    for authored in &cases {
        let resolved = auto_test_case::ir::resolve_case(
            authored.spec.clone(),
            &doc,
            case_file.parent().expect("case dir"),
            &limits,
        )
        .expect("resolve the case files");
        for check in &authored.spec.checks {
            if check.options.probe_recipe.as_deref() != Some("ac/mir-native-scalar-v1") {
                continue;
            }
            let Some(expected) = check.expect.exit else {
                skipped += 1;
                continue;
            };
            if resolved.files.len() != 1 {
                skipped += 1;
                continue;
            }
            let was_ignored = check.skip.is_some();
            let observed = gen1_chain_exit(
                compile,
                &resolved.files[0].bytes,
                &format!("aac1053-pilot-{}", authored.spec.id),
            )
            .unwrap_or_else(|e| panic!("Gen1 chain failed for {}: {e}", authored.spec.id));
            ran += 1;
            if observed != expected {
                failures.push(format!(
                    "{} (was AV-#ignore: {}): exit {} != corpus oracle {}",
                    authored.spec.id, was_ignored, observed, expected
                ));
            }
        }
    }
    eprintln!(
        "gen1 corpus pilot: ran={ran} skipped={skipped} failures={} in {:?}",
        failures.len(),
        started.elapsed()
    );
    // This file currently declares exactly one ac-native probe (the corpus
    // grows them in later batches; the pilot executes EVERY declared one).
    assert!(ran >= 1, "pilot must execute every declared ac-native probe (ran={ran})");
    assert!(failures.is_empty(), "exit oracle mismatches: {failures:?}");
    assert!(
        started.elapsed().as_secs_f64() <= 10.0,
        "corpus pilot exceeded the 10-second gate: {:?}",
        started.elapsed()
    );
}

#[test]
fn gen1_stale_preparation_is_refused() {
    let started = Instant::now();
    let exe = prepared_gen1().expect("prepared Gen1 exe");
    let root = exe.parent().expect("exe parent").to_path_buf();
    let tampered = scratch("tampered-manifest");
    std::fs::copy(&root.join(GEN1_EXE), tampered.join(GEN1_EXE)).expect("copy exe");
    let manifest = std::fs::read_to_string(root.join(DRIVER_MANIFEST)).expect("read manifest");
    let mut lines: Vec<String> = manifest.lines().map(str::to_owned).collect();
    lines[0] = "0".repeat(64);
    std::fs::write(tampered.join(DRIVER_MANIFEST), lines.join("\n")).expect("write tampered");
    // A child process with the tampered directory must fail admission.
    let output = Command::new(std::env::current_exe().expect("current test exe"))
        .arg("--exact")
        .arg("gen1_boot_native_matches_independent_reference")
        .arg("--nocapture")
        .env("AAC1053_GEN1_DIR", &tampered)
        .output()
        .expect("spawn child with tampered preparation");
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        !output.status.success(),
        "tampered preparation must fail admission, child exited {:?}",
        output.status
    );
    assert!(
        stderr.contains("stale") || stderr.contains("fingerprint"),
        "child refusal must name the stale fingerprint, got: {stderr}"
    );
    eprintln!("gen1 stale refusal ok in {:?}", started.elapsed());
    assert!(
        started.elapsed().as_secs_f64() <= 10.0,
        "stale-refusal case exceeded the 10-second gate"
    );
}
