//! PLAN-1045 T-03 conformance: the AAC hosted file/path/process client on
//! the sys lane (aggregates + sys set), executed on the AV executor driving
//! the REAL Win32PlatformIo backend — real Unicode-path file reads with
//! embedded content NULs, the adapter-level NUL-in-OS-string refusal,
//! deterministic directory listing + module location, env round trip, real
//! child processes (exit code / timeout-kill / missing tool) and the Auto
//! replication of the rust-lld discovery. All oracles are computed on the
//! Rust side (std::fs, blake3 crate, std::process) — never the client's
//! own output.

#![cfg(all(windows, target_arch = "x86_64"))]

use ace_aac_host_conformance as lane;

fn sys_case(tag: &str, func: &str, arg: Option<&str>) -> String {
    let main_src = match arg {
        Some(a) => {
            format!("use app.host_files.{func}\nfn main() Str {{\n    return {func}(\"{a}\")\n}}\n")
        }
        None => format!("use app.host_files.{func}\nfn main() Str {{\n    return {func}()\n}}\n"),
    };
    let bytes = lane::run_sys_case(tag, &main_src).unwrap_or_else(|e| panic!("{tag}: {e}"));
    String::from_utf8(bytes).unwrap_or_else(|e| panic!("{tag}: non-utf8 observable: {e}"))
}

/// The T-03 fixture root: the module set (identical pair + near-miss +
/// empty), the "a" file for the NUL-truncation proof, and the empty dir.
fn fixture_root(tag: &str) -> std::path::PathBuf {
    let root = lane::scratch(&format!("t03-{tag}"));
    let main_content = "auto 模块 a\0中 b";
    std::fs::write(root.join("模块-α.at"), main_content).unwrap();
    std::fs::write(root.join("same.at"), main_content).unwrap();
    std::fs::write(root.join("near.at"), "auto 模块 a\0中 c").unwrap();
    std::fs::write(root.join("empty.at"), b"").unwrap();
    std::fs::write(root.join("a"), b"x").unwrap();
    std::fs::create_dir_all(root.join("leer")).unwrap();
    root
}

fn forward_slashes(p: &std::path::Path) -> String {
    p.to_string_lossy().replace('\\', "/")
}

/// V-01 module-read leg: a module under a non-ASCII path keeps its embedded
/// content NUL as content (length + hash exact vs the Rust-side read of the
/// SAME file), identical modules intern together, the near-miss does not,
/// and the empty file interns as its own slot.
#[test]
fn v01_module_read() {
    let root = fixture_root("v01");
    let got = sys_case(
        "v01-module",
        "module_read_report",
        Some(&forward_slashes(&root)),
    );
    let main_bytes = std::fs::read(root.join("模块-α.at")).unwrap();
    let nul = main_bytes.iter().filter(|b| **b == 0).count();
    let dg = blake3::hash(&main_bytes).to_hex().to_string();
    let want = format!("0|main:{},{nul},{dg},0,1,2|3", main_bytes.len());
    assert_eq!(got, want);
}

/// V-01 adapter leg: a path containing U+0000 is refused BY NAME — and the
/// refusal is proven NOT a truncation: the root really contains a file
/// named "a", so a truncated "a\0b" → "a" would have been FOUND.
#[test]
fn v01_nul_path_refused() {
    let root = fixture_root("nul");
    let got = sys_case("nul-path", "nul_path_report", Some(&forward_slashes(&root)));
    assert_eq!(got, "none,false");
}

/// T-03 directory/module location: deterministic byte-lex listing (Rust
/// `Ord` over the same entries), the module is located under root and NOT
/// under a missing sibling, and the empty dir lists zero names.
#[test]
fn t03_dir_locate() {
    let root = fixture_root("dir");
    let got = sys_case(
        "dir-locate",
        "dir_locate_report",
        Some(&forward_slashes(&root)),
    );
    let mut names: Vec<String> = std::fs::read_dir(&root)
        .unwrap()
        .flatten()
        .map(|e| e.file_name().to_string_lossy().into_owned())
        .collect();
    names.sort();
    let want = format!("present|{}|n:0", names.join(","));
    assert_eq!(got, want);
}

/// T-03 env/target: set→get equals (Unicode value), remove→get_or default,
/// home_dir non-empty. The report pins each leg's own status code.
#[test]
fn t03_env_target() {
    let got = sys_case("env-target", "env_target_report", None);
    assert_eq!(got, "0,1,0,1,1");
}

/// T-03 process assembly: real cmd.exe child with exit code 7; the ping
/// child far over its 800 ms budget is timeout-KILLED (-11, controlled
/// reclaim); the missing tool is an explicit -1 refusal, never a hang.
#[test]
fn t03_process() {
    let got = sys_case("process", "process_report", None);
    assert_eq!(got, "7,-11,-1");
}

