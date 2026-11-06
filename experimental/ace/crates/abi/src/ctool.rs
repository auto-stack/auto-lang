//! Independent C compiler toolchain for the interop legs (plan §3, T-01).
//!
//! Rules frozen here:
//! * The C side is compiled by a REAL, separately located MSVC `cl.exe`.
//!   A missing tool is a controlled INFRA error — the affected legs stop;
//!   there is no fallback to "Rust repr(C) is good enough" and nothing is
//!   auto-downloaded or installed.
//! * Discovery: `AUTOABI_MSVC_CL` env override -> vswhere (fixed installer
//!   path) -> highest `VC/Tools/MSVC/*/bin/Hostx64/x64/cl.exe`. The
//!   resolved identity (path + banner version + content hash) is part of
//!   the toolchain fingerprint that later feeds cache keys; it is pinned
//!   in contract-decision.md §5.
//! * Linking stays with the merged ac-core entries (rust-lld); cl.exe is
//!   only ever invoked with `/c` (compile to object).

use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::{Mutex, OnceLock};
use std::time::Duration;

const VSWHERE: &str = r"C:\Program Files (x86)\Microsoft Visual Studio\Installer\vswhere.exe";
const DISCOVERY_DEADLINE: Duration = Duration::from_secs(10);
pub const COMPILE_DEADLINE: Duration = Duration::from_secs(60);
pub(crate) const TOOL_STREAM_MAX: usize = 1 << 20;

/// Frozen flags for every C compile (freestanding objects: no CRT init,
/// no stack cookies, no default-lib directives, warnings as errors).
pub const CL_FLAGS: &[&str] = &["-c", "-nologo", "-GS-", "-Zl", "-W4", "-WX", "-O2"];

#[derive(Debug)]
pub struct CToolError(pub String);

impl std::fmt::Display for CToolError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "abi.ctool: {}", self.0)
    }
}

/// The resolved C toolchain identity.
#[derive(Clone, Debug)]
pub struct CToolchain {
    pub cl_exe: PathBuf,
    /// Version banner line from `cl -?` (e.g. "19.44.35222 for x64").
    pub version: String,
    /// blake3 content hash of the cl.exe binary.
    pub cl_hash: String,
}

/// Locate cl.exe per the frozen discovery order. Result cached for the
/// process; a missing tool is an error, never a skip.
pub fn find_cl() -> Result<&'static CToolchain, CToolError> {
    find_cl_admitted(None).map_err(|e| match e {
        CToolFailure::Tool(error) => error,
        CToolFailure::Resource(message) => CToolError(message),
    })
}

static CL_CACHE: OnceLock<Result<CToolchain, String>> = OnceLock::new();
static CL_INIT: Mutex<()> = Mutex::new(());

/// Resource denial belongs to this request, so it must not poison discovery
/// for a later repaired request. Ordinary missing-tool errors retain the
/// original process cache behavior. Initialization has a single writer.
pub(crate) fn find_cl_admitted(
    resources: Option<&ace_source_query::resources::Resources>,
) -> Result<&'static CToolchain, CToolFailure> {
    if CL_CACHE.get().is_none() {
        let _writer = CL_INIT.lock().expect("C tool discovery initializer");
        if CL_CACHE.get().is_none() {
            let result = match resolve_cl(resources) {
                Ok(tool) => Ok(tool),
                Err(CToolFailure::Tool(error)) => Err(error.0),
                Err(resource @ CToolFailure::Resource(_)) => return Err(resource),
            };
            let _ = CL_CACHE.set(result);
        }
    }
    CL_CACHE
        .get()
        .expect("discovery result installed")
        .as_ref()
        .map_err(|e| CToolFailure::Tool(CToolError(e.clone())))
}

