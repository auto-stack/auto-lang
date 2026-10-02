---
plan_id: PLAN-727
status: drafting
feature_name: http-file-transfer-lifecycle
author: [agent]
created_at: 2026-10-02
updated_at: 2026-10-02

plan_revision: 1
supersedes_spec_components:
  - docs/specs/stdlib/project.md
  - docs/specs/stdlib/design/backend-assembly.md
  - docs/specs/stdlib/design/http-stream-lifecycle.md
  - docs/specs/a2r-std/project.md
  - docs/specs/a2r-std/design/http-client-runtime.md
  - docs/specs/auto-lang/trans/design/http-client-lowering.md
  - docs/specs/auto-lang/runtime/design/networking-stdlib.md
new_spec_components:
  - docs/specs/stdlib/design/http-file-transfer.md
touched_goals: [GOAL-003]

affects: [stdlib/auto/http.at, stdlib/auto/http.vm.at, crates/a2r-std/src/http, crates/auto-lang/src/a2r_std.rs, crates/auto-lang/src/trans/rust.rs, crates/auto-lang/src/vm/ffi, crates/auto-lang/src/vm/native.rs, crates/auto-lang/src/vm/engine.rs, crates/auto-lang/src/vm/native_catalog.rs, docs/specs/stdlib, docs/specs/a2r-std]
current_step: 0
total_steps: 8
---

# [PLAN-727] HTTP 文件传输：增量读写、可靠提交、续传校验与取消

## 0. 变更摘要

接续 [Design 33](../design/33-stdlib-runtime-and-http.md) 阶段 C2c。[PLAN-724](archive/724-a2r-http-client-async-convergence.md) 已归档，共享 Rust async 客户端、流背压与取消进入 canonical Spec，但其明确保留了文件 helper。下一步以共享 Rust 传输执行代码支撑 VM/a2r 文件下载、上传和进度，不再整文件缓冲或每传输创建线程/runtime。

交付重点是客户端文件 I/O 与任务生命周期：下载失败不覆盖旧文件、续传不盲目追加、上传不吞读盘错误、取消不遗留后台写入，以及同源 Auto 的两后台实际编译运行。本计划不实现服务端任意文件上传路由、文件服务 Range 响应、API/IPC 契约统一、CPU 抢占或部署配置。

附带核实共享内核的一处前置差距：现行 `KernelInstance::execute` 先等待 active 许可，再监听调用方取消并包裹 timeout；静态看到排队等待未被该取消/总期限覆盖。T-02 必须以 gate 未开的复现验证并闭合，不能将 724 报告中的强结论当成该路径的实证。

## 1. 目标

- G1：新文件传输公共声明与 VM/a2r 实现具有相同语义；两个 Rust facade 和 VM 的文件传输网络/读写执行复用独立 a2r-std 的纯 Rust 核心。
- G2：网络与文件增量读写，准入、缓冲、文件大小、期限有边界；慢网络/慢磁盘/慢进度消费者不占用 VM owner 或 Tokio reactor。
- G3：临时文件到正式文件有明确提交点；失败/取消保留原目标，验证 Range/Content-Range/本地长度后才续传，写盘错误不能报告成功。
- G4：公开可取消句柄、owned 结果与进度；排队/传输/写盘/收尾取消能回收 producer、文件句柄和许可，VM 请求 scope 能级联取消。
- G5：保留已有 helper 的可识别兼容面，纠正数据损坏/假成功；真 TCP、文件故障注入与同源编译运行覆盖 VM/a2r/原生 Rust。

## 2. 架构方案

```text
auto.http 新 FileTransfer 面 / 既有 helper 与 multipart builder
           ├─ VM shim：owned 描述、句柄、park/resume、scope
           └─ a2r lowering：sync bridge / async await、Rust facade
                              │
           a2r_std::http::transfer（新共享 Rust 传输核心）
           724 Client/runtime + 文件专用许可/预算 + 取消信号
           │ 下载：网络 bytes → 有限块 → 临时文件 → 提交
           └ 上传：文件有限块 → raw 或 multipart → 有界响应
```

