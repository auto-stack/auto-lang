# 20 - AutoUI 分离架构设计文档

> 来源：前期在 DeepSeek 上的架构讨论成果，2026-07 整理入库。
> 关联：docs/design/08-ui-systems.md（AURA IR）、docs/plans/364-a2r-cosmic-replication-readiness.md。

## 1. 项目背景与目标

AutoUI 是一个跨平台 UI 解决方案，包含声明式语言前端、独立中间表示层和可替换的渲染后端。其长期目标是驱动 **AutoOS** 操作系统桌面环境，实现**百应用级别轻量运行**和**全局共享渲染资源**。

当前基于 iced 的单体应用架构，每个应用需独立持有 wgpu 上下文、字体图集、管线缓存等资源，内存基线高达 100 MB，无法支撑数十个应用同时运行。分离架构的核心思想是：

- **应用不拥有 GPU 资源**：每个应用进程极轻量（目标 1–5 MB），只包含逻辑和 UI 描述。
- **宿主集中渲染**：系统级 Compositor 进程持有唯一 GPU 上下文，执行所有应用的绘制命令，复用全局字体、纹理、着色器等。
- **共享渲染**：通过无锁共享内存传递轻量绘制指令，消除重复资源，将系统总内存控制在 1 GB 以内（100 个应用）。

## 2. 架构概览

系统由三大核心部分构成：**AutoTree**、**RenderCommand** 和 **RenderQueue**，以及运行它们的 **宿主 Compositor** 与 **应用壳**。

```
[应用进程]                          [宿主 Compositor 进程]
Auto 语言前端                          ┌─────────────────┐
   ↓                                  │ 窗口/输入管理    │
AutoTree (VNode 树)                    │ 全局 wgpu/Vulkan │
   ↓                                  │ 字体图集/纹理池  │
RenderCommand 生成 ──── RenderQueue ──▶ 合成器/渲染器    │
(共享内存无锁队列)                     └─────────────────┘
```

- **应用进程**：解析 `.auto` 源码，维护 AutoTree，按帧生成 RenderCommand 增量/全量数据，推送到 RenderQueue。
- **宿主进程**：管理窗口树、Z-order、输入事件；从每个窗口的 RenderQueue 中按序取出命令，执行 GPU 绘制并合成最终桌面。
- **RenderQueue**：基于共享内存的单生产者单消费者无锁环形队列，实现极低延迟（纳秒级）的应用-宿主通信。

## 3. AutoTree 设计

AutoTree 是 AutoUI 在 Rust 环境中的唯一中间表达，取代原有的 `AbstractComponents`。它是**声明式 Widget 虚拟树**，并非绘制基元列表。

### 3.1 VNode 结构

每个节点代表一个 UI 组件（Button、Row、Text 等），存储其语义属性与来源信息。

```rust
pub struct VNode {
    pub id: NodeId,                     // 稳定全局唯一标识
    pub kind: VNodeKind,                // 组件类型
    pub props: HashMap<String, Value>,  // 组件属性（文本、颜色、样式等）
    pub layout: Option<Layout>,         // 布局计算结果（Rect 等）
    pub children: Vec<VNode>,           // 子节点
    pub source: SourceSpanSet,          // 源码位置映射
}

pub enum VNodeKind {
    Button,
    Text,
    Input,
    Row,
    Column,
    Custom(String),                     // 用户自定义组件
    // ...
}
```

**稳定标识（`NodeId`）** 由编译器分配，基于源码声明位置和循环键值生成，确保跨帧持久。辅助的 `VPath`（如 `App/navbar/col[1]/button[0]`）提供人类可读路径，用于 Inspector 面包屑。

### 3.2 动态结构处理

- **条件分支**：记录条件表达式源码位置（`condition_span`），VTree 仅包含当前激活的分支子树。
- **循环生成**：每个实例节点携带 `(模板NodeId, 迭代键)` 复合标识，数据变化时通过键值稳定定位。

### 3.3 AutoTree 与源码的映射（SourceMap）

SourceMap 建立 **VNode → 源码多位置** 的精确关联：

```rust
pub struct SourceSpanSet {
    /// 在 view 块中的 widget 声明位置
    pub view_span: Option<SourceSpan>,
    /// 绑定的 model 变量声明与引用位置
    pub model_refs: Vec<ModelRef>,
    /// 事件绑定与处理位置
    pub event_refs: Vec<EventRef>,
}
```

