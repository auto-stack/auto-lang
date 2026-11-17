//! PLAN-969 E0 (T-E0-I): the reference explicit-ACE executor bridge.
//!
//! This crate is the executable side of the public facade frozen in
//! `auto_lang_core::ace_entry` (`AceRequest`/`AceOutcome`/`run_ace`):
//! the host calls [`install()`] once and every
//! `auto_lang_core::run_ace(AceEngineChoice::AceAV, ..)` request is
//! served here — synchronously, on the caller's (UI main) thread, with
//! per-run authorization/identity re-checks and no path into the legacy
//! AutoVM.
//!
//! Spine (frozen E0 lane selection):
//! - pure Auto source → `ace_av::query_entry::compile_scalar_or_core_source_product`
//!   → `ace_lowering::lower_checked_owned` → `verify_mir` → ABC encode/
//!   decode/`verify_abc` → `ace_av_core::av::execute`;
//! - provider source (`use.rs` frontdoor) → per-run manifest admission +
//!   provider load (own `ForeignStubTable`) → `compile_text_record_product`
//!   with the admitted environment → same lower/verify/wire chain →
//!   `ForeignSession` adopt/execute/teardown (a torn-down session refuses
//!   late callees before any dispatch).
//!
//! E0 scope: ONE synchronous run per request; the provider registry is
//! host-owned lifecycle (build/admit/revoke/readmit/teardown). Sessions
//! are created and closed inside the request — cross-request warm
//! sessions, edits and async are NOT part of this slice.

#![cfg(windows)]
// (crate root gate, the ace-abi pattern: this crate path-depends on the
// Windows-only ace-abi provider lane, so non-Windows builds compile it as
// an empty crate rather than failing on ace-abi types)

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, OnceLock, RwLock};

use ace_av_core::av::{CancelFlag, ExecLimits};
use ace_av_core::contract::{fnv1a64, ExecOutcome, Value};
use ace_av_core::mir::MirModule;
use ace_core::atom_text::Diagnostic;
use ace_core::verify::CheckedModule;
use auto_lang_core::{
    AceConsoleReport, AceExecutor, AceOutcome, AceRequest, AceRunReport, AceSpan, AceValue,
};

/// The engine identity this bridge serves (matches
/// `auto_lang_core::AceEngineChoice::AceAV`).
mod debug;
/// PLAN-982（B-CONFIG-I E6）：配置文档桥（AceExecutor::eval_config）。
pub mod config_document;
pub mod python_transpile;
pub mod rust_transpile;
pub mod project;
pub mod session_bridge;

pub const ENGINE_ID: &str = "av";

// ---------------------------------------------------------------------------
// Host-owned provider registry
// ---------------------------------------------------------------------------

struct ProviderEntry {
    build: ace_abi::rust_provider::RustProviderBuild,
    /// Revoked entries refuse every request BEFORE any dispatch until the
    /// host readmits them.
    revoked: AtomicBool,
    /// The capability names this provider's admission actually grants;
    /// request grants are checked against this set per run.
    granted_caps: Vec<String>,
}

struct HostExecutor {
    providers: RwLock<HashMap<String, Arc<ProviderEntry>>>,
}

static EXECUTOR: OnceLock<Arc<HostExecutor>> = OnceLock::new();

fn host() -> &'static Arc<HostExecutor> {
    EXECUTOR.get_or_init(|| {
        Arc::new(HostExecutor {
            providers: RwLock::new(HashMap::new()),
        })
    })
}

/// Install the process-wide ACE executor (idempotent — the same host
/// singleton serves every call). Until this runs, explicit ACE requests
/// report `ace.host/executor-unregistered`.
///
/// PLAN-971 B-PROOF-I: installing the host also installs the reference
/// PROJECT executor (the `ace_entry::run_ace_project` face) over the
/// 960 packages lane — same single-owner discipline, idempotent with
/// the same singleton. The project registry is released only by
/// `shutdown_ace_project_executor`; the E0 [`shutdown`] below
/// intentionally does not touch it, so E0 consumers keep their close
/// semantics unchanged.
pub fn install() -> Result<(), String> {
    auto_lang_core::install_ace_executor(host().clone() as Arc<dyn AceExecutor>)?;
    auto_lang_core::install_ace_project_executor(project::project_host())
}

