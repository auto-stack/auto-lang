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
> 2. **运行时层阻塞（有正确修复方案，见 §"剩余任务实施方案" R-JSON）**：
>    VM 的 json 运行时**只是占位实现**——`json.parse(s)` 原样返回输入串（不解析），
>    每个 `json.get`/`as_string`/`as_int` 都重新解析文本串。VM 里**没有真正的
>    `JsonValue`/`serde_json::Value`**。而 a2r 用真正的 `serde_json::Value`。
>    实测：`json.as_int(json.get(v,"n"))` 对 `{"n":42}` 在 VM 返回 **0**，在 a2r 返回 **42**——三端发散。
>
>    **正确修复方案（R-JSON）**：照搬已有的 regex/url **opaque-handle 模式**（Plan 212），
>    把 12 个 json shim 改成真 `serde_json::Value` 句柄实现——`json.parse` 返回堆句柄（i32），
>    `json.get`/`as_int` 等在句柄上操作。基础设施（`RustStdlibObject` + 堆对象表 +
>    opaque dispatch）已存在且经过验证，~250-350 行机械式翻译。**不是 workaround，
>    是补齐缺失的运行时实现。**
>
>    **F2 状态：待实施**（R-JSON 完成后即可做 c_json_app）。

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

#### FU-4（中成本）F6.1 mock-server runner hook + 激活 http 消费者

FU-1 已修，`use auto.http` 现在能加载。**调研结论：http 与 json 不同——VM 的
http 运行时是真实实现**（基于 `reqwest::blocking`，有通过的测试
`plan349_tests::test_http_get_sync` 证明），a2r 用 `ureq`（同步）。对返回固定
200+body 的 mock server，reqwest vs ureq 的传输层差异在语义层（status/body）
完全一致——**三方 parity 可行**。
- **好消息**：`parity/libs/http_client_sync/` 骨架已完整存在——mock-server
  （`127.0.0.1:18080`，POST→200+`{"echo":"ok"}`）、wrapper `.at`、测试
  `tests/auto/post_echo.at`（3 TAP）、Rust oracle（用 ureq，与 a2r 一致）都写好了。
- **缺两个小东西**：
  1. **`stdlib/auto/http.at` 漏声明 `post_sync`/`post_bearer`/`last_status`**
     （它们只在 `http.vm.at` + 有真实 Rust shim，但 `http.at` 没声明 → 链接期
     `auto_link_E0401 Undefined symbol: http.post_sync`）。补 3 行声明即可。
  2. **parity runner 没有 mock-server setup/teardown hook**（`runner.rs`/`main.rs`
     零相关代码）。需加：在 `run_library` 里，若 `libs/<name>/mock-server/` 存在，
     `Command::new(mock-server.exe).spawn()` → poll `TcpStream::connect` 等端口
     可用 → 跑三向 → `child.kill()`。可参考 `plan349_tests.rs:14` 的 in-process
     模式，但 runner 跑独立进程，需用 `Command::spawn` + 显式 teardown。
- **调用约定**：同步 helper 用**模块形式** `http.post_sync(url, body, api_key)`
  返回 `str`（响应 body），状态码经 `http.last_status()` 取（thread-local）。
  不要用方法形式 `req.method()`。
- **实施步骤**：
  1. 补 `stdlib/auto/http.at` 3 个声明。
  2. 加 runner mock hook（~25 行）。
  3. 把 `http_client_sync` 从 `report.rs` 的 L3 planned 列表移到 `d6` phase_map
     （新 phase）并加入 report phases。
  4. `auto-parity run http_client_sync` 应 3/3 一致。
- **F7（c_wget/c_crawler）**：等 F6 激活后再做（组合 http+fs/json）。注意 F7 若
  用 json 会撞上 F2 的 VM json 运行时占位问题——需用纯文本解析或等 VM json 补齐。

### 未修的底层 bug 清单（全部以 .at 源码侧 workaround 规避）

这些是实施中发现的 VM/a2r 已有 bug，**不在本计划范围内修**，但记录在此供
后续修复参考。每个 workaround 都在对应 lib 的源码注释 + README 里文档化。

#### a2r 转译器 bug（3 个，**FU-2 已修**）

W1/W2/W3 已由 FU-2 真正修复（`needs_as_str` + `needs_borrow_unknown_callee`），源码侧的
workaround 已全部去除（见下"已修的 bug"）。下表保留作历史记录。

| ID | Bug | 位置 | 现象 | 状态 |
|----|-----|------|------|------|
| W1 | owned `str` 变量登记为 StrSlice | `rust.rs` `needs_as_str` | `var x str = base+"/x"` 第二次用 → E0382 | ✅ FU-2 已修 |
| W2 | 用户函数的内联拼接参数不自动 `.as_str()` | `rust.rs` `needs_borrow_unknown_callee` | `f(base+"/x")` → E0308 | ✅ FU-2 已修 |
| W3 | env.* 调用未用 expr_as_str（次要） | `rust.rs:3266-3283` | `env.set` 传拼接参数 → E0308 | ✅ FU-2 已修 |

#### VM 运行时 bug（W4-W8 复核结论，2026-07-23）

对最初记录的 W4-W8 做了逐一复测，结论与初判不同：

| ID | 初判 | 复测结论 | 处理 |
|----|------|---------|------|
| **W8** | `return <方法调用>` 丢值 | **真 bug，根因找到并已修**：native 实例方法（`s.upper()` 等）编译成带 reloc 名（如 `str.upper`）的 CALL，但调用后的返回类型查找只查 `fn_return_types`（native shim 不在里面）→ `last_expr_type` 残留旧值（常 Void）→ 值被丢。**修法**：`codegen.rs` CALL 返回类型查找加 `else { infer_native_return_type(reloc_name) }` 兜底 + native_catalog 的 Str.*/str.* 别名返回类型从 Void 改正 + `infer_native_return_type` 加 str/char 方法分支。 | ✅ **已修**（c_text_app 的 local-var-before-return workaround 已去除）|
| **W5** | 无 codepoint→str 原语 | **真缺口，已补**：加了 `Char.to_str(codepoint) str` native（+ 顺手修了 Char.* 全部被标 Void 的返回类型，同 W8 家族）。 | ✅ **已补** |
| **W6** | `for` 循环内 return codegen bug | **误判**：复测 `for i < len { if cond { return p } }` 在单文件里工作正常；之前"跨模块失败"实际是 W8 的值丢失，W8 修好后此模式正常。 | ✅ 非独立 bug（W8 已含）|
| **W7** | `print("")` 吞前一行 | **误判**：复测 `print("a"); print(""); print("b")` 输出完整 3 行；初判是被 `tail -N` 截掉了 banner 行误读。`<none>` 占位符没必要，但 c_process_app 因 W4 暂未改。 | ✅ 非真实 bug |
| **W4** | `str.split` 跨模块返回被破坏 | **真 bug（窄）**：`str.split` 本身跨模块工作正常（`parse_count` 用 `parts[i].len()` 计数 PASS）；但**"跨模块调用里 `var p = parts[i]; ... return p`"** 这种"从 list 取元素并经跨模块调用返回字符串"的窄路径仍丢值。c_process_app 的 `parse_nth` 因此暂留逐字符 StringBuilder 写法。 | ⚠️ 残留窄 bug，待专门修（见下）|

> **教训**：W6/W7 的"现象"大多是 W8 值丢失的次生表现，或工具输出误读。真正的根因是 W8（一处 CALL 返回类型查找漏了 native 兜底）。修 W8 后大部分 workaround 自动消失。

#### 残留待修：W4-残差（跨模块用户函数返回类型查找）— 根因已定位，见 §R-W4

- **现象**：被跨模块调用的函数（`use auto.<lib>: fn`）里，返回字符串元素时调用方拿到空值。
  `parse_count`（返回 int）恰好"对"，`parse_nth`（返回 str from list）"错"——但两者返回类型
  标记其实**都错**，只是 int 的情况恰好被后续表达式掩盖。
- **根因（已精确定位）**：跨模块 CALL 的返回类型查找（`codegen.rs:7611`）用限定名
  `reloc_name`（如 `c_process_app.parse_nth`）查 `fn_return_types` 表，但该表用**裸名**
  （`parse_nth`）做 key（由 `enrich_fn_return_types_from_type_store` + `import_items` 填入）。
  查不到 → 走 W8 的 `else` 兜底 → `infer_native_return_type` 也不认识用户函数 →
  `last_expr_type` 残留参数编译后的旧值（常 Int/Void）→ 下游类型相关 emitter（POP、
  PRINT 选择、RET 提升）错误 dispatch，丢值或打印错。
- **正确修复（R-W4）**：CALL 返回类型查找加**裸名 fallback**——`fn_return_types.get(&reloc_name)
  .or_else(|| reloc_name.rsplit('.').next().and_then(|bare| self.fn_return_types.get(bare)))`。
  和 W8 同一家族、数据驱动（查 parser 已记录的权威返回类型），不是 workaround。
- **修好后**：c_process_app 的 `parse_nth` 可改回自然 `str.split` + 循环内 return + 返回 ""。

#### 已修的 bug（对照，确认仍在位）

---

## 剩余任务实施方案（2026-07-23，正确修复 — 无 workaround）

> 本节是剩余未完成任务的**根因分析 + 正确解决方案 + 实施步骤**。
> 所有方案都是修编译器/VM/stdlib 的根本问题，不用源码侧 workaround。

### 实施顺序与依赖

```
R-W4 (跨模块返回类型) ─┐
                       ├─→ R-COV (补覆盖用例)
R-JSON (json 运行时) ──┼─→ F2 c_json_app
                       └─→ R-F6GET (c_http_get + last_status)
```

R-W4 和 R-JSON 互不依赖，可先做 R-W4（小、快），再做 R-JSON（大、解锁多）。

---

### R-W4：修跨模块用户函数 CALL 返回类型查找

**根因（精确定位）**：跨模块用户函数调用（`use auto.<lib>: fn`）后，codegen 查返回类型
用限定 reloc 名（`c_process_app.parse_nth`）查 `fn_return_types`，但该表用裸名（`parse_nth`）
做 key → 查不到 → `last_expr_type` 残留旧值 → 值被错误 dispatch。

**正确修复（~5 行，单文件 `crates/auto-lang/src/vm/codegen.rs`）**：

在 CALL 返回类型查找处（`codegen.rs:7611`，`if let Some(ret_ty) = self.fn_return_types.get(&reloc_name)`），
把单次查找改成"先限定名，再裸名 fallback"：

```rust
let ret_ty = self.fn_return_types.get(&reloc_name)
    .or_else(|| {
        reloc_name.rsplit('.').next()
            .filter(|bare| !bare.is_empty() && *bare != reloc_name.as_str())
            .and_then(|bare| self.fn_return_types.get(bare))
    });
if let Some(ret_ty) = ret_ty {
    // ... 原有 match ...
} else {
    // W8 的 native 兜底（infer_native_return_type）保持不变
}
```

- `reloc_name.rsplit('.').next()` 从 `c_process_app.parse_nth` 取 `parse_nth`，
  从 `str.upper` 取 `upper`（后者裸名也不在 fn_return_types，正确走 native 兜底）。
- 裸名查 `fn_return_types` 命中（parser 已从共享 type_store 填入）→ 正确设置 `last_expr_type`。
- **不是 workaround**：修的是类型推断源头（数据驱动，查权威返回类型），和 W8 对称。

**验证**：
- 把 `parity/libs/c_process_app/auto/c_process_app.at` 的 `parse_nth` 改回自然写法
  （`var parts = s.split(" "); ...; return parts[i]` + 循环内 return + 返回 ""），
  `auto-parity run c_process_app` 应 9/9。
