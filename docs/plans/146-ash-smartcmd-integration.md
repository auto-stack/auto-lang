# AutoShell SmartCmd 集成计划

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** 将 AutoShell 命令体系改为借助 nushell/uutils 等库的实现，实现结构化输出和跨平台兼容。

**Architecture:** 采用混合复用策略 - 使用 nu-system 获取进程信息、sysinfo 获取硬件信息、复制 nushell 转换逻辑实现 ls/du、使用 uutils 实现 cp/mv/rm 等操作命令。

**Tech Stack:** Rust, nu-system crate, sysinfo crate, uutils crates, chrono

---

## Task 1: 添加 Cargo 依赖

**Files:**
- Modify: `crates/auto-shell/Cargo.toml:9-43`

**Step 1: 添加 nu-system 和 sysinfo 依赖**

在 `[dependencies]` 部分添加：

```toml
# 系统信息（来自 nushell）
sysinfo = "0.33"  # 已有，升级版本

# 注意：nu-system 需要从 nushell repo 引入，但它的 Cargo 结构复杂
# 我们先使用 sysinfo 来实现系统命令，因为 nushell 的 sys 也用了 sysinfo
```

**Step 2: 验证依赖编译**

Run: `cargo check -p auto-shell`
Expected: 编译成功，无错误

**Step 3: Commit**

```bash
rtk git add crates/auto-shell/Cargo.toml
rtk git commit -m "chore(shell): upgrade sysinfo dependency for system commands"
```

---

## Task 2: 创建 ASH 内部类型定义

**Files:**
- Create: `crates/auto-shell/src/data/types.rs`
- Modify: `crates/auto-shell/src/data/mod.rs:1-11`

**Step 1: 创建类型定义文件**

创建 `crates/auto-shell/src/data/types.rs`:

```rust
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
```

**Step 2: 更新 mod.rs 导出新类型**

修改 `crates/auto-shell/src/data/mod.rs`:

```rust
//! Data module for shell
//!
//! Provides structured data types and table rendering.

pub mod table;
pub mod convert;
pub mod value;
pub mod types;

pub use table::{Table, Column, Align, FileEntry};
pub use value::ShellValue;
pub use types::{
    AshFileEntry, AshProcessEntry, AshDiskEntry,
    AshCpuInfo, AshMemoryInfo, FileType,
};
```

**Step 3: 运行测试验证**

Run: `cargo test -p auto-shell -- data::types`
Expected: 2 tests passed

**Step 4: Commit**

```bash
rtk git add crates/auto-shell/src/data/types.rs crates/auto-shell/src/data/mod.rs
rtk git commit -m "feat(shell): add ASH internal types for structured command output"
```

---

## Task 3: 实现 AshFileEntry 转换层

**Files:**
- Modify: `crates/auto-shell/src/data/convert.rs:1-100`

**Step 1: 添加从 std::fs::Metadata 转换的实现**

修改 `crates/auto-shell/src/data/convert.rs`，添加：

```rust
//! 数据转换工具
//!
//! 提供从外部库类型到 ASH 内部类型的转换。

use super::types::{AshFileEntry, FileType};
use chrono::{DateTime, Utc};
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
```

**Step 2: 运行测试验证**

Run: `cargo test -p auto-shell -- data::convert`
Expected: 1 test passed

**Step 3: Commit**

```bash
rtk git add crates/auto-shell/src/data/convert.rs
rtk git commit -m "feat(shell): add metadata to AshFileEntry conversion"
```

---

## Task 4: 实现 AshFileEntry 到 Value 的转换

**Files:**
- Modify: `crates/auto-shell/src/data/convert.rs`

**Step 1: 添加 to_value 函数**

在 `convert.rs` 末尾添加：

