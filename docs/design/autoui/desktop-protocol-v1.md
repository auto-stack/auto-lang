# AutoUI 桌面协议 v1（Desktop Protocol v1）

> **版本**：v1（2026-08-29 定稿，Plan 386 Stage 1）。
> **施工图**：[desktop-shell](desktop-shell.md) §7/§7.1（曾用名 AutoShell/25-autoshell-dsl-unified-shell）
> （五通道表）；**单源实现**：
> `crates/auto-lang/src/ui/desktop_protocol/`（feature `ui-iced`）。
> **定位**：进程外 App（路线 B）与桌面（compositor 角色）之间的接缝协议，
> 角色对标 Wayland——桌面合成 App 表面，App 上报帧、消费输入。Stage 1 在
> 同进程 loopback 走通本协议（消息/序列化/双端状态机全部真实过线），
> Stage 2 换真 transport（命名管道/共享内存）时**只替换 loopback 层**。

## 1. 版本化

| 版本 | 日期 | 变更 | 关联 |
|---|---|---|---|
| v1.0 | 2026-08-29 | 初版：五通道消息 + 二进制编解码 + 双端状态机 + loopback 传输 + 462 会话绑定 | Plan 386 S1–S7 |
| v1.1 | 2026-08-29 | 真两进程增量：命名管道传输 / 共享内存帧缓冲 / broker + 入口裁决 / L2 detach-attach（`PROTOCOL_VERSION` 仍为 1——全部为追加式演进，见 §1 纪律） | Plan 386 S8–S12 |
| v1.2 | 2026-08-29 | 真桌面壳增量：通用 client 运行时 / broker 桌面接入 + 多 App 驻留宿主 / 多 App 压测与内存实测 / 弹性重连 / L1 换窗 / L3 v2a 快照迁移（`StateSnapshot` tag 11 追加；`PROTOCOL_VERSION` 仍为 1） | Plan 480 S1–S10 |
| v1.3 | 2026-08-31 | RenderQueue 并行渲染模式——帧载荷二态化（Commands \| Pixels）+ 三态渲染开关（auto/queue/independent）+ `AppProjector` 覆盖爬坡 + 宿主两态合成 + 三臂 parity 纪律（见 §1.3；T1 定案 D1–D4 + 497 快照结论复核） | Plan 500（Stage 4） |
| v1.4 | 2026-09-01 | Stage 6 收官：默认策略裁定（`shell.apps.process_model` 配置位；实测维持 inproc 缺省、outproc 为隔离选项，见 §1.4）+ 远程 command 流消费（WS transport 第五实现 + 镜像会话 + `HitTable` tag 9 追加 + `packages/drawlist-renderer` TS/Canvas2D + 浏览器点击闭环；`PROTOCOL_VERSION` 仍 1） | Plan 508（Stage 6） |
| v1.5 | 2026-09-01 | queue 臂保真收口——scissor 裁剪算子（`Scissor` tag 3 / `ScissorPop` tag 4 追加式，栈语义取交）+ typography 差分（`TextStyled` tag 5 追加式，weight/italic；见 §1.5；`PROTOCOL_VERSION` 仍 1） | Plan 515（§1.5） |
| v1.6 | 2026-09-15 | 编译 exe 一等客户端——`Component` seam 双投影器（解释态 `AppProjector` + native `RqProjector<C>`；a2r 产物经孵化参数接入 compositor；见 §1.6） | Plan 020（§1.6） |
| v1.7 | 2026-09-18 | native 覆盖爬坡（form/payload 族 + scroll 入册）+ 输入路由收口（见 §1.7；`PROTOCOL_VERSION` 仍 1） | PLAN-025（§1.7） |
| v1.8 | 2026-09-18 | native 覆盖爬坡第二批（display 族 image/progress + grid）+ IME 闭环两端（v1.0 在册变体启用）+ auto 裁决复测（见 §1.8；`PROTOCOL_VERSION` 仍 1） | PLAN-026（§1.8） |
| v1.9 | 2026-09-18 | 图像 DrawOp 通道——`DrawOp::Image`（tag 6 追加式，src 引用 + 宿主侧解析，零位图字节过线）+ src 词汇表（file/`builtin:`/`data:`/`http(s)`/`thumbnail://` 虚拟引用）+ 未解析降级纪律 + 两投影臂真图升级 + TS decode/占位（见 §1.9；`PROTOCOL_VERSION` 仍 1） | PLAN-028（§1.9） |
| v1.10 | 2026-09-18 | live 输入接线（§1.7 壳侧缺口清偿：`desktop_window_events` 键盘/滚轮/IME 三族臂 + `DesktopEvent::LiveInput` 泵入 + `route_live_input` 生产路由）+ native 覆盖第三批（shell 面四 kind：popover/mousearea/windowthumbnail/workspacepreview）+ `lucide:` 词汇真渲 + `workspace://` 虚拟引用 + fallback 语法 `!{icon}` + acceptance key verb（见 §1.10；`PROTOCOL_VERSION` 仍 1——全部宿主侧/词汇表演进，零新 wire tag） | PLAN-029（§1.10） |
| v1.11 | 2026-09-19 | B 程序主体（shell outproc client）：投影下行推（`ShellProjectionPush` tag 12 / `ShellClockTick` tag 13 / `ShellCursorMove` tag 14——typed 载体 wire 编码 + per-face 宿主侧指纹门）+ 命令上行执行（`DesktopBus` 端点拆臂 + registry_id 归因 + `desktop_bus_inbox` 泵后同拍执行）+ 表面 z 平面声明（Hello/Welcome 尾段多表面协商：background/chrome）+ 壳看门兵（死亡检出 → 退避 respawn → 全量重推）+ pointer 生产接线（press/release 命中 broker wid 路由）+ `shell.apps.shell_model` 双轨开关（缺省 inproc；见 §1.11；`PROTOCOL_VERSION` 仍 1——ControlMsg/Handshake 追加式，空尾段字节级不变） | PLAN-030（§1.11） |
| v1.12 | 2026-09-19 | rqhost 第四运行形态——rendezvous 采纳协议（well-known 管道 + `adopt␟<name>` 记录 + 锁管道单实例仲裁）+ 客户端权威采纳（宿主零装载）+ rqhost 生命周期语义（末窗退出/app EOF 回收/宿主死 exit-on-EOF 策略档）+ 大帧 shm 超槽回退管道内联（见 §1.12；rendezvous 记录 = 传输层管道串约定，零 codec 变体，`PROTOCOL_VERSION` 仍 1） | PLAN-031（§1.12） |
| v1.13 | 2026-09-19 | native 覆盖 ramp v3 + **缺省翻转**——六缺项五族补齐（tabs 整 kind 全链[a2r 断裂修复]/hidden display:none[display 族响应式覆盖]/样式版 grid[GridCols→Grid walker 分岔]/定位族分层[absolute+offset 覆盖序真渲 + fixed/sticky 降级放行随注]/012 SelfCenter 映射臂 + D5 族运行时面[Inset/LineClamp/FlexWrap]）+ 复测 judged 22/22 = 100% ≥ 95% 过门 → `resolve_native_frame_mode` Covered 臂翻 Commands（auto 缺省 queue）+ 仪器 judged 口径升级与防漏断言反转（见 §1.13；零 wire 变体，`PROTOCOL_VERSION` 仍 1） | PLAN-032（§1.13） |
| v1.14 | 2026-09-19 | **单投影器统一（rq-projector-unify）**——`-q` VM 轨改接 RqProjector（View 全展开投影 + 启动覆盖门，a2r/解释同律）+ VM 轨三补（input_state_map 绑定字段回写[on() 单写点，a2r 生成物同构]/多 timer 泵侧驱动[Component::fire_due_timers 钩子]/`__desktop_cmd` 读走上行[DesktopBus 泛化]）+ `AppProjector` 退役（解释 re-exec 臂/pixels 臂/process_model=outproc 选项拔除——解释态两合法形态 = inproc 直挂 / `-q` 经 native 臂）+ `NativeProjector` 更名 `RqProjector`（见 §1.14；客户端臂内部演进零 wire 变体，`PROTOCOL_VERSION` 仍 1） | PLAN-033（§1.14） |

- 版本常量：`desktop_protocol::PROTOCOL_VERSION = 1`，随每条消息信封头过线。
- **协商规则**：Hello 携带版本；宿主校验不符 → `ProtocolError::VersionMismatch`
  拒收孵化；信封层版本不符 → `CodecError::UnsupportedVersion` 拒收。
- **演进纪律**：只许追加（新变体 tag、新通道号），不许改义/重排既有 tag；
  载荷种类（`DrawList` 的 kind tag）与通道各自独立编号。

## 1.1 v1.1 增量（Stage 2 两进程落地）

| 增量 | 内容 | 落点 |
|---|---|---|
| 传输层 | `Transport` trait（send/try_recv/pending/is_eof/recv_wait）；loopback 收编为实现之一；Windows 命名管道（tokio named_pipe + split 读写半 + select!{read,shutdown} 常驻 runtime）| `transport.rs` |
| 共享内存帧 | `SharedFrameBuffer`（CreateFileMappingW 双槽，`[u32 len][payload]`）+ `FrameMsg::FrameReadyShared{wid,frame_id,slot,damage,revision,len}`（tag 7 追加）+ `BufferAlloc.shm: Option<String>`（尾部追加，约定名 `autodesk-shm-<surface>`）——大帧走 shm、管道只过元数据 | `shm.rs` / `message.rs` |
| broker | `adjudicate()` 入口裁决三步（①`--autodesk-client=<pipe>` ②探测 broker ping ③Standalone）+ `Broker::serve_once`/`request_incubation`（DesktopBus 同形 `incubate` 记录，per-app 管道名分配 + 先行 listen + 转连；ping 吞弃） | `broker.rs` |
| L2 迁移 | `ControlMsg::L2Detach/L2Detached/L2AttachRequest`（tag 8/9/10 追加）+ App 态 `Standalone`：Active→L2Detach→Standalone→L2Detached→宿主回收→connect()（同入口）→Active；**revision 不归零 = 状态未动** | `endpoint.rs` |
| 两进程验证 | re-exec 集成测试：spawn 子进程（双模 ① 路径）→ 孵化 → 共享内存帧随点击递增 → L2 Standalone → 子进程 stdout 状态标记（`count=3 rev=4`）= app 进程持有状态的跨进程证据 | `dual_mode.rs` |

## 1.2 v1.2 增量（Stage 3 真桌面壳落地，Plan 480）

