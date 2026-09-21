//! PLAN-013 T2: pac.at 声明的 .rs 侧车供给(auto-term at-app 三形态)。
//!
//! `auto run` 的两个 Rust 生成点——rust UI front crate(rust_ui.rs)与
//! back crate(api_gen.rs)——只写生成代码,没有用户 .rs 供给机制;而
//! a2r 对 `use auto.term: …` 的下降是 `use crate::term::{…}`,要求生成
//! 工程里存在同名模块。本模块把 pac.at 声明的用户 .rs 文件与 Cargo
//! 依赖注入两个生成点:
//!
//! ```text
//! rust_sidecar {
//!     modules: ["term:term.rs"]
//!     deps: ["libloading:0.8"]
//! }
//! ```
//!
//! - `modules`: `<mod名>:<工程相对源路径>`;复制为生成 crate 的
//!   `src/<mod名>.rs`,并在 crate 的 main.rs 追加 `mod <mod名>;`;
//! - `deps`: `<crate>:<版本>`;注入生成 Cargo.toml 的 [dependencies]。
//!
//! 幂等:mod 声明与依赖条目已存在则不重复追加;模块文件以工程源为准
//! 覆盖(源是唯一真身,生成物勿手改)。

use std::path::{Path, PathBuf};

use auto_val::{Obj, Value};

use crate::AutoResult;

/// pac.at `rust_sidecar` 解析结果(缺省 = 空,零行为)。
#[derive(Debug, Default, Clone)]
pub struct SidecarSpec {
    /// (module name, project-relative source path)
    pub modules: Vec<(String, PathBuf)>,
    /// (crate name, version requirement)
    pub deps: Vec<(String, String)>,
}

impl SidecarSpec {
    pub fn is_empty(&self) -> bool {
        self.modules.is_empty() && self.deps.is_empty()
    }
}

/// 从工程根的 pac.at 读取 `rust_sidecar` 节;无 pac.at / 无节点 / 字段
/// 缺失一律返回空 spec(生成流程不因侧车缺省而分叉)。
pub fn load_sidecar(project_dir: &Path) -> SidecarSpec {
    let mut spec = SidecarSpec::default();
    let pac = project_dir.join("pac.at");
    if !pac.is_file() {
        return spec;
    }
    let Ok(config) = auto_lang::config::AutoConfig::from_file(&pac, &Obj::new()) else {
        return spec;
    };
    // 主形态:`rust_sidecar { … }` 是 root 的具名子节点。
    let (modules_v, deps_v) = match config.root.get_nodes("rust_sidecar").into_iter().next() {
        Some(node) => (node.get_prop("modules"), node.get_prop("deps")),
        None => {
            // 兜底:obj/嵌套 value 形态。
            match config.root.get_prop("rust_sidecar") {
                Value::Obj(obj) => (
                    obj.get("modules").unwrap_or(Value::Nil),
                    obj.get("deps").unwrap_or(Value::Nil),
                ),
                Value::Node(node) => (node.get_prop("modules"), node.get_prop("deps")),
                _ => return spec,
            }
        }
    };
    fill(&mut spec, read_str_list(&modules_v), read_str_list(&deps_v));
    spec
}

fn fill(spec: &mut SidecarSpec, modules: Vec<String>, deps: Vec<String>) {
    for entry in modules {
        // 首个 ':' 分隔;模块名必须是合法 Rust ident(防 "C:/..." 被误拆)。
        if let Some((name, path)) = entry.split_once(':') {
            let name = name.trim();
            let valid_ident = name
                .chars()
                .next()
                .map(|c| c.is_ascii_lowercase() || c == '_')
                .unwrap_or(false)
                && name.chars().all(|c| c.is_ascii_alphanumeric() || c == '_');
            if valid_ident && !path.trim().is_empty() {
                spec.modules.push((name.to_string(), PathBuf::from(path.trim())));
            }
        }
    }
    for entry in deps {
        if let Some((name, version)) = entry.split_once(':') {
            let name = name.trim();
            if !name.is_empty() && !version.trim().is_empty() {
                // PLAN-025 续(024 顺带根修):name:path:REL 形态 = path
                // 依赖(025 pac.at 注入约定;注入器此前未实现该分支,
                // 生成 `name = "path:REL"` 畸形 toml 挡死 cargo build)。
                // 值存最终 toml 表达式:版本依赖原样;path 依赖展开
                // `{ path = "REL" }`。幂等检查按行前缀 name 仍命中。
                if let Some(rel) = version.strip_prefix("path:") {
                    let rel = rel.trim();
                    if !rel.is_empty() {
                        spec.deps.push((
                            name.to_string(),
                            format!("{{ path = \"{rel}\" }}"),
                        ));
                        continue;
                    }
                }
                spec.deps.push((name.to_string(), version.trim().to_string()));
            }
        }
    }
}

