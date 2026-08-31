# AutoUI 桌面协议 v1（Desktop Protocol v1）

> **版本**：v1（2026-08-29 定稿，Plan 386 Stage 1）。
> **施工图**：[autoshell](../../25-autoshell-dsl-unified-shell.md) §7/§7.1
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
→ Stage 5（覆盖爬坡全 widget 族 + IME 光标下发，parity 门禁进日常档）→ Stage 6（默认
策略 + web/远程端消费同一 command 流）。

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

窗口形态迁移 L1/L3（autoshell §7.1）：v1.2 已落地（L1 = 宿主侧 WM 登记翻转
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

## 7. 验证（Stage 1 + Stage 2 + Stage 3 增量）

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
