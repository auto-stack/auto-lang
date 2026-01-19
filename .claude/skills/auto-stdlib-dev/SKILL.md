---
name: autolang-stdlib-dev
description: AutoLang 标准库开发助手。帮助开发者进行标准库文件的创建、拆分和维护。使用场景包括：(1) 拆分标准库文件为 .at、.vm.at、.c.at 多文件结构 (2) 添加新的 C 函数声明和类型定义 (3) 实现 Auto 方法（纯 Auto 或 fn.vm）(4) 使用场景特定代码块 (# C {} 和 # VM {}) (5) 创建和更新测试用例 (6) 生成期望输出文件。包含 Plan 036 完整实施经验和标准库开发最佳实践。
---

# AutoLang 标准库开发助手

帮助开发和维护 AutoLang 标准库（stdlib/auto/），遵循 Plan 036 的文件组织原则。

## 快速参考

### 文件类型

- **`.at`** - 纯 Auto 代码（所有场景加载）
- **`.vm.at`** - VM/解释器专用代码（spec、enum、Auto 实现的方法）
- **`.c.at`** - C 转译器专用代码（fn.c、type.c、let c 声明）

### 加载顺序

```
解释器:  name.vm.at → name.at
转译器:  name.c.at → name.at
```

文件内容在解析前自动合并，section 标记（# AUTO, # C）自动过滤。

## 常见任务

### 1. 拆分标准库文件

**评估文件是否需要拆分**：

检查 `stdlib/auto/X.at` 是否包含：
- `# C` section → 需要，拆分为 `X.at` + `X.c.at`
- `# AUTO` section 中只有 fn.vm 声明 → 需要，创建 `X.vm.at`
- 纯 Auto 代码（无 section 标记）→ 无需拆分

**拆分步骤**：

1. 创建 `X.vm.at`：从 `# AUTO` section 提取
   - `spec` 声明
   - `enum` 定义
   - fn.vm 函数签名（不含实现）

2. 创建 `X.c.at`：从 `# C` section 提取
   - `use.c <header.h>` 包含
   - `fn.c name(...) ret_type` 函数声明
   - `type.c TypeName` 类型声明
   - `let c var_name` 变量声明

3. 精简 `X.at`：保留纯 Auto 代码
   - `type` 定义（包含 fn 方法实现）
   - `fn` 函数（用 Auto 实现）
   - 场景特定代码使用 `# C {}` 和 `# VM {}` 块

**示例**：见 [FILE_SPLITTING.md](references/FILE_SPLITTING.md)

### 2. 添加 C 函数声明

**在 `X.c.at` 中添加**：

```auto
use.c <stdio.h>
fn.c printf(fmt cstr, ...) int
type.c FILE
let c stdin *FILE
```

**规则**：
- 所有 C 声明集中到 `.c.at`
- 使用 `fn.c` 声明函数（不包含实现）
- 使用 `type.c` 声明类型
- 使用 `let c` 声明全局变量

### 3. 添加方法到类型

**纯 Auto 实现**（在 `X.at` 中）：

```auto
type File {
    fn char_count() int {
        .size
    }
}
```

**VM 实现**（在 `X.at` 中，使用 `[vm]` 属性）：

```auto
type File {
    [vm]
    fn read_all() str
}
```

然后在 `crates/auto-lang/src/libs/file.rs` 中实现 Rust 代码，注册到 builtin。

**详细语法**：见 [SYNTAX_PATTERNS.md](references/SYNTAX_PATTERNS.md)

### 4. 场景特定代码

**使用 `# C {}` 和 `# VM {}` 块**：

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

**规则**：
- 同一个函数，不同场景有不同实现
- 块内代码使用对应场景的语言（C 或 Auto）
- 避免在 `X.at` 中重复声明类型（声明一次即可）

### 5. 创建测试用例

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
- 000-099: 核心语言特性
- 100-199: 标准库测试

**详细指南**：见 [TESTING.md](references/TESTING.md)

### 6. 提交更改

**提交消息格式**：

```
Implement Plan 036 Phase X: brief description

Main changes:
- Change 1
- Change 2

Test results: N tests passing
```

**示例**：
```
Implement Plan 036 Phase 3: Split sys.at into sys.at + sys.c.at

- Create sys.c.at with getpid() declaration
- Simplify sys.at to pure Auto wrapper

Test results: 556 tests passing
```

## 开发工作流

1. **阅读相关文档**：
   - Plan 036: [FILE_SPLITTING.md](references/FILE_SPLITTING.md)
   - 语法模式: [SYNTAX_PATTERNS.md](references/SYNTAX_PATTERNS.md)
   - 测试指南: [TESTING.md](references/TESTING.md)

2. **实现功能**：
   - 拆分文件（如需要）
   - 添加类型/函数
   - 实现 Auto 方法或 fn.vm 声明

3. **创建测试**：
   - 编写测试用例
   - 生成期望输出
   - 添加测试函数

4. **验证**：
   - 运行 `cargo test -p auto-lang -- trans`
   - 确保所有测试通过
   - 检查生成的代码质量

5. **提交**：
   - 编写清晰的提交消息
   - 更新 Plan 036 文档状态

## 关键文件位置

- **标准库**: `stdlib/auto/`
- **测试**: `crates/auto-lang/test/a2c/`
- **C 转译器**: `crates/auto-lang/src/trans/c.rs`
- **Parser**: `crates/auto-lang/src/parser.rs`
- **VM libs**: `crates/auto-lang/src/libs/`

## 常见问题

**Q: 什么时候需要拆分文件？**

A: 当 `.at` 文件包含 `# C` 或 `# AUTO` section 时。纯 Auto 代码无需拆分。

**Q: fn.vm 和纯 Auto 方法有什么区别？**

A: `fn.vm` 在 Rust (VM) 中实现，注册到 builtin 系统。纯 Auto 方法直接在 `.at` 文件中用 Auto 编写。

**Q: 如何在测试中调试转译输出？**

A: 运行测试后检查 `.wrong.c` 和 `.wrong.h` 文件，与期望输出对比。

**Q: 测试失败怎么办？**

A: 先检查 `.wrong.c` 是否正确实现。如果是，重命名为 `.expected.c`。如果不是，修复转译器或源代码。

## 参考资源

- **Plan 036 完整文档**: `docs/plans/036-unified-auto-section.md`
- **CLAUDE.md**: 项目根目录的开发指南
- **示例拆分**:
  - `io.at` → `io.at` + `io.vm.at` + `io.c.at`
  - `sys.at` → `sys.at` + `sys.c.at`
  - `str.at` - 纯 Auto，无需拆分
