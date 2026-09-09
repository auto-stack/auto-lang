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
parity/libs/python/py_math/
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
- 创建 `parity/libs/python/py_math/` 骨架 + 一个用例验证三方跑通

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

## 7. 中缀运算符调研节（Plan 539 收官追加，2026-09-04）

Plan 539 定案：`*` 保持逐元素语义（torch/numpy 同义），矩阵乘走
`py_matmul(a, b)` / `a.matmul(b)` 函数形式。以下两个中缀运算符为
**独立语言层决策**，登记于此供后续计划调研：

### 7.1 中缀 `@`（matmul）

- **现状**：`@` 为注解前缀（声明位置）。词法上注解在声明位置、
  matmul 在表达式中缀位置，可区分——但需要 lexer/parser 双通道改造
  与歧义测试（装饰器风格 `@fn` 表达式、`a @ b @ c` 链）。
- **收益面**：`W = A @ B + C` 训练脚本可读性；与 Python 数值生态
  逐字对齐。
- **成本面**：词法双义 + 既有 `@` 注解语料回归面；dunder 路由臂
  （Plan 539 T10 已建 `__matmul__` 通道）只差中缀表面。
- **建议**：独立小计划，先探针注解/中缀冲突语料，再定案。

### 7.2 中缀 `**`（幂）

- **现状**：Auto 无幂运算符（`pow` 函数在）。`**` 词法与指针/解引用
  家族无冲突，风险低于 `@`。
- **收益面**：`lr ** 0.5`、数学脚本惯用法。
- **成本面**：dunder `__pow__` 路由臂 + 优先级表插入（右结合）。
- **建议**：可与 `@` 合并同一运算符批计划。

### 7.3 W3 类派生（py_subclass）延期备注

自定义 `nn.Module`/`Dataset` 需要 Python 侧类工厂（`exec` 生成类 +
`__len__`/`__getitem__`/`forward` 方法绑回 Auto 回调）。回调桥
（Plan 539 T21，thread-local 任务槽）已打通单回调通道（map/apply_
探针实证），但类工厂的方法绑定面 + GIL/生存期约束审查超出 W3 预算，
**显式延期**——组合式替代（`nn.Sequential`/裸 Linear 栈）已由
py_torch_train 套件覆盖为金样。见 KNOWN-DEBT P539-D4。

### 7.4 PyTorch 后续计划队列（2026-09-04，Plan 539 归档后登记）

> Plan 539 已归档（22/22 步，W3 类派生按 7.3 延期）。以下为 539 明确
> 划出范围或执行中实证的后续方向，立项时直接引用本节，避免重新考古。
> 立项纪律：跟着需求与 539 折叠结论逐个立项，不预建；每个计划沿用
> 三方 parity 方法论（AutoVM ≡ a2py ≡ 原生 Python）+ README 惯用法
> 固化 + phase 表注册。

| # | 方向 | 范围 / 动机 | 前置与关联 |
|---|------|------------|-----------|
| ① | ~~中缀 `@` / `**` 运算符~~ ✅ 已交付（[Plan 560](../../plans/archive/560-script-mode-w2-sugar-batch.md) T07 C5/C6，归档在案；2026-09-09 复审核销时复跑实证）：`@` 词法消歧（`@T` 引用不动）+ `a @ b`→py_matmul(456)、`a ** b`→py_pow(473, GIL operator.pow 数字/`__pow__` 通用）——解析层直产桥调用；torch 句柄 `a @ b`=tensor(32)、原生 `2 ** 10`=1024 实测绿 | ~~7.1/7.2~~ 560 T07（语义等价分歧注记：发射从 s2s 规则前移到解析层，`@T` 类型位零影响实证） |
| ② | `use.py` 别名/子模块语法 | 539 T08 裁定别名语法**不做**（importlib 句柄通道覆盖需求面：`use.py importlib: import_module` → 句柄 + py_getattr）；子模块项导入（`use.py torch.nn: Linear`）可用但同名项跨模块静默后者胜——别名语法消除该摩擦与静默覆盖风险 | 539 T08 四形态探针（待澄清②核销位） |
| ③ | ~~py 返回值方法分派动态化~~ ✅ 已交付（2026-09-06, [Plan 569](../../plans/569-py-ret-dynamic-dispatch.md)）：codegen py-类型侧表+组合子双通道路由（`.len()`→obj_len/其余→obj_call），py_list 去规避示范 8/8，p5-p9 全相位零回归 | ~~P539-D2 根治：py 调用返回被 codegen 谎记类型，`.len()` 静态路由到 str.len 读垃圾~~ 已清偿（KNOWN-DEBT P539-D2 ✅）；顺登记 use.rs 同族谎言 P569-D1 |
| ④ | Auto 原生 struct 运算符重载 | 539 dunder 路由只服务 `PyObjectHandle`；Auto 自己的 `type T` 上重载 `+ * ==` 需 trait 体系——与 Plan 525 延后的 trait/动态分发同一条语言线，宜合并立项 | 525 非目标清单；W1 dunder 路由为语义对照 |
| ⑤ | bulk ndarray buffer 封送 | 训练批数据 Auto 侧持有时的必要件（缓冲协议/零拷贝）；539 靠"数据活 Python 侧"约定绕开，批输入规模化后绕不开 | 539 非目标清单；需求驱动 |
| ⑥ | for-in tuple 解包 | DIV-PY-TUPLE-1：tuple→List 拍平后多变量循环解包在 a2py 侧 unpack 报错，W2 套件用单变量+索引规避；语法级解包或 Auto tuple 值类型二选一 | 539 T07 执行注记；DIV-PY-TUPLE-1 |
| ⑦ | a2py 语义修补批 | P539-D3 复合接收者无括号（`py_call(t==t,"sum")` 优先级错）+ DIV-PY-CLOSURE-1 语句体闭包降级 set 字面量——两项集中清偿（531 式债批；~~P539-D5 py_call_may×kwargs~~ 已被 Plan 567 清偿，2026-09-09 复核更新） | KNOWN-DEBT P539-D3 + DIV-PY-CLOSURE-1 |
| ⑧ | 条件立项 | GPU/CUDA device 路径验证专项（539 只断言确定性 CPU）；transformers/datasets 上层生态套件（沿用 461/539 三方方法论） | 需求触发 |

### 7.5 长期方向：Auto→C（libtorch）替代热路径

`use.py torch` 的热路径（matmul/conv/reduction）有 C 底层（libtorch），
§4.5 调研线的延伸：`use.c` + libtorch 绑定可作 PyFFI 之外的免-GIL 高
性能替代路径——与 §6-D 数据分析类同轨，暂不立项，需求驱动再调研。
