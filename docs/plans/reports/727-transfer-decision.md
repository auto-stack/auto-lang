# PLAN-727 T-01 决策报告：文件传输公共面、提交/续传/取消契约冻结

- 阶段：work（T-01..T-04 核心合并交付的契约快照）
- 基线：worktree `D:/autostack/.wt/lang-727/auto-lang`，分支 `plan-727-dev`；
  实施起点 commit `b34852532`（master @ 2026-10-02，725 review 簿记头）。
- 725/726 差异核对：两计划的 dev 分支（`plan-725-dev`/`plan-726-dev`）均
  **未合入 master**（725 处 reviewed、726 处 executing，均在各自 worktree），
  本计划基线不受其触面（ui 帧/执行吞吐）影响；trans/rust.rs、VM 注册表
  与本计划触面无已落地交叠。
- 性质：本报告冻结文件传输的公共签名、收据/进度 JSON、options 模式、
  legacy 差异、64 位 ABI、取消/FS/提交规则与预算矩阵。全部决策**有实编/
  实运行证据**（独立探针 `t01-probe` 三场景、内核测试 61 例含 transfer 23
  例、plan724 内核回归 20 例绿、排队取消/期限新门禁 2 例），不是纸面推断。

## 1. 公共面签名（冻结）

### 1.1 Auto/VM 面（`http.at` / `http.vm.at` 新增；自由函数避免方法名碰撞）

```text
type FileTransfer
http.transfer_download(url str, path str, options str) FileTransfer   # 非阻塞提交
http.transfer_upload(url str, path str, options str) FileTransfer     # 非阻塞提交
http.transfer_wait(t FileTransfer) str          # 阻塞至终态，返回收据 JSON（VM park）
http.transfer_next_progress(t FileTransfer) str # 最新进度/终态收据单次交付；"" = 无新内容
http.transfer_cancel(t FileTransfer) void       # 幂等取消
http.transfer_error(t FileTransfer) str         # 终结错误消息；"" = 无
```

a2r 发射（T-06 落地）：`transfer_download/transfer_upload/transfer_next_progress/
transfer_cancel/transfer_error` 在 sync/async 上下文**同形发射**（非阻塞，无
`_async` 变体）；仅 `transfer_wait` 分叉——sync 发 `transfer_wait(&t)`（同步
桥接，响亮边界），async 发 `transfer_wait_async(&t).await`。

### 1.2 Rust 面（`a2r_std::http` + auto_lang `a2r_std::http` 同词汇）

```rust
pub fn transfer_download(url: &str, path: &str, options_json: &str) -> FileTransfer;
pub fn transfer_upload(url: &str, path: &str, options_json: &str) -> FileTransfer;
pub fn transfer_wait(t: &FileTransfer) -> String;               // 同步桥接
pub async fn transfer_wait_async(t: &FileTransfer) -> String;   // async 面
pub fn transfer_next_progress(t: &FileTransfer) -> String;
pub fn transfer_cancel(t: &FileTransfer);
pub fn transfer_error(t: &FileTransfer) -> String;
pub fn transfer_wait_typed(t: &FileTransfer) -> TransferReceipt; // typed（legacy/原生用）
pub fn cancel_transfer_by_id(id: u64) -> bool;                   // VM scope 级联入口
```

## 2. 收据/进度 JSON 契约（冻结）

```json
{"kind":"success|failed|cancelled","status":200|null,"bytes":N,
 "total":N|null,"headers":{"K":"V"},"error":null|{"kind":"...","message":"..."},
 "body":""}                                  // body 仅上传非空（response 有界）
{"kind":"progress","bytes":N,"total":N|null,"percent":P|null}   // total 未知 percent=null
```

- error.kind 枚举：`options|queue_full|http_status|transport|file|budget|
  timeout|cancelled|conflict|source_changed`。
- bytes：下载=已落盘字节（含续传前缀）；上传=已发送源文件字节。
- status：请求未发出（options/queue_full/本地前置失败）为 null。
- 终态收据单次交付（`transfer_next_progress` 消费一次后恒 ""）；进度保留
  最新值（慢消费者只合并进度，不阻塞落盘/关闭）。
- 非 2xx = `failed`（kind=http_status，status 保留）——文件传输不以 200 之外
  的状态宣称成功；普通 HTTP/SSE 面的"非 2xx 是正常值"语义**不适用于本面**。

## 3. options 严格模式（冻结；未知键/坏值 = 终态 Failed(options)，请求不发出）

