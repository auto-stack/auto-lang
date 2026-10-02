# PLAN-730 T-01 决策报告：服务端上传接收的 ABI 与机制冻结

- 基线：master `7d50989f7a`（worktree `D:/autostack/.wt/lang-730/auto-lang`，分支 `plan-730-dev`）。
- 依赖核对：PLAN-729 已 reviewed+archived+merged（spec 沉淀 `050e2ee90`，归档收据 `9c34fb387`/`4135cb222`/`7d50989f7`，组目录已清）。
  729 最终 Spec（`docs/specs/stdlib/design/http-server-files.md`）与本合同无冲突：
  受限根目录算法（`safe_open_file` 逐段 no-follow walk + Windows
  `FILE_FLAG_OPEN_REPARSE_POINT|BACKUP_SEMANTICS`）、`FileServeLimits` env 样板、
  两期限交接（handler 30s deadline 不延伸进 body）、版本无关 `FileReply` 投影全部直接复用形态。
  729 的 `validate_relative_path`（a2r-std `server_file.rs`，pub）被 commit 目标词法校验复用。
- 探针证据（本仓 `cargo t plan730_probe`，Windows 实测 3/3 绿，Linux CI 侧同断言）：
  - **P1 hard_link create-only**：`std::fs::hard_link` 对已存在目标 `AlreadyExists`
    （不覆盖、原文件字节不变）；新目标成功；删 staging 后 target 保留 →
    **发布 = `hard_link` + `remove_file(staging)`**，std 级原子 create-only。
  - **P2 rename 反证**：`std::fs::rename` 在 Windows=MoveFileExW(REPLACE_EXISTING)、
    Linux=rename(2)，均覆盖已存在目标 → rename 与 `exists`+rename（TOCTOU）都不符合
    create-only 合同，拒绝采用。
  - **P3 同卷判定**：Windows canonicalize 前缀（`\\?\C:`/`\\?\Volume{GUID}`）、
    unix st_dev 可判同卷；词法嵌套（staging 在 root 之内）可静态检出 →
    跨卷/嵌套配置在**读取 body 之前**拒绝（commit 侧 hard_link EXDEV 仍兜底）。

## 1. 差距复现（静态确认，行号为基线修订）

| 计划差距 | 代码事实 |
|---|---|
| 桥先收完整 body | `http_transport.rs:226-245` `to_bytes(body, 10MiB)` 在 scope 建立与入队**之前**执行；CL 预检 `:215-225` 同为 10MiB。 |
| 写盘先于路由/中间件 | `http_server.rs:4331-4353` 在 `match_route`(`:4355`) 与 middleware 之前调 `multipart_to_handler_json`→`store_multipart_file`；404/中间件拒绝也落盘。 |
| 写错误假成功 | `http_server.rs:416` `let _ = std::fs::write(...)` 后 `:417` 无条件返回路径字符串；`create_dir_all` 错误同样被忽略。 |
| owner 阻塞写盘 | `store_multipart_file` 在 VM owner 线程同步 `std::fs::write` 整文件。 |
| 解析器简化 | `parse_multipart`(`:319-385`) 全量 Vec 扫描、无错误结果、各 part 复制。 |
| 无 UploadRequest 分类 | codegen/api 元数据无注入参数分类；`bind_api_args_by_name` 的 whole-body 单参容忍(`:6238`)与 `req/request` meta 名约定(`:6272`)会吞掉名为 req 的注入参数——**类型识别必须置于这些规则之前**。 |
| TS 无 multipart | `typescript.rs:368-380` body 恒 `JSON.stringify`。 |
| 无 multer | 全仓 Cargo 无 multer；现有 multipart 全靠 reqwest client feature。 |

## 2. 模块拆分与依赖方向（冻结）

```text
a2r-std/src/http/server_upload.rs   纯面：owned 类型 + 严格 options + 限额表(env)
  ├─ UploadRequest(method/path/headers + UploadBodyStream，宿主构造)
  ├─ UploadSession(id + 接收快照)、UploadReceipt(status+kind+json)
  ├─ async facade: upload_receive/commit/reject/metadata/error
  │    → UPLOAD_EXECUTOR: OnceLock<Arc<dyn UploadExecutor>>（宿主 hook）
  │    未安装 → 确定性诊断失败（零 I/O）
  └─ 错误种类→HTTP status 纯映射、metadata/receipt JSON 组装
auto-lang/src/http_upload_service.rs  宿主 executor：增量 multipart 状态机、
  raw 接收、staging 写入、预算强制、hard_link 发布、reject/清理、lease 看门狗、
  配额信号量、session 注册表（无 axum 类型；消费 UploadBodyStream）
vm/ffi/http_upload.rs  VM 桥：三注册表 + 5 shims + scope 组收口
http_transport.rs/http_server.rs  桥 deferral + 注入 + 编组门 + 期限切换 + legacy 修复
auto-man/api_gen.rs + trans/rust.rs + api/targets/*  生成腿
```

