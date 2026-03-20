# AutoShell SmartCmd 设计方案

## 背景

AutoShell (ASH) 需要将常见 shell 命令封装成可以理解自然语言输入的 SmartCmd。核心问题是：

1. 哪些命令需要结构化输出（如 `ls`, `ps`, `find`）？
2. 哪些命令只需要成功/失败状态（如 `rm`, `cp`, `mv`）？
3. 如何最小化实现成本？

## 方案对比

### 方案 A: Fork uutils/coreutils

**优点：**
- 统一的实现来源
- 可以添加公共结构化 API

**缺点：**
- 需要修改 ~100 个命令的内部结构
- 维护 fork 的成本高
- uutils 的 `PathData` 等内部结构不对外暴露

### 方案 B: 完全自己实现

**优点：**
- 完全控制 API 设计
- 可以针对 ASH 优化

**缺点：**
- 工作量大（每个命令都要实现）
- 跨平台兼容性难以保证
- 边界情况处理不完善

### 方案 C: 混合复用（推荐）

**来源分布：**
- `nu-system` crate: 系统命令（ps 等）
- `sysinfo` crate: 硬件信息（disks, cpu, mem）
- 复制 nushell 转换逻辑: 文件系统命令（ls, du）
- `uutils`: 操作命令（cp, mv, rm）

**优点：**
- 利用现有公开 API
- 最小化实现成本
- 保持跨平台兼容性

## 详细设计

### 命令分类

#### 1. 需要结构化输出的命令

| 命令 | 输出类型 | 推荐来源 |
|------|----------|----------|
| `ls` | `List<FileEntry>` | 复制 nushell 逻辑 |
| `ps` | `List<ProcessEntry>` | nu-system crate |
| `sys disks` | `List<DiskEntry>` | sysinfo crate |
| `sys cpu` | `CpuInfo` | sysinfo crate |
| `sys mem` | `MemInfo` | sysinfo crate |
| `du` | `List<DuEntry>` | 复制 nushell 逻辑 |
| `find` | `List<Path>` | nu-glob crate |

#### 2. 只需要成功/失败的命令

| 命令 | 推荐来源 |
|------|----------|
| `cp` | uutils uu_cp |
| `mv` | uutils uu_mv |
| `rm` | uutils uu_rm |
| `mkdir` | uutils uu_mkdir |
| `touch` | uutils uu_touch |

### ASH 内部类型定义

```rust
// 文件条目（用于 ls 命令）
pub struct AshFileEntry {
    pub name: String,
    pub file_type: String,  // "file" | "dir" | "symlink"
    pub size: i64,
    pub modified: Option<DateTime<Utc>>,
    pub permissions: Option<String>,
    pub owner: Option<String>,
}

// 进程条目（用于 ps 命令）
pub struct AshProcessEntry {
    pub pid: i32,
    pub ppid: i32,
    pub name: String,
    pub status: String,
    pub cpu_usage: f64,
    pub mem_usage: i64,
    pub start_time: Option<DateTime<Utc>>,
}

// 磁盘条目（用于 sys disks）
pub struct AshDiskEntry {
    pub device: String,
    pub file_system: String,
    pub mount_point: String,
    pub total: i64,
    pub free: i64,
    pub removable: bool,
}
```

### 转换层设计

```rust
// 从 nu-system ProcessInfo 转换
impl From<nu_system::ProcessInfo> for AshProcessEntry {
    fn from(info: nu_system::ProcessInfo) -> Self {
        AshProcessEntry {
            pid: info.pid,
            ppid: info.ppid,
            name: info.name,  // 或 Windows 上的 command
            status: info.status(),  // 通过 trait 方法
            cpu_usage: info.percent_cpu(),
            mem_usage: info.mem_resident as i64,
            start_time: None,  // 需要额外处理
        }
    }
}

// 从 sysinfo Disk 转换
impl From<&sysinfo::Disk> for AshDiskEntry {
    fn from(disk: &sysinfo::Disk) -> Self {
        AshDiskEntry {
            device: disk.name().to_string_lossy().to_string(),
            file_system: disk.file_system().to_string_lossy().to_string(),
            mount_point: disk.mount_point().to_string_lossy().to_string(),
            total: disk.total_space() as i64,
            free: disk.available_space() as i64,
            removable: disk.is_removable(),
        }
    }
}
```

### Cargo 依赖

```toml
[dependencies]
# 系统信息
nu-system = { git = "https://github.com/nushell/nushell" }
sysinfo = "0.30"

# 文件操作（从 uutils）
uu_cp = "0.0.27"
uu_mv = "0.0.27"
uu_rm = "0.0.27"
uu_mkdir = "0.0.27"

# 文件查找
nu-glob = { git = "https://github.com/nushell/nushell" }
```

## 实现计划

### Phase 1: 基础设施（1-2 天）

1. 添加 Cargo 依赖
2. 定义 ASH 内部类型（`AshFileEntry`, `AshProcessEntry` 等）
3. 实现 `From` trait 转换

### Phase 2: 结构化命令（3-5 天）

1. 实现 `ps` 命令（使用 nu-system）
2. 实现 `sys disks` 命令（使用 sysinfo）
3. 实现 `ls` 命令（复制 nushell 转换逻辑）

### Phase 3: 操作命令（1-2 天）

1. 集成 uutils 的 cp/mv/rm/mkdir
2. 封装为 ASH 的 SmartCmd 接口

### Phase 4: 自然语言接口（后续）

1. 设计 SmartCmd trait
2. 实现自然语言解析
3. 添加 AI 辅助命令理解

## 风险与缓解

### 风险 1: nu-system API 不稳定

**缓解：**
- 锁定具体版本
- 或 fork nu-system 到 ASH 组织

### 风险 2: 跨平台兼容性

**缓解：**
- nu-system 和 sysinfo 已经处理了跨平台问题
- 测试覆盖 Windows/Linux/macOS

### 风险 3: 依赖更新

**缓解：**
- 使用 Cargo.lock 锁定版本
- 定期检查上游更新

## 结论

推荐采用**方案 C（混合复用）**，理由：

1. **最低成本**：利用现有公开 API，避免重复造轮子
2. **最高质量**：nu-system 和 sysinfo 经过大规模测试
3. **最易维护**：依赖活跃维护的上游项目
4. **最快实现**：预计 1 周内可完成核心命令
