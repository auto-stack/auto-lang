# AutoLang 标准库测试指南

本文档描述如何为 AutoLang 标准库创建和维护测试用例。

## 测试框架

AutoLang 使用 a2c (Auto-to-C) 转译器测试框架来验证标准库功能。

### 测试位置

```
crates/auto-lang/test/a2c/
├── 000-099/          # 核心语言特性测试
├── 100-199/          # 标准库测试
│   ├── 100_std_hello/
│   ├── 101_std_getpid/
│   ├── 105_std_str/
│   └── ...
└── XXX_name/         # 测试目录
    ├── name.at              # AutoLang 源文件
    ├── name.expected.c      # 期望的 C 输出
    └── name.expected.h      # 期望的 header 输出
```

### 测试命名规范

**格式**: `NNN_description`

**NNN 范围**:
- `000-099`: 核心语言特性（hello, array, func, struct 等）
- `100-199`: 标准库测试（std_hello, std_getpid, std_str 等）
- `200-299`: 高级特性
- `300+`: 其他功能

**description**: 简短描述（snake_case）

**示例**:
- `100_std_hello` - 标准库 hello 测试
- `105_std_str` - 字符串扩展测试
- `037_unified_section` - Plan 036 文件拆分测试

## 创建测试用例

### 步骤 1: 创建测试目录

```bash
cd crates/auto-lang/test/a2c
mkdir 106_my_feature
cd 106_my_feature
```

### 步骤 2: 编写 AutoLang 源文件

**创建** `my_feature.at`:

```auto
use auto.io: say

fn main() {
    say("Hello from stdlib!")
}
```

**测试用例要求**:
- 必须包含 `fn main()` 函数
- main 函数返回类型通常为 `void` 或无返回值
- 测试功能应该简单清晰

### 步骤 3: 运行测试生成期望输出

```bash
# 从项目根目录
cargo test -p auto-lang test_106_my_feature
```

**第一次运行会失败**，生成 `.wrong.c` 和 `.wrong.h` 文件:
```
106_my_feature/
├── my_feature.at
├── my_feature.wrong.c      # 生成的 C 代码
└── my_feature.wrong.h      # 生成的 header
```

### 步骤 4: 审查生成的代码

**检查** `my_feature.wrong.c`:
```c
#include "my_feature.h"

int main(void) {
    say("Hello from stdlib!");
    return 0;
}
```

**检查** `my_feature.wrong.h`:
```c
#pragma once
#include "auto/io.h"
```

**验证**:
- C 代码是否正确？
- 函数签名是否正确？
- 包含的头文件是否正确？

### 步骤 5: 确认期望输出

**如果生成的代码正确**:
```bash
mv my_feature.wrong.c my_feature.expected.c
mv my_feature.wrong.h my_feature.expected.h
```

**如果生成的代码不正确**:
1. 修复 AutoLang 源代码（`.at` 文件）
2. 或修复转译器（`crates/auto-lang/src/trans/c.rs`）
3. 重新运行测试

### 步骤 6: 添加测试函数

**编辑** `crates/auto-lang/src/trans/c.rs`:

在测试模块末尾添加：

```rust
#[test]
fn test_106_my_feature() {
    test_a2c("106_my_feature").unwrap();
}
```

**位置**: 文件末尾，其他 `test_XXX_` 函数附近

### 步骤 7: 运行测试验证

```bash
# 运行单个测试
cargo test -p auto-lang test_106_my_feature

# 运行所有 a2c 测试
cargo test -p auto-lang -- trans

# 查看测试输出
cargo test -p auto-lang -- trans -- --nocapture
```

**期望输出**:
```
running 1 test
test test_106_my_feature ... ok
test result: ok. 1 passed; 0 failed; 0 ignored
```

## 测试最佳实践

### 1. 简单清晰

**好的测试**:
```auto
fn main() {
    let s = "hello"
    print(s.len())
}
```

**不好的测试**（太复杂）:
```auto
fn main() {
    let files = ["a.at", "b.at", "c.at"]
    let results = []
    for f in files {
        let content = read_file(f)
        results.push(parse(content))
    }
    print(results.len())
}
```

