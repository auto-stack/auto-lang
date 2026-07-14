# Auto 语言 Rust 库复刻路线图

> **目标**：通过用 Auto"复刻"常见 Rust 库，验证并扩展 Auto 作为 Rust 脚本语言的能力——使 AutoVM 脚本运行、a2r 转译发布、原始 Rust 三方行为完全一致，从而将 Auto 的"脚本开发→转译发布"开发模式从较小示例扩展到中等甚至更大的工程。

## 1. 背景与动机

### 1.1 开发模式

Auto 语言的设计目标是作为 Rust 的脚本语言，支持如下开发流程：

1. **开发阶段**：用 Auto 的脚本模式（AutoVM 运行）实现代码逻辑，可调用任意 Rust 代码。无需每次编译，开发周转率高。
2. **发布阶段**：开发迭代完成后，用 a2r 转译器将 Auto 代码转译为 Rust 代码，即可发布。

核心要求：**a2r 转译的发布版 Rust 代码行为必须与开发阶段的 AutoVM 脚本运行行为完全一致。**

### 1.2 当前状态

该开发模式已初步实现，但仅在较小示例上验证过：

- **双模式架构已存在**：同一个解析器通过 `CompileDest::{Interp, TransRust}` 同时服务于 AutoVM 和 a2r。`#[vm]`/`#[rs]` 注解允许按目标选择实现。
- **a2r 覆盖核心子集**：函数、结构体、枚举、tag、union、spec（trait）、泛型、闭包、async/await、模式匹配（`is`→`match`）、所有权（view/mut/take）、错误传播（`.?`→`?`），有 308 个 golden 测试。
- **FFI 桥已存在**：静态注册（`#[rust_fn]` + `inventory`）+ 动态加载（`use.rust` + `RustFfiBridge` dlopen）。
- **`a2r-std` crate**：为转译代码提供运行时镜像。

### 1.3 核心挑战：parity gap（一致性缺口）

这是扩展到更大工程时最大的结构性障碍：

| | AutoVM 路径 | a2r 路径 |
|---|---|---|
| `foo()` 如何解析 | `CALL_NAT <id>` → `NativeInterface` → `stdlib.rs` 中的 shim | 直接调用 `a2r_std::foo` |
| 实现来源 | `#[rust_fn]` 标注的函数 | `a2r-std` 中**独立重实现**的版本 |
| 当前测试方式 | 字节精确 golden 文件比较 | （未做运行时行为比较） |
| 不透明对象 | `RustStdlibObject` 堆句柄 | 原生 Rust 类型（无等价物） |

简言之：VM 和 a2r 调用 `foo()` 时调用的是**两套不同的 Rust 实现**，且**没有运行时行为一致性测试**。现有测试只检查转译出的 Rust 文本是否符合 golden 文件。

### 1.4 方案思路

用 Auto 来"复刻"常见 Rust 库，从简单库开始，验证 **AutoVM 运行的复刻版、a2r 转译的复刻版、原始 Rust 库**三者运行行为一致。原始 Rust 库作为行为预言机（oracle）。

### 1.5 设计决策

本设计基于以下确认的决策：

| 决策项 | 选择 | 理由 |
|--------|------|------|
| 复刻方式 | **混合方式** | Auto 原生构造重写库逻辑 + `use.rust` 调用 Auto 无法原生表达的原语。同时测试语言表达力和 FFI 一致性，最贴近真实 Auto 项目的开发模式。 |
| 验证方法 | **测试套件端口** | 将原始 Rust 库的官方测试套件端口到 Auto，三方各自运行同一套测试。覆盖面广、回归保障强。 |
| 验证框架 | **Auto 双后端 + Rust 原生** | 测试用例用 Auto 写，同一文件既被 AutoVM 执行也被 a2r 转译后执行；原始 Rust 库的测试用 `cargo test` 独立运行。外部工具 `auto-parity` 运行三方、收集输出、做规范化比较。 |
| 起点复杂度 | **纯算法/编码库起步** | 代码量小、纯计算、无重副作用、有清晰 API 和官方测试套件。验证框架能快速迭代。 |

## 2. 验证框架架构

### 2.1 三方流水线

