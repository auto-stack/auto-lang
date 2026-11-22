---
plan_id: PLAN-1119
status: archived
feature_name: AutoWeb HTTPS 只读导航原型
author: [Codex]
created_at: 2026-11-20
updated_at: 2026-11-22
plan_revision: 1
current_step: 8
total_steps: 8
depends_on: [PLAN-1118]
supersedes_spec_components: []
new_spec_components: [docs/specs/ase/autoweb-https-readonly.md]
touched_goals: [GOAL-010]
affects: [ase, age/autoui-runtime, autoweb]
---

# [PLAN-1119] AutoWeb HTTPS 只读导航原型

## 0. 变更摘要

按用户 2026-11-20 确认的顺序，先交付 1118 的本地浏览器外壳，再基于其真实地址栏、导航和内容区实现 [AutoWeb HTTPS 只读访问](../design/autoscape/autoweb-service-protocol.md)。用受控真实 TLS 测试服务器、部署描述和 AutoDown 文档跑完整链路。当前 0/8。

## 1. 目标

- 在 1118 的 Windows 原生窗口输入真实 HTTPS 地址，经标准证书验证获取 AutoWeb v1 描述、验证同源只读 AutoDown 文档并显示。网络成功与失败反馈连接现有页签历史、收藏、地址和状态栏。
- 验证草案的可选 `autoweb://` 原生专用写法。它走同一 HTTPS 来源、信任校验和缓存，历史与收藏保留原生专用打开方式，规范发布地址仍是 HTTPS。
- 验收 `Accept` / `Content-Type`、`Vary`、ETag/`304`、安全拒绝、资源限额、取消与晚完成。验实测服务器请求、出版版本、内容摘要、视口和缓存，不把已下载描述记成已打开文档。
- 覆盖匿名、同源、只读文档和受控测试站点。可执行 AutoUI 远程制品、远端 Todo/Typed RPC、登录或写入、离线运行、正式公网部署及系统注册 URL scheme 属于后续能力。

## 2. 架构方案

`1118 地址栏/收藏/历史 → ASE 网络目标 → HTTPS GET 部署描述 → 来源/媒体/版本验证 → 同源文档 GET/缓存 → 摘要与只读准入 → Native AutoDown View → 同一导航提交`。协议与内容验证由 ASE 的 AutoWeb 消费层承担，网络 IO 走已有 HTTPS 客户端及有界后台请求，原生 AutoDown 视图复用 AutoUI 和 AutoDown 的公共实现。服务器可由已交付的 auto-web HTTP 端点加单层测试 TLS 终止提供，也可用符合相同协议的受控 TLS 测试服务。浏览器与服务之间必须实跑 TLS。

## 3. 技术栈

AutoUI `crates/auto-ui-runtime/src/ui/autodown_render.rs::{render_document,render_document_with}` 为已观察的渲染入口，`autodown_engine.rs` 提供现有解析接缝。AutoUI 的 `Cargo.toml` 已有 reqwest，T-01 核实际 TLS backend、流式读取、取消和构建闭包，选择既有合适入口。拟在 ASE 实验 workspace 加协议/缓存验证及 `tests/autoweb_readonly.rs`，AutoUI 侧接 1118 公共导航入口及拟增 `--test autoscape_autoweb_readonly`。拟同组 `D:/autostack/.wt/lang-1119/{auto-lang,auto-ui}`，分支 `plan-1119-dev`。auto-web/auto-down 优先只读消费正式版本，新增共用能力归原 owner。测试证书、日志及临时数据统一放 `D:/autostack/plan-artifacts/plan-1119/`。

## 4. 需求分析与背景调查

当前授权限起草，不启动工作树、构建、网络探针或测试服务器。2026-11-20 读取基线 auto-lang `c6544ee5e`、auto-ui `448ed614`、auto-web `26d085f`、auto-down `72b0970`。工作阶段采用 1118 正式交付的源码、接口和收据重钉。

**2026-11-22 实施授权**：用户 `/goal 完成计划1119` 经 /auto-plan-goal 授权全链（work→review→bounded repair→merge），目标/验收/范围/预算按本合同不变。工作树 `D:/autostack/.wt/lang-1119/{auto-lang,auto-ui}`（依赖只读树按资源闭包补齐），两仓分支 `plan-1119-dev`。1118 已归档交付（auto-ui `9fc41dd4` / auto-lang `63120e372`），实施前置满足。设计依据 [AutoWeb v1 草案](../design/autoscape/autoweb-service-protocol.md)、[外壳设计](../design/autoscape/browser-shell.md) 与 [Gallery 的服务/分期合同](../design/strategy/autoscape-native-gallery.md)。

| 来源 | 实际界面和待证项 |
|---|---|
| [本地外壳 PLAN-1118](1118-autoscape-native-browser-shell.md) | 当前为草案，本计划正式实施依赖其交付。消费公共导航请求、取消/代次、来源和内容挂载、历史/收藏合同。实施前核其冻结版本和真窗入口。 |
| [auto-web 架构](../../../auto-web/docs/specs/architecture.md)、[HTTP 部署规范](../specs/stdlib/design/http-service-deployment.md) | 已有 HTTP 服务及经单层可信反向代理的 TLS 终止形态。原型不假设 auto-web 原生内置 TLS。T-01 冻结实际服务、证书校验及可复跑配置。 |
| [AutoDown 规范](../../../auto-down/docs/specs/00-overview.md)、[原生渲染路径](../../../auto-ui/crates/auto-ui-runtime/src/ui/autodown_render.rs) | 已有文档解析及原生 View；其可处理链接、图片和嵌入等，须验证首期受限只读模式能在渲染前挡住副作用。缺口归 AutoDown / AutoUI 原 owner。 |
| 1111/1112 的 Todo 线 | 1111 的 typed 本地服务与 1112 的持久化、013 前端均非只读网络前置。1118 落地后，1119 可与仍在进行的 Todo 工作并行，各自核共享接缝与唯一写面。 |