### 2. 测试单一功能

每个测试应该验证一个特定的功能或特性。

**示例**:
- `100_std_hello` - 测试 say() 函数
- `101_std_getpid` - 测试 get_pid() 函数
- `105_std_str` - 测试字符串扩展方法

### 3. 使用描述性名称

```auto
// 测试字符串分割
// test_str_split.at
fn main() {
    let parts = "hello world".split(" ")
    print(parts[0])
}

// 测试文件读取
// test_file_read.at
fn main() {
    let f = open("test.txt")
    let content = f.read_text()
    print(content)
}
```

### 4. 包含注释

对于复杂测试，添加注释说明测试目的：

```auto
// Test: File read_text() method
// Expected: Print first line of file
fn main() {
    let f = open("data.txt")
    let line = f.read_text()
    print(line)
}
```

## 测试场景

### 场景 1: 测试新函数

**目的**: 验证新添加的函数工作正常

**示例** (test_std_getpid.at):
```auto
use auto.sys: get_pid

fn main() {
    let pid = get_pid()
    print(pid)
}
```

### 场景 2: 测试类型方法

**目的**: 验证类型的方法正确转译

**示例** (test_std_str.at):
```auto
use auto.str: str

fn main() {
    let s str = "Hello"
    print(s)
}
```

### 场景 3: 测试场景特定代码

**目的**: 验证 `# C {}` 和 `# VM {}` 块正确处理

**示例** (test_say.at):
```auto
use auto.io: say

fn main() {
    say("test")
}
```

### 场景 4: 测试 Plan 036 文件拆分

**目的**: 验证文件合并和解析正确

**示例** (037_unified_section/test.at):
```auto
use auto.io: File, open

fn main() {
    let f = open("test.txt")
    f.close()
}
```

**验证**:
- `.vm.at` 和 `.at` 正确合并
- 无重复声明
- 正确的 C 代码生成

### 场景 5: 测试数组类型

**目的**: 验证数组返回类型（Plan 037）

**示例** (038_str_split/test.at):
```auto
use auto.str: str

fn main() {
    let parts str.split(" ")
    print(parts[0])
}
```

## 运行测试

### 运行所有转译测试

```bash
cargo test -p auto-lang -- trans
```

**输出示例**:
```
running 94 tests
test test_000_hello ... ok
test test_001_array ... ok
...
test test_105_std_str ... ok

test result: ok. 94 passed; 0 failed; 3 ignored
```

### 运行单个测试

```bash
cargo test -p auto-lang test_100_std_hello
```

### 运行特定范围的测试

```bash
# 运行所有标准库测试 (100-199)
cargo test -p auto-lang -- trans | grep "test_1"

# 运行所有核心功能测试 (000-099)
cargo test -p auto-lang -- trans | grep "test_0"
```

### 显示测试输出

```bash
# 显示所有输出
cargo test -p auto-lang -- trans -- --nocapture

# 显示测试名称
cargo test -p auto-lang -- trans -- --test-threads=1
```

## 调试测试失败

### 查看生成的错误代码

```bash
# 运行失败的测试
cargo test -p auto-lang test_XXX_name

# 查看 .wrong.c 和 .wrong.h
cat test/a2c/XXX_name/name.wrong.c
cat test/a2c/XXX_name/name.wrong.h
```

### 对比期望输出

```bash
# Windows
fc /b name.wrong.c name.expected.c

# Unix
diff name.wrong.c name.expected.c
```

### 常见失败原因

#### 原因 1: 转译器 bug

**症状**: 生成的 C 代码语法错误或逻辑错误

**解决**:
1. 检查 `crates/auto-lang/src/trans/c.rs`
2. 修复转译逻辑
3. 重新测试

#### 原因 2: 期望输出过时

**症状**: 生成的代码看起来正确，但与期望不匹配

**解决**:
1. 审查生成的代码是否确实正确
2. 如果正确，更新期望输出
3. 重新测试

