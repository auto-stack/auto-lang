# Plan 283: a2py Python 转译器成熟度提升计划

## 一、现状分析

### 1.1 代码量对比

| 指标 | a2py (Python) | a2r (Rust) | 比率 |
|------|--------------|------------|------|
| 转译器代码行数 | 1,862 行 | 12,677 行 | 14.7% |
| 测试用例数 | 88 个 | 180 个 | 48.9% |
| 测试分类 | 20 个目录 | 81 个目录 | 24.7% |
| 标准库支持 | 无 | 742 行 (a2r-std) | 0% |
| 多文件项目支持 | 无 | 有 | 0% |
| FFI 运行时 | py_ffi.rs (305行) | 无（直接DLL调用） | - |

### 1.2 功能覆盖率矩阵

| 功能分类 | a2r | a2py | 状态 | 优先级 |
|---------|-----|------|------|--------|
| **基础语法** | | | | |
| 变量声明 (let/var/const) | ✅ | ✅ | 完成 | - |
| 函数定义 (含类型注解) | ✅ | ✅ | 完成 | - |
| 基础类型 (int/str/bool/float) | ✅ | ✅ | 完成 | - |
| 算术/比较/逻辑运算 | ✅ | ✅ | 完成 | - |
| print() | ✅ | ✅ | 完成 | - |
| **控制流** | | | | |
| if/elif/else | ✅ | ✅ | 完成 | - |
| for 循环 (range) | ✅ | ✅ | 完成 | - |
| for 条件循环 (→ while) | ✅ | ✅ | 完成 | - |
| loop + break | ✅ | ✅ | 完成 | - |
| continue | ✅ | ✅ | 完成 | - |
| **类型系统** | | | | |
| struct → class/dataclass | ✅ | ✅ | 完成 | - |
| enum → Enum | ✅ | ✅ | 完成 | - |
| union → dataclass | ✅ | ✅ | 完成 | - |
| tag → dataclass + factory | ✅ | ✅ | 完成 | - |
| 方法 (含 self) | ✅ | ✅ | 完成 | - |
| 泛型函数 (类型擦除) | ✅ | ✅ | 完成 | - |
| 泛型结构体 | ✅ | ✅ | 完成 | - |
| **字符串** | | | | |
| F-string (→ f-string) | ✅ | ✅ | 完成 | - |
| 字符串拼接 | ✅ | ✅ | 完成 | - |
| **模式匹配** | | | | |
| is → match/case | ✅ | ✅ | 完成 | - |
| 通配符模式 | ✅ | ✅ | 完成 | - |
| 枚举模式 | ✅ | ⚠️ | 基础完成 | - |
| 结构体解构 | ✅ | ❌ | **缺失** | P1 |
| **Option/Result** | | | | |
| Some(x) → x | ✅ | ✅ | 完成 | - |
| None → None | ✅ | ✅ | 完成 | - |
| Ok/Err | ✅ | ✅ | 完成 | - |
| ?. 传播 | ✅ | ⚠️ | **语义丢失** | P1 |
| Null 合并 (??) | ✅ | ✅ | 完成 | - |
| **高级特性** | | | | |
| 闭包/lambda | ✅ | ✅ | 完成 | - |
| 元组 | ✅ | ✅ | 完成 | - |
| 对象字面量 | ✅ | ✅ | 完成 | - |
| async/await | ✅ | ✅ | 完成 | - |
| spec → Protocol | ✅ | ✅ | 完成 | - |
| **模块/导入** | | | | |
| use 语句 | ✅ | ❌ | **完全跳过** | P0 |
| pub 可见性 | ✅ | ❌ | **缺失** | P2 |
| 多文件项目 | ✅ | ❌ | **缺失** | P1 |
| `use.py` 导入Python库 | N/A | ❌ | **缺失** | P0 |
| **标准库映射** | | | | |
| AutoLang→Python stdlib映射 | ✅ | ❌ | **完全缺失** | P0 |
| Python第三方库调用 | N/A | ❌ | **缺失** | P0 |
| **Rust/Python特有** | | | | |
| 所有权/借用 | ✅ | N/A | 不适用 | - |
| 指针操作 | ✅ | N/A | 不适用 | - |
| C FFI | ✅ | N/A | 不适用 | - |
| derive 属性 | ✅ | ❌ | 需要@dataclass等 | P2 |
| 纯Python模式 | N/A | ❌ | 有价值 | P2 |
| **运行时FFI** | | | | |
| PyO3 桥接 (VM内) | N/A | ✅ | 完成 | - |

