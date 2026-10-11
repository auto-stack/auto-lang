---
plan_id: PLAN-760
status: execution_done         # drafting → executing → execution_done → reviewed → archived
feature_name: remote-frame .at 元件（宿主侧远程帧消费面）
author: [agent]
created_at: 2026-10-11
updated_at: 2026-10-11

# /auto-plan:review 结束时填写：
supersedes_spec_components:
  - docs/specs/auto-lang/ui/overview.md
new_spec_components: []
touched_goals: []

affects: [auto-lang/ui]        # 受影响的 specs 路径
current_step: 6
total_steps: 6
---

# [PLAN-760] remote-frame .at 元件（宿主侧远程帧消费面）

## 0. 变更摘要

新增 `.at` 内建元件 **`remote-frame`**：在 .at 视图中按 `url`（bridge WS 端点）挂载
一个宿主侧远程帧消费面——订阅远端应用的 DisplayList v2 帧并渲染（iced 轨经
`broker_surface::displaylist_element`；web 轨生成 div 由宿主前端自行接管），把本
地指针/键盘输入按桌面协议 `InputMsg` 回传远端。它是 auto-musk PLAN-102（Canvas
内嵌交互预览）T-04 的 VM 轨消费面（上游缺口 J2，用户已裁定全量实现）；本计划
只交付上游元件与其自身验收，musk 侧集成归 PLAN-102 T-04 收口。

本阶段只建立执行合同，不实现代码，不修改权威 Specs，不宣称运行验证通过。

## 1. 目标

- `.at` 视图可写 `remote-frame { url: "ws://…?token=…" }` 并在两轨生成合法产物：
  web = 透传 class 的 div 占位（宿主 JS 接管）；VM = 订阅-渲染-输入一体的 iced 元件。
- VM 轨实机：bridge（`auto rqhost --bridge-ws`）后的夹具应用帧持续显示；operate
  输入（指针按下/字符）经协议到达远端（计数器递增）；断连在重连预算内恢复。
- 生命周期安全：url 切换/元件卸载断开旧连、清线程与句柄；迟到帧不污染新代次。
- 既有门零回归（schema_drift 双向栅栏 / element_coverage / lib 测试 / 442 gap 语料）。

非目标：AutoUI 全局远程桌面壳（broker_clients/窗口管理器既有面不动）；rqhost 宿
主本身的行为变化；web 轨的 JS 渲染器（auto-musk 已有 canvas_embed_web.ts 消费
@auto/drawlist-renderer，本计划 web 臂只生成占位 div）；多 surface 聚合/窗口管理。

## 2. 架构方案

四层接线（全部沿用既有积木，新增的是「元件化宿主客户端注册表」）：

1. **元件声明**：`schema/aura.at` 增 `element remote-frame`（props: `url` 必填、
   `class`/`style` 泛化；`allows_children: false`；`backends.iced: "full"`、
   `web: "component"`）+ `aura/element_coverage.rs::element_table()` 登记（双向
   栅栏，tests/schema_drift.rs）。解析器零改动（tag 白名单不存在；S001/S002 为
   advisory，schema 声明即消音——extract/validators 已核）。
2. **View 变体**：`ui/view.rs` 新增 `View::RemoteFrame { url, class, style }`（先
   例：`WindowThumbnail`/`WorkspacePreview` 宿主数据叶——数据不经 VM 状态）。必须
   带 `convert_view_messages` 臂（PLAN-002 A1 教训：新变体缺臂静默落 Empty）。
   `aura_view_builder.rs` 双镜像分派各加 `"remote-frame"` 臂（tracked :2589 族 /
   untracked :4695 族；`convert_mouse_area_untracked` 是最简模板）。**不选**
   `View::Custom`——iced 渲染器当前把 Custom 渲染成 0 高 Space（renderer.rs:4520，
   factory 注册表未建，提案 066 §3.2 首件另立）。
3. **宿主客户端注册表**（新模块 `ui/iced/remote_frame_registry.rs`）：全局
   （once_cell）`url → RemoteFrameClient` 表；client = WS 端（`transport::ws::
   connect(url)`，tokio-tungstenite 读线程同步收件箱——WsEnd 既有形态）+ Hello
   订阅 + 最新 DisplayList 槽 + 重连预算（`ReconnectPolicy` 30s/50ms 语义对齐
   client_runtime）。**复用判定归 T-01 定案**：shell 的 remote-mirror 客户端
   （`attach_pending_remotes`/`pump_remote_mirrors` 族，renderer.rs:23092-95）与
   自建迷你 host-client 二选一——以最小侵入为准，决策工件落 T-01。