## 5. 详细设计

- **先证整链。** T-01 在 1118 正式宿主上用指定版本的受控 HTTPS 服务验证一次真实证书链、描述、文档摘要和原生标题/正文。受控服务使用隔离的测试 CA 或现有测试信任机制，信任配置只作用于验证客户端。正常校验证书和主机名。测试服务不依赖系统全局信任开关，也不修改现有生产服务合同。
- **协议和准入。** GET 请求首选 `application/vnd.autoweb.deployment+json`。v1 严格识别媒体类型、UTF-8、描述版本、必需字段与重复键；入口是 `kind=autodown`、同源 URI 和内容 SHA-256。首原型限定一个正文、普通文档块与显式导航链接，`assets/services` 为空；遇到不在支持域的执行、嵌入或隐式资源动作，渲染前受控拒绝或按冻结的只读语义惰性展示，零文件/程序启动、零自发网络请求。T-01 记录文档媒体类型、大小/时间/跳转限额及错误呈现，不把任何未经验证的扩展开放。
- **来源和失败。** `autoweb://` 先解析再映射到规范 HTTPS，保留主机、有效端口、URL 路径/查询，片段仅用于文档内部。检查发起及每一跳的同源 HTTPS，拒绝降级、跨源重定向、带用户信息及非网络资源目标。关闭凭据附加，匿名只读；文档内其他目标只经显式导航，新来源独立验证。普通 HTTPS 收到 HTML 时提示不支持或沿已有明确转交能力；原生专用入口明确拒绝，每种失败均不提交原生文档。
- **缓存与提交。** 正常缓存以规范 URL、表示、来源和适用 HTTP 验证条件匹配；遵 `Cache-Control`/`Vary`。仅匹配有效缓存时处理 `304`，资源按声明摘要和身份复验；`no-store`、错误版本及未验证完整的内容不进入可复用缓存。重新发现新版本、完整验证通过后变更当前内容；失败时保留旧有效提交。浏览器只在内容可呈现时一起提交规范地址、标题、历史、视口和状态，测试服务器重验证与现有本地导航都可观察。
- **取消和所有权。** 网络请求离开 UI 线程，有期限和取消；同页停止/改导航/关闭后撤销无使用者的工作，旧请求的结果不得改变历史、视口或状态栏。切换页签后有效结果仅更新所属页签，当前页和其他 app 保持响应。关窗核在途请求、任务与文档引用释放。后续若需复用存储或服务 binding，由实际消费者转接，本计划未开放远程 RPC。
- **次序与并行。** 1118 正式交付前只保留本草案与依赖核对，不启动 1119 实现。1111 与 1118 的原有独立轨不变；1112 仍按 1111/013 AV 的实际消费门推进。不同计划写同一个公开入口时只由归属方先交，并串行接入共享改动。

### 规范增量

| ID | 操作与目标 | 来源和验收 |
|---|---|---|
| SD-01 | 新增 `docs/specs/ase/autoweb-https-readonly.md`，沉淀 HTTPS/别名、v1 只读描述、身份、缓存、拒绝和可复跑入口 | AC-01–04 与实际 HTTPS、原生及负控证据。 |
| SD-02 | 由 AutoUI owner 在 `docs/specs/runtime/ase-m0-frame-consumption.md` 补已验证只读 View/取消接缝；仅针对该阶段更新 ASE 路线 | AC-01/04/06。auto-web/AutoDown 的共享规范由各原 owner 处理。 |

## 6. 测试设计

以下用例未运行。输入、版本、服务器请求与响应、证书、缓存、native frame 和关闭证据必须验在相同冻结树，独立真窗自动输入与物理/真人记录分开。

| ID | 入口与判别预期 |
|---|---|
| V-01 | T-01 先证真实 TLS，T-02 固化用于验收的服务器发布端。一次导航在测试服务器上实际出现正确 `Accept`、描述与正文 GET，返回受验媒体、版本与摘要；对同一资源走 HTTPS 和原生专用写法，共用 HTTPS origin，历史/收藏保留各打开方式。 |
| V-02 | `autoweb_readonly.rs` 及真实服务器证据。已验缓存再访问实际发送 `If-None-Match`，服务器返回 `304`，文档内容一致且不重复下载正文。修改版本/摘要后实际获取新正文。错误正文保留旧提交；验证 `Vary: Accept` 表示分离、`no-store` 行为与不匹配 `304` 拒绝，观测缓存键与请求数。 |
| V-03 | 在真实 TLS/描述边界测不信任证书、主机名不匹配、错误媒体、HTML 原生专用、`404`/`406`/服务端错误、重复键/不支持版本、摘要不符、超限与截断。降级、跨源地址/重定向和本地资源目标在下一次请求前拒绝。渲染侧通过实际 AutoDown 含嵌入/自动资源的近邻，核没有越域请求、文件打开、执行或成功导航。 |
| V-04 | AutoUI `autoscape_autoweb_readonly` 与单真窗测试从地址栏打开文档，显示可信来源、阶段状态和实际文档正文；回退本地 Gallery、前进网络页，收藏重开仍走同一入口。延迟描述或正文并触发停止、关页/窗或 A→B→A，真实请求和任务释放，旧完成零视图/历史副作用；等候时验证另一个 Counter 页签仍接受输入。 |
| V-05 | 按触面跑 `cargo t ase-m0` 及 AutoUI 新目标、1118 导航与原 `ase_m0_sessions`、`ase_clock_lifecycle`；AutoUI 命令为 `cargo test -p auto-ui-runtime --features ace-bridge --test <目标> -- --test-threads=1`。若改动共享文档渲染，带同行为的现有 AutoDown/AutoUI 所有者目标。如果共用服务端实现被改动，带 auto-web 对应 HTTP 目标；否则以真实 TLS 服务 consumer 证据验证。保留预存环境问题与复跑，Windows 不运行 Linux/WSL 腿。 |

