//! PLAN-1098 T-02: warm the content-bound native runtime builders ONCE during
//! explicit preparation (``fixed-tests.py --prepare-only``), so the daily run
//! phase performs no cargo builds.
//!
//! This binary only calls the real builders; their own identity/stamp logic
//! decides whether a build is needed and validates any reused library. It is a
//! preparation entry, never a way to skip those checks.

fn run(label: &str, prepared: Result<(u64, std::path::PathBuf), String>) -> bool {
    match prepared {
        Ok((builds, library)) => {
            println!("{label}: builds={builds} library={}", library.display());
            true
        }
        Err(message) => {
            eprintln!("{label}: {message}");
            false
        }
    }
}

fn main() {
    let collections = run("collections", {
        ace_native::collections_runtime::prepare(None)
            .map(|p| (p.cargo_invocations, p.library))
            .map_err(|e| e.message())
    });
    let sched = run("sched", {
        ace_native::sched_runtime::prepare()
            .map(|p| (p.cargo_invocations, p.library))
            .map_err(|e| e.message())
    });
    let typed = run("typed", {
        ace_native::typed_runtime::prepare(None)
            .map(|p| (p.cargo_invocations, p.library))
            .map_err(|e| e.message())
    });
    if !(collections && sched && typed) {
        std::process::exit(1);
    }
}