# Plan 368 — Consumer-Mode Parity Suite Implementation Plan

> **计划号变更说明**：本计划原编号为 367。master 上另一个 agent 实施了
> "Plan 367 — codegen quality improvements"（Vue/DSL 代码生成质量），为消解
> 编号冲突，本 consumer-parity 计划重编号为 **368**（2026-07-22）。代码注释里
> 仍残留 "Plan 367" 字样（见 §"后续工作" FU-3），待批量同步。

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 为"Auto 作为库消费者做应用开发"这个使用情形，建立一组 parity 测试套件——AutoVM（调 `auto.*` stdlib）/ a2r 转译（调 `a2r_std::*`）/ 原生 Rust（直调底层 crate）三方对比，验证 Auto 调用外部能力的应用代码与等价 Rust 应用行为一致。填补现有 241 用例（全是"实现者模式"）未覆盖的"消费者模式"空白。

**Architecture:** 每个消费者用例是一个 `parity/libs/c_<name>/` 库，复用现有 `auto-parity` 三方对比框架。Auto 侧用 `use auto.<module>: <fn>` 调 stdlib 能力（fs/json/process/env/http），Rust oracle 直调底层 Rust 库（std::fs/serde_json/std::env）。三方对比 TAP 输出（应用行为结果）。确定性通过临时目录 + 固定输入 + in-process mock server 保证。

**Tech Stack:** Auto 语言 (.at), Rust (std::fs / serde_json / std::env / std::process), auto-parity 三方对比框架, in-process mock HTTP server (std::net::TcpListener)

**Design spec:** `docs/design/consumer-parity-strategy.md`

---

## 阶段总览

> **实施进度（2026-07-22 worktree `plan-367/consumer-parity`）**
>
> - **F1 ✅ 完成**（c_fs_app，7/7 三方一致，已提交）。实际用例数 7（非预估 ~16），
>   覆盖 write/read/empty/overwrite/exists±/mkdir+nested。
>   实施中发现并修复两个 a2r 缺口（让消费者代码在三端行为一致）：
>   1. `a2r-std::fs::read_text/read_to_string` 返回 `Option<String>` 而 VM 的
>      `auto.fs.read_text` 返回 `String`（错误时空串）——已对齐为返回 `String`。
>   2. a2r 转译器 `fs.*` 的 obj.method 分支未借用字符串参数，导致 owned path
>      被 move——已改为 `expr_as_str` 借用（见 commit）。
>   另外两条 a2r 转译器怪癖（owned `str` 变量登记为 StrSlice；用户函数的内联
>   拼接参数不自动 `.as_str()`）以 `.at` 源码侧的写法规避并记录在 README，
>   不在本计划范围内修。
> - **F2 ⛔ 阻塞**（c_json_app）：VM 无法 import `auto.json`——stdlib `json.at`
>   里的 `pub fn JsonValue.as_int(self JsonValue) int` 方法声明触发
>   `auto_syntax_E0007 Expected term, got Newline`（解析 `json.at:58` 附近）。
>   仅 `use auto.json`（无任何调用）即可复现。这是 VM 解析 stdlib json.at 的 bug，
>   类似 DIV-HTTP-LANG-1 但针对 json 模块，**超出 Plan 367 范围**。需单独修 VM
>   解析器（`self Type` 方法声明语法）后才能做 F2。
> - **F3 / F4 / F5 → 继续实施**：`auto.env`、`auto.process.args`、`auto.fs`+文本
>   均在 VM + a2r 双端实测可用，立即可做。实施顺序调整为 F3 → F4 → F5。

| 阶段 | 用例 | 调用能力 | 前置 | 预估用例数 |
|------|------|---------|------|-----------|
| **F1** | c_fs_app（文件读写器） | auto.file / auto.fs | 无 | ~16 |
| **F2** | c_json_app（JSON 配置处理器） | auto.json | 无（**实际阻塞：VM 解析 json.at**） | ~14 |
| **F3** | c_env_app（环境变量工具） | auto.env | 无 | ~10 |
| **F4** | c_process_app（CLI 参数解析） | auto.process.args + auto.file | 无 | ~8 |
| **F5** | c_text_app（文本批处理器） | auto.file + regex(复用) | 无 | ~12 |
| **F6** | c_http_get + c_http_post（HTTP 客户端） | auto.http | Phase 359 E4 (DIV-HTTP-LANG-1) + mock-server runner hook | ~20 |
| **F7** | c_wget（简易下载器）+ c_crawler（爬虫） | auto.http + auto.file + auto.json | F6 | ~20 |

**关键路径**：F1→F2→F3→F4→F5（Layer 1，立即可做，无前置）‖ Phase 359 E4 → F6 → F7（Layer 2，需修 parser bug）。

**L1 用例累计预期**：Layer 1 完成后 +60 新用例（241→301）；Layer 2 完成后再 +40（→341）。

---

## 文件结构（本计划触及的关键路径）