依赖方向不变：a2r-std 不依赖 auto-lang；执行经 hook 注入（§2 契约的"owned async hook"）。
`install_upload_executor()` 幂等，安装点：VM `serve_with` 入口、生成 main.rs 模板、
独立 axum 生成器 main、e2e fixture。

## 3. 公共 ABI（冻结；Auto 与 Rust 同词汇）

```text
type UploadRequest / UploadSession / UploadReceipt        # opaque，无用户构造 native
http.upload_receive(req UploadRequest, root str, staging_root str, options str) UploadSession
http.upload_metadata(session UploadSession) str
http.upload_commit(session UploadSession, relative_target str) UploadReceipt
http.upload_reject(session UploadSession, status int, message str) UploadReceipt
http.upload_error(status int, message str) UploadReceipt
```

- Rust 面：`a2r_std::http::upload_receive(req, root, staging_root, options).await -> UploadSession`
  等（int→i64；VM shim i32 弹栈升位）。receive/commit/reject 为 await 点；metadata/error 纯同步。
- VM：receive/commit/reject 复用 727 live-op park 样板（`waiting_http_request_id` 重入先于弹栈）；
  sync/async handler 同形。native id：`upload_receive=9937(Int)`、`upload_metadata=9938(String)`、
  `upload_commit=9939(Int)`、`upload_reject=9940(Int)`、`upload_error=9941(Int)`（9936 后顺延，
  auto. 前缀双注册）。
- **options（严格 JSON，`deny_unknown_fields`；解析在零 I/O 处）**：
  `{"mode":"multipart"|"raw"`, `"file_field":str`(默认 "file"，multipart),
  `"text_fields":[str]`(默认 []，multipart), `"raw_content_types":[str]`(raw 可选，缺省=允许非 multipart 任意),
  `"max_file_bytes"`, `"max_wire_bytes"`, `"max_text_fields"`, `"max_text_field_bytes"`,
  `"max_text_total_bytes"`, `"max_parts"`, `"max_part_header_bytes"`, `"max_part_header_items"}`
  ——全部可下调不可超过服务硬上限（env）。root/staging_root 为显式实参（受信配置，
  绝不来自请求字段）；staging_root 必须存在、与 root 同卷、不在 root 之内且不包含 root
  （违反在读取 body 前 500 拒绝）。
- **commit**：`relative_target` 经 729 `validate_relative_path` + 父目录 no-follow walk
  （复用 `safe_open_file` 算法形态）→ `hard_link(staging, target)`（§探针 P1）→ 成功后删 staging。
  AlreadyExists→409（原文件不变）；EXDEV/不支持 hard link 的 FS→500 明确诊断，不回退复制。
- **receipt JSON**：成功 `{"ok":true,"path":...,"size":"<十进制串>","field","filename","fields":[有序]}`；
  失败 `{"ok":false,"kind":...,"message":...}`（kind∈invalid_options/body_too_large/
  unsupported_media/bad_multipart/idle_timeout/total_timeout/queue_full/target_conflict/
  forbidden_path/io_error/session_conflict/cancelled/expired_lease/rejected）。
  状态映射：413/415/400/408(两期)/503+Retry-After/409/403/500/应用 4xx。
- **metadata JSON**：`{"state":"received"|"failed", "mode","field","filename","size":"串",
  "content_type","fields":[有序含重复], "kind","message"(失败时), "suggested_status"(失败时)}`。
  upload_reject 对 failed session 传 `status=0` → 采用 suggested_status（省 app 样板）。
  upload_error 状态域 400..599，域外→500 receipt + 可观察诊断（不静默）。
- 成功 commit=201；早拒 upload_error 构造零 I/O 零文件。

## 4. 机制冻结

1. **桥 deferral**：POST/PUT 先查上传路由（`get_routes`+`match_route`+param sigs 含
   `UploadRequest`——方法+类型双条件，不按 Content-Type 猜）。命中 → CL（若有）对上传
   wire 预算（默认 65MiB）预检 413、**不 `to_bytes`**，`ApiRequest.raw_body` 携带
   `axum::body::Body` 入队；未命中 → 现行 10MiB 行为不变。授权前不解析 part、不开临时文件。
2. **30s→上传期限切换**：`RequestScope` 增 `started_at` + `deadline watch`；
   桥 reply 等待改 loop 重臂（sleep_until(当前)+watch.changed()，修"select 保留旧捕获值"）；
   receive 启动时 phase hook 调 `extend_scope_deadline(scope_id, started_at+total)`（默认
   10min；owner parked 定时臂每轮重读 scope.deadline，现状已满足）。idle 60s 在接收循环内
   逐 chunk 强制；staged lease 30s 看门狗。生成腿 phase hook = 服务级计数 no-op。
3. **VM 注入与编组**：`bind_api_args_by_name` 循环顶按 `ty.contains("UploadRequest")`
   push 宿主句柄（先于 path/body/query/meta 全部规则——防 "req" meta 名与 whole-body 吞参）；
   start_handler 从 ctx.raw_body 建 UploadRequest 记录（scope 组绑定）。编组门三重命中
   （返回类型含 `UploadReceipt` + i32 + `take_upload_receipt`）→ `ApiReply::Full{receipt.status,
   Text(receipt.json), application/json}`；声明但未登记→500 诊断（file 门同形）。
   codegen 加载期诊断：UploadRequest 参数仅允许 POST/PUT；其余参数必须 path/query/meta
   可绑定（否则编译错误）；UploadRequest 返回类型非法。
