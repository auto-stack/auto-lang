//! PLAN-1093: prepared foreign providers on the ordinary language-source
//! worker. The provider cdylib is built ONCE in an explicit prepare lane
//! (test-discipline: daily cases never compile) into a provider closure
//! (`<artifact_root>/<provider>.descriptor.json` + the DLL). The worker
//! derives the manifest from the frozen provider source (pure), verifies
//! the prepared artifact identity (stamp + sha256), loads the DLL
//! in-process and runs the checked Auto source through the shared
//! admitted-foreign front door.

use std::path::{Path, PathBuf};

use ace_av_core::abc::encode_abc;
use ace_av_core::av::{execute_console_foreign, CancelFlag, ExecLimits, ExecTrace};
use ace_av_core::contract::ExecOutcome;
use ace_av_core::foreign_exec::{ForeignCallable, ForeignStub, ForeignStubTable};
use ace_av_core::mir_verify::MirVerifyError;
use ace_av_core::verify_mir;
use ace_core::source::adapter::compile_language_source_with_foreign;
use serde::Deserialize;
use serde_json::{json, Value as Json};

use crate::console_host::ConsoleCapture;

/// The request `foreign` section (declarative: kind + provider; the
/// prepared closure carries every artifact identity).
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ForeignSpec {
    /// `rust` | `c`.
    pub kind: String,
    /// Provider id (`f0_text`, or the C header/provider name).
    pub provider: String,
    /// The prepared artifact root (must exist; the run phase never
    /// compiles — missing/stale artifacts are named preparation failures).
    pub artifact_root: String,
}

/// The prepared provider closure written by the explicit prepare lane
/// (frozen source + built DLL identities). Schema-strict.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProviderDescriptor {
    pub schema: String,
    pub kind: String,
    pub provider: String,
    /// SHA-256 hex of the provider DLL the run binds.
    pub artifact_sha256: String,
    /// The frozen provider source: Rust annotated source (rust) or the
    /// annotated header text (c), base64.
    #[serde(default)]
    pub source_b64: Option<String>,
    /// The C provider implementation unit (c only), base64.
    #[serde(default)]
    pub unit_b64: Option<String>,
    /// The prepared manifest JSON (c only — its derivation needs the
    /// real preprocessor, so the manifest rides the closure), base64.
    #[serde(default)]
    pub manifest_b64: Option<String>,
    /// The rustc identity the prepared Rust build used (stamp axis).
    #[serde(default)]
    pub rustc_identity: Option<String>,
    /// The C provider header digest (blake3 hex, c only).
    #[serde(default)]
    pub provider_digest: Option<String>,
    /// DLL file name; defaults to `<provider>.dll` (C) / `f0_text_provider.dll` (rust).
    #[serde(default)]
    pub dll: Option<String>,
    /// Staticlib file name (rust only; defaults to `f0_text_provider.lib`).
    #[serde(default)]
    pub lib: Option<String>,
    /// Build-stamp file name inside `out/` (rust only; defaults to
    /// `f0-build-stamp.txt`). PLAN-1094: the ordinary AV view shares ONE
    /// provider root across registered providers, so a second rust provider
    /// (the py bridge) carries its own stamp file name.
    #[serde(default)]
    pub stamp: Option<String>,
}

pub const DESCRIPTOR_SCHEMA: &str = "plan1093.provider-descriptor/1";

fn diag(code: &str, message: impl Into<String>) -> Json {
    json!({"stage":"admit","code":code,"message":message.into()})
}

fn load_library(path: &Path) -> Result<windows_sys::Win32::Foundation::HMODULE, String> {
    let wide: Vec<u16> = path
        .to_string_lossy()
        .encode_utf16()
        .chain(std::iter::once(0))
        .collect();
    let library =
        unsafe { windows_sys::Win32::System::LibraryLoader::LoadLibraryW(wide.as_ptr()) };
    if library.is_null() {
        return Err(format!("LoadLibraryW failed for {}", path.display()));
    }
    Ok(library)
}

