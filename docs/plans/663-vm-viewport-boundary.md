---
plan_id: PLAN-663
status: drafting               # drafting → executing → execution_done → reviewed → archived
feature_name: vm-viewport-boundary
author: [agent]
created_at: 2026-09-20
updated_at: 2026-09-20

# /auto-plan:review 结束时填写：
supersedes_spec_components: []
new_spec_components: []
touched_goals: []             # 引用 docs/specs/goals.md 的 GOAL-NNN

affects: [widgets/project.md]
current_step: 0
total_steps: 6
---

# [PLAN-663] vm-viewport-boundary

## 变更摘要

VM/iced 臂引入 **视口边界（viewport boundary）语义**：`h-screen`/`min-h-screen`/`w-screen`/视口单位（svh/lvh/dvh）在"定高嵌入容器"子树内重锚定到边界尺寸（iframe 语义），窗口根缺省行为不变。修复 ui-gallery VM 臂内嵌全屏 demo（020-music-player、027-file-manager 等）塌缩为一条播控条的回归。附带交付 **C2 垂直安全居中**（`my-auto`/`m-auto`，Tailwind 惯用居中写法）作为后续嵌入宿主的声明通道。T-12 Shrink 兜底不退役，与新语义组合使用（裁定见详细设计 §裁定-1）。

## 目标

- **G1**：VM/iced 解释臂中，`h-screen`（及 svh/lvh/dvh/min-h-screen/w-screen）在定高容器子树内解析为该定高，而不是无锚点的 Fill（塌缩根因）。
- **G2**：无定高祖先时（独立窗口根）行为与现状逐字节等价（Screen→Fill=满窗）。
- **G3**：ui-gallery VM 臂桌面/平板档：全屏类 demo 铺满视口 frame（播控条贴底、侧栏/舞台可见），自由尺寸 demo 居中形态不回归。
- **G4**：`my-auto`/`m-auto` 在定高列内实现安全居中（矮内容居中、超高内容顶对齐），供后续嵌入宿主用纯 Tailwind 声明（`overflow-y-auto` frame + `m-auto` wrapper）。
- **非目标**：不改 Vue 臂（web 臂已有 `.h-screen→100%` 覆盖，工作正常）；不改 iced 版本；不退役 PLAN-642 T-12 Shrink 兜底；不做 Fill（h-full）的边界重锚定（见 §待澄清 1）。

## 架构方案

**事故链条（本计划的靶心）**：gallery frame（`AppViewport.vm.at:43`，`h-[720px] overflow-hidden justify-center`）→ 渲染器 `apply_column_style` 对 justify 列让渡列高（renderer.rs:1651 附近，列自身 Shrink）+ T-12 兜底把内容包进 Shrink 高 Scrollable（renderer.rs:1664-1690）→ demo 根 `h-screen` 在 VM 侧解析为 `SizeValue::Full`→`Length::Fill`（class.rs:1943 把 `"full"|"screen"` 合并解析！），Fill 嵌在 Shrink 祖先下退化为最小内容高度 → `flex-1` 主区归零，只剩 Controls 自然高度，被居中 → 截图症状。

**方案：渲染前 pre-pass 重写（而非渲染器穿线 context）**。理由：`Style` 是类表（`Vec<StyleClass>`），尺寸以 `StyleClass::Height(SizeValue)` 承载；AbstractView 树在渲染前是纯数据，仓里已有同类树遍历先例（`inherit_text_color` renderer.rs:6321、`axis_fix_col_child` 等）。pre-pass 可独立单测，且天然覆盖 Plan 319 双入口（`render_dynamic_view` 与 `into_iced`）——在各自根入口调一次即可，零渲染器签名改动。