4. **上传路由 middleware fail-closed**：上传端点的 middleware 执行 Err→500 短路（零落盘），
   不沿用 legacy Err→Continue；非上传路由语义不变。
5. **legacy 修复**：`match_route` 提前（404/方法不匹配零解析零写盘）；multipart 内存解析
   保持在 dispatch 段（10MiB 桥上限兜底）；**落盘移到 middleware 链后**——新
   `ParkStage::LegacyStore{op_id}` + `spawn_blocking` 有界宿主执行 + live-op 唤醒（复用
   `ParkedWait::HttpRequest` 全套管线）；写失败→500 真实错误（不返回假路径）；
   provisional 文件归 scope legacy_files 组，绑定失败/handler Err/取消删除本次新建文件，
   成功 marshal 保留（历史语义）。`store_multipart_file` 所有调用点错误传播。
6. **取消仲裁**：commit gate = `hard_link` syscall 本身。gate 前 cancel 胜出（清理 staging、
   不发布）；进入 gate 后迟到取消等实际结果（失败清理、成功保留——不回滚已发布文件）。
   断连（conn watcher→cancel_scopes_for_conn）与 session Drop 经 `cancel_session(id)` 级联。
   磁盘不强制 abort：停止 issuance、等在途写返回后清柄；cleanup 失败记 request id+受限诊断，
   不宣称零残留。
7. **预算公式**（默认，可 env 调：`AUTO_HTTP_UPLOAD_{ACTIVE,QUEUE,BLOCK,PENDING,FS_OPS,
   TOTAL_MS,IDLE_MS,LEASE_MS,QUEUE_TIMEOUT_MS,MAX_FILE_BYTES,MAX_WIRE_BYTES,MAX_TEXT_FIELDS,
   MAX_TEXT_FIELD_BYTES,MAX_TEXT_TOTAL_BYTES,MAX_PARTS,MAX_PART_HEADER_BYTES,
   MAX_PART_HEADER_ITEMS,BOUNDARY}`）：active 4 / queue 16（不读 body 不开文件）/
   fs_ops 4 / 块 64KiB / 待写块 2 / 单文件 64MiB / wire 65MiB（含 framing+epilogue）/
   文本 ≤16 个·单 16KiB·合计 64KiB / part ≤32 / part headers ≤16KiB·≤32 项 /
   boundary ≤70B / total 10min（自 headers 入 scope）/ idle 60s / lease 30s。
   staged 占 active 许可至终态。应用缓冲峰值 = carry(≤boundary+8)+part header buf+当前文本
   字段+2×64KiB 待写+单 wire 读帧，全部有计数探针。
8. **multipart 解析器**：自研增量状态机（**拒绝引入 multer**，理由：① §5.3 的 per-part-header
   16KiB/32 项与 carry 预算无法施加于 multer 内部缓冲，仅靠外层计数 stream 只能兜总量；
   ② 子集窄——单文件+有界文本、无嵌套、identity-only，状态机可完全表驱动测试；
   ③ 零新依赖，延续 729 决策 §8）。RFC 2046 delimiter 语义：CRLF--boundary、close
   --boundary--、transport padding、preamble/epilogue（计入 wire）；假前缀经 carry
   回吐为内容；closing boundary 缺失/截断→400；坏 UTF-8 文本→400；charset≠utf-8、
   嵌套 multipart、CTE 头→明确拒绝；filename 缺失/空合法（lossy 入 metadata，不作路径）。
   raw：整 body 为文件，Content-Encoding 非 identity→415。
9. **消费矩阵**：TS 上传方法参数收 `body: FormData`（fetch 直传、浏览器产 boundary、
   返回 `response.json()`；不 JSON.stringify(FormData)）；Tauri IPC→Unsupported 诊断
   （729 形态）；back_proxy→501（fn_meta 返回含 UploadReceipt）；IPC/merged 无构造面
   （UploadRequest 无用户 native）+ executor 未安装诊断；独立 axum 生成器同形 UPLOAD glue。
   生成腿 body extractor（`axum::extract::Request`）置最后、不预读；转译失败=位置诊断 500。
10. **端口**：e2e 用 18970-18999（729 已占 18950-18966；http_server e2e 已占 18731-18775）。

## 5. 原型结论

平台探针（§探针 P1-P3）+ 729 全套可复用基座（scope/live-op/限额/安全 walk/FileReply
投影模式）证明 §3/§4 设计可实施，无需 HTTP Request/语言 ownership 重构——不触发
needs_replan。最小双端 raw/multipart commit/reject + 鉴权 gate 的运行证据随 T-05/T-08
e2e 落地（`http_e2e_plan730` 族）。

## 6. 依赖

零新增 crate 依赖（multer 拒绝理由见 §4.8；`streaming-http` feature 维持休眠不绑定）。
