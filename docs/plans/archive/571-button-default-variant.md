---
plan_id: PLAN-571
status: archived               # drafting → executing → execution_done → reviewed → archived
feature_name: button-default-variant
author: []
created_at: 2026-09-06
updated_at: 2026-09-06

# /auto-plan:review 结束时填写：
supersedes_spec_components:
  - "docs/design/autoui/base-styles-and-visual-parity.md: 修改（Design 22 §1.2 default=UA-stylesheet 等价物声明；§3 button default/primary 分行 + secondary/muted 分档）"
  - "docs/specs/auto-lang/ui/overview.md: 修改（ui 样式子系——button 缺省变体语义与 --secondary 令牌分档沉淀）"
new_spec_components:
  - "docs/specs/auto-lang/ui/architecture.md: 新增——button variant/size preset 单源契约（ui::style::variants 单源 + ui_gen 模板/auto-man 烘焙资产/auto CLI 模板 + CSS 变量层四方互锁测试锚）"
touched_goals:
  - "goal-007: AutoUI 跨端视觉一致——button 三臂缺省皮肤与 CSS 变量层收敛（default 中性基线/primary 显式 CTA/secondary 深一档），VM 与 Vue 截图同屏实证"

affects: [auto-lang/ui, auto-lang/ui_gen, docs/design/autoui]
current_step: 10
total_steps: 10
---

# [PLAN-571] button-default-variant

## 变更摘要

`button` 的缺省 variant 从"primary 填充的别名"升级为一等 `default` variant——中性基线按钮。
设计动机：Web 端裸 `<button>` 有 UA stylesheet / CSS 预填兜底，VM(iced) 端没有任何等价物，
因此必须显式定义 `default` variant 承载"未声明样式时的基线观感"，三臂（Vue / VM / Rust
transpile）同表同观感，文档可解释。`primary` 随之成为显式 variant（醒目 CTA 专用）。

三处同步：① `ui/style` 新增共享 preset 表（单一事实源）；② VM 臂 `convert_button` 与
Vue 臂 cva 表改弦；③ Rust transpile 臂补齐 preset 注入（顺带修复 PLAN-571 前发现的
"transpile 臂无 preset"偏离——031-image-viewer 双端截图差异的根因之一）。

随带修订（用户定调 2026-09-06）：`secondary` 语义色当前与 `muted` 完全同值（两端皆然，
系忠实继承 shadcn 默认主题 secondary==muted==accent 的灰），default 落在 muted 后
secondary 必须拉开一档，否则 default/secondary 两 variant 视觉坍缩。双端 token 同步
分化（vue index.css 模板 + theme.rs）。

## 目标

1. `default`（缺省）= 中性基线按钮：muted 填充 + `border-border` 发丝描边——即 Web UA
   预填裸 button 的完整形态（灰底+边框+圆角），在任意表面（含 dark 的 background/
   card，对比度仅 ~1.15:1 的场景）都保有按钮可辨识度。
2. `primary` = 显式醒目 variant：主题色填充（即现缺省观感），留给 CTA/主操作。
3. 三臂 variant preset 同源同观感：Vue（cva）、VM（解释器）、Rust（transpile codegen）。
4. preset 表单一事实源 + Design 22 §3 规约表同步修订。
5. 存量 examples 缺省按钮从主题色填充变为中性填充——这是预期方向的视觉回归，需盘点可控。
6. `secondary` 与 `muted` token 分档：secondary 变为"比 muted 强一档"的中性灰，双端一致。

## 架构方案

**共享 preset 表**：新增 `crates/auto-lang/src/ui/style/variants.rs`，
`pub fn button_variant_preset(variant: &str) -> &'static str` 与
`pub fn button_size_preset(size: &str) -> &'static str`。VM 臂（aura_view_builder）与
Rust codegen 臂（ui_gen/rust）直接调用；Vue 臂的 cva 是生成到前端工程的文本，无法共享
Rust 代码，以注释互锁 + 单测断言三表字符串一致（防漂移）。

**目标 variant 表（核心设计决策）**：

| variant | preset 类 | 观感 |
|---|---|---|
| 缺省 / `"default"` | `bg-muted border border-border text-foreground font-medium rounded-md hover:bg-muted/70` | 中性填充+发丝描边（UA 预填等价形态，任何表面可辨） |
| `"primary"` | `bg-primary text-primary-foreground font-medium rounded-md hover:bg-primary/90` | 主题色填充（现缺省观感），留给 CTA |
| `"submit"` | 同 primary | 行为语义：表单主操作应醒目 |
| `"secondary"` | `bg-secondary text-secondary-foreground font-medium rounded-md hover:bg-secondary/80` | 深一档纯填充、**无边框**（边框是 outline 的专属语言） |
| `"destructive"` / `"outline"` / `"ghost"` / `"link"` / `"icon"` | 现状保留 | 不变 |
| `"text"` / 未知 | 无 preset（chromeless，由 user class 主导） | 不变 |

