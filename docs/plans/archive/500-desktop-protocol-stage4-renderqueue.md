---
plan_id: PLAN-500
status: archived               # drafting → executing → execution_done → reviewed → archived（终态）
feature_name: desktop-protocol-stage4-renderqueue
author: [zhaopuming]
created_at: 2026-08-31
updated_at: 2026-08-31

# /auto-plan:review 结束时填写：
supersedes_spec_components:
  - "桌面协议 v1.2 宿主本地 dynamic_view 重渲染消费段: 退役为非 broker 窗路径（broker 表面窗改 child 帧两态合成，兑现 host.rs 'live-iced 消费留 Stage 2' 遗留点）"
  - "docs/design/autoui/desktop-protocol-v1.md §1.3 草案: 转正定稿（D1–D4 定案 + §1.3.1 爬坡目标集回写）"
new_spec_components:
  - "desktop_protocol/message.rs: FrameReadyPixels(tag 8) + Welcome.frame_mode 尾部协商位——帧载荷二态（协议 v1.3，PROTOCOL_VERSION 仍 1）"
  - "desktop_protocol/coverage.rs: Coverage 能力表 + scan_view/judge 覆盖探测 + RenderMode 三态裁决链（spawn > desktop_render: > auto）+ effective_frame_mode 降级观测"
  - "desktop_protocol/pixels.rs: PixelsChild 泵（自带 iced 运行时 + 隐藏窗 + window::screenshot → shm RGBA → FrameReadyPixels）"
  - "desktop_protocol/stage3.rs + ui/iced/broker_surface.rs: 宿主两态合成（queue=DrawList canvas 降级 / independent=image::Handle::from_rgba 上传）"
  - "auto-man pac.rs: `desktop_render:` manifest 字段（与 Plan 276 render: 正交改名）+ auto spawn `--autodesk-render=` 三态开关"
  - "test/parity/001_queue.expected.txt + test/a2vue/p500_001_helloworld/: 三臂 parity 首条金样（queue/vue 自动档 + iced 实机档）"
touched_goals:
  - "GOAL-009: 虚拟桌面桌面协议 Stage 4——attach 态渲染两路并存（queue/independent）+ per-App 三态开关与覆盖探测降级"
  - "GOAL-007: AutoUI 跨端视觉一致——三臂 parity 对拍纪律（iced/vue/queue）首条金样入档"

affects: [auto-lang/ui, auto-man]
current_step: 10
total_steps: 10
---

# [PLAN-500] 桌面协议 Stage 4——RenderQueue 并行渲染模式（v1.3）

## 变更摘要

承接设计文档 `docs/design/autoui/desktop-protocol-v1.md` **§1.3 v1.3 草案**（本
计划 T1 转正）：让"app 自带 iced/wgpu 独立渲染"与"app 免 GPU 上下文、宿主侧
栅格化（RenderQueue）"两条路径**在同一桌面内并存**，per-App 启动时三态选择
（`auto/queue/independent`）。现状底座：帧通道 v1.0 起即 commands 载荷
（`DrawList{clear, ops:[Quad|Text]}`），480 已落 `AppProjector`（AuraNode→
DrawList，text/button + 线性堆叠）与多 App 驻留宿主；缺的是：attach 态 app
**自渲染像素帧路径**（Pixels）、**三态开关与覆盖度探测**、投影器与栅格化的
**产能化爬坡**。既有两条渲染路径（进程内 iced 直挂 / Standalone 独立窗）
**零改动**——I1 不变式延续。

四块交付：

1. **帧载荷二态**：`FramePayload ::= Commands(DrawList) | Pixels{shm, w, h,
   stride, format}`（追加式，遵守 §1 演进纪律）。
2. **三态开关**：pac.at manifest `render:` 字段（auto-man 侧）+ spawn 参数
   覆盖 + `adjudicate()` 裁决链；`auto` 按覆盖度探测降级 `independent`。
3. **产能爬坡**：AppProjector 从 text/button 爬到 **001–005 示例 widget 子集**
   （含布局引擎复用 `ui/layout`）；宿主栅格化器 demo→生产（抗锯齿/damage）。
