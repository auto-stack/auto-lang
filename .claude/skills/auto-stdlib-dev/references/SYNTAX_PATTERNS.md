# AutoLang 标准库语法模式

本文档描述标准库开发中使用的语法模式和约定。

## 文件级声明

### use.c - C 头文件包含

**位置**: `.c.at` 文件顶部

**语法**:
```auto
use.c <stdio.h>
use.c <stdlib.h>
use.c <unistd.h>
```

**用途**: 声明 C 转译时需要的头文件

**转译结果** (C):
```c
#include <stdio.h>
#include <stdlib.h>
#include <unistd.h>
```

### use - 模块导入

**位置**: `.at` 文件顶部

**语法**:
```auto
use auto.io: File, open, open_read
use c.stdio: printf, fopen, fclose, FILE
use c.stdlib: exit
```

**用途**:
- `use auto.X`: 从标准库模块导入
- `use c.X`: 从 C 库导入（需在 `.c.at` 中声明）

## 函数声明

### fn.c - C 函数声明

**位置**: `.c.at` 文件

**语法**:
```auto
fn.c printf(fmt cstr, arg cstr)
fn.c fopen(filename cstr, mode cstr) *FILE
fn.c getpid() int
```

**规则**:
- 只声明函数签名，不包含实现
- 参数类型使用 AutoLang 类型（int, str, cstr, *FILE 等）
- 返回类型写在括号后
- 无返回值的函数使用 `void`

**转译结果** (C):
```c
// 这些声明会直接包含到生成的 C 代码中
// 实际的函数由 C 标准库提供
```

### fn.vm - VM 函数声明

**位置**:
- `.vm.at` 文件（spec 和 enum 中的方法）
- `.at` 文件中 type 的方法（使用 `[vm]` 属性）

**语法**:
```auto
// 在 .vm.at 的 spec 中
spec Reader {
    fn.vm read_line() str
    fn.vm is_eof() bool
}

// 在 .at 的 type 中（带 [vm] 属性）
type File {
    [vm]
    fn read_all() str
}
```

**用途**: 声明函数在 Rust VM 中实现

**实现位置**: `crates/auto-lang/src/libs/`

**示例** (file.rs):
```rust
pub fn file_read_all(file: &Obj) -> Value {
    // Rust 实现
    // ...
}
```

**注册到 builtin** (在 `eval.rs` 或对应 libs 文件):
```rust
fn register_file_methods(mut ctx: &mut EvalContext) {
    ctx.register_type_method("File", "read_all", file_read_all);
    // ...
}
```

### fn - 普通 Auto 函数

**位置**: `.at` 文件

**语法**:
```auto
fn say(msg str) {
    print(msg)
}

fn get_pid() int {
    getpid()
}

fn add(a int, b int) int {
    a + b
}
```

**用途**: 用 Auto 语言实现函数

**特点**:
- 可以包含任意 Auto 代码
- 可以调用其他 Auto 函数
- 可以调用 fn.vm 和 fn.c 声明的函数

## 类型声明

### type.c - C 类型声明

**位置**: `.c.at` 文件

**语法**:
```auto
type.c FILE
type.c Dir
```

**用途**: 声明 C 类型（通常来自头文件）

### type - Auto 类型定义

**位置**: `.at` 文件

**语法**:
```auto
type File {
    path str
    file *FILE

    fn close() {
        fclose(.file)
    }

    fn read_text() str {
        let buf cstr = c"                                        "
        fgets(buf, 40, .file)
        buf
    }

    fn char_count() int {
        .path.size
    }

    [vm]
    fn read_all() str
}
```

**组成部分**:
1. **字段**: AutoLang 类型的字段
2. **方法实现**: 用 Auto 实现的方法
3. **纯 Auto 方法**: 完全用 Auto 代码实现
4. **VM 方法**: 用 `[vm]` 标记，在 Rust 中实现

## 变量声明

### let c - C 全局变量

**位置**: `.c.at` 文件