size preset 不变：`sm→h-9 px-3`、`lg→h-11 px-8`、`icon→h-10 w-10`、缺省→`h-10 px-4`。

**Token 事实与 secondary 分档**：当前主题 `Muted` 与 `Secondary` 的 RGB 完全相同（dark
(30,41,59) / light (240,235,226)，见 theme.rs resolve_semantic_rgb），Vue 端 index.css
模板同构（`ui_gen/vue.rs:15206-15235`，light `--secondary`==`--muted` `210 40% 96.1%`、
dark 均为 `217.2 32.6% 17.5%`）——系忠实继承 shadcn 默认主题 secondary==muted==accent
共用同灰的设计。default 落在 muted 后此设计不再成立（default/secondary 两 variant 视觉
坍缩），secondary 分化为"强一档"：

| 模式 | muted（不动） | secondary（新值，执行期以双端截图定稿） |
|---|---|---|
| dark | `217.2 32.6% 17.5%` #1e293b (30,41,59) | `215 25% 27%` #334155 (51,65,85)，slate-700 |
| light（暖纸系） | #f0ebe2 (240,235,226) | ≈#e3ddd1 (227,221,209)，暖灰一档深 |

OnSecondary 两端现值（dark 白 / light 暖墨）与新底对比度均足够，不动。原"视觉暂同、
token 分离"的表述作废——用户定调：secondary==muted 本身就是要修的问题。

## 需求分析与背景调查

起因：031-image-viewer 双端截图差异（iced 臂按钮裸文本 vs VM 臂主题色填充按钮）。
调查结论（2026-09-06）：

- VM 臂 `aura_view_builder.rs:7336-7348`：`""|"default"|"primary"|"submit"` 四写法共用
  同一 `bg-primary` preset（shadcn-vue 语义：default 即 primary 填充）。
- Vue 臂 `ui_gen/vue.rs:16463`（生成的 variants.ts cva）：六键
  default/destructive/outline/secondary/ghost/link，default=primary 填充；
  **无 `primary` 键**——.at 写 `variant:"primary"` 在 web 端落空（cva 查无此键→无类）。
- Rust transpile 臂 `ui_gen/rust.rs`：普通 button 只透传 user class，不注入任何 preset
  （仅 modal action/cancel 硬编码）；共享 iced 渲染器对无背景类按钮有意渲染为
  chromeless（renderer.rs:3607 注释）→ 031 截图图1 的裸文本观感。
- 现行规约 Design 22（docs/design/autoui/base-styles-and-visual-parity.md §3）规定
  button 缺省 = primary 填充——本 plan 修订该规约。
- secondary==muted 证据：VM `theme.rs::resolve_semantic_rgb` 两分支同返回值；Vue 端
  `ui_gen/vue.rs:15206-15208`（light `--secondary`==`--muted`==`210 40% 96.1%`）与
  `:15232-15234`（dark 均为 `217.2 32.6% 17.5%`）——两端一致同灰，系 shadcn 默认主题
  原样继承，非抄写错误；default 落 muted 后必须分化（用户定调 2026-09-06）。
- 测试影响面干净：仓内含 `bg-primary text-primary-foreground` 的断言均为 .at 源码自带
  类字符串（vm_bridge.rs:2986 日历选中格、musk_vm_track_tests.rs:2130 气泡），与
  default preset 无关，不需改。

## 详细设计

1. **variants.rs**（新）：两张 `match` 表 + 单测（default 含 `bg-muted` 与 `border`、
   不含 `bg-primary`；primary 含 `bg-primary`、不含 `border`；secondary 含
   `bg-secondary`、不含 `border`；未知 variant 返回空串）。函数带文档注释：说明
   "default 承载 Web UA stylesheet 等价物"的设计语义，指向 Design 22 §3。
2. **VM 臂** `aura_view_builder.rs::convert_button`：删除内联 preset match，改调
   variants.rs；`""|"default"` 中性、`"primary"|"submit"` 醒目，其余透传。
