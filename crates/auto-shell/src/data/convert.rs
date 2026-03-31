//! 数据转换工具
//!
//! 提供从外部库类型到 ASH 内部类型的转换。

use super::types::{AshFileEntry, FileType};
use chrono::DateTime;
use std::fs::Metadata;
use std::path::Path;
use std::time::UNIX_EPOCH;

/// 从文件元数据创建 AshFileEntry
pub fn metadata_to_entry(
    path: &Path,
    name: &str,
    metadata: &Metadata,
) -> AshFileEntry {
    let file_type = if metadata.is_dir() {
        FileType::Dir
    } else if metadata.is_symlink() {
        FileType::Symlink
    } else {
        FileType::File
    };

    let size = if file_type == FileType::Dir {
        0
    } else {
        metadata.len() as i64
    };

    let modified = metadata.modified()
        .ok()
        .and_then(|t| {
            let secs = t.duration_since(UNIX_EPOCH).ok()?.as_secs() as i64;
            DateTime::from_timestamp(secs, 0)
        });

    let permissions = Some(get_permissions_string(metadata));
    let owner = get_owner(metadata);

    let target = if file_type == FileType::Symlink {
        path.read_link()
            .map(|p| p.to_string_lossy().to_string())
            .ok()
    } else {
        None
    };

    AshFileEntry {
        name: name.to_string(),
        file_type,
        size,
        modified,
        permissions,
        owner,
        target,
    }
}

/// 获取权限字符串（Unix 风格）
#[cfg(unix)]
fn get_permissions_string(metadata: &Metadata) -> String {
    use std::os::unix::fs::PermissionsExt;
    let mode = metadata.permissions().mode();
    let file_type = if metadata.is_dir() { 'd' } else { '-' };

    let user = format_perm_bits(mode >> 6);
    let group = format_perm_bits(mode >> 3);
    let other = format_perm_bits(mode);

    format!("{}{}{}{}", file_type, user, group, other)
}

#[cfg(unix)]
fn format_perm_bits(bits: u32) -> String {
    let r = if bits & 0b100 != 0 { 'r' } else { '-' };
    let w = if bits & 0b010 != 0 { 'w' } else { '-' };
    let x = if bits & 0b001 != 0 { 'x' } else { '-' };
    format!("{}{}{}", r, w, x)
}

/// 获取权限字符串（Windows 简化版）
#[cfg(windows)]
fn get_permissions_string(metadata: &Metadata) -> String {
    if metadata.permissions().readonly() {
        "-r--r--r--".to_string()
    } else {
        "-rw-rw-rw-".to_string()
    }
}

/// 获取所有者
#[cfg(unix)]
fn get_owner(metadata: &Metadata) -> Option<String> {
    use std::os::unix::fs::MetadataExt;
    Some(metadata.uid().to_string())
}

/// 获取所有者（Windows 暂不支持）
#[cfg(windows)]
fn get_owner(_metadata: &Metadata) -> Option<String> {
    None
}

use auto_val::{Value, Obj, Array};

/// 将 AshFileEntry 转换为 auto_val::Value::Obj
pub fn file_entry_to_value(entry: &AshFileEntry) -> Value {
    let mut obj = Obj::new();

    obj.set("name", Value::str(&entry.name));
    obj.set("type", Value::str(entry.file_type.as_str()));
    obj.set("size", Value::Int(entry.size as i32));

    if let Some(modified) = &entry.modified {
        obj.set("modified", Value::str(&modified.format("%Y-%m-%d %H:%M:%S").to_string()));
    }

    if let Some(permissions) = &entry.permissions {
        obj.set("permissions", Value::str(permissions));
    }

    if let Some(owner) = &entry.owner {
        obj.set("owner", Value::str(owner));
    }

    if let Some(target) = &entry.target {
        obj.set("target", Value::str(target));
    }

    Value::Obj(obj)
}

/// 将 AshFileEntry 列表转换为 Value::Array
pub fn file_entries_to_value(entries: &[AshFileEntry]) -> Value {
    let values: Vec<Value> = entries.iter().map(file_entry_to_value).collect();
    Value::Array(Array { values })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn test_metadata_to_entry() {
        // 使用当前目录测试
        let path = std::env::current_dir().unwrap();
        let metadata = fs::metadata(&path).unwrap();
        let name = path.file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown");

        let entry = metadata_to_entry(&path, name, &metadata);

        assert_eq!(entry.file_type, FileType::Dir);
        assert!(entry.name.len() > 0);
    }
}
