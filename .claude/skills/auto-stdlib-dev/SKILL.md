---
name: autolang-stdlib-dev
description: AutoLang 标准库开发助手。帮助开发者使用新的 ext 机制进行标准库开发。使用场景包括：(1) 使用 .at + .vm.at/.c.at 多平台文件组织 (2) 使用 ext 扩展类型添加平台特定方法和私有字段 (3) 实现带 #[vm]/#[c] 标注的方法 (4) 编写 OOP 风格的类型方法 (5) 创建和更新测试用例 (6) 遵循 AutoLang-first 原则。包含最新的标准库架构原则和最佳实践。
---

# AutoLang 标准库开发助手

帮助使用 **新架构** 开发和维护 AutoLang 标准库（`stdlib/auto/`），遵循 AutoLang-first 原则和 ext 机制。

## 核心架构原则

### ✅ 新架构（当前方法）

```
AutoLang 源码 (.at) → a2c transpiler → C 代码
                     → VM evaluator → 执行
```

**关键特性**：
- **AutoLang-first**：用 AutoLang 编写，NOT 手写 C
- **ext 机制**：支持平台特定的方法和私有字段
- **OOP 风格**：方法在类型内部（类似 Java）
- **自动转译**：a2c 自动生成 C 代码

### ❌ 旧架构（已弃用）

- 手写 C 代码 → 手动 FFI → AutoLang
- 使用 `# C {}` 和 `# AUTO {}` section 分离
- 模块前缀函数：`File_open()`, `File_close()`
- 使用 `fn.c` 声明外部函数（仅在必要时）

## 快速参考

### 文件组织

```
stdlib/auto/
├── module.at        # 公共接口（用户可见）
├── module.vm.at     # VM 平台实现（ext + #[vm] 方法）
└── module.c.at      # C 平台实现（ext + #[c] 或实现）
```

### 文件加载顺序（关键！）

```rust
// parser.rs:2283
CompileDest::Interp => vec![".at", ".vm.at"],    // 接口 → 实现
CompileDest::TransC => vec![".at", ".c.at"],     // 接口 → 实现
```

**为什么？** 接口声明（`.at`）必须先加载，然后实现（`.vm.at`/`.c.at`）才能覆盖/完成它们。

### 标注语法

**Rust 风格标注**（✅ 正确）：
```auto
#[pub]      # 公共方法/字段
#[vm]       # VM 实现（Rust 中注册）
#[c]        # C 实现（转译为 C）
#[pub, vm]  # 组合标注
```

**旧语法**（❌ 已弃用）：
```auto
[vm]        # 旧语法，不再使用
[c]         # 旧语法，不再使用
fn.vm       # 点记法，向后兼容但不推荐
```

## 常见任务

### 1. 创建新类型和方法

**在 `module.at` 中定义接口**：

```auto
// 公共接口定义
type File {
    #[pub]
    path str

    #[pub]
    static fn open(path str) File

    #[pub]
    fn read_text() str

    #[pub]
    fn close()
}
```

**在 `module.vm.at` 中提供 VM 实现**：

```auto
// 使用 ext 扩展类型
ext File {
    // 添加平台特定的私有字段（仅 VM 可见）
    #[private]
    _reader *BufReader

    // VM 实现的方法
    #[vm]
    static fn open(path str) File

    #[vm]
    fn read_text() str

    #[vm]
    fn close()
}
```

**在 Rust 中注册**（`crates/auto-lang/src/vm/module.rs`）：

```rust
pub fn open(uni: Shared<Universe>, path: Value) -> Value {
    // 实现代码
}

pub fn close_method(uni: Shared<Universe>, this: &mut Value, args: Vec<Value>) -> Value {
    // 实现代码
}

// 注册到 VM
let mut file_type = VmTypeEntry {
    name: "File".into(),
    methods: HashMap::new(),
};

file_type.methods.insert("close".into(), close_method as VmMethod);
```

### 2. 添加平台特定私有字段

**ext 机制支持**：可以在同模块的 ext 中添加私有字段

```auto
// module.vm.at
ext File {
    // 添加 VM 平台专用的私有字段
    #[private]
    _id usize        // VM 引用 ID
    #[private]
    _buffer *Buffer  // 内部缓冲区
}

// module.c.at
ext File {
    // 添加 C 平台专用的私有字段
    #[private]
    _fp *FILE        // C 文件指针
    #[private]
    _mode int        // 打开模式
}
```

**规则**：
- 私有字段用 `_` 前缀命名约定
- 同一模块的 ext 可以添加字段
- 私有字段仅在对应平台可见

### 3. 添加静态方法和实例方法

**静态方法**（在类型上调用）：

