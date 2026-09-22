---
plan_id: PLAN-692
status: execution_done               # drafting → executing → execution_done → reviewed → archived
feature_name: widget-gallery-scroll-polish
author: [agent]
created_at: 2026-09-22
updated_at: 2026-09-22

# /auto-plan:review 结束时填写：
supersedes_spec_components: []
new_spec_components: []
touched_goals: []             # 引用 docs/specs/goals.md 的 GOAL-NNN

affects: [widgets/scroll-pane]
current_step: 6
total_steps: 7
plan_revision: 4
---

# [PLAN-692] widget-gallery-scroll-polish

## 0. 变更摘要

Vue 版 widgets-gallery（auto-os 仓）`#/scroll` 页实机走查（2026-09-22，PLAN-656 scroll 替换核实会话）发现四个 demo 呈现问题；用户同场对 `scroll` 组件提出三项视觉增强。本计划：

1. **Demo 重设计**（auto-os `widgets-gallery/src/front/pages/scroll.at`）：竖向 demo 容器加宽度根修塌条；横向/Both-Axes demo 保证真实溢出使滚动条出现。
2. **双路径滚动条统一**（auto-lang vue 脚手架）：语义 `scroll`（Reka ScrollArea）与 utility 路径（`overflow-*` 原生滚动条）统一到同一套视觉参数（宽 8px/药丸/`--border` 色/hover 加宽加深/active 高亮）；utility 路径自动挂 `.ash-scroll` 类。
3. **scroll 组件三项增强**：新 `size` prop（thumb 宽度 px）；thumb hover 左右各加宽 2px 且 `cursor: pointer`；thumb 按住时高亮色。

跨仓计划：auto-lang 为主仓（schema/生成器/脚手架模板/CSS），auto-os 为兄弟 worktree（demo 页源码）。**Vue 臂为验收面**；VM 臂保证不红 + `size` prop 兼容（实现或降级注记二选一，执行期裁定）。

## 1. 目标

- G1：widgets-gallery scroll 页四个 demo 在 Vue 臂呈现为"像样的大样例"——竖向容器全宽、横向/双轴 demo 滚动条实际出现（用户原话："div 做大一点"）。
- G2：`scroll` 元素与 `overflow-y-auto`/`overflow-x-auto`/`overflow-auto` utility 类产出的滚动条视觉一致（同一套：宽度、色、圆角、hover、active）。
- G3：`scroll` 组件支持 `size` prop 端到端（schema → 生成 .vue → DOM thumb 宽度）+ hover 加宽 2px/侧 + cursor:pointer + 按住高亮。

**非目标**：
- 不改 preview-card 生成器包装结构（`flex items-center justify-center` 保留；宽度在 demo 源码层解决）。
- 不动 VM 臂 iced 滚动条的视觉重设计（`renderer.rs:2727` scrollbar_style 仅按"size 兼容"最小处理）。
- 不处理 Firefox 原生滚动条的 hover 加宽（`scrollbar-width` 无此能力；Chromium 是验收浏览器）。
- native（utility 路径）thumb 的 `cursor: pointer` 为浏览器平台限制——若 `::-webkit-scrollbar-thumb { cursor }` 实测无效，cursor 仅 Reka 侧生效并在 specs 记边界（G2 的"同一套"以色彩/宽度/圆角/hover/active 五参数为准）。

**成功样貌**：浏览器打开 `#/scroll`，四 demo 全部出现应有滚动条且样式一致；`scroll (size: 16)` 生成 16px thumb；hover 变宽、按住变主色。

## 2. 架构方案

### 2.1 统一视觉参数表（canonical，双路径对齐基准）

| 参数 | 值 | Reka 侧（ScrollBar.vue） | native 侧（.ash-scroll CSS） |
|---|---|---|---|
| thumb 宽（缺省） | 8px | thumb 宽随 `size` prop，缺省 8px | `::-webkit-scrollbar { width: size+4px }`，thumb 视觉 = size（透明 border 2px/侧 + background-clip） |
| 轨道 | 透明 | track 无背景 | `track { background: transparent }` |
| thumb 色 | `hsl(var(--border))` | `bg-border` 药丸 | 同左 |
| hover | 加宽 +2px/侧 + 加深 `--muted-foreground` | thumb `w: size+4px` + `hover:bg-...` + `cursor-pointer` | `:hover { border-width: 0 }`（视觉 8→12px）+ `:hover { background-color }` |
| active（按住） | `hsl(var(--primary))` 高亮 | `active:` 主色 | `::-webkit-scrollbar-thumb:active` |
| 过渡 | 0.15s | `transition-all` | webkit 伪元素 transition 支持有限，不强求 |