#### 原因 3: 源代码问题

**症状**: 转译失败或生成错误代码

**解决**:
1. 检查 `.at` 源文件语法
2. 检查 `use` 语句是否正确
3. 检查类型声明是否完整

## 测试文件模板

### 基本模板

```auto
// Test: [测试目的]
// Description: [详细描述]

use auto.module: function1, function2

fn main() {
    // 测试代码
    let result = function1()
    print(result)
}
```

### 文件 I/O 测试模板

```auto
// Test: File operations
use auto.io: File, open, open_write

fn main() {
    let f = open("test.txt")
    let content = f.read_text()
    print(content)
    f.close()
}
```

### 字符串方法测试模板

```auto
// Test: String extension methods
use auto.str: str

fn main() {
    let s = "hello world"
    let parts = s.split(" ")
    print(parts[0])
}
```

### 类型方法测试模板

```auto
// Test: Type methods
use auto.io: File, open

fn main() {
    let f = open("test.txt")
    f.close()
}
```

## 测试维护

### 更新期望输出

**当转译器行为改变时**:

1. 运行测试生成新的 `.wrong` 文件
2. 审查新的输出是否正确
3. 如果正确，替换 `.expected` 文件:
   ```bash
   mv name.wrong.c name.expected.c
   mv name.wrong.h name.expected.h
   ```

### 删除过时测试

```bash
# 删除测试目录
rm -r test/a2c/XXX_name

# 从 c.rs 移除测试函数
# 删除对应的 test_XXX_name() 函数
```

### 重命名测试

```bash
# 重命名目录
mv test/a2c/100_old_name test/a2c/100_new_name

# 重命名文件
cd test/a2c/100_new_name
mv old_name.at new_name.at
mv old_name.expected.c new_name.expected.c
mv old_name.expected.h new_name.expected.h

# 更新测试函数名
# 在 c.rs 中改为:
#[test]
fn test_100_new_name() {
    test_a2c("100_new_name").unwrap();
}
```

## 测试覆盖

### 核心功能覆盖

确保以下功能都有测试：

- [ ] 基本语法（变量、函数、控制流）
- [ ] 数组和切片
- [ ] 类型定义和方法
- [ ] 扩展方法 (ext)
- [ ] 模块导入 (use)
- [ ] 场景特定代码 (# C {}, # VM {})

### 标准库覆盖

确保每个标准库模块都有测试：

- [ ] io - say, File, open, etc.
- [ ] sys - get_pid, etc.
- [ ] str - split, lines, words, etc.
- [ ] math - square, etc.

## 性能考虑

### 测试速度

**当前**: 所有测试并行运行（默认）

**单个线程**（用于调试）:
```bash
cargo test -p auto-lang -- trans -- --test-threads=1
```

### 测试数量

**当前**: ~94 个 a2c 测试

**增长**: 每个新功能添加 1-2 个测试

**目标**: 保持测试数量在合理范围内（< 200）

## 集成到 CI

测试应该在 CI 中自动运行：

```yaml
# .github/workflows/test.yml
steps:
  - name: Run transpiler tests
    run: cargo test -p auto-lang -- trans
```

## 常用命令参考

```bash
# 运行所有转译测试
cargo test -p auto-lang -- trans

# 运行单个测试
cargo test -p auto-lang test_100_std_hello

# 显示详细输出
cargo test -p auto-lang -- trans -- --nocapture

# 运行测试并显示结果
cargo test -p auto-lang -- trans -- --test-threads=1

# 生成期望输出
cargo test -p auto-lang test_XXX_name
mv XXX_name.wrong.c XXX_name.expected.c
mv XXX_name.wrong.h XXX_name.expected.h

# 查看测试列表
cargo test -p auto-lang -- trans -- --list
```

## 参考资源

- **Plan 036**: `docs/plans/036-unified-auto-section.md`
- **C 转译器**: `crates/auto-lang/src/trans/c.rs`
- **测试示例**: `crates/auto-lang/test/a2c/`
- **CLAUDE.md**: 项目根目录的开发指南
