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
use shim_metadata::classify::{classify_all_third_party as classify_all, Exceptions};
use shim_metadata::emit_cdylib::{emit_pack_parts, PackMeta};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::OnceLock;

/// rustc 检查器剔除环的总构建尝试上限(首建 + 重试)。
/// 430 复审修复:此前无上限,最坏 O(plans) 轮全量 build。
const MAX_BUILD_ATTEMPTS: u32 = 5;

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
        //    430 复审修复:核对 manifest 记录的 crate 版本与当前解析版本——
        //    `dep uuid = "1"` 这类半开区间声明升级后(1.0→1.9),声明版本串不变,
        //    若不核对则指纹永续陈旧(C3"防签名漂移"空转)。核对失败(无构建目录/
        //    cargo metadata 不可用)时按缓存接受,保持降级姿态。
        if let Some((lib, manifest_json)) = self.find_methods_pack(crate_name) {
            let stale = match (
                extract_crate_version(&manifest_json),
                resolved_crate_version(
                    &self.cargo_path(),
                    &self.root().join("builds").join(&wrapper),
                    crate_name,
                ),
            ) {
                (Some(cached), Some(current)) if cached != current => {
                    log::info!(
                        "plan430: cached methods pack for {} is stale ({} -> {}), rebuilding",
                        crate_name,
                        cached,
                        current
                    );
                    true
                }
                _ => false,
            };
            if !stale {
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

        // 4+5. 生成 → 编译,失败时借 rustc 当检查器:从报错提取肇事符号,
        // 剔除对应方法后重试(至多 4 轮;Plan D/E 同款做法,防止个别
        // 不可编译的 wrapper 弄死整包 —— u128 参数/跨 crate opaque 实参等)。
        // 430 复审修复:crate_version 用 cargo metadata 解析出的**真实版本**
        // (此前用声明版本,`dep uuid = "1"` 下升级 1.0→1.9 指纹不变,缓存陈旧)。
        // 解析失败(离线/metadata 异常)回退声明版本,保持降级姿态。
        let resolved_version = resolved_crate_version(&self.cargo_path(), &build_dir, crate_name);
        if let Some(rv) = &resolved_version {
            if Some(rv) != source.version.as_ref() {
                log::info!(
                    "plan430: {} resolved to version {} (declared {:?})",
                    crate_name,
                    rv,
                    source.version
                );
            }
        }
        let meta = PackMeta {
            crate_name: crate_name.to_string(),
            crate_version: resolved_version
                .or_else(|| source.version.clone())
                .unwrap_or_else(|| "unknown".into()),
            toolchain,
        };
        let mut plans = classified.plans.clone();
        let mut skips = classified.skips.clone();
        // 430 复审修复:剔环轮次上限(首建之外至多重试 MAX_BUILD_ATTEMPTS-1 轮)。
        let mut retries_left: u32 = MAX_BUILD_ATTEMPTS - 1;
        let (fp, files) = loop {
            let (fp, files) = emit_pack_parts(&meta, &dep_line, &plans, &skips, &exc, &parsed.free_fns);
            std::fs::write(src_dir.join("lib.rs"), &files.lib_rs)?;

            let out = Command::new(self.cargo_path())
                .args(["build", "--release"])
                .current_dir(&build_dir)
                .output()
                .map_err(|e| {
                    SandboxError::CompilationFailed(format!("cargo spawn failed: {e}"))
                })?;
            if out.status.success() {
                break (fp, files);
            }
            let stderr = String::from_utf8_lossy(&out.stderr).to_string();
            let before = plans.len();
            let offenders = offending_symbols(&stderr);
            partition_out_offenders(&mut plans, &mut skips, &offenders);
            if plans.len() == before {
                // 报错与已知符号对不上(或已无可剔)——如实失败
                return Err(SandboxError::CompilationFailed(format!(
                    "methods wrapper build failed for {}: {}",
                    crate_name, stderr
                )));
            }
            log::warn!(
                "plan430: rustc check dropped {} methods for {} (offenders: {:?}), retrying",
                before - plans.len(),
                crate_name,
                offenders
            );
            if plans.is_empty() {
                return Err(SandboxError::CompilationFailed(format!(
                    "all methods dropped by rustc check for {}: {}",
                    crate_name, stderr
                )));
            }
            if retries_left == 0 {
                return Err(SandboxError::CompilationFailed(format!(
                    "methods wrapper build for {} still failing after {} rustc-check retries \
                     (dropped {} of {} methods so far): {}",
                    crate_name,
                    MAX_BUILD_ATTEMPTS - 1,
                    classified.plans.len() - plans.len(),
                    classified.plans.len(),
                    stderr
                )));
            }
            retries_left -= 1;
        };
        std::fs::write(build_dir.join("manifest.json"), &files.manifest_json)?;
        std::fs::write(build_dir.join("signatures.json"), &files.signatures_json)?;
        std::fs::write(build_dir.join("rules.json"), &files.rules_json)?;

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
            plans.len(),
            skips.len(),
            output_path.display()
        );

        Ok(Some(MethodsPackBuilt {
            methods: plans.len(),
            skipped: skips.len(),
            fingerprint: fp,
            manifest_json: files.manifest_json,
            lib_path: output_path,
        }))
    }
}

/// 从 rustc 报错文本提取本生成器的导出符号名(auto_ 前缀)。
fn offending_symbols(stderr: &str) -> Vec<String> {
    let mut found: Vec<String> = Vec::new();
    for tok in stderr.split(|c: char| !(c.is_ascii_alphanumeric() || c == '_')) {
        if tok.starts_with("auto_") && !found.iter().any(|f| f == tok) {
            found.push(tok.to_string());
        }
    }
    found
}