4. **泵与输入**：泵挂 `DesktopEvent::ServiceTick` 臂（既有 400ms 订阅拍，
   renderer.rs:23062 `pump_broker_clients` 同位；纯订阅拍不强制 present——沿
   `presentation_keepalive_tick` 的 InvalidateRect 保活）。输入 = iced
   `mouse_area(displaylist_element(...))` 包装（rqhost.rs:1334 同款先例）+
   自定义 canvas Program 的 update 臂或 mouse_area on_press/move/release →
   `ProtocolMsg::Input(InputMsg::PointerPressed/Released/CharTyped{…})` 经
   client 的 Transport 发送（坐标 = 元件内局部逻辑域，事件→注册表路由）。

**帧节奏设计约束（如实登记）**：帧推进上限 = ServiceTick 400ms 拍 + keepalive；
对 AC「持续更新」够用，交互手感（逐帧跟手）弱于 shell 原生 remote 窗。这是
embedded 面板的受控取舍，非缺陷；若实机走查不可接受，再立独立拍提频项。

## 3. 技术栈

- Rust：auto-lang crate（schema/aura extract、view.rs、aura_view_builder、iced
  renderer、desktop_protocol transport/codec）；once_cell 全局注册表。
- 协议：桌面协议 v2（Hello/Welcome/FrameReadyV2/DisplayList；InputMsg
  Pointer/Char——P6 探针编解码同源，与 @auto/drawlist-renderer messages.ts 对齐）。
- 测试：schema_drift/element_coverage 栅栏、lib 单测（注册表生命周期/变体往返/
  提取）、e2e 实机腿（rqhost bridge + 计数器夹具 + .at 宿主 demo，沿 PLAN-102
  P6 探针形态）；Rust↔TS golden 对拍不动（codec 既有）。

## 4. 需求分析与背景调查

### 4.1 授权与上游关系

- 需求源：auto-musk PLAN-102（T-04 VM 轨消费面缺口 J2）。用户于 2026-10-11 经
  AskUserQuestion 裁定「J2 全量实现（上游）」（弃 VM 受控降级方案）；随即指示
  在本仓建立上游计划。授权范围 = 上游元件交付与其自身验收；musk 侧集成与四轨
  回归仍归 PLAN-102。
- 本仓流程授权：PLAN-102 §4.1 的跨仓 worktree 约定 + 本仓 AGENTS.md auto-plan
  范式。执行 worktree：`D:/autostack/.wt/lang-760/auto-lang`，分支 `plan-760-dev`
  （Plan 529 布局）。

### 4.2 代码事实（全部文件:行已核，2026-10-11）

- 积木在位：`displaylist_element`（broker_surface.rs:575，canvas Program 纯视觉
  ——输入需包装）；`ClientPump`（client_runtime.rs:451，客户端侧——本件是宿主
  侧，仅借 ReconnectPolicy 语义）；`transport::ws`（WsEnd/WsListener，
  transport.rs:454+，读线程 + 同步收件箱）；shell 泵（ServiceTick 400ms 订阅拍
  session.rs:3104 / renderer.rs:23062；keepalive InvalidateRect renderer.rs:12523）；
  shell 输入回传（broker_pointer_down 等 session.rs:4560+，窗口管理器级）。
- 缺口：仓内无任何 remote-frame/RemoteFrame（全库 grep 零命中）；元件 schema/
  coverage、View 变体、双臂转换、注册表、输入包装五件全缺。
- 元件面：解析器无 tag 白名单（extract.rs 直通）；S001/S002 advisory
  （ui_gen/validators.rs:1046/1081，strict 下 S002 阻塞）；schema/aura.at mouse-area
  声明形态（:887-899）为模板；`element_coverage.rs:28` element_table 双向栅栏。
- View 变体先例：WindowThumbnail/WorkspacePreview（view.rs:1100/1112）宿主数据
  叶 + PLAN-002 A1 `convert_view_messages` 缺臂教训（view.rs:1110 注释）。
