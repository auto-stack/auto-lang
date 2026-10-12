# PLAN-729 T-01 决策报告：模块拆分、根目录打开与生成接入冻结

- worktree: `D:/autostack/.wt/lang-729/auto-lang`（分支 `plan-729-dev`）
- 基线：master `e5b068bd0a`（组脚本实测；起草时 9e379c9c8 之后的 728 系列提交不属本计划代码）
- 日期：2026-10-02
- 探针：`C:/Users/zhaop/AppData/Local/Temp/plan729-probe/probe.rs`（worktree 外独立编译运行，
  红线遵守：链接仅创建于探针独占临时目录并在探针内回收）

## 1. 基线核对（与计划 §4.2 的差异核对）

| 计划预判 | 实测 | 结论 |
|---|---|---|
| `ApiBody` 只有 Text/Sse | http_server.rs:3319-3322 属实 | 新增 `File` 变体 |
| scope 交接只有 SSE 分支 | http_transport.rs:312-316 属实 | File 与 SSE 同形：body 代持 scope |
| 声明返回类型元数据发布点存在 | codegen.rs:1406 `record_api_return_type`（`API_RETURN_TYPES`，`#[api]` fn 全量发布，值= `ret.unique_name()`） | 复用为文件返回门（含 `Future<FileResponse>` 形态，contains 判定） |
| 异步 `~T` 终值走 `final_value_reply`（直落 JSON） | http_server.rs:4487 属实；全部 park 恢复臂汇聚 `marshal_handler_value`/`final_value_reply` 两点 | 两点各插同一 file 门助手 |
| 生成 crate 与 auto-lang axum 版本 | **生成 workspace（rust_ui.rs:3686）钉 axum 0.7，auto-lang 0.8，不统一** | 宿主服务必须返回**版本无关 FileReply**（纯数据+流），Response 组装留在各消费端（见 §3） |
| tokio 统一 | 生成 crate `tokio = "1"` 与 auto-lang workspace tokio 同主版本，cargo 统一实例 | mpsc/File 等 tokio 类型跨 crate 可用 |
| futures | auto-lang 无条件 `futures = "0.3"`（Cargo.toml:240）；axum 0.7/0.8 均传递依赖 futures-core | 自实现 Stream（`futures::Stream` 即 `futures_core::Stream` 再导出），生成 crate 无需新增命名依赖 |
| `streaming-http` feature | 休眠（全仓零 `cfg` 引用） | 不启用、不绑定；文件面零新依赖 |
| mime_guess | 仅 auto-lang（Cargo.toml:230）；a2r-std 无 | MIME 推断留在宿主（auto-lang）侧 |
| 727 词汇 | `a2r_std::http::{transfer_*}`、`FileTransfer`；`http.at:278-331`、`http.vm.at:221-247`、native_catalog 9930+ 高段 | 完全沿模板；file_response 取 9936 |

## 2. 根目录打开：平台探针证据（Windows 实测）

| # | 探针 | 结果 | 决策输入 |
|---|---|---|---|
| P1a | `mklink /J root\jdir → secret`（junction，免管理员） | 创建成功 | junction 是现实可达攻击向量 |
| P1b | `symlink_metadata(junction)` | **`is_symlink=true`, `is_dir=false`** | **逐段 no-follow walk 可检出 junction** |
| P1c | `metadata(junction)`（follow） | `is_symlink=false, is_dir=true`（伪装成普通目录） | follow 式检查必然漏检——被计划禁用的方案坐实 |
| P1d | 直接 `open(junction/escape.txt)` | 读到 `ESCAPED-CONTENT`（逃逸成功） | 不做 walk 的直接后果 |
| P1e | 终段 `OPEN_REPARSE_POINT` 打开 junction 内普通文件 | 正常打开，`is_symlink=false` | 终段 reparse 拒绝用该 flag；对普通文件无害 |
| P2 | `mklink /D`（真符号链接，需 dev-mode/admin） | 本机无权限，SKIP | **环境缺项如实记录**：真 symlink 未实测；std 对 symlink 与 junction 走同一 `is_symlink` 判定路径（reparse tag 分类在 std 内一致），junction 证据覆盖检测类；Linux 腿由 CI/Linux 侧 lstat 语义（POSIX 单一路径）覆盖 |
| P3a | `open(root\NUL)` | **打开成功（设备句柄），`metadata()` 报 os error 1** | is_file 门拦不住设备名——**词法设备名拒绝必须**（含 `CON/PRN/AUX/NUL/COM1-9/LPT1-9`，大小写不敏感） |
| P3b | `open(root\NUL.txt`) | 文件不存在（现代 Win32 不做带点设备名解析） | 拒绝裸设备名即足够；带点形态顺带拒绝（belt&braces） |
| P4 | 句柄打开（share READ\|WRITE\|DELETE）后 rename-over + 新内容 | 句柄读回 `AAA-original`（原内容）；新开读到 `BBB-replacement` | **同句柄服务钉住原表示**——metadata/seek/read 同句柄语义成立 |
| P5 | 打开后 set_len 截断 | promised=4096 实读=100 | **提前 EOF 必须报发送失败**，不得按正常完成 |

