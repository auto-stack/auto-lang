//! PLAN-1053 T-02 (AC-01/02/03): the whole-subject Gen1 build.
//!
//! `gen1_modules()` is the complete AAC subject tree S — every
//! `experimental/ace/aac/**/*.at` module — assembled as ONE package and
//! compiled by the bound Gen0 toolchain (Rust AC frontend, assembled
//! stdlib/sys lane). The `prepare-gen1` bin lowers the checked product
//! through the shared Auto MIR into per-function COFF and links a real
//! Windows PE against the pinned collections runtime: the Gen1 native AAC
//! executable. A source-bound manifest (whole-tree fingerprint, executable
//! digest, runtime identity, tool identity) is the admission contract the
//! daily tests consume — they never build the executable themselves.
//!
//! Assembly note: `aac/support/driver.at` and `aac/driver/driver.at` share
//! the file stem `driver`. The packages lane binds the module path from the
//! file stem, and nothing in S imports `app.driver`, so the support driver
//! is included under the unique stem `support_driver` (same bytes, distinct
//! module path). `aac/hir/core_hir.at` imports `app.hir_schema_data.*`,
//! which every prior lane synthesizes from the frozen schema files; the
//! same synthesis is reproduced here.

use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::OnceLock;

use ace_source_query::modules::packages::PackageProduct;
use auto_tool_execution::{run_command_capture, ExitKind, TreeScope};

pub const GEN1_EXE: &str = "aac1053-gen1.exe";
pub const DRIVER_MANIFEST: &str = "driver.manifest";
pub const GEN1_MAIN_SOURCE: &str = include_str!("../gen1_main.at");
/// The Gen1 compile entry: PLAN-1047's delivered whole-pipeline stage
/// driver (parse→bind→infer→checked HIR→typed MIR, project mode, foreign
/// admission, CT read-set), reused verbatim as the `app::main` of the
/// whole-subject compile executable — the sanctioned stage-driver reuse;
/// the 1053 crate adds the whole-S assembly, native link and identity.
pub const COMPILE_EXE: &str = "aac1053-compile.exe";
pub const COMPILE_MANIFEST: &str = "compile.manifest";
pub const COMPILE_DRIVER_SOURCE: &str = include_str!("../../aac-syntax/semantic_driver.at");
/// The 1051 machine-backend service, part of the Gen1 chain's declared
/// tools: prepare-gen1 copies it into the preparation directory and pins
/// its digest in `tools.manifest` so archive/debug group runners (which
/// remap the test binaries away from the target dir) admit the same
/// service identity.
pub const BRIDGE_EXE: &str = "ace-aac-bridge.exe";
pub const TOOLS_MANIFEST: &str = "tools.manifest";