1. **解析层区分（C1a）**：`SizeValue` 新增 `Screen` 变体；`parse_size_value` 中 `screen|svh|lvh|dvh → Screen`（`full` 保持 `Full`）。`IcedSize` 新增 `Screen`；adapter 映射 `SizeValue::Screen → IcedSize::Screen`；`iced_length(Screen) → Length::Fill`（窗口根兜底=现状）。`min-h-screen` 走既有 `MinHeight(f32::MAX)` 标记（class.rs:1346 已区分），无需新变体。
2. **边界重写 pre-pass（C1b）**：新函数（暂名 `rewrite_viewport_units`）递归遍历 AbstractView：某节点样式含**定高**（`Height(SizeValue::Pixels(u16)|Pixels(f32)) > 0`）即建立边界（宽侧对称：定宽 → 锚定 `Width(Screen)`）；子树内 `Height(Screen) → Height(Pixels(边界高))`、`MinHeight(f32::MAX) → MinHeight(边界高)`、`Width(Screen) → Width(Pixels(边界宽))`；遇到**嵌套边界**（后代自身定高）切换锚点。overflow 类不作为边界条件（iframe 语义：定高嵌入即视口，与 overflow 无关）。
3. **入口接线（C1c）**：在 `render_dynamic_view` 的两处根调用（renderer.rs:20237、20616）与 `into_iced` 根入口各调一次 pre-pass。
4. **C2 安全居中**：解析 `my-auto`/`m-auto`（垂直 auto-margin；水平 auto 已有 PLAN-619 先例）；`apply_column_style` 对**无 overflow 的定高列**中携带垂直 auto-margin 的子项以上下 `Length::Fill` spacer 三明治包裹（矮内容=均分居中；超高=spacer 压 0、内容顶对齐被裁剪——即 CSS safe center 语义）。带 overflow 的列不套用（滚动交互由 T-12 现行机制承接，见裁定-1）。

**裁定-1（T-12 处置）**：不退役。C1 落地后 T-12 的 Shrink 包裹与 `Screen→Fixed(边界)` 组合恰好成立（Fixed 不受 Shrink 塌缩影响：demo 根 Fixed(720) → T-12 scrollable ideal=720 → frame 居中=恰好满帧）；退役需要 iced 0.14 scrollable 的安全内容对齐能力（未验证存在），收益（去 CSS 偏差债）不抵风险。退役决策挂 KNOWN-DEBT，待 iced 升级或 gallery 第二嵌入宿主出现时重裁。

**执行裁定登记（置顶请求裁决项——按既有授权先行，复审时复核）**：
- 边界条件不含 overflow（更贴 iframe 语义；若复审认为应含，pre-pass 加一个谓词即可，不影响其余设计）。
- `h-full`（`SizeValue::Full`）不重写：Full 在 demo 内部非根元素上合法引用局部父高，全局重写会破坏内部布局；全屏外壳的仓内惯例恰是 `h-screen`（020/027 实证），语义通道已存在。

## 需求分析与背景调查

- **授权范围**：用户 2026-09-20 会话裁定"直接做 C"（视口边界 + 安全居中 + T-12 处置裁定），即本计划全部范围。仓：auto-lang（实现）；auto-os（仅验收运行 ui-gallery，零提交）。
- **事故现场**：`D:/autostack/auto-os/ui-gallery/src/gallery/AppViewport.vm.at:43`（frame 样式）+ `:45`（662 内层 `overflow-y-auto` 列）；demo 适配器 `demos/020-music-player.at` 根 `w-full h-screen` + 主区 `flex-1 min-h-0`。Plan 662 的修复（frame 内滚动容器）只验收了"控制条可见"，未解决 Fill 塌缩。
- **代码事实**（master 4cbc810eb 实证）：
  - `crates/auto-lang/src/ui/style/class.rs:10`（`SizeValue` 枚举）、`:1943-1945`（`full|screen|svh|lvh|dvh` 合并 → `Full`，本计划的解析靶点）、`:1341-1346`（min-height：screen 族 → `f32::MAX` 标记）。
  - `crates/auto-lang/src/ui/style/iced_adapter.rs:293-300`（`IcedSize`）；`renderer.rs:1382`（`SizeValue::Full → IcedSize::Full`）；`renderer.rs:24361-24369`（`iced_length`）。
  - 渲染消费点：`renderer.rs:2504-2532`（宽高 → iced Length；`min_height >= 9999.0 → Fill` 于 `:1639-1645`）、`apply_column_style`（`:1604` 起，justify 让渡列高 + T-12 兜底 `:1664-1690`，注释自证"画廊根 h-screen 实测整页塌缩"）。
  - 渲染入口：`render_dynamic_view`（`:23591`）根调用 `:20237`/`:20616`；`into_iced`（`:3283`，Plan 319 双入口）。
  - 树遍历先例：`inherit_text_color`（`:6321`）、`view_classes_mut`（`:1094`）。
  - codegen 侧（ark/jet/tailwind.rs:589）`h-screen` 本就独立于 `h-full`（`Size::Screen`→100%/fillMaxHeight）——本计划使解释臂与 codegen 对齐。