3. **Vue 臂** `ui_gen/vue.rs`：variants.ts cva 模板——`default` 值改为中性类，新增
   `primary`、`submit` 键（值=primary 填充类）；cva 注释标注"与 ui/style/variants.rs
   互锁"。`defaultVariants.variant` 仍为 `'default'`。
4. **Rust codegen 臂** `ui_gen/rust.rs`：button 生成路径（无 children / 有 children /
   label 折叠三处 `View::button(...)`）读取 `variant`/`size` prop，命中
   `button_variant_preset`/`button_size_preset` 后**前置于** user class（与 VM 臂
   "user class 追加在后可覆盖"同一语义）；variant 为空串时仍注入 default preset（本臂
   首次获得 preset 能力，直接落新语义）。
5. **渲染器零改动**：iced renderer 本就 class 驱动，中性类即得中性观感。
6. **secondary token 分档（双端）**：`theme.rs::resolve_semantic_rgb` 的 `Color::Secondary`
   臂改为新值（dark #334155 / light #e3ddd1，表见上）；`ui_gen/vue.rs` index.css 模板
   `--secondary` 两段（light/dark）同步改 HSL 表达。执行时全仓 grep `bg-secondary`/
   `Secondary` 盘点受影响面（badge secondary、hover:bg-secondary 等）并截图核对无违和。
7. **Design 22**：§3 表新增 default 行、primary 行改"显式声明"，§1.2 补一段
   "default variant = UA stylesheet 等价物"的设计说明；§3 下方补 secondary/muted
   分档说明（shadcn 原样继承的偏离声明）。

## 测试设计

- variants.rs 单测：表内容断言（见上）。
- VM 臂：convert_button 缺省 button 的 View::Button.style 含 `bg-muted`、不含
  `bg-primary`；`variant:"primary"` 含 `bg-primary`。
- codegen 臂：生成的 Rust 源码字符串含中性 preset 前缀 + user class 后缀。
- 双端一致性（autoui-verifier）：002-counter、005-login、031-image-viewer 三例
  Vue/VM 截图比对——缺省按钮两臂同为中性填充、secondary 按钮比 default 深一档，
  `variant:"primary"`（031 无则临时构造最小 .at）两臂同为主题色填充。
- 全仓盘点：`grep -rn "button" examples/ui/*/src/front` 统计未声明 variant 的按钮
  数量与所在 demo，作为回归面清单入 plan 执行证据。
- theme.rs 既有断言（如 `assert_eq!(rgb(Color::Muted), ...)` 一族）核对 Secondary
  相关断言并更新。
- 可辨识度检查（T8 截图判定项）：dark 主题下 default 按钮（muted 填充 #1e293b +
  border #283146）置于 background #141a29 与 card #1a2235 上轮廓清晰可辨；若发丝
  描边仍不足，升级方案为 default 提用 secondary 深填充（执行期与用户确认）。

## 验收标准

- [x] Vue 与 VM 两端：未声明 variant 的 button 渲染为"muted 填充 + 发丝描边"，双端观感一致；dark 主题下置于 background/card 上轮廓可辨。
- [x] `variant:"primary"` 两端均主题色填充；`variant:"submit"` 同 primary。
- [x] `variant:"secondary"`（深填充、无边框）与 default（浅填充、有边框）、outline（边框无填充）同屏三者可辨，双端一致。
- [x] Rust transpile 臂缺省按钮与 VM 臂同观感（三臂收敛）。
- [x] Design 22 §3 与实现一致，含 default=UA-stylesheet-等价物的设计说明与 secondary/muted 分档声明。
- [x] `cargo check -p auto-lang` 零新警告；`cargo t ui` 全绿；受影响既有断言已更新。
- [x] 存量 examples 抽查 3 例无布局破坏（双端截图入证据）。

## 执行步骤

（原子任务：精确文件路径 + 确切操作 + 验证命令；每步完成后追加 [✅ 已完成] 一行证据）

- [x] T1 新建 `crates/auto-lang/src/ui/style/variants.rs`：variant/size 两张 preset 表
      + 单测；`style/mod.rs` 导出。验证：`cargo t variants`。 [✅ 已完成]
      （TDD 红→绿：stub 7 失败→实现后 `cargo t variants` 17/17 绿；commit 243a5064c；
      auto-down 兄弟 worktree 已入组建 `D:/autostack/.wt/lang-571/auto-down`@lang-571-dev）
