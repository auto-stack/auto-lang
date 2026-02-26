//! 文件类型注册表
//!
//! 集中管理硬编码的文件扩展名检查逻辑，提供类型安全的文件类型识别。

use crate::TargetKind;
use auto_val::AutoStr;
use std::collections::HashSet;

/// 文件类型枚举
///
/// 定义所有支持的源文件和头文件类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FileType {
    /// Auto 语言源文件 (.at)
    Auto,
    /// C 源文件 (.c)
    CSource,
    /// C 头文件 (.h)
    CHeader,
    /// 汇编源文件 (.s, .S)
    Asm,
    /// GHS 项目文件 (.gpj)
    GhsProject,
    /// RH850 汇编文件 (.850)
    Rh850,
    /// Rust 源文件 (.rs)
    RustSource,
}

impl FileType {
    /// 获取此文件类型的所有扩展名
    pub fn extensions(&self) -> &[&str] {
        match self {
            FileType::Auto => &["at"],
            FileType::CSource => &["c"],
            FileType::CHeader => &["h"],
            FileType::Asm => &["s", "S"],
            FileType::GhsProject => &["gpj"],
            FileType::Rh850 => &["850"],
            FileType::RustSource => &["rs"],
        }
    }

    /// 从扩展名获取文件类型
    ///
    /// 如果扩展名不匹配任何已知类型，返回 None
    pub fn from_extension(ext: &str) -> Option<Self> {
        match ext {
            "at" => Some(FileType::Auto),
            "c" => Some(FileType::CSource),
            "h" => Some(FileType::CHeader),
            "s" | "S" => Some(FileType::Asm),
            "gpj" => Some(FileType::GhsProject),
            "850" => Some(FileType::Rh850),
            "rs" => Some(FileType::RustSource),
            _ => None,
        }
    }

    /// 判断是否是头文件类型
    pub fn is_header(&self) -> bool {
        matches!(self, FileType::CHeader)
    }

    /// 判断是否是源文件类型
    pub fn is_source(&self) -> bool {
        matches!(
            self,
            FileType::Auto
                | FileType::CSource
                | FileType::Asm
                | FileType::Rh850
                | FileType::RustSource
        )
    }
}

/// 文件过滤器
///
/// 根据目标类型和配置过滤文件
#[derive(Debug, Clone)]
pub struct FileFilter {
    /// 包含的源文件类型
    pub source_types: Vec<FileType>,
    /// 排除的文件名（不含路径）
    pub exclude_files: HashSet<AutoStr>,
}

impl FileFilter {
    /// 为特定目标类型创建文件过滤器
    pub fn for_target(kind: &TargetKind) -> Self {
        let source_types = match kind {
            // Device 目标包含所有文件类型
            TargetKind::Device => vec![
                FileType::Auto,
                FileType::CSource,
                FileType::CHeader,
                FileType::Asm,
                FileType::GhsProject,
                FileType::Rh850,
            ],
            // 其他目标只包含标准源文件类型
            _ => vec![
                FileType::Auto,
                FileType::CSource,
                FileType::CHeader,
                FileType::Asm,
                FileType::Rh850,
                FileType::RustSource,
            ],
        };

        // 总是排除配置文件
        let mut exclude_files = HashSet::new();
        exclude_files.insert("pac.at".into());
        exclude_files.insert("device.at".into());

        Self {
            source_types,
            exclude_files,
        }
    }