4. **三臂 parity 首条金样**（iced 直挂 / vue / queue 同源，I4' 纪律）。

## 目标

- **G1 载荷二态**：`independent` 模式 app 侧离屏 iced 渲染 → shm 像素帧 →
  宿主纹理上传合成；`queue` 模式 DrawList 命令帧 → 宿主栅格化合成；两模式
  同一宿主同屏并存（一 App 各一）。
- **G2 三态开关**：裁决链 spawn 参数 > pac.at `render:` > `auto`（探测降级
  + 观测 Log 一行留痕）；裁决挂既有 `adjudicate()`/`cmd_autodesk` 机制。
- **G3 子集端到端**：001–005 五示例在 `queue` 模式 attach 到桌面渲染运行
  （点击/输入 handler 闭环，行为与直挂模式一致）。
- **G4 覆盖表**：AppProjector 能力表（widget kind × prop × 布局）+ App 视图
  装载期扫描 → 可行性判定；未覆盖项显式 not-yet，**禁止静默错绘**。
- **G5 parity 首条**：一个示例（001-helloworld）三臂同源金样（iced/vue/queue
  输出对拍基线），门禁挂金样体系。
- **非目标**：全 widget 族覆盖（Stage 5）；web/远程端 command 流消费
  （Stage 6）；默认模式翻转；进程内 iced 直挂路径与 Standalone 路径任何改动。

## 架构方案

```
per-App 启动裁决：spawn 参数 > pac.at render: > auto（覆盖探测）
   ├─ queue：AuraNode ─AppProjector(爬坡)→ DrawList ─shm→ 宿主栅格化→ 合成
   └─ independent：iced/wgpu 离屏 ─shm 像素帧→ 宿主纹理上传→ 合成
（进程内直挂 / Standalone 两路径不动）
```

- **模块落点**（`crates/auto-lang/src/ui/desktop_protocol/`）：
  `message.rs`（载荷变体追加）、`shm.rs`（像素帧槽）、`client_runtime.rs`
  （AppProjector 爬坡 + Pixels 泵 + 覆盖探测）、`stage3.rs`/宿主合成段
  （两态栅格化/上传）、`broker.rs`/`dual_mode.rs`（裁决链）；
  `crates/auto/src/cmd_autodesk.rs`（spawn 参数）；`crates/auto-man/src/pac.rs`
  + 模板 pac.at（manifest 字段）。
- **D1–D4 深水决策**（设计 §1.3 已登记，T1 定案）：Text op 形态（倾向
  glyph run 已定位下发）、布局引擎归属（倾向复用 `ui/layout`）、输入命中
  细化（交互区表上报）、IME 光标/popover 坐标下发。

## 技术栈

既有协议栈（message/codec/shm/transport）+ iced 离屏渲染 + `ui/layout`。
零新三方依赖。

## 需求分析与背景调查

（取材 docs/specs/overview.md §ui + 设计文档 §1.1–1.3 + 现场核验 2026-08-31）

- **设计依据**：`desktop-protocol-v1.md` §1.3 v1.3 草案（2026-08-31 已提交，
  本计划 T1 转正 + D1–D4 定案）。
- **底座现状**（§1.1/§1.2 已落）：五通道协议 + 命名管道/shm 传输 +
  `adjudicate()` 三步裁决 + broker 多 App 驻留宿主（480：N=3/5 压测、
  4.81MiB/App）+ `AppProjector` v1（text/button + 线性堆叠 + button 命中区
  推导 + FStr 插值）+ L1/L2/L3 形态迁移。**帧载荷现状 = DrawList 单态**。
- **I1 不变式**：386 复活时 WM/会话/事件路由零改动——本期两新路径均为
  R4 接缝后的渲染叶变体，接缝前代码零触碰。
- **排程**：497（S3）在途（shell.at/switcher.at/renderer 消费段）；498/499
  （charts 线）drafting。本期改动面 = desktop_protocol 模块 + auto-man pac +
  cmd_autodesk——与三者**零交叠**，可并行。

