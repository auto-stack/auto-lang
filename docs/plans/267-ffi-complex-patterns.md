# Plan 267: FFI Complex Patterns — 外部迭代器与多 Crate 组合

> **状态**: 📋 PLANNED
> **前置**: Plan 212 Phase 1-4 已完成
> **范围**: 解决 Plan 212 遗留的 "Hard" 复杂 FFI 场景（约 12 个测试）
>
> **不在范围内**: Closure FFI（rayon/crossbeam）和 Custom Serde Deserialization — 这两个属于 "Impossible" 级别，需要 VM 核心架构扩展，另行规划。
>
> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**目标**: 为 AutoVM 添加外部迭代器协议和多 Crate 组合能力，解锁 csv/walkdir 等迭代器场景和 tar+flate2 等多 crate 组合场景。

---

## 背景分析

Plan 212 完成后，82 个 B-tier 测试中：
- 59 个已通过（Phase 1-4 覆盖）
- 10 个 log/tracing 已通过（`#macro` 语法）
- **12 个 "Hard" 测试未通过**（本 Plan 目标）
- 11 个 "Impossible" 测试暂不处理

### Hard 测试分类

| 模式 | 测试数 | 涉及 Crate | 核心问题 |
|------|--------|-----------|----------|
| 外部迭代器 | 8 | walkdir, csv | Rust 迭代器无法直接用于 Auto `for` 循环 |
| 多 Crate 组合 | 4 | tar+flate2, same_file | 多个 opaque handle 需要嵌套交互 |

---

## 问题 1: 外部迭代器

### 当前状态

Auto 的 `for x in collection` 循环期望 collection 实现内部迭代器协议（VM `for-in` 操作码）。但 Rust crate 的迭代器（如 `walkdir::IntoIter`, `csv::StringRecordsIter`）是 Rust 侧状态机，无法直接映射到 VM 迭代器。

### 方案: Collect-then-Iterate（预收集模式）

不尝试桥接外部迭代器协议，而是在 shim 层预收集所有元素为 Auto `List`，然后用 Auto 原生 `for` 循环遍历。

```
用户代码:                          实际执行:
  let entries = WalkDir.new("src")    → 创建 WalkDir opaque handle
  for entry in entries { ... }        → entries 自动转为 List<str>
                                      → for entry in list { ... }
```

关键：为迭代器类型添加 `.to_list()` / `.collect()` 自动转换。

### 具体实现

#### Step 1: 添加 WalkDir 迭代器 shim

为 `WalkDir` 添加 `collect` 方法，一次性收集所有路径为 `List<String>`：

**文件**: `crates/auto-lang/src/vm/native.rs`

```rust
// shim_walkdir_collect: WalkDir handle → List<String>
fn shim_walkdir_collect(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let handle_id = task.ram.pop_i32() as u64;
    let obj = vm.get_heap_object(handle_id).ok_or(...)?;
    let guard = obj.read().unwrap();
    let rso = guard.as_any().downcast_ref::<RustStdlibObject>().ok_or(...)?;
    let walker = rso.downcast_ref::<Mutex<walkdir::WalkDir>>().ok_or(...)?;

    let mut paths = Vec::new();
    for entry in walker.lock().unwrap() {
        match entry {
            Ok(e) => paths.push(e.path().display().to_string()),
            Err(_) => continue,
        }
    }

    // Push as Auto List
    task.ram.push_list(&paths);
    Ok(())
}
```

**Native ID**: 2891 (`auto.walkdir.collect`)

#### Step 2: Codegen 路由 — 迭代器自动转换

在 codegen 中检测 `for x in opaque_handle` 模式，自动插入 `.collect()` 调用：

**文件**: `crates/auto-lang/src/vm/codegen.rs`

```rust
// 当 for-in 的迭代对象是 opaque handle 类型时
// 自动插入 .collect() 将其转为 List
if is_opaque_iterator(&iter_type) {
    // 编译为:
    //   temp = iter.collect()    // opaque → List
    //   for x in temp { ... }    // 正常 List 迭代
    self.emit_call_nat("auto.walkdir.collect");
}
```

更简单的方案：在迭代器构造时直接返回 `List` 而不是 opaque handle：

```rust
// WalkDir.new("src") 直接返回 List<String> 而不是 handle
fn shim_walkdir_new(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let path = task.ram.pop_str();
    let mut paths = Vec::new();
    for entry in walkdir::WalkDir::new(&path) {
        match entry {
            Ok(e) => paths.push(e.path().display().to_string()),
            Err(_) => continue,
        }
    }
    task.ram.push_list(&paths);  // 直接返回 List
    Ok(())
}
```

这样 `for entry in WalkDir.new("src")` 自然工作。

#### Step 3: 添加 CSV Reader shim

