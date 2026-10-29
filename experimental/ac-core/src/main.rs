//! auto-ac-prototype — PLAN-741 CLI over the AC prototype pipeline, extended
//! by PLAN-745 T-04 with the explicit source profile
//! `auto.source.core-i32.draft` and by PLAN-767 with the scalar profiles
//! (`auto.source.scalar-i32.draft` source, `auto.hir.scalar.draft` HIR)
//! ("ac-probe" is this binary's historical documentation name; invoke it
//! via `cargo run --manifest-path experimental/ac-core/Cargo.toml -- <command>`).
//!
//! Usage:
//!   auto-ac-prototype check <file>
//!       Bind + verify a Schema-bound HIR document — the descriptor is
//!       selected by the document's own schema/revision/profile header
//!       (registered triples only; unknown identities are controlled bind
//!       diagnostics). Print structured diagnostics. Exit 0 on success,
//!       non-zero otherwise.
//!
//!   auto-ac-prototype build <file> --entry <DefId> --output <exe> [--capability <name>]...
//!       check -> capability gate -> native lowering -> rust-lld link.
//!       Leaves <exe>, the intermediate .obj and a link receipt on success.
//!       Artifacts are published as a transaction with rollback: a failed
//!       build never overwrites previously successful artifacts.
//!
//!   auto-ac-prototype check-source <file>
//!       --source-profile auto.source.core-i32.draft|auto.source.scalar-i32.draft
//!       [--capability <name>]...
//!       Parse + resolve + typecheck the source subset of the selected
//!       profile, then bind + verify the generated Atom through the same
//!       pipeline as `check` (the scalar profile binds under the scalar
//!       descriptor and verifies through the scalar entry). The mode is
//!       selected only by the explicit profile flag, never inferred from
//!       the file extension.
//!
//!   auto-ac-prototype build-source <file>
//!       --source-profile auto.source.core-i32.draft|auto.source.scalar-i32.draft
//!       --entry <fn-name> --output <exe> [--capability <name>]...
//!       [--support-lib <lib>]...
//!       Same checked-module build path as `build`; the entry is a source
//!       function NAME (zero-parameter, int/i32 result) instead of a DefId.
//!       Source profile, HIR identity triple and input hash are written
//!       into the .ac-link.txt receipt within the same publish transaction.
//!
//! A missing or unknown --source-profile value is a usage error (exit 2).
//! Pipeline rejections exit 1; success 0; usage errors 2. All diagnostics
//! are printed as `file:line:col: error[stage/code]: message`.

use auto_ac_prototype::source::{
    compile_record_source, compile_scalar_source, compile_source, SourceProfile,
    RECORD_SOURCE_PROFILE, SCALAR_SOURCE_PROFILE, SOURCE_PROFILE, TEXT_RECORD_SOURCE_PROFILE,
    TEXT_SOURCE_PROFILE,
};
use auto_ac_prototype::verify;
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
    let bundle = match auto_ac_prototype::bind_document(&src) {
        Ok(b) => b,
        Err(diags) => return Err((EXIT_PIPELINE, diags)),
    };
    // verify through the entry of the document's own domain
    let checked = match match bundle.domain {
        auto_ac_prototype::hir::HirDomain::Core => verify::verify(bundle),
        auto_ac_prototype::hir::HirDomain::Scalar => verify::verify_scalar(bundle),
        auto_ac_prototype::hir::HirDomain::Text => verify::verify_text(bundle),
        auto_ac_prototype::hir::HirDomain::Record => verify::verify_record(bundle),
        auto_ac_prototype::hir::HirDomain::TextRecord => verify::verify_text_record(bundle),
        // PLAN-862 (T-03a): the entry-unit domain has no verifier entry
        // yet; a document claiming it is a controlled rejection, never a
        // fallback onto another domain's rules.
        auto_ac_prototype::hir::HirDomain::EntryUnit => {
            return Err((
                EXIT_PIPELINE,
                vec![auto_ac_prototype::atom_text::Diagnostic::new(
                    auto_ac_prototype::atom_text::Stage::Verify,
                    "verify.domain",
                    "the entry-unit HIR domain has no verifier entry yet (PLAN-862 T-03b)",
                    auto_ac_prototype::atom_text::Span { start: 0, end: 0 },
                )],
            ));
        }
        // PLAN-864: the collections domain verifies through its own entry
        // (wired with the collections emitter/verifier delivery).
        auto_ac_prototype::hir::HirDomain::Collections => {
            return Err((
                EXIT_PIPELINE,
                vec![auto_ac_prototype::atom_text::Diagnostic::new(
                    auto_ac_prototype::atom_text::Stage::Verify,
                    "verify.domain",
                    "the collections HIR domain is not accepted by this standalone binary",
                    auto_ac_prototype::atom_text::Span { start: 0, end: 0 },
                )],
            ));
        }
        auto_ac_prototype::hir::HirDomain::Closures => verify::verify_closures(bundle),
        auto_ac_prototype::hir::HirDomain::HostConsole => verify::verify_host_console(bundle),
    } {
        Ok(c) => c,
        Err(diags) => return Err((EXIT_PIPELINE, diags)),
    };
    Ok(format!(
        "OK {} module(s), hir-profile {}, capabilities required: {:?}",
        checked.bundle().modules.len(),
        checked.domain().as_str(),
        checked.required_capabilities()
    ))
}