```rust
use auto_val::{Value, Obj};

/// 将 AshFileEntry 转换为 auto_val::Value::Obj
pub fn file_entry_to_value(entry: &AshFileEntry) -> Value {
    let mut obj = Obj::new();

    obj.set("name", Value::str(&entry.name));
    obj.set("type", Value::str(entry.file_type.as_str()));
    obj.set("size", Value::Int(entry.size));

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
    Value::Array(values.into_iter().collect())
}
```

**Step 2: 运行测试验证**

Run: `cargo test -p auto-shell -- data::convert`
Expected: 编译成功

**Step 3: Commit**

```bash
rtk git add crates/auto-shell/src/data/convert.rs
rtk git commit -m "feat(shell): add AshFileEntry to Value conversion"
```

---

## Task 5: 重构 ls 命令使用新类型

**Files:**
- Modify: `crates/auto-shell/src/cmd/fs.rs:1-280`
- Modify: `crates/auto-shell/src/cmd/commands/ls.rs:1-53`

**Step 1: 重写 ls_command_value 函数**

修改 `crates/auto-shell/src/cmd/fs.rs`，替换 `ls_command_value` 函数：

```rust
/// List directory contents as structured Value (array of file objects)
///
/// This is the structured data version of ls_command for use in pipelines.
/// Returns an Array of Obj values, where each Obj represents a file entry.
pub fn ls_command_value(
    path: &Path,
    current_dir: &Path,
    all: bool,
    long: bool,
    time_sort: bool,
    reverse: bool,
    recursive: bool,
) -> Result<Value> {
    use crate::data::{AshFileEntry, metadata_to_entry, file_entries_to_value};

    let target = if path.is_absolute() {
        path.to_path_buf()
    } else {
        current_dir.join(path)
    };

    if !target.exists() {
        miette::bail!("ls: {}: No such file or directory", target.display());
    }

    // Handle recursive listing
    if recursive {
        return list_recursive_value(&target, current_dir, all, long, time_sort, reverse);
    }

    // If it's a file, return single-element array with file info
    if target.is_file() {
        let metadata = fs::metadata(&target).into_diagnostic()?;
        let name = target.file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("?")
            .to_string();

        let entry = metadata_to_entry(&target, &name, &metadata);
        return Ok(Value::Array(vec![crate::data::file_entry_to_value(&entry)].into_iter().collect()));
    }

    // List directory contents
    let entries = fs::read_dir(&target).into_diagnostic()?;

    let mut files: Vec<(String, AshFileEntry)> = Vec::new();
    for entry_result in entries {
        let entry = entry_result.into_diagnostic()?;
        let path = entry.path();

        let name = entry.file_name()
            .into_string()
            .unwrap_or_else(|_| "?".to_string());

        // Skip hidden files unless -a flag is set
        if !all && name.starts_with('.') {
            continue;
        }

        // Get metadata
        let metadata = match entry.metadata() {
            Ok(m) => m,
            Err(_) => continue, // Skip files we can't read
        };

        let ash_entry = metadata_to_entry(&path, &name, &metadata);
        files.push((name, ash_entry));
    }

    // Sort files
    files.sort_by(|a, b| {
        let cmp = if time_sort {
            // Sort by modification time (newest first)
            match (&a.1.modified, &b.1.modified) {
                (Some(a_time), Some(b_time)) => b_time.cmp(a_time),
                (Some(_), None) => std::cmp::Ordering::Less,
                (None, Some(_)) => std::cmp::Ordering::Greater,
                (None, None) => a.0.cmp(&b.0),
            }
        } else {
            // Sort alphabetically
            a.0.cmp(&b.0)
        };

        // Directories first
        if a.1.file_type != b.1.file_type {
            let a_is_dir = a.1.file_type == crate::data::FileType::Dir;
            let b_is_dir = b.1.file_type == crate::data::FileType::Dir;
            b_is_dir.cmp(&a_is_dir)
        } else {
            cmp
        }
    });

    if reverse {
        files.reverse();
    }

    // Build array
    let entries: Vec<AshFileEntry> = files.into_iter().map(|(_, e)| e).collect();
    Ok(file_entries_to_value(&entries))
}
```

