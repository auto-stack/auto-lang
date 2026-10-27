//! Shared source-owner validation for generated Rust consumers.
use auto_val::AutoResult;
use std::{
    fs,
    path::{Path, PathBuf},
};

/// Verify the actual UI crate files against a managed Full installation's
/// ownership and content receipt. This is consistency checking, not a signature.
fn installed_crate_source(
    producer_root: &Path,
    name: &str,
    copied_root: Option<&Path>,
    owner: &str,
) -> AutoResult<Option<PathBuf>> {
    use sha2::{Digest, Sha256};
    if producer_root.file_name().and_then(|s| s.to_str()) != Some("sdk") {
        return Ok(None);
    }
    let Some(install) = producer_root.parent() else {
        return Ok(None);
    };
    let receipt = install.join("install-manifest.json");
    if !receipt.is_file() {
        return Ok(None);
    }
    let doc: serde_json::Value = serde_json::from_slice(&fs::read(&receipt)?)
        .map_err(|e| format!("delivery: producer 安装清单无效: {e}"))?;
    let source_rev = doc["sources"][owner].as_str().unwrap_or("");
    if doc["schema"] != "man-install/1"
        || doc["profile"] != "full"
        || doc["source_closure"]["sdk_layout"] != "combined-crates"
        || source_rev.len() != 40
        || !source_rev.bytes().all(|b| b.is_ascii_hexdigit())
    {
        return Err("delivery: producer 安装清单版本、profile 或 UI 来源无效".into());
    }
    let packages = doc["source_closure"]["packages"]
        .as_array()
        .ok_or("delivery: producer 安装清单缺 source_closure.packages")?;
    let owners: Vec<_> = packages.iter().filter(|p| p["name"] == name).collect();
    let expected_path = format!("crates/{name}");
    if owners.len() != 1
        || owners[0]["source_repo"] != owner
        || owners[0]["installed_path"] != expected_path
        || owners[0]["source_path"] != expected_path
    {
        return Err(format!("delivery: producer 安装 crate 源所有权无效: {name}").into());
    }
    let files = doc["files"]
        .as_object()
        .ok_or("delivery: producer 安装清单缺 files")?;
    let prefix = format!("sdk/crates/{name}/");
    let expected: std::collections::BTreeMap<_, _> = files
        .iter()
        .filter(|(path, _)| path.starts_with(&prefix))
        .collect();
    if expected.is_empty() {
        return Err(format!("delivery: producer 安装 crate 文件清单为空: {name}").into());
    }
    let root = producer_root.join("crates").join(name);
    let mut seen = std::collections::BTreeSet::new();
    fn visit(
        dir: &Path,
        rel: &str,
        prefix: &str,
        expected: &std::collections::BTreeMap<&String, &serde_json::Value>,
        seen: &mut std::collections::BTreeSet<String>,
    ) -> AutoResult<()> {
        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let ty = entry.file_type()?;
            let name = entry.file_name().to_string_lossy().to_string();
            let path = entry.path();
            let child = if rel.is_empty() {
                name.clone()
            } else {
                format!("{rel}/{name}")
            };
            if ty.is_symlink() {
                return Err(
                    format!("delivery: producer 安装源包含链接: {}", path.display()).into(),
                );
            }
            if ty.is_dir() {
                if ["target", ".git", "node_modules"].contains(&name.as_str()) {
                    continue;
                }
                visit(&path, &child, prefix, expected, seen)?;
            } else if ty.is_file() {
                let key = format!("{prefix}{child}");
                let record = expected
                    .get(&key)
                    .ok_or_else(|| format!("delivery: producer 安装 crate 文件未登记: {key}"))?;
                let bytes = fs::read(&path)?;
                let hash = format!("{:x}", Sha256::digest(&bytes));
                if record["bytes"].as_u64() != Some(bytes.len() as u64)
                    || record["sha256"].as_str() != Some(hash.as_str())
                {
                    return Err(format!("delivery: producer 安装 crate 文件校验失败: {key}").into());
                }
                seen.insert(key);
            } else {
                return Err(format!(
                    "delivery: producer 安装 crate 源文件类型无效: {}",
                    path.display()
                )
                .into());
            }
        }
        Ok(())
    }
    // Validate the staged copy too: the source may change between the first
    // check and copying, so a preflight-only hash is not output provenance.
    visit(
        copied_root.unwrap_or(&root),
        "",
        &prefix,
        &expected,
        &mut seen,
    )?;
    if seen.len() != expected.len() {
        return Err(format!("delivery: producer 安装 crate 源缺少登记文件: {name}").into());
    }
    Ok(Some(root))
}

