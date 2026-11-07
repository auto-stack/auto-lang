//! Explicit CRT build inputs for the source certification host. The default
//! uses the canonical SDK locator and the same real MSVC compiler/CRT flags;
//! an explicit vcvars override keeps its original batch-script semantics.
//! Nothing is cached, and this module is never used by pure target emission.
use crate::{
    bridge::{BridgeError, BRIDGE_BUILD_DEADLINE, BRIDGE_CRT_FLAGS, BRIDGE_STREAM_CAP},
    ctool, ctool_capture,
};
use ace_core::{
    atom_text::{Diagnostic, Span, Stage},
    link::CaptureCaps,
};
use ace_source_query::resources::{Charge, Reservation, Resources};
use std::{
    path::{Path, PathBuf},
    process::Command,
};

#[derive(Clone)]
enum Mode {
    Sdk {
        include: [PathBuf; 4],
        libs: [PathBuf; 3],
        bin: PathBuf,
    },
    Batch(PathBuf),
}

/// PLAN-938 T-11 (user ruling 2026-11-07): the SDK/MSVC closure resolution
/// (cl discovery + content hash + platform query + closure layout) is
/// PROCESS-LEVEL, keyed by the explicit override inputs — every request
/// re-charges its own metadata reservation but never re-runs the resolver.
/// Test-visible counters distinguish a cached reuse from a re-resolution.
static RESOLUTIONS: std::sync::Mutex<Vec<ResolutionKey>> = std::sync::Mutex::new(Vec::new());
static RESOLUTIONS_DONE: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
static SDK_DIRECT_COMPILES: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
#[derive(Clone)]
struct ResolutionKey {
    vcvars_override: Option<String>,
    sdk_um_override: Option<String>,
    resolved: std::sync::Arc<ResolvedCrt>,
}
/// The pure resolution result shared across requests.
struct ResolvedCrt {
    tool: &'static ctool::CToolchain,
    mode: Mode,
    material: String,
    /// The index bytes the original resolution charged for the model.
    metadata_index_bytes: usize,
}
/// Test observation: completed resolutions (platform query + hashing runs).
#[cfg(test)]
pub(crate) fn resolution_runs() -> u64 {
    RESOLUTIONS_DONE.load(std::sync::atomic::Ordering::Relaxed)
}
/// Test observation: cl invocations that ran through the direct SDK closure.
#[cfg(test)]
pub(crate) fn sdk_direct_compiles() -> u64 {
    SDK_DIRECT_COMPILES.load(std::sync::atomic::Ordering::Relaxed)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::ffi::OsString;
    #[test]
    fn crt_output_batch_counts_objects_and_pe_but_not_input_objects() {
        let _env = ENV_LOCK
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        use ace_source_query::resources::OutputLimits;
        let root = PathBuf::from("D:/autostack/plan-artifacts/plan-938/tests")
            .join(format!("crt-output-{}", std::process::id()));
        std::fs::create_dir_all(&root).unwrap();
        std::fs::write(
            root.join("provider.c"),
            "int provider(void) { return 42; }\n",
        )
        .unwrap();
        let provider = root.join("provider.obj");
        ctool::compile_c(&root.join("provider.c"), &root, &provider).unwrap();
        std::fs::write(
            root.join("value.c"),
            "int provider(void); int main(void) { return provider(); }\n",
        )
        .unwrap();
        std::fs::write(root.join("unused.c"), "int unused(void) { return 7; }\n").unwrap();
        let exe = root.join("value.exe");
        let resources = Resources::default();
        let initial = resources.stats().current;
        let crt = Toolchain::resolve(&resources).unwrap();
        let baseline = resources.stats().current;
        let sources = ["value.c", "unused.c"];
        crt.compile(&root, &sources, std::slice::from_ref(&provider), &exe)
            .unwrap();
        let sizes = [root.join("value.obj"), root.join("unused.obj"), exe.clone()]
            .map(|path| std::fs::metadata(path).unwrap().len());
        let size: u64 = sizes.iter().sum();
        resources
            .enable_output_budget(OutputLimits {
                artifact: 4 << 20,
                request: 8 << 20,
            })
            .unwrap();
        let provider_bytes = std::fs::read(&provider).unwrap();
        let mut previous_spend = 0;
        for count in 1..=2 {
            crt.compile(&root, &sources, std::slice::from_ref(&provider), &exe)
                .unwrap();
            let spent = resources.output_stats().unwrap().produced;
            // The linker can grow then truncate its PE. Its observed peak is
            // still produced bytes, rather than a refund on final truncation.
            assert!(
                spent - previous_spend >= size,
                "a new compiler output was omitted"
            );
            eprintln!(
                "batch {count}: terminal={size} observed={}",
                spent - previous_spend
            );
            previous_spend = spent;
            assert_eq!(std::fs::read(&provider).unwrap(), provider_bytes);
            assert_eq!(resources.output_stats().unwrap().pending, 0);
            assert_eq!(resources.stats().current, baseline);
            assert_eq!(
                ace_native::link_driver::run_pe(
                    &exe,
                    std::time::Duration::from_secs(5),
                    CaptureCaps {
                        stdout_max: 1024,
                        stderr_max: 1024
                    }
                )
                .unwrap()
                .exit_code,
                Some(42)
            );
        }
        let remaining = (8 << 20) - previous_spend;
        // Spend the rest through the same account, without another tool run.
        for bytes in [remaining.min(4 << 20), remaining.saturating_sub(4 << 20)] {
            resources
                .reserve_output(bytes as usize)
                .unwrap()
                .complete(bytes as usize)
                .unwrap();
        }
        let previous = std::fs::read(&exe).unwrap();
        assert!(crt
            .compile(&root, &sources, std::slice::from_ref(&provider), &exe)
            .is_err());
        assert_eq!(std::fs::read(&exe).unwrap(), previous);
        assert_eq!(resources.output_stats().unwrap().produced, 8 << 20);
        drop(crt);
        assert_eq!(resources.stats().current, initial);
    }
    /// The configuration test mutates process environment variables; the
    /// reuse/legacy tests read them through the resolver. Serialize the
    /// three (no behavior change under nextest's process-per-test).
    static ENV_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());

    #[test]
    fn direct_sdk_freezes_flags_and_an_explicit_batch_override_really_runs() {
        let _env = ENV_LOCK
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        struct Environment(Vec<(&'static str, Option<OsString>)>);
        impl Drop for Environment {
            fn drop(&mut self) {
                for (name, value) in &self.0 {
                    match value {
                        Some(value) => std::env::set_var(name, value),
                        None => std::env::remove_var(name),
                    }
                }
            }
        }
        // nextest runs this one case in a fresh process; no library caller's
        // environment is changed by the production compiler configuration.
        let saved = Environment(
            [
                "A2C_VCVARS64",
                "AC741_SDK_UM_LIB",
                "CL",
                "_CL_",
                "LINK",
                "_LINK_",
                "SOURCE938_REAL_VCVARS",
            ]
            .into_iter()
            .map(|name| (name, std::env::var_os(name)))
            .collect(),
        );
        let tool = ctool::find_cl().unwrap();
        let root = PathBuf::from("D:/autostack/plan-artifacts/plan-938/tests")
            .join(format!("crt-config-{}", std::process::id()))
            .join("CRT 工具 with spaces");
        std::fs::create_dir_all(&root).unwrap();
        std::fs::write(
            root.join("value.c"),
            "#ifndef VALUE\n#define VALUE 42\n#endif\nint main(void) { return VALUE; }\n",
        )
        .unwrap();
        std::env::remove_var("A2C_VCVARS64");
        std::env::set_var("CL", "/DVALUE=99");
        std::env::set_var("_CL_", "/W0");
        std::env::set_var("LINK", "/MACHINE:X86");
        let resources = Resources::default();
        let baseline = resources.stats().current;
        let crt = Toolchain::resolve(&resources).unwrap();
        let sdk_um = match &crt.mode {
            Mode::Sdk { libs, .. } => libs[2].clone(),
            _ => unreachable!(),
        };
        assert!(crt.fingerprint().contains("|crt:explicit-sdk/1|"));
        assert!(crt.fingerprint().contains("|crt-link-hash:"));
        let exe = root.join("default value.exe");
        crt.compile(&root, &["value.c"], &[], &exe).unwrap();
        let execute = |exe: &Path| {
            ace_native::link_driver::run_pe(
                exe,
                std::time::Duration::from_secs(5),
                CaptureCaps {
                    stdout_max: BRIDGE_STREAM_CAP,
                    stderr_max: BRIDGE_STREAM_CAP,
                },
            )
            .unwrap()
            .exit_code
        };
        assert_eq!(
            execute(&exe),
            Some(42),
            "ambient compiler flags changed the frozen source profile"
        );
        drop(crt);
        assert_eq!(resources.stats().current, baseline);
        std::env::set_var("AC741_SDK_UM_LIB", root.join("missing SDK override"));
        let error = Toolchain::resolve(&resources)
            .err()
            .expect("invalid explicit SDK silently fell back to another install");
        assert!(
            error.0.contains("AC741_SDK_UM_LIB has no kernel32.lib"),
            "{error}"
        );
        assert_eq!(resources.stats().current, baseline);
        std::env::set_var("AC741_SDK_UM_LIB", &sdk_um);
        let explicit_sdk = Toolchain::resolve(&resources).unwrap();
        assert!(explicit_sdk
            .fingerprint()
            .contains(&format!("|crt-sdk:{}|", sdk_um.display())));
        drop(explicit_sdk);
        let refused = ace_core::link::find_sdk_um_dir_with_query(None, |_| {
            Err(Diagnostic::new(
                Stage::Link,
                "foreign.resource",
                "query quota refusal",
                Span { start: 0, end: 0 },
            ))
        });
        assert_eq!(
            refused.unwrap_err().code,
            "foreign.resource",
            "a quota refusal became known-root fallback"
        );
        for name in ["CL", "_CL_", "LINK", "_LINK_"] {
            std::env::remove_var(name);
        }
        let vcvars = tool
            .cl_exe
            .ancestors()
            .nth(7)
            .unwrap()
            .join("Auxiliary/Build/vcvars64.bat");
        let script = root.join("explicit setup.bat");
        let marker = root.join("override executed.txt");
        std::env::set_var("SOURCE938_REAL_VCVARS", &vcvars);
        // Keep batch source ASCII even when its directory/tool path is not:
        // cmd's script code page must not change the Unicode path oracle.
        let body = "@echo off\r\ncall \"%SOURCE938_REAL_VCVARS%\" >nul\r\nif errorlevel 1 exit /b 1\r\necho executed>\"override executed.txt\"\r\nset CL=/DVALUE=43\r\n";
        std::fs::write(&script, &body).unwrap();
        std::env::set_var("A2C_VCVARS64", &script);
        let crt = Toolchain::resolve(&resources).unwrap();
        let fingerprint = crt.fingerprint();
        assert!(fingerprint.contains("|crt:vcvars/1|"));
        assert!(fingerprint.contains(&format!(
            "|crt-vcvars-hash:{}",
            blake3::hash(body.as_bytes()).to_hex()
        )));
        let exe = root.join("override value.exe");
        crt.compile(&root, &["value.c"], &[], &exe).unwrap();
        assert_eq!(
            execute(&exe),
            Some(43),
            "explicit trusted setup script was ignored"
        );
        assert!(marker.is_file());
        drop(crt);
        assert_eq!(resources.stats().current, baseline);
        drop(saved);
    }

    /// PLAN-938 T-11 (user ruling): the SDK/MSVC closure is resolved once
    /// per process (per override key); a second request reuses it without a
    /// new platform query or cl hash, yet still charges and releases its own
    /// metadata reservation, and both requests carry the same identity.
    #[test]
    fn crt_resolution_is_reused_across_requests_and_recharges_each_request() {
        let _env = ENV_LOCK
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let first = Resources::default();
        let first_baseline = first.stats().current;
        let runs_before = resolution_runs();
        let crt1 = Toolchain::resolve(&first).unwrap();
        let runs_after_first = resolution_runs();
        assert!(
            runs_after_first == runs_before || runs_after_first == runs_before + 1,
            "the first request may resolve at most once"
        );
        let first_charge = first.stats().current.index - first_baseline.index;
        assert!(first_charge > 0, "a request must charge its metadata model");
        let second = Resources::default();
        let second_baseline = second.stats().current;
        let crt2 = Toolchain::resolve(&second).unwrap();
        assert_eq!(
            resolution_runs(),
            runs_after_first,
            "a second request re-ran the closure resolution"
        );
        assert_eq!(crt1.fingerprint(), crt2.fingerprint());
        assert_eq!(
            second.stats().current.index - second_baseline.index,
            first_charge,
            "a reused resolution must still charge the request's own metadata"
        );
        drop(crt1);
        drop(crt2);
        assert_eq!(first.stats().current, first_baseline);
        assert_eq!(second.stats().current, second_baseline);
    }

    /// The legacy bridge leg (no shared account) compiles through the SAME
    /// resolved direct closure: one direct cl invocation, no per-compile
    /// setup batch, and the frozen flags still produce the program.
    #[test]
    fn legacy_bridge_leg_compiles_through_the_resolved_direct_closure() {
        let _env = ENV_LOCK
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        if std::env::var_os("A2C_VCVARS64").is_some_and(|v| !v.is_empty()) {
            eprintln!("explicit batch override present; direct-closure leg not asserted here");
            return;
        }
        let root = PathBuf::from("D:/autostack/plan-artifacts/plan-938/tests")
            .join(format!("legacy-bridge-direct-{}", std::process::id()));
        std::fs::create_dir_all(&root).unwrap();
        std::fs::write(
            root.join("value.c"),
            "int main(void) { return 42; }
",
        )
        .unwrap();
        let exe = root.join("legacy.exe");
        let direct_before = sdk_direct_compiles();
        crate::bridge::compile_bridge_crt(&root, &["value.c"], &exe).unwrap();
        assert_eq!(
            sdk_direct_compiles(),
            direct_before + 1,
            "the legacy bridge leg did not use the resolved direct closure"
        );
        let run = ace_native::link_driver::run_pe(
            &exe,
            std::time::Duration::from_secs(5),
            CaptureCaps {
                stdout_max: BRIDGE_STREAM_CAP,
                stderr_max: BRIDGE_STREAM_CAP,
            },
        )
        .unwrap();
        assert_eq!(run.exit_code, Some(42));
    }
}
pub(crate) struct Toolchain {
    tool: &'static ctool::CToolchain,
    mode: Mode,
    identity: String,
    resources: Resources,
    // Owned compiler-context metadata drops before its reservation.
    _allocation: Reservation,
}
fn failure(error: ctool_capture::CaptureError) -> BridgeError {
    match error {
        ctool_capture::CaptureError::Resource(message) => {
            BridgeError(format!("foreign.resource: CRT capture: {message}"))
        }
        ctool_capture::CaptureError::Subprocess(
            error @ ace_core::link::SubprocError::OutputCapExceeded { .. },
        ) => BridgeError(format!("foreign.resource: CRT stream cap: {error:?}")),
        ctool_capture::CaptureError::Subprocess(error) => {
            BridgeError(format!("contained CRT tool: {error:?}"))
        }
    }
}
fn diag(error: BridgeError) -> Diagnostic {
    Diagnostic::new(
        Stage::Link,
        if error.0.starts_with("foreign.resource:") {
            "foreign.resource"
        } else {
            "link.sdk-query"
        },
        error.0,
        Span { start: 0, end: 0 },
    )
}
impl Toolchain {
    pub(crate) fn resolve(resources: &Resources) -> Result<Self, BridgeError> {
        // PLAN-938 T-11: resolve the closure ONCE per (override) key per
        // process; each request still charges its own metadata reservation
        // from the stored sizes — never a second platform query or cl hash.
        const VCVARS: &[u16] = &[65, 50, 67, 95, 86, 67, 86, 65, 82, 83, 54, 52, 0];
        const SDK_UM: &[u16] = &[
            65, 67, 55, 52, 49, 95, 83, 68, 75, 95, 85, 77, 95, 76, 73, 66, 0,
        ];
        let mut overrides = 0usize;
        let mut override_key = (None::<String>, None::<String>);
        let mut override_tickets = Vec::new();
        for (needle, slot) in [(VCVARS, &mut override_key.0), (SDK_UM, &mut override_key.1)] {
            if let Some(environment) =
                crate::ctool_environment::read(needle, resources).map_err(BridgeError)?
            {
                let crate::ctool_environment::Value { value, allocation } = environment;
                let text = value.to_string_lossy().into_owned();
                overrides = overrides.saturating_add(text.len() * 16);
                if slot.is_none() && !text.is_empty() {
                    *slot = Some(text);
                }
                override_tickets.push(allocation);
            }
        }
        let tool = ctool::find_cl_admitted(Some(resources)).map_err(|error| match error {
            ctool::CToolFailure::Tool(error) => BridgeError(error.to_string()),
            ctool::CToolFailure::Resource(message) => {
                BridgeError(format!("foreign.resource: {message}"))
            }
        })?;
        let initial = tool
            .cl_exe
            .as_os_str()
            .len()
            .checked_mul(128)
            .and_then(|n| n.checked_add(tool.version.len() * 2 + 4096))
            .ok_or_else(|| BridgeError("foreign.resource: CRT metadata size".into()))?;
        let mut allocation = resources
            .reserve(Charge::index(initial))
            .map_err(|e| BridgeError(e.to_string()))?;
        for ticket in override_tickets {
            allocation = allocation
                .merge(ticket)
                .map_err(|e| BridgeError(e.to_string()))?;
        }
        let resolved: std::sync::Arc<ResolvedCrt> = {
            let cache = RESOLUTIONS.lock().expect("CRT resolution cache");
            if let Some(hit) = cache.iter().find(|entry| {
                entry.vcvars_override == override_key.0
                    && entry.sdk_um_override == override_key.1
                    && entry.resolved.tool.cl_exe == tool.cl_exe
            }) {
                hit.resolved.clone()
            } else {
                drop(cache);
                let resolved = Self::resolve_uncached(
                    resources,
                    tool,
                    override_key.clone(),
                    &mut allocation,
                    initial,
                    overrides,
                )?;
                let resolved = std::sync::Arc::new(resolved);
                RESOLUTIONS_DONE.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                let mut cache = RESOLUTIONS.lock().expect("CRT resolution cache");
                if cache.len() >= 8 {
                    cache.remove(0);
                }
                cache.push(ResolutionKey {
                    vcvars_override: override_key.0,
                    sdk_um_override: override_key.1,
                    resolved: resolved.clone(),
                });
                resolved
            }
        };
        // Charge this request's metadata model at the stored size.
        allocation
            .resize(Charge::index(
                initial.saturating_add(resolved.metadata_index_bytes),
            ))
            .map_err(|e| BridgeError(e.to_string()))?;
        let identity = format!(
            "cl:{}|ver:{}|hash:{}|flags:{}|crt:{}|crt-flags:{}",
            resolved.tool.cl_exe.display(),
            resolved.tool.version,
            resolved.tool.cl_hash,
            ctool::CL_FLAGS.join(" "),
            resolved.material,
            BRIDGE_CRT_FLAGS.join(" ")
        );
        Ok(Self {
            tool: resolved.tool,
            mode: resolved.mode.clone(),
            identity,
            resources: resources.clone(),
            _allocation: allocation,
        })
    }
    fn resolve_uncached(
        resources: &Resources,
        tool: &'static ctool::CToolchain,
        override_key: (Option<String>, Option<String>),
        allocation: &mut Reservation,
        initial: usize,
        overrides: usize,
    ) -> Result<ResolvedCrt, BridgeError> {
        // The override inputs were read (and accounted) by the caller.
        let mut metadata = overrides;
        let mode = match override_key.0 {
            Some(path) => {
                allocation
                    .resize(Charge::index(initial + metadata))
                    .map_err(|e| BridgeError(e.to_string()))?;
                let path = PathBuf::from(path);
                if !path.is_file() {
                    return Err(BridgeError("explicit A2C_VCVARS64 is missing".into()));
                }
                Mode::Batch(path)
            }
            None => {
                let explicit = override_key.1.map(PathBuf::from);
                let um = ace_core::link::find_sdk_um_dir_with_query(explicit, |command| {
                    let captured = ctool_capture::run(
                        command,
                        BRIDGE_BUILD_DEADLINE,
                        CaptureCaps {
                            stdout_max: BRIDGE_STREAM_CAP,
                            stderr_max: BRIDGE_STREAM_CAP,
                        },
                        Some(resources),
                    )
                    .map_err(failure)
                    .map_err(diag)?;
                    // Reserve the platform query's owned text/path projection
                    // before the resolver copies it, retaining the model fee.
                    allocation
                        .resize(Charge::index(initial + captured.value.stdout.len() * 64))
                        .map_err(|e| diag(BridgeError(e.to_string())))?;
                    let text = crate::ctool_text::display(&captured.value.stdout, Some(resources))
                        .map_err(|e| diag(BridgeError(format!("foreign.resource: {e}"))))?;
                    Ok((captured.value.exit_code == Some(0)).then(|| text.value.to_string()))
                })
                .map_err(|e| {
                    BridgeError(if e.code == "foreign.resource" {
                        format!("foreign.resource: {}", e.message)
                    } else {
                        e.message
                    })
                })?;
                allocation
                    .resize(Charge::index(initial + um.as_os_str().len() * 128))
                    .map_err(|e| BridgeError(e.to_string()))?;
                let version = um
                    .ancestors()
                    .nth(2)
                    .ok_or_else(|| BridgeError("SDK version layout".into()))?;
                let kits = um
                    .ancestors()
                    .nth(4)
                    .ok_or_else(|| BridgeError("SDK root layout".into()))?;
                let name = version
                    .file_name()
                    .ok_or_else(|| BridgeError("SDK version name".into()))?;
                let vc = tool
                    .cl_exe
                    .ancestors()
                    .nth(4)
                    .ok_or_else(|| BridgeError("MSVC toolset layout".into()))?;
                let includes = kits.join("Include").join(name);
                let include = [
                    vc.join("include"),
                    includes.join("ucrt"),
                    includes.join("shared"),
                    includes.join("um"),
                ];
                let libs = [vc.join("lib/x64"), version.join("ucrt/x64"), um.clone()];
                let bin = kits.join("bin").join(name).join("x64");
                if !include.iter().all(|p| p.is_dir())
                    || !libs.iter().all(|p| p.is_dir())
                    || !tool.cl_exe.with_file_name("link.exe").is_file()
                {
                    return Err(BridgeError(
                        "selected SDK/MSVC CRT closure is incomplete".into(),
                    ));
                }
                Mode::Sdk { include, libs, bin }
            }
        };
        let material = match &mode {
            Mode::Sdk { include, libs, .. } => format!(
                "explicit-sdk/1|crt-sdk:{}|crt-msvc:{}|crt-link-hash:{}",
                libs[2].display(),
                include[0].display(),
                crate::semref::blake3_reader(
                    std::fs::File::open(tool.cl_exe.with_file_name("link.exe"))
                        .map_err(|e| BridgeError(e.to_string()))?
                )
                .map_err(|e| BridgeError(e.to_string()))?
            ),
            Mode::Batch(path) => format!(
                "vcvars/1|crt-vcvars:{}|crt-vcvars-hash:{}",
                path.display(),
                crate::semref::blake3_reader(
                    std::fs::File::open(path).map_err(|e| BridgeError(e.to_string()))?
                )
                .map_err(|e| BridgeError(e.to_string()))?
            ),
        };
        metadata = metadata.saturating_add(material.len() * 2);
        Ok(ResolvedCrt {
            tool,
            mode,
            material,
            metadata_index_bytes: metadata,
        })
    }
    pub(crate) fn fingerprint(&self) -> String {
        self.identity.clone()
    }
    pub(crate) fn compile(
        &self,
        dir: &Path,
        sources: &[&str],
        objects: &[PathBuf],
        exe: &Path,
    ) -> Result<(), BridgeError> {
        let units = dir.as_os_str().len()
            + exe.as_os_str().len()
            + sources.iter().map(|s| s.len()).sum::<usize>()
            + objects.iter().map(|p| p.as_os_str().len()).sum::<usize>();
        let _command = self
            .resources
            .reserve(Charge::index(
                units * 8 + (sources.len() + objects.len() + 32) * 256,
            ))
            .map_err(|e| BridgeError(e.to_string()))?;
        let mut command = match &self.mode {
            Mode::Sdk { include, libs, bin } => {
                SDK_DIRECT_COMPILES.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                let mut command = Command::new(&self.tool.cl_exe);
                command.env(
                    "INCLUDE",
                    std::env::join_paths(include).map_err(|e| BridgeError(e.to_string()))?,
                );
                command.env(
                    "LIB",
                    std::env::join_paths(libs).map_err(|e| BridgeError(e.to_string()))?,
                );
                command.env(
                    "PATH",
                    std::env::join_paths([self.tool.cl_exe.parent().unwrap(), bin.as_path()])
                        .map_err(|e| BridgeError(e.to_string()))?,
                );
                // These source profiles have frozen flags. Ambient compiler
                // option variables cannot silently append a different policy.
                for name in ["CL", "_CL_", "LINK", "_LINK_"] {
                    command.env_remove(name);
                }
                command
                    .args(BRIDGE_CRT_FLAGS)
                    .arg(format!("-Fe:{}", exe.display()))
                    .args(sources);
                if !objects.is_empty() {
                    command.arg("/link").args(objects);
                }
                command
            }
            Mode::Batch(path) => {
                let mut line = format!(
                    "call \"{}\" >nul && \"{}\" {} -Fe:\"{}\"",
                    path.display(),
                    self.tool.cl_exe.display(),
                    BRIDGE_CRT_FLAGS.join(" "),
                    exe.display()
                );
                for source in sources {
                    line.push_str(&format!(" \"{source}\""));
                }
                if !objects.is_empty() {
                    line.push_str(" /link");
                    for object in objects {
                        line.push_str(&format!(" \"{}\"", object.display()));
                    }
                }
                let mut command = Command::new("cmd.exe");
                command.args(["/D", "/C"]);
                use std::os::windows::process::CommandExt;
                command.raw_arg(line);
                command
            }
        };
        command.current_dir(dir);
        // CL emits one .obj per C source plus the PE. Objects supplied to the
        // linker already exist and must not be counted as new production.
        let path_units = sources
            .iter()
            .try_fold(exe.as_os_str().len(), |sum, source| {
                sum.checked_add(dir.as_os_str().len())?
                    .checked_add(source.len() + 8)
            })
            .ok_or_else(|| BridgeError("compiler output paths overflow".into()))?;
        let _paths = self
            .resources
            .reserve(Charge::index(
                path_units
                    .checked_mul(8)
                    .and_then(|n| {
                        n.checked_add((sources.len() + 1) * std::mem::size_of::<PathBuf>())
                    })
                    .ok_or_else(|| BridgeError("compiler output index overflow".into()))?,
            ))
            .map_err(|e| BridgeError(e.to_string()))?;
        let mut paths = Vec::new();
        paths
            .try_reserve_exact(sources.len() + 1)
            .map_err(|e| BridgeError(e.to_string()))?;
        for source in sources {
            let name = Path::new(source)
                .file_name()
                .ok_or_else(|| BridgeError("missing C source name".into()))?;
            paths.push(dir.join(name).with_extension("obj"));
        }
        paths.push(exe.to_path_buf());
        let mut outputs =
            crate::ctool_output::Outputs::new(&self.resources, paths.iter().map(|p| p.as_path()))
                .map_err(BridgeError)?;
        let captured = ctool_capture::run_observed(
            &mut command,
            BRIDGE_BUILD_DEADLINE,
            CaptureCaps {
                stdout_max: BRIDGE_STREAM_CAP,
                stderr_max: BRIDGE_STREAM_CAP,
            },
            Some(&self.resources),
            || outputs.observe(),
        );
        outputs.finish().map_err(BridgeError)?;
        let captured = captured.map_err(failure)?;
        if captured.value.exit_code != Some(0) {
            return Err(BridgeError(format!(
                "cl exited {:?}:\n{}\n{}",
                captured.value.exit_code,
                String::from_utf8_lossy(&captured.value.stdout),
                String::from_utf8_lossy(&captured.value.stderr)
            )));
        }
        if !exe.is_file() {
            return Err(BridgeError("cl reported success but exe missing".into()));
        }
        Ok(())
    }
}