| 增量 | 内容 | 落点 |
|---|---|---|
| client 运行时 | `AppProjector`（AuraNode view → DrawList 最小投影器 v1：text/button + 线性堆叠、button 命中区推导、prop text/label + FStr 插值代入 VM 状态；带参 handler 不投影）+ `ClientPump`（可步进协议泵：握手 → Active → 输入 → handler → shm 产帧 → L2 处理；`run()` 阻塞主循环供真实 child 进程；动态组件持 Rc 非 Send，不跨线程） | `client_runtime.rs` |
| broker 桌面接入 | `DesktopSession::enable_broker`（serve 线程受理孵化，端点搬运排队；`ProtocolHost` 持 `&mut session` 不可跨线程）+ `attach_pending_incubations`/`pump_broker_clients`（属主线程落地/泵帧；desktop boot 开 broker、ServiceTick 帧泵周期消费）；`auto run` 双模入口 `--autodesk-client=<pipe>` / `--autodesk-incubate` / `--app386=<name>`（③ 无标记现行行为零改动） | `session.rs` / `renderer.rs` / `crates/auto/src/cmd_autodesk.rs` |
| 多 App 驻留宿主 | `stage3::BrokerClient`（per-app 端点+表面+shm 驻留 `broker_clients` 表）——v1.1 单 client 的"多 App 并发归 Stage 3"兑现；`broker_pointer_down` 按 WM 命中窗归属路由输入 | `stage3.rs` / `session.rs` |
| 压测与内存 | N=3/5 child 全 Active → 逐 App 点击帧递增 → 30s 稳定存活（re-exec 真协议主循环）；内存采样 `K32GetProcessMemoryInfo`（零新依赖）+ N=1/3/5 边际增量报告（Private 口径 4.8MiB/App 临界达标，WS 口径 23.2MiB/App 未达标——度量+判定形态） | `stage3.rs` / `docs/plans/reports/480-memory-baseline.md` |
| 弹性重连 | child EOF → `ReconnectPolicy` 预算内重试连回同一 per-app 管道 → 同一 projector 重建端点续跑（VM 状态/revision 原地） | `client_runtime.rs` |
| L1 换窗 | `DesktopSession::detach_surface_to_os_window` / `attach_surface_back`：虚拟窗 ↔ 独立 OS 窗登记翻转（`wm_remove_win` ↔ `iced::window::open`+`register_window`），App/VM 对象原地 | `session.rs` |
| L3 v2a 快照 | `ControlMsg::StateSnapshot{wid,payload}`（tag 11 追加，host→app）：载荷 = revision + 原始状态字段（Int/Double/Bool/Str；复合类型 Nil 占位）；child `on_control` 逐字段写回 + revision 续接，应用后产帧同步宿主 | `message.rs` / `client_runtime.rs` / `stage3.rs` |
| 缺陷修复 | shm 段名 `autodesk-shm-<surface>` 全局撞名（Windows `CreateFileMappingW` 同名 = 打开既有段）→ 加 pid 前缀——压测暴露的真多宿主缺陷 | `host.rs` / `session.rs` |

## 1.3 v1.3 增量（Stage 4 RenderQueue 并行渲染模式，Plan 500；T1 定案 2026-08-31）

> **动机**：让"app 自带 iced/wgpu 独立渲染"与"app 免 GPU 上下文、宿主侧
> 栅格化（RenderQueue）"两条路径**在同一桌面内并存**，per-App 启动时选择。
> 现状：帧通道 v1.0 起即为 commands 载荷（`DrawList`），但 `AppProjector`
> 覆盖仅 text/button + 线性堆叠（demo 级）；attach 态 app **没有**自渲染的
> 像素帧路径；宿主对 broker 表面的消费停留在 `SurfaceStore` 存取（消息级
> 断言），**未落像素合成**。两条既有渲染路径（进程内 iced 直挂 /
> Standalone 独立窗）不动——I1 零删除不变式延续。

| 增量 | 内容 | 落点 |
|---|---|---|
| 帧载荷二态 | `FrameMsg::FrameReadyPixels{wid,frame_id,slot,damage,revision,w,h,stride,format}`（tag 8 追加）——像素帧元数据过管道、RGBA 在 shm 槽；`HandshakeMsg::Welcome` 尾部追加 `frame_mode: Commands\|Pixels`（旧端缺省 = Commands，向后兼容）。shm 槽载荷解释随 Welcome 协商的模式位而定（Commands = `[u32 len][DrawList 编码]` 既有格式；Pixels = 定长 `h×stride` RGBA 行序列） | `message.rs` / `shm.rs` |
| 三态渲染开关 | per-App：`desktop_render: auto \| queue \| independent`——pac.at manifest 字段（Plan 276 既有前端后端 `render:` 字段撞名，语义正交不可复用，执行定案改名）+ spawn 参数 `--autodesk-render=<mode>` 覆盖（CLI `run` 具名 `--render` 撞名）+ 裁决链（spawn 参数 > manifest > auto；进程形态 `adjudicate()` 链正交）。`auto` = 装载期覆盖度探测，可行走 queue、不可行降级 independent（宿主记观测 `Log` 一行） | `cmd_autodesk.rs` / `coverage.rs` / `auto-man pac.rs` |
| 覆盖度探测 | `AppProjector` 能力表 `Coverage{kinds, props, layouts}` vs App 视图清单（装载期静态扫描）→ 可行/不可行判定；未覆盖项显式 not-yet，**禁止静默错绘** | `client_runtime.rs` |
| Pixels 路径（independent 臂） | child 自带 iced 运行时 + **隐藏窗**（app 尺寸）→ 状态变更/输入后重渲染 → `iced::window::screenshot` 整窗抓取（497 T1 已验证的唯一公开栅格化通道；物理像素 ×scale_factor）→ RGBA 写 shm 槽 → `FrameReadyPixels`。协议泵从 iced update 周期驱动（消息经 reader 线程转交主线程，`DynamicComponent` 持 Rc 不跨线程） | `client_runtime.rs`（`dual_mode` 同型扩展） |
| 投影器爬坡 | `AppProjector` 从 text/button 爬到 §1.3.1 清单（001–005 实测集合）；布局 = projector 自带轻量块/行流，**参数源复用 `ui/style::BoxLayout`**（tailwind 类 → padding/margin/gap/尺寸提取，vue 臂同词汇） | `client_runtime.rs` / `ui/style/` |
| 宿主两态合成 | queue 臂：DrawList → canvas Program 降级（Quad=fill 抗锯齿、Text=fill_text 宿主侧 shaping）挂虚拟窗内容；independent 臂：shm RGBA → `image::Handle::from_rgba` 上传（497 快照同通道）→ Image 挂虚拟窗。`damage` v1.3 作重绘提示（全帧重画，正确性不受损） | `stage3.rs` / 宿主渲染段 |
| 三臂 parity | iced 直挂 / vue / queue 三臂同源金样（I4 扩展为 I4'：`window_thumbnail` 同族纪律）——queue 臂缺失 lowering 的 widget 在覆盖表中标 not-yet，**禁止静默错绘** | 金样体系 |

### 1.3.1 爬坡目标集（001–005 实测清单，T1 扫描定案）

| 示例 | widget kind | 关键 prop/事件 | 布局形态 |
|---|---|---|---|
| 001-helloworld | text | style(text-4xl/font-bold/text-primary)、selectable | center |
| 002-counter | text（FStr 插值）、button | onclick 内联 lambda | center、row |
| 003-converter | text（style）、input | value 绑定、oninput 内联、placeholder | center、col(gap/flex-1/p/bg/border/rounded/shadow/max-w/mx)、row(gap) |
| 004-profile-card | text（selectable）、image、button | image src 绑定 + style、button style | center、col(渐变 bg/-mt/items-center/gap)、row(gap/items-center) |
| 005-login | h2、text、input、button、a | input type:password、onclick/oninput msg 路径、`if` 条件块 | center、col、row(justify-center/items-center) |

爬坡目标集（`Coverage` 基线）：kinds = text（含 h1–h6/p/span/label 变体）、button、input、image、a；props = text{text/style/selectable/插值}、button{label/style/onclick 零参}、input{value/oninput 零参/placeholder/type}、image{src/style}、a{text/style}；layouts = center、col、row + BoxLayout 参数子集（padding/gap/margin/固定尺寸/圆角矩形底/对齐）+ `if` 条件块。样式**语义子集化**：style 串按 `ui/style` 解析取布局/底色/前景子集，无法解析的类不静默丢弃——整 widget 记 not-yet。

### 1.3.2 深水决策定案（T1，2026-08-31）

| # | 决策 | **定案** | 依据 |
|---|---|---|---|
| D1 | `Text` op 形态 | **A：字符串+样式，宿主侧 shaping**（草案倾向 B 被推翻） | ①`DrawOp::Text` 线格式 v1.0 冻结即"shaping 留宿主"（413 §7 约束同款）——B 需新增 glyph run op + 字体图集协议，违背追加式演进的经济性；②宿主（ui-iced）已带 iced 文本栈（canvas `fill_text`），零新依赖；③B 要把 shaping 引擎塞进 queue 臂 child，与"app 免 GPU/轻 child"初衷相悖；④命中坐标同源性：命中判定用投影器 widget 级矩形（非 glyph 级），与宿主绘制的偏差不构成命中错误 |
| D2 | 布局引擎归属 | **projector 自带轻量块/行流布局，参数源复用 `ui/style::BoxLayout`**（草案"复用 `ui/layout`"前提有误） | ①`ui/layout` 实为桌面**窗位**引擎（Free/Grid/MasterStack，R9），非 widget 布局——不可复用；②iced 离屏布局要求 queue 臂 child 引 iced runtime，违背独立性；③`ui/style/layout_extract.rs` 的 `BoxLayout`（tailwind 类 → padding/margin/gap/width/height/max_w 像素提取）正是两渲染臂的共同词汇，零成本复用 |
| D3 | 输入命中细化 | **widget 交互区表上报**：投影器布局期顺产 `Vec<(WRect, kind, action)>`（button→onclick token；input→聚焦目标 + 文本插入路由；a→link） | 480 的 button 命中区推导泛化；input 臂 = 命中聚焦 + KeyPressed/CharTyped 按聚焦目标写回 value 绑定（001–005 的 `if` 错误提示随 revision 重渲染） |
| D4 | IME/弹层坐标下发 | **形态定案、v1.3 不落线**：`ControlMsg::ImeCursor{wid,rect}`（tag 12 追加）为 queue 臂 IME 闭环的既定形态；001–005 验收口径 = CharTyped 闭环（非 IME），IME 光标下发 + popover 锚点登记 not-yet（Stage 5 随 input 臂 IME 交付） | 无消费面不占协议号位——v1.3 落线的只有 T1–T4 用到的变体（`FrameReadyPixels`/`Welcome.frame_mode`） |

**pixels 格式定案**：v1 仅 RGBA8 **straight（非预乘）**、`stride = w×4`、`format` 字段仅 1=RGBA8（扩展位）。宿主经 `image::Handle::from_rgba` 上传（497 快照同通道口径："预乘与否同截图原样——`from_rgba` 直接受纳"）；预乘换算在 iced 渲染器内部与 494 预乘 alpha swapchain 衔接，协议层不感知。