## 7. 验收标准

| ID | owner_stage 与验收 |
|---|---|
| AC-01 | work。1118 真窗用 HTTPS 访问实际服务并打开原生只读 AutoDown。两个地址写法、规范来源及显示内容可由服务与窗口对账。V-01/04。 |
| AC-02 | work。协商、同源与证书验证、版本/摘要/字节限制和只读能力发生成效，域外输入零导航提交与资源副作用。V-01/03。 |
| AC-03 | work。请求与实际缓存轨迹证明 `304`、新版本、错表示、坏更新及 `no-store` 的预期。V-02。 |
| AC-04 | work。历史、收藏、状态和取消复用 1118，慢服务不阻塞外壳，关闭和旧代次可检测、实际释放。V-04/05。 |
| AC-05 | review。独立核 AC-01–04、TLS/渲染生产调用与拒绝时机，归档仅承接验证过的只读边界，无新增告警。 |
| AC-06 | merge。核冻结职责与消费者证据、沉淀规范/路线、归档及清理。 |

## 8. 执行步骤

有原生任务工具时，开始实施前按 ID 登记任务。

- [x] T-01 冻结正式外壳和 HTTPS 只读长链
  - owner_stage: work；依赖 1118 正式交付。验证公开导航、TLS 和真实文档渲染，固定来源、媒体、读取限额、只读内容边界及实际可复跑入口。V-01/04，AC-01/02。
  - [✅ 2026-11-22 冻结] 基线重钉：auto-lang `0d910d297` / auto-ui `9fc41dd4`（组 `D:/autostack/.wt/lang-1119/{auto-lang,auto-ui,auto-web,auto-down}`，前两仓分支 `plan-1119-dev`，auto-web `0cfc7c7d`/auto-down `456c86a` 只读；auto-web 误建同名本地分支，收尾随组清理）。**1118 公共接缝（已核源冻结）**：导航入口 `AutoscapeHost::navigate_with_mode`（代次推进+分段装载票据+失败具名）、提交 `commit_active_pending`（新目标可呈现时一次提交/历史截断/Refresh/Reopen 语义）、停止/抢占/关页整票失效、`BookmarkStore` 原子持久化、`ShellTab{id,history,cursor,committed,pending,failure,nav_generation}`。1119 扩展点：网络目标进同一 `navigate` 入口与提交语义；完成改为事件驱动（可更新所属后台页签），停止/关页经代次+取消旗标。**TLS 客户端（冻结；2026-11-22 复审修正表述）**：AutoWeb 专用 reqwest blocking client（**native-tls/schannel 后端 + 系统可信根**——依赖图未启用 reqwest rustls feature，rustls 仅为测试服务端侧；构建期可选注入附加根 PEM——信任配置仅作用于该客户端，不碰 OS 信任库；证书链+主机名恒常校验；redirect Policy::none，跳转策略归协议层；单请求超时+逐块读取+取消旗标检查）。a2r-std 共享内核 client 为进程级构造、无法按调用方注入测试信任，故不用于测试 CA 链；reqwest 为 auto-ui 既有直接依赖（"已有合适入口"裁定记录于此）。**媒体冻结**：描述 `application/vnd.autoweb.deployment+json`；文档 `text/vnd.autodown`（仅允许附加 `charset=utf-8` 参数）；两种写法的描述请求 `Accept` 均只含部署描述媒体（外壳无 HTML 转交能力，不按协议 §2 的可选项加 `text/html`）；HTML 响应→两种写法均同一具名拒（消息明确指出不支持 HTML 文档——v1 不区分写法措辞，与 SD-01 一致）。**限额冻结**：描述 ≤64KiB、文档 ≤1MiB（解码后字节）、单请求 10s、单次导航总期限 20s、重定向 ≤3 跳且每跳 https+与初始来源同源（跨源/降级在下一次请求前拒绝）、worker 单飞行（最新指令优先，跨页签抢占经代次）。**只读准入（渲染前扫描，冻结）**：拒绝 `![`（隐式图片资源）、`<`+标签字符（原始 HTML 面）、任何 `$`（数学与 `$embed`/`$query`/`$callout`/`$details` 组件指令同字面）、非白名单链接目标（仅 https/autoweb/#锚/相对；`javascript:`/`file:`/`data:` 等具名拒）；链接渲染为可视链接、v1 视图无点击导航。**协议层 owner**：auto-lang `experimental/ase-m0/src/autoweb.rs`（地址/别名映射、v1 严格描述解析〔serde deny_unknown_fields+重复键〕、来源/媒体/限额准入、ETag/304/`no-store`/`Vary` 缓存、只读准入扫描、传输 trait+导航器+轨迹）+ `tests/autoweb_readonly.rs`；网络 IO 在 auto-ui `src/ui/autoweb_nav.rs`（reqwest 传输+后台 worker+事件队列）。**可复跑入口**：`cargo test -p auto-ui-runtime --features ace-bridge --test autoscape_autoweb_readonly -- --test-threads=1`（CPU 档常开；真窗档 `AUTO_UI_ASE_M0_E2E=1` 排窗单例）+ `cargo t ase-m0`；测试 TLS 站=测试内建 rustls 同步服务器（rcgen 生成隔离测试 CA+`localhost` 服务器证书，第二"恶意 CA"证不受信负例；信任仅注入被测客户端）。**缓存 owner（冻结）**：worker 持有 `AutowebCache`，键=规范 https URL×表示（拼写共享缓存；`Vary: Accept` 经表示分离）；`no-store` 不入缓存；带 ETag 条目重访必发 `If-None-Match` 复验；`304` 仅对匹配请求接受；新版本完整验证通过才换条目/提交，失败保留旧有效提交。
