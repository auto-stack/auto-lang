# Auto 语言 Python Parity 验证路线图

> **目标**：扩展 Auto 语言的 parity 测试集到第三维度——Auto 作为 Python 替代。验证 AutoVM 脚本模式（通过 `use.py` 调用 Python 库）的行为与原始 Python 脚本一致，且 a2py 转译器能将 Auto 脚本转译回与原始 Python 行为一致的代码。同时调研 Auto→C 的高性能转译路径可行性。

## 1. 背景与动机

### 1.1 三大能力维度

Auto 语言的 parity 测试已覆盖两大维度：

| 维度 | 含义 | 现有覆盖 |
|------|------|---------|
| **Producer** | Auto 作为 Rust 的替代，实现 Rust 功能库 | base64, url, serde_json, regex, sha2, rusqlite, tokio (7 库, 257 测试) |
| **Consumer** | Auto 作为应用书写语言，实现 Rust app | cli_app, c_fs_app, c_text_app 等 |
| **Python 替代** ← 新增 | Auto 作为 Python 的替代，实现常见 Python 脚本 | 本文档 |

### 1.2 为什么 Auto 能替代 Python

AutoVM 支持直接调用 Python 库（通过 `use.py` 语法 + PyO3 嵌入）。这意味着：
- 开发者可以用 Auto 的脚本模式写 Python 脚本
- Auto 脚本调用 Python 库的方式与 Python 原生几乎一致
- 开发周转快（脚本模式，无需编译）
- 额外优势：Auto 脚本可通过 a2py 转译回 Python，或通过 a2c/use.c 路径转译成高性能 C 版本

### 1.3 当前 Python 互操作状态

| 能力 | 状态 | 说明 |
|------|------|------|
| `use.py module: fn`（VM 调用 Python） | ✅ 完整 | PyO3 嵌入，自动类型检测 |
| `use.py module`（自动发现模块可调用项） | ✅ 完整 | `dir()` + `inspect.signature()` |
| Auto↔Python 值 marshalling | ✅ 完整 | int/float/bool/str/list/dict 嵌套 |
| a2py 转译器（Auto→Python 源码） | ✅ 完整 | 96 个快照测试 |
| `use.c <header.h>`（C FFI） | ✅ 完整 | libloading, 5 个标准库 |
| a2c 转译器（Auto→C 源码） | ✅ 完整 | 224 个测试 |
| PyFFI float 返回值 | ⚠️ 有 bug | float 被字符串化（codegen 只存 1 slot） |
| 真实 `.at` 示例调用 Python | ❌ 缺失 | 只有 import 语法测试 |
| Auto→Python→C 桥接路径 | ❌ 不存在 | 两个 FFI 系统完全独立 |

### 1.4 设计决策

| 决策项 | 选择 | 理由 |
|--------|------|------|
| 验证范围 | **Python 三方对比** | 原始 Python vs AutoVM(use.py) vs a2py 转译 |
| Auto→C 路径 | **只做调研记录** | 首轮不纳入验证框架，调研可行性 |
| 起点 | **Tier 1 纯计算库** | 全部有 C 底层，无 IO 副作用，最易测试 |
| 验证框架 | **复用 auto-parity** | 扩展现有工具新增 Python backend，非新建 |
| 脚本类型 | **按脚本类型渐进** | 从最简单的 math 开始，逐步增加复杂度 |

## 2. 三方流水线与验证框架

### 2.1 Python parity 三方流水线