**分期**：Stage 4（本增量：二态载荷+开关+001–005 子集端到端+parity 首条）
→ Stage 5（覆盖爬坡全 widget 族 + IME 光标下发，parity 门禁进日常档；✅ 已落地
69/388 Tier1+2，Plan 507）→ Stage 6（默认策略 + web/远程端消费同一 command 流；
✅ 已落地，见 §1.4，Plan 508）。

## 1.4 v1.4 增量（Stage 6 默认策略裁定 + 远程 command 流，Plan 508）

- **进程模型配置位**：storage `shell.apps.process_model`（缺省 `inproc`
  = 现状零变化；`outproc` = `launch_app` 走 broker 孵化链——spawn 真
  `auto run --autodesk-incubate` 子进程 + 同步泵 attach + registry_id
  回填；resolver 目录名兜底保外部根两形态同源）。
- **默认策略裁定**（G2 实测，报告
  `docs/plans/reports/508-process-model-verdict.md`）：**维持 inproc
  缺省，outproc 为显式隔离选项**——系统总边际 0.86 vs 7.64 MiB/App
  （真 auto.exe 生产口径 6.48）、启动 2–17ms vs 25–250ms、交互引擎核心
  0.07ms vs 端到端 1.54ms（差值 ≈1.47ms 协议税，绝对值在流畅带内不构成
  否决）；翻转三闸 T-覆盖（507 合入后复测）/T-稳定性/T-远程。
- **WS transport**（transport.rs `ws` 模块）：tokio-tungstenite（本线
  唯一新三方依赖——手写 WS 帧协议得不偿失，计划待澄清①）；**一条 WS
  Binary 消息 = 一个 codec 信封**（消息边界原生，无二次分帧——信封
  不变纪律）；token = HTTP 升级期 query 校验（浏览器 WebSocket API 不
  可设自定义头），401 终态拒收。
- **远程会话接纳**（`remote.rs` 镜像泵）：宿主 WS 监听 `127.0.0.1:17800`
  （storage `shell.remote.token` 有值才监听——缺省拒绝 = 零远程面）；
  远程端 Hello 订阅 → Welcome + `HitTable`（FrameMsg tag 9 追加式变体
  ——D3 交互区表的过线形态，宿主孪生投影器同源布局）+ composed DrawList
  变化推送（FrameReady payload 内联，v1.0 纯管道帧形态）；输入
  InputMsg 坐标直传路由（与 `broker_pointer_down` 同一收尾，权威命中
  在 app 侧）。镜像 ≠ 端点状态机——pipe/loopback 会话零改动。
- **web 渲染器**：`packages/drawlist-renderer/`（TS/Canvas2D：信封/消息
  codec 镜像 + `renderFrame`（Quad=fillRect、Text=fillText——D1-A 定案
  web 同义实现，零 glyph 协议工作）+ `hitTest` + `connect`（重连
  ReconnectPolicy 对齐））；codec 对拍 = Rust
  `p508_ts_crosscheck_golden_bytes` ↔ TS `fixtures.golden.ts` 双侧钉
  同一批字节（计划②：手工镜像 + golden 兜底）。demo 页
  `examples/remote/viewer/`（vite）为验收载体。
- **vue 桌面远程窗（Plan 516）**：渲染器包经 `auto-man` wm_assets 编译期
  拷贝进 vue 虚拟桌面宿主（`src/wm/remote-renderer/`），远程 App 作为
  `kind:"remote"` 虚拟窗进 WM。**尺寸协商注记（v1）**：远程会话的
  `Welcome.rect`（远端逻辑尺寸）为 canvas 位图源空间，vue 虚拟窗 rect
  只决定 CSS 呈现——**v1 以 vue 侧为源不回传 resize**（远端不跟随 vue
  窗缩放；Hello 的 wh 仅建连初值）。后续若要远端跟随（512 fit 动态重测
  的远端联动语义），需 Hello 重订阅/尺寸上行变体——协议号位预留，未占。

**RenderQueue 线终态小结**：六阶段收线——协议五通道（386 S1）→ 真两
进程（386 S8+）→ 真桌面壳多 App 驻留（480）→ 并行渲染二态载荷（500）
→ 全 widget 覆盖爬坡 + parity 日常门禁（507）→ 默认策略裁定 + 远程
command 流消费（508）。终态形态：**桌面默认进程内直挂（实测裁定），
RenderQueue 为统一帧词汇**——queue 臂 DrawList（Stage 5 覆盖集）同时
喂宿主虚拟窗与远程镜像端（WS/浏览器），outproc 为隔离/远程形态的
App 载体选项；协议 `PROTOCOL_VERSION` 全程为 1（追加式演进出口纪律
验证有效）。

## 1.5 v1.5 增量（queue 臂保真收口——scissor 裁剪算子 + typography 差分，Plan 515）

- **scissor 裁剪算子**（G1）：`DrawOp::Scissor{rect}`（tag 3 追加式）=
  压栈裁剪矩形（宿主坐标空间，与当前有效裁剪**取交集**）；`DrawOp::ScissorPop`
  （tag 4）= 出栈。投影器 scrollable 臂在内容溢出时产 push/pop 对——
  溢出内容宿主侧正确裁剪（507-1 收口）。**栈深度定案**（计划待澄清①）：
  线格式不设上限（ops 序列天然任意深），v1 投影器产出 ≤2 层（嵌套
  scrollable 实测两层层级够用），消费端（宿主栅格化 / TS 渲染器）实现
  为无界栈、须支持 ≥2 层。配对纪律：编码端保证 push/pop 平衡；栅格端
  宽容——空栈 pop = no-op、未闭合 push 裁到序列尾；解码端逐 op 无状态。
- **typography 差分通道**（G2）：`DrawOp::TextStyled{x,y,size,line_height,
  color,weight,italic,text}`（tag 5 追加式）——`weight` = CSS 字重刻度
  u16（400 = normal / 700 = bold），`italic` = bool。既有 `Text`（tag 2）
  线格式冻结不动；投影器在 weight = 400 且非斜体时仍产 `Text`（旧端
  远程消费面零破坏），bold/italic 档产 `TextStyled`。宿主 `fill_text` /
  TS `fillText` 按 weight/italic 产视觉差分（507-1 typography 收口）。

## 1.6 v1.6 增量（编译 exe 一等客户端——Component seam 双投影器，Plan 020）

- **客户端臂泛型化**：协议客户端入口自解释态 `DynamicComponent` 泛化到
  `Component` seam——a2r 编译产物（`auto build -r rust` 的独立 exe）经
  孵化参数（`--autodesk-incubate` / `--autodesk-client=` /
  `--autodesk-render=`）作为一等客户端接入 compositor。双投影器并存：
  `AppProjector`（解释态 AuraNode，零改动）+ `RqProjector<C>`（新，
  `View<C::Msg>` 运行期投影——策略 B：prop/handler/条件/插值在 view()
  构建期已物化，投影器即 `View` 的协议后端；命中表
  `Vec<(WRect, C::Msg)>` 物化消息 clone 派发）。`FrameSource` 追加缺省
  空方法 `poll_tick`（泵空拍对账；native 侧 = `tick_interval_ms` 到期
  派发 `tick_msg`）。
- **native 覆盖集与缺省臂**：`Coverage::native_queue_set()` = text/
  button + col/row/container/list + 布局样式子集（与解释态 target_set
  分表——native 渲染面更窄，爬坡随投影器扩臂同步扩表）。三态裁决：
  显式 `queue` 遇 not-yet = **拒绝退出留痕**（缺项清单即载荷，禁静默
  错绘）；**native `auto` 缺省 = independent**（queue 覆盖爬坡前的安全
  缺省，降级观测行留痕——与解释态 auto 语义并列，本节裁定入册）；
  `independent` 直通（隐藏窗自渲 + screenshot，v1.3 臂复用）。pac
  `desktop_render:` 声明由宿主 spawn 侧透传 `--autodesk-render=`（生成物
  无 pac 位置感知）。
- **宿主孵化分流**：注册表 pac `desktop_exe:` 声明（相对 App 根）>
  rust-workspace 约定路径（`<root>/rust-workspace/<dir>/target/
  {release,debug}/<exe>.exe`，exe 名先 pac `name:` 蛇形后目录名）>
  缺席 = 现行 `auto` re-exec 解释臂（零变化）。路由触发：全局
  process_model = outproc **或**发现命中编译 exe（"exe App 天然 outproc"
  ——inproc 缺省下同样走孵化链，纯解释 spec 不受影响）。native spawn
  参数面与解释态同形（无 `run` 子命令、不注入 `AUTO_386_APP_ROOT`）；
  stdio 静默。`PROTOCOL_VERSION` 维持 1（本节零 wire 变体——纯客户端臂泛化）。
- **v1.6 已知边界**（随注非静默）：L3 `StateSnapshot` 注入 native
  not-yet（typed 组件无字段写回路径）；键盘/滚轮/右键不路由（覆盖集无
  input 族）；门后动态分支遭遇未覆盖变体 = 占位盒 + `uncovered_seen`
  留痕。度量：queue 臂边际 ≈2.42 MiB/App（508 解释 outproc 6.48 的
  ≈2.7×），见 `docs/plans/reports/020-rust-exe-compositor-metrics.md`。

## 1.7 v1.7 增量（native 覆盖爬坡 + 输入路由收口，PLAN-025）

- **投影器 form/payload 族入册**（`Coverage::native_queue_set` 扩容，
  `PROTOCOL_VERSION` 维持 1——零 wire 变体）：kind 扩 input/textarea/
  checkbox/radio/slider/select，layouts 扩 scroll；样式子集扩 flex-1/
  shadow（**降级放行**——解释态 target_set 同款保真边界：shadow 渲染
  no-op、flex 族自然宽，非静默扩权）。**switch 无 native 对象**：View
  枚举无 Switch 变体（解释态 aura 标签专属），native 覆盖集不列（I4
  分表，非缺口）。防漏钉：native 覆盖矩阵 × 投影器臂双向钉测试
  （`native_coverage_matrix_pinned_to_projector`）+ 003-converter 形态
  native queue 金样（`test/parity/native/003-converter.expected.txt`，
  与解释态金样 `drawlist_to_text` 同构可对读）。
- **聚焦与编辑闭环**（T-01 D1/D2 定案）：native View 全物化无字段身份
  → 聚焦 = **Input/Textarea 槽位序**（命中表登记序 = 确定性树序），帧
  后重定位（槽位越界 = 结构变化 → 失焦，不猜测对位）；编辑 = 投影器侧
  buffer（聚焦时自视图值初始化，聚焦期显示 buffer——iced 内部 buffer
  同语义）→ `store_input_text` 同线程代写 INPUT_TEXT thread-local →
  a2r 生成 `on()` 经 `last_input_text()` 读面按字段类型 parse 回写 →
  on_change 物化派发。零生成器改动。**on_submit/Enter 不派发**（解释态
  臂同边界 not-yet）。多行 = 按 '