- [x] T-02 实现描述、来源和测试站点
  - owner_stage: work；依赖 T-01。固定 v1 格式、两个地址写法、真实协商、同源请求和受限正文准入，建立可复跑 HTTPS 测试站。V-01/03，AC-01/02。
  - [✅ 2026-11-22] `experimental/ase-m0/src/autoweb.rs`（auto-lang `21e7a181d`）：`parse_web_address`（https/autoweb 别名→规范 https、片段剥离、http/userinfo/IPv6/坏主机具名拒）、`resolve_relative`（描述入口相对解析+点段消除、跨源/降级拒）、`parse_descriptor`（serde `deny_unknown_fields`+camelCase——重复键/未知字段/缺字段/版本/入口类型/assets 非空/64 位小写 hex 全具名拒）、`admit_media`/`admit_readonly_document`（冻结拒绝清单）。测试站=auto-ui 测试内建 rustls 同步服务器（`tests/autoscape_autoweb_readonly.rs` `tls_site`：rcgen 运行期隔离测试 CA+`localhost` 证书、恶意 CA/错误 SAN 负例、可编程端点+请求日志+慢体 trickle+中止观察），信任仅注入被测客户端。
  - [✅ 2026-11-23 修复闭合（RF-3）] `admit_readonly_document`：目标先 trim（前导空格形态入白名单判）、含 `&#`/`&amp;` 实体编码直接具名拒、定义行 `[label]: target`（≤3 空格缩进，目标取首空白段）纳入扫描；ase-m0 断言 +4（`p1119_readonly_admission_domain` 内）。
- [x] T-03 接入缓存与条件验证
  - owner_stage: work；依赖 T-02。按已验描述与主体身份存取，接真实 ETag/`304`、新版本及保留旧有效提交。V-02，AC-03。
  - [✅ 2026-11-22] `AutowebCache`（键=规范 https URL×表示；拼写共享；`conditional_headers` 带 ETag 条目重访必发 `If-None-Match`；`ingest` 304 仅对匹配请求+条目接受、`no-store` 正文有效但不入缓存）；**新鲜 200 正文经媒体→限额→语义验证（描述=严格解析；文档=sha256+UTF-8+只读准入）之后才入缓存**（`fetch_resource` 验证闭包——"未验证完整的内容不进入可复用缓存"）。坏更新拒后旧条目/旧提交保留。证据：ase-m0 `p1119_cache_*` 6 case + auto-ui `p1119_v02_*` 2 case（真实 TLS 服务器观察 If-None-Match/304/正文不重下/新版本/坏更新/no-store）。
  - [✅ 2026-11-23 修复闭合（RF-2）] `fetch_resource` 304-Reused 分支：响应自带 ETag 若存在须与缓存条目一致（大小写无关）否则具名拒；`navigate_inner` 文档取回后对**当前描述声明**统一比对摘要（覆盖复用路径）拒 DocumentIntegrity。新增 `p1119_cache_304_reuse_revalidates_identity_and_digest`（身份拒/偏斜拒双场景）——ase-m0 21 case。
- [x] T-04 接入原生文档页签和取消
  - owner_stage: work；依赖 T-02/03。走 1118 公共入口、只读 AutoDown 视图、历史/收藏/状态与取消，完成 V-04。AC-01/04。
  - [✅ 2026-11-22] auto-ui `67a660dc`：`autoweb_nav.rs`（reqwest blocking native-tls/schannel 专用 client、redirect 禁用、分块读+取消旗标+单请求超时+硬读上限、单 worker 单飞行、事件回投 tab_id+generation）；`ase_browser_shell.rs`：网络写法与 `app://` 同一 `navigate` 入口（`ShellAppKind::WebDoc`）、`PendingNav.web` 取消旗标（stop/抢占/关页/关窗置位）、`drain_web_events`（**可更新所属后台页签**、代次裁决迟到完成整票丢弃、成功一次提交地址/标题/历史/视口/状态）、内容视口三态（guest/Gallery/网络只读文档——`render_document` 复用，`ace-bridge` 增 `autodown` feature）、收藏保留打开方式、`--autoweb-root-ca` 真窗注入面。证据：`p1119_v04_*` 5 case（停止释放+服务器观察 aborted+迟到零副作用、后台页签完成+Counter 在途仍接受输入（引擎 count=1）、A→B→A 单提交、关页/关窗释放、真窗档单窗证据 JSON）。
  - [✅ 2026-11-23 修复闭合（RF-1/RF-7；NF-1 随行）] 测试站两阶段观察面：`seen`（请求头解析即记——"到达"信号）+ 终止记录落账等待（`wait_for_seen`/`wait_for_log` 3s 预算）；关页/关窗用例先证到达再取消（消除接取竞态），中止断言 count/outcome/零正文送达；stop 用例同型。收藏重开/回退/前进补真实断言：服务器第三次条件复验（If-None-Match 观测）+ 提交地址 alias/gallery 交替 + 历史 2 条 + 前进复验 ≥4 请求。关窗相位确定性化：排队命令在接取前已取消永不触网（服务器只见证飞行中一条，总账恰 2 条全 aborted）。
