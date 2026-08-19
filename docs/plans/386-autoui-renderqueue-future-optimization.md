# Plan 386: AutoUI RenderQueue / 分离渲染架构（未来优化）

> **状态**：⏸ 暂缓（明确的前置条件未满足）
> **来源**：从 Plan 365 W5 独立出来。Plan 365 的 Host ①/②（in-process）已
> 完成；RenderQueue 是 Host ③（AutoOS 愿景），不影响 COSMIC 兼容性。

## 背景

### 为什么独立出来

Plan 365（AutoUI 可插拔宿主架构）的 W5 原本包含 RenderQueue/共享内存
IPC/分离 compositor。但在 Plan 365 实施过程中明确了：

- **COSMIC 兼容不需要 RenderQueue**。Linux 上的 COSMIC 组件靠 Host ②
  （VTree → libcosmic Element，**in-process**）运行——和 COSMIC 原生组件
  （cosmic-applet、cosmic-panel 等）一样，都是各自独立的 in-process iced
  应用，直接通过 libcosmic + Wayland 与 cosmic-comp 交互。
- **RenderQueue 是 AutoOS 的内存优化**，为"100 个 app 共享 GPU"场景设计
  （当前单体架构每 app ~100MB → 分离架构目标 ~1-5MB/app）。它与 COSMIC
  兼容性正交。

因此，把 RenderQueue 从 Plan 365 独立为本计划，让 Plan 365 干净地收束为
"in-process 架构就绪"。短期目标是把 in-process 架构在 Windows（dev host）
和 Linux（libcosmic host）上都跑通；RenderQueue 是达成后的性能优化。

### 短期目标（不含本计划）

- **Windows**：in-process iced，每 app 独立渲染，mock 系统端口。已就绪
  （Plan 365 W1–W2）。只要保证新代码不影响现有逻辑即可。
- **Linux/COSMIC**：in-process libcosmic，每 app 独立渲染，真实系统端口。
  Plan 365 W3（host-libcosmic 脚手架）+ W4（ports-linux 脚手架）已就绪，
  真实实现在 Linux 环境上单独开发。

## 启动条件（Plan 365 D4）

本计划**不立即启动**。启动需同时满足：

1. **≥3 个复刻的 COSMIC app 在 Host ②（libcosmic in-process）上跑通**——
   证明 in-process 架构功能完备，分离架构是纯优化而非功能补全。
2. **测得内存/延迟预算证明需要拆分**——如果 3 个 in-process app 的总内存
   在可接受范围内（如 <1GB），分离架构的复杂度收益比不合理。

满足后，按 Plan 365 Migration 的 Stage 1→2→3 渐进推进。

## 工作项（启动后）

### Stage 1 — VTree → RenderCommand lowering + in-process loopback

| 项 | 内容 |
|----|------|
| 目标 | 在 in-process 内（无 IPC）把 VTree 降为 RenderCommand 流，用 loopback executor 执行，证明渲染等价性 |
| 验收 | 每个 example 的 VTree snapshot → RenderCommand 序列 golden-compared；loopback executor 渲染结果与 in-process iced 像素等价 |
| 文件 | `crates/auto-lang/src/ui/render_command.rs`（新）、`crates/auto-lang/src/ui/loopback_executor.rs`（新） |
| 关键约束 | 同一 VTree snapshot → 同一 RenderCommand 序列（可序列化 golden 比较） |

### Stage 2 — RenderQueue transport，两进程，单 app

| 项 | 内容 |
|----|------|
| 目标 | RenderQueue 共享内存传输（app 进程 → host 进程），单 app，Windows first（wgpu/winit 跨平台） |
| 验收 | 一个 example app 跑在独立进程，经 RenderQueue 发送 RenderCommand 到 host 进程渲染；opt-in |
| 文件 | `crates/autoui-host/`（新）、transport 层（Windows: `CreateFileMapping` + named events；Linux: memfd + eventfd） |
| 弹性 | app 检测 host 断连 → 等待重连；host 崩溃不杀 app（app 自有状态 + VTree，host 只持有可重建的 GPU 资源） |

### Stage 3 — 多 app 共享 host

| 项 | 内容 |
|----|------|
| 目标 | 多个 app 共享同一 host 进程（字体图集/纹理池/Pipeline 缓存集中化）— doc 20 的真正内存收益 |
| 验收 | ≥3 个 app 共享一个 host，总内存显著低于 3 × 独立进程 |
| 文件 | host 的窗口注册表（app 连接 → 窗口 → surface） |

## 设计输入（Plan 365 已记录，启动时参考）

- **Windows host 不是 compositor**——DWM 是。host 是 DWM client：winit
  多窗口进程，持有唯一 wgpu 上下文，每 app 一个 OS 窗口，执行该 app 的
  RenderCommand 流。窗口堆叠/装饰/焦点/最终合成由 Windows 负责。
- **Linux host**：同一二进制可作为 winit client 调试；AutoOS 阶段才生长
  smithay-based 真 compositor 变体（Linux-only）。
- **弹性**：host 是接受的 SPOF（与 Wayland compositor / Chrome GPU 进程
  同风险级别）→ 目标是快速无状态恢复，非消除 SPOF。详见 Plan 365 D2
  resilience requirements（host 代计数器、Full-frame 重发、watchdog 重启、
  RenderCommand 边界检查、可选按关键性分片）。
- **终极回退**：永久的 in-process 路径让 app 在无 host 可达时降级为自渲染。
- **code_editor（Plan 413）约束与反馈**：编辑器 core 产出按行稳定 id 键控的
  `EditorDrawList`（文本 run + quad + 行号文本 run），shaping 留 app 侧（光标/
  命中/换行需同步布局，不做 IPC shaping 服务）。启动 Stage 1 时：验收应纳入
  `examples/ui/041-code-editor` 作为 golden 样例（最严苛的文本消费者）；协议需
  补三点——事件下行通道的 IME（preedit/commit/cursor rect，分离模式下 winit
  在 host 侧）、字体注册命令（app 自带字体的上传）、按行 CacheControl/
  DirtyRect。详见 Plan 413 §7。
- **可选加速（与 Plan 413 并行，不改变本计划启动条件）**：先做 editor-only 的
  `EditorDrawList` → RenderCommand golden lowering **薄切片**（纯函数、无
  transport、无 host、无 IPC，数天级工作量）——在 transport 开工前用最严苛的
  文本消费者（千级 glyph、按行高亮、IME）自下而上硬化文本协议。时序结论：
  不采用"RenderQueue 先行、auto-edit 后行"——分离架构是纯内存优化而非功能
  前置，editor 先行零丢弃（core 事件类型与 draw 契约已隔离，见 413 §3.1/§7）。

## 不在本计划范围

- COSMIC 组件复刻（cosmic-screenshot → cosmic-session → cosmic-monitor）—
  那是驱动 Host ② 真实实现的工作，与 RenderQueue 正交。
- in-process 架构的任何改动 — 那是 Plan 365（W1–W4 已完成）的范围。
- VM 后端 GUI — COSMIC 复刻是 a2r-only（Plan 364 约束）。

## 关联

- **Plan 365**：in-process 宿主架构（Host ①/②），W5 原指向本计划。
- **Design Doc 20**：分离架构的完整设计（AutoTree / RenderCommand /
  RenderQueue / Compositor）。
- **Plan 364**：a2r COSMIC 就绪——复刻 app 的语言能力前置。