```
parity/libs/
├── c_fs_app/                      # F1: 文件读写消费者
│   ├── README.md
│   ├── auto/c_fs_app.at
│   ├── tests/auto/basic.at
│   └── tests/rust/{Cargo.toml, tests/c_fs_app.rs}
├── c_json_app/                    # F2: JSON 消费者
├── c_env_app/                     # F3: env 消费者
├── c_process_app/                 # F4: process 消费者
├── c_text_app/                    # F5: 文本批处理消费者
├── c_http_get/                    # F6: HTTP GET 消费者
├── c_http_post/                   # F6: HTTP POST 消费者
├── c_wget/                        # F7: 下载器
└── c_crawler/                     # F7: 爬虫
parity/crates/auto-parity/src/
├── main.rs                        # F6: phase_map 加 d5/d6 + mock setup hook
└── runner.rs                      # F6: run_library 加 mock-server setup/teardown
```

---

## 关键技术约定（所有用例通用）

### 确定性策略

1. **文件操作**：每方用独立临时目录（`auto.fs.temp_dir()` / `std::env::temp_dir()` + 固定子目录名如 `c_fs_app_test`），写入固定内容，读回断言。每方测试开始前清理目录，避免残留。
2. **TAP 对比**：三方都输出 `ok N - test_name` / `not ok N - test_name # diag`。parity runner 按测试名镜像对比。
3. **Rust oracle 不调 Auto stdlib**：直调底层 Rust 库（std::fs / serde_json），产出相同 TAP。

### parity 库命名与结构（复用现有约定）

每个 `c_<name>/` 库：
- `auto/c_<name>.at`：Auto 应用代码（`use auto.<module>: <fn>`，定义被测函数）
- `tests/auto/<scenario>.at`：Auto 测试（`use auto.c_<name>: <fn>`，TAP 断言）
- `tests/rust/Cargo.toml`：独立 crate（`[workspace]` 隔离，依赖 std 或对应 crate）
- `tests/rust/tests/c_<name>.rs`：Rust oracle（`#[test]`，`println!("ok N - name")`）

### Auto stdlib API（已核实，来自 stdlib/auto/*.at + VM 注册）

- **file**：`auto.file.read_text(path) str`、`write_text(path, content) int`、`exists(path) bool`、`read_bytes(path) []int`、`write_bytes(path, bytes) int`、`delete(path)`、`create_dir(path)`
- **fs**（别名）：`auto.fs.temp_dir() str`、`auto.fs.read_text`（= file.read_text）、`auto.fs.write_text` 等
- **json**：`auto.json.encode(value) str`、`parse(s) JsonValue?`、`JsonValue.type() str`、`as_string() str`、`as_int() int`、`get(key) JsonValue?`、`keys() str`、`has_key(key) int`
- **env**：`auto.env.get(key) str`、`set(key, val)`、`get_or(key, default) str`
- **process**：`auto.process.args() str`（返回空格分隔的参数串）
- **http**（Layer 2，需 E4）：`auto.http.get(url)`、`post(url, body)`

---

# Phase F1: c_fs_app — 文件读写消费者

**目标：** 验证 Auto 通过 `auto.file` 调用文件系统（读写/exists/create_dir）的应用，与 Rust std::fs 应用行为三方一致。

**出口条件：** `auto-parity run c_fs_app` 报 100% consistent（三方 TAP 输出一致）。

---

### Task F1.1: 写 Auto 应用 + 测试

**Files:**
- Create: `parity/libs/c_fs_app/auto/c_fs_app.at`
- Create: `parity/libs/c_fs_app/tests/auto/basic.at`

- [ ] **Step 1: 写 Auto 应用代码**

Create `parity/libs/c_fs_app/auto/c_fs_app.at`：

```auto
/// c_fs_app — 文件读写消费者应用。
/// 通过 auto.file 调用文件系统，验证 Auto 消费 std::fs 能力的行为。
use auto.file: read_text, write_text, exists, delete, create_dir
use auto.fs: temp_dir

/// 写入并读回文本，返回读回的内容（确定性）。
fn write_and_read(path str, content str) str {
    write_text(path, content)
    return read_text(path)
}

/// 检查文件是否存在，返回 1/0。
fn check_exists(path str) int {
    if exists(path) { return 1 }
    return 0
}

/// 创建目录、写文件、读回，返回读回内容。
fn mkdir_write_read(dir str, filename str, content str) str {
    var fullpath = dir + "/" + filename
    create_dir(dir)
    write_text(fullpath, content)
    return read_text(fullpath)
}
```

- [ ] **Step 2: 写 Auto 测试（TAP 断言）**

Create `parity/libs/c_fs_app/tests/auto/basic.at`：

```auto
use auto.c_fs_app: write_and_read, check_exists, mkdir_write_read
use auto.fs: temp_dir, delete
use auto.file: write_text

fn tap_ok(n int, name str) {
    print("ok " + n.to(str) + " - " + name)
}
fn tap_not_ok(n int, name str, diag str) {
    print("not ok " + n.to(str) + " - " + name + " # " + diag)
}
fn check_str(n int, name str, actual str, expected str) {
    if actual == expected { tap_ok(n, name) }
    else { tap_not_ok(n, name, "got \"" + actual + "\" want \"" + expected + "\"") }
}
fn check_int(n int, name str, actual int, expected int) {
    if actual == expected { tap_ok(n, name) }
    else { tap_not_ok(n, name, "got " + actual.to(str) + " want " + expected.to(str)) }
}

fn main() {
    var base = temp_dir() + "/c_fs_app_test"

    // 清理 + 准备
    delete(base)

    // 1. write_and_read
    var p1 = base + "/hello.txt"
    check_str(1, "test_write_read_basic", write_and_read(p1, "hello world"), "hello world")
    check_str(2, "test_write_read_unicode", write_and_read(p1, "你好世界"), "你好世界")
    check_str(3, "test_write_read_empty", write_and_read(p1, ""), "")
    check_str(4, "test_write_overwrite", write_and_read(p1, "second"), "second")

    // 2. exists
    check_int(5, "test_exists_yes", check_exists(p1), 1)
    var p_missing = base + "/no_such_file.txt"
    check_int(6, "test_exists_no", check_exists(p_missing), 0)

    // 7. mkdir + write + read
    var subdir = base + "/sub"
    check_str(7, "test_mkdir_write_read", mkdir_write_read(subdir, "nested.txt", "nested content"), "nested content")

    // 8. exists on nested
    check_int(8, "test_nested_exists", check_exists(subdir + "/nested.txt"), 1)

    // 清理
    delete(base)
}
```