由 Auto 编译器在生成 AutoTree 时填充，运行时只读。这为 Inspect 面板、源码跳转、反向修改提供了基础。

### 3.4 Diff 与 Patch

当 model 变化时，新的 AutoTree 与上一帧的旧树进行差异比较，产出 **Patch 集合**。Patch 是描述树变更的最小单元：

```rust
pub enum Patch {
    UpdateProps { node_id: NodeId, props: HashMap<String, Value> },
    AppendChild { parent_id: NodeId, child: VNode },
    RemoveNode { node_id: NodeId },
    Reorder { parent_id: NodeId, new_order: Vec<NodeId> },
    // ...
}
```

这些 Patch 再转换为 RenderCommand 序列发送给宿主。对于静态区域，宿主可缓存上一帧的绘制结果，仅重绘脏区。

## 4. RenderCommand 设计

RenderCommand 是宿主渲染器执行的**平台无关的轻量绘制指令**，由 AutoTree 增量转换而来。

### 4.1 命令分类

```rust
pub enum RenderCommand {
    // 图形基元
    DrawRect { rect: Rect, color: Color, radius: f32 },
    DrawText { font_id: FontId, glyphs: Vec<GlyphInfo>, color: Color },
    DrawImage { image_id: ImageId, src: Rect, dst: Rect },

    // 状态管理
    SetClip { rect: Rect },
    PushLayer { opacity: f32, transform: Matrix },
    PopLayer,

    // 增量更新指令
    UpdateTexture { node_id: NodeId, data: Vec<u8> },
    InvalidateRect { rect: Rect },
    CacheControl { node_id: NodeId, cache: bool },

    // 自定义（保留扩展）
    Custom { shader: ShaderId, params: Vec<u8> },
}
```

命令设计为**窗口内相对坐标**，应用不感知屏幕绝对位置或 Z-order，由宿主在合成时统一处理窗口变换。

### 4.2 全量与增量帧

每个 RenderQueue 条目包含帧类型标记：

- **Full**：后跟完整 RenderCommand 列表，用于首帧、窗口大小改变或缓存失效。
- **Incremental**：仅含基于上一帧变化的命令，宿主根据自身缓存重绘指定区域。

应用在帧头写入 `DirtyRect` 列表，告知宿主窗口内哪些矩形区需重新合成，其余区域可复用上一帧纹理。

## 5. RenderQueue 设计

RenderQueue 是应用与宿主间的高性能通信通道，基于**共享内存 + 无锁单生产者单消费者环形队列**。

### 5.1 队列结构

每个应用分配两个队列（命令上行、事件下行），每个队列包含：

- 共享内存控制块：包含读写指针（原子变量）、缓冲区大小、帧同步信号（如 `eventfd`）。
- 数据环形缓冲：存储序列化的 RenderCommand 流或事件数据。

### 5.2 通信流程

- **应用端**：
  1. 构造增量/全量 RenderCommand 列表。
  2. 写入共享缓冲区（零拷贝时直接构造在缓冲区内）。
  3. 原子更新写指针。
  4. 可选触发 `eventfd` 通知宿主（或宿主定期轮询）。
- **宿主端**：
  1. 在合成循环中遍历所有窗口（按 Z-order）。
  2. 对每个窗口，原子读取读指针到写指针之间的数据块。
  3. 解析命令并立即提交 GPU 执行。
  4. 处理后更新读指针，回写事件（如输入）到应用的事件队列。

### 5.3 性能特点

- 单次通信延迟 <5 μs（含内存写入和原子操作），对 16 ms 帧预算可忽略。
- 零锁竞争，无内核调用（除可选事件通知）。
- 应用只发送窗口局部命令，典型每帧 10–100 KB，共享内存带宽足够支持 100+ 应用。

## 6. 宿主 Compositor 设计

宿主是系统的唯一渲染服务进程，负责窗口管理、输入分发、GPU 资源管理和合成。

### 6.1 窗口管理

维护全局窗口树，存储每个窗口的：
- Z-order、位置、大小、可见性。
- 对应的 RenderQueue 引用。
- 离屏纹理缓存（用于增量合成和透明窗口）。

宿主按照 Z-order 从低到高依次执行每个窗口的 RenderQueue，绘制到其离屏纹理或直接到帧缓冲。支持透明混合和跨窗口效果（阴影、模糊等）。

