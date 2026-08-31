---
plan_id: PLAN-500
status: drafting               # drafting → executing → execution_done → reviewed → archived
feature_name: desktop-protocol-stage4-renderqueue
author: [zhaopuming]
created_at: 2026-08-31
updated_at: 2026-08-31

# /auto-plan:review 结束时填写：
supersedes_spec_components: []
new_spec_components: []
touched_goals: []             # 引用 docs/specs/goals.md 的 GOAL-NNN

affects: [auto-lang/ui, auto-man]
current_step: 0
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
2. **协议载荷二态**：`crates/auto-lang/src/ui/desktop_protocol/message.rs`
   追加 `FrameReadyPixels` + `Welcome.frame_mode`（尾部）+ codec 扩展 +
   round-trip/golden bytes 单测。
   验证：`cargo t desktop_protocol`。
3. **覆盖探测**：`client_runtime.rs` Coverage 能力表 + 视图扫描判定 +
   T2 单测。
   验证：`cargo t desktop_protocol`。
4. **Pixels 泵**：`client_runtime.rs` 离屏 iced（`ui/headless` 地基）逐帧
   栅格化 → shm 写槽 → `FrameReadyPixels`。
   验证：`cargo t desktop_protocol`（单 client 帧递增）。
5. **宿主两态合成**：`stage3.rs`/宿主合成段 Pixels 上传路径 + 栅格化器
   抗锯齿/damage 产能化。
   验证：`cargo t desktop_protocol && cargo t ui`。
6. **三态开关**：`crates/auto-man/src/pac.rs` + 模板 pac.at `render` 字段 +
   `crates/auto/src/cmd_autodesk.rs` `--render` 参数 + `adjudicate()` 裁决链
   （auto→探测降级+Log）。
   验证：`cargo t -p auto-man pac && cargo t desktop_protocol`。
7. **投影器爬坡**：`client_runtime.rs` 按 T1 清单扩 AppProjector（widget 臂
   + `ui/layout` 复用，D2 落地）。
   验证：`cargo t desktop_protocol`（爬坡逐项单测）。
8. **re-exec 集成 T3**：001–005 queue 模式端到端 + independent 模式 + 双模
   并存用例（480 re-exec 形态）。
   验证：`cargo t desktop_protocol`（集成档，feature 门控沿既有）。
9. **parity 金样 T4**：001 三臂对拍基线入金样体系。
   验证：金样套件绿（随 `cargo t vue`/金样档）。
10. **实机冒烟 + 收尾**：T5 三态各一轮 + 降级日志目检；健康检查；状态翻
    execution_done。
    验证：`cargo check -p auto-lang && cargo t ui`。

## 复审记录

（/auto-plan:review 填写）

## 待澄清事项

- **D1 Text op 形态**（T1 定案）：glyph run 已定位下发（倾向）vs 字符串+
  样式宿主侧布局——影响宿主是否引入 cosmic-text 依赖与命中坐标同源性。
- **D2 布局引擎**（T1 定案）：复用 `ui/layout`（倾向）vs iced 离屏布局——
  决定 projector 的独立性。
- **D3/D4**：交互区表上报粒度、IME 光标/popover 坐标下发——随 D1 同批定案。
- **pixels 格式**：v1 仅 RGBA8（Bgra 预乘与否随 494 真洞结论对齐——宿主
  合成两处消费同一约定，执行期核对 494 合入后的 alpha 口径）。
- **性能预算**：queue 帧编码吞吐与 independent shm 带宽——T3 集成顺带采样
  记录（不设门槛，480 内存基线先例同型）。
- 与 497 的 shell.at 消费无耦合；若 497 先合入，本计划虚拟窗合成路径按其
  缩略快照结论复核一次（快照与 Pixels 帧的协同）。