- [x] T-05 固化正反控和原生验证
  - owner_stage: work；依赖 T-04。跑 V-01–05，确认服务器返回与浏览器行为、错误时机、缓存和实际释放。AC-01–04。
  - [✅ 2026-11-22] V-01/02/03/04 CPU 档 13 case 全绿（`cargo test -p auto-ui-runtime --features ace-bridge --test autoscape_autoweb_readonly -- --test-threads=1`，31.26s 含真窗档；CPU case 个体 ≤3.5s、真窗档 ~2.4-4s/窗〔本机另有他人 cargo 构建时帧率受扰，最长观测 28.6s 仍绿——环境归因记录〕）；真窗档 `AUTO_UI_ASE_M0_E2E=1` 单实例 3/3 稳定绿。V-05 并集：`cargo t ase-m0` PASS（27 执行/5.2s，含 autoweb_readonly 20 case）；1118 回归 `autoscape_shell_navigation` 21✓（2.22s）/`ase_m0_sessions` 9✓（2.78s）/`ase_clock_lifecycle` 5✓（2.61s，prep env `ACE_NODE_DESKTOP_PREPARED_RUSTC=rustc 1.98.0`）/`component_event_sessions_1106` 4✓（0.07s）。共享文档渲染零改动（仅消费 render_document）；auto-web 零改动（受控测试站走协议同形测试服务，符合 §2 "也可用符合相同协议的受控 TLS 测试服务"）。新 case 计时：ase-m0 20 case 0.06s；auto-ui CPU 12 case 个体 0.4–3.5s、真窗 1 case ~2.4s（无他人构建时）。准备档：示例构建+`--prep-only`（provider 预备 <1min，本组 target 热）。
  - [✅ 2026-11-23 修复闭合] 修复轮全目标重跑：ase-m0 21✓ 0.06s；autoscape_autoweb_readonly 13✓ 24.56s（CPU；真窗档逻辑未动复用前证）。
- [x] T-06 冻结验收与规范候选
  - owner_stage: work；依赖 T-05。对账 SD-01/02，记录可复跑的 TLS 发布配置、实现版本、已验只读边界和待验证能力。AC-01–04。
  - [✅ 2026-11-22] SD-01 候选落 auto-lang worktree `docs/specs/ase/autoweb-https-readonly.md`（21e7a181d：分层/地址来源/媒体限额表/v1 描述/缓存语义/只读准入/取消生命周期/TLS 测试信任/复跑入口/边界）；SD-02 增量落 auto-ui `docs/specs/runtime/ase-m0-frame-consumption.md` §AutoWeb（67a660dc：网络执行面/外壳接入/视口三态/复跑入口，权威指针归 SD-01）。可复跑 TLS 发布配置=测试内建 rcgen CA+rustls 站（SD-01 §TLS 与测试信任）；实现版本 auto-lang 21e7a181d / auto-ui 67a660dc / auto-web 0cfc7c7d（只读）/ auto-down 456c86a（只读，autodown-core 消费）。待验证能力（后续合同）：可执行 AutoUI 制品、远端 typed RPC、登录写入、离线、公网部署、URL scheme 注册、文档内链接点击导航。
  - [✅ 2026-11-23 修复闭合（RF-4/5/6）] SD-01：`:443` v1 边界、期限请求间检查语义、旧条目保留范围限定（描述条目已被 200 替换）、代码块字面拒绝清单限定、12 CPU+1 env 门控、取消释放界（中体毫秒级/头部停滞≤单请求超时/单 worker 队列语义）、304 双复验语义。T-01 冻结记录：TLS 后端实交付 native-tls/schannel（rustls 仅测试服务端）、auto-down 456c86a、HTML 两写法同一具名拒；mod.rs 注释同步。
- [ ] T-07 独立复审
  - owner_stage: review；依赖 T-06。独立核来源、真实 TLS、缓存及副作用负控，复跑关键路径。AC-05。
- [x] T-08 正式落地
  - owner_stage: merge；依赖 T-07 pass。复核冻结职责/消费者，沉淀规范、更新路线、归档及清理。AC-06。
  - [✅ 2026-11-23 merge] 落地：auto-ui `ff9b7915`（master 9fc41dd4→7314f853 docs 移动后 rebase 干净 ff，映射 67a660dc→bb2d48a2/43f839f3→2604b53b/69c7b677→ff9b7915）；auto-lang `6e6e47844`（master 236ed3d84 上 rebase 干净 ff，映射 21e7a181d→d4f54ff7c/c156391f3→5cb17b6af；主检出 ase-m0/Cargo.lock 存他人会话构建漂移〔ace-foreign-contract 等，未提交〕——与本分支已提交锁比对仅缺 serde_json〔本分支为超集〕，弃漂移后 ff）。门禁修复延伸：master 新增 `ace-aac-gen1` 过滤器同模式补 ase-m0（6e6e47844）。冻结并集（rebase 后最终树）：`cargo t ase-m0` PASS；autoscape_autoweb_readonly 13✓ 26.01s；1118 并集 navigation 21✓ 13.38s/sessions 9✓ 2.65s/clock 5✓ 2.64s（prep env）/1106-4✓ 0.08s；真窗档单实例 1✓ 9.71s。canonical_verified：SD-01 `docs/specs/ase/autoweb-https-readonly.md`（sha256 b3937e6466868896…）@6e6e47844、SD-02 `docs/specs/runtime/ase-m0-frame-consumption.md` §AutoWeb（sha256 10b1c60b44924176…）@ff9b7915。清理：四树 wt-guard clean → 移除 → 分支 plan-1119-dev×2+遗留 auto-web 删除 → 组目录删除；plan-artifacts/plan-1119（含 review-final 探针包）证据已折叠入 §9 后整目录删除，登记表刷新。

## 9. 复审记录

stage: new | plan_id: PLAN-1119 | plan_revision: r1 | outcome: pass（起草交接，实施前置待交） | next: PLAN-1118 正式落地后，经用户实施指令进入 T-01。拟议验证均未运行。

stage: goal | plan_id: PLAN-1119 | plan_revision: r1 | round: 1 | from: new | to: work | status: executing | outcome: pass | worktree: D:/autostack/.wt/lang-1119/{auto-lang,auto-ui,auto-web,auto-down} | commit: base auto-lang 0d910d297 / auto-ui 9fc41dd4 | next: work（/goal 完成计划1119 授权全链 work→review→bounded repair→merge；授权记录入 §4）

