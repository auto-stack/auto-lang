// Register custom cfg values so rustc doesn't warn about them being unexpected.
//
// `feature = "interpreter"` guards legacy converter implementations
// (convert_node_dynamic, etc.) that have been superseded by newer code.
// The feature is intentionally never defined, so the guarded code stays
// disabled — but we register it here to silence `unexpected_cfgs` warnings
// rather than leaving readers to wonder whether it's a typo.
//
// PLAN-814 T-03/T-04 清偿：AUTO_UI_GEN_FINGERPRINT 生产已迁 auto-ui-compiler
// build.rs（generator_fingerprint() 公开面）；本 crate src/ui_gen 已不存在，
// 旧 stamp 为死代码移除。
use std::fs;
use std::path::Path;


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
}