- [x] T2 VM 臂接线：`aura_view_builder.rs::convert_button` preset match 改调共享表。
      验证：`cargo t aura_view_builder`（或所属测试模块）。 [✅ 已完成]
      （`cargo t plan571_button` 3/3 绿；`cargo t button` 46/46 绿；`cargo t
      aura_view_builder` 83/84——唯一红 plan055_strip_html 在 master 同样红，预存红
      与本 plan 无关；commit d0a825357）
- [x] T3 Vue 臂：`ui_gen/vue.rs` cva 模板 default 改中性、新增 primary/submit 键。
      验证：`cargo t ui_gen`。 [✅ 已完成]
      （plan571 互锁测试 1/1 绿——顺带实证 ghost hover 双端既有分歧（VM bg-secondary
      vs web bg-accent，iced 无 Accent token）；`cargo t ui_gen` 531/532，唯一红
      test_charts_gallery_compiles 在 master 同红属预存；commit 9f4d973c0）
- [x] T4 Rust codegen 臂：`ui_gen/rust.rs` 三处 button 生成点注入 preset（variant/size
      prop 解析 + 前置拼接）+ 生成断言测试。验证：`cargo t ui_gen`。 [✅ 已完成]
      （实现收敛为单点注入：generate_view_tree Element 臂 with_button_preset 前置
      合并，覆盖全部三处 `View::button` 生成点；plan571 codegen 测试 3/3 绿；
      `cargo t ui_gen` 530/531 唯一红为 charts gallery 预存红；`cargo check` 无
      feature 与 --features ui 双配置 0 error；commit d22cb5f7a）
- [x] T5 secondary token 分档：`theme.rs` Secondary 臂改新值（dark #334155 / light
      #e3ddd1）+ 相关断言更新；`ui_gen/vue.rs` index.css 模板 `--secondary` light/dark
      两段同步。验证：`cargo t theme` + `cargo t ui_gen`；grep `bg-secondary` 盘点面
      记录入本节。 [✅ 已完成]
      （theme 19/19 绿含新增双主题 secondary≠muted 断言；plan449 style_parity 2/2——
      `bg-secondary` 仅登记可解析性、与值无关；ui_gen 533/534 唯一红 charts gallery
      预存红；commit 043ed68b9）
- [x] T6 断言/golden 清理：全仓 grep `hover:bg-primary/90`、`bg-muted` 冲突点、renderer
      快照相关 golden，逐个更新。验证：`cargo t ui`。 [✅ 已完成]
      （grep 余留引用均为合法：modal action preset（显式 action 语义，保持填充）、
      musk/vm_bridge 的 .at 源码自带类、cva base；`cargo t ui` 1830/1848，失败 18 例
      与 master 同过滤失败集合逐一相等（comm 双向空集）——全部预存红，零新增；
      本任务无代码改动）
- [x] T7 Design 22 文档更新（§3 表 + §1.2 说明段 + secondary/muted 分档声明）。
      [✅ 已完成]（§1.2 增"default variant = UA stylesheet 等价物"设计声明段；
      §3 表拆 default/primary 两行 + secondary 行补分档与无边框说明；
      commit b73fc5e4f）
- [x] T8 examples 盘点 + 双端 spot check（002/005/031，autoui-verifier 截图入
      scratch/p571/）。 [✅ 已完成]
      （盘点：examples/ui 405 处 button 引用、61 处显式 variant、涉及 38 个 demo——
      缺省按钮全面转中性为预期回归面。**执行期发现 Vue 臂有第二真源**：`auto run`
      的 gen/front 管线用 `auto-man/assets/shadcn-ui/button/index.ts` 烘焙快照（非
      ui_gen/vue.rs 模板），已同步收敛 + SNAPSHOT.md 烘焙补丁记录（循 PLAN-457
      sonner 先例）+ vue_shadcn 互锁测试锚定（1/1 绿，plan_457 守卫 1/1 绿不受扰）。
      双端截图 6 张入 scratch/p571/{vm,vue}\_{002,005,031}\_\*.png：002 与 031 双端
      均中性 chip 观感一致；005 双端仍蓝系（demo 自带 bg-blue-500 显式类，后类胜
      语义正确非回归）；commit a14bba3b0）
- [x] T9 review 前全量门禁：`cargo tf`。 [✅ 已完成]
      （执行期适配两处：ui_gen 互锁/产物级测试在 tf 档（无 ui-iced，Plan 507）随
      `cfg(all(test, feature = "ui"))` 关闭——preset 注入本体亦 ui 门控，该档按设计
      退化透传；终跑 `cargo tf --no-fail-fast` 失败集合 = {test_charts_gallery_compiles}
      唯一预存红（master 同红），3459/3461 通过；commit 53259c4da。终验：`cargo
      check -p auto-man` 0 error；plan571 全族 7/7（auto-lang）+8/8（auto-man）+
      variants 18/18 绿）