fn resolve_cl(
    resources: Option<&ace_source_query::resources::Resources>,
) -> Result<CToolchain, CToolFailure> {
    let cl = if let Ok(env_path) = std::env::var("AUTOABI_MSVC_CL") {
        let p = PathBuf::from(env_path);
        if p.is_file() {
            p
        } else {
            return Err(CToolError(format!(
                "AUTOABI_MSVC_CL points at a missing file: {}",
                p.display()
            ))
            .into());
        }
    } else {
        locate_via_vswhere(resources)?
    };

    // Banner version: `cl -?` prints the version line and exits 0. Runs
    // under the contained executor like every other subprocess.
    let mut cmd = Command::new(&cl);
    cmd.arg("-?");
    let captured = run_deadlined(&mut cmd, resources, "cl -?")?;
    let code = captured.value.exit_code;
    let out = crate::ctool_text::display(&captured.value.stdout, resources)
        .map_err(CToolFailure::Resource)?;
    let err = crate::ctool_text::display(&captured.value.stderr, resources)
        .map_err(CToolFailure::Resource)?;
    if code != Some(0) {
        return Err(CToolError(format!(
            "cl -? exited {code:?}: {}{}",
            out.value.trim(),
            err.value.trim()
        ))
        .into());
    }
    // Banner version: `cl -?` prints the version banner on STDERR and
    // the option help on stdout. The banner is e.g.
    // "用于 x64 的 Microsoft (R) C/C++ 优化编译器 19.44.35222 版" — note
    // the trailing single "版", NOT "版本" — so the match requires a
    // compiler word plus a dotted version-number token; a help line can
    // never satisfy both.
    let banner = err
        .value
        .lines()
        .chain(out.value.lines())
        .find(|l| {
            let compiler_word =
                l.contains("编译器") || l.contains("Compiler") || l.contains("Version");
            let dotted = l.split_whitespace().any(|w| {
                w.chars().filter(|c| *c == '.').count() >= 2
                    && w.chars().next().is_some_and(|c| c.is_ascii_digit())
            });
            compiler_word && dotted
        })
        .unwrap_or("")
        .trim()
        .to_string();
    if banner.is_empty() {
        return Err(CToolError("cl banner version line not found".into()).into());
    }

    let image = std::fs::File::open(&cl).map_err(|e| CToolError(format!("read cl.exe: {e}")))?;
    let cl_hash =
        crate::semref::blake3_reader(image).map_err(|e| CToolError(format!("read cl.exe: {e}")))?;

    Ok(CToolchain {
        cl_exe: cl,
        version: banner,
        cl_hash,
    })
}

fn locate_via_vswhere(
    resources: Option<&ace_source_query::resources::Resources>,
) -> Result<PathBuf, CToolFailure> {
    if !Path::new(VSWHERE).is_file() {
        return Err(CToolError(format!(
            "vswhere missing at {VSWHERE} and AUTOABI_MSVC_CL unset — \
             C toolchain unavailable (infra blocker, plan §3)"
        ))
        .into());
    }
    let mut cmd = Command::new(VSWHERE);
    cmd.args([
        "-latest",
        "-products",
        "*",
        "-requires",
        "Microsoft.VisualStudio.Component.VC.Tools.x86.x64",
        "-property",
        "installationPath",
    ]);
    let captured = run_deadlined(&mut cmd, resources, "vswhere")?;
    let code = captured.value.exit_code;
    let out = crate::ctool_text::display(&captured.value.stdout, resources)
        .map_err(CToolFailure::Resource)?;
    let err = crate::ctool_text::display(&captured.value.stderr, resources)
        .map_err(CToolFailure::Resource)?;
    if code != Some(0) {
        return Err(CToolError(format!(
            "vswhere exited {code:?}: {}{}",
            out.value.trim(),
            err.value.trim()
        ))
        .into());
    }
    let inst = PathBuf::from(out.value.trim());
    if inst.as_os_str().is_empty() {
        return Err(CToolError(
            "vswhere returned no VC.Tools installation (infra blocker, plan §3)".into(),
        )
        .into());
    }
    let tools = inst.join("VC/Tools/MSVC");
    let mut versions: Vec<PathBuf> = std::fs::read_dir(&tools)
        .map_err(|e| CToolError(format!("read {}: {e}", tools.display())))?
        .flatten()
        .map(|e| e.path())
        .filter(|p| p.join("bin/Hostx64/x64/cl.exe").is_file())
        .collect();
    versions.sort(); // version-named dirs: lexical == numeric for MSVC
    versions
        .pop()
        .map(|v| v.join("bin/Hostx64/x64/cl.exe"))
        .ok_or_else(|| {
            CToolError(format!(
                "no VC/Tools/MSVC/*/bin/Hostx64/x64/cl.exe under {} (infra blocker)",
                inst.display()
            ))
            .into()
        })
}

