# RQ 远程 Renderer 架构（方案 2：Iced 组件照常渲染，RenderQueue 转手一道）

> **状态**：已裁定（2026-09-22，四方案架构评审收敛；用户裁定现阶段走方案 2）
> **来源**：2026-09-21/22 RQ 双轨对拍系列（003 验收 → 004 差距 → 组件重实现成本
> 追问 → 四方案架构评审）。原始诉求见会话记录：用户最初设想即为本文方案 2
> ——「iced 组件照常渲染，唯一区别是渲染经 RenderQueue 转手一道（函数调用
> 变 RPC）」。
> **关联**：[Design 20](../20-autoui-separation-architecture.md)（分离架构）、
> [desktop-protocol-v1](desktop-protocol-v1.md)（wire v1 现状）、
> [Design 22](base-styles-and-visual-parity.md)（样式单源）、
> PLAN-679（RQ 臂样式保真，本轮驱动案例）、P020-D1（双投影器债，本文为其
> 终局解）。

## 0. 一句话

**AuraView → iced 组件树照常构建/布局/命中（iced 全责），渲染原语经
RecordRenderer 序列化为 DisplayList 过 RenderQueue，RQHost 重放绘制。**
渲染逻辑与 VM 独立轨同出 iced 一份实现——「函数调用变 RPC」的字面达成。

## 1. 四方案对比与裁定记录

| 方案 | 结构 | 组件渲染语义实现份数 | 漂移风险 | 崩溃隔离 | 状态 |
|---|---|---|---|---|---|
| 1 全独立轨 | 各 app 自持 iced+wgpu | 一份 × N 进程 | 无 | ✓ | ❌ 内存论点否决（~223MB×N，PLAN-034 实测） |
| 2 **远程 Renderer**（本设计） | iced 组件树在 app，渲染原语序列化过队列，daemon 重放 | **一份（iced）** | **构造上不可能** | ✓（app 侧渲染语义自包含） | ✅ **现阶段选定** |
| 3 手写投影器（现状 v1） | AuraView → 手写块流布局+命令发射 → daemon 画布 | 两份（renderer.rs vs RqProjector） | 结构性存在（本轮 003/004 全部差距的根源） | ✓ | 降级为 legacy 兼容 wire；**保留为未来选项（§4）** |
| 4 Widget+Event Queue（语义过线，daemon 建组件树渲染） | View/事件过线，daemon 驻留组件树并用 iced 渲染 | 一份（daemon 端 renderer.rs） | 无 | ❌ daemon 单点：任一 app widget panic 波及全部桌面 UI | ❌ 用户四条反对否决（§2） |

### 2. 方案 4 否决记录（2026-09-22 用户四条反对，全部成立）

1. **带宽/延迟**：AuraView 串行化背着字符串形态样式（远胖于编译后参数）；
   120Hz 下每帧「反序列化→iced 树重建→布局」三道重活，RenderQueue 只需
   逐条填表。命令流在动态内容下更瘦、每帧成本更可预测。
2. **daemon 单体化**：方案 4 的 daemon = 多租户 UI 运行时（全量 AutoUI 引擎
   + N app 的 widget 树 + 命名空间隔离）；RenderQueue daemon = 无语义重放器。
   复杂度差永久存在。
3. **崩溃**：RenderQueue 的帧是无状态推导物——app model 才是真源，daemon
   崩溃重孵后下一帧全量恢复，协议零状态。方案 4 的 daemon 驻留派生态
   （widget 树），且任一 app widget panic 可能波及全部桌面 UI（故障隔离
   丧失）。
4. **头重脚轻/工业先例**：把 widget 语义放进合成器 = X11 server-side
   widget（Motif 时代）的已抛弃模型。工业界一致选择 app 侧渲染到
   display list：Chrome（渲染进程产 display list → 浏览器合成器）、
   Android（RenderNode → SurfaceFlinger）、Wayland（app 自绘 → 合成器收
   buffer）。方案 2 与三者同构。

### 3. 方案 2 vs 方案 3 的区别（本设计核心裁定）