原理：native 侧 scrollbar 总宽 = size+4，thumb 用 2px 透明 border + `background-clip: padding-box` 使视觉宽 = size；hover 时 border 归零 → 视觉加宽 2px/侧，**不需要**改变 scrollbar 布局宽度（webkit 不支持 `::-webkit-scrollbar-thumb:hover { width }`）。Reka 侧用 CSS 变量 `--sb-size` 驱动，`size` prop 写内联 `style="--sb-size: Npx"`，thumb/track 用 `w-[var(--sb-size)]` / `w-[calc(var(--sb-size)+4px)]` 任意值类。

### 2.2 utility 路径自动挂类

vue 生成器（`crates/auto-lang/src/ui_gen/vue.rs`）在 div/容器 class 组装处检测 style/class 串含 `overflow-y-auto`/`overflow-x-auto`/`overflow-auto` → 追加 `ash-scroll` 类。检测点：class 属性最终组装（`push_passthrough_attrs` / `generate_shadcn_attrs` 的 div 分支）；实现为对"最终 class 串"的一次性后扫描，避免逐 prop 判断。

### 2.3 `size` prop 管道

`schema/aura.at:1076` element scroll props 追加 `{ name: "size", type: "uint", default: "8", description: "Scrollbar thumb width in px (PLAN-692; hover widens +2px/side)" }` → vue.rs scroll 发射点（:12525）读 `size` → `<ScrollArea ... style="--sb-size: Npx">`（或 ScrollBar prop 透传，执行期择简）。S001 drift 检查以 schema 为准，demo 使用 `size` 不漂移。

### 2.4 Demo 重设计（auto-os scroll.at）

| Demo | 现状 | 改法 |
|---|---|---|
| Vertical（语义 scroll） | 容器无宽度 → 预览区 shrink-to-fit 塌成 ~20px 细条 | `scroll (style: "w-full h-40 border rounded-md p-2")` |
| Vertical（overflow-y-auto） | 同塌 | div style 加 `w-full` |
| Horizontal | 6×w-32 卡 = 816px < 预览区 ~900px → 不溢出无滚动条 | 卡片 6→10 张（10×128+9×8=1352px > 容器宽，必溢出） |
| Both Axes | 2 行 h-12（无竖溢出）+ w-160=640px（无横溢出）→ 双双无滚动条 | 行 2→5（5×48+4×8=272px > h-32 内高 112px）；网格 w-160→w-240（960px > 内容区宽） |

方格 `w-full` 在容器有宽后由 col 的 align-stretch 自然满铺，无需改子元素。

### 2.5 仓库与 worktree 布局（Plan 529 分组平铺）

- auto-lang：`D:/autostack/.wt/lang-692/auto-lang`（branch `plan-692-dev`）——schema、vue.rs、ScrollBar.vue/ScrollArea.vue 模板、单测。
- auto-os：`D:/autostack/.wt/lang-692/auto-os`（branch `plan-692-dev`，auto-os 仓同款分支命名惯例）——`widgets-gallery/src/front/pages/scroll.at`。
- 实机验收：worktree 构建 `cargo build`（auto.exe）→ 在 auto-os worktree 的 widgets-gallery 下 `auto run -r vue --server vm`（auto-os 无 rust 后端工程，VM HTTP 后端形态；`AUTOUI_MCP_PORT` 钉独立端口；注意本会话在 3024 已有一个在跑 dev server，验收前停掉或 `-F` 换端口）。
- **红线**：worktree 内禁 junction/symlink；auto-os worktree 首跑需 `pnpm install`（gen/ 全 gitignored，首次生成 + 装依赖）；gen pnpm junction 卡 wt-guard 用 `cmd /c rmdir /s /q` 清。

## 3. 技术栈

