---
plan_id: PLAN-730
status: archived
feature_name: http-server-upload-ingress
author: [agent]
created_at: 2026-10-02
updated_at: 2026-10-02
plan_revision: 1
current_step: 9
total_steps: 9
supersedes_spec_components:
  - docs/specs/stdlib/project.md
  - docs/specs/stdlib/design/http-server.md
  - docs/specs/stdlib/design/backend-assembly.md
  - docs/specs/stdlib/design/http-handler-async-lifecycle.md
  - docs/specs/a2r-std/project.md
  - docs/specs/auto-man/project.md
  - docs/specs/auto-lang/trans/overview.md
new_spec_components:
  - docs/specs/stdlib/design/http-server-uploads.md
  - docs/specs/auto-lang/trans/design/http-upload-lowering.md
touched_goals: [GOAL-003]
affects: [stdlib/auto/http.at, stdlib/auto/http.vm.at, crates/a2r-std/src/http, crates/auto-lang/src/a2r_std.rs, crates/auto-lang/src/vm/ffi, crates/auto-lang/src/vm/codegen.rs, crates/auto-lang/src/vm/native.rs, crates/auto-lang/src/trans/rust.rs, crates/auto-lang/src/api, crates/auto-man/src/api_gen.rs, crates/auto-lang/src/back_proxy.rs]
---

# [PLAN-730] HTTP 服务端上传：流式接收、业务校验、可靠提交与取消

## 0. 变更摘要

接续 [Design 33](../design/33-stdlib-runtime-and-http.md) D1b，补 **server 接收客户端文件上传**。727 已交付客户端上传/下载，729 正在实施服务端文件下载；本期提供普通 `api.at` 的 multipart/raw 流式上传入口，覆盖 VM 默认 HTTP 与生成 Rust/Axum HTTP，同源客户端和服务器实际互通。

核心合同：路由/鉴权先于落盘，网络与磁盘增量处理，接收先进入不可公开下载的 staging，完整解析后允许业务校验，再显式提交；文件失败/超限/断连/取消不假成功、不覆盖既有文件。不能只把全量 body 上限放大或用现有 `parse_multipart(Vec)` 宣称流式上传。

本期提供**单文件 + 有界文本字段**与 raw 单文件，不承诺多文件事务、断点分片/tus/S3、自动内容安全扫描、磁盘全局永久配额/崩溃恢复、Builder 完全等价、IPC 传文件或直接公网部署等级。旧小体积 multipart 保留兼容成功形状，但必须消除路由/中间件之前写盘和写失败假成功。

## 1. 目标

- G1：命名 `#[api]` POST/PUT handler 接收 HTTP 注入的 `UploadRequest`，先做 header/path/query 鉴权，再调用异步接收；VM park/resume 和生成 Rust await 都不阻塞 owner/reactor。
- G2：multipart 与 raw 增量接收；boundary/part header、文本字段、单文件、请求总量、缓冲、并发、排队与期限均有上限。
- G3：接收 → staging → 业务校验 → commit/reject 明确分阶段；最终目标由受信 handler 选择，默认原子 create-only，存在即 409，绝不先删/覆盖旧文件。
- G4：输入结束、解析/磁盘故障、Drop、断连、deadline、shutdown 一次收口；提交临界点与取消仲裁准确，未提交临时文件清理可观测。
- G5：真实生成的 api.at 上传 handler 与 VM 协议、错误、资源一致；与 727 上传客户端、729 下载路由组成实际上传后下载验证。
- G6：旧 multipart 路径鉴权与写盘顺序、错误处理有定向修复；其他 JSON/SSE/文件下载保持既有行为。

只涉及本仓；auto-down 兄弟检出只读用于解析，不修改 AutoUI 应用业务/auto-musk。存储、鉴权、最终命名与内容校验是应用策略；库提供有界执行与提交语义。

## 2. 架构方案

```text
加载 api.at → 上传能力由参数类型确定的路由表（不按 Content-Type 猜能力）
  HTTP headers → 有界准入/scope → owner 路由、中间件、header/path/query 校验
    └─ UploadRequest：一次性请求体能力；body 保留在宿主，未授权不解析/写盘
        └─ upload_receive → 共同宿主 parser + 有界磁盘写 → UploadSession
            └─ handler 读有界字段/文件 metadata 做业务校验
                ├─ upload_reject → 清理 staging → 错误 UploadReceipt
                └─ upload_commit → 根目录内原子 create-only → 成功 UploadReceipt
                    └─ HTTP adapter：真实 status + 有界 JSON，不返回资源 id
```

拟新增 `crates/a2r-std/src/http/server_upload.rs`：公共 owned 类型/options/receipt 与 async facade；不反向依赖 auto-lang。拟新增 `crates/auto-lang/src/http_upload_service.rs`：共同接收、multipart/raw、staging/commit 和资源 guard；VM 与 auto-man 使用同一宿主代码。复用 729 已验证的 root/capability、HTTP 类型分类及响应适配基础，不复制安全路径算法。

请求体/socket/HTTP 对象和文件操作留在宿主；跨 VM owner 桥只传 owned metadata、有类型 id/控制消息与有界结果，不传 `Rc<VM>` 或裸指针。VM 等接收/提交以既有 live-op/完成通知 park/resume，Rust 在服务器 runtime await；新操作不复用客户端 FileTransfer id，不每上传新建线程/runtime。

T-01 用真实编译原型冻结公共 ABI 和 a2r-std 到宿主 executor 的 owned async hook（或保持依赖方向的等价拆分）；不得依靠全局不受控 callback、不透明整数碰撞、字符串控制哨兵或同步 blocking 兜底。

## 3. 技术栈

- 现有 Axum 0.8/Hyper、Tokio 文件和通知/队列；复用 server runtime 与既有 VM 完成协议。
- 优先 `multer` 3.1 系增量 parser + 显式 Constraints；T-01 固定实际可兼容版本和依赖。boundary/header/carry/part 个数等仍需额外验证与外层预算，不因依赖库存在而宣称自动有界。
- 729 的受限根目录访问机制；发布需要 create-only/no-replace 的平台原语，不把普通 `rename` 等同于不覆盖。
- RFC 7578 的 multipart/form-data 子集；仅 identity request Content-Encoding，非 identity 明确 415。HTTP framing 仍由 Hyper，不引入另一套 TCP 解析器。
- scoped、真实 TCP、磁盘注入/gate、生成产物编译运行；端口临时分配，OS 链接测试在独占临时目录内，worktree 内禁止链接。