## 详细设计

### 1. 帧载荷二态（message.rs / shm.rs）

- `FrameMsg` 追加 `FrameReadyPixels{wid, frame_id, slot, damage, revision,
  w, h, stride, format}`（tag 顺延，追加式）；shm 槽复用既有双槽翻面/
  FrameAck 语义（载荷解释随 Welcome 协商的模式位而定）。
- `HandshakeMsg::Welcome` 追加 `frame_mode: Commands|Pixels`（尾部字段，
  旧端缺省 = Commands，向后兼容）。

### 2. Pixels 路径（client_runtime.rs + 宿主）

- app 侧：离屏 iced 运行时（headless wgpu surface，复用 `ui/headless` 地基）
  逐帧栅格化 → RGBA 写 shm 槽 → `FrameReadyPixels`；
- 宿主侧：shm → 纹理上传（既有合成位图路径同型）→ 虚拟窗合成。

### 3. 三态开关与覆盖探测

- `crates/auto-man/src/pac.rs` manifest 增 `render` 字段（枚举，缺省 auto）；
  模板 pac.at 同步；
- `crates/auto/src/cmd_autodesk.rs` spawn 参数 `--render=<mode>`（覆盖
  manifest）；
- `adjudicate()` 裁决链末端接 mode：`auto` → 装载期扫描 App 视图 widget
  清单 vs `AppProjector` 能力表 → 可行走 queue、不可行降级 independent +
  `Log` 观测一行。

### 4. 投影器爬坡（client_runtime.rs）

- 能力表结构：`Coverage { kinds: Set<WidgetKind>, props: Map<Kind, Set<Prop>>,
  layouts: Set<LayoutMode> }`；
- T1 扫描 001–005 实际 widget/布局清单定爬坡目标集（预期：input/checkbox/
  image/card/column-row-gap 等，以扫描结果为准）；布局复用 `ui/layout`
  （D2 定案后）。

### 5. 宿主栅格化产能化（stage3.rs / 合成段）

- Quad/Text 抗锯齿栅格化（既有 demo 级 → 生产路径）；`damage` 局部重绘；
  双槽语义不变。

### 6. parity 金样（I4'）

- 001-helloworld 三臂基线：iced 直挂截图 / vue 渲染 / queue 模式栅格化输出
  ——像素容差对拍；挂金样体系（a2vue 同族登记）。

## 测试设计

1. **T1 协议单测**：新变体 round-trip + golden bytes（追加式兼容：旧 codec
   解码新 tag 的拒收/缺省行为）——既有协议测试套同型扩展。
2. **T2 覆盖探测单测**：能力表 vs 视图清单判定（覆盖/不覆盖/降级）。
3. **T3 re-exec 集成**（480 形态）：001–005 五示例 queue 模式 spawn →
   Active → 输入注入 handler 闭环 → 帧递增；一示例 independent 模式像素帧
   合成；**同宿主双模式并存**（一 queue 一 independent）。
4. **T4 parity 金样**：001 三臂对拍绿。
5. **T5 实机**：`auto run` 双模入口三态各跑一轮；auto 降级日志可见；
   001–005 queue 模式桌面内交互。

## 验收标准

1. T1–T4 绿；T5 实机清单 PASS 留痕。
2. 001–005 在 queue 模式渲染正确（与直挂行为一致，截图对拍）且输入闭环。
3. 双模式同宿主同屏并存（T3 用例）。
4. `PROTOCOL_VERSION` 仍为 1（追加式演进纪律核验）；既有直挂/Standalone
   路径零改动（diff 核验）；`cargo t` 默认档不回归；`cargo check` 零警告。
5. 设计文档 §1.3 草案转正（T1 定案 D1–D4 回写）。

## 执行步骤

（原子任务：精确文件路径 + 确切操作 + 验证命令；每步完成后追加 [✅ 已完成] 一行证据）

