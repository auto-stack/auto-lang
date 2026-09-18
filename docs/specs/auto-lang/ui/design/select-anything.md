# Select Anything —— 任意基面框选 → 结构化 Auto/JSON 回吐契约

> 来源：PLAN-646（2026-09-18 执行）。三消费面共享同一语义：VM/Iced 交互
> （Alt+拖拽 → DevTools「采集」标签页）、Vue dev 页 overlay（Alt+拖拽 →
> 浮动面板）、MCP 工具 `autoui_select_rect`（agent 程序化采集）。

## 范围

框选语义（命中判定/修剪/排序）、结果信封（字段与三格式渲染）、双端交互
键位与面板行为、MCP 工具契约。知识库的存储/检索/问答不属于本契约（采集
出口而已）。

## 1. 选择语义（三端同一，纯函数）

1. **中心点包含**：节点布局 bounds 矩形**中心**落在框选矩形内 → 命中。
   边缘被扫过但中心不在 → 不命中（避免框到半个大容器吞掉整容器）。
2. **顶层修剪**：命中集合中，某节点的任一祖先也在命中集合中 → 该节点被
   祖先吸收（剔除）。祖先链从 children 列表派生（不依赖 `VNode.parent`
   回填——拓扑事实源）。
3. **文档序输出**：按 VTree/DOM 文档序（DFS 先序）。
4. **降级**：无 bounds 的节点（LayoutCollector 只覆盖挂 id 的 widget；VM
   端非容器 widget 的探测容器仅在 F12 debug 模式挂载）不参与命中，但作为
   选中节点的子孙出现在 structure 输出中。零命中是合法结果（空 nodes）。

VM 侧实现：`crates/auto-lang/src/ui/selection/`（`select_nodes` /
`trim_to_topmost` / `Marquee`，内联单测）。Vue 侧 overlay 同语义独立实现。

## 2. 结果信封

```
{ surface, app, rect:[x,y,w,h], nodes:[{id, kind, span, source, structure}] }
```

- `surface`：`"vm"` | `"vue"`。
- `app`：App/widget 名（VM=widget_name；Vue=源映射键 stem）。
- `nodes[].kind`：源 widget 关键字（`kind_keyword` / `data-auto-tag`）。
- `nodes[].span`：`.at` 源码字节区间 `(offset, len)`（**字节口径**——Vue
  端 JS 切片必须经 UTF-8 字节（TextEncoder/Decoder），不得用 UTF-16 码元
  索引直切；中文注释会错位）。合成节点（无 span）为 null。
- `nodes[].source`：原文切片（逐字节等于源码子串后**去公共缩进**）。无
  span 或源文不可得时为 null。
- `nodes[].structure`：子树结构（VM=`VTreeAtomBuilder` scope=节点，Atom
  Node；Vue=剥 `data-auto-*` 后的 DOM 子树 JSON `{tag, props, children}`）。

三格式渲染（`SelectionFormat::Auto | Json | Atom`）：

- **Auto**（面板默认视图）：头注释
  `// ── AutoUI Select Anything ── surface=… app=… rect=(…) nodes=N` +
  每节点 `// [i/N] <kind>  span=off..end` + 切片；无 span 节点标注
  `(synthetic, no source span)` 并仅输出结构。
- **Json**：信封对象 pretty 序列化（`node_to_json` 本地 walker，不动
  auto-val crate）。
- **Atom**：逐节点 structure 的 Display 文本拼接（无头注释）。

## 3. VM/Iced 端交互

- **键位**：Alt+左键按下起笔（`GlobalPress` 臂，命中即吞事件抑制 WM 焦点
  抢占；编辑壳在途避让；v1 落 primary App），拖拽更新 marquee（根 Stack
  注入 canvas 蒙层，非 opaque 层命中穿透），松开定稿。
