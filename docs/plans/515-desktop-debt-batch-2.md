---
plan_id: PLAN-515
status: drafting               # drafting → executing → execution_done → reviewed → archived
feature_name: desktop-debt-batch-2
author: [zhaopuming]
created_at: 2026-09-01
updated_at: 2026-09-01

# /auto-plan:review 结束时填写：
supersedes_spec_components: []
new_spec_components: []
touched_goals: []             # 引用 docs/specs/goals.md 的 GOAL-NNN

affects: [auto-lang/ui]
current_step: 0
total_steps: 9
---

# [PLAN-515] 桌面 DEBT 批处理二期——queue 臂保真收口 + vue 对齐 + 工具链可见性

## 变更摘要

桌面线第二批集中清偿（一期=505 已归档）。四族：

| 族 | 内容 | 债务号 |
|---|---|---|
| **A queue 臂渲染保真收口** | ① DrawOp 增 **scissor 裁剪算子**（追加式 tag）——scroll 无裁剪致溢出内容不剪，P507-1 原文"宿主 Stage 5+ 裁剪算子后收口"即本期；② typography **字重/斜体差分通道**（现同串 bold/italic 无视觉差分） | P507-1 |
| **B vue 轨对齐** | vue 桌面宿主桌面区**无壁纸图层**（VM 轨有 `desktop_wallpaper_scrim` 10%/35% 叠层，vue 轨纯色无锚点）——补壁纸层并对齐 scrim 语义 | P503-2 |
| **C 工具链可见性** | ① 覆盖率数字默认档不可见（nextest 吞 stdout；`element_counts` 挂 bin/写文件）② `auto_exe()` 优先现存二进制、陈旧产物伪装成回归 ③ desktop 真实 launch e2e 未通（合成输入打不进 winit——505 C 族验收通道资产复用） | P507-3 / P500-1 / P504-3 |
| **D 增强候选对账收拢** | 六项散落增强（HICON 真图标/窗口选择器面板/外来虚拟文件拉流/真延迟回调/native DWM 缩略/vue 真缩略 web 路径）逐项判定**纳入本批 / 显式不做+理由 / 挂起**——产出判定表，终结"散在各归档计划待澄清里无主"状态 | 各归档计划待澄清 |

**与 513（仓库整合清理）的边界**：513 处置**账目**（陈年计划归档判死、
债务簿核销已修未销项）；本期清偿**开放工作项**。互不越界。

## 目标

- **G1 scissor 算子**：DrawOp 追加裁剪算子（协议演进纪律），投影器
  scrollable 臂产裁剪区、宿主栅格化与 queue 金样贯通——溢出内容正确裁剪。
- **G2 typography 差分**：Text op 样式通道补字重/斜体，projector 产差分、
  宿主渲染差分可见（金样对拍更新）。
- **G3 vue 壁纸层**：vue 桌面宿主桌面区壁纸图层（读 `shell.desktop.wallpaper`
  同源键），图片壁纸上叠层语义与 VM scrim 对齐（10%/35%）。
- **G4 工具链三件**：覆盖率数字日常可见；`auto_exe` 陈旧二进制防伪装
  （时间戳警示或构建校验）；launch e2e 经验收通道跑通一次留痕。
- **G5 判定表**：D 族六项判定成文（做/不做+理由/挂起+触发条件），纳入项
  转任务执行。
- **非目标**：DrawOp 其他保真缺口（P507-1 未列项）；508 远程线增强；
  语言/VM 线债务（P510-1 等）；债务簿核销（513 域）。

## 架构方案

- A 族：`ui/desktop_protocol/message.rs`（DrawOp 变体追加 scissor）→
  `client_runtime.rs`（scrollable 投影臂产裁剪栈）→ 宿主栅格化段
  （裁剪应用）→ 金样更新（含 TS 渲染器侧）；typography 差分同链路。