1. **T1 设计转正**：D1–D4 定案回写 `docs/design/autoui/desktop-protocol-v1.md`
   §1.3（草案→定稿）；扫描 001–005 widget/布局清单定爬坡目标集并回写。
   验证：设计文档 diff + 清单成文。
   [✅ 已完成] §1.3 转正（版本表 v1.3 行 + §1.3 主体 + §1.3.1 爬坡目标集
   清单表 + §1.3.2 D1–D4 定案表 + pixels 格式定案）；D1=A（宿主侧 shaping，
   草案倾向 B 推翻——DrawOp::Text v1.0 冻结口径 + 宿主零新依赖）、D2=
   ui/style::BoxLayout（ui/layout 实为窗位引擎，草案前提修正）、D4=形态
   定案不落线（无消费面不占 tag 位）；Pixels 泵按 497 T1 结论修正为
   自带 iced 运行时+隐藏窗+screenshot 通道。
2. **协议载荷二态**：`crates/auto-lang/src/ui/desktop_protocol/message.rs`
   追加 `FrameReadyPixels` + `Welcome.frame_mode`（尾部）+ codec 扩展 +
   round-trip/golden bytes 单测。
   验证：`cargo t desktop_protocol`。
   [✅ 已完成] `FrameReadyPixels`（tag 8：wid/frame_id/slot/damage/revision/
   w/h/stride/format）+ `FrameMode`/`PixelFormat` 线格式 + `Welcome` 尾部
   `frame_mode`（旧线无尾字节 → 解码缺省 Commands，`Reader::remaining()` 新
   API 做判据）+ 端点 `activate()` 签名扩展（宿主/AppEndpoint 记录模式位）；
   单测：round-trip ×2 模式、旧线 Welcome 兼容、未知 tag 拒收、golden
   bytes（40B 载荷锚点）。`cargo t desktop_protocol --features ui-iced`
   58/58 绿。
3. **覆盖探测**：`client_runtime.rs` Coverage 能力表 + 视图扫描判定 +
   T2 单测。
   验证：`cargo t desktop_protocol`。
   [✅ 已完成] 落点为新文件 `desktop_protocol/coverage.rs`（模块内聚，
   client_runtime 不掺入纯函数）：`Coverage` 能力表（§1.3.1 目标集：
   kinds/props/events/layouts/style_prefixes 前缀规则）+ `scan_view`
   装载期扫描（标签不预分类，judge 时按能力表定 widget/布局）+ `judge`
   → `Verdict{Covered, NotCovered(缺项清单)}`。T2：001–005 **真源文件**
   扫描全 Covered（覆盖表与实例清单一致性钉）+ 未覆盖 widget/布局/
   带参 handler/样式类逐项显式列举。`cargo t desktop_protocol
   --features ui-iced` 64/64 绿。
4. **Pixels 泵**：`client_runtime.rs` 离屏 iced（`ui/headless` 地基）逐帧
   栅格化 → shm 写槽 → `FrameReadyPixels`。
   验证：`cargo t desktop_protocol`（单 client 帧递增）。
   [✅ 已完成] 落点为新文件 `desktop_protocol/pixels.rs`（T1 定案修正：
   路径 = 自带 iced 运行时 + 隐藏窗 + `window::screenshot` 整窗抓取——
   497 T1 已证 headless/overlay 不可行）：`PixelsChild` 桥（端点状态机 +
   shm 开段 + 截图泵去重 + 输入边界 v1=触发截图无 handler 派发）+
   `AppEndpoint::produce_frame_pixels`（槽纪律/frame_id 与命令帧同源）+
   iced 宿主接线（`DesktopOptions.pixels` 隐藏窗位、boot 取桥、协议轮询
   订阅、PixelsProtocol/PixelsShot update 臂——复用 run_session 单管线，
   既有 Standalone 行为零变化）+ `run_independent_child` 入口。单测：
   合成帧泵全循环（握手→开段→帧×3：frame_id 单调/槽轮转/宿主读槽字节
   /revision 不动/输入触发截图）。`cargo t desktop_protocol` 65/65、
   `cargo t ui` 1566/1566 绿。