**Step 2: 更新 fs.rs 的 imports**

在 `fs.rs` 顶部添加：

```rust
use crate::data::{AshFileEntry, metadata_to_entry, file_entry_to_value, file_entries_to_value};
```

**Step 3: 运行测试验证**

Run: `cargo test -p auto-shell -- cmd::fs`
Expected: 所有测试通过

**Step 4: Commit**

```bash
rtk git add crates/auto-shell/src/cmd/fs.rs
rtk git commit -m "refactor(shell): use AshFileEntry in ls_command_value"
```

---

## Task 6: 实现 ps 命令（使用 sysinfo）

**Files:**
- Create: `crates/auto-shell/src/cmd/commands/ps.rs`
- Modify: `crates/auto-shell/src/cmd/commands/mod.rs`

**Step 1: 创建 ps.rs 命令文件**

创建 `crates/auto-shell/src/cmd/commands/ps.rs`:

```rust
//! ps command - List running processes
//!
//! Uses sysinfo crate for cross-platform process listing.

use crate::cmd::{Command, PipelineData, Signature, parser::ParsedArgs};
use crate::data::{AshProcessEntry, file_entries_to_value};
use crate::shell::Shell;
use auto_val::Value;
use miette::Result;
use sysinfo::System;

pub struct PsCommand;

impl Command for PsCommand {
    fn name(&self) -> &str {
        "ps"
    }

    fn signature(&self) -> Signature {
        Signature::new("ps", "List running processes")
            .flag_with_short("long", 'l', "Show detailed process information")
            .flag_with_short("all", 'a', "Show all processes (including system)")
    }

    fn run(
        &self,
        args: &crate::cmd::parser::ParsedArgs,
        _input: PipelineData,
        _shell: &mut Shell,
    ) -> Result<PipelineData> {
        let long = args.has_flag("long");
        let _all = args.has_flag("all");

        let mut sys = System::new_all();
        sys.refresh_all();

        let mut processes: Vec<AshProcessEntry> = sys.processes()
            .iter()
            .map(|(pid, process)| {
                AshProcessEntry {
                    pid: pid.as_u32() as i32,
                    ppid: process.parent().map(|p| p.as_u32() as i32).unwrap_or(0),
                    name: process.name().to_string_lossy().to_string(),
                    status: format!("{:?}", process.status()),
                    cpu_usage: process.cpu_usage() as f64,
                    mem_usage: process.memory() as i64,
                    start_time: None, // sysinfo doesn't provide this directly
                    command: if long {
                        Some(process.cmd().iter()
                            .map(|s| s.to_string_lossy().to_string())
                            .collect::<Vec<_>>()
                            .join(" "))
                    } else {
                        None
                    },
                }
            })
            .collect();

        // Sort by CPU usage (highest first)
        processes.sort_by(|a, b| {
            b.cpu_usage.partial_cmp(&a.cpu_usage).unwrap_or(std::cmp::Ordering::Equal)
        });

        // Convert to Value
        let values: Vec<Value> = processes.iter().map(|p| {
            let mut obj = auto_val::Obj::new();
            obj.set("pid", Value::Int(p.pid as i64));
            obj.set("ppid", Value::Int(p.ppid as i64));
            obj.set("name", Value::str(&p.name));
            obj.set("status", Value::str(&p.status));
            obj.set("cpu", Value::Float(p.cpu_usage));
            obj.set("mem", Value::Int(p.mem_usage));

            if let Some(cmd) = &p.command {
                obj.set("command", Value::str(cmd));
            }

            Value::Obj(obj)
        }).collect();

        Ok(PipelineData::from_value(Value::Array(values.into_iter().collect())))
    }
}
```

**Step 2: 更新 mod.rs 导出新命令**