/// Verify UI source ownership and bytes against a managed installation.
pub fn installed_ui_source(
    producer_root: &Path,
    name: &str,
    copied_root: Option<&Path>,
) -> AutoResult<Option<PathBuf>> {
    installed_crate_source(producer_root, name, copied_root, "auto-ui")
}

/// The SDK adjacent to this executable remains a managed installation even
/// when its receipt is missing; explicit overrides cannot reclassify it as a
/// free-standing development source.
pub fn is_managed_sdk(producer_root: &Path) -> bool {
    if producer_root.file_name().and_then(|s| s.to_str()) != Some("sdk") {
        return false;
    }
    if producer_root.parent().is_some_and(|parent| {
        parent.join("install-manifest.json").exists()
            || (parent
                .join(format!("auto{}", std::env::consts::EXE_SUFFIX))
                .is_file()
                && parent
                    .join(format!("auto-ui-tool{}", std::env::consts::EXE_SUFFIX))
                    .is_file())
    }) {
        return true;
    }
    std::env::current_exe()
        .ok()
        .and_then(|exe| exe.parent().map(|p| p.join("sdk")))
        .and_then(|adjacent| {
            Some((
                fs::canonicalize(adjacent).ok()?,
                fs::canonicalize(producer_root).ok()?,
            ))
        })
        .is_some_and(|(adjacent, source)| adjacent == source)
}

/// Frozen consistency receipt for one managed Full SDK. Hashes are integrity
/// checks against the local receipt; they are not signatures.
pub struct InstalledSdkReceipt {
    path: PathBuf,
    raw: Vec<u8>,
    doc: serde_json::Value,
}

impl InstalledSdkReceipt {
    pub fn open(producer_root: &Path) -> AutoResult<Option<Self>> {
        if producer_root.file_name().and_then(|s| s.to_str()) != Some("sdk") {
            return Ok(None);
        }
        let Some(parent) = producer_root.parent() else {
            return Ok(None);
        };
        let path = parent.join("install-manifest.json");
        if !path.is_file() {
            return Ok(None);
        }
        let raw = fs::read(&path).map_err(|e| format!("producer 安装清单不可读: {e}"))?;
        let doc: serde_json::Value =
            serde_json::from_slice(&raw).map_err(|e| format!("producer 安装清单无效: {e}"))?;
        let rev = doc["sources"]["auto-lang"].as_str().unwrap_or("");
        if doc["schema"] != "man-install/1"
            || doc["profile"] != "full"
            || doc["source_closure"]["sdk_layout"] != "combined-crates"
            || rev.len() != 40
            || !rev.bytes().all(|b| b.is_ascii_hexdigit())
            || !doc["files"].is_object()
            || !doc["source_closure"]["packages"].is_array()
        {
            return Err("producer 安装 SDK 清单版本、来源或闭包无效".into());
        }
        Ok(Some(Self { path, raw, doc }))
    }

    pub fn verify_bytes(&self, rel: &str, bytes: &[u8]) -> AutoResult<()> {
        use sha2::{Digest, Sha256};
        let key = format!("sdk/{rel}");
        let record = &self.doc["files"][&key];
        let hash = format!("{:x}", Sha256::digest(bytes));
        if record["bytes"].as_u64() != Some(bytes.len() as u64)
            || record["sha256"].as_str() != Some(hash.as_str())
        {
            return Err(format!("producer 安装 SDK 文件未登记或校验失败: {key}").into());
        }
        Ok(())
    }

    pub fn require_package(&self, name: &str, owner: &str, rel: &str) -> AutoResult<()> {
        let matches: Vec<_> = self.doc["source_closure"]["packages"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|package| package["name"] == name)
            .collect();
        if matches.len() != 1
            || matches[0]["source_repo"] != owner
            || matches[0]["source_path"] != rel
            || matches[0]["installed_path"] != rel
        {
            return Err(format!("producer 安装 SDK 包所有权无效: {rel}").into());
        }
        Ok(())
    }