5. **宿主两态合成**：`stage3.rs`/宿主合成段 Pixels 上传路径 + 栅格化器
   抗锯齿/damage 产能化。
   验证：`cargo t desktop_protocol && cargo t ui`。
   [✅ 已完成] 端点：`HostAction::ComposeFramePixels`（Active 臂收
   FrameReadyPixels）；`stage3.rs`：`BrokerClient` 像素前缓冲 +
   `compose_pixels`（shm 槽 RGBA → front + FrameAck 回带）；
   `session.rs`：attach 期模式位定档（shm 槽尺寸 Commands=16KiB 既有 /
   Pixels=像素上限；`Welcome.frame_mode` 透传）。渲染臂
   `ui/iced/broker_surface.rs`：queue 臂 DrawList → canvas 降级（Quad=
   抗锯齿 fill、Text=fill_text 宿主侧 shaping——D1=A 落地；damage v1.3
   作重绘提示全帧重建）；independent 臂 RGBA → `image::Handle::
   from_rgba` 上传（497 快照同通道）。虚拟窗层循环：broker 表面窗客户
   区接 child 帧两态合成（v1.2 的宿主本地重渲染退役为非 broker 窗路径
   ——兑现 host.rs 头注"live-iced 消费留 Stage 2"遗留点）。单测
   `broker_pixels_compose_front_buffer`（元数据+槽字节+ack 全链）。
   `cargo t desktop_protocol` 66/66、`cargo t ui` 1567/1567 绿。
6. **三态开关**：`crates/auto-man/src/pac.rs` + 模板 pac.at `render` 字段 +
   `crates/auto/src/cmd_autodesk.rs` `--render` 参数 + `adjudicate()` 裁决链
   （auto→探测降级+Log）。
   验证：`cargo t -p auto-man pac && cargo t desktop_protocol`。
   [✅ 已完成] **执行定案**：pac.at 字段名 `desktop_render:`（草案措辞
   "`render:` 字段"与 Plan 276 既有前端后端字段撞名，语义正交不可复用，
   执行期发现并改名——pac.rs 字段注释留档）。裁决链 = `RenderMode::
   resolve`（spawn `--render=` > pac.at `desktop_render:` > auto）+
   `effective_frame_mode`（auto → 装载期覆盖探测：Covered→queue /
   NotCovered→降级 independent + 缺项清单观测行）；模式位跨线走**孵化
   记录第三字段**（`incubate␟<name>␟queue|pixels|pixels:auto`——
   DesktopBus 文本记录追加，不占冻结二进制协议 tag 位）；宿主 attach
   据此定档 shm 槽尺寸 + Welcome 模式位 + `pixels:auto` 降级观测行
   （ui_console 面板 + 控制台双落）；与 `adjudicate()` 进程形态链正交
   （Client/Broker/Standalone × queue/independent 两维）。单测：裁决链
   优先级/宽容、探测降级、pac 解析正交性。`cargo t -p auto-man pac`
   106/106、`cargo t desktop_protocol` 68/68 绿。
7. **投影器爬坡**：`client_runtime.rs` 按 T1 清单扩 AppProjector（widget 臂
   + `ui/layout` 复用，D2 落地）。
   验证：`cargo t desktop_protocol`（爬坡逐项单测）。
   [✅ 已完成] AppProjector 重写为块流布局引擎：`NodeStyle`（`ui/style::
   BoxLayout` 盒模参数源——D2 定案落地（`ui/layout` 为窗位引擎不适用，
   见待澄清 T1 定案）；色/字号/对齐扩展走 `Color::from_tailwind` +
   theme 双盘）。widget 臂：text（`Dot(self,field)` 状态绑定 + text-4xl
   字号档 + selectable 容错）、input（value 绑定 + placeholder + 聚焦 +
   CharTyped/VK_BACK 保型写入（Double 字段不漂 Str）+ 零参 oninput 派发
   = 输入闭环）、image（样式尺寸占位 Quad——保真边界：位图归 Stage 5，
   Coverage 表随注非静默）、a（accent 文本）、button（v1 既有 + 样式底）。
   布局：col/row/center + bg Quad + border 四边 + padding/gap/max-w/
   margin；`if` 条件块**展开取枝**（`.field ==/!= 字面量` + 裸真值）。
   命中区表泛化 `hit_regions`（Button/Input 两类，D3 落地；v1 `buttons`
   口径保留）。爬坡单测 ×5：001–005 **真源**逐项（001 样式档 / 002 row
   横排 / 003 卡片容器+双输入+换算联动 / 004 渐变头+image 占位 / 005
   if 块 + msg 提交错误显示）。`cargo t desktop_protocol` 73/73、
   `cargo t ui` 1574/1574 绿。
