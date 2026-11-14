# ACE 标准库领域迁移差额与发行接管就绪合同

> **Status**: active (PLAN-948 M2-A 交付，2026-11-15)
> **依据**：[Design54](../../../design/54-stdlib-semantics-and-ace-integration.md)、[Design55](../../../design/55-ace-m2-complete-language-and-product-replacement.md)、[Design56](../../../design/56-ace-m2-ffi-and-native-stdlib.md)、[backend-provider-coverage](../../../design/stdlib/backend-provider-coverage.md)、[native-platform-io](../../../design/stdlib/native-platform-io.md)、[m2-denominator](../../ace-migration/m2-denominator.md)。
> **口径**：本文件只记录已核实来源、差额映射、owner 边界、平台/覆盖/加载分账与发行阻塞；**不将未交付或未运行的能力标为已实现或已通过**。

## 1. 来源与 M1-TAG 门链

- **M1-TAG**：`v0.6-ACE-M1`（tag `acc3eb5b7`，指向 mainline commit `b923362f4`，PLAN-1025 归档收据 `1006 PASS / 0 FAIL / 0 UNOBSERVED`），已核实为 `master`（`d2e7d4ab6`）祖先。
- **消费归档输入**：`943`（操作合同 17 对）、`944`（装配模型）、`945`（`std.math`/`std.text` 四端）、`946`（`std.list` 5/6 路径，AV/C 与 AV/Rust pending）、`947`（`console/1` AV+mn，`File`/`Task`/`Stream` 预览）、`959`（M2 分母）、`1039`/`1040`（Win/Linux 无 libc 原语）、`1041`（`platform/io` stdio，AV/C 与 AV/Rust pending `967`）、`1042`（Win A2PY 就绪）、`1043`（`.at.{at,c,rs,py}` 后缀装配与最小 `#in sys is current`）。
- **分母增量（959 冻结 `f5773081e` → `d2e7d4ab6`）**：`stdlib/` 新增 9 个 `.at` 文件（`stdlib/ace/internal/platform_mem.at`、`stdlib/ace/std/{list,math,text}.at`、`stdlib/auto/platform/io.{at,at.at,c.at,rs.at,py.at}`，188→197）及 `compatibility.toml` 同步；959 原有行不修改，新增义务以子行并入。

## 2. 代表性真实消费者追踪与已知差额

| 域 / 代表性输入 | 实测观察与语义边界 | 状态 | 后继 Owner + AC | 独立 Oracle |
|---|---|---|---|---|
| **JSON** (`test/vm/07_objects/007_json/json.at`、`test/vm/18_ffi/038_json_encode_parse`、`stdlib/auto/json.at`) | `007_json` 为对象数组字面量而非 JSON 解析；`json.at` 与 `038` 存在命名漂移（`JsonValue.type` vs `Json.type_of`）；`json.rs.at` `parse` 已知坏（P6）；C 目标 `unsupported`；typed `decode[T]` 类型擦除未实现；ACE 零 conformance 用例 | `missing` (typed/ACE) / `unknown` (动态执行) | `975` AC-02/07/08 | `007_json` (`ClearDiagnosticInformation`) + `038` 独立 JSON 值 + bad-JSON `null`/error 负控 |
| **HTTP / Net** (`stdlib/auto/{http,net,sse}.at`、`test/a2r/30_plan724/001_http_client_sync`、`http-host-port.md`) | `http.vm.at` 仅 VM 绑定，无 `.rs.at`/`.c.at`；`HTTPStream.iter() -> int` 与 `for c in s.iter()` 类型待核；`last_status()` 为全局可变状态；有服务端 host-port 规范但缺客户端规范 | `unknown` (跨端执行) / `missing` (客户端 port/cancel/timeout) | `974` AC-01..06、`983` (Web) | loopback stub 独立 stdout/status/EOF/cancel/disconnect/多会话隔离（禁单用 `.expected.rs`） |
| **Async / Task / Channel** (`stdlib/auto/async.at`、`async.vm.at`) | `spawn` 返回 `void`（无句柄/join/cancel）；`Sender`/`Receiver` 缺 `close`/`drop`/断开状态与 backpressure 契约；`try_recv` 混淆 `0` 与空 | `missing` (生命周期/语义) | `968` T-02..08、`973` AC-08 | `task_count==0`、send-after-close、`0` vs empty 负控 |
| **UI** (`auto-ui/examples/ui/002-counter/src/front/app.at`、`stdlib/auto/{ui,widget,app}.at`) | `002-counter` 无邻接 expected；`ui.at`/`widget.at` 与 `auto-ui` loader 合同及 Vue/VM 双端 parity 未在本包实跑 | `unknown` / `missing` (oracle) | `980` (UI)、`981` (AGE)、`971` | 交互状态快照与双端截图/树 parity（不以 console 接入冒 UI 迁移） |
| **其余域** (`clock`/`random`/`fs`/`env`/`process`/`hash`/`regex`/`sqlite`/`redis`/`storage`/`test`) | `regex`/`sched`/`test`/`prelude`/`url`/`llm` 与 `encoding`/`iter`/`platform` 归属补齐：`hash`/`regex`→`972`、`fs`/`env`/`process`/`time`/`rand`/`storage`/`sched`→`973`、`url`/`llm`→`974`、`encoding`/`sqlite`/`redis`→`975`、`test`→`971`；`storage` 仅记需求不冻结新接口 | `missing` (ACE 完整域) | `971..975`、`967` T-15 | 假时钟、temp-dir 原子读写、SQLite 内存值+事务、公开标准向量 |

