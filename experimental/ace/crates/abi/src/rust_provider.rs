//! PLAN-967 F0 (T-F0-I): the Rust provider build unit and in-process
//! loader. One build closure (B-01 seed): the frozen annotated provider
//! source and the GENERATED C-compatible shim compile together into a
//! single `cdylib`; the loader resolves the `#[no_mangle]` exports into
//! the AV executor's stub table. No second loader/manifest world — the
//! shared manual manifest and `ForeignStubTable` are the only contracts.

use std::path::{Path, PathBuf};

/// The frozen F0 provider fixture (the annotations ARE the metadata).
pub fn f0_provider_source() -> &'static str {
    include_str!("../rustside/f0_text.rs")
}

/// One built provider: the cdylib plus every identity the host needs to
/// admit and load it.
#[derive(Clone)]
pub struct RustProviderBuild {
    pub cdylib: PathBuf,
    pub manifest: auto_bindgen::rust_import::RustProviderManifest,
    pub shim_source: String,
    pub manual: ace_foreign_contract::manual::ManualManifest,
    pub provider_digest: String,
    pub rustc_identity: String,
}

fn run_rustc(args: &[&str], cwd: &Path) -> Result<(i32, String, String), String> {
    let mut command = std::process::Command::new("rustc");
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
    Ok(RustProviderBuild {
        cdylib,
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
        let callable = match f.form {
            auto_bindgen::rust_import::RustFnForm::PureI32 => unsafe {
                ace_av_core::foreign_exec::ForeignCallable::PureI32(std::mem::transmute::<
                    _,
                    unsafe extern "C" fn(*const u8, u64) -> i32,
                >(sym))
            },
            auto_bindgen::rust_import::RustFnForm::PureBool => unsafe {
                ace_av_core::foreign_exec::ForeignCallable::PureBool(std::mem::transmute::<
                    _,
                    unsafe extern "C" fn(*const u8, u64) -> u8,
                >(sym))
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