8. **re-exec 集成 T3**：001–005 queue 模式端到端 + independent 模式 + 双模
   并存用例（480 re-exec 形态）。
   验证：`cargo t desktop_protocol`（集成档，feature 门控沿既有）。
   [✅ 已完成] `t3_examples_queue_end_to_end`：五真源 App 同宿主 re-exec
   端到端——全孵化 Active → 逐示例交互闭环（本地**孪生投影器**出命中
   坐标，同引擎同源确定性布局）：002 点 "+"→Counter: 1、003 聚焦
   celsius 输入 100→联动 212、005 输入 email+提交→"Password is required"
   经 if 块显示；001/004 帧文本到位。`t3_independent_pixels_and_dual_mode`：
   双模并存（queue 臂测试二进制 re-exec + independent 臂**真 `auto run`
   生产二进制** `--autodesk-render=independent`——winit Windows 事件循环
   主线程约束的执行定案，兼得三态 spawn 参数全真链路）——两臂帧同达
   宿主（DrawList 文本 / 像素前缓冲）。**顺带真缺陷修复**：SurfaceStore
   表面句柄改进程级全局计数（per-instance 自增跨 client 撞 shm 段名
   `autodesk-shm-<pid>-<surface>`——Windows 同名=打开既有段；480 压测
   五同源 App 掩蔽，异源 App 直接串段）。执行定案：spawn 参数名
   `--autodesk-render=`（CLI `run` 具名 `--render` clap 先吞——撞名第二
   处）；T3 表面 480×900（005 内容高 ~812px 溢出 320 高表面）。
   `cargo t desktop_protocol` 76/76、`cargo t ui` 1577/1577 绿。
9. **parity 金样 T4**：001 三臂对拍基线入金样体系。
   验证：金样套件绿（随 `cargo t vue`/金样档）。
   [✅ 已完成] 三臂基线落两自动档 + 一实机档：**queue 臂** =
   `test/parity/001_queue.expected.txt`（001 真源投影 DrawList 金样：
   clear 暗盘 + Text 36px text-primary——`parity_001_queue_golden`，
   `cargo t desktop_protocol` 档）；**vue 臂** = a2vue 同族登记
   `test/a2vue/p500_001_helloworld/`（真源 SFC 金样，挂
   `test_a2vue_counter`）；**iced 像素臂** = 实机档注记（headless 栅格化
   不可行——497 T1 结论，截图对拍走实机/MCP 管线）。`cargo t vue`
   266/266、`cargo t desktop_protocol` 77/77 绿。