### 1.3 关键问题分析

#### 问题1: `use` 语句完全被跳过 (P0)
```rust
// python.rs line 373
Stmt::Use(_) => Ok(false),  // 完全跳过，不生成任何 import
```
**影响**: 无法导入任何 Python 库（标准库或第三方），这是最致命的缺陷。

#### 问题2: `?.` 错误传播语义丢失 (P1)
```rust
// python.rs line 249
Expr::ErrorPropagate(e) => self.expr(e, out),  // 直接透传，丢失提前返回语义
```
**影响**: Auto 的 `x.?` 意味着"如果 x 是 None/Err 则提前返回"，但 Python 输出只是 `x`，完全丢失了错误传播逻辑。

#### 问题3: 静态方法生成错误 (P1)
```python
# 生成的代码 (错误):
class Math:
    def add(self, a: int, b: int) -> int:  # 有 self 参数
        return a + b

# 调用:
result = Math.add(3, 4)  # 但作为静态方法调用，少传了 self!
```
**影响**: 生成的代码运行时会报 `TypeError`。

#### 问题4: 无标准库映射 (P0)
a2r 有 742 行的 `a2r-std` 库映射 AutoLang 内置函数到 Rust stdlib。a2py 完全没有对应实现。
- `len(x)` → `len(x)` (相同，但需要映射)
- `list.push(item)` → `list.append(item)`
- `map.set(k, v)` → `dict[k] = v`
- `str.contains(s)` → `s in str`

#### 问题5: 多文件项目不支持 (P1)
a2r 支持 `transpile_rust_project()` 生成完整 Rust 项目。a2py 没有多文件支持，无法生成 Python 包结构。

#### 问题6: 生成代码不够 Pythonic (P1)

| 问题 | 当前输出 | Pythonic 输出 |
|------|---------|--------------|
| 结构体带方法 | 手写 `__init__` | 始终用 `@dataclass` + 方法 |
| 异常处理 | `Exception(msg)` | `raise ValueError(msg)` 或 `try/except` |
| 枚举 | `Enum` + `auto()` | 可用 `StrEnum`, `IntEnum` |
| 字典方法 | 需映射 | `dict.update()`, `dict.get()` |
| 列表推导 | 不支持 | `[x for x in ...]` |
| 上下文管理器 | 不支持 | `with` 语句 |

## 二、目标

### 核心目标
1. **Python库调用**: 能通过 Auto 代码调用任意 Python 库（标准库+第三方），生成正确的 `import` 语句
2. **Pythonic输出**: 生成的 Python 代码应该是地道的 Python，不是"带着外语味道的翻译"
3. **库编写能力**: 能用 Auto 编写功能完整的 Python 库，生成的代码可以作为独立 Python 包发布

### 量化指标
- 测试从 88 → 200+ 个
- 转译器代码从 1,862 → ~4,000 行
- 覆盖 a2r 80%+ 的可迁移功能
- 生成代码能通过 `mypy --strict` 类型检查
- 生成代码能通过 `pylint` 基本检查

## 三、实施计划

### Phase 1: 基础能力补全 (P0 — 预估 3 天)