```auto
// module.at
type File {
    #[pub]
    static fn open(path str) File
}

// 使用
let file = File.open("test.txt")
```

**实例方法**（在实例上调用）：

```auto
// module.at
type File {
    #[pub]
    fn read_text() str
}

// 使用
let content = file.read_text()
```

### 4. 实现 AutoLang-first 方法

**纯 Auto 实现**（在 `.at` 文件中直接编写）：

```auto
type StringBuilder {
    #[pub]
    data *char

    #[pub]
    len int

    #[pub]
    fn append(s str) StringBuilder {
        // Auto 代码实现
        let new_data = realloc(this.data, new_size)
        // ...
        this
    }
}
```

**好处**：
- ✅ AutoLang 编写，a2c 自动转译为 C
- ✅ 代码在所有平台可用
- ✅ 无需手动维护 C 代码

### 5. 使用 fn.c 声明外部 C 函数

**仅在需要时使用**（调用现有的 C 库函数）：

```auto
// module.c.at
use.c <stdio.h>

// 声明外部 C 函数（无实现）
#[c]
fn printf(fmt cstr, ...) int

#[c]
fn fopen(path cstr, mode cstr) *FILE

#[c]
fn fclose(fp *FILE) int
```

**规则**：
- ✅ 用于声明标准 C 库函数（stdio.h, stdlib.h 等）
- ✅ 不包含函数体（只有签名）
- ❌ 不要用于我们自己的代码（用 AutoLang 编写！）

### 6. 创建测试用例

**测试位置**：`crates/auto-lang/test/a2c/XXX_name/`

**文件结构**：
```
XXX_name/
├── name.at              # AutoLang 源码
├── name.expected.c      # 期望的 C 输出
└── name.expected.h      # 期望的 header 输出
```

**创建步骤**：

1. 创建测试目录：`mkdir test/a2c/106_my_feature`
2. 编写 `my_feature.at` 源文件
3. 运行测试：`cargo test -p auto-lang test_106_my_feature`
4. 检查生成的 `.wrong.c` 和 `.wrong.h`
5. 如果正确，重命名为 `.expected.c` 和 `.expected.h`
6. 在 `crates/auto-lang/src/trans/c.rs` 添加测试函数：
   ```rust
   #[test]
   fn test_106_my_feature() {
       test_a2c("106_my_feature").unwrap();
   }
   ```

**命名规范**：
- `000-099`: 核心语言特性
- `100-199`: 标准库测试
- `200-299`: 集成测试

**测试模板**：见 `assets/templates/test-case.at`

### 7. OOP 风格 API 设计

**正确方式** ✅（OOP 风格）：

```auto
// AutoLang 源码（我们写什么）
type File {
    #[pub]
    fn read_line() str
}

// 使用
let content = file.read_line()
```

**转译为 C**（a2c 自动生成）：

```c
// a2c 添加前缀
char* File_read_line(File* self);
```

**错误方式** ❌（旧方法）：

```auto
// 不要在 AutoLang 中使用前缀！
fn File_read_line(f File) str
```

### 8. 迁移旧代码到新架构

**步骤**：

1. **识别旧代码**：
   - 手写的 `.c` 和 `.h` 文件
   - 使用 `# C {}` section 的代码
   - 模块前缀函数（`May_empty()`, `File_open()`）

2. **转换为 AutoLang**：
   - 用 AutoLang 重写逻辑
   - 使用 `tag` 替代手动 tag 字段
   - 使用方法而非函数

3. **使用 ext 分离平台代码**：
   ```auto
   // module.at - 接口
   type File { ... }

   // module.vm.at - VM 实现
   ext File {
       #[vm]
       fn read() str
   }

   // module.c.at - C 实现（未来）
   ext File {
       #[c]
       fn read() str
   }
   ```

4. **更新测试**：
   - 确保测试覆盖新实现
   - 验证转译输出正确

## 开发工作流

1. **设计接口**（`module.at`）：
   - 定义公共类型和方法
   - 使用 `#[pub]` 标注公共成员
   - OOP 风格（方法在类型内）

2. **实现 VM 版本**（`module.vm.at`）：
   - 使用 `ext` 扩展类型
   - 添加 `#[vm]` 标注的方法
   - 可选：添加平台特定私有字段

3. **注册到 VM**（Rust 代码）：
   - 在 `crates/auto-lang/src/vm/module.rs` 中实现
   - 注册到 VmTypeEntry

4. **创建测试**：
   - 编写测试用例
   - 生成期望输出
   - 添加测试函数

5. **验证**：
   - 运行 `cargo test -p auto-lang -- trans`
   - 确保所有测试通过
   - 检查生成的 C 代码质量