/// The whole subject tree S: (module stem, source bytes). `main.at` is
/// appended per build; `hir_schema_data` is synthesized below.
pub fn gen1_modules() -> Vec<(String, String)> {
    let mut files: Vec<(String, String)> = vec![
        ("syntax_cst.at".into(), include_str!("../../../aac/syntax/syntax_cst.at").into()),
        ("syntax_format_edit.at".into(), include_str!("../../../aac/syntax/syntax_format_edit.at").into()),
        ("syntax_frontend.at".into(), include_str!("../../../aac/syntax/syntax_frontend.at").into()),
        ("syntax_fstr.at".into(), include_str!("../../../aac/syntax/syntax_fstr.at").into()),
        ("syntax_grammar_data.at".into(), include_str!("../../../aac/syntax/syntax_grammar_data.at").into()),
        ("syntax_incremental.at".into(), include_str!("../../../aac/syntax/syntax_incremental.at").into()),
        ("syntax_lexer.at".into(), include_str!("../../../aac/syntax/syntax_lexer.at").into()),
        ("syntax_parse.at".into(), include_str!("../../../aac/syntax/syntax_parse.at").into()),
        ("syntax_program.at".into(), include_str!("../../../aac/syntax/syntax_program.at").into()),
        ("syntax_smoke.at".into(), include_str!("../../../aac/syntax/syntax_smoke.at").into()),
        ("syntax_snapshot.at".into(), include_str!("../../../aac/syntax/syntax_snapshot.at").into()),
        ("source_ast.at".into(), include_str!("../../../aac/source/source_ast.at").into()),
        ("source_catalog.at".into(), include_str!("../../../aac/source/source_catalog.at").into()),
        ("source_position.at".into(), include_str!("../../../aac/source/source_position.at").into()),
        ("binding.at".into(), include_str!("../../../aac/semantics/binding.at").into()),
        ("choices.at".into(), include_str!("../../../aac/semantics/choices.at").into()),
        ("declarations.at".into(), include_str!("../../../aac/semantics/declarations.at").into()),
        ("facts_report.at".into(), include_str!("../../../aac/semantics/facts_report.at").into()),
        ("float_literal.at".into(), include_str!("../../../aac/semantics/float_literal.at").into()),
        ("foreign_contract.at".into(), include_str!("../../../aac/semantics/foreign_contract.at").into()),
        ("foreign_json.at".into(), include_str!("../../../aac/semantics/foreign_json.at").into()),
        ("initializers.at".into(), include_str!("../../../aac/semantics/initializers.at").into()),
        ("nominal_layout.at".into(), include_str!("../../../aac/semantics/nominal_layout.at").into()),
        ("numeric_literal.at".into(), include_str!("../../../aac/semantics/numeric_literal.at").into()),
        ("patterns.at".into(), include_str!("../../../aac/semantics/patterns.at").into()),
        ("semantic_tree.at".into(), include_str!("../../../aac/semantics/semantic_tree.at").into()),
        ("typecheck.at".into(), include_str!("../../../aac/semantics/typecheck.at").into()),
        ("types.at".into(), include_str!("../../../aac/semantics/types.at").into()),
        ("control.at".into(), include_str!("../../../aac/semantics/control/control.at").into()),
        ("cte.at".into(), include_str!("../../../aac/semantics/cte/cte.at").into()),
        ("domains.at".into(), include_str!("../../../aac/semantics/domains/domains.at").into()),
        ("instances_core.at".into(), include_str!("../../../aac/semantics/instances/instances_core.at").into()),
        ("instances_solve.at".into(), include_str!("../../../aac/semantics/instances/instances_solve.at").into()),
        ("instances_expand.at".into(), include_str!("../../../aac/semantics/instances/instances_expand.at").into()),
        ("meta.at".into(), include_str!("../../../aac/semantics/meta/meta.at").into()),
        ("resources.at".into(), include_str!("../../../aac/semantics/resources/resources.at").into()),
        ("core_atom_bound.at".into(), include_str!("../../../aac/hir/core_atom_bound.at").into()),
        ("core_globals.at".into(), include_str!("../../../aac/hir/core_globals.at").into()),
        ("core_hir.at".into(), include_str!("../../../aac/hir/core_hir.at").into()),
        ("core_ops.at".into(), include_str!("../../../aac/hir/core_ops.at").into()),
        ("core_records.at".into(), include_str!("../../../aac/hir/core_records.at").into()),
        ("base_abc.at".into(), include_str!("../../../aac/mir/base_abc.at").into()),
        ("base_ir.at".into(), include_str!("../../../aac/mir/base_ir.at").into()),
        ("lower_base.at".into(), include_str!("../../../aac/mir/lower_base.at").into()),
        ("verify_base.at".into(), include_str!("../../../aac/mir/verify_base.at").into()),
        ("client.at".into(), include_str!("../../../aac/host/bridge/client.at").into()),
        ("factory.at".into(), include_str!("../../../aac/host/cte-entry/factory.at").into()),
        ("invoke.at".into(), include_str!("../../../aac/host/cte-entry/invoke.at").into()),
        ("host_client.at".into(), include_str!("../../../aac/host/host_client.at").into()),
        ("host_diag.at".into(), include_str!("../../../aac/host/host_diag.at").into()),
        ("host_files.at".into(), include_str!("../../../aac/host/host_files.at").into()),
        ("host_main.at".into(), include_str!("../../../aac/host/host_main.at").into()),
        ("host_publish.at".into(), include_str!("../../../aac/host/host_publish.at").into()),
        ("store.at".into(), include_str!("../../../aac/store/store.at").into()),
        ("driver.at".into(), include_str!("../../../aac/driver/driver.at").into()),
        ("package.at".into(), include_str!("../../../aac/driver/package.at").into()),
        ("query.at".into(), include_str!("../../../aac/driver/query.at").into()),
        ("atom_data.at".into(), include_str!("../../../aac/support/atom_data.at").into()),
        ("codec.at".into(), include_str!("../../../aac/support/codec.at").into()),
        ("digest.at".into(), include_str!("../../../aac/support/digest.at").into()),
        // stem disambiguation, see module header: same bytes, unique path.
        ("support_driver.at".into(), include_str!("../../../aac/support/driver.at").into()),
        // crate-local boot reduction helpers (not part of S).
        ("boot_lib.at".into(), include_str!("../boot_lib.at").into()),
        ("hir_schema_data.at".into(), hir_schema_data()),
    ];
    files.sort_by(|a, b| a.0.cmp(&b.0));
    files
}