    /// Check every recorded file and reject unrecorded files in the selected
    /// source or staged subtree, excluding build caches that are never copied.
    pub fn verify_tree(&self, rel: &str, root: &Path) -> AutoResult<()> {
        let prefix = if rel.is_empty() {
            "sdk/".to_owned()
        } else {
            format!("sdk/{rel}/")
        };
        let expected: std::collections::BTreeSet<String> = self.doc["files"]
            .as_object()
            .unwrap()
            .keys()
            .filter(|path| path.starts_with(&prefix))
            .cloned()
            .collect();
        if expected.is_empty() {
            return Err(format!("producer 安装 SDK 文件清单为空: {prefix}").into());
        }
        let mut seen = std::collections::BTreeSet::new();
        fn walk(
            receipt: &InstalledSdkReceipt,
            path: &Path,
            rel: &str,
            seen: &mut std::collections::BTreeSet<String>,
        ) -> AutoResult<()> {
            let entries = fs::read_dir(path)
                .map_err(|e| format!("producer 安装 SDK 目录缺失 {}: {e}", path.display()))?;
            for entry in entries {
                let entry = entry?;
                let ty = entry.file_type()?;
                let name = entry.file_name().to_string_lossy().to_string();
                let child = if rel.is_empty() {
                    name.clone()
                } else {
                    format!("{rel}/{name}")
                };
                if ty.is_symlink() {
                    return Err(
                        format!("producer 安装 SDK 含链接: {}", entry.path().display()).into(),
                    );
                }
                if ty.is_dir() {
                    if ["target", ".git", "node_modules"].contains(&name.as_str()) {
                        continue;
                    }
                    walk(receipt, &entry.path(), &child, seen)?;
                } else if ty.is_file() {
                    let bytes = fs::read(entry.path())?;
                    receipt.verify_bytes(&child, &bytes)?;
                    seen.insert(format!("sdk/{child}"));
                } else {
                    return Err(format!(
                        "producer 安装 SDK 文件类型无效: {}",
                        entry.path().display()
                    )
                    .into());
                }
            }
            Ok(())
        }
        walk(self, root, rel, &mut seen)?;
        if seen != expected {
            return Err(format!("producer 安装 SDK 文件缺失或额外: {prefix}").into());
        }
        Ok(())
    }

    pub fn ensure_unchanged(&self) -> AutoResult<()> {
        if fs::read(&self.path).ok().as_deref() != Some(self.raw.as_slice()) {
            return Err("producer 安装清单在生成期间发生变化".into());
        }
        Ok(())
    }
}

/// A detected managed SDK is authoritative: validate its workspace manifest
/// and language packages against its receipt before generating any output.
pub fn validate_installed_language_source(producer_root: &Path) -> AutoResult<()> {
    let receipt = InstalledSdkReceipt::open(producer_root)?.ok_or("producer 安装 SDK 清单缺失")?;
    receipt.verify_tree("", producer_root)?;
    for name in ["auto-lang", "auto-lang-core", "a2r-std", "auto-media-core"] {
        let source = installed_crate_source(producer_root, name, None, "auto-lang")
            .map_err(|err| format!("producer 安装语言源校验失败 {name}: {err}"))?
            .ok_or("producer 安装 SDK 清单缺失")?;
        if !valid_source_crate(&source, name) {
            return Err(format!("producer 安装语言包无效: crates/{name}").into());
        }
    }
    receipt.ensure_unchanged()
}

// A directory alone is not a usable Cargo package. Validate before callers
// write generated workspaces or stage a delivery, including explicit roots.
pub(crate) fn valid_source_crate(source: &Path, name: &str) -> bool {
    let Some(manifest) = std::fs::read_to_string(source.join("Cargo.toml"))
        .ok()
        .and_then(|text| text.parse::<toml::Value>().ok())
    else {
        return false;
    };
    if manifest
        .get("package")
        .and_then(|package| package.get("name"))
        .and_then(toml::Value::as_str)
        != Some(name)
    {
        return false;
    }
    let lib_path = manifest
        .get("lib")
        .and_then(|lib| lib.get("path"))
        .and_then(toml::Value::as_str)
        .unwrap_or("src/lib.rs");
    source.join(lib_path).is_file()
}