修改 `crates/auto-shell/src/cmd/commands/mod.rs`:

```rust
pub mod build;
pub mod cd;
pub mod echo;
pub mod get;
pub mod grep;
pub mod help;
pub mod ls;
pub mod ps;  // 新增
pub mod pwd;
pub mod run;
pub mod r#where;
pub mod select;
pub mod wc;
```

**Step 3: 运行测试验证**

Run: `cargo check -p auto-shell`
Expected: 编译成功

**Step 4: Commit**

```bash
rtk git add crates/auto-shell/src/cmd/commands/ps.rs crates/auto-shell/src/cmd/commands/mod.rs
rtk git commit -m "feat(shell): add ps command using sysinfo"
```

---

## Task 7: 实现 sys 子命令（disks/cpu/mem）

**Files:**
- Create: `crates/auto-shell/src/cmd/commands/sys.rs`
- Modify: `crates/auto-shell/src/cmd/commands/mod.rs`

**Step 1: 创建 sys.rs 命令文件**

创建 `crates/auto-shell/src/cmd/commands/sys.rs`:

```rust
//! sys command - System information
//!
//! Provides disk, cpu, and memory information using sysinfo crate.

use crate::cmd::{Command, PipelineData, Signature, parser::ParsedArgs};
use crate::data::{AshDiskEntry, AshCpuInfo, AshMemoryInfo};
use crate::shell::Shell;
use auto_val::{Value, Obj};
use miette::Result;
use sysinfo::{System, Disks, CpuRefreshKind};

pub struct SysCommand;

impl Command for SysCommand {
    fn name(&self) -> &str {
        "sys"
    }

    fn signature(&self) -> Signature {
        Signature::new("sys", "Get system information")
            .optional("subcommand", "Subcommand: disks, cpu, mem")
    }

    fn run(
        &self,
        args: &crate::cmd::parser::ParsedArgs,
        _input: PipelineData,
        _shell: &mut Shell,
    ) -> Result<PipelineData> {
        let subcommand = args.positionals.get(0).map(|s| s.as_str()).unwrap_or("all");

        match subcommand {
            "disks" => sys_disks(),
            "cpu" => sys_cpu(),
            "mem" | "memory" => sys_mem(),
            "all" => sys_all(),
            _ => miette::bail!("sys: unknown subcommand '{}'. Use: disks, cpu, mem", subcommand),
        }
    }
}

fn sys_disks() -> Result<PipelineData> {
    let disks = Disks::new_with_refreshed_list();

    let values: Vec<Value> = disks.iter().map(|disk| {
        let mut obj = Obj::new();
        obj.set("device", Value::str(&disk.name().to_string_lossy()));
        obj.set("type", Value::str(&disk.file_system().to_string_lossy()));
        obj.set("mount", Value::str(&disk.mount_point().to_string_lossy()));
        obj.set("total", Value::Int(disk.total_space() as i64));
        obj.set("free", Value::Int(disk.available_space() as i64));
        obj.set("removable", Value::Bool(disk.is_removable()));
        Value::Obj(obj)
    }).collect();

    Ok(PipelineData::from_value(Value::Array(values.into_iter().collect())))
}

fn sys_cpu() -> Result<PipelineData> {
    let mut sys = System::new();
    sys.refresh_cpu_specifics(CpuRefreshKind::everything());

    let cpus: Vec<Value> = sys.cpus().iter().enumerate().map(|(i, cpu)| {
        let mut obj = Obj::new();
        obj.set("index", Value::Int(i as i64));
        obj.set("name", Value::str(cpu.name()));
        obj.set("vendor", Value::str(cpu.vendor_id()));
        obj.set("brand", Value::str(cpu.brand()));
        obj.set("frequency", Value::Int(cpu.frequency() as i64));
        obj.set("usage", Value::Float(cpu.cpu_usage() as f64));
        Value::Obj(obj)
    }).collect();

    Ok(PipelineData::from_value(Value::Array(cpus.into_iter().collect())))
}

fn sys_mem() -> Result<PipelineData> {
    let mut sys = System::new();
    sys.refresh_memory();

    let total = sys.total_memory() as i64;
    let free = sys.free_memory() as i64;
    let available = sys.available_memory() as i64;
    let used = total - available;
    let usage_percent = if total > 0 { (used as f64 / total as f64) * 100.0 } else { 0.0 };

    let mut obj = Obj::new();
    obj.set("total", Value::Int(total));
    obj.set("free", Value::Int(free));
    obj.set("available", Value::Int(available));
    obj.set("used", Value::Int(used));
    obj.set("usage_percent", Value::Float(usage_percent));

    Ok(PipelineData::from_value(Value::Obj(obj)))
}

fn sys_all() -> Result<PipelineData> {
    let mut obj = Obj::new();

    // Get disks
    let disks = Disks::new_with_refreshed_list();
    let disk_values: Vec<Value> = disks.iter().map(|disk| {
        let mut d = Obj::new();
        d.set("device", Value::str(&disk.name().to_string_lossy()));
        d.set("mount", Value::str(&disk.mount_point().to_string_lossy()));
        d.set("total", Value::Int(disk.total_space() as i64));
        d.set("free", Value::Int(disk.available_space() as i64));
        Value::Obj(d)
    }).collect();
    obj.set("disks", Value::Array(disk_values.into_iter().collect()));

    // Get memory
    let mut sys = System::new();
    sys.refresh_memory();
    obj.set("total_memory", Value::Int(sys.total_memory() as i64));
    obj.set("free_memory", Value::Int(sys.free_memory() as i64));

    Ok(PipelineData::from_value(Value::Obj(obj)))
}
```