/// The synthesized schema module every prior lane provides for
/// `app.hir_schema_data.*` (frozen schema files, byte-identical framing).
fn hir_schema_data() -> String {
    format!(
        "pub fn core_schema() Str {{ return {:?} }}\npub fn scalar_schema() Str {{ return {:?} }}\npub fn global_schema() Str {{ return {:?} }}\n",
        include_str!("../../../crates/core/schema/core-i32.atom"),
        include_str!("../../../crates/core/schema/scalar-i32.atom"),
        include_str!("../../../crates/core/schema/text-record.atom")
    )
}

fn fingerprint_part(hash: &mut blake3::Hasher, value: &[u8]) {
    hash.update(&(value.len() as u64).to_le_bytes());
    hash.update(value);
}

/// Whole-build identity: every module byte, the assembled stdlib/sys bytes,
/// the host stack contract, the boot entry source, the prepare bin source,
/// and the dependency lock.
pub fn gen1_fingerprint() -> blake3::Hash {
    fingerprint_of(GEN1_MAIN_SOURCE, gen1_modules())
}

static NEXT: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);

/// Scratch root: the single plan-artifacts home when set
/// (`AUTO_PLAN_ARTIFACTS`), else the OS temp dir.
pub fn scratch_root() -> PathBuf {
    std::env::var_os("AUTO_PLAN_ARTIFACTS")
        .map(PathBuf::from)
        .unwrap_or_else(std::env::temp_dir)
}

pub fn scratch(tag: &str) -> PathBuf {
    let safe: String = tag
        .chars()
        .map(|c| if c.is_ascii_alphanumeric() { c } else { '-' })
        .collect();
    let d = scratch_root().join(format!(
        "aac1053-{safe}-{}-{}",
        std::process::id(),
        NEXT.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
    ));
    let _ = std::fs::remove_dir_all(&d);
    std::fs::create_dir_all(&d).unwrap();
    d
}

/// Assemble + compile the whole subject S plus the boot entry on the
/// sys/aggregates lane (Gen0 frontend window — the produced checked module
/// is the authority the native lowering consumes).
pub fn compile_gen1() -> Result<PackageProduct, String> {
    let mut files = gen1_modules();
    files.push(("main.at".to_owned(), GEN1_MAIN_SOURCE.to_owned()));
    product_from("aac1053-gen1", files)
}