#### Task 1.1: `use` 语句 → Python import
```
// Auto 代码                    →  Python 代码
use json                        →  import json
use json: dumps, loads          →  from json import dumps, loads
use os: path                    →  from os import path
use.py numpy                    →  import numpy as np
use.py numpy: array             →  from numpy import array
use.py pandas: DataFrame        →  from pandas import DataFrame
```

**实现要点**:
- 修改 `Stmt::Use(_)` 从 `Ok(false)` 改为实际生成 import
- 增加 `UseKind::Python` 支持（或识别 `use.py` 语法）
- 收集所有 import 并放到文件顶部
- 支持 `as` 别名

#### Task 1.2: AutoLang 内置函数 → Python stdlib 映射
```
// Auto 内置                    →  Python 映射
len(x)                          →  len(x)           # 相同
print(x)                        →  print(x)          # 相同
type_name(x)                    →  type(x).__name__
sleep_ms(ms)                    →  time.sleep(ms / 1000)
time_now()                      →  time.time()
```

**实现要点**:
- 在 `call()` 方法中添加内置函数识别
- 维护 AutoLang → Python stdlib 函数映射表
- 自动追踪需要的 stdlib import (如 `import time`)

#### Task 1.3: 集合方法 Pythonic 映射
```
// Auto 方法                     →  Python 方法
list.push(item)                 →  list.append(item)
list.pop()                      →  list.pop()
list.len()                      →  len(list)
list.contains(item)             →  item in list
list.join(sep)                  →  sep.join(list)
map.set(key, val)               →  map[key] = val
map.get(key)                    →  map.get(key)
map.has(key)                    →  key in map
str.len()                       →  len(str)
str.contains(s)                 →  s in str
str.split(sep)                  →  str.split(sep)
str.trim()                      →  str.strip()
```

**实现要点**:
- 在 `dot()` 方法中拦截方法调用
- 根据 receiver 类型和方法名进行映射
- 某些方法需要改为函数调用形式 (如 `.len()` → `len()`)

### Phase 2: 语义正确性修复 (P1 — 预估 3 天)

#### Task 2.1: 修复 `?.` 错误传播
```
// Auto 代码:
let result = parse(data).?

// Python 应生成:
result = parse(data)
if result is None:
    return None
```

**实现要点**:
- `Expr::ErrorPropagate` 不能简单透传
- 需要生成为 `if x is None: return None` 语句序列
- 对于 Result 类型，需要 `try/except` 或返回 `Err`

#### Task 2.2: 修复静态方法
```
// Auto 代码:
type Math {
    fn add(a int, b int) int { a + b }
}

// Python 应生成:
class Math:
    @staticmethod
    def add(a: int, b: int) -> int:
        return a + b
```

**实现要点**:
- 检测方法是否使用 `self`（Auto 中无 self 参数的方法 = 静态方法）
- 为无 self 的方法添加 `@staticmethod` 装饰器

#### Task 2.3: 异常处理
```
// Auto 代码:
fn safe_divide(a int, b int) Result {
    if b == 0 {
        return Err("division by zero")
    }
    return Ok(a / b)
}

// Python 应生成:
def safe_divide(a: int, b: int) -> Result:
    if b == 0:
        return Err("division by zero")
    return Ok(a / b)

// 或更 Pythonic 的方式:
def safe_divide(a: int, b: int):
    if b == 0:
        raise ValueError("division by zero")
    return a / b
```

#### Task 2.4: 结构体解构模式匹配
```
// Auto 代码:
is point {
    Point { x, y } -> print(x)
}

// Python 应生成:
match point:
    case Point(x=x, y=y):
        print(x)
```

### Phase 3: Pythonic 增强 (P1 — 预估 3 天)

#### Task 3.1: 始终使用 @dataclass + 方法
当前：有方法时手写 `__init__`，无方法时用 `@dataclass`
目标：始终用 `@dataclass`，方法自然附加

```python
# 当前输出 (不一致):
class Point:
    def __init__(self, x: int, y: int):
        self.x = x
        self.y = y

    def modulus(self) -> int:
        return self.x * self.x + self.y * self.y

# 目标输出 (Pythonic):
@dataclass
class Point:
    x: int
    y: int

    def modulus(self) -> int:
        return self.x * self.x + self.y * self.y
```