### 6.2 全局资源管理

- **字体图集**：宿主加载系统字体，光栅化常用字形到一张大纹理（如 2048×2048），所有应用共享 `FontId`。
- **图标/控件样式纹理**：全局主题纹理，应用仅引用 ID。
- **着色器/管线缓存**：宿主预编译常用管线，应用通过 `ShaderId` 引用。

应用不持有任何 GPU 资源，仅发送命令。

### 6.3 渲染后端抽象

初期使用 wgpu 实现快速原型，后期通过 trait 替换为自研 Vulkan 渲染器，实现极致轻量（宿主 <30 MB）。抽象接口定义：

```rust
pub trait GpuBackend {
    fn execute_commands(&mut self, commands: &[RenderCommand], target: &mut RenderTarget);
    fn create_font_atlas(&mut self, font_data: &[u8]) -> FontId;
    fn update_texture(&mut self, id: TextureId, data: &[u8]);
    // ...
}
```

### 6.4 输入与事件

宿主从窗口系统（winit/SDL3）捕获输入，根据焦点窗口路由事件至对应应用的事件队列。应用在空闲时读取事件并更新 AutoTree。

## 7. 应用端流程

每个应用编译为独立的原生二进制，链接 `autoui-client` 库。该库屏蔽渲染后端，应用仅操作 AutoTree。

### 7.1 应用生命周期

1. **初始化**：解析 `.auto` 文件，构建初始 AutoTree，分配共享内存队列，连接宿主。
2. **事件循环**：接收宿主发送的事件（鼠标、键盘、自定义），调用 `update` 函数修改 model，触发 AutoTree 增量构建。
3. **生成 RenderCommand**：将 AutoTree Diff 转换为全量或增量 RenderCommand，推入 RenderQueue。
4. **帧同步**：等待宿主完成信号（可异步），进入下一帧。

### 7.2 内存基线

- AutoTree 状态：几百 KB。
- 通信缓冲：共享内存约 256 KB。
- Rust 运行时 + 应用逻辑：~1–2 MB。
- 不链接任何图形库。

**总内存占用：2–5 MB**，未来通过 `no_std` 等优化可进一步降低。

## 8. 开发工具与调试

### 8.1 内嵌 DevTools

在单应用开发阶段，提供类似浏览器 F12 的面板，内嵌于应用中（基于 iced 实现，功能稳定后抽取为独立服务）。

**核心功能**：
- **元素选择与高亮**：鼠标悬停显示 VNode 边界、ID、VPath。
- **Inspector 面板**：展示选中节点的属性（类型、样式、布局、数据绑定、事件）。通过 SourceMap 可查看和跳转源码位置。
- **实时编辑**：修改简单属性（文本、颜色）立即反映到界面，并可反向写入源码（通过 SourceMap 定位替换）。
- **事件追踪**：记录消息流和状态变化。

### 8.2 外部 AutoDesign 工具

当切换到共享宿主后，抽取 Inspector 逻辑为独立应用，通过宿主授权的调试通道获取任意应用的 AutoTree 快照和 SourceMap，实现**全局调试与 AI 辅助设计**。

## 9. 未来演进

### 9.1 渲染后端优化

- 宿主从 wgpu 迁移到自研 SDL3 + Vulkan 后端，内存基线降至 <30 MB。
- 实现跨平台 Vulkan 渲染器（Linux 原生，Windows 原生，macOS 通过 MoltenVK 兼容）。
- 提供 CPU 2D 后端（基于 tiny-skia）作为降级方案，单应用内存 <10 MB。

### 9.2 AutoOS 集成

- Compositor 直接基于 Wayland 协议和自研渲染器，替代 X11/Wayland 桌面环境。
- 应用通过 `autoui-client` 作为系统标准开发方式。
- 系统级字体、主题、资源服务全部集中管理，应用零冗余。
- AutoDesign 成为系统原生设计/开发工具，支持 AI 局部修改 UI。

## 10. 总结

本架构通过 **AutoTree 中间表示、RenderCommand 轻量指令、RenderQueue 无锁通信** 和 **共享宿主 Compositor**，实现了 UI 逻辑与渲染的彻底分离。它兼具声明式开发的便利性与命令式渲染的性能优势，能够以极低的内存代价同时运行上百个应用，为 AutoOS 桌面环境提供了坚实的技术基座。
