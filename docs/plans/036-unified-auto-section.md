# Plan 036: 统一 Auto Section - 让 Auto 语言实现自己的标准库

## 实施状态

**最后更新**: 2025-01-17

### 阶段完成情况

- ✅ **阶段 1**: 实现文件加载逻辑 (已完成)
  - ✅ 1.1 添加 `get_file_extensions()` 方法
  - ✅ 1.2 修改 `import()` 函数支持文件合并
  - ✅ 1.3 实现自动过滤 section 标记

- ✅ **阶段 2**: 拆分 `io.at` 为多个文件 (已完成)
  - ✅ 2.1 创建 `io.vm.at` (27 行)
  - ✅ 2.2 创建 `io.c.at` (40 行)
  - ✅ 2.3 精简 `io.at` (133 行)
  - ✅ 2.4 测试验证通过 (94 passed)

- ✅ **阶段 3**: 拆分其他标准库文件 (已完成)
  - ✅ 3.1 拆分 `sys.at` → `sys.at` + `sys.c.at`
  - ✅ 3.2 确认 `math.at` 为纯 Auto 代码（无需拆分）
  - ✅ 3.3 确认 `str.at` 为纯 Auto 代码（无需拆分）
  - ✅ 3.4 测试验证通过 (94 passed)
  - ✅ 3.5 修复测试失败 (2025-01-16)
    - ✅ 修复 test_105_std_str (更新为 Plan 025 新字符串系统)
    - ✅ 修复 test_017_spec (spec 方法调用语法)
    - ✅ 修复 test_111_io_specs (spec 方法调用语法)
    - ✅ 修复 test_vm_function_error (错误处理预期)
  - ✅ 3.6 全部测试通过 (551 passed)

- ⏸️ **阶段 4**: 添加用 Auto 实现的示例方法 (部分完成 - 2025-01-17)
  - ✅ 4.1 添加 str.char_count() 方法
  - ✅ 4.2 添加 str.split() 方法签名 (placeholder 实现)
    - ✅ Parser 现在支持数组返回类型 []str (Plan 037 Phase 3)
    - ✅ 方法签名可以正确解析
    - ✅ C transpiler 生成正确的函数签名
    - ✅ 554 tests passing
    - ⏸️ 完整实现需要更多表达式支持 (待完成)
  - ⏸️ 4.3 File 高级方法 (需要更多实现)
    - read_all() 需要复杂 while 条件
    - write_lines() 需要数组索引支持
  - ✅ 4.4 添加 TODO 注释说明未来工作
- ⏸️ **阶段 5**: 文档和测试 (待实施)

### 标准库文件拆分状态

| 文件 | 状态 | 说明 |
|------|------|------|
| `io.at` | ✅ 已拆分 | `io.at` + `io.vm.at` + `io.c.at` |
| `sys.at` | ✅ 已拆分 | `sys.at` + `sys.c.at` |
| `math.at` | ✅ 无需拆分 | 纯 Auto 代码 |
| `str.at` | ✅ 无需拆分 | 纯 Auto 代码 (ext statements) |

### 核心创新：文件合并策略

采用用户建议的优化方案，将多个文件内容合并后统一解析：

```rust
// 实现方式 (parser.rs:1975-1987)
let merged_content: String = file_contents
    .iter()
    .map(|(content, _)| {
        content
            .lines()
            .filter(|line| !line.trim().starts_with("# ") && !line.trim().starts_with("#\t"))
            .collect::<Vec<_>>()
            .join("\n")
    })
    .collect::<Vec<_>>()
    .join("\n\n");
```

**优势**：
- 无需考虑 scope 依赖问题
- 合并后的文件就像原来的单一文件
- 自动过滤 section 标记，避免与现有机制冲突
- 保持向后兼容（fallback 到原始 .at 文件）

---

## Executive Summary

重新设计标准库的组织形式，**将原来的单一文件拆分为多个文件**，通过文件后缀区分不同场景的代码：
- `.at` - 纯 Auto 代码（所有场景加载）
- `.vm.at` - VM 专用代码（只有解释器加载）
- `.c.at` - C 专用代码（只有转译器加载）

**关键设计**: 加载顺序遵循分层架构原则，先加载底层（场景相关），再加载上层（通用）：
- 解释器: `io.vm.at` → `io.at`
- 转译器: `io.c.at` → `io.at`

**目标**: 通过文件分离实现清晰的分层架构，让 Auto 语言成为标准库的主要实现语言，`fn.vm` 和 `fn.c` 只作为底层接口

**预计工期**: 8-12 小时
**实际工期**: 阶段 1-2 已完成 (约 4-5 小时)

---

## 问题分析

### 当前实现的问题

**文件**: `stdlib/auto/io.at` (以及所有标准库文件)

#### 问题 1: 类型重复声明

```auto
# AUTO
type File {
    fn.vm close()
    fn.vm read_text() str
    fn.vm write_line(s str)
    // ... 15 个方法，只有声明
}

# C
type File {              // ← 重复声明！
    path str
    file *FILE

    fn read_text() str {  // ← C 实现
        let buf cstr = c"..."
        fgets(buf, 40, .file)
        buf
    }

    fn close() {         // ← C 实现
        fclose(.file)
    }
    // ... 重复所有方法
}
```

