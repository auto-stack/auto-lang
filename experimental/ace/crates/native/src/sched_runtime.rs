//! PLAN-1067: content/build-bound preparation of the native task-scheduler
//! runtime static library (`ace-sched-runtime`, the callee side of the
//! `mn.sched.runtime/1` ABI).
//!
//! Mirrors the typed-entry/collections runtime discipline: a matching stamp
//! avoids cargo, but the current source/build inputs are ALWAYS hashed — an
//! unattested output is rebuilt, never blessed by mtime.

use crate::{config, MnError};
use serde::{Deserialize, Serialize};
use std::{
    io::Read,
    path::{Path, PathBuf},
    process::Command,
    sync::Mutex,
};

/// Runtime ABI tag of the scheduler helpers.
pub const ABI: &str = "mn.sched.runtime/1";

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Identity {
    pub abi: String,
    pub build_key: String,
    pub content_hash: String,
}

pub struct Prepared {
    pub identity: Identity,
    pub library: PathBuf,
    pub cargo_invocations: u64,
}

fn io(error: impl std::fmt::Display) -> MnError {
    MnError::Infra(format!("scheduler runtime: {error}"))
}

pub fn file_hash(path: &Path) -> Result<String, MnError> {
    let mut file = std::fs::File::open(path).map_err(io)?;
    let mut buffer = [0u8; 65536];
    let mut h = blake3::Hasher::new();
    loop {
        let n = file.read(&mut buffer).map_err(io)?;
        if n == 0 {
            break;
        }
        h.update(&buffer[..n]);
    }
    Ok(h.finalize().to_hex().to_string())
}