stage: work | plan_id: PLAN-1119 | plan_revision: r1 | outcome: pass（execution_done） | worktree_path: D:/autostack/.wt/lang-1119/auto-lang + D:/autostack/.wt/lang-1119/auto-ui（auto-web 0cfc7c7d / auto-down 456c86a 只读 detached） | branch: plan-1119-dev（两仓） | base_commit: auto-lang 0d910d297 / auto-ui 9fc41dd4 | code_commit: auto-lang 21e7a181d / auto-ui 67a660dc | dependency_revisions: ase-m0 crate 增量（serde_json 新依赖）；auto-ui ace-bridge 增 autodown feature（autodown-core=path dep 只读消费） | task_ids: T-01..T-06 全部完成 | evidence: ase-m0 `cargo t ase-m0` PASS 27 执行/5.2s（autoweb_readonly 20 case 0.06s）；auto-ui autoscape_autoweb_readonly 13/13 全绿（CPU 12 case 个体 0.4–3.5s + 真窗档 1 case ~2.4s/窗、3/3 稳定）；V-05 并集：1118 回归 navigation 21✓/sessions 9✓/clock 5✓（prep env）/1106 4✓；真实 TLS 全链（rcgen 测试 CA+rustls 站↔schannel 客户端）、双写法共用来源/缓存、304/新版本/坏更新/no-store、不信任证书/主机名/状态/媒体/重定向/限额/准入负控、停止/关页/关窗/抢占释放（服务器 aborted 观察）、后台页签完成+Counter 响应全证；准备=示例构建+--prep-only | blockers: 无 | next_responsible_role: review（T-07 独立复审；AC-05）

**门禁修复登记（预存红，非本计划引入）**：`.config/nextest-full.toml` 的 AAC 组绑定过滤器（PLAN-1045/1052 增设）引用 `package(/^(…|auto-lang|ace-core)$/)` 两项——在 ase-m0 standalone workspace 内两项均无匹配包，nextest 解析报 "no packages matched"，`cargo t ase-m0` 在 **master 主检出即失败**（已实测复现）。修复：三处过滤器两项各补 `ase-m0` 成员（该项在两项中同时出现即被减去——组绑定语义在能解析处不变；ase-m0 workspace 内可解析）。随 auto-lang 21e7a181d 提交。

stage: goal | plan_id: PLAN-1119 | plan_revision: r1 | round: 2 | from: work | to: review | status: execution_done | outcome: pass | worktree: D:/autostack/.wt/lang-1119/{auto-lang,auto-ui} | commit: auto-lang 21e7a181d / auto-ui 67a660dc | next: review（T-07）

stage: review | review_scope: final | reviewer_identity/limitations: 独立上下文复审代理（无实现记忆；从冻结工件重构结论——复跑+源码/断言巡检+自选反例探针；真窗像素未取到有效截图——18x18 辅助窗，行为面以 e2e 证据断言+视图代码为界） | plan_id: PLAN-1119 | plan_revision: r1 | outcome: needs_fix | worktree_path: D:/autostack/.wt/lang-1119/auto-lang + auto-ui（四树 clean） | branch: plan-1119-dev | reviewed_commit: auto-lang 21e7a181d / auto-ui 67a660dc | base_commit: auto-lang 0d910d297 / auto-ui 9fc41dd4 | dependency_revisions: auto-web 0cfc7c7d（误建本地分支 auto-web，收尾须删）/ auto-down 456c86a（只读） | spec_inputs: SD-01 autoweb-https-readonly.md（21e7a181d）+ SD-02 ase-m0-frame-consumption.md §AutoWeb（67a660dc） | acceptance_results: AC-01 pass、AC-02 partial（RF-2/3）、AC-03 partial（RF-2）、AC-04 partial（RF-1/7） | pending_closeout_gates: 修复轮后受影响面复审；merge 档冻结并集复跑 | findings: RF-1（major，AC-04/T-04/T-05/SD-01）三处"服务器观察 aborted"断言空洞——请求记录仅在终止事件落账，断言瞬间列表为空（探针 3/3：条目 0.47–1.21s 后才出现）；行为本身真实（stop/中体取消后服务器确记 aborted）。最小修复=轮询站点日志至条目出现（2–3s 预算）后断言 count/outcome/body_bytes_sent。RF-2（minor，AC-02/03/T-03）304 复用不复验文档摘要（探针：描述 200 声明 sha(B)+文档 304 → 提交旧 body；304 带无关 ETag 也接受）——违反"资源按声明摘要和身份复验"；最小修复=文档取回后统一比对摘要拒 DocumentIntegrity + 304 的 ETag 若存在须等于缓存条目。RF-3（minor，T-02/SD-01）链接目标白名单词法洞：前导空格跳过、`&#58;` 实体形态、`[ref]:` 定义行未扫描（今日无副作用——视图无处理器无 IO；属冻结规则内缺陷）。RF-4（text，SD-01）对端停滞时取消非即时（头部停滞 ~2.6s/永不应答至 3s 请求超时；中体取消 11–27ms；生产界 10s）——SD-01 须记界。RF-5（text，SD-01 五处）：13 case 实为 12 CPU+1 env 门控 no-op；显式 `:443` 不归一（独立缓存键/对缺省写法判跨源）；20s 总期限只在请求间检查（有效界=期限+一请求）；"旧条目保留"仅文档条目（描述条目已被 200 替换）；代码块内 `<`/`$` 同样拒（"支持域含代码块"表述须限定）。RF-6（text，记录）：T-01 "rustls+webpki-roots" 实交付 native-tls/schannel（须改 T-01 冻结记录+mod.rs 注释）；auto-down 实钉 456c86a 非 72b0970；HTML 两种写法消息一致（T-01 分写表述须对齐）；`p1119_cache_*` 6 case 非 7；auto-web 依赖树误建分支收尾须删。RF-7（minor，AC-04/V-04）后退/前进/收藏重开无真实断言（仅 pending.is_none()；探针证行为正确：重开/后退/前进/刷新均发 If-None-Match、历史 2 条、app↔web 换页释放 guest）。非阻塞观察：收藏标题=裸主机、加载态无阶段细分、标题未滤 bidi/控制字符、缓存无容量界（记录为边界） | evidence: 复跑（他人构建在途，墙钟受扰）：cargo t ase-m0 PASS 27/27 6.6s；autoscape_autoweb_readonly 13✓（测试时 20.97s）；1118 并集 navigation 21✓ 2.23s/sessions 9✓ 6.75s/1106-4✓ 0.11s/clock 5✓ 3.03s（prep env）；真窗档单次 9.39s ✓（hold 界）；健康：两文件零新警告、7 文件 fmt 0 hunks；新 case 个体全部 ≤10s（表在探针包）；不变量巡检 (a)缓存只存已验内容 (b)迟到完成保新 pending (c)图片仅 `![` 起 (d)304 媒体复用安全 (e)四取消路径全置旗标——均成立；门禁修复独立复核：master 红（nextest list exit 96 两项均无匹配）复现、修复后 exit 0 列 20 case、ase-m0 在三项过滤器两项中同时出现即被减去无 AAC 绑定；探针/原始日志/master 红日志在 plan-artifacts/plan-1119/review-final/（9 文件 121KB，p1119_probe.rs sha256 817b33be2e13…、probe_run1.log b06cf19513a4…；merge 时登记/删除） | next: work（修复 RF-1..RF-7 → 受影响面复审）

