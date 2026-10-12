# HTTP 文件传输（可取消、增量、原子提交）——stdlib 客户端文件传输契约

> 路径：`crates/a2r-std/src/http/transfer.rs`（共享执行核心单源）+ `vm/ffi/http_transfer.rs`（VM 桥）
> 状态：active（PLAN-727 交付；设计冻结细节见 `docs/plans/archive/727-*.md` 附带报告）

## 1. 公共面（Auto / VM / 原生 Rust 同词汇）

```text
type FileTransfer                                          # opaque 可取消句柄
http.transfer_download(url str, path str, options str) FileTransfer   # 非阻塞提交
http.transfer_upload(url str, path str, options str) FileTransfer     # 非阻塞提交
http.transfer_wait(t) str          # 阻塞至终态，返回收据 JSON（VM park / Rust 同步桥接）
http.transfer_wait_async(&t) str   # async await 面（唯一 async 变体；future 丢弃=取消）
http.transfer_next_progress(t) str # 最新进度 / 终态收据单次交付；"" = 无新内容
http.transfer_cancel(t) void       # 幂等取消
http.transfer_error(t) str         # 终结错误消息；"" = 无
```

Rust 面（`a2r_std::http` 与 `auto_lang::a2r_std::http` 同形）：上述自由函数 +
`transfer_wait_typed`（typed 收据）+ `cancel_transfer_by_id`（宿主 scope 级联）+
`FileTransfer::observer()`（非拥有观察句柄，Drop 不取消）。

发射规则（a2r）：submit/observe 四函数在 sync/async 上下文**同形发射**（非阻塞，
无 async 变体）；仅 `transfer_wait` 分叉——sync 发 `transfer_wait(&t)`（同步桥接，
响亮边界），async 发 `transfer_wait_async(&t).await`。

## 2. 跨后台 JSON 契约（冻结；加字段须走 Spec delta）

```json
{"kind":"success|failed|cancelled","status":N|null,"bytes":N,"total":N|null,
 "headers":{...},"error":null|{"kind":"...","message":"..."},"body":"..."}
{"kind":"progress","bytes":N,"total":N|null,"percent":P|null}
```

- `error.kind` ∈ `options|queue_full|http_status|transport|file|budget|timeout|
  cancelled|conflict|source_changed`。
- bytes：下载=已落盘（含续传前缀）；上传=已发送源文件字节。status：请求未发出
  为 null。**非 2xx = failed（http_status）**——与普通 HTTP 面"非 2xx 是正常值"
  语义不同，不以状态码宣称文件成功。
- 进度保留最新值（慢消费者只合并进度，不反压落盘/网络）；终态收据独立槽、
  单次交付（查询一次后恒 ""）；body 仅上传非空（受 response_body_budget）。

## 3. options（严格解析：未知键/坏值 = 终态 Failed(options)，请求不发出）

- 下载：`headers`、`offset`(u64)、`validator`（`etag` 强验证器，拒 `W/`｜
  `last_modified`，二者互斥）、`max_bytes`、`timeout_ms`、`idle_timeout_ms`、
  `on_exists`（`overwrite` 默认｜`fail`）。
- 上传：`headers`、`mode`（`multipart` 默认，字段名 `field`="file"｜`raw`）、
  `field`/`filename`/`fields`（raw 模式拒绝）、`timeout_ms`、`retries`（默认 0）。

## 4. 执行语义

- 状态机 `Queued → Opening → Transferring → Committing → Succeeded`；任何非终态
  可进 `Failed/Cancelled`；只终结一次（首个终态胜出）。
- **提交**：验证成功状态后，目标同目录建独占 staging（`.{name}.plan727-{id}.part`）
  → 增量落盘 → flush+sync → `std::fs::rename`（Windows =
  `MoveFileExW(REPLACE_EXISTING)`，**不先删原文件**）。提交成功是唯一 success 点；
  HTTP/网络/文件/预算/取消失败与 416/坏范围均保留原目标。