/// Validate local Cargo path dependencies for an unmanaged development
/// source before generating a workspace. Cargo resolves local paths even when
/// optional; registry dependencies and ordinary source edits remain allowed.
pub fn validate_development_dependencies(source: &Path) -> AutoResult<()> {
    use std::collections::{BTreeMap, BTreeSet};
    fn workspace(dir: &Path, manifest: &toml::Value) -> Option<(PathBuf, toml::Value)> {
        if let Some(relative) = manifest
            .get("package")
            .and_then(|p| p.get("workspace"))
            .and_then(toml::Value::as_str)
        {
            let root = dir.join(relative);
            let doc = fs::read_to_string(root.join("Cargo.toml"))
                .ok()?
                .parse::<toml::Value>()
                .ok()?;
            return doc.get("workspace").map(|_| (root.clone(), doc.clone()));
        }
        for parent in dir.ancestors().skip(1) {
            if let Ok(text) = fs::read_to_string(parent.join("Cargo.toml")) {
                if let Ok(doc) = text.parse::<toml::Value>() {
                    if doc.get("workspace").is_some() {
                        return Some((parent.to_path_buf(), doc));
                    }
                }
            }
        }
        None
    }
    fn read_package(dir: &Path) -> AutoResult<(PathBuf, toml::Value)> {
        let canonical = fs::canonicalize(dir)
            .map_err(|e| format!("producer 路径依赖缺失 {}: {e}", dir.display()))?;
        let text = fs::read_to_string(canonical.join("Cargo.toml"))
            .map_err(|e| format!("producer 路径依赖缺清单 {}: {e}", canonical.display()))?;
        let doc: toml::Value = text
            .parse()
            .map_err(|e| format!("producer 路径依赖清单无效 {}: {e}", canonical.display()))?;
        let name = doc
            .get("package")
            .and_then(|p| p.get("name"))
            .and_then(toml::Value::as_str)
            .ok_or_else(|| format!("producer 路径依赖缺包名: {}", canonical.display()))?;
        if !valid_source_crate(&canonical, name) {
            return Err(format!("producer 路径依赖缺库入口: {}", canonical.display()).into());
        }
        Ok((canonical, doc))
    }
    fn package_version(dir: &Path, doc: &toml::Value) -> AutoResult<semver::Version> {
        let value = doc.get("package").and_then(|p| p.get("version"));
        let text = match value {
            None => "0.0.0".to_owned(),
            Some(value) if value.as_str().is_some() => value.as_str().unwrap().to_owned(),
            Some(value) if value.get("workspace").and_then(toml::Value::as_bool) == Some(true) => {
                let (_, ws) = workspace(dir, doc)
                    .ok_or_else(|| format!("producer 路径包版本缺工作区: {}", dir.display()))?;
                ws.get("workspace")
                    .and_then(|w| w.get("package"))
                    .and_then(|p| p.get("version"))
                    .and_then(toml::Value::as_str)
                    .ok_or("producer 工作区包版本缺失")?
                    .to_owned()
            }
            _ => return Err(format!("producer 路径包版本无效: {}", dir.display()).into()),
        };
        semver::Version::parse(&text)
            .map_err(|e| format!("producer 路径包版本无效 {text}: {e}").into())
    }
    fn walk(dir: &Path, graph: &mut BTreeMap<PathBuf, Vec<PathBuf>>) -> AutoResult<()> {
        let (canonical, doc) = read_package(dir)?;
        if graph.contains_key(&canonical) {
            return Ok(());
        }
        let ws = workspace(&canonical, &doc);
        let mut owners = vec![&doc];
        if let Some(targets) = doc.get("target").and_then(toml::Value::as_table) {
            owners.extend(targets.values());
        }
        let mut all = Vec::new();
        let mut normal = Vec::new();
        for owner in owners {
            for section in ["dependencies", "dev-dependencies", "build-dependencies"] {
                let Some(deps) = owner.get(section).and_then(toml::Value::as_table) else {
                    continue;
                };
                for (key, spec) in deps {
                    let (base, resolved) =
                        if spec.get("workspace").and_then(toml::Value::as_bool) == Some(true) {
                            let (root, ws_doc) = ws
                                .as_ref()
                                .ok_or_else(|| format!("producer 路径依赖缺工作区: {key}"))?;
                            let value = ws_doc
                                .get("workspace")
                                .and_then(|w| w.get("dependencies"))
                                .and_then(|d| d.get(key))
                                .ok_or_else(|| format!("producer 工作区依赖缺声明: {key}"))?;
                            (root.as_path(), value)
                        } else {
                            (canonical.as_path(), spec)
                        };
                    let Some(path) = resolved.get("path").and_then(toml::Value::as_str) else {
                        continue;
                    };
                    let (target, target_doc) = read_package(&base.join(path))?;
                    let actual = target_doc["package"]["name"].as_str().unwrap();
                    let expected = resolved
                        .get("package")
                        .and_then(toml::Value::as_str)
                        .unwrap_or(key);
                    if actual != expected {
                        return Err(format!(
                            "producer 路径依赖 {key} 包身份不符：期望 {expected}，实际 {actual}"
                        )
                        .into());
                    }
                    if let Some(version) = resolved.get("version").and_then(toml::Value::as_str) {
                        let req = semver::VersionReq::parse(version)
                            .map_err(|e| format!("producer 依赖 {key} 版本约束无效: {e}"))?;
                        let actual = package_version(&target, &target_doc)?;
                        if !req.matches(&actual) {
                            return Err(format!(
                                "producer 路径依赖 {key} 版本不符：{version} / {actual}"
                            )
                            .into());
                        }
                    }
                    if section != "dev-dependencies" {
                        normal.push(target.clone());
                    }
                    all.push(target);
                }
            }
        }
        graph.insert(canonical, normal);
        for target in all {
            walk(&target, graph)?;
        }
        Ok(())
    }
    fn visit(
        node: &PathBuf,
        graph: &BTreeMap<PathBuf, Vec<PathBuf>>,
        active: &mut BTreeSet<PathBuf>,
        done: &mut BTreeSet<PathBuf>,
    ) -> AutoResult<()> {
        if done.contains(node) {
            return Ok(());
        }
        if !active.insert(node.clone()) {
            return Err(format!("producer 普通/构建路径依赖存在循环: {}", node.display()).into());
        }
        if let Some(edges) = graph.get(node) {
            for edge in edges {
                visit(edge, graph, active, done)?;
            }
        }
        active.remove(node);
        done.insert(node.clone());
        Ok(())
    }
    let mut graph = BTreeMap::new();
    walk(source, &mut graph)?;
    let mut done = BTreeSet::new();
    for node in graph.keys() {
        visit(node, &graph, &mut BTreeSet::new(), &mut done)?;
    }
    Ok(())
}