### 冻结的打开算法（`safe_open_file`）

1. **词法校验**（纯函数，a2r-std server_file）：relative 拒绝——绝对路径（`/`、`\` 开头）、Windows
   drive（`X:`）、UNC（`\\`）、NUL、`..` 段、设备名段（基名含扩展剥离后匹配）、空串。命中 →
   构造期 `PathRejected`（adapter 映射 **403**，不泄露主机路径）。
2. **逐段 no-follow walk**（宿主执行）：对每个目录前缀 `symlink_metadata` → 必须存在、是目录、
   **非 symlink**（Windows 覆盖 junction+symlink；Linux 同一函数即 lstat）。命中 → 403。
3. **终段打开**：`OpenOptions::read(true)`，Windows 加
   `FILE_FLAG_OPEN_REPARSE_POINT(0x00200000)`（P1e：对普通文件无害；终段为链接时句柄指向
   reparse point 本身，`file_type().is_symlink()` → 403）。随后**句柄 metadata** 判定
   `is_file()`（目录/其他 → 404）。
4. **同句柄服务**：len/mtime/seek/read 全走该句柄（P4：路径替换不换文件；P5：EOF<len →
   发送失败）。转 tokio 句柄经 `tokio::fs::File::from_std`。
5. 残余竞态（明示）：中间段在 walk 之后、更深 open 之前被换成链接——Windows 无 std 级
   句柄相对 open，无法完全闭合；威胁边界=拥有 root 写权限的本地竞态者（计划 §5.3 已
   豁免）。Linux 腿 CI 覆盖 lstat 语义。
6. 解码次数：VM `match_route` 对 `:param` 做**一次** url_decode（http_server.rs:214）；
   axum Path 同样一次。双重编码输入解码后含字面 `%2e%2e` → 与磁盘文件不匹配 → 404
   （wire 测试覆盖）。

## 3. 模块拆分与数据流（冻结）

```text
api.at: `#[api] fn get_file(name str) FileResponse { return http.file_response(root, name, "{}") }`
  │
  ├─ VM 腿：native `http.file_response`(id 9936) → 描述符登记（VM_FILE_RESPONSES）
  │   → marshal（声明返回类型门 record_api_return_type contains "FileResponse"
  │     + 登记命中，单次取出）→ ApiReply::Full{ body: ApiBody::File(desc) }
  │   → transport：scope 不立即完成（SSE 同形）→ serve_file_response(desc,
  │     method, req_headers).await → FileReply → axum 0.8 Response（body 持 scope，
  │     EOF/Drop/错误 → complete_scope）
  │
  └─ a2r 腿：trans/rust.rs 发射 `a2r_std::http::file_response(root, name, "{}")`
      （qualify_type_name: FileResponse → a2r_std::http::FileResponse）
      → auto-man api_gen 文件分支：handler 签名 `-> axum::response::Response` +
        `method: axum::http::Method` + `headers: axum::http::HeaderMap` 提取器 +
        尾 `return auto_lang::http_file_service::serve_file_response(值, method,
        header_pairs).await`（生成 crate 自己的 axum 0.7 组装 Response）
