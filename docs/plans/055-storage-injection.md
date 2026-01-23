# AutoLang Storage 环境注入实现计划

## 目标

实现基于 Storage 的环境注入机制，让 `List<T>` 能够根据目标平台（MCU vs PC）自动选择合适的存储策略（Fixed 静态分配 vs Dynamic 动态分配）。

## 核心愿景

**用户体验**：用户只需写 `List<int>`，编译器自动根据目标平台选择：
- **MCU 环境** → `List<int, Fixed<64>>`（静态分配，无堆）
- **PC 环境** → `List<int, Dynamic>`（动态分配，有堆）

## 背景与现状

### 当前状态
- **List<T> 已实现**：使用 `Vec<Value>`（堆分配动态存储）
- **Prelude 系统**：`stdlib/auto/prelude.at` 自动加载，但只导入 `say`
- **编译目标**：只有 `CompileDest` (Interp/TransC/TransRust)，无 MCU vs PC 区分
- **环境注入**：`env_vals: HashMap` 存在但未使用

### 需求（来自 Plan 054）
1. **Storage 类型**：Fixed（静态）、Dynamic（堆）
2. **扩展 List**：从 `List<T>` 改为 `List<T, S>`，S 为 Storage 参数
3. **环境注入**：编译器启动时注入默认 Storage
4. **Prelude 集成**：导出 `type List<T> = List<T, DefaultStorage>`
5. **目标检测**：自动识别 MCU vs PC

---

## 实现架构

### 核心概念

```
用户代码              Compiler              Prelude
────────────────────────────────────────────────────────
let x List<int>  →  检测目标  →  注入环境  →  DefaultStorage
                          ↓              ↓
                      MCU: Fixed<64>   type List<T> =
                      PC:  Dynamic     List<T, DefaultStorage>
```

### Storage 类型层次

```auto
// stdlib/auto/storage.at
type Storage {              // Marker trait
}

type Fixed<N> : Storage {    // 静态分配（MCU）
    const CAPACITY: N = N
}

type Dynamic : Storage {     // 动态分配（PC）
}

type DefaultStorage : Storage  // 目标依赖的别名
```

---

## 实施阶段

### 阶段 1：类型系统扩展（1-2 天）

#### 1.1 添加 Storage 类型到 AST

**文件**：`crates/auto-lang/src/ast/types.rs`

```rust
// 在 Type 枚举中添加（约 line 37）
pub enum Type {
    // ... 现有变体 ...
    Storage(StorageType),   // 新增：Storage 策略类型
}

// 新增结构（约 line 270 之后）
#[derive(Debug, Clone)]
pub struct StorageType {
    pub kind: StorageKind,
}

#[derive(Debug, Clone, PartialEq)]
pub enum StorageKind {
    Fixed { capacity: usize },   // Fixed<N>
    Dynamic,                     // Dynamic
}
```

#### 1.2 更新类型系统方法

**文件**：`crates/auto-lang/src/ast/types.rs`

- `unique_name()`: 处理 `Storage` 类型
- `default_value()`: 返回 `"Storage"`
- `Display`: 格式化为 `Fixed<N>` 或 `Dynamic`

#### 1.3 解析器支持

**文件**：`crates/auto-lang/src/parser.rs`

添加 `parse_storage_type()` 方法（约 line 2000）：

```rust
fn parse_storage_type(&mut self) -> AutoResult<Type> {
    match self.cur.text.as_str() {
        "Fixed" => {
            self.expect(TokenKind::Lt)?;
            let capacity = self.parse_expr()?;
            self.expect(TokenKind::Gt)?;
            // 解析容量值...
        }
        "Dynamic" => Ok(Type::Storage(StorageType {
            kind: StorageKind::Dynamic,
        })),
        _ => Err(...),
    }
}
```

**验证**：`cargo test -p auto-lang test_storage_parsing`

---

### 阶段 2：目标检测系统（1 天）

#### 2.1 创建 Target 模块

**文件**：`crates/auto-lang/src/target.rs`（新建）

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Target {
    Mcu,  // 微控制器（无 OS）
    Pc,   // PC（有 OS）
}

impl Target {
    pub fn detect() -> Self {
        // 1. 检查 AUTO_TARGET 环境变量
        // 2. 检查 CARGO_BUILD_TARGET（交叉编译）
        // 3. 默认返回 PC
    }

    pub fn has_heap(&self) -> bool { matches!(self, Target::Pc) }
    pub fn default_storage_capacity(&self) -> Option<usize> {
        match self {
            Target::Mcu => Some(64),
            Target::Pc => None,
        }
    }
}
```

#### 2.2 集成到 CLI

**文件**：`crates/auto/src/main.rs`

```rust
#[arg(short, long)]
target: Option<TargetArg>,  // 添加到 C 命令