**影响**:
- File 类型声明了两次，维护成本高
- 两边的类型定义可能不同步
- 违反 DRY (Don't Repeat Yourself) 原则
- **单文件过大**：在同一个文件中跳转，编辑困难

#### 问题 2: 函数重复实现

```auto
# AUTO
fn say(msg str) {
    print(msg)  // 调用 Auto 的 print
}

# C
fn say(msg str) {        // ← 完全不同的实现！
    printf(c"%s\n", msg)
}
```

**影响**:
- 同一个函数有两个完全不同的实现
- 无法共享逻辑
- 如果要修改行为，需要改两个地方

#### 问题 3: Auto 语言无法实现自己的标准库

当前标准库中：
- `# AUTO` section: 只包含类型声明和 `fn.vm` 函数声明，实际实现在 Rust
- `# C` section: 包含完整的 C 实现

**根本问题**: 没有一个地方可以用 Auto 语言实现共同的逻辑。

**例子**: 想要实现 `File.read_all()` 方法读取整个文件：
- ❌ 不能在 `# AUTO` 中实现（因为需要调用 `fn.vm` 的方法）
- ❌ 不能在 `# C` 中实现（因为那是 C 代码，不是 Auto）
- ✅ 应该有一个共同的地方用 Auto 实现

### 当前 Section 过滤逻辑

**文件**: `crates/auto-lang/src/parser.rs` (lines 441-510)

```rust
pub enum CodeSection {
    None,
    C,
    Rust,
    Auto,     // ← 当前：只有解释器加载
}

// Section 名称映射
match section.as_str() {
    "C" => current_section = CodeSection::C,
    "RUST" => current_section = CodeSection::Rust,
    "AUTO" => current_section = CodeSection::Auto,  // ← 当前名称
}

// 过滤逻辑
match self.compile_dest {
    CompileDest::Interp => match current_section {
        CodeSection::None | CodeSection::Auto => {}  // ← 只加载 AUTO
        _ => { self.skip_line()?; continue; }
    },
    CompileDest::TransC => match current_section {
        CodeSection::None | CodeSection::C => {}     // ← 只加载 C
        _ => { self.skip_line()?; continue; }
    },
    CompileDest::TransRust => match current_section {
        CodeSection::None | CodeSection::Rust => {}  // ← 只加载 RUST
        _ => { self.skip_line()?; continue; }
    },
}
```

**关键问题**:
1. 当前 `CodeSection::Auto` 只被解释器加载，不被转译器加载
2. 所有代码混在同一个文件中，通过 `#` section 切换
3. **没有清晰的分层架构**：底层接口和上层实现混在一起

---

## 设计方案

### 新的文件组织方式

**核心思想**: 通过文件后缀分离不同场景的代码，而不是在同一个文件中用 `#` section 切换。

#### 文件命名规则

| 文件后缀 | 用途 | 加载场景 | 架构层次 |
|---------|------|---------|---------|
| `.at` | 纯 Auto 代码 | **所有场景** | 上层（业务逻辑） |
| `.vm.at` | VM 专用代码 | **仅解释器** | 底层（系统接口） |
| `.c.at` | C 专用代码 | **仅转译器** | 底层（系统接口） |

#### 加载顺序（分层架构）

**关键原则**: 先加载底层（场景相关），再加载上层（通用）

```
解释器模式: use io
  1. 先加载: io.vm.at (底层 - 提供 fn.vm 接口)
  2. 再加载: io.at    (上层 - 调用 fn.vm 实现高级功能)

转译器模式: use io
  1. 先加载: io.c.at   (底层 - 提供 fn.c 接口)
  2. 再加载: io.at    (上层 - 调用 fn.c 实现高级功能)
```

**为什么这个顺序很重要**？
- `io.at` 需要调用 `io.vm.at` 中定义的 `fn.vm` 函数
- 底层文件提供接口，上层文件实现业务逻辑
- 符合"底层先定义，上层后调用"的依赖关系

#### 新的文件结构示例

**`stdlib/auto/` 目录结构**:
```
stdlib/auto/
├── io.at       ← 纯 Auto 代码（所有场景）
├── io.vm.at    ← VM 专用代码（解释器）
├── io.c.at     ← C 专用代码（转译器）
├── math.at     ← 纯 Auto 代码（所有场景）
├── math.vm.at  ← VM 专用代码（解释器）
└── math.c.at   ← C 专用代码（转译器）
```

**`io.at` (纯 Auto 代码 - 所有场景)**:

```auto
# ============================================================================
# AUTO - 共同代码（所有场景都会加载）
# ============================================================================

// 类型声明（只声明一次！）
type File {
    path str

    // 共同的 Auto 方法实现
    fn read_all() str {
        let result = ""
        while !self.is_eof() {
            let line = self.read_line()
            if self.is_eof() {
                break
            }
            result = result.append(line)
        }
        result
    }

    fn read_lines() [str] {
        // Auto 实现的复杂逻辑
        let lines = []
        while !self.is_eof() {
            lines.push(self.read_line())
        }
        lines
    }

    // VM/C 专用方法声明（不实现）
    fn.vm close()
    fn.vm read_text() str
    fn.vm is_eof() bool
}

// 共同的 Auto 函数实现
fn say(msg str) {
    print(msg)
    print("\n")
}

fn read_file(path str) str {
    let f = open(path)
    let content = f.read_all()
    f.close()
    content
}

# ============================================================================
# VM - VM 专用代码（只有解释器加载）
# ============================================================================

// VM 函数实现（在 Rust 中）
fn.vm open(path str) File
fn.vm open_read(path str) File
fn.vm open_write(path str) File

// File 类型的 VM 方法实现
impl File {
    fn.vm close()
    fn.vm read_text() str
    fn.vm read_line() str
    fn.vm is_eof() bool
}

# ============================================================================
# C - C 专用代码（只有转译器加载）
# ============================================================================

use.c <stdio.h>
use.c <stdlib.h>

// C 外部函数声明
fn.c fopen(filename cstr, mode cstr) *FILE
fn.c fclose(stream *FILE)
fn.c fgets(buf cstr, n int, stream *FILE) cstr
type.c FILE

// File 类型的 C 字段
impl File {
    file *FILE  // C 特定字段
}

// File 类型的 C 方法实现
impl File {
    fn read_text() str {
        let buf cstr = c"..."
        fgets(buf, 40, .file)
        buf
    }

    fn close() {
        fclose(.file)
    }
}

// open 函数的 C 实现
fn open(path str) File {
    let mode = c"r"
    let f = fopen(path, mode)
    File { path: path, file: f }
}
```

### 关键改进

1. **物理分离**: 不同场景的代码在不同文件中，清晰直观
2. **类型只声明一次**: `type File` 在 `io.at` 中声明，不再重复
3. **分层架构**: 底层文件（`.vm.at`/`.c.at`）提供接口，上层文件（`.at`）实现逻辑
4. **性能优化**: 解释器不读取 `.c.at`，转译器不读取 `.vm.at`
5. **清晰的职责分离**:
   - `.at`: 业务逻辑、算法、数据处理（纯 Auto）
   - `.vm.at`: 系统调用、文件 I/O（Rust 实现）
   - `.c.at`: 系统调用、文件 I/O（C 实现）
6. **加载顺序正确**: 先加载底层（`.vm.at`/`.c.at`），再加载上层（`.at`）
7. **场景特定字段**: 使用 `# C { ... }` 语法声明特定场景的字段
8. **函数声明语义**: 无 body 的函数声明表示需要底层实现

### 语法设计

#### 1. 无 body 函数声明

**语义**: 表示此函数需要在底层实现（VM 或 C）

```auto
// io.at
type File {
    fn read_text() str  // ← 无 body，需要底层实现
    fn close()          // ← 无 body，需要底层实现

    fn read_all() str {  // ← 有 body，用 Auto 实现
        // ...
    }
}
```

**对应关系**:
- `.at` 中: `fn read_text() str` (无 body)
- `.vm.at` 中: `fn.vm File.read_text() str` (VM 实现)
- `.c.at` 中: 可以有 C 实现，或者使用 `fn.c` 声明

#### 2. 场景特定字段语法

**语法**: `# C { ... }` / `# VM { ... }` / `# }`

**语义**: 内容只在特定场景中生效

```auto
type File {
    path str

# C {
    file *FILE  // 只在 C 场景中定义
# }

    fn read_text() str { /* ... */ }
}
```

**编译时处理**:
- 解释器: 忽略 `# C { ... }` 块，File 只有 `path` 字段
- 转译器: 读取 `# C { ... }` 块，File 有 `path` 和 `file *FILE` 字段

**多行支持**:

```auto
type File {
    path str

# C {
    file *FILE
    mode int
    buffer *byte
# }

    fn close() { /* ... */ }
}
```

#### 3. 未来工作: 条件编译逻辑

**暂不实现**: `#if...else` 编译期逻辑

**未来可能语法** (仅示例，未实现):

```auto
fn open(path str) File {
# if C {
    let mode = c"r"
    let f = fopen(path, mode)
    File { path: path, file: f }
# } else if VM {
    // VM 实现
    File { path: path, handle: open_file_vm(path) }
# } else {
    // 默认实现
    File { path: path }
# }
}
```

### 向后兼容

**旧的 section 方式仍然支持**:
- 如果没有 `.vm.at` 或 `.c.at` 文件，则回退到旧的 section 查找
- 例如：`io.at` 中仍可包含 `# VM` 和 `# C` section
- 两种方式可以共存，逐步迁移

---

## 实现计划

### 阶段 1: 实现文件加载逻辑

**工期**: 2-3 小时
**依赖**: 无
**风险**: 中

#### 1.1 修改 `use` 语句解析逻辑

**文件**: `crates/auto-lang/src/parser.rs`

**当前逻辑**: `use io` → 加载 `io.at`
**新逻辑**: `use io` → 加载多个文件，按顺序

**实现步骤**:

1. **添加文件后缀识别函数**:

```rust
impl<'a> Parser<'a> {
    /// 根据编译目标确定要加载的文件后缀
    fn get_file_extensions(&self) -> Vec<&'static str> {
        match self.compile_dest {
            CompileDest::Interp => vec![".vm.at", ".at"],     // 解释器: 先底层后上层
            CompileDest::TransC => vec![".c.at", ".at"],      // 转译器: 先底层后上层
            CompileDest::TransRust => vec![".rust.at", ".at"], // Rust转译器
        }
    }

    /// 检查文件是否存在
    fn file_exists(&self, path: &Path) -> bool {
        path.exists()
    }
}
```

2. **修改 `use` 语句处理**:

```rust
// 在 parse_use 或相关函数中
fn parse_use_import(&mut self, path: AutoStr) -> AutoResult<()> {
    let extensions = self.get_file_extensions();
    let mut loaded_files = Vec::new();

    // 按顺序尝试加载文件
    for ext in extensions {
        let file_path = format!("{}{}", path, ext);
        if self.file_exists(Path::new(&file_path)) {
            // 加载这个文件
            self.parse_file(&file_path)?;
            loaded_files.push(file_path);
        }
    }

    // 如果都没有找到，回退到原始路径（向后兼容）
    if loaded_files.is_empty() {
        self.parse_file(&path.to_string())?;
    }

    Ok(())
}
```

**关键点**:
- **加载顺序**: 先 `.vm.at`/`.c.at`（底层），再 `.at`（上层）
- **向后兼容**: 如果没有找到 `.vm.at` 或 `.c.at`，则尝试加载原始路径
- **只加载存在的文件**: 如果 `io.vm.at` 不存在，不报错，继续加载 `io.at`

#### 1.2 更新标准库查找逻辑

**文件**: `crates/auto-lang/src/util.rs` (如果需要)

**当前逻辑**: 查找 `io.at`
**新逻辑**: 查找 `io.vm.at`, `io.c.at`, `io.at`

```rust
pub fn find_std_lib_files(name: &str) -> AutoResult<Vec<PathBuf>> {
    let std_path = find_std_lib()?;
    let base_path = PathBuf::from(std_path.to_string()).join(format!("{}.at", name));

    let mut files = Vec::new();

    // 解释器模式: 优先查找 .vm.at
    if cfg!(feature = "interpreter") {
        let vm_path = base_path.with_extension("vm.at");
        if vm_path.exists() {
            files.push(vm_path);
        }
    }

    // 转译器模式: 优先查找 .c.at
    if cfg!(feature = "transpiler") {
        let c_path = base_path.with_extension("c.at");
        if c_path.exists() {
            files.push(c_path);
        }
    }

    // 总是添加基础 .at 文件
    if base_path.exists() {
        files.push(base_path);
    }

    Ok(files)
}
```

**成功标准**:
- [ ] `use io` 正确加载 `io.vm.at` + `io.at`（解释器）
- [ ] `use io` 正确加载 `io.c.at` + `io.at`（转译器）
- [ ] 文件加载顺序正确（先底层后上层）
- [ ] 向后兼容：没有 `.vm.at`/`.c.at` 时仍能工作

---

### 阶段 2: 拆分 `io.at` 为多个文件

**工期**: 2-3 小时
**依赖**: 阶段 1
**风险**: 中

#### 2.1 创建 `io.vm.at`

**文件**: `stdlib/auto/io.vm.at` (新建)

**内容**: 将当前 `io.at` 中的 `# AUTO` section 的底层部分移到这里

```auto
// ============================================================================
// io.vm.at - VM 专用实现（只有解释器加载）
// ============================================================================

// Spec declarations for polymorphic I/O
spec Reader {
    fn read_line() str
    fn is_eof() bool
}

spec Writer {
    fn write_line(s str)
    fn flush()
}

spec Seekable {
    fn seek(offset int, origin int) int
    fn tell() int
    fn rewind()
}

enum SeekOrigin {
    Set = 0
    Cur = 1
    End = 2
}

// 底层 VM 函数（实现在 Rust 中）
// 注意: 不需要重复声明 type File，只在 io.at 中声明一次
fn.vm open(path str) File
fn.vm open_read(path str) File
fn.vm open_write(path str) File
fn.vm open_append(path str) File
fn.vm stdin() File
fn.vm stdout() File
fn.vm stderr() File

// File 类型的底层 VM 方法实现
fn.vm File.close()
fn.vm File.read_text() str
fn.vm File.read_line() str
fn.vm File.write_line(s str)
fn.vm File.flush()
fn.vm File.getc() int
fn.vm File.putc(c int)
fn.vm File.ungetc(c int)
fn.vm File.read(buf []byte, size int, count int) int
fn.vm File.write(buf []byte, size int, count int) int
fn.vm File.gets(buf []byte) str
fn.vm File.puts(s str)
fn.vm File.seek(offset int, origin int) int
fn.vm File.tell() int
fn.vm File.rewind()
fn.vm File.is_eof() bool
fn.vm File.has_error() bool
fn.vm File.clear_error()
```

#### 2.2 创建 `io.c.at`

**文件**: `stdlib/auto/io.c.at` (新建)

**内容**: 保持当前 `io.at` 中的 `# C` section 不变

```auto
// ============================================================================
// io.c.at - C 专用实现（只有转译器加载）
// ============================================================================

use.c <stdlib.h>
fn.c exit(status int) void

use.c <stdio.h>
fn.c printf(fmt cstr, arg cstr)
type.c FILE
fn.c getline(lineptr *cstr, n *int, stream *FILE) int
let c stdin *FILE
let c stdout *FILE
let c stderr *FILE

fn.c fopen(filename cstr, mode cstr) *FILE
fn.c fclose(stream *FILE)
fn.c fgets(buf cstr, n int, stream *FILE) cstr
fn.c fputs(s cstr, stream *FILE) int
fn.c fflush(stream *FILE) int

fn.c fgetc(stream *FILE) int
fn.c fputc(c int, stream *FILE) int
fn.c ungetc(c int, stream *FILE) int

fn.c fread(ptr *void, size int, count int, stream *FILE) int
fn.c fwrite(ptr *void, size int, count int, stream *FILE) int

fn.c fseek(stream *FILE, offset int, origin int) int
fn.c ftell(stream *FILE) int
fn.c rewind(stream *FILE)

fn.c feof(stream *FILE) int
fn.c ferror(stream *FILE) int
fn.c clearerr(stream *FILE)

// File 方法的 C 实现
// 注意: 不需要重复声明 type File，只在 io.at 中声明一次
// 这里只提供 C 实现或 fn.c 外部声明

fn File.read_text() str {
    let buf cstr = c"                                        "
    fgets(buf, 40, .file)
    buf
}

fn File.read_line() str {
    let buf cstr = c"                                                                                "
    fgets(buf, 80, .file)
    buf
}

fn File.write_line(s str) {
    fputs(s, .file)
    fputs(c"\n", .file)
}

fn File.close() {
    fclose(.file)
}

fn File.flush() {
    fflush(.file)
}

fn File.getc() int {
    fgetc(.file)
}

fn File.putc(c int) {
    fputc(c, .file)
}

fn File.ungetc(c int) {
    ungetc(c, .file)
}

fn File.read(buf []byte, size int, count int) int {
    fread(buf, size, count, .file)
}

fn File.write(buf []byte, size int, count int) int {
    fwrite(buf, size, count, .file)
}

fn File.gets(buf []byte) str {
    fgets(buf, 80, .file)
    buf
}

fn File.puts(s str) {
    fputs(s, .file)
}

fn File.seek(offset int, origin int) int {
    fseek(.file, offset, origin)
}

fn File.tell() int {
    ftell(.file)
}

fn File.rewind() {
    rewind(.file)
}

fn File.is_eof() bool {
    feof(.file) != 0
}

fn File.has_error() bool {
    ferror(.file) != 0
}

fn File.clear_error() {
    clearerr(.file)
}

// open 函数的 C 实现
fn open(path str) File {
    let mode = c"r"
    let f = fopen(path, mode)
    File { path: path, file: f }
}

fn open_read(path str) File {
    let mode = c"r"
    let f = fopen(path, mode)
    File { path: path, file: f }
}

fn open_write(path str) File {
    let mode = c"w"
    let f = fopen(path, mode)
    File { path: path, file: f }
}

fn open_append(path str) File {
    let mode = c"a"
    let f = fopen(path, mode)
    File { path: path, file: f }
}

fn stdin() File {
    File { path: c"<stdin>", file: stdin }
}

fn stdout() File {
    File { path: c"<stdout>", file: stdout }
}

fn stderr() File {
    File { path: c"<stderr>", file: stderr }
}

fn say(msg str) {
    printf(c"%s\n", msg)
}
```

#### 2.3 精简 `io.at`

**文件**: `stdlib/auto/io.at` (修改)

**内容**: 只保留纯 Auto 代码，删除 `# C` section

```auto
// ============================================================================
// io.at - 纯 Auto 实现（所有场景都会加载）
// ============================================================================

// 类型声明（只声明一次！）
type File {
    path str

# C {
    file *FILE  // 只在 C 场景中定义
# }

    // 底层方法声明（无 body = 需要底层实现）
    fn close()
    fn read_text() str
    fn read_line() str
    fn write_line(s str)
    fn flush()
    fn is_eof() bool

    // 用 Auto 实现的高级方法
    fn read_all() str {
        let result = ""
        while !self.is_eof() {
            let line = self.read_line()
            if self.is_eof() && line.len() == 0 {
                break
            }
            result = result.append(line)
        }
        result
    }

    fn read_lines() [str] {
        let lines = []
        while !self.is_eof() {
            let line = self.read_line()
            if self.is_eof() && line.len() == 0 {
                break
            }
            lines.push(line)
        }
        lines
    }

    fn write_lines(lines [str]) {
        let i = 0
        while i < lines.len() {
            self.write_line(lines[i])
            i = i + 1
        }
    }
}

// 底层函数声明（无 body = 需要底层实现）
fn open(path str) File
fn open_read(path str) File
fn open_write(path str) File
fn open_append(path str) File

// 共同的 Auto 函数实现
fn say(msg str) {
    print(msg)
    print("\n")
}

// 高级函数（用 Auto 实现业务逻辑）
fn read_file(path str) str {
    let f = open(path)           // 调用底层 open (在 .vm.at/.c.at 中)
    let content = f.read_all()   // 调用 Auto 方法
    f.close()                    // 调用底层 close
    content
}
```

**关键**:
- `type File` 只声明一次（在 `io.at` 中）
- `# C { ... }` 块标记 C 特定字段
- 无 body 的函数表示需要底层实现
- 有 body 的函数是纯 Auto 实现

#### 2.4 处理 C 特定字段

**挑战**: `io.c.at` 中的 `File` 类型有 `file *FILE` 字段，但 `io.at` 中不应该有这个 C 特定字段。

**解决方案**: 使用 `# C { ... }` 语法（已设计）

```auto
// io.at
type File {
    path str

# C {
    file *FILE  // 只在 C 场景中定义
# }

    fn read_text() str  // 无 body，需要底层实现
    fn close()          // 无 body，需要底层实现
}
```

**优势**:
- ✅ 类型只声明一次
- ✅ 清晰的场景特定字段标记
- ✅ 不需要实现 `extend` 机制
- ✅ C 类型不泄漏到其他场景

**实现要求**:
- 解释器: 忽略 `# C { ... }` 块
- 转译器: 读取 `# C { ... }` 块
- 编译时处理，类似 C 预处理器

**在 `.c.at` 中不需要重复声明**:

```auto
// io.c.at
use.c <stdio.h>

// File 的 C 实现（不需要重复声明 type File）
fn File.read_text() str {
    let buf cstr = c"..."
    fgets(buf, 40, .file)
    buf
}

fn File.close() {
    fclose(.file)
}
```

**关键**:
- `io.at` 中声明完整的 `type File`，包括 `# C { ... }` 块
- `io.c.at` 中**不需要**重复声明 `type File`
- `io.c.at` 只提供 C 实现或 `fn.c` 外部声明

**成功标准**:
- [ ] `io.vm.at` 创建完成
- [ ] `io.c.at` 创建完成
- [ ] `io.at` 精简为纯 Auto 代码
- [ ] 所有测试通过（解释器和转译器）

---

### 阶段 3: 拆分其他标准库文件

**工期**: 2-3 小时
**依赖**: 阶段 1, 2
**风险**: 中

**文件列表**:
- `math.at` → `math.at`, `math.vm.at`, `math.c.at`
- `sys.at` → `sys.at`, `sys.vm.at`, `sys.c.at`
- `str.at` → `str.at` (可能只需要 `.at`，因为已经有 ext 实现)

**迁移策略**: 逐个文件迁移，每次迁移后运行完整测试。

**注意**: `str.at` 中的 ext 方法已经在 `str.at` 中实现，不需要 `.vm.at` 或 `.c.at`（除非有底层方法）。

**成功标准**:
- [ ] 所有标准库文件拆分完成
- [ ] 所有测试通过（解释器和转译器）

2. **方案 B**: 使用条件编译
   ```auto
   # AUTO
   type File {
       path str
       #if C
       file *FILE
       #endif
   }
   ```
   - ✅ 清晰分离
   - ❌ 需要实现条件编译

3. **方案 C**: 在 `# C` 中扩展类型
   ```auto
   # AUTO
   type File {
       path str
   }

   # C
   extend File {
       file *FILE
   }
   ```
   - ✅ 符合"扩展"语义
   - ❌ 需要实现 `extend` 机制

4. **方案 D**: 使用 impl 块（推荐）
   ```auto
   # AUTO
   type File {
       path str
   }

   # C
   impl File {
       file *FILE  // C 特定字段

       fn read_text() str {
           // 可以访问 .file
       }
   }
   ```
   - ✅ 不需要新语法
   - ❌ `impl` 块可能不支持添加字段

**建议**: 先使用方案 A（在 `# AUTO` 中声明所有字段），后续考虑实现 `extend` 机制。

#### 3.3 迁移其他标准库文件

**文件列表**:
- `stdlib/auto/math.at` → `math.at`, `math.vm.at`, `math.c.at`
- `stdlib/auto/sys.at` → `sys.at`, `sys.vm.at`, `sys.c.at`
- `stdlib/auto/str.at` → `str.at` (已有的 ext 方法，可能不需要 .vm.at/.c.at)

**迁移策略**: 逐个文件迁移，每次迁移后运行完整测试。

**注意**: `str.at` 中的 ext 方法已经在 `str.at` 中实现，不需要 `.vm.at` 或 `.c.at`（除非有底层方法）。

**成功标准**:
- [ ] 所有标准库文件拆分完成
- [ ] 所有测试通过（解释器和转译器）

---

### 阶段 4: 添加用 Auto 实现的示例方法

**工期**: 1-2 小时
**依赖**: 阶段 3
**风险**: 低

#### 4.1 实现复杂字符串方法

**文件**: `stdlib/auto/str.at`

**添加用 Auto 实现的方法**:

```auto
# AUTO
ext str {
    // VM/C 专用方法声明
    fn.vm len() int
    fn.vm upper() str
    fn.vm lower() str
    fn.vm contains(pattern str) bool

    // 用 Auto 实现的复杂方法
    fn split(delimiter str) [str] {
        let result = []
        let current = ""
        let i = 0

        while i < self.len() {
            if self.sub(i, i + delimiter.len()) == delimiter {
                result.push(current)
                current = ""
                i = i + delimiter.len()
            } else {
                current = current.append(self.char_at(i))
                i = i + 1
            }
        }

        if current.len() > 0 {
            result.push(current)
        }

        result
    }

    fn join(parts [str], delimiter str) str {
        if parts.len() == 0 {
            return ""
        }

        let result = parts[0]
        let i = 1

        while i < parts.len() {
            result = result.append(delimiter)
            result = result.append(parts[i])
            i = i + 1
        }

        result
    }

    fn words() [str] {
        self.split(" ")
    }

    fn lines() [str] {
        self.split("\n")
    }

    fn trim() str {
        // 用 Auto 实现的 trim 逻辑
        let start = 0
        let end = self.len()

        while start < end && self.char_at(start) == " " {
            start = start + 1
        }

        while end > start && self.char_at(end - 1) == " " {
            end = end - 1
        }

        self.sub(start, end)
    }
}
```

**优势**:
- ✅ 展示 Auto 语言的表达能力
- ✅ 这些方法在所有场景中都可用
- ✅ 减少对 VM/C 的依赖

#### 4.2 实现 File 的高级方法

**文件**: `stdlib/auto/io.at`

```auto
# AUTO
type File {
    // ...

    fn read_all() str {
        let result = ""
        while !self.is_eof() {
            let line = self.read_line()
            if self.is_eof() && line.len() == 0 {
                break
            }
            result = result.append(line)
        }
        result
    }

    fn write_lines(lines [str]) {
        let i = 0
        while i < lines.len() {
            self.write_line(lines[i])
            i = i + 1
        }
    }

    fn copy_from(other File) {
        while !other.is_eof() {
            let line = other.read_line()
            self.write_line(line)
        }
    }
}
```

**成功标准**:
- [ ] 添加至少 3 个用 Auto 实现的复杂方法
- [ ] 这些方法在解释器和转译器中都能正常工作
- [ ] 添加测试用例验证功能

---

### 阶段 5: 文档和测试

**工期**: 1 小时
**依赖**: 阶段 4
**风险**: 低

#### 5.1 更新文档

**创建**: `docs/tutorials/stdlib-organization.md`

```markdown
# 标准库组织结构

## Section 语义

AutoLang 标准库文件分为三个 section：

### # AUTO - 共同代码
**用途**: 所有编译目标都会加载
**内容**:
- 类型声明
- 用 Auto 语言实现的函数和方法
- 业务逻辑和算法

**例子**:
```auto
# AUTO
fn say(msg str) {
    print(msg)
    print("\n")
}

ext str {
    fn split(delimiter str) [str] {
        // Auto 实现
    }
}
```

### # VM - VM 专用代码
**用途**: 只有解释器会加载
**内容**:
- `fn.vm` 函数声明（实现在 Rust 中）
- 系统调用、文件 I/O、网络操作等底层功能

**例子**:
```auto
# VM
fn.vm open(path str) File
fn.vm getpid() int
```

### # C - C 专用代码
**用途**: 只有 C 转译器会加载
**内容**:
- `fn.c` 外部函数声明
- `use.c` C 头文件导入
- C 特定的类型和实现

**例子**:
```auto
# C
use.c <stdio.h>
fn.c printf(fmt cstr, ...)
```

## 设计原则

1. **优先用 Auto 实现**: 大部分功能应该用 Auto 语言实现
2. **最小化 VM/C 依赖**: 只在必要时使用 `fn.vm` 或 `fn.c`
3. **避免重复**: 类型只声明一次，避免在多个 section 中重复
4. **清晰分离**: 共同逻辑在 `# AUTO`，平台特定在 `# VM`/`# C`
```

#### 5.2 添加测试

**创建**: `test/a2c/037_unified_section/`

**测试 1: 共同函数测试**
```auto
// unified_functions.at
# AUTO
fn add(a int, b int) int {
    a + b
}

# VM
fn.vm subtract(a int, b int) int

# C
fn.c subtract(a int, b int) int

fn main() {
    print(add(1, 2))      // 3 - 用 Auto 实现
    print(subtract(5, 2)) // 3 - 用 VM/C 实现
}
```

**测试 2: 共同类型测试**
```auto
// unified_types.at
# AUTO
type Point {
    x int
    y int

    fn distance() int {
        // 用 Auto 实现的复杂逻辑
        let x2 = self.x * self.x
        let y2 = self.y * self.y
        // 简化版本
        x2 + y2
    }
}

# VM
fn.vm get_origin() Point

# C
fn.c get_origin() Point

fn main() {
    let p = get_origin()
    print(p.distance())
}
```

**运行测试**:
```bash
# 解释器测试
cargo run -- --run test/a2c/037_unified_section/unified_functions.at

# 转译器测试
cargo test -p auto-lang test_037_unified_section

# 完整测试套件
cargo test -p auto-lang
```

**成功标准**:
- [ ] 文档完整清晰
- [ ] 添加至少 2 个测试用例
- [ ] 所有测试通过（解释器和转译器）

---

## 关键文件

### 需要修改的文件

1. **`crates/auto-lang/src/parser.rs`**
   - 添加 `get_file_extensions()` 方法
   - 修改 `use` 语句处理逻辑，支持加载多个文件
   - 实现文件加载顺序（先 `.vm.at`/`.c.at`，后 `.at`）

2. **`stdlib/auto/io.vm.at`** (新建)
   - VM 专用代码，包含 `fn.vm` 声明

3. **`stdlib/auto/io.c.at`** (新建)
   - C 专用代码，包含 `fn.c` 声明和 C 实现

4. **`stdlib/auto/io.at`** (修改)
   - 精简为纯 Auto 代码
   - 添加用 Auto 实现的高级方法

5. **`stdlib/auto/math.vm.at`**, **`math.c.at`**, **`math.at`** (同上模式)
6. **`stdlib/auto/sys.vm.at`**, **`sys.c.at`**, **`sys.at`** (同上模式)

### 新建文件

6. **`docs/tutorials/stdlib-organization.md`**
   - 标准库组织结构文档

7. **`test/a2c/037_unified_section/`**
   - `unified_functions.at` + `.expected.c/.h`
   - `unified_types.at` + `.expected.c/.h`

---

## 实现顺序

### 推荐执行顺序

1. **阶段 1** (实现文件加载逻辑) - 基础设施，必须首先完成
2. **阶段 2** (拆分 io.at) - 核心工作，验证设计
3. **阶段 3** (拆分其他标准库文件) - 全面迁移
4. **阶段 4** (添加示例) - 展示新设计的价值
5. **阶段 5** (文档和测试) - 确保长期可维护性

### 关键路径

**最短可行路径** (如果时间有限):
- 阶段 1: 实现文件加载逻辑
- 阶段 2: 只拆分 `io.at` 作为示例
- 阶段 5: 基本文档和测试

**完整路径** (推荐):
- 所有阶段按顺序执行
- 迁移所有标准库文件
- 添加丰富的 Auto 实现示例

---

## 风险分析

### 风险 1: 破坏现有代码
**影响**: 高
**概率**: 中
**缓解**:
- 保持向后兼容（旧的 `# AUTO` 内容可以移到 `# VM`）
- 全面的测试套件验证
- 渐进式迁移（一次迁移一个文件）

### 风险 2: C 特定字段的处理
**影响**: 中
**概率**: 高
**缓解**:
- 先使用简单的方案（在 `# AUTO` 中声明所有字段）
- 后续实现 `extend` 机制
- 文档中说明临时方案

### 风险 3: 性能影响
**影响**: 低
**概率**: 低
**缓解**:
- Auto 实现的性能应该足够好（编译后优化）
- 关键路径仍可用 `fn.vm`/`fn.c`
- 可以后续添加性能分析和优化

### 风险 4: Section 语义混淆
**影响**: 中
**概率**: 中
**缓解**:
- 清晰的文档说明每个 section 的用途
- 代码注释和示例
- IDE 支持（语法高亮不同的 section）

---

## 验证步骤

### 端到端测试

1. **创建测试文件** `test_unified.at`:
```auto
# AUTO
fn greet(name str) str {
    "hello, " + name
}

# VM
fn.vm get_name() str

# C
fn.c get_name() str

fn main() {
    let name = get_name()
    let msg = greet(name)
    print(msg)
}
```

2. **解释器测试**:
```bash
cargo run -- --run test_unified.at
# 应该输出: hello, <name>
```

3. **转译器测试**:
```bash
cargo run -- --c test_unified.at
# 检查生成的 C 代码
```

### 标准库测试

```bash
# 运行所有测试
cargo test -p auto-lang

# 运行解释器测试
cargo run -- --run stdlib/auto/io.at

# 运行转译器测试
cargo test -p auto-lang -- trans

# 检查生成的代码
cargo run -- --c stdlib/auto/io.at
cat io.c
cat io.h
```

---

## 成功标准

### 必须有 (MVP)
- [ ] `CodeSection` 枚举更新完成，包含 `VM` variant
- [ ] 过滤逻辑更新：所有编译目标都加载 `# AUTO`
- [ ] `io.at` 迁移到新的三 section 结构
- [ ] 至少一个用 Auto 实现的复杂方法
- [ ] 所有现有测试通过
- [ ] 基本文档说明新结构

### 应该有
- [ ] 所有标准库文件迁移完成
- [ ] 至少 5 个用 Auto 实现的复杂方法
- [ ] 完整的文档和教程
- [ ] 专门的测试用例验证新功能

### 可以有
- [ ] `extend` 机制实现（处理 C 特定字段）
- [ ] 性能基准测试
- [ ] IDE 支持不同的 section
- [ - 自动迁移工具

---

## 时间线估算

| 阶段 | 工期 | 依赖 |
|-------|------|------|
| 阶段 1: 实现文件加载逻辑 | 2-3 小时 | 无 |
| 阶段 2: 拆分 io.at | 2-3 小时 | 阶段 1 |
| 阶段 3: 拆分其他标准库文件 | 2-3 小时 | 阶段 1, 2 |
| 阶段 4: 添加示例 | 1-2 小时 | 阶段 2, 3 |
| 阶段 5: 文档和测试 | 1 小时 | 阶段 4 |
| **总计 (MVP)** | **7-10 小时** | **阶段 1-2 + 5** |
| **总计 (完整)** | **8-12 小时** | **所有阶段** |

---

## 未来工作

### 短期 (此计划后)
- 实现 `extend` 机制，支持在特定 section 中添加字段
- 添加更多用 Auto 实现的标准库方法
- 性能优化和基准测试

### 中期
- 条件编译支持（`#if C`, `#if VM`）
- 自动迁移工具（将旧的二 section 结构转换为三 section）
- IDE 支持和语法高亮

### 长期
- 更多的目标平台（JavaScript、WASM 等）
- 标准库性能优化
- 社区贡献的标准库扩展

---

## 设计决策记录

### 决策 1: 为什么保持 `# AUTO` 名称？

**选项**:
- A. `# AUTO` → `# COMMON`, `# AUTO` → `# VM`
- B. `# AUTO` → `# AUTO`, 添加 `# VM`（选择）

**理由**:
- `# AUTO` 名称已经存在，修改会导致大量现有文件需要更新
- 语义改变（从"解释器"到"所有场景"）更符合"Auto"的含义
- 降低迁移成本

### 决策 2: 为什么不实现 `extend` 机制？

**理由**:
- `extend` 需要设计新的语法和语义
- 实现复杂度高，工期长
- 先用简单方案（在 `# AUTO` 中声明所有字段）验证设计
- 后续可以单独计划实现 `extend`

### 决策 3: 为什么不在 `# AUTO` 中实现所有功能？

**理由**:
- 某些功能（文件 I/O、系统调用）必须依赖底层平台
- `fn.vm` 和 `fn.c` 提供了必要的平台抽象
- Auto 语言无法直接实现这些功能
- 设计目标是用 Auto 实现"尽可能多"，不是"所有"

---

## 阶段 4 实施详情 (2025-01-17)

### 背景

Plan 037 成功实现了数组返回类型支持，解除了 Plan 036 阶段 4 的主要阻塞。

### 已完成的工作

#### 1. 添加 str.split() 方法签名

**文件**: [stdlib/auto/str.at:117-122](../../stdlib/auto/str.at)

```auto
ext str {
    /// Split string by delimiter into array of strings
    /// NOTE: Placeholder implementation - full version requires complex expressions
    fn split(delimiter str) []str {
        let result = []
        result
    }
}
```

**实现说明**:
- ✅ 方法签名可以正确声明
- ✅ Parser 接受 `[]str` 返回类型 (Plan 037 Phase 3)
- ✅ 类型系统正确处理
- ✅ C transpiler 生成正确的函数签名
- ⏸️ 当前为 placeholder 实现，返回空数组

#### 2. 测试验证

**测试添加**: [lib.rs:1757-1794](../../crates/auto-lang/src/lib.rs)

```rust
#[test]
fn test_array_return_eval() {
    // Test that array return types work in the evaluator
    let code = r#"
fn get_numbers() []int {
    [1, 2, 3, 4, 5]
}

let nums = get_numbers()
nums[0]
"#;

    let result = run(code).unwrap();
    assert_eq!(result, "1");
}
```

**测试结果**: ✅ 554 tests passing

### 当前限制

完整实现 `split()` 方法需要以下支持:

#### 1. 复杂表达式在循环条件中

```auto
// 当前不支持
for i < self.len() {
    // ...
}
```

**需要**: 修改 parser 允许二元表达式在 for 循环条件中

#### 2. 方法链式调用

```auto
// 当前不支持
let current = self.sub(i, i + delimiter.len())
```

**需要**: 实现方法调用作为表达式的支持

#### 3. 数组操作

```auto
// 当前不支持
result = result.append(current)
```

**需要**: 实现数组 append/push 操作

### 未来工作

为了完整实现 Plan 036 阶段 4，需要:

1. **实现完整的 split() 方法**
   - 循环遍历字符串
   - 查找分隔符
   - 构建结果数组

2. **添加 lines() 和 words() 方法**
   ```auto
   fn lines() []str {
       .split("\n")
   }

   fn words() []str {
       .split(" ")
   }
   ```

3. **实现 File 高级方法**
   - `read_all()` - 读取整个文件
   - `write_lines()` - 写入多行数据

4. **添加静态 join() 方法**
   ```auto
   static fn join(parts []str, delimiter str) str {
       // 实现...
   }
   ```

### 依赖关系

Plan 036 阶段 4 依赖于:
- ✅ Plan 037 Phase 3: 数组返回类型 (已完成)
- ⏸️ 更多表达式支持 (待实现)
- ⏸️ 数组操作方法 (待实现)

### 成果

虽然完整实现还需要更多工作，但当前成果已经:
- ✅ 证明了 AutoLang 可以声明返回数组的方法
- ✅ 建立了方法签名的模式
- ✅ 为未来实现奠定了基础

**状态**: Phase 4 部分完成，基础已就绪

---

## 参考材料

### 相关文档
- [Plan 035: ext Statement](./035-ext-statement.md)
- [CLAUDE.md: 标准库组织](../CLAUDE.md#standard-library)
- [docs/tutorials/ext-statement.md](../tutorials/ext-statement.md)

### 相关代码
- `crates/auto-lang/src/parser.rs`: Section 解析和过滤
- `crates/auto-lang/src/trans/c.rs`: C 转译器
- `crates/auto-lang/src/eval.rs`: 解释器
- `stdlib/auto/`: 标准库文件

---

**计划结束**