- [ ] **Step 3: VM 验证 Auto 代码语法正确**

Run: `cd parity/libs/c_fs_app && D:/autostack/auto-lang/target/release/auto.exe tests/auto/basic.at`
Expected: 输出 8 行 `ok N - ...`（无 FAIL/syntax error）。

- [ ] **Step 4: Commit**

```bash
git add parity/libs/c_fs_app/auto/ parity/libs/c_fs_app/tests/auto/
git commit -m "feat(parity): c_fs_app Auto consumer code (Plan 367 F1)"
```

---

### Task F1.2: 写 Rust oracle + 跑 parity

**Files:**
- Create: `parity/libs/c_fs_app/tests/rust/Cargo.toml`
- Create: `parity/libs/c_fs_app/tests/rust/tests/c_fs_app.rs`
- Create: `parity/libs/c_fs_app/README.md`

- [ ] **Step 1: 写 Rust oracle（直调 std::fs）**

Create `parity/libs/c_fs_app/tests/rust/Cargo.toml`：

```toml
[package]
name = "c-fs-app-tests"
version = "0.1.0"
edition = "2021"

[dependencies]

[workspace]
```

Create `parity/libs/c_fs_app/tests/rust/tests/c_fs_app.rs`：用 `std::fs` 实现 `write_and_read`/`check_exists`/`mkdir_write_read` 等价逻辑，每个 `#[test]` 产出与 Auto 侧镜像的 TAP（`println!("ok N - test_name")`）。测试名必须与 Auto 侧完全一致：`test_write_read_basic`、`test_write_read_unicode`、`test_write_read_empty`、`test_write_overwrite`、`test_exists_yes`、`test_exists_no`、`test_mkdir_write_read`、`test_nested_exists`。用 `std::env::temp_dir().join("c_fs_app_test")` 做临时目录，每个 test 开头清理。

- [ ] **Step 2: 写 README**

Create `parity/libs/c_fs_app/README.md`：说明这是消费者模式 parity（调 auto.file 对比 std::fs），列出 API（write_and_read/check_exists/mkdir_write_read）+ 8 个用例。

- [ ] **Step 3: 跑 parity 三向**

Run: `cd parity && cargo run -p auto-parity -- --auto-binary ../target/release/auto.exe run c_fs_app`
Expected: `Consistency: 8/8 (100.0%)`（三方 TAP 一致）。

- [ ] **Step 4: 若有 diverge，诊断**

若不一致：检查 VM `auto.file.exists` 返回 bool vs Rust bool 在 TAP 串化是否一致；检查 `write_text` 返回值（int）是否被 TAP 断言用到。对齐 Auto 与 Rust 的行为定义。

- [ ] **Step 5: Commit**

```bash
git add parity/libs/c_fs_app/
git commit -m "feat(parity): c_fs_app three-way consumer parity (8/8) (Plan 367 F1)"
```

---

### Task F1.3: 把 c_fs_app 加入 phase 表

**Files:**
- Modify: `parity/crates/auto-parity/src/main.rs`

- [ ] **Step 1: phase_map 加 d5 消费者阶段**

在 `discover_libraries_by_phase` 的 phase_map 里加：

```rust
("d5", &["c_fs_app"]),
```

- [ ] **Step 2: report phases 加 d5**

在 Report 子命令的 phases 数组里加 `"d5"`。

- [ ] **Step 3: 验证 report 含 c_fs_app**

Run: `cd parity && cargo run -p auto-parity -- report --output docs/parity-dashboard.html`（确认 dashboard 含 c_fs_app，可选——较慢）。

- [ ] **Step 4: Commit**

```bash
git add parity/crates/auto-parity/src/main.rs
git commit -m "feat(parity): add c_fs_app to d5 phase (Plan 367 F1)"
```

---

# Phase F2: c_json_app — JSON 配置处理器消费者

