// PLAN-814: 随 stdlib_assembly/database 迁入 auto-lang-core 的 build 输入契约。
// PLAN-814 T-03：ui_gen 指纹随生成器源码迁 auto-ui-compiler/build.rs；UICache
// 失效语义改为运行时注入（database/ui_cache.rs），None=foreign build 语义保持。
// PLAN-814 R3-03/R4（build 输入 owner 闭包收窄）：身份源=**具名清单**——
// SDK 编译闭包内的 workspace path 依赖 crate（SDK rlib 真正包含的源码）。
// 旧的全 crates/ 扫描把 auto-lang/auto-man/auto-ui-{compiler,runtime,
// dialect} 源码纳入 BUILD_INPUTS——它们不进 SDK 产物，其变化不应改变
// SDK assembly 身份（负控=manifest.rs 测试断言禁入清单 + Cargo.toml
// 直接 path 依赖全覆盖断言；清单随依赖演进由该断言强制同步）。
const IDENTITY_SOURCE_CRATES: &[&str] = &[
    "a2r-std",
    "auto-ast",
    "auto-atom",
    "auto-bindgen",
    "auto-cache",
    "auto-diagnostics",
    "auto-lang-core",
    "auto-lex",
    "auto-macros",
    "auto-media-core",
    "auto-parser-api",
    "auto-terminal-core",
    "auto-test-case",
    "auto-text-core",
    "auto-val",
    "shim-metadata",
];
// 具名清单中不在 crates/ 下的成员：vendor/ 路径（相对仓库根）。
const IDENTITY_SOURCE_OUT_OF_TREE: &[(&str, &str)] =
    &[("vendor/autodown-test-parser", "vendor/autodown-test-parser")];

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
    // 792 受管投影/无 UI 快照只携带清单子集：具名目录缺席即跳过（身份
    // 只覆盖实际存在的源——与旧全扫描"存在即扫"语义对齐；完整检出下
    // 清单全命中）。
    for crate_name in IDENTITY_SOURCE_CRATES {
        let dir = root.join("crates").join(crate_name);
        if dir.is_dir() {
            collect(&dir, &format!("crates/{crate_name}"), &mut inputs);
        }
    }
    for (dir, prefix) in IDENTITY_SOURCE_OUT_OF_TREE {
        let dir = root.join(dir);
        if dir.is_dir() {
            collect(&dir, prefix, &mut inputs);
        }
    }
    for name in ["Cargo.toml", "Cargo.lock"] {
        let path = root.join(name);
        println!("cargo:rerun-if-changed={}", path.display());
        let mut hash = 0xcbf29ce484222325u64;
        for byte in fs::read(path).unwrap() {
            hash = (hash ^ byte as u64).wrapping_mul(0x100000001b3);
        }
        inputs.push((name.into(), hash));
    }
    // 具名外部输入（可选）：跨仓 auto-down 源——仅显式 AUTO_DOWN_ROOT
    // 指定时纳入（交付 vendoring 身份；本地默认缺席）。
    let down = std::env::var_os("AUTO_DOWN_ROOT")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|| root.join("../auto-down"));
    if down.join("Cargo.toml").is_file() && std::env::var_os("AUTO_DOWN_ROOT").is_some() {
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
    println!(
        "cargo:rustc-env=AUTO_IDENTITY_SOURCE_CRATES={}",
        IDENTITY_SOURCE_CRATES.join(",")
    );
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