fn workspace() -> Result<PathBuf, MnError> {
    let mut roots = Vec::new();
    if let Some(root) = std::env::var_os("AUTO_LANG_ROOT") {
        roots.push(PathBuf::from(root).join("experimental/ace"));
    }
    roots.push(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../.."));
    roots.push(PathBuf::from("D:/autostack/auto-lang/experimental/ace"));
    roots
        .into_iter()
        .find(|root| root.join("crates/sched-runtime/Cargo.toml").is_file())
        .ok_or_else(|| io("scheduler runtime source workspace not found"))
}

fn source_files(root: &Path, files: &mut Vec<PathBuf>) -> Result<(), MnError> {
    for entry in std::fs::read_dir(root).map_err(io)? {
        let path = entry.map_err(io)?.path();
        if path.is_dir() {
            source_files(&path, files)?;
        } else if path.extension().is_some_and(|e| e == "rs") {
            files.push(path);
        }
    }
    Ok(())
}

fn command(cmd: &mut Command) -> Result<ace_core::link::CapturedRun, MnError> {
    ace_core::link::run_deadlined_captured(
        cmd,
        config::BUILD_BUDGET,
        ace_core::link::CaptureCaps {
            stdout_max: config::STREAM_CAP,
            stderr_max: config::STREAM_CAP,
        },
    )
    .map_err(|e| io(format!("{e:?}")))
}

fn executable_hash(name: &str) -> Result<String, MnError> {
    let path = Path::new(name);
    if path.is_file() {
        return file_hash(path);
    }
    if let Some(paths) = std::env::var_os("PATH") {
        for directory in std::env::split_paths(&paths) {
            for extension in ["", ".exe", ".cmd", ".bat"] {
                let candidate = directory.join(format!("{name}{extension}"));
                if candidate.is_file() {
                    return file_hash(&candidate);
                }
            }
        }
    }
    Err(io(format!(
        "cannot bind compiler/wrapper executable {name:?}"
    )))
}

/// Prepare (build if needed) the scheduler runtime static library with a
/// content-bound identity. Serial within the process.
pub fn prepare() -> Result<Prepared, MnError> {
    static SERIAL: Mutex<()> = Mutex::new(());
    let _guard = SERIAL.lock().map_err(|_| io("runtime lock poisoned"))?;
    let root = workspace()?;
    let runtime = root.join("crates/sched-runtime");
    let text = root.join("crates/text-runtime");
    let mut files = vec![
        runtime.join("Cargo.toml"),
        root.join("Cargo.toml"),
        root.join("Cargo.lock"),
        // The owned-text runtime is EMBEDDED in this staticlib (dependency),
        // so its sources are part of the runtime identity.
        text.join("Cargo.toml"),
    ];
    source_files(&runtime.join("src"), &mut files)?;
    source_files(&text.join("src"), &mut files)?;
    files.sort();
    let mut configs = Vec::new();
    for parent in runtime.ancestors() {
        for name in ["config", "config.toml"] {
            let p = parent.join(".cargo").join(name);
            if p.is_file() {
                configs.push(p);
            }
        }
    }
    let home = std::env::var_os("CARGO_HOME")
        .map(PathBuf::from)
        .or_else(|| std::env::var_os("USERPROFILE").map(|p| PathBuf::from(p).join(".cargo")));
    if let Some(home) = home {
        for name in ["config", "config.toml"] {
            let p = home.join(name);
            if p.is_file() {
                configs.push(p);
            }
        }
    }
    configs.sort();
    configs.dedup();
    let mut compiler = Command::new(std::env::var_os("RUSTC").unwrap_or_else(|| "rustc".into()));
    compiler.arg("-vV");
    let compiler = command(&mut compiler)?;
    if compiler.exit_code != Some(0) {
        return Err(io("rustc identity query failed"));
    }
    let mut h = blake3::Hasher::new();
    h.update(b"mn.sched.runtime-build/1");
    h.update(config::MN_TARGET.as_bytes());
    h.update(&compiler.stdout);
    for key in [
        "RUSTFLAGS",
        "CARGO_ENCODED_RUSTFLAGS",
        "CARGO_BUILD_RUSTFLAGS",
        "RUSTC",
        "RUSTUP_TOOLCHAIN",
        "RUSTC_WRAPPER",
        "RUSTC_WORKSPACE_WRAPPER",
    ] {
        let value = std::env::var(key).unwrap_or_default();
        h.update(&(value.len() as u64).to_le_bytes());
        h.update(value.as_bytes());
        if matches!(key, "RUSTC" | "RUSTC_WRAPPER" | "RUSTC_WORKSPACE_WRAPPER") && !value.is_empty() {
            h.update(executable_hash(&value)?.as_bytes());
        }
    }
    let mut extra: Vec<_> = std::env::vars()
        .filter(|(key, _)| {
            key.starts_with("CARGO_PROFILE_RELEASE_")
                || key.starts_with("CARGO_PROFILE_RUNTIME_RELEASE_")
                || key.starts_with("CARGO_TARGET_X86_64_PC_WINDOWS_MSVC_")
        })
        .collect();
    extra.sort();
    for (key, value) in extra {
        h.update(key.as_bytes());
        h.update(&(value.len() as u64).to_le_bytes());
        h.update(value.as_bytes());
    }
    for config in configs {
        let bytes = std::fs::read(config).map_err(io)?;
        h.update(&(bytes.len() as u64).to_le_bytes());
        h.update(&bytes);
    }
    for file in &files {
        let bytes = std::fs::read(file).map_err(io)?;
        h.update(&(bytes.len() as u64).to_le_bytes());
        h.update(&bytes);
    }
    let build_key = h.finalize().to_hex().to_string();
    let library = root
        .join("target")
        .join(config::MN_TARGET)
        .join("runtime-release/ac_sched_runtime.lib");
    let stamp = library.with_extension("identity.json");
    let old: Option<Identity> = std::fs::read(&stamp)
        .ok()
        .and_then(|bytes| serde_json::from_slice(&bytes).ok());
    let mut cargo_invocations = 0;
    let identity = if old
        .as_ref()
        .is_some_and(|old| old.build_key == build_key && old.abi == ABI)
        && library.is_file()
    {
        let old = old.unwrap();
        if file_hash(&library)? != old.content_hash {
            return Err(MnError::Admit(
                "runtime-content",
                "current scheduler library changed without a matching build receipt".into(),
            ));
        }
        old
    } else {
        if library.is_file() {
            std::fs::remove_file(&library).map_err(io)?;
        }
        let mut cargo = Command::new("cargo");
        cargo
            .current_dir(&runtime)
            .args([
                "build",
                "--profile",
                "runtime-release",
                "--locked",
                "--target",
                config::MN_TARGET,
            ])
            .arg("--target-dir")
            .arg(root.join("target"))
            .env_remove("CARGO_TARGET_DIR");
        let out = command(&mut cargo)?;
        cargo_invocations = 1;
        if out.exit_code != Some(0) {
            return Err(io(format!(
                "build failed: {}",
                String::from_utf8_lossy(&out.stderr)
            )));
        }
        let identity = Identity {
            abi: ABI.into(),
            build_key,
            content_hash: file_hash(&library)?,
        };
        std::fs::write(&stamp, serde_json::to_vec(&identity).map_err(io)?).map_err(io)?;
        identity
    };
    Ok(Prepared {
        identity,
        library,
        cargo_invocations,
    })
}