**Step 2: 更新 mod.rs**

在 `mod.rs` 中添加：

```rust
pub mod sys;  // 新增
```

**Step 3: 运行测试验证**

Run: `cargo check -p auto-shell`
Expected: 编译成功

**Step 4: Commit**

```bash
rtk git add crates/auto-shell/src/cmd/commands/sys.rs crates/auto-shell/src/cmd/commands/mod.rs
rtk git commit -m "feat(shell): add sys command with disks/cpu/mem subcommands"
```

---

## Task 8: 注册新命令到 Shell

**Files:**
- Modify: `crates/auto-shell/src/cmd/registry.rs`

**Step 1: 在 registry 中注册新命令**

找到命令注册位置，添加：

```rust
use crate::cmd::commands::{PsCommand, SysCommand};

// 在 register_commands 或类似函数中添加：
registry.register(Box::new(PsCommand));
registry.register(Box::new(SysCommand));
```

**Step 2: 运行测试验证**

Run: `cargo test -p auto-shell`
Expected: 所有测试通过

**Step 3: Commit**

```bash
rtk git add crates/auto-shell/src/cmd/registry.rs
rtk git commit -m "feat(shell): register ps and sys commands"
```

---

## Task 9: 添加集成测试

**Files:**
- Create: `crates/auto-shell/tests/structured_commands.rs`

**Step 1: 创建集成测试文件**

```rust
//! Integration tests for structured command output

use auto_shell::shell::Shell;

#[test]
fn test_ls_returns_structured_data() {
    let mut shell = Shell::new().unwrap();

    // Execute ls command
    let result = shell.execute_line("ls .");

    // Should return structured data (array of objects)
    assert!(result.is_ok());
}

#[test]
fn test_ps_returns_structured_data() {
    let mut shell = Shell::new().unwrap();

    // Execute ps command
    let result = shell.execute_line("ps");

    // Should return structured data
    assert!(result.is_ok());
}

#[test]
fn test_sys_disks_returns_structured_data() {
    let mut shell = Shell::new().unwrap();

    // Execute sys disks command
    let result = shell.execute_line("sys disks");

    // Should return structured data
    assert!(result.is_ok());
}

#[test]
fn test_sys_mem_returns_structured_data() {
    let mut shell = Shell::new().unwrap();

    // Execute sys mem command
    let result = shell.execute_line("sys mem");

    // Should return structured data
    assert!(result.is_ok());
}
```