## 4. 需求分析与背景调查

### 4.1 来源、版本、授权与依赖

起草主检出基线 `e5b068bd0`；730 取号提交 `1a15c8eee`（exclusive allocation，活跃/归档唯一，计数到 731）。用户说明 729 已开始；已发现 `D:/autostack/.wt/lang-729/auto-lang`，起草期间其簿记从 drafting 更新为 executing:r1（0/8）。不能把工作开始等同于已合入或复审通过，其 proposed Specs 尚未成为 canonical；本轮保留其他会话对729的在途簿记。

| 来源 | 本计划依据 |
|---|---|
| `docs/specs/overview.md`、`docs/specs/stdlib/project.md`、`design/backend-assembly.md` | stdlib 装配层和宿主实现分离；VM、生成 HTTP、IPC/merged/back-proxy 各有边界。 |
| `docs/specs/stdlib/design/http-server.md` §4.1/§8.1/§10 | 参数注入与普通 body 规则；server multipart 的完整增强仍列后续，不等于没有任何 legacy parser。 |
| `docs/specs/stdlib/design/http-handler-async-lifecycle.md` | owner 段等待、typed metadata、scope/完成通知；半关闭不是取消证据。 |
| `docs/specs/stdlib/design/http-file-transfer.md`、`docs/specs/a2r-std/design/http-client-runtime.md` | 727 client 上传、shared core、FS 不可强制 abort 与提交竞态，可作为互通和取消基线。 |
| `docs/specs/auto-lang/trans/design/http-client-lowering.md`、`docs/specs/a2r-std/project.md`、`docs/specs/auto-man/project.md` | 两 facade、typed async lowering 和 API 生成职责；不能 fallback 模板当成功。 |
| [PLAN-729:r1](729-http-server-file-responses.md) §2/§5/§10 | 下载、安全根目录和响应体生命期；上传被明确排除。730 承接其未来最终接口，不提前修改其实施文件。 |
| [RFC 7578](https://www.rfc-editor.org/rfc/rfc7578.html) §4/§5/§7 | multipart boundary/字段/文件名语义；客户端文件名不能直接作为存储路径，重复字段需要明确处理。 |
| [multer Multipart](https://docs.rs/multer/3.1.0/multer/struct.Multipart.html)、[Constraints](https://docs.rs/multer/3.1.0/multer/struct.Constraints.html) | stream 分块解析与 whole/per-field 约束；默认 Constraints 不施加限制，不能裸用默认。 |
| [Axum Multipart](https://docs.rs/axum/latest/axum/extract/struct.Multipart.html) | extractor 的默认 body 限制需要单独处理；只对上传路由调节，不能放开普通 JSON 的限制。实施验证实际锁版本。 |

用户授权本轮继续**规划下一步**；允许本仓只读调查及计划/设计簿记，无实施、部署或修改其他计划执行状态的授权；无用户预算/自动续跑限制。

**执行依赖：729 reviewed 且 merged 后再开始 730 work**。两计划同触 `http_transport/http_server`、typed API 生成和根目录基础；730 现在可以完整起草，不能并行实现这些文件后以合并冲突代替契约对接。729 如发生语义修订，T-01 对照其最终 Spec/review receipt，必要时修订 730 revision。

### 4.2 已核查的源码差距

| 文件/符号 | 观察（静态，尚未运行复现） | 对应设计 |
|---|---|---|
| `vm/ffi/http_transport.rs::bridge_handler` | `to_bytes(body, 10MiB)` 收完整 body 后才建 scope/入 owner 队列；不能用于大文件流式 intake。 | 仅已声明上传能力的 route 走 header-first 有界准入与 lazy body；普通 JSON 上限保持。 |
| `vm/ffi/http_server.rs::parse_multipart` | 扫全量 Vec，分隔/header 解析简化，各 part 复制；没有完整错误结果。 | 新上传路由只走共享增量 parser；分块和截断 wire 矩阵。 |
| 同文件 `store_multipart_file` / `multipart_to_handler_json` | create_dir/write 错误被忽略仍返回路径；解析落盘发生于 `match_route` 与 middleware 之前。 | 未匹配/拒绝不得写盘；legacy write 错误必须失败，写盘移出 owner。 |
| 同文件 `advance_dispatch` / `run_middleware_at` / `RequestScope` | 中间件段驱动与流/transfer 资源组已存在，但没有上传 request/session 资源组。 | 有类型关联、期限和迟到完成守卫，避免资源 id 混用。 |
| `vm/codegen.rs` API 签名采集、`api/types.rs`/`api/mod.rs` | 已有 API 参数/返回元数据；尚无 UploadRequest 宿主注入分类。 | 由类型识别，不被名为 req 的 metadata 约定或 JSON whole-body 规则误吞。 |
| `auto-man/src/api_gen.rs::endpoint_body_params` / `is_meta_param` | 除 meta str 外普通 POST 参数按 JSON body 编组，未见 UploadRequest 流分支。 | 排除注入参数、body stream 放最后，真实 handler 注入与错误 status 适配。 |
| `a2r-std/src/http/transfer.rs` | 客户端 staging/取消/提交已有实现；其 replace 语义不能直接用作 server create-only。 | 复用已证明原理/工具，另验 root-relative/no-replace/服务端事务。 |

当前没有通用 server-upload canonical Spec，SD-01 新增。旧 parser 的静态问题须 T-01 复现后改良，不宣称本轮已修复；不在主检出跑 Cargo。

## 5. 详细设计

### 5.1 公共能力与调用形态

拟采用以下**语义草图**（T-01 冻结合法 Auto/Rust 签名、借用/所有权与 ABI，不要求照抄为语法）：

```text
UploadRequest   # HTTP 注入、一次消费、不能 JSON 构造
UploadSession   # 收完整请求后仍未公开的 staging；也可携带接收失败
UploadReceipt   # 完成后的 owned status/有界 JSON，非文件资源
upload_error(status, message) -> UploadReceipt                     # 无I/O的早拒构造
upload_receive(req, trusted_root, strict_options) -> UploadSession   # 可等待
upload_metadata(session) -> str                                    # 有界 JSON
upload_commit(session, server_relative_target) -> UploadReceipt    # 可等待
upload_reject(session, status, message) -> UploadReceipt            # 等清理收口
```

receive/commit/reject 在 `~UploadReceipt` named handler 内按 Auto 等待语义 park；a2r async 发 await，不能偷偷阻塞 reactor。同步 Rust 桥不作为首期服务器入口；不支持的同步调用须诊断。UploadRequest 只能由宿主注入，一次消费；重复 receive/commit/reject 有明确冲突终态，资源 handle 不允许跨请求继续使用。

HTTP 返回 UploadReceipt，由专用适配器产生真实 status + `application/json`。成功 201，body 含 relative path/文件 size/客户端 filename 仅作为 metadata/有序文本字段；不暴露 root/staging/绝对路径。接收失败的 session 能 inspect/reject，但不能再 commit 为成功。错误 receipt 含稳定 kind/message；计数字段内部 u64、JSON 字节数用十进制字符串，TS 不发生精度丢失。Receipt 自身不持在途文件资源，拷贝不重复提交。

handler在receive之前使用纯`upload_error(401/403/其他合法4xx或5xx, message)`返回typed早拒，不必为了构造结果先接收文件；该函数不等待、不打开文件、不能创建伪201成功。metadata/早拒构造与receive/commit/reject的sync/async发射分开验证。

VM 默认 HTTP 和 auto-man 生成 Rust 支持命名 POST/PUT `~UploadReceipt` handler（一个 UploadRequest 参数，可有 path/query/meta）。不允许再同时声明 JSON whole-body 注入，加载/生成阶段指名诊断；字节流不能从 JSON/query 填造。HTTP TS 上传方法接收 FormData/Blob/ReadableStream 等实际 body，删除注入参数、不得 JSON.stringify(FormData)，让浏览器生成 multipart boundary；各平台实际消费点由 T-01 定位。

IPC/merged/back-proxy、legacy stdnet/Builder 未接入新流能力时明确 Unsupported/源码诊断及 HTTP URL 指引，不隐式序列化请求句柄。普通 API 不受影响，单独 AxumGenerator 若未做实编验证仍标限制。新上传方法体转译失败必须报源码位置，不能回 CRUD 模板。

### 5.2 header-first、鉴权与 body 所有权

加载时从 UploadRequest 参数元数据形成 method/path 上传路由 policy；网络预分类与 owner 使用一致路由匹配，不通过 Content-Type 或 URL 子串把任意接口升级大 body。默认普通 JSON/legacy 小 body 仍为 10MiB/原期限，新文件额度只对上传 capability route 生效。

上传 headers 到达后先获得既有总请求许可、登记 scope 和有限 body capability，再入 owner；未接受 receive 前不解析 part、不打开临时文件、不主动拉取整个 body。框架可能已读入少量网络字节，不等同于授权后的 intake。route/middleware 或 handler header/path/query 鉴权拒绝直接释放 body/资源；返回拒绝时不进行无界 drain。以真实 `Expect: 100-continue` 验证客户端无需先发送文件；继续响应/未读体的连接复用或关闭策略明确，由 Hyper 承担 framing。

显式业务校验分两层：认证/目标 root 选择在 receive 之前；文件 metadata/文本字段/内容业务检查在 staged 之后、commit 之前。multipart 字段中的“授权/路径”不是可信 header 配置，不能自动成为磁盘目标；库不自行提供认证机制。

新 typed 上传路由的 middleware 执行错误须 fail-closed（500/取消，零落盘），不能沿用 legacy Err→Continue 的静默放行；正常空返回继续链的语义保持。生成服务的应用鉴权由同源 handler/已支持middleware实际执行，不能从 `auth` 注解存在就宣称生成器自动实现授权。T-01 核实鉴权路径，测试绑定其真实拒绝与异常行为；不扩大为全面重写旧 API middleware 契约。

UploadRequest body 留宿主，生成 Rust 也不得先抽取为 Bytes/Multipart.text/JSON 再调用共同核心。body channel/控制消息有上限且注册随 scope；宿主 receiver 无 await 在 owner 执行。拒绝/未消费/handler 异常、live-op 迟到都清理 capability，不保留无人消费的 producer。

### 5.3 multipart/raw 子集与预算

| 项 | 默认合同（可按受信配置降低；提高须有总资源公式） |
|---|---|
| 文件/请求 | multipart 恰 1 个声明 field（默认 file），raw 恰 1 个；单文件 64MiB，wire 总请求 65MiB（含所有 framing/字段/epilogue） |
| 文本/part | 文本 ≤16 个，单字段 ≤16KiB、合计 ≤64KiB，part ≤32；字段有序列表保留重复，不静默覆盖 |
| 解析结构 | boundary ≤70 字节；单 part headers ≤16KiB/≤32 项；header/carry/名称都有预算，坏结构、缺 closing boundary、错误 utf-8 文本为 400 |
| 流/磁盘 | active 4、等待 16（不读 body/不打开文件）；FS 在途 ≤4；应用块 ≤64KiB、最多 2 个待写块，磁盘慢反压网络 |
| 时间 | header/鉴权/队列期限各阶段有界；从 headers 入 scope 起 total 默认 10min（排队计入），接收 idle 60s，staged 等业务判定 lease 30s；关闭沿用现有 drain |

mode 显式 multipart/raw；raw 接受配置的媒体类型，缺 Content-Length 的 chunked 也受逐字节总量限制，不能只信声明值。空文件合法；multipart 无 file/第二个 file/未知不允许字段为明确错误，不偷偷只接第一份。文件 filename 可缺/空，field 决定其角色；原 filename 不作存储路径。文本按 UTF-8，其他 charset/嵌套 multipart/Content-Transfer-Encoding 的不支持形式明确拒绝；不宣称完整 MIME 框架。

严格 options 包含 mode/file_field、允许的文本字段、上述可降低的限额，以及受信 `staging_root` 配置；上传请求自身不能指定 root/staging_root/最终 target。服务器硬上限在 route policy/服务配置中，handler options 不得无界扩大；T-01 冻结名称和配置层次。

boundary/header 可跨任何 TCP/HTTP chunk，内容里的相似 boundary 前缀和假 delimiter 不能截断文件。closing boundary、剩余 epilogue 与 HTTP body 完整结束均校验/计入总量；后续 part 或超限不能发生在文件已发布之后。库 parser 的 default/field.text 全量缓冲必须被外层限制和分块读取替代。

Body framing 超限 413、媒体/编码不支持 415、坏 multipart/短体 400、接收 idle/total 408、满额/队列期限 503+Retry-After、目标冲突 409、越界/权限 403、I/O/flush/sync/发布错误 500；应用 reject 可选择受限 4xx。失败不是 HTTP 200 里写 error。零/非法预算配置不变成无限，未知 options 在读取/写盘前拒绝。

上传总期限不能被现有固定 handler 30s 提前杀死：header/鉴权仍受普通等待期限；进入 receive 后桥与 owner 同步切换到 upload total/idle/lease，显式取消仍有效。不能仅改变 scope.deadline 而让网络 select 保留旧捕获值，也不能全局关掉 HTTP timeout。T-01 必须 gate 原型和 >30s 的受控慢上传验证。

缓冲公式包括 parser carry、headers、文本、pending writes、读体原始 frame 的实际分配/retained Bytes、框架与 quota 数量；切片 len 小不能掩盖持有大原始块。per-session 的磁盘占用上限和 active+staged 的总上限一并计算，staged lease 仍占会话许可，防止业务不 commit 积累无界临时文件。

### 5.4 staging、校验与 create-only 提交

临时文件在受信 `staging_root` 独占创建，权限受限。配置采用**同卷、公开下载 root 之外的私有目录**（示例为 private-staging/public-files 两个兄弟目录）；缺少安全 staging 配置明确拒绝，不降到公开 root 下的“隐藏文件”。受信应用不能把私有目录另配为 FileResponse root；示例/测试明确验证同服务全部公开下载路径不能读取 staging，不依赖随机名保密。文件总量/字段校验、EOF 与写入完成后才返回 Received session。

业务可读 metadata/字段；提供可等待、每窗口≤64KiB 的受限 staging 内容读取用于业务检查，具体合法签名由 T-01 冻结；不能把临时绝对路径返回远端或在 owner 整文件读。reject、session Drop、lease 超时清理 staging，跨卷配置在读取前失败。

最终 relative target 由 handler 决定，复用 729 root-relative 策略并扩展安全创建/发布；拒绝 traversal、drive/UNC/device、ADS、链接/reparse 越界和目录。不能 canonicalize 后再按不可信原路径创建，也不能跟随攻击者替换的目标父目录。默认只允许配置中已存在的安全父目录，不自动按客户端字段创建目录。

commit 顺序：复核 session/期限/目标 → flush + sync → 关闭必要句柄 → 进入取消仲裁 gate → 同卷原子 no-replace 发布 → 生成 201 receipt。用平台已验证的原语；普通 rename 在部分系统覆盖目标，`exists` + rename 也有竞态，都不符合合同。不支持 no-replace 的文件系统明确失败，不退回复制覆盖。并发相同目标恰 1 成功，其余 409，原文件全字节不变。

取消在 commit gate 前胜出：不发布、收口/清理；commit gate 胜出后迟到取消不宣称失败回滚，等实际发布结果（失败仍清理，成功保留最终文件）。已经提交但 HTTP 回复丢失，客户端可能报告 transport/cancelled，不能删除已提交文件或承诺分布式恰一次；727 默认不自动重放 POST，应用重试策略另案。

磁盘操作不强制 abort：停止新写，等待在途写/发布返回后关柄/清理、释放实际 FS 许可。不能因 scope 逻辑取消就提前腾槽产生无限后台写。cleanup 失败必须有 request ID/受限运维诊断与遗留标记，不能成功日志宣称零残留；无故障矩阵须确实回基线。异常重启的陈 staging 扫描、全局存储配额与断电持久性不在本期，运行中临时空间仍有硬预算。

### 5.5 legacy 小 multipart 兼容修复

现有 `form str` 等老入口继续 10MiB 限制和历史 `fields/files` 成功形状，不能自动获得 64MiB 新能力。默认 VM HTTP 路径须先 match route/run middleware，再做必要 multipart 写盘；404、method 不匹配、middleware 拒绝不生成任何文件。解析/FS 错误明确 HTTP 400/500，不返回不存在的 file path。

legacy 存储迁入有界宿主执行，不在 owner `std::fs::write`；正常文件元数据形状/成功调用可维持，旧 parser 只能作为已界定小体积 adapter，不参与新 capability 路由。为 legacy 创建的 provisional 文件归 scope：绑定失败/handler 错误/取消清理本次新建文件，正常成功按历史语义保留。已发生的业务文件移动/数据库写等副作用不自动回滚；不能声称 legacy 获得新显式 commit 的事务语义。同步 Builder 无法异步复用时明确保留支持限制，但**所有实际调用的旧存储函数必须传播磁盘错误**。

### 规范增量

起草只提出 delta；729 merge 后 T-01 核对重叠目标，work 准备、review 验证、merge 沉淀，不覆盖 729 新知识。

| delta_id | add/modify/retire | docs/specs/... target | before/after rule | rationale | acceptance IDs |
|---|---|---|---|---|---|
| SD-01 | add | `docs/specs/stdlib/design/http-server-uploads.md` | 无通用流式上传 → request/session/receipt、接收/校验/提交、parser/预算/取消与支持子集 | 独立 server intake 契约 | AC-01..07 |
| SD-02 | modify | `docs/specs/stdlib/design/http-server.md` | multipart 完整增强列后续/legacy 参数注入 → 当前 typed HTTP 上传与 legacy 区分；默认 JSON 不扩大 | 声明实际支持和非目标 | AC-01/02/06/07 |
| SD-03 | modify | `docs/specs/stdlib/design/backend-assembly.md`、`docs/specs/stdlib/project.md` | 729 下载覆盖 → 增加上传公共声明/VM/Rust 宿主单源与非 HTTP 限制 | 不把客户端或 IPC 当 server 支持 | AC-01/07 |
| SD-04 | modify | `docs/specs/stdlib/design/http-handler-async-lifecycle.md` | 既有请求/下载资源组 → body capability/staging 资源组、分阶段期限、late completion/commit 仲裁 | scope 终结不等于在途 FS 已结束 | AC-04/05 |
| SD-05 | modify | `docs/specs/a2r-std/project.md` | client/transfer/729 descriptor → 上传 owned 类型和 async hook，不宣称 crate 自行监听 | 保持依赖方向/两 facade | AC-01/07 |
| SD-06 | add | `docs/specs/auto-lang/trans/design/http-upload-lowering.md` | 无 upload typed lowering → 宿主注入/receive-commit-reject await/typed receipt/unsupported | 同源实编、不能误用 client helper | AC-01/07 |
| SD-07 | modify | `docs/specs/auto-lang/trans/overview.md`、`docs/specs/auto-man/project.md` | 普通 JSON POST/返回模板 → 上传 body 分类、真实 status/字段、HTTP TS 与生成失败诊断 | 实际生成代码与 VM 对齐 | AC-01/07 |
| SD-08 | modify | `docs/specs/stdlib/design/http-server.md` legacy 状态/错误规则 | 旧 multipart 全 body/隐含存储 → 路由/中间件先行、有界宿主写盘、错误/新建文件清理 | 兼容成功形状，消除静态缺口 | AC-03/05/06 |

## 6. 测试设计

### 6.1 真实双端协议与存储矩阵

同一 api.at 在 VM 默认 HTTP 与 auto-man 实际生成/编译/运行的 Rust 服务上执行，不用手写 Axum route 替代：

| 族 | 样本和期望 |
|---|---|
| 上传 | multipart 单文件+字段、raw、0B、含 NUL/无效 UTF-8 文件、12MiB（>普通10MiB）、Content-Length/chunked/无声明长度；201+最终 hash 相同、字段有序、无 JSON/base64 文件缓冲。 |
| 解析 | boundary/header 每字节切块、quoted boundary、假前缀/相似 delimiter、字段重复、filename 缺失/空/Unicode、缺 file/第二 file、截断/坏 headers/closing/epilogue；合法逐字节正确，错误不发布。 |
| 预算 | file/body/text单项与合计/parts/header/carry 各临界±1；声明长度与实际不一致、超限 chunked；413/400/415 准确且临时文件清零。普通 JSON 大体积依旧413。 |
| 鉴权/早拒 | 404、method 不匹配、middleware 401/403/执行异常、handler 在 receive 前拒绝、Expect:100-continue（只发 headers）；磁盘 open/write/commit 计数均0、可及时收到拒绝、不需要发完整文件。 |
| 存储 | traversal/编码/drive/UNC/device/ADS/链接或 reparse/父目录替换、旧目标、同目标并发、无权限/空间/写/flush/sync/no-replace故障；不越界、不覆盖，失败 status 正确。 |
| 业务 | Received 后字段/content 校验 reject、忘记 commit/Drop/lease 到期、重复 receive/commit、failed session commit；只在显式成功 commit 后有公开文件；staging不能通过729下载。 |
| legacy | 既有 `e2e_b6_multipart_upload_field_and_file` 成功形状/字节；新增404/拒绝/write故障/绑定失败/handler错误探针，误处理没有残留/假路径，owner不写盘；普通JSON/SSE/729下载回归。 |

解析库的行为和外层 limit 分别验证；额外文件/字段不能在第一个文件已经发布后才拒绝。receipt 与 HTTP status/hash/磁盘目录交叉核对，不能只断言 contains("success")。

### 6.2 async/资源/提交竞态与互通

- gate 卡网络/磁盘/业务判定；active4+queue16 下一笔503，queued 取消/总期限计入，staged继续占额度；body不读/不poll也有期限。无固定sleep，所有测试总截止。
- 大小递增 fixture 验应用 buffered峰值公式（含 retained allocation）、在途FS/producer/job/permit/scope/session/temp计数；健康请求独立连接预热后20次均<500ms，慢文件不占 owner/reactor。上传持续>30s仍可合法成功，408/lease/normal timeout阶段切换单测用可控clock/gate。
- EOF完整/短体、确证断连、发送完body后的半关闭、handler/future Drop、idle、total、shutdown drain/force；FS gate放行后实际退出/清理、无迟到live-op复活。OS操作未返回时记录真实未退出，不虚记零资源。
- commit gate之前取消不发布；之后取消不把已发布文件误删；目标同时创建/两session同名仅一个201，其余409；回复丢失后文件已存在的边界有准确receipt/日志。
- 727 `transfer_upload`（multipart/raw、字段、默认不重试）对 VM/Rust 实际上传；201后用729 `FileResponse` 路由下载比hash，再用727 `transfer_download`落盘比hash。拒绝/413/磁盘失败时727收到非2xx失败，取消时server未commit清理；不是两组各自mock。

### 6.3 实施门禁（本轮不运行）

全部在 `D:/autostack/.wt/lang-730/auto-lang`：

- 快速 `cargo check -p auto-lang`、`cargo check -p auto-man`、`cargo check -p a2r-std`；新增能力不依赖UI，改依赖/feature补 `cargo check -p auto-lang --no-default-features` / `--features streaming-http`。
- scoped `cargo t plan730`；`cargo test -p a2r-std server_upload -- --test-threads=1`；`cargo test -p auto-man api_gen:: -- --test-threads=1`；真TCP族命名 `http_e2e_plan730`，`cargo nextest run -p auto-lang --lib --features test-http-e2e --test-threads=1 http_e2e_plan730`。legacy已有族用实际函数名/feature过滤复测。
- 复审裸 `cargo t` + `cargo tv` + `cargo tt` + `cargo th --test-threads=1`；a2r-std改动补 `cargo test -p a2r-std -- --test-threads=1`，生成服务必须真实build/run；source/dependency hash缓存允许，golden不替代实编。
- 只有实际触 `ui_gen/**` 的上传HTTP消费分支才补 `cargo tu`；不触aavm，不跑taa；不改schema/文档生成器/语法参考，不额外跑docs_gen。tf仅merge到期后主检出单实例 `/auto-plan:regress`。
- 格式、警告差分、残余debug/延期/workaround扫描；预存红同命令基线逐名对照，零新增确定性红，不笼统归环境。Windows/Linux路径/发布原语均有证据，测试链接不得入worktree，不能把无创建权限的skip记作通过。

新报告 `docs/plans/reports/730-upload-{decision,protocol,storage,lifecycle,parity,verification}.md`，记录实际代码/基线revision、命令/退出码、平台、AC/SD映射、预算公式及文件/句柄回收。

## 7. 验收标准

| ID | 可观察交付 | 证据/期望 |
|---|---|---|
| AC-01 | 公共upload request/session/receipt与VM/Rust实际命名异步handler | 同源生成实编和VM wire，multipart/raw 12MiB真实201+hash、typed注入和receipt status，普通int/同名用户函数不误判。 |
| AC-02 | parser和预算正确，普通JSON额度不变 | §6.1分块/畸形/边界±1矩阵，两端状态/实际字节/文本字段一致；没有完整文件Vec或无界carry。 |
| AC-03 | 授权先于存储、staging不可公开、root限制 | 早拒open/write计数0、Expect headers-only拒绝；Windows/Linux安全创建与链接/父目录替换探针；729无法下载未提交内容。 |
| AC-04 | 有界async intake、排队/长上传/lease期限正确 | quota/gate、应用真实缓冲/FS/session总量公式、20次health<500ms、>30s合法传输可完成；无每请求线程/runtime。 |
| AC-05 | 两阶段提交/取消/故障与资源收口 | Received后reject无最终文件、create-only同名仅一成功、旧目标不变；commit竞态/在途FS/断连/shutdown矩阵资源回基线，无假成功/迟到复活。 |
| AC-06 | legacy正常行为保持并清偿本期必要缺口 | 旧B6成功字节与JSON形状；404/中间件拒绝不写盘，write失败真实500，绑定/handler异常清理本次文件；JSON/SSE/729回归无新增红。 |
| AC-07 | 真实生成/客户端消费与727+729互通，不支持面诊断 | 新上传转译错不fallback，TS不把FormData做JSON、不传造request；IPC/merged/back-proxy明确拒绝；VM/Rust上传→下载同hash、非2xx/取消端到端正确。 |

## 8. 执行步骤

### T-01：等待依赖落地、复现差距与有界原型 [✅ e88d06f45]

- 前置：729独立review通过并merge；核对最终Spec/receipt与本合同，不在729工作树实施730。
- `bash scripts/new-wt-group.sh lang-730 --branch plan-730-dev`，记录master基线和729落地hash，auto-down兄弟只读。
- 核对 `http_transport.rs::bridge_handler`、`http_server.rs::dispatch_api_request_segment/advance_dispatch/store_multipart_file`、`vm/codegen.rs` 与 API参数/返回生成器；真实原型header→owner授权→lazy body→receive session→commit receipt，生成api.at编译运行。
- 复现legacy404/拒绝写盘与FS假成功；冻结public ABI/options/error JSON、两facade executor hook、stage资源、deadline watch/100-continue、multer版本/实际header-carry上限、root私有staging与原子no-replace原语。Windows/Linux探针明确平台证据，报告 `730-upload-decision.md`。
- 验证：最小双端raw/multipart commit/reject与鉴权gate；覆盖AC-01..07/SD-01..08的设计可实施。若需完整HTTP Request/语言ownership重构，提交needs_replan，不以全量缓冲/弱路径检查代替。

### T-02：公共类型、async facade与路由能力分类 [✅ 90748a083]

- 依赖T-01。新 `crates/a2r-std/src/http/server_upload.rs`；修改`http.rs`导出、`auto-lang/src/a2r_std.rs`、`stdlib/auto/{http,http.vm}.at`；共有严格options/有序metadata/receipt。
- 修改`api/{types,mod}.rs`、`vm/codegen.rs`签名采集与上传policy发布：精确UploadRequest/异步receipt，参数冲突/method不支持诊断，不重用meta str猜测规则。
- 验证：`cargo test -p a2r-std server_upload -- --test-threads=1`、`cargo t plan730`类型/options/路由分类金样；错误options未开始I/O；AC-01/02/07，SD-01/02/03/05/06。

### T-03：共用增量parser与受限staging接收 [✅ 0df7d9886]

- 依赖T-02。新`auto-lang/src/http_upload_service.rs`，修改`src/lib.rs`/必要Cargo依赖；复用729root工具，实施multipart/raw和各预算、准入、FS许可、文本有序字段、staged lease。
- parser必须逐块写文件、完整HTTP EOF后Received；报错/第二file/超限清理已收内容；所有应用缓冲/retained块有计数或公式，staging不能被文件下载路由读取。
- 验证：`cargo t plan730` parser/gate/metadata表驱动和feature checks；§6.1协议错误/大小边界，AC-02/03/04，SD-01/03/04。

### T-04：业务校验、no-replace提交与取消仲裁 [✅ 0df7d9886]

- 依赖T-03。同新宿主模块及729共享安全工具：metadata/受限内容检查能力、commit/reject/Drop、sync与原子发布、目标冲突/取消gate；必要平台helper为新增路径，T-01登记。
- create-only不用先exists再覆盖；failed session不能成功，commit重复明确拒绝，scope终结与FS实际退出分开；cleanup失败记录可追踪遗留，不提前腾FS槽。
- 验证：`cargo t plan730` + OS发布/链接/权限/磁盘注入矩阵，报告`730-upload-storage.md`；AC-03/05，SD-01/04/05。

### T-05：VM流式入口、native等待与scope生命周期 [✅ ae460c8c4 + R1 dcd151932]

- 依赖T-02/03/04。修改`vm/ffi/{http_transport,http_server,async_http,stdlib}.rs`、`ffi/mod.rs`、`vm/{native,native_catalog}.rs`，必要新`ffi/http_upload.rs`；native/public名称以T-01冻结。
- header-first仅用于上传policy route；body保留宿主、UploadRequest有类型注入、授权后receive；live-op重入/typed完成结果、上传资源组和stage deadline/idle/lease对齐。完整receipt作为真实HTTP状态/JSON回复，不由整数位模式猜类型。
- 验证：`cargo check -p auto-lang`、`cargo t plan730`及VM真实HTTP scoped；中间件/handler早拒无open、半关闭成功、>30s传输、取消/迟到回收；AC-01/03/04/05/07，SD-02/03/04/06。

### T-06：a2r/auto-man真实上传与HTTP客户端消费 [✅ a30c29595/893c3cec5]

- 依赖T-02/04/05。修改`trans/rust.rs`、`auto-man/src/api_gen.rs`（参数分类、生成extractor/handler/返回adapter）、`api/targets/{typescript,tauri,axum}.rs`及实际split/merged消费点；back_proxy仅本类型Unsupported守卫。
- receive/commit/reject async await单源；自动注入不进JSON参数结构，body extractor最后且不预读/过早鉴权；精确receipt HTTP status，上传体转译失败定位api.at而不fallback。HTTP TS接FormData/raw body；非HTTP形态诊断，不传handle。只有独立UI消费点需要时加窄分支并记录tu触发。
- 验证：`cargo test -p auto-man api_gen:: -- --test-threads=1`、a2r golden+实际generated build/run；auth/reject/成功/unsupported都有调用证据；AC-01/07，SD-03/05/06/07。

### T-07：legacy落盘顺序与错误兼容修复 [✅ 8a2fc567e]

- 依赖T-03/05。修改`http_server.rs::parse_multipart/multipart_to_handler_json/store_multipart_file`及实际调用点：route/middleware后有界宿主文件I/O、正常legacy形状、错误传播/provisional资源清理。活Builder调用传播错误；不扩大旧body额度/宣称流式。
- 在既有B6与新plan730族加入404、401/403、write失败、bind/handler失败；报告协议legacy专节记录所有调用路径和成功形状，绑定scope而不手工任意路径删除。
- 验证：`cargo t plan730`、串行实际B6 e2e及JSON/SSE/729 scoped回归；未授权/未匹配零文件，错误无假路径，owner不做阻塞write；AC-03/05/06，SD-02/08。

### T-08：双端wire、资源负载及727/729闭环 [✅ f11c83486 + R1 dcd151932]

- 依赖T-05..07。新`crates/auto-lang/src/tests/plan730_http_upload_tests.rs`，修改`src/tests.rs`注册；auto-man api_gen tests真实fixture；新`examples/http_server/uploads/{README.md,pac.at,src/back/api.at}`，上传后下载路由复用729，fixture数据测试临时生成。
- 全覆盖§6.1/6.2、FS注入/commit gate/100-continue与legacy；727真实upload→729路由→727download，比hash/receipt/资源，不用独立mock替代。
- 验证：`cargo nextest run -p auto-lang --lib --features test-http-e2e --test-threads=1 http_e2e_plan730` +生成服务实际运行入口；报告`730-upload-{protocol,lifecycle,parity}.md`，逐格明确两端/平台/资源；所有AC/SD实证。

### T-09：分级门禁、独立review与规范交接 [✅ 12c508341/493081841——门禁+验证报告完成；独立review=/auto-plan:review 下一阶段]

- 依赖T-01..08。按§6.3门禁，`730-upload-verification.md`绑定最终代码hash和全部AC/SD；预存红逐名对照、零新增确定性红，警告/格式/未批准延期检查。
- work完成后 `/auto-plan:review` 独立复验；准备SD-01..08沉淀稿，merge再更新canonical/ledger/Design33与索引、archive。729落地新增Spec不得被旧基线稿覆盖。
- 清理前`bash D:/autostack/wt-guard.sh D:/autostack/.wt/lang-730/auto-lang`和兄弟仓guard输出clean再移除；tf到期只主检出单实例；所有AC/SD闭合才可归档。

## 9. 复审记录

### 合并收据 PLAN-730:r1（2026-10-03，/auto-plan:merge）

- stage: merge | plan_id: PLAN-730 | plan_revision: 1 | outcome: pass
- **prepared**：reviewed 基线 dcd151932（R2 pass）→ rebase 到 master@36912a975
  （master 前移：731/732 已并、735 簿记在途，规范目标零冲突）→ 新链
  e88d06f45'..05645a444（`git range-diff` 11 提交逐对全等——安全改写证明）→
  canonical 沉淀 SD-01..08（新 stdlib/design/http-server-uploads.md、
  auto-lang/trans/design/http-upload-lowering.md；改 http-server.md §10+当前支持面、
  backend-assembly.md、stdlib/project.md、http-handler-async-lifecycle.md、
  a2r-std/project.md、trans/overview.md、auto-man/project.md）+ plans 索引三行 +
  ledger upsert P730-1(designs)/P730-2(designs)/P730-7(reviews)/P730-8(reports)
  （docsha 绑定）+ `scripts/spec-index.py` 再生 → 投影后裔 **delivery commit
  f60cfeea2**（实现/依赖与 reviewed 状态零改动）。
- **landed**：主检出 `git merge --ff-only plan-730-dev` → tip=f60cfeea2（无 merge
  提交）；主检出冒烟 `cargo t plan730` 26/26（一次并行 flake=plan730_commit_target_matrix
  磁盘重测试，隔离+复跑皆绿——在案观察，非确定性红）。
- **ledger_refreshed**：`.autoos/specs.json`（本仓 tracked，worktree 内 upsert +
  git 提交——仓惯例）P730-1/2/7/8 回读在档；INDEX.md 再生（26 projects）。
- **archived**：`docs/plans/archive/730-http-server-upload-ingress.md`
  （git mv；status archived；completion_kind: delivered）。
- **cleaned**：见下一提交（wt-guard ×2 + worktree/branch/组目录移除后回填）。
- 部署观察：landing 非部署——本计划改动面（stdlib/VM/生成器）无在运行生产进程
  消费本仓发布二进制（auto 桌面壳未在本机常驻）；无 `auto build` 产物待重建。
- 批量回归到期判定：**730 % 5 = 0 → 到期**（landing of plan id divisible by 5；
  另 `.last-batch-regression.json` 收据 2026-10-02 已 >48h 窗口临界）→ 交接
  `/auto-plan:regress` 主检出单实例执行 tf 批量档。

### 复审 R2（2026-10-03，/auto-plan:review）

- stage: review
- plan_id: PLAN-730
- plan_revision: 1
- outcome: pass
- reviewed_commit: dcd151932（R1 回工提交；分支累计 11 提交 e88d06f45..dcd151932）
- base_commit: 7d50989f7a
- dependency_revisions: 同 R1
- spec_inputs: 同 R1
- acceptance_results: AC-01..07 全 pass（R1 的 F-2/F-4/F-5 证据缺口全部补齐）
- findings: R1 五项全部修复并复验——F-1 收据登记（insert_upload_receipt:+register_scope_upload）；
  F-2 0B 任意切块 + quoted boundary（单测 + wire 独立用例）；F-3 chunked 上传 wire；
  F-4 配额 503（**暴露真缺陷**：active 等待臂误用 total 而非队列期限——已修，
  31.8s 桥超时挂死 → 2.5s 全绿）+ 20×health<500ms；F-5 断连清理 + total 408 滴流。
  回工期间两起测试数据事故（heredoc 转义损坏用例体/批量替换误伤 CL 值）均已
  定位修正——不构成实现缺陷。规范增量 SD-01..08 目标路径/前后规则与实现一致，
  沉淀稿在 merge 阶段落 canonical。
- evidence: `cargo t plan730` 26/26；`http_e2e_plan730` 全族 14/14（R1 复跑）；
  回归面 t/tv/tt/a2r-std/auto-man 零新增确定性红（预存逐名同 R1；a2r-std 首轮
  1 传输 flake 复跑绿）；报告六份已补 R1 节（lifecycle §6 / verification §6）。
- next: merge（/auto-plan:merge）——SD-01..08 canonical 沉淀 + ledger + Design33
  索引 + archive + wt-guard 后清理组目录。master 已前移（731/732 会话簿记），
  需真实 merge 非 ff。
- 独立性：R2 与实施同会话——从工件重建（修复 diff 逐行核对 + 定向/回归复跑）。

### 复审 R1（2026-10-03，/auto-plan:review）

- stage: review
- plan_id: PLAN-730
- plan_revision: 1
- outcome: needs_fix
- reviewed_commit: 493081841（分支 plan-730-dev 全部 10 提交；worktree clean）
- base_commit: 7d50989f7a（master；其后 master 另有 731/732 会话簿记推进——merge 阶段需真实 merge 非_ff）
- dependency_revisions: PLAN-729 archived@7d50989f7（spec 050e2ee90）；a2r-std/auto-man 同 worktree
- spec_inputs: docs/specs/stdlib/design/http-server-files.md（729 最终版）+ overview/stdlib project 等 §4.1 表
- acceptance_results: AC-01 ✅/AC-02 partial（F-2/F-3 证据缺口）/AC-03 ✅/AC-04 partial（F-4/F-5 证据缺口）/
  AC-05 ✅（并发同名以 hard_link 原子性+顺序 409 证，真并发不可确定性复现——注记）/AC-06 ✅/AC-07 ✅
- findings:
  - **F-1 [P2]** `vm/ffi/http_upload.rs insert_upload_receipt` 未调 `register_scope_upload`——
    构造后未编组的收据（如 handler 早分支丢弃收据返回他值）在 `VM_UPLOAD_RECEIPTS` 无界滞留；
    729 shim 对每个构造登记（http_server_file.rs:59 先例），本桥自身注释承诺"防注册表无界增长"。
    影响 AC-04/05 资源回收面。修复=登记一行。
  - **F-2 [P3]** §6.1 上传族"0B 文件"、解析族"quoted boundary"无显式测试（结构支持，证据缺）。
  - **F-3 [P3]** §6.1 "chunked/无声明长度"未测——现有用例全部携带 Content-Length。
  - **F-4 [P2]** §6.2/AC-04 "active4+queue16 满下一笔 503"与"慢传输期间 20 次 health<500ms"未测。
  - **F-5 [P3]** §6.2 断连（中途关连接→staging 清理）与 total 期限到期无直接测试（idle 旋钮已测）。
  - 观察（非缺陷）：examples standalone `auto run` 报 "Skipping back" 为预存（729 files 示例同形同报，
    两者 api.at 内容均经 e2e fixture 编译运行验证）——候选 P730-D4。
- evidence: 复现命令与结果——`cargo nextest run -p auto-lang --lib plan730` 23/23（R1 重跑）；
  示例审查（upload_program 同形 harness 已运行 + files 示例同报对照）；dispatch 段门序核对
  （match_route→upload→legacy parse→file gate→websocket，websocket/文件门未受重排影响）；
  F-1 代码核对（insert_upload_receipt:84-90 无 register 调用 vs http_server_file.rs:59 先例）。
  其余门禁面证据沿用 493081841 工作交接记录（代码/依赖/测试配置未变，R1 复跑定向面一致）。
- next: 回工修复 F-1..F-5（T-05/T-08 重开，current_step=7）→ 复审 R2 → merge。
- 独立性：与实施同会话复审——已按技能要求从工件重建结论（代码核对/复跑/逐项 diff 狩猎），
  未采信实施自述；发现项均附可复验证据。

### 工作交接（2026-10-02，/auto-plan:work）

- stage: work
- plan_id: PLAN-730
- plan_revision: 1
- outcome: pass
- code_commit: 493081841（分支 plan-730-dev，master 基线 7d50989f7a；11 个提交
  e88d06f45..493081841，worktree D:/autostack/.wt/lang-730/auto-lang 保留待审）
- task_ids: T-01..T-09 全部完成
- evidence: 决策/协议/存储/生命期/对等/验证六报告（docs/plans/reports/730-upload-*.md，
  绑定最终代码 hash 12c508341/493081841）；门禁面——plan730 23/23、a2r-std 87+7+6、
  auto-man plan730 5/5 + 全量 339/342（3 预存 master 同名）、裸 cargo t 1479/1482
  （3 预存 musk p053 族）、tv 162/162、tt 1869/1872（同 3 预存）、th 92/93
  （1 预存 back_proxy master 同名）、VM e2e 10/10（含 38s 慢上传跨 30s 期限）、
  生成服务真实编译运行 e2e 1/1、back_proxy 501 1/1、727↔730↔729 互通闭环同字节。
  AC-01..07 与 SD-01..08 对照见验证报告 §3/§4。
- blockers: 无。已知债 P730-D1..D3 记录于验证报告 §5（均预存/边界如实注记，
  不阻塞验收）。
- next: /auto-plan:review PLAN-730（独立复验 → SD 沉淀稿定稿 → merge 阶段更新
  canonical/ledger/Design33 与索引、archive、wt-guard 后移除组目录）。
- 执行注记：multer 依计划"优先"项经 T-01 论证后拒绝（per-part-header/carry 预算
  无法外部施加于其内部缓冲），自研增量状态机（合成 CRLF 统一首边界仲裁）+ 零新依赖；
  facade 形参定为 impl AsRef<str>（字面量 .as_str() E0658 不稳定坑，报告在案）；
  中间件空串短路与 Plan 352 文档分歧为预存语义（未改写，示例用 ?str nil 放行）。

### 起草交接（2026-10-02）

- stage: new
- plan_id: PLAN-730
- plan_revision: 1
- outcome: blocked
- blocker: 实施前置729正在进行，尚无review/merge后的最终接口与规范。
- next: PLAN-729 reviewed + merged → T-01核对其最终契约 → /auto-plan:work PLAN-730。
- changed_tasks: T-01..T-09（新）
- changed_acceptance: AC-01..AC-07（新）
- 规划合同已完成；blocked只表示现在不能交接依赖中的实施，不要求用户补充信息，不影响729继续执行。没有将729进行中代码当作已交付。
- 起草检查当前Spec/代码、0..10章节、T/AC/SD映射、源路径/命令/链接和唯一取号；没有创建730实施worktree、没有跑Cargo、没有改canonical或729进度。

## 10. 待澄清事项

- 用户输入无缺项。执行顺序已选729合入后730；公共ABI/host hook、stage deadline/body和safe no-replace的机制由T-01有界调查负责，若729最终设计不兼容应修订合同而非绕过。
- 单文件multipart/raw满足本期；多文件原子事务、断点/分片上传、全局磁盘配额与崩溃残留扫描另案。上传内容类型由用户提供，不等于内容验证；应用可在commit前做业务检查。
- 原子发布≠HTTP回复恰一次送达，scope取消≠在途FS立即退出；明确上述实证边界。文件根由应用授权，不承诺防御主机管理员/恶意本地root写入者。
- 730之后HTTP强化候选为 API多形态契约/生成失败诊断全面收敛、部署配置/支持等级、装配manifest与CPU纪律；应由729/730实证再确定下一独立计划，不预定剩余总数。