fn run_deadlined(
    cmd: &mut Command,
    resources: Option<&ace_source_query::resources::Resources>,
    tool: &str,
) -> Result<crate::ctool_capture::Captured, CToolFailure> {
    // Discovery output is part of the same tool surface as compilation.
    // A valid banner cannot authorize an over-cap help/diagnostic stream.
    crate::ctool_capture::run(
        cmd,
        DISCOVERY_DEADLINE,
        ace_core::link::CaptureCaps {
            stdout_max: TOOL_STREAM_MAX,
            stderr_max: TOOL_STREAM_MAX,
        },
        resources,
    )
    .map_err(|error| match error {
        crate::ctool_capture::CaptureError::Resource(message) => CToolFailure::Resource(message),
        crate::ctool_capture::CaptureError::Subprocess(
            error @ ace_core::link::SubprocError::OutputCapExceeded { .. },
        ) => CToolFailure::Resource(format!("{tool} spawn: {error:?}")),
        crate::ctool_capture::CaptureError::Subprocess(error) => {
            CToolFailure::Tool(CToolError(format!("{tool} spawn: {error:?}")))
        }
    })
}

/// Compile one C source into `obj_out` with the frozen flags. `include_dir`
/// is the probe's `cside/` directory; the resolved cl.exe's own MSVC
/// include dir is added automatically so freestanding headers (stdint.h)
/// resolve without a vcvars environment. The wider SDK/ucrt header
/// closure is NOT implied — T-01/T-02 C sources include only probe-owned
/// headers plus freestanding fixed-width types; header closure enters
/// the cache keys with the T-05 recipe.
pub fn compile_c(src: &Path, include_dir: &Path, obj_out: &Path) -> Result<(), CToolError> {
    compile_c_with_deps(src, include_dir, obj_out).map(|_| ())
}

/// F-797-R4-07 (R8): compile and OBSERVE the real dependency closure.
/// The compile runs with `-showIncludes` (a reporting-only flag: it does
/// not affect codegen, so it stays OUT of the frozen CL_FLAGS
/// fingerprint); every file the preprocessor actually read — probe-local
/// AND system/MSVC headers — is returned as an absolute path. This is
/// the dependency truth the cache consumes: text scanning cannot see
/// comment-hidden includes, line splices, or macro includes, but the
/// compiler's own preprocessor can.
pub fn compile_c_with_deps(
    src: &Path,
    include_dir: &Path,
    obj_out: &Path,
) -> Result<Vec<PathBuf>, CToolError> {
    compile_c_with_deps_admitted(src, include_dir, obj_out, None, |_| Ok(())).map_err(|e| match e {
        CToolFailure::Tool(error) => error,
        CToolFailure::Resource(message) => CToolError(message),
    })
}

#[derive(Debug)]
pub(crate) enum CToolFailure {
    Tool(CToolError),
    Resource(String),
}
impl From<CToolError> for CToolFailure {
    fn from(error: CToolError) -> Self {
        Self::Tool(error)
    }
}

/// Source callers retain their admission ticket until the cache has consumed
/// every returned path. The original public entry uses the same walk.
pub(crate) fn compile_c_with_deps_admitted(
    src: &Path,
    include_dir: &Path,
    obj_out: &Path,
    resources: Option<&ace_source_query::resources::Resources>,
    admit: impl FnMut(usize) -> Result<(), String>,
) -> Result<Vec<PathBuf>, CToolFailure> {
    let captured = compile_c_output(src, include_dir, obj_out, resources)?;
    let out = crate::ctool_text::display(&captured.value.stdout, resources)
        .map_err(CToolFailure::Resource)?;
    crate::ctool_dependencies::parse(&out.value, src, admit).map_err(CToolFailure::Resource)
}