**语法**:
```auto
let c stdin *FILE
let c stdout *FILE
let c stderr *FILE
```

**用途**: 声明 C 全局变量（如 stdin, stdout, errno）

**转译结果** (C):
```c
extern FILE* stdin;
extern FILE* stdout;
extern FILE* stderr;
```

### let/mut/const - Auto 变量

**位置**: `.at` 文件（函数内部或顶层）

**语法**:
```auto
let x int = 10
mut counter = 0
const MAX_SIZE = 100
```

## 场景特定代码

### # C {} - C 转译器代码块

**位置**: `.at` 文件函数内部

**语法**:
```auto
fn say(msg str) {
# C {
    printf(c"%s\n", msg)
# }
# VM {
    print(msg)
# }
}
```

**规则**:
- 块内使用 C 语法
- 可访问 Auto 变量（如 msg）
- 需要使用 c 前缀的字面量（c"string"）
- 仅在 C 转译模式下使用

### # VM {} - VM 解释器代码块

**位置**: `.at` 文件函数内部

**语法**:
```auto
fn say(msg str) {
# C {
    printf(c"%s\n", msg)
# }
# VM {
    print(msg)
# }
}
```

**规则**:
- 块内使用 Auto 语法
- 可访问 Auto 变量
- 仅在 VM 解释模式下使用

## 扩展方法

### ext - 扩展现有类型

**位置**: `.at` 文件

**语法**:
```auto
ext str {
    fn char_count() int {
        .size
    }

    fn.vm split(delimiter str) []str

    fn to_upper() str {
        // Auto 实现
    }
}
```

**用途**: 为内置类型（str, int, [] 等）添加方法

**目标类型**:
- `str` - 字符串
- `int`, `float`, `bool` - 基本类型
- `[]T` - 数组类型
- 用户定义类型（虽然通常直接在 type 定义中添加方法）

## 接口和枚举

### spec - 多态接口

**位置**: `.vm.at` 文件

**语法**:
```auto
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
```

**用途**:
- 定义多态接口
- 描述类型应该实现的方法
- 用于类型约束和泛型（未来功能）

**实现**:
```auto
type File {
    // File 实现 Reader, Writer, Seekable
    fn read_line() str { ... }
    fn is_eof() bool { ... }
    fn write_line(s str) { ... }
    fn flush() { ... }
    fn seek(offset int, origin int) int { ... }
    fn tell() int { ... }
    fn rewind() { ... }
}
```

### enum - 枚举

**位置**: `.vm.at` 文件

**语法**:
```auto
enum SeekOrigin {
    Set = 0
    Cur = 1
    End = 2
}

enum Color {
    Red = 0
    Green = 1
    Blue = 2
}
```

**用途**:
- 定义枚举类型
- 提供命名常量

**使用**:
```auto
fn seek_to_start(f File) {
    f.seek(0, SeekOrigin.Set)
}
```

## 属性标记

### [vm] - VM 实现属性

**位置**: type 方法声明

**语法**:
```auto
type File {
    [vm]
    fn read_all() str

    [vm]
    fn write_lines(lines []str)
}

type str {
    [vm]
    fn split(delimiter str) []str
}
```

**用途**: 标记方法在 Rust VM 中实现

**实现要求**:
1. 在 `crates/auto-lang/src/libs/` 创建对应文件
2. 实现函数：`pub fn type_method(args: &[Value]) -> Value`
3. 在 builtin 系统注册：`ctx.register_type_method("TypeName", "method_name", function)`

## 语法对比表

