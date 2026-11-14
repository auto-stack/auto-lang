//! PLAN-967 F0 (T-F0-I): the Rust provider build unit and in-process
//! loader. One build closure (B-01 seed): the frozen annotated provider
//! source and the GENERATED C-compatible shim compile together into a
//! single `cdylib`; the loader resolves the `#[no_mangle]` exports into
//! the AV executor's stub table. No second loader/manifest world — the
//! shared manual manifest and `ForeignStubTable` are the only contracts.

use std::path::{Path, PathBuf};

/// The frozen F0 provider fixture (the annotations ARE the metadata).
/// PLAN-967 T-11: the frozen RICH provider fixture.
pub fn f0_rich_provider_source() -> &'static str {
    include_str!("../rustside/f0_rich.rs")
}

pub fn f0_provider_source() -> &'static str {
    include_str!("../rustside/f0_text.rs")
}

/// One built provider: the cdylib plus every identity the host needs to
/// admit and load it.
#[derive(Clone)]
pub struct RustProviderBuild {
    pub cdylib: PathBuf,
    /// PLAN-967 T-11 (B-02): the staticlib product of the same build
    /// closure (native-relink distribution form).
    pub staticlib: PathBuf,
    pub manifest: auto_bindgen::rust_import::RustProviderManifest,
    pub shim_source: String,
    pub manual: ace_foreign_contract::manual::ManualManifest,
    pub provider_digest: String,
    pub rustc_identity: String,
}

fn run_rustc(args: &[&str], cwd: &Path) -> Result<(i32, String, String), String> {
    // PLAN-967 T-11 (V-ABI-CROSS): an explicit second-toolchain override;
    // the default stays the ambient rustc; identity is recorded per build.
    let program = std::env::var("AC741_RUSTC").unwrap_or_else(|_| "rustc".into());
    let mut command = std::process::Command::new(program);
    command.args(args).current_dir(cwd);
    let run = ace_core::link::run_deadlined_typed(
        &mut command,
        std::time::Duration::from_secs(120),
        ace_core::link::CaptureCaps {
            stdout_max: 1 << 20,
            stderr_max: 1 << 20,
        },
        vec![],
    )
    .map_err(|e| format!("rustc spawn: {e:?}"))?;
    Ok((
        run.exit_code.unwrap_or(-1),
        String::from_utf8_lossy(&run.stdout).into_owned(),
        String::from_utf8_lossy(&run.stderr).into_owned(),
    ))
}

/// Build the F0 provider cdylib from the frozen source + the generated
/// shim (one rustc closure). `root` is the caller-owned scratch root
/// (plan-artifacts scope for tests).
pub fn build_f0(root: &Path) -> Result<RustProviderBuild, String> {
    build_source(root, f0_provider_source())
}

/// Same build closure for a (possibly edited) provider source — the
/// identity/rebind leg rebuilds the unit from new provider bytes.
pub fn build_source(root: &Path, source: &str) -> Result<RustProviderBuild, String> {
    let manifest = auto_bindgen::rust_import::import(source)
        .map_err(|e| format!("{}: {}", e.code, e.message))?;
    let shim_source = auto_bindgen::rust_import::generate_shim(&manifest);
    let provider_digest = auto_bindgen::rust_import::provider_digest(source);
    let manual = auto_bindgen::rust_import::to_manual_manifest(
        &manifest,
        "f0-rust-provider/1",
        &provider_digest,
        &auto_bindgen::rust_import::shim_digest(&shim_source),
    )?;

    let crate_dir = root.join("f0-provider-crate");
    let src = crate_dir.join("src");
    let out = root.join("out");
    std::fs::create_dir_all(&src).map_err(|e| e.to_string())?;
    std::fs::create_dir_all(&out).map_err(|e| e.to_string())?;
    std::fs::write(src.join("provider.rs"), source).map_err(|e| e.to_string())?;
    std::fs::write(src.join("shim.rs"), &shim_source).map_err(|e| e.to_string())?;
    // The crate root is the FLAT concatenation of the provider source and
    // the generated shim (inner doc comments and items splice legally;
    // `--cap-lints allow` covers the mixed-use lint surface).
    std::fs::write(
        src.join("lib.rs"),
        format!(
            "{}\n// PLAN-967 F0 build unit: provider + generated shim, one closure.\n{}",
            source, shim_source
        ),
    )
    .map_err(|e| e.to_string())?;

    let (code, _out, err) = run_rustc(&["--version"], &crate_dir)?;
    if code != 0 {
        return Err(format!("rustc --version failed: {err}"));
    }
    let rustc_identity = _out.trim().to_string();
    let (code, _out, err) = run_rustc(
        &[
            "--edition",
            "2021",
            "--crate-type",
            "cdylib",
            "--crate-name",
            "f0_text_provider",
            "--cap-lints",
            "allow",
            "-O",
            "src/lib.rs",
            "--out-dir",
            out.to_str().ok_or("non-utf8 out dir")?,
        ],
        &crate_dir,
    )?;
    if code != 0 {
        return Err(format!("rustc cdylib build failed: {err}"));
    }
    let cdylib = out.join("f0_text_provider.dll");
    if !cdylib.is_file() {
        return Err(format!("cdylib not produced at {}", cdylib.display()));
    }
    // PLAN-967 T-11 (B-02): the STATICLIB artifact — the native-relink
    // distribution form (one more product of the SAME build closure; its
    // LTO/link-tool conditions are the consumer's contract, recorded by
    // the artifact existing beside the cdylib).
    let (code, _out, err) = run_rustc(
        &[
            "--edition",
            "2021",
            "--crate-type",
            "staticlib",
            "--crate-name",
            "f0_text_provider",
            "--cap-lints",
            "allow",
            "-O",
            "src/lib.rs",
            "--out-dir",
            out.to_str().ok_or("non-utf8 out dir")?,
        ],
        &crate_dir,
    )?;
    if code != 0 {
        return Err(format!("rustc staticlib build failed: {err}"));
    }
    let staticlib = out.join("f0_text_provider.lib");
    if !staticlib.is_file() {
        return Err(format!("staticlib not produced at {}", staticlib.display()));
    }
    Ok(RustProviderBuild {
        cdylib,
        staticlib,
        manifest,
        shim_source,
        manual,
        provider_digest,
        rustc_identity,
    })
}

