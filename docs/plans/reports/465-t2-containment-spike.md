# Plan 465 T2：页面级假设 containment spike 实测记录

> 2026-08-28。方法：真实管线生成（`auto build --gen-only --render vue` + 完整
> `auto run` 物化 shadcn 组件）→ 双窗口容器（win-a 裸 absolute+clip；win-b 加
> `transform: translateZ(0)` + `contain: layout` 模拟 containing block）各
> `createApp(App).mount()` 一实例 → Playwright 驱动 + DOM 探针面板。
> spike 工程：`scratch/spike-465/`（pac.at + src/front/app.at +
> gen/front/vue/{index.html,src/spike-main.ts} harness）。
> 截图：`docs/plans/reports/assets/465-t2/`。

## 1. 实测结果（§3.3 表格四项 + 焦点策略）

| # | 假设 | 实测 | 结论 |
|---|---|---|---|
| ① | `modal`→`fixed inset-0` 可用 containing block 收敛 | **dialog/modal 均映射 shadcn `Dialog`**（`ui_gen/vue.rs` L9559 `"dialog" \| "modal"` → `v-model:open`），reka `DialogPortal` 把 overlay/content **DOM 移动**到 `document.body`：探针 `dialog-overlay: div < BODY`、`dialog-content: span < div < BODY`、`overlay-rect: 1280x800 @(0,0)`（全视口） | **CSS containing block 挡不住**（DOM 重挂 vs CSS 收敛是两层）；win-b 的 translateZ+contain 对它无效。portal 族（dialog/modal/dropdown 等 reka 系）= 已知限制清单；非 portal 的裸 `fixed` 元素仍可用容器 CSS 收敛（DOM 未搬家） |
| ② | `teleport to:"body"` 逃到 body | 探针 `teleport-marker: span < BODY`（两实例均然），页面底部可见两个 TELEPORT-MARKER | 证实。v1 照计划：登记限制 + 启动时警告占位，不改写 portal |
| ③ | `.window/.document` 全局监听跨窗 | 单次按键后 `keydown-counters: ["document keydowns: 2","document keydowns: 2"]`——**两个窗口实例计数同涨**（各自 `document.addEventListener('keydown')`，onMounted 注册/onUnmounted 注销） | 证实。桌面热键必须走 **document 捕获段 + `stopImmediatePropagation`** 吞掉已消费组合键；App 自注册的 document 监听天然广播 → 已知限制清单（受害 App 白名单先行） |
| ④ | 主题页面级 | 两窗共享 `html.dark` 主题（截图目视）；主题机制在 index.html/documentElement | v1 全桌面单主题成立，无每窗主题 |
| 附 | 多挂载先例验证 | 双 `createApp(App).mount(container)` 实例独立运行（计数器独立、UI 独立） | T5 挂载模型成立 |

## 2. DOM 焦点/键盘策略定案（T2 交付项）

1. **桌面热键**：document `keydown` **捕获段**统一处理；已消费组合键
   （Ctrl+Space 召唤 / Alt+Tab 轮转 / 布局切换）`preventDefault` +
   `stopImmediatePropagation`，未消费按键自然落到焦点窗内元素（原生 DOM 焦点）。
2. **Tab 篱笆**：不用 `inert`（会连带杀死 WM 自己的窗框 chrome 点击/拖拽）；
   捕获段拦 `Tab`——取焦点窗容器内可见可聚焦元素表
   （`a,button,input,select,textarea,[tabindex]` 过滤 disabled/hidden），
   手动循环 `.focus()`。约 30 行，v1 定案。
3. **点击聚焦**：容器 pointerdown 命中测试（T4 WM 语义）置焦点；
   窗外点击由 reka 的 outside-interaction 自然关 dialog（实测 probe 点击
   关闭了 dialog——桌面语义正确）。

## 3. 后续改进项登记（非本批）

- **portal 收敛正规解**：a2vue 生成器自有 DialogContent 模板
  （`ui_gen/vue.rs` L14483）可改 `DialogPortal :to` + provide/inject
  注入窗容器——generator 级改写，与 §3.3「v1 不做 portal 改写」冲突，登记
  KNOWN-DEBT 候选。
- reka 警告 `Missing Description/aria-describedby`（噪音，无阻塞）。

## 4. 截图

- `01-initial.png`：双窗多挂载初态；TELEPORT-MARKER 已见页面底部（②）
- `02-dialog-open.png`：win-a 开 dialog——内容全视口居中、遮罩盖全页（①）
- `04-dialog-probe.png`：探针实况 `overlay/content < BODY` + rect 1280x800（①②）
- `05-keydown-cross.png`：单键后双窗计数器同涨 2/2（③）
