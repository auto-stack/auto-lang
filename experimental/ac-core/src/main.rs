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
    compile_scalar_source, compile_source, SourceProfile, RUNTIME_TEXT_CAPABILITY,
    SCALAR_SOURCE_PROFILE, SOURCE_PROFILE, TEXT_SOURCE_PROFILE,
};
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
    let bundle = match auto_ac_prototype::bind_document(&src) {
        Ok(b) => b,
        Err(diags) => return Err((EXIT_PIPELINE, diags)),
    };
    // verify through the entry of the document's own domain
    let checked = match match bundle.domain {
        auto_ac_prototype::hir::HirDomain::Core => verify::verify(bundle),
        auto_ac_prototype::hir::HirDomain::Scalar => verify::verify_scalar(bundle),
        auto_ac_prototype::hir::HirDomain::Text => verify::verify_text(bundle),
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

struct BuildArgs {
    file: PathBuf,
    entry: String,
    output: PathBuf,
    capabilities: Vec<String>,
    support_libs: Vec<PathBuf>,
}

struct SourceArgs {
    file: PathBuf,
    profile: SourceProfile,
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
    let profile = SourceProfile::of(&profile).ok_or_else(|| {
        format!(
            "unknown source profile `{}` (this build accepts `{}`, `{}` or `{}`)",
            profile, SOURCE_PROFILE, SCALAR_SOURCE_PROFILE, TEXT_SOURCE_PROFILE
        )
    })?;
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
        profile,
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
    let compile = match args.profile {
        SourceProfile::Core => compile_source(&src, &module_path, &args.capabilities),
        SourceProfile::Scalar => compile_scalar_source(&src, &module_path, &args.capabilities),
        SourceProfile::Text => {
            auto_ac_prototype::source::compile_text_source(&src, &module_path, &args.capabilities)
        }
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

    let compiled = match match args.profile {
        SourceProfile::Core => compile_source(&src, &module_path, &args.capabilities),
        SourceProfile::Scalar => compile_scalar_source(&src, &module_path, &args.capabilities),
        SourceProfile::Text => {
            auto_ac_prototype::source::compile_text_source(&src, &module_path, &args.capabilities)
        }
    } {
        Ok(c) => c,
        Err(diags) => return fail(diags),
    };
    let domain = compiled.checked.domain();
    let checked = compiled.checked;
    if let Err(diags) = native::capabilities_check(&checked, &args.capabilities) {
        return fail(diags);
    }
    // The source entry is a function NAME. The adapter's def ids are
    // `d_<name>` (identity), while the declared `name` field is the native
    // symbol (`sf_<name>`, QA-02) — so the lookup goes through the def id,
    // and the backend's entry checks own the signature rules.
    let entry_name = args.entry.clone().unwrap_or_default();
    let entry_def_id = format!("d_{}", entry_name);
    let bundle = checked.bundle();
    let exists = bundle
        .modules
        .iter()
        .any(|m| m.defs.iter().any(|d| d.id_text == entry_def_id));
    if !exists {
        return fail(vec![auto_ac_prototype::atom_text::Diagnostic::new(
            auto_ac_prototype::atom_text::Stage::Backend,
            "entry.not-found",
            format!("no function named `{}` in this source module", entry_name),
            auto_ac_prototype::atom_text::Span { start: 0, end: 0 },
        )]);
    }
    let sel = match native::find_entry(&checked, &entry_def_id) {
        Ok(s) => s,
        Err(diags) => return fail(diags),
    };

    // Source metadata travels with the receipt inside the same publish
    // transaction (no side-channel receipt writes): the source profile, the
    // HIR identity triple of the checked module and the input hash; the
    // shared post-Checked build tail does the rest.
    let output = args
        .output
        .clone()
        .unwrap_or_else(|| PathBuf::from("a.exe"));
    let receipt_extra = format!(
        "source-profile: {}\nhir-schema: {}\nhir-revision: {}u32\nhir-profile: {}\nsource-input-hash: {}\n",
        compiled.profile,
        domain.schema(),
        domain.revision(),
        domain.as_str(),
        compiled.input_hash
    );
    let success_extra = format!(
        "source-profile: {}\n  hir-identity: {}/{}/{}\n  source-input-hash: {}",
        compiled.profile,
        domain.schema(),
        domain.revision(),
        domain.as_str(),
        compiled.input_hash
    );
    finish_build(
        &checked,
        &sel,
        &output,
        &args.support_libs,
        &receipt_extra,
        &success_extra,
    )
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

    let bundle = match auto_ac_prototype::bind_document(&src) {
        Ok(b) => b,
        Err(diags) => return fail(diags),
    };
    let checked = match match bundle.domain {
        auto_ac_prototype::hir::HirDomain::Core => verify::verify(bundle),
        auto_ac_prototype::hir::HirDomain::Scalar => verify::verify_scalar(bundle),
        auto_ac_prototype::hir::HirDomain::Text => verify::verify_text(bundle),
    } {
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
    finish_build(&checked, &sel, &args.output, &args.support_libs, "", "")
}

/// The one post-Checked build tail shared by BOTH modes (plan §2/§5.4,
/// QA-08): lowering → staged object → toolchain discovery → staged link →
/// transactional publish. Mode front-ends construct this process' Checked
/// module, resolve the entry, and provide any extra receipt metadata; the
/// staged-object cleanup contract (R3-QA-02) and the publish transaction
/// live only here.
///
/// PLAN-770: a module requiring `ac.runtime.text.v1` additionally passes
/// the runtime contract gate BEFORE any emission or publish — the support
/// library must carry a `<lib>.contract.json` sidecar whose
/// api/revision/target match the lowering's consumption and whose hash is
/// the actual library file's SHA-256. The capability string alone is never
/// trusted; the receipt records the real library and contract fingerprint.
fn finish_build(
    checked: &verify::CheckedModule,
    sel: &native::EntrySelection,
    output: &Path,
    support_libs: &[PathBuf],
    receipt_extra: &str,
    success_extra: &str,
) -> Result<String, (u8, Vec<auto_ac_prototype::atom_text::Diagnostic>)> {
    let fail = |diags: Vec<auto_ac_prototype::atom_text::Diagnostic>| Err((EXIT_PIPELINE, diags));
    let mut receipt_extra = receipt_extra.to_string();
    if checked
        .required_capabilities()
        .iter()
        .any(|c| c == RUNTIME_TEXT_CAPABILITY)
    {
        match check_runtime_contract(support_libs) {
            Ok((lib, contract_sha)) => {
                receipt_extra.push_str(&format!(
                    "runtime-lib: {}\nruntime-contract-sha256: {}\n",
                    lib.display(),
                    contract_sha
                ));
            }
            Err(diags) => return fail(diags),
        }
    }
    let obj_bytes = match native::lower_object(checked, sel) {
        Ok(b) => b,
        Err(diags) => return fail(diags),
    };

    // Artifacts live next to the requested exe; the object is written to a
    // temporary path first so a failed link never leaves a stale .obj pair.
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
        output,
        native::START_SYMBOL,
        support_libs,
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
    // `receipt_extra` joins the receipt inside the same transaction (no
    // side-channel writes); `success_extra` echoes the same metadata on the
    // success line (empty for the HIR mode).
    let receipt =
        match link::publish_artifacts_with_receipt(&staged, &tmp_obj, &obj_path, &receipt_extra) {
            Ok(r) => r,
            Err(d) => return Err((EXIT_PIPELINE, vec![d])),
        };
    Ok(format!(
        "BUILT {} (entry {})\n  obj: {}\n  exe: {}\n  receipt: {}\n  linker: {}{}",
        output.display(),
        sel.symbol,
        receipt.obj.display(),
        receipt.exe.display(),
        receipt.receipt_path.display(),
        receipt.linker.display(),
        if success_extra.is_empty() {
            String::new()
        } else {
            format!("\n  {}", success_extra)
        },
    ))
}

/// Frozen runtime contract identity the text lowering consumes (design.md
/// §5: api `ac.runtime.text.v1`, revision 1, Windows x64 staticlib).
const RUNTIME_CONTRACT_API: &str = "ac.runtime.text.v1";
const RUNTIME_CONTRACT_REVISION: i64 = 1;
const RUNTIME_CONTRACT_TARGET: &str = "x86_64-pc-windows-msvc";

fn contract_diag(code: &str, msg: impl Into<String>) -> auto_ac_prototype::atom_text::Diagnostic {
    auto_ac_prototype::atom_text::Diagnostic::new(
        auto_ac_prototype::atom_text::Stage::Capability,
        code,
        msg,
        auto_ac_prototype::atom_text::Span { start: 0, end: 0 },
    )
}

/// Minimal SHA-256 (FIPS 180-4), mirrored from
/// `runtime/text/tests/contract.rs` (no external deps in this workspace).
fn sha256_hex(data: &[u8]) -> String {
    const K: [u32; 64] = [
        0x428a2f98, 0x71374491, 0xb5c0fbcf, 0xe9b5dba5, 0x3956c25b, 0x59f111f1, 0x923f82a4,
        0xab1c5ed5, 0xd807aa98, 0x12835b01, 0x243185be, 0x550c7dc3, 0x72be5d74, 0x80deb1fe,
        0x9bdc06a7, 0xc19bf174, 0xe49b69c1, 0xefbe4786, 0x0fc19dc6, 0x240ca1cc, 0x2de92c6f,
        0x4a7484aa, 0x5cb0a9dc, 0x76f988da, 0x983e5152, 0xa831c66d, 0xb00327c8, 0xbf597fc7,
        0xc6e00bf3, 0xd5a79147, 0x06ca6351, 0x14292967, 0x27b70a85, 0x2e1b2138, 0x4d2c6dfc,
        0x53380d13, 0x650a7354, 0x766a0abb, 0x81c2c92e, 0x92722c85, 0xa2bfe8a1, 0xa81a664b,
        0xc24b8b70, 0xc76c51a3, 0xd192e819, 0xd6990624, 0xf40e3585, 0x106aa070, 0x19a4c116,
        0x1e376c08, 0x2748774c, 0x34b0bcb5, 0x391c0cb3, 0x4ed8aa4a, 0x5b9cca4f, 0x682e6ff3,
        0x748f82ee, 0x78a5636f, 0x84c87814, 0x8cc70208, 0x90befffa, 0xa4506ceb, 0xbef9a3f7,
        0xc67178f2,
    ];
    let mut h: [u32; 8] = [
        0x6a09e667, 0xbb67ae85, 0x3c6ef372, 0xa54ff53a, 0x510e527f, 0x9b05688c, 0x1f83d9ab,
        0x5be0cd19,
    ];
    let bitlen = (data.len() as u64) * 8;
    let mut msg = data.to_vec();
    msg.push(0x80);
    while msg.len() % 64 != 56 {
        msg.push(0);
    }
    msg.extend_from_slice(&bitlen.to_be_bytes());
    for chunk in msg.chunks(64) {
        let mut w = [0u32; 64];
        for i in 0..16 {
            w[i] = u32::from_be_bytes([
                chunk[i * 4],
                chunk[i * 4 + 1],
                chunk[i * 4 + 2],
                chunk[i * 4 + 3],
            ]);
        }
        for i in 16..64 {
            let s0 = w[i - 15].rotate_right(7) ^ w[i - 15].rotate_right(18) ^ (w[i - 15] >> 3);
            let s1 = w[i - 2].rotate_right(17) ^ w[i - 2].rotate_right(19) ^ (w[i - 2] >> 10);
            w[i] = w[i - 16]
                .wrapping_add(s0)
                .wrapping_add(w[i - 7])
                .wrapping_add(s1);
        }
        let (mut a, mut b, mut c, mut d, mut e, mut f, mut g, mut hh) =
            (h[0], h[1], h[2], h[3], h[4], h[5], h[6], h[7]);
        for i in 0..64 {
            let s1 = e.rotate_right(6) ^ e.rotate_right(11) ^ e.rotate_right(25);
            let ch = (e & f) ^ ((!e) & g);
            let t1 = hh
                .wrapping_add(s1)
                .wrapping_add(ch)
                .wrapping_add(K[i])
                .wrapping_add(w[i]);
            let s0 = a.rotate_right(2) ^ a.rotate_right(13) ^ a.rotate_right(22);
            let maj = (a & b) ^ (a & c) ^ (b & c);
            let t2 = s0.wrapping_add(maj);
            hh = g;
            g = f;
            f = e;
            e = d.wrapping_add(t1);
            d = c;
            c = b;
            b = a;
            a = t1.wrapping_add(t2);
        }
        h[0] = h[0].wrapping_add(a);
        h[1] = h[1].wrapping_add(b);
        h[2] = h[2].wrapping_add(c);
        h[3] = h[3].wrapping_add(d);
        h[4] = h[4].wrapping_add(e);
        h[5] = h[5].wrapping_add(f);
        h[6] = h[6].wrapping_add(g);
        h[7] = h[7].wrapping_add(hh);
    }
    h.iter().map(|x| format!("{:08x}", x)).collect()
}

/// Extract one string field from the fixed-shape contract JSON (written by
/// the runtime sidecar step; this parser does not accept arbitrary JSON).
fn contract_field_str(text: &str, key: &str) -> Option<String> {
    let needle = format!("\"{}\"", key);
    let start = text.find(&needle)? + needle.len();
    let rest = &text[start..];
    let q1 = rest.find('"')?;
    let rest = &rest[q1 + 1..];
    let q2 = rest.find('"')?;
    Some(rest[..q2].to_string())
}

/// Extract one numeric field from the fixed-shape contract JSON.
fn contract_field_num(text: &str, key: &str) -> Option<i64> {
    let needle = format!("\"{}\"", key);
    let start = text.find(&needle)? + needle.len();
    let rest = &text[start..];
    let colon = rest.find(':')?;
    let digits: String = rest[colon + 1..]
        .trim_start()
        .chars()
        .take_while(|c| c.is_ascii_digit() || *c == '-')
        .collect();
    digits.parse().ok()
}

/// The PLAN-770 runtime contract gate: exactly one support library carries
/// a `<file>.contract.json` sidecar, and its api/revision/target fields and
/// library hash must match the real artifact. Returns the library path and
/// the (validated) contract hash for the receipt.
fn check_runtime_contract(
    support_libs: &[PathBuf],
) -> Result<(PathBuf, String), Vec<auto_ac_prototype::atom_text::Diagnostic>> {
    let mut candidates: Vec<(PathBuf, PathBuf)> = Vec::new();
    for lib in support_libs {
        if lib.extension().map_or(false, |e| e == "lib") {
            let fname = lib.file_name().map(|s| s.to_string_lossy().to_string());
            if let Some(fname) = fname {
                let sidecar = lib.with_file_name(format!("{}.contract.json", fname));
                if sidecar.is_file() {
                    candidates.push((lib.clone(), sidecar));
                }
            }
        }
    }
    let (lib, sidecar) = match candidates.len() {
        0 => {
            return Err(vec![contract_diag(
                "runtime.contract-missing",
                "this module requires `ac.runtime.text.v1` but no --support-lib carries a <lib>.contract.json runtime contract sidecar",
            )]);
        }
        1 => candidates.remove(0),
        n => {
            return Err(vec![contract_diag(
                "runtime.contract-missing",
                format!(
                    "{} support libraries carry a runtime contract sidecar; exactly one text runtime is allowed per build",
                    n
                ),
            )])
        }
    };
    let text = std::fs::read_to_string(&sidecar).map_err(|e| {
        vec![contract_diag(
            "runtime.contract-missing",
            format!("cannot read runtime contract {}: {}", sidecar.display(), e),
        )]
    })?;
    let api = contract_field_str(&text, "api").ok_or_else(|| {
        vec![contract_diag(
            "runtime.contract-api",
            format!("runtime contract {} has no `api` field", sidecar.display()),
        )]
    })?;
    if api != RUNTIME_CONTRACT_API {
        return Err(vec![contract_diag(
            "runtime.contract-api",
            format!(
                "runtime contract api `{}` does not match the lowering's consumption `{}`",
                api, RUNTIME_CONTRACT_API
            ),
        )]);
    }
    match contract_field_num(&text, "api_revision") {
        Some(rev) if rev == RUNTIME_CONTRACT_REVISION => {}
        other => {
            return Err(vec![contract_diag(
                "runtime.contract-api",
                format!(
                    "runtime contract api_revision {:?} does not match the expected {}",
                    other, RUNTIME_CONTRACT_REVISION
                ),
            )])
        }
    }
    let target = contract_field_str(&text, "target").ok_or_else(|| {
        vec![contract_diag(
            "runtime.contract-target",
            format!(
                "runtime contract {} has no `target` field",
                sidecar.display()
            ),
        )]
    })?;
    if target != RUNTIME_CONTRACT_TARGET {
        return Err(vec![contract_diag(
            "runtime.contract-target",
            format!(
                "runtime contract target `{}` does not match this build target `{}`",
                target, RUNTIME_CONTRACT_TARGET
            ),
        )]);
    }
    let declared = contract_field_str(&text, "lib_sha256").ok_or_else(|| {
        vec![contract_diag(
            "runtime.contract-hash",
            format!(
                "runtime contract {} has no `lib_sha256` field",
                sidecar.display()
            ),
        )]
    })?;
    let bytes = std::fs::read(&lib).map_err(|e| {
        vec![contract_diag(
            "runtime.contract-hash",
            format!("cannot read runtime library {}: {}", lib.display(), e),
        )]
    })?;
    let actual = sha256_hex(&bytes);
    if declared != actual {
        return Err(vec![contract_diag(
            "runtime.contract-hash",
            format!(
                "runtime contract hash `{}` does not match the actual library SHA-256 `{}`",
                declared, actual
            ),
        )]);
    }
    Ok((lib, declared))
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
                        "usage: auto-ac-prototype check-source <file> --source-profile {}|{}|{} [--capability <name>]...",
                        SOURCE_PROFILE, SCALAR_SOURCE_PROFILE, TEXT_SOURCE_PROFILE
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
                        "usage: auto-ac-prototype build-source <file> --source-profile {}|{}|{} --entry <fn-name> --output <exe> [--capability <name>]... [--support-lib <lib>]...",
                        SOURCE_PROFILE, SCALAR_SOURCE_PROFILE, TEXT_SOURCE_PROFILE
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