> **⛔ 阻塞（2026-07-22 审计 + 2026-07-23 FU-1 后复核）**：
>
> 1. **解析层阻塞（FU-1 已修）**：原 VM parser 不支持顶层
>    `Type.method(self Type)` 声明 + `T?` 返回 + `[T]` 泛型 + 裸 `type X`。
>    **FU-1 已修复全部 4 个解析缺口**，`use auto.json`（及 http/net/async/llm）
>    现在能成功加载。
>
> 2. **运行时层阻塞（FU-1 未解，新发现）**：即便能 `use auto.json`，VM 的
>    json 运行时**只是占位实现**——`json.parse(s)` 原样返回输入串（不解析），
>    每个 `json.get`/`as_string`/`as_int` 都重新解析文本串。VM 里**没有真正的
>    `JsonValue`/`serde_json::Value`**，方法形式 `v.get(k).as_string()` 完全
>    没接线（会 dispatch 成字符串方法）。而 a2r 用真正的 `serde_json::Value`。
>    实测：`json.as_int(json.get(v,"n"))` 对 `{"n":42}` 在 VM 返回 **0**（get 返回
>    带引号的 `"42"`，as_int 期望裸数字），在 a2r 返回 **42**——**三端发散**。
>    `json.type_of` 在 a2r 无映射；`json.len`/`has_key` 在 a2r 会生成不存在的
>    `len_str`/`has_key_str`（编译失败）。
>
>    **结论**：靠 `auto.json` 做三方一致目前**不可行**——VM 运行时是占位
>    string-roundtrip，与 a2r/native 的真 Value 语义根本不一致。这不是调用约定
>    问题，是 VM json shim 实现不完整（需把 `json.parse` 改成真正返回
>    `serde_json::Value` 并接线方法 dispatch，或改 VM 用 opaque handle）。
>
>    **可选出路**（本计划不做）：照搬 `parity/libs/serde_json` 的纯 Auto 手写
>    parser（不调 `auto.json`）——但那是"实现者模式"，失去 F2"消费者"意义。
>
>    **F2 状态：搁置**，等 VM json 运行时补齐（独立工作）。FU-1 已让 http/net
>    等模块可加载，对后续 http 消费者（如果 VM http 运行时是完整的）仍有价值。

**目标：** 验证 Auto 通过 `auto.json` 调 serde_json 能力（parse/encode/查询）的应用，与 Rust serde_json 行为三方一致。

**出口条件：** `auto-parity run c_json_app` 报 100% consistent。

---

### Task F2.1: Auto 应用 + 测试 + Rust oracle + parity

结构同 F1。Auto 侧用 `use auto.json: encode, parse`，定义消费者函数：

- `encode_and_pretty(value) str`：encode 一个 object 再返回（验证序列化）
- `parse_get_field(json_str, key) str`：parse JSON 再取字段
- `parse_type(json_str) str`：parse 再返回 type（object/array/string/number）
- `parse_keys(json_str) str`：parse 再返回 keys

测试用例（~14 个）：encode 基础 object/array、parse 取嵌套字段、parse array 元素、parse 数字/字符串/bool/null、keys 列表、type 判断。

Rust oracle 用 `serde_json::Value` + `serde_json::to_string` / `from_str` 实现等价，测试名镜像。

**关键 API 核实**：`auto.json.parse(s)` 返回 `JsonValue?`（Option）；`JsonValue.get(key)` 返回 `JsonValue?`；`as_string()` 返回 str。Rust 侧用 `serde_json::from_str::<Value>` + `value.get(key)` + `value.as_str()`。

- [ ] **Step 1-4: 同 F1 流程**（Auto 代码 → VM 验证 → Rust oracle → parity 三向 → phase 表加 d5 的 c_json_app → Commit）

出口：`Consistency: 14/14 (100.0%)`。

---

# Phase F3: c_env_app — 环境变量消费者

**目标：** 验证 Auto 通过 `auto.env` 调 std::env 的应用，与 Rust std::env 行为一致。

### Task F3.1: Auto + Rust oracle + parity

Auto 侧 `use auto.env: get, set, get_or`：
- `set_and_get(key, val) str`：set 再 get 回
- `get_or_default(key, default) str`：get 不存在则返回 default
- `get_missing(key) str`：get 不存在的 key（返回空串 vs panic？需核实 VM 行为）

测试（~10 个）：set+get 往返、get_or 默认值、get 不存在、覆盖 set。

**确定性注意**：env 是进程级共享，三方在独立进程跑。set 的 env 只影响当前进程，所以 VM 进程 set 的值 VM 进程能读到，但 a2r/rust 进程读不到。**因此 env 测试只能测"本进程内 set→get 往返"**（每方独立 set 一个唯一 key 再 get 回），不能跨进程对比同一个 env 值。三方对比的是"set(k,v) 后 get(k) == v"这个**行为模式**是否一致。

Rust oracle 用 `std::env::set_var` / `get_var`，同样的 set→get 往返模式。

- [ ] **Step 1-4: 同 F1 流程**，出口 `Consistency: 10/10`。

---

# Phase F4: c_process_app — CLI 参数消费者

**目标：** 验证 Auto 通过 `auto.process.args` 读命令行参数的应用，与 Rust std::env::args 行为一致。

### Task F4.1: Auto + Rust oracle + parity

**确定性挑战**：`process.args()` 返回进程的 argv。parity runner 跑 VM/a2r/rust 时各是独立进程，argv 不同。**解决**：测试不直接断言 argv 的绝对值，而是断言"args() 返回的串包含/不包含某子串"或"args 串长度 >= 某值"（程序名总在）。更稳妥：让被测函数接受一个 `args_str` 参数（模拟 argv），测试用例传入固定串——这样测的是"解析逻辑"而非"真 argv"。

