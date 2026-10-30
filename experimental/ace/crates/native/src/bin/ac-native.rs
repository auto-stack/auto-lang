//! `ac-native` — the PLAN-871 native CLI (the five-profile consumer face).
//!
//! It is the driver-layer switch point that replaces the retiring
//! `auto-ac-prototype build|build-source` commands (PLAN-871 T-02 F4,
//! explicit experimental-CLI compatibility notice):
//!
//! - `build <hir.atom> --entry <id|name> --output <exe>` — raw canonical
//!   HIR input (bind + domain verify + shared-MIR lowering);
//! - `build-source <file.at> --source-profile <p> --entry <name> --output <exe>`
//!   — the five frozen source profiles (core/scalar/text/record/text-record);
//! - `backend [--json]` — the OBSERVABLE switch state (per profile).
//!
//! Products: the exe plus `<stem>.ac-link.txt`, a receipt carrying the old
//! identity needles (source-profile / hir-schema / hir-revision /
//! hir-profile / source-input-hash) AND the new axes (backend /
//! mir-lowering / per-unit SHA-256 / manifest fingerprint). The single
//! `<stem>.obj` of the retiring CLI is gone — mn links a multi-unit closure
//! and its canonical objects live in the A09' store, never beside the exe.
//!
//! Backend selection is the freeze-F1 env switch (`AC_NATIVE_BACKEND`,
//! read per invocation). This CLI speaks ONLY the shared-MIR backend: when
//! the resolved backend for the requested profile is `legacy`, it refuses
//! with an explicit message (the legacy CLI is the migration-window
//! fallback) instead of silently building something else.

use ace_core::atom_text::{Diagnostic, Stage};
use ace_core::verify::CheckedModule;
use ace_native::backend::{self, Backend, Profile};
use ace_native::driver::MnDriver;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

const EXIT_USAGE: u8 = 2;
const EXIT_PIPELINE: u8 = 1;

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let Some(cmd) = args.first().map(|s| s.as_str()) else {
        usage();
        return ExitCode::from(EXIT_USAGE);
    };
    match cmd {
        "backend" => print_backend(&args[1..]),
        "build" => match parse_build(&args[1..]) {
            Ok(req) => report(run_build(&req, None)),
            Err(e) => {
                eprintln!("usage: ac-native build <hir.atom> --entry <id|name> --output <exe> [--capability <name>]... [--support-lib <lib>]");
                eprintln!("error: {e}");
                ExitCode::from(EXIT_USAGE)
            }
        },
        "build-source" => match parse_build(&args[1..]) {
            Ok(req) => report(run_build(&req, req.profile)),
            Err(e) => {
                eprintln!("usage: ac-native build-source <file.at> --source-profile <core|scalar|text|record|text-record|collections> --entry <name> --output <exe> [--capability <name>]... [--support-lib <lib>]");
                eprintln!("error: {e}");
                ExitCode::from(EXIT_USAGE)
            }
        },
        "help" | "--help" | "-h" => {
            usage();
            ExitCode::SUCCESS
        }
        other => {
            eprintln!("unknown command `{other}`; expected build|build-source|backend");
            ExitCode::from(EXIT_USAGE)
        }
    }
}

fn usage() {
    eprintln!("ac-native — PLAN-871 shared-MIR native CLI");
    eprintln!("  build <hir.atom> --entry <id|name> --output <exe> [--capability <name>]... [--support-lib <lib>]");
    eprintln!("  build-source <file.at> --source-profile <core|scalar|text|record|text-record|collections> --entry <name> --output <exe> [--capability <name>]... [--support-lib <lib>]");
    eprintln!("  backend [--json]   print the AC_NATIVE_BACKEND switch state (per profile)");
}

fn report(result: Result<String, (u8, Vec<Diagnostic>)>) -> ExitCode {
    match result {
        Ok(msg) => {
            println!("{msg}");
            ExitCode::SUCCESS
        }
        Err((code, diags)) => {
            for d in &diags {
                eprintln!("{} {}: {}", d.stage.as_str(), d.code, d.message);
            }
            ExitCode::from(code)
        }
    }
}