## 3. FFI、平台 I/O、后端覆盖与加载分账（Design56 + 1039–1043）

- **16 项 Design56 父义务（`FFI-M2-01..08`、`LIB-M2-01..08`）**：
  - 已交付有限切片：`FFI-M2-01` F0 单 `&str` `use.rs`（`967`）；`LIB-M2-01` 17 对合同（`943`）；`LIB-M2-02/03` `.{at,c,rs,py}.at` 后缀与最小 `#in sys is current`（`1043`）；`LIB-M2-05` Win32/Linux syscall 原语（`1039`/`1040`）与 stdio（`1041`）；`LIB-M2-07/08` `platform/io` 默认无 libc 实现与 5B NUL-safe 字符串切片（`1041`/`1042`）。
  - 仍阻塞/缺失：`binding.rs:825-831` 仍拒绝项目级 `use.c`/`use.rs`（阻塞 `1041`/`946` 的 `AV/C` 与 `AV/Rust`）；C 传递 include/typedef/回调、Rust 泛型/Drop/bidir/cdylib compat、async FFI port 归 `967` T-09..15；`sys.arch`/`env`/`ABI`（`source.cte.in-name`）与未选平台模块 read-set 缓存归 `978`/`967`/`966` RV-01。
- **六路径 + 旧 A2C 覆盖口径**：新增公共能力须逐操作追踪 `AV/native`、`AV/C`、`AV/Rust`、`AC`、`A2R`、`A2PY` 及旧 `A2C` × `Windows/Linux`；`a2r-std` 隐含 `std::io`/libc 保持具名缺口；仅 emit、仅文件存在或 WSL2/Windows 单端证据不得提升他端状态。

## 4. CTE、CAP 与发行接管阻塞（V-04）

在以下硬阻塞清除并完成洁净仓外实跑前，**禁止签署发行通过**：

1. **开发仓绝对路径与本机耦合**：`crates/auto-lang-core/src/stdlib_assembly/loader.rs:68,78` 仍含 `env!("CARGO_MANIFEST_DIR")` 回退；`~/.auto/libs/stdlib` 为指向活仓的 Junction；`PATH` 上 `auto` 指向陈旧 `auto-man/target/debug/auto.exe` 且 `auto-lang/target/{debug,release}/auto.exe` 缺失（归 `978`/`986`/`OS064`）。
2. **跨仓安装器漂移**：`web-install.py` 滞后 `man-install.py`（`cb4ab3dd`）；`auto-musk/install.sh` 无 manifest/hash/回滚；`auto-ui` 两冻结 SDK 钉不一致（`983`/`981`/`OS064`）。
3. **未闭合的 CTE 与 CAP 前置**：`sys.arch`/`env`/`ABI` 缺失、read-set 缓存未完；`experimental/ace/crates/*/src` 中 `ContextProfile`/`Capability` 求解器零实现，Design46 §10 Gate A（`978`+`985`）与 Gate B（`980`/`830`）未闭。
4. **未执行场景（全部 `pending`）**：洁净 `SCRUB` 仓外 CWD 实跑、`man-install.py --install-root` clean install/升级/回滚、安装目录重定位、`web-install.py` 主题校验实跑、完整 CAP profile 矩阵（交 `978`/`986`/`983`/Design46 后继执行）。