' 分行、无自动换行（解释态同边界）。
- **select 开合语义**（D3）：闭态 = 值盒 + ▾ + 点击开；开态 = 主块渲染
  后**追加覆盖序选项列**（DrawList paint order 天然置顶，无 overlay 协议
  语义）+ **命中互斥**（开态仅选项项：命中 → `SelectCallback` 物化派发
  + 关闭；外点 → 仅关闭吞掉不下穿；Esc 关闭）。选项消息布局期经回调
  物化（命中表存 `(rect, M)` 零新类型）；键盘跳项 not-yet。
- **输入路由两端**（P020-D2 键/滚轮/右键半句核销）：
  - 投影器消费：`PointerPressed{Right}` → 右键命中表（Button/Row/Column/
    Container `on_right_click` 整框登记，禁用态不登记）；`Scroll` →
    scrollable 滚轮派发（内层命中倒序胜 + 唯一 Scrollable 兜底——wire
    Scroll 无坐标，投影器以 PointerMoved/Pressed 跟踪最近指针位定位）；
    `offset' = clamp(快照+delta)` 组装 `ScrollMetrics`（镜像 iced
    `absolute_offset()` = 滚动后偏移语义）；on_scroll 不在场 = 滚轮不
    路由留痕。scrollable 溢出 = Scissor push/pop + 视口外命中区过滤
    （解释态 layout_scroll 镜像）；滚动偏移 = app 状态经 on_scroll 重入
    view()（投影器只裁剪不缓存——与解释态"偏移 v1 不载"的口径差即此：
    native 全交 app）。
  - 宿主生产：`DesktopSession::broker_key_event/broker_char`（WM **焦点
    窗**路由）+ `broker_scroll`（指针命中窗 hover 路由）——镜像
    broker_pointer_down 收尾；解释态臂 CharTyped/退格消费同册受益实证
    （t3_examples_queue_end_to_end 003 键入改走 broker_char）。
  - **live 壳接线缺口（新债随注）**：现桌面 iced 壳无键盘/滚轮事件订阅
    通道（调查证据：session.rs 零键盘事件臂），live 接线需
    DesktopMessage 扩展另立；本节生产路径 = 协议/broker 层（真机链路
    经 p025_native_input_arm 协议级承载——P020-D4 GUI 自动化债未清的
    既有口径）。
- **slider 语义**：点击定位（PointerPressed 一次派发，f32 = min +
  clamp((x-x0)/w)×range，step 取整 + 端点钳制）→ fn 指针物化派发（零
  thread-local）；**按住拖拽连续派发 not-yet**（分型命中表无按住态机）。
  a2r codegen 补臂：slider（`View::slider(min..=max, value, 变体构造
  fn 指针)`，f32 载荷变体；f64 字段 as f32 补码）+ select
  （`View::select(options)+.selected(i)+.on_choose(物化闭包)`，闭包形态
  随变体载荷数自适应）；input value 绑定形状容差（Ident '.' 前缀 /
  Dot('.'|'self',f)——对齐 `binding_field`）；`math.*` 内建 handler 臂
  降级（round/floor/ceil/abs/sqrt → f64 方法）。
- **v1.7 已知边界**（随注非静默）：IME（ImePreedit/Commit）not-yet 维持；
  grid 布局 native 臂 not-yet；L3 StateSnapshot 注入 native not-yet 维持
  （P020-D2 后半句另立）；live iced 壳键盘/滚轮订阅接线缺口（上述新债）；
  slider 拖拽、select 键盘跳项、input on_submit not-yet。

## 1.8 v1.8 增量（native 覆盖爬坡第二批 + IME 闭环 + auto 裁决复测，PLAN-026）

- **display 族 native 臂入册**（`Coverage::native_queue_set` 扩容，
  `PROTOCOL_VERSION` 维持 1——零 wire 变体）：kind 扩 **image/progress**
  （`View::Image`/`View::ProgressBar` 变体臂——占位保真：image = 样式
  尺寸占位 Quad（缺省 min(avail,96) 方形，IMAGE_PLACEHOLDER 底色），
  progress = 轨道 + fill=w×frac 比例几何，style.bg 覆盖权同解释态；
  on_seek 点击定位 not-yet）；layouts 扩 **grid**（`View::Grid` walker
  ——cols 等宽格 × row-major，行高 = 行内最大，bg 两遍法置子级之下，
  gap = Grid.gap 字段优先 + style gap- 类回退）。**icon/badge/avatar/
  divider/separator/spacer/a/img 不入册——分表非缺口**（025"switch 无
  View 变体"口径）：a2r codegen 降级归一（§下 codegen 条），View 层
  产出 kind 全在册（image/text/row/container/empty）。**imagesurface
  整 kind not-yet**（交互回调无采集面，登记即静默放行——I3）；渲染
  顺带 = catch-all 占位盒（门后动态防线臂）。**center 不入册**：
  `View::center` 归一 Container（scan 无 "center" kind 产出点——防漏
  钉钉②口径）；native 容器臂补 center_x/center_y 消费（水平两遍法 +
  fixed_h/legacy_height 外框垂直居中）+ legacy width/height 兜底 +
  **w-full（Width(Full)）满宽承载**（iced 消费面同语义——divider/
  spacer 降级形态宽度）。
- **IME 闭环两端**（D2 定案：preedit 尾拼 + 协议级注入口径）：wire
  `ImePreedit/ImeCommit/ImeCancelled` v1.0 在册变体启用——投影器消费
  （Commit = 聚焦 buffer 追加 → INPUT_TEXT 代写 → on_change 派发 →
  rev 前进；Preedit = 暂存 + 聚焦框渲染尾拼（单行独立差分色 op——真
  下划线无 DrawOp 通道随注；多行并入末行）；Cancelled/Esc 消解；无聚焦
  丢弃 + `ime_dropped()` 留痕观测面）；宿主生产 = `DesktopSession::
  broker_ime_commit/preedit/cancelled`（025 broker_char 焦点窗路由同型
  ——live iced 壳订阅缺口并入 P025-D1 在册债；协议级注入为证据承载，
  p026_native_display_arm 第三腿实证 broker_ime_commit("100") → 换算
  联动帧 212）。
- **a2r codegen display 族降级臂**（§5.1 D1/D1' 定案——对齐 VM 轨
  AuraViewBuilder 既有降级形态，零 View 变体）：icon → `View::image_
  styled("lucide:{name}")`（PLAN-018 三前缀透传 + 尺寸契约：显式 w-/h-
  类 > size prop 精确 px（w-[Npx] 任意值通道）> 默认 20px——VM
  with_icon_size 两端一致契约）；badge → 样式 Row（shadcn 基类 +
  variant 预设 + label 必达）；card/divider/separator/spacer/avatar →
  styled container（VM convert_* 同型底档；direction/orientation 竖档）；
  scroll → `View::scrollable`；link/a → styled text（AuraNode::Link 有
  子件时组合子件——原实现静默丢子件内容）；image src 绑定形状容差
  （Ident '.' 前缀 / Dot('.'|'self',f)——004 真源 src 曾静默丢失）；
  image 样式尺寸 native 臂消费（Tailwind 刻度）。断裂映射移除：
  badge/card/icon/scroll/link（降级臂先行，未达臂落 col 兜底）；
  **modal/tooltip/spinner/option/toggle/radiogroup/tab 映射断裂维持**——
  显式拒绝策略归 shell a2r 设计 S1（KNOWN-DEBT 随注）。
- **auto 裁决复测（508 三闸 T-覆盖数据行）**：examples/ui 全量 →
  AuraViewBuilder（VM 轨运行时 aura→View 构造器）→ scan_native_view ×
  judge(native_queue_set)——**overall 16/35 = 45.7% / judged 16/21 =
  76.2%，均未达 ≥95% 阈值 → 裁定维持 native `auto` = independent
  （不翻）**；缺项面全在册 not-yet（opacity/hidden/样式版 grid/popover/
  定位族/rotate/truncate 等——报告
  `docs/plans/reports/p026-native-flip-data-row.md`）。**翻转点已备**：
  `resolve_native_frame_mode` 升级扫描制观测（Auto → 真扫描，观测行
  携带逐 App 缺项清单 + queue-covered 命名，裁决仍 Auto→Pixels）；达标
  时 Covered 臂改返 Commands 即翻转（one-line，随 ramp v3 复评）。
  样式子集降级放行批（解释态 target_set 同款保真边界，逐类随注
  `native_queue_set`）：overflow-（含 x/y 轴族）/min-w-/min-h-/leading-
  （PLAN-527 typed LineHeight 可达）/flex/block/underline 族/cursor-/
  outline-/transition/antialiased/shrink-/whitespace-/relative/tracking-
  /backdrop-（518 G8 冻结词汇）+ 字重族补全。
- **v1.8 已知边界**（随注非静默）：位图真渲/图像 DrawOp 算子 not-yet
  （image/icon/avatar 占位保真口径——图像通道归 shell a2r 设计 §4-B
  独立线；**PLAN-028 图像半句已由 v1.9 核销——见 §1.9；icon/avatar
  中字形/头像语义占位半句维持，未解析降级转兜底**）；imagesurface 交互族 not-yet（D5）；live iced 壳
  键盘/滚轮/IME 事件订阅缺口（P025-D1 债扩展）；opacity/hidden/定位族/
  样式版 grid/样式版 grid-col 数 native 渲染 not-yet（覆盖表显式缺项，
  auto 降级留痕）；slider 拖拽/select 键盘跳项/on_submit/L3 快照注入
  not-yet 维持（025 在册）；modal/tooltip/spinner a2r 映射断裂（S1）。

## 1.9 v1.9 增量（图像 DrawOp 通道——src 引用 + 宿主侧解析，PLAN-028）

> **动机**：shell a2r 化裁定 B 形态（outproc shell 经 RenderQueue 渲染，
> desktop-shell-a2r.md 裁定落定）——壁纸/窗口缩略图/壁纸预览全是图像，
> DrawList 仅有 Quad/Text/TextStyled/Scissor 五算子是 B 硬阻断（§1.8
> 已知边界"图像 DrawOp 算子 not-yet"即此债）。核心路线 = **src 引用 +
> 宿主侧解析**：零位图字节过线、child 免解码（"轻 child"哲学延续）、
> 复用宿主既有图像词汇与缓存（`load_image_bytes` / snapshot 基建），
> 零新依赖。