类似 WalkDir，`csv::Reader` 的 `records()` 返回迭代器。提供 `read_all()` 一次性收集：

```rust
// csv_read_all(path) → List<List<String>>
fn shim_csv_read_all(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let path = task.ram.pop_str();
    let mut reader = csv::Reader::from_path(&path)
        .map_err(|e| VMError::RuntimeError(e.to_string()))?;

    let mut rows = Vec::new();
    for record in reader.records() {
        match record {
            Ok(rec) => {
                let row: Vec<String> = rec.iter().map(|f| f.to_string()).collect();
                rows.push(row);
            }
            Err(_) => continue,
        }
    }

    // Push as Auto List<List<String>>
    task.ram.push_nested_list(&rows);
    Ok(())
}
```

#### Step 4: Native ID 分配

> **注意**: 原计划 2750-2782 范围已被 Plan 250 占用，更新为 2890+。

| ID | Name | 功能 |
|----|------|------|
| 2890 | `auto.walkdir.new` | `WalkDir::new(path)` → `List<String>` |
| 2891 | `auto.walkdir.collect` | handle → `List<String>`（备用） |
| 2892 | `auto.csv.read_all` | `Reader::from_path()` → `List<List<String>>` |
| 2893 | `auto.csv.headers` | 读取 CSV header → `List<String>` |

### 涉及的测试文件

| 测试 | 模式 | 解决方案 |
|------|------|----------|
| `file/002_find_files.at` | WalkDir 迭代 | `WalkDir.new()` 直接返回 List |
| `file/005_duplicate_name.at` | `fs::read_dir()` 迭代 | 添加 `read_dir_list()` shim |
| `encoding/003_csv_read.at` | CSV Reader records | `csv.read_all()` 返回嵌套 List |

### 依赖变更

`crates/auto-lang/Cargo.toml` 添加：
```toml
walkdir = "2"
csv = "1"
```

---

## 问题 2: 多 Crate 组合

### 当前状态

Opaque handle 模式是"扁平"的——每个 handle 对应一个 Rust 对象。当需要组合多个 crate 的类型时（如 `tar::Builder` 包裹 `flate2::GzEncoder`），无法表达嵌套关系。

### 方案: Combined Shim（组合 shim）

不为每个 crate 独立创建 opaque handle，而是为常见的组合场景提供预构建的 combined shim：

```
用户代码:
  dep tar
  dep flate2
  let archive = TarGzip.create("output.tar.gz")
  archive.add("file1.txt", "content1")
  archive.add("file2.txt", "content2")
  archive.finish()

底层:
  TarGzip.create → 创建 tar::Builder<flate2::write::GzEncoder<File>>
  一个 opaque handle 持有整个组合对象
```

### 具体实现

#### Step 1: 添加 TarGzip 组合 shim

```rust
use std::io::Write;

// 组合类型: tar::Builder<flate2::write::GzEncoder<std::fs::File>>
type TarGzip = tar::Builder<flate2::write::GzEncoder<std::fs::File>>;

fn shim_targzip_create(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let path = task.ram.pop_str();
    let file = std::fs::File::create(&path)
        .map_err(|e| VMError::RuntimeError(e.to_string()))?;
    let enc = flate2::write::GzEncoder::new(file, flate2::Compression::default());
    let builder = tar::Builder::new(enc);
    let obj = RustStdlibObject::new("TarGzip", Mutex::new(builder));
    let id = vm.insert_heap_object(obj);
    task.ram.push_i32(id as i32);
    Ok(())
}

fn shim_targzip_add_string(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let content = task.ram.pop_str();
    let name = task.ram.pop_str();
    let handle_id = task.ram.pop_i32() as u64;
    // ... downcast to TarGzip, call append_data()
    let mut header = tar::Header::new_gnu();
    header.set_size(content.len() as u64);
    header.set_path(&name).unwrap();
    builder.append(&header, content.as_bytes()).unwrap();
    Ok(())
}

fn shim_targzip_finish(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let handle_id = task.ram.pop_i32() as u64;
    // ... downcast, call finish(), drop handle
    Ok(())
}
```

#### Step 2: TarExtract 组合 shim

类似地，解压需要 `flate2::read::GzDecoder<std::fs::File>` 包裹 `tar::Archive`：

```rust
type TarExtract = tar::Archive<flate2::read::GzDecoder<std::fs::File>>;

fn shim_tarextract_open(task, vm) { ... }
fn shim_tarextract_list(task, vm) { ... }  // → List<String>
fn shim_tarextract_unpack(task, vm) { ... }
```

#### Step 3: Native ID 分配