保留 VM 所有者单线程；传输核心只收 owned、Send 的请求/路径/options，不持有 VM 引用。VM 与 Rust consumer 共享文件传输实现，但各自桥接生命周期；不把 VM token 直接套入 Rust runtime 的资源表。普通 HTTP/SSE 默认配额不被大文件传输长期占满；复用既有 runtime 和 Client，并为文件传输建立独立许可。

文件操作允许使用 Tokio 文件 I/O 底层受限的阻塞池，但禁止阻塞网络、同步 join/recv 在 owner/reactor 上运行。已经进入 OS 的读写未必能被 Future abort 抢占；取消后须等待/确认在途文件操作收口，再关闭、清理和报告终态。不能把丢弃 async wrapper 等同于物理写盘已经停止。

## 3. 技术栈

- 724 的 reqwest async、固定 Tokio runtime、Client 复用和独立 KernelInstance 测试实例；按需加入 multipart/stream 适配依赖，不新增自制 HTTP 解析器。
- Tokio fs/AsyncRead/AsyncWrite、有界块生产与取消；必要的 Windows 文件替换原语通过局部平台 helper 封装。
- VM 既有 managed result、完成通知、RequestScope 和 native 注册；a2r 既有 typed async 上下文 lowering。
- OS 临时端口 TCP stub、临时目录、可注入文件 I/O 故障与 gate；复用现有同源转译编译运行基建。

## 4. 需求分析与背景调查

### 4.1 授权、版本与现行依据