```
                    ┌─────────────────────────────────────────────┐
                    │         原始 Python 脚本 (.py 文件)            │
                    │   (Python 写的脚本，调用标准库)                │
                    └──────────────┬──────────────────────────────┘
                                   │ (行为 oracle)
                                   ▼
                    ┌──────────────────────────┐
                    │     python3 script.py     │
                    │       → TAP 输出           │
                    └──────────────┬────────────┘
                                   │
                    ┌──────────────┼──────────────┐
                    ▼                             ▼
             ┌──────────────┐            ┌──────────────┐
             │   AutoVM     │            │    a2py      │
             │  script.at   │            │  script.at   │
             │ (use.py 调用  │            │  → transpile │
             │  Python 库)   │            │  → .py 文件   │
             │ auto script  │            │ python3 运行  │
             │   .at        │            │  转译后的.py  │
             └──────┬───────┘            └──────┬───────┘
                    │                           │
                    ▼                           ▼
             ┌──────────────────────────────────────────┐
             │        输出规范化 (TAP)                    │
             │   每条用例: [name] [pass/fail] [actual]   │
             └──────────────────┬───────────────────────┘
                                ▼
             ┌──────────────────────────────────────────┐
             │     auto-parity 三方比较器                  │
             │  (复用现有 compare.rs + report.rs)          │
             └──────────────────────────────────────────┘
```

### 2.2 与 Rust parity 的对比