| ID | Name | 功能 |
|----|------|------|
| 2900 | `auto.targzip.create` | 创建 tar.gz 压缩文件 |
| 2901 | `auto.targzip.add_string` | 添加字符串内容到 archive |
| 2902 | `auto.targzip.add_file` | 添加文件到 archive |
| 2903 | `auto.targzip.finish` | 完成压缩 |
| 2910 | `auto.tarextract.open` | 打开 tar.gz 文件 |
| 2911 | `auto.tarextract.list` | 列出文件名 |
| 2912 | `auto.tarextract.unpack` | 解压到目录 |

#### Step 4: Codegen 路由

```rust
// 在 codegen 方法路由中添加
("TarGzip", "create") => Some("auto.targzip.create".into()),
("TarGzip", "add") => Some("auto.targzip.add_string".into()),
("TarGzip", "finish") => Some("auto.targzip.finish".into()),
("TarExtract", "open") => Some("auto.tarextract.open".into()),
("TarExtract", "list") => Some("auto.tarextract.list".into()),
("TarExtract", "unpack") => Some("auto.tarextract.unpack".into()),
```

### 依赖变更

`crates/auto-lang/Cargo.toml` 添加：
```toml
tar = "0.4"
flate2 = "1"
```

---

## 不在范围内的 "Impossible" 场景

以下场景需要 VM 核心架构扩展，不在本 Plan 范围内：

### Closure FFI（6 个测试）

**涉及**: rayon, crossbeam
**原因**: Auto 闭包是 VM 字节码 + 捕获环境，无法转换为 Rust `fn` 指针或 `dyn Fn`。
**可能方案**:
- JIT 编译 Auto 闭包为原生代码（工程量巨大）
- Trampoline 模式：为每个闭包调用创建 C ABI 回调桩（复杂，性能差）
- 在 Auto 层面重新实现并行原语（不依赖 Rust crate）

**建议**: 在 Auto 标准库中实现 `par_map`、`par_filter` 等并行原语，而不是桥接 rayon。

### Custom Serde Deserialization（4 个测试）

**涉及**: serde derive, ndarray
**原因**: `serde_json::from_str::<T>()` 需要编译时 `Deserialize` impl，Auto 没有等价的 derive macro 系统。
**可能方案**:
- 运行时类型反射：在 Auto 中定义 type schema，运行时动态解析 JSON
- 类似 Python `dataclasses.from_dict()` 的模式

**建议**: 作为 Auto 类型系统增强的子项目来做（需要 `reflect` 模块）。

---

## 实施计划

### Phase A: 外部迭代器支持（WalkDir + CSV）

| Step | 任务 | 文件 |
|------|------|------|
| A.1 | 添加 walkdir/csv 依赖 | `Cargo.toml` |
| A.2 | WalkDir shim（直接返回 List） | `native.rs` |
| A.3 | CSV Reader shim（返回嵌套 List） | `native.rs` |
| A.4 | 注册 native ID + 返回类型 | `native_registry.rs` |
| A.5 | Codegen 方法路由 | `codegen.rs` |
| A.6 | 验证测试通过 | `test/a2r/cookbook/` |

**预计工作量**: 1 天

### Phase B: 多 Crate 组合支持（TarGzip）

| Step | 任务 | 文件 |
|------|------|------|
| B.1 | 添加 tar/flate2 依赖 | `Cargo.toml` |
| B.2 | TarGzip 组合 shim | `native.rs` |
| B.3 | TarExtract 组合 shim | `native.rs` |
| B.4 | 注册 native ID | `native_registry.rs` |
| B.5 | Codegen 方法路由 | `codegen.rs` |
| B.6 | 验证测试通过 | `test/a2r/cookbook/` |

**预计工作量**: 1 天

### Phase C: same_file 支持

| Step | 任务 | 文件 |
|------|------|------|
| C.1 | same_file shim | `native.rs` |
| C.2 | 验证测试通过 | |

**预计工作量**: 半天

### 执行顺序

```
Phase A (迭代器) → Phase B (tar+flate2) → Phase C (same_file)
```

---

## 成功标准

- [ ] `WalkDir.new("src")` 在 AutoVM 中返回路径列表
- [ ] `csv.read_all("data.csv")` 返回行列表
- [ ] `TarGzip.create("out.tar.gz").add("f.txt", "content").finish()` 完成压缩
- [ ] `TarExtract.open("in.tar.gz").list()` 列出文件
- [ ] 对应 cookbook 测试全部通过
- [ ] 新增 shim 全部注册返回类型（`NativeRetType`）

---

## 与其他 Plan 的关系

| Plan | 关系 |
|------|------|
| **Plan 212** | 本 Plan 是 212 Phase 2.3 Complex 的延续 |
| **Plan 266** | VM↔a2r 对偶测试 — 本 Plan 实现后需要补充对偶测试 |
| **Plan 249** | 统一 native registry — 新 shim 应使用 `#[rust_fn]` 宏 |
| **Plan 250** | Auto stdlib 增强 — WalkDir/CSV/TarGzip 可作为 stdlib 模块暴露 |