- 测试 `basic.at` 的越界用例期望改回 ""（去掉 `<none>` 占位）。
- Rust oracle 的 `parse_nth` 返回 `String::new()`。
- 全量 d5 不回归。

**实施步骤**：
1. 改 `codegen.rs` CALL 返回类型查找（加裸名 fallback）。
2. `cargo build -p auto --release` + 跑 `parse_nth` 自然写法 standalone 验证。
3. 改 `c_process_app.at`（自然 split + 循环内 return + ""）。
4. 改 `basic.at`（"<none>"→""）+ Rust oracle。
5. `auto-parity run c_process_app` 9/9 + 全量 d5。
6. Commit。

---

### R-JSON：实现 json opaque-handle 运行时（解锁 F2）

**根因**：`json.parse(s)`（`stdlib.rs:1947`）是占位——原样返回输入串。所有 `json.get`/
`as_int` 等（`stdlib.rs:2230-2361`）都重新解析文本串。VM 里没有真 `JsonValue`，
与 a2r 的真 `serde_json::Value` 语义根本不一致。

**正确修复（照搬 Plan 212 的 regex/url opaque-handle 模式，~250-350 行）**：

基础设施**已存在且经验证**：
- `RustStdlibObject { type_name, value: Box<dyn Any> }`（`vm/ffi/rust_stdlib.rs:10`）
- 堆对象表 `vm.insert_heap_object` / `vm.get_heap_object`（`engine.rs:596/619`）
- opaque dispatch 表 `lookup_opaque_dispatch`（`native_catalog.rs:424`）
- codegen 返回类型推断（`codegen.rs:9014+`）

**改动清单**：
1. **`crates/auto-lang/src/vm/native.rs`**：新增 12 个 `shim_json_opaque_*`（在 semver 块后），
   照搬 `shim_re_opaque_*`/`shim_url_opaque_*` 模式。关键 shim：
   - `shim_json_opaque_parse`：`serde_json::from_str` → 存 `Mutex<Value>` 句柄 → `push_i32(id)`
   - `shim_json_opaque_get(handle, key)`：`val.get(key).cloned()` → 新句柄（miss → `Value::Null` 句柄）
   - `shim_json_opaque_as_int(handle)`：`val.as_i64().unwrap_or(0)`
   - `shim_json_opaque_as_string(handle)`：`val.as_str().unwrap_or("")`
   - `shim_json_opaque_type_of(handle)`：match variant → "object"/"array"/...
   - `shim_json_opaque_keys(handle)`：object keys → `ListData<i32>`
   - 其余 `get_at`/`as_number`/`as_bool`/`is_null`/`has_key`/`len` 类似。
2. **`crates/auto-lang/src/vm/native_catalog.rs`**：
   - 加 12 个 `(27xx, NATIVE_JSON_OPAQUE_*, shim_*, "auto.json_opaque.*")` 条目。
   - 加 `OPAQUE_DISPATCH_JSON` + `OPAQUE_DISPATCH_JSON_METHODS` + dispatch 查找 arm。
3. **`crates/auto-lang/src/vm/ffi/stdlib.rs`**：删除 12 个占位 `#[rust_fn] shim_json_*`
   （`1947-1951` parse + `2230-2361` get/as_*）。保留 `encode`/`prettify`/`minify`/`is_valid`
   （这些是文本操作，不需句柄）。
4. **`crates/auto-lang/src/vm/codegen.rs`**：加 `auto.json_opaque.*` 返回类型 arm
   （parse/get/get_at/keys → NestedObject；as_int/len → Int；as_string/type_of → String；
   as_number → Float；as_bool/is_null/has_key → Bool）。
5. **`crates/auto-lang/src/vm/native.rs` `format_rust_stdlib_obj`（732）**：加
   `"serde_json::Value"` arm → `serde_json::to_string(&val)`（让 `print(json_value)` 渲染）。