```
                    ┌─────────────────────────────────────────────┐
                    │            测试用例 (.at 文件)                 │
                    │   (Auto 写的测试，调用被测库的 Auto 复刻版)     │
                    └──────────────┬──────────────────────────────┘
                                   │
                    ┌──────────────┼──────────────┐
                    ▼              ▼               ▼
             ┌──────────┐   ┌──────────┐    ┌──────────────┐
             │  AutoVM  │   │   a2r    │    │  Rust 原生    │
             │ auto run │   │auto build│    │  cargo test  │
             │  (脚本)   │   │ + rustc  │    │ (上游端口)    │
             └────┬─────┘   └────┬─────┘    └──────┬───────┘
                  │              │                  │
                  ▼              ▼                  ▼
             ┌──────────────────────────────────────────┐
             │        输出规范化 (TAP)                   │
             │   每条用例: [name] [pass/fail] [actual]   │
             └──────────────────┬───────────────────────┘
                                ▼
             ┌──────────────────────────────────────────┐
             │        三方比较器 (auto-parity)            │
             │  对每个用例: 三方结果必须一致              │
             │  不一致 → 分类: VM bug / a2r bug / 复刻bug │
             └──────────────────────────────────────────┘
```

### 2.2 关键设计决策

**2.2.1 测试用例用 Auto 写，但刻意保持简单**

测试用例本身只用最简单的 Auto 构造（赋值、函数调用、assert、print），避免测试本身成为 a2r/VM 的负担。测试逻辑是：调用库函数 → 拿到结果 → assert 预期值。这样如果测试用例的 a2r 转译失败，能立刻定位为转译器 bug，而非测试本身复杂。

**2.2.2 原始 Rust 库作为 oracle，其测试用 `cargo test` 独立跑**

从上游端口一份 Rust 测试到 `tests/` 目录，`cargo test` 运行。这份测试断言同样的输入→输出关系。它是"正确答案"的来源。

**2.2.3 输出规范化：TAP 格式**

每个后端输出 TAP（Test Anything Protocol）格式（简单文本，一行一个测试结果）：

```
ok 1 - base64_encode_empty
not ok 2 - base64_encode_hello # got "aGVsbG8=" expected "aGVsbG8="
```

三方都输出 TAP，比较器逐行比对。TAP 比 JSON 简单，且 `auto run` 只需 `print` 即可生成，不需要 JSON 库依赖（降低测试本身对被测系统的依赖）。

**2.2.4 三方比较器是一个独立的小工具**

一个 Rust 写的 CLI 工具 `auto-parity`（parity = 一致性），职责单一：

- 运行 `auto run <test.at>` → 收集 TAP
- 运行 `auto build <test.at> && ./build/<name>` → 收集 TAP
- 运行 `cargo test` → 收集 TAP（`cargo test` 原生输出可转 TAP）
- 逐用例比对三方结果，输出差异报告

**2.2.5 Bug 来源分类逻辑**

| AutoVM | a2r | Rust 原生 | 判定 |
|--------|-----|-----------|------|
| ✓ | ✓ | ✓ | 一致，通过 |
| ✓ | ✓ | ✗ | 复刻 bug（Auto 复刻版与原始库行为不一致）|
| ✓ | ✗ | ✓ | a2r 转译 bug |
| ✗ | ✓ | ✓ | AutoVM bug |
| ✗ | ✗ | ✓ | 复刻 bug（VM 和 a2r 一致地错，但与原始库不一致）|
| ✗ | ✗ | ✗ | 测试用例本身有问题，或三方一致地"正确"（需人工确认）|

这张表是整个方案的诊断核心——它能精确定位问题出在 VM、转译器还是复刻逻辑。

## 3. 库选择与复刻层次模型

### 3.1 库选择原则

1. **纯计算优先**：无 IO/网络/文件系统副作用（输出可确定性比较）
2. **有官方测试套件**：上游 `cargo test` 可直接端口作为 oracle
3. **API 表面小且稳定**：公共函数数量可控（<50 个），签名稳定
4. **递进覆盖 Auto 构造**：每个库要能压力测试一组特定的 Auto 语言特性
5. **依赖少**：不引入庞大的依赖树（否则 FFI marshalling 复杂度爆炸）
6. **真实使用频率高**：复刻的库应该是在真实 Rust 项目中常见的

### 3.2 库选择清单（四个阶段，共 8 个库）