#[derive(Clone, ValueEnum)]
enum TargetArg {
    Mcu,
    Pc,
    Auto,  // 默认：自动检测
}
```

**验证**：
```bash
cargo run -- c test.at --target mcu
cargo run -- c test.at --target pc
```

---

### 阶段 3：环境注入系统（1-2 天）

#### 3.1 Universe 环境注入

**文件**：`crates/auto-lang/src/universe.rs`

```rust
impl Universe {
    pub fn inject_environment(&mut self, target: Target) {
        self.set_env_val("TARGET", match target {
            Target::Mcu → "mcu",
            Target::Pc → "pc",
        });

        self.set_env_val("DEFAULT_STORAGE", match target {
            Target::Mcu → "Fixed<64>",
            Target::Pc → "Dynamic",
        });

        self.set_env_val("HAS_HEAP", if target.has_heap() { "1" } else { "0" });
    }

    fn set_env_val(&mut self, name: &str, value: &str) { ... }
    pub fn get_env_val(&self, name: &str) -> Option<AutoStr> { ... }
}
```

#### 3.2 解释器初始化

**文件**：`crates/auto-lang/src/interp.rs`

在 `Interpreter::new()` 中（约 line 24）：

```rust
pub fn new() -> Self {
    let scope = shared(Universe::new());

    // 在加载 Prelude 之前注入环境
    {
        let mut uni = scope.borrow_mut();
        uni.inject_environment(Target::detect());
    }

    // ... 继续初始化 ...
}
```

**验证**：
```rust
#[test]
fn test_mcu_environment_injection() {
    let mut uni = Universe::new();
    uni.inject_environment(Target::Mcu);
    assert_eq!(uni.get_env_val("DEFAULT_STORAGE"), Some("Fixed<64>"));
}
```

---

### 阶段 4：Storage 标准库（1 天）

#### 4.1 创建 Storage 模块

**文件**：`stdlib/auto/storage.at`（新建）

```auto
/// Storage strategies for collections

// Marker trait
type Storage {
}

/// Fixed-capacity storage（stack/static）
type Fixed<N> : Storage {
    const CAPACITY: N = N
}

/// Dynamic-capacity storage（heap）
type Dynamic : Storage {
}

/// Target-dependent default storage
type DefaultStorage : Storage
```

#### 4.2 VM 函数注册

**文件**：`crates/auto-lang/src/interp.rs`

在加载 Prelude 之前（约 line 35）：

```rust
// Load storage.at to register Storage types
let storage_code = std::fs::read_to_string("../../stdlib/auto/storage.at")
    .unwrap_or(String::new());
if !storage_code.is_empty() {
    let _ = interpreter.interpret(&storage_code);
}
```

**验证**：`cargo test -p auto-lang test_storage_module`

---

### 阶段 5：List 扩展（1-2 天）

#### 5.1 更新 List 类型定义

**文件**：`stdlib/auto/list.at`

```auto
type List<T, S : Storage = DefaultStorage> {
    // T 是元素类型
    // S 是存储策略（默认为目标依赖的 DefaultStorage）

    // 新增方法
    #[c, vm, pub]
    fn capacity() int  // 返回 Fixed 的容量或 Dynamic 的 usize::MAX
}
```

#### 5.2 VM 实现

**文件**：`crates/auto-lang/src/vm/list.rs`

修改 `list_new()` 检查存储容量限制：

```rust
pub fn list_new(uni: Shared<Universe>, initial: Value) -> Value {
    let storage = uni.borrow().get_env_val("DEFAULT_STORAGE")
        .unwrap_or_else(|| "Dynamic".into());

    if storage.starts_with("Fixed") {
        // 强制执行容量限制
        let capacity: usize = parse_fixed_capacity(&storage).unwrap_or(64);
        if elems.len() > capacity {
            return Value::Error("capacity exceeded".into());
        }
    }

    // ... 创建 List ...
}
```

添加 `list_capacity()` 函数：

```rust
pub fn list_capacity(uni: Shared<Universe>, instance: &mut Value, _args: Vec<Value>) -> Value {
    let storage = get_instance_storage(instance);
    if storage.starts_with("Fixed") {
        Value::Int(parse_fixed_capacity(&storage).unwrap_or(64) as i32)
    } else {
        Value::Int(i32::MAX)  // Dynamic = "unlimited"
    }
}
```

**验证**：`cargo test -p auto-lang test_list_fixed_capacity`

---

### 阶段 6：Prelude 集成（0.5 天）

#### 6.1 更新 Prelude

**文件**：`stdlib/auto/prelude.at`

```auto
// ============================================================================
// Storage Strategies
// ============================================================================
use auto.storage: Storage, Fixed, Dynamic, DefaultStorage