fn export_of(
    library: windows_sys::Win32::Foundation::HMODULE,
    name: &str,
) -> Result<unsafe extern "system" fn() -> isize, String> {
    let bytes: Vec<u8> = name.bytes().chain(std::iter::once(0)).collect();
    unsafe { windows_sys::Win32::System::LibraryLoader::GetProcAddress(library, bytes.as_ptr()) }
        .ok_or_else(|| format!("export `{name}` missing from the provider DLL"))
}

fn free_library(library: windows_sys::Win32::Foundation::HMODULE) {
    unsafe {
        windows_sys::Win32::Foundation::FreeLibrary(library as _);
    }
}

fn sha256_hex(path: &Path) -> Result<String, String> {
    let bytes = std::fs::read(path).map_err(|e| format!("read artifact {}: {e}", path.display()))?;
    Ok(ace_core::runtime_contract::sha256_hex(&bytes))
}

fn b64_bytes(value: &str, code: &str) -> Result<Vec<u8>, Json> {
    use base64::Engine as _;
    base64::engine::general_purpose::STANDARD
        .decode(value)
        .map_err(|e| diag(code, format!("{e}")))
}

/// Resolve the prepared closure for the declared provider.
pub fn load_provider(
    spec: &ForeignSpec,
) -> Result<(ace_foreign_contract::manual::AdmittedForeign, ForeignStubTable), Json> {
    let root = Path::new(&spec.artifact_root);
    let descriptor = root.join(format!("{}.descriptor.json", spec.provider));
    let bytes = std::fs::read(&descriptor).map_err(|e| {
        diag(
            "foreign.prepared",
            format!(
                "prepared provider closure missing at {}: {e} (run the prepare lane)",
                descriptor.display()
            ),
        )
    })?;
    let descriptor: ProviderDescriptor = serde_json::from_slice(&bytes)
        .map_err(|e| diag("foreign.prepared", format!("bad provider descriptor: {e}")))?;
    if descriptor.schema != DESCRIPTOR_SCHEMA {
        return Err(diag(
            "foreign.prepared",
            format!("expected {DESCRIPTOR_SCHEMA}, got {}", descriptor.schema),
        ));
    }
    if descriptor.kind != spec.kind || descriptor.provider != spec.provider {
        return Err(diag(
            "foreign.prepared",
            format!(
                "provider descriptor identity mismatch: declared {} {} but closure holds {} {}",
                spec.kind, spec.provider, descriptor.kind, descriptor.provider
            ),
        ));
    }
    match descriptor.kind.as_str() {
        "rust" => load_prepared_rust(&descriptor, root),
        "c" => load_prepared_c(&descriptor, root),
        other => Err(diag("foreign.kind", format!("unknown provider kind {other:?}"))),
    }
}

