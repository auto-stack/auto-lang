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
    // Capture the producer/compiler build closure. Cargo.lock identifies
    // external dependency versions/checksums; local crate source changes must
    // also invalidate generated consumers, including helpers outside a core
    // module's hand-maintained list.
    fn collect(dir: &Path, prefix: &str, files: &mut Vec<(String, u64)>) {
        let mut entries: Vec<_> = fs::read_dir(dir).unwrap().map(Result::unwrap).collect();
        entries.sort_by_key(|entry| entry.file_name());
        for entry in entries {
            let path = entry.path();
            if entry.file_type().unwrap().is_symlink() {
                continue;
            }
            let name = entry.file_name().to_string_lossy().into_owned();
            if matches!(name.as_str(), "target" | ".git" | "node_modules") {
                continue;
            }
            let id = format!("{prefix}/{name}");
            if path.is_dir() {
                collect(&path, &id, files);
            } else if matches!(
                path.extension().and_then(|ext| ext.to_str()),
                Some("rs" | "toml" | "lock")
            ) {
                println!("cargo:rerun-if-changed={}", path.display());
                let mut hash = 0xcbf29ce484222325u64;
                for byte in fs::read(&path).unwrap() {
                    hash = (hash ^ byte as u64).wrapping_mul(0x100000001b3);
                }
                files.push((id, hash));
            }
        }
    }
    let root = Path::new("../..");
    let mut inputs = Vec::new();
    collect(&root.join("crates"), "crates", &mut inputs);
    for name in ["Cargo.toml", "Cargo.lock"] {
        let path = root.join(name);
        println!("cargo:rerun-if-changed={}", path.display());
        let mut hash = 0xcbf29ce484222325u64;
        for byte in fs::read(path).unwrap() {
            hash = (hash ^ byte as u64).wrapping_mul(0x100000001b3);
        }
        inputs.push((name.into(), hash));
    }
    let down = std::env::var_os("AUTO_DOWN_ROOT")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|| root.join("../auto-down"));
    if down.join("Cargo.toml").is_file() {
        collect(&down, "dependency/auto-down", &mut inputs);
    }
    inputs.sort();
    let serialized = inputs
        .iter()
        .map(|(id, hash)| format!("({id:?}, {hash}u64),"))
        .collect::<Vec<_>>()
        .join("\n");
    let out = std::path::PathBuf::from(std::env::var_os("OUT_DIR").unwrap());
    fs::write(
        out.join("assembly_build_inputs.rs"),
        format!("const BUILD_INPUTS: &[(&str, u64)] = &[\n{serialized}\n];"),
    )
    .unwrap();
    let mut features: Vec<_> = std::env::vars()
        .filter_map(|(name, _)| {
            name.strip_prefix("CARGO_FEATURE_")
                .map(|feature| feature.to_lowercase().replace('_', "-"))
        })
        .collect();
    features.sort();
    println!(
        "cargo:rustc-env=AUTO_ASSEMBLY_FEATURES={}",
        features.join(",")
    );
    println!(
        "cargo:rustc-env=AUTO_ASSEMBLY_TARGET={}",
        std::env::var("TARGET").unwrap()
    );
    println!("cargo::rustc-check-cfg=cfg(feature, values(\"interpreter\"))");
    println!("cargo:rerun-if-changed=src/ui_gen");
    println!(
        "cargo:rustc-env=AUTO_UI_GEN_FINGERPRINT={:016x}",
        hash_ui_gen_sources()
    );
}