fn compile_c_output(
    src: &Path,
    include_dir: &Path,
    obj_out: &Path,
    resources: Option<&ace_source_query::resources::Resources>,
) -> Result<crate::ctool_capture::Captured, CToolFailure> {
    let tool = find_cl_admitted(resources)?;
    // cl.exe = <msvc>/bin/Hostx64/x64/cl.exe -> <msvc>/include
    let msvc_root = tool
        .cl_exe
        .ancestors()
        .nth(4)
        .ok_or_else(|| CToolError("cannot derive MSVC root from cl path".into()))?;
    let msvc_include = msvc_root.join("include");
    if !msvc_include.join("stdint.h").is_file() {
        return Err(CToolError(format!(
            "MSVC include dir missing stdint.h: {}",
            msvc_include.display()
        ))
        .into());
    }
    let mut cmd = Command::new(&tool.cl_exe);
    cmd.args(CL_FLAGS);
    cmd.arg("-showIncludes");
    cmd.arg(format!("-I{}", include_dir.display()));
    cmd.arg(format!("-I{}", msvc_include.display()));
    cmd.arg(src);
    cmd.arg(format!("-Fo:{}", obj_out.display()));
    // F-797-R4-10 (R5): compile through the merged RAW CAPPED executor
    // — a cap violation terminates the tree (no truncated-green), raw
    // bytes for display.
    let captured = crate::ctool_capture::run(
        &mut cmd,
        COMPILE_DEADLINE,
        ace_core::link::CaptureCaps {
            stdout_max: TOOL_STREAM_MAX,
            stderr_max: TOOL_STREAM_MAX,
        },
        resources,
    )
    .map_err(|e| match e {
        crate::ctool_capture::CaptureError::Resource(message) => CToolFailure::Resource(message),
        crate::ctool_capture::CaptureError::Subprocess(
            e @ ace_core::link::SubprocError::OutputCapExceeded { .. },
        ) => CToolFailure::Resource(format!("cl spawn: {e:?}")),
        crate::ctool_capture::CaptureError::Subprocess(other) => {
            CToolFailure::Tool(CToolError(format!("cl spawn: {other:?}")))
        }
    })?;
    let run = &captured.value;
    let code = run.exit_code;
    if code != Some(0) {
        let out =
            crate::ctool_text::display(&run.stdout, resources).map_err(CToolFailure::Resource)?;
        let err =
            crate::ctool_text::display(&run.stderr, resources).map_err(CToolFailure::Resource)?;
        return Err(CToolError(format!(
            "cl exited {code:?} for {}:\n{}\n{}",
            src.display(),
            out.value.trim(),
            err.value.trim()
        ))
        .into());
    }
    if !obj_out.is_file() {
        return Err(CToolError(format!(
            "cl reported success but object missing: {}",
            obj_out.display()
        ))
        .into());
    }
    Ok(captured)
}