/// Derive the Rust provider manifest from the frozen source (pure) and
/// load the PREPARED cdylib (stamp + sha verified; no compiler here).
fn load_prepared_rust(
    descriptor: &ProviderDescriptor,
    root: &Path,
) -> Result<(ace_foreign_contract::manual::AdmittedForeign, ForeignStubTable), Json> {
    let source_bytes = descriptor
        .source_b64
        .as_deref()
        .ok_or_else(|| diag("foreign.prepared", "rust provider closure lacks its source"))?;
    let provider_source =
        String::from_utf8(b64_bytes(source_bytes, "foreign.source")?)
            .map_err(|e| diag("foreign.source", format!("{e}")))?;
    let manifest = auto_bindgen::rust_import::import(&provider_source)
        .map_err(|e| diag("foreign.import", format!("{}: {}", e.code, e.message)))?;
    let shim_source = auto_bindgen::rust_import::generate_shim(&manifest);
    let provider_digest = auto_bindgen::rust_import::provider_digest(&provider_source);
    let shim_digest = auto_bindgen::rust_import::shim_digest(&shim_source);
    let manual = auto_bindgen::rust_import::to_manual_manifest(
        &manifest,
        "f0-rust-provider/1",
        &provider_digest,
        &shim_digest,
    )
    .map_err(|e| diag("foreign.manifest", e))?;
    let rustc_identity = descriptor
        .rustc_identity
        .as_deref()
        .ok_or_else(|| diag("foreign.prepared", "rust provider closure lacks a rustc identity"))?;
    let dll_name = descriptor
        .dll
        .clone()
        .unwrap_or_else(|| "f0_text_provider.dll".to_string());
    let lib_name = descriptor
        .lib
        .clone()
        .unwrap_or_else(|| "f0_text_provider.lib".to_string());
    let stamp_name = descriptor
        .stamp
        .clone()
        .unwrap_or_else(|| "f0-build-stamp.txt".to_string());
    let out = root.join("out");
    let stamp_path = out.join(&stamp_name);
    let stamp = format!("{provider_digest}\n{shim_digest}\n{rustc_identity}\n");
    let prepared = stamp_path.is_file()
        && std::fs::read_to_string(&stamp_path)
            .map(|s| s == stamp)
            .unwrap_or(false)
        && out.join(&dll_name).is_file()
        && out.join(&lib_name).is_file();
    if !prepared {
        return Err(diag(
            "foreign.prepared",
            format!(
                "prepared Rust provider missing or stale at {} (run the prepare lane)",
                out.display()
            ),
        ));
    }
    let dll = out.join(&dll_name);
    let actual = sha256_hex(&dll).map_err(diag_wrap("foreign.artifact"))?;
    if actual != descriptor.artifact_sha256 {
        return Err(diag(
            "foreign.artifact",
            format!(
                "prepared provider dll sha {actual} != declared {}",
                descriptor.artifact_sha256
            ),
        ));
    }
    let authority = ace_foreign_contract::manual::HostAuthority {
        review_id: manual.review_id.clone(),
        contract_digest: manual
            .contract_digest()
            .map_err(|e| diag("foreign.manifest", e))?,
        provider: manual.provider.clone(),
        provider_digest,
        header_digest: manual.header_digest.clone(),
        physical_table_digest: manual.physical_table_digest.clone(),
        target: manual.target.clone(),
        capabilities: manual.facts.capabilities.iter().cloned().collect(),
        available_symbols: manifest
            .functions
            .iter()
            .map(|f| f.export.clone())
            .collect(),
        binding_generation: 1,
    };
    let admitted = manual
        .clone()
        .admit(&authority)
        .map_err(|e| diag("foreign.admit", e))?;
    let library = load_library(&dll).map_err(diag_wrap("foreign.load"))?;
    let lease = std::sync::Arc::new(PreparedRustLibrary(library));
    let mut table = ForeignStubTable::new(1);
    table.pin_code_image(lease);
    unsafe { bind_rust_exports(library, &manifest, &manual, 1, &mut table) }
        .map_err(diag_wrap("foreign.load"))?;
    Ok((admitted, table))
}

#[derive(Debug)]
struct PreparedRustLibrary(windows_sys::Win32::Foundation::HMODULE);

// A LoadLibrary reference may be held and freed on any worker thread.
#[cfg(windows)]
unsafe impl Send for PreparedRustLibrary {}
#[cfg(windows)]
unsafe impl Sync for PreparedRustLibrary {}

impl Drop for PreparedRustLibrary {
    fn drop(&mut self) {
        free_library(self.0);
    }
}

