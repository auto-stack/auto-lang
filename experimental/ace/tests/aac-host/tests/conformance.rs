//! PLAN-1045 T-02 conformance: the AAC hosted pure-compute client on the
//! assembled aggregates lane — AV legs assert the FULL string observables
//! byte-exact against independent oracles; the native AC (mn) leg links a
//! real PE and asserts its exit code equals the FNV reduction recomputed
//! over the independently derived expected reports.

use ace_aac_host_conformance as lane;

fn case(tag: &str, func: &str) -> String {
    let bytes = lane::run_case(
        tag,
        &format!("app.host_client.{func}"),
        &format!("{func}()"),
    )
    .unwrap_or_else(|e| panic!("{tag}: {e}"));
    String::from_utf8(bytes).unwrap_or_else(|e| panic!("{tag}: non-utf8 observable: {e}"))
}

/// V-04 bytes/spans: embedded NUL is content (3 bytes [0x61,0x00,0x62] for
/// "a\0b", decimal 97,0,98); "α"'s byte span is [0,2); "a中b"'s 中 span is
/// [1,4) — length strings, no C-string truncation anywhere.
#[test]
fn v04_bytes_span() {
    let got = case("v04-bytes", "bytes_span_report");
    assert_eq!(got, "3,3,97,0,98|206,177,0,2|5,3,97,228,184,173,98,1,4");
}

/// V-04 map: a→1, b→2, a→3 keeps exactly two entries, last writer wins
/// (a=3), and the report is key-sorted — no random iteration order.
#[test]
fn v04_map() {
    let got = case("v04-map", "map_report");
    assert_eq!(got, "001|2,a=3,b=2");
}

/// V-01 compute leg: intern by exact bytes — "a\0b" ≠ "ab" (NUL is
/// content), repeats collapse to first-seen indices, distinct = 6 of 9.
#[test]
fn v01_intern() {
    let got = case("v01-intern", "intern_report");
    assert_eq!(got, "0,1,2,3,1,0,4,3,5|6,11");
}

/// Deterministic byte-lexicographic sort — the same total order Rust's
/// `Ord` gives over the identical inputs (incl. "" and non-ASCII).
#[test]
fn sort_deterministic() {
    let got = case("sort", "sort_report");
    let mut want = vec!["b", "a", "α", "a\0b", "", "中", "ab", "b"];
    want.sort();
    assert_eq!(got, want.join(","));
}

/// Precise integers: the wrap32 FNV reduction, exact parse/format round
/// trips incl. i32::MIN, refusal on "9x", and i64::MAX+1 classified as
/// overflow (not a wrap, not a trap) while 5+7 stays exact.
#[test]
fn int_precision() {
    let got = case("int", "int_report");
    let h = (lane::fnv32(b"a\0b") % 1_000_000_000) as i32;
    assert_eq!(got, format!("{h}|2147483647,-2147483648,ref1|ovf1,12"));
}

/// CS-203 budget: the same arena walk under a sufficient budget reaches
/// all 7 nodes, and under a 3-step budget is REFUSED by name — a distinct
/// classified outcome, not a partial silent result.
#[test]
fn budget_refusal() {
    let got = case("budget", "budget_report");
    assert_eq!(got, "ok:7,refused");
}

/// CS-205 consumption: sha256/blake3 from the frozen aac-support package
/// over content with embedded NUL and non-ASCII — byte-exact against the
/// independent sha2/blake3 crates.
#[test]
fn digest_consumption() {
    let got = case("digest", "digest_report");
    use sha2::{Digest, Sha256};
    let sha = |b: &[u8]| {
        let mut h = Sha256::new();
        h.update(b);
        h.finalize()
    };
    let hx = |bs: &[u8]| -> String { bs.iter().map(|b| format!("{b:02x}")).collect() };
    assert_eq!(
        got,
        format!(
            "{},{},{}",
            hx(&sha(b"abc")),
            hx(&sha(b"a\0b")),
            hx(blake3::hash("α".as_bytes()).as_bytes())
        )
    );
}