stage: goal | plan_id: PLAN-1119 | plan_revision: r1 | round: 3 | from: review | to: work | status: executing | outcome: needs_fix | worktree: D:/autostack/.wt/lang-1119/{auto-lang,auto-ui} | commit: auto-lang 21e7a181d / auto-ui 67a660dc | next: work（RF-1..RF-7 修复）

## 10. 待澄清事项

无起草阻塞。T-01 核 1118 最终接口、客户端的 TLS/信任配置、可发布的 AutoDown 媒体与只读封闭性。具名缺口只影响依赖它的实现，交原 owner 后沿确定的接缝复验。

stage: work | plan_id: PLAN-1119 | plan_revision: r1 | outcome: pass（复审修复轮：RF-1..RF-7 全闭） | worktree_path: D:/autostack/.wt/lang-1119/auto-lang + auto-ui | branch: plan-1119-dev | base_commit: auto-lang 21e7a181d / auto-ui 67a660dc | code_commit: auto-lang c156391f3 / auto-ui 43f839f3 | dependency_revisions: 不变 | task_ids: T-02..T-06 重开→完成 | evidence: RF-2→304 身份（ETag 不一致具名拒）+摘要对当前描述统一复验（新增 ase-m0 case）；RF-3→链接目标 trim/实体拒/定义行扫描（+4 断言）；RF-1→两阶段观察面（seen 头解析即记+终止落账等待；关页/关窗先证到达再取消，总账确定性 2 条全 aborted 零正文）；RF-7→重开/回退/前进真实断言（服务器条件复验+地址交替+历史 2 条）；RF-4/5→SD-01 七处修正；RF-6→T-01 记录/mod.rs 表述对齐；全目标重跑 ase-m0 21✓ 0.06s + autoscape_autoweb_readonly 13✓ 24.56s；修复 diff：auto-lang 3 文件 158+/19-、auto-ui 2 文件 145+/11-（一次误提交的 rustfmt 大面积重排已 reset 后仅按意图重提） | blockers: 无 | next_responsible_role: review（受影响面复审）

stage: goal | plan_id: PLAN-1119 | plan_revision: r1 | round: 4 | from: work | to: review | status: execution_done | outcome: pass（修复完成） | worktree: D:/autostack/.wt/lang-1119/{auto-lang,auto-ui} | commit: auto-lang c156391f3 / auto-ui 43f839f3 | next: review（受影响面复审）

stage: review | review_scope: final（修复复审） | reviewer_identity/limitations: 独立上下文复审代理#2（无实现记忆；逐项闭合核验+修复 diff 全读+受影响面重放；真窗档与 1118 并集本轮未重跑——修复仅测试断言+一行注释与 ase-m0 增量拒绝守卫，运行时面未动，冻结并集归 merge 档） | plan_id: PLAN-1119 | plan_revision: r1 | outcome: **pass（final）** | worktree_path: D:/autostack/.wt/lang-1119/auto-lang + auto-ui（两树 clean） | branch: plan-1119-dev | reviewed_commit: auto-lang c156391f3 / auto-ui 43f839f3 | base_commit: auto-lang 0d910d297 / auto-ui 9fc41dd4 | dependency_revisions: 不变（auto-web 0cfc7c7d/auto-down 456c86a 只读且未触） | spec_inputs: SD-01（c156391f3 修正版：`:443` 边界/期限请求间语义/旧条目保留范围/代码块字面清单/12+1 计数/释放界/304 双复验——逐条与实现一致）+ SD-02 §AutoWeb（67a660dc 未动） | acceptance_results: AC-01 pass、AC-02 pass（RF-2/3 闭合：304 双复验+词法洞修补）、AC-03 pass、AC-04 pass（RF-1/7 闭合：两阶段观察面+确定性总账+重开/回退/前进真实断言）、AC-05 pass（本轮 final） | pending_closeout_gates: merge 档冻结并集复跑（本仓 V-05 并集；修复轮输入为测试+文档，1118 并集证据可复用性由 merge 核）；plan-artifacts/plan-1119/review-final 登记后随收尾删除；auto-web 本地分支 auto-web 删除 | findings: RF-1..RF-7 全闭合（逐项证据锚点见收据）。NF-1（minor，随行闭合）：stop 用例中止断言原仍竞态空洞（记录延迟落账 vs 即时读取）——已按复审建议折叠修复（auto-ui `69c7b677`：+wait_for_log 落账等待，测试 only 9+/1-，stop case 3/3 稳定 2.59-2.61s；"stop 用例同型"表述成立）。观察项（不动作）：O-1 命名实体（`&colon;` 等）仍以无冒号相对形态通过（同类零执行面，v1 无点击导航+地址栏严格重解析）；O-2 304 ETag 大小写无关比较略欠严格（RFC 不透明串）——由无条件摘要复验兜底；O-3 auto-web 遗留本地分支（收尾项） | evidence: 重放：ase-m0 autoweb_readonly 21✓ 0.08s；autoscape_autoweb_readonly 13✓ 25.0s（首跑含 1m38s 构建）；健康：4 修复文件 fmt 0 hunks（mod.rs 6 hunks 为 PLAN-980/1118 期既有模块序漂移，非修复行）、零新警告；修复 diff 边界核验：无范围蔓延、auto-web/auto-down 未触；确定性核验（代码级）：close_all 先置全取消后 worker 才可能接取排队命令（navigate_inner 首检拒）→ 排队命令可证永不触网 | next: merge（T-08）