6. **提交**：
   - 使用清晰的提交消息
   - 更新相关计划文档

## 关键文件位置

- **标准库**：`stdlib/auto/`
- **VM 实现**：`crates/auto-lang/src/vm/`
- **C 转译器**：`crates/auto-lang/src/trans/c.rs`
- **Parser**：`crates/auto-lang/src/parser.rs`
- **测试**：`crates/auto-lang/test/a2c/`

## 设计模式示例

### 模式 1：简单类型（无平台差异）

```auto
// math.at - 纯 Auto 实现
type Math {
    #[pub]
    static fn square(x int) int {
        x * x
    }
}
```

### 模式 2：平台特定类型（使用 ext）

```auto
// io.at - 接口定义
type File {
    #[pub]
    path str

    #[pub]
    static fn open(path str) File

    #[pub]
    fn read_text() str
}

// io.vm.at - VM 实现
ext File {
    #[private]
    _id usize

    #[vm]
    static fn open(path str) File

    #[vm]
    fn read_text() str
}

// io.c.at - C 实现（未来）
ext File {
    #[private]
    _fp *FILE

    #[c]
    static fn open(path str) File {
        let f = fopen(path, c"r")
        // ...
    }

    #[c]
    fn read_text() str {
        // C 实现
    }
}
```

### 模式 3：Tag 类型（枚举）

```auto
// may.at
tag May<T> {
    val T
    nil Nil
    err Err

    #[pub]
    fn is_some() bool {
        is self {
            val(_) => true,
            _ => false
        }
    }

    #[pub]
    fn unwrap() T {
        is self {
            val(v) => v,
            nil => panic("unwrap on nil"),
            err(e) => panic(f"error: $e")
        }
    }
}
```

## 常见问题

**Q: .at 和 .vm.at 的加载顺序是什么？**

A: **先加载 `.at`，后加载 `.vm.at`**。接口声明必须在实现之前加载，这样实现才能正确覆盖/完成接口。

**Q: 什么时候使用 ext？**

A: 当需要为现有类型添加：
- 平台特定的方法实现
- 平台特定的私有字段
- VM 或 C 平台的专有功能

**Q: `#[vm]` 和纯 Auto 实现有什么区别？**

A: `#[vm]` 在 Rust 中实现并注册到 VM 系统。纯 Auto 实现直接在 `.at` 文件中用 AutoLang 编写，a2c 会将其转译为 C。

**Q: 还应该使用 `# C {}` 和 `# VM {}` 块吗？**

A: 不推荐。新架构使用 `.vm.at` 和 `.c.at` 文件分离平台代码，而不是 section 块。

**Q: 什么时候使用 `fn.c`？**

A: **仅用于声明外部 C 库函数**（stdio.h, stdlib.h 等）。不要用于自己的代码！

**Q: 如何确保我的代码符合新架构？**

A: 检查清单：
- ✅ 代码用 AutoLang 编写（NOT 手写 C）
- ✅ 使用 `#[vm]`/`#[c]` 标注（NOT `[vm]`）
- ✅ OOP 风格（方法在类型内）
- ✅ 使用 ext 进行平台扩展
- ✅ 文件加载顺序正确（.at → .vm.at）

## 参考资源

- **架构总览**：[Plan 027: Standard Library Implementation](../../../docs/plans/027-stdlib-c-foundation.md)
- **I/O 实现**：[Plan 020: Standard Library I/O](../../../docs/plans/020-stdlib-io-expansion.md)
- **设计文档**：[stdlib-organization.md](../../../docs/design/stdlib-organization.md)
- **教程**：[stdlib-organization.md](../../../docs/tutorials/stdlib-organization.md)
- **项目指南**：[CLAUDE.md](../../../CLAUDE.md)

## 迁移指南

### 从旧架构迁移

**识别旧模式**：
- ❌ `# C {}` 或 `# AUTO {}` sections
- ❌ 手写的 `.c` 和 `.h` 文件
- ❌ 模块前缀函数（`May_empty()`）
- ❌ `fn.vm` 点记法

**转换为新模式**：
- ✅ 使用 `.at` + `.vm.at`/`.c.at` 文件分离
- ✅ 用 AutoLang 编写，a2c 转译
- ✅ OOP 风格（`May.empty()`）
- ✅ `#[vm]` 标注

**示例**：

```auto
// 旧方法（❌ 不要这样做）
# C
#include "may.h"
fn.c May_empty() May

# AUTO
fn May_empty() May {
    May { tag: 0 }
}

// 新方法（✅ 应该这样做）
// may.at
tag May<T> {
    nil Nil
    val T
    err Err

    static fn empty() May<T> {
        May.nil()
    }
}
```