- [x] T10（复审 F1 修复）CSS 变量层 `--secondary` 分档收敛剩余三源：
      `crates/auto-man/src/vue.rs:1225/:1266`、`crates/auto/src/cmd_vue.rs:1642/:1682/:1841`、
      `crates/auto/src/cmd_tauri.rs:760/:794`——light→`40 24% 85.5%`、dark→
      `215 25% 27%`（与 theme.rs/ui_gen 已落值一致）；删 031 `gen/` 重生成并 grep
      产物 css 断言新值；Vue 端 secondary 探针截图补双端证据；加 css 互锁断言测试
      防"第四源"。验证：`cargo t -p auto-man plan571` + 重生成产物 grep + 截图。
      [✅ 已完成]（三源 6 处收敛：auto-man/vue.rs:1227/:1269（活源）、cmd_vue.rs
      :1644/:1684/:1844、cmd_tauri.rs:762/:797——后两文件系未挂线死文件顺手对齐防
      复活带旧值；三 crate 各挂 `generate_index_css` 互锁断言（auto-man plan571
      9/9 绿、auto bin 8/8 绿、check 0 error）；031 删 gen 重生成产物 css 实证新值
      （:21 light/:63 dark）；`vue_variant_probe.png` 与 `vm_variant_probe.png`
      五档同屏双端一致；`cargo t -p auto` 24 红逐一对拍 master 同红（clipboard 族
      OS 并发 flaky+预存）零新增；commit 350414c12）

## 复审记录

- 复审人：ZCode（/auto-plan:review，2026-09-06）
- 复审基线：worktree `D:/autostack/.wt/lang-571/auto-lang` @ plan-571-dev（merge-base
  b0d430349），diff 恰含计划声称的 10 文件、无夹带（+561/-29）。

### 验收标准逐条复验

1. **双端缺省按钮 = muted 填充+发丝描边、dark 可辨** — ✅ pass。plan571 VM 臂测试
   3/3（BackgroundColor(Muted)+Border+BorderColor(Border)、无 Primary）；
   `vm_031_imageviewer.png`/`vue_002_counter.png`（重生成后）均中性 chip。
2. **variant:"primary"/"submit" 两端主题色填充** — ✅ pass。codegen 产物断言
   `explicit_primary_variant_keeps_accent_fill` + 资产互锁测试（primary/submit 键
   三源在册）。
3. **secondary/default/outline 同屏可辨** — ✅ pass（VM 端视觉实证）。探针 app
   五档同屏截图 `scratch/p571/vm_variant_probe.png`：五 variant 两两可辨、secondary
   强一档无描边清晰。Vue 端 cva 层一致（互锁测试）；但**令牌层发现缺口 → 见下**。
4. **Rust transpile 臂与 VM 同观感** — ✅ pass。`plain_button_gets_default_neutral_preset`
   断言生成串逐类一致。
5. **Design 22 §3/§1.2 与实现一致** — ✅ pass（b73fc5e4f）。
6. **cargo check 零新警告 + cargo t ui 全绿 + 既有断言更新** — ⚠️ pass（带预存红
   注记）。check 双配置 0 error；`cargo tf` 3460/3461 唯一红 charts gallery
   （master 同红）；`cargo t`（ui-iced 档）21 红 = 17 与 master 对拍相同 + 3 直接
   master 复红（plan370 d2/d8、plan492 c2）+ 1 flaky 环境红（osconfig
   resolve_all_miss_is_none，隔离重跑 4/4 绿）。零新增红。
7. **存量 examples 抽查 3 例无布局破坏** — ✅ pass（6 截图 + 005 蓝系为 demo 显式
   bg-blue-500 覆盖、语义正确）。

### 发现（fail 依据）

**[F1] CSS 变量层 `--secondary` 分档漏了第三类源（判 fail 的唯一项）**：T5 收敛了
`ui_gen/vue.rs` 模板，但 `auto run` Vue 管线实际物化的 `gen/front/vue/src/index.css`
来自 **auto-man/src/vue.rs 内嵌模板**——031 重生成产物 `--secondary` 仍为
`210 40% 96.1%`/`217.2 32.6% 17.5%`（== muted，未分档）。即 **web 端 secondary 按钮
皮肤（cva 类）已分档、但语义色令牌未分档**，Vue 端 secondary 观感退回与 muted 同灰，
与 VM 端（theme.rs 已分档）不一致。另两处同族模板一并修：
- `crates/auto-man/src/vue.rs:1225`（light `210 40% 96.1%` → `40 24% 85.5%`）、
  `:1266`（dark `217.2 32.6% 17.5%` → `215 25% 27%`）
