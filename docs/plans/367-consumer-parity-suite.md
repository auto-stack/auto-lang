# Plan 367 — Consumer-Mode Parity Suite Implementation Plan

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
- [x] **V4**: parity dashboard 含所有 c_* 消费者库（d5 阶段已加入 report phases；
      dashboard 重新生成）。
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