#### Task 3.2: Python 类型注解完善
```python
# 使用现代 Python 类型注解 (3.10+)
from typing import Optional, Union

# 集合类型注解
def process(items: list[int]) -> dict[str, int]:
    ...

# 使用 Python 3.10+ 语法
def process(items: list[int]) -> dict[str, int] | None:
    ...
```

#### Task 3.3: 枚举增强
```python
# 当前:
class Color(Enum):
    RED = auto()
    GREEN = auto()

# 支持 IntEnum, StrEnum:
class Color(str, Enum):
    RED = "red"
    GREEN = "green"
```

#### Task 3.4: 列表推导和生成器表达式
```
// Auto 代码:
let squares = [x * x for x in items]

// Python:
squares = [x * x for x in items]
```

### Phase 4: Python 库生态集成 (P0 — 预估 3 天)

#### Task 4.1: `use.py` 语法支持
```auto
// 调用 Python 标准库
use.py os
use.py os.path: join, exists
use.py json: dumps, loads
use.py collections: defaultdict

// 调用 Python 第三方库
use.py numpy
use.py requests: get, post
use.py pandas: DataFrame
```

#### Task 4.2: requirements.txt 生成
```toml
# requirements.txt (自动生成)
requests>=2.28.0
numpy>=1.24.0
pandas>=2.0.0
```

#### Task 4.3: Python 标准库智能映射
```
// Auto 惯用写法          →  Python 惯用写法
fs.read("file.txt")       →  open("file.txt").read()
fs.write("f.txt", data)   →  open("f.txt", "w").write(data)
http_get(url)              →  requests.get(url).json()
json_parse(s)              →  json.loads(s)
json_stringify(obj)        →  json.dumps(obj)
```

### Phase 5: 多文件项目支持 (P2 — 预估 2 天)

#### Task 5.1: Python 包结构生成
```
myproject/
├── __init__.py
├── main.py          # 入口文件
├── models.py        # 类型定义
├── utils.py         # 工具函数
└── requirements.txt # 依赖清单
```

#### Task 5.2: 模块间导入
```auto
// main.at
use models: User, Post
use utils: format_date

// 生成 main.py:
from models import User, Post
from utils import format_date
```

### Phase 6: 测试扩充和质量保证 (持续)

#### Task 6.1: 测试对齐 a2r
参照 a2r 的测试分类，为 a2py 补充以下测试类别：
- `13_delegation` — Python 通过 `__getattr__` 实现委托
- `14_modules` — import 语句测试
- `16_python_std` — Python 标准库调用测试
- `17_pure_python` — 纯 Python 代码生成测试

#### Task 6.2: Python 代码质量验证
- 使用 `python -m py_compile` 验证生成的代码语法正确
- 使用 `mypy` 验证类型注解
- 使用 `black --check` 验证代码格式
- 关键测试用例实际运行 Python 验证输出

## 四、优先级排序

| 优先级 | 任务 | 理由 |
|--------|------|------|
| **P0** | Task 1.1 use → import | 没有import就无法调用任何Python库 |
| **P0** | Task 1.2 内置函数映射 | 基础功能不映射则代码无法运行 |
| **P0** | Task 1.3 集合方法映射 | 数据操作是最常用功能 |
| **P0** | Task 4.1 use.py 支持 | 调用Python库的核心能力 |
| **P0** | Task 4.2 requirements.txt | 第三方库必须管理依赖 |
| **P0** | Task 4.3 stdlib智能映射 | 生成可运行代码的基础 |
| **P1** | Task 2.1 ?. 传播修复 | 语义正确性 |
| **P1** | Task 2.2 静态方法修复 | 当前生成错误代码 |
| **P1** | Task 2.3 异常处理 | Python异常处理习惯 |
| **P1** | Task 2.4 结构体解构 | 模式匹配完整性 |
| **P1** | Task 3.1 @dataclass统一 | 代码一致性 |
| **P1** | Task 3.2 类型注解完善 | 类型安全 |
| **P1** | Task 3.3 枚举增强 | Pythonic程度 |
| **P2** | Task 5.1 包结构生成 | 多文件项目 |
| **P2** | Task 5.2 模块间导入 | 大型项目支持 |
| **P2** | Task 3.4 列表推导 | Pythonic增强 |

