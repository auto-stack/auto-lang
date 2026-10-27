//! Shared source-owner validation for generated Rust consumers.
use auto_val::AutoResult;
use std::{
    fs,
    path::{Path, PathBuf},
};

/// Verify the actual UI crate files against a managed Full installation's
/// ownership and content receipt. This is consistency checking, not a signature.
pub fn installed_ui_source(
    producer_root: &Path,
    name: &str,
    copied_root: Option<&Path>,
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
    let ui_rev = doc["sources"]["auto-ui"].as_str().unwrap_or("");
    if doc["schema"] != "man-install/1"
        || doc["profile"] != "full"
        || doc["source_closure"]["sdk_layout"] != "combined-crates"
        || ui_rev.len() != 40
        || !ui_rev.bytes().all(|b| b.is_ascii_hexdigit())
    {
        return Err("delivery: producer 安装清单版本、profile 或 UI 来源无效".into());
    }
    let packages = doc["source_closure"]["packages"]
        .as_array()
        .ok_or("delivery: producer 安装清单缺 source_closure.packages")?;
    let owners: Vec<_> = packages.iter().filter(|p| p["name"] == name).collect();
    let expected_path = format!("crates/{name}");
    if owners.len() != 1
        || owners[0]["source_repo"] != "auto-ui"
        || owners[0]["installed_path"] != expected_path
        || owners[0]["source_path"] != expected_path
    {
        return Err(format!("delivery: producer 安装 UI 源所有权无效: {name}").into());
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
        return Err(format!("delivery: producer 安装 UI 文件清单为空: {name}").into());
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
                    .ok_or_else(|| format!("delivery: producer 安装 UI 文件未登记: {key}"))?;
                let bytes = fs::read(&path)?;
                let hash = format!("{:x}", Sha256::digest(&bytes));
                if record["bytes"].as_u64() != Some(bytes.len() as u64)
                    || record["sha256"].as_str() != Some(hash.as_str())
                {
                    return Err(format!("delivery: producer 安装 UI 文件校验失败: {key}").into());
                }
                seen.insert(key);
            } else {
                return Err(format!(
                    "delivery: producer 安装 UI 源文件类型无效: {}",
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
        return Err(format!("delivery: producer 安装 UI 源缺少登记文件: {name}").into());
    }
    Ok(Some(root))
}

// A directory alone is not a usable Cargo package. Validate before callers
// write generated workspaces or stage a delivery, including explicit roots.
fn valid_ui_crate(source: &Path, name: &str) -> bool {
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
        return valid_ui_crate(&source, name)
            .then_some(source)
            .ok_or_else(|| {
                format!(
                    "delivery: producer 显式 AUTO_UI_SOURCE_ROOT 无效或缺少 crates/{name}: {}",
                    root.display()
                )
                .into()
            });
    }
    if let Some(source) = installed_ui_source(producer_root, name, None)? {
        if valid_ui_crate(&source, name) {
            return Ok(source);
        }
        return Err(format!("delivery: producer 安装 UI 包无效: crates/{name}").into());
    }
    if let Some(parent) = producer_root.parent() {
        let source = parent.join("auto-ui/crates").join(name);
        if source.is_dir() {
            if valid_ui_crate(&source, name) {
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