Auto 侧：
- `parse_count(args_str) int`：数 args 串里的参数个数（按空格分）
- `parse_nth(args_str, n) str`：取第 n 个参数

测试（~8 个）：固定 args_str 串解析。Rust oracle 用 `args_str.split_whitespace()` 等价。

- [ ] **Step 1-4: 同 F1 流程**，出口 `Consistency: 8/8`。

---

# Phase F5: c_text_app — 文本批处理消费者

**目标：** 验证 Auto 组合 `auto.file`（读写）+ 文本处理（复用 parity/libs/string_utils 的纯逻辑）的批处理应用。

### Task F5.1: Auto + Rust oracle + parity

Auto 侧：读文件 → 文本变换（trim/lower/replace，复用 string_utils 模式）→ 写文件 → 读回。
- `transform_file(path, from, to) str`：读文件、replace(from,to)、写回、读回返回

测试（~12 个）：读→replace→写→读、读→trim→写→读、文件不存在错误路径。

Rust oracle 用 `std::fs::read_to_string` + `str::replace` + `std::fs::write`。

- [ ] **Step 1-4: 同 F1 流程**，出口 `Consistency: 12/12`。

---

# Phase F6: c_http_get + c_http_post — HTTP 客户端消费者（依赖 E4）

**⚠️ 前置：** Phase 359 Task E4（修 DIV-HTTP-LANG-1，让 `use auto.http` 可解析）必须先完成。否则本 Phase 全部无法启动。

**⚠️ 前置 2：** 需给 `auto-parity` runner 加 mock-server setup/teardown hook（`http_client_sync` 骨架已设计但未实现）。

---

### Task F6.1: 实现 runner mock-server setup/teardown hook

**Files:**
- Modify: `parity/crates/auto-parity/src/main.rs`（run_library）
- Modify: `parity/crates/auto-parity/src/runner.rs`（可选）

- [ ] **Step 1: 设计 hook 机制**

在 `run_library`（`main.rs`）里，`build_comparison_report` 前检查 `libs/<name>/mock-server/` 是否存在。若存在，`cargo run` 它（后台），等端口（127.0.0.1:18081）可用，跑完三向后 kill。

- [ ] **Step 2: 实现 spawn + wait-for-port + kill**

约 20-30 行 Rust：`std::process::Command::new("cargo").args(["run","--manifest-path", mock_cargo]).spawn()` → poll `TcpStream::connect` 直到成功或超时 → 跑 parity → `child.kill()`。

- [ ] **Step 3: 验证 hook（用 http_client_sync 骨架）**

Run: `cd parity && cargo run -p auto-parity -- run http_client_sync`
Expected: mock-server 自动启停，三向跑通（若 E4 已修）。

- [ ] **Step 4: Commit**

```bash
git commit -m "feat(auto-parity): mock-server setup/teardown hook (Plan 367 F6)"
```

---

### Task F6.2: c_http_get 消费者

**Files:**
- Create: `parity/libs/c_http_get/`（auto + tests/auto + tests/rust）

Auto 侧 `use auto.http: get`：
- `fetch_body(url) str`：GET url，返回 response body
- `fetch_status(url) int`：GET url，返回 status code（需核实 `auto.http` 返回结构）

mock-server（复用 `http_client_sync/mock-server/` 或新建，监听 18081，回固定 body `{"ok":true}` + 200）。

测试（~10 个）：GET 固定 URL 返回固定 body、status 200、不同 path 返回不同固定响应。

Rust oracle 用 reqwest（与 VM 一致）或 ureq（与 a2r-std 一致）——**这里需决策策略 A/B**（见设计文档 §3.4）。建议先用 ureq（与 a2r-std 一致），VM 侧若 reqwest 行为在语义层（status/body）一致即可。

- [ ] **Step 1-4: 同 F1 流程** + mock-server，出口 `Consistency: 10/10`。

---

### Task F6.3: c_http_post 消费者

同 F6.2，测 POST：发 JSON body → mock echo → 断言响应 body。

Auto 侧 `use auto.http: post`。测试（~10 个）：POST 固定 body 返回 echo、POST JSON 解析、不同 body。

- [ ] **Step 1-4: 同 F1 流程**，出口 `Consistency: 10/10`。

---

# Phase F7: c_wget + c_crawler — 组合应用消费者（依赖 F6）

### Task F7.1: c_wget（简易下载器）

Auto 侧组合 `auto.http.get`（或 download）+ `auto.file.write_bytes`：GET URL → 存文件 → 读回断言内容。

mock-server 提供固定文件内容（如一段文本/二进制）。测试（~10 个）：下载文本文件、下载二进制、404 错误路径。

- [ ] **Step 1-4: 同 F1 流程** + mock-server，出口 `Consistency: 10/10`。

---

### Task F7.2: c_crawler（简易爬虫）

Auto 侧组合 `auto.http.get` + 文本解析（提取 `<a href>` 链接，或 JSON 字段）：GET → 提取链接 → 对固定链接再 GET → 断言。

mock-server 提供固定 HTML/JSON 页面（含已知链接）。测试（~10 个）：提取主页面链接数、跟踪一个链接的内容、空页面。

**注意**：爬虫用例较复杂，确保 mock-server 提供确定的多页面响应（不同 path 回不同固定内容）。

