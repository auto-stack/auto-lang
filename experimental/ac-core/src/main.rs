//! auto-ac-prototype — PLAN-741 CLI over the AC prototype pipeline, extended
//! by PLAN-745 T-04 with the explicit source profile
//! `auto.source.core-i32.draft` ("ac-probe" is this binary's historical
//! documentation name; invoke it via
//! `cargo run --manifest-path experimental/ac-core/Cargo.toml -- <command>`).
//!
//! Usage:
//!   auto-ac-prototype check <file>
//!       Bind + verify a Schema-bound HIR document; print structured
//!       diagnostics. Exit 0 on success, non-zero otherwise.
//!
//!   auto-ac-prototype build <file> --entry <DefId> --output <exe> [--capability <name>]...
//!       check -> capability gate -> native lowering -> rust-lld link.
//!       Leaves <exe>, the intermediate .obj and a link receipt on success.
//!       Artifacts are published as a transaction with rollback: a failed
//!       build never overwrites previously successful artifacts.
//!
//!   auto-ac-prototype check-source <file> --source-profile auto.source.core-i32.draft
//!       [--capability <name>]...
//!       Parse + resolve + typecheck the source subset, then bind + verify
//!       the generated Atom through the same pipeline as `check`. The mode
//!       is selected only by the explicit profile flag, never inferred from
//!       the file extension.
//!
//!   auto-ac-prototype build-source <file>
//!       --source-profile auto.source.core-i32.draft --entry <fn-name>
//!       --output <exe> [--capability <name>]... [--support-lib <lib>]...
//!       Same checked-module build path as `build`; the entry is a source
//!       function NAME (zero-parameter, int/i32 result) instead of a DefId.
//!       Source profile and input hash are written into the .ac-link.txt
//!       receipt within the same publish transaction.
//!
//! A missing or unknown --source-profile value is a usage error (exit 2).
//! Pipeline rejections exit 1; success 0; usage errors 2. All diagnostics
//! are printed as `file:line:col: error[stage/code]: message`.

use auto_ac_prototype::source::{compile_source, SOURCE_PROFILE};
use auto_ac_prototype::{link, native, verify};
use std::path::{Path, PathBuf};
use std::process::ExitCode;

const EXIT_USAGE: u8 = 2;
const EXIT_PIPELINE: u8 = 1;

fn print_diags(file: &str, src: &str, diags: &[auto_ac_prototype::atom_text::Diagnostic]) {
    for d in diags {
        eprintln!("{}", d.render(file, src));
    }
}

fn check(file: &Path) -> Result<String, (u8, Vec<auto_ac_prototype::atom_text::Diagnostic>)> {
    let src = match std::fs::read_to_string(file) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("error: cannot read {}: {}", file.display(), e);
            return Err((EXIT_USAGE, vec![]));
        }
    };
    let bundle = match auto_ac_prototype::bind_source(&src) {
        Ok(b) => b,
        Err(diags) => return Err((EXIT_PIPELINE, diags)),
    };
    match verify::verify(bundle) {
        Ok(checked) => Ok(format!(
            "OK {} module(s), capabilities required: {:?}",
            checked.bundle().modules.len(),
            checked.required_capabilities()
        )),
        Err(diags) => Err((EXIT_PIPELINE, diags)),
    }
}

struct BuildArgs {
    file: PathBuf,
    entry: String,
    output: PathBuf,
    capabilities: Vec<String>,
    support_libs: Vec<PathBuf>,
}

struct SourceArgs {
    file: PathBuf,
    entry: Option<String>,
    output: Option<PathBuf>,
    capabilities: Vec<String>,
    support_libs: Vec<PathBuf>,
}

/// Parse the shared option set of the two source commands. `need_build`
/// demands --entry/--output (build-source).
fn parse_source_args(args: &[String], need_build: bool) -> Result<SourceArgs, String> {
    let mut file: Option<PathBuf> = None;
    let mut profile: Option<String> = None;
    let mut entry: Option<String> = None;
    let mut output: Option<PathBuf> = None;
    let mut capabilities = Vec::new();
    let mut support_libs = Vec::new();
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--source-profile" => {
                i += 1;
                profile = Some(args.get(i).ok_or("--source-profile needs a value")?.clone());
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
                    return Err(format!("unknown option `{}`", other));
                }
                if file.is_some() {
                    return Err(format!("unexpected extra argument `{}`", other));
                }
                file = Some(PathBuf::from(other));
            }
        }
        i += 1;
    }
    let profile = profile.ok_or("missing --source-profile <name>")?;
    if profile != SOURCE_PROFILE {
        return Err(format!(
            "unknown source profile `{}` (this build accepts `{}`)",
            profile, SOURCE_PROFILE
        ));
    }
    let file = file.ok_or("missing input file")?;
    if need_build {
        if entry.is_none() {
            return Err("missing --entry <fn-name>".to_string());
        }
        if output.is_none() {
            return Err("missing --output <exe>".to_string());
        }
    }
    Ok(SourceArgs {
        file,
        entry,
        output,
        capabilities,
        support_libs,
    })
}