/// 按肇事符号划分保留/剔除:方法导出符号**精确**命中,或该类型的
/// auto__drop_<Type> 命中(类型级问题 → 连带剔除整个类型的方法)。
/// 430 复审修复:此前用 starts_with("auto_Type_method") 前缀匹配,
/// auto_Counter_newest_p_p 会连带误伤 Counter::new(set/set_label 同理)。
fn partition_out_offenders(
    plans: &mut Vec<shim_metadata::types::MarshalPlan>,
    skips: &mut Vec<shim_metadata::types::Skip>,
    offenders: &[String],
) {
    if offenders.is_empty() {
        return;
    }
    let dropped_types: Vec<String> = offenders
        .iter()
        .filter_map(|s| s.strip_prefix("auto__drop_").map(String::from))
        .collect();
    let mut i = 0;
    while i < plans.len() {
        let export = shim_metadata::emit_cdylib::plan_export_symbol(&plans[i]);
        let hit = offenders.iter().any(|o| *o == export)
            || dropped_types.contains(&plans[i].method.type_name);
        if hit {
            let p = plans.remove(i);
            skips.push(shim_metadata::types::Skip {
                type_name: p.method.type_name.clone(),
                method: p.method.method.clone(),
                reason: "rustc check failed (dropped by retry loop)".into(),
            });
        } else {
            i += 1;
        }
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

/// 从 manifest JSON 提取 crate_version 字段(430 复审修复:缓存陈旧检测用)。
fn extract_crate_version(manifest_json: &str) -> Option<String> {
    serde_json::from_str::<serde_json::Value>(manifest_json)
        .ok()?
        .get("crate_version")?
        .as_str()
        .map(String::from)
}

/// 用 cargo metadata 查询构建目录中 crate **解析后的真实版本**。
/// `--locked` 保证不动 Cargo.lock(此刻依赖已被 rustdoc 解析过,本地必有);
/// 失败(无构建目录/离线缺依赖/JSON 异常)返回 None,调用方按降级处理。
fn resolved_crate_version(cargo: &Path, build_dir: &Path, crate_name: &str) -> Option<String> {
    if !build_dir.join("Cargo.lock").exists() {
        return None;
    }
    let run = |locked: bool| {
        let mut cmd = Command::new(cargo);
        cmd.args(["metadata", "--format-version", "1"]);
        if locked {
            cmd.arg("--locked");
        }
        cmd.current_dir(build_dir).output().ok()
    };
    let out = run(true).filter(|o| o.status.success()).or_else(|| run(false))?;
    let json: serde_json::Value = serde_json::from_slice(&out.stdout).ok()?;
    json.get("packages")?
        .as_array()?
        .iter()
        .find(|p| p.get("name").and_then(|n| n.as_str()) == Some(crate_name))
        .and_then(|p| p.get("version"))?
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

    #[test]
    fn crate_version_extraction() {
        // 430 复审修复:缓存陈旧检测依赖 manifest 记录的 crate_version 可读
        let m = r#"{"format":1,"crate_version":"1.9.0","fingerprint":"abc"}"#;
        assert_eq!(extract_crate_version(m).as_deref(), Some("1.9.0"));
        assert_eq!(extract_crate_version(r#"{"format":1}"#), None);
    }

    #[test]
    fn resolved_version_requires_lockfile_or_metadata() {
        // 无 Cargo.lock 的目录(如临时目录)→ None(降级,不报错)
        let tmp = std::env::temp_dir().join("plan430_no_lock_dir");
        assert_eq!(
            resolved_crate_version(Path::new("cargo"), &tmp, "whatever"),
            None
        );
    }

    #[test]
    fn offender_partition_matches_exactly_not_by_prefix() {
        // 430 复审修复:auto_Counter_newest_p_p 不得误伤 Counter::new
        use shim_metadata::types::{ArgPlan, MarshalPlan, RetPlan, SelfKind, ShimMethod, Ty};
        fn plan(method: &str) -> MarshalPlan {
            MarshalPlan {
                method: ShimMethod {
                    type_name: "Counter".into(),
                    method: method.into(),
                    self_kind: SelfKind::Write,
                    params: vec![],
                    ret: Ty::Void,
                    generic: false,
                    fallible: false,
                    field: None,
                },
                args: vec![],
                ret: RetPlan::Void,
                copy_result: false,
                fallible: false,
            }
        }
        let mut plans = vec![plan("new"), plan("newest"), plan("set"), plan("set_label")];
        let mut skips = Vec::new();
        // 精确肇事符号:newest(签名 p_v)与 set_label(签名 p_v)——
        // 完整导出名由 plan_export_symbol 给出
        let offenders = vec![
            shim_metadata::emit_cdylib::plan_export_symbol(&plans[1]),
            shim_metadata::emit_cdylib::plan_export_symbol(&plans[3]),
        ];
        partition_out_offenders(&mut plans, &mut skips, &offenders);
        let remain: Vec<&str> = plans.iter().map(|p| p.method.method.as_str()).collect();
        assert_eq!(remain, vec!["new", "set"], "前缀同名方法不得被误伤");
        assert_eq!(skips.len(), 2);

        // 类型级:auto__drop_Counter 命中 → 整类型连带剔除
        let mut plans = vec![plan("new"), plan("value")];
        let mut skips = Vec::new();
        partition_out_offenders(&mut plans, &mut skips, &["auto__drop_Counter".to_string()]);
        assert!(plans.is_empty());
        assert_eq!(skips.len(), 2);
    }
}