| 阶段 | 库 | 代码量 | 测试套件 | 压力测试的 Auto 特性 | 用 use.rust 调用的原语 |
|------|-----|--------|---------|---------------------|----------------------|
| **P1** | `base64` | ~500 行 | ✓ 丰富 | 字符串操作、字节切片、循环、错误处理 | 无（纯 Auto 可实现）|
| **P1** | `url` | ~2000 行 | ✓ 丰富 | 结构体、枚举、模式匹配、字符串解析、Option | 无 |
| **P2** | `serde_json`（子集）| ~1500 行 | ✓ 丰富 | 递归数据结构（tag/enum）、泛型、trait(spec)、模式匹配 | 无 |
| **P2** | `regex`（简化版）| ~1000 行 | ✓ 丰富 | 状态机、枚举、字符迭代、回溯/递归 | 无 |
| **P3** | `sha2`（SHA-256）| ~800 行 | ✓ 丰富 | 位运算、固定大小数组、u32/u64 运算、循环展开 | 无 |
| **P3** | `rusqlite`（查询层）| ~1200 行 | ✓ 丰富 | trait 对象、错误传播(.?)、Result/Option、泛型 | `use.rust rusqlite`（Connection/Statement）|
| **P4** | `reqwest`（同步子集）| ~1000 行 | ✓ 丰富 | async/await、错误传播、结构体、Builder 模式 | `use.rust reqwest`、`use.rust hyper` |
| **P4** | `tokio`（任务子集）| ~800 行 | ✓ 适中 | async、spawn/join、channel、task 模型 | `use.rust tokio` |

### 3.3 阶段递进逻辑

- **P1（纯字符串/编码）**：验证基础验证框架可用，测试 Auto 的字符串、字节、循环、错误处理。`base64` 和 `url` 都是纯计算、零依赖、API 清晰。
- **P2（数据结构与算法）**：引入递归数据结构、泛型、trait——这些是中等工程的骨架。`serde_json` 测试 `tag`/`enum` 的递归表达力；`regex` 测试复杂控制流。
- **P3（位运算 + FFI 起点）**：`sha2` 测试 u32/u64/位运算的精确性（VM 和 a2r 在整数类型上必须完全一致）；`rusqlite` 首次引入 `use.rust`，测试 FFI marshalling 一致性。
- **P4（异步与并发）**：最终挑战——async/await 和任务模型。如果三方在 async 语义上一致，说明 Auto 可以支撑中等规模的后端项目。

### 3.4 复刻层次模型（混合方式的具体规则）

每个库的 Auto 复刻版分为两层：

```
┌─────────────────────────────────────────┐
│           公共 API 层 (Auto 原生)          │  ← 用 Auto 的类型、枚举、
│  encode(input str) str                  │     函数、模式匹配重写
│  decode(input str) Result[str, Error]   │
│  Url.parse(input str) Result[Url, Error]│
└──────────────────┬──────────────────────┘
                   │ 调用
┌──────────────────▼──────────────────────┐
│          原语层 (use.rust / use auto)     │
│                                         │
│  P1-P2: 纯 use auto (str, list, map)    │  ← 无外部 Rust 依赖
│  P3+:   use.rust rusqlite::Connection   │  ← 调用原始 Rust crate
│         use.rust reqwest::blocking::get │
└─────────────────────────────────────────┘
```

**分层规则**：

1. **公共 API 层必须用 Auto 原生构造实现**——这是被测试的部分。类型定义（`type`/`enum`/`tag`）、控制流（`if`/`for`/`is`）、错误处理（`Result`/`.?`）、泛型、trait（`spec`/`ext`）都用 Auto 写。

2. **原语层分两种情况**：
   - **P1-P2**（纯计算库）：只用 `use auto.str`、`use auto.list` 等内置 stdlib。不引入 `use.rust`。完全测试 Auto 语言表达力 + a2r 对 Auto stdlib 的转译一致性。
   - **P3+**（IO/系统库）：对 Auto 无法原生表达的原语（数据库连接、HTTP 客户端、网络 socket），用 `use.rust` 调用原始 Rust crate。同时测试 FFI marshalling 一致性（VM 通过 `RustFfiBridge` 动态加载 vs a2r 直接 `use` 编译时链接）。

