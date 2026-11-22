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

use auto_tool_execution::{run_command_capture, ExitKind, TreeScope};

pub const GEN1_EXE: &str = "aac1053-gen1.exe";
pub const DRIVER_MANIFEST: &str = "driver.manifest";
pub const GEN1_MAIN_SOURCE: &str = include_str!("../gen1_main.at");

/// The whole subject tree S: (module stem, source bytes). `main.at` is
/// appended per build; `hir_schema_data` is synthesized below.
pub fn gen1_modules() -> Vec<(String, String)> {
    let mut files: Vec<(String, String)> = vec![
        ("syntax_cst".into(), include_str!("../../aac/syntax/syntax_cst.at").into()),
        ("syntax_format_edit".into(), include_str!("../../aac/syntax/syntax_format_edit.at").into()),
        ("syntax_frontend".into(), include_str!("../../aac/syntax/syntax_frontend.at").into()),
        ("syntax_fstr".into(), include_str!("../../aac/syntax/syntax_fstr.at").into()),
        ("syntax_grammar_data".into(), include_str!("../../aac/syntax/syntax_grammar_data.at").into()),
        ("syntax_incremental".into(), include_str!("../../aac/syntax/syntax_incremental.at").into()),
        ("syntax_lexer".into(), include_str!("../../aac/syntax/syntax_lexer.at").into()),
        ("syntax_parse".into(), include_str!("../../aac/syntax/syntax_parse.at").into()),
        ("syntax_program".into(), include_str!("../../aac/syntax/syntax_program.at").into()),
        ("syntax_smoke".into(), include_str!("../../aac/syntax/syntax_smoke.at").into()),
        ("syntax_snapshot".into(), include_str!("../../aac/syntax/syntax_snapshot.at").into()),
        ("source_ast".into(), include_str!("../../aac/source/source_ast.at").into()),
        ("source_catalog".into(), include_str!("../../aac/source/source_catalog.at").into()),
        ("source_position".into(), include_str!("../../aac/source/source_position.at").into()),
        ("binding".into(), include_str!("../../aac/semantics/binding.at").into()),
        ("choices".into(), include_str!("../../aac/semantics/choices.at").into()),
        ("declarations".into(), include_str!("../../aac/semantics/declarations.at").into()),
        ("facts_report".into(), include_str!("../../aac/semantics/facts_report.at").into()),
        ("float_literal".into(), include_str!("../../aac/semantics/float_literal.at").into()),
        ("foreign_contract".into(), include_str!("../../aac/semantics/foreign_contract.at").into()),
        ("foreign_json".into(), include_str!("../../aac/semantics/foreign_json.at").into()),
        ("initializers".into(), include_str!("../../aac/semantics/initializers.at").into()),
        ("nominal_layout".into(), include_str!("../../aac/semantics/nominal_layout.at").into()),
        ("numeric_literal".into(), include_str!("../../aac/semantics/numeric_literal.at").into()),
        ("patterns".into(), include_str!("../../aac/semantics/patterns.at").into()),
        ("semantic_tree".into(), include_str!("../../aac/semantics/semantic_tree.at").into()),
        ("typecheck".into(), include_str!("../../aac/semantics/typecheck.at").into()),
        ("types".into(), include_str!("../../aac/semantics/types.at").into()),
        ("control".into(), include_str!("../../aac/semantics/control/control.at").into()),
        ("cte".into(), include_str!("../../aac/semantics/cte/cte.at").into()),
        ("domains".into(), include_str!("../../aac/semantics/domains/domains.at").into()),
        ("instances_core".into(), include_str!("../../aac/semantics/instances/instances_core.at").into()),
        ("instances_solve".into(), include_str!("../../aac/semantics/instances/instances_solve.at").into()),
        ("instances_expand".into(), include_str!("../../aac/semantics/instances/instances_expand.at").into()),
        ("meta".into(), include_str!("../../aac/semantics/meta/meta.at").into()),
        ("resources".into(), include_str!("../../aac/semantics/resources/resources.at").into()),
        ("core_atom_bound".into(), include_str!("../../aac/hir/core_atom_bound.at").into()),
        ("core_globals".into(), include_str!("../../aac/hir/core_globals.at").into()),
        ("core_hir".into(), include_str!("../../aac/hir/core_hir.at").into()),
        ("core_ops".into(), include_str!("../../aac/hir/core_ops.at").into()),
        ("core_records".into(), include_str!("../../aac/hir/core_records.at").into()),
        ("base_abc".into(), include_str!("../../aac/mir/base_abc.at").into()),
        ("base_ir".into(), include_str!("../../aac/mir/base_ir.at").into()),
        ("lower_base".into(), include_str!("../../aac/mir/lower_base.at").into()),
        ("verify_base".into(), include_str!("../../aac/mir/verify_base.at").into()),
        ("client".into(), include_str!("../../aac/host/bridge/client.at").into()),
        ("factory".into(), include_str!("../../aac/host/cte-entry/factory.at").into()),
        ("invoke".into(), include_str!("../../aac/host/cte-entry/invoke.at").into()),
        ("host_client".into(), include_str!("../../aac/host/host_client.at").into()),
        ("host_diag".into(), include_str!("../../aac/host/host_diag.at").into()),
        ("host_files".into(), include_str!("../../aac/host/host_files.at").into()),
        ("host_main".into(), include_str!("../../aac/host/host_main.at").into()),
        ("host_publish".into(), include_str!("../../aac/host/host_publish.at").into()),
        ("store".into(), include_str!("../../aac/store/store.at").into()),
        ("driver".into(), include_str!("../../aac/driver/driver.at").into()),
        ("package".into(), include_str!("../../aac/driver/package.at").into()),
        ("query".into(), include_str!("../../aac/driver/query.at").into()),
        ("atom_data".into(), include_str!("../../aac/support/atom_data.at").into()),
        ("codec".into(), include_str!("../../aac/support/codec.at").into()),
        ("digest".into(), include_str!("../../aac/support/digest.at").into()),
        // stem disambiguation, see module header: same bytes, unique path.
        ("support_driver".into(), include_str!("../../aac/support/driver.at").into()),
        ("hir_schema_data".into(), hir_schema_data()),
    ];
    files.sort_by(|a, b| a.0.cmp(&b.0));
    files
}