fn read_str_list(value: &Value) -> Vec<String> {
    match value {
        Value::Array(arr) | Value::Block(arr) => arr
            .iter()
            .map(|v| v.to_astr().trim().to_string())
            .filter(|s| !s.is_empty())
            .collect(),
        Value::Str(s) if !s.trim().is_empty() => vec![s.trim().to_string()],
        _ => Vec::new(),
    }
}

/// 把侧车施加到一个生成的 Rust crate(`<crate>/src/main.rs` +
/// `<crate>/Cargo.toml`)。复制模块文件 + 追加 mod 声明 + 注入依赖,
/// 全部幂等。`mod_names_extra`:除 spec 外还要声明 mod 的名字(供
/// 调用方把生成期额外的 .rs 模块一起接线)。
pub fn apply_sidecar_to_crate(
    spec: &SidecarSpec,
    project_dir: &Path,
    crate_dir: &Path,
) -> AutoResult<()> {
    let src_dir = crate_dir.join("src");
    std::fs::create_dir_all(&src_dir)
        .map_err(|e| format!("sidecar: failed to create {}: {}", src_dir.display(), e))?;

    // 1) 模块文件复制(源为准,覆盖)。
    for (name, source_rel) in &spec.modules {
        let source = project_dir.join(source_rel);
        if !source.is_file() {
            return Err(format!(
                "sidecar: module '{name}' source not found: {}",
                source.display()
            )
            .into());
        }
        let dest = src_dir.join(format!("{name}.rs"));
        std::fs::copy(&source, &dest)
            .map_err(|e| format!("sidecar: copy {} → {}: {}", source.display(), dest.display(), e))?;
    }

    // 2) main.rs 追加 mod 声明(幂等)。
    if !spec.modules.is_empty() {
        let main_rs = src_dir.join("main.rs");
        let mut content = std::fs::read_to_string(&main_rs).unwrap_or_default();
        let mut missing = String::new();
        for (name, _) in &spec.modules {
            let decl = format!("mod {name};");
            if !content.contains(&decl) {
                missing.push_str(&decl);
                missing.push('\n');
            }
        }
        if !missing.is_empty() {
            content.push('\n');
            content.push_str("// rust_sidecar: 用户 .rs 侧车模块声明(PLAN-013 T2;勿手改)\n");
            content.push_str(&missing);
            std::fs::write(&main_rs, content)
                .map_err(|e| format!("sidecar: write {}: {}", main_rs.display(), e))?;
        }
    }

    // 3) Cargo.toml 注入 [dependencies](幂等)。
    if !spec.deps.is_empty() {
        let cargo_path = crate_dir.join("Cargo.toml");
        let mut cargo = std::fs::read_to_string(&cargo_path)
            .map_err(|e| format!("sidecar: read {}: {}", cargo_path.display(), e))?;
        let mut missing = String::new();
        for (name, version) in &spec.deps {
            let already = cargo
                .lines()
                .any(|l| l.starts_with(name.as_str()) && l.contains(version.as_str()));
            if !already && !cargo.lines().any(|l| l.starts_with(&format!("{name} ="))) {
                missing.push_str(&format!("{name} = \"{version}\"\n"));
            }
        }
        if !missing.is_empty() {
            if cargo.contains("[dependencies]") {
                cargo = cargo.replacen("[dependencies]", &format!("[dependencies]\n{}", missing), 1);
            } else {
                cargo.push_str(&format!("\n[dependencies]\n{missing}"));
            }
            std::fs::write(&cargo_path, cargo)
                .map_err(|e| format!("sidecar: write {}: {}", cargo_path.display(), e))?;
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 临时工程(pac.at + 侧车源)→ 载入 spec 断言字段。
    #[test]
    fn sidecar_spec_parses_pac_at() {
        let dir = std::env::temp_dir().join(format!("auto_sidecar_spec_{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(dir.join("src/back")).unwrap();
        std::fs::write(
            dir.join("pac.at"),
            r#"name: "demo"
rust_sidecar {
    modules: ["term:term.rs", "glue:src/back/glue.rs"]
    deps: ["libloading:0.8"]
}
"#,
        )
        .unwrap();
        let spec = load_sidecar(&dir);
        assert_eq!(spec.modules.len(), 2, "{spec:?}");
        assert_eq!(spec.modules[0].0, "term");
        assert_eq!(spec.modules[0].1, PathBuf::from("term.rs"));
        assert_eq!(spec.modules[1].0, "glue");
        assert_eq!(spec.deps, vec![("libloading".to_string(), "0.8".to_string())]);
        let _ = std::fs::remove_dir_all(&dir);
    }

    /// PLAN-025 续(024 顺带根修):deps 的 name:path:REL 形态 = path
    /// 依赖,值展开为 `{ path = "REL" }` toml 表达式(注入器此前未实现,
    /// 生成 `name = "path:REL"` 畸形 toml 挡死 cargo build——auto-term
    /// rust 载具实录)。
    #[test]
    fn sidecar_spec_parses_path_deps() {
        let dir = std::env::temp_dir().join(format!("auto_sidecar_path_{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(
            dir.join("pac.at"),
            r#"name: "demo"
rust_sidecar {
    deps: ["libloading:0.8", "autoterm-config:path:../../../crates/autoterm-config"]
}
"#,
        )
        .unwrap();
        let spec = load_sidecar(&dir);
        assert_eq!(
            spec.deps,
            vec![
                ("libloading".to_string(), "0.8".to_string()),
                (
                    "autoterm-config".to_string(),
                    "{ path = \"../../../crates/autoterm-config\" }".to_string()
                ),
            ],
            "{spec:?}"
        );
        let _ = std::fs::remove_dir_all(&dir);
    }

    /// 无 pac.at / 无 rust_sidecar → 空 spec,不报错。
    #[test]
    fn sidecar_spec_defaults_empty() {
        let dir = std::env::temp_dir().join(format!("auto_sidecar_empty_{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        assert!(load_sidecar(&dir).is_empty());
        std::fs::write(dir.join("pac.at"), "name: \"x\"\n").unwrap();
        assert!(load_sidecar(&dir).is_empty());
        let _ = std::fs::remove_dir_all(&dir);
    }

    /// 施加到生成 crate:复制 + mod 声明 + 依赖注入;二次施加幂等。
    #[test]
    fn sidecar_applies_idempotent() {
        let dir = std::env::temp_dir().join(format!("auto_sidecar_apply_{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(dir.join("src")).unwrap();
        std::fs::write(dir.join("term.rs"), "pub fn hi() {}\n").unwrap();
        std::fs::write(
            dir.join("pac.at"),
            "rust_sidecar {\n    modules: [\"term:term.rs\"]\n    deps: [\"libloading:0.8\"]\n}\n",
        )
        .unwrap();

        let crate_dir = dir.join("gen/app");
        std::fs::create_dir_all(crate_dir.join("src")).unwrap();
        std::fs::write(crate_dir.join("src/main.rs"), "fn main() {}\n").unwrap();
        std::fs::write(
            crate_dir.join("Cargo.toml"),
            "[package]\nname = \"app\"\n\n[dependencies]\niced = \"0.13\"\n",
        )
        .unwrap();

        let spec = load_sidecar(&dir);
        assert!(!spec.is_empty());
        apply_sidecar_to_crate(&spec, &dir, &crate_dir).unwrap();

        let copied = std::fs::read_to_string(crate_dir.join("src/term.rs")).unwrap();
        assert_eq!(copied, "pub fn hi() {}\n");
        let main_rs = std::fs::read_to_string(crate_dir.join("src/main.rs")).unwrap();
        assert!(main_rs.contains("mod term;"), "{main_rs}");
        let cargo = std::fs::read_to_string(crate_dir.join("Cargo.toml")).unwrap();
        assert!(cargo.contains("libloading = \"0.8\""), "{cargo}");

        // 幂等:再施加一次,不重复声明/注入。
        apply_sidecar_to_crate(&spec, &dir, &crate_dir).unwrap();
        let main_rs2 = std::fs::read_to_string(crate_dir.join("src/main.rs")).unwrap();
        assert_eq!(main_rs2.matches("mod term;").count(), 1, "{main_rs2}");
        let cargo2 = std::fs::read_to_string(crate_dir.join("Cargo.toml")).unwrap();
        assert_eq!(cargo2.matches("libloading").count(), 1, "{cargo2}");
        let _ = std::fs::remove_dir_all(&dir);
    }

    /// 侧车源缺失 → 显式报错(防静默断线)。
    #[test]
    fn sidecar_missing_source_errors() {
        let dir = std::env::temp_dir().join(format!("auto_sidecar_miss_{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(dir.join("src")).unwrap();
        std::fs::write(dir.join("term.rs"), "pub fn hi() {}\n").unwrap();
        std::fs::write(
            dir.join("pac.at"),
            "rust_sidecar {\n    modules: [\"nope:absent.rs\"]\n}\n",
        )
        .unwrap();
        let spec = load_sidecar(&dir);
        let crate_dir = dir.join("gen/app");
        std::fs::create_dir_all(crate_dir.join("src")).unwrap();
        std::fs::write(crate_dir.join("src/main.rs"), "fn main() {}\n").unwrap();
        let err = apply_sidecar_to_crate(&spec, &dir, &crate_dir);
        assert!(err.is_err(), "missing source must error");
        let _ = std::fs::remove_dir_all(&dir);
    }
}