struct SourceArgs {
    file: PathBuf,
    profile: SourceProfile,
    capabilities: Vec<String>,
}

/// Parse the option set of the source CHECK command (PLAN-871 T-06: the
/// build/build-source faces moved to the `ac-native` CLI together with the
/// lowering, so --entry/--output/--support-lib and the runtime-injection
/// knobs are no longer accepted here).
fn parse_source_args(args: &[String]) -> Result<SourceArgs, String> {
    let mut file: Option<PathBuf> = None;
    let mut profile: Option<String> = None;
    let mut capabilities = Vec::new();
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--source-profile" => {
                i += 1;
                profile = Some(args.get(i).ok_or("--source-profile needs a value")?.clone());
            }
            "--capability" => {
                i += 1;
                capabilities.push(args.get(i).ok_or("--capability needs a value")?.clone());
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
    let profile = SourceProfile::of(&profile).ok_or_else(|| {
        format!(
            "unknown source profile `{}` (this build accepts `{}`, `{}`, `{}` or `{}`)",
            profile,
            SOURCE_PROFILE,
            SCALAR_SOURCE_PROFILE,
            TEXT_SOURCE_PROFILE,
            RECORD_SOURCE_PROFILE
        )
    })?;
    let file = file.ok_or("missing input file")?;
    Ok(SourceArgs {
        file,
        profile,
        capabilities,
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
    let compile = match args.profile {
        SourceProfile::HostConsole => {
            auto_ac_prototype::source::adapter::compile_host_console_source(
                &src,
                &module_path,
                &args.capabilities,
            )
        }
        SourceProfile::Core => compile_source(&src, &module_path, &args.capabilities),
        SourceProfile::Scalar => compile_scalar_source(&src, &module_path, &args.capabilities),
        SourceProfile::Text => {
            auto_ac_prototype::source::compile_text_source(&src, &module_path, &args.capabilities)
        }
        SourceProfile::Record => compile_record_source(&src, &module_path, &args.capabilities),
        SourceProfile::TextRecord => auto_ac_prototype::source::compile_text_record_source(
            &src,
            &module_path,
            &args.capabilities,
        ),
        SourceProfile::EntryUnit => auto_ac_prototype::source::compile_entry_unit_source(
            &src,
            &module_path,
            &args.capabilities,
        ),
        // PLAN-864: the collections source profile rides its own entry.
        SourceProfile::Collections => {
            return Err((
                EXIT_PIPELINE,
                vec![auto_ac_prototype::atom_text::Diagnostic::new(
                    auto_ac_prototype::atom_text::Stage::Verify,
                    "verify.domain",
                    "the collections source profile is not accepted by this standalone binary",
                    auto_ac_prototype::atom_text::Span { start: 0, end: 0 },
                )],
            ));
        }
        SourceProfile::Closures => auto_ac_prototype::source::compile_closures_source(
            &src,
            &module_path,
            &args.capabilities,
        ),
    };
    match compile {
        Ok(c) => Ok(format!(
            "OK source profile={}, input-hash={}, module(s)=1, capabilities required: {:?}",
            c.profile,
            c.input_hash,
            c.checked.required_capabilities()
        )),
        Err(diags) => Err((EXIT_PIPELINE, diags)),
    }
}

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let Some(cmd) = args.first() else {
        eprintln!("usage: auto-ac-prototype <check|check-source> ...");
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
        "check-source" => {
            let parsed = match parse_source_args(&args[1..]) {
                Ok(a) => a,
                Err(e) => {
                    eprintln!(
                        "usage: auto-ac-prototype check-source <file> --source-profile {}|{}|{}|{}|{} [--capability <name>]...",
                        SOURCE_PROFILE, SCALAR_SOURCE_PROFILE, TEXT_SOURCE_PROFILE,
                        RECORD_SOURCE_PROFILE, TEXT_RECORD_SOURCE_PROFILE
                    );
                    eprintln!("error: {}", e);
                    return ExitCode::from(EXIT_USAGE);
                }
            };
            let file = parsed.file.clone();
            (file, check_source(&parsed))
        }
        other => {
            eprintln!(
                "unknown command `{}`; expected check|check-source (PLAN-871: build/build-source moved to the `ac-native` CLI)",
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