/// UI crates are selected from their UI owner or a receipt-validated
/// installation. Other crates must come from the language producer itself.
pub fn ui_crate_source(producer_root: &Path, name: &str) -> AutoResult<PathBuf> {
    let ui = matches!(
        name,
        "auto-ui-dialect" | "auto-ui-compiler" | "auto-ui-runtime" | "auto-ui-tools" | "shell-pack"
    );
    if !ui {
        let source = producer_root.join("crates").join(name);
        return source
            .is_dir()
            .then_some(source)
            .ok_or_else(|| format!("delivery: producer 语言源缺失: crates/{name}").into());
    }
    if let Some(value) = std::env::var_os("AUTO_UI_SOURCE_ROOT") {
        let root = PathBuf::from(value);
        let source = root.join("crates").join(name);
        if !valid_source_crate(&source, name) {
            return Err(format!(
                "delivery: producer 显式 AUTO_UI_SOURCE_ROOT 无效或缺少 crates/{name}: {}",
                root.display()
            )
            .into());
        }
        if is_managed_sdk(&root) {
            validate_installed_language_source(&root)?;
            installed_ui_source(&root, name, None)?.ok_or("producer 显式 UI 安装清单缺失")?;
        } else {
            validate_development_dependencies(&source).map_err(|err| {
                format!("producer 显式 AUTO_UI_SOURCE_ROOT 的本地依赖无效: {err}")
            })?;
        }
        return Ok(source);
    }
    if let Some(source) = installed_ui_source(producer_root, name, None)? {
        if valid_source_crate(&source, name) {
            return Ok(source);
        }
        return Err(format!("delivery: producer 安装 UI 包无效: crates/{name}").into());
    }
    if let Some(parent) = producer_root.parent() {
        let source = parent.join("auto-ui/crates").join(name);
        if source.is_dir() {
            if valid_source_crate(&source, name) {
                validate_development_dependencies(&source)
                    .map_err(|err| format!("producer 组内 UI 包依赖无效: {err}"))?;
                return Ok(source);
            }
            return Err(format!("delivery: producer 组内 UI 包无效: crates/{name}").into());
        }
    }
    Err(format!(
        "delivery: producer UI owner 源缺失: crates/{name}（AUTO_UI_SOURCE_ROOT / 组内 auto-ui）"
    )
    .into())
}