- View::Custom 现状：renderer.rs:4520 渲染为 0 高 Space（factory 表未建）——不采。

### 4.3 调查基线

代码基线：auto-lang master `f9d647bc353987a8fa0c2ca2ce20af35fa90ff67`；
musk-102 依赖 worktree（`D:/autostack/.wt/musk-102/auto-lang`，分支
`auto-musk-dev-102`，领先 master 3 提交：S1 + D-2 + D-1——rqhost bridge 宿主与
MCP 共位，T-05 e2e 依赖其中 `--bridge-ws`）。权威来源 SHA256（执行期复验）：
`docs/specs/auto-lang/ui/overview.md` = 2e93a616b4e10e0b…（全量以 sha256sum 复核）。

### 4.4 前置与依赖

- **P-1（执行序前置）**：`auto-musk-dev-102` 的 3 提交按 musk AGENTS.md「依赖改
  动消费验证通过后尽快合回」应在 T-05 e2e 前进入本仓 master（PLAN-102 P5/P6
  收据 = 消费验证证据）。两条路线择一：①先快进合回（推荐，消除双基线）；②
  plan-760-dev 改从 `auto-musk-dev-102` 分出（合回序在 review 前理顺）。执行期
  定案并记录。
  [✅ 2026-10-11 定案=路线①] master 已因 PLAN-757/758 前进（dd5457fe1），纯快进
  不可行；`git merge-tree --write-tree` 确认零冲突后普通 merge 落地（a58a2142d，
  6 文件 +513/-8）。lang-760 worktree 从 merge 后 master 分出，单基线。 specs
  SHA 漂移登记：overview.md 起草值 2e93a616… → 执行期 3b49d016…（PLAN-757 merge
  沉淀改写，ui/ 代码面零改动已核实 f9d647bc..dd5457fe1 无 src/ui diff）。

## 5. 详细设计

### 5.1 元件契约

```
remote-frame {
    url: "ws://127.0.0.1:<port>/?token=…"   // 必填；变化 = 断旧连建新连
    class: "w-full h-full"                   // 泛化（UNIVERSAL_PROPS）
    style: "flex-1 min-h-0"                  // 泛化
}
```

- 无子件、无业务事件（v1）；帧到达不进 VM 状态（宿主数据叶——渲染器直读注册表）。
- web 生成：`<div class="[style 字面量] [class]" data-remote-frame="1" />`——
  宿主前端（musk canvas_embed_web.ts 同族）自行 querySelector 接管；本仓 web 臂
  不实现 JS 渲染器。
- VM 生成：`View::RemoteFrame` → 渲染臂 = mouse_area 包装 displaylist_element
  （无帧 = 灰底占位 + "connecting…" 文本）。

### 5.2 注册表与生命周期

- `RemoteFrameRegistry`（once_cell::sync::Lazy + Mutex<HashMap<String, Client>>）。
  引用计数：View::RemoteFrame 每帧构建时对当前 url 计数 +1（渲染器侧 attach 钩
  子），url 消失/切换 → 计数归零 → 断开清线程（借 WsEnd Drop；读线程随收件箱
  关闭退出——以 transport.rs 既有 Drop 语义为准，T-01 核）。
- Client：`{ end: WsEnd, hello: Hello, latest: Arc<Mutex<Option<DisplayList>>>,
  reconnect: ReconnectPolicy, last_seen: Instant }`；pump（ServiceTick）对每
  client try_recv 全量排空 → Welcome/FrameReady → latest 更新；dead（EOF/协议
  错）→ 预算内重连（新 WsEnd + Hello 重订阅），预算尽 → 占位态 + 诊断文本。
- 身份防串：url 即身份；url 变化整体换 Client（迟到帧 只可能进旧 Client 对象，
  随 Drop 丢弃）。

#### T-01 复用面定案（2026-10-11 spike 收据，基线 a58a2142d）

**判定：自建迷你 host-client**（remote_frame_registry.rs 专属小模块），两既有面均
不直接复用，证据：

1. shell remote-mirror 族（remote.rs:30 RemoteMirror / attach_pending_remotes
   :82 / pump_remote_mirrors :99）是**宿主侧 WS 服务端**——WsListener 等远端浏览器
   连入、把宿主本地 broker 帧推出去，方向与 remote-frame（宿主作 WS **客户端**连
   bridge 消费远端帧）相反；可借鉴的是其泵形态（非阻塞 try_recv 全排空 +
   is_eof 摘除 + 版本键去重），照搬进注册表 pump。
