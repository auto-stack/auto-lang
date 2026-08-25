fn main() {
    // Set a large stack for the main thread on Windows.
    // The parser uses deep recursion that overflows Windows' 1MB default thread
    // stack. 4MB covered small/medium files; large real-world .at modules
    // (e.g. auto-musk specs.at, ~1100 lines) need ~16MB+. Default raised to
    // 64MB (virtual reservation only — physical memory is committed on demand).
    // Linux/macOS default to an 8MB main-thread stack, so no flag is needed there.
    // NOTE: gate on target_os = "windows" — target_env = "gnu" is also true on
    // Linux (x86_64-unknown-linux-gnu), and `-Wl,--stack` is a Windows/MinGW flag
    // that Linux's linker rejects.
    let stack_size = option_env!("AUTO_STACK_SIZE")
        .or(option_env!("auto_stack_size"))
        .and_then(|s| s.parse::<usize>().ok())
        .unwrap_or(64 * 1024 * 1024);

    #[cfg(all(target_os = "windows", target_env = "msvc"))]
    println!("cargo:rustc-link-arg=/STACK:{}", stack_size);

    #[cfg(all(target_os = "windows", target_env = "gnu"))]
    println!("cargo:rustc-link-arg=-Wl,--stack,{}", stack_size);

    stamp_git_version();
}

// Plan 448 follow-up: stamp the binary with the git commit it was built
// from, so stale PATH installs are distinguishable via `auto --version`
// (the 002-counter incident: two `auto 0.1.0` binaries, one pre-merge —
// indistinguishable until you diff their generated output).
//
// Injects `AUTO_BUILD_GIT_VERSION` = `git describe --always --dirty=-dirty`
// (short hash when the repo has no tags; `-dirty` suffix when the working
// tree had uncommitted changes — exactly the WIP-binary vs clean-binary
// distinction that matters here). Falls back to `unknown` when git is not
// available (e.g. exported source trees).
fn stamp_git_version() {
    use std::process::Command;

    // Re-run when the checked-out commit moves so the stamp never lags:
    // track the git dir's HEAD plus the loose ref it points at (commits on
    // the same branch rewrite the ref file, not HEAD itself). Worktrees
    // resolve the real git dir via rev-parse; packed-refs is the fallback
    // for repos whose refs are packed.
    if let Ok(out) = Command::new("git")
        .args(["rev-parse", "--git-dir"])
        .output()
    {
        if out.status.success() {
            let git_dir = String::from_utf8_lossy(&out.stdout).trim().to_string();
            if !git_dir.is_empty() {
                let head_path = format!("{}/HEAD", git_dir);
                if std::path::Path::new(&head_path).exists() {
                    println!("cargo:rerun-if-changed={}", head_path);
                }
                if let Ok(head) = std::fs::read_to_string(&head_path) {
                    if let Some(ref_path) = head.trim().strip_prefix("ref: ") {
                        let loose = format!("{}/{}", git_dir, ref_path);
                        if std::path::Path::new(&loose).exists() {
                            println!("cargo:rerun-if-changed={}", loose);
                        } else {
                            let packed = format!("{}/packed-refs", git_dir);
                            if std::path::Path::new(&packed).exists() {
                                println!("cargo:rerun-if-changed={}", packed);
                            }
                        }
                    }
                }
            }
        }
    }

    let mut stamp = String::from("unknown");
    if let Ok(out) = Command::new("git")
        .args(["describe", "--always", "--dirty=-dirty"])
        .output()
    {
        if out.status.success() {
            let s = String::from_utf8_lossy(&out.stdout).trim().to_string();
            if !s.is_empty() {
                stamp = s;
            }
        }
    }
    println!("cargo:rustc-env=AUTO_BUILD_GIT_VERSION={}", stamp);
}
