# HTTP 服务端文件响应（GET/HEAD 下载面）——stdlib 服务端文件契约

> 路径：`crates/a2r-std/src/http/server_file.rs`（描述符+纯协议决策单源）+
> `crates/auto-lang/src/http_file_service.rs`（宿主执行：受限打开/准入/有界 body/收口）+
> `vm/ffi/http_server_file.rs`（VM 桥）+ auto-man/api_gen 文件分支（生成 Rust 腿）
> 状态：active（PLAN-729 交付；决策/验证报告见 `docs/plans/reports/729-server-files-*.md`）

## 1. 公共面（Auto / VM / 原生 Rust 同词汇）

```text
type FileResponse                                            # opaque owned 响应描述符
http.file_response(root str, relative_path str, options str) FileResponse
```

- 构造**零 I/O**、零 VM 引用：只保存配置 + 构造期纯校验结果。坏 options（未知键/错类型/
  非法头值/弱 `W/` etag）与坏路径（`..`/绝对/drive/UNC/NUL/DOS 设备名）不产生"成功描述符"
  ——`init_error` 携带原因，HTTP adapter 在发送 headers 前映射（Options→500 / Path→403）。
- 从命名 `#[api]` GET/HEAD handler 返回该描述符 → HTTP 层以文件响应执行（VM 默认 HTTP 与
  生成 Rust/Axum 服务同一执行代码）；网络与读盘不在 VM owner 中执行。
- 返回类型识别 = 声明返回类型（`FileResponse`/`Future<FileResponse>`）+ native 登记**共同
  命中**：普通 int 即使数值撞上描述符 id 仍是普通 int（JSON 兜底）；声明文件返回但值非
  登记句柄 → 500 诊断。

## 2. options（严格 JSON；加字段须走 Spec delta）

```json
{"content_type": str, "download_name": str,
 "disposition": "attachment"|"inline", "etag": str}
```

- 空对象 `{}` 合法（空串同）；未知键/错类型 → 500（不静默回退默认）。
- `content_type` 覆盖 MIME 推断（宿主 mime_guess，不明 → `application/octet-stream`）。
- `download_name`：剥离路径成分；CR/LF/NUL 拒绝；非 ASCII 走 RFC 5987
  `filename*=UTF-8''…` + 净化 ASCII 回退。默认（无名且未显式 inline）不发
  Content-Disposition。
- `etag`：应用提供的**强**验证器（拒 `W/`）。**mtime+len 永不冒充强 ETag**：默认不发
  ETag，Last-Modified 恒来自打开句柄 metadata。应用供 etag 时由应用保证其对应稳定内容
  版本（同尺寸同秒改写不可证明一致——秒级分辨率限制，见 §4 日期）。
- 第一期不开放 Content-Length/Content-Range/Transfer-Encoding 覆盖、压缩、多区间。

## 3. 协议子集（RFC 9110；位置/长度 u64，checked）

- **GET 200**：identity 字节；Content-Length=打开句柄表示长度；`Accept-Ranges: bytes`；
  零字节文件 200 + 长度 0。
- **HEAD**：按 GET 同表示 metadata（同前置条件）；忽略 Range；wire body 0 但
  Content-Length 保持表示长度。GET 文件路由自动具备 HEAD（显式同路径 HEAD 路由优先）。
  **文件端点仅 GET/HEAD**：其他注解方法 → 405 + `Allow: GET, HEAD`（VM dispatch 与
  生成器双面）。
- **单区间 Range**：`bytes=S-E`/`S-`/`-N`；end 越 EOF 截至 EOF；suffix≥len → 整文件 206；
  `-0`/S≥len/S==EOF/零字节文件带区间 → 416 + `Content-Range: bytes */len` + 空体；
  未知单位/畸形（含 S>E）/多区间/数值溢出 → 忽略 Range → 200（不用错误 206 冒充）。
- **前置条件先于 Range**（§13.2.2 顺序）：If-Match（`*` 存在即真；列表仅强匹配；无 etag
  → 412）> If-Unmodified-Since > If-None-Match（`*`→304；弱比较允许）>
  If-Modified-Since（GET/HEAD）。日期仅接受 IMF-fixdate；坏日期忽略。
- **If-Range**（仅当 Range 将产 206 时评估）：强 etag 精确匹配 → 206；失配/弱标签 → 完整
  200；HTTP-date：`lm 截秒 ≤ date 截秒` → 206，否则 200；无法解析/无验证器 → 200。
- **错误映射**：缺失/非普通文件 404；越界/链接/reparse/设备 403（不泄露主机绝对路径）；
  坏应用 options 500；打开/读取内部故障 500；队满/排队超 503 + `Retry-After: 1`。
  已有 auth/middleware 先于打开。headers 后故障只能终结 body/连接（记 request id），
  不得伪造 JSON 500。