- AutoLang schema（`schema/aura.at`）、Rust（vue 生成器 `ui_gen/vue.rs`、脚手架 `crates/auto-man`、CSS 生成 `auto-man/src/vue.rs generate_css` 段 :2185-2204）。
- Vue3 + Reka UI（ScrollArea 组件）、Tailwind 任意值类、`::-webkit-scrollbar` 伪元素。
- 验收浏览器：Chromium（ZCode IAB）；后端 AutoVM HTTP（`--server vm`）。

## 4. 需求分析与背景调查

### 授权记录

- 用户 2026-09-22 会话内直接下达：四个 demo 问题修复 + 滚动条统一 + scroll 三项视觉增强，"记录到一个新的计划里，然后用 auto-plan-work 实施"。范围=本计划 §1；无额外预算/自动继续限制记录。

### 走查证据（2026-09-22，Vue 臂，auto-os widgets-gallery @ main 0fbe421）

1. 竖向两 demo 塌成 ~20px 细条：预览包装 `flex items-center justify-center p-4`（scroll.vue 生成物 :147），`scroll`/div 容器无宽度类 → flex item shrink-to-fit；方块 `w-full` 解析不出宽度。两路径（语义/utility）塌相一致，证明非 scroll 替换引入。
2. 滚动条两套样式：语义 scroll → Reka ScrollArea（无箭头药丸 thumb）；utility div → 原生 `scrollbar-width` 细滚动条带 ▲▼ 箭头（`.ash-scroll` 未挂）。
3. 横向 demo：6 张 w-32 卡不溢出 → 无滚动条（截图 2）。
4. Both Axes：2 行 h-12 + w-160 网格，双轴均无溢出 → 无滚动条。

### 代码勘定（master 检出 2026-09-22）

- `schema/aura.at:1076-1090`：element scroll，aliases `["Scroll","scrollable","scroll-pane"]`，props 含 axis/scrollbar/controller/onscroll/direction；vue 映射 `component: "ScrollArea"`。
- `crates/auto-man/assets/shadcn-ui/scroll-area/{ScrollArea,ScrollBar}.vue`：shadcn 快照模板（ScrollBar thumb `bg-border`，track `w-2.5`）。
- `crates/auto-lang/src/ui_gen/vue.rs:12525` scroll 发射点（orientation/axis、controller、scrollbar:hidden）；:74 组件表注记 scroll 支持 `class, orientation, hide_delay`——`size` 需登记进透传支持面。
- `crates/auto-man/src/vue.rs:2185-2204`：`.ash-scroll` / `.ash-scroll-fade` CSS 已存在（8px/`--border`/药丸/hover 加深），仅对显式挂类元素生效。
- `crates/auto-lang/src/ui/iced/renderer.rs:2727-2845`：VM 臂 scrollable 样式位（scrollbar_style()，固定宽度）。
- 生成物 `gen/front/vue/src/components/ui/scroll-area/` 由脚手架模板物化；拷贝是否 skip-if-exists 执行期验证——若 skip，改模板后须删 gen 组件目录强制重拷（风险 R-1）。
- 既有债务：`docs/plans/KNOWN-DEBT-AND-RISKS.md` 无 scroll 相关在册条目；`docs/specs/widgets/scroll-pane.md` 为 PLAN-656 沉淀的 scroll 契约（SD 修改目标）。

### 相关 Spec

- `docs/specs/widgets/scroll-pane.md`：scroll 语义与双端契约（SD-01 修改落点）。
- 本计划不改 preview-card 契约、不改 overflow-* utility 语义（只加视觉类）。

## 5. 详细设计

### 5.1 ScrollBar.vue 重做（auto-lang `crates/auto-man/assets/shadcn-ui/scroll-area/`）

- `ScrollBar.vue`：track 保持 Reka `ScrollAreaScrollbar`，宽度类改 `w-[calc(var(--sb-size,8px)+4px)]`（竖）/`h-[calc(...)]`（横），去实心 border-l 改透明占位；thumb 类 `w-[var(--sb-size,8px)] rounded-full bg-border transition-all cursor-pointer hover:w-[calc(var(--sb-size,8px)+4px)] hover:bg-muted-foreground active:bg-primary`（横臂对称 h-）。
- `ScrollArea.vue`：新增可选 `size?: number` prop（缺省 8），根元素内联 `:style="{ '--sb-size': (size ?? 8) + 'px' }"`；ScrollBar 无需显式传（CSS 变量继承）。
- 兼容性：Reka `ScrollAreaScrollbarProps` 不受影响；`scrollbar: hidden` 路径（vue.rs 内联 `scrollbar-width:none`）不变。