- B 族：vue 桌面宿主桌面区组件增壁纸层（widgets 包桌面宿主资产，465 线）。
- C 族：`stage3.rs`/验收通道脚本（P504-3）；覆盖率 bin（新小 bin 或测试
  改写文件）；`auto_exe`（dual_mode/stage3 侧函数）。
- D 族：判定表回写本计划，纳入项按族并入任务。

## 技术栈

既有栈。零新依赖。

## 需求分析与背景调查

（KNOWN-DEBT 开放项对账 + 各归档计划待澄清扫拢 2026-09-01）

- **P507-1 原文**："scroll 无裁剪（DrawOp v1 无 scissor——溢出内容不剪，
  宿主 Stage 5+ 裁剪算子后收口）；typography bold/italic 不产生视觉差分
  （无字重/斜体通道）"——507 复审明确登记非静默，收口点即本期。
- **P503-2 原文**：VM 轨 `desktop_wallpaper_scrim`（图片壁纸上叠
  bg-background 10%/35%），vue 轨桌面区纯色无壁纸层锚点。
- **P507-3**：nextest 隐藏通过测试 stdout，`[queue-coverage]` 行需
  `--success-output immediate`（命令已档 `.cargo/config.toml` 注释）；
  可选偿还=element_counts 挂 bin 或写文件。
- **P500-1**：T3 `auto_exe()` 优先取现存二进制，陈旧产物伪装成回归。
- **P504-3**：desktop 真实 launch 实况 e2e 未通（合成输入打不进 winit）
  ——505 C 族已建实机验收通道（放行机制+规程），复用解。
- **D 族散项出处**：HICON（473/486 两度延期）、窗口选择器（486 判"需求
  出现再立"）、外来虚拟文件拉流与真延迟回调（488 非目标+增强）、DWM
  缩略（497 待澄清②，挂 494 真洞反馈）、vue 真缩略 web 路径（497 待澄清①）。
- **排程**：worktree 全空机队闲置；与 509/513 零交叠（A 族碰
  desktop_protocol——507/508 已合入无在途冲突）。

## 详细设计

### 1. scissor 算子（G1）

- `DrawOp::Scissor { rect }`（tag 顺延）+ 栈语义（进入/退出）：投影器
  scrollable 臂在内容溢出时产 push/pop；宿主栅格化按栈裁剪；TS 渲染器
  （`packages/drawlist-renderer/`）同步——golden 双侧更新，追加式不破坏
  既有帧兼容。

### 2. typography 差分（G2）

- Text op 样式通道增 weight/style（bold/italic 解析既有——projector 补
  透传，宿主 fill_text 产差分）；金样补 bold/italic 用例。

### 3. vue 壁纸层（G3）

- vue 桌面宿主桌面区根容器增壁纸图层（image/纯色，数据源与 VM 同键
  `shell.desktop.wallpaper`——vue 侧读取链按 465 机制）；图片壁纸叠
  `bg-background` 10%/35% 双段 scrim 与 VM 对齐。

### 4. 工具链三件（G4）

- 覆盖率：`element_counts` 写 `target/queue-coverage.json` + 小 bin
  （`cargo run --bin queue-coverage`）读表打印；
- `auto_exe`：现存二进制加 mtime 检查——陈旧于源码则警告并可
  `AUTO_FRESH_EXE=1` 强制重建（默认警告不阻断，防 e2e 静默假绿）；
- launch e2e：经 505 验收通道（放行机制）跑 011 真实 launch 实况一次
  留痕，P504-3 按结果清偿或精确注记。

### 5. D 族判定表（G5）

六项逐项：判定（纳入/不做/挂起）+ 理由 + 纳入者的任务映射。起草人
预判（T1 复核定稿）：HICON=纳入；窗口选择器=挂起（触发=真机使用反馈）；
拉流与延迟回调=不做（低频+复杂度高，理由成文）；DWM 缩略=挂起（触发=
真洞翻默认反馈）；vue 真缩略=挂起（触发=vue 远程形态落地后）。

## 测试设计