- **死区**：拖拽位移 < 4px 视同点击，不触发框选。
- **延迟计算**：松开只暂存 `pending_selection` + 置 `needs_bounds`；
  `__bounds_collected` 臂消费（保证 bounds 新鲜）→ 信封 → 打开 DevTools
  「采集」标签页（debug_mode/devtools_open 联动，`__toggle_inspect` 同款）。
- **面板**：Auto/JSON/Atom 三视图 chips + 复制按钮（`clipboard_set`，带
  成功/失败反馈）。Esc（app 级 Escape binding 优先）关面板并清 marquee，
  结果保留。
- **坐标系**：marquee（窗口逻辑坐标）与 LayoutCollector bounds 同空间，
  直接比较。desktop 多虚拟窗跨窗框选 v1 不支持（单 App/独立模式为目标）。

## 4. Vue 端（dev-only）

- **DOM 标记**（`AUTOUI_SELECT_MARKERS=1` 时 `node_to_html` 注入；产物
  构建不设 env，输出逐字节不变；快照测试同锚）：通用元素统一追加
  `data-auto-tag`（源 widget 关键字）、`data-auto-src`（源 .at 文件 stem，
  子件 span 归各自源文）、`data-auto-id="aura_N"`（有 debug_id 时）、
  `data-auto-span="off:len"`（有 span 时，字节口径）。
- **源码映射**：脚手架 `src/auto-sources.ts`——
  `AUTO_SOURCES: Record<stem, 全文>`，`auto run`/增量编译同步重写（内容
  hash 防抖）；overlay 资产 `src/auto-select/overlay.ts` 同点自愈刷新。
  `main.ts` 以 `import.meta.env.DEV` 动态引用，产物构建 tree-shake。
- **overlay**：Alt+mousedown（capture 阶段，抑制原生交互）起笔 → fixed
  蒙层 → mouseup 采集 `document.querySelectorAll('[data-auto-span]')` →
  中心包含 + parentElement 链顶层修剪（同 §1）→ 源映射键取
  `AUTO_SOURCES[data-auto-src]` 字节切片 + DOM 子树 JSON → 浮动面板
  （fixed 右下，Auto/JSON 切换 + `navigator.clipboard.writeText` 复制）。
  Esc 关面板。gallery/desktop 宿主多 App 页 v1 不接。

## 5. MCP 工具 `autoui_select_rect`

- 参数：`{x, y, w, h(必填, 窗口逻辑 px, w/h>0), format?: "auto"|"json"|
  "atom"(默认 "json")}`。
- 数据源：`autoui_vtree` 同源 styled_vtree 快照 + `layout_bounds`
  （`vnode_N` 键直解析——iced id 内嵌 VNodeId；`aura_*` 键无注册表不参与）
  + 随帧发布的 source_code（`SharedState::source_code`，
  `ensure_source_loaded` 幂等装载）。
- 返回：信封 JSON（默认，pretty）/ Auto 文本 / Atom 文本；缺参、非法
  w/h、无快照返回结构化 `isError`。
- 语义与交互框选完全一致（§1 同一纯函数）。

## 6. 布局 bounds 采集链（Plan 282 补线）

`needs_bounds` 的消费点在 app 消息尾部批处理臂；静默会话（无交互）只有
500ms 泵消息（走早退分支）——`__hot_reload` 泵臂代为消费（armed/消费同帧
闭环），MCP/DevTools 的 layout_bounds 在无交互会话照常落盘。

## 验证锚点

- 纯函数：`cargo t selection`（中心包含/边缘扫过不命中/顶层修剪/文档序/
  切片逐字节锚点/去公共缩进/synthetic 降级/JSON 字段完整）。
- 生成器：`cargo t plan646_vue_emits_data_auto_markers`。
- MCP：`autoui-verifier/scripts/test_vm_mcp.py`（信封字段 + 结构化错误）。
- Vue e2e：`test_vue_playwright.mjs` `altDrag`/`assertDataAuto`/
  `assertPanel` 动作（015-notes 实证：93 标记元素，面板切片=sidebar.at
  字节区间原文）。