- **wire（tag 6 追加式）**：`DrawOp::Image { rect: WRect, src: String,
  fit: ImageFit }`。`ImageFit` v1 仅 `Stretch = 1`（拉伸至 rect——与
  占位尺寸盒同位；Cover/Contain = fit 精化另立），未知 fit 值
  `UnknownTag` 拒收（FrameMode/PixelFormat 同纪律）。`PROTOCOL_VERSION`
  维持 1；tag 1–5 语义冻结；旧端（不识 6）= 拒收会话（既有 unknown
  tag 纪律，无静默漂移）。**op 字段定长不可尾部追加**（DrawOp 逐 op
  解码共享 Reader，无载荷尾判据——v1.3 `Welcome` 尾部追加先例仅适用
  载荷末字段）：filter_method/border_radius **不入 wire**（宿主缺省
  Linear/方形填充），未来呈现参数 = 新 tag。落点 `message.rs`（codec
  对称 + round-trip/golden 单测，六算子锚点入 `ts_fixtures`）。
- **src 词汇表**（宿主解析面，`load_image_bytes` 单源）：本地文件路径
  / `builtin:ricepaper|inkwash`（518 内嵌壁纸）/ `data:[<mediatype>]
  [;base64],<payload>` / `http(s)://`（reqwest 3s 超时，结果含负缓存
  每 URL 至多一次）/ `thumbnail://{wid}` 虚拟引用（宿主快照解析，下条）
  ；`/api/__auto/media/` 票据前缀随注在册（宿主本地管线词汇，投影器不
  特判）。**not-yet 词汇显式成文**：`lucide:`/`svgdoc:`（字形/矢量
  栅格化独立线——P026-D1 字形半句维持）经未解析降级处理；未知 scheme
  同。
- **宿主栅格化与降级纪律（I3）**：queue 臂 `broker_surface::paint_ops`
  增 Image 臂——src → 进程级 Handle 缓存（key = src，解码一次防每帧
  重上传；负缓存 = 已定失败，占位与观测去重的依据）→
  `Frame::draw_image`（仓内 canvas 首用，iced 0.14 image feature 链，
  wgpu/tiny_skia 双后端）。**未解析 src（缺文件/超时/未知 scheme/
  字形词汇）= `IMAGE_PLACEHOLDER` 同色占位 Quad + `[drawlist-image]`
  观测行（stderr + ui_console 双落，src 级首败去重）**——禁静默错绘。
  **http miss = 占位先行 + 后台线程解码 + 下帧翻真**（paint 路径禁
  阻塞——os-007 P534-D4 三秒挂渲染教训；翻真由宿主任一后续重绘兑现：
  鼠标/toast tick 250ms/MCP 心跳/帧泵，零新增触发器）。
- **`thumbnail://{wid}` 虚拟引用（D3）**：每帧 `snapshot_window_stale`
  直查（**不进永久 Handle 缓存**——SWR 刷新语义，冻结句柄锁死旧图；
  `WindowThumbnail` 消费臂 from_rgba 每帧重建同律）：命中（含过期）→
  `Handle::from_rgba` 直绘；过期 → `request_capture` 静默重抓（SWR，
  旧图续帧）；真 miss → request_capture + 占位当帧，重抓 `cache_put`
  落地后下帧翻真。wid 非法 = 未解析降级。复用 snapshot.rs 全套
  （TTL 2s / 冷却 500ms / 整窗 screenshot × rect 裁剪），零新机制——
  B 形态 shell（showdesk/workspace preview）的前置能力本期即可测。
- **两投影臂真图升级（I4 同刻度）**：解释态 `layout_image`（src 字面
  量/绑定形态经 `read_state` 代入——026 T-07 a2r 容差同款）与 native
  `View::Image` 臂，src 在场即发 Image op（**rect 推导逐字零变化**——
  `fixed_w/h else min(avail,96)`；src 缺/空 → 占位 Quad 原样容差）；
  icon 的 a2r 降级形态（`View::image_styled("lucide:{name}")`）随之
  入线（宿主字形解析 not-yet → 未解析降级占位，行为连续）。§1.8
  "位图真渲/图像 DrawOp 算子 not-yet"边界条款核销（图像半句）——
  KNOWN-DEBT P026-D1 图像半句同步核销，字形半句维持。
- **TS 远程端（D5）**：`packages/drawlist-renderer` decode tag 6 必达
  （不增臂 = unknown tag throw 破坏 WS 会话——防线义务）+ `fit` u8
  镜像（未知值 throw，与 Rust 对称）；`renderFrame` image 臂 = 占位
  灰底（IMAGE_PLACEHOLDER 同值）——**web 真位图渲染 not-yet 成文**
  （fetch/跨源/data 通道独立增量，TS 无 shm 位图通道）。对拍锚点
  `IMAGE_FRAME_HEX` ↔ Rust `ts_fixtures` 第六锚点双侧钉。
- **帧尺寸边界（D6）**：Commands 档 shm 槽 16KiB + `write_slot`
  超槽**显式拒绝**（`payload N exceeds slot size`）——超长 src（极端
  data: URL）= 帧编码超槽响亮失败，v1 不设投影器侧截断；帧字节增量
  ≈ src 串长（正常 URL 路径零压力，度量行随 e2e 在册）。
- **验证面**：`Image` round-trip（src 四形态/空 src）+ DrawList 直编
  golden + 未知 fit 拒收；宿主解析单测族 `t028_*`（data:/builtin:
  解析缓存、负缓存与 not-yet 词汇降级、http 占位先行后台落缓存、
  thumbnail miss/命中/SWR 三路径）；`p028_image_arm` e2e（AUTO_DESKTOP_
  E2E 门）；p026 既有 image 占位 Quad 断言改写为 Image op 断言（随注
  归因）；TS 对拍 + 占位渲染测试。

### §1.10 v1.10 增量：live 输入接线 + shell queue 面 + lucide 真渲（PLAN-029）

**① live 输入接线两端入册（§1.7 壳侧缺口清偿，P025-D1 核销）**：

- 壳侧订阅：`desktop_window_events()` 扩键盘/滚轮/IME 三族臂
  （session.rs；`EventStatus::Ignored` 门——Captured = host 真 widget
  已消费不转发，keyboard_subscription 先例）。
- 映射纯函数（单测钉死）：`Key::Named(n)` → Windows VK u32（转发集 =
  编辑/导航/功能键 Backspace 8/Tab 9/Enter 13/Escape 27/Space 32/
  Page 33-34/End-Home 35-36/方向 37-40/Insert-Delete 45-46/F1-F12
  0x70-0x7B；修饰键与其余 Named 不转发——热键链路
  `desktop_hotkey_subscription` 自理）；`Key::Character`+text →
  `Chars`（逐字符 `broker_char`）；`Event::InputMethod`：
  Preedit/Commit/Closed → IME 三态（Opened 无子侧语义）；
  `WheelScrolled`：Lines × **40px**（`WHEEL_LINE_PX` 新约定）/
  Pixels 直通。修饰位 wire 布局 bit0 shift/bit1 ctrl/bit2 alt/
  bit3 logo（`wire_modifiers`——iced 0.14 Modifiers 三位一段布局
  不可直用）。
- 泵入与路由：`DesktopEvent::LiveInput { window, input }`（携带
  发生 OS 窗 id——update 臂按 `HostCtx.window` 桌面窗过滤 + picker
  模态避让）→ `route_live_input` → broker_* 六函数**零改动直用**
  （键盘/IME = 焦点窗、滚轮 = last_cursor 命中窗）。KeyReleased
  不转发（无子侧消费者）。
- 真机证据分层（D2 定案）：主腿 e2e `p029_live_input_arm`（真
  outproc native 子进程五腿：⑤协议级/Chars/VK_BACK/ImeCommit 中文/
  滚轮）；辅腿 acceptance channel 扩 **key verb**（`autoui_desktop
  action=key` kind ∈ chars/key/ime_*/wheel——真桌面 update 循环内
  半真机驱动，`DesktopInject::Key`）；`sendinput.rs` FFI 模块就绪
  （user32 SendInput + KEYEVENTF_UNICODE 组装层 + 布局钉死单测；
  **真桌面 SendInput e2e 腿 not-yet**——依赖 B 程序启动序 + child
  观察 channel，启用前置 = `foreground_window()` 目标窗断言）。

**② native 覆盖第三批：shell queue 面四 kind**：

- `popover`：开态覆盖序渲染（`PopoverOverlay` 主块后追加 = paint
  order 置顶，select 同序）；placement 全 14 枚举（Bottom/Top 族 ×
  Start/End + Left/Right = 锚 rect 偏移 + 视口溢出对向翻转 + 边距
  钳制；Modal = 半透明 scrim + 视口居中；Edge* = 贴边 sheet；Pointer
  = 最近右键点）；Point 锚恒 BottomStart 语义（原点对齐）；命中 =
  全屏 catcher（`PopoverDismiss`）先登记、面板项后登记（rev 序面板
  项胜、外点/Esc → `on_dismiss` + 吞不落穿）；**开合零投影器状态**
  （`open` = View 字段随帧）。
- `mousearea`：透传渲染 + 命中序（area 命中先 push、content 子树后
  push = content 项 rev 序优先、空白落 area）；click/contextmenu 族；
  hover 族（on_enter/on_exit/on_move/on_release/on_double_click）
  not-yet（投影器无 hover 态）。
- `windowthumbnail`/`workspacepreview`：`DrawOp::Image` 桥接
  （见 ③ 词汇增量）。
- 覆盖表扩容 + `scan_native_node` Popover 双子树递归（:562 缺口）+
  防漏钉矩阵四夹具；**shell pack 五件装载期 Covered**（B 程序覆盖
  门预演断言 `shell_pack_native_covered`——解析序 $AUTO_OS_ROOT →
  兄弟 auto-os → 主检出）。
- 降级放行两枚（flex-1/shadow 同册先例）：`flex-col-reverse`（列序
  渲染 not-yet）、`opacity-`（alpha 合成无通道——视觉全不透明降级）。
- 翻转复测（D7）：overall 16/36 = 44.4% / judged 16/22 = 72.7%
  （样本扩容稀释 + 046-tabs 入分母；009 opacity 翻绿、041 popover
  半句清偿）→ **双口径均 <95% 维持不翻**（报告
  `docs/plans/reports/p029-native-flip-retest-row.md`；shell 五件
  单列不入 examples 分母）。

**③ 词汇表增量（§1.9 词汇表扩展，零 wire 变化）**：

- `lucide:{name}` / `lucide:{name}#{rrggbb}`——**字形真渲入册**
  （P026-D1 字形半句核销）：`lucide_svg_doc_with`（stroke 按 size
  推导 ≥48px→1.5 否则 2.0）→ currentColor 文档内替换 tint（缺省
  `#FFFFFF` 深色壳面约定）→ resvg 0.45 + tiny-skia 0.11 Contain
  栅格化 → RGBA Handle；缓存键 `{src}@{w}x{h}`（`resolve_drawlist_
  image` 签名扩 `(w, h)`——尺寸维度入键）；未知名 → 负缓存 +
  `[drawlist-image]` 观测（I3）。