1. **T1 判定表**成文回写（复审对照物）。
2. **T2 A 族**：scissor 单测（投影器裁剪栈产出手写断言）+ 宿主/TS 双侧
   golden 更新 + scrollable 溢出示例（013-notes）queue 模式端到端视觉断言。
3. **T3 B 族**：vue 壁纸层组件测（vitest）+ 双轨 scrim 一致性截图对拍。
4. **T4 C 族**：覆盖率 bin 输出演练；`auto_exe` 陈旧警告单测；launch
   e2e 留痕。
5. **T5 回归**：507/508 既有套件全绿（协议追加不破坏）；`cargo t ui`。

## 验收标准

1. A 族：scrollable 溢出正确裁剪（端到端视觉断言）+ bold/italic 差分
   金样绿；协议追加式纪律核验（既有帧/旧端兼容测试绿）。
2. B 族：vue 桌面图片壁纸可见 + scrim 双轨一致。
3. C 族三件各留痕（覆盖率输出/警告演练/e2e 记录）。
4. D 族判定表成文，纳入项执行完、不做项理由回写债务簿。
5. `cargo t ui`、`desktop_protocol` 套件、vue 档不回归；零警告。

## 执行步骤

（原子任务：精确文件路径 + 确切操作 + 验证命令；每步完成后追加 [✅ 已完成] 一行证据）

1. **D 族判定表**：六项判定+理由+任务映射回写本计划。
   验证：判定表成文。
2. **scissor 协议算子**：`crates/auto-lang/src/ui/desktop_protocol/message.rs`
   DrawOp 追加 Scissor（tag 顺延）+ codec/golden 单测。
   验证：`cargo t desktop_protocol --features ui-iced`。
3. **投影器+宿主贯通**：`crates/auto-lang/src/ui/desktop_protocol/client_runtime.rs`
   scrollable 裁剪栈 + 宿主栅格化裁剪 + TS 侧（`packages/drawlist-renderer/`）
   同步与 golden。
   验证：`cargo t desktop_protocol --features ui-iced` + 渲染器包测试绿。
4. **typography 差分**：projector 透传 + 宿主差分渲染 + 金样用例。
   验证：`cargo t desktop_protocol --features ui-iced`。
5. **端到端视觉断言**：013-notes（scroll 溢出）queue 模式渲染断言。
   验证：desktop_mcp/快照断言绿。
6. **vue 壁纸层**：widgets 包桌面宿主壁纸层 + scrim 对齐 + vitest/截图
   对拍。
   验证：vue 档测试绿 + 对拍留痕。
7. **C 族三件**：覆盖率 bin + `auto_exe` 陈旧警告 + launch e2e 通道留痕。
   验证：各自演练输出。
8. **纳入项执行**：按 T1 判定（预期 HICON）执行并清偿。
   验证：对应债务行已清偿注记。
9. **收尾**：全部清偿/判定回写 KNOWN-DEBT；健康检查；状态翻
   execution_done。
   验证：`cargo check -p auto-lang && cargo t ui`。

## 复审记录

（/auto-plan:review 填写）

## 待澄清事项

- **scissor 栈深度**：嵌套 scrollable 的栈语义 v1 支持几层（预期 2 层
  够用）——实现期定并写入协议文档 §1.3 增量注记。
- **typography 差分的字体资源**：bold/italic 差分依赖宿主字体栈有对应
  face——Hello.fonts 上传链（v1.0 既留）是否需真启用，T4 时核。
- **vue 壁纸数据源**：vue 侧 storage 读取链（465 机制）与 VM 同键一致性
  ——若 vue 侧无 storage 桥则降级"配置注入"并注记差异。
- **D 族判定倾向**为起草人预判（T1 复核），最终以判定表为准——挂起项
  必须写明触发条件，不做项必须写理由，防"永久悬置"。
- `auto_exe` 强制重建默认值（警告 vs 阻断）以 e2e 稳定性优先执行期定。