- **同句柄服务**：metadata/seek/read 用同一打开句柄（路径替换不换文件；句柄被截断导致
  提前 EOF = 发送失败，不当正常完成）。

## 4. 受限根目录打开

- root 来自应用受信配置；relative_path 可来自路由参数（不可信面）。
- 算法：词法校验（§1）→ 逐段 no-follow walk（`symlink_metadata` 检出 symlink/junction；
  非目录 404）→ 终段打开（Windows `FILE_FLAG_OPEN_REPARSE_POINT|BACKUP_SEMANTICS`；
  终段为链接 → 403）→ 句柄 is_file 门（目录/设备/管道 404）。
- 解码次数：路由参数**一次** url_decode（VM match_route / axum Path 同语义）；双重编码
  输入解码后含字面 `%2e%2e` → 不匹配磁盘 → 404（wire 验证）。
- 残余竞态（明示）：中间段在 walk 后被换成链接的窗口——Windows 无 std 级句柄相对打开；
  威胁边界=拥有 root 写权限的本地竞态者（不抵抗主机管理员）。Linux 腿 lstat 语义同源。
- 注入值（头值/download_name）拒 CR/LF/NUL/非 ASCII（RFC 5987 编码除外）。

## 5. 配额/期限/缓冲/收口

```text
max_active=4  queue_capacity=16  app_block=64KiB  max_pending_blocks=2  fs_ops=4
prepare_timeout=30s  body_idle_timeout=60s  body_total=none
env: AUTO_HTTP_FILE_{ACTIVE,QUEUE,BLOCK,PENDING,FS_OPS,PREPARE_MS,IDLE_MS,TOTAL_MS}
（0/非法回默认；TOTAL_MS 显式 >0 才启用；不把 0 当无限——与传输限额同规约）
```

- 排队期零句柄零缓冲；队满即时 503。准备期限（等许可+open+metadata+seek）与调用方
  deadline 取更早者（VM 腿 = min(30s, scope 剩余)）。
- 两期限交接：handler 30s deadline 不延伸进 body；body 期 idle watchdog（独立计时任务，
  覆盖不被 poll 的黑洞客户端）+ 可选总期限。EOF/错误/Drop/断连/超时恰一次收口
  （VM 腿 = scope 幂等终结；生成腿 = 服务级资源回收）。
- pump 读驱动（poll 才读）：每 active 待发送 ≤2 块（在读 1 + 通道 1）；fs op 并发 ≤4。
  在途读不强制中断：停止 issuance、等收口后退出，执行槽（active 许可）随 pump 退出归还
  （逻辑取消与 FS 实际退出分开计数）。
- 文件配额独立于普通请求/SSE/传输配额；慢下载不阻塞 `/health`/CRUD。
- 304 的 Content-Length 由 hyper 剥离（wire 现实；表示长度语义由 200/HEAD 承载）。

## 6. 调用形态支持矩阵

| 形态 | 状态 |
|---|---|
| VM 默认 `#[api]` GET/HEAD（同步/`~` 异步） | ✅（auth/middleware/请求 ID/CORS 不变） |
| auto-man 生成 Rust HTTP（axum 0.7） | ✅（同 api.at 生成文件 adapter；转译失败 = 位置诊断 500，**不落模板**） |
| `api/targets/axum` 单独生成器 | ✅ 生成（Response glue 同形；实编验证于 auto-man fixture） |
| TypeScript HTTP 客户端 | ✅ 文件方法返回原生 `Response`（不 `.json()`；状态/headers 保留） |
| Tauri IPC | ❌ 明确 Unsupported 诊断（Err + 改走 HTTP URL 指引） |
| 进程内 back-proxy | ❌ 501 明确拒绝（描述符非可序列化数据） |
| legacy stdnet / `Server.static` | ❌ 500 诊断 / 占位不支持（本期非交付面） |
| multipart/byteranges、压缩、目录列表 | ❌ 非本期面 |

## 7. 跨实现一致性（单源）

描述符/选项/Range/条件/If-Range/HTTP 日期/响应头策略只在
`a2r_std::http::server_file`；受限打开/准入/pump/watchdog/收口只在
`auto_lang::http_file_service`（**不含 axum 类型**——版本无关 `FileReply` 投影，
0.7/0.8 两代消费端各自组装 Response）。VM 桥（shim 9936 + `ApiBody::File` +
声明返回类型门）与生成腿（trans lowering `a2r_std::http::file_response` + api_gen
文件分支）都只做投影，不复制协议决策。