- **续传**：`offset>0` 要求本地文件长度精确匹配；206 需合法 `Content-Range`
  （起点=offset、范围长度=实收长度、total 一致），请求带 `Accept-Encoding:
  identity` 防解压位置漂移；200（忽略 Range 或 If-Range 失配）= 完整重启，
  **绝不追加**；续传前缀分块复制到 staging（内存不随前缀增长）。无 validator
  时仅承诺严格字节对齐，不承诺远端版本一致；新面默认完整重下。
- **上传**：流式读文件（缺失/读失败终结，不省略 part、不假成功）；raw 带
  `Content-Length`（否则 chunked）；重试默认 0（非幂等 POST 不自动重放），显式
  `retries` 每次重开文件并复核 len+mtime，不符 → `source_changed`；仅
  transport/timeout 可重试；不能重用已消费 body。
- **取消**：句柄 Drop / `transfer_cancel` / `transfer_wait_async` future Drop /
  scope 级联。网络即时停（每个 await 点 select）；**文件操作不可强制中止**——
  停止 issuing 新写 → 等在途写收口 → 关柄 → 清理 staging（失败保留可追踪路径，
  不宣称零遗留）→ 交付终态。commit gate 通过后的迟到取消不回滚（替换已成功 =
  success）。
- **仲裁**：同目标（归一化绝对路径）并发传输 → 第二笔终态 `conflict`；终态
  （含取消）释放槽位。

## 5. 预算（独立于普通 HTTP/SSE 配额；env 首次使用读取一次）

活跃 4（`AUTO_A2R_TRANSFER_ACTIVE`）/ 队列 16（**在途总量上限**，满 = 终态
`queue_full` 提交即拒）/ 应用块 ≤64KiB / 待处理块 ≤2（慢磁盘反压网络，慢网络
反压读盘）/ 单文件 1GiB / 总期限 10min（自提交起，排队计入）/ idle 60s /
内核级并发文件操作 4（单 FS 许可覆盖单传输文件阶段）/ 上传响应体 10MiB（与
普通 response 10MiB 上限分开计）。内核复用 724 的固定 runtime 与连接池。

## 6. VM 桥接与生命周期

- 注册表 `VM_TRANSFERS`（id → own 句柄+观察句柄）；等待复用 live-op 单次终结
  协议（`waiting_http_request_id` + `Waiting("http")`，CALL_NAT 重入消费）；
  丢弃等待不取消传输。
- 请求 scope：`transfer_resources` **有类型**组（与流 id 分离），
  `finalize_scope` 组收口（取消+出注册表）；deadline/断连/shutdown 级联。
- 进度迭代器：`http.download_with_progress` 的生产者迁移至本核心（观察句柄泵
  进既有 `ASYNC_STREAMS` 通道；进度 try_send 满则合并、终态不丢）；消费端
  迭代器机制不变。
- legacy adapter（形状逐字节保留，执行已迁移）：`http.download/download_resume
  -> bool`（yield 模式）、`http.upload -> Response`、a2r `download/upload/
  download_resume -> u32`（成功/HTTP 状态失败 → status；文件/传输/取消 → 0）。
  VM legacy resume 的 offset 自 i32 栈（32 位 ABI 事实保留；64 位路径在新面
  options）。builder `multipart_file/multipart_text`：路径描述到发送（不预读
  Vec）、预算 ≤16 文件 part / ≤64 文本字段 / 单值 64KiB / 总量 1MiB（builder
  期记录，send 终结）。

## 7. 非目标

服务端任意文件上传路由与 Range 响应（另案）；API/IPC 契约统一；CPU 抢占；
部署配置；断电/崩溃持久性承诺；无 validator 续传的远端版本一致性承诺。

## 8. 验证锚点

内核 23 例回环（transfer.rs mod tests）+ 串行矩阵 6 例（tests/http_transfer.rs：
If-Range/416/12MiB/chunked/队列饱和回基线）+ VM 腿（plan727_http_transfer_tests：
新面收据/坏 options/取消/进度 relay）+ 31_plan727 golden（sync/async 发射矩阵）
+ e2e 编译运行 4 件（`http_e2e_plan727_*`，test-http-e2e 门）。证据报告：
`docs/plans/reports/727-transfer-{decision,parity,resources,verification}.md`。