2. ClientPump（client_runtime.rs:451）是 **app 侧**客户端——VM 状态原地重连
   （projector 暂存/shm 协商/L2Detach 生命周期），remote-frame 是宿主侧被动投影
   （无 projector、无 shm、帧不进 VM 状态），复用会拖入整个 FrameSource 状态机；
   只借 `ReconnectPolicy`（:427 budget_ms/interval_ms）语义为内联字段（struct 自带
   pipe 字段不适用，不 import）。
3. 自建件全部落既有积木：Transport=`transport::ws::connect(url, timeout_ms)`
   （transport.rs:740，401/升级拒绝终态直返）；渲染=`displaylist_element(&DisplayList)`
   （broker_surface.rs:575，纯视觉 canvas Program）+ `mouse_area` 包装
   （rqhost.rs:1339 先例）+ `.interaction(cursor_kind)`；输入=mouse_area 消息 →
   `InputMsg::Pointer{Pressed,Released,Moved}{wid, button, x, y, modifiers}` 直接
   `end.send(&ProtocolMsg::Input(..))`（rqhost.rs:1252-1308 同款收尾，含
   `wire_button` 映射先例）。

**WsEnd Drop 语义核证（修正计划原假设）**：transport.rs:594-610——Drop 发 shutdown
oneshot → 读线程 select! shutdown 臂退出 → **join 等待读线程结束**。比 §5.2 原假设
「读线程随收件箱关闭退出」更强：确定性线程回收，无泄漏路径。每 client 一个独立
tokio runtime（1 worker + reader 线程）——v1 元件数少，可接受。

**bridge 协议消费端形态**（rqhost.rs:1492 run_bridge 实读）：connect(url with
?token=) → 发 `Hello{app_name}` → 收 `Welcome{app_id, wid, rect:{0,0,w,h},
frame_mode:Commands}` → 收 `FrameReadyV2{payload: DisplayList}`（v1 退化
FrameReady）→ 上行 `Input(InputMsg)` 直投。Welcome.rect 与 DisplayList 逻辑域 1:1
（bridge 侧 rect = client.width/height 直填），无换算因子。

**ServiceTick 挂点**：renderer.rs ServiceTick 臂内（a58a2142d :23076
pump_broker_clients / :23109 pump_remote_mirrors 同拍），泵挂
`pump_remote_mirrors` 之后。

### 5.3 输入回传

- mouse_area on_press/move/release（+ canvas keydown 焦点位 v1 可选）：局部坐标
  （元件像素域）→ 远端逻辑域换算（Welcome.rect 为基，等比；与 web 消费面
  canvas_embed_web.ts 同一算式语义）→ `InputMsg::PointerPressed/Released{wid,
  x, y, button, modifiers}` / `CharTyped{ch}`（wid = Welcome 载荷）。
- inspect 门控不在本层（宿主应用以 `class` 显隐/或忽略输入自行组合；协议层无
  模式概念——如实记录）。

### 规范增量

| delta_id | add/modify/retire | docs/specs/... target | before/after rule | rationale | acceptance IDs |
|---|---|---|---|---|---|
| SD-01 | modify | docs/specs/auto-lang/ui/overview.md | 元件谱无远程帧消费面 → 新增 remote-frame 元件条目（url 契约/双轨产物形态/宿主数据叶不进 VM 状态/400ms 拍节奏约束/输入回传 InputMsg 语义/生命周期与重连预算） | J2 上游接缝的唯一权威沉淀位 | AC-01,02,03,04 |
| SD-02 | modify | docs/specs/auto-lang/ui/overview.md | 桌面协议消费面记录宿主侧元件化路径（bridge WS 宿主端复用面） | 与 PLAN-683 remote 渲染条目衔接 | AC-02,03 |

（musk 侧 spec 增量归 PLAN-102 SD-01/04 的既有行，不在本计划。）

## 6. 测试设计