fn check_source(
    args: &SourceArgs,
) -> Result<String, (u8, Vec<auto_ac_prototype::atom_text::Diagnostic>)> {
    let src = match std::fs::read_to_string(&args.file) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("error: cannot read {}: {}", args.file.display(), e);
            return Err((EXIT_USAGE, vec![]));
        }
    };
    let module_path = args
        .file
        .file_name()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_else(|| "source.at".to_string());
    match compile_source(&src, &module_path, &args.capabilities) {
        Ok(c) => Ok(format!(
            "OK source profile={}, input-hash={}, module(s)=1, capabilities required: {:?}",
            c.profile,
            c.input_hash,
            c.checked.required_capabilities()
        )),
        Err(diags) => Err((EXIT_PIPELINE, diags)),
    }
}

fn build_source(
    args: &SourceArgs,
) -> Result<String, (u8, Vec<auto_ac_prototype::atom_text::Diagnostic>)> {
    let src = match std::fs::read_to_string(&args.file) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("error: cannot read {}: {}", args.file.display(), e);
            return Err((EXIT_USAGE, vec![]));
        }
    };
    let module_path = args
        .file
        .file_name()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_else(|| "source.at".to_string());
    let fail = |diags: Vec<auto_ac_prototype::atom_text::Diagnostic>| Err((EXIT_PIPELINE, diags));

    let compiled = match compile_source(&src, &module_path, &args.capabilities) {
        Ok(c) => c,
        Err(diags) => return fail(diags),
    };
    let checked = compiled.checked;
    if let Err(diags) = native::capabilities_check(&checked, &args.capabilities) {
        return fail(diags);
    }
    // The source entry is a function NAME; locate its def id in the bound
    // module, then let the backend's entry checks own the signature rules.
    let entry_name = args.entry.clone().unwrap_or_default();
    let bundle = checked.bundle();
    let mut found = Vec::new();
    for (mi, m) in bundle.modules.iter().enumerate() {
        for (di, d) in m.defs.iter().enumerate() {
            if let auto_ac_prototype::hir::DefKind::Function { name, .. } = &d.kind {
                if *name == entry_name {
                    found.push((mi, di, d.id_text.clone()));
                }
            }
        }
    }
    let entry_def_id = match found.len() {
        0 => {
            return fail(vec![auto_ac_prototype::atom_text::Diagnostic::new(
                auto_ac_prototype::atom_text::Stage::Backend,
                "entry.not-found",
                format!("no function named `{}` in this source module", entry_name),
                auto_ac_prototype::atom_text::Span { start: 0, end: 0 },
            )]);
        }
        1 => found[0].2.clone(),
        _ => {
            return fail(vec![auto_ac_prototype::atom_text::Diagnostic::new(
                auto_ac_prototype::atom_text::Stage::Backend,
                "entry.ambiguous",
                format!(
                    "function name `{}` occurs in {} modules; names are module-local",
                    entry_name,
                    found.len()
                ),
                auto_ac_prototype::atom_text::Span { start: 0, end: 0 },
            )]);
        }
    };
    let sel = match native::find_entry(&checked, &entry_def_id) {
        Ok(s) => s,
        Err(diags) => return fail(diags),
    };
    let obj_bytes = match native::lower_object(&checked, &sel) {
        Ok(b) => b,
        Err(diags) => return fail(diags),
    };

    // Artifacts live next to the requested exe; the object is written to a
    // temporary path first so a failed link never leaves a stale .obj pair.
    let output = args
        .output
        .clone()
        .unwrap_or_else(|| PathBuf::from("a.exe"));
    let out_dir = output.parent().unwrap_or(Path::new("."));
    let stem = output
        .file_stem()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_else(|| "ac".to_string());
    if let Err(e) = std::fs::create_dir_all(out_dir) {
        eprintln!("error: create {}: {}", out_dir.display(), e);
        return Err((EXIT_USAGE, vec![]));
    }
    let tmp_obj = out_dir.join(format!("{}.obj.tmp-{}", stem, link::staging_tag()));
    let discard_staged_obj = |tmp_obj: &Path| match std::fs::remove_file(tmp_obj) {
        Err(e) if e.kind() != std::io::ErrorKind::NotFound => {
            eprintln!(
                "error: cleanup of staged {} failed: {} (release the blocking handle, then remove)",
                tmp_obj.display(),
                e
            );
        }
        _ => {}
    };
    if let Err(e) = std::fs::write(&tmp_obj, &obj_bytes) {
        discard_staged_obj(&tmp_obj);
        eprintln!("error: write {}: {}", tmp_obj.display(), e);
        return Err((EXIT_PIPELINE, vec![]));
    }
    let obj_path = out_dir.join(format!("{}.obj", stem));
    let lld = match link::find_rust_lld() {
        Ok(l) => l,
        Err(d) => {
            discard_staged_obj(&tmp_obj);
            return Err((EXIT_PIPELINE, vec![d]));
        }
    };
    let sdk = match link::find_sdk_um_dir() {
        Ok(s) => s,
        Err(d) => {
            discard_staged_obj(&tmp_obj);
            return Err((EXIT_PIPELINE, vec![d]));
        }
    };
    let staged = match link::link_object_staged(
        &lld,
        &sdk,
        &tmp_obj,
        &output,
        native::START_SYMBOL,
        &args.support_libs,
    ) {
        Ok(s) => s,
        Err(d) => {
            discard_staged_obj(&tmp_obj);
            return Err((EXIT_PIPELINE, vec![d]));
        }
    };
    // Source metadata joins the receipt inside the same publish
    // transaction (no side-channel receipt writes).
    let receipt_extra = format!(
        "source-profile: {}\nsource-input-hash: {}\n",
        compiled.profile, compiled.input_hash
    );
    let receipt =
        match link::publish_artifacts_with_receipt(&staged, &tmp_obj, &obj_path, &receipt_extra) {
            Ok(r) => r,
            Err(d) => return Err((EXIT_PIPELINE, vec![d])),
        };
    Ok(format!(
        "BUILT {} (entry {})\n  obj: {}\n  exe: {}\n  receipt: {}\n  linker: {}\n  source-profile: {}\n  source-input-hash: {}",
        output.display(),
        sel.symbol,
        receipt.obj.display(),
        receipt.exe.display(),
        receipt.receipt_path.display(),
        receipt.linker.display(),
        compiled.profile,
        compiled.input_hash
    ))
}