- [ ] **Step 1-4: 同 F1 流程** + mock-server，出口 `Consistency: 10/10`。

---

## Layer 3（路线图，不在本计划范围）

以下消费者场景因底层模块缺失，列为远期路线图（见 `docs/design/consumer-parity-strategy.md` §2.2 Layer 3）：

- **c_sqlite_client**：需先实现 `auto.sqlite` 模块（VM + a2r 双端），调 rusqlite 或重实现查询层
- **c_redis_client**：需先实现 `auto.redis` 模块
- **c_http_server**：需 a2r_std 补 server 模块（或 axum 生成）

---

## 全局验收

- [x] **V1**: Layer 1 全部 100% consistent（F2 json 阻塞除外）。实际 +29 新 L1
      用例（241→270）：c_fs_app 7 + c_env_app 7 + c_process_app 9 + c_text_app 6。
      （预估 ~60 偏高——每个 lib 的用例数按"可靠三端一致的用例"收敛，未凑数。）
      `auto-parity phase d5` 报 4/4 libs 全 100%。
- [ ] **V2**: Layer 2（F6-F7）全部 100% consistent（需 E4 完成），累计 +40（→341）。
- [ ] **V3**: mock-server runner hook 实现，http_client_sync 解锁（若 E4 已修）。
- [x] **V4**: parity dashboard 配置已就绪——d5 阶段已加入 `main.rs` 的 report
      phases 数组，运行 `auto-parity report` 会包含所有 c_* 消费者库。
      ⚠️ 注意：dashboard HTML **尚未实际重新生成**（`report` 命令会全量重跑 13
      个 lib 的三向对比，单次约 10+ 分钟，会话中未跑完）。`docs/parity-dashboard.html`
      仍是旧版（不含 c_* 库）。需要时手动跑：
      `cd parity && cargo run -p auto-parity -- --auto-binary ../target/release/auto.exe report --output docs/parity-dashboard.html`
- [ ] **V5**: 消费者用例覆盖 fs/json/env/process/text/http 六大能力域。
      当前覆盖 fs（F1）/env（F3）/process（F4）/text（F5）= 4/6；json（F2）阻塞于
      VM stdlib json.at 解析；http（Layer 2）阻塞于 E4。

---

## 风险与对策

- **R1 VM/a2r HTTP 底层不一致（reqwest vs ureq）**：Layer 2 限语义层（status/body），或统一为 reqwest（策略 A，远期）。
- **R2 env 测试跨进程不可比**：F3 改测"本进程 set→get 往返"行为模式。
- **R3 process.args 跨进程不可比**：F4 改测"解析固定 args_str"逻辑。
- **R4 临时文件残留**：每方测试开头清理 + 结尾清理临时目录。
- **R5 E4 未修导致 Layer 2 全部阻塞**：Layer 1（F1-F5）独立可做，不阻塞；Layer 2 等 E4。

---

## 执行说明

本计划 Layer 1（F1-F5）可立即用 worktree 方式实施，无前置依赖。建议顺序 F1→F2→F3→F4→F5（每个独立 worktree，修完合并）。Layer 2（F6-F7）等 Phase 359 Task E4 完成后实施。

每个 Task 用 `superpowers:subagent-driven-development`：派 fresh subagent，先写 Auto 代码 + VM 验证，再写 Rust oracle，最后跑 parity 三向确认 100% consistent。

---

## 实施后审计与后续工作（2026-07-23）

Layer 1（F1/F3/F4/F5）实施完成后做了一次完整审计，确认了**未实现的 Phase**、
**以 workaround 方式实现的地方**，以及若干**超出本计划范围但阻塞后续 Phase 的
底层 bug**。全部固化在下面，按优先级排序为可追踪的后续工作项（FU = Follow-Up）。

### 当前进度快照

| Phase | 状态 | 用例数 | 说明 |
|-------|------|--------|------|
| F1 c_fs_app | ✅ 完成 | 7/7 | auto.fs 文件读写 |
| F2 c_json_app | ⛔ 阻塞 | — | VM parser bug（FU-1）|
| F3 c_env_app | ✅ 完成 | 7/7 | auto.env |
| F4 c_process_app | ✅ 完成 | 9/9 | CLI argv 解析（规避最多 VM bug）|
| F5 c_text_app | ✅ 完成 | 6/6 | auto.fs + 文本变换 |
| F6 c_http_get/post | ⛔ 未实现 | — | 同 FU-1 + mock hook（FU-4）|
| F7 c_wget/c_crawler | ⛔ 未实现 | — | 依赖 F6 |
| F6.1 mock-server hook | ⛔ 未实现 | — | runner.rs 无相关代码 |

**已完成**：4 lib / 29 用例，`auto-parity phase d5` 全 100% 一致，无回归
（string_utils 22/22 不变，332 个转译器测试通过）。
**未实现**：F2、F6、F7、F6.1。没有 `d6` phase 注册。

### 后续工作项（按投入产出比排序）

#### FU-1（高收益）修 parser 的 `Type.method` 顶层声明解析

- **现状**：`crates/auto-lang/src/parser.rs` 的 `fn_decl_stmt()` 在读完
  类型名（如 `JsonValue`）后直接 `expect(LParen)`，遇到 `.` 就报 E0007。
  `parse_name()` 也只读第一个标识符。词法确认：`JsonValue.as_int` 被切成
  3 个 token（`Ident`/`Dot`/`Ident`），`.` 不会被词法器融合。