stage: goal | plan_id: PLAN-1119 | plan_revision: r1 | round: 5 | from: review | to: merge | status: reviewed | outcome: pass | worktree: D:/autostack/.wt/lang-1119/{auto-lang,auto-ui} | commit: auto-lang c156391f3 / auto-ui 43f839f3（NF-1 随行 69c7b677） | next: merge

stage: merge | plan_id: PLAN-1119 | plan_revision: r1 | outcome: pass（delivered） | completion_kind: delivered | checkpoints: prepared（reviewed r1 final pass auto-lang c156391f3 / auto-ui 43f839f3 + NF-1 test-only 随行 69c7b677〔复审代理人明示可 pre-merge 折叠，3/3 重放验证〕；SD-01/02 已在 worktree 提交并随复审落地）；landed（两仓 master 均经 rebase 后 `--ff-only`：auto-ui tip=ff9b7915、auto-lang tip=6e6e47844，rebase 映射与等价性记录于 T-08；主检出 ase-m0/Cargo.lock 他人构建漂移核为分支锁真子集后弃置）；canonical_verified（SD-01 sha256 b3937e6466868896 @6e6e47844、SD-02 sha256 10b1c60b44924176 @ff9b7915；无 ledger/索引写入）；archived（本文件移入 docs/plans/archive/、status: archived、docs 提交在默认分支）；cleaned（四树 wt-guard clean→移除→分支删除→组目录删除；plan-artifacts/plan-1119 整目录删除〔review-final 探针包证据已折叠 §9：p1119_probe.rs sha256 817b33be2e13…/probe_run1.log b06cf19513a4…〕；.retention/policy.json 刷新） | delivery_commit: auto-ui ff9b7915 / auto-lang 6e6e47844 | spec_paths: docs/specs/ase/autoweb-https-readonly.md（auto-lang）+ docs/specs/runtime/ase-m0-frame-consumption.md §AutoWeb（auto-ui） | gates: 冻结并集（rebase 后最终树）：cargo t ase-m0 PASS（27 执行）；autoscape_autoweb_readonly 13✓ 26.01s（12 CPU + 1 真窗档单实例 9.71s）；1118 并集 navigation 21✓ 13.38s / sessions 9✓ 2.65s / clock 5✓ 2.64s / 1106 4✓ 0.08s | deployment_observation: 无生产面工件（实验外壳+测试；不触发发行/重启） | next: done

stage: goal | plan_id: PLAN-1119 | plan_revision: r1 | round: 6 | from: merge | to: done | status: archived | outcome: pass | worktree: （已清理） | commit: auto-ui ff9b7915 / auto-lang 6e6e47844 | next: done

## 完成摘要

AutoWeb HTTPS 只读导航交付：ase-m0 `autoweb` 协议/缓存层（地址/别名→规范 https、v1 严格描述〔serde 重复键/未知字段拒〕、同源/重定向/降级拒、媒体/限额准入、ETag/`304`/`no-store`/`Vary` 缓存〔新鲜正文验证后才入缓存、304 身份+摘要双复验〕、只读准入扫描〔图片/HTML/`$` 指令/链接目标白名单〕）+ auto-ui 网络执行面（reqwest native-tls 专用客户端可注入测试信任、单 worker 单飞行、取消旗标+事件回投）+ 外壳接入（网络写法与 `app://` 同一 navigate 入口、后台页签完成、WebDoc 只读视图、收藏保留打开方式）。验证：ase-m0 21 case + auto-ui 13 case（真实 TLS 全链 rcgen 测试 CA↔rustls 站、真窗档）+ 1118 并集回归；两轮独立复审（7 发现全闭 + NF-1 随行）；master 预存红门禁修复（nextest AAC 过滤器 ×4 处补 ase-m0 成员）。后续能力合同见 SD-01 边界节。

**复审覆盖说明**：独立上下文复审代理×2（final + 修复复审），从冻结工件重构结论；反例探针验证 304 偏斜拒/词法洞/取消释放界；不变量巡检（缓存只存已验内容/迟到完成保新 pending/四取消路径置旗标）成立；边界记录：真窗像素未取有效截图（e2e 证据断言+视图代码为界）、仅 Windows/schannel+注入测试 CA 面、命名实体与 ETag 大小写两观察项（零执行面/摘要复验兜底）。