```json
// 下载
{"headers":{"K":"V"},"offset":N(u64),"validator":{"etag":"\"strong\""}|{"last_modified":"..."},
 "max_bytes":N,"timeout_ms":N,"idle_timeout_ms":N,"on_exists":"overwrite"|"fail"}
// 上传（默认 multipart，字段名 file，对齐 VM 既有 upload）
{"headers":{"K":"V"},"mode":"multipart"|"raw","field":"file","filename":"a.bin",
 "fields":{"text":"v"},"timeout_ms":N,"retries":N}
```

- 弱 ETag（`W/`）拒绝；etag 与 last_modified 互斥；raw 模式拒绝
  field/filename/fields。
- `on_exists=overwrite`（默认）：目标存在 → 成功后经 commit 原子替换；
  `fail`：目标已存在 → 终态失败。offset>0（续传）隐含目标必须存在且长度
  精确匹配，`fail` 检查对续传路径不适用。

## 4. legacy adapter 差异表（返回形状逐字节保留；执行面全部迁移核心）

| 旧面 | 旧行为（缺陷） | 新行为（迁移后） | 形状 |
|---|---|---|---|
| VM `http.download(url,path)` | 线程+join、整文件入内存、先 create 目标、未验证状态、bool | 核心同步桥接：成功=true；任何失败/取消=false；原目标只在提交成功后替换 | `-> bool` 不变 |
| VM `http.download_resume(url,path,offset)` | 直接 append、未查 206/Content-Range、offset 自 i32 栈（32 位截断） | 严格字节对齐（offset==本地长度，i32 陷阱后述）+206 严格 Content-Range；200=完整重启 | `-> bool` 不变 |
| VM `http.upload(url,path)` | blocking multipart 整文件、失败伪造成 500 Response | 核心 multipart（field=file）流式；失败=错误 Response（status 0 语义） | `-> Response` 不变 |
| VM `http.download_with_progress` | 专用线程+独立 runtime、同步写、64 项进度通道 | 核心执行 + ASYNC_STREAMS 生产者迁移（进度合并+终态不丢） | iterator 不变 |
| a2r `http.download/upload/download_resume` | ureq；copy 错误吞掉仍返状态；resume 盲目 append | 核心执行：成功/HTTP 状态失败 → status；文件/传输/取消 → 0；last_status 同步 | `-> u32` 不变 |
| builder `multipart_file/multipart_text` | send 时全量预读 Vec、读失败 filter_map 静默跳过 | 发送时开文件流式；读失败=请求失败；重试重开+复核 len/mtime | 链式形状不变 |

> 旧 a2r golden `17_rust_std/011` 把 u32 赋给 i64（`var status int`）：发射侧
> 合法性在 T-06 验证（u32→i64 无损），legacy 样本保留；**不得**据此声称与
> VM bool 同语义（已在此登记为已知差异）。

## 5. 64 位 ABI（冻结）

- 新面 offset/大小经 **options JSON 的 u64** 传递——VM 侧全 str 参数，无栈
  位宽陷阱；Rust 面参数 `u64`。
- legacy VM `download_resume` 的 offset 自 i32 栈取值（`pop_arg_i32`）——
  32 位上限 2^31-1，**为既有 ABI 事实**，本计划不改签名（改变即破兼容）；
  新面是 64 位正确路径。T-07 用稀疏文件探针验证 64 位位置（不传 GiB 数据）。

## 6. 取消 / FS 收口 / 提交规则（冻结）

1. 状态机 `Queued → Opening → Transferring → Committing → Succeeded`；任何
   非终态可进 `Failed/Cancelled`；只终结一次（首个终态胜出）。
2. 取消入口：Rust 句柄 Drop / `transfer_cancel` / **`transfer_wait_async`
   future Drop**（结构化取消，与内核 execute 同语义）；progress 观察不取消；
   VM scope finalize 级联（`cancel_transfer_by_id`）。
3. 网络取消即时（select 于每 chunk/每 await 点）；**文件操作不可强制中止**
   ——写盘任务停止 issuing 新写，等在途写完成（收口），随后关柄、清理
   staging、释放许可/句柄，再交付终态。
4. staging：目标同目录独占文件 `.{name}.plan727-{id}.part`（同卷保证 rename
   原子性）。提交 = flush + sync_all + **`std::fs::rename`**——Windows 上即
   `MoveFileExW(REPLACE_EXISTING)`，**不先删原文件**（仓库既有记载
   `state_file.rs:10`；t01-probe 场景 2 实测：旧文件持读句柄时替换成功）。