10. **实机冒烟 + 收尾**：T5 三态各一轮 + 降级日志目检；健康检查；状态翻
    execution_done。
    验证：`cargo check -p auto-lang && cargo t ui`。
    [✅ 已完成] 实机形态（真桌面窗口宿主）：`ui_desktop --apps-dir <临时
    注册表>`（6 条目：001–005 + 099-downgrade checkbox 降级钉）+ 三态
    child 各一（`--autodesk-render=queue/independent` + auto）同宿主
    驻留——**宿主降级观测行可见**：`[autodesk-broker] [render]
    099-downgrade: auto -> independent (coverage downgrade)`；三 child 全
    alive（attached）。顺带：`auto run --desktop` 为 vue 脚手架桌面
    （465），VM/iced 桌面宿主 = `ui_desktop` 示例二进制——实机结论注记。
    冒烟暴露并修复：ui_desktop 示例 `DesktopOptions` 初始化补
    `..Default::default()`。健康检查：`cargo check -p auto-lang
    --features ui-iced` 零错误、`cargo t ui` 1578/1578 绿（T3/T4 计入）。
    状态翻 `execution_done`。
   [✅ 已完成]（终检补录）全部 10 步 [✅] 齐备；复审档证据链：T1 设计
   转正（D1–D4 定案+爬坡清单）→ T2 协议二态 round-trip/golden →
   覆盖探测真源钉 → Pixels 泵合成帧全循环 → 宿主两态合成 → 三态裁决
   链 → 投影器爬坡 001–005 → T3 re-exec 端到端/双模 → T4 金样首条 →
   T5 实机三态+降级日志。

## 复审记录

- **复审人/时间**：ZCode（GLM-5.3-Flash），2026-08-31。复审基线 = worktree
  `plan-500-dev` @ `c7e3e0fe4`（执行 11 提交 + 复审修复 1 提交）。
- **门禁事实（复审纠正项）**：`cargo tf`（3316/3316 绿，含 1M churn 大档）
  **不带 ui-iced**，而 desktop_protocol 整模块 `cfg(ui-iced)`——tf 档实际
  不覆盖本计划交付面；故本计划的套件在复审中显式补跑：
  `cargo t desktop_protocol --features ui-iced` **77/77**（T1 round-trip/
  golden/旧线兼容、T2 coverage、T3 `t3_examples_queue_end_to_end` +
  `t3_independent_pixels_and_dual_mode`、T4 `parity_001_queue_golden`
  逐条显式确认 PASS）；auto-man `pac::` **15/15**。
- **逐条验收**：
  1. T1–T4 绿 + T5 留痕 → **PASS**。T1 设计转正 diff 核验（§1.3 版本表 +
     主体 + §1.3.1 清单 + §1.3.2 D1–D4 定案 + pixels 格式）；T5 实机留痕
     （宿主降级观测行 `099-downgrade: auto -> independent` + 三 child
     attached）+ `cargo check --example ui_desktop`（ui-iced）通过。
     ⚠ 记账更正：步骤 6 声明的 "`cargo t -p auto-man pac` 106/106" 系
     filter "pac" 误匹配 auto-lang `test_s**pac**er` 等用例的**误读计数**
     （`cargo t` 别名硬编码 `-p auto-lang`，`-p auto-man` 不生效）——实际
     pac 模块 15 例全绿；裁决链单测实位于 coverage.rs
     `effective_mode_probe_and_downgrade`，随 77 计入。
  2. 001–005 queue 渲染正确 + 输入闭环 → **PASS**（T3 断言：002 点击
     Counter:1、003 100→212 联动、005 if 块错误显示、001/004 帧文本）。
  3. 双模同宿主并存 → **PASS**（T3 双臂：queue 测试二进制 re-exec + 真
     `auto run --autodesk-render=independent` 生产二进制）。
  4. PROTOCOL_VERSION=1 ✓（mod.rs:61）；直挂/Standalone 零改动 ✓（diff
     核验：renderer/session 改动全为 `opts.pixels`（缺省 false）与
     `broker_client_content` 门控的追加臂，既有臂逻辑未改）；`cargo tf`
     默认档不回归 ✓；cargo check 警告 ✓——**复审初查分支净增 1 条**
     stage3.rs:21 unused `FrameMode` import（327/1312 裸用全在 cfg(test)），
     随复审修复清零，警告集与 master **逐文件全等**（242/242）。
  5. 设计文档 §1.3 转正 ✓——⚠ 复审发现：待澄清"§1.3 表已随步骤 6 提交
     同步措辞"**实未同步**（表内仍 `render:`/`--render=`），复审补齐
     （`c7e3e0fe4`）。