- **影响范围**（实测，5 个 stdlib 模块全部 `use` 失败）：
  - `http.at`（35 处 self-Type 声明）= **DIV-HTTP-LANG-1**（已记录在
    `parity/docs/known-divergences.md`，open/L3）
  - `json.at`（12 处）= **F2 阻塞**
  - `net.at`（12 处）、`async.at`（4 处，泛型叠加更复杂）、`llm.at`（2 处）
- **根因纠正**：之前以为是 `self Type` 参数的问题；实测确认是**点分
  `Type.method` 声明名**不可解析（`pub fn Foo.bar(x int) int` 同样报
  E0001）。`self` 作为参数名本身能解析。
- **关键调研结论**：`.at` 里的 `Type.method` 声明**不是纯文档**——它是
  load-bearing 的。模块加载（`compile.rs` `parse_module_to_type_store`）
  必须把它解析成 `Fn { name, parent: Some(Type), ... }`，下游
  `register_fn_decl`（`types.rs`）和 `enrich_fn_return_types_from_type_store`
  （`codegen.rs:10358`）读 `fn_decl.parent` 做返回类型推断。因此不能"跳过"，
  必须真正解析成正确的 `Fn` 形状。好消息：这与 `ext Type { fn method(self) }`
  块（`parse_ext_stmt`，已有且经过验证）语义完全等价——后者把 `parent_name`
  传给 `fn_decl_stmt`，触发 `self` 自动定义 + `Fn.parent` 设置。带点形式只是
  缺了花括号块。
- **实施方案（Option A，~15 行，单文件 `parser.rs`）**：
  1. 新增 helper `parse_fn_name()`（在 `parse_name` 之后）：先 `parse_name()`
     读第一个标识符；若下一个 token 是 `Dot` 且再下一个是 `Ident`，则消费掉
     `.`，再 `parse_name()` 读方法名，返回 `(method_name, Some(parent_type))`；
     否则返回 `(name, None)`。对 `fn.c`/`fn.vm` 向后兼容路径无冲突（那条路径
     在解析名字之前已消费掉点）。
  2. 在 `fn_decl_stmt()` 和 `fn_decl_stmt_with_annotations()` 里，把
     `let name = self.parse_name()?;` 换成 `let (name, dotted_parent) =
     self.parse_fn_name()?;`，当 `dotted_parent` 非空时用它覆盖 `parent_name`。
     下游代码（`parent_name.is_empty()` 判断、`Fn.parent`、
     `unique_name = "Type.method"`）无需改动——已经按 parent 工作。
- **风险**：极低。带点形式当前是硬解析错误，没有合法程序在顶层用它。
  helper 只在 `fn`/`pub fn` 后 `Ident . Ident` 时激活；裸名和 `ext` 块路径不受
  影响。泛型参数 `<T>` 的解析在名字之后、点已消费，也不受影响。
- **验证**：
  - `use auto.json` + 调 `json.parse`/`JsonValue` 方法能跑通（之前 E0007）。
  - 同样验证 `use auto.http`、`use auto.net`。
  - `cargo test -p auto-lang --lib` 全过；现有 332 个转译器测试无回归。
  - 解锁后即可做 F2（c_json_app）。
- **修复后解锁**：F2（json）+ F6（http）+ F7（wget/crawler）+ DIV-HTTP-LANG-1。
  **一处修复解锁整个 Layer 2**。

#### FU-2（中收益）修 a2r 的 StrSlice 类型追踪

一次性修掉 **3 个相关 workaround**（W1/W2/W3，见下表），让消费者代码写得更自然
（不再需要"内联拼接"或"全用字面量"的写法规避）。
- **关键调研结论（比预想更简洁）**：真正的 bug **不在 `store()`**，而在
  `needs_as_str()`（`rust.rs`）。该函数对**所有** `StrSlice` 登记的 ident 返回
  `false`（当作 `&str`），但实际上只有 **`StrSlice` 参数**才是 `&str`；
  `var x str = ...` 的局部变量被渲染成 owned `String`（`rust_type_name` 把
  StrSlice → `"String"`）。所以局部 `str` 变量在 `&str` 使用点也需要
  `.as_str()` 借用。不需要动 `store()` 或 `infer_type_from_expr()`。
- **实施方案（Option C = A + B，共 ~16 行，单文件 `trans/rust.rs`）**：
  - **Option A（修 `needs_as_str`，~6 行）**：把 `Expr::Ident(name)` 分支从
    "StrSlice 登记就返回 false" 改成 "仅当 name 是当前函数的 `str` 参数
    （`current_fn_str_params.contains(name)`）才返回 false"。修掉 W1（`var x str`
    第二次用被 move → E0382）和"传 `str` 局部给 `&str` 参数"（E0308）。
  - **Option B（修用户函数调用的借用，~10 行）**：把 `needs_borrow_unknown_callee`
    （跨模块/未知被调的借用判断）从只认 `Expr::Ident` 扩展到也认产生 owned
    String 的表达式——对 `Arg::Pos(Expr::Bina(..))` 用现成的
    `expr_contains_string()`（`rust.rs:470`，已检测字符串拼接）判断。
    修掉 W2（`f(base+"/x")` 内联拼接给用户函数 → E0308）。
