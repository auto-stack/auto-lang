//! ASH 内部类型定义
//!
//! 这些类型用于结构化命令输出，与 auto_val::Value 配合使用。

use chrono::{DateTime, Utc};

/// 文件条目（用于 ls 命令输出）
#[derive(Debug, Clone, PartialEq)]
pub struct AshFileEntry {
    pub name: String,
    pub file_type: FileType,
    pub size: i64,
    pub modified: Option<DateTime<Utc>>,
    pub permissions: Option<String>,
    pub owner: Option<String>,
    pub target: Option<String>, // symlink target
}

/// 文件类型
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FileType {
    File,
    Dir,
    Symlink,
    Unknown,
}

impl FileType {
    pub fn as_str(&self) -> &'static str {
        match self {
            FileType::File => "file",
            FileType::Dir => "dir",
            FileType::Symlink => "symlink",
            FileType::Unknown => "unknown",
        }
    }
}

impl std::fmt::Display for FileType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// 进程条目（用于 ps 命令输出）
#[derive(Debug, Clone, PartialEq)]
pub struct AshProcessEntry {
    pub pid: i32,
    pub ppid: i32,
    pub name: String,
    pub status: String,
    pub cpu_usage: f64,
    pub mem_usage: i64,
    pub start_time: Option<DateTime<Utc>>,
    pub command: Option<String>,
}

/// 磁盘条目（用于 sys disks 命令输出）
#[derive(Debug, Clone, PartialEq)]
pub struct AshDiskEntry {
    pub device: String,
    pub file_system: String,
    pub mount_point: String,
    pub total: i64,
    pub free: i64,
    pub removable: bool,
}

/// CPU 信息（用于 sys cpu 命令输出）
#[derive(Debug, Clone, PartialEq)]
pub struct AshCpuInfo {
    pub name: String,
    pub brand: String,
    pub frequency: u64,
    pub cores: usize,
    pub usage: f64,
}

/// 内存信息（用于 sys mem 命令输出）
#[derive(Debug, Clone, PartialEq)]
pub struct AshMemoryInfo {
    pub total: i64,
    pub free: i64,
    pub available: i64,
    pub used: i64,
    pub usage_percent: f64,
}

impl AshFileEntry {
    /// 格式化文件大小（人类可读）
    pub fn format_size(&self) -> String {
        if self.size < 1024 {
            format!("{}B", self.size)
        } else if self.size < 1024 * 1024 {
            format!("{:.1}K", self.size as f64 / 1024.0)
        } else if self.size < 1024 * 1024 * 1024 {
            format!("{:.1}M", self.size as f64 / (1024.0 * 1024.0))
        } else {
            format!("{:.1}G", self.size as f64 / (1024.0 * 1024.0 * 1024.0))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_file_type_display() {
        assert_eq!(FileType::File.to_string(), "file");
        assert_eq!(FileType::Dir.to_string(), "dir");
        assert_eq!(FileType::Symlink.to_string(), "symlink");
    }

    #[test]
    fn test_format_size() {
        let entry = AshFileEntry {
            name: "test".to_string(),
            file_type: FileType::File,
            size: 512,
            modified: None,
            permissions: None,
            owner: None,
            target: None,
        };
        assert_eq!(entry.format_size(), "512B");

        let entry = AshFileEntry {
            name: "test".to_string(),
            file_type: FileType::File,
            size: 1536,
            modified: None,
            permissions: None,
            owner: None,
            target: None,
        };
        assert_eq!(entry.format_size(), "1.5K");
    }
}