```

- **`crates/a2r-std/src/http/server_file.rs`**（新）：`FileResponse` owned 描述符（root/rel/
  options/构造期错误，零 I/O 零 VM 引用）+ 严格 options 解析 + 纯协议决策（Range 解析、
  前置条件评估、If-Range、响应头策略、HTTP 日期格式化/解析）。依赖仅 std+serde。
- **`crates/auto-lang/src/http_file_service.rs`**（新）：宿主执行——准入/排队（Semaphore）、
  safe_open_file（§2）、prepare（同句柄 metadata → 条件/Range → FileReply）、
  `FileBodyStream`（读盘窗口流 + fs op 信号量 + idle watchdog + scope/许可 guard）、
  资源计数探针（测试）。`lib.rs` 导出。**不含 axum 类型**（FileReply=纯数据+流），
  供 0.7/0.8 两代消费端共用。
- **`crates/auto-lang/src/vm/ffi/http_server_file.rs`**（新）：VM 桥（描述符注册表 + shim +
  scope 组收口），模板=http_transfer.rs。
- 消费端 axum 组装：VM=http_transport.rs（0.8）；生成 crate=api_gen 发射的小胶水
  （0.7）。`FileReply { status, headers: Vec<(String,String)>, body: Option<FileBodyStream> }`；
  HEAD/304/412 → body=None。

## 4. 协议决策冻结（§5.2/§5.3 落地）

- **GET 200**：identity 字节；Content-Length=句柄表示长度；Accept-Ranges: bytes；
  零字节文件 200+长度 0。
- **HEAD**：与 GET 同表示 metadata（同前置条件/同 Range 忽略），wire body 0。VM 侧
  match_route 二遍扫描（HEAD 命中声明文件返回的 GET 路由；显式同路径 HEAD 路由先匹配
  优先）；生成侧 `get(h).head(h)` 链式挂载。**文件端点仅 GET/HEAD**：VM dispatch 对
  其他方法 405 诊断；api_gen 提取期 eprintln 诊断 + 405 handler。
- **Range（单区间）**：`bytes=S-E`/`S-`/`-N`；end>EOF 截至 EOF；suffix≥len → 整文件 206；
  `-0`/S≥len/S==EOF/零字节文件带 Range → 416 + `Content-Range: bytes */len` + 空体；
  未知单位/畸形（含 S>E）/多区间 → **忽略 Range → 200**（不用错误 206 冒充）。
  全程 checked u64。
- **前置条件先于 Range**（RFC 9110 §13.2.2 顺序）：If-Match（`*`=存在即真；列表仅强
  etag 匹配；无 etag → 412）> If-Unmodified-Since（仅无 If-Match；lm>date → 412）>
  If-None-Match（`*` 存在 → 304；弱比较允许）> If-Modified-Since（GET/HEAD；lm≤date →
  304）。日期仅接受 IMF-fixdate。**mtime+len 永不冒充强 ETag**：默认不发 ETag，
  Last-Modified 恒来自句柄 metadata；应用显式 `etag`（options，拒绝 `W/`）才发。
- **If-Range**：仅当 Range 将产 206 时评估；强 etag 匹配 → 206，失配/弱标签 → 完整
  200；HTTP-date 形态：`lm 截秒 ≤ date 截秒` → 206，否则 200；无法解析 → 200。日期
  验证器秒级分辨率限制写入 Spec（同秒改写不可分辨——同尺寸同秒改写探针在测试矩阵）。
- **错误映射**：缺失/非普通文件 404；越界/链接/reparse/设备 403（不泄露主机绝对路径）；
  坏应用 options（构造期 InitError）500；打开/读取内部故障 500（headers 前真实状态，
  headers 后终结 body+记 request id）；额度满/排队超 503+Retry-After。auth/middleware
  先于打开（VM dispatch 链不变——文件打开在 transport 侧，天然晚于 owner 内全部段）。
- **options（严格 JSON，deny_unknown_fields）**：`content_type`（覆盖推断，header 安全
  校验）、`download_name`（去路径成分；非 ASCII 走 RFC 5987 `filename*=UTF-8''…`，
  ASCII 回退名净化）、`disposition`（"attachment"|"inline"；默认：有 download_name →
  attachment，否则不发 Content-Disposition）、`etag`（应用供强验证器，拒 `W/` 与非法
  header 值）。第一期不开放 Content-Length/Content-Range/Transfer-Encoding 覆盖与压缩。
  MIME 默认 mime_guess（宿主侧），不明 → application/octet-stream。
- **响应头注入安全**：所有注入值拒 CR/LF/NUL/非 ASCII（download_name 经编码除外）；
  恶意文件名/头值 wire 测试覆盖（Unicode、引号、换行）。

## 5. 配额/期限/缓冲冻结（§5.4 落地）

```rust
FileServeLimits { max_active: 4, queue_capacity: 16, app_block_bytes: 64KiB,
  max_pending_blocks: 2, fs_ops_max: 4, prepare_timeout: 30s, body_idle_timeout: 60s,
  total_body_timeout: none }