fn parse_build_args(args: &[String]) -> Result<BuildArgs, String> {
    let mut file = None;
    let mut entry = None;
    let mut output = None;
    let mut capabilities = Vec::new();
    let mut support_libs = Vec::new();
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--entry" => {
                i += 1;
                entry = Some(args.get(i).ok_or("--entry needs a value")?.clone());
            }
            "--output" => {
                i += 1;
                output = Some(args.get(i).ok_or("--output needs a value")?.clone());
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
                    return Err(format!("unknown option `{}`", other));
                }
                if file.is_some() {
                    return Err(format!("unexpected extra argument `{}`", other));
                }
                file = Some(PathBuf::from(other));
            }
        }
        i += 1;
    }
    Ok(BuildArgs {
        file: file.ok_or("missing input file")?,
        entry: entry.ok_or("missing --entry <DefId>")?,
        output: PathBuf::from(output.ok_or("missing --output <exe>")?),
        capabilities,
        support_libs,
    })
}

fn build(args: &BuildArgs) -> Result<String, (u8, Vec<auto_ac_prototype::atom_text::Diagnostic>)> {
    let src = match std::fs::read_to_string(&args.file) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("error: cannot read {}: {}", args.file.display(), e);
            return Err((EXIT_USAGE, vec![]));
        }
    };
    let fail = |diags: Vec<auto_ac_prototype::atom_text::Diagnostic>| Err((EXIT_PIPELINE, diags));

    let bundle = match auto_ac_prototype::bind_source(&src) {
        Ok(b) => b,
        Err(diags) => return fail(diags),
    };
    let checked = match verify::verify(bundle) {
        Ok(c) => c,
        Err(diags) => return fail(diags),
    };
    if let Err(diags) = native::capabilities_check(&checked, &args.capabilities) {
        return fail(diags);
    }
    let sel = match native::find_entry(&checked, &args.entry) {
        Ok(s) => s,
        Err(diags) => return fail(diags),
    };
    let obj_bytes = match native::lower_object(&checked, &sel) {
        Ok(b) => b,
        Err(diags) => return fail(diags),
    };

    // Artifacts live next to the requested exe; the object is written to a
    // temporary path first so a failed link never leaves a stale .obj pair.
    let out_dir = args.output.parent().unwrap_or(Path::new("."));
    let stem = args
        .output
        .file_stem()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_else(|| "ac".to_string());
    if let Err(e) = std::fs::create_dir_all(out_dir) {
        eprintln!("error: create {}: {}", out_dir.display(), e);
        return Err((EXIT_USAGE, vec![]));
    }
    let tmp_obj = out_dir.join(format!("{}.obj.tmp-{}", stem, link::staging_tag()));
    // R3-QA-02: staging cleanup errors are never swallowed — if a blocked
    // handle leaves a staged file behind, the operator sees the path and the
    // real OS error. `publish_artifacts` reports its own cleanup.
    let discard_staged_obj = |tmp_obj: &Path| match std::fs::remove_file(tmp_obj) {
        Err(e) if e.kind() != std::io::ErrorKind::NotFound => {
            eprintln!(
                "error: cleanup of staged {} failed: {} (release the blocking handle, then remove)",
                tmp_obj.display(),
                e
            );
        }
        _ => {}
    };
    if let Err(e) = std::fs::write(&tmp_obj, &obj_bytes) {
        discard_staged_obj(&tmp_obj);
        eprintln!("error: write {}: {}", tmp_obj.display(), e);
        return Err((EXIT_PIPELINE, vec![]));
    }
    let obj_path = out_dir.join(format!("{}.obj", stem));
    let lld = match link::find_rust_lld() {
        Ok(l) => l,
        Err(d) => {
            discard_staged_obj(&tmp_obj);
            return Err((EXIT_PIPELINE, vec![d]));
        }
    };
    let sdk = match link::find_sdk_um_dir() {
        Ok(s) => s,
        Err(d) => {
            discard_staged_obj(&tmp_obj);
            return Err((EXIT_PIPELINE, vec![d]));
        }
    };
    let staged = match link::link_object_staged(
        &lld,
        &sdk,
        &tmp_obj,
        &args.output,
        native::START_SYMBOL,
        &args.support_libs,
    ) {
        Ok(s) => s,
        Err(d) => {
            discard_staged_obj(&tmp_obj);
            return Err((EXIT_PIPELINE, vec![d]));
        }
    };
    // Publish exe + obj + receipt as one transaction; a failure at any step
    // rolls back and leaves previously successful artifacts untouched. The
    // transaction owns its staged files and reports blocked cleanups itself.
    let receipt = match link::publish_artifacts(&staged, &tmp_obj, &obj_path) {
        Ok(r) => r,
        Err(d) => return Err((EXIT_PIPELINE, vec![d])),
    };
    Ok(format!(
        "BUILT {} (entry {})\n  obj: {}\n  exe: {}\n  receipt: {}\n  linker: {}",
        args.output.display(),
        sel.symbol,
        receipt.obj.display(),
        receipt.exe.display(),
        receipt.receipt_path.display(),
        receipt.linker.display()
    ))
}

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let Some(cmd) = args.first() else {
        eprintln!("usage: auto-ac-prototype <check|build|check-source|build-source> ...");
        return ExitCode::from(EXIT_USAGE);
    };
    let (file, result) = match cmd.as_str() {
        "check" => {
            let Some(file) = args.get(1) else {
                eprintln!("usage: auto-ac-prototype check <file>");
                return ExitCode::from(EXIT_USAGE);
            };
            (PathBuf::from(file), check(Path::new(file)))
        }
        "build" => {
            let parsed = match parse_build_args(&args[1..]) {
                Ok(a) => a,
                Err(e) => {
                    eprintln!(
                        "usage: auto-ac-prototype build <file> --entry <DefId> --output <exe> [--capability <name>]..."
                    );
                    eprintln!("error: {}", e);
                    return ExitCode::from(EXIT_USAGE);
                }
            };
            let file = parsed.file.clone();
            (file, build(&parsed))
        }
        "check-source" => {
            let parsed = match parse_source_args(&args[1..], false) {
                Ok(a) => a,
                Err(e) => {
                    eprintln!(
                        "usage: auto-ac-prototype check-source <file> --source-profile {} [--capability <name>]...",
                        SOURCE_PROFILE
                    );
                    eprintln!("error: {}", e);
                    return ExitCode::from(EXIT_USAGE);
                }
            };
            let file = parsed.file.clone();
            (file, check_source(&parsed))
        }
        "build-source" => {
            let parsed = match parse_source_args(&args[1..], true) {
                Ok(a) => a,
                Err(e) => {
                    eprintln!(
                        "usage: auto-ac-prototype build-source <file> --source-profile {} --entry <fn-name> --output <exe> [--capability <name>]... [--support-lib <lib>]...",
                        SOURCE_PROFILE
                    );
                    eprintln!("error: {}", e);
                    return ExitCode::from(EXIT_USAGE);
                }
            };
            let file = parsed.file.clone();
            (file, build_source(&parsed))
        }
        other => {
            eprintln!(
                "unknown command `{}`; expected check|build|check-source|build-source",
                other
            );
            return ExitCode::from(EXIT_USAGE);
        }
    };
    match result {
        Ok(msg) => {
            println!("{}", msg);
            ExitCode::SUCCESS
        }
        Err((code, diags)) => {
            let src = std::fs::read_to_string(&file).unwrap_or_default();
            print_diags(&file.display().to_string(), &src, &diags);
            ExitCode::from(code)
        }
    }
}