// ============================================================================
// Collections（List with Target-Dependent Storage）
// ============================================================================
use auto.list: List

// 用户写 List<T> → 自动展开为 List<T, DefaultStorage>
// MCU: List<T, Fixed<64>>
// PC: List<T, Dynamic>
```

移除旧的注释（lines 35-40）关于 List 禁用的说明。

**验证**：`cargo test -p auto-lang test_prelude_list`

---

### 阶段 7：C Transpiler 增强（1-2 天）

#### 7.1 Storage 类型生成

**文件**：`crates/auto-lang/src/trans/c.rs`

在 `c_type_name()` 中（约 line 1525）：

```rust
Type::List(elem) => {
    let storage = self.scope.borrow()
        .get_env_val("DEFAULT_STORAGE")
        .unwrap_or_else(|| "Dynamic".into());

    let elem_type = self.c_type_name(elem);

    if storage.starts_with("Fixed") {
        let capacity = parse_fixed_capacity(&storage).unwrap_or(64);
        format!("list_fixed_{}_{}", elem_type, capacity)
    } else {
        format!("list_{}*", elem_type)
    }
}
```

#### 7.2 生成存储结构

在头文件生成中添加：

```c
// Fixed storage（stack allocated）
typedef struct {
    void* data[64];
    size_t len;
} list_fixed_int_64;

// Dynamic storage（heap allocated）
typedef struct {
    void** data;
    size_t len;
    size_t cap;
} list_int;
```

#### 7.3 目标特定的 push 实现

```c
// Fixed: check capacity
if (list->len < LIST_FIXED_CAPACITY) {
    list->data[list->len++] = value;
}

// Dynamic: grow if needed
if (list->len >= list->cap) {
    list->cap = list->cap == 0 ? 8 : list->cap * 2;
    list->data = realloc(list->data, list->cap * sizeof(void*));
}
list->data[list->len++] = value;
```

**验证**：`cargo test -p auto-lang test_a2c_054`

---

### 阶段 8：测试基础设施（1 天）

#### 8.1 MCU 测试用例

**文件**：`crates/auto-lang/test/a2c/054_list_mcu/list_mcu.at`

```auto
use auto.list: List

fn main() {
    let list = List.new()
    list.push(1)
    list.push(2)
    let cap = list.capacity()  // 应返回 64
}
```

**预期输出**（`list_mcu.expected.c`）：
```c
#define LIST_FIXED_CAPACITY 64
typedef struct {
    void* data[64];
    size_t len;
} list_fixed_int;

int cap = 64;
```

#### 8.2 PC 测试用例

**文件**：`crates/auto-lang/test/a2c/054_list_pc/list_pc.at`

```auto
use auto.list: List

fn main() {
    let list = List.new()
    for i in 0..1000 {
        list.push(i)  // PC 下可以增长
    }
    let cap = list.capacity()  // 应返回 INT_MAX
}
```

#### 8.3 VM 单元测试

**文件**：`crates/auto-lang/src/tests/storage_tests.rs`

```rust
#[test]
fn test_mcu_fixed_storage() {
    let mut uni = Universe::new();
    uni.inject_environment(Target::Mcu);
    assert_eq!(uni.get_env_val("DEFAULT_STORAGE"), Some("Fixed<64>"));
}

#[test]
fn test_pc_dynamic_storage() {
    let mut uni = Universe::new();
    uni.inject_environment(Target::Pc);
    assert_eq!(uni.get_env_val("DEFAULT_STORAGE"), Some("Dynamic"));
}