- `crates/auto/src/cmd_vue.rs:1642`（light 同上）、`:1682`（`217 33% 17%` →
  `215 25% 27%`）、`:1841`（`217.2 32.6% 17.5%` → `215 25% 27%`）
- `crates/auto/src/cmd_tauri.rs:760`（light）、`:794`（dark）同口径
修后须：删 031 的 `gen/` 重生成 → grep 产物 css 断言新值 → Vue 端重截 secondary
探针图（补双端一致性证据）。互锁测试建议顺手加：assert 生成/模板 css 含
`--secondary: 40 24% 85.5%` 与 `215 25% 27%`（防第四源）。

### 遗漏/延后/workaround 猎查

- 无计划任务被静默丢弃；T6 为"验证性 no-op"（grep 证实无测试锁死旧行为），非回避。
- 既有分歧/债务如实入账：ghost hover accent vs secondary（VM 无 Accent token）、
  动态 class 不注入 preset（codegen 臂，行为同前）、Vue cva 双真源（已收敛+互锁，
  维护面+1）。均为记录在案的债务候选，非本 plan 引入。

### 判定

**fail** → 回 `/auto-plan:work`，修复项 = T10（F1，预计小改：3 文件 6 行 + 重生成
验证 + 互锁断言）。状态回滚 `executing`。

### 复审第二轮（2026-09-06，T10 修复后）

- **F1 已闭合**：三源 6 处收敛（auto-man/vue.rs:1227/:1269 活源 + cmd_vue.rs 3 处 +
  cmd_tauri.rs 2 处死文件防复活对齐）；031 删 `gen/` 重生成，产物
  `gen/front/vue/src/assets/index.css:21`（light `40 24% 85.5%`）与 `:63`
  （dark `215 25% 27%`）实证新值落盘；`vue_variant_probe.png` 与 `vm_variant_probe.png`
  五档（default/secondary/outline/ghost/primary）同屏双端两两可辨。
- **防回潮**：三个 crate 各挂 `generate_index_css` 互锁断言
  （`plan571_css_secondary_interlock_tests`）——auto-man 1/1 绿；auto bin 8/8 绿
  （cmd_* 系未挂线死文件，断言随文件复活自动生效）。
- **回归面**：`cargo t -p auto` 24 红逐一对拍 master 的 `-p auto` 全量失败集——
  全部 master 同红（clipboard 族 OS 并发 flaky + 预存行为红），零新增；
  `cargo check -p auto` 0 error；auto-man plan571 9/9 绿。
- **判定：pass** → `status: reviewed`，待 `/auto-plan:merge`（fold + specs 六节
  存款 + KNOWN-DEBT 记账：iced Accent token no-op、Vue cva 多真源维护面）。

## 待澄清事项

- ~~default 的 hover 观感暂定 `hover:bg-muted/70`~~ 已落（T1-T2 实现，双端截图观感
  通过）；`hover:bg-accent` 备选方案仍受制于 iced 侧 Accent token 无解析臂（见下条）。
- ~~`"submit"` 跟随 primary~~ 已按 primary 落（三臂一致 + 测试锁定）。
- 调查中发现的相邻债务（不入本 plan，merge 时记 KNOWN-DEBT）：iced 侧
  `resolve_semantic_rgb` 无 `Color::Accent` 臂——ghost/outline preset 的
  `hover:bg-accent` 在 VM 端疑似 no-op（Vue 端有 `--accent`），两 variant 的 hover
  反馈缺失，建议后续独立 plan 收口。
- secondary 新值（dark #334155 / light #e3ddd1）为提案值，T8 双端截图定稿；若用户
  对档位有偏好（更浅/更深）在 T5 前提出。
- **执行期新录（T6/T8）**：① 预存红与 master 逐一对拍（`cargo t ui` 18 例、tf 档
  charts gallery 1 例）均与本 plan 无关，review 时按在案预存红清单核对；② Vue 臂
  存在**两套** cva 真源（ui_gen/vue.rs 模板 + auto-man 烘焙资产），已双双收敛并各
  挂互锁测试，后续 variant 演进须三处（Rust 单源 + 两 Vue 源）同步。