/// Stable toolchain fingerprint for cache keys and receipts (cl identity
/// + frozen flags; the linker/SDK side comes from the ac-core entries and
/// is pinned in contract-decision.md §5).
pub fn toolchain_fingerprint() -> Result<String, CToolError> {
    let t = find_cl()?;
    Ok(format!(
        "cl:{}|ver:{}|hash:{}|flags:{}",
        t.cl_exe.display(),
        t.version,
        t.cl_hash,
        CL_FLAGS.join(" ")
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ctool_dependencies::extract_drive_path;

    #[test]
    fn admitted_discovery_resource_failure_is_not_cached_or_hidden_as_a_cache_miss() {
        const CHILD: &str = "SOURCE938_DISCOVERY_BUDGET_CHILD";
        const CASE: &str = "ctool::tests::admitted_discovery_resource_failure_is_not_cached_or_hidden_as_a_cache_miss";
        if std::env::var_os(CHILD).is_some() {
            use ace_source_query::resources::{Charge, Resources};
            assert!(CL_CACHE.get().is_none(), "discovery was already warm");
            let resources = Resources::default();
            let pressure = resources.reserve(Charge::io((6 << 20) + 1)).unwrap();
            let baseline = resources.stats().current;
            assert!(matches!(
                find_cl_admitted(Some(&resources)),
                Err(CToolFailure::Resource(_))
            ));
            assert!(
                CL_CACHE.get().is_none(),
                "resource failure poisoned the cache"
            );
            assert_eq!(resources.stats().current, baseline);
            let stamp = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos();
            let root = PathBuf::from("D:/autostack/plan-artifacts/plan-938/tests")
                .join(format!("discovery-budget-{}-{stamp}", std::process::id()));
            std::fs::create_dir_all(&root).unwrap();
            let source = root.join("provider.c");
            let object = root.join("provider.obj");
            std::fs::write(&source, "int provider(void) { return 42; }\n").unwrap();
            let store =
                ace_artifacts::store::ManagedStore::open(&root.with_extension("store")).unwrap();
            let denied = crate::cache::produce_c_object_with_resources(
                &store,
                &source,
                &root,
                &[object],
                &resources,
                || panic!("discovery resource error became a cache miss and invoked the producer"),
            );
            assert!(matches!(
                denied,
                Err(ace_artifacts::store::StoreError::ResourceExhausted(_))
            ));
            assert!(CL_CACHE.get().is_none());
            assert_eq!(resources.stats().current, baseline);
            drop(pressure);
            assert_eq!(resources.stats().current.io, 0);
            let repaired_resources = Resources::default();
            let previous_override = std::env::var_os("AUTOABI_MSVC_CL");
            let fake = root.join("over-cap-cl.cmd");
            for stream in ["stdout", "stderr"] {
                let writer = if stream == "stdout" { "Out" } else { "Error" };
                std::fs::write(&fake, format!(
                    "@echo off\r\npowershell.exe -NoLogo -NoProfile -NonInteractive -Command \"$w=[Console]::{writer}; $w.WriteLine('Microsoft C Compiler Version 19.44.35222'); $w.Write(('x'*2097152))\"\r\n"
                )).unwrap();
                // Only this fresh, single-test child changes its environment.
                std::env::set_var("AUTOABI_MSVC_CL", &fake);
                let rejected = find_cl_admitted(Some(&repaired_resources));
                assert!(
                    matches!(rejected, Err(CToolFailure::Resource(ref message)) if message.contains("OutputCapExceeded") && message.contains(stream)),
                    "wrong discovery cap classification: {rejected:?}"
                );
                assert!(CL_CACHE.get().is_none(), "over-cap identity was cached");
                assert_eq!(repaired_resources.stats().current.io, 0);
            }
            if let Some(previous) = previous_override {
                std::env::set_var("AUTOABI_MSVC_CL", previous);
            } else {
                std::env::remove_var("AUTOABI_MSVC_CL");
            }
            let tool = find_cl_admitted(Some(&repaired_resources)).unwrap();
            assert!(tool.cl_exe.is_file());
            assert!(!tool.version.is_empty());
            assert_eq!(tool.cl_hash.len(), 64);
            assert_eq!(repaired_resources.stats().current.io, 0);
            assert_eq!(repaired_resources.stats().peak.io, 2 << 20);
            assert!(std::ptr::eq(
                tool,
                find_cl_admitted(Some(&resources)).unwrap()
            ));
            return;
        }
        // A fresh exact-test process makes the OnceLock observation valid
        // even under the ordinary parallel libtest runner.
        let mut child = Command::new(std::env::current_exe().unwrap());
        child.args(["--exact", CASE, "--nocapture"]).env(CHILD, "1");
        let result = ace_core::link::run_deadlined_captured(
            &mut child,
            Duration::from_secs(30),
            ace_core::link::CaptureCaps {
                stdout_max: 1 << 20,
                stderr_max: 1 << 20,
            },
        )
        .unwrap();
        assert_eq!(
            result.exit_code,
            Some(0),
            "fresh discovery child failed:\n{}\n{}",
            String::from_utf8_lossy(&result.stdout),
            String::from_utf8_lossy(&result.stderr)
        );
    }

    #[test]
    fn real_compile_dependency_admission_keeps_resource_kind_and_allows_repair() {
        let stamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let root = PathBuf::from("D:/autostack/plan-artifacts/plan-938/tests").join(format!(
            "real-dependency-admission-{}-{stamp}",
            std::process::id()
        ));
        std::fs::create_dir_all(&root).unwrap();
        let source = root.join("dependency.c");
        let object = root.join("dependency.obj");
        std::fs::write(
            &source,
            "#include <stdint.h>\nint dependency_probe(void) { return 42; }\n",
        )
        .unwrap();
        let mut attempts = 0;
        let resources = ace_source_query::resources::Resources::default();
        let denied =
            compile_c_with_deps_admitted(&source, &root, &object, Some(&resources), |_| {
                attempts += 1;
                Err("fixture dependency quota".into())
            });
        assert!(
            matches!(denied, Err(CToolFailure::Resource(ref message)) if message == "fixture dependency quota")
        );
        assert_eq!(attempts, 1);
        assert_eq!(resources.stats().current.io, 0);
        assert!(
            object.is_file(),
            "compiler did not really produce before dependency admission"
        );
        let repaired =
            compile_c_with_deps_admitted(&source, &root, &object, Some(&resources), |_| Ok(()))
                .unwrap();
        assert_eq!(resources.stats().current.io, 0);
        assert!(repaired
            .iter()
            .any(|p| p.file_name().is_some_and(|n| n == "stdint.h")));
        assert!(!repaired.contains(&source));
        std::fs::write(
            &source,
            "int dependency_probe(void) { return missing_output_text_control; }\n",
        )
        .unwrap();
        let original_error = compile_c_with_deps(&source, &root, &object).unwrap_err();
        let accounted_error =
            compile_c_with_deps_admitted(&source, &root, &object, Some(&resources), |_| {
                panic!("a rejected compilation must not publish dependencies")
            });
        let Err(CToolFailure::Tool(error)) = accounted_error else {
            panic!("compiler rejection lost its tool diagnostic: {accounted_error:?}");
        };
        assert_eq!(error.0, original_error.0, "lossy diagnostic text changed");
        assert!(error.0.contains("missing_output_text_control"));
        assert_eq!(resources.stats().current.io, 0);
    }

    #[test]
    fn include_path_extraction_borrows_locale_independent_original_slice() {
        for (line, expected) in [
            (
                "Note: including file: D:\\foo\\header.h  ",
                Some("D:\\foo\\header.h"),
            ),
            (
                "注意: 包含文件: \"D:/目录/header.h\"",
                Some("D:/目录/header.h"),
            ),
            (
                "Ω prefix E:/with spaces/settings",
                Some("E:/with spaces/settings"),
            ),
            ("no drive-qualified path", None),
            ("D:", None),
            ("\\\\server\\share\\header.h", None),
        ] {
            let actual = extract_drive_path(line);
            assert_eq!(actual, expected);
            if let Some(path) = actual {
                assert!(path.as_ptr() >= line.as_ptr());
                assert!(
                    path.as_ptr() as usize + path.len() <= line.as_ptr() as usize + line.len(),
                    "include extraction allocated or returned unrelated storage"
                );
            }
        }
    }

    /// Tool presence is a hard infra precondition on this machine; the
    /// test asserts the pinned identity shape (path exists, banner
    /// captured, hash 64-hex) and prints it as freeze evidence.
    #[test]
    fn cl_toolchain_resolves() {
        let t = find_cl().expect("MSVC cl.exe must resolve (infra blocker otherwise)");
        assert!(t.cl_exe.is_file());
        assert!(!t.version.is_empty());
        assert_eq!(t.cl_hash.len(), 64);
        println!("cl.exe = {}", t.cl_exe.display());
        println!("banner = {}", t.version);
        println!("blake3 = {}", t.cl_hash);
    }
}
