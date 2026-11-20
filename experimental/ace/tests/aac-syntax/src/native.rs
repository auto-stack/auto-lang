use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::OnceLock;
use std::time::Duration;

use auto_tool_execution::{run_command_capture, ExitKind, TreeScope};

use crate::{scratch, syntax_av_modules};

pub const NATIVE_DRIVER_SOURCE: &str = include_str!("../native_driver.at");
pub const DRIVER_EXE: &str = "aac1046-driver.exe";
pub const DRIVER_MANIFEST: &str = "driver.manifest";

fn fingerprint_part(hash: &mut blake3::Hasher, value: &[u8]) {
    hash.update(&(value.len() as u64).to_le_bytes());
    hash.update(value);
}

pub fn native_fingerprint() -> blake3::Hash {
    use ace_core::source::packages::{stdlib_assembled_files, stdlib_sys_files};

    let mut hash = blake3::Hasher::new();
    for (name, source) in syntax_av_modules() {
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
        NATIVE_DRIVER_SOURCE,
        include_str!("bin/prepare-native.rs"),
        include_str!("../../../Cargo.lock"),
    ] {
        fingerprint_part(&mut hash, source.as_bytes());
    }
    hash.finalize()
}

static PREPARED: OnceLock<Result<PathBuf, String>> = OnceLock::new();

fn prepared_exe() -> Result<&'static Path, String> {
    PREPARED
        .get_or_init(|| {
            let root = std::env::var_os("AAC1046_NATIVE_DIR")
                .map(PathBuf::from)
                .ok_or("prepare AAC native driver and set AAC1046_NATIVE_DIR")?;
            if !root.is_absolute() {
                return Err("AAC1046_NATIVE_DIR must be an absolute preparation directory".into());
            }
            let text = std::fs::read_to_string(root.join(DRIVER_MANIFEST))
                .map_err(|err| format!("read AAC driver manifest: {err}"))?;
            let mut fields = text.lines();
            let expected = native_fingerprint().to_hex();
            if fields.next() != Some(expected.as_str()) {
                return Err("AAC native driver preparation is stale".into());
            }
            let exe = root.join(DRIVER_EXE);
            let bytes = std::fs::read(&exe).map_err(|err| format!("read AAC driver: {err}"))?;
            if fields.next() != Some(blake3::hash(&bytes).to_hex().as_str()) {
                return Err("AAC native executable digest differs from preparation".into());
            }
            Ok(exe)
        })
        .as_deref()
        .map_err(Clone::clone)
}

pub fn run_native_case(
    case: &str,
    profile: &str,
    sample: &str,
    edit: Option<(i32, i32, &str)>,
) -> Result<String, String> {
    let exe = prepared_exe()?;
    let root = scratch(case);
    let result = run_in(exe, &root, case, profile, sample, edit);
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
    sample: &str,
    edit: Option<(i32, i32, &str)>,
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
        .env("AAC1046_START", start.to_string())
        .env("AAC1046_END", end.to_string())
        .env("AAC1046_INSERT", &insert_path);
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