/// Release the installed executor (host close; see
/// `auto_lang_core::shutdown_ace_executor`).
pub fn shutdown() -> bool {
    auto_lang_core::shutdown_ace_executor()
}

/// Build the frozen F0 text provider (`text_byte_len`/`text_is_ascii`/
/// `text_first_byte`) from its source and register it under `name`.
/// `root` is the caller-owned scratch root for the rustc build products
/// (transient scratch; not plan evidence).
#[cfg(windows)]
pub fn build_f0_text_provider(name: &str, root: &Path) -> Result<(), String> {
    build_f0_text_providers(&[name], root)
}

/// Build the frozen F0 text provider ONCE and register it under every
/// name in `names` (separate host identities sharing one build unit —
/// used to prove authorization isolation between identities).
#[cfg(windows)]
pub fn build_f0_text_providers(names: &[&str], root: &Path) -> Result<(), String> {
    let build = ace_abi::rust_provider::build_f0(root)
        .map_err(|e| format!("F0 provider build failed: {e}"))?;
    for name in names {
        register_built_provider(name, &build)?;
    }
    Ok(())
}

fn register_built_provider(
    name: &str,
    build: &ace_abi::rust_provider::RustProviderBuild,
) -> Result<(), String> {
    let granted_caps = vec![
        ace_foreign_contract::manual::SOURCE_CAP.to_string(),
        ace_foreign_contract::resource::RESOURCE_CAP.to_string(),
    ];
    let mut providers = host()
        .providers
        .write()
        .map_err(|e| format!("provider registry lock poisoned: {e}"))?;
    if providers.contains_key(name) {
        return Err(format!("provider `{name}` is already registered"));
    }
    providers.insert(
        name.to_string(),
        Arc::new(ProviderEntry {
            build: clone_build(build),
            revoked: AtomicBool::new(false),
            granted_caps,
        }),
    );
    Ok(())
}

/// Field-wise copy of an already-built provider unit (registry entries
/// own their copy; the build products on disk are shared read-only).
#[cfg(windows)]
fn clone_build(
    build: &ace_abi::rust_provider::RustProviderBuild,
) -> ace_abi::rust_provider::RustProviderBuild {
    ace_abi::rust_provider::RustProviderBuild {
        cdylib: build.cdylib.clone(),
        staticlib: build.staticlib.clone(),
        manifest: build.manifest.clone(),
        shim_source: build.shim_source.clone(),
        manual: build.manual.clone(),
        provider_digest: build.provider_digest.clone(),
        rustc_identity: build.rustc_identity.clone(),
    }
}

/// PLAN-980 NATIVE (B-NATIVE-I): register an arbitrary annotated Rust
/// provider SOURCE under `name` — the generic admission face for
/// domain-owned providers (the frozen F0 fixtures above stay the only
/// special-cased identities). The build rides `build_source`'s freshness
/// cache (same provider/shim digests + rustc identity + artifacts on
/// disk ⇒ no rustc); `root` is the caller-owned scratch root. Returns
/// the built cdylib path (hosts load it to install their own
/// side-channel exports — same module, LoadLibraryW refcounting).
#[cfg(windows)]
pub fn register_source_provider(name: &str, root: &Path, source: &str) -> Result<PathBuf, String> {
    let build = ace_abi::rust_provider::build_source(root, source)
        .map_err(|e| format!("source provider build failed: {e}"))?;
    let cdylib = build.cdylib.clone();
    register_built_provider(name, &build)?;
    Ok(cdylib)
}

/// Revoke a provider: every later request naming it is refused before any
/// dispatch until [`readmit_provider`]. Returns whether an admitted
/// provider was revoked.
#[cfg(windows)]
pub fn revoke_provider(name: &str) -> bool {
    lookup_provider(name).is_some_and(|entry| !entry.revoked.swap(true, Ordering::SeqCst))
}

/// Readmit a revoked provider (the exact-fix path: the SAME public door
/// serves again after the host restores authority).
#[cfg(windows)]
pub fn readmit_provider(name: &str) -> bool {
    lookup_provider(name).is_some_and(|entry| entry.revoked.swap(false, Ordering::SeqCst))
}

/// Tear a provider down entirely (host close): the registration dies and
/// later requests report `ace.provider/unknown`.
#[cfg(windows)]
pub fn teardown_provider(name: &str) -> bool {
    let Ok(mut providers) = host().providers.write() else {
        return false;
    };
    providers.remove(name).is_some()
}