3. **`use.rust` 的一致性是 P3+ 的核心验证点**——同一个 `use.rust rusqlite::Connection`，在 VM 模式下通过 `RustFfiBridge` dlopen + `NativeInterface` 调度，在 a2r 模式下编译为直接 `rusqlite::Connection` 调用。两者必须行为一致。这是当前最大的 parity gap，P3+ 专门暴露并修复它。

4. **每个库的复刻版和原始 Rust 版共享同一套测试断言**——即"输入 X → 期望输出 Y"的映射在三方完全相同。只是测试代码的语言不同（Auto vs Rust）。

## 4. 目录结构与项目布局

### 4.1 顶层结构

整个验证方案作为一个独立的工作区，挂在现有的 `auto-lang` workspace 下：

```
auto-lang/
├── Cargo.toml                      # workspace 根，新增 parity/ 成员
├── crates/
│   ├── auto-lang/                  # 现有
│   ├── a2r-std/                    # 现有
│   └── ...
└── parity/                         # 新增：整个验证方案的工作区
    ├── Cargo.toml                  # workspace，管理 a2r-std 扩展 + auto-parity 工具
    │
    ├── crates/
    │   ├── auto-parity/            # 三方比较器 CLI 工具
    │   │   ├── Cargo.toml
    │   │   └── src/main.rs
    │   │
    │   └── a2r-std-ext/            # a2r-std 的扩展层（P3+ 需要）
    │       ├── Cargo.toml
    │       └── src/lib.rs
    │
    ├── libs/                       # 被复刻的库（Auto 源 + Rust 原生 oracle）
    │   │
    │   ├── base64/                 # P1 第一个库
    │   │   ├── README.md           # 复刻说明：API 覆盖范围、已知偏差
    │   │   ├── auto/               # Auto 复刻版
    │   │   │   ├── base64.at       # 公共 API 层（Auto 原生实现）
    │   │   │   └── tables.at       # 辅助表（编码表等）
    │   │   ├── tests/              # 测试用例
    │   │   │   ├── auto/           # Auto 写的测试（VM + a2r 共用）
    │   │   │   │   ├── encode.at
    │   │   │   │   ├── decode.at
    │   │   │   │   └── edge_cases.at
    │   │   │   └── rust/           # Rust 原生测试（oracle）
    │   │   │       └── Cargo.toml  # 依赖原始 base64 crate
    │   │   └── expected/           # 每个测试的期望 TAP 输出（可选，黄金文件）
    │   │
    │   ├── url/                    # P1 第二个库（同构）
    │   ├── serde_json/             # P2
    │   ├── regex/                  # P2
    │   ├── sha2/                   # P3
    │   ├── rusqlite/               # P3
    │   ├── reqwest/                # P4
    │   └── tokio/                  # P4
    │
    ├── results/                    # auto-parity 运行结果（gitignore）
    │   └── <timestamp>/
    │       ├── base64.tap.vm
    │       ├── base64.tap.a2r
    │       ├── base64.tap.rust
    │       └── base64.report.md    # 三方差异报告
    │
    └── docs/
        ├── parity-guide.md         # 如何运行验证、如何添加新库
        └── known-divergences.md    # 已知的、已接受的偏差清单
```

### 4.2 单个库的内部结构（以 base64 为例）

```
libs/base64/
├── README.md
├── auto/
│   ├── base64.at              # 公共 API: encode(str) str, decode(str) Result[str, Error]
│   └── tables.at              # const 编码表
├── tests/
│   ├── auto/
│   │   ├── encode.at          # test: encode("") == ""
│   │   │                      # test: encode("f") == "Zg=="
│   │   │                      # test: encode("fo") == "Zm8="
│   │   │                      # test: encode("foo") == "Zm9v"
│   │   │                      # test: encode("foob") == "Zm9vYg=="
│   │   │                      # test: encode("fooba") == "Zm9vYmE="
│   │   │                      # test: encode("foobar") == "Zm9vYmFy"
│   │   ├── decode.at          # 反向测试
│   │   └── edge_cases.at      # 边界: 空串、非法字符、padding 错误
│   └── rust/
│       ├── Cargo.toml         # [dependencies] base64 = "0.22"
│       └── tests/
│           ├── encode.rs      # 同样的断言，用原始 base64 crate
│           └── decode.rs
└── expected/
    └── encode.tap.golden      # 期望的 TAP 输出（可选）
```