/// `backend` subcommand: the switch table as the process sees it NOW
/// (env read per call — the freeze-F1 observability surface).
fn print_backend(args: &[String]) -> ExitCode {
    let json = args.iter().any(|a| a == "--json");
    let raw = std::env::var("AC_NATIVE_BACKEND").ok();
    let table = match backend::table(raw.as_deref()) {
        Ok(t) => t,
        Err(e) => {
            eprintln!("error: {e}");
            return ExitCode::from(EXIT_USAGE);
        }
    };
    if json {
        let rows: Vec<String> = table
            .iter()
            .map(|(p, b)| {
                format!(
                    "{{\"profile\":\"{}\",\"backend\":\"{}\",\"tag\":\"{}\"}}",
                    p.as_str(),
                    b.as_str(),
                    b.tag()
                )
            })
            .collect();
        println!("[{}]", rows.join(","));
    } else {
        println!("AC_NATIVE_BACKEND switch state (default table = mn after the T-03 gates)");
        if let Some(raw) = raw.as_deref().filter(|r| !r.trim().is_empty()) {
            println!("env override: {raw}");
        } else {
            println!("env override: (none)");
        }
        for (p, b) in table {
            println!("  {:<12} {}  ({})", p.as_str(), b.as_str(), b.tag());
        }
    }
    ExitCode::SUCCESS
}

struct BuildReq {
    file: PathBuf,
    profile: Option<Profile>,
    entry: String,
    output: PathBuf,
    capabilities: Vec<String>,
    support_libs: Vec<PathBuf>,
}

fn parse_build(args: &[String]) -> Result<BuildReq, String> {
    let mut file = None;
    let mut profile = None;
    let mut entry = None;
    let mut output = None;
    let mut capabilities = Vec::new();
    let mut support_libs = Vec::new();
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--source-profile" => {
                i += 1;
                let v = args.get(i).ok_or("--source-profile needs a value")?;
                profile = Some(parse_profile(v)?);
            }
            "--entry" => {
                i += 1;
                entry = Some(args.get(i).ok_or("--entry needs a value")?.clone());
            }
            "--output" => {
                i += 1;
                output = Some(PathBuf::from(args.get(i).ok_or("--output needs a value")?));
            }
            "--capability" => {
                i += 1;
                capabilities.push(args.get(i).ok_or("--capability needs a value")?.clone());
            }
            "--support-lib" => {
                i += 1;
                support_libs.push(PathBuf::from(
                    args.get(i).ok_or("--support-lib needs a value")?,
                ));
            }
            other => {
                if other.starts_with("--") {
                    return Err(format!("unknown option `{other}`"));
                }
                if file.is_some() {
                    return Err(format!("unexpected extra argument `{other}`"));
                }
                file = Some(PathBuf::from(other));
            }
        }
        i += 1;
    }
    if support_libs.len() > 1 {
        return Err(
            "at most one --support-lib (the domain runtime library); the multi-lib build face is retired"
                .to_string(),
        );
    }
    Ok(BuildReq {
        file: file.ok_or("missing input file")?,
        profile,
        entry: entry.ok_or("missing --entry <id|name>")?,
        output: output.ok_or("missing --output <exe>")?,
        capabilities,
        support_libs,
    })
}

fn parse_profile(raw: &str) -> Result<Profile, String> {
    if let Some(p) = Profile::parse(raw) {
        return Ok(p);
    }
    // The retiring CLI spelled profiles as their source-profile constants;
    // accepting them keeps the T-05 script rewrite a one-line change.
    let p = match raw {
        "auto.source.core-i32.draft" => Profile::Core,
        "auto.source.scalar-i32.draft" => Profile::Scalar,
        "auto.source.text.draft" => Profile::Text,
        "auto.source.record.draft" => Profile::Record,
        "auto.source.text-record.draft" => Profile::TextRecord,
        "auto.source.collections.draft" => Profile::Collections,
        other => {
            return Err(format!(
                "unknown source profile `{other}` (expected core|scalar|text|record|text-record or its auto.source.*.draft constant)"
            ))
        }
    };
    Ok(p)
}

