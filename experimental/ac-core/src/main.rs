//! ac-probe — PLAN-741 CLI over the AC prototype pipeline.
//!
//! Usage:
//!   ac-probe check <file>
//!       Bind + verify a Schema-bound HIR document; print structured
//!       diagnostics. Exit 0 on success, non-zero otherwise.
//!
//!   ac-probe build <file> --entry <DefId> --output <exe> [--capability <name>]...
//!       check -> capability gate -> native lowering -> rust-lld link.
//!       Leaves <exe>, the intermediate .obj and a link receipt on success.
//!       A failed build never overwrites previously successful artifacts.
//!
//! All diagnostics are printed as `file:line:col: error[stage/code]: message`
//! with non-zero process exit codes (plan AC-07).

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
    let tmp_obj = out_dir.join(format!("{}.obj.tmp-ac741", stem));
    if let Err(e) = std::fs::write(&tmp_obj, &obj_bytes) {
        eprintln!("error: write {}: {}", tmp_obj.display(), e);
        return Err((EXIT_PIPELINE, vec![]));
    }
    let obj_path = out_dir.join(format!("{}.obj", stem));
    let lld = match link::find_rust_lld() {
        Ok(l) => l,
        Err(d) => {
            let _ = std::fs::remove_file(&tmp_obj);
            return Err((EXIT_PIPELINE, vec![d]));
        }
    };
    let sdk = match link::find_sdk_um_dir() {
        Ok(s) => s,
        Err(d) => {
            let _ = std::fs::remove_file(&tmp_obj);
            return Err((EXIT_PIPELINE, vec![d]));
        }
    };
    let receipt = match link::link_object(
        &lld,
        &sdk,
        &tmp_obj,
        &args.output,
        native::START_SYMBOL,
        &args.support_libs,
    ) {
        Ok(r) => r,
        Err(d) => {
            let _ = std::fs::remove_file(&tmp_obj);
            return Err((EXIT_PIPELINE, vec![d]));
        }
    };
    // obj final placement after the exe replaced the target atomically.
    if let Err(e) = std::fs::rename(&tmp_obj, &obj_path) {
        eprintln!("error: placing {}: {}", obj_path.display(), e);
        return Err((EXIT_PIPELINE, vec![]));
    }
    Ok(format!(
        "BUILT {} (entry {})\n  obj: {}\n  exe: {}\n  receipt: {}\n  linker: {}",
        args.output.display(),
        sel.symbol,
        obj_path.display(),
        receipt.exe.display(),
        receipt.exe.with_extension("ac-link.txt").display(),
        receipt.linker.display()
    ))
}

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let Some(cmd) = args.first() else {
        eprintln!("usage: ac-probe <check|build> ...");
        return ExitCode::from(EXIT_USAGE);
    };
    let (file, result) = match cmd.as_str() {
        "check" => {
            let Some(file) = args.get(1) else {
                eprintln!("usage: ac-probe check <file>");
                return ExitCode::from(EXIT_USAGE);
            };
            (PathBuf::from(file), check(Path::new(file)))
        }
        "build" => {
            let parsed = match parse_build_args(&args[1..]) {
                Ok(a) => a,
                Err(e) => {
                    eprintln!(
                        "usage: ac-probe build <file> --entry <DefId> --output <exe> [--capability <name>]..."
                    );
                    eprintln!("error: {}", e);
                    return ExitCode::from(EXIT_USAGE);
                }
            };
            let file = parsed.file.clone();
            (file, build(&parsed))
        }
        other => {
            eprintln!("unknown command `{}`; expected check|build", other);
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