- `thumbnail://{wid}!{fallback}`——**fallback 后缀语法**：快照 miss
  → 宿主转 `lucide:{fallback}` 占位图标真渲（灰 quad 降级升级；
  无后缀 miss 维持占位 None——028 口径零回归）。
- `workspace://{ws}!{fallback}`——**宿主合成虚拟引用**（D4-A）：
  `workspace_preview::current()` 数据面（壁纸基色铺底 + 分区 tile
  等比 Contain——`tile_rect` 复用 + tile 近邻缩放 blit/灰块）；
  Published 缺席/分区空 → fallback 图标；逐帧合成不进永久缓存
  （SWR 活语义）。
- not-yet 维持：`svgdoc:`（通用内联 SVG 独立线——D5-b 定案）。

**④ 验证面**：live_input 单测族 4（VK 表/键盘映射/IME+滚轮+修饰/
生产路由落 wire）+ broker 回归 12；sendinput 组装单测 4；popover
几何 14 枚举 + 开闭态渲染/命中/Esc/Modal scrim + 桥接语法 +
MouseArea 命中序单测；broker_surface `t029_*` 3（ink/tint/缓存/
负缓存 + fallback + 合成三路径）；`shell_pack_native_covered`
（五件 Covered）；e2e `p029_live_input_arm` + `p029_shell_face_arm`
（AUTO_DESKTOP_E2E 门；五件套 ops 落 wire + popover 命中闭环四腿 +
mousearea；帧留痕 assets/029/）。

### §1.11 v1.11 增量：B 程序主体——shell outproc client（PLAN-030）

**①投影下行推（宿主→壳连接级，tag 12–14 追加式）**：

- `ShellProjectionPush{face, payload}`——face = `shell_face` 常量
  （1 shell/2 desktop/3 switcher/4 notification_center/5 dashboard，对齐
  `SHELL_MANIFEST`）；payload = 027 typed 载体 wire 编码
  （`shell_projection::wire` 单源：ShellProjection 家族/DesktopSurface
  Snapshot，叶面保形——bool 载体 bool，"1"/"" lowering 单点在 child 侧
  apply）。指纹门宿主侧缓存（per-face；respawn attach 强制失效 = 全量
  重推）；节拍 = 事件驱动（update 排空点邻位）+ ServiceTick 400ms 兜底。
- `ShellClockTick{face, time, date}`——分钟门独立脏帧通道（不入指纹组）。
- `ShellCursorMove{face, x, y}`——事件级数据（设计上不入快照）；宿主
  消费门 = 命中 background 伪窗（桌面空白区语义）或拖拽中。
- 连接级寻址（`wid()` = 0 哨兵）；golden 字节冻结锚 + 全家族 round-trip。

**②命令上行执行（app→host 既有变体落地）**：`DesktopBus{wid, record}`
自端点丢弃臂拆出（`HostAction::DesktopBus`——此前 `let _ = control`
无声消失，desktop.* wire 化缺口清偿）→ `parse_records` 单点解析（52
动词词表零变化）→ registry_id 归因（wid → VWinState；壳伪窗 = 面名
  "shell"/"desktop-face"）→ `desktop_bus_inbox` → renderer ServiceTick
泵后同拍直调 `execute_desktop_commands`（drain 排空点先于泵，桥式写
  会拖到下拍）。acceptance Bus verb 在 outproc 壳下改道同一 inbox。

**③表面 z 平面声明（Hello/Welcome 尾段，追加式）**：`Hello` 尾部
`surfaces: Vec<SurfaceDecl{role,w,h}>`（role ∈ window[缺省/旧端线]/
background/chrome）；`Welcome` 尾部 `WelcomeSurface{role,wid,surface,
rect}`。空则不写尾段——既有消息字节级不变。端点多表面态：`HostEndpoint
.extra_surfaces` wid→surface 路由（帧按消息 wid 定面）+ `activate_multi`；
输入路由 `BrokerClient::owns_wid` 去单值化。**壳双表面 = 宿主 Stack
层槽**（background 替 desktop 面槽[壁纸上/dashboard 下]、chrome 替
任务栏槽[vwin 上/overlay 下]）+ 双伪窗承载 WM 命中（background 垫底
`z_order.insert(0)`、chrome 置顶带矩形——`hit_test` 无透明直通，全屏
chrome 会吞带外点击）。

**④壳客户端与双轨**：`--autodesk-shell` 入口（`shell_client.rs`：双
表面协商 + 投影消费[RqProjector View 全展开——AppProjector 队列臂
对 ForLoop/Conditional no-op] + `__desktop_cmd` 读走 child 化 + 内联帧
免 shm）；spawn = re-exec auto 本体 + `AUTO_SHELL_GEOM`/`AUTO_SHELL_
PACK` env。`shell.apps.shell_model: inproc|outproc`（缺省 inproc——
I1 双轨零回归；读序 env AUTO_SHELL_MODEL > storage）。装在失败回退
in-proc。**v1 边界（D6）**：仅常驻双面（shell taskbar + desktop
surface）outproc；switcher/notification/dashboard + launcher 维持
in-proc（dashboard z 带在 App 窗下与置顶 chrome 冲突 + face 卡宿主活
渲染；launcher 保 iced 聚焦链）。

**⑤看门兵**：pump EOF 死亡检出 → respawn 登记分拍消费（退避 1s/2s/5s
封顶、预算 3 次/60s 窗；不内联 launch——UI 线程阻塞 ~25s）→ attach
指纹失效全量重推；预算耗尽 = 降级观测 + 一次性回退 in-proc（I5）。

**⑥pointer 生产接线（D3 缺口清偿）**：GlobalPress/release 臂
`hit_test` 命中 broker wid（含壳伪窗）时 `broker_pointer_down/up`
（坐标平移窗局部系）。

**验证面**：wire round-trip/golden + 全家族编解码 + 端点多表面路由 +
DesktopBus 拆臂单测；inbox 全链（布局/通知/壁纸族 + 归因 + 幂等）+
指纹门三态；双轨断言（from_storage 缺省 inproc + 垫底窗 z 序/焦点/
MRU）；e2e `p030_shell_outproc_arm` 四腿（双表面首帧 / 真按钮点击 →
DesktopBus → 归因执行 / 投影推送帧变 / kill → 看门兵 → respawn →
全量重推恢复；帧留痕 assets/030/）；回归门 261/262（唯一红 = 基线
预存 imagesurface）。**债务随注**：a2r 编译面轨（shell-lib 组件库
生成模式）not-yet——v1 交付 = 解释面 outproc child（AppProjector/
RqProjector 接缝即替换点）；TS decode tag 12–14 not-yet（无 TS
消费面）。

## §1.12 v1.12 增量：rqhost 第四运行形态（PLAN-031）

**形态定位**：`auto run -r vm -q`（`-r rust -q` 同族）= 宿主 OS
**普通原生窗**运行——不启虚拟桌面，app 进程只产 RenderQueue 帧，一个
**共享后台合成器进程 rqhost**（`auto rqhost`，iced daemon）为每客户端
开一枚真实 OS 窗、栅格化（DrawListPainter，与桌面 broker 窗同一
栅格化器）、转发输入。多 app 共享单实例（compositor 架构，虚拟桌面
同型；用户裁定不做 standalone——单 app 由 shared 自动孵化等价）。

**① rendezvous 采纳协议**（传输层管道串约定，零 codec 变体）：

- well-known 管道 `autodesk-rqhost`（`autodesk-broker` 同族；测试缝
  env `AUTO_RQHOST_WELLKNOWN`，P489 `adjudicate_on` 同型）。**单实例
  仲裁**：锁管道 `<well-known>-lock` 以 `FILE_FLAG_FIRST_PIPE_INSTANCE`
  声明（tokio `ServerOptions::first_pipe_instance`）——第二实例创建即
  PermissionDenied → 干净退出（码 0）；OS 级原子零竞态窗口。
- 采纳记录（DesktopBus 载荷，`incubate␟<name>␟<mode>` 约定族）：
  `adopt␟<app_name>` → 应答 `adopt␟<per-app pipe>`；per-app 管道
  `<wellknown>-app-<n>` 先行 listen（broker 同型）。**双动词兼容**：
  兼收 `incubate␟<name>␟<mode>`（broker 族记录——旧生成物
  `--autodesk-incubate` 直连 rqhost 零改接驳；mode 字段忽略，宿主恒
  Welcome=Commands）。探测 ping（连上即关）吞掉不占名额。
- per-app `wait_connect` 线程化（broker serve_once 的阻塞第二等连
  不适用——rust 轨 cargo build 分钟级延迟不得阻塞 adopt 环路）。

**② 客户端权威采纳**（与桌面"宿主内容权威"的对偶）：rqhost 侧
`ResolveAndAttach` **无 resolver**——Hello 凭据（title/width/height）
直接 `activate`（app_id/wid = rqhost 自有计数器，rect=(0,0,w,h) 窗口
本地坐标）。桌面 `broker_apply_actions` 的 resolver MISS 弃连路径
零牵连。**策略档**：`ClientTarget::Rqhost{wellknown,app_name}`
（rendezvous 采纳内建）→ `reconnect=None` = **exit-on-EOF**（宿主死
= 客户端干净退出 + `[rqhost-client] host lost` 观测行；原生 app 心智）；
桌面档（Direct/Broker）30s/50ms 重连缺省不变。

**③ rqhost 生命周期语义**：用户关窗 → 宿主 `Close` → app
ExitRequest → Reclaim（退出码 0）；app 死 → EOF → 窗回收；**末窗
关闭 → daemon 退出**（iced daemon 空窗不自动退出的反面——自建
`iced::exit`；门 = 无在册窗 ∧ 无待定采纳 ∧ 曾开过窗）；帧泵 15ms
（400ms ServiceTick 对原生窗输入→帧响应太钝）。

**④ 输入按窗路由**：事件自带 window_id 即路由键（免桌面 WM
hit_test/焦点语义——原生窗 OS 自理）；坐标 = 窗内坐标即表面坐标；
键盘/IME/滚轮走 029 LiveInput 映射族（Ignored 门），指针全事件直订。

**⑤ 大帧回退**（v1.12 附带根修，桌面 broker 同益）：shm 槽 16KiB
（Commands 档）装不下的帧载荷，客户端回退**管道内联** `FrameReady`
（v1 合法变体）——此前 `if let Ok` 静默弃帧 = 冻结。

