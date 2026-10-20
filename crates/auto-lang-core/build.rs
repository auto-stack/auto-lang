// PLAN-814: 随 stdlib_assembly/database 迁入 auto-lang-core 的 build 输入契约。
// 与 auto-lang/build.rs 同源（闭包=../../crates + 根锁 + AUTO_DOWN_ROOT +
// ui_gen 指纹）；差异仅一处：ui_gen 源在 T-03 迁入前位于兄弟 crate
// （../auto-lang/src/ui_gen），指纹对该目录扫描——UICache 失效语义保持绑定
// 真实生成器源码，迁移后随 owner 改扫本 crate。
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
    // PLAN-814 A 阶段：ui_gen 实现在兄弟 auto-lang crate（T-03 迁入本 crate）。
    walk(Path::new("../auto-lang/src/ui_gen"), "", &mut files);
    files.sort();

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
    println!("cargo:rustc-target={}", std::env::var("TARGET").unwrap());
    println!(
        "cargo:rustc-env=AUTO_ASSEMBLY_TARGET={}",
        std::env::var("TARGET").unwrap()
    );
    println!("cargo::rustc-check-cfg=cfg(feature, values(\"interpreter\"))");
    println!("cargo:rerun-if-changed=../auto-lang/src/ui_gen");
    println!(
        "cargo:rustc-env=AUTO_UI_GEN_FINGERPRINT={:016x}",
        hash_ui_gen_sources()
    );
}
