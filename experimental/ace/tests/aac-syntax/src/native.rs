use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::OnceLock;
use std::time::Duration;

use auto_tool_execution::{run_command_capture, ExitKind, TreeScope};

use crate::{scratch, syntax_av_modules};

pub const NATIVE_DRIVER_SOURCE: &str = include_str!("../native_driver.at");
pub const DRIVER_EXE: &str = "aac1046-driver.exe";
pub const DRIVER_MANIFEST: &str = "driver.manifest";
pub const SEMANTIC_DRIVER_SOURCE: &str = include_str!("../semantic_driver.at");
pub const SEMANTIC_DRIVER_EXE: &str = "aac1047-driver.exe";

pub fn semantic_modules() -> Vec<(String, String)> {
    let mut files = syntax_av_modules();
    files.extend([
        (
            "atom_data.at".into(),
            include_str!("../../../aac/support/atom_data.at").into(),
        ),
        (
            "semantic_tree.at".into(),
            include_str!("../../../aac/semantics/semantic_tree.at").into(),
        ),
        (
            "binding.at".into(),
            include_str!("../../../aac/semantics/binding.at").into(),
        ),
        (
            "initializers.at".into(),
            include_str!("../../../aac/semantics/initializers.at").into(),
        ),
        (
            "declarations.at".into(),
            include_str!("../../../aac/semantics/declarations.at").into(),
        ),
        (
            "foreign_contract.at".into(),
            include_str!("../../../aac/semantics/foreign_contract.at").into(),
        ),
        (
            "foreign_json.at".into(),
            include_str!("../../../aac/semantics/foreign_json.at").into(),
        ),
        (
            "types.at".into(),
            include_str!("../../../aac/semantics/types.at").into(),
        ),
        (
            "numeric_literal.at".into(),
            include_str!("../../../aac/semantics/numeric_literal.at").into(),
        ),
        (
            "float_literal.at".into(),
            include_str!("../../../aac/semantics/float_literal.at").into(),
        ),
        (
            "typecheck.at".into(),
            include_str!("../../../aac/semantics/typecheck.at").into(),
        ),
        (
            "core_hir.at".into(),
            include_str!("../../../aac/hir/core_hir.at").into(),
        ),
        (
            "core_globals.at".into(),
            include_str!("../../../aac/hir/core_globals.at").into(),
        ),
        (
            "core_records.at".into(),
            include_str!("../../../aac/hir/core_records.at").into(),
        ),
        (
            "patterns.at".into(),
            include_str!("../../../aac/semantics/patterns.at").into(),
        ),
        (
            "nominal_layout.at".into(),
            include_str!("../../../aac/semantics/nominal_layout.at").into(),
        ),
        (
            "facts_report.at".into(),
            include_str!("../../../aac/semantics/facts_report.at").into(),
        ),
        (
            "core_atom_bound.at".into(),
            include_str!("../../../aac/hir/core_atom_bound.at").into(),
        ),
    ]);
    files.push((
        "hir_schema_data.at".into(),
        format!(
            "pub fn core_schema() Str {{ return {:?} }}\npub fn global_schema() Str {{ return {:?} }}\n",
            include_str!("../../../crates/core/schema/core-i32.atom"),
            include_str!("../../../crates/core/schema/text-record.atom")
        ),
    ));
    files
}

fn fingerprint_part(hash: &mut blake3::Hasher, value: &[u8]) {
    hash.update(&(value.len() as u64).to_le_bytes());
    hash.update(value);
}

pub fn native_fingerprint() -> blake3::Hash {
    driver_fingerprint(syntax_av_modules(), NATIVE_DRIVER_SOURCE)
}

pub fn semantic_fingerprint() -> blake3::Hash {
    driver_fingerprint(semantic_modules(), SEMANTIC_DRIVER_SOURCE)
}

fn driver_fingerprint(files: Vec<(String, String)>, driver: &str) -> blake3::Hash {
    use ace_core::source::packages::{stdlib_assembled_files, stdlib_sys_files};

    let mut hash = blake3::Hasher::new();
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
        driver,
        include_str!("bin/prepare-native.rs"),
        include_str!("../../../Cargo.lock"),
    ] {
        fingerprint_part(&mut hash, source.as_bytes());
    }
    hash.finalize()
}

static PREPARED: OnceLock<Result<PathBuf, String>> = OnceLock::new();
static SEMANTIC_PREPARED: OnceLock<Result<PathBuf, String>> = OnceLock::new();

fn prepared_exe() -> Result<&'static Path, String> {
    admit_driver(
        &PREPARED,
        "AAC1046_NATIVE_DIR",
        DRIVER_EXE,
        native_fingerprint,
    )
}