#[cfg(test)]
mod plan842_receipt_tests {
    use super::*;
    use sha2::{Digest, Sha256};

    #[test]
    fn developer_source_checks_inherited_target_paths_and_cycles() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        fs::write(
            root.join("Cargo.toml"),
            "[workspace]\n[workspace.dependencies]\nshared = { path = \"crates/shared\" }\n",
        )
        .unwrap();
        let make = |name: &str, extra: &str| {
            let path = root.join("crates").join(name);
            fs::create_dir_all(path.join("src")).unwrap();
            fs::write(path.join("src/lib.rs"), "").unwrap();
            fs::write(
                path.join("Cargo.toml"),
                format!("[package]\nname = \"{name}\"\nversion = \"0.1.0\"\n{extra}"),
            )
            .unwrap();
        };
        make("app", "[dependencies]\nshared.workspace = true\n[target.'cfg(windows)'.dependencies]\nextra = { path = \"../extra\" }\n");
        make("shared", "[dependencies]\napp = { path = \"../app\" }\n");
        assert!(
            validate_development_dependencies(&root.join("crates/app")).is_err(),
            "missing target path"
        );
        make("extra", "");
        assert!(
            validate_development_dependencies(&root.join("crates/app")).is_err(),
            "ordinary cycle must be rejected"
        );
        make(
            "shared",
            "[dev-dependencies]\napp = { path = \"../app\" }\n",
        );
        validate_development_dependencies(&root.join("crates/app")).unwrap();
    }

    #[test]
    fn developer_source_checks_package_identity_alias_and_version() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        fs::write(root.join("Cargo.toml"), "[workspace]\n[workspace.package]\nversion = \"1.2.3\"\n[workspace.dependencies]\nrenamed = { package = \"shared\", path = \"crates/shared\", version = \"^1.2\" }\n").unwrap();
        for name in ["app", "shared", "other"] {
            let path = root.join("crates").join(name);
            fs::create_dir_all(path.join("src")).unwrap();
            fs::write(path.join("src/lib.rs"), "").unwrap();
            fs::write(
                path.join("Cargo.toml"),
                format!("[package]\nname = \"{name}\"\nversion.workspace = true\n"),
            )
            .unwrap();
        }
        let app = root.join("crates/app/Cargo.toml");
        let header = "[package]\nname = \"app\"\nversion.workspace = true\n[dependencies]\n";
        fs::write(&app, format!("{header}renamed.workspace = true\n")).unwrap();
        validate_development_dependencies(&root.join("crates/app")).unwrap();
        fs::write(
            &app,
            format!("{header}renamed = {{ package = \"shared\", path = \"../other\" }}\n"),
        )
        .unwrap();
        assert!(
            validate_development_dependencies(&root.join("crates/app")).is_err(),
            "wrong package identity"
        );
        fs::write(&app, format!("{header}renamed = {{ package = \"shared\", path = \"../shared\", version = \"=9.9.9\" }}\n")).unwrap();
        assert!(
            validate_development_dependencies(&root.join("crates/app")).is_err(),
            "wrong package version"
        );
    }

    #[test]
    fn detached_full_install_sdk_is_managed_even_without_receipt() {
        let tmp = tempfile::tempdir().unwrap();
        let sdk = tmp.path().join("sdk");
        fs::create_dir_all(&sdk).unwrap();
        assert!(
            !is_managed_sdk(&sdk),
            "standalone development source remains supported"
        );
        fs::write(
            tmp.path()
                .join(format!("auto{}", std::env::consts::EXE_SUFFIX)),
            "",
        )
        .unwrap();
        fs::write(
            tmp.path()
                .join(format!("auto-ui-tool{}", std::env::consts::EXE_SUFFIX)),
            "",
        )
        .unwrap();
        assert!(
            is_managed_sdk(&sdk),
            "a Full install remains managed when its receipt is missing"
        );
        assert!(validate_installed_language_source(&sdk).is_err());
    }

    #[test]
    fn installed_sdk_rejects_transitive_changes_missing_and_extra() {
        let tmp = tempfile::tempdir().unwrap();
        let sdk = tmp.path().join("sdk");
        fs::create_dir_all(&sdk).unwrap();
        fs::write(sdk.join("Cargo.toml"), "[workspace]\n").unwrap();
        let mut packages = Vec::new();
        for name in [
            "auto-lang",
            "auto-lang-core",
            "a2r-std",
            "auto-media-core",
            "auto-val",
        ] {
            let dir = sdk.join("crates").join(name);
            fs::create_dir_all(dir.join("src")).unwrap();
            fs::write(
                dir.join("Cargo.toml"),
                format!("[package]\nname = \"{name}\"\nversion = \"0.1.0\"\n"),
            )
            .unwrap();
            fs::write(dir.join("src/lib.rs"), "// initial\n").unwrap();
            let path = format!("crates/{name}");
            packages.push(serde_json::json!({"name":name, "source_repo":"auto-lang", "source_path":path, "installed_path":path}));
        }
        let mut files = serde_json::Map::new();
        fn add(dir: &Path, install: &Path, files: &mut serde_json::Map<String, serde_json::Value>) {
            for entry in fs::read_dir(dir).unwrap() {
                let path = entry.unwrap().path();
                if path.is_dir() {
                    add(&path, install, files);
                } else {
                    let bytes = fs::read(&path).unwrap();
                    let rel = path
                        .strip_prefix(install)
                        .unwrap()
                        .to_string_lossy()
                        .replace(char::from(92), "/");
                    files.insert(rel, serde_json::json!({"bytes":bytes.len(), "sha256":format!("{:x}", Sha256::digest(&bytes))}));
                }
            }
        }
        add(&sdk, tmp.path(), &mut files);
        let receipt = serde_json::json!({"schema":"man-install/1", "profile":"full", "sources":{"auto-lang":"0123456789abcdef0123456789abcdef01234567"}, "source_closure":{"sdk_layout":"combined-crates", "packages":packages}, "files":files});
        fs::write(
            tmp.path().join("install-manifest.json"),
            serde_json::to_vec(&receipt).unwrap(),
        )
        .unwrap();
        validate_installed_language_source(&sdk).unwrap();
        let frozen = InstalledSdkReceipt::open(&sdk).unwrap().unwrap();
        let staged = tmp.path().join("staged-auto-val");
        fs::create_dir_all(staged.join("src")).unwrap();
        fs::copy(
            sdk.join("crates/auto-val/Cargo.toml"),
            staged.join("Cargo.toml"),
        )
        .unwrap();
        fs::copy(
            sdk.join("crates/auto-val/src/lib.rs"),
            staged.join("src/lib.rs"),
        )
        .unwrap();
        frozen.verify_tree("crates/auto-val", &staged).unwrap();
        fs::write(staged.join("src/lib.rs"), b"changed after preflight").unwrap();
        assert!(
            frozen.verify_tree("crates/auto-val", &staged).is_err(),
            "staged drift"
        );
        fs::copy(
            sdk.join("crates/auto-val/src/lib.rs"),
            staged.join("src/lib.rs"),
        )
        .unwrap();
        fs::write(staged.join("src/unlisted.rs"), b"extra").unwrap();
        assert!(
            frozen.verify_tree("crates/auto-val", &staged).is_err(),
            "staged extra"
        );
        fs::remove_file(staged.join("src/unlisted.rs")).unwrap();
        fs::remove_file(staged.join("src/lib.rs")).unwrap();
        assert!(
            frozen.verify_tree("crates/auto-val", &staged).is_err(),
            "staged missing"
        );
        frozen.ensure_unchanged().unwrap();
        fs::write(tmp.path().join("install-manifest.json"), b"changed").unwrap();
        assert!(frozen.ensure_unchanged().is_err(), "receipt drift");
        fs::write(
            tmp.path().join("install-manifest.json"),
            serde_json::to_vec(&receipt).unwrap(),
        )
        .unwrap();
        let file = sdk.join("crates/auto-val/src/lib.rs");
        let original = fs::read(&file).unwrap();
        fs::write(&file, b"// tampered\n").unwrap();
        assert!(
            validate_installed_language_source(&sdk).is_err(),
            "transitive source changed"
        );
        fs::write(&file, &original).unwrap();
        fs::remove_file(&file).unwrap();
        assert!(
            validate_installed_language_source(&sdk).is_err(),
            "transitive source missing"
        );
        fs::write(&file, &original).unwrap();
        let extra = sdk.join("crates/auto-val/src/unlisted.rs");
        fs::write(&extra, b"unlisted").unwrap();
        assert!(
            validate_installed_language_source(&sdk).is_err(),
            "transitive source extra"
        );
        fs::remove_file(extra).unwrap();
        validate_installed_language_source(&sdk).unwrap();
    }
}