- 用户确认「724 已经完成」，请求规划下一个计划。本次只起草一个计划、更新 Design 33/索引；仓库范围为 auto-lang，无实施、跨仓修改、预算或自动连续执行授权。
- 静态调查基线 master `17292c07e`；724 归档 revision 1，delivery commit `8a7fa3cee`，current Spec 于 2026-10-02 沉淀。T-01 记录实际实施基线与相关文件 hash。
- 权威输入：`docs/specs/overview.md`、`stdlib/project.md`、`stdlib/design/backend-assembly.md`、`http-handler-async-lifecycle.md`、`http-stream-lifecycle.md`、`a2r-std/project.md`、`a2r-std/design/http-client-runtime.md`、`auto-lang/trans/design/http-client-lowering.md` 与 `test-convention.md`。
- 外部依据（2026-10-02 核对）：[RFC 9110 §13.1.5 If-Range](https://www.rfc-editor.org/rfc/rfc9110.html#section-13.1.5)、[§14.4 Content-Range](https://www.rfc-editor.org/rfc/rfc9110.html#section-14.4) 用于续传校验；[Tokio fs](https://docs.rs/tokio/1.53.1/tokio/fs/index.html) 与 [spawn_blocking](https://docs.rs/tokio/1.53.1/tokio/task/fn.spawn_blocking.html) 说明文件 I/O 底层与不可强制中止的限制（Cargo.lock 为 Tokio 1.53.1）。据此将「取消网络」和「文件操作收口」分开验收。
- 活跃计划 725/726 分别涉及 UI 帧/执行吞吐，与本计划不重复；可能共同触 trans/rust.rs、VM/tests 注册，实施前核对已落地 diff，按实际基线重验触面。726 的 P707-R1 任务表仍有历史项，但该问题已由 724 清偿，本计划不重复立项。
- 排他扫描活跃区与 archive 后取号 727（下一号 728）。代码实施目录 `D:/autostack/.wt/lang-727/auto-lang`，分支 `plan-727-dev`；禁止 worktree 内 junction/symlink。

### 4.2 源码实证与差异

| 证据 | 现状 | 设计影响 |
|---|---|---|
| `vm/ffi/stdlib.rs::shim_http_download` | 每调用线程+join，response.bytes 整体入内存，先 create 目标，未校验 HTTP 状态 | 增量落盘、先验证响应、失败不破坏旧目标 |
| `shim_http_download_resume` | 直接 append，未检查 200/206、Content-Range、目标长度；offset 从 i32 栈取值 | 明确 64 位字节位置、严格续传与重启策略 |
| `shim_http_upload`、builder send 中 mp_file_data | upload 是 blocking multipart；builder 对文件 fs::read 整体预读，读失败 filter_map 静默跳过，重试 clone 文件字节 | 流式 multipart、读失败终结、每次重试重开且验证源文件 |
| `spawn_download_with_progress` | 专用线程/current-thread runtime；同步 write_all；64 项进度通道、ASYNC_STREAMS，无传输取消链 | 独立 managed 资源与完成通知；进度合并、有界终态、scope 清理 |
| `crates/a2r-std/src/http.rs` 文件 helper | ureq；download/resume 忽略 create/copy 错仍返回状态；upload fs::read 全文件且 raw POST，与 VM multipart 不同 | 新可移植面统一，旧直接 Rust raw/status 形状以 adapter 保留；不吞文件失败 |
| `stdlib/auto/http.at/http.vm.at` 与 native_catalog.rs | 文件 helper/multipart 扩展已注册，但无完整公共声明；catalog 的 2270..2273 标 Void 与实际返回形状不一致 | 补新声明/注册/类型检查；旧形状不能未经调查统一重标 |
| `trans/rust.rs` file helper 分派；`test/a2r/17_rust_std/011_http_download_upload/` | 总是同步映射；旧 golden 把 u32 返回赋给 i64，仅文本金样不能证明编译 | async lowering 与实际 rustc；旧样本兼容差异必须记录 |
| `http_server.rs::RequestScope.resources/register_scope_stream` | 当前资源组以 u64 交给 http_stream::stream_cancel | 文件资源需有类型区分或独立组，不能把 transfer id 当 HTTPStream id 清理 |
| `a2r-std/src/http/client.rs::KernelInstance::execute` | active.acquire_owned 在 tx.closed/timeout 之前 | 静态差距，未在本次运行复现；T-02 核实排队取消和准入期限 |
| networking-stdlib Spec、Design 33 | 仍有「a2r 未迁移」的旧表述；724 新 Spec 明确文件 helper 不在交付集 | 修正过时路线，新增文件传输 canonical 契约，而非假定旧文档已经覆盖 |

尚无完整的文件传输 Spec。server Spec 中「multipart 后续增强」指服务端，本计划不能因客户端支持 multipart 就删除该边界。

## 5. 详细设计

### 5.1 公共面与兼容

建议新增 opaque `FileTransfer`，`transfer_download(url,path,options:str)` / `transfer_upload(url,path,options:str)` 返回该句柄；`transfer_wait`、`transfer_next_progress`、`transfer_cancel`、`transfer_error` 为自由函数，避免方法名碰撞。T-01 用最小 VM/a2r 实编样本冻结准确签名与类型映射，允许等价名称调整，不允许取消 owned 结果/可控生命周期要求。

options 为严格解析的 JSON：下载 headers、offset、validator、预算/期限与已存在目标策略；上传 raw/multipart、field/filename/text 字段、headers。坏 options 不发请求。上传默认 multipart 的 file 字段，与 VM 既有 upload 对齐；raw 明确选择。路径是客户端本机文件路径，不由此授予服务端任意路径访问。

共享核心用 typed Receipt/Error/Progress；Auto adapter 输出固定 JSON 字符串以适配既有语言面，字段至少有 kind（success/failed/cancelled）、status、bytes、total（未知为 null）、headers、error（kind/message）。上传返回响应 body 受既有响应预算约束；不能以 status=200 判断写盘成功。progress 事件有 kind/bytes/total/percent，未知 total 的 percent 为 null；终态独立于普通进度，不因通道满丢失。wait 与 progress 等待在 VM park，在 a2r async await；重复查询不能重新发请求。

旧 download/download_resume 的 VM bool、upload 的 Response、旧进度 iterator 与直接 Rust status/raw upload 的差异逐项登记为 legacy adapter。不静默重定已有源/Rust 返回类型或 wire 格式。新增可移植面统一；旧 helper 也迁移网络/文件执行，并纠正写盘失败假成功、200 盲目追加等错误。旧 a2r 显式 int 金样若保留，须合法转换并实编，不能据它声称与 VM bool 同语义。

builder `multipart_file/multipart_text` 补公共声明及两 Rust facade 支持，文件保持路径/metadata 描述直到发送，不能预读为 Vec；整个 multipart 的字段数/文本总量受预算，响应仍走有界普通 Response。

### 5.2 下载提交与续传

- 验证成功状态后，在目标同目录创建独占、不可覆盖他人文件的临时文件；增量落盘，关闭/必要 flush 后提交。提交成功是唯一 success 点。HTTP 404/500、网络/文件错误、超限/取消均保留原目标。
- 目标同路径并发传输必须仲裁（冲突报错或显式串行），不可互相覆盖 temp。Windows 已存在目标的替换不得「先删原文件再 rename」；T-01 spike 验证平台 helper 与失败保留语义。只承诺已验证的可见性/完整文件替换，不承诺跨平台断电持久性。
- offset 用 u64/语言对应非负 64 位整数，拒绝负数/溢出。offset>0 须匹配本地文件长度；206 必须有合法 Content-Range，起点、范围长度、实际接收长度一致，字节复制 identity 编码以免解压后位置变化。
- 200（上游忽略 Range 或 If-Range 失配）必须从零完整重启并通过临时文件提交，绝不 append；416/错误范围/截断保留原文件并给可观察错误，不自动当成功。
- 新面使用对应本地前缀的 validator 发 If-Range；ETag 必须为强验证器（拒绝 W/ 弱 ETag），Last-Modified 按 RFC 的日期验证器适用条件处理。不满足条件或无 validator 默认完整重下，不能证明版本一致仍宣称可靠续传。legacy 三参 offset 可保留严格字节对齐模式，但明示不能验证远端版本。调用者提供的 validator 与本地前缀对应关系是前置条件，offset 本身不能证明内容正确。
- 续传须维护旧目标完整性，可分块复制旧前缀到 staging，再接后缀；内存不随前缀长度增长。额外磁盘空间不足为文件错误。取消/失败清理 staging；清理失败须保留可追踪路径/错误，不能宣称零遗留。

### 5.3 上传、重试与资源预算

新 transfer 默认 active=4、queue=16、应用读写块≤64 KiB、待处理块≤2、进度保留最新值+终态；单文件预算 1 GiB、准入至终结总期限 10min、上游 idle=60s，可通过严格 options/配置覆盖。文件大小上限与普通 response 的 10 MiB 上限分开。T-01 冻结字段/文本/headers 保留上限、最大并发文件句柄数、有限磁盘 I/O 并发及内存公式；现有用户显式超限要求可配置，不能改回无界读写。

支持普通文件，上传拒绝目录/设备/FIFO；目标路径、父目录、符号链接/reparse point 的处理规则在 T-01 固定并验证，不承诺从不可取消的 OS/设备阻塞中强制抢占。期限到达先取消网络与后续文件操作，已入 OS 的操作必须收口；若清理滞后，状态/结果须明确区分 cancelling 与已完成取消，不能为了按时返回先释放仍被占用的资源。

raw/multipart 逐块读文件，错误不省略 part。每次被允许的重试重开文件并复核长度/身份/mtime，修改导致 FileChanged；不能重用已消费 body。新上传默认不自动重试非幂等 POST，不承诺 exactly-once；用户显式开启且能重放时才重试，旧 builder retry 配置要保留并写清对端可能重复接收。

慢磁盘向网络读取背压；慢进度消费者只使进度合并，不阻止落盘与关闭。应用缓冲高水位、file descriptor/许可/producer 计数可观测，记明 HTTP 库/OS 缓冲不在应用内存公式内。

### 5.4 生命周期与桥接

状态：Queued → Opening → Transferring → Committing → Succeeded，任何非终态可进入 Failed/Cancelled；只终结一次。取消到达提交点之前，原目标保持；若替换已成功，结果为成功，迟到取消不能回滚或覆盖已提交结果。测试用 commit gate 冻结竞态规则。

核心兼容普通共享 executor 的排队取消/期限，但 transfer 配额独立。文件操作 wrapper abort 后底层在途 I/O 的收口须被跟踪；许可/句柄归还与 temp 清理完成才符合资源回基线。不得每任务 spawn OS 线程/runtime，允许标准库文件 I/O 的受限线程池。

VM 请求资源组增加有类型的 transfer 资源或独立登记；scope deadline/断连/shutdown 级联取消，CALL_NAT 重入只消费一次并恢复原栈；非请求 CLI/UI 持有者可显式 cancel，task/iterator 销毁有清理路径。旧 ASYNC_STREAMS 进度适配迁移本传输 producer，不重做其它 io/bus stream。

Rust FileTransfer 最后拥有者 Drop、显式 cancel、终结等待 Future Drop 均取消执行；丢弃一次 progress 读取 Future 只放弃该次观察，不取消整个传输。T-01 冻结 VM task/iterator 与 Rust 所有者的准确映射及 runtime 退出顺序。async 上下文内旧 helper 由 typed lowering 选 async facade；显式同步桥接边界沿用 724，不作嵌套 block_on 兜底。

### 规范增量

| delta_id | add/modify/retire | docs/specs/... target | before/after rule | rationale | acceptance IDs |
|---|---|---|---|---|---|
| SD-01 | add | `docs/specs/stdlib/design/http-file-transfer.md`（新） | 无完整契约 → FileTransfer/legacy、raw/multipart、提交/续传/错误/预算/取消与支持表 | 跨后台文件传输依据 | AC-01..06 |
| SD-02 | modify | `docs/specs/a2r-std/design/http-client-runtime.md`、`a2r-std/project.md` | 文件 helper 为 ureq 剩余项、排队取消/期限强表述 → 实证共享 transfer 与准入/取消边界 | 724 接续且修正代码/文档差距 | AC-02/05/06 |
| SD-03 | modify | `docs/specs/stdlib/project.md`、`stdlib/design/backend-assembly.md` | 文件声明/两后台差异未完整登记 → 新可移植与旧兼容支持集，服务端上传仍另案 | 装配事实准确 | AC-01/06 |
| SD-04 | modify | `docs/specs/auto-lang/trans/design/http-client-lowering.md` | 文件 helper 同步直发/无资源形态 → typed 文件 handle、sync/async wait、multipart 发射与实编样本 | 语言调用真正可用 | AC-01/02/06 |
| SD-05 | modify | `docs/specs/stdlib/design/http-stream-lifecycle.md` | 文件进度 worker 未迁 → 本计划 transfer 进度/取消/终态；其余 IO/bus 不变 | 生命周期归属明确 | AC-05 |
| SD-06 | modify | `docs/specs/auto-lang/runtime/design/networking-stdlib.md` | 仍称 a2r 客户端未迁 → 724 当前状态、本计划文件传输与后续边界 | 清除过时路线 | AC-01/06 |

本次只提出 delta；review 验证后 merge 沉淀，不先修改 canonical Specs。

## 6. 测试设计

- 二进制/零字节/未知 Content-Length/chunked 下载与大于 10 MiB 的文件；两种规模比较应用缓冲高水位，文件大小增大不导致同比内存增长。超文件限额/截断/磁盘写失败保留旧文件。
- 206 正确范围、起点/长度不符、200 忽略 Range、If-Range 版本变更、416、本地 offset 不符/负数/64 位溢出；大 offset 用稀疏文件/注入 metadata 验证，不真的传输 GiB。
- Windows 覆盖已有文件、同目标并发、temp 创建/写/flush/替换/清理失败；commit gate 与 cancel 竞态。检验原目标字节/临时文件/打开句柄，不能只检查返回码。
- raw 和 multipart 检查 wire 的方法、headers、字段/filename/text 和完整 binary 文件内容；缺文件、读失败、传输中源文件修改、显式重试后重放与响应预算。
- current-thread/multi-thread 与 VM handler gate：慢上游/磁盘期间 health 完成；queued/opening/read/write/commit 取消，active=1 时上游 gate 不开也能回收旧 job。进度慢消费/断消费者必须可退出，未知 total/单次终态正确。
- 同源 Auto 新面、legacy shape、builder multipart 的 VM/a2r 执行；原生 Rust与两 facade；实际编译运行、64 位长度、空文件成功和失败结果对拍。沿用 plan707/724 取消与 ordinary HTTP/SSE 守卫。

新增文件：`crates/a2r-std/tests/http_transfer.rs`、`crates/auto-lang/src/tests/plan727_http_transfer_tests.rs`，在 `src/tests.rs` 注册；golden 目录 `crates/auto-lang/test/a2r/31_plan727/`（新，按既有目录约定）。VM 真 TCP/编译运行用例加 test-http-e2e 守卫、命名 `http_e2e_plan727_*`，归 cargo th 串行；纯类型/状态逻辑留日常档。临时端口/独立临时目录/可停止且有截止时间的 stub，失败同样 join/cleanup，不复制旧裸 accept 模式。

开发只跑 scoped check/test：`cargo check -p a2r-std -p auto-lang`、`cargo test -p a2r-std http_transfer`、`cargo t plan727`。复审在 worktree 串行跑 `cargo test -p a2r-std`、裸 `cargo t`、`cargo tv`、`cargo tt`、`cargo th`；ui_gen/auto-man/书籍不在预定触面，不跑 tu/tb；若实际触及再按 AGENTS 加相应档。aavm 无触发。tf 仅 merge 时到期批量回归，在主检出单实例；本次文档起草不运行 Cargo。

## 7. 验收标准

| ID | 通过条件 | 证据 |
|---|---|---|
| AC-01 | 新公共 FileTransfer、multipart 声明/注册/类型/两 facade 映射齐备；新面 VM/a2r 结果一致；legacy 返回/wire 差异不被悄悄重定 | decision 支持矩阵、类型/golden、wire 与实编 |
| AC-02 | 文件传输无整文件 Vec、阻塞网络、每任务线程/runtime；async 等待让出 reactor/owner；应用内存守独立预算；排队取消/总期限覆盖准入 | 大小阶梯高水位、源码扫描、VM/Tokio gate health、queued cancel/deadline 探针 |
| AC-03 | 仅完整成功文件可提交；HTTP/读写/flush/替换/超限/取消失败保留原目标；同目标冲突有规则，Windows 替换不先删旧文件 | 故障注入、原文件字节、临时/句柄清理、commit race |
| AC-04 | Range/本地长度/实际长度严格验证；200 完整重启不追加；416/坏范围保旧；validator 与无 validator 的承诺清楚；64 位位置正确 | 续传 TCP 矩阵/稀疏位置/版本变更测试 |
| AC-05 | cancel/Drop/scope/shutdown 停止网络生产与后续文件修改；在途 FS 收口后许可/句柄回基线；进度有界、终态不丢且单次，慢消费不挂下载 | queued/opening/read/write/commit gate、peer/文件变化/producer 退出与资源报告 |
| AC-06 | raw/multipart/legacy/builder 与同源 VM/a2r/原生 Rust实际执行正确；缺失文件/重试读失败不省略，不报告假成功；门禁无新增红，SD 有实证 | parity/verification 报告、两 facade 编译运行、门禁日志 |

## 8. 执行步骤

### T-01：冻结公共与兼容契约，验证文件提交原型

- 依赖：724 archived；记录新基线/hash，核对 725/726 已落地差异，按 AGENTS 创建唯一实施 worktree。
- 触面：§4 Specs、http.at/http.vm.at、stdlib.rs 文件 helper/native_catalog、a2r-std http/client/facade、trans/rust.rs、旧 plan349 与 a2r 011 样本。
- 输出新 `docs/plans/reports/727-transfer-decision.md`：精确签名/结果 JSON、legacy 差异、64 位 ABI、取消/FS/commit 规则、独立预算与文件重试矩阵。做 FileTransfer 最小 VM/a2r 转译实编、Windows 保旧替换与有限 FS 并发原型。
- 验证：最小脚本实际编译运行，新 API 不需泛化语言语义改动，替换故障保留旧文件。不能以调查未决删减 AC；若需公共语义超范围改动提交 needs_replan。覆盖 AC-01/03/04/05、SD-01/03/04。

### T-02：闭合共享准入取消与实现文件传输执行骨架

- 依赖：T-01。
- 文件：a2r-std/src/http/client.rs、新 `crates/a2r-std/src/http/transfer.rs`、Cargo.toml/必要 lock。
- 先用 active=1 gate 验证现行 execute queued Drop/timeout；取消与总期限从提交/许可等待开始覆盖。实施 transfer typed state/Receipt/Error、独立许可、Client/runtime 复用、受限 FS 操作及观测，保持普通 HTTP/SSE 基线协议。
- 验证：`cargo test -p a2r-std http_transfer`，旧 gate 不放行也能取消并回队列/许可；普通 execute 排队 deadline 有界；`cargo test -p a2r-std http_client` 无回归。覆盖 AC-02/05、SD-01/02。

### T-03：增量下载、提交与严格续传

- 依赖：T-02。
- 文件：新 transfer.rs（必要子模块 paths 在 T-01 登记），平台提交 helper；独立核心单测。
- 流读/分块写/staging/唯一提交、同目标仲裁、response/header 验证、206/200/416 与 validator、64 位 offset；文件错误与清理结果进入 typed receipt。
- 验证：`cargo test -p a2r-std http_transfer`；Range/故障/commit race 全绿、原目标字节符合 AC，应用缓冲不随文件扩大。覆盖 AC-02/03/04/05、SD-01/02。

### T-04：流式 raw/multipart 上传与 builder 文件 part

- 依赖：T-02。
- 文件：a2r-std/src/http.rs/client.rs/transfer.rs，auto-lang/src/a2r_std.rs HTTP facade；后续 VM builder 接入在 T-05。
- 路径描述到发送时开文件，支持 multipart file/text + raw；每次允许的重试重开/复核；缺失文件/读取失败不跳过 part；response 有界；旧直接 Rust raw/status adapter 与新 portable 面分开。
- 验证：`cargo test -p a2r-std http_transfer`，native fixture 检查 binary wire/fields、缺文件/读失败/变更/重放及非 2xx。覆盖 AC-01/02/05/06、SD-01/02/03。

### T-05：VM native、请求 scope 与进度桥接

- 依赖：T-03/04。
- 文件：stdlib/auto/http.at/http.vm.at；vm/ffi/stdlib.rs、http_server.rs、必要 ffi/mod.rs 与新 `http_transfer.rs` VM adapter、native_catalog.rs、vm/native.rs/engine.rs。
- 添加新 public/native 面；CALL_NAT 重入通知等待、typed transfer scope 归属/清理；旧 helper/进度迁移 producer，multipart builder 去掉 mp_file_data 全量预读/静默省略；不重做其它 ASYNC_STREAMS worker。
- 验证：`cargo check -p auto-lang`、`cargo t plan727` 与 feature 门下 VM gate；scope/iterator 销毁取消、进度不热循环、原栈/返回类型与 native ID 无碰撞。覆盖 AC-01/02/05/06、SD-03/05。

### T-06：a2r typed 文件接口发射与兼容 adapter

- 依赖：T-01、T-04/05。
- 文件：trans/rust.rs、a2r_std.rs、a2r-std/src/http.rs；新测试/31_plan727 fixtures、旧 17_rust_std/011 样本与 tests/a2r_tests.rs 必要更新。
- FileTransfer 类型与自由函数 sync/async lowering；multipart builder typed 方法；旧 helper 在 async 上下文不落阻塞面；合法整型转换保 legacy 显式 int 样本。用户同名方法不重写，受限同步 helper 边界明示。
- 验证：`cargo t plan727` 与 scoped golden；新 sync/async、legacy、两 runtime 限定名 fixture 实际编译运行，64 位位置/bytes 无截断。覆盖 AC-01/02/06、SD-04。

### T-07：文件故障、资源与三方端到端矩阵

- 依赖：T-02..T-06。
- 新文件：a2r-std/tests/http_transfer.rs、auto-lang/src/tests/plan727_http_transfer_tests.rs；src/tests.rs 注册；报告 `docs/plans/reports/727-transfer-parity.md`、`727-transfer-resources.md`。
- 按 §6 完整矩阵跑真实下载/上传/续传、FS 注入、提交竞态/资源计数与 slow health；不是只比状态字符串。大文件用确定数据/hash，offset 大值用稀疏/metadata 探针避免重载。
- 验证：`cargo test -p a2r-std --test http_transfer -- --test-threads=1`、`cargo nextest run -p auto-lang --lib --features test-http-e2e --test-threads=1 http_e2e_plan727`；矩阵命令/退出码、缓冲公式与基线回收入报告。覆盖全部 AC、SD-01..05。

### T-08：触面门禁、独立复审与规范沉淀提案

- 依赖：T-01..T-07。
- 按 §6 分级门禁，保留 plan707/724 scoped 回归；差异扫描无新增警告/调试输出/未批准延后。新 `docs/plans/reports/727-transfer-verification.md` 逐项绑定 AC/SD 与代码 revision；确定性红逐名与新基线比较，不笼统归环境。
- `/auto-plan:review` 独立验证后进入 merge：沉淀 Spec/项目卡、ledger 与设计、归档；到期 tf 仅主检出单实例。T-01 的命名等价调整记录在合同，实质验收变化须 revision。
- 验证：AC/SD 实证全覆盖；兼容/FS 取消限制明确，所有代码在计划 worktree，主检出只簿记。覆盖全部 AC/SD。

## 9. 复审记录

### 起草交接（2026-10-02）

- stage: new
- plan: PLAN-727
- revision: 1
- outcome: pass
- next: work
- changed_tasks: T-01..T-08（新）
- changed_acceptance: AC-01..AC-06（新）
- 起草核对 724 archived/current Specs、文件 helper/进度/builder/native/a2r 发射与请求资源组；任务覆盖 AC/SD。pass 只表示执行合同可交接，不是代码复审通过。
- 本次已检查 0..10 章节、T/AC/SD 覆盖、现有源码/Spec 路径、计划与 Design 33 链接、唯一取号及 git diff --check。未创建实施 worktree、未运行 Cargo、未改 canonical Specs。排队取消与文件失败均为执行期需复现的静态线索。

## 10. 待澄清事项

- 没有阻碍规划的用户输入缺项。T-01 负责准确 API/ABI、Windows 提交与 FS 取消边界的有界原型；若需超范围语言重构或改变 legacy 正常成功类型，提交具体修订。
- 无 validator 的 resume 不能保证版本/前缀来源；新面默认完整重下，legacy 字节续接的限制必须明确。并不承诺下载到任意崩溃/断电下保持持久。
- 完成后仍有 API 多形态契约/任意 Rust 流 handler 生成、后台装配 manifest、CPU 纪律及部署支持等级。服务端文件上传/下载路由与 Range 响应另案；本计划不把它们算作已交付。