/// Populate the checked Rust provider table from a generated shim.
///
/// # Safety
/// `library` must be the pinned image built from the checked `manifest` and
/// its generated C shim. The table must keep that image leased through the
/// lifetime of every bound entry.
pub unsafe fn bind_rust_exports(
    library: windows_sys::Win32::Foundation::HMODULE,
    manifest: &auto_bindgen::rust_import::RustProviderManifest,
    manual: &ace_foreign_contract::manual::ManualManifest,
    generation: u64,
    table: &mut ForeignStubTable,
) -> Result<(), String> {
    bind_rust_panic_flag(library, table)?;
    for f in &manifest.functions {
        let sym = export_of(library, &f.export)?;
        // PLAN-967 T-11: mixed view+scalar frames select their callable
        // by the PARAM SHAPE + form (view first, then i32 lanes).
        let view_first = matches!(
            f.params.first(),
            Some(auto_bindgen::rust_import::RustParam::ViewUtf8)
                | Some(auto_bindgen::rust_import::RustParam::FrameCall)
        );
        let has_out_bytes = f
            .params
            .iter()
            .any(|p| matches!(p, auto_bindgen::rust_import::RustParam::OutBytes));
        let only_out_bytes = f.params.len() == 1 && has_out_bytes;
        let scalars = f
            .params
            .iter()
            .filter(|p| matches!(p, auto_bindgen::rust_import::RustParam::I32))
            .count();
        use auto_bindgen::rust_import::{RustCompletion, RustFnForm, RustParam};
        let callable = if let Some(mode) = f.completion {
            if f.params != [RustParam::I32]
                || f.form != RustFnForm::PureI32
                || f.owned.is_some()
                || f.frame.is_some()
            {
                return Err(format!(
                    "foreign.signature: invalid Rust completion export {}",
                    f.export
                ));
            }
            ace_av_core::foreign_exec::ForeignCallable::Completion {
                call: unsafe {
                    std::mem::transmute::<_, ace_av_core::foreign_exec::CompletionI32Fn>(sym)
                },
                asynchronous: mode == RustCompletion::Async,
            }
        } else if f.params.contains(&RustParam::Callback) {
            if f.params != [RustParam::Callback, RustParam::I32, RustParam::I32]
                || f.form != RustFnForm::PureI32
                || f.owned.is_some()
                || f.frame.is_some()
            {
                return Err(format!(
                    "foreign.signature: invalid Rust callback export {}",
                    f.export
                ));
            }
            ace_av_core::foreign_exec::ForeignCallable::Callback(unsafe {
                std::mem::transmute::<_, ace_av_core::foreign_exec::CallbackI32Fn>(sym)
            })
        } else if let Some(owned) = &f.owned {
            use ace_av_core::foreign_exec::{ForeignCallable, OwnedCall, OwnedHandleFn, OwnedOpen};
            use auto_bindgen::rust_import::RustOwnedRole;

            let (class, close) = manifest
                .owned_close(&owned.type_name)
                .ok_or_else(|| format!("missing close export for {}", owned.type_name))?;
            let close = export_of(library, close)?;
            // Every export was generated from the importer-checked Rust
            // signature; its physical C function type is fixed by the role.
            let close: OwnedHandleFn = unsafe { std::mem::transmute(close) };
            let operation = match owned.role {
                RustOwnedRole::OpenText => OwnedCall::Open {
                    start: OwnedOpen::Text(unsafe { std::mem::transmute(sym) }),
                    close,
                    class,
                },
                RustOwnedRole::OpenI32 => OwnedCall::Open {
                    start: OwnedOpen::I32(unsafe { std::mem::transmute(sym) }),
                    close,
                    class,
                },
                RustOwnedRole::OpenUnit => OwnedCall::Open {
                    start: OwnedOpen::Unit(unsafe { std::mem::transmute(sym) }),
                    close,
                    class,
                },
                RustOwnedRole::Borrow => OwnedCall::Borrow {
                    call: unsafe { std::mem::transmute(sym) },
                    class,
                },
                RustOwnedRole::Close => OwnedCall::Close { call: close, class },
            };
            ForeignCallable::Owned(operation)
        } else {
            match f.form {
                auto_bindgen::rust_import::RustFnForm::PureI32
                    if view_first && has_out_bytes && scalars == 0 =>
                unsafe {
                    ace_av_core::foreign_exec::ForeignCallable::ViewOutBytesToI32(
                        std::mem::transmute::<
                            _,
                            unsafe extern "C" fn(*const u8, u64, *mut u8, u64) -> i32,
                        >(sym),
                    )
                },
                auto_bindgen::rust_import::RustFnForm::PureI32 if only_out_bytes => unsafe {
                    ace_av_core::foreign_exec::ForeignCallable::OutBytesToI32(
                        std::mem::transmute::<_, unsafe extern "C" fn(*mut u8, u64) -> i32>(sym),
                    )
                },
                auto_bindgen::rust_import::RustFnForm::PureI32 if view_first && scalars == 2 => unsafe {
                    ace_av_core::foreign_exec::ForeignCallable::ViewI32I32ToI32(
                        std::mem::transmute::<
                            _,
                            unsafe extern "C" fn(*const u8, u64, i32, i32) -> i32,
                        >(sym),
                    )
                },
                auto_bindgen::rust_import::RustFnForm::PureI32 => unsafe {
                    ace_av_core::foreign_exec::ForeignCallable::PureI32(std::mem::transmute::<
                        _,
                        unsafe extern "C" fn(*const u8, u64) -> i32,
                    >(sym))
                },
                auto_bindgen::rust_import::RustFnForm::PureBool if view_first && scalars == 1 => unsafe {
                    ace_av_core::foreign_exec::ForeignCallable::ViewI32ToBool(
                        std::mem::transmute::<_, unsafe extern "C" fn(*const u8, u64, i32) -> u8>(
                            sym,
                        ),
                    )
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
                    ace_av_core::foreign_exec::ForeignCallable::FallibleOutI32(
                        std::mem::transmute::<
                            _,
                            unsafe extern "C" fn(*const u8, u64, *mut i32) -> i32,
                        >(sym),
                    )
                },
            }
        };
        table
            .insert(ace_av_core::foreign_exec::ForeignStub {
                provider: manifest.provider.clone(),
                symbol: f.export.clone(),
                content_version: manual.facts.content_version.clone(),
                generation,
                callable,
                frame: f.frame.is_some(),
            })
            .map_err(|e| e.to_string())?;
    }
    Ok(())
}