/// The controlled async provider fixture (PLAN-967 T-08).
pub fn f0_async_provider_source() -> &'static str {
    include_str!("../rustside/f0_async.rs")
}

/// Build a RAW cdylib from a provider source (no importer, no manifest —
/// for controlled fixtures whose shape is outside the auto-imported F0
/// domain; bindings/manifests stay auto-derived wherever they exist).
pub fn build_raw_cdylib(root: &Path, source: &str, crate_name: &str) -> Result<PathBuf, String> {
    let dir = root.join(format!("raw-{crate_name}"));
    let src = dir.join("src");
    let out = root.join("out");
    std::fs::create_dir_all(&src).map_err(|e| e.to_string())?;
    std::fs::create_dir_all(&out).map_err(|e| e.to_string())?;
    std::fs::write(src.join("lib.rs"), source).map_err(|e| e.to_string())?;
    let (code, _o, err) = run_rustc(
        &[
            "--edition",
            "2021",
            "--crate-type",
            "cdylib",
            "--crate-name",
            crate_name,
            "--cap-lints",
            "allow",
            "-O",
            "src/lib.rs",
            "--out-dir",
            out.to_str().ok_or("non-utf8 out dir")?,
        ],
        &dir,
    )?;
    if code != 0 {
        return Err(format!("rustc cdylib build failed: {err}"));
    }
    let dll = out.join(format!("{crate_name}.dll"));
    if !dll.is_file() {
        return Err(format!("cdylib not produced at {}", dll.display()));
    }
    Ok(dll)
}

/// A raw loaded library: keeps the OS handle alive; resolves one symbol
/// at a time; dropping frees exactly once.
pub struct RawLibrary {
    library: windows_sys::Win32::Foundation::HMODULE,
}

impl RawLibrary {
    pub fn load(path: &Path) -> Result<Self, String> {
        let wide: Vec<u16> = path
            .to_str()
            .ok_or("non-utf8 path")?
            .encode_utf16()
            .chain(std::iter::once(0))
            .collect();
        let library =
            unsafe { windows_sys::Win32::System::LibraryLoader::LoadLibraryW(wide.as_ptr()) };
        if library.is_null() {
            return Err(format!("LoadLibraryW failed for {}", path.display()));
        }
        Ok(Self { library })
    }

    /// Resolve an exported symbol (raw pointer).
    pub fn symbol(&self, name: &str) -> Option<*const core::ffi::c_void> {
        let bytes: Vec<u8> = name.bytes().chain(std::iter::once(0)).collect();
        let sym = unsafe {
            windows_sys::Win32::System::LibraryLoader::GetProcAddress(self.library, bytes.as_ptr())
        };
        sym.map(|p| p as *const core::ffi::c_void)
    }
}

impl Drop for RawLibrary {
    fn drop(&mut self) {
        if !self.library.is_null() {
            unsafe {
                windows_sys::Win32::Foundation::FreeLibrary(self.library as _);
            }
        }
    }
}

/// The host authority for the F0 provider build (the compile host's
/// current review registry entry; the loaded library backs the symbols).
pub fn f0_authority(
    build: &RustProviderBuild,
    binding_generation: u64,
) -> ace_foreign_contract::manual::HostAuthority {
    ace_foreign_contract::manual::HostAuthority {
        review_id: build.manual.review_id.clone(),
        contract_digest: build.manual.contract_digest().unwrap(),
        provider: build.manual.provider.clone(),
        provider_digest: build.provider_digest.clone(),
        header_digest: build.manual.header_digest.clone(),
        physical_table_digest: build.manual.physical_table_digest.clone(),
        target: build.manual.target.clone(),
        capabilities: build.manual.facts.capabilities.iter().cloned().collect(),
        available_symbols: build
            .manifest
            .functions
            .iter()
            .map(|f| f.export.clone())
            .collect(),
        binding_generation,
    }
}

/// PLAN-967 T-10: the frozen C provider header (the annotations ARE the
/// metadata) and its implementation.
pub fn f0_c_header() -> &'static str {
    include_str!("../cside/f0_c.h")
}
pub fn f0_c_source() -> &'static str {
    include_str!("../cside/f0_c.c")
}

/// One imported C provider: the derived manifest plus the compiled real
/// provider object (cl.exe, observed include closure).
pub struct CProvider {
    pub manifest: ace_foreign_contract::manual::ManualManifest,
    pub object: std::path::PathBuf,
    pub include_closure: Vec<std::path::PathBuf>,
    pub provider_digest: String,
    /// the extracted enum constants (standalone bindings for consumers)
    pub constants: Vec<(String, i32)>,
}