/// The compile entry product: PLAN-1047's delivered semantic module set plus
/// its pipeline driver as the entry, emitted on the A2R+rustc lane — the
/// same lane every delivered stage driver (1046/1047) links on.
/// NOTE (T-03 finding): the AC(mn) machine lane cannot currently lower this
/// entry: `core_atom_bound.bind_body`'s closure ownership analysis needs
/// ~34 MiB, above the frozen 16 MiB lowering admission
/// (`MAX_VERIFY_ANALYSIS_BYTES`, contract.rs:290; the 64 MiB tier is gated
/// on the AGE elem-view capability, which this product must not claim). The
/// mn lane has never linked the semantic pipeline (all prior semantic
/// drivers are A2R+rustc exes); the gap is recorded for the T-09 整机
/// combination / Design59 owner. The compiler TOOL riding the A2R emitter
/// keeps every pipeline algorithm in Auto; the programs it compiles still
/// target AC(mn) through the 1051 bridge service.
pub fn compile_gen1_compile() -> Result<PackageProduct, String> {
    use ace_core::source::packages::{
        AssemblyBackend, PackageAssemblyContext, ProviderChoice, TargetOs,
    };
    use ace_source_query::modules::packages::assemble_stdlib_sys_inputs;

    let mut files = ace_aac_syntax_conformance::semantic_modules();
    files.push(("main.at".to_owned(), COMPILE_DRIVER_SOURCE.to_owned()));
    let mut ctx = PackageAssemblyContext::new(
        AssemblyBackend::A2R,
        ProviderChoice::Auto,
        TargetOs::Windows,
    );
    let inputs = assemble_stdlib_sys_inputs(&files, "app::main", &mut ctx)
        .map_err(|e| format!("aac1053-compile assemble: {e}"))?;
    ace_source_query::modules::packages::compile_assembled_aggregates_inputs(
        &inputs,
        "app::main",
        ace_aac_syntax_conformance::BUDGET,
        &ctx,
    )
    .map_err(|ds| {
        format!(
            "aac1053-compile compile: {} {} {} @{}..{}",
            ds[0].module,
            ds[0].diagnostic.code,
            ds[0].diagnostic.message,
            ds[0].diagnostic.span.start,
            ds[0].diagnostic.span.end
        )
    })
}

fn product_from(tag: &str, files: Vec<(String, String)>) -> Result<PackageProduct, String> {
    ace_aac_host_conformance::sys_product_backend(tag, &files, ace_core::source::packages::AssemblyBackend::Ac)
}

fn fingerprint_of(main_source: &str, modules: Vec<(String, String)>) -> blake3::Hash {
    use ace_core::source::packages::{stdlib_assembled_files, stdlib_sys_files};

    let mut hash = blake3::Hasher::new();
    let mut files = modules;
    files.push(("main.at".to_owned(), main_source.to_owned()));
    files.sort_by(|a, b| a.0.cmp(&b.0));
    for (name, source) in files {
        fingerprint_part(&mut hash, name.as_bytes());
        fingerprint_part(&mut hash, source.as_bytes());
    }
    for (name, source) in stdlib_assembled_files()
        .into_iter()
        .chain(stdlib_sys_files())
    {
        fingerprint_part(&mut hash, name.as_bytes());
        fingerprint_part(&mut hash, source.as_bytes());
    }
    for source in [
        "windows-x64/a2r/dev/stack-16777216/v1",
        include_str!("bin/prepare-gen1.rs"),
        include_str!("../../../Cargo.lock"),
    ] {
        fingerprint_part(&mut hash, source.as_bytes());
    }
    hash.finalize()
}