#[cfg(windows)]
fn lookup_provider(name: &str) -> Option<Arc<ProviderEntry>> {
    let providers = host().providers.read().ok()?;
    providers.get(name).cloned()
}

// ---------------------------------------------------------------------------
// Executor: lane selection + mapping
// ---------------------------------------------------------------------------

impl AceExecutor for HostExecutor {
    /// PLAN-982（B-CONFIG-I E6）：配置文档走 ACE 宿主车道
    /// （compile→lower→verify→ABC→session→execute→export）。没有桥时 SDK 侧
    /// 的默认实现返回 ce.config/executor-unsupported——这里必须真的服务，
    /// 且失败不回退旧配置引擎。
    fn eval_config(
        &self,
        request: &auto_lang_core::ace_config::AceConfigRequest<'_>,
    ) -> auto_lang_core::ace_config::AceConfigReport {
        config_document::eval_request(request)
    }

    fn execute(&self, request: &AceRequest<'_>) -> AceRunReport {
        // T-08 compat discipline: argv/stdin are EXPLICIT request inputs
        // the AV lane cannot serve yet — named refusals before any lane
        // work (never a silent drop, never a legacy-VM route).
        if !request.args.is_empty() {
            return denied_report(
                "ace.args/unsupported",
                "the AV execution lane has no argv surface yet; the request is refused rather than silently dropped",
            );
        }
        if request.input.is_some() {
            return denied_report(
                "ace.input/unsupported",
                "the AV execution lane has no stdin surface yet; the request is refused rather than silently dropped",
            );
        }
        if request.console {
            if request.provider.is_some() {
                // Freeze (T-02): the console lane and the F0 foreign lane
                // are not composed yet — an explicit named refusal, never
                // a silent route to the other lane.
                return denied_report(
                    "ace.console/foreign-unsupported",
                    "console+provider composition is outside the frozen T-02 surface",
                );
            }
            return self.run_console(request);
        }
        match request.provider {
            None => self.run_pure(request),
            Some(name) => self.run_foreign(request, name),
        }
    }
}

fn report(
    outcome: AceOutcome,
    diag_count: usize,
    callee: Option<u64>,
    rejected: Option<u64>,
) -> AceRunReport {
    AceRunReport {
        engine: ENGINE_ID,
        outcome,
        diag_count,
        callee_count: callee,
        rejected_count: rejected,
        wall_ms: 0.0, // filled by auto_lang_core::run_ace (whole-request wall)
        console: None,
    }
}

fn diag_report(
    stage: &str,
    code: &str,
    message: impl Into<String>,
    span: Option<AceSpan>,
) -> AceRunReport {
    report(
        AceOutcome::Diagnostic {
            stage: stage.to_string(),
            code: code.to_string(),
            message: message.into(),
            span,
        },
        1,
        None,
        None,
    )
}

fn host_report(code: &str, message: impl Into<String>) -> AceRunReport {
    report(
        AceOutcome::HostFailure {
            code: code.to_string(),
            message: message.into(),
        },
        0,
        None,
        None,
    )
}

fn denied_report(code: &str, message: impl Into<String>) -> AceRunReport {
    report(
        AceOutcome::Denied {
            code: code.to_string(),
            message: message.into(),
        },
        0,
        None,
        None,
    )
}

/// The ACE frontend's own diagnostics (stage/code/message/span) preserved
/// verbatim — the facade never rewrites the producing stage or span.
fn diag_list_report(diags: &[Diagnostic]) -> AceRunReport {
    let Some(primary) = diags.first() else {
        return host_report(
            "ace.host/empty-diagnostics",
            "compile failed without a diagnostic",
        );
    };
    report(
        AceOutcome::Diagnostic {
            stage: primary.stage.as_str().to_string(),
            code: primary.code.clone(),
            message: primary.message.clone(),
            span: Some(AceSpan {
                start: primary.span.start as u64,
                end: primary.span.end as u64,
            }),
        },
        diags.len(),
        None,
        None,
    )
}

fn limits_from(request: &AceRequest<'_>) -> ExecLimits {
    let mut limits = ExecLimits::default();
    if let Some(wall) = request.wall {
        limits.wall = wall;
    }
    limits
}

fn cancel_flag<'a>(request: &'a AceRequest<'_>, fallback: &'a CancelFlag) -> &'a CancelFlag {
    match request.cancel {
        Some(flag) => flag,
        None => fallback,
    }
}

