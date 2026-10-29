//! PLAN-871 T-06: the PLAN-770 text-runtime CONTRACT gate, moved out of the
//! retiring `main.rs` build face into the library so the `ac-native` CLI can
//! enforce it on the paths that still take an explicit support library.
//!
//! Frozen identity (design.md §5): api `ac.runtime.text.v1`, revision 1,
//! Windows x64 staticlib. Exactly one support library must carry a
//! `<lib>.contract.json` sidecar whose api/revision/target match this
//! compiler's consumption and whose `lib_sha256` equals the real file's
//! SHA-256 — the capability string alone is never trusted.

use std::path::PathBuf;

const RUNTIME_CONTRACT_API: &str = "ac.runtime.text.v1";
const RUNTIME_CONTRACT_REVISION: i64 = 1;
const RUNTIME_CONTRACT_TARGET: &str = "x86_64-pc-windows-msvc";

fn contract_diag(code: &str, msg: impl Into<String>) -> crate::atom_text::Diagnostic {
    crate::atom_text::Diagnostic::new(
        crate::atom_text::Stage::Capability,
        code,
        msg,
        crate::atom_text::Span { start: 0, end: 0 },
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
pub fn check(
    support_libs: &[PathBuf],
) -> Result<(PathBuf, String), Vec<crate::atom_text::Diagnostic>> {
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
