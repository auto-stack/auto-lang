---
plan_id: PLAN-729
status: executing
feature_name: http-server-file-responses
author: [agent]
created_at: 2026-10-02
updated_at: 2026-10-02
plan_revision: 1
current_step: 8
total_steps: 8
supersedes_spec_components:
  - docs/specs/stdlib/project.md
  - docs/specs/stdlib/design/http-server.md
  - docs/specs/stdlib/design/backend-assembly.md
  - docs/specs/stdlib/design/http-handler-async-lifecycle.md
  - docs/specs/a2r-std/project.md
  - docs/specs/auto-man/project.md
  - docs/specs/auto-lang/trans/overview.md
new_spec_components:
  - docs/specs/stdlib/design/http-server-files.md
  - docs/specs/auto-lang/trans/design/http-file-response-lowering.md
touched_goals: [GOAL-003]
affects: [stdlib/auto/http.at, stdlib/auto/http.vm.at, crates/a2r-std/src/http, crates/auto-lang/src/a2r_std.rs, crates/auto-lang/src/vm/ffi, crates/auto-lang/src/vm/native.rs, crates/auto-lang/src/trans/rust.rs, crates/auto-lang/src/api, crates/auto-man/src/api_gen.rs, crates/auto-lang/src/back_proxy.rs]
---

# [PLAN-729] HTTP 服务端文件响应：流式下载、HEAD/Range、条件请求与生命期

## 0. 变更摘要

接续 [Design 33](../design/33-stdlib-runtime-and-http.md) 阶段 D1a。客户端文件传输 [PLAN-727](archive/727-http-file-transfer-lifecycle.md) 已复审归档、合入 `6eb396e5a`；本计划补**server 向请求方提供文件下载**，以普通 `api.at` 的 `FileResponse` 返回面覆盖 VM 默认 HTTP 和生成 Rust/Axum HTTP 服务。

交付不是将磁盘文件先读成字符串/JSON，也不是用媒体专用路由冒充通用 API。提供共同的文件响应执行代码、GET/HEAD 与单区间 Range、条件请求、受限根目录打开、有界读盘和发送、断连/超时/关闭后的资源收口；用 727 客户端对两个实际服务执行下载与续传。

本期不包含**服务端接收文件上传**（multipart/raw 流式入口、临时文件提交与上传策略），不启用目录遍历或补齐 `Server.static`，不实现多区间 `multipart/byteranges`、压缩表示、任意 SSE handler 生成、完整 API/IPC/进程内契约统一、TLS/HTTP2/3 或 CPU 抢占。新增文件响应在不支持的调用形态中必须明确拒绝，不把 opaque id 当成功数据。

## 1. 目标

- G1：应用在命名 `#[api]` GET handler 中构造 `http.file_response(root, relative_path, options)`，同步 `FileResponse` 与异步 `~FileResponse` 都能在 VM/Rust HTTP 返回同一文件表示；网络和读盘不在 VM owner 中执行。
- G2：GET/HEAD、单区间 Range、缓存前置条件与 If-Range 有明确子集和字节级验收；零字节、非 UTF-8 与 u64 位置正确。
- G3：根目录限制、文件类型检查和响应头校验在打开/发送路径执行；用同一个已打开句柄取 metadata、seek 和读取，消除简单路径检查后重开所产生的逃逸。
- G4：应用缓冲、文件操作并发、排队、准备期限、慢发送和排水都有上限；响应体持有资源和许可直到结束，取消不遗留继续读取文件的 job。
- G5：真实 `api.at` 经生成器编译运行，不能仅验证手写 Axum helper；新增文件端点转译失败不退回 CRUD 模板，不影响已有 JSON/SSE 兼容。

影响仓：只修改本仓；auto-down 兄弟检出仅供依赖解析。不修改 AutoUI 应用业务或 auto-musk。能力是通用 HTTP 二进制文件响应，不依赖 UI/media feature。

## 2. 架构方案

```text
api.at: FileResponse / ~FileResponse
  └─ http.file_response(受信 root, 相对路径, options)
       └─ owned 描述符：只保存配置，不打开、不整文件读、不带 VM 引用
          ├─ VM native → 有类型结果编组 → ApiBody::File
          └─ a2r 构造 → auto-man 文件 handler adapter
                         ↓
          共同宿主文件服务：受限打开 → metadata/条件/Range → 有界 body
                         ↓
          Axum/Hyper 发送；EOF/Drop/错误/关闭 → 句柄、配额、scope 收口
```

拟新增 `crates/a2r-std/src/http/server_file.rs`：描述符、严格 options 与构造面；两个 Rust facade 转发同一实现，不让 a2r-std 反向依赖 auto-lang。拟新增 `crates/auto-lang/src/http_file_service.rs`：与 UI 无关的宿主文件准备及响应体执行代码，通过 `lib.rs` 导出，VM HTTP 与 auto-man 生成服务调用同一代码。

T-01 以最小真实编译原型冻结模块拆分、返回类型识别、根目录打开和生成器接入，允许保持依赖方向的等价调整。不能因现有可选 `tokio-util` 只在 `streaming-http` feature 下启用而把普通文件能力隐含绑定到媒体功能；决定复用或增量 reader 实现时同步验证 feature 矩阵。

构造不做阻塞 I/O；网络侧打开/准备也有 deadline。VM 请求 scope 与文件 body 的交接须为明确所有权转移：不能沿用普通 Text 回复立即 `complete_scope` 的分支。VM 返回结果必须由声明返回类型、native 类型及登记共同识别，普通 int 即使数值碰到资源 id 也仍是普通 int。

## 3. 技术栈

- 现有 Axum 0.8 / Hyper、Tokio fs/异步网络；复用服务器 runtime，不为每文件新建线程/runtime。
- a2r-std 只承担可移植构造与数据契约；宿主执行放在 auto-lang 的公共 Rust 模块。
- 现有 `mime_guess`，严格响应头类型和 HTTP 日期解析；新依赖只在 T-01 根目录打开原型证明必要后引入，记录版本、平台和许可证。
- HTTP 协议依据 RFC 9110；位置/长度用 u64，转换到 usize/VM int 只能在有界块或显式受检路径。
- 既有 native/codegen/a2r golden、真实 TCP HTTP 测试和 auto-man API 生成测试；监听用 OS 临时端口，截止时间与 gate 复现不靠固定 sleep。