```
- env 覆盖（首读一次）：`AUTO_HTTP_FILE_ACTIVE/QUEUE/BLOCK/PENDING/FS_OPS/PREPARE_MS/
  IDLE_MS/TOTAL_MS`；解析失败回默认；0 不当无限（TOTAL_MS=0 显式无限并在 Spec 注记，
  其余 0 → 回默认）。
- 排队无句柄/无缓冲，队满即时 503；**从进入文件准备起**（等许可+open+metadata+seek）
  30s 总期限，与 scope.deadline 取更早者（select: sleep + cancel_notify + scope deadline）。
- **两期限交接**：handler 30s deadline 不误杀大下载——reply 送达后 scope 移交 body；
  body 期 idle watchdog（60s，每次活动重置）由**独立 tokio 计时任务**驱动（覆盖
  body 不再被 poll 的黑洞客户端；非仅读盘 await 内 timeout）；EOF/Drop/确证断连/
  watchdog/shutdown 恰一次收口（complete_scope + 许可/句柄释放，原子幂等）。
- 读盘：读驱动（poll_next 才读）单块 ≤64KiB → 应用侧在途块 ≤1 + 框架帧缓冲 ≤1 ≈
  每 active ≤2 块；fs op 并发 ≤4（信号量围每个读操作；tokio fs 内部 blocking pool）。
  **缓冲公式（每 active）**：1×app_block(≤64KiB) + 1×hyper 帧在途(≤64KiB) + 描述符
  O(1) + 框架另行缓冲（不计入应用预算声明）。
- 在途 FS 不强断：watchdog/取消 = 停止 issuance + 标记终结；读 future 由 poll 自然
  收口；执行槽（active 许可）在流 Drop（=最后一个读 await 退出）才归还——逻辑取消
  与实际 FS 退出分开计数（探针分别断言）。
- shutdown：现有 watch 关停面；文件 body 流在 shutdown 信号时收尾当前块后终结
  （drain 窗语义沿 serve_with 现状）。文件配额独立：满额/慢下载时 /health 与 CRUD
  不等文件许可（独立信号量验证）。

## 6. 生成/消费形态矩阵（诊断冻结）

| 形态 | 行为 |
|---|---|
| VM 默认 HTTP `#[api]` GET/HEAD（同步/`~` 异步） | 支持；auth/middleware/请求 ID/CORS 不变 |
| auto-man 生成 Rust HTTP | 支持；同 api.at 生成文件 adapter；转译失败**诊断到 api.at 位置不落模板**（新分支在 fallback 链最前） |
| TS HTTP 客户端 | 文件方法返回原生 `Response`（不 `.json()`） |
| Tauri IPC | 生成期明确 Unsupported 诊断（"file endpoints require HTTP transport; use the HTTP URL"） |
| VM 进程内/merged fn 直调 | 构造无害（零 I/O）；值是 int 句柄——经 IPC/JSON 序列化面明确拒绝（back_proxy：文件端点 Unsupported 响应） |
| legacy stdnet/Builder | 未接入：marshal File→500 诊断 "file responses require the default HTTP transport"；`Server.static` 维持占位不支持 |
| api/targets/axum.rs 单独生成器 | 加 FileResponse 分支（Response glue 同形）；标注"生成物验证于 auto-man fixture" |
| 普通 int 撞号 | 声明返回类型门 + 登记命中共同判定；声明文件返回但值非登记句柄 → 500 诊断（不 JSON 200） |

## 7. T-01 验证记录

- 探针：§2 表（Windows 实测，junction/NUL/rename/truncate 四族证据；真 symlink 环境
  缺项如实记录）。
- 基线核对：§1 表。
- 最小真实编译原型（api.at → a2r+auto-man → 二进制 adapter → GET/HEAD wire）：所需
  全部代码在 T-02..T-05 落地，wire+编译验证在 T-06 e2e fixture（`http_e2e_plan729` 族）
  统一交付——本报告冻结的拆分/门/形态即其设计。设计可同时满足 AC-01..06、SD-01..07
  （映射：§2→AC-03，§3→AC-01/06，§4→AC-02，§5→AC-04/05，§6→AC-06）。
- 平台安全无需超范围重构；无 needs_replan 项。

## 8. 依赖/feature 决策

零新依赖。a2r-std server_file：仅 std+serde（现有）。auto-lang http_file_service：
tokio/fs（现有 full）、futures（现有无条件）、mime_guess（现有）。不启用
streaming-http；不改 ui 依赖方向；`cargo check --no-default-features` 不受影响
（http_file_service 不引用 ui 族符号）。
