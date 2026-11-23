//! PLAN-1123: prepare the AV/Rust stdlib provider closures — the whole
//! delivered `stdlib/ace/avrs` unit and the secure-transport selection
//! (`str` + `tls`) — into the provider root: this executable's own directory, or the directory the
//! harness forwards as `ACE_AVRS_TLS_PROVIDER_ROOT`.
//!
//! The dependency closure of the secure transport (rustls + ring + the
//! native root store) is built HERE, once per (source, toolchain,
//! resolution) identity. The daily lane never compiles: it opens a prepared
//! closure through `ace_abi::rust_provider::open_prepared_avrs_session` /
//! M2Build's `RustProjectInputs::prepared`, which verify the closure
//! against the delivered sources and fail with the named `prepared.*`
//! error when it is missing or stale.

use std::process::ExitCode;
use std::time::Instant;

use ace_abi::rust_provider::{
    prepared_provider_root, AVRS_STDLIB_CLOSURE_DIR, AVRS_TLS_CLOSURE_DIR, PROVIDER_CLOSURE_SCHEMA,
};

fn main() -> ExitCode {
    let started = Instant::now();
    let prepared = prepared_provider_root().and_then(|root| {
        let whole = ace_abi::rust_provider::delivered_avrs_provider()?;
        let tls = ace_abi::rust_provider::compose_delivered_avrs(
            &ace_abi::rust_provider::AVRS_TLS_SELECTION,
        )?;
        let mut ready = Vec::new();
        for (directory, source) in [
            (AVRS_STDLIB_CLOSURE_DIR, &whole),
            (AVRS_TLS_CLOSURE_DIR, &tls),
        ] {
            let closure =
                ace_abi::rust_provider::prepare_provider_closure(&root.join(directory), source)?;
            ready.push(format!(
                "{} (provider {}, deps [{}])",
                closure.root.display(),
                closure.provider_digest,
                dependencies(source)
            ));
        }
        Ok(ready)
    });
    match prepared {
        Ok(ready) => {
            for line in ready {
                println!("avrs provider closure ready: {line} ({PROVIDER_CLOSURE_SCHEMA})");
            }
            println!(
                "avrs provider closures prepared in {} ms",
                started.elapsed().as_millis()
            );
            ExitCode::SUCCESS
        }
        Err(error) => {
            eprintln!("avrs provider preparation failed: {error}");
            ExitCode::FAILURE
        }
    }
}

/// The declared dependencies of one provider source, for the receipt line.
fn dependencies(source: &str) -> String {
    ace_abi::rust_provider::provider_dep_specs(source)
        .map(|deps| {
            deps.iter()
                .map(|dep| {
                    let features = if dep.features.is_empty() {
                        String::new()
                    } else {
                        format!(" [{}]", dep.features.join(", "))
                    };
                    format!("{} {}{features}", dep.name, dep.version)
                })
                .collect::<Vec<_>>()
                .join("; ")
        })
        .unwrap_or_default()
}
