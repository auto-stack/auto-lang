# Plan 465 T4：WM DOM 叶 + WmStore（E1/E2 语义规范 + I4/I6 落地记录）

> 2026-08-28。消费 `reports/462-i1-seam-review.md` §3（E1/E2 加法扩展点，
> "465 落地时若先行实现 E1/E2，386 复活成本进一步下降"）。

## 1. E2：AppWindow 叶子形态枚举（成文）

```
AppWindow ::=
  | Element       // 462/463 v1：叶子在宿主 widget 树内（iced Element 子树）
  | RenderCommand // 386 路线 B：纹理/图层 RenderCommand 叶（复活时追加）
  | Wayland       // 外部 wayland surface 叶（远期）
  | DOM           // 本计划：vue 宿主的 DOM 节点叶（R2「Web = DOM 子树」）
```

- 形态切换是**加法操作**：iced 侧叶子构造点单一（renderer.rs desktop 分支
  per-vwin `dynamic_view(view, is_primary)` → 叶无关的 `virtual_window_element`）；
  vue 侧叶子构造点单一（宿主 App.vue z-stack 内 per-win `<VirtualWindow :win="w">`）。
  两侧互不感知，WM 语义层（矩形/焦点/z/命中）完全共享。
- **不出现第二条桌面代码路径**（与 I3 同型）：桌面宿主唯一，叶子在装配点替换。

## 2. E1：`(AppId, event)` 注入形状（成文 + 实现）

叶子不在宿主 widget 树内时，widget 事件需"指针/键盘 → (AppId, event) 注入"。
vue 宿主 v1 的注入形状（TS 侧，`assets/wm/store.ts` + `keyboard.ts`）：

1. **指针半边**：`focusAtPoint(x, y)` —— z 序顶到底做**命中矩形含点测试** →
   命中窗口的 `appId` 即归属 App；该窗内 pointer 事件经 DOM 冒泡天然归属
   （浏览器即注入通道）；窗体 chrome 事件 `stopPropagation` 阻断穿透
   （与 iced mouse_area 吞噬语义对齐，对拍项）。
2. **键盘半边**：`document` keydown **捕获段**统一拦截（T2 spike ③ 定案）——
   已消费桌面热键 `preventDefault + stopImmediatePropagation`；未消费按键
   经 DOM 焦点自然投递给焦点窗（`focusedWid` → `appId` 即归属）。
3. **扇出对应**：iced `DM::App(AppId, msg)` 扇出 = vue 侧组件实例事件
   （app 实例内部处理，跨窗命令经 WmStore 函数调用直连，T1 蓝图 §5 差异登记）。
   386 复活时按此形状在 WmState 命中矩形与 DM::App 扇出间补注入枚举臂即可。

## 3. I4：同一登记源出两端实现

- **登记源**：`schema/aura.at` —— `virtual_window`（新增：vue: → `@/wm/VirtualWindow`，
  props win/title/class）+ `taskbar`（web 臂 none → component，vue: → `@/wm/Taskbar`）。
- **iced 端**：`ui/iced/virtual_window.rs`（462 定位/clip/chrome/八向把手）+
  shell 装配（463）。
- **DOM 端**：`auto-man assets/wm/{VirtualWindow,Taskbar}.vue`（每次 run 覆写
  进宿主 `src/wm/`）。
- **金样**：`test/a2vue/virtual_window/`（input.at → expected SFC）锁定
  `.at` 作者视角的两端同源 emit（`import { VirtualWindow } from '@/wm/VirtualWindow'`
  + 组件树 + slot）。测试 `test_a2vue_virtual_window`（shadcn 模式）。
- **已登记缺口**：a2vue 组件路径的**任意 prop 直通**（`win: w` 不透传）对
  shadcn 组件路径未实现——宿主叶子自读 store，不依赖该透传；.at 直书桌面
  属后续需求，届时补齐（KNOWN-DEBT 候选）。

## 4. WmStore（vue 叶的 WM 状态）

`assets/wm/store.ts`：`wins[] {wid, appId, title, rect, z, focused, container, app}`、
`layoutMode`、`focusedWid`。写点纪律同 462/463：
- rect 唯一批量写点 = `applyLayout()`（R9 排布策略）；单窗交互（拖拽/缩放）
  直改 `win.rect`（free 语义：用户位置即真值）。
- `launchWindow/close/focus/cycleFocus/setLayout` 对应 iced WM 最小集命令；
  close = `app.unmount() + container.remove()`（459「窗关 App 随之退」）。

## 5. I6：布局期望值表双端共享

- **共享表**：`crates/auto-lang/src/ui/layout_cases.json`（17 例：usable/grid
  1-9/master-stack 4 态/free 恒等/cascade/snap 4 态）。
- **Rust 消费**：`ui/layout.rs::layout_parity_cases_shared_table`（--features
  ui-iced；ε=0.51 与既有 approx 同参）。
- **TS 消费**：`scripts/ui-layout-parity.mjs`（Node 原生 strip-types 直跑
  `assets/wm/layout.ts`，零转译依赖）——17/17 绿。
- 改布局语义必须**两侧同改 + 表同改**（表头注释已声明）。

## 6. DOM 叶细节（T2 spike 结论的落地）

- 窗容器 = absolute 定位 + `overflow: clip` 语义（rounded + overflow-hidden）；
  **不**依赖 containing-block 挡 portal（T2 ①：DOM 重挂不可 CSS 收敛，限制清单）。
- 拖拽/缩放：`setPointerCapture` + `win.rect` 直改；八向把手 `data-dir`。
- 焦点：窗内 pointerdown → `focus(wid)`（命中测试含 z 序）。
- 键盘/Tab 篱笆：`keyboard.ts`（捕获段 + 焦点窗内手动循环，不用 inert）。
- 全局监听跨窗广播（T2 ③）：已知限制，受害 App 白名单先行。

## 7. 验证清单（本任务门禁）

- [x] a2vue 金样 `test_a2vue_virtual_window` 绿
- [x] I6 共享表：Rust 测试绿（ui-iced feature）+ `node scripts/ui-layout-parity.mjs` 17/17 绿
- [x] `cargo check -p auto-lang -p auto-man` 绿