### 5.2 .ash-scroll 扩展（auto-man/src/vue.rs CSS 生成段）

```css
.ash-scroll::-webkit-scrollbar { width: calc(var(--sb-size, 8px) + 4px); height: calc(var(--sb-size, 8px) + 4px); }
.ash-scroll::-webkit-scrollbar-track { background: transparent; }
.ash-scroll::-webkit-scrollbar-thumb {
  background-color: hsl(var(--border));
  border-radius: 9999px;
  border: 2px solid transparent;          /* 视觉宽 = size */
  background-clip: padding-box;
}
.ash-scroll::-webkit-scrollbar-thumb:hover { border-width: 0; background-color: hsl(var(--muted-foreground)); }  /* 视觉 +2px/侧 */
.ash-scroll::-webkit-scrollbar-thumb:active { background-color: hsl(var(--primary)); }
.ash-scroll { cursor: default; }          /* native thumb cursor 平台限制，见 G3 非目标 */
```

（现 :2187-2191 规则并入；`--sb-size` 可由元素内联 style 提供，与 Reka 侧同参数源。）

### 5.3 vue.rs 生成器改动

1. **scroll 发射点**（:12525）：读 `size` prop（uint），`attrs.push(style="--sb-size: Npx")`（与既有 `scrollbar: hidden` 内联 style 共存合并）；组件表注记（:74）同步登记 size。
2. **utility 路径挂类**：div/容器 class 最终串组装处后扫描 `overflow-y-auto|overflow-x-auto|overflow-auto` → 追加 `ash-scroll`。只对非 `scroll` 标签的普通容器生效（scroll 元素走 ScrollArea，不重复挂）。
3. **单测**（vue.rs 内既有 `extract_classes`/`generate_shadcn_attrs` 测试模式，:21519 邻域）：
   - `scroll` + `size: 16` → 产出 `--sb-size: 16px`；
   - div + `overflow-y-auto` style → class 含 `ash-scroll`；
   - div + 无 overflow → 不含 `ash-scroll`（无误伤）；
   - `scroll` 元素本身 → 不含 `ash-scroll`。

### 5.4 scroll.at demo 重设计（auto-os）

见 §2.4 表。改动仅 `src/front/pages/scroll.at`；`square` 子元素不动。预期 Auto 代码面板同步展示新样式串（demo 自证）。

### 5.5 VM 臂兼容

- schema 加 prop 本身对 VM 臂非破坏（VM 按 schema 解析未知 prop 忽略）。
- `renderer.rs` scrollable 宽度若可从 props 取值则顺带实现（iced `scrollable::Scrollbar` width 字段）；若牵扯 >20 行改动则本计划只保证编译/回归不红，`size` 在 VM 臂记降级注记（specs SD-02），不阻塞验收。

### 规范增量

| delta_id | add/modify/retire | docs/specs/... target | before/after rule | rationale | acceptance IDs |
|---|---|---|---|---|---|
| SD-01 | modify | widgets/scroll-pane.md | scroll 新增 `size` prop（thumb 宽 px，缺省 8）；thumb hover 加宽 +2px/侧 + Reka 侧 cursor:pointer；active 主色高亮；vue 臂 utility 路径 overflow-* 自动挂 .ash-scroll，双路径统一视觉参数表（§2.1） | 用户裁定 2026-09-22 三项增强 + 统一需求 | AC-03 AC-04 AC-05 AC-06 |
| SD-02 | add | widgets/scroll-pane.md（VM 臂段） | VM(iced) 臂对 `size` 的支持形态（实现值或降级注记），执行期 T-06 裁定后回填 | 双端契约完整性 | AC-07 |

## 6. 测试设计