pub(crate) fn map_value(value: Value) -> AceValue {
    match value {
        Value::Unit => AceValue::Unit,
        Value::Bool(b) => AceValue::Bool(b),
        Value::I8(v) => AceValue::I8(v),
        Value::U8(v) => AceValue::U8(v),
        Value::I32(v) => AceValue::I32(v),
        Value::U32(v) => AceValue::U32(v),
        Value::I64(v) => AceValue::I64(v),
        Value::U64(v) => AceValue::U64(v),
        Value::F32(v) => AceValue::F32(v),
        Value::F64(v) => AceValue::F64(v),
        Value::Char(c) => AceValue::Char(c),
        Value::Record(_) => AceValue::Opaque("record"),
        _ => AceValue::Opaque("opaque"),
    }
}

fn map_span(span: &ace_av_core::contract::SpanInfo) -> AceSpan {
    AceSpan {
        start: span.start as u64,
        end: span.end as u64,
    }
}

fn map_outcome(outcome: ExecOutcome, callee: Option<u64>, rejected: Option<u64>) -> AceRunReport {
    let mapped = match outcome {
        ExecOutcome::Success { value, steps } => AceOutcome::Success {
            value: map_value(value),
            steps,
        },
        ExecOutcome::Trap { kind, span } => AceOutcome::Diagnostic {
            stage: "run".to_string(),
            code: format!("av.trap/{kind:?}"),
            message: format!("language trap: {kind:?}"),
            span: span.as_ref().map(map_span),
        },
        ExecOutcome::Error { message, span } => AceOutcome::Diagnostic {
            stage: "run".to_string(),
            code: "av.error".to_string(),
            message,
            span: span.as_ref().map(map_span),
        },
        ExecOutcome::Resource { kind, message } => AceOutcome::ResourceDenied {
            kind: format!("{kind:?}"),
            message,
        },
        ExecOutcome::Cancelled => AceOutcome::Cancelled,
        ExecOutcome::Infra(message) => AceOutcome::HostFailure {
            code: "ace.executor/infra".to_string(),
            message,
        },
        // PLAN-973: the guest requested its own termination — a normal
        // named outcome (the requested exit code rides the message).
        ExecOutcome::Exit { code } => AceOutcome::HostFailure {
            code: "av.exit".to_string(),
            message: format!("process exit requested: {code}"),
        },
    };
    report(mapped, 0, callee, rejected)
}

/// Entry-existence pre-check (the `d_main` convention): a wrong entry id
/// is diagnosed at the `entry` stage, not as a lower-stage crash.
fn entry_missing(entry_def_id: &str) -> AceRunReport {
    diag_report(
        "entry",
        "ace.entry/missing",
        format!("entry def `{entry_def_id}` not found in the checked module"),
        None,
    )
}

/// Lower the checked module to MIR through the shared seams, after the
/// entry pre-check; every failure maps to a staged diagnostic.
fn lower_and_check_entry(
    checked: &CheckedModule,
    entry_def_id: &str,
    source: &str,
    owned: bool,
) -> Result<MirModule, AceRunReport> {
    let bundle = checked.bundle();
    let module = bundle
        .modules
        .first()
        .ok_or_else(|| host_report("ace.host/empty-bundle", "checked bundle has no module"))?;
    if !module.defs.iter().any(|d| d.id_text == entry_def_id) {
        return Err(entry_missing(entry_def_id));
    }
    // Lane-correct lowering entry: `lower_checked_owned` is the F0
    // owned/text-record family; `lower_checked` is the scalar lane — the
    // same seam the AV warm session uses for its cold compile.
    let lowered = if owned {
        ace_lowering::hir_adapter::lower_checked_owned(
            checked,
            entry_def_id,
            fnv1a64(source.as_bytes()),
        )
    } else {
        ace_lowering::hir_adapter::lower_checked(checked, entry_def_id, fnv1a64(source.as_bytes()))
    };
    let mir =
        lowered.map_err(|e| diag_report("lower", "ace.lower/failed", format!("{e:?}"), None))?;
    ace_av_core::mir_verify::verify_mir(&mir)
        .map_err(|e| diag_report("verify", "ace.mir/verify-failed", format!("{e:?}"), None))?;
    Ok(mir)
}