5. 提交竞态：`before_commit` gate（spawn_blocking 执行，不占 worker）定义
   提交点——gate 前取消 = Cancelled（原目标保留）；gate 后迟到取消**不回滚**
   （替换已成功 = Success 收据）。内核测试 `plan727_commit_gate_late_cancel_
   does_not_rollback` 冻结。
6. 失败/取消一律保留原目标（404/500/超限/写盘失败/416/坏范围/offset 不符/
   冲突）；staging 清理失败 → 收据消息附可追踪路径，**不宣称零遗留**。
7. 同目标仲裁：归一化绝对路径在途表，第二个同目标传输 → 终态
   `conflict`；终态（含取消）释放槽位。

## 7. 独立预算与文件重试矩阵（冻结）

| 项 | 默认 | env 覆盖（首次使用读取） |
|---|---|---|
| 传输活跃许可 | 4 | `AUTO_A2R_TRANSFER_ACTIVE` |
| 传输队列容量 | 16（满 = 终态 queue_full，提交即拒绝） | `AUTO_A2R_TRANSFER_QUEUE` |
| 应用读写块 | ≤64 KiB | `AUTO_A2R_TRANSFER_BLOCK` |
| 待处理块（背压高水位） | ≤2 | `AUTO_A2R_TRANSFER_PENDING` |
| 单文件预算 | 1 GiB（options `max_bytes` 可收窄） | `AUTO_A2R_TRANSFER_FILE_BUDGET` |
| 准入至终结总期限 | 10 min（options `timeout_ms` 可收窄；排队计入） | `AUTO_A2R_TRANSFER_TIMEOUT_MS` |
| 上游 idle 期限 | 60 s（每 chunk 重置；写盘背压不计时） | `AUTO_A2R_TRANSFER_IDLE_MS` |
| 内核级并发文件操作 | 4（单 FS 许可覆盖单传输整个文件阶段） | `AUTO_A2R_TRANSFER_FS_OPS` |
| 上传响应体预算 | 10 MiB（与普通 response 上限分开计） | `AUTO_A2R_TRANSFER_RESP_BUDGET` |

- 文件大小上限与普通 response 的 10 MiB 上限**分开**；传输配额与普通
  HTTP/SSE 配额**独立**（KernelInstance 内独立信号量组）。
- 上传重试：默认 **0**（不自动重试非幂等 POST，不承诺 exactly-once）；
  options `retries=N` 显式开启；每次重试**重开文件并复核 len+mtime**，不符
  → `source_changed` 终态；仅 transport/timeout 失败可重试；不能重用已消费
  body（每 attempt 新通道+新流）。
- 慢磁盘 → 网络背压（通道满挂起 producer）；慢进度消费者 → 仅进度合并。

## 8. 证据索引（截至本报告）

| 证据 | 命令/位置 | 结果 |
|---|---|---|
| 独立实编探针（转译产物形状） | `D:/autostack/.wt/lang-727/t01-probe`（独立 cargo 工程，path-dep a2r-std） | 3 场景过：404 保留原目标；Windows 持读句柄替换成功；取消收据正确 |
| transfer 内核测试 | `cargo test -p a2r-std --lib http::transfer` | 23/23 绿（提交/保留/续传/重启/超限/取消/gate/冲突/上传 wire/缺失文件/目录拒绝/零字节） |
| 排队取消/期限门禁（T-02） | `cargo test -p a2r-std --lib http::client` | 22/22 绿（含 plan727_execute_queued_* 两例新门禁） |
| plan724 内核回归 | 同上 | 20 例 plan724 全绿（execute 重排无回归） |
| a2r-std 全量 | `cargo test -p a2r-std` | 61+7 全绿，0 失败 |
| ureq 退役 | `grep -rn ureq crates/a2r-std/src/` | 零引用；Cargo.toml 已移除 |

## 9. 边界（不因本报告扩权）

不承诺跨平台断电/崩溃持久性；不实现服务端任意文件上传路由、文件服务
Range 响应、API/IPC 契约统一、CPU 抢占、部署配置；legacy VM download_resume
的 i32 offset 陷阱保留为兼容事实（新面为 64 位路径）；无 validator 的续传
不承诺远端版本一致（新面默认完整重下，206 严格对齐例外如 §3）。