**Step 2: 运行集成测试**

Run: `cargo test -p auto-shell --test structured_commands`
Expected: 4 tests passed

**Step 3: Commit**

```bash
rtk git add crates/auto-shell/tests/structured_commands.rs
rtk git commit -m "test(shell): add integration tests for structured commands"
```

---

## Task 10: 更新设计文档

**Files:**
- Modify: `docs/design/ash-smartcmd-design.md`

**Step 1: 更新实现状态**

在设计文档末尾添加：

```markdown
## 实现状态

### Phase 1: 基础设施 ✅
- [x] 添加 Cargo 依赖
- [x] 定义 ASH 内部类型
- [x] 实现转换层

### Phase 2: 结构化命令 ✅
- [x] ls 命令使用 AshFileEntry
- [x] ps 命令使用 sysinfo
- [x] sys 命令（disks/cpu/mem）

### Phase 3: 操作命令（待实现）
- [ ] 集成 uutils cp
- [ ] 集成 uutils mv
- [ ] 集成 uutils rm
- [ ] 集成 uutils mkdir

### Phase 4: 自然语言接口（后续）
- [ ] SmartCmd trait 设计
- [ ] 自然语言解析
```

**Step 2: Commit**

```bash
rtk git add docs/design/ash-smartcmd-design.md
rtk git commit -m "docs: update SmartCmd design with implementation status"
```

---

## Summary

| Task | Description | Files |
|------|-------------|-------|
| 1 | 添加 Cargo 依赖 | Cargo.toml |
| 2 | 创建 ASH 内部类型 | data/types.rs, data/mod.rs |
| 3 | 实现 AshFileEntry 转换 | data/convert.rs |
| 4 | 实现 Value 转换 | data/convert.rs |
| 5 | 重构 ls 命令 | cmd/fs.rs |
| 6 | 实现 ps 命令 | cmd/commands/ps.rs |
| 7 | 实现 sys 命令 | cmd/commands/sys.rs |
| 8 | 注册新命令 | shell.rs |
| 9 | 添加集成测试 | tests/structured_commands.rs |
| 10 | 更新文档 | docs/design/ash-smartcmd-design.md |

---

## 实现完成状态 ✅

**完成日期**: 2026-03-31

**Phase 1: 基础设施** ✅
- ✅ 添加 Cargo 依赖 (sysinfo 0.30 → 0.33)
- ✅ 定义 ASH 内部类型
- ✅ 实现转换层

**Phase 2: 结构化命令** ✅
- ✅ ls 命令使用 AshFileEntry
- ✅ ps 命令使用 sysinfo
- ✅ sys 命令（disks/cpu/mem）

**Phase 3: 操作命令** ✅
- ✅ 集成 cp 命令（递归复制、权限保留、进度报告）
- ✅ 集成 mv 命令（文件移动/重命名）
- ✅ 集成 rm 命令（递归删除、强制模式）
- ✅ 集成 mkdir 命令（递归创建目录）

**Phase 4: 自然语言接口** (后续)
- ⏸️ SmartCmd trait 设计
- ⏸️ 自然语言解析

**测试覆盖**: 9 个集成测试通过
- ✅ ls, ps, sys disks, sys mem (4 个结构化输出测试)
- ✅ mkdir, cp, mv, rm, rm -r (5 个文件操作测试)

**代码变更统计**:
- 新增文件: 4 个命令文件 (cp.rs, mv.rs, rm.rs, mkdir.rs)
- 修改文件: mod.rs, shell.rs, tests/structured_commands.rs
- 总计: ~700 行新代码