- **遗漏/延后/workaround 扫描**：新增行无 TODO/FIXME/HACK/dbg!；cmd_autodesk
  的 println/eprintln 为 `[autodesk-*]` 设计内观测行（控制台双落），非调试
  残留。延后均经批准且显式登记：D4 IME（Stage 5）、image 位图保真=占位
  （Coverage 显式非静默，Stage 5）、damage 作重绘提示全帧重建、pixels 臂
  输入 v1=触发截图无 handler 派发——v1.3 边界非静默缩水。执行定案三处
  （`desktop_render:` 改名、`--autodesk-render=` 改名、Pixels 泵按 497
  screenshot 结论修正路径）均已回写计划；其一的设计文档同步由复审补齐。
  顺带真缺陷修复（SurfaceStore 句柄进程级计数防异源串段）正确带单测。
  rustfmt 本仓无强制约束（未触碰文件差异更大），不计门禁。
- **结论**：全部验收标准 PASS，无未批准延后/阻断性债务 → **status:
  reviewed**，可进 `/auto-plan:merge`。

## 待澄清事项

- **D1 Text op 形态**（T1 定案 2026-08-31）：**定案 A = 字符串+样式，宿主侧
  shaping**（草案倾向 B 推翻）。依据：DrawOp::Text 线格式 v1.0 冻结即"shaping
  留宿主"（413 §7 同款）；宿主 ui-iced 已带 iced 文本栈（canvas fill_text），
  零新依赖；B 需 glyph run op + 字体图集协议且把 shaping 引擎塞进 queue 臂
  child。命中坐标同源性靠 widget 级矩形（非 glyph 级）保障。
- **D2 布局引擎**（T1 定案 2026-08-31）：**定案 = projector 自带轻量块/行流
  布局，参数源复用 `ui/style::BoxLayout`**。草案"复用 `ui/layout`"前提有误
  ——`ui/layout` 实为桌面窗位引擎（Free/Grid/MasterStack），非 widget 布局；
  步骤 7 的"`ui/layout` 复用"措辞随本定案按 `ui/style` 执行。
- **D3/D4**（T1 定案 2026-08-31）：D3 = widget 交互区表上报
  （`Vec<(WRect, kind, action)>`，button/input/a 三类，input 臂聚焦 + 文本
  路由）；D4 = 形态定案 `ControlMsg::ImeCursor{wid,rect}`（tag 12）但 v1.3
  **不落线**——001–005 验收口径为 CharTyped 闭环，IME 归 Stage 5。
- **pixels 格式**（T1 定案 2026-08-31）：v1 仅 RGBA8 **straight（非预乘）**、
  stride=w×4、format 仅 1=RGBA8；宿主经 `image::Handle::from_rgba` 上传
  （497 快照同口径），预乘换算在 iced 渲染器内部与 494 预乘 swapchain 衔接，
  协议层不感知。
- **性能预算**：queue 帧编码吞吐与 independent shm 带宽——T3 集成顺带采样
  记录（不设门槛，480 内存基线先例同型）。
- **（执行定案 2026-08-31）pac.at 字段撞名**：草案/设计 §1.3 的 manifest
  "`render:` 字段"与 Plan 276 既有 `render:`（前端后端 vue/rust/arkts）
  撞名且语义正交——执行定案改用 **`desktop_render:`**（pac.rs 字段 +
  模板注释位 + `--render` spawn 参数不变）；设计文档 §1.3 表已随步骤 6
  提交同步措辞（复审核实原同步**未落**，已随 `c7e3e0fe4` 补齐）。
- ~~与 497 的 shell.at 消费无耦合~~（已复核，2026-08-31）：497 已合入，其 T1
  结论（headless/overlay 子树栅格化不可行；`window::screenshot` 为唯一公开
  栅格化通道）**修正了本计划 Pixels 泵路径**——independent 臂 child = 自带
  iced 运行时 + 隐藏窗 + 整窗 screenshot 抓取（替代草案"headless wgpu
  surface"）；快照（TTL 缓存缩略）与 Pixels 帧（实时合成）消费面不同窗类，
  无协同冲突。