**边界注记**：Hello.icon（编码字节）v1 忽略（iced Settings 有 icon
位但需 RGBA+尺寸元数据）；多用户终端服务器下 well-known 全局命名
空间（`autodesk-broker` 同口径，单实例 = 单机器域）；job object 级
宿主亡兜底仍为 KD 债（协议级 exit-on-EOF v1 够用）。**与桌面
ad-hoc attach 同源**：PLAN-030（shell-outproc）的采纳协议与本节
rendezvous 同族不同宿主——接口演进互链（`--rq-host=desktop` 参数位
预留，实现归 030 线）。

**验证面**：rqhost 单测/集成 13（rendezvous 往返/锁仲裁/零装载采纳/
并发接纳/泵回收 EOF/ensure 退避/全循环×真 ClientPump/输入不串扰/
策略档/vm fork 凭据/大帧回退/末窗门四态/键入闭环[键入 100→宿主合成
帧文本 212 联动 + 零串扰]）+ e2e `p031_rqhost_arm`（七腿全景：
单/多 app、竞态、resize、kill 双向、**用户关窗 X→app 码 0**、
**末窗自退→daemon 退出**、**真实自动孵化[锁持有→自退→锁让出]**、
**rust 轨[a2r 重生成→注入→采纳→关窗码 0]**、降级显式；AUTO_DESKTOP_
E2E 门，留痕 `docs/plans/reports/assets/031/`[进程清单+daemon stderr]）
+ os 侧 `scripts/smoke-031-rqhost.sh`（生产 well-known 演示位）。

**输入闭环承载裁定**（2026-09-19 修复轮）：真机合成输入（SendInput
全局队列 / PostMessage legacy 鼠标消息）在 ToDesk 输入钩子类环境不可
用（SendInput 零送达 + winit 0.30 WM_POINTER 路径忽略 legacy 投递；
native_dock_e2e T4 同款环境事实先例）——键入→换算联动闭环由**集成
测试承载**（vm_typing_loop_over_pipe：真管道×真源×真 ClientPump×
rq_update 输入臂，断言宿主合成帧文本 212）；宿主侧 revision 观测行
（`[rqhost] revision` 仅变化打行）为帧变断言锚点。工程注：子进程
stderr PIPE 必须排水——构建期警告超管道缓冲即阻塞写端（假死）。

**保真口径**：-q 渲染面 = DrawListPainter → vm 轨解释态全保真
（AppProjector 全 vocabulary 投影，无覆盖门）；not-yet 词汇（popover
族/lucide 未知名/未解析 src）占位盒 + 观测行既有纪律承袭；native 轨
`-q` 走 `ensure_covered` 既有门（queue 档拒 NotCovered 退出留痕）。

## §1.13 v1.13 增量：native 覆盖 ramp v3 + 缺省翻转（PLAN-032）

**裁定**：复测（2026-09-19，仪器 `native_flip_coverage_data_row`）
judged **22/22 = 100% ≥ 95%** 阈值 → **native `Auto` 缺省 = queue**
（`resolve_native_frame_mode` Covered 臂 `FrameMode::Pixels` →
`Commands`；观测行 `native auto -> queue … default flipped@ramp3`）。
026/029 两复测未达标维持 independent 的在案裁定由 p032 报告收束
（`docs/plans/reports/p032-native-flip-row.md`——对差表/保真边界/翻转
清单）。翻转前缀：覆盖判定/投影器/宿主侧全部为追加式演进，**零 wire
变体**，`PROTOCOL_VERSION` 仍 1。

**数据门口径升级**：仪器本批补 judged 计算（剔除仪器桶 parse-fail/
extract-fail/bridge-fail/no-widget——026 D3 剔除集同源）；
`assert!(!flip)`（达标即红逼翻转的向下任）**反转**为 `assert!(flip)`
——翻转后 judged 跌破 95% 门即红（降级需显式裁定：台账裁定行 +
报告更新，禁静默回归）。

**五族补齐口径**（缺项清偿 = 判定翻绿；真渲/降级分层明示）：

1. **tabs 整 kind**：kinds 入册 + `View::Tabs` 投影臂（等宽托盘 ×
   default/enclosed[选中下划线]两变体 + Top/Bottom + 内容区
   contents[selected] + `on_select` 逐项命中 `TabSelect` →
   `TabsSelectCallback.call(index)` 物化——VM 轨首参 = value 串在
   回调内包装）。a2r 断裂映射修复：`ui_gen/rust.rs` 专属臂
   （labels/contents 折叠、value 绑定运行时 position、variant
   full-path、onselect 闭包物化载荷——select 臂先例）；
   `tag_to_view_fn` 的 "tabs"/"tab" 断裂映射移除。M7-c②（jade-garden
   tab×27 / auto-musk tab×16）依赖解锁。
2. **hidden = display:none 真渲**：`NodeStyle.hidden` 布尔 +
   `layout_view_node` 单一 choke（子树整体不渲染不占位）+ 块流 gap
   序整段跳过（兄弟间只留一个 gap）。**display 族响应式覆盖规则**：
   parser 剥 sm/md/lg/xl/2xl 前缀（class.rs）——"hidden md:flex" 解析
   为 [Hidden, Flex]，display 族类在场清位 = 桌面档可见（CSS 生成序
   + 桌面目标假设）；裸 md:hidden = 隐藏。
3. **样式版 grid 真渲**：`NodeStyle.grid_cols/grid_rows` +
   `layout_view_block` 入口单一分岔 choke（堆叠族/容器/scrollable 全
   路径）复用 Grid walker（`layout_grid_cells` 单源——`View::Grid`
   变体臂同构零变化）。解析序：GridCols(n) 优先 / GridRows(m) →
   ceil(cells/m) / 裸 Grid 不记档（无模板单列与纵向堆叠等价）。
4. **定位族分层**（D1 定案）：`absolute` + offset **真渲**——
   `place_absolute_children` 延迟放置（脱离流零占位不贡献父高 /
   left-top 优先·right-bottom 按父盒尺寸反算 / 同层 z 稳定排序 /
   ops-hits 追加主序之后 = 覆盖序置顶——Popover 平移臂先例）；
   `fixed`/`sticky` **降级放行**（渲染 in-flow 原位 no-op——视口锚定
   真渲债另立；opacity/overflow 先例）。`top-/left-/right-/bottom-/z-`
   前缀族放行。
5. **映射兜底族**（D5 + 运行时面）：`SelfCenter` → center_children
   真渲（012）；`Inset(N)` → 四槽偏移填充真渲（组合 absolute =
   全覆盖锚定；024 模态纱）；`LineClamp` / `FlexWrap` → 判定放行
   渲染 no-op 随注（DrawOp 无行数裁剪/折行通道——真渲债登记）。

**运行时口径差**（T-07 发现，`native_gate_runtime_views_of_six` 钉）：
生产 auto 裁决（a2r main / VM 孵化）消费 `component.view()`——**路由
解析后**子树含页组件样式，较仪器静态 App 壳扫描宽。021/024 的运行时
缺项经 D5 族补臂清偿；**018（truncate——文本裁剪）/ 041（codeeditor
——整 kind）为真 not-yet 家族**，运行时 auto 裁决维持降级
independent + queue 门拒收留痕（I3——禁静默错绘），家族债登记
KNOWN-DEBT（M7-c 撞面立项）。

**保真边界随注**（判定翻绿 ≠ 渲染全真）：fixed/sticky in-flow 降级；
z 完整栈序（in-flow z / 跨层栈序 not-yet——absolute 同层相对层级已
渲）；absolute 锚定最近父块内容盒（CSS nearest positioned ancestor
爬升近似）；堆叠族自身 fixed 高度既有丢弃边界（锚定按内容盒自洽）；
tabs position Left/Right 渲染降级 Top；line-clamp/flex-wrap 渲染
no-op。

## §1.14 v1.14 增量：单投影器统一——rq-projector-unify（PLAN-033）

**裁定背景**（用户 2026-09-19 定调）：`-q` = `--render-queue` 对两轨
一视同仁——VM 版经 `-q` 走 RqProjector 获得**省内存**（免每 app
iced/wgpu 后端）+ **native 保真**（View 全展开渲染——popover/图标/
图像面真渲，对照解释降级投影形态）。更名决策同拍授权
（NativeProjector → RqProjector，P020-D1 销账）。

**① 改接**：`run_dynamic_client` Commands 臂换 RqProjector 装配
（`ensure_covered` 启动门 + `run_client_session` 泛型泵）——调用面零
改动（cmd_autodesk/rqhost/lib.rs 直车）；`auto run -r vm -q` 全链经
物化 View 入投影器，与 a2r 编译轨同律。

**② VM 轨三补**（改接暴露的缺口）：

- **input_state_map 回写**：`DynamicComponent::on()` 内绑定字段类型
  保值回写（D1=A 单写点——与 a2r 生成 on() 同构：输入事件先
  last_input_text 写全绑定字段再跑 handler 体）。零参内联闭包
  oninput（003-converter 双向换算）与单参 handler 两形态闭环；
  on()（投影臂入口）与 on_with_input_for（renderer 自开窗入口）
  互斥派发——无同事件双写路径。
- **VM 定时器泵侧驱动**（D2=B）：`Component::fire_due_timers` 缺省
  钩子（false）+ DynamicComponent override（timesources 逐条到期
  派发：Timer 条目走 fire_timer[when 门内置]/Tick 条目走
  on_with_input_for；首拍对齐 interval；mount 过滤近似）。RqProjector
  的 `poll_tick` 首行消费（true → revision 前进）；a2r 结构体走缺省
  零影响。
- **`__desktop_cmd` 读走上行**（D3）：`Component::
  drain_desktop_commands` 缺省钩子 + FrameSource 转发 + ClientPump
  双读走点（输入派发后/周期拍后）→ `ControlMsg::DesktopBus`——
  shell_client c4 语义泛化到任意 VM `-q` client。

**③ AppProjector 退役**：生产消费者清零——解释 re-exec 臂
（spawn_outproc_child）+ `process_model=outproc` 全局配置位拔除
（storage 键读到 outproc = 忽略留痕迁移注记，I3；**解释态两合法
形态 = inproc 直挂 / `-q` 经 native 臂**）；解释 pixels 臂
（run_independent_child）退役（a2r 轨像素兜底
run_independent_native_child 不受影响）；本体 + 块流 walker 删除，
ClientPump 缺省泛参收口；remote.rs 宿主孪生迁 RqProjector（D4=A
——孪生随 native 布局 = 镜像真实命中面）。

**④ 更名**：NativeProjector → RqProjector（代码 + canonical 文档 +
两侧 specs.json 派生同步；归档零改动——AGENTS 纪律）。

**⑤ L3 v2a 快照面平移**：`Component::apply_state_snapshot` 钩子
（VM 组件实现；a2r typed 缺省 false 维持 not-yet）——RqProjector
on_control 的 StateSnapshot 臂经钩子写回 + revision 续接
（AppProjector 退役语义不丢）。