### 4.3 测试用例的写法约定

**Auto 测试用例**（`tests/auto/encode.at`）——刻意保持简单：

```auto
# base64 编码测试
use base64: encode

fn test_encode_empty() {
    assert(encode("") == "", "empty string")
}

fn test_encode_single() {
    assert(encode("f") == "Zg==", "single char")
}

fn test_encode_foobar() {
    assert(encode("foobar") == "Zm9vYmFy", "foobar")
}

fn main() {
    test_encode_empty()
    test_encode_single()
    test_encode_foobar()
}
```

每个 `assert` 失败时 print 一行 TAP：`not ok 2 - test_encode_single # got "Zg==" expected "Zg=="`。成功时 print：`ok 2 - test_encode_single`。

**Rust 原生测试**（`tests/rust/tests/encode.rs`）——同样的断言：

```rust
use base64::{engine::general_purpose, Engine};

#[test]
fn test_encode_empty() {
    assert_eq!(general_purpose::STANDARD.encode(b""), "");
}

#[test]
fn test_encode_single() {
    assert_eq!(general_purpose::STANDARD.encode(b"f"), "Zg==");
}
```

### 4.4 设计理由

1. **隔离性**：验证方案不污染 `auto-lang` 的核心 crate。`auto-lang` 的改动（修 VM bug、修 a2r bug）是验证方案的*结果*，但验证方案本身住在独立目录。
2. **可扩展性**：每加一个库只需 `libs/` 下加一个目录，遵循固定结构，`auto-parity` 自动发现。
3. **a2r-std 扩展有处可放**：P3+ 需要扩展 `a2r-std`（比如添加 rusqlite 的 Rust 运行时支持），放在 `parity/crates/a2r-std-ext/` 而非侵入原始 `a2r-std`。

### 4.5 `auto-parity` 的发现机制

`auto-parity` 扫描 `libs/*/` 目录，每个含 `auto/*.at` 和 `tests/` 的目录视为一个待验证库。运行时：

```
auto-parity base64          # 只验 base64
auto-parity --phase p1      # 验所有 P1 库
auto-parity --all           # 验所有库
auto-parity base64 --fix    # 发现差异后交互式修复（更新 golden / 提示修 bug）
```

## 5. 阶段划分与成功标准

### 5.1 阶段总览

每个阶段有明确的**入口条件**（上一阶段必须达成）和**出口条件**（本阶段必须达成才能进入下一阶段）。阶段不是按时间排，而是按**验证框架能力的成熟度**排——每个阶段验证框架本身也要进化。

```
P0: 框架就绪 → P1: 纯字符串/编码 → P2: 数据结构与算法 → P3: 位运算 + FFI → P4: 异步与并发
```

各阶段严格线性推进：每个阶段的入口条件包含上一阶段的全部出口条件。

### 5.2 P0: 框架就绪（前置阶段）

**目标**：搭建 `auto-parity` 工具 + `parity/` 工作区骨架，用一个 hello-world 级的假库验证整个流水线跑通。

**工作内容**：
- 创建 `parity/` 工作区结构
- 实现 `auto-parity` CLI：发现库、运行三方、收集 TAP、比较、生成报告
- 定义 TAP 输出规范（成功/失败的格式）
- 实现 bug 分类逻辑（§2.2.5 的判定表）
- 创建一个 `libs/_dummy/` 假库：Auto 写一个 `add(a,b)` 函数，三方各跑一个 `assert(add(1,2)==3)`，验证流水线端到端跑通

**出口条件**：
- [ ] `auto-parity _dummy` 三方全部 pass，报告显示 "3/3 consistent"
- [ ] 人为注入一个 VM bug（让 `add` 返回 `a+b+1`），`auto-parity` 正确分类为 "AutoVM bug"
- [ ] 人为注入一个 a2r bug（让转译输出少一个 `+`），`auto-parity` 正确分类为 "a2r transpiler bug"
- [ ] 报告可读、差异定位到具体测试用例

### 5.3 P1: 纯字符串/编码（base64 + url）

**目标**：验证 Auto 的字符串、字节操作、循环、错误处理在三方完全一致。这是最低风险的起点，也是对验证框架的第一次真实考验。

**入口条件**：P0 出口条件全部满足。