/// Load a prepared C provider closure (manifest JSON + DLL) and map its
/// entries onto the AV stub table. The physical param shapes live in the
/// resource signature; the frozen domain admits the ViewText scalar pair.
fn load_prepared_c(
    descriptor: &ProviderDescriptor,
    root: &Path,
) -> Result<(ace_foreign_contract::manual::AdmittedForeign, ForeignStubTable), Json> {
    let manifest_b64 = descriptor
        .manifest_b64
        .as_deref()
        .ok_or_else(|| diag("foreign.prepared", "c provider closure lacks its manifest"))?;
    let manifest: ace_foreign_contract::manual::ManualManifest =
        serde_json::from_slice(&b64_bytes(manifest_b64, "foreign.manifest")?)
            .map_err(|e| diag("foreign.manifest", format!("{e}")))?;
    let dll_name = descriptor
        .dll
        .clone()
        .unwrap_or_else(|| format!("{}.dll", descriptor.provider));
    let dll = PathBuf::from(root).join(dll_name);
    let actual = sha256_hex(&dll).map_err(diag_wrap("foreign.artifact"))?;
    if actual != descriptor.artifact_sha256 {
        return Err(diag(
            "foreign.artifact",
            format!(
                "prepared provider dll sha {actual} != declared {}",
                descriptor.artifact_sha256
            ),
        ));
    }
    let provider_digest = descriptor
        .provider_digest
        .clone()
        .ok_or_else(|| diag("foreign.prepared", "c provider closure lacks a header digest"))?;
    let authority = ace_foreign_contract::manual::HostAuthority {
        review_id: manifest.review_id.clone(),
        contract_digest: manifest
            .contract_digest()
            .map_err(|e| diag("foreign.manifest", e))?,
        provider: manifest.provider.clone(),
        provider_digest,
        header_digest: manifest.header_digest.clone(),
        physical_table_digest: manifest.physical_table_digest.clone(),
        target: manifest.target.clone(),
        capabilities: manifest.facts.capabilities.iter().cloned().collect(),
        available_symbols: manifest.facts.functions.iter().map(|f| f.symbol.clone()).collect(),
        binding_generation: 1,
    };
    let admitted = manifest
        .clone()
        .admit(&authority)
        .map_err(|e| diag("foreign.admit", e))?;
    let library = load_library(&dll).map_err(diag_wrap("foreign.load"))?;
    let mut table = ForeignStubTable::new(1);
    for f in &manifest.facts.functions {
        let sig = f.resource.as_ref().ok_or_else(|| {
            diag(
                "foreign.unsupported-shape",
                format!("{} lacks a resource signature", f.symbol),
            )
        })?;
        let callable = match (&sig.params[..], &sig.form) {
            (
                [ace_foreign_contract::resource::ResKind::ViewText],
                ace_foreign_contract::resource::ResourceForm::Pure {
                    ret: ace_foreign_contract::resource::ResOut::I32,
                },
            ) => unsafe {
                ForeignCallable::PureI32(
                    std::mem::transmute::<_, unsafe extern "C" fn(*const u8, u64) -> i32>(
                        export_of(library, &f.symbol).map_err(|e| {
                            free_library(library);
                            diag("foreign.load", e)
                        })?,
                    ),
                )
            },
            (
                [ace_foreign_contract::resource::ResKind::ViewText],
                ace_foreign_contract::resource::ResourceForm::Pure {
                    ret: ace_foreign_contract::resource::ResOut::Bool,
                },
            ) => unsafe {
                ForeignCallable::PureBool(
                    std::mem::transmute::<_, unsafe extern "C" fn(*const u8, u64) -> u8>(
                        export_of(library, &f.symbol).map_err(|e| {
                            free_library(library);
                            diag("foreign.load", e)
                        })?,
                    ),
                )
            },
            _ => {
                free_library(library);
                return Err(diag(
                    "foreign.unsupported-shape",
                    format!(
                        "the C entry `{}` carries a shape outside the frozen ordinary-AV domain (params {:?})",
                        f.symbol, sig.params
                    ),
                ));
            }
        };
        table
            .insert(ForeignStub {
                provider: manifest.provider.clone(),
                symbol: f.symbol.clone(),
                content_version: manifest.facts.content_version.clone(),
                generation: 1,
                callable,
                frame: false,
            })
            .map_err(|e| {
                free_library(library);
                diag("foreign.load", e.to_string())
            })?;
    }
    Ok((admitted, table))
}

