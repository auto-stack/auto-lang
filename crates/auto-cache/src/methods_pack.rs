// Plan 430 C2: 三方 crate 方法 shim 包管线。
//
// dep 声明的 crate 在自由函数 wrapper(plan-212 syn 路径)之外,再多做一步:
//   nightly rustdoc 提取方法元信息 → shim-metadata 分类/生成 → 编译独立 cdylib
//   → 按"元信息版本指纹"(C3) 缓存到 ~/.auto/sandbox/crates/。
// 与自由函数路径共存、互不干扰:方法 wrapper 全部形如 auto_<Type>_<method>_<sig>,
// manifest 由 cdylib 自带(auto__shim_manifest 导出),加载侧解析后注册 dispatch。
//
// 降级策略:nightly 不可用 → 返回 Ok(None),仅自由函数路径可用(现状不变)。

use crate::sandbox::{DepSource, Sandbox, SandboxError};
use shim_metadata::classify::{classify_all, Exceptions};
use shim_metadata::emit_cdylib::{emit_pack, PackMeta};
use std::path::PathBuf;
use std::process::Command;
use std::sync::OnceLock;

/// 一次成功构建的产物描述。
pub struct MethodsPackBuilt {
    /// 编译产物路径(缓存的 cdylib)
    pub lib_path: PathBuf,
    /// shim 包清单 JSON(auto-lang 加载侧解析;与 cdylib 内嵌 manifest 一致)
    pub manifest_json: String,
    /// 元信息版本指纹(C3)
    pub fingerprint: String,
    pub methods: usize,
    pub skipped: usize,
}

/// nightly cargo 是否可用(进程内缓存)。
pub fn nightly_available() -> bool {
    static OK: OnceLock<bool> = OnceLock::new();
    *OK.get_or_init(|| {
        Command::new("cargo")
            .arg("+nightly")
            .arg("--version")
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
    })
}

/// nightly rustc 版本串(指纹输入;进程内缓存)。
fn nightly_rustc_version() -> Option<String> {
    static V: OnceLock<Option<String>> = OnceLock::new();
    V.get_or_init(|| {
        let out = Command::new("rustc")
            .arg("+nightly")
            .arg("--version")
            .output()
            .ok()
            .filter(|o| o.status.success())?;
        Some(String::from_utf8_lossy(&out.stdout).trim().to_string())
    })
    .clone()
}

impl Sandbox {
    /// 方法 shim 包的 wrapper 目录名(builds/ 下)与包名。
    pub fn methods_wrapper_name(crate_name: &str) -> String {
        format!("{}_methods_wrapper", crate_name.replace('-', "_"))
    }

    /// 在缓存中查找已编译的方法 shim 包(按 manifest.json 指纹定位 cdylib)。
    /// 返回 (lib 路径, manifest JSON)。
    pub fn find_methods_pack(&self, crate_name: &str) -> Option<(PathBuf, String)> {
        let wrapper = Self::methods_wrapper_name(crate_name);
        let manifest_path = self.root().join("builds").join(&wrapper).join("manifest.json");
        let manifest_json = std::fs::read_to_string(&manifest_path).ok()?;
        let fp = extract_fingerprint(&manifest_json)?;
        let version = format!("fp{}", &fp[..12.min(fp.len())]);
        let lib = self.crate_library_path(&wrapper, &version);
        if lib.exists() {
            Some((lib, manifest_json))
        } else {
            None
        }
    }