- **单测**（auto-lang worktree，Category B 门禁）：vue.rs 新增 4 断言（§5.3.3）+ 既有 scroll 断言（:21519 邻域）不回归；`cargo check -p auto-lang` + `cargo t vue`（或所在测试模块名，执行期以 `cargo t <mod>` 实际组为准）。若触及 auto-man：`cargo t -p auto-man` scoped（shadcn 模板/golden 面若存在 fixture 会吃改动——plan593_index_css.golden 邻域，执行期核对是否需更新 golden）。
- **实机验收**（双仓 worktree + 浏览器）：
  1. worktree 构建 auto.exe；
  2. auto-os worktree widgets-gallery 下 `AUTOUI_MCP_PORT=9352 auto run -r vue --server vm`（端口冲突则 -F 换）；
  3. 浏览器（IAB）打开 `#/scroll`：
     - 截图四 demo：滚动条全部出现、竖向容器全宽（AC-01/02）；
     - `evaluate` CSS 断言：两路径 thumb 计算样式同宽同色（AC-03）；
     - `scroll (size: 16)` 演示行（临时或在 properties demo 区加一行）→ DOM `offsetWidth` == 16（AC-04）；
     - hover/active 断言：伪类态用 CSS 规则存在性 + Reka thumb 类断言（运行态 hover 截图为辅）（AC-05/06）。
  4. 语义 scroll demo 实际拖动滚动一次（确认功能未破）。
- **VM 臂回归**：`cargo t iced`（若改 renderer.rs）；VM 窗跑一次 scroll demo 走查不红（可选，时间盒）。

## 7. 验收标准

- **AC-01**：`#/scroll` 竖向两 demo（语义 scroll / overflow-y-auto）容器全宽，8/6 根色条满铺可辨，无细条塌相。验证：浏览器截图 + DOM `clientWidth > 300`。
- **AC-02**：横向 demo 出现横向滚动条；Both Axes demo 同时出现横竖滚动条。验证：截图 + `evaluate` 检查 `scrollWidth > clientWidth`（横）/ `scrollHeight > clientHeight`（竖）。
- **AC-03**：两路径滚动条同套视觉：同宽（8px thumb/12px 轨）、同色（`--border`）、药丸圆角、hover 加宽加深、active 主色。验证：截图对照 + computed style 断言（thumb 背景 rgb 一致）。
- **AC-04**：`size` prop 端到端：demo 中 `scroll (size: 16, ...)` 生成 thumb offsetWidth ≈ 16px（±1px 容差）。验证：`evaluate` 断言。
- **AC-05**：Reka thumb hover 加宽 +2px/侧 且 cursor:pointer（CSS 类断言 + getComputedStyle cursor）；native 侧 hover 视觉加宽（border trick 规则在产物 CSS 中）。验证：规则断言 + 手动 hover 截图。
- **AC-06**：thumb 按住时高亮主色（Reka `active:bg-primary` 类 + native `:active` 规则在案）。验证：规则断言。
- **AC-07**：VM 臂回归不红：`cargo t iced` scoped 绿；`size` 兼容形态（实现或降级）已按 SD-02 记录。
- **AC-08**：auto-lang 门禁：`cargo check -p auto-lang` 零警告、`cargo t`（日档 scoped 至 ui_gen/auto-man 相关组）绿；gen golden 若吃改动已对齐。

## 8. 执行步骤