/// Shared ABC encode/decode/admission tail (pure lane admits without a
/// stub table; the foreign lane admits through its loaded table).
fn wire_admit(
    mir: &MirModule,
    table: Option<&mut ace_av_core::foreign_exec::ForeignStubTable>,
) -> Result<ace_av_core::abc_verify::LoadedModule, AceRunReport> {
    let encoded = ace_av_core::abc::encode_abc(mir)
        .map_err(|e| diag_report("wire", "ace.abc/encode-failed", format!("{e:?}"), None))?
        .to_flat();
    let decoded = ace_av_core::abc::decode_abc(&encoded)
        .map_err(|e| diag_report("wire", "ace.abc/decode-failed", format!("{e:?}"), None))?;
    match table {
        None => ace_av_core::abc_verify::verify_abc(decoded, &mir.owner),
        Some(table) => ace_av_core::abc_verify::verify_abc_foreign(decoded, &mir.owner, table),
    }
    .map_err(|e| denied_report("ace.abc/refused", format!("{e:?}")))
}

impl HostExecutor {
    // -- pure Auto lane -----------------------------------------------------

    fn run_pure(&self, request: &AceRequest<'_>) -> AceRunReport {
        if !request.grants.is_empty() {
            // Grants are provider-lane authority in E0; a pure run has
            // nothing admitted to check them against — refuse explicitly.
            return denied_report(
                "ace.grant/no-provider",
                "grants require an admitted provider; the pure lane grants nothing",
            );
        }
        let product = match ace_av::query_entry::compile_scalar_or_core_source_product(
            request.source,
            "p969.at",
            request.capabilities,
            "auto-lang-ace-host",
        ) {
            Ok(product) => product,
            Err(diags) => return diag_list_report(&diags),
        };
        let mir = match lower_and_check_entry(
            &product.compiled.checked,
            request.entry_def_id,
            request.source,
            false,
        ) {
            Ok(mir) => mir,
            Err(report) => return report,
        };
        let loaded = match wire_admit(&mir, None) {
            Ok(loaded) => loaded,
            Err(report) => return report,
        };
        let fallback = CancelFlag::new(false);
        let (outcome, _trace) = ace_av_core::av::execute(
            &loaded,
            &limits_from(request),
            cancel_flag(request, &fallback),
        );
        map_outcome(outcome, None, None)
    }

    // -- console lane (T-02: print/eprint under per-run grants) ----------

    fn run_console(&self, request: &AceRequest<'_>) -> AceRunReport {
        let compiled = match ace_core::source::compile_host_console_source(
            request.source,
            "p969.at",
            request.capabilities,
        ) {
            Ok(compiled) => compiled,
            Err(diags) => return diag_list_report(&diags),
        };
        // The console front door enforces the main-only entry policy; a
        // wrong entry surfaces the adapter's own named code (`lower.entry`).
        let mir = match ace_lowering::hir_adapter::lower_checked_owned(
            &compiled.checked,
            request.entry_def_id,
            fnv1a64(request.source.as_bytes()),
        ) {
            Ok(mir) => mir,
            Err(e) => return diag_report("lower", &e.code(), format!("{e:?}"), None),
        };
        if let Err(e) = ace_av_core::mir_verify::verify_mir(&mir) {
            return diag_report("verify", "ace.mir/verify-failed", format!("{e:?}"), None);
        }
        let loaded = match wire_admit(&mir, None) {
            Ok(loaded) => loaded,
            Err(report) => return report,
        };
        let mut cap = ace_av::console_host::ConsoleCapture::default();
        let fallback = CancelFlag::new(false);
        // Per-run console authorization: the request grants are the ONLY
        // authority (structural credentials carry none).
        let result = ace_av_core::av::execute_console(
            &loaded,
            &limits_from(request),
            cancel_flag(request, &fallback),
            request.grants,
            &mut cap,
        );
        let mut out = match result {
            Ok((outcome, _trace)) => map_outcome(outcome, None, None),
            Err(e) => report(
                AceOutcome::Denied {
                    code: e.code.to_string(),
                    message: e.message.clone(),
                },
                0,
                None,
                None,
            ),
        };
        out.console = Some(AceConsoleReport {
            stdout: cap.stdout.clone(),
            stderr: cap.stderr.clone(),
            dispatches: cap.dispatches,
        });
        out
    }

    // -- F0 Rust provider lane (Windows: the provider build/load chain) ----

