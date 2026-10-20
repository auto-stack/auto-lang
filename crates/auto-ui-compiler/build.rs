// PLAN-814 T-03：ui_gen 生成器源指纹（随 owner 从 auto-lang build.rs 迁入）。
// UICache 失效语义（Plan 692 W-1/698 SD-01）：生成器源变化即令全部缓存
// 失效——指纹经本 crate 编译期 env + lib::generator_fingerprint() 公开，
// SDK database::ui_cache 经运行时注入消费（SDK 不反依赖本 crate）。
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

    let mut hash: u64 = 0xcbf29ce484222325;
    for (rel, content) in &files {
        for b in format!("{}:{}\u{FF}", rel, content.len()).bytes() {
            hash ^= b as u64;
            hash = hash.wrapping_mul(0x100000001b3);
        }
    }
    hash
}

fn main() {
    println!("cargo:rerun-if-changed=src/ui_gen");
    println!(
        "cargo:rustc-env=AUTO_UI_GEN_FINGERPRINT={:016x}",
        hash_ui_gen_sources()
    );
}