- **T-01** [x]（auto-lang worktree）schema + 模板：`schema/aura.at` scroll 加 `size` prop；`crates/auto-man/assets/shadcn-ui/scroll-area/{ScrollBar,ScrollArea}.vue` 按 §5.1 重做。→ AC-04/05/06；验证：`cargo check -p auto-lang` 绿。[✅ 已完成] commit f986a2140；设计修正：reka thumb 内联 `width/height: var(--reka-scroll-area-thumb-{width,height})`（长度比例变量），厚度侧变量未定义——用类内变量分层接管（纵向 width/横向 height），非纯类宽度（会被内联声明压制）。
- **T-02** [x]（auto-lang worktree）生成器 + CSS：vue.rs scroll 发射点传 `--sb-size`（与 hidden 合并同一 style 属性）、plain 装配点 overflow-* 容器挂 `ash-scroll`、auto-man CSS 生成段 `.ash-scroll` 统一参数表；组件表注记。→ AC-03；验证：`cargo check -p auto-lang` 绿。[✅ 已完成] commit f986a2140。注：passthrough 已跳过 `size`（Plan 412 square 尺寸族）无重复发射。
- **T-03** [x]（auto-lang worktree）单测：新增 `p692_utility_scroll_ash_scroll_class`（div+overflow 挂类/无 overflow 不挂/scroll 元素不挂三断言）+ size→`--sb-size: 16px`、size+hidden style 合并断言；既有 `scrollbar-width:none` 断言放宽为 `scrollbar-width`（发射改带空格）。golden 参照 `plan593_index_css.golden` 的 `.ash-scroll` 块同步。[✅ 已完成] `cargo nextest run -E 'test(p692...)+...'` 3/3 绿。预存红（master 同败基线，与本分支无关）：`test_a2vue_desktop_surface_asset`（a2vue 金样未同步）、`index_css_values_match_p1_baseline`（主题 token 漂移）。
- **T-04** [x]（auto-os worktree）demo 重设计：四 demo + Properties 表 size 行。→ AC-01/02；commit 20bf4dc。[✅ 已完成]
- **T-05** [x]（双仓 worktree）实机验收：worktree auto.exe 构建 → auto-os worktree `run -r vue --server vm`（MCP 9352）→ IAB 逐 AC。**实机修正两枚**（commit 396ab85）：横向 demo 容器 `shrink-0` 使其撑至内容宽 1368px 破版且不溢出 → 去 shrink-0 加 `w-full`；Both Axes 外层塌至 114px → 加 `w-full`；内网格 `w-160` 不在 Tailwind 标度（预存暗病，从未生效）→ `w-[60rem]`。终版断言：语义 demo cw=892 满铺+`--sb-size:8px`；utility 竖 898/横 908 hOver=true/双轴 898 双溢出=true；**size:16 临时验证**（后还原）thumb offsetWidth=16 端到端 ✓；thumb cursor=pointer/radius 9999/bg=--border/active:bg-primary 类在案；hover 加宽 16→20px 实测（禁过渡后 width=20px——headless 下 transition-all 因 style attr 重挂反复重启属环境伪象，var 级联机制已证）；语义区 scrollTop 持久化功能 ✓。[✅ 已完成] AC-01..06 全过。
- **T-06** [x]（auto-lang worktree）VM 臂：未动 renderer.rs（size 为 vue 臂视觉面；iced scrollbar 样式固定宽度，schema 纯增量兼容）。`cargo nextest run -E 'test(iced)'` 271/271 绿。SD-02 降级注记已入 spec。[✅ 已完成]
- **T-07** [~]（主检出，merge 阶段执行）账本：SD-01/02 已备于 worktree（commit 70231e2dd，`docs/specs/widgets/scroll-pane.md`）；`docs/specs/widgets/plans.md` 行 + INDEX 再生 + 归档按 merge 流程落。

依赖：T-01→T-02→T-03；T-04 独立可并行；T-05 依赖 T-01..T-04；T-06 依赖 T-01；T-07 最后。

## 9. 复审记录

