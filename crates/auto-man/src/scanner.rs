//! 统一的目录扫描逻辑
//!
//! 提供跨 Target 和 Dir 复用的目录扫描功能

use auto_val::{AutoStr, PathBufExt};
use crate::TargetKind;
use crate::AutoResult;
use std::collections::HashSet;
use std::path::Path;

/// 扫描配置
///
/// 控制目录扫描行为的参数
#[derive(Debug, Clone)]
pub struct ScanConfig {
    /// 是否递归扫描子目录
    pub recurse: bool,
    /// 是否将头文件识别为源文件
    pub show_headers: bool,
    /// 是否扫描标准子目录（src/, include/, inc/）
    pub scan_std_dirs: bool,
    /// 跳过的文件/目录集合
    pub skips: HashSet<AutoStr>,
}

impl ScanConfig {
    /// 创建默认扫描配置
    pub fn new() -> Self {
        Self {
            recurse: false,
            show_headers: false,
            scan_std_dirs: true,
            skips: HashSet::new(),
        }
    }
}

impl Default for ScanConfig {
    fn default() -> Self {
        Self::new()
    }
}

/// 扫描结果
///
/// 包含扫描后发现的源文件和包含目录
#[derive(Debug, Clone)]
pub struct ScanResult {
    /// 发现的源文件集合
    pub sources: HashSet<AutoStr>,
    /// 发现的包含目录集合
    pub includes: HashSet<AutoStr>,
    /// 是否发现了头文件
    pub has_headers: bool,
}

impl ScanResult {
    /// 创建空的扫描结果
    pub fn new() -> Self {
        Self {
            sources: HashSet::new(),
            includes: HashSet::new(),
            has_headers: false,
        }
    }
}

impl Default for ScanResult {
    fn default() -> Self {
        Self::new()
    }
}

/// 扫描指定目录
///
/// 这是核心扫描函数，将来会包含 `scan_dir_flat` 的逻辑
///
/// # 参数
/// - `path`: 要扫描的目录路径
/// - `kind`: 目标类型
/// - `config`: 扫描配置
///
/// # 返回
/// 扫描结果，包含源文件和包含目录
pub fn scan_directory(
    _path: &Path,
    _kind: &TargetKind,
    _config: &ScanConfig,
) -> AutoResult<ScanResult> {
    // TODO: Phase 2 实现
    // 这个函数会在 Phase 2 中完整实现
    // 目前返回空结果以让代码编译通过
    Ok(ScanResult::new())
}

/// 扫描标准子目录（src, include, inc）
///
/// 提取 `Dir::scan_dir()` 中的公共逻辑
///
/// 这个函数扫描三个标准子目录：src/, include/, inc/
///
/// # 参数
/// - `root`: 根目录路径
/// - `scan_dir_flat`: 扫描函数（来自 dir.rs）
/// - `target_kind`: 目标类型
/// - `skips`: 跳过的文件集合
/// - `show_headers`: 是否将头文件识别为源文件
///
/// # 返回
/// 扫描结果，包含所有标准子目录的源文件和包含目录
pub fn scan_standard_subdirs<F>(
    root: &Path,
    target_kind: &TargetKind,
    skips: &HashSet<AutoStr>,
    show_headers: bool,
    scan_dir_flat: F,
) -> AutoResult<ScanResult>
where
    F: Fn(&Path, &TargetKind, &HashSet<AutoStr>, bool) -> AutoResult<(HashSet<AutoStr>, bool)>,
{
    let mut all_srcs = HashSet::new();
    let mut all_incs = HashSet::new();

    // 扫描标准子目录列表
    let std_dirs = ["src", "include", "inc"];

    for subdir_name in &std_dirs {
        let subdir_path = root.join(subdir_name);

        // 只有当目录存在时才扫描
        if subdir_path.is_dir() {
            let (files, has_headers) = scan_dir_flat(&subdir_path, target_kind, skips, show_headers)?;
            all_srcs.extend(files);

            if has_headers {
                all_incs.insert(subdir_path.unified());
            }
        }
    }

    Ok(ScanResult {
        sources: all_srcs,
        includes: all_incs.clone(),
        has_headers: !all_incs.is_empty(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scan_config_new() {
        let config = ScanConfig::new();
        assert!(!config.recurse);
        assert!(!config.show_headers);
        assert!(config.scan_std_dirs);
        assert!(config.skips.is_empty());
    }

    #[test]
    fn test_scan_config_default() {
        let config = ScanConfig::default();
        assert!(!config.recurse);
        assert!(!config.show_headers);
        assert!(config.scan_std_dirs);
        assert!(config.skips.is_empty());
    }

    #[test]
    fn test_scan_result_new() {
        let result = ScanResult::new();
        assert!(result.sources.is_empty());
        assert!(result.includes.is_empty());
        assert!(!result.has_headers);
    }

    #[test]
    fn test_scan_result_default() {
        let result = ScanResult::default();
        assert!(result.sources.is_empty());
        assert!(result.includes.is_empty());
        assert!(!result.has_headers);
    }

    // TODO: Phase 2 添加更多测试
    // - 测试扫描空目录
    // - 测试扫描混合文件
    // - 测试扫描跳过逻辑
    // - 测试标准子目录扫描
}