/// Import a C provider PROJECT: preprocess `<root>/<name>.h` with the
/// REAL preprocessor, derive the manifest from the annotations + the
/// preprocessed declarations, admit it, and compile `<root>/<name>.c`
/// with cl.exe (returning the observed include closure). Every identity
/// derives from the project files; no hand-written manifest exists.
pub fn open_c_provider(root: &Path, name: &str) -> Result<CProvider, String> {
    let header_path = root.join(format!("{name}.h"));
    let source_path = root.join(format!("{name}.c"));
    if !header_path.is_file() {
        return Err(format!(
            "c-import.provider-missing: `use.c {name}` found no header at {}",
            header_path.display()
        ));
    }
    if !source_path.is_file() {
        return Err(format!(
            "c-import.provider-missing: `use.c {name}` found no implementation at {}",
            source_path.display()
        ));
    }
    let raw_header = std::fs::read_to_string(&header_path)
        .map_err(|e| format!("c-import.provider-read: {e}"))?;
    let provider_header = auto_bindgen::c_import::parse_provider_header(&raw_header)
        .map_err(|e| format!("{}: {}", e.code, e.message))?;
    if provider_header.name != name {
        return Err(format!(
            "c-import.provider-name: header declares `{}` but the source imports `{name}`",
            provider_header.name
        ));
    }
    let entries = auto_bindgen::c_import::parse_annotations(&raw_header)
        .map_err(|e| format!("{}: {}", e.code, e.message))?;
    let preprocessed = crate::ctool::preprocess_c(&header_path, root)
        .map_err(|e| format!("c-import.preprocess: {e}"))?;
    auto_bindgen::c_import::audit_shapes(&raw_header)
        .map_err(|e| format!("{}: {}", e.code, e.message))?;
    let decls = auto_bindgen::c_import::parse_declarations(&preprocessed);
    let constants = auto_bindgen::c_import::extract_constants(&decls);
    let inline_shim = auto_bindgen::c_import::generate_inline_shim(&decls, &entries);
    let provider_digest = ace_foreign_contract::blake3_hex(raw_header.as_bytes());
    let manifest = auto_bindgen::c_import::to_manual_manifest(
        name,
        &entries,
        &decls,
        "f0-c-provider/1",
        &provider_digest,
    )
    .map_err(|e| format!("{}: {}", e.code, e.message))?;
    // the REAL provider object (include closure observed by cl itself)
    let build_dir = root.join(format!(".c-build-{name}"));
    std::fs::create_dir_all(&build_dir).map_err(|e| e.to_string())?;
    std::fs::write(build_dir.join(format!("{name}.c")), {
        // compile from a copy beside the header so `#include "f0_c.h"` resolves
        std::fs::read_to_string(&source_path).map_err(|e| e.to_string())?
    })
    .map_err(|e| e.to_string())?;
    std::fs::copy(&header_path, build_dir.join(format!("{name}.h"))).map_err(|e| e.to_string())?;
    // PLAN-967 T-15: the project's OTHER headers/sources (the staged
    // dependency files — e.g. the pinned amalgamation) stage beside the
    // copy too, so the provider's own `#include "sqlite3.h"` resolves
    // in the build dir exactly like in the project dir.
    if let Ok(entries) = std::fs::read_dir(root) {
        for entry in entries.flatten() {
            let file = entry.file_name().to_string_lossy().into_owned();
            if (file.ends_with(".h") || file.ends_with(".c"))
                && file != format!("{name}.h")
                && file != format!("{name}.c")
            {
                let dest = build_dir.join(&*entry.file_name());
                if !dest.is_file() {
                    std::fs::copy(entry.path(), &dest).map_err(|e| e.to_string())?;
                }
            }
        }
    }
    // the generated static-inline shim joins the provider build (one
    // closure: the header bodies are the implementation)
    if let Some(shim) = inline_shim {
        std::fs::write(build_dir.join("inline_shim.c"), shim).map_err(|e| e.to_string())?;
        let shim_obj = build_dir.join("inline_shim.obj");
        crate::ctool::compile_c(&build_dir.join("inline_shim.c"), &build_dir, &shim_obj)
            .map_err(|e| format!("c-import.inline-shim: {e}"))?;
    }
    let obj = build_dir.join(format!("{name}.obj"));
    let include_closure =
        crate::ctool::compile_c_with_deps(&build_dir.join(format!("{name}.c")), &build_dir, &obj)
            .map_err(|e| format!("c-import.compile: {e}"))?;
    Ok(CProvider {
        manifest,
        object: obj,
        include_closure,
        provider_digest,
        constants,
    })
}

/// The C provider's host authority (header digest = provider identity).
pub fn c_authority(
    provider: &CProvider,
    binding_generation: u64,
) -> ace_foreign_contract::manual::HostAuthority {
    let m = &provider.manifest;
    ace_foreign_contract::manual::HostAuthority {
        review_id: m.review_id.clone(),
        contract_digest: m.contract_digest().unwrap(),
        provider: m.provider.clone(),
        provider_digest: provider.provider_digest.clone(),
        header_digest: m.header_digest.clone(),
        physical_table_digest: m.physical_table_digest.clone(),
        target: m.target.clone(),
        capabilities: m.facts.capabilities.iter().cloned().collect(),
        available_symbols: m.facts.functions.iter().map(|f| f.symbol.clone()).collect(),
        binding_generation,
    }
}