/// Build the Gen1 native executables into `root` and write their identity
/// manifests: the whole-subject boot exe and the whole-subject compile
/// exe (1047 pipeline driver entry). `runtime` is the pinned
/// collections-runtime library (`ACE_AAC_COLLECTIONS_LIB`).
pub fn prepare_gen1(
    root: &Path,
    runtime: &Path,
) -> Result<PathBuf, Box<dyn std::error::Error>> {
    let runtime_identity = ace_native::collections_runtime::file_hash(runtime)?;
    std::fs::create_dir(root)?;
    let tool = Command::new(std::env::var_os("RUSTC").unwrap_or_else(|| "rustc".into()))
        .arg("-vV")
        .output()?;
    if !tool.status.success() {
        return Err(format!("read tool identity: {}", tool.status).into());
    }
    let tool_id = String::from_utf8(tool.stdout)?;

    let product = compile_gen1().map_err(|e| format!("compile whole AAC subject S: {e}"))?;
    let built = ace_native::driver::MnDriver::ephemeral()
        .build_hosted_collections_checked(
            &product.compiled.checked,
            "d_main",
            &root.join("build"),
            runtime,
            &runtime_identity,
            1053,
        )
        .map_err(|e| format!("link Gen1 native AAC: {e:?}"))?;
    let exe = root.join(GEN1_EXE);
    std::fs::copy(&built.exe_path, &exe)?;
    let manifest = format!(
        "{}\n{}\n{}\n{}",
        fingerprint_of(GEN1_MAIN_SOURCE, gen1_modules()),
        blake3::hash(&std::fs::read(&exe)?),
        runtime_identity,
        tool_id,
    );
    std::fs::write(root.join(DRIVER_MANIFEST), manifest)?;

    let compile = match compile_gen1_compile() {
        Ok(product) => Some(product),
        Err(err) => {
            // Named, non-silent degradation: the shared A2R emitter
            // currently rejects the AAC pipeline set (see the function's
            // NOTE and PLAN-1053 §10). The boot exe and its manifest are
            // already prepared; the compile manifest is withheld so
            // `prepared_compile` refuses with a named preparation failure
            // instead of admitting a stale tool.
            eprintln!("aac1053-compile preparation SKIPPED (named external blocker): {err}");
            None
        }
    };
    if let Some(compile) = compile {
    let compile_prep: Result<(), Box<dyn std::error::Error>> = (|| {
    // A2R emit + host rustc link — the delivered stage-driver lane (the
    // AC(mn) lane cannot lower this entry within the frozen admission
    // budget; see the function's NOTE).
    let generated = root.join("generated-compile");
    std::fs::create_dir(&generated)?;
    let target = ace_a2x_store::modules::packages::PackageA2xTarget::cold_aggregates(
        &root.join("store-compile"),
        "aac1053-compile",
        "app",
        16 << 20,
        ace_a2x_rust::RustTargetProvider::v1(),
        &compile,
    )
    .map_err(|e| format!("emit aac1053-compile: {e:?}"))?;
    for (path, source) in &target.current.files {
        std::fs::write(generated.join(path), source)?;
    }
    let cexe = root.join(COMPILE_EXE);
    let output = Command::new(std::env::var_os("RUSTC").unwrap_or_else(|| "rustc".into()))
        .args(["--edition", "2021", "-C", "link-arg=/STACK:16777216"])
        .arg("main.rs")
        .arg("-o")
        .arg(&cexe)
        .current_dir(&generated)
        .output()?;
    std::fs::write(root.join("rustc-compile.log"), &output.stderr)?;
    if !output.status.success() {
        return Err(format!("compile-entry rustc failed: {}", output.status).into());
    }
    let cmanifest = format!(
        "{}\n{}\n{}\n{}",
        fingerprint_of(COMPILE_DRIVER_SOURCE, ace_aac_syntax_conformance::semantic_modules()),
        blake3::hash(&std::fs::read(&cexe)?),
        runtime_identity,
        tool_id,
    );
    std::fs::write(root.join(COMPILE_MANIFEST), cmanifest)?;

    // The declared machine-backend service, pinned alongside the exes so
    // archive/remap group runners admit the same tool identity.
    let bridge_src = std::env::var_os("ACE_AAC1053_BRIDGE")
        .map(PathBuf::from)
        .filter(|p| p.is_absolute() && p.is_file())
        .or_else(|| {
            std::env::current_exe()
                .ok()
                .and_then(|p| p.parent().map(|d| d.join(BRIDGE_EXE)))
                .filter(|p| p.is_file())
        })
        .ok_or("set ACE_AAC1053_BRIDGE to the built ace-aac-bridge.exe (or build it in this target dir)")?;
    std::fs::copy(&bridge_src, root.join(BRIDGE_EXE))?;
    let tools_manifest = format!(
        "ace-aac-bridge\n{}\n{}",
        blake3::hash(&std::fs::read(root.join(BRIDGE_EXE))?),
        bridge_src.display(),
    );
    std::fs::write(root.join(TOOLS_MANIFEST), tools_manifest)?;
    Ok(())
    })();
    if let Err(err) = compile_prep {
        let _ = std::fs::remove_file(root.join(COMPILE_MANIFEST));
        let _ = std::fs::remove_file(root.join(COMPILE_EXE));
        eprintln!("aac1053-compile preparation SKIPPED (named external blocker): {err}");
    }
    }
    Ok(exe)
}