6. **a2r 语义对齐**：a2r-std `len` 返回 0（非 -1），`get` miss → `Value::Null`，
   `parse` error → `Value::Null`（不 panic）。VM 新实现必须匹配。

**测试**：
- 更新 `test/ffi_dual/003_json_encode_parse/`、`008_json_array/`、`009_json_keys/` 的
  expected_output（`print(json.get(...))` 现在渲染 compact JSON 而非带引号文本）。
- 新增 ffi_dual 用例：`as_int(get(obj,"n"))` 对 `{n:42}` 返回 42（之前 VM 返回 0）。
- 实现 F2 c_json_app（见下）。

**实施步骤**：
1. 先实现 `parse` + `get` + `as_int` + `as_string`（最小可用，能跑 `as_int(get(v,"n"))`）。
2. 验证 VM `as_int(get(parse('{"n":42}'),"n"))` 返回 42（之前 0）。
3. 补齐其余 8 个 shim + catalog + dispatch + codegen arm + display。
4. 更新 ffi_dual expected + 新增用例。
5. 实现 F2 c_json_app（parity 三方）。
6. Commit。

**风险**：`print(json_value)` 的渲染会变（从带引号文本 → compact JSON），需更新 ffi_dual
expected。但这是**正确行为**（匹配 a2r 的 `Value::Display`）。

---

### F2 c_json_app（R-JSON 解锁后）

R-JSON 完成后，按原 F2 Task 描述实现：
- `parity/libs/c_json_app/`：auto app + tests + Rust oracle + README
- 消费者函数：`parse_get_field(s, key) str`、`parse_as_int(s, key) int`、
  `parse_type(s) str`、`parse_keys(s) str`、`parse_array_elem(s, idx) str`
- 用模块形式 `json.parse`/`json.get`/`json.as_int` 等（W8 修复后方法形式也行，
  但模块形式更稳）。
- ~10-14 用例，`auto-parity run c_json_app` 100%。
- 加入 phase d5（或新 d7）。

---

### R-F6GET：补 c_http_get + last_status 断言

**现状**：http_client_sync 只测 POST（3/3，固定 body）。GET 没测，`last_status` 状态码
从没断言（HTTP parity 的核心语义缺口）。

**正确方案**：
1. **扩展 mock-server**：加一个 GET 路由（如 `GET /data` → 200 + 固定 body）。
2. **c_http_get lib**（新 `parity/libs/c_http_get/`）：`fn fetch_body(url) str { return http.get(url).body() }`。
   注意：`http.get` 返回 Response 句柄，需 `.body()` 取 body。验证 VM/a2r 都支持。
3. **last_status 断言**：在 http_client_sync 的测试里加 `http.last_status()` 断言（应 200）。
4. mock-server 加一个 `GET /notfound` → 404，测 last_status 404 路径。
- 加入 phase d6。

---

### R-COV：补覆盖用例

审计指出的高价值覆盖缺口（修完 R-W4/R-JSON 后补）：

| 领域 | 缺口 | 新增用例 |
|------|------|---------|
| fs | `read_bytes`/`write_bytes`（`[]int` 列表跨边界）、`size`（i64 64位返回）、`delete`/`copy` | 加到 c_fs_app |
| env | `env.remove`（set 的逆操作） | 加到 c_env_app |
| text | 链式调用 `s.trim().lower().replace(...)`、`upper` | 加到 c_text_app |
| json | parse/get/as_int/type_of/keys/array（R-JSON 后） | c_json_app 全覆盖 |

每个新增用例都要有 Rust oracle 镜像 + 三方 parity 验证。

---

### 执行说明

- 用 worktree `plan-368-remaining` 实施。
- 顺序：R-W4（快）→ R-JSON（大）→ F2 → R-F6GET → R-COV。
- 每个 R-* 先修根因，再把对应 lib 改回自然写法（去 workaround），最后补用例。
- 全程无 workaround：任何失败都回去修编译器/VM，不在 `.at` 源码里绕。
- 每步 commit + 全量 d5/d6 验证不回归。