fn admit_driver(
    cache: &'static OnceLock<Result<PathBuf, String>>,
    env: &str,
    executable: &str,
    fingerprint: fn() -> blake3::Hash,
) -> Result<&'static Path, String> {
    cache
        .get_or_init(|| {
            let root = std::env::var_os(env)
                .map(PathBuf::from)
                .ok_or_else(|| format!("prepare AAC native driver and set {env}"))?;
            if !root.is_absolute() {
                return Err(format!("{env} must be an absolute preparation directory"));
            }
            let text = std::fs::read_to_string(root.join(DRIVER_MANIFEST))
                .map_err(|err| format!("read AAC driver manifest: {err}"))?;
            let mut fields = text.lines();
            let expected = fingerprint().to_hex();
            if fields.next() != Some(expected.as_str()) {
                return Err("AAC native driver preparation is stale".into());
            }
            let exe = root.join(executable);
            let bytes = std::fs::read(&exe).map_err(|err| format!("read AAC driver: {err}"))?;
            if fields.next() != Some(blake3::hash(&bytes).to_hex().as_str()) {
                return Err("AAC native executable digest differs from preparation".into());
            }
            Ok(exe)
        })
        .as_deref()
        .map_err(Clone::clone)
}

fn prepared_semantic() -> Result<&'static Path, String> {
    admit_driver(
        &SEMANTIC_PREPARED,
        "AAC1047_NATIVE_DIR",
        SEMANTIC_DRIVER_EXE,
        semantic_fingerprint,
    )
}

pub fn run_semantic_case(sample: &[u8], steps: Option<u32>) -> Result<String, String> {
    run_semantic_source(sample, steps, false, None)
}

pub fn run_type_case(sample: &[u8], steps: Option<u32>) -> Result<String, String> {
    run_semantic_source(sample, steps, true, None)
}

pub fn run_core_hir_case(sample: &[u8], mode: &str) -> Result<String, String> {
    run_semantic_source(sample, None, true, Some(mode))
}

pub fn run_global_hir_admission(source: &str, artifact: &str) -> Result<String, String> {
    let exe = prepared_semantic()?;
    with_fixture("semantic-global-hir", |root| {
        let input = root.join("input.at");
        let output = root.join("report.txt");
        let draft = root.join("artifact.atom");
        std::fs::write(&input, source).map_err(|err| format!("write source authority: {err}"))?;
        std::fs::write(&draft, artifact).map_err(|err| format!("write untrusted HIR: {err}"))?;
        let mut cmd = semantic_command(
            exe,
            root,
            &input,
            &output,
            None,
            true,
            Some("admit-globals"),
        );
        cmd.env("AAC1047_HIR_INPUT", draft);
        execute_report(cmd, &output, "semantic-global-hir")
    })
}

#[cfg(all(windows, feature = "aac-foreign-semantics"))]
pub fn run_semantic_foreign_case(
    source: &str,
    metadata: &[u8],
    authority: &str,
) -> Result<String, String> {
    let exe = prepared_semantic()?;
    with_fixture("semantic-foreign", |root| {
        let input = root.join("input.at");
        let output = root.join("report.txt");
        let page = root.join("source-declarations.json");
        std::fs::write(&input, source).map_err(|err| format!("write foreign input: {err}"))?;
        std::fs::write(&page, metadata).map_err(|err| format!("write foreign page: {err}"))?;
        let mut cmd = semantic_command(exe, root, &input, &output, None, true, Some("verify"));
        cmd.env("AAC1047_FOREIGN_PATH", page)
            .env("AAC1047_FOREIGN_AUTHORITY", authority);
        execute_report(cmd, &output, "semantic-foreign")
    })
}

#[cfg(windows)]
pub fn observe_hir_publication(
    sample: &[u8],
    old: &[u8],
    steps: Option<u32>,
    lock_destination: bool,
) -> Result<(String, Vec<u8>, bool), String> {
    use std::os::windows::fs::OpenOptionsExt;

    let exe = prepared_semantic()?;
    with_fixture("semantic-publish", |root| {
        let input = root.join("input.at");
        let output = root.join("report.txt");
        let artifact = root.join("artifact.atom");
        let staged = root.join("staged.atom");
        std::fs::write(&input, sample).map_err(|err| format!("write semantic input: {err}"))?;
        std::fs::write(&artifact, old).map_err(|err| format!("seed consumer artifact: {err}"))?;
        let locked = lock_destination
            .then(|| {
                std::fs::OpenOptions::new()
                    .read(true)
                    .share_mode(1)
                    .open(&artifact)
            })
            .transpose()
            .map_err(|err| format!("lock destination without delete sharing: {err}"))?;
        let mut cmd = semantic_command(exe, root, &input, &output, steps, true, Some("verify"));
        cmd.env("AAC1047_ARTIFACT", &artifact)
            .env("AAC1047_STAGED", &staged);
        let report = execute_report(cmd, &output, "semantic-publish")?;
        drop(locked);
        let contents = std::fs::read(artifact).map_err(|err| format!("read artifact: {err}"))?;
        Ok((report, contents, staged.exists()))
    })
}