| 维度 | 方案 2（远程 Renderer，现阶段） | 方案 3（自有渲染，未来选项） |
|---|---|---|
| 组件→命令的生成者 | **iced 自己**（组件照常渲染，Renderer 边界截获原语） | 自研投影器（块流布局+命令发射，RqProjector 形态） |
| 组件覆盖 | 全部 iced 组件自动获得，永不过时 | 逐 widget 手工爬坡（coverage 门禁台账） |
| 布局/文本度量 | iced/cosmic-text 真字形度量（App 内嵌字体，CPU） | 手写块流 + 启发式测量（固有精度缝） |
| 交互（命中/聚焦/IME/选区） | iced 原生（事件转发回 App 的真组件树） | 手写（PLAN-026 起自建） |
| 漂移类 bug | 构造上不可能 | 结构性存在，靠对拍发现 |
| 对 iced 的依赖 | **深度依赖**（program::Renderer/Compositor 内部面，版本 churn 税） | **零依赖**（只依赖 wire 协议；iced 可整体替换） |
| 渲染效果上限 | iced 支持什么有什么 | **可超越 iced**（自有渲染管线可做 iced 没有的效果/性能优化） |
| 实现成本 | 一次性后端投资（RecordRenderer + 事件回路径） | 持续性组件爬坡投资（每能力 × wire/投影/交互三处） |
| 状态 | **选定实施**（PLAN-680） | **保留为未来选项**：前提 = 投入大量精力复刻并**超过** iced；重估触发条件见 §5 |

一句话：方案 2 = 「正确性由 iced 构造保证」；方案 3 = 「自由度由自有管线
换取」。两者共享同一 wire（DisplayList 家族）与同一 daemon 重放面——
**切换只换命令的生产者，不换消费者**，故方案 3 作为未来选项始终开放。

### 4. 架构

```
App 进程（headless iced 宿主——无窗口、无 GPU）
┌────────────────────────────────────────────┐
│ view() → iced 组件树（真组件：button/input/ │
│           scrollable/text_input…）          │
│ UserInterface::layout(Size)   ← iced 布局   │
│ UserInterface::update(远程事件) ← 输入回路径 │
│ UserInterface::draw(RecordRenderer)         │
│   └─ RecordRenderer：fill_quad/draw_text/   │
│      layer/clip/transformation → 序列化     │
└──────────────┬─────────────────────────────┘
               │ DisplayList v2（管道）
┌──────────────▼─────────────────────────────┐
│ RQHost：重放绘制（paint_ops 扩展）           │
│ 窗口/事件捕获（iced daemon 或 winit 均可——  │
│ 后端可插拔，SDL3 等亦在此位）                │
└────────────────────────────────────────────┘
```

实现要点（风险 lowest 路线——**不走自定义 `iced::daemon` Compositor**）：
`iced_runtime::user_interface::UserInterface` 为公开 API（iced_test 仿真
同一驱动方式），headless 驱动 = view → layout → draw(RecordRenderer) 三步
直调；输入 = 远端事件转 `iced::Event` 注入 `UserInterface::update`。
RecordRenderer 需实现文本子 trait（App 内 cosmic-text 度量整形——字体已内
嵌，CPU 整形，无 GPU 面；内存论点保住：贵的部分是 wgpu 驻留）。

### 5. 方案 3 重估触发条件（未来选项的回看点）

- iced 版本 churn 税连续侵蚀（program::Renderer 面多次破坏性变更）；
- 渲染效果需求越过 iced 能力上限（特效/性能/平台）；
- 组件战略收敛（若 AutoUI 组件族稳定且不再追 iced 新能力，自有渲染的
  维护面收敛到可承受）。

满足任一且投入产出评审通过 → 按 §3 表的「自有渲染」列立项（届时本文档
§4 的 App 侧「headless iced 宿主」替换为「自有块流引擎」，daemon 重放面
不变）。

### 6. 迁移与兼容

- DisplayList v1（现 DrawList）保留为 legacy 兼容 wire（旧客户端）；
- 迁移灰度：`desktop_render: remote` 三态新增，按 app 逐个切换
  （001 → 003 → 004 试点先行）；
- RqProjector 迁移完成后冻结退役（coverage 门禁随之只约束 legacy 模式）；
- PLAN-679 Phase 1/2 修复对 v1 wire 真实有效，不浪费。

### 7. Spike（go/no-go 前置）

时间盒验证三件事（PLAN-680 T-00）：①`UserInterface` + 段落引擎（cosmic）
在无 wgpu feature 下可用；②RecordRenderer 录制 001 首帧 → 现有 paint_ops
回放成功；③与 wgpu 独立轨渲染做结构对照。红 → 回退方案 3 续航 + StyleSpec
单源兜底。