| 语法 | 位置 | 用途 | 实现语言 |
|------|------|------|----------|
| `use.c <header.h>` | `.c.at` | 包含 C 头 | - |
| `fn.c name(...) type` | `.c.at` | C 函数声明 | C |
| `type.c Name` | `.c.at` | C 类型声明 | - |
| `let c name` | `.c.at` | C 全局变量 | - |
| `fn.vm name(...)` | `.vm.at`, `.at` | VM 函数声明 | Rust |
| `spec Name { ... }` | `.vm.at` | 接口定义 | - |
| `enum Name { ... }` | `.vm.at` | 枚举定义 | - |
| `[vm]` | `.at` type 内 | 标记 VM 方法 | Rust |
| `fn name(...) { ... }` | `.at` | Auto 函数 | Auto |
| `type Name { ... }` | `.at` | Auto 类型 | Auto |
| `ext Type { ... }` | `.at` | 扩展类型 | Auto |
| `# C { ... }` | `.at` 函数内 | C 代码块 | C |
| `# VM { ... }` | `.at` 函数内 | VM 代码块 | Auto |

## 最佳实践

### 1. 选择正确的实现方式

**纯 Auto 实现**（优先）:
```auto
fn char_count() int {
    .size
}
```

**VM 实现**（当 Auto 无法实现时）:
```auto
[vm]
fn read_all() str
```

**场景特定代码**（当不同场景需要不同实现时）:
```auto
fn say(msg str) {
# C { printf(c"%s\n", msg) # }
# VM { print(msg) # }
}
```

### 2. 保持声明集中

- 所有 C 声明在 `.c.at`
- 所有 spec/enum 在 `.vm.at`
- 所有实现在 `.at`

### 3. 避免重复

- 不要在 `.at` 和 `.c.at` 中重复声明类型
- 使用 `# C {}` 块处理场景差异，不要定义多个 type

### 4. 清晰的命名

- 使用描述性的名称
- 遵循命名约定（snake_case 函数，PascalCase 类型）

## 转译示例

### 输入 (io.at):
```auto
use c.stdio: printf, fopen, fclose, FILE

fn say(msg str) {
# C {
    printf(c"%s\n", msg)
# }
# VM {
    print(msg)
# }
}

type File {
    path str
    file *FILE

    fn close() {
        fclose(.file)
    }
}
```

### 输出 (io.h):
```c
#pragma once
#include "auto/io.c.h"
#include <stdio.h>

void say(const char* msg);

typedef struct File {
    const char* path;
    FILE* file;
} File;

void File_close(File* self);
```

### 输出 (io.c):
```c
#include "io.h"

void say(const char* msg) {
    #if defined(AUTO_C_TRANSPILER)
    printf("%s\n", msg);
    #else
    // VM implementation
    #endif
}

void File_close(File* self) {
    fclose(self->file);
}
```

## 常见错误

### 错误 1: fn.c 包含实现

**错误**:
```auto
// .c.at
fn.c say(msg str) {
    printf(c"%s\n", msg)  // 不应该有实现！
}
```

**正确**:
```auto
// .c.at
fn.c say(msg str)

// .at
fn say(msg str) {
    printf(c"%s\n", msg)
}
```

### 错误 2: [vm] 方法包含实现

**错误**:
```auto
type File {
    [vm]
    fn read_all() str {
        // 不应该有实现！
    }
}
```

**正确**:
```auto
type File {
    [vm]
    fn read_all() str
}

// 在 crates/auto-lang/src/libs/file.rs 中实现
```

### 错误 3: 场景特定代码未使用块

**错误**:
```auto
fn say(msg str) {
    printf(c"%s\n", msg)  // C 代码，但没有 # C {} 块
}
```

**正确**:
```auto
fn say(msg str) {
# C {
    printf(c"%s\n", msg)
# }
# VM {
    print(msg)
# }
}
```

## 调试技巧

### 检查生成的 C 代码

```bash
# 转译单个文件
auto.exe c stdlib/auto/io.at

# 检查生成的 .c 和 .h 文件
cat stdlib/auto/io.c
cat stdlib/auto/io.h
```

### 测试语法

```bash
# 运行转译测试
cargo test -p auto-lang -- trans

# 测试特定功能
cargo test -p auto-lang test_100_std_hello
```

### 查看 AST

```bash
# 使用 --ast 选项（如果支持）
auto.exe parse --ast myfile.at
```