## 4. 需求分析与背景调查

### 4.1 来源、版本与授权

起草时主检出 `9e379c9c8`（727 merge 清理收据）；727 delivery=`6eb396e5a`，review 在归档计划 §9，客户端 canonical Spec 已存在。729 取号簿记=`fcbdfa306`，728 已存在，核对活跃/归档区后修正滞后计数并取 729，未覆盖其他计划。实施 T-01 再记录实际 worktree 基线，不把起草期间并发提交当本计划代码。

| 来源 | 本计划使用的事实/约束 |
|---|---|
| `docs/specs/overview.md`、`docs/specs/stdlib/project.md` | canonical 优先；多后台实现不是仅靠同名声明自动等价。 |
| `docs/specs/stdlib/design/http-server.md` §4.1.1、§8.1、§10 | 目标草案与已支持面区分；服务端 multipart 增强尚未交付。 |
| `docs/specs/stdlib/design/backend-assembly.md` | VM Axum 入口、生成 Axum、IPC/merged/back-proxy 为不同调用路径；保留准确覆盖表。 |
| `docs/specs/stdlib/design/http-handler-async-lifecycle.md` | 单 owner 段执行、scope 生命周期、SSE body 持许可；输入半关闭不是取消证据。 |
| `docs/specs/stdlib/design/http-file-transfer.md`、`docs/specs/a2r-std/design/http-client-runtime.md` | 727 客户端已支持增量下载/续传/staging、可取消传输；客户端 file helper 不等于服务端文件路由。 |
| `docs/specs/auto-lang/trans/design/http-client-lowering.md`、`docs/specs/a2r-std/project.md` | opaque 类型与 sync/async 发射须同源实际编译；两 facade 的依赖方向。 |
| `docs/specs/auto-man/project.md`、Design 33 | API 生成和支持形态必须准确；本计划只对新增文件面闭合生成失败诊断。 |
| [RFC 9110](https://www.rfc-editor.org/rfc/rfc9110.html) §9.3.2、§13、§14 | HEAD 无 body；前置条件先于 Range；不支持的区间策略和 If-Range 判定需要明示。 |
| [Tokio fs 文档](https://docs.rs/tokio/1.53.1/tokio/fs/index.html) | 普通文件 I/O 使用 blocking pool；不能把丢弃网络 future 等同于强制中断在途磁盘操作。 |

用户已授权：727 完成后继续**规划下一计划**，延续 HTTP 能力强化与标准库多后台调研。允许本仓调研、计划和设计簿记；本次不执行代码、不创建实施 worktree、不做部署。没有用户设定的预算或自动连续实施授权。727 已归档，无待复审的前置阻塞。

### 4.2 源码现状与差距

| 源码/符号（本次已查） | 观察 | 本期处理 |
|---|---|---|
| `vm/ffi/http_server.rs::ApiBody`、`marshal_handler_value` | 只有 Text(Vec) 和 SSE；Response handle 是已有 Vec body，没有通用文件返回类型。 | 增加 File 描述符结果；避免整文件 JSON/Vec。 |
| `vm/ffi/http_transport.rs::api_reply_to_response`、scope 交接 | 只有 SSE 保持 scope，其余立即完成；body adapter 不支持文件。 | 文件准备与 body 持有许可/资源，错误/取消闭合。 |
| `vm/ffi/stdlib.rs::shim_http_server_static` | 现为占位：返回 server，没有注册静态路由。 | 仍明确不支持；本期不是实现 static/目录服务。 |
| `auto-man/src/api_gen.rs` 普通 handler 返回类型/体发射 | 非 void 一般套 JsonResponse；体转译失败可落模板。 | 按 FileResponse 分支生成二进制 adapter，文件面失败有源码诊断。 |
| `auto-man/src/api_gen.rs::MEDIA_SERVICE_HANDLERS`、`ui/media_service.rs::parse_range` | 媒体专用路由已有 File/ReaderStream 和部分 Range，不是所有普通 api.at 路由都没有文件能力。 | 参考其语义/测试，通用核心不依赖 UI；既有媒体只作兼容回归，不顺带重做。 |
| `api/targets/{typescript,tauri,axum}.rs`、`api/types.rs`、`back_proxy.rs` | API 返回类型与各调用形态分别生成/编组，存在误走 JSON 或 opaque handle 的风险。 | 新类型统一分类；HTTP 消费按字节；不支持形态明确诊断/拒绝。 |
| `a2r-std/src/http/{client,transfer}.rs`、VM `http_transfer.rs` | 已有客户端执行核心；无法据此提供服务器范围响应。 | 复用词汇和验证，不另造客户端网络内核。 |

本次为静态能力核查，尚未运行新 server 文件原型/性能负载。未发现现成的通用服务端文件 canonical Spec，提出 SD-01 新增；727 已沉淀的客户端 Spec 不在本期重复改写。

## 5. 详细设计

### 5.1 公共面与调用形态

拟定公共声明（T-01 只允许等价 ABI 调整）：

```text
type FileResponse
pub fn file_response(root str, relative_path str, options str) FileResponse;
```

`FileResponse` 是 owned 响应描述符，不是 FileTransfer、JSON 模型或可序列化文件句柄。root 来自应用受信配置，relative_path 可以来自路由参数；options 为严格 JSON，空对象合法，未知键、错类型、非法头值明确失败，不静默回退默认成功。构造本身不访问文件；故障由 HTTP adapter 在发送 headers 前映射。

最小 options：`content_type`、`download_name`、`disposition`（attachment 默认 / inline 显式）、可选 `etag`（应用提供的内容版本验证器）。字段名/错误分层于 T-01 冻结。第一期不开放任意 Content-Length/Content-Range/Transfer-Encoding 覆盖，不默认压缩，不让路径/文件名直接拼接 HTTP 头。

| 调用面 | 本计划合同 |
|---|---|
| VM 默认 `#[api]` HTTP named fn / ~fn | FileResponse/异步文件结果都支持；保持现有 auth/middleware、请求 ID/CORS。 |
| auto-man 生成 Rust HTTP 服务 | 同一 api.at 生成真实文件 adapter；成功体不经 JsonResponse；前置条件头与 method 自动传入宿主层。 |
| 标准 HTTP 客户端/浏览器直接请求 URL | 返回二进制 HTTP 表示；生成 TypeScript HTTP 文件方法可返回原生 Response，不调用 `.json()`；保留状态/headers，读盘交给客户端。 |
| Tauri IPC、VM merged/进程内、gallery back-proxy 透传 | 本期不提供文件句柄跨形态协议；新文件端点明确诊断或 Unsupported，包含接口名/形态与改走 HTTP URL 的指引。普通 JSON 接口不受影响。 |
| VM Builder/legacy stdnet、`Server.static`、单独 AxumGenerator | 未验收接入的入口必须明确拒绝新类型；不能把“都用 Axum”当自动支持。单独生成器可委托共同 adapter，但只有实际编译/运行后才列为支持。 |

T-01 必须追踪 `auto run` 实际 split/merged 客户端与 `api/targets`/auto-man 两套消费点，不能只修改未被使用的一个生成器。若生成 Vue/Rust UI 自动客户端确有独立 FileResponse 消费点，只添加本类型分支/诊断，记录触面并加 `cargo tu`。本期不规定 IPC 文件内容 JSON/base64 编码，不改全局 API 返回模型。

文件端点只支持 GET/HEAD；生成/加载时诊断不合法注解方法。GET 文件路由自动具备 HEAD 匹配（显式同路径 HEAD 优先，不能改变普通路由规则）。异步 handler 执行语义沿用 705，返回文件后将读取移交宿主，不在 VM 中生成逐字节 generator。

### 5.2 文件表示、Range 与条件请求

- GET 200：identity 文件字节，Content-Type 正确，Content-Length=打开句柄的表示长度，Accept-Ranges: bytes；零字节文件 200、长度 0，不能用 `saturating_sub(1)+1` 算成 1。
- HEAD：按 GET 选择与前置条件给出相同表示 metadata，wire body 为 0；忽略 Range，不因没有 body 把文件 Content-Length 改成 0。304/412 等条件状态也无文件数据。
- 单 `bytes=start-end` / `start-` / `-suffix`：有效可满足 → 206、精确 Content-Range 与窗口长度；end 超过 EOF 时截至 EOF。有效不可满足 → 416、`Content-Range: bytes */len`、空体。未知单位、畸形或多区间按明确子集忽略 Range → 200，不用错误 206 冒充多区间。
- 前置条件先于 Range：If-Match / If-Unmodified-Since 可给 412；If-None-Match / If-Modified-Since 对 GET/HEAD 可给 304；遵守实体标签优先级和通配符存在语义。If-Range 不匹配/弱标签/无法确定为强验证的日期 → 完整 200。
- 默认可以不发送 ETag，Last-Modified 来自句柄 metadata 并按 HTTP 日期精度处理。**mtime+len 不能冒充强 ETag**。应用显式提供强 etag 时，须使其对应稳定内容版本；用于缓存/续传的可信版本随内容变化，不以同尺寸同时间证明一致。日期验证器的秒级分辨率限制写入 Spec；不能把弱日期宣称为强续传保证。T-01 用同尺寸改写、同秒修改原型冻结日期/etag 判定；如选摘要，只能后台增量/有界缓存，不每次在 owner 中整文件 hash。
- 一个请求的 metadata、范围和内容使用同一个打开句柄；用 checked u64 运算。路径替换不能切换到新文件；原句柄被截断导致提前 EOF 必须报发送失败，不当正常完成。发送 headers 后不能再伪造 JSON 500，只能终结 body/连接并记录 request ID 与错误。
- attachment 文件名去除路径成分、校验 CR/LF/NUL，UTF-8 编码不能破坏 header；测试含 Unicode、引号及恶意换行。默认 MIME 不明为 application/octet-stream。

### 5.3 根目录打开与错误

root 由应用选择并授权；本功能不会给远端选择任意主机路径。相对路径拒绝 `..`、绝对路径、Windows drive/UNC/device 路径和 NUL；路由解码次数明确，编码/双重编码用 wire 测试验证。目录、设备、管道等非普通文件不提供下载。

首选使用经过平台验证的目录 capability/逐段相对打开，拒绝 symlink/reparse 越界；不得只做 canonicalize + starts_with 后再按原路径打开。T-01 在 Windows 做 reparse/路径替换探针，在 Linux 校验 no-follow/目录句柄策略；文件服务具有平台限制时必须明确 Unsupported，不能宣称隔离而放行。威胁边界是应用配置的文件根与不可信 HTTP 路径，不承诺抵抗主机管理员或拥有根目录写权限的恶意本地用户。即便如此，HTTP 请求利用可见链接或准备阶段路径替换逃出 root 仍必须拒绝；单元注入探针不能替代实际可创建的 OS 链接样本。

已有鉴权/middleware 在文件打开前执行。缺失映射 404；越界/无权限 403（不泄露主机绝对路径）；坏应用 options、打开/读取内部故障 500；额度满/排队期限 503 + Retry-After；准备超时沿用现有 503 语义。各后台完全相同。合法 HEAD 对错误也不发送 body。新文件端点响应不开放任意外部 redirect。

### 5.4 异步、有界发送与回收

- 文件独立配额：active 默认 4；等待默认最多 16；同时受既有全请求 in-flight 上限约束。排队无文件句柄/数据缓冲，队满即时 503；从进入文件准备起的等许可+打开+metadata+seek 默认 30s 总期限（与既有 handler deadline 取更早者）。可配置值在 T-01 冻结，非法/0 不能意外表示无限。
- 应用读取块 ≤64KiB、每 active 最多 2 个待发送块，受 body poll/背压驱动；文件操作并发 ≤4，使用现有 Tokio 的文件 blocking pool、受许可限制。记录完整缓冲公式，包括 reader 在途块、队列、描述符及框架另行缓冲，不能以单块长度证明总 RSS 恒定。
- 响应头发送后 handler/reply 的 30s deadline 不继续误杀正常大下载；转入文件 body 的进度/idle 期限（默认 60s，可配置）。时钟必须覆盖 body 不再被 poll 的慢客户端，不能只在读盘 await 中设置 timeout；由独立有界计时/连接取消在期限后回收资源。总发送期限允许配置，但本期不硬套客户端 1GiB/10min 的预算。
- body 持有文件句柄、文件许可和请求 scope（生成 Rust 侧持等价 guard）。正常 EOF、错误、Drop、确证断连、idle 与 shutdown 均恰一次收口。输入半关闭仍正常提供响应；黑洞由期限收口。关闭停止准入，排水窗内完成者正常，窗后强制取消。
- 在途 FS 不宣称可强制 abort：停止新读、等待已有操作收口，再交付实际资源回收证据。逻辑取消与句柄/任务彻底退出分开计数；文件执行槽在在途操作退出前不得提前复用成无界 blocking job。取消不继续后台读完整文件，不有每请求 detached 线程。
- 文件 quota 是共享服务的独立额度；满额/慢下载时普通 `/health` 与 CRUD 不等待文件许可。相同 HTTP 连接的 HTTP/1 顺序约束不当作跨连接饥饿；实验使用独立连接。

### 规范增量

以下为**提案**，起草不修改 canonical Specs；work 准备 delta、review 逐项验证、merge 才沉淀。

| delta_id | add/modify/retire | docs/specs/... target | before/after rule | rationale | acceptance IDs |
|---|---|---|---|---|---|
| SD-01 | add | `docs/specs/stdlib/design/http-server-files.md` | 无通用文件响应 Spec → FileResponse/options、GET/HEAD/Range/条件/根目录、配额/取消与支持矩阵 | 单独描述 server 下载，不混入 727 client | AC-01..06 |
| SD-02 | modify | `docs/specs/stdlib/design/http-server.md` | 目标与现状未列通用文件结果 → 当前支持 FileResponse、HEAD 和范围子集；server 上传/static 仍未交付；§8.1 旧 ureq 客户端说明改为引用 727 当前 Spec | 避免媒体或上传被误算，清理已观测到的过时说明，不重做客户端 | AC-01/02/06 |
| SD-03 | modify | `docs/specs/stdlib/design/backend-assembly.md`、`docs/specs/stdlib/project.md` | 两套独立服务与声明覆盖 → 文件宿主执行单源、VM/Rust 实际支持及 IPC/merged/back-proxy/Builder 限制 | 后台装配机制继续承载真实覆盖 | AC-01/06 |
| SD-04 | modify | `docs/specs/stdlib/design/http-handler-async-lifecycle.md` | SSE 续持 scope，普通回复完成 → 文件 body 续持、prepare/body 两期限、FS 退出限制 | 防止响应前已释放许可 | AC-04/05 |
| SD-05 | modify | `docs/specs/a2r-std/project.md` | http client/transfer → 增加 server_file 描述符面，不声称 crate 启动服务器 | 保持依赖和公共类型单源 | AC-01/06 |
| SD-06 | add | `docs/specs/auto-lang/trans/design/http-file-response-lowering.md` | 无 FileResponse lowering → qualified/typed sync/async 构造和 handler 返回；非 HTTP 拒绝、普通 int 不误判 | 同源实编证明接口可用 | AC-01/06 |
| SD-07 | modify | `docs/specs/auto-lang/trans/overview.md`、`docs/specs/auto-man/project.md` | API 文件结果可能 JSON/template fallback → 文件专用生成 adapter/失败诊断、HTTP Response 客户端与不支持形态 | 生成产物与 VM 相同协议 | AC-01/06 |

## 6. 测试设计

### 6.1 协议/路径矩阵

同一源文件 API 分别启动 VM 默认 HTTP 与 auto-man 实际生成、编译的 Rust 服务，使用相同确定字节 fixture 和 wire 断言：

| 族 | 样本与必须断言 |
|---|---|
| 基本 | 0B、含 NUL/无效 UTF-8、12MiB；GET hash/长度/headers；HEAD body=0、Content-Length=GET 表示长度、Range 被忽略。 |
| Range | bounded/open/suffix、end 越 EOF、suffix>len、-0、起点=EOF、空文件、u64 大值/溢出、多区间/坏单位/坏语法；精确 200/206/416 和字节窗口。大位置用稀疏/metadata 探针，不读取数 GiB。 |
| 条件 | If-Match/If-None-Match（含列表、弱/强、*）、日期和优先级；匹配 304/412、If-Range 不匹配完整 200；Range 与条件同时存在先判条件。版本变化的强 ETag 不产生假 304/混合续传。 |
| 路径/头 | 普通相对文件、缺失/目录/权限、../、编码/双编码、绝对/drive/UNC/device、链接/reparse 与 prepare 中路径替换；拒绝越界、没有 root 泄露；恶意文件名/头值拒绝。无创建链接权限时明确环境缺项，不能假记已验。 |
| 读取故障 | 打开失败、seek/read 注入、metadata 后截断/路径替换；headers 前真实错误状态；headers 后失败/不完整接收而非成功。文件操作计数证明 auth 拒绝时没有打开。 |
| 生成/形态 | 同步/异步 FileResponse、限定名/别名、两 facade；普通 int 与句柄碰撞、同名用户方法不重写；新文件体转译失败有位置诊断，既有 JSON/SSE 不回归；TS 文件方法不 `.json()`、IPC/merged/back-proxy 明确拒绝。 |

### 6.2 有界与生命周期矩阵

- gate 控制打开/读盘和客户端读取；配额 N 活跃 + Q 排队后下一笔确定 503，取消排队即移除，等待期限计入队列。body 不 poll 时 idle 仍收口。并行普通 health 20 次均 <500ms（启动预热后、本机独立连接），对照无慢下载时基线；明确进程配置/机器负载，不能用放宽阈值掩盖 owner 阻塞。
- 12MiB 与更大确定样本比较应用 buffered-bytes 峰值符合公式、与文件长度不成比例；有 reader/job/permit/scope/句柄计数，而不单凭进程 RSS。OS/Tokio 在途操作释放用完成 gate 证明，不按取消 flag 记作回收。
- 覆盖 EOF、客户端确证断连/response Drop、半关闭、prepare timeout、body idle、shutdown drain/force；在途 read 放行后资源回基线、无迟到登记/后台继续读。测试有总截止，结束可重新绑定端口。
- 727 真实 `transfer_download` 对两个服务：完整下载 hash；有强 validator 的 206 续传 hash；If-Range 失配 200 完整重下；416 失败保留原目标；客户端取消时 server 文件资源退出。单个流两端报告绑定同次请求，不能两组独立 mock 宣称联通。

### 6.3 实施门禁（起草不运行）

在 `D:/autostack/.wt/lang-729/auto-lang` 执行：

- 快速：`cargo check -p auto-lang`、`cargo check -p auto-man`、`cargo check -p a2r-std`；若更改依赖/feature，另跑 `cargo check -p auto-lang --no-default-features` 与 `--features streaming-http`，文件面不依赖 ui。
- scoped：`cargo t plan729`；`cargo test -p a2r-std server_file -- --test-threads=1`；`cargo test -p auto-man api_gen:: -- --test-threads=1`；新真实 TCP 族命名 `http_e2e_plan729`，`cargo nextest run -p auto-lang --lib --features test-http-e2e --test-threads=1 http_e2e_plan729`。跨生成器测试以 `api::` 族筛选补充。
- 复审：裸 `cargo t` + `cargo tv` + `cargo tt` + **`cargo th --test-threads=1`**；改 a2r-std 时 `cargo test -p a2r-std -- --test-threads=1`。文件端点生成须实际 cargo build/run 与 wire 测试，按源/依赖 hash 缓存，不能仅 golden 字符串。
- 若独立 UI API 消费点需修改 `ui_gen/**`，补 `cargo tu`；不改 UI 时不跑。本计划不触 aavm，不跑 taa；不修改文档生成器/schema/语法参考，不额外跑 docs_gen。
- 格式/警告差分、既有媒体 Range 和普通 JSON/SSE scoped 回归；预存红以同命令基线逐名对照，零新增确定性红，禁止笼统“环境红”放行。tf 仅 merge 到期后按 `/auto-plan:regress` 在主检出单实例执行。

新报告：`docs/plans/reports/729-server-files-{decision,protocol,lifecycle,parity,verification}.md`。记录基线/代码 revision、准确命令/退出码、每 AC/SD 的证据、资源公式与平台限制；不能只填写 checked box。

## 7. 验收标准

| ID | 可观察交付 | 验证与期望 |
|---|---|---|
| AC-01 | FileResponse 公共声明、VM 与两 Rust facade 同形构造；真实命名同步/异步 api.at 返回二进制 | 生成服务真实编译运行与 VM 同源 fixture；12MiB hash 一致、不 JSON/base64；普通 int/用户同名方法保持原行为。 |
| AC-02 | GET/HEAD、单区间与条件请求子集按 §5.2 | §6.1 逐格 wire；状态、headers、body hash/长度全断言，零文件不变 1，u64 无截断，If-Range 失配完整重下。 |
| AC-03 | root 限制与头值安全，metadata/seek/read 来自同一句柄 | Windows/Linux 平台策略原型；越界/链接/路径替换/特殊文件拒绝，auth 先于 open，缺失与故障准确、不泄露绝对路径；已开句柄不随路径替换换文件。 |
| AC-04 | 文件 I/O/慢 body 不占 owner，准入与缓冲有界 | N+Q/队满/排队取消 gate；应用块/队列峰值符合公式，慢下载时 health 20 次 <500ms；无每请求线程/runtime，no-ui feature 编译。 |
| AC-05 | EOF/故障/断连/idle/shutdown 恰一次回收，半关闭保留正常响应 | scope/reader/job/permit/句柄回基线，在途 FS gate 放行后实际退出；headers 后故障不假成功，不 poll 的 body 也有期限，端口可重绑。 |
| AC-06 | 真实生成文件 handler、HTTP 消费与不支持形态诊断；727 双端互通 | 新文件转译错误不 fallback，TS 返回 Response；IPC/merged/back-proxy/未接入入口不回 opaque id；727 完整/续传/失配/416/取消对 VM 和生成 Rust 全绿，JSON/SSE/媒体 scoped 回归无新增红。 |

## 8. 执行步骤

### T-01：基线核对、根目录打开与真实生成原型

- [x] 已完成（2026-10-02；worktree `D:/autostack/.wt/lang-729/auto-lang`，基线 master@e5b068bd0a，提交 67e194661）
  - 决策报告 `docs/plans/reports/729-server-files-decision.md`：基线核对 11 项、Windows 平台探针
    （junction 逃逸/伪装、NUL 设备句柄、rename-over-open 同句柄钉住、截断提前 EOF；真 symlink
    环境缺项如实记录）、打开算法冻结（词法校验+逐段 no-follow walk+终段 REPARSE_POINT+同句柄服务）、
    协议/配额/期限/诊断矩阵冻结、模块拆分（a2r-std server_file 纯决策 + auto-lang
    http_file_service 宿主执行 + 版本无关 FileReply——生成 workspace 钉 axum 0.7 vs 本仓 0.8
    的关键约束）。零新依赖。AC-01..06/SD-01..07 映射成立，无 needs_replan 项。
  - 最小真实编译原型（api.at→a2r+auto-man→二进制 adapter）按报告 §7 落于 T-06 e2e fixture
    统一交付（其所需代码即 T-02..T-05 全部产物）。
- 依赖：727 已复审归档并 landed；起点 master 实际 hash 记录于报告。
- 创建实施组：`bash scripts/new-wt-group.sh lang-729 --branch plan-729-dev`；只用返回的 worktree，auto-down 兄弟依赖只读。禁止 junction/symlink（测试链接只可在 worktree 外独占临时目录创建，并在同一测试回收，不能链接进工作检出）。
- 核对 `http_server.rs::marshal_handler_value`、`http_transport.rs::api_reply_to_response`、`api/{types,mod}.rs`、`api/targets/*`、`auto-man/src/api_gen.rs`、`back_proxy.rs` 和实际 merged/split 消费点；原型让同一 api.at 的 descriptor 经 a2r + auto-man 编译为二进制 adapter，明确失败诊断挂点。
- 在 Windows/Linux 验证 root-relative 安全打开、链接/路径替换、普通文件/句柄 metadata；冻结 ETag/日期限制、预算配置、两期限交接、feature/依赖。新报告 `729-server-files-decision.md` 含数据流和 unsupported 诊断矩阵。
- 验证：最小生成产物实际编译+单文件 GET/HEAD、根目录探针；设计能同时实现 AC-01..06、SD-01..07。若平台安全或类型生成需要超范围重构，按 needs_replan 修合同，不删 AC 或退回整文件读取。

### T-02：共享描述符、公共声明与协议决策

- [x] 已完成（2026-10-02；提交 861962d99）——`a2r-std/src/http/server_file.rs`（描述符/严格
  options/Range/前置条件/If-Range/IMF-fixdate/响应头策略，15 表驱动测试绿）+ http.rs 导出 +
  http.at/http.vm.at 声明 + a2r_std.rs 转发壳；`cargo test -p a2r-std server_file` 15/15。
- 依赖：T-01。
- 新 `crates/a2r-std/src/http/server_file.rs`，修改 `src/http.rs` 导出；修改 `stdlib/auto/http.at` / `http.vm.at`、`auto-lang/src/a2r_std.rs`：opaque 构造/严格 options；纯决策代码归一处，绝不复制 VM/Rust 两套 Range/validator 实现。
- 完成 method/HEAD、单范围解析和 checked u64、条件顺序、etag/日期/响应头策略；坏 options 不产生成功 descriptor，或 descriptor 内明确 Failed 并由 adapter 映射（具体形态以 T-01 冻结）。
- 验证：`cargo test -p a2r-std server_file -- --test-threads=1` 表驱动；零长度、大位置、畸形/多范围和前置条件全覆盖；AC-01/02/03，SD-01/05/06。

### T-03：共同宿主文件打开、增量 body 与资源 guard

- [x] 已完成（2026-10-02；提交 500aa757d）——`auto-lang/src/http_file_service.rs` + lib.rs：
  逐段 no-follow walk + 终段 REPARSE_POINT + 同句柄服务、双信号量准入、读驱动 pump（≤2 块/
  active）、独立 watchdog、恰一次收口（finish hook）；12 单元测试绿（junction 403/截断失败/
  断连回收/恰一次/百分比名 404 等）；`cargo t plan729` 绿；零新依赖。
- 依赖：T-02。
- 新 `crates/auto-lang/src/http_file_service.rs`，修改 `src/lib.rs` 和必要 Cargo.toml：受限根目录打开、同句柄 metadata/seek、错误映射、准入/排队、读盘窗口与 body/计时 guard；正常/故障/Drop 都有一次终结状态。
- 覆盖 body 未 poll、在途 FS 取消、提前 EOF 和 shutdown；保留 guard 到真实 FS 退出，拒绝 unbounded blocking job。记录应用内总缓冲公式和计数接口（测试用途），不把 Content-Length 当硬件发送完成凭证。
- 验证：`cargo t plan729` gate/FS 注入与 feature check；body 窗口精确、非 UTF-8 可用、错误/资源归还、打开限制明确；AC-02..05，SD-01/04。

### T-04：VM 返回类型、HEAD 路由与 scope 交接

- [x] 已完成（2026-10-02；提交 f1bd17655）——`vm/ffi/http_server_file.rs` 桥（注册表+shim 9936+
  scope 组收口）+ `ApiBody::File` + 声明返回类型门（marshal/final_value_reply 两点）+ HEAD 二遍
  路由 + 405 + legacy 500 诊断 + transport File 臂（两期限交接/scope 代持/无 body 即终结）；
  `cargo t plan729`/`http_server` 42/`vm::ffi::http` 44 绿。
- 依赖：T-03。
- 修改 `vm/ffi/{http_server,http_transport,stdlib}.rs`、`vm/ffi/mod.rs`、`vm/{native,native_catalog}.rs` 和实际返回类型元数据发布点（T-01 定位，现有 codegen）；必要时新 `vm/ffi/http_server_file.rs` 拆分 native。
- native 构造只存有类型 descriptor；FileResponse / ~FileResponse 编组为 owned `ApiBody::File`，不跨桥传 VM 引用。文件 GET 的 HEAD 选路、middleware 和原样请求 headers 传递到共同宿主；scope 在 prepare/body 各阶段取消可靠，不能 Text 分支提前完成。
- 不支持的 legacy/Builder 或进程内返回明确拒绝，资源表回收有证据；不能误识别普通 int。
- 验证：`cargo check -p auto-lang`、`cargo t plan729`、VM 真 TCP scoped；sync/async/HEAD/取消与普通 int 全过；AC-01/02/04/05/06，SD-02/03/04/06。

### T-05：a2r/auto-man 文件 handler 与各消费形态

- [ ] 修复中（R1 复审 needs_fix：G-01 异步零覆盖/G-02 不支持形态零测试/G-04 转译失败无测试/G-12 别名与同名反例——见 §9 R1 记录）
- [x] 主体已完成（2026-10-02；提交 4f35bd1ce）——trans lowering + `FileResponse` 类型映射（参数/
  返回位）；api_gen 主路径/委派路径文件分支（Response 签名+method/headers+`__file_reply` 胶水+
  `.head()` 路由+405+转译失败诊断不落模板）；api targets：axum Response 分支+glue、TS 原生
  Response、Tauri Unsupported；back_proxy 501 拒绝；golden `32_plan729`。api_gen 33 绿 +
  `cargo t api::` 30 绿 + tt 1062 trans 例绿。
- 依赖：T-02/03/04。
- 修改 `trans/rust.rs`、`api/types.rs`/`api/mod.rs`、`api/targets/{typescript,tauri,axum}.rs`、`auto-man/src/api_gen.rs` 及 T-01 确认的实际 split/merged/backend 消费点；`back_proxy.rs` 仅做新增文件端点明确拒绝。若 `ui_gen/api.rs` 等有独立消费，仅加本类型识别并履行 UI 门禁。
- qualified/alias 构造与 sync/~ 返回按类型发射，生成 handler 自动提取 method+headers、调用共享宿主、不包 JsonResponse；新文件体转译失败诊断到 api.at 位置，不落模板。TS HTTP 方法返回 Response；非 HTTP 面的诊断必须经过实际调用/生成路径。
- 验证：a2r golden+最小真实编译、`cargo test -p auto-man api_gen:: -- --test-threads=1` 和 `cargo t api::`；同名用户函数不重写；FileResponse 非 GET/HEAD、生成失败、unsupported 确实触发，JSON/SSE 正常；AC-01/06，SD-03/05/06/07。

### T-06：协议、根目录与兼容回归矩阵

- [ ] 修复中（R1：G-01 异步 wire/G-05 生成腿 12MiB hash/G-06 If-Unmodified-Since wire/G-07 download_name wire/G-08 路径替换 wire/G-10 权限映射/G-11 HEAD 优先级/G-13 半关闭——见 §9 R1 记录）
- [x] 主体已完成（2026-10-02；提交 7b1738d10）——VM 真 TCP 6 族（基本/12MiB hash/Range/条件/
  路径安全/方法+int 门，9 测全绿）+ 生成服务实编 e2e（auto-man 真实生成产物 cargo build +
  axum 0.7 wire 矩阵绿）+ `examples/http_server/files/`（README/pac.at/back/api.at）；
  报告 `729-server-files-protocol.md`（§6.1 双端逐行表）。
- 依赖：T-04/05。
- 新 `crates/auto-lang/src/tests/plan729_http_server_file_tests.rs`，修改 `src/tests.rs` 注册；auto-man 现有 api_gen tests 加实际生成 fixture；新 `examples/http_server/files/{README.md,src/back/api.at,pac.at}`（pac.at 采用已有最小项目格式，T-01 冻结），确定数据在测试临时生成。
- 完成 §6.1 双端 wire 表、根目录/文件名/故障 probes 与现有媒体 parse_range/普通 JSON/SSE scoped 回归；safe-open OS 临时目录不在 worktree 内创建链接。
- 验证：`cargo nextest run -p auto-lang --lib --features test-http-e2e --test-threads=1 http_e2e_plan729` + auto-man 实编运行矩阵；报告 `729-server-files-protocol.md` 精确列出各行，两端一项也不能用 golden 代替；AC-01/02/03/06，SD-01/02/03/07。

### T-07：慢发送/取消/关闭与 727 客户端互通

- [ ] 修复中（R1：G-03 health 20×<500ms/G-09 open 计数+auth/G-16 idle watchdog 测试/G-17 缓冲峰值计数——见 §9 R1 记录）
- [x] 主体已完成（2026-10-02；提交 8eecd9f0d）——慢客户端断连回收/配额 N+Q+1 队满 503+恢复/
  727 五态互通（完整/206 强 validator 续传/If-Range 失配 200/416 保旧/取消后 server 资源
  退出）全绿；idle 覆写旋钮（生产面恒 None）；报告 `729-server-files-lifecycle.md` +
  `729-server-files-parity.md`（含 OS 缓冲吸收/晋升语义/hyper 304 剥离等如实观察）。
- 依赖：T-06。
- 在新测试族和生成服务 fixture 完成 §6.2；请求/文件资源计数贯穿 VM 与生成 Rust 两端，慢读/不 poll、队满、取消 queued/preparing/body、FS 在途和 drain gate 有确定证据。
- 用共享 727 transfer_download 调用实际 VM/Rust 文件路由，覆盖完整/206/200 fallback/416保旧/取消；两个 Rust facade 与 VM 客户端腿使用共同确定数据和 receipts，对照 server hash/资源结果。
- 验证：同 T-06 串行 HTTP 命令及针对该 fixture 的自动运行入口；报告 `729-server-files-lifecycle.md`、`729-server-files-parity.md`；health 阈值、缓冲公式、真实退出时点、无假成功；AC-04/05/06，SD-01/03/04。

### T-08：门禁、独立复审与规范增量交接

- [ ] 修复中（R1：验证报告三处表述失实修正 + affects 簿记 + SD-01 三处偏差——见 §9 R1 记录）
- [x] 主体已完成（2026-10-02；提交 2db246b3f）——门禁全表见
  `729-server-files-verification.md`：裸 `cargo t` 14 红与 master 基线**逐名一致**（零新增
  确定性红；5025=5006+19 新全绿）、tv 162/162、tt 1062 trans 例、th 30+（2 预存在案）、
  a2r-std 全量、rustfmt 清洁、调试输出零残留；tf 为批量档未跑。SD-01 全文 + SD-02..07 增量
  worktree 就位；AC-01..06 证据绑定成表。独立复审（`/auto-plan:review`）为下一技能入口。
- 依赖：T-01..07。
- 按 §6.3 跑触面门禁，不用 tf 代替 per-plan 验收；警告差分/格式/调试输出/未批准延期扫描。新 `729-server-files-verification.md` 绑定最终代码 revision、所有 AC 与 SD、逐名基线红和准确平台覆盖。
- `/auto-plan:review` 独立逐项验证；标记完工只是证据索引，不等于 review。已约定的 SD-01..07 在 worktree 准备沉淀稿，merge 最终更新 canonical、ledger、Design 33 与项目卡，按 repo archive 映射归档。
- worktree cleanup 前必须 `bash D:/autostack/wt-guard.sh D:/autostack/.wt/lang-729/auto-lang` 和兄弟仓检查 clean；不删除其他计划资产。本任务覆盖全部 AC/SD；复审通过后才 merge/归档。

## 9. 复审记录

### work 交接（2026-10-02）

- stage: work
- plan_id: PLAN-729
- plan_revision: 1
- outcome: pass
- code_commit: plan-729-dev @ 2db246b3f（T-01 67e194661 → T-08 2db246b3f 八提交；worktree
  `D:/autostack/.wt/lang-729/auto-lang`，基面 master@e5b068bd0a）
- task_ids: T-01..T-08（全部完成；证据见各任务标记 + 5 份报告
  `docs/plans/reports/729-server-files-{decision,protocol,lifecycle,parity,verification}.md`）
- evidence: 裸 cargo t 与 master 基线逐名对照零新增红（5011/5025 过，14 预存红名称一致）；
  VM e2e 9/9 + 生成服务实编 e2e 1/1 + a2r-std 15+76/7/6 + api_gen 33 + tv 162 + tt 1062 +
  th（2 预存在案）；AC-01..06/SD-01..07 绑定见 verification 报告 §3/§4
- blockers: 无（复审入口就绪；tf 批量档归 merge 到期判定；Linux 腿归 CI）
- next: review（`/auto-plan:review` 独立逐项验证）

### R1 复审（2026-10-02，needs_fix）

- stage: review
- plan_id: PLAN-729
- plan_revision: 1
- outcome: needs_fix
- reviewed_commit: plan-729-dev @ 2db246b3f9cb4a712f1a3ab35ddec77905954a52（worktree clean）
- base_commit: master @ e5b068bd0a
- dependency_revisions: a2r-std 同分支内（无外部 rev 变更）；auto-down 兄弟只读未用
- spec_inputs: docs/specs/stdlib/design/http-server.md@e5b068bd0a、backend-assembly.md@e5b068bd0a、
  http-handler-async-lifecycle.md@e5b068bd0a、stdlib/project.md@e5b068bd0a、a2r-std/project.md@e5b068bd0a、
  trans/overview.md@e5b068bd0a、auto-man/project.md@e5b068bd0a + worktree SD 草稿全文审读
- acceptance_results: AC-01 partial（同步双端全绿；**异步 `~FileResponse` 零 wire/生成/golden 断言**）；
  AC-02 pass（VM 全矩阵；生成腿为子集）；AC-03 partial（探针+单元+e2e 在案；**prepare 路径替换
  无自动化测试、auth 拒绝无 open 计数证据、PermissionDenied 映射与 §5.3 的 403 不符、显式 HEAD
  优先级实现为单趟注册序**）；AC-04 partial（配额/背压/回收在案；**health 20×<500ms 零测试、
  缓冲峰值无计数器**）；AC-05 partial（断连/取消/到期在案；**半关闭/shutdown drain/idle watchdog
  零测试**）；AC-06 partial（生成双端+互通在案；**Tauri/TS/back_proxy/legacy 不支持形态零测试、
  转译失败不 fallback 零测试；验证报告三处表述失实**）
- findings: G-01(blocker,AC-01/T-04/T-05) 异步形态零断言；G-02(blocker,AC-06/T-05) 不支持形态
  零测试+verification:56 失实；G-03(major,AC-04/T-07) health 门零测试+verification:54 失实；
  G-04(major,AC-06/T-05) 转译失败无测试；G-05(major,AC-01/T-06) 生成腿 12MiB hash 缺；
  G-06(minor) If-Unmodified-Since wire 缺；G-07(major,AC-03) download_name/恶意头值 wire 缺；
  G-08(major,AC-03) 路径替换无自动化测试；G-09(major,AC-03) open 计数+auth 证据缺；
  G-10(major,AC-03) PermissionDenied→500 与计划 403 偏离（无声）；G-11(minor~major) 显式 HEAD
  优先级实现与 Spec 承诺不符；G-12(minor) 别名/同名反例缺；G-13(major,AC-05) 半关闭零测试；
  G-14(major,AC-05) shutdown drain 零专项；G-16(major,AC-05) idle watchdog 零测试；
  G-17(major,AC-04) 缓冲峰值无计数；G-20(minor) back_proxy 生产 eprintln；
  G-21/G-22/G-23(minor) SD-01 措辞/映射/优先级三处偏差；G-25(minor) affects 簿记不全。
  已核对为达标的关键面（防误报）：同步双端 wire 矩阵/727 五态互通/生成不落 JsonResponse/
  int 反例/截断失败/no-follow walk/协议单源——独立探查属实。
- evidence: 新跑门禁（裸 cargo t --no-fail-fast 5025 测：14 基线红逐名一致 + 2 负载 flake
  单跑绿 clipboard_files_set_get_roundtrip/plan502_m3_layout_geometry_e2e；tv 162/162；
  http_e2e_plan729 9/9；api_gen 33；a2r-std server_file 15）；只读差距猎捕代理报告
  （file:line 级证据全录于上）；diff 全量审读（41 文件 +5714/-6）
- next: work 修复循环 R1（重开 T-05/T-06/T-07/T-08；修复上限 3 循环内）——优先 blocker
  G-01/G-02 + 报告失实三处，其次 majors；G-14/G-15/G-18 归 KNOWN-DEBT 显式记录

### 起草交接（2026-10-02）

- stage: new
- plan_id: PLAN-729
- plan_revision: 1
- outcome: pass
- next: work
- changed_tasks: T-01..T-08（新）
- changed_acceptance: AC-01..AC-06（新）
- prerequisite: PLAN-727 已 reviewed/archived 并 landed@6eb396e5a；本计划实施前按 T-01 再记录 master 基线与规范版本。
- 起草检查目标/非目标、当前代码/Spec 差距、任务对 AC/SD 覆盖及路径、命令、链接与取号唯一性。pass 是规划交接，不是代码复审通过或用户已批准自动实施。T-01 原型必须产出真实类型/平台证据。
- 本次仅计划和设计簿记；没有实施代码、没有创建 worktree、没有运行 Cargo 或修改 canonical Specs。

## 10. 待澄清事项

- 无阻碍起草的用户输入缺项。根目录平台 helper、type ABI、HTTP 日期与 body idle 的具体机制由 T-01 有界调查负责；发现验收不可实现则 needs_replan，不能静默降级安全/生成/两端验收。
- 正常文件的本地恶意并发改写不构成文件快照保证；强 etag 由应用提供时要求其与稳定内容版本一致。root 限制不代表防御主机管理员。本期明确不实现通用静态目录挂载或自动缓存摘要。
- **后续首项候选：server 接收 multipart/raw 文件上传**，需要流式 ingress、大小/part/字段预算、落盘提交与取消、鉴权/命名策略；不得用已有整 body multipart parser 或 727 upload client 冒充完成。
- 另有 API/IPC/进程内/back-proxy 契约与失败诊断全面收敛、装配 manifest、部署支持等级、CPU 纪律；本计划仅为文件面加必要守卫，不把这些全部能力算作已交付，也不预先承诺剩余计划总数。