/// T-03 linker discovery: the Auto replication must agree with the Rust
/// `find_rust_lld` algorithm byte-for-byte on the discovered sysroot and
/// the existence verdict of rust-lld.exe.
#[test]
fn t03_linker_discovery() {
    let got = sys_case("linker", "linker_report", None);
    let sysroot = std::process::Command::new("rustc")
        .arg("--print")
        .arg("sysroot")
        .output()
        .expect("rustc --print sysroot");
    if !sysroot.status.success() {
        assert_eq!(got, "|rustc-missing");
        return;
    }
    let sr = String::from_utf8_lossy(&sysroot.stdout).trim().to_string();
    let lld = std::path::Path::new(&sr).join("lib/rustlib/x86_64-pc-windows-msvc/bin/rust-lld.exe");
    let verdict = if lld.is_file() { "found" } else { "not-found" };
    assert_eq!(got, format!("{sr}|{verdict}"));
}

// ---------------------------------------------------------------------------
// T-04: failure & publish primitives (host_publish.at)
// ---------------------------------------------------------------------------

fn publish_case(tag: &str, func: &str, root: &str) -> String {
    let main_src = format!(
        "use app.host_publish.{func}\nfn main() Str {{\n    return {func}(\"{root}\")\n}}\n"
    );
    let bytes = lane::run_sys_case(tag, &main_src).unwrap_or_else(|e| panic!("{tag}: {e}"));
    String::from_utf8(bytes).unwrap_or_else(|e| panic!("{tag}: non-utf8 observable: {e}"))
}

fn publish_root(tag: &str) -> std::path::PathBuf {
    let root = lane::scratch(&format!("t04-{tag}"));
    std::fs::create_dir_all(root.join("adir")).unwrap();
    root
}

/// V-02 publish leg: after a published artifact exists, BOTH injected
/// failures (replace onto an existing directory; replace from a missing
/// source) return negative OS-failure codes, the old artifact's hash is
/// unchanged byte-exactly, the committed retry publishes content C2 and
/// removes the temp file. Hashes are recomputed on the Rust side.
#[test]
fn v02_publish_preserves_old() {
    let root = publish_root("pub");
    let got = publish_case("t04-pub", "publish_report", &forward_slashes(&root));
    let h1 = blake3::hash(b"artifact-v1 \xe5\x86\x85\xe5\xae\xb9 \xce\xb1\0\xe5\xb0\xbe")
        .to_hex()
        .to_string();
    let h2 = blake3::hash(b"artifact-v2 \xe5\x86\x85\xe5\xae\xb9 \xce\xb2\0\xe5\xb0\xbe")
        .to_hex()
        .to_string();
    let want = format!("0,0,NEG,NEG,1,0,1|{h1}|{h2}");
    // the codes are OS-specific negatives; assert the structure explicitly
    let parts: Vec<&str> = got.split('|').collect();
    assert_eq!(parts.len(), 3, "publish report shape: {got}");
    assert_eq!(parts[1], h1, "old-artifact hash h1");
    assert_eq!(parts[2], h2, "published hash h2");
    let nums: Vec<i64> = parts[0].split(',').map(|n| n.parse().unwrap()).collect();
    assert_eq!(nums[0], 0, "write C1 ok");
    assert_eq!(nums[1], 0, "write tmp1 ok");
    assert!(
        nums[2] < 0,
        "inject-1 (onto dir) must fail, got {}",
        nums[2]
    );
    assert!(
        nums[3] < 0,
        "inject-2 (missing src) must fail, got {}",
        nums[3]
    );
    assert_eq!(nums[4], 1, "old artifact preserved byte-exactly");
    assert_eq!(nums[5], 0, "committed retry publishes");
    assert_eq!(nums[6], 1, "temp removed by the commit");
    let _ = want;
}

/// T-04 classified IO failures: the missing-dir write and the NUL-path
/// write are both negative (distinct failure families, never success);
/// the over-EOF read clamps; three children all exit 0 (balanced reaping);
/// fifty sequential reads stay byte-stable (no handle leak).
#[test]
fn t04_io_failure_classification() {
    let root = publish_root("io");
    let got = publish_case("t04-io", "io_failure_report", &forward_slashes(&root));
    let nums: Vec<i64> = got.split(',').map(|n| n.parse().unwrap()).collect();
    assert_eq!(nums.len(), 8, "io report shape: {got}");
    assert_eq!(nums[0], 0, "base write ok");
    assert!(nums[1] < 0, "missing-dir write must fail, got {}", nums[1]);
    assert!(
        nums[2] < 0,
        "NUL-path write must be refused, got {}",
        nums[2]
    );
    assert_eq!(nums[3], 1, "over-EOF read clamps to tail");
    assert_eq!(nums[4], 0, "child 1 reaped");
    assert_eq!(nums[5], 0, "child 2 reaped");
    assert_eq!(nums[6], 0, "child 3 reaped");
    assert_eq!(nums[7], 1, "fifty sequential reads byte-stable");
}