fn diag_wrap(code: &'static str) -> impl Fn(String) -> Json {
    move |message| diag(code, message)
}

/// The F0 panic-flag export a generated Rust shim always provides.
fn bind_rust_panic_flag(
    library: windows_sys::Win32::Foundation::HMODULE,
    table: &mut ForeignStubTable,
) -> Result<(), String> {
    let flag_sym = export_of(library, "f0_panic_flag")?;
    let flag_fn: unsafe extern "C" fn() -> *mut u32 = unsafe { std::mem::transmute(flag_sym) };
    table.set_panic_flag(unsafe { flag_fn() });
    Ok(())
}

/// The foreign-capable ordinary chain: shared Auto front door with an
/// admitted provider → closures lowering → MIR verify → ABC foreign
/// admission → AV execute with console capture and per-run stub checks.
pub fn run_foreign_chain(
    source: &str,
    path: &str,
    caps: &[String],
    foreign: &ForeignSpec,
    capture: &mut ConsoleCapture,
) -> Result<(ExecOutcome, ExecTrace), Json> {
    let (admitted, mut table) = load_provider(foreign)?;
    // PLAN-1094: the py bridge is an ordinary-AV provider like any other,
    // but its failures stage the Python exception lanes — the runtime must
    // attach the error observers so a failed py op unwinds as a
    // `LanguageErrorKind::Python(class)` language error (catchable).
    if foreign.provider == "py" {
        table
            .bind_python_error_observers(&admitted)
            .map_err(|e| diag("foreign.py-observer", format!("{}: {}", e.code, e.message)))?;
    }
    let program = compile_foreign_program(source, path, "d_main", caps, &admitted)?;
    execute_foreign_program(&program, caps, &CancelFlag::new(false), capture, &mut table)
}

