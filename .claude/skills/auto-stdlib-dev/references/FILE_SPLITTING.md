# 文件拆分规则 (Plan 036)

## 原则

将单一 `.at` 文件拆分为多个文件，通过文件后缀区分不同场景的代码。

## 文件类型

### `.at` - 纯 Auto 代码

**用途**: 所有场景共享的 Auto 代码

**内容**:
- `type` 定义（包含完整的字段和方法）
- `fn` 函数（用 Auto 实现的逻辑）
- 场景特定代码使用 `# C {}` 和 `# VM {}` 块
- Auto 实现的方法（纯 Auto 代码）

**加载**: 始终加载

### `.vm.at` - VM/解释器专用

**用途**: 只有解释器需要的声明和接口

**内容**:
- `spec` 声明（多态接口）
- `enum` 定义
- `fn.vm` 函数签名（不含实现，实现在 Rust 中）

**加载**: 仅当 `compile_dest == Interp`

### `.c.at` - C 转译器专用

**用途**: 只有 C 转译器需要的声明

**内容**:
- `use.c <header.h>` - C 头文件包含
- `fn.c name(...) ret_type` - C 函数声明
- `type.c TypeName` - C 类型声明
- `let c var_name` - C 全局变量声明

**加载**: 仅当 `compile_dest == TransC`

## 加载顺序

### 解释器模式
```
name.vm.at → name.at → 合并后解析
```

### C 转译器模式
```
name.c.at → name.at → 合并后解析
```

### Rust 转译器模式（未来）
```
name.rust.at → name.at → 合并后解析
```

**关键**: 文件内容在解析前自动合并，合并后的文件就像原来的单一文件。

## 拆分决策树

```
检查 X.at 是否包含 section 标记:
  |
  ├─ 包含 # C section?
  |  ├─ 是 → 拆分为 X.at + X.c.at
  |  └─ 否 → 继续
  |
  ├─ 包含 # AUTO section 且只有 fn.vm 声明?
  |  ├─ 是 → 拆分为 X.at + X.vm.at
  |  └─ 否 → 继续
  |
  ├─ 纯 Auto 代码（无 section）?
  |  └─ 是 → 无需拆分
  |
  └─ 同时包含 # C 和 # AUTO?
     └─ 是 → 拆分为 X.at + X.vm.at + X.c.at
```

## 拆分步骤

### 步骤 1: 评估文件

```bash
# 检查文件内容
grep -E "^# (AUTO|C|VM)" stdlib/auto/X.at
```

### 步骤 2: 创建 .vm.at（如需要）

从 `# AUTO` section 提取：
- spec 声明
- enum 定义
- fn.vm 函数签名

**示例** (io.vm.at):
```auto
// Spec declarations for polymorphic I/O
spec Reader {
    fn read_line() str
    fn is_eof() bool
}

spec Writer {
    fn write_line(s str)
    fn flush()
}

enum SeekOrigin {
    Set = 0
    Cur = 1
    End = 2
}
```

### 步骤 3: 创建 .c.at（如需要）

从 `# C` section 提取：
- use.c 声明
- fn.c 函数
- type.c 类型
- let c 变量

**示例** (sys.c.at):
```auto
use.c <unistd.h>

fn.c getpid() int
```

### 步骤 4: 精简 .at

移除 section 标记，保留纯 Auto 代码。

**场景 1**: 原本在 # AUTO 中只有 fn.vm 声明
- 移除 # AUTO section 标记
- 这些声明已移到 .vm.at，直接删除

**场景 2**: 原本在 # C 中有完整类型定义
- 移除 # C section 标记
- 保留 type 定义，但移除 C 实现细节
- 使用 `fn.c` 声明的函数已移到 .c.at

**场景 3**: 不同场景需要不同实现
- 使用 `# C {}` 和 `# VM {}` 块

**示例** (io.at - say 函数):
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

### 步骤 5: 测试验证

```bash
# 运行所有转译测试
cargo test -p auto-lang -- trans

# 检查测试数量
cargo test -p auto-lang -- trans 2>&1 | grep "test result"
```

## 实际案例

### 案例 1: io.at

**原始状态**: 单一文件，包含 # AUTO 和 # C sections

**拆分结果**:
- `io.at` (133 行) - 纯 Auto 代码
  - type File 定义
  - fn say() 使用 # C {} 和 # VM {} 块
  - File 的方法实现
- `io.vm.at` (27 行) - spec 和 enum
- `io.c.at` (40 行) - C 函数和类型声明

### 案例 2: sys.at

**原始状态**: 单一文件，包含 # C section

**拆分结果**:
- `sys.at` (6 行) - 纯 Auto wrapper
  ```auto
  fn get_pid() int {
      getpid()
  }
  ```
- `sys.c.at` (4 行) - C 声明
  ```auto
  use.c <unistd.h>

  fn.c getpid() int
  ```

### 案例 3: str.at

**原始状态**: 纯 Auto 代码，无 section 标记

**拆分结果**: 无需拆分，保持单一 `str.at` 文件

**原因**: 只包含 ext str { ... } 和纯 Auto 方法

### 案例 4: math.at

**原始状态**: 纯 Auto 代码，无 section 标记

**拆分结果**: 无需拆分，保持单一 `math.at` 文件

**原因**: 只包含简单的 fn 函数，用 Auto 实现

## 向后兼容

**Fallback 机制**: 如果拆分文件不存在，自动回退到原始 `.at` 文件。

**实现** (parser.rs):
```rust
fn get_file_extensions(&self) -> Vec<&'static str> {
    match self.compile_dest {
        CompileDest::Interp => vec![".vm.at", ".at"],
        CompileDest::TransC => vec![".c.at", ".at"],
        CompileDest::TransRust => vec![".rust.at", ".at"],
    }
}
```

**行为**:
1. 尝试加载 `.vm.at`（或 `.c.at`）
2. 如果不存在，fallback 到 `.at`
3. 如果都不存在，报错

## 常见错误

### 错误 1: 重复声明

**问题**: 在 `.at` 中重复声明类型
```auto
# C
type File { ... }

# AUTO
type File { ... }  // 重复！
```

**解决**: 拆分后只保留一份 type 声明在 `.at` 中

### 错误 2: fn.vm 实现放在错误位置

**问题**: 在 `.vm.at` 中添加实现
```auto
// io.vm.at
fn.vm read_all() str {
    // 实现不应该在这里！
}
```

**解决**: `.vm.at` 只放签名。实现在 `crates/auto-lang/src/libs/` 的 Rust 代码中

### 错误 3: 场景特定代码未使用块语法

**问题**: 在 `.at` 中混合场景代码
```auto
fn say(msg str) {
    printf(c"%s\n", msg)  // C 代码，但没有用 # C {} 包裹
}
```

**解决**: 使用 `# C {}` 和 `# VM {}` 块明确场景

## 核准清单

拆分完成后，确保：

- [ ] `.at` 文件不包含 section 标记（# AUTO, # C, # VM）
- [ ] `.vm.at` 只包含 spec、enum、fn.vm 签名
- [ ] `.c.at` 只包含 use.c、fn.c、type.c、let c
- [ ] 场景特定代码使用 `# C {}` 和 `# VM {}` 块
- [ ] 所有测试通过 (`cargo test -p auto-lang -- trans`)
- [ ] 无重复声明
- [ ] 文件命名正确（name.at, name.vm.at, name.c.at）