- 单测：schema 声明解析 + element_coverage 双向栅栏绿；`View::RemoteFrame` 构造/
  `convert_view_messages` 臂；aura 提取（remote-frame 节点 props/events 直通）；
  vue 生成快照（div + class 透传）；注册表生命周期（url 切换计数归零清连、dead
  重连状态机、预算尽占位）——mock Transport（trait 注入，不启真 WS）。
- 集成（#[ignore] 实机腿，沿 PLAN-102 P6 探针形态）：`auto rqhost --pipe …
  --bridge-ws <port> --bridge-token <t>` + 计数器夹具（AUTO_VM_RENDER=remote）+
  .at 宿主 demo（一个 remote-frame 指向桥）→ 断言：首帧到达（latest 非空/截图
  md5 变化）、press 后远端计数递增（state 快照）、kill 桥进程 → 占位 → 重启 →
  帧恢复。
- 回归：`cargo test -p auto-lang`（lib + schema_drift + element_coverage）+ 442
  gap 语料门 + `auto build --gen-only --strict`（本仓 demo 工程）。

## 7. 验收标准

| ID | 可观察结果 | 验证与预期 |
|---|---|---|
| AC-01 | `.at` 内 remote-frame 双轨合法产物 | demo 工程 `auto build --gen-only --strict` 退出 0；vue 生成含 data-remote-frame div；schema_drift/coverage 栅栏绿 |
| AC-02 | VM 实机帧持续显示与断连恢复 | 实机腿：首帧 ≤10s 到达；kill 桥 → 占位态；重启 → 预算内帧恢复；全程无 panic |
| AC-03 | 输入回传到远端 | 实机腿 press/char 后夹具计数器/文本状态递增（typed 断言）；坐标换算误差 ≤2 逻辑 px（同夹具按钮命中） |
| AC-04 | 生命周期安全 | 单测：url 切换/卸载 → 旧连关闭（mock Transport 断言 close 调用）；e2e 后无残留线程/句柄（进程树 census 收窄） |
| AC-05 | 既有门零回归 | lib/栅栏/语料门全绿；`cargo test -p auto-lang` 退出 0 |
| AC-06 | PLAN-102 可消费交付 | 元件契约文档（SD-01 落 spec）+ J2 合同对齐注记；musk 集成验收由 PLAN-102 T-04 收口（引用非重复） |

## 8. 执行步骤

所有实现都在 `D:/autostack/.wt/lang-760/auto-lang`（分支 plan-760-dev）；每步
落产物、命令、退出码与 AC 关联。

- [x] **T-01 复用面定案 spike**（无依赖；AC-02/03 设计前置）：host 客户端
  复用判定——shell remote-mirror 客户端族 vs 自建迷你 host-client（§2.3）；
  WsEnd Drop 语义（读线程退出路径）核证；输出决策注记入本计划 §5.2（修订
  plan_revision）。
  [✅ 2026-10-11] 判定=自建迷你 host-client（§5.2 T-01 定案小节全文）：RemoteMirror
  是服务端方向相反不采；ClientPump 是 app 侧重状态机不采；积木=ws::connect +
  displaylist_element + mouse_area + InputMsg 直发（文件:行均核，基线 a58a2142d）。
  WsEnd Drop = shutdown oneshot + join 确定性回收（强于原假设）。
- [x] **T-02 元件声明 + View 变体 + 双臂转换**（依赖 T-01；AC-01/05）：schema/
  aura.at element 块 + element_coverage 登记 + view.rs View::RemoteFrame（含
  convert_view_messages 臂）+ aura_view_builder 双镜像臂 + 单测（解析/栅栏/往返）。
  [✅ 2026-10-11, 77a8f5ebd] aura.at element + aura/schema.rs insert +
  render_support TagSupport::full + element_coverage 四处登记（P3/P6-3 对齐）；
  View::RemoteFrame{url, style}（class 经 extract_style_with 统一入 Style——
  §5.1 的 class 字段由 Style 承载，语义等价实现调整）；aura_view_builder 双
  镜像分派 + convert_remote_frame；renderer/coverage/snapshot/vnode 7 处穷尽
  臂（含 convert_view_messages 透传）；ui_gen/rust.rs a2r codegen 臂。单测 3
  例全绿。schema_drift 双栅栏红集与 master 基线逐字一致（carousel/command
  族预存红 = REG-4/P695 在案债，零新增）。
- [x] **T-03 注册表 + 渲染/输入臂**（依赖 T-02；AC-02/03/04）：
  remote_frame_registry.rs（生命周期/重连）+ ServiceTick 泵挂点 + 渲染臂
  （displaylist_element + mouse_area 输入包装 → InputMsg 发送）+ mock 单测。
  [✅ 2026-10-11, 6c80f64ca + 0b61b9ad3] 注册表：租约法生命周期（2s ATTACH_LEASE
  ——§5.2 引用计数的等价稳健实现）/后台拨号线程（ws::connect 阻隔渲染线程）/
  30s-50ms 重连预算（ReconnectPolicy 语义内联）/EOF 清残帧。渲染臂 = 自建
  RemoteFramePainter canvas Program（draw=最新帧等比 fit+居中直绘/相位占位；
  update=指针事件→等比换算→InputMsg 直发）——mouse_area 无坐标回调、
  PointerArea 需消息返回，canvas update 臂即计划 §2.4 备选的正解。订阅名 =
  url query `app` 参数（缺省 remote-frame；计划 §5.1 未载——实现层定案，
  musk 集成传 &app=musk-canvas-embed 与 web 消费面 canvas_embed_web.ts
  appName 同源）。ServiceTick 泵挂 pump_remote_mirrors 后 + standalone
  （auto run -r vm）经 remote_frame_tick 条件拉起（注册表非空订/退场回落）。
  mock 单测 8 例（Arc 共享态 MockTransport）5 轮零 flake（全局 atomic 唯一
  ns + thread-local 键空间 + 族内串行锁防泵交叉）。
- [x] **T-04 web 生成臂**（依赖 T-02；AC-01）：vue.rs remote-frame → div 透传
  （data-remote-frame 标记）+ 生成快照单测。
  [✅ 2026-10-11, 6a85f0170] map_tag fallback 归一 + 属性发射专用臂
  （data-remote-frame="1" + data-remote-frame-url 字面量直发/动态绑定
  :data-remote-frame-url 双形态）；golden case test/a2vue/remote_frame 全绿；
  实机 demo strict gen 产物核对（App.vue 含 data-remote-frame div，双工程
  --gen-only --strict 均 exit 0）。
- [x] **T-05 实机 e2e 腿**（依赖 T-03 + P-1；AC-02/03/04）：rqhost bridge +
  计数器夹具 + demo 宿主——帧/输入/断连重连断言 + 进程 census；驱动脚本入库
  scripts/（不依赖机器残留）。
  [✅ 2026-10-11, 0b61b9ad3] scripts/remote_frame_e2e.py + examples/ui/
  remote-frame-e2e/{fixture,host}（宿主含本地按钮对照臂 + __RF_PORT__ 模板
  替换）。**11/11 PASS（三轮）**：bridge-up / fixture-mcp-up / fixture-adopted
  / host-mcp-up / first-frame（mirror subscribed + 夹具紫字特征像素，≤10s）/
  host-window-found（hwnd PID 对账 own=True）/ os-inject-control-arm（local
  +1——OS 注入→iced→VM 分层定位）/ input-closed-loop（夹具 count 0→1——
  canvas update→InputMsg→bridge→app 全链）/ dead-placeholder（特征像素归零）/
  reconnect-recovered（30s 预算内恢复）/ no-panic。方法论定案：输入注入 =
  PostMessage 直投客户区 WM_MOUSEMOVE/WM_LBUTTONDOWN/UP（mouse_event 全局
  注入 press 被输入系统丢弃——对照臂实证；winit 0.30.13 event_loop.rs:1781
  处理该组 legacy 消息）+ 前台化 + 光标同步；帧断言 = 特征像素法（md5 法
  在首帧早于首截图时失效）。e2e 暴露并修复三缺陷（standalone ServiceTick
  不驱动/重连收编死锁/EOF 残帧）——回归测试随附。
- [x] **T-06 spec 沉淀与收口**（依赖 T-02..05；AC-06/SD-01/02）：overview.md
  增量 + PLAN-102 对齐注记 + 全门回归清单收据，交 `/auto-plan:review`。
  [✅ 2026-10-11] overview.md 新「现状（2026-10-11）——remote-frame」节
  （SD-01/02 全量：元件契约含 app 订阅名 query 约定/双轨产物/租约生命周期/
  400ms 帧节奏约束/输入回传算式/验收面）。PLAN-102 对齐 = 该节「musk 侧集成
  归 PLAN-102 T-04 收口」注记 + url query app=musk-canvas-embed 订阅名契约
  即 J2 合同面。门禁收据见 §9 执行记录。

## 9. 复审记录

2026-10-11 起草交接：`stage: new`，`plan: PLAN-760`，`revision: 1`，`outcome:
pass`（合同完整性自检；代码面五点已按 master f9d647bc 逐处核实），`next: work`
（T-01 spike 起步；P-1 前置在 T-05 前定案）。AC-01..06 与 SD-01/02 均有任务覆
盖；musk 侧消费面（PLAN-102 T-04 收口）明确不在本计划重复验收。

2026-10-11 work 收口：`stage: work | PLAN-760 | rev 2（T-01 定案注记 + 任务
回填）| outcome: pass | code: plan-760-dev @ aca5a5bba（merge master 005118d48
后全绿收口；演进链 77a8f5ebd → 6c80f64ca → 6a85f0170 → 0b61b9ad3 → aca5a5bba，
基线 a58a2142d = dd5457fe1 + P-1 merge）| tasks: T-01..T-06 全勾 | evidence:
①单测：注册表 8 例（5 轮零 flake）+ 双路径分派 + schema 登记 + a2vue golden
全绿；②栅栏：schema_drift 双 fence 全绿（merge PLAN-759 批次消除基线 carousel
族后,本计划增量零新增红——P0 fence 曾暴露 schema.rs 多行 insert 不被扫描
（P759-D2 在案债务）与派发臂别名孤儿,已单行化 + canonical 收敛修正）；
③e2e：11/11 PASS 三轮（scripts/remote_frame_e2e.py,AC-02/03/04 全链——首帧/
输入闭环/断连占位/重连恢复/no-panic）；④门禁：裸 cargo t 全量 **5243/5243
passed 0 failed**（99.9s,--no-fail-fast;含 merge 前基线的 10 预存红全部随
PLAN-759 合流消除——musk_vm_track_p053_4/desktop_surface_merge/plan606/
plan502/p748×2/projector/ash×2/双 fence 均为基线旧形态,master 侧已修）;
demo 双工程 auto build --gen-only --strict exit 0（AC-01）;文档同步 = core.md
再生成 + DOC_EXCLUDE 登记（windowthumbnail 先例）+ auto-os kitchen-sink 53→54
（单文件提交 0532b8f）| blockers: 无 | next: review`。

实现层偏差登记（相对合同 §2/§5，语义等价或增强）：
1. View::RemoteFrame 字段 {url, style}（无独立 class——extract_style_with 统
   一承载，window_thumbnail 先例）。
2. 输入包装 = canvas Program::update 臂（§2.4 备选路径的正解——mouse_area
   on_press 无坐标、PointerArea 需消息返回）。
3. 渲染 = 自建 RemoteFramePainter（displaylist_element 纯视觉无输入面，
   painter 委托 paint_ops_v2 + fit/居中 = 画面任意 bounds 尺寸不裁形增强）。
4. 订阅名（Hello.app_name）= url query `app` 参数（合同未载的必要契约——
   bridge 按 app_name 路由订阅；SD-01 已沉淀）。
5. 生命周期 = 租约法（2s note 续租）而非逐帧计数（§5.2 引用计数的等价稳健
   实现——避免计数泄漏/竞态）。
6. standalone（auto run -r vm）ServiceTick 拉起（合同假设泵恒驱动——实际
   仅 desktop 模式订阅；remote_frame_tick 条件补齐，e2e 实测暴露）。

## 10. 待澄清事项

1. P-1 执行序（§4.4）：auto-musk-dev-102 三提交合回本仓 master 的时机——推荐
   T-05 前先行合回（PLAN-102 P5/P6 收据已构成消费验证）；若 musk-102 会话先行
   合回则本计划基线更新哈希即可。
2. ServiceTick 400ms 帧节奏是否满足 musk AC-01「持续更新」的实机观感——按
   §2 约束执行期实测定案；不可接受则另立提频项（不阻塞本计划 AC）。
3. v1 不含元件级业务事件（onframe/ondead 上抛）——musk 消费面现不需要
   （canvas_embed_web 同族形态）；需要时另增（记 SD-01 增量）。
