// PLAN-814: 随 stdlib_assembly/database 迁入 auto-lang-core 的 build 输入契约。
// 与 auto-lang/build.rs 同源（闭包=../../crates + 根锁 + AUTO_DOWN_ROOT）。
// PLAN-814 T-03：ui_gen 指纹随生成器源码迁 auto-ui-compiler/build.rs（本
// crate 不再扫兄弟 UI 目录——SDK 闭包零 UI 实现的 build 边同样成立）；
// UICache 失效语义改为运行时注入（database/ui_cache.rs），None=foreign
// build 语义保持。
use std::fs;
use std::path::Path;

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
}