fn run_build(req: &BuildReq, profile: Option<Profile>) -> Result<String, (u8, Vec<Diagnostic>)> {
    if profile == Some(Profile::Collections) {
        return run_build_collections(req);
    }
    let src = std::fs::read_to_string(&req.file).map_err(|e| {
        eprintln!("error: cannot read {}: {}", req.file.display(), e);
        (EXIT_USAGE, Vec::new())
    })?;

    // ---- switch: this CLI builds mn only; a legacy-resolved profile is an
    // explicit refusal, never a silent different-backend build (F1/F4).
    let effective_profile = profile.unwrap_or(Profile::Core);
    match backend::resolve(effective_profile) {
        Ok(Backend::Mn) => {}
        Ok(Backend::Legacy) => {
            eprintln!(
                "error: profile `{}` resolves to the LEGACY HIR→CLIF backend, which was RETIRED (PLAN-871 T-06); unset AC_NATIVE_BACKEND or set it to `mn`",
                effective_profile.as_str()
            );
            return Err((EXIT_USAGE, Vec::new()));
        }
        Err(e) => {
            eprintln!("error: {e}");
            return Err((EXIT_USAGE, Vec::new()));
        }
    }

    let (compiled, receipt_head) = match profile {
        None => {
            let bundle = ace_core::bind_document(&src).map_err(|ds| (EXIT_PIPELINE, ds))?;
            let domain = bundle.domain;
            let checked = verify_domain(bundle)?;
            let head = format!(
                "hir-domain: {}\nhir-schema: {}\nhir-revision: {}u32",
                domain.as_str(),
                domain.schema(),
                domain.revision()
            );
            (checked, head)
        }
        Some(p) => {
            let caps = req.capabilities.clone();
            let c = compile_profile(p, &src, &caps).map_err(|ds| (EXIT_PIPELINE, ds))?;
            let head = format!(
                "source-profile: {}\nhir-schema: {}\nhir-revision: {}u32\nhir-profile: {}\nsource-input-hash: {}",
                c.profile,
                c.checked.domain().schema(),
                c.checked.domain().revision(),
                c.checked.domain().as_str(),
                c.input_hash
            );
            (c.checked, head)
        }
    };

    if let Err(ds) = ace_core::verify::capabilities_check(&compiled, &req.capabilities) {
        return Err((EXIT_PIPELINE, ds));
    }
    // PLAN-871 T-06: the text runtime CONTRACT gate follows the owned
    // domains. mn auto-locates the repo's own runtime library, but an
    // EXPLICIT --support-lib is caller-supplied and must prove its identity
    // (api/revision/target + lib_sha256 sidecar) exactly like the retiring
    // CLI did — the capability string alone is never trusted.
    {
        use ace_core::hir::HirDomain;
        let domain = compiled.domain();
        if matches!(domain, HirDomain::Text | HirDomain::TextRecord) {
            if let Some(lib) = req.support_libs.first() {
                if let Err(ds) = ace_core::runtime_contract::check(std::slice::from_ref(lib)) {
                    return Err((EXIT_PIPELINE, ds));
                }
            }
        }
    }
    let (entry_id, entry_symbol) =
        ace_native::entry::resolve(&compiled, &req.entry).map_err(|ds| (EXIT_PIPELINE, ds))?;

    // ---- build through the shared-MIR backend ------------------------------
    let out_dir = req.output.parent().unwrap_or(Path::new("."));
    std::fs::create_dir_all(out_dir).map_err(|e| {
        eprintln!("error: create {}: {}", out_dir.display(), e);
        (EXIT_USAGE, Vec::new())
    })?;
    let stem = req
        .output
        .file_stem()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_else(|| "ac".to_string());
    let workdir = out_dir.join(format!(".ac-native-{stem}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&workdir);

    let snapshot = ace_av_core::contract::fnv1a64(src.as_bytes());
    let mut driver = MnDriver::ephemeral();
    let built = driver.build_checked(
        &compiled,
        &entry_id,
        &workdir,
        None,
        req.support_libs.first().map(|p| p.as_path()),
        snapshot,
    );

    let cleanup = |workdir: &Path| {
        if let Err(e) = std::fs::remove_dir_all(workdir) {
            if e.kind() != std::io::ErrorKind::NotFound {
                eprintln!(
                    "warning: build scratch {} not removed: {}",
                    workdir.display(),
                    e
                );
            }
        }
    };

    let built = match built {
        Ok(b) => b,
        Err(e) => {
            cleanup(&workdir);
            eprintln!("error: {}", e);
            return Err((EXIT_PIPELINE, Vec::new()));
        }
    };

    // Publish the exe at the requested path (same-volume rename; cross-volume
    // falls back to copy + remove so a temp output on another drive works).
    if let Err(e) = std::fs::rename(&built.exe_path, &req.output) {
        match std::fs::copy(&built.exe_path, &req.output) {
            Ok(_) => {
                let _ = std::fs::remove_file(&built.exe_path);
            }
            Err(_) => {
                cleanup(&workdir);
                eprintln!(
                    "error: publish {} -> {}: {}",
                    built.exe_path.display(),
                    req.output.display(),
                    e
                );
                return Err((EXIT_PIPELINE, Vec::new()));
            }
        }
    }
    let exe_sha = match std::fs::read(&req.output) {
        Ok(b) => sha256_hex(&b),
        Err(e) => {
            cleanup(&workdir);
            eprintln!("error: read published {}: {}", req.output.display(), e);
            return Err((EXIT_PIPELINE, Vec::new()));
        }
    };
    let linker = match ace_core::link::find_rust_lld() {
        Ok(l) => l,
        Err(d) => {
            cleanup(&workdir);
            return Err((EXIT_PIPELINE, vec![d]));
        }
    };

    // ---- receipt: old needles + the new backend/lowering axes --------------
    let receipt_path = out_dir.join(format!("{stem}.ac-link.txt"));
    let mut receipt = String::new();
    receipt.push_str("PLAN-871 ac-native link receipt (ac-native/1)\n");
    receipt.push_str(&format!(
        "backend: {}\nmir-lowering: {}\n",
        Backend::Mn.tag(),
        ace_lowering::MIR_LOWERING_VERSION
    ));
    receipt.push_str(&format!("{receipt_head}\n"));
    receipt.push_str(&format!(
        "entry: {entry_id}\nentry-symbol: {entry_symbol}\n"
    ));
    receipt.push_str(&format!(
        "units: {}\nmanifest-fp: {}\n",
        built.units, built.manifest_fp
    ));
    for (name, sha) in &built.unit_sha256 {
        receipt.push_str(&format!("unit: {name} sha256={sha}\n"));
    }
    receipt.push_str(&format!("exe: {} sha256={exe_sha}\n", req.output.display()));
    receipt.push_str(&format!("linker: {}\n", linker.display()));
    if let Some(p) = req.support_libs.first() {
        receipt.push_str(&format!("support-lib: {}\n", p.display()));
    }
    if let Err(e) = std::fs::write(&receipt_path, &receipt) {
        cleanup(&workdir);
        eprintln!("error: write {}: {}", receipt_path.display(), e);
        return Err((EXIT_PIPELINE, Vec::new()));
    }
    cleanup(&workdir);

    Ok(format!(
        "BUILT {} (entry {})\n  exe: {}\n  receipt: {}\n  linker: {}\n  backend: {}\n  units: {}\n  manifest-fp: {}\n  {}",
        req.output.display(),
        entry_symbol,
        req.output.display(),
        receipt_path.display(),
        linker.display(),
        Backend::Mn.tag(),
        built.units,
        built.manifest_fp,
        receipt_head.replace('\n', "\n  "),
    ))
}

fn run_build_collections(req: &BuildReq) -> Result<String, (u8, Vec<Diagnostic>)> {
    let fail = |error: String| {
        eprintln!("error: {error}");
        (EXIT_PIPELINE, Vec::new())
    };
    match backend::resolve(Profile::Collections) {
        Ok(Backend::Mn) => {}
        Ok(Backend::Legacy) => {
            return Err(fail(
                "collections has no legacy backend; the HIR→CLIF backend was retired".into(),
            ))
        }
        Err(e) => return Err(fail(e.to_string())),
    }
    if !req.capabilities.is_empty() {
        return Err(fail("collections source has no host capabilities".into()));
    }
    use std::io::Read;
    let mut bytes = Vec::new();
    std::fs::File::open(&req.file)
        .map_err(|e| fail(e.to_string()))?
        .take(ace_native::config::MAX_INPUT_BYTES as u64 + 1)
        .read_to_end(&mut bytes)
        .map_err(|e| fail(e.to_string()))?;
    if bytes.len() > ace_native::config::MAX_INPUT_BYTES {
        return Err(fail("collections source exceeds input budget".into()));
    }
    let source = String::from_utf8(bytes).map_err(|e| fail(e.to_string()))?;
    let parent = req
        .output
        .parent()
        .filter(|p| !p.as_os_str().is_empty())
        .unwrap_or(Path::new("."));
    std::fs::create_dir_all(parent).map_err(|e| fail(e.to_string()))?;
    let stamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_err(|e| fail(e.to_string()))?
        .as_nanos();
    let workdir = parent.join(format!(".ac-collections-{}-{stamp}", std::process::id()));
    std::fs::create_dir(&workdir).map_err(|e| fail(e.to_string()))?;
    let built = MnDriver::ephemeral().build_collections(
        &source,
        &req.entry,
        &workdir,
        req.support_libs.first().map(PathBuf::as_path),
    );
    let result = (|| {
        let built = built.map_err(|e| fail(e.to_string()))?;
        let stem = req
            .output
            .file_stem()
            .and_then(|s| s.to_str())
            .ok_or_else(|| fail("invalid output stem".into()))?;
        let receipt = parent.join(format!("{stem}.ac-link.txt"));
        let mut text=format!("PLAN-889 ac-native collections receipt (ac-native.collections/1)\nsource-profile: {}\nhir-schema: {}\nhir-revision: {}u32\nhir-profile: {}\nsource-input-hash: {:016x}\nbackend: mn:{}\nmir-lowering: {}\nprofile: {}\ncodegen-build: {}\nmanifest-fp: {}\nunits: {}\nexe: {} sha256={}\n",
            ace_core::source::COLLECTIONS_SOURCE_PROFILE,ace_core::hir::HirDomain::Collections.schema(),
            ace_core::hir::HirDomain::Collections.revision(),ace_core::hir::HirDomain::Collections.as_str(),
            ace_av_core::contract::fnv1a64(source.as_bytes()),ace_native::collections_lower::ABI,
            ace_lowering::MIR_LOWERING_VERSION,ace_native::collections_layout::PROFILE,ace_native::collections_layout::codegen_build(),
            built.manifest_fp,built.units,req.output.display(),sha256_hex(&std::fs::read(&built.exe_path).map_err(|e|fail(e.to_string()))?));
        for (name, hash) in built.unit_sha256 {
            text.push_str(&format!("unit: {name} sha256={hash}\n"));
        }
        text.push_str(
            &std::fs::read_to_string(&built.receipt_path).map_err(|e| fail(e.to_string()))?,
        );
        // Finish the complete receipt before replacing either user product.
        std::fs::write(&built.receipt_path, text).map_err(|e| fail(e.to_string()))?;
        ace_native::link_driver::publish_products(
            &ace_native::link_driver::PeLink {
                exe_path: built.exe_path,
                receipt_path: built.receipt_path,
            },
            &req.output,
            &receipt,
        )
        .map_err(|e| fail(e.to_string()))?;
        Ok(format!(
            "BUILT {}\n  profile: collections\n  receipt: {}\n  manifest-fp: {}",
            req.output.display(),
            receipt.display(),
            built.manifest_fp
        ))
    })();
    // These are the only scratch product names this driver publishes. Backups
    // from a blocked rollback are deliberately retained with the error path.
    for name in ["collections.exe", ".mn-collections-link.txt"] {
        let _ = std::fs::remove_file(workdir.join(name));
    }
    let _ = std::fs::remove_dir(workdir);
    result
}

fn verify_domain(bundle: ace_core::hir::Bundle) -> Result<CheckedModule, (u8, Vec<Diagnostic>)> {
    use ace_core::hir::HirDomain;
    let checked = match bundle.domain {
        HirDomain::Core => ace_core::verify::verify(bundle),
        HirDomain::Scalar => ace_core::verify::verify_scalar(bundle),
        HirDomain::Text => ace_core::verify::verify_text(bundle),
        HirDomain::Record => ace_core::verify::verify_record(bundle),
        HirDomain::TextRecord => ace_core::verify::verify_text_record(bundle),
        other => {
            return Err((
                EXIT_PIPELINE,
                vec![Diagnostic::new(
                    Stage::Verify,
                    "verify.domain",
                    format!(
                        "the {} HIR domain has no ac-native lowering arm (PLAN-871 covers the five frozen profiles)",
                        other.as_str()
                    ),
                    ace_core::atom_text::Span { start: 0, end: 0 },
                )],
            ))
        }
    };
    checked.map_err(|ds| (EXIT_PIPELINE, ds))
}

fn compile_profile(
    p: Profile,
    src: &str,
    caps: &[String],
) -> Result<ace_core::source::CompiledSource, Vec<Diagnostic>> {
    use ace_core::source::*;
    match p {
        Profile::Core => compile_source(src, "main.at", caps),
        Profile::Scalar => compile_scalar_source(src, "main.at", caps),
        Profile::Text => compile_text_source(src, "main.at", caps),
        Profile::Record => compile_record_source(src, "main.at", caps),
        Profile::TextRecord => compile_text_record_source(src, "main.at", caps),
        Profile::Collections => compile_collections_source(src, "main.at", caps),
    }
}

fn sha256_hex(bytes: &[u8]) -> String {
    use sha2::{Digest, Sha256};
    let mut h = Sha256::new();
    h.update(bytes);
    format!("{:x}", h.finalize())
}
