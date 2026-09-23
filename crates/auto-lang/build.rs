// Register custom cfg values so rustc doesn't warn about them being unexpected.
//
// `feature = "interpreter"` guards legacy converter implementations
// (convert_node_dynamic, etc.) that have been superseded by newer code.
// The feature is intentionally never defined, so the guarded code stays
// disabled — but we register it here to silence `unexpected_cfgs` warnings
// rather than leaving readers to wonder whether it's a typo.
//
// Plan 698 SD-01: stamp a content fingerprint of the UI generator sources
// (src/ui_gen/**) as AUTO_UI_GEN_FINGERPRINT. UICache keys its entries on
// it, so a generator upgrade invalidates stale outputs instead of replaying
// them as "fresh" (692 W-1: fixed generator + unchanged source hashes kept
// serving old artifacts).
use std::fs;
use std::path::Path;

fn hash_ui_gen_sources() -> u64 {
    fn walk(dir: &Path, rel: &str, out: &mut Vec<(String, Vec<u8>)>) {
        let Ok(entries) = fs::read_dir(dir) else {
            return;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            let name = entry.file_name().to_string_lossy().to_string();
            let child_rel = if rel.is_empty() {
                name
            } else {
                format!("{}/{}", rel, name)
            };
            if path.is_dir() {
                walk(&path, &child_rel, out);
            } else if let Ok(content) = fs::read(&path) {
                out.push((child_rel, content));
            }
        }
    }

    let mut files = Vec::new();
    walk(Path::new("src/ui_gen"), "", &mut files);
    files.sort();

    // FNV-1a over "path:len\xFFcontent" per file — relative paths only, so
    // the value is stable across machines and checkouts.
    let mut hash: u64 = 0xcbf29ce484222325;
    for (rel, content) in &files {
        for b in format!("{}:{}\u{FF}", rel, content.len()).bytes() {
            hash ^= b as u64;
            hash = hash.wrapping_mul(0x100000001b3);
        }
        for b in content {
            hash ^= *b as u64;
            hash = hash.wrapping_mul(0x100000001b3);
        }
    }
    hash
}

fn main() {
    println!("cargo::rustc-check-cfg=cfg(feature, values(\"interpreter\"))");
    println!("cargo:rerun-if-changed=src/ui_gen");
    println!(
        "cargo:rustc-env=AUTO_UI_GEN_FINGERPRINT={:016x}",
        hash_ui_gen_sources()
    );
}