fn semantic_command(
    exe: &Path,
    root: &Path,
    input: &Path,
    output: &Path,
    steps: Option<u32>,
    typed: bool,
    hir_mode: Option<&str>,
) -> Command {
    let mut cmd = Command::new(exe);
    cmd.current_dir(root)
        .env("AAC1047_INPUT", input)
        .env("AAC1047_OUTPUT", output)
        .env("AAC1047_TYPED", if typed { "1" } else { "0" })
        .env_remove("AAC1047_HIR")
        .env_remove("AAC1047_PROJECT")
        .env_remove("AAC1047_STEPS")
        .env_remove("AAC1047_ARTIFACT")
        .env_remove("AAC1047_STAGED")
        .env_remove("AAC1047_FOREIGN_PATH")
        .env_remove("AAC1047_FOREIGN_AUTHORITY");
    if let Some(steps) = steps {
        cmd.env("AAC1047_STEPS", steps.to_string());
    }
    if let Some(mode) = hir_mode {
        cmd.env("AAC1047_HIR", mode);
    }
    cmd
}

fn run_semantic_source(
    sample: &[u8],
    steps: Option<u32>,
    typed: bool,
    hir_mode: Option<&str>,
) -> Result<String, String> {
    let exe = prepared_semantic()?;
    with_fixture("semantic", |root| {
        let input = root.join("input.at");
        let output = root.join("report.txt");
        std::fs::write(&input, sample).map_err(|err| format!("write semantic input: {err}"))?;
        let cmd = semantic_command(exe, root, &input, &output, steps, typed, hir_mode);
        execute_report(cmd, &output, "semantic")
    })
}

pub fn run_semantic_project(modules: &[(&str, &str)]) -> Result<String, String> {
    run_project(modules, false, None, None)
}

pub fn run_type_project(modules: &[(&str, &str)]) -> Result<String, String> {
    run_project(modules, true, None, None)
}

pub fn run_core_hir_project(modules: &[(&str, &str)]) -> Result<String, String> {
    run_project(modules, true, Some("verify"), None)
}

pub fn observe_project_inventory(inventory: &str) -> Result<String, String> {
    run_project(
        &[("main", "fn main() int { return 3 }")],
        true,
        Some("verify"),
        Some(inventory),
    )
}

fn run_project(
    modules: &[(&str, &str)],
    typed: bool,
    hir_mode: Option<&str>,
    inventory: Option<&str>,
) -> Result<String, String> {
    let exe = prepared_semantic()?;
    with_fixture("semantic-project", |root| {
        let output = root.join("report.txt");
        let mut names = Vec::with_capacity(modules.len());
        for (name, source) in modules {
            if name.split('/').count() > 2
                || name.split('/').any(|part| {
                    part.is_empty() || !part.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'_')
                })
            {
                return Err(format!("invalid fixture module name: {name}"));
            }
            let input = root.join(format!("{name}.at"));
            std::fs::create_dir_all(input.parent().unwrap()).map_err(|err| err.to_string())?;
            std::fs::write(input, source).map_err(|err| err.to_string())?;
            names.push(*name);
        }
        let declared = inventory
            .map(str::to_owned)
            .unwrap_or_else(|| names.join("\n"));
        let mut cmd = Command::new(exe);
        cmd.current_dir(root)
            .env("AAC1047_PROJECT", root)
            .env("AAC1047_PACKAGE", "app")
            .env("AAC1047_INVENTORY", declared)
            .env("AAC1047_OUTPUT", &output)
            .env("AAC1047_TYPED", if typed { "1" } else { "0" })
            .env_remove("AAC1047_HIR")
            .env_remove("AAC1047_STEPS")
            .env_remove("AAC1047_ARTIFACT")
            .env_remove("AAC1047_STAGED")
            .env_remove("AAC1047_FOREIGN_PATH")
            .env_remove("AAC1047_FOREIGN_AUTHORITY");
        if let Some(mode) = hir_mode {
            cmd.env("AAC1047_HIR", mode);
        }
        execute_report(cmd, &output, "semantic-project")
    })
}