    /// Plan 430 C2: 提取方法元信息并编译 shim 包。
    ///
    /// 成功返回 Some(built);nightly 不可用返回 Ok(None)(降级);
    /// rustdoc/编译失败返回 Err(调用方记日志后继续,不影响自由函数路径)。
    pub fn compile_dep_methods(
        &self,
        crate_name: &str,
        source: &DepSource,
    ) -> Result<Option<MethodsPackBuilt>, SandboxError> {
        if !nightly_available() {
            log::info!(
                "plan430: nightly toolchain unavailable, skip methods pack for {}",
                crate_name
            );
            return Ok(None);
        }
        let toolchain = nightly_rustc_version().unwrap_or_else(|| "nightly-unknown".into());
        let crate_ident = crate_name.replace('-', "_");
        let wrapper = Self::methods_wrapper_name(crate_name);

        // 1. 缓存快路径(manifest 指纹 → cdylib)
        if let Some((lib, manifest_json)) = self.find_methods_pack(crate_name) {
            log::info!(
                "plan430: cached methods pack for {}: {}",
                crate_name,
                lib.display()
            );
            let fp = extract_fingerprint(&manifest_json).unwrap_or_default();
            let (methods, skipped) = count_entries(&manifest_json);
            return Ok(Some(MethodsPackBuilt {
                lib_path: lib,
                manifest_json,
                fingerprint: fp,
                methods,
                skipped,
            }));
        }

        // 2. wrapper 工程(独立工作区,防被宿主 repo 吸收)。
        //    先写占位 lib.rs——rustdoc 解析 manifest 需要 lib 目标存在,
        //    真正的 wrapper 源码在第 4 步生成后覆盖。
        let build_dir = self.root().join("builds").join(&wrapper);
        let src_dir = build_dir.join("src");
        std::fs::create_dir_all(&src_dir)?;
        let dep_line = source.to_cargo_line(crate_name);
        std::fs::write(build_dir.join("Cargo.toml"), wrapper_cargo_toml(&crate_ident, &dep_line))?;
        let placeholder = src_dir.join("lib.rs");
        if !placeholder.exists() {
            std::fs::write(&placeholder, "// placeholder (plan-430 C2); overwritten by the generator\n")?;
        }

        // 3. nightly rustdoc 提取元信息(-p 在依赖里文档化目标 crate)
        let out = Command::new("cargo")
            .arg("+nightly")
            .arg("rustdoc")
            .arg("-p")
            .arg(crate_name)
            .arg("-Zunstable-options")
            .arg("--output-format")
            .arg("json")
            .current_dir(&build_dir)
            .output()
            .map_err(|e| SandboxError::CompilationFailed(format!("rustdoc spawn failed: {e}")))?;
        if !out.status.success() {
            return Err(SandboxError::CompilationFailed(format!(
                "rustdoc failed for {}: {}",
                crate_name,
                String::from_utf8_lossy(&out.stderr)
            )));
        }
        let doc_json = find_rustdoc_json(&build_dir.join("target").join("doc"), &crate_ident)
            .ok_or_else(|| {
                SandboxError::CompilationFailed(format!(
                    "rustdoc json not found under {} for {}",
                    build_dir.join("target").join("doc").display(),
                    crate_name
                ))
            })?;
        let doc = std::fs::read_to_string(&doc_json)?;
        let parsed = shim_metadata::rustdoc::parse_all(&doc)
            .map_err(|e| SandboxError::CompilationFailed(format!("rustdoc parse failed: {e}")))?;
        let exc = Exceptions::default();
        let classified = classify_all(&parsed.methods, &exc);

        // 4. 指纹 + 生成(C3:工具链 × crate × 生成器 × 签名集)
        let meta = PackMeta {
            crate_name: crate_name.to_string(),
            crate_version: source.version.clone().unwrap_or_else(|| "1".into()),
            toolchain,
        };
        let (fp, files) = emit_pack(&meta, &dep_line, &classified, &exc, &parsed.free_fns);
        std::fs::write(src_dir.join("lib.rs"), &files.lib_rs)?;
        std::fs::write(build_dir.join("manifest.json"), &files.manifest_json)?;
        std::fs::write(build_dir.join("signatures.json"), &files.signatures_json)?;
        std::fs::write(build_dir.join("rules.json"), &files.rules_json)?;

        // 5. 编译 cdylib(stable 工具链即可;rustdoc 仅用于提取)
        let out = Command::new(self.cargo_path())
            .args(["build", "--release"])
            .current_dir(&build_dir)
            .output()
            .map_err(|e| SandboxError::CompilationFailed(format!("cargo spawn failed: {e}")))?;
        if !out.status.success() {
            return Err(SandboxError::CompilationFailed(format!(
                "methods wrapper build failed for {}: {}",
                crate_name,
                String::from_utf8_lossy(&out.stderr)
            )));
        }
        let target_dir = build_dir.join("target").join("release");
        let lib_file = self
            .find_cdylib_in_dir(&target_dir, &wrapper)
            .ok_or_else(|| {
                SandboxError::CompilationFailed(format!(
                    "methods cdylib not found in {}",
                    target_dir.display()
                ))
            })?;
        let version = format!("fp{}", &fp[..12.min(fp.len())]);
        let output_path = self.crate_library_path(&wrapper, &version);
        std::fs::copy(&lib_file, &output_path)?;
        log::info!(
            "plan430: compiled methods pack for {} (fp={}, methods={}, skips={}): {}",
            crate_name,
            version,
            classified.plans.len(),
            classified.skips.len(),
            output_path.display()
        );

        Ok(Some(MethodsPackBuilt {
            methods: classified.plans.len(),
            skipped: classified.skips.len(),
            fingerprint: fp,
            manifest_json: files.manifest_json,
            lib_path: output_path,
        }))
    }
}

