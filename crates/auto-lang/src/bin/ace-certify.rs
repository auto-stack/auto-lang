//! `ace-certify` — PLAN-852 T-04 explicit independent a2r certification
//! entry. This is the ONLY tool that compiles/runs Rust for the ace-m1
//! corpus; the suite runner and every daily scope never touch it. Empty stub
//! without `--features test-cases`.
//!
//! ```text
//! ace-certify --corpus <crates/auto-lang/test/cases/ace-m1>
//!             [--certs <dir>]        (default <corpus>/oracle/certs)
//!             [--only b01,b02,...]   (default: every ok-emit row)
//!             [--force]              (re-certify even when axes are unchanged)
//!             [--rustc <path>]       (default $RUSTC or `rustc`)
//! ```

#[cfg(not(feature = "test-cases"))]
fn main() {
    eprintln!("ace-certify: this binary needs `--features test-cases` (empty stub build)");
}

#[cfg(feature = "test-cases")]
fn main() {
    use auto_lang::test_cases::ace_certify::{certify_and_report, CertifyArgs};
    use std::path::PathBuf;

    let mut corpus: Option<PathBuf> = None;
    let mut certs: Option<PathBuf> = None;
    let mut only: Vec<String> = Vec::new();
    let mut force = false;
    let mut rustc = std::env::var("RUSTC").unwrap_or_else(|_| "rustc".to_string());

    let args: Vec<String> = std::env::args().skip(1).collect();
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--corpus" => {
                i += 1;
                corpus = Some(PathBuf::from(args.get(i).ok_or("--corpus needs a value").unwrap()));
            }
            "--certs" => {
                i += 1;
                certs = Some(PathBuf::from(args.get(i).ok_or("--certs needs a value").unwrap()));
            }
            "--only" => {
                i += 1;
                let list = args.get(i).ok_or("--only needs a value").unwrap();
                only = list.split(',').map(|s| s.trim().to_lowercase()).filter(|s| !s.is_empty()).collect();
            }
            "--force" => force = true,
            "--rustc" => {
                i += 1;
                rustc = args.get(i).ok_or("--rustc needs a value").unwrap().clone();
            }
            other => {
                eprintln!("ace-certify: unknown argument {other:?}");
                std::process::exit(2);
            }
        }
        i += 1;
    }

    let corpus = match corpus {
        Some(c) => c,
        None => {
            eprintln!("ace-certify: --corpus is required");
            std::process::exit(2);
        }
    };
    let certs_dir = certs.unwrap_or_else(|| corpus.join("oracle").join("certs"));

    std::process::exit(certify_and_report(&CertifyArgs {
        corpus,
        certs_dir,
        only,
        force,
        rustc,
    }));
}