fn admit(
    cache: &'static OnceLock<Result<PathBuf, String>>,
    env: &str,
    manifest: &str,
    executable: &str,
    fingerprint: fn() -> blake3::Hash,
) -> Result<&'static Path, String> {
    cache
        .get_or_init(|| {
            let root = PathBuf::from(
                std::env::var_os(env)
                    .ok_or_else(|| format!("prepare the Gen1 exe (prepare-gen1) and set {env}"))?,
            );
            if !root.is_absolute() {
                return Err(format!("{env} must be an absolute preparation directory"));
            }
            let text = std::fs::read_to_string(root.join(manifest))
                .map_err(|err| format!("read Gen1 manifest: {err}"))?;
            let mut fields = text.lines();
            let expected = fingerprint().to_hex();
            if fields.next() != Some(expected.as_str()) {
                return Err("Gen1 preparation is stale (source fingerprint differs)".into());
            }
            let exe = root.join(executable);
            let bytes = std::fs::read(&exe)
                .map_err(|err| format!("read Gen1 executable: {err}"))?;
            if fields.next() != Some(blake3::hash(&bytes).to_hex().as_str()) {
                return Err("Gen1 executable digest differs from preparation".into());
            }
            Ok(exe)
        })
        .as_deref()
        .map_err(Clone::clone)
}

static PREPARED_COMPILE: OnceLock<Result<PathBuf, String>> = OnceLock::new();
static PREPARED_BRIDGE: OnceLock<Result<PathBuf, String>> = OnceLock::new();

/// Admit the prepared machine-backend service (see `TOOLS_MANIFEST`).
pub fn prepared_bridge() -> Result<&'static Path, String> {
    PREPARED_BRIDGE
        .get_or_init(|| {
            let root = PathBuf::from(
                std::env::var_os("AAC1053_GEN1_DIR")
                    .ok_or("prepare the Gen1 exe (prepare-gen1) and set AAC1053_GEN1_DIR")?,
            );
            if !root.is_absolute() {
                return Err("AAC1053_GEN1_DIR must be an absolute preparation directory".into());
            }
            let text = std::fs::read_to_string(root.join(TOOLS_MANIFEST))
                .map_err(|err| format!("read tools manifest: {err}"))?;
            let mut fields = text.lines();
            if fields.next() != Some("ace-aac-bridge") {
                return Err("tools manifest does not declare ace-aac-bridge".into());
            }
            let exe = root.join(BRIDGE_EXE);
            let bytes =
                std::fs::read(&exe).map_err(|err| format!("read bridge executable: {err}"))?;
            if fields.next() != Some(blake3::hash(&bytes).to_hex().as_str()) {
                return Err("bridge executable digest differs from preparation".into());
            }
            Ok(exe)
        })
        .as_deref()
        .map_err(Clone::clone)
}

/// Admit the prepared whole-subject COMPILE executable (same contract as
/// the boot exe, separate manifest).
pub fn prepared_compile() -> Result<&'static Path, String> {
    admit(
        &PREPARED_COMPILE,
        "AAC1053_GEN1_DIR",
        COMPILE_MANIFEST,
        COMPILE_EXE,
        || fingerprint_of(COMPILE_DRIVER_SOURCE, ace_aac_syntax_conformance::semantic_modules()),
    )
}