// ---------------------------------------------------------------------------
// PLAN-967 T-15: pinned REAL libraries (V-15) — serde_json (Rust) and
// SQLite (C). A clean project configures ONLY the dependency (a `dep`
// annotation / the staged amalgamation) and the public use import; the
// binding, shim, manifest and build unit all derive from those bytes.
// ---------------------------------------------------------------------------

/// The frozen REAL-LIBRARY Rust provider (pinned serde_json via its own
/// `dep` annotation).
pub fn f0_json_provider_source() -> &'static str {
    include_str!("../rustside/f0_json.rs")
}

/// `// auto-ffi dep <name> <spec>` lines are the clean project's whole
/// dependency configuration: the importer ignores them (they are not fn
/// metadata) and the cargo build unit consumes them verbatim.
pub fn provider_deps(source: &str) -> Vec<(String, String)> {
    let mut deps = Vec::new();
    for line in source.lines() {
        let t = line.trim();
        if let Some(rest) = t.strip_prefix("// auto-ffi dep ") {
            let mut it = rest.trim().split_whitespace();
            if let (Some(name), Some(spec)) = (it.next(), it.next()) {
                deps.push((name.to_string(), spec.to_string()));
            }
        }
    }
    deps
}

/// Build a provider with REAL pinned dependencies (the B-01 source-mode
/// build closure): a real Cargo unit — Cargo.toml generated from the
/// provider's `dep` annotations, the provider + the GENERATED shim flat
/// in src/, `cargo build --release --offline`. Both closure products
/// (cdylib + staticlib) come out of the same invocation. A failed
/// dependency resolution (wrong version / missing crate) is a NAMED
/// error carrying cargo's own diagnostics — never a silent tool change.
pub fn build_cargo_provider(
    root: &Path,
    provider_source: &str,
) -> Result<RustProviderBuild, String> {
    let manifest = auto_bindgen::rust_import::import(provider_source)
        .map_err(|e| format!("{}: {}", e.code, e.message))?;
    let shim_source = auto_bindgen::rust_import::generate_shim(&manifest);
    let provider_digest = auto_bindgen::rust_import::provider_digest(provider_source);
    let manual = auto_bindgen::rust_import::to_manual_manifest(
        &manifest,
        "f0-rust-provider/1",
        &provider_digest,
        &auto_bindgen::rust_import::shim_digest(&shim_source),
    )?;

    let dir = root.join("cargo-unit");
    let src = dir.join("src");
    std::fs::create_dir_all(&src).map_err(|e| e.to_string())?;

    let mut toml = String::from(
        "[package]
name = \"provider_unit\"
version = \"0.1.0\"
edition = \"2021\"

[lib]
crate-type = [\"cdylib\", \"staticlib\"]
path = \"src/lib.rs\"

[dependencies]
",
    );
    for (name, spec) in provider_deps(provider_source) {
        toml.push_str(&format!("{name} = \"{spec}\"\n"));
    }
    std::fs::write(dir.join("Cargo.toml"), toml).map_err(|e| e.to_string())?;
    std::fs::write(
        src.join("lib.rs"),
        format!(
            "{provider_source}\n// PLAN-967 T-15 cargo unit: provider + generated shim, one closure.\n{shim_source}"
        ),
    )
    .map_err(|e| e.to_string())?;

    let (code, _out, err) = run_rustc(&["--version"], &dir)?;
    if code != 0 {
        return Err(format!("rustc --version failed: {err}"));
    }
    let rustc_identity = _out.trim().to_string();

    let mut cmd = std::process::Command::new("cargo");
    cmd.args(["build", "--release", "--offline"])
        .current_dir(&dir)
        .env_remove("CARGO_TARGET_DIR");
    let run = ace_core::link::run_deadlined_typed(
        &mut cmd,
        std::time::Duration::from_secs(300),
        ace_core::link::CaptureCaps {
            stdout_max: 1 << 20,
            stderr_max: 1 << 20,
        },
        vec![],
    )
    .map_err(|e| format!("f0.cargo-build: cargo spawn: {e:?}"))?;
    if run.exit_code != Some(0) {
        return Err(format!(
            "f0.cargo-build: cargo exited {:?}: {}",
            run.exit_code,
            String::from_utf8_lossy(&run.stderr).trim()
        ));
    }
    let out = dir.join("target/release");
    let cdylib = out.join("provider_unit.dll");
    if !cdylib.is_file() {
        return Err(format!(
            "f0.cargo-build: cdylib not produced at {}",
            cdylib.display()
        ));
    }
    let staticlib = out.join("provider_unit.lib");
    if !staticlib.is_file() {
        return Err(format!(
            "f0.cargo-build: staticlib not produced at {}",
            staticlib.display()
        ));
    }
    Ok(RustProviderBuild {
        cdylib,
        staticlib,
        manifest,
        shim_source,
        manual,
        provider_digest,
        rustc_identity,
    })
}

// ---- T-15 C side: the pinned REAL SQLite amalgamation ----

/// The frozen REAL-LIBRARY C provider header/implementation (the real
/// sqlite3 open/exec/prepare/step/close line behind view frames).
pub fn p967_sqlite_header() -> &'static str {
    include_str!("../cside/p967_sqlite.h")
}
pub fn p967_sqlite_source() -> &'static str {
    include_str!("../cside/p967_sqlite.c")
}