/// PLAN-1045 T-02 backend slice: the typed I64 `^ & |` emission on the
/// collections mn lane (collections_lower.rs) — a minimal single-source
/// program asserts all three ops' result as the exit code against the
/// Rust-computed expected value (the same emission the full client's
/// digest consumption exercises).
#[test]
#[cfg(all(windows, target_arch = "x86_64"))]
fn native_bitops_emission() {
    if !tools_available() {
        eprintln!("SKIP native_bitops_emission: rust-lld / SDK um dir not found");
        return;
    }
    let src = "fn main() int {\n    let x = (123456789i64 ^ 987654321i64) & 4294967295i64\n    let z = x | 1i64\n    return (z % 2147483648i64).to(i32)\n}\n";
    let files = vec![("main.at".to_string(), src.to_string())];
    let product = lane::product_from_files("native-bitops", &files, "app::main")
        .unwrap_or_else(|e| panic!("native bitops compile: {e}"));
    let dir = lane::scratch("mn-native-bitops");
    let mut driver = ace_native::driver::MnDriver::ephemeral();
    let built = driver
        .build_collections_checked(
            &product.compiled.checked,
            "d_main",
            &dir,
            &product.bound.input_hash,
            1045,
        )
        .unwrap_or_else(|e| panic!("native bitops mn build: {e:?}"));
    let out = std::process::Command::new(&built.exe_path)
        .current_dir(&dir)
        .output()
        .expect("run bitops exe");
    let want = (((123456789u64 ^ 987654321u64) & 0xFFFF_FFFF) | 1) % (1u64 << 31);
    assert_eq!(
        out.status.code(),
        Some(want as i32),
        "bitops exe {} exit != expected {want}",
        built.exe_path.display()
    );
}

/// The reduction oracle discriminates: flipping ONE byte of ONE expected
/// report must change the combined exit code (pre-checked on the Rust
/// side so the native leg's equality assertion is a real oracle).
#[test]
fn combine_reduction_discriminates() {
    let reports = lane::expected_reports();
    let refs: Vec<&str> = reports.iter().map(|s| s.as_str()).collect();
    let base = lane::combine(&refs);
    for i in 0..refs.len() {
        let mut mutated = reports.clone();
        // flip the first byte (or append when empty) of report i
        if mutated[i].is_empty() {
            mutated[i].push('x');
        } else {
            let b = mutated[i].as_bytes();
            let flip = if b[0] == b'0' { '1' } else { '0' };
            mutated[i].replace_range(0..1, &flip.to_string());
        }
        let mrefs: Vec<&str> = mutated.iter().map(|s| s.as_str()).collect();
        assert_ne!(
            lane::combine(&mrefs),
            base,
            "mutating report {i} must change the reduction"
        );
    }
}

/// The real native AC (mn) leg: the SAME checked product lowers through
/// shared MIR, emits per-function COFF, links a real PE and runs — the
/// exe exit code equals the reduction over the INDEPENDENTLY derived
/// expected reports (Windows x64 hosted; needs lld + the SDK um dir, the
/// same toolchain gate the ac-native cli cases use).
#[test]
#[cfg(all(windows, target_arch = "x86_64"))]
fn native_exe_reduction() {
    if !tools_available() {
        eprintln!("SKIP native_exe_reduction: rust-lld / SDK um dir not found");
        return;
    }
    let (code, exe) =
        lane::run_native_combined("native-combined").unwrap_or_else(|e| panic!("native leg: {e}"));
    let reports = lane::expected_reports();
    let refs: Vec<&str> = reports.iter().map(|s| s.as_str()).collect();
    let want = lane::combine(&refs);
    assert_eq!(
        code,
        want,
        "exe {} exit {code} != expected {want}",
        exe.display()
    );
}

#[cfg(all(windows, target_arch = "x86_64"))]
fn tools_available() -> bool {
    ace_core::link::find_rust_lld().is_ok() && find_sdk_um_dir().is_some()
}

#[cfg(all(windows, target_arch = "x86_64"))]
fn find_sdk_um_dir() -> Option<std::path::PathBuf> {
    // Mirrors crates/native/tests/common/mod.rs (the ac-native cli gate):
    // locate the Windows SDK um directory the PE link needs.
    let candidates = [
        r"C:\Program Files (x86)\Windows Kits\10\Lib",
        r"C:\Program Files\Windows Kits\10\Lib",
    ];
    for root in candidates {
        let rd = std::fs::read_dir(root).ok()?;
        let first = rd.flatten().next()?.path();
        let um = first.join("um");
        if um.is_dir() {
            return Some(um);
        }
    }
    None
}