/// Run the Gen1 compile entry once over `input` in the given HIR mode
/// ("verify" | "mir" | ...), returning the report text and, for "mir", the
/// emitted ABC bytes (written to `abc_path` when provided).
pub fn run_compile_case(
    exe: &Path,
    input: &Path,
    output: &Path,
    hir_mode: &str,
    abc_path: Option<&Path>,
) -> Result<(String, Option<Vec<u8>>), String> {
    let started = std::time::Instant::now();
    let mut cmd = Command::new(exe);
    cmd.current_dir(input.parent().expect("input parent"))
        .env("A2X_RT_BUDGET", "33554432")
        .env("AAC1047_INPUT", input)
        .env("AAC1047_OUTPUT", output)
        .env("AAC1047_TYPED", "1")
        .env("AAC1047_HIR", hir_mode)
        .env_remove("AAC1047_PROJECT")
        .env_remove("AAC1047_STEPS")
        .env_remove("AAC1047_ARTIFACT")
        .env_remove("AAC1047_STAGED")
        .env_remove("AAC1047_FOREIGN_PATH")
        .env_remove("AAC1047_FOREIGN_AUTHORITY");
    if let Some(abc) = abc_path {
        cmd.env("AAC1055_ABC", abc);
    }
    let run = run_command_capture(cmd, TreeScope::ProcessTree, Some(std::time::Duration::from_secs(8)))
        .map_err(|err| format!("run Gen1 compile: {err}"))?;
    if !matches!(run.kind, ExitKind::Success) {
        return Err(format!(
            "Gen1 compile: {:?} {}",
            run.kind,
            run.stderr_lossy().unwrap_or_default()
        ));
    }
    if started.elapsed().as_secs() > 8 {
        return Err("Gen1 compile exceeded its 8s budget".into());
    }
    let report = std::fs::read_to_string(output)
        .map_err(|err| format!("read compile report: {err}"))?;
    let abc = abc_path
        .map(std::fs::read)
        .transpose()
        .map_err(|err| format!("read ABC bytes: {err}"))?;
    Ok((report, abc))
}

static PREPARED: OnceLock<Result<PathBuf, String>> = OnceLock::new();

/// Admit the prepared Gen1 executable: environment-provided absolute
/// preparation directory, manifest fingerprint and executable digest must
/// match the current source identity (a stale preparation is a named
/// refusal, never a silent rebuild).
pub fn prepared_gen1() -> Result<&'static Path, String> {
    PREPARED
        .get_or_init(|| {
            let root = PathBuf::from(
                std::env::var_os("AAC1053_GEN1_DIR")
                    .ok_or("prepare the Gen1 AAC exe (prepare-gen1) and set AAC1053_GEN1_DIR")?,
            );
            if !root.is_absolute() {
                return Err("AAC1053_GEN1_DIR must be an absolute preparation directory".into());
            }
            let text = std::fs::read_to_string(root.join(DRIVER_MANIFEST))
                .map_err(|err| format!("read Gen1 manifest: {err}"))?;
            let mut fields = text.lines();
            let expected = gen1_fingerprint().to_hex();
            if fields.next() != Some(expected.as_str()) {
                return Err("Gen1 preparation is stale (source fingerprint differs)".into());
            }
            let exe = root.join(GEN1_EXE);
            let bytes = std::fs::read(&exe)
                .map_err(|err| format!("read Gen1 executable: {err}"))?;
            if fields.next() != Some(blake3::hash(&bytes).to_hex().as_str()) {
                return Err("Gen1 executable digest differs from preparation".into());
            }
            Ok(exe)
        })
        .as_deref()
        .map_err(Clone::clone)
}

/// Run the Gen1 executable once and return its exit code (the boot
/// reduction). An 8-second process-tree budget bounds the run.
pub fn run_gen1_boot(exe: &Path) -> Result<i32, String> {
    let mut cmd = Command::new(exe);
    cmd.env("A2X_RT_BUDGET", "33554432");
    let run = run_command_capture(cmd, TreeScope::ProcessTree, Some(std::time::Duration::from_secs(8)))
        .map_err(|err| format!("run Gen1 boot: {err}"))?;
    match run.kind {
        ExitKind::Success => Ok(0),
        ExitKind::Failure(code) => Ok(code),
        other => Err(format!("Gen1 boot: unexpected exit kind {other:?}")),
    }
}
