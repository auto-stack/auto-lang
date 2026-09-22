---
plan_id: PLAN-679
status: archived              # drafting → executing → execution_done → reviewed → archived
feature_name: rq-style-parity
author: [zcode]
created_at: 2026-09-21
updated_at: 2026-09-22

supersedes_spec_components: []
new_spec_components: [SD-01 RQ 臂缺省调色板=theme 语义 token 单源（Design 22 §3 对表）+ Phase 2 增量：DrawOp tag 7 QuadR 圆角通道/投影端预折行/渐变条带近似/group 视觉面（canonical 散文随 683 重写弃，delta 走账本 P679-1 存档）]
touched_goals: []
affects: [docs/design/autoui/base-styles-and-visual-parity.md]
current_step: 3
total_steps: 3
---

# [PLAN-679] rq-style-parity——RQ 臂缺省样式接 theme 单源

> 来源：2026-09-21 用户实机对拍裁定——RQHost 渲染观感"iced 原生控件"，
> 与 VM/vue 差距远。规约 = [Design 22](../design/autoui/base-styles-and-visual-parity.md)
> （vue/vm 默认样式对拍单源）。

> **收口裁定（2026-09-22 用户）**：PLAN-683（rq-remote-renderer，方案 2：
> iced 组件照常渲染、渲染原语过 RenderQueue）更换 RQ 架构路线，本计划与
> PLAN-678 的实现面将随 683 重写。**未竟验收（T-2 的 004 修后目视复验等）
> 弃做**；已落地主体（T-1 theme 单源 + Phase 2 视觉保真四件 + 两枚
> D-1 根修）按收缩版直接收口合并。

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

- [x] T-2 实机双轨对拍（001/003/012 三例 `auto run -r vm` vs `-r vm -q`
      截图对表，用户验收）。
      [2026-09-22 收口判定：partial——003 双轨用户验收通过（Phase 2 立项
      前提）；004 Phase 2 修复后目视复验弃做（683 裁定）]
- [x] T-3 复审 + specs 沉淀（Design 22 增 RQ 臂锚点列）。
      [✅ 以收缩形态完成 2026-09-22——canonical Design 22/desktop-protocol-v1
      散文编辑跳过（683 即将重写该面），delta 走账本 P679-1 存档]

## 6. 复审记录

**stage: review | PLAN-679 | plan_revision: 立项稿+Phase 2 注记回填（legacy 无 revision 字段，以提交链为基线） | outcome: pass（范围收缩版） | reviewed_commit: 5a629a54e（rebase 链 75c217b67→73f67d334→8373585b1→8bdbee3d0→5a629a54e） | base_commit: 979ce80c2（含 PLAN-678 落地） | dependency_revisions: Design 22 规约；master 侧 674/677/680/681/678 随 rebase 入基 | spec_inputs: docs/design/autoui/base-styles-and-visual-parity.md、docs/design/autoui/rq-remote-renderer.md（683 裁定档）**

- **范围收缩授权**：2026-09-22 用户裁定——683 远程 renderer 路线重写实现
  面，未竟验收（T-2 的 004 修后目视等）弃做，主体收口。非未批准缩面。
- **rebase 完整性（重点核查面）**：4 提交 rebase 至新 master **零文本
  冲突但 2/4 patch-id 不等**——range-diff 逐项定性：①commit1（theme
  单源）测试构造器适配 678 落地的 `proj_center_off`（双向意图保留：
  679 色值断言 × 678 关居中构造）；②commit4（Phase 2）hunk 边界随基座
  位移（折行语义 `line_n*line_h` 不变）。**git 文本静默合并漏一枚语义
  缺口**：678 平移通道的 ops 穷尽 match 未含 679 新增 `DrawOp::QuadR`
  → E0004 两处（bbox 实测/平移执行），补臂提交 5a629a54e 修复（臂语义
  与 679 自有 shift_draw_op 同构=rect 平移、color/radius 保留）。
  内容级特征存活核查：theme fn 16 用点、常量族代码残留 0（6 处命中全
  为注释）、QuadR 消费点齐、autocenter(678)/flex(679) 共存。
- **acceptance_results**：T-1 处置 pass；Phase 2 四件 pass（QuadR/预折行/
  渐变条带/group 视觉面——desktop_protocol+fit 背书）；T-2 partial
  （003 用户验收过/004 目视弃-裁定）；T-3 pass（收缩形态）。
- **门禁复现（2026-09-22，lang-679 worktree @5a629a54e）**：`cargo check
  -p auto` 0 error；`cargo t desktop_protocol --no-fail-fast` 193/194
  （唯一红 projector_counter_layout_and_hits=master 预存，主检出同败
  实证见 PLAN-678 复审）；`cargo t fit` 20/20；`cargo t typing` 2/2。
  **`cargo tf` 未跑完**——该 worktree 冷编（full 配置+churn 特征集）+
  test-groups 串行重测致 >70min，2026-09-22 用户裁定"尽快收口、有些
  错误不要紧（683 将重写实现面）"后中止；证据面=上述作用域门禁全绿 +
  同基线 PLAN-678 tf 3722/3722（同一代码基座，679 增量面已被作用域门
  覆盖）。**p508_g2_outproc_arm 首跑红→`cargo build
  -p auto` 刷新后单测绿+全量复跑绿=在案 p508 陈旧 exe 假红**（§Phase 2
  执行期发现同源），非回归。
- **findings**：F-1（非阻塞，已在案）p508 陈旧 exe 假红再现——纪律
  （e2e 前先 build）已有登记，无需新动作；F-2（移交 683）跨分支枚举
  扩容时穷尽 match 消费者静默断——DisplayList v2 落地时对新 op 变体做
  全消费者扫描（本计划 E0004 为实例）。
- **evidence**：本记录内命令/结果摘录（durable）；range-diff/patch-id
  存档于复审会话。
- **next**: merge（ff-only 5a629a54e）→ 账本 P679-1 → 归档 → worktree
  清理。