| | Rust parity (现有) | Python parity (新增) |
|---|---|---|
| oracle | `cargo test`（Rust 原生） | `python3 script.py`（Python 原生） |
| 脚本后端 | AutoVM (`auto run`) | AutoVM (`auto run`，通过 `use.py`） |
| 转译后端 | a2r → Rust → cargo build → 运行 | a2py → Python → python3 运行 |
| Bug 分类 | VM bug / a2r bug / 复刻 bug | VM+PyFFI bug / a2py bug / 复刻 bug |

### 2.3 Bug 来源分类

| AutoVM | a2py | Python 原版 | 判定 |
|--------|------|-----------|------|
| ✓ | ✓ | ✓ | 一致，通过 |
| ✓ | ✓ | ✗ | 复刻 bug（Auto 与 Python 行为不一致） |
| ✓ | ✗ | ✓ | a2py 转译 bug |
| ✗ | ✓ | ✓ | AutoVM/PyFFI bug |
| ✗ | ✗ | ✓ | 复刻 bug（VM 和 a2py 一致地错） |
| ✗ | ✗ | ✗ | 测试用例问题（需人工确认） |

### 2.4 auto-parity 扩展

在现有 `runner.rs` 中新增两个函数：

- `run_python_oracle(config)` — 运行 `python3 tests/python/*.py`，收集 TAP
- `run_a2py(config)` — `auto trans --path <file>.at python` → 转译成 `.py` → `python3` 运行 → 收集 TAP

`auto-parity` 的 `run_library` 函数增加 `ParityMode` 判断：
- `ParityMode::Rust`：现有逻辑（VM + a2r + cargo test）
- `ParityMode::Python`：新逻辑（Python oracle + AutoVM + a2py）

模式通过库目录下的结构自动检测：如果 `tests/python/` 目录存在则用 Python 模式，否则用 Rust 模式。

## 3. 库选择与目录结构

### 3.1 首轮库选择（Tier 1 纯计算库，5 个）

| 库 | Python 模块 | C 底层 | 用途 | 测试用例数 | 特殊考虑 |
|-----|------------|--------|------|-----------|---------|
| **py_math** | `math` | libm | 三角函数、对数、取整、常量 | ~12 | float 返回值（当前被字符串化的 bug） |
| **py_random** | `random` | `_random` (Mersenne Twister) | 种子可重现的随机数 | ~8 | 必须用 `seed(n)`，否则不可重现 |
| **py_datetime** | `datetime` | `_datetime` | 日期算术、格式化、解析 | ~10 | 用显式日期，不用 `now()` |
| **py_struct** | `struct` | C 模块 | 二进制打包/解包 | ~8 | 字节精确比较 |
| **py_uuid** | `uuid` | `_uuid` | UUID 生成（uuid5 确定性） | ~5 | 只用 `uuid5`，不用随机 `uuid4` |

**选择理由**：
1. 全部有 **C 底层**（对 Auto→C 路径有价值）
2. 全部是 **纯计算**（无 IO/网络/文件副作用，输出确定性可比较）
3. **复杂度递进**：math（最简单）→ random（状态）→ datetime（对象方法）→ struct（字节精确）→ uuid（命名空间+哈希）
4. **已知限制的暴露**：py_math 会暴露 float 字符串化 bug；py_datetime 会暴露 Python 对象方法调用的 marshalling

### 3.2 目录结构

沿用现有 parity 三段式，新增 `tests/python/` 段：

```
parity/libs/py_math/
├── README.md              # 复刻说明：Python 模块、版本、覆盖范围
├── tests/
│   ├── python/            # 原始 Python 脚本（oracle）
│   │   ├── test_math.py   # 调用 math 库，输出 TAP
│   │   └── requirements.txt
│   └── auto/              # Auto 测试（VM + a2py 共用）
│       └── test_math.at   # use.py math 调用，输出 TAP
```

与 Rust parity 的区别：
- **无 `auto/` 库源目录**——Python parity 不复刻库实现，而是通过 `use.py` 调用原始 Python 库。所以 `tests/auto/*.at` 就是全部 Auto 代码。
- **新增 `tests/python/`**——存放原始 Python 脚本作为 oracle。
- **无 `tests/rust/`**——不需要 Rust oracle。

### 3.3 测试用例的写法约定

**原始 Python 脚本**（`tests/python/test_math.py`）：
```python
import math

def tap_ok(n, name):
    print(f"ok {n} - {name}")

def tap_not_ok(n, name, diag):
    print(f"not ok {n} - {name} # {diag}")

def check(n, name, actual, expected):
    if abs(actual - expected) < 1e-9:
        tap_ok(n, name)
    else:
        tap_not_ok(n, name, f"got {actual} expected {expected}")

if __name__ == "__main__":
    check(1, "test_sqrt", math.sqrt(16), 4.0)
    check(2, "test_pi", round(math.pi, 5), 3.14159)
    check(3, "test_ceil", math.ceil(3.2), 4)
    check(4, "test_floor", math.floor(3.8), 3)
```

**Auto 测试**（`tests/auto/test_math.at`）：
```auto
use.py math: sqrt, ceil, floor, pi

fn tap_ok(n int, name str) {
    print("ok " + n.to(str) + " - " + name)
}

fn check(n int, name str, actual float, expected float) {
    if actual == expected {
        tap_ok(n, name)
    } else {
        print("not ok " + n.to(str) + " - " + name + " # got " + actual.to(str))
    }
}

fn main() {
    check(1, "test_sqrt", sqrt(16), 4.0)
    check(2, "test_pi", pi, 3.14159)
    check(3, "test_ceil", ceil(3.2).to(float), 4.0)
    check(4, "test_floor", floor(3.8).to(float), 3.0)
}
```

### 3.4 float 比较策略

当前 PyFFI 的 float 返回值被字符串化（`py_ffi.rs:444-447` 的已知 bug）。采用混合策略：
- 前 8 个用例用整数结果（`sqrt(16)=4`），避开 float 精度问题
- 最后 2-4 个用例用非整数 float（`sqrt(2)=1.414...`），暴露并记录 bug 为 known-divergence

### 3.5 a2py 转译验证

a2py 转译后的 Python 代码应该和原始 Python **行为一致**（代码细节可能不同，如 `n.to(str)` → `str(n)`，但输出相同）。比较器只比 TAP 输出，不比源码。

## 4. 阶段划分与成功标准

### 4.1 阶段总览

```
P0: 框架扩展 → P1: math + random → P2: datetime + struct + uuid
```

### 4.2 P0: 框架扩展（auto-parity 增加 Python backend）

**目标**：在 `auto-parity` 中新增 Python parity 模式，用 py_math 的第一个用例验证端到端跑通。

**工作内容**：
- `runner.rs`：新增 `run_python_oracle()` 和 `run_a2py()`
- `main.rs`：新增 `ParityMode` 检测
- `compare.rs`：BugSource 分类复用，新增 "PyFFI bug" 标签
- 创建 `parity/libs/py_math/` 骨架 + 一个用例验证三方跑通

**出口条件**：
- [ ] `auto-parity run py_math` 三方跑通，至少 1 个用例一致
- [ ] Python 模式和 Rust 模式不互相干扰（现有 7 个 Rust 库仍 100%）

### 4.3 P1: py_math + py_random

**目标**：验证数学计算和有状态随机数生成的三方一致性。

**py_math**（~12 用例）：`sqrt`, `ceil`, `floor`, `pow`, `log`, `fabs`, `pi`, `e`
- 整数结果 8 个 + 非整数 float 4 个（暴露 float 字符串化 bug）

**py_random**（~8 用例）：`seed(n)` + `randint`, `random()`, `choice`, `shuffle`
- 关键验证点：种子相同时，三方是否产生相同序列

**出口条件**：
- [ ] py_math：三方一致率 ≥80%
- [ ] py_random：种子化用例三方一致率 100%

### 4.4 P2: py_datetime + py_struct + py_uuid

**目标**：验证日期对象方法、二进制打包、哈希 UUID 的三方一致性。

**py_datetime**（~10 用例）：`date` 构造、`timedelta` 算术、`isoformat`、`strftime`

**py_struct**（~8 用例）：`pack`/`unpack` 各种格式，字节级精确比较

**py_uuid**（~5 用例）：`uuid5(namespace, name)` 确定性生成

**出口条件**：
- [ ] py_datetime：三方一致率 ≥80%
- [ ] py_struct：三方一致率 ≥90%
- [ ] py_uuid：三方一致率 100%

### 4.5 Auto→C 路径调研（贯穿 P0-P2）

**目标**：调研 Auto 脚本通过 a2c 转译 + `use.c` 替代 `use.py` 实现高性能版本的可行性。

**调研 1：胶水代码转译可行性**（P0 期间）
- 取 py_math 的 Auto 脚本，手动尝试 a2c 转译
- 验证 Auto 的控制流、变量、函数调用能否转译成合法 C

**调研 2：Python 库的 C 底层映射**（P1 期间）
- 对 math 库：`use.py math: sqrt` → 对应 `use.c <math.h>` 的 `sqrt`
- 验证 `use.c` 调用 `libm` 的结果是否与 `use.py` 调用 Python `math` 一致

**调研 3：完整 Auto→C 路径验证**（P2 期间）
- 将 py_math 的 Auto 脚本改写为 `use.c` 版本
- a2c 转译 + C 编译 + 运行
- 四方对比：原始 Python vs AutoVM(use.py) vs a2py(use.py) vs AutoC(use.c)

**调研产出**：一份可行性报告，记录在每个 Tier 1 库上 Auto→C 路径的状态。

## 5. 预期的 known-divergences

| 预期 divergence | 类型 | 影响 |
|----------------|------|------|
| PyFFI float 返回值字符串化 | 待修复 bug | py_math 非整数用例 |
| Python 对象方法 marshalling | 待验证 | py_datetime 的 `d.isoformat()` |
| `use.py` 常量引用 | 待验证 | py_uuid 的 `uuid.NAMESPACE_DNS` |
| a2py 对 `use.py` 的转译 | 待验证 | 所有库的 import 处理 |

## 6. 后续扩展方向（本轮不做）

完成 Tier 1 后，可按以下优先级扩展：

| 优先级 | 库类别 | 示例 | C 底层 |
|--------|--------|------|--------|
| B | 文本/格式 | csv, toml, configparser, tabulate, jinja2 | 否（纯 Python） |
| C | 配置 | yaml (pyyaml) | 部分（libyaml C） |
| D | 数据分析 | numpy, pandas | 是（C 核心） |
| E | 解析 | lxml, xml.etree, beautifulsoup4 | 是（libxml2/expat） |
| F | 复杂 | matplotlib, pillow | 混合 |
