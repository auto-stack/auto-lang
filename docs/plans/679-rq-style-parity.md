---
plan_id: PLAN-679
status: executing               # drafting → executing → execution_done → reviewed → archived
feature_name: rq-style-parity
author: [zcode]
created_at: 2026-09-21
updated_at: 2026-09-21

supersedes_spec_components: []
new_spec_components: [SD-01 RQ 臂缺省调色板 = theme 语义 token 单源（Design 22 §3 对表）]
touched_goals: []
affects: [docs/design/autoui/base-styles-and-visual-parity.md]
current_step: 1
total_steps: 3
---

# [PLAN-679] rq-style-parity——RQ 臂缺省样式接 theme 单源

> 来源：2026-09-21 用户实机对拍裁定——RQHost 渲染观感"iced 原生控件"，
> 与 VM/vue 差距远。规约 = [Design 22](../design/autoui/base-styles-and-visual-parity.md)
> （vue/vm 默认样式对拍单源）。

## 1. 根因（实勘定案）

- **View 层双轨同源**：`DynamicComponent::view()` → `AuraViewBuilder`（含
  PLAN-571 变体 preset 注入 + h1-h6 缺省排版）→ 同一 View 喂 iced 轨与
  RqProjector——preset 类（`bg-muted border-border h-10` 等）双轨都拿到。
- **分叉在消费层回退**：RqProjector 的"无样式时缺省调色板"为硬编码暗色
  RGB 常量（BG 24,24,28 / BUTTON_BG 蓝 48,96,200 / INPUT 族灰 / TEXT_FG
  等），绕过 theme 体系；iced 轨则全走 `resolve_semantic_rgb`。主题 token
  解析（`resolve_typed_color`）RQ 臂本就有——缺的只是回退接。

## 2. 处置（2026-09-21 落地，plan-679-dev 412a4ccca）

常量族 → theme fn（client_runtime.rs），消费点逐站映射（native_projector.rs）：

| 旧常量 | 语义槽 | 对表行 |
| --- | --- | --- |
| BG（clear）/ INPUT_BG | `Color::Background` | §3 input `bg-background` |
| BUTTON_BG（按钮臂） | `Color::Muted` | §3 default 变体 `bg-muted` |
| TEXT_FG / LABEL_FG | `Color::OnBackground` | text-foreground |
| PLACEHOLDER_FG | `Color::OnSurface` | muted-foreground |
| INPUT_BORDER / POP_BORDER | `resolve_border_rgb()` | §3 border-input |
| 滑轨 / progress 轨 | `Color::Muted` | 轨道 muted |
| slider/progress/tabs 填充 | `Color::Primary` | accent 驱动 |
| select 选中项 | `Color::Accent` | PLAN-601 交互高亮槽 |
| FOCUS_BORDER（蓝） | `Color::Primary` | shadcn focus ring |
| POP_BG | `Color::Surface` | shadcn popover 面 |

- `const → fn`：theme 值为运行时态（dark_mode thread-local + accent 名）。
- broker_surface.rs 的 IMAGE_PLACEHOLDER 消费点随迁。
- 金样再生：003-converter（**diff 纯色值零几何漂移实证**——clear
  24,24,28→9,14,26 / Border→30,41,59 / Foreground→247,249,251，stella
  dark 槽值）；focus 色钉测试随迁。

## 3. 边界登记（not-yet）

- **圆角**：DrawOp wire 无 radius 通道（v1 op 定长约束）——rounded-md 6px
  在 RQ 臂直角化，既有保真边界。新 tag 提案另立。
- **hover/dark 切换**：RQ 臂无 hover 态（命令帧静态）；dark/light 随
  theme thread-local 与 VM 轨同律。

## 4. 门禁

- `cargo check -p auto` 净（余警告=master 预存）。
- desktop_protocol 187/188（唯一红 = master 预存 counter）。
- 金样 003 再生 + input 聚焦色钉绿。

## 5. 待办

### Phase 2（2026-09-21 立项并实施，004-profile-card 双轨对拍驱动，用户验收 003 后裁定）

**004 差距分析**（RQ vs VM 实机截图逐项）：

| # | 差距 | 根因 | 处置 |
|---|---|---|---|
| 1 | 按钮/卡片直角、状态点方块 | DrawOp wire 无圆角通道 | **tag 7 QuadR**（rect+color+radius 追加式），daemon 走 Path::rounded_rectangle；rounded-full 哨兵 9999 编码端解析 min(w,h)/2 |
| 2 | bio 单行溢出右缘 | 文本不折行（单行 Text op） | 投影端按 avail_w 预折行（词优先+CJK 兜底），逐行发射、text_center 逐行居中 |
| 3 | 渐变头带缺失 | bg-gradient/from/to 未消费 | NodeStyle 三件齐判定 → 24 条带近似（wire 无渐变 op，真 op 另立提案） |
| 4 | 卡片无边界/不满宽/不居中 | **group（Row/Column/List）臂不画 bg/border/圆角/渐变、不消费 padding/max_width** | group 补齐 container 同款视觉面（003 的 bg-card 同根因一并解决） |
| 5 | 头像方形不叠压 | Image 无圆角裁剪 wire + -mt 负边距 | v1 登记：方图+负边距叠加可用；圆裁需 Image 半径语义（wire 提案另立） |
| 6 | 按钮宽大、无 rounded-lg | 桶定宽 BUTTON_MIN_W + 无圆角 | radius 通道覆盖圆角；宽度 v1 维持 |

**落地**（plan-679-dev，Phase 2 提交）：①wire QuadR+broker_surface 栅格+serializer/shift_draw_op 全消费者；②NodeStyle radius/grad 三件+解析；③文本换行+badge 垫底盒；④group 视觉面（padding 消费+max_width 钳制+渐变/圆角/border）。门禁：desktop_protocol 187/188（唯一红=master 预存 counter）。

**执行期发现**：group 补丁后 p508 孵化子 auto.exe 陈旧（stale guard 的测试内嵌套 cargo build 不可行）→ twin/child 几何错位假红；手动 cargo build 刷新即绿——e2e 前置纪律 = **先 cargo build -p auto 再跑 stage3 族**（登记为测试基建边界）。

**残余边界（v1 登记）**：fit 窗 toast 不显示；fit 宽度视口钳制不收窄；渐变=条带近似；头像方形无圆裁；Vertical 列 flex 不消费；hover 态无。

- [ ] T-2 实机双轨对拍（001/002/003/004，用户验收）。
- [ ] T-3 复审 + specs 沉淀（Design 22 增 RQ 臂锚点列 + desktop-protocol-v1 tag 7 增补）。

- [ ] T-2 实机双轨对拍（001/003/012 三例 `auto run -r vm` vs `-r vm -q`
      截图对表，用户验收）。
- [ ] T-3 复审 + specs 沉淀（Design 22 增 RQ 臂锚点列）。