**工作内容**：
- 复刻 `base64`：`encode`/`decode`，纯 Auto 实现（只用 `use auto.str`）
- 复刻 `url`：`Url.parse`，用 Auto 的 `type`/`enum`/`is`（模式匹配）实现
- 端口两个库的官方测试套件到 `tests/auto/` 和 `tests/rust/`
- 运行 `auto-parity base64 url`

**出口条件**：
- [ ] base64：三方测试一致率 100%（所有用例 VM=a2r=Rust）
- [ ] url：三方测试一致率 ≥95%，剩余差异全部记录在 `known-divergences.md` 并有明确原因
- [ ] 发现的所有 VM/a2r bug 已修复或已记录为 issue
- [ ] `known-divergences.md` 建立并遵循格式规范

### 5.4 P2: 数据结构与算法（serde_json 子集 + regex 简化版）

**目标**：引入递归数据结构、泛型、trait（spec）——中等工程的骨架。验证 Auto 的 `tag`/`enum` 递归表达力和复杂控制流。

**入口条件**：P1 出口条件满足。

**工作内容**：
- 复刻 `serde_json` 子集：`Value` enum（Null/Bool/Num/Str/Array/Object）、`parse`/`to_string`，用 Auto 的 `tag` 实现
- 复刻 `regex` 简化版：支持 `.`/`*`/`+`/`?`/字符类，用 Auto 的状态机+枚举实现
- 端口测试套件
- 运行 `auto-parity serde_json regex`

**出口条件**：
- [ ] serde_json 子集：三方一致率 ≥95%
- [ ] regex 简化版：三方一致率 ≥95%
- [ ] Auto 的 `tag`（递归枚举）在三方行为一致（这是 P2 的关键验证点）
- [ ] Auto 的 `spec`（trait）在三方行为一致
- [ ] 泛型类型在三方行为一致

### 5.5 P3: 位运算 + FFI 起点（sha2 + rusqlite 查询层）

**目标**：两个关键验证——(1) u32/u64 位运算的精确一致性，(2) 首次引入 `use.rust`，测试 FFI marshalling 一致性（VM 动态加载 vs a2r 编译时链接）。

**入口条件**：P2 出口条件满足。

**工作内容**：
- 复刻 `sha2`（SHA-256）：纯 Auto 实现，压力测试 u32 位运算、固定数组、循环展开
- 复刻 `rusqlite` 查询层：公共 API 用 Auto 写（连接管理、查询构建、结果遍历），底层通过 `use.rust rusqlite::Connection` 调用原始 crate
- 扩展 `a2r-std-ext`：为 rusqlite 添加 Rust 运行时支持（a2r 转译后链接的代码需要 rusqlite 的 Rust 侧封装）
- 运行 `auto-parity sha2 rusqlite`

**出口条件**：
- [ ] sha2：三方一致率 100%（位运算必须精确，无容错）
- [ ] rusqlite 查询层：三方一致率 ≥90%
- [ ] `use.rust` 在 VM（`RustFfiBridge` dlopen）和 a2r（编译时 `use`）下行为一致——**这是 P3 的核心验证点**
- [ ] `VMConvertible` 对 rusqlite 涉及的类型（`Connection`、`Statement`）的 marshalling 一致性已验证或已修复
- [ ] `a2r-std-ext` 机制可用：新增 FFI 库的 Rust 运行时支持有清晰路径

### 5.6 P4: 异步与并发（reqwest 同步子集 + tokio 任务子集）

**目标**：最终挑战——async/await 和任务模型的三方一致性。如果通过，说明 Auto 可以支撑中等规模的后端项目。

**入口条件**：P3 出口条件满足，特别是 `use.rust` 一致性已验证。

**工作内容**：
- 复刻 `reqwest` 同步子集：`get`/`post`/`Client`，用 Auto 的 async/`.await` 包装 `use.rust reqwest`
- 复刻 `tokio` 任务子集：`spawn`/`join`/`channel`，用 Auto 的 task 模型包装 `use.rust tokio`
- async 测试需要特殊处理：三方都跑同一组 async 测试，输出规范化需处理异步完成的顺序问题
- 运行 `auto-parity reqwest tokio`