#[test]
fn test_list_capacity_enforcement() {
    // MCU 下超过容量应报错
    let code = r#"
        use auto.list: List
        fn main() {
            let list = List.new()
            for i in 0..1000 { list.push(i) }  // 超过 Fixed<64>
        }
    "#;
    // 应返回错误...
}
```

---

## 向后兼容性策略

### 兼容性保证

```auto
type List<T, S : Storage = DefaultStorage> {
    // S 默认为 DefaultStorage
    // 旧代码 List<T> 自动变为 List<T, DefaultStorage>
}
```

### 分阶段推出

1. **Phase 1-4**：添加 Storage 类型（无破坏性变更）
2. **Phase 5-6**：更新 List 使用 `S = DefaultStorage`（向后兼容）
3. **Phase 7-8**：启用 Prelude 导出（可通过 feature flag 控制）

### Feature Flag

```toml
[features]
default = []
storage-injection = []  # 启用 Storage 环境注入
```

---

## 关键实施文件

### 必须修改的文件（按优先级）

1. **`crates/auto-lang/src/ast/types.rs`**
   - 添加 `StorageType` 和 `StorageKind`
   - 更新 Type 枚举

2. **`crates/auto-lang/src/target.rs`**（新建）
   - Target 枚举和检测逻辑

3. **`crates/auto-lang/src/universe.rs`**
   - `inject_environment()` 方法
   - `get_env_val()` / `set_env_val()`

4. **`crates/auto-lang/src/interp.rs`**
   - 在初始化时调用环境注入
   - 加载 storage.at 模块

5. **`stdlib/auto/storage.at`**（新建）
   - Storage, Fixed, Dynamic 类型定义

6. **`stdlib/auto/list.at`**
   - 改为 `type List<T, S : Storage = DefaultStorage>`
   - 添加 `capacity()` 方法

7. **`crates/auto-lang/src/vm/list.rs`**
   - 更新 `list_new()` 检查存储限制
   - 添加 `list_capacity()`

8. **`crates/auto-lang/src/trans/c.rs`**
   - 生成 Fixed vs Dynamic 的不同 C 代码

9. **`stdlib/auto/prelude.at`**
   - 导出 DefaultStorage
   - 启用 List 导出

10. **`crates/auto/src/main.rs`**
    - 添加 `--target` CLI 参数

---

## 成功标准

### Phase 1-2: 类型系统和目标检测
- ✅ Storage 类型正确解析
- ✅ 目标检测工作正常
- ✅ CLI `--target` 标志功能正常

### Phase 3-4: 环境和 Storage 模块
- ✅ 环境注入正确填充 Universe
- ✅ `storage.at` 成功转译为 C
- ✅ Storage 类型在 Prelude 中可用

### Phase 5-6: List 增强
- ✅ List 接受存储参数
- ✅ VM 实现尊重存储策略
- ✅ Prelude 导出目标依赖的 List

### Phase 7-8: 转译和测试
- ✅ MCU 目标生成固定大小 C 代码
- ✅ PC 目标生成堆分配 C 代码
- ✅ 所有测试通过

### 最终验收
- ✅ 用户写 `List<int>`，MCU 得 Fixed，PC 得 Dynamic
- ✅ 无需修改现有用户代码
- ✅ 生成的 C 代码无警告编译
- ✅ Fixed 存储性能优于 Dynamic

---

## 时间估算

- **类型系统扩展**：1-2 天
- **目标检测系统**：1 天
- **环境注入**：1-2 天
- **Storage 模块**：1 天
- **List 扩展**：1-2 天
- **Prelude 集成**：0.5 天
- **C Transpiler**：1-2 天
- **测试**：1 天
- **总计**：8.5-11.5 天

---

## 风险缓解

### 技术风险

**风险 1：类型系统复杂性**
- 影响：高 - 可能破坏解析器
- 缓解：增量实现，大量单元测试
- 回退：存储参数可选，默认为 Dynamic

**风险 2：C Transpiler Bug**
- 影响：高 - 生成无效 C 代码
- 缓解：全面的 a2c 测试，人工审查
- 回退：两个目标都使用堆分配

**风险 3：性能回归**
- 影响：中 - Fixed 可能比预期慢
- 缓解：基准测试，优化热路径
- 回退：允许手动覆盖策略

### 运营风险

**风险 4：破坏性变更**
- 影响：高 - 破坏现有用户代码
- 缓解：Feature flags，逐步推出
- 回退：维护独立的 List 类型

---

## 未来增强（超出范围）

1. **自定义 Fixed 容量**：用户显式指定 `List<int, Fixed<128>>`
2. **混合存储**：超过 Fixed 容量时自动切换到 Dynamic
3. **编译时容量分析**：分析器建议最优 Fixed<N> 大小
4. **内存池集成**：使用自定义分配器替代系统 malloc
5. **String 存储**：对 String 应用相同策略（FixedString<64> vs HeapString）
6. **Rust Transpilation**：Dynamic 生成 `Vec<T>`，Fixed 生成 `[T; N]`

---

## 验证步骤

### 本地验证

```bash
# 1. 编译检查
cargo build --release

# 2. 运行所有测试
cargo test -p auto-lang

# 3. 测试目标检测
cargo run -- detect-target
cargo run -- c test.at --target mcu
cargo run -- c test.at --target pc

# 4. 检查生成的 C 代码
cat test/a2c/054_list_mcu/list_mcu.wrong.c
cat test/a2c/054_list_pc/list_pc.wrong.c

# 5. 编译生成的 C 代码（PC）
gcc test/a2c/054_list_pc/list_pc.expected.c -o test_pc
./test_pc

# 6. MCU 测试（需要交叉编译工具链）
arm-none-eabi-gcc test/a2c/054_list_mcu/list_mcu.expected.c
```

### 集成测试

```bash
# MCU 场景：固定存储
cargo test -p auto-lang test_mcu_fixed_storage

# PC 场景：动态存储
cargo test -p auto-lang test_pc_dynamic_storage

# 性能基准
cargo bench --bench storage_comparison
```