- **为什么选 A+B 而非改 `store()`**：改 `store()` 把 StrSlice 改登记为 StrOwned
  会影响 ~15 个其它查询点（`expr_contains_string`、借用路径等），爆炸半径更大。
  Option A 达到相同运行时效果（`.as_str()` 借用），但只改一个函数的一处判断，
  因为真正的恒等式"StrSlice 局部渲染成 owned String"已经成立——只有
  `needs_as_str` 的假设错了。
- **影响**：仅消费者代码可读性 + 解锁更自然的写法，不影响正确性。
- **验证**：把 `parity/libs/c_fs_app/auto/c_fs_app.at` 的 `mkdir_write_read`
  改回自然写法（`var fullpath str = dir + "/" + filename` 后两处用 `fullpath`），
  `auto-parity run c_fs_app` 仍 7/7；测试里直接传内联拼接
  `write_and_read(base + "/c.txt", "x")` 也能过。全量 15 个 parity lib 不回归
  （grep 各 `.at` 里 `var .* str = .* +` 找所有可简化的 workaround）。
- **W3**（env.* 未用 expr_as_str）顺手在 Option B 同批修掉（env.* 分支也借用）。

#### FU-3（低成本，纯文档）同步计划号 367→368

代码注释和 README 里有 **15 处 "Plan 367"** 残留（实施时计划还叫 367），重命名
为 368 后未同步。涉及：`parity/crates/auto-parity/src/main.rs`（注释）、4 个
README、`parity/.gitignore`、`crates/a2r-std/src/{fs.rs,env.rs,string_builder.rs}`
的 doc 注释、`crates/auto-lang/src/a2r_std.rs`。批量 sed 即可。

#### FU-4（中成本）F6.1 mock-server runner hook + Layer 2 实施

等 FU-1 完成后：实现 `runner.rs` 的 mock-server setup/teardown hook（约 20-30 行：
`cargo run` mock-server 后台进程 → poll `TcpStream::connect` 等端口 → 跑 parity →
`child.kill()`），然后按 F6.2/F6.3/F7.1/F7.2 实施 http 消费者库。

### 未修的底层 bug 清单（全部以 .at 源码侧 workaround 规避）

这些是实施中发现的 VM/a2r 已有 bug，**不在本计划范围内修**，但记录在此供
后续修复参考。每个 workaround 都在对应 lib 的源码注释 + README 里文档化。

#### a2r 转译器 bug（3 个，still-open）

| ID | Bug | 位置 | 现象 | workaround |
|----|-----|------|------|-----------|
| W1 | owned `str` 变量登记为 StrSlice | `rust.rs:6748-6753` | `var x str = base+"/x"` 第二次用 → E0382 moved value | c_fs_app/c_text_app：内联拼接直接传 `fs.*`，不存变量 |
| W2 | 用户函数的内联拼接参数不自动 `.as_str()` | `rust.rs:6402-6420` + `6028-6053` | `f(base+"/x")`（`fn f(s str)`）→ E0308 | c_fs_app 测试：拼接先存变量；c_env_app：全用字面量 |
| W3 | env.* 调用未用 expr_as_str（次要） | `rust.rs:3266-3283` | `env.set` 传拼接参数 → E0308 | c_env_app：全用字面量 key |

> 共同根因见 FU-2。

#### VM 运行时 bug（4 个，still-open，主要影响 c_process_app）

| ID | Bug | 现象 | workaround |
|----|-----|------|-----------|
| W4 | `str.split` 的 `[]str` 返回跨模块调用被破坏 | 被测函数内 split 在跨模块调用时返回空 | 逐字符状态机，只返回 int/str 原语 |
| W5 | `int.to(str)` 把字符码点拼成十进制数字串 | `char_at` 的 99 → "99" 而非 "c" | `StringBuilder.append_char(code)` + `.build()` |
| W6 | `for` 循环体内 `return` 的 codegen bug | 循环内 return 让调用方输出丢失 | `done`/`built` 标志，循环结束后再 return |
| W7 | `print("")` 吞掉前一行输出 | 空 print 后前一行内容消失 | 越界用 `"<none>"` 占位符 |

另有 W8（`return <方法调用>` 丢值，同 string_utils/Plan 359 C2 家族），c_text_app
通过"变换结果先存局部变量再 return"规避。

#### 已修的 bug（对照，确认仍在位）

这 7 处是实施时**真正修了**的（commit 61a0c03d / d37cfdc9 / 78c05d8c / 5c1222b6）：
- `a2r-std::fs::read_text/read_to_string` → 返回 `String`（`fs.rs:19,26`）
- `a2r-std::env::get` → 返回 `String`（`env.rs:10`）
- `StringBuilder::build(&self)` → 非消费（`string_builder.rs:57` + 嵌入副本 `a2r_std.rs:247`）
- `fs.*` obj.method 分支用 `expr_as_str` 借用（`rust.rs:3284-3397`）
- `env.*` 路由到 `a2r_std::env::*`（`rust.rs:3266-3283`）
- `Map.set→insert` 重写跳过 stdlib 模块接收者（`rust.rs:4565-4627`，guard 4573-4576）
- `str.lower()/upper()` → `to_lowercase/to_uppercase`（`rust.rs:4053` + `5173`）