    /// 判断是否应该包含此文件
    ///
    /// # 参数
    /// - `filename`: 文件名（不含路径）
    /// - `ext`: 文件扩展名（可选）
    ///
    /// # 返回
    /// - `true`: 应该包含此文件
    /// - `false`: 应该排除此文件
    pub fn should_include(&self, filename: &str, ext: Option<&str>) -> bool {
        // 检查是否在排除列表中
        if self.exclude_files.iter().any(|f| f.as_str() == filename) {
            return false;
        }

        // 如果没有扩展名，总是排除（除非将来有特殊需求）
        let ext = match ext {
            Some(e) => e,
            None => return false,
        };

        // 检查扩展名是否匹配支持的文件类型
        if let Some(file_type) = FileType::from_extension(ext) {
            // GhsProject 文件总是被排除（它们是项目文件，不是源文件）
            if file_type == FileType::GhsProject {
                return false;
            }

            // 检查文件类型是否在支持列表中
            self.source_types.contains(&file_type)
        } else {
            // 未知扩展名，总是排除
            false
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_file_type_extensions() {
        assert_eq!(FileType::Auto.extensions(), &["at"]);
        assert_eq!(FileType::CSource.extensions(), &["c"]);
        assert_eq!(FileType::CHeader.extensions(), &["h"]);
        assert_eq!(FileType::Asm.extensions(), &["s", "S"]);
        assert_eq!(FileType::GhsProject.extensions(), &["gpj"]);
        assert_eq!(FileType::Rh850.extensions(), &["850"]);
        assert_eq!(FileType::RustSource.extensions(), &["rs"]);
    }

    #[test]
    fn test_file_type_from_extension() {
        assert_eq!(FileType::from_extension("at"), Some(FileType::Auto));
        assert_eq!(FileType::from_extension("c"), Some(FileType::CSource));
        assert_eq!(FileType::from_extension("h"), Some(FileType::CHeader));
        assert_eq!(FileType::from_extension("s"), Some(FileType::Asm));
        assert_eq!(FileType::from_extension("S"), Some(FileType::Asm));
        assert_eq!(FileType::from_extension("gpj"), Some(FileType::GhsProject));
        assert_eq!(FileType::from_extension("850"), Some(FileType::Rh850));
        assert_eq!(FileType::from_extension("rs"), Some(FileType::RustSource));
        assert_eq!(FileType::from_extension("cpp"), None);
    }

    #[test]
    fn test_file_type_is_header() {
        assert!(FileType::CHeader.is_header());
        assert!(!FileType::CSource.is_header());
        assert!(!FileType::Auto.is_header());
    }

    #[test]
    fn test_file_type_is_source() {
        assert!(FileType::Auto.is_source());
        assert!(FileType::CSource.is_source());
        assert!(FileType::Asm.is_source());
        assert!(FileType::Rh850.is_source());
        assert!(FileType::RustSource.is_source());
        assert!(!FileType::CHeader.is_source());
        assert!(!FileType::GhsProject.is_source());
    }

    #[test]
    fn test_file_filter_for_app_target() {
        let filter = FileFilter::for_target(&TargetKind::App);

        // 应该包含标准源文件
        assert!(filter.should_include("main.c", Some("c")));
        assert!(filter.should_include("utils.c", Some("c")));
        assert!(filter.should_include("header.h", Some("h")));
        assert!(filter.should_include("lib.at", Some("at")));
        assert!(filter.should_include("asm.s", Some("s")));

        // 应该排除配置文件
        assert!(!filter.should_include("pac.at", Some("at")));
        assert!(!filter.should_include("device.at", Some("at")));

        // 应该排除 GHS 项目文件
        assert!(!filter.should_include("project.gpj", Some("gpj")));

        // 不应该包含未知扩展名
        assert!(!filter.should_include("data.txt", Some("txt")));
    }

    #[test]
    fn test_file_filter_for_device_target() {
        let filter = FileFilter::for_target(&TargetKind::Device);

        // Device 目标包含更多类型
        assert!(filter.should_include("main.c", Some("c")));
        assert!(filter.should_include("header.h", Some("h")));
        assert!(filter.should_include("lib.at", Some("at")));
        assert!(filter.should_include("asm.s", Some("s")));
        assert!(filter.should_include("rh850.850", Some("850")));

        // 仍然排除配置文件
        assert!(!filter.should_include("pac.at", Some("at")));
        assert!(!filter.should_include("device.at", Some("at")));
    }

    #[test]
    fn test_file_filter_no_extension() {
        let filter = FileFilter::for_target(&TargetKind::App);

        // 无扩展名的文件不被包含
        assert!(!filter.should_include("Makefile", None));
        assert!(!filter.should_include("README", None));
    }
}