/// The synthesized schema module every prior lane provides for
/// `app.hir_schema_data.*` (frozen schema files, byte-identical framing).
fn hir_schema_data() -> String {
    format!(
        "pub fn core_schema() Str {{ return {:?} }}\npub fn scalar_schema() Str {{ return {:?} }}\npub fn global_schema() Str {{ return {:?} }}\n",
        include_str!("../../crates/core/schema/core-i32.atom"),
        include_str!("../../crates/core/schema/scalar-i32.atom"),
        include_str!("../../crates/core/schema/text-record.atom")
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
    use ace_core::source::packages::{stdlib_assembled_files, stdlib_sys_files};

    let mut hash = blake3::Hasher::new();
    let mut files = gen1_modules();
    files.push(("main.at".to_owned(), GEN1_MAIN_SOURCE.to_owned()));
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
        include_str!("../../Cargo.lock"),
    ] {
        fingerprint_part(&mut hash, source.as_bytes());
    }
    hash.finalize()
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
pub fn compile_gen1() -> Result<ace_core::source::packages::PackageProduct, String> {
    let mut files = gen1_modules();
    files.push(("main.at".to_owned(), GEN1_MAIN_SOURCE.to_owned()));
    ace_aac_host_conformance::sys_product_backend("aac1053-gen1", &files, ace_core::source::packages::AssemblyBackend::Ac)
}

/// Build the Gen1 native executable into `root` and write the identity
/// manifest. `runtime` is the pinned collections-runtime library
/// (`ACE_AAC_COLLECTIONS_LIB`).
pub fn prepare_gen1(
    root: &Path,
    runtime: &Path,
) -> Result<PathBuf, Box<dyn std::error::Error>> {
    let runtime_identity = ace_native::collections_runtime::file_hash(runtime)?;
    std::fs::create_dir(root)?;
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
    let tool = Command::new(std::env::var_os("RUSTC").unwrap_or_else(|| "rustc".into()))
        .arg("-vV")
        .output()?;
    if !tool.status.success() {
        return Err(format!("read tool identity: {}", tool.status).into());
    }
    let manifest = format!(
        "{}\n{}\n{}\n{}",
        gen1_fingerprint(),
        blake3::hash(&std::fs::read(&exe)?),
        runtime_identity,
        String::from_utf8(tool.stdout)?,
    );
    std::fs::write(root.join(DRIVER_MANIFEST), manifest)?;
    Ok(exe)
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