- 2026-09-22 draft（rev1）：/auto-plan:new 起草完成。stage: new，PLAN-692 rev1。outcome: pass。next: work。授权=用户会话内直接下达（修复+统一+三增强，work 执行）。待澄清：无阻塞项（native 侧 cursor 平台限制已按非目标处理）。
- 2026-09-22 work（rev1）：/auto-plan:work 执行完毕。stage: work | plan_id: PLAN-692 | plan_revision: 1 | outcome: **pass** | code_commit: auto-lang `f986a2140`+`70231e2dd`（branch plan-692-dev，base bebd09387）；auto-os `20bf4dc`+`396ab85`（branch plan-692-dev，base 0fbe421）| task_ids: T-01..T-06 done，T-07 merge 阶段 | evidence: 三单测绿（size/合并 style/ash-scroll 三态）；`cargo check -p auto-lang` 绿；iced 271/271 绿；实机 IAB 断言 AC-01..06 全过（几何+thumb 16px 端到端+hover 20px 禁过渡实测+功能滚动）| blockers: 无 | next: review。计划内设计修正已记录（T-01 reka 变量分层接管；T-05 shrink-0/w-160 两枚实机修正）。预存红两枚 master 同败（a2vue 金样、index_css token 漂移）非本支引入。
- 2026-09-22 F-1（rev1，用户实机反馈修复）：用户报告语义 scroll thumb 拖拽方向/位置错乱。根因=ScrollBar.vue 轨道 `items-center` 把 thumb 静态布局原点在长度轴居中（纵轨 cross 轴即长度轴），而 reka 以 `transform: translate3d` 从布局原点位移定位 thumb（ScrollAreaScrollbarVisible.onThumbPositionChange），原点偏移实测 70px → 位移映射整体错位（grab 跳变+方向感错乱）。修复=轨道 `items-start justify-center`（长度轴起点+仅厚度轴居中；纵横两向同形），实测 originTopRelTrack 70→1px（commit 32ea85b66）。A/B 对拍证实 18px thumb 长度为预存现象（原模板同值，reka sizes 测量面问题，与本次改动无关，记观察项）。合成拖拽在 IAB 无法穿透 reka 的 setPointerCapture/emit 链（cua.drag 只发 move；合成 PointerEvent 的 capture 语义受限），终验以几何断言（原点=内容盒起点）+ reka 源码数学推演为准，真实鼠标拖拽待用户复核。
- 2026-09-22 rev2（用户裁定：active 高亮撤销改 hover 微亮）：用户手测后裁定——不做"按住后高亮"，与普通 scroll 一致改为 hover 时高亮，且非主题色大高亮而是轻微变亮。变更：thumb `active:bg-primary` 移除 → `hover:bg-muted-foreground/60`；`.ash-scroll` hover 改 `hsl(var(--muted-foreground) / 0.6)` 同参、`:active` 规则退役；hover 加宽 +2px/侧与 cursor:pointer 保留。**AC-05 改述**：hover 加宽 +2px/侧 + cursor:pointer + thumb 微亮（muted-foreground/60）。**AC-06 退役**（用户裁定 2026-09-22，非计划让步）。SD-01 相应改述（spec 增量在 worktree 于 merge 时同步落）。实机验证：compiled CSS 规则确认（hoverRule=新值，activeRuleCount=0）；禁过渡探针读 hover 真值 bg=rgba(148,163,184,0.6)+宽 12px。commit beebcb832（branch plan-692-dev）。plan_revision 1→2（AC 契约文本变化；授权=用户直接下达）。
- 2026-09-22 rev3（用户实机反馈三则）：①thumb 闪现根修——hover 挂载时 thumb 先现于轨道顶再滑到真实位置：`transition-all` 把 reka 挂载后施加 `transform: translate3d` 定位的过程也动画化了；改 `transition-[width,background-color]`（transform 排除，实测 transitionProperty="width, background-color"）。②可见色加 accent——纯灰过浅，底色 `bg-border`→`bg-[hsl(var(--primary)/0.4)]`（tailwind config 无 alpha-value 占位故用任意值写法；实测 rgba(147,149,246,0.4)）。③变粗时色相不变仅亮度自适应——hover 色 muted-foreground/60→`primary/60`（浅色底变暗、深色底变亮，透明度合成自然达成）；`.ash-scroll` 同参（底 primary/40、hover primary/60+border 归零加宽）+Firefox scrollbar-color 同步。commit 9193d9915+9cc0813eb（spec 同步）。A/B 复核：IAB 合成悬停下 reka 定位链（RO→sizes→watchOnce→transform）不完整为环境既有伪象（原模板同态）；真实鼠标链路通（用户截图 2 证 thumb 会落位）。plan_revision 2→3（视觉契约文本变化；授权=用户直接下达）。
- 2026-09-23 rev4（用户实机裁定）：hover 加宽撤销，thumb 恒宽——方向性 `hover:[--reka-*-thumb-*:+4px]` 变体移除；轨宽从 calc(size+6px)+p-px 简化为恒等 size（2px 透明 border 加宽技巧退役）；thumb 过渡收窄 `transition-colors`（width 不再参与）。hover 色深 primary/60、accent 底 primary/40、cursor:pointer、size prop 保留。实机验证：thumb 8px=轨 8px 恒等、origin 0、过渡族仅颜色、编译 CSS 加宽产物零残留（reka 底/hover 两规则在案）。commit 3bba2fd77+4068c86fb（spec 同步）。plan_revision 3→4（授权=用户直接下达）。

## 10. 待澄清事项

- 无阻塞。两处执行期裁定已在案内预留：R-1 gen 组件 skip-if-exists 需强制重拷；VM 臂 size 实现 vs 降级（SD-02 回填）。