/// Portable output of the ordinary foreign front door.
pub struct ForeignProgram {
    pub owner: ace_av_core::mir::OwnerScope,
    pub abc: Vec<u8>,
}

pub fn compile_foreign_program(
    source: &str,
    path: &str,
    entry: &str,
    caps: &[String],
    admitted: &ace_foreign_contract::manual::AdmittedForeign,
) -> Result<ForeignProgram, Json> {
    let compiled =
        compile_language_source_with_foreign(source, path, caps, admitted).map_err(|ds| {
            let d = &ds[0];
            json!({
                "stage": format!("{:?}", d.stage).to_lowercase(),
                "code": d.code, "message": d.message,
                "path": path, "span": {"start": d.span.start, "end": d.span.end}
            })
        })?;
    let mir = ace_lowering::lower_checked_auto_language(&compiled.checked, entry, 0)
        .map_err(|e| json!({"stage":"lower","code":"lower","message":format!("{e:?}")}))?;
    verify_mir(&mir).map_err(|error| {
        let mut diagnostic = json!({
            "stage": "verify",
            "code": error.code(),
            "message": format!("{error:?}"),
        });
        if let MirVerifyError::Invalid {
            span: Some(span), ..
        }
        | MirVerifyError::Unsupported {
            span: Some(span), ..
        } = error
        {
            diagnostic["path"] = json!(path);
            diagnostic["span"] = json!({"start": span.start, "end": span.end});
        }
        diagnostic
    })?;
    super::language_source_worker::authorize_console_calls(&mir, caps, path)?;
    let abc = encode_abc(&mir)
        .map_err(|e| json!({"stage":"encode","code":"abc","message":format!("{e:?}")}))?;
    Ok(ForeignProgram {
        owner: mir.owner,
        abc: abc.to_flat(),
    })
}

/// Re-admit the portable program against the current session and grants.
pub fn execute_foreign_program(
    program: &ForeignProgram,
    caps: &[String],
    cancel: &CancelFlag,
    capture: &mut ConsoleCapture,
    table: &mut ForeignStubTable,
) -> Result<(ExecOutcome, ExecTrace), Json> {
    let decoded = ace_av_core::abc::decode_abc(&program.abc)
        .map_err(|e| json!({"stage":"admit","code":"abc","message":format!("{e:?}")}))?;
    let loaded = ace_av_core::abc_verify::verify_abc_foreign(decoded, &program.owner, table)
        .map_err(|e| json!({"stage":"admit","code":"abc","message":e.to_string()}))?;
    let declarations = loaded
        .module()
        .funcs
        .iter()
        .flat_map(|function| &function.blocks)
        .flat_map(|block| &block.instrs)
        .filter_map(|instr| match instr {
            ace_av_core::mir::MirInstr::ForeignCall { declaration, .. } => Some(declaration),
            _ => None,
        });
    for declaration in declarations {
        let missing = std::iter::once(ace_foreign_contract::manual::SOURCE_CAP)
            .chain(declaration.capabilities.iter().map(String::as_str))
            .find(|need| !caps.iter().any(|granted| granted.as_str() == *need));
        if let Some(missing) = missing {
            return Err(diag(
                "source.foreign-capability",
                format!("current foreign grant {missing} is required"),
            ));
        }
    }
    let grants: Vec<_> = caps
        .iter()
        .filter(|cap| {
            matches!(
                cap.as_str(),
                ace_av_core::host::STDOUT | ace_av_core::host::STDERR
            )
        })
        .cloned()
        .collect();
    execute_console_foreign(
        &loaded,
        &ExecLimits::default(),
        cancel,
        &grants,
        capture,
        table,
    )
    .map_err(|e| json!({"stage":"run","code":e.code,"message":e.message}))
}
