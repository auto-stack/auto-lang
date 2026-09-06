---
plan_id: PLAN-571
status: drafting               # drafting → executing → execution_done → reviewed → archived
feature_name: button-default-variant
author: []
created_at: 2026-09-06
updated_at: 2026-09-06

# /auto-plan:review 结束时填写：
supersedes_spec_components: []
new_spec_components: []
touched_goals: []             # 引用 docs/specs/goals.md 的 GOAL-NNN

affects: [auto-lang/ui, auto-lang/ui_gen, docs/design/autoui]
current_step: 0
total_steps: 8
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

## 目标

1. `default`（缺省）= 中性基线按钮：muted 填充 + 正常前景色，不抢视觉焦点。
2. `primary` = 显式醒目 variant：主题色填充（即现缺省观感），留给 CTA/主操作。
3. 三臂 variant preset 同源同观感：Vue（cva）、VM（解释器）、Rust（transpile codegen）。
4. preset 表单一事实源 + Design 22 §3 规约表同步修订。
5. 存量 examples 缺省按钮从主题色填充变为中性填充——这是预期方向的视觉回归，需盘点可控。

## 架构方案

**共享 preset 表**：新增 `crates/auto-lang/src/ui/style/variants.rs`，
`pub fn button_variant_preset(variant: &str) -> &'static str` 与
`pub fn button_size_preset(size: &str) -> &'static str`。VM 臂（aura_view_builder）与
Rust codegen 臂（ui_gen/rust）直接调用；Vue 臂的 cva 是生成到前端工程的文本，无法共享
Rust 代码，以注释互锁 + 单测断言三表字符串一致（防漂移）。

**目标 variant 表（核心设计决策）**：

| variant | preset 类 | 观感 |
|---|---|---|
| 缺省 / `"default"` | `bg-muted text-foreground font-medium rounded-md hover:bg-muted/70` | 中性灰 chip（新基线） |
| `"primary"` | `bg-primary text-primary-foreground font-medium rounded-md hover:bg-primary/90` | 主题色填充（现缺省观感） |
| `"submit"` | 同 primary | 行为语义：表单主操作应醒目 |
| `"secondary"` | `bg-secondary text-secondary-foreground font-medium rounded-md hover:bg-secondary/80` | 不变 |
| `"destructive"` / `"outline"` / `"ghost"` / `"link"` / `"icon"` | 现状保留 | 不变 |
| `"text"` / 未知 | 无 preset（chromeless，由 user class 主导） | 不变 |

size preset 不变：`sm→h-9 px-3`、`lg→h-11 px-8`、`icon→h-10 w-10`、缺省→`h-10 px-4`。

**Token 事实**：当前主题 `Muted` 与 `Secondary` 的 RGB 完全相同（dark (30,41,59) /
light (240,235,226)，见 theme.rs resolve_semantic_rgb）。default 与 secondary 视觉暂同
但 **token 分离**——这正是"最初就定下来"的意义：token 层已各归其位，后续主题分化
（如 default 降一档明度）只动 theme.rs 不动三臂。

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
- 测试影响面干净：仓内含 `bg-primary text-primary-foreground` 的断言均为 .at 源码自带
  类字符串（vm_bridge.rs:2986 日历选中格、musk_vm_track_tests.rs:2130 气泡），与
  default preset 无关，不需改。

## 详细设计

1. **variants.rs**（新）：两张 `match` 表 + 单测（default 不含 `bg-primary`、primary
   含 `bg-primary`、未知 variant 返回空串）。函数带文档注释：说明"default 承载 Web UA
   stylesheet 等价物"的设计语义，指向 Design 22 §3。
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
6. **Design 22**：§3 表新增 default 行、primary 行改"显式声明"，§1.2 补一段
   "default variant = UA stylesheet 等价物"的设计说明。

## 测试设计

- variants.rs 单测：表内容断言（见上）。
- VM 臂：convert_button 缺省 button 的 View::Button.style 含 `bg-muted`、不含
  `bg-primary`；`variant:"primary"` 含 `bg-primary`。
- codegen 臂：生成的 Rust 源码字符串含中性 preset 前缀 + user class 后缀。
- 双端一致性（autoui-verifier）：002-counter、005-login、031-image-viewer 三例
  Vue/VM 截图比对——缺省按钮两臂同为中性填充，`variant:"primary"`（031 无则临时构造
  最小 .at）两臂同为主题色填充。
- 全仓盘点：`grep -rn "button" examples/ui/*/src/front` 统计未声明 variant 的按钮
  数量与所在 demo，作为回归面清单入 plan 执行证据。

## 验收标准

- [ ] Vue 与 VM 两端：未声明 variant 的 button 渲染为中性填充（非主题色、非裸文本），双端观感一致。
- [ ] `variant:"primary"` 两端均主题色填充；`variant:"submit"` 同 primary。
- [ ] Rust transpile 臂缺省按钮与 VM 臂同观感（三臂收敛）。
- [ ] Design 22 §3 与实现一致，含 default=UA-stylesheet-等价物的设计说明。
- [ ] `cargo check -p auto-lang` 零新警告；`cargo t ui` 全绿；受影响既有断言已更新。
- [ ] 存量 examples 抽查 3 例无布局破坏（双端截图入证据）。

## 执行步骤

（原子任务：精确文件路径 + 确切操作 + 验证命令；每步完成后追加 [✅ 已完成] 一行证据）

- [ ] T1 新建 `crates/auto-lang/src/ui/style/variants.rs`：variant/size 两张 preset 表
      + 单测；`style/mod.rs` 导出。验证：`cargo t variants`。
- [ ] T2 VM 臂接线：`aura_view_builder.rs::convert_button` preset match 改调共享表。
      验证：`cargo t aura_view_builder`（或所属测试模块）。
- [ ] T3 Vue 臂：`ui_gen/vue.rs` cva 模板 default 改中性、新增 primary/submit 键。
      验证：`cargo t ui_gen`。
- [ ] T4 Rust codegen 臂：`ui_gen/rust.rs` 三处 button 生成点注入 preset（variant/size
      prop 解析 + 前置拼接）+ 生成断言测试。验证：`cargo t ui_gen`。
- [ ] T5 断言/golden 清理：全仓 grep `hover:bg-primary/90`、`bg-muted` 冲突点、renderer
      快照相关 golden，逐个更新。验证：`cargo t ui`。
- [ ] T6 Design 22 文档更新（§3 表 + §1.2 说明段）。
- [ ] T7 examples 盘点 + 双端 spot check（002/005/031，autoui-verifier 截图入
      scratch/p571/）。
- [ ] T8 review 前全量门禁：`cargo tf`。

## 复审记录

（review 时填写）

## 待澄清事项

- default 的 hover 观感暂定 `hover:bg-muted/70`（变暗一档）；若双端截图观感不佳，
  执行期可与用户确认改 `hover:bg-accent`（需先确认 iced 侧 Accent token 可解析）。
- `"submit"` 跟随 primary 的语义（表单主操作醒目）如需改为跟随 default，在 T2 前提出。
