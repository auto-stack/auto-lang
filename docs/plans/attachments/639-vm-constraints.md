# PLAN-639 T-07：VM 三约束调查与决策工件

> 状态：调查产物（2026-09-17，plan-639-dev）。**不含修复实现**——修复本体
> 另立项（建议名 vm-component-parity）。原始登记：Plan 449 §3.1–3.3
> （`docs/plans/archive/449-auto-edit-componentization.md`）、DEBTS "449 vm
> 组件渲染三缺口" 行、ui/overview.md:217。
>
> 本文在 449 文献基础上做了 **2026-09-17 代码面复核**（最小复现探针实跑，
> 探针代码随文附录，未入库），并给出修复方案选项与工作量级估计。

---

## 约束一：回调 props 声明即编译失败（449 §3.1"回调 props 退化"）

### 现象（449 文献）
组件签名加 `on_xxx: msg` 即整体退化为空 fallback——**声明即可破坏，无需调用**。
015-notes 的 NavTree/EditorPanel 实测同为 fallback。

### 2026-09-17 复核（探针 3.1）
```auto
widget Child {
    msg { Ping }
    props { on_ping: msg }
    view { col { button "ping me" { onclick: .Ping } } }
    on { .Ping -> { } }
}
widget Host { view { col { Child (on_ping: .Ping) {} } } on { .Ping -> {...} } }
```
**结果：编译中止（20 errors：undefined variable + 括号级联）**——比 449 记载的
"空 fallback" 更早爆掉：`props {}` 块中的 `msg` 类型标注在 UI 场景 parser/
extractor 链即不被接受。规避式（013 式"组件无 props、读 .store"）仍为唯一通路。

### 根因面
`AuraProp` 类型系统（`aura/types.rs` props: Vec<AuraProp>）无 msg/回调类别；
`props { on_ping: msg }` 的 `msg` 落到类型解析即失败。事件回传通道现存实现是
`record_child_callback_routes_for`（aura_view_builder:5798）——基于**调用位
events**（`Child {}` 子位的 onclick 覆盖）而非 props 声明。

### 修复方案选项
| 方案 | 内容 | 工作量 |
| --- | --- | --- |
| A. props 类型面补 msg 类别 | `AuraProp` 增加 `Msg` 类别；parser 接受 `on_xxx: msg`；render_child_widget 把回调 props 映射为 child 的 handler 别名（调用位 events 同路） | M（parser+类型+render 三点；回归面 = 全部组件渲染测试） |
| B. 弃 props 声明、走调用位覆盖语义化 | 不补 `props {}` 语法；把调用位 `Child { onclick: .Ping }` 覆盖正式化为**唯一**回调通道并成文（现状半可用），`props {}` 中 msg 显式报错 + 迁移提示 | S（报错+文档；无新能力） |
| C. 经 bind 工件回避（PLAN-639 L1） | 需要回调定制的组合场景用 L1 bind 工件在应用侧装配（bind 内 onclick 直指应用 handler），bp 组件保持无回调 props | S（机制已在 T-06 落地；无编译器改动） |

**建议**：B 短期成文（消歧）+ C 承接平台消费；A 待组件化需求真实回流再立项。

## 约束二：组件子树对 MCP 快照的可见性（449 §3.2）

### 现象（449 文献）
组件内按钮/文本在 autoui_snapshot 中不存在（渲染/派发正常，快照只走根模板）；
aura_view_builder.rs 审计注释佐证（013 B12(iii)）。

### 2026-09-17 复核（探针 3.2）
**运行时 View 树已包含组件子树**（探针：宿主+子组件，`view_with_debug()` 的
View 含 "child button"）——`render_child_widget` 内联子视图 + Plan 476 slot
透传 + 437 Phase 2 子组件 Init 之后，渲染面已与 449 时代不同。
**MCP 快照路径（autoui_snapshot 的遍历器）未重测**——需 `test_vm_mcp.py`
实机核对快照输出是否随之覆盖子树。可能性：快照遍历器已随运行时修复
（快照读 View 树）或仍走根模板（需修遍历器）。