    #[cfg(windows)]
    fn run_foreign(&self, request: &AceRequest<'_>, name: &str) -> AceRunReport {
        let Some(entry) = lookup_provider(name) else {
            return host_report(
                "ace.provider/unknown",
                format!("provider `{name}` is not registered with this host"),
            );
        };
        if entry.revoked.load(Ordering::SeqCst) {
            // Pre-dispatch refusal: nothing is compiled, loaded or called.
            return denied_report(
                "ace.provider/revoked",
                format!("provider `{name}` admission withdrawn by the host"),
            );
        }
        // Explicit capability discipline (freeze): the E0 foreign lane
        // serves exactly the frozen F0 capability set; a request that
        // names any OTHER capability is refused before admission instead
        // of silently dropping it.
        for capability in request.capabilities {
            if capability != ace_foreign_contract::manual::SOURCE_CAP
                && capability != ace_foreign_contract::resource::RESOURCE_CAP
            {
                return denied_report(
                    "ace.capability/unsupported",
                    format!(
                        "capability `{capability}` is outside the frozen E0 foreign lane {{foreign.scalar, foreign.resource}}"
                    ),
                );
            }
        }
        for grant in request.grants {
            if !entry.granted_caps.iter().any(|cap| cap == grant) {
                return denied_report(
                    "ace.grant/unadmitted",
                    format!("grant `{grant}` is not admitted by provider `{name}`"),
                );
            }
        }
        // Per-run identity + admission re-check (fresh authority every run).
        let generation = 1;
        let authority = ace_abi::rust_provider::f0_authority(&entry.build, generation);
        let admitted = match entry.build.manual.clone().admit(&authority) {
            Ok(admitted) => admitted,
            Err(e) => {
                return denied_report(
                    "ace.admission/refused",
                    format!("per-run manifest admission failed: {e}"),
                )
            }
        };
        // Per-run provider load: an OWN table gives this request isolated
        // counters and a session that can be torn down atomically.
        let mut loaded = match ace_abi::rust_provider::load(&entry.build, generation) {
            Ok(loaded) => loaded,
            Err(e) => {
                return host_report(
                    "ace.provider/load-failed",
                    format!("provider `{name}` failed to load: {e}"),
                )
            }
        };
        // Compile the source against the admitted environment.
        let mut query = ace_source_query::SourceQuerySession::new("p969", "m969");
        query.set_foreign(Some(admitted));
        let caps = [
            ace_foreign_contract::manual::SOURCE_CAP.to_string(),
            ace_foreign_contract::resource::RESOURCE_CAP.to_string(),
        ];
        let product = match ace_source_query::consumer::compile_text_record_product(
            request.source,
            "p969.at",
            &caps,
            "p969",
            "m969",
            &mut query,
        ) {
            Ok(product) => product,
            Err(diags) => return diag_list_report(&diags),
        };
        let mir = match lower_and_check_entry(
            &product.compiled.checked,
            request.entry_def_id,
            request.source,
            true,
        ) {
            Ok(mir) => mir,
            Err(report) => return report,
        };
        let loaded_module = match wire_admit(&mir, Some(&mut loaded.table)) {
            Ok(loaded_module) => loaded_module,
            Err(report) => return report,
        };
        // Session spine: adopt → execute (per-run authority re-check) →
        // teardown (late callees refuse before any dispatch).
        let mut session = ace_av_core::foreign_exec::ForeignSession::new();
        // Same extraction seam as `open_session_from_build`: the table
        // moves into the session, a fresh empty table stays with the
        // provider (which keeps the cdylib alive until scope end).
        session.adopt(std::mem::replace(
            &mut loaded.table,
            ace_av_core::foreign_exec::ForeignStubTable::new(generation),
        ));
        let fallback = CancelFlag::new(false);
        let result = session.execute(
            &loaded_module,
            &limits_from(request),
            cancel_flag(request, &fallback),
        );
        let callee = session.table_shared().callee_count();
        let rejected = session.table_shared().rejected_count();
        session.teardown();
        match result {
            Ok((outcome, _trace)) => map_outcome(outcome, Some(callee), Some(rejected)),
            Err(e) => report(
                AceOutcome::Denied {
                    code: e.code.to_string(),
                    message: e.message.clone(),
                },
                0,
                Some(callee),
                Some(rejected),
            ),
        }
    }
}