- **Spec 现状**：`docs/specs/widgets/project.md` 含样式/tailwind 语义记载，为本计划规范增量落点；无专门"布局语义"spec，SD 在该文件增节。
- **约束**：`cargo fmt` 全仓分叉不可整跑（仓工具链雷区）；worktree 红线（禁 junction）；Layout 语义改动回归面大 → golden 资产 = `D:/autostack/auto-os/ui-gallery/src/front/tests/screenshots/inv_*.png` 清单 + `cargo tf` 终局门禁。

## 详细设计

### C1a 解析层

- `class.rs` `SizeValue`：新增 `/// 视口单位(screen/svh/lvh/dvh)。窗口根=Fill(满窗)；定高嵌入边界内=边界高(iframe 语义,PLAN-663)。` `Screen,`。
- `parse_size_value`：`"full" => Full`；`"screen" | "svh" | "lvh" | "dvh" => Screen`（Plan 527 T3 注记保留）。
- `iced_adapter.rs` `IcedSize`：新增 `Screen`；`SizeValue → IcedSize` 映射（renderer.rs:1382 附近）加 `SizeValue::Screen => IcedSize::Screen`。
- `iced_length`：`IcedSize::Screen => iced::Length::Fill`（窗口根缺省=现状，逐字节兼容）。
- 显式消费 `Height(SizeValue)`/`Width(SizeValue)` 的 match 臂（exhaustive 受益于编译器）：Screen 落 Fill 同 Full。

### C1b pre-pass

```text
fn rewrite_viewport_units(root: &mut AbstractView<M>)
  walk(node):
    if let Some((bw, bh)) = definite_size(node.classes):   // 定宽/定高(任一即该轴边界)
        rewrite_subtree(node_children, anchor=(bw, bh))     // 子树内重写,遇嵌套边界换锚
    else:
        for child in node: walk(child)

fn rewrite_subtree(nodes, anchor):
  for node in nodes:
    if definite_size(node.classes).is_some(): continue  // 嵌套边界:自身不动,由 walk 主循环接管其后代
    node.classes: Height(Screen) → Height(Pixels(anchor.h as u16))
                  MinHeight(f32::MAX) → MinHeight(anchor.h)
                  Width(Screen) → Width(Pixels(anchor.w as u16))
    rewrite_subtree(node.children, anchor)
```

- 定高判定：`StyleClass::Height(SizeValue::Fixed(u16))` 或 `Pixels(f32)` 且 >0（`h-[720px]` → `Pixels(720)`；Tailwind spacing `h-44` → `Fixed(44)=176px`？——**注意**：spacing 单位定高（`h-44` 等）也是 CSS 意义上的定高，iframe 内同样锚定；但为控制爆炸半径，本计划**仅锚定显式 px（arbitrary）定高**（`h-[Npx]`），spacing 定高不建立边界（登记 KNOWN-DEBT，gallery frame 均为 px 形态，不受影响）。
- `definite_size` 同时返回宽/高两轴，各自独立成立（`w-[1024px]` 锚宽、`h-[720px]` 锚高）。
- 变体类（hover: 等）不参与边界判定/重写（尺寸类不进变体，实证于 parse 双轨）。

### C1c 入口

- renderer.rs `:20237`、`:20616` 两处根调用前对 `converted` 调 pre-pass；`into_iced` 根入口（执行时定位精确调用点，Plan 319 同口径）。

### C2 my-auto

- parse：`my-auto`/`m-auto` → `StyleClass` 新变体（暂名 `MarginYAuto`/`MarginAllAuto`——与既有水平 auto 表示对齐，执行时按 IcedStyle 既有字段形状定）。
- `apply_column_style`：列定高且无 overflow 类时，对带垂直 auto-margin 的直接子项以 `spacer(Fill)` 上下三明治替换原位置（多子项场景仅处理声明项自身——gallery/宿主 wrapper 是单子项形态，多子项为 KNOWN-DEBT）。
- Row 主轴（水平）对称臂不在本计划内（水平 auto 已有 PLAN-619）。

### 规范增量

| delta_id | add/modify/retire | docs/specs/... target | before/after rule | rationale | acceptance IDs |
|---|---|---|---|---|---|
| SD-01 | add | widgets/project.md（布局语义节） | before: VM 臂 h-screen≈h-full→Fill，嵌入定高容器内塌缩为最小内容高；after: 视口单位在定高(px)嵌入边界内重锚定为边界高，窗口根不变 | iframe 语义对齐；修 gallery VM 臂塌缩 | AC-01/02/03 |
| SD-02 | add | widgets/project.md（同节） | before: 垂直 auto-margin 未实现；after: my-auto/m-auto 定高列内安全居中（超高顶对齐） | CSS safe center 对齐，嵌入宿主纯 Tailwind 声明通道 | AC-07 |