/// The SQLite version a staged amalgamation declares — a CONTENT read
/// of its own sqlite3.h (`#define SQLITE_VERSION "x.y.z"`).
pub fn sqlite_version_of(sqlite_dir: &Path) -> Result<String, String> {
    let header_path = sqlite_dir.join("sqlite3.h");
    let header = std::fs::read_to_string(&header_path).map_err(|e| {
        format!(
            "f0.sqlite-missing: cannot read {}: {e}",
            header_path.display()
        )
    })?;
    for line in header.lines() {
        let t = line.trim();
        if let Some(rest) = t.strip_prefix("#define SQLITE_VERSION") {
            let v = rest.trim().trim_matches('"');
            if v.chars().next().is_some_and(|c| c.is_ascii_digit()) {
                return Ok(v.to_string());
            }
        }
    }
    Err("f0.sqlite-missing: sqlite3.h declares no SQLITE_VERSION".into())
}

/// Locate the PINNED SQLite amalgamation by CONTENT: the cargo registry
/// source whose sqlite3.h declares exactly `expected`. Pinning is a
/// content fact, not a directory-name guess.
pub fn locate_pinned_sqlite(expected: &str) -> Result<PathBuf, String> {
    let home = std::env::var("USERPROFILE")
        .map_err(|_| "f0.sqlite-missing: no USERPROFILE".to_string())?;
    let src_root = PathBuf::from(home).join(".cargo/registry/src");
    let mut candidates = Vec::new();
    if let Ok(mirrors) = std::fs::read_dir(&src_root) {
        for mirror in mirrors.flatten() {
            if let Ok(packages) = std::fs::read_dir(mirror.path()) {
                for pkg in packages.flatten() {
                    if pkg
                        .file_name()
                        .to_string_lossy()
                        .starts_with("libsqlite3-sys-")
                    {
                        candidates.push(pkg.path().join("sqlite3"));
                    }
                }
            }
        }
    }
    for dir in &candidates {
        if sqlite_version_of(dir).as_deref() == Ok(expected) {
            return Ok(dir.clone());
        }
    }
    Err(format!(
        "f0.sqlite-missing: no SQLite {expected} under {} ({} registry candidates scanned)",
        src_root.display(),
        candidates.len()
    ))
}

/// Verify a staged amalgamation against the declared pin — the
/// wrong-version case is a NAMED rejection, never a silent swap.
pub fn verify_sqlite_pin(sqlite_dir: &Path, expected: &str) -> Result<(), String> {
    let found = sqlite_version_of(sqlite_dir)?;
    if found != expected {
        return Err(format!(
            "f0.sqlite-pin: the project pins sqlite ={expected} but the staged amalgamation declares {found}"
        ));
    }
    Ok(())
}

/// Stage the pinned amalgamation files FLAT beside the provider project
/// (the clean project's configured dependency).
pub fn stage_pinned_sqlite(root: &Path, sqlite_dir: &Path) -> Result<(), String> {
    for f in ["sqlite3.c", "sqlite3.h"] {
        std::fs::copy(sqlite_dir.join(f), root.join(f))
            .map_err(|e| format!("f0.sqlite-missing: stage {f}: {e}"))?;
    }
    Ok(())
}

/// The SDK include closure a full-CRT C compilation needs, derived from
/// the same discovery as the lib dirs: `<sdk>/Lib/<ver>/um/x64` ->
/// `<sdk>/Include/<ver>/{ucrt,um,shared,winrt,cppwinrt}`.
pub fn sdk_include_dirs() -> Vec<PathBuf> {
    let Ok(um) = ace_core::link::find_sdk_um_dir() else {
        return Vec::new();
    };
    // um = <sdk>/Lib/<ver>/um/x64
    let Some(ver_dir) = um.parent().and_then(|u| u.parent()) else {
        return Vec::new();
    };
    let Some(ver) = ver_dir.file_name() else {
        return Vec::new();
    };
    let Some(sdk_root) = ver_dir.parent().and_then(|l| l.parent()) else {
        return Vec::new();
    };
    let inc = sdk_root.join("Include").join(ver);
    ["ucrt", "um", "shared", "winrt", "cppwinrt"]
        .iter()
        .map(|s| inc.join(s))
        .filter(|d| d.is_dir())
        .collect()
}

