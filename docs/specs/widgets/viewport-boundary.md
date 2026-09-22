# viewport-boundary — VM/iced 视口边界与嵌入居中契约（PLAN-663）

> **Status**: active
> 路径：`crates/auto-lang/src/ui/iced/renderer.rs`  | 技术栈：Rust（iced 0.14 解释臂）
> 关联：[scroll-pane](scroll-pane.md)（滚动通道）、[terminal-iced-draw](terminal-iced-draw.md)

## 1. 语义

### 1.1 视口边界（iframe 语义）

Tailwind 视口单位（`h-screen`/`w-screen`/`h-dvh`/`h-svh`/`h-lvh`/`min-h-screen`，解析为
`SizeValue::Screen`，与 `h-full` 的 `SizeValue::Full` **严格区分**）的消费规则：

- **窗口根**（无定高祖先）：`Screen → iced Length::Fill`（满窗，历史行为不变）。
- **定高(px)嵌入边界子树内**：渲染前 pre-pass `rewrite_viewport_units` 把 Screen 族
  重写为 `Pixels(边界高)`（`min-h-screen` 的 `f32::MAX` 标记 → `MinHeight(边界高)`），
  即 CSS iframe 模型——嵌入框建立新视口，`100vh` 重锚定到框高。
- 边界判定：节点样式含 `Height(Pixels(px>0))` / `Width(Pixels(px>0))`（仅 arbitrary
  px；spacing 定高 `h-44` 不建边界，见 P663-D2）。宽/高两轴独立锚定；嵌套边界逐轴
  覆盖外层。
- 边界链下潜变体：Row/Column/Container/Scrollable/MouseArea/Overlay/Grid。

**背景**：修复前 `screen` 与 `full` 合并解析为 Fill；Fill 嵌在 Shrink 祖先
（PLAN-642 T-12 的 Shrink scroll 兜底、justify-center 让渡列高）下失去定高锚点，
整树塌缩为最小内容高——ui-gallery VM 臂全屏 demo 只剩一条播控条的根因。

### 1.2 垂直安全居中（`my-auto`/`m-auto`）

iced 无 margin-auto 布局原语。渲染前 `expand_margin_y_auto` 把「定高(px)且**无
overflow** 的 Column」内携带 `my-auto`/`m-auto` 的直接子改写为
`h-full w-full flex-col justify-center` 包裹列：矮内容垂直居中、超高内容溢出裁剪——
CSS `margin:auto` safe-center 语义。带 overflow 的列不展开（滚动交互归 T-12/662
机制）。多子项同时声明的空间分配与 CSS 逐项吸收不完全等价（P663-D4）。

## 2. 实现位

| 阶段 | 位置 | 消费 |
|---|---|---|
| 解析 | `ui/style/class.rs` `SizeValue::Screen` + `MarginYAuto/MarginAuto` | — |
| 适配 | `ui/style/iced_adapter.rs` `IcedSize::Screen` + `margin_y_auto` | `iced_length(Screen)=Fill` 缺省 |
| 重写 | `ui/iced/renderer.rs` `rewrite_viewport_units` / `expand_margin_y_auto_walk` | `render_dynamic_view`（path 为空=根）与 `into_iced` 三根点调用 |

幂等性：重写仅 Screen→px 单向，缓存帧复用安全。嵌入宿主（ui-gallery VM 臂
`AppViewport.vm.at`、未来 desktop-host/虚拟桌面窗）零声明改动即得正确语义；
新宿主的规范声明形态 = `w-[W] h-[H] overflow-y-auto` frame + 子项 `m-auto`。

## 3. demo 壳层 chrome 禁令（PLAN-684 SD-01）

demo 根部件（examples/ui/*/src/front/app.at 外壳）禁用 viewport-fixed 铬件：
`fixed` / `inset-y-0` / `inset-0` / `sticky` 锚定**浏览器视口**而非嵌入容器，
gallery `AppViewport` 的 deep 覆盖只改写 `.h-screen/.min-h-screen/.w-screen`
高宽类、不覆盖 fixed——侧栏/遮罩穿出视口卡（018 `fixed inset-y-0 z-40`
实证，P675-D1 ②）。正确形态：**flex 行 + 侧栏 `shrink-0 h-full` + 主区
`flex-1 min-w-0 overflow-auto`**（高度锚定父容器，deep 覆盖后=容器高）；
模态遮罩用容器内 absolute（滚动容器 `relative`）。仅容器内滚动条/吸顶
元素允许容器坐标的 absolute/sticky。同族修复示例：018（T-01）/022
（rev2 app frame）/025/026/027（`h-full min-h-screen` 全高壳）。

## 4. 契约边界

- `h-full`（Full）不重锚定（父容器百分比原义保留；全屏外壳仓内惯例 = h-screen）。
- T-12 Shrink 兜底**不退役**：与 `Screen→Fixed(边界)` 组合成立（Fixed 不受 Shrink
  塌缩影响）；退役决策挂 P663-D1。
- 已知债：P663-D2（spacing 定高不建边界）、P663-D3（h-full 不重写）、P663-D4
  （my-auto 多子项）。