### 修复方案选项
| 方案 | 内容 | 工作量 |
| --- | --- | --- |
| A. 快照遍历器改读运行时 View 树 | 若仍走根模板：遍历 `view_with_debug()` 产物（含 DebugIdMap 定位），子树天然可见 | S–M（单点遍历器 + MCP 快照 golden 回归） |
| B. 仅实测确认 | 先跑 test_vm_mcp.py 核对现状，已是修复态则销行 DEBTS | S（半日实测） |

**建议**：先 B 后 A。DEBTS 行改写为"待实测"而非"确定缺陷"。

## 约束三：参数化条件在组件视图不求值（449 §3.3）

### 现象（449 文献）
TabItem 片段实测：渲染树中 onclick 实参、条件样式、if 分支全缺失（仅 vue 轨
验证过）。449 规避：tab 条回退基线双分支形态（`.store.` 读取本身可用）。

### 2026-09-17 复核（探针 3.3 / 3.3b）
- 3.3（表达式 props `label: it.name, is_active: it.name == .active`）：
  **子视图整片缺失**（label a/b 均不在渲染树、条件 "*" 计数 0）。
- 3.3b（字面量 prop `label: "literal-a"`，子视图 `text .label`）：
  **同样缺失**——指向两种可能：①props → 子视图 state 通道断（
  prepare_child_render_state 的 resolved_props 未接到 `text .expr` 求值链）；
  ②探针混淆——裸 `text .label`（无 props 块的表达式文本）本身可能是
  不支持形态（S001 事件的反面：表达式文本需 f-string 形态）。
  **修复立项须先跑区分探针**：`text f"${.label}"` / `text .label {}` 两形态。

### 根因面
`prepare_child_render_state`（aura_view_builder:5571）把 props 解析进 child
state 对象——但子视图 build 时 `text .expr` 的求值走 resolve_expr_to_value/
bindings 链，props 种子是否进入该链、以及 ForLoop 实参（it.name）在
`current_loop_var` 上下文外的解析，均为断点候选。

### 修复方案选项
| 方案 | 内容 | 工作量 |
| --- | --- | --- |
| A. props 种子接通子视图求值链 | child build 时把 resolved_props 并入 override state（含 loop-var 上下文），补三形态探针为回归测试 | M（builder 单点+回归；风险=现依赖"同名父 state 同步"的组件行为漂移） |
| B. 仅支持字面量 props（MVP） | 字面量标量 props 落 child state；表达式 props 显式报错+指引（用 bind 工件/store 通道） | S |
| C. store 通道绕行（现状规避成文化） | 参数化数据经 `.store.` 读——449 已证可用 | S（零改动，文档化边界） |

**建议**：先区分探针定性（3.3b 混淆排除），再 B 起步、A 立项。

## 附：工作量级与优先级总览

| 约束 | 现状定性 | 建议动作 | 级别 |
| --- | --- | --- | --- |
| 3.1 回调 props | 编译失败（比 449 更早爆） | B 成文消歧 + C bind 承接；A 缓立项 | 中 |
| 3.2 快照子树 | 运行时树已含子树；MCP 路径待实测 | B 实测 → A（若需） | 低–中 |
| 3.3 参数化条件 | 表达式+字面量 props 均缺（有探针混淆待排除） | 区分探针 → B 起步 A 立项 | 中高 |

**与 449 记载的差异**：①3.1 从"空 fallback"恶化为"编译中止"；②3.2 运行时
面可能已被 437/476 顺带修复（仅剩快照遍历器存疑）；③3.3 面貌未变但探针
提示需先排除 `text .expr` 形态混淆。

## 附：探针代码（复现用；未入库）

三个探针均为 tempfile 双文件（host.at + child.at）→ `build_dynamic_component`
→ `view_with_debug()` 断言/打印，见本计划执行会话记录（t07_probe_tests.rs，
2026-09-17）。复现要点：
- 3.1：child.at `props { on_ping: msg }` → 编译中止 20 errors。
- 3.2：host+child 正常编译 → 运行时 View 含 "child button" ✓。
- 3.3：`TabItem (label: it.name, is_active: it.name == .active)` → 渲染树无
  label、无条件 "*"（star_count=0）。
- 3.3b：`TabItem (label: "literal-a")` + `text .label` → 渲染树无 "literal-a"
  （混淆候选：裸 `text .expr` 形态本身）。