/// Build the INDEPENDENT C oracle executable: `main_c` plus the pinned
/// sources, compiled by the same real cl into a standalone exe — a
/// separate process sharing NO FFI machinery with the provider under
/// test (only the pinned library bytes).
pub fn build_c_oracle(
    root: &Path,
    main_c: &str,
    extra_sources: &[PathBuf],
) -> Result<PathBuf, String> {
    let dir = root.join("c-oracle");
    std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    std::fs::write(dir.join("oracle_main.c"), main_c).map_err(|e| e.to_string())?;
    for src in extra_sources {
        let name = src.file_name().ok_or("oracle source without a name")?;
        std::fs::copy(src, dir.join(name)).map_err(|e| e.to_string())?;
    }
    let tool = crate::ctool::find_cl().map_err(|e| format!("cl: {e}"))?;
    let msvc_root = tool
        .cl_exe
        .ancestors()
        .nth(4)
        .ok_or("cannot derive MSVC root from cl path")?;
    let exe = dir.join("p967_oracle.exe");
    let mut cmd = std::process::Command::new(&tool.cl_exe);
    cmd.args(["-nologo", "-O1"])
        .arg(dir.join("oracle_main.c"))
        .arg(format!("-I{}", dir.display()))
        .arg(format!("-I{}", msvc_root.join("include").display()))
        .arg(format!("-Fe:{}", exe.display()));
    for inc in sdk_include_dirs() {
        cmd.arg(format!("-I{}", inc.display()));
    }
    for src in extra_sources {
        // only compilation units (.c) go on the command line; staged
        // headers resolve through -I
        if src.extension().and_then(|e| e.to_str()) == Some("c") {
            cmd.arg(dir.join(src.file_name().ok_or("oracle source without a name")?));
        }
    }
    cmd.arg("-link")
        .arg(format!("-LIBPATH:{}", msvc_root.join("lib/x64").display()));
    if let Ok(sdk_um) = ace_core::link::find_sdk_um_dir() {
        if let Some(ucrt) = sdk_um
            .parent()
            .and_then(|um| um.parent())
            .map(|lib| lib.join("ucrt/x64"))
        {
            cmd.arg(format!("-LIBPATH:{}", ucrt.display()));
        }
        cmd.arg(format!("-LIBPATH:{}", sdk_um.display()));
    }
    let run = ace_core::link::run_deadlined_typed(
        &mut cmd,
        std::time::Duration::from_secs(180),
        ace_core::link::CaptureCaps {
            stdout_max: 1 << 16,
            stderr_max: 1 << 16,
        },
        vec![],
    )
    .map_err(|e| format!("oracle cl spawn: {e:?}"))?;
    if run.exit_code != Some(0) {
        return Err(format!(
            "oracle cl exited {:?}: {} | {}",
            run.exit_code,
            String::from_utf8_lossy(&run.stdout).trim(),
            String::from_utf8_lossy(&run.stderr).trim()
        ));
    }
    if !exe.is_file() {
        return Err(format!("oracle exe not produced at {}", exe.display()));
    }
    Ok(exe)
}

/// PLAN-967 T-09: PROJECT-level provider resolution for the `use.rs`
/// frontdoor. The source's `use.rs <provider>` names a PROJECT file
/// `<root>/<provider>.rs`; the binding, shim, build unit and manifest
/// are all DERIVED from that file at open time (no hand-written manifest
/// exists anywhere in the project — V-09's "delete the manifest" check
/// is vacuously satisfied by construction and asserted by the caller).
/// Named rejections: missing provider file, importer-domain violations.
pub fn open_project_session(
    project_root: &Path,
    provider_name: &str,
    generation: u64,
) -> Result<F0Session, String> {
    if provider_name.is_empty()
        || !provider_name
            .chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_')
    {
        return Err(format!(
            "rust-import.provider-name: `{provider_name}` is not a project provider spelling [a-z0-9_]+"
        ));
    }
    let provider_path = project_root.join(format!("{provider_name}.rs"));
    if !provider_path.is_file() {
        return Err(format!(
            "rust-import.provider-missing: `use.rs {provider_name}` found no project provider at {}",
            provider_path.display()
        ));
    }
    let source = std::fs::read_to_string(&provider_path).map_err(|e| {
        format!(
            "rust-import.provider-read: {}: {e}",
            provider_path.display()
        )
    })?;
    let build = build_source(
        &project_root.join(format!(".f0-build-{provider_name}-{generation}")),
        &source,
    )?;
    open_session_from_build(&build, generation)
}

/// PLAN-967 T-12 (V-COVERAGE AV-C): build the C provider as a REAL DLL
/// (cl /LD with an export .def listing the manifest symbols) so the AV
/// stub table can load and dispatch it exactly like the Rust lane.
pub fn build_c_dll(provider: &CProvider, root: &Path) -> Result<std::path::PathBuf, String> {
    build_c_dll_with(provider, root, &[], &[])
}