fn wrapper_cargo_toml(crate_ident: &str, dep_line: &str) -> String {
    format!(
        "# Generated by auto-cache methods_pack (plan-430 C2). DO NOT EDIT BY HAND.\n\
         [package]\n\
         name = \"{crate_ident}_methods_wrapper\"\n\
         version = \"1.0.0\"\n\
         edition = \"2021\"\n\
         \n\
         [workspace]\n\
         \n\
         [lib]\n\
         crate-type = [\"cdylib\"]\n\
         \n\
         [dependencies]\n\
         {dep_line}\n"
    )
}

/// 从 manifest JSON 提取 fingerprint 字段(轻量,不建全结构)。
fn extract_fingerprint(manifest_json: &str) -> Option<String> {
    serde_json::from_str::<serde_json::Value>(manifest_json)
        .ok()?
        .get("fingerprint")?
        .as_str()
        .map(String::from)
}

fn count_entries(manifest_json: &str) -> (usize, usize) {
    match serde_json::from_str::<shim_metadata::emit_cdylib::ShimManifest>(manifest_json) {
        Ok(m) => (m.methods.len(), m.functions.len()),
        Err(_) => (0, 0),
    }
}

/// 在 target/doc 下找 rustdoc JSON:优先 crate 同名文件,否则唯一的 *.json。
fn find_rustdoc_json(doc_dir: &std::path::Path, crate_ident: &str) -> Option<PathBuf> {
    if !doc_dir.is_dir() {
        return None;
    }
    let preferred = doc_dir.join(format!("{crate_ident}.json"));
    if preferred.is_file() {
        return Some(preferred);
    }
    let mut jsons: Vec<PathBuf> = std::fs::read_dir(doc_dir)
        .ok()?
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .filter(|p| p.extension().map(|x| x == "json").unwrap_or(false))
        .collect();
    if jsons.len() == 1 {
        jsons.pop()
    } else {
        // 多个 json 且无同名:无法裁决(依赖也被文档化的场景)
        log::warn!(
            "plan430: ambiguous rustdoc jsons under {} ({} files)",
            doc_dir.display(),
            jsons.len()
        );
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wrapper_name_normalizes_hyphens() {
        assert_eq!(
            Sandbox::methods_wrapper_name("percent-encoding"),
            "percent_encoding_methods_wrapper"
        );
    }

    #[test]
    fn fingerprint_extraction() {
        let m = r#"{"format":1,"fingerprint":"0123456789abcdef","methods":[]}"#;
        assert_eq!(extract_fingerprint(m).as_deref(), Some("0123456789abcdef"));
        assert_eq!(extract_fingerprint("not json"), None);
    }
}