pub fn run_native_case(
    case: &str,
    profile: &str,
    sample: impl AsRef<[u8]>,
    edit: Option<(i32, i32, &str)>,
) -> Result<String, String> {
    run_native(case, profile, sample.as_ref(), edit, (0, 0))
}

pub fn run_native_projection(
    profile: &str,
    sample: &str,
    origin: (i32, i32),
    edit: Option<(i32, i32, &str)>,
) -> Result<String, String> {
    let case = if edit.is_some() {
        "project-edit"
    } else {
        "project"
    };
    run_native(case, profile, sample.as_bytes(), edit, origin)
}

fn run_native(
    case: &str,
    profile: &str,
    sample: &[u8],
    edit: Option<(i32, i32, &str)>,
    origin: (i32, i32),
) -> Result<String, String> {
    let exe = prepared_exe()?;
    with_fixture(case, |root| {
        run_in(exe, root, case, profile, sample, edit, origin)
    })
}

fn with_fixture<T, F>(case: &str, run: F) -> Result<T, String>
where
    F: FnOnce(&Path) -> Result<T, String>,
{
    let root = scratch(case);
    let result = run(&root);
    let cleanup = std::fs::remove_dir_all(&root);
    match (result, cleanup) {
        (Err(err), _) => Err(err),
        (Ok(_), Err(err)) => Err(format!("clean native fixture {}: {err}", root.display())),
        (Ok(report), Ok(())) => Ok(report),
    }
}

fn run_in(
    exe: &Path,
    root: &Path,
    case: &str,
    profile: &str,
    sample: &[u8],
    edit: Option<(i32, i32, &str)>,
    origin: (i32, i32),
) -> Result<String, String> {
    let input = root.join("input.at");
    let output = root.join("report.txt");
    let insert_path = root.join("insert.txt");
    let (start, end, insert) = edit.unwrap_or((0, 0, ""));
    std::fs::write(&input, sample).map_err(|err| format!("write fixture input: {err}"))?;
    std::fs::write(&insert_path, insert).map_err(|err| format!("write fixture edit: {err}"))?;

    let mut cmd = Command::new(exe);
    cmd.current_dir(root)
        .env("AAC1046_CASE", case)
        .env("AAC1046_PROFILE", profile)
        .env("AAC1046_INPUT", &input)
        .env("AAC1046_OUTPUT", &output)
        .env("AAC1046_SOURCE_ID", origin.0.to_string())
        .env("AAC1046_ORIGIN_ID", origin.1.to_string())
        .env("AAC1046_START", start.to_string())
        .env("AAC1046_END", end.to_string())
        .env("AAC1046_INSERT", &insert_path);
    execute_report(cmd, &output, case)
}

pub fn run_native_catalog(
    profile: &str,
    package: &str,
    files: &[(&str, &str)],
    inventory: &[&str],
) -> Result<String, String> {
    let exe = prepared_exe()?;
    with_fixture("catalog", |root| {
        let sources = root.join("sources");
        std::fs::create_dir(&sources).map_err(|err| err.to_string())?;
        for (name, source) in files {
            let mut parts = Path::new(name).components();
            if !matches!(parts.next(), Some(std::path::Component::Normal(_)))
                || parts.next().is_some()
            {
                return Err(format!("invalid fixture filename: {name}"));
            }
            std::fs::write(sources.join(name), source).map_err(|err| err.to_string())?;
        }
        let input = root.join("inventory.txt");
        let output = root.join("report.txt");
        std::fs::write(&input, inventory.join("\n")).map_err(|err| err.to_string())?;
        let mut cmd = Command::new(exe);
        cmd.current_dir(root)
            .env("AAC1046_CASE", "catalog")
            .env("AAC1046_PROFILE", profile)
            .env("AAC1046_INPUT", input)
            .env("AAC1046_OUTPUT", &output)
            .env("AAC1046_PACKAGE", package)
            .env("AAC1046_PROJECT", sources);
        execute_report(cmd, &output, "catalog")
    })
}

fn execute_report(cmd: Command, output: &Path, case: &str) -> Result<String, String> {
    let run = run_command_capture(cmd, TreeScope::ProcessTree, Some(Duration::from_secs(8)))
        .map_err(|err| format!("run AAC native case {case}: {err}"))?;
    let stdout = run.stdout_lossy().unwrap_or_default();
    if run.kind != ExitKind::Success || stdout.trim() != "value:0" {
        let stderr = run.stderr_lossy().unwrap_or_default();
        return Err(format!(
            "AAC native case {case}: {:?} {stdout}{stderr}",
            run.kind
        ));
    }
    std::fs::read_to_string(output).map_err(|err| format!("read AAC native result: {err}"))
}