/// PLAN-967 T-15: the DLL build with EXTRA compilation units and
/// include dirs — the real-library lane stages the pinned amalgamation
/// (sqlite3.c compiles as its own TU beside the provider) and needs the
/// full SDK include closure (the amalgamation uses CRT + windows
/// headers; the freestanding provider object never did).
pub fn build_c_dll_with(
    provider: &CProvider,
    root: &Path,
    extra_sources: &[PathBuf],
    extra_includes: &[PathBuf],
) -> Result<std::path::PathBuf, String> {
    let dir = root.join("c-dll");
    std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    for src in extra_sources {
        let name = src.file_name().ok_or("extra source without a name")?;
        std::fs::copy(src, dir.join(name))
            .map_err(|e| format!("stage extra {}: {e}", name.to_string_lossy()))?;
    }
    std::fs::copy(
        &provider
            .object
            .with_file_name(format!("{}.c", provider.manifest.provider)),
        dir.join("provider.c"),
    )
    .or_else(|_| {
        // the source lives in the build dir beside the object
        std::fs::copy(
            &provider
                .object
                .with_file_name(format!("{}.c", provider.manifest.provider)),
            dir.join("provider.c"),
        )
    })
    .map_err(|e| format!("stage provider.c: {e}"))?;
    std::fs::copy(
        &provider
            .object
            .with_file_name(format!("{}.h", provider.manifest.provider)),
        dir.join(format!("{}.h", provider.manifest.provider)),
    )
    .map_err(|e| format!("stage provider.h: {e}"))?;
    let dll_name = format!("{}_provider", provider.manifest.provider);
    let mut def = format!("LIBRARY {dll_name}\nEXPORTS\n");
    for f in &provider.manifest.facts.functions {
        def.push_str(&format!(
            "  {}
",
            f.symbol
        ));
    }
    def.push_str("  ace_foreign_fail\n");
    std::fs::write(dir.join("exports.def"), def).map_err(|e| e.to_string())?;
    let dll = dir.join(format!("{dll_name}.dll"));
    // cl /LD compiles AND links the DLL in one invocation
    let tool = crate::ctool::find_cl().map_err(|e| format!("cl: {e}"))?;
    // the MSVC include dir (same derivation as the compile path — cl /LD
    // has no ambient INCLUDE in this environment)
    let msvc_root = tool
        .cl_exe
        .ancestors()
        .nth(4)
        .ok_or("cannot derive MSVC root from cl path")?;
    let msvc_include = msvc_root.join("include");
    // the generated static-inline shim and the CRT-free failure runtime
    // join the DLL build (one closure — the exported symbols must resolve)
    if let Some(shim) = provider.object.with_file_name("inline_shim.c").to_str() {
        if std::path::Path::new(shim).is_file() {
            std::fs::copy(shim, dir.join("inline_shim.c")).map_err(|e| e.to_string())?;
        }
    }
    std::fs::write(
        dir.join("failure_runtime.c"),
        "void ace_foreign_fail(int status);
__declspec(dllimport) void __stdcall ExitProcess(unsigned int code);
void ace_foreign_fail(int status) { (void)status; ExitProcess(70); }
",
    )
    .map_err(|e| e.to_string())?;
    let mut cmd = std::process::Command::new(&tool.cl_exe);
    // the CRT link paths: MSVC lib/x64 (LIBCMT) + the SDK ucrt lib
    cmd.args(["-nologo", "-LD"])
        .arg(format!("-I{}", msvc_include.display()))
        .arg(dir.join("provider.c"))
        .arg(dir.join("failure_runtime.c"))
        .arg(format!("-Fe:{}", dll.display()))
        .arg(format!("-I{}", dir.display()));
    // the generated static-inline shim joins only when one exists
    if dir.join("inline_shim.c").is_file() {
        cmd.arg(dir.join("inline_shim.c"));
    }
    // T-15: the real-library extra units + their include closure
    for inc in extra_includes {
        cmd.arg(format!("-I{}", inc.display()));
    }
    for src in extra_sources {
        // only compilation units (.c) go on the command line; headers
        // staged beside them resolve through -I
        if src.extension().and_then(|e| e.to_str()) == Some("c") {
            let staged = dir.join(src.file_name().ok_or("extra source without a name")?);
            cmd.arg(&staged);
        }
    }
    cmd.arg("-link")
        .arg(format!("-DEF:{}", dir.join("exports.def").display()))
        .arg(format!("-LIBPATH:{}", msvc_root.join("lib/x64").display()));
    if let Some(sdk_um) = ace_core::link::find_sdk_um_dir().ok() {
        // <sdk>/Lib/<ver>/um/x64 -> sibling ucrt/x64 (two parents up)
        if let Some(ucrt) = sdk_um
            .parent()
            .and_then(|um| um.parent())
            .map(|lib| lib.join("ucrt/x64"))
        {
            cmd.arg(format!("-LIBPATH:{}", ucrt.display()));
        }
        cmd.arg(format!("-LIBPATH:{}", sdk_um.display()));
    }
    // the amalgamation TU is big — the real-library build gets a wider
    // deadline than the freestanding provider objects
    let deadline = if extra_sources.is_empty() {
        std::time::Duration::from_secs(60)
    } else {
        std::time::Duration::from_secs(300)
    };
    let run = ace_core::link::run_deadlined_typed(
        &mut cmd,
        deadline,
        ace_core::link::CaptureCaps {
            stdout_max: 1 << 16,
            stderr_max: 1 << 16,
        },
        vec![],
    )
    .map_err(|e| format!("cl /LD spawn: {e:?}"))?;
    if run.exit_code != Some(0) {
        return Err(format!(
            "cl /LD exited {:?}: {} | {}",
            run.exit_code,
            String::from_utf8_lossy(&run.stdout).trim(),
            String::from_utf8_lossy(&run.stderr).trim()
        ));
    }
    if !dll.is_file() {
        return Err(format!("DLL not produced at {}", dll.display()));
    }
    Ok(dll)
}

/// PLAN-967 T-02 composition: one OPEN foreign session = build → admit →
/// load → bind, holding every identity piece alive together. Consumers
/// (969/E0 and later A2X lanes) drive `session` and never re-assemble
/// the pieces themselves.
pub struct F0Session {
    pub build: RustProviderBuild,
    pub admitted: ace_foreign_contract::manual::AdmittedForeign,
    /// Keeps the cdylib (and its exports) alive for the session's life;
    /// dropping the session drops it exactly once (原方 destroy 一次).
    pub provider: LoadedRustProvider,
    pub session: ace_av_core::foreign_exec::ForeignSession,
}

/// Open the frozen F0 provider as a session at generation 1.
pub fn open_f0_session(root: &Path) -> Result<F0Session, String> {
    let build = build_f0(root)?;
    open_session_from_build(&build, 1)
}

/// Re-open (or rebind) from an already-built provider unit at `generation`.
pub fn open_session_from_build(
    build: &RustProviderBuild,
    generation: u64,
) -> Result<F0Session, String> {
    let admitted = build
        .manual
        .clone()
        .admit(&f0_authority(build, generation))
        .map_err(|e| format!("admit: {e}"))?;
    let mut provider = load(build, generation)?;
    let mut session = ace_av_core::foreign_exec::ForeignSession::new();
    session.adopt(std::mem::replace(
        &mut provider.table,
        ace_av_core::foreign_exec::ForeignStubTable::new(generation),
    ));
    Ok(F0Session {
        build: build.clone(),
        admitted,
        provider,
        session,
    })
}

