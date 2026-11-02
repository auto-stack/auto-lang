//! `a2r-emit-917-cert` — PLAN-917 T-04 explicit certification for the legacy
//! A2R mut-record (R07) and text-slice (T08) evidence domain
//! (`ace.m1.a2r-emit917.cert/1`). Explicit, independent, never invoked by the
//! suite runner or any daily scope.
//!
//! Usage:
//!   a2r-emit-917-cert --case-dir <crates/auto-lang/test/cases/ace-m1/a2r-emit-917>
//!                     [--certs-dir <dir>] [--only <id> ...]
//!                     [--rustc <path>] [--scratch <dir>]

fn main() {
    let mut args = auto_lang::test_cases::a2r_emit_917_cert::CertifyArgs {
        case_dir: std::path::PathBuf::new(),
        certs_dir: std::path::PathBuf::new(),
        only: Vec::new(),
        rustc: None,
        scratch: None,
    };
    let mut have_case_dir = false;
    let mut have_certs_dir = false;
    let mut it = std::env::args().skip(1);
    while let Some(flag) = it.next() {
        let mut need = |flag: &str| -> String {
            it.next()
                .unwrap_or_else(|| panic!("a2r-emit-917-cert: {flag} needs a value"))
        };
        match flag.as_str() {
            "--case-dir" => {
                args.case_dir = std::path::PathBuf::from(need("--case-dir"));
                have_case_dir = true;
            }
            "--certs-dir" => {
                args.certs_dir = std::path::PathBuf::from(need("--certs-dir"));
                have_certs_dir = true;
            }
            "--only" => args.only.push(need("--only")),
            "--rustc" => args.rustc = Some(need("--rustc")),
            "--scratch" => args.scratch = Some(std::path::PathBuf::from(need("--scratch"))),
            other => {
                // Also accept `--only=<id>` style.
                if let Some(v) = other.strip_prefix("--only=") {
                    args.only.push(v.to_string());
                } else {
                    eprintln!("a2r-emit-917-cert: unknown argument {other:?}");
                    std::process::exit(2);
                }
            }
        }
    }
    if !have_case_dir {
        // Default: the in-repo case dir, resolved from the workspace root that
        // contains this binary's manifest.
        let manifest = env!("CARGO_MANIFEST_DIR");
        args.case_dir = std::path::PathBuf::from(manifest).join("test/cases/ace-m1/a2r-emit-917");
    }
    if !have_certs_dir {
        args.certs_dir = args.case_dir.join("certs");
    }
    std::process::exit(auto_lang::test_cases::a2r_emit_917_cert::certify_and_report(&args));
}