**验收口径**：003/001/027 `-q` 经 RqProjector 渲染（e2e
p033_rq_unify_arm：027 开窗+首帧+覆盖门零拒；vm_typing 集成断言
键入→换算帧 212）；VM 内存对照行 = 003 直挂 225088KB vs `-q`
8016KB（省 217072KB/合 212MB——免每 app iced/wgpu 后端；≤10MB 门
沿用）；回归门 = desktop_protocol 173/174（唯一红 covered_elements_
within_target_set 为 PLAN-656 表同步尾巴在册红，与本计划无关——
stash 对照同败实证）。

## 2. Wire Format（信封）

小端。头部 12 字节定长：

```text
[0..4)   magic   "APDL"（0x41 0x50 0x44 0x4C）
[4..6)   u16     protocol version
[6]      u8      channel（1..=5）
[7]      u8      保留（0）
[8..12)  u32     payload 长度
[12..)   ...     payload（通道内消息：u8 tag + 字段）
```

- 原语：bool=1B(0/1)、String/bytes=u32 长度前缀+内容、Rect=4×f32、RGBA=4×u8。
- 解码终点断言：消息载荷必须恰好耗尽（`TrailingBytes` 拒收——结构漂移哨兵）。
- 零新依赖；`DesktopBus` 载荷沿用既有 `DesktopCommand` 文本记录格式
  （`verb\u{1F}arg`，shell.at 写入侧同源）。

## 3. 五通道

| # | 通道 | 方向 | 消息（tag = 变体定义序） |
|---|---|---|---|
| 1 | 孵化/握手 | app→host→app | `Hello{version,app_name,title,icon,wh,fonts}`(1) → `Welcome{app_id,wid,surface,rect}`(2) → `Ready`(3) |
| 2 | 帧 | app→host（ack 反向） | `FrameReady{wid,frame_id,slot,damage,revision,DrawList}`(4)、`FrameAck{wid,frame_id,slot}`(5)、`BufferAlloc`(1)/`BufferRelease`(2)/`Resize`(3) host→app、`CacheControl{wid,drop_keys}`(6) app→host |
| 3 | 输入 | host→app | (Wid, event) 编码：`PointerMoved`(1)/`Pressed`(2)/`Released`(3)、`KeyPressed`(4)/`KeyReleased`(5)、`CharTyped`(6)、`Scroll`(7)、**`ImePreedit`(8)/`ImeCommit`(9)/`ImeCancelled`(10)** |
| 4 | 控制 | 双向 | host→app：`Close`(1)/`Focus`(2)/`Resize`(3)；app→host：`TitleChanged`(4)/`Notify`(5)/`ExitRequest`(6)/`DesktopBus{record}`(7) |
| 5 | 观测 | 双向 | host→app：`Attach{sink}`(1)/`Detach`(2)；app→host：`Log{level,message}`(3)/`Metric{key,value}`(4) |

### 3.1 帧：共享纹理模拟（Stage 1）→ 共享内存（Stage 2）

- 表面句柄 `surface` 由宿主在孵化时分配；**双缓冲 2 槽**：app 写非前台槽 →
  `FrameReady(slot)` → 宿主合成（翻面）→ `FrameAck` 归还原前台槽。
- `damage` = 脏区（widget 本地坐标；None = 全帧）。`revision` = app 侧内容
  单调版本，宿主缓存键与 413 §7.3 同源。
- 载荷 v1 = `DrawList{clear, ops:[Quad|Text]}`（kind tag=1，扩展位预留给
  Stage 2 的全量 RenderCommand lowering——见 §6 偏差记录①）。
- Stage 1 的"共享缓冲" = 宿主内存 `SurfaceStore`（每 surface 2 槽）；
  Stage 2 换 OS 共享内存句柄，槽位/翻面/ack 语义不变。

### 3.2 Plan 413 §7 三点落位

| §7 要求 | v1 落位 |
|---|---|
| ① IME 下行（preedit/commit/光标矩形） | `InputMsg::ImePreedit{text,cursor}` / `ImeCommit{text}` / `ImeCancelled`；`EditorFrameSource` 映射到 `EditorInput` 同名输入 |
| ② 字体注册（app 自带字体上传） | `HandshakeMsg::Hello.fonts: Vec<FontBlob{family,data}>`；Stage 1 协议通道就位，真上传随 Stage 2 transport |
| ③ 按行 CacheControl / DirtyRect | `FrameReady.damage`（脏区）+ `FrameMsg::CacheControl.drop_keys`；编辑器键 = `(revision<<16) ^ fold_hidden`（fold 翻转不改 revision 但必须重绘 gutter） |

## 4. 状态机（非法迁移一律拒收）

```text
App  : Detached --connect()--> Handshaking --Welcome(+Ready)--> Active
       Active --Control::Close--> Closing --ExitRequest--> (BufferRelease) --> Detached
       Active --send_exit()--> Closing
Host : Listening --Hello(版本校验)--> (ResolveAndAttach) --activate()--> Active
       Active --ExitRequest--> (ReclaimWindow：wm_remove_win+App 移除+表面释放) --> Listening
```

错误形态（`ProtocolError`）：`WrongState{state,msg}` / `VersionMismatch` /
`ResolveFailed(String)` / `NotActive` / `Codec(_)`。方向错（app 收 Hello、
宿主收 FrameReady 于 Listening 等）与时序错均拒绝，不静默丢弃。

## 5. Stage 2 换 transport 面（本协议的验收遗留兑现点）

| Stage 1（loopback） | Stage 2（真两进程） | 不变量 |
|---|---|---|
| `LoopbackEnd::send/try_recv`（内存字节管道） | `\\.\pipe\autodesk-broker` 命名管道 / memfd | 管道保序 + 字节化过线 |
| `SurfaceStore`（内存 2 槽） | OS 共享内存句柄 + named events | 槽位/翻面/ack 语义 |
| `ResolveAndAttach` → `build_dynamic_component` | spawn-client 双模 exe（`--autodesk-client=<pipe>`） | 孵化三步：身份上报 → 分配 Wid+表面 → 回传句柄 |
| `ReclaimWindow` → 进程内移除 | 进程退出 → 回收虚拟窗（462 Close 语义） | 窗随 App 消亡 |
| `ObserveMsg` 收件箱 | desktop_mcp per-app 端口代理 | 观测通道可代理 |

窗口形态迁移 L1/L3（desktop-shell §7.1）：v1.2 已落地（L1 = 宿主侧 WM 登记翻转
`detach_surface_to_os_window`/`attach_surface_back`；L3 v2a = `StateSnapshot`
快照注入恢复）。像素级投影保真与 live-iced 渲染器换接仍归后续（投影器 v1
以 text/button + 线性堆叠为界）。

## 6. 偏差与决策记录（对 Plan 386 原文的修正）

1. **帧载荷 v1 = `DrawList`（quad+text run），不含全量 VTree→RenderCommand
   lowering**。原"工作项 Stage 1"的逐 example golden lowering 已被 2026-08-28
   复活重构（计划 §0）取代；全量 lowering 归 Stage 2。依据：`code_editor/
   draw.rs` 头注明言 Stage 1 = "serializes it to quads and text runs"；
   §0 "frame 用内存缓冲模拟共享纹理"。
2. **loopback demo 为 headless 会话对象级验证**（`DesktopSession`/`WmState`
   真实参与孵化/命中/回收；live-iced 渲染器换接 `dynamic_view` → 协议客户端
   留 Stage 2 随真 transport 一起做——I1 评审已证零删除替换可达）。
3. 单客户端 v1：`HostEndpoint` 一次只持一个客户端；多 App 并发复用同一
   协议归 Stage 3（多 App + 形态迁移）——v1.2 已兑现（`BrokerClient`
   驻留多 client 宿主；`HostEndpoint` 单例本身不动，多 App = 每连接一份）。
4. **远程 WS 端口 `:17800`**（Stage 6，Plan 508）：回环 v1——`remote::REMOTE_WS_PORT`
   常量；token 走 URL query 而非自定义头（浏览器 WebSocket API 限制，
   计划待澄清③边界的落地形态）。跨网/TLS/鉴权另立计划。

## 7. 验证（Stage 1 + Stage 2 + Stage 3 增量）

- **Stage 6（v1.4，Plan 508）**：WS transport 单测四条（round-trip/
  golden bytes/EOF/token 拒收）；远程会话 T2 集成（outproc 孵化 → WS
  三件套 → 点击 → Counter:1 闭环，headless）；TS 渲染器 vitest 20/20
  （Rust↔TS 对拍锚点三件双侧钉）；Playwright 端到端（真 auto.exe 宿主
  → Chromium 首帧 Counter:0 → 点击 "+" → Counter:1，截图留痕
  `docs/plans/reports/assets/508/`）；G2 对比实测两臂 × 两轮（内存差
  <1%）。

- **Stage 3（v1.2，Plan 480，56 测试 ×2 连跑全绿）**：投影快照/命中派发
  （prop + FStr 插值）、真实 `auto` 二进制双进程 smoke、`enable_broker`
  + `request_incubation` 双端连通落 462 会话、N=3/5 压测（孵化→全 Active
  →逐 App 点击帧递增→30s 稳定存活→收尾退出码 0）、内存边际增量采样
  （N=1/3/5，数字见报告）、弹性重连（server drop → 存活等待 → 重建管道
  → count/revision 连续）、L1 往返状态连续、L3 v2a 快照注入（迁移前后
  count 一致 + revision 延续）。

- **Stage 2（v1.1，44 测试 ×N 连跑全绿）**：命名管道 FIFO/残帧/EOF、
  shm 跨端读写 + DrawList 槽内往返、broker 全链孵化（通真实 462 会话）、
  L2 往返 revision 连续、**双模两进程集成**（re-exec 子进程：
  adjudicate→孵化→共享内存帧→点击→L2 Standalone→状态标记）。
- 平台教训（Win32 管道，已沉淀 `transport.rs` 头注）：同步句柄阻塞
  ReadFile 的跨句柄唤醒、PeekNamedPipe 空管阻塞、他线程 CancelIoEx/
  CancelSynchronousIo 均不可依赖——overlapped/async IO + select 取消
  是唯一可靠路径。


- 协议测试 31 项（`cargo t desktop_protocol --features ui-iced` 全绿）：
  全消息 round-trip、每通道 golden bytes、坏 magic/版本/未知 tag 拒收、
  状态机全迁移 + 非法迁移、FIFO/线上破坏、462 对象真实孵化/命中/回收、
  计数器 loopback demo 直挂等价（state/frame parity）、编辑器 golden
  （打字/IME/缓存键 + 协议输入与 core 直喂无差）。
- 验收句（计划 §0）："一个 App 的帧/输入/控制经协议通路渲染进 462 虚拟
  窗口，行为与直挂无差" —— `counter_loopback_demo_parity_with_direct_mount`。