**出口条件**：
- [ ] reqwest 同步子集：三方一致率 ≥85%（async 引入更多不确定性，门槛适当降低）
- [ ] tokio 任务子集：三方一致率 ≥85%
- [ ] Auto 的 `~T`（async）→ `async fn` 转译在三方行为一致
- [ ] Auto 的 `expr.go`（spawn）→ `tokio::spawn` 在三方行为一致
- [ ] channel（`send`/`recv`）在三方行为一致
- [ ] 异步测试的输出规范化方案确立（处理完成顺序不确定的问题）

## 6. 已知风险与缓解策略

| 风险 | 影响 | 概率 | 缓解策略 |
|------|------|------|---------|
| **a2r 的 24-pass regex 后处理在大规模代码上崩溃** | P2+ 转译产出不可编译的 Rust | 高 | 每个库复刻后立即跑 `auto build`，发现 regex fix 失败时回退到手动修复转译输出，同时记录为 a2r 的 issue。长期目标是用类型推断替代 regex post-pass。 |
| **`use.rust` FFI marshalling 类型不足** | P3 rusqlite 的 `Connection` 等复杂类型无法通过 `VMConvertible` | 高 | P3 开始时先扩展 `VMConvertible`（或用 `RustStdlibObject` opaque handle 模式），将此作为 P3 的首要任务而非事后补救。 |
| **a2r-std 与 VM stdlib 行为偏差** | P1-P2 中 `use auto.str` 的行为在三方不一致 | 中 | P1 第一个库就暴露此问题。`auto-parity` 的 bug 分类表能区分"复刻 bug"和"stdlib 偏差"。建立 `known-divergences.md` 记录并逐步修复。 |
| **async 测试输出顺序不确定** | P4 三方比较失败不是因为 bug 而是因为时序 | 高 | 输出规范化层引入"排序模式"：按测试名而非执行顺序排序 TAP 行。async 测试内部用同步断言点（`await` 后立即 assert）而非依赖顺序。 |
| **上游库 API 变更** | oracle 测试失效 | 低 | `tests/rust/Cargo.toml` 锁定版本号（如 `base64 = "=0.22.0"`）。README 记录每个库的上游版本。 |
| **复刻工作量超出预期** | 阶段拖延 | 中 | 每个库定义"子集范围"（README 明确列出覆盖的 API），不追求 100% 复刻。P2 的 serde_json 和 regex 都是"简化版"。 |
| **VM 整数溢出/类型宽度不一致** | P3 sha2 的 u32 运算在 VM 和 Rust 间结果不同 | 中 | P3 专门用 sha2 压力测试此点。如果发现 VM 的 u32 wrap-around 行为与 Rust 不同，这是一个必须修复的 VM bug（非已知偏差）。 |

## 7. 一致率定义

**一致率**（consistency rate）= 三方结果完全一致的测试用例数 / 总测试用例数 × 100%。

"三方结果完全一致"指：AutoVM、a2r、Rust 原生对同一用例的断言结果（pass/fail）相同，且 fail 时的实际输出值相同。

排除项：状态为 `accepted` 的 divergence 不计入不一致用例（其测试断言已调整以兼容可接受的表现形式差异）。

示例：100 个用例中，98 个三方一致，1 个 VM bug（待修复），1 个可接受的错误消息格式差异（accepted），则一致率 = 99%（排除 accepted 后，98/99）。

## 8. known-divergences.md 格式规范

每条 divergence 有唯一 ID、明确分类、三方各自的行为描述、以及处理决策。

```markdown
## DIV-0001: base64 decode 对非法输入的行为

- **库**: base64
- **用例**: decode_with_invalid_chars
- **AutoVM 行为**: 返回 Err("invalid char at pos 3")
- **a2r 行为**: 返回 Err("invalid char at pos 3")
- **Rust 原生行为**: 返回 Err(InvalidByte(3, b'!'))
- **偏差类型**: 可接受（错误消息格式不同，但都正确拒绝非法输入）
- **状态**: accepted
- **原因**: Auto 的 Error 类型用字符串，Rust 原生用结构化错误。
  测试断言改为"decode 失败"而非"错误消息匹配"。
```

**偏差类型**取值：
- `可接受` — 三方语义一致但表现形式不同（如错误消息格式），测试断言已调整以兼容。
- `待修复` — 确认为 bug，需修复 VM 或 a2r，已记录为 issue。
- `已修复` — 曾经的 divergence，已通过修复消除。

**状态**取值：`accepted` / `open` / `fixed`
