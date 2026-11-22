---
plan_id: PLAN-1119
status: drafting
feature_name: AutoWeb HTTPS 只读导航原型
author: [Codex]
created_at: 2026-11-20
updated_at: 2026-11-20
plan_revision: 1
current_step: 0
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

- [ ] T-01 冻结正式外壳和 HTTPS 只读长链
  - owner_stage: work；依赖 1118 正式交付。验证公开导航、TLS 和真实文档渲染，固定来源、媒体、读取限额、只读内容边界及实际可复跑入口。V-01/04，AC-01/02。
- [ ] T-02 实现描述、来源和测试站点
  - owner_stage: work；依赖 T-01。固定 v1 格式、两个地址写法、真实协商、同源请求和受限正文准入，建立可复跑 HTTPS 测试站。V-01/03，AC-01/02。
- [ ] T-03 接入缓存与条件验证
  - owner_stage: work；依赖 T-02。按已验描述与主体身份存取，接真实 ETag/`304`、新版本及保留旧有效提交。V-02，AC-03。
- [ ] T-04 接入原生文档页签和取消
  - owner_stage: work；依赖 T-02/03。走 1118 公共入口、只读 AutoDown 视图、历史/收藏/状态与取消，完成 V-04。AC-01/04。
- [ ] T-05 固化正反控和原生验证
  - owner_stage: work；依赖 T-04。跑 V-01–05，确认服务器返回与浏览器行为、错误时机、缓存和实际释放。AC-01–04。
- [ ] T-06 冻结验收与规范候选
  - owner_stage: work；依赖 T-05。对账 SD-01/02，记录可复跑的 TLS 发布配置、实现版本、已验只读边界和待验证能力。AC-01–04。
- [ ] T-07 独立复审
  - owner_stage: review；依赖 T-06。独立核来源、真实 TLS、缓存及副作用负控，复跑关键路径。AC-05。
- [ ] T-08 正式落地
  - owner_stage: merge；依赖 T-07 pass。复核冻结职责/消费者，沉淀规范、更新路线、归档及清理。AC-06。

## 9. 复审记录

stage: new | plan_id: PLAN-1119 | plan_revision: r1 | outcome: pass（起草交接，实施前置待交） | next: PLAN-1118 正式落地后，经用户实施指令进入 T-01。拟议验证均未运行。

## 10. 待澄清事项

无起草阻塞。T-01 核 1118 最终接口、客户端的 TLS/信任配置、可发布的 AutoDown 媒体与只读封闭性。具名缺口只影响依赖它的实现，交原 owner 后沿确定的接缝复验。