## 五、完成度总结

### 当前完成度评估

| 维度 | 完成度 | 说明 |
|------|--------|------|
| **语法转换** | 70% | 基本语法转换完成，但缺少 import、delegation 等 |
| **类型系统** | 60% | 基础类型映射完成，但缺乏 Python 类型注解深度 |
| **语义正确性** | 50% | ?. 语义丢失，静态方法有 bug，Option/Result 过于简化 |
| **Pythonic度** | 55% | dataclass、Protocol 用得好，但很多地方有"翻译腔" |
| **库调用能力** | 5% | use 被跳过，无 stdlib 映射，无法调用 Python 库 |
| **项目支持** | 0% | 无多文件、无包结构、无依赖管理 |
| **测试覆盖** | 49% | 88 vs 180 测试，缺少多个关键分类 |
| **综合** | **~40%** | 基础框架可用，但离"生产级"差距较大 |

### 与 a2r 差距根因

1. **代码量差距 7x**: a2r 12,677 行 vs a2py 1,862 行 — a2py 只实现了 a2r 14.7% 的代码量
2. **无标准库层**: a2r 有专门的 a2r-std crate，a2py 完全没有
3. **import 系统空白**: 这是调用 Python 库的前提，被完全跳过了
4. **方法映射缺失**: AutoLang 方法到 Python 方法的映射逻辑完全缺失

### 达到目标所需的投入

- **Phase 1-2 (P0+P1 核心功能)**: ~6 天，~2,000 行新增代码
- **Phase 3-4 (Pythonic + 库生态)**: ~6 天，~1,500 行新增代码  
- **Phase 5-6 (项目 + 测试)**: ~4 天，~1,000 行新增代码
- **总计**: ~16 天，~4,500 行新增代码

## 六、实施记录

### Batch 1-4（已完成）

| Batch | 任务 | 状态 |
|-------|------|------|
| **Batch 1** | Task 1.1 `use` → import, Task 1.2 内置函数映射, Task 4.1 `use.py` | ✅ |
| **Batch 2** | Task 1.3 方法 Pythonic 映射, Task 2.2 `@staticmethod` | ✅ |
| **Batch 3** | Task 3.1 统一 `@dataclass` | ✅ |
| **Batch 4** | `!` → `not` 修复, 空 class `pass`, py_compile 验证 | ✅ |

### Batch 5（已完成）

| 任务 | 状态 | 说明 |
|------|------|------|
| **Task 2.4** 结构体解构 | ✅ | `Expr::StructPattern` → Python dataclass 模式 `case Point(x, y)` |
| **Task 2.1** 类型追踪 | ✅ 基础设施 | `local_var_types` + `infer_type_from_expr()`, ErrorPropagate 透传行为不变 |
| **Task 2.3** 异常处理 | 延期 | 依赖类型追踪，需更多测试用例驱动 |
| **Task 2.1** ErrorPropagate 语义 | 延期 | 需类型推断 + 表达式→语句重构 |

### 当前完成度

- **96 个 a2py 测试全部通过**
- **所有 .expected.py 通过 `py_compile` 语法验证**
- **测试覆盖**: 96 vs 原始 88（新增结构体解构、静态方法、import、方法映射等测试）
- **关键新增能力**: import 系统、stdlib 映射、方法 Pythonic 映射、@dataclass 统一、@staticmethod、结构体解构模式、类型追踪基础设施