/// A loaded provider: keeps the OS library alive and owns the stub table
/// built from its exports. Dropping frees the library.
pub struct LoadedRustProvider {
    library: windows_sys::Win32::Foundation::HMODULE,
    pub table: ace_av_core::foreign_exec::ForeignStubTable,
}

impl Drop for LoadedRustProvider {
    fn drop(&mut self) {
        if !self.library.is_null() {
            unsafe {
                windows_sys::Win32::Foundation::FreeLibrary(self.library as _);
            }
        }
    }
}

/// Load the built cdylib in-process and resolve its exports into the AV
/// stub table at `generation`.
pub fn load(build: &RustProviderBuild, generation: u64) -> Result<LoadedRustProvider, String> {
    let path = build
        .cdylib
        .to_str()
        .ok_or("non-utf8 cdylib path")?
        .to_string();
    let wide: Vec<u16> = path.encode_utf16().chain(std::iter::once(0)).collect();
    let library = unsafe { windows_sys::Win32::System::LibraryLoader::LoadLibraryW(wide.as_ptr()) };
    if library.is_null() {
        return Err(format!(
            "LoadLibraryW failed for {}",
            build.cdylib.display()
        ));
    }
    let mut table = ace_av_core::foreign_exec::ForeignStubTable::new(generation);
    // The frozen panic channel: bind the exported flag before any call.
    let flag_name: Vec<u8> = b"f0_panic_flag\0".to_vec();
    let flag_sym = unsafe {
        windows_sys::Win32::System::LibraryLoader::GetProcAddress(library, flag_name.as_ptr())
    };
    let flag_fn: unsafe extern "C" fn() -> *mut u32 = match flag_sym {
        Some(sym) => unsafe { std::mem::transmute(sym) },
        None => {
            unsafe {
                windows_sys::Win32::Foundation::FreeLibrary(library as _);
            }
            return Err("the F0 panic flag export `f0_panic_flag` is missing".into());
        }
    };
    table.set_panic_flag(unsafe { flag_fn() });
    for f in &build.manifest.functions {
        let name: Vec<u8> = f.export.bytes().chain(std::iter::once(0)).collect();
        let sym = unsafe {
            windows_sys::Win32::System::LibraryLoader::GetProcAddress(library, name.as_ptr())
        };
        if sym.is_none() {
            unsafe {
                windows_sys::Win32::Foundation::FreeLibrary(library as _);
            }
            return Err(format!(
                "export `{}` missing from the provider cdylib",
                f.export
            ));
        }
        // PLAN-967 T-11: mixed view+scalar frames select their callable
        // by the PARAM SHAPE + form (view first, then i32 lanes).
        let view_first = matches!(
            f.params.first(),
            Some(auto_bindgen::rust_import::RustParam::ViewUtf8)
        );
        let scalars = f
            .params
            .iter()
            .filter(|p| matches!(p, auto_bindgen::rust_import::RustParam::I32))
            .count();
        let callable = match f.form {
            auto_bindgen::rust_import::RustFnForm::PureI32 if view_first && scalars == 2 => unsafe {
                ace_av_core::foreign_exec::ForeignCallable::ViewI32I32ToI32(std::mem::transmute::<
                    _,
                    unsafe extern "C" fn(*const u8, u64, i32, i32) -> i32,
                >(sym))
            },
            auto_bindgen::rust_import::RustFnForm::PureI32 => unsafe {
                ace_av_core::foreign_exec::ForeignCallable::PureI32(std::mem::transmute::<
                    _,
                    unsafe extern "C" fn(*const u8, u64) -> i32,
                >(sym))
            },
            auto_bindgen::rust_import::RustFnForm::PureBool if view_first && scalars == 1 => unsafe {
                ace_av_core::foreign_exec::ForeignCallable::ViewI32ToBool(std::mem::transmute::<
                    _,
                    unsafe extern "C" fn(*const u8, u64, i32) -> u8,
                >(sym))
            },
            auto_bindgen::rust_import::RustFnForm::PureBool => unsafe {
                ace_av_core::foreign_exec::ForeignCallable::PureBool(std::mem::transmute::<
                    _,
                    unsafe extern "C" fn(*const u8, u64) -> u8,
                >(sym))
            },
            auto_bindgen::rust_import::RustFnForm::FallibleI32 { .. }
                if view_first && scalars == 1 =>
            unsafe {
                ace_av_core::foreign_exec::ForeignCallable::ViewI32FallibleI32(
                    std::mem::transmute::<
                        _,
                        unsafe extern "C" fn(*const u8, u64, i32, *mut i32) -> i32,
                    >(sym),
                )
            },
            auto_bindgen::rust_import::RustFnForm::FallibleI32 { .. } => unsafe {
                ace_av_core::foreign_exec::ForeignCallable::FallibleOutI32(std::mem::transmute::<
                    _,
                    unsafe extern "C" fn(*const u8, u64, *mut i32) -> i32,
                >(sym))
            },
        };
        table
            .insert(ace_av_core::foreign_exec::ForeignStub {
                provider: build.manifest.provider.clone(),
                symbol: f.export.clone(),
                content_version: build.manual.facts.content_version.clone(),
                generation,
                callable,
            })
            .map_err(|e| e.to_string())?;
    }
    Ok(LoadedRustProvider { library, table })
}