## 测试设计

- **单测（parse）**：`h-screen`/`min-h-screen`/`h-dvh`/`h-svh` → `SizeValue::Screen`/`MinHeight(MAX)`；`h-full` → `Full`（class.rs 既有测试块追加）。
- **单测（pre-pass）**：构造 AbstractView：frame `h-[720px]` + 子 `h-screen` → 重写为 `h-[720px]`；嵌套边界换锚；无定高祖先不重写；`w-screen`/`min-h-screen` 各臂；px-only 判定（`h-44` 不建边界）。
- **布局测试**：`layout_tests.rs` 风格——frame+demo 组合经 pre-pass 后 builder 产物尺寸断言（720 满高、Controls 贴底）。
- **端到端（截图验收）**：worktree 构建 `auto` 二进制 → `D:/autostack/auto-os/ui-gallery` 跑 VM 臂 → desktop_mcp 截图：020（桌面/平板）、027、001（居中不回归）；020 独立 `run -r vm` 满窗不回归。与 vue 臂同 demo 对拍结构。
- **回归门禁**：`cargo check -p auto-lang` → `cargo t`（在册红口径）→ `cargo tv`（renderer/语料 golden）→ 终局 `cargo tf`。

## 验收标准

- **AC-01** 解析区分：`h-screen`/`h-dvh` 解析为 `SizeValue::Screen`，`h-full` 为 `Full`；`iced_length(Screen)==Fill`。验证：单测绿。
- **AC-02** 边界重写：定高(px)容器子树内 Screen 族→Fixed(边界)；嵌套边界换锚；无定高祖先不动。验证：pre-pass 单测绿。
- **AC-03** 画廊 020 桌面档：VM 臂截图显示侧栏+舞台+播控条全量、播控条贴 frame 底、无上下空带。验证：desktop_mcp 截图人工判读 + 与 vue 臂结构对拍。
- **AC-04** 画廊 027 同 AC-03。
- **AC-05** 自由尺寸 demo（001）居中形态不回归；平板档 020 满屏（1024 高）。验证：截图。
- **AC-06** 独立窗口 020 `run -r vm` 满窗布局与 master 行为一致。验证：截图对拍。
- **AC-07** C2：定高列内 my-auto 矮内容居中、超高内容顶对齐。验证：布局/单测绿。
- **AC-08** 门禁：check/t/tv 绿（在册红归因）、tf 终局跑通（在册红差集为空）。验证：命令输出留档。

## 执行步骤

- **T-01** C1a 解析层（class.rs + iced_adapter.rs + renderer.rs 映射/iced_length 两臂）+ parse 单测。【AC-01】验证：`cargo t style` / `cargo check -p auto-lang`。
- **T-02** C1b pre-pass 函数 + 单测（renderer.rs 内，沿 inherit_text_color 邻位落码）。【AC-02】验证：`cargo t iced::` 相关滤串。
- **T-03** C1c 双入口接线。【AC-02】验证：`cargo check` + 单测仍绿。
- **T-04** C2 my-auto（parse + 列臂 + 测试）。【AC-07】验证：`cargo t`。
- **T-05** 端到端验收（构建 worktree 二进制 → auto-os gallery VM 臂截图 020/027/001/平板/独立窗；对照 vue 臂）。【AC-03..06】
- **T-06** 门禁（check/t/tv/tf）+ KNOWN-DEBT 登记（T-12 退役决策、spacing 定高边界、多子项 my-auto）+ SD 回写 + 复审交接。【AC-08】

依赖：T-01→T-02→T-03→T-05；T-04 独立可并行；T-06 收口。

## 复审记录

（draft handoff）stage: new，PLAN-663 r1。outcome: pass——任务覆盖全部 AC 与 SD；路径/命令已对照 master 4cbc810eb 实证；两处执行裁定（边界不含 overflow、h-full 不重写）已置顶登记，复审时请求裁决。next: work。

## 待澄清事项

1. `h-full` 根 demo 在嵌入场景的语义（CSS 里 100% 在 scroll 容器内解析到 scrollport）——本计划不重写 Full，登记 KNOWN-DEBT；若未来语料出现 `h-full` 全屏外壳，再裁。
2. spacing 定高（`h-44`）是否建立边界——本计划 px-only，见详细设计。
3. iced 0.14 scrollable 是否具备安全内容对齐（决定 T-12 退役可行性）——不阻塞本计划，挂账。
