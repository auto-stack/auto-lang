---
plan_id: PLAN-515
status: reviewed                # drafting → executing → execution_done → reviewed → archived
feature_name: desktop-debt-batch-2
author: [zhaopuming]
created_at: 2026-09-01
updated_at: 2026-09-02

# /auto-plan:review 结束时填写：
supersedes_spec_components:
  - "auto-lang/ui: desktop_protocol DrawOp v1.5 增量——Scissor/ScissorPop(tag3/4)+TextStyled(tag5) 追加式、queue 臂投影保真（scroll 裁剪栈/typography 差分/layout_block 垂直高度修正/命中区视口过滤）、iced 栅格化 with_clip+Font 差分、native HICON 真图标链（win32 提取+native_icon 缓存+hicon: 渲染方案串）、e2e_exe 陈旧防护+queue-coverage bin"
  - "auto-man/vue: desktop 宿主 scaffold 增壁纸层（assets/wm/Wallpaper.vue 三档 + host App.vue 生成期配置注入 shell.desktop.wallpaper）"
  - "design/autoui/desktop-protocol-v1.md: §1.5 v1.5 增量（scissor 栈语义/typography 差分通道）"
new_spec_components: []             # 无全新 spec 模块——全部为既有组件修改
touched_goals:                # 引用 docs/specs/goals.md 的 GOAL-NNN
  - "GOAL-009: 桌面线保真收口——queue 臂 scissor/typography、HICON 真图标、真 launch e2e 通道、vue 壁纸层、工具链可见性三件"
  - "GOAL-007: vue/VM 双轨壁纸 scrim 对齐（bg-background 10%/35% token 级 parity）"

affects: [auto-lang/ui]
current_step: 9
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

### 5. D 族判定表（G5，T1 定稿）

六项逐项：判定（纳入/不做/挂起）+ 理由 + 纳入者的任务映射。起草人
预判（T1 复核定稿）：HICON=纳入；窗口选择器=挂起（触发=真机使用反馈）；
拉流与延迟回调=不做（低频+复杂度高，理由成文）；DWM 缩略=挂起（触发=
真洞翻默认反馈）；vue 真缩略=挂起（触发=vue 远程形态落地后）。

**T1 判定表（定稿，2026-09-01）：**

| # | 项 | 出处 | 判定 | 理由 / 触发条件 | 任务映射 |
|---|---|---|---|---|---|
| D1 | HICON 真图标 | 473/486 两度延期 | **纳入** | 真机窗口图标当前为默认占位；宿主已持有 HWND，取 HICON 链路成熟（`WM_GETICON`/`GetClassLongPtr`），成本低、收益直接可见 | 本计划步骤 8 |
| D2 | 窗口选择器面板 | 486 判"需求出现再立" | **挂起** | 当前桌面形态单窗口为主，无切换使用面；触发=真机使用反馈出现多窗口切换需求 | 无（挂起登记债务簿，含触发条件） |
| D3 | 外来虚拟文件拉流 | 488 非目标+增强 | **不做** | 低频（外来虚拟文件为边缘场景）+复杂度高（跨进程文件流协议、生命周期管理）；显式不做防永久悬置 | 无（理由回写债务簿核销） |
| D4 | 真延迟回调 | 488 非目标+增强 | **不做** | 现有合成 tick 已满足动画需求；真延迟回调需引入定时器线程+跨线程回调安全，复杂度/收益不成比例 | 无（理由回写债务簿核销） |
| D5 | native DWM 缩略 | 497 待澄清②，挂 494 真洞反馈 | **挂起** | DWM 缩略图仅在真洞翻默认（remote 形态默认开启）时进入用户视野；触发=494 真洞翻默认的反馈出现缩略保真问题 | 无（挂起登记债务簿，含触发条件） |
| D6 | vue 真缩略 web 路径 | 497 待澄清① | **挂起** | vue 桌面宿主当前为本地渲染形态，真缩略 web 路径依赖远程形态；触发=vue 远程形态落地后 | 无（挂起登记债务簿，含触发条件） |

判定结论：纳入 1 项（D1→步骤 8）、不做 2 项（D3/D4，理由回写债务簿）、
挂起 3 项（D2/D5/D6，登记触发条件）。

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
   [✅ 已完成] 详细设计 §5 判定表定稿：纳入 1（D1 HICON→步骤 8）/不做 2（D3/D4 理由成文）/挂起 3（D2/D5/D6 触发条件登记）。
   验证：判定表成文。
2. **scissor 协议算子**：`crates/auto-lang/src/ui/desktop_protocol/message.rs`
   DrawOp 追加 Scissor（tag 顺延）+ codec/golden 单测。
   [✅ 已完成] Scissor{rect}(tag3)/ScissorPop(tag4) + round-trip + golden 字节锚点；编译器强制贯通宿主 broker_surface with_clip 栈语义 + parity 序列化臂；协议文档 §1.5 增量注记（栈深度定案）。`cargo t desktop_protocol --features ui-iced` 113/113 绿（ff2201970）。
   验证：`cargo t desktop_protocol --features ui-iced`。
3. **投影器+宿主贯通**：`crates/auto-lang/src/ui/desktop_protocol/client_runtime.rs`
   scrollable 裁剪栈 + 宿主栅格化裁剪 + TS 侧（`packages/drawlist-renderer/`）
   同步与 golden。
   [✅ 已完成] layout_scroll 臂（溢出产 push/pop + 命中区视口过滤 + 无固定高零扰动）3 单测；顺带修 layout_block 垂直块高度既有 bug（500 期误取 cross_max——parity 金样 2 份几何修正：card 紧包内容/重叠消除）；TS messages/render scissor 栈 + 对拍锚点 SCISSOR_FRAME_HEX；补 drawlist-renderer package.json（508 漏提交）。cargo t desktop_protocol 116/116 + vitest 22/22（19139f18e）。
   验证：`cargo t desktop_protocol --features ui-iced` + 渲染器包测试绿。
4. **typography 差分**：projector 透传 + 宿主差分渲染 + 金样用例。
   [✅ 已完成] TextStyled（tag 5：weight u16 + italic bool；常规文本仍产 Text 零扰动）；b/strong/heading/font-bold→700、em/i/italic→italic；宿主 iced Font 差分 + TS font 前缀；对拍锚点 STYLED_FRAME_HEX；金样 t2_typography/001 差分行。cargo t desktop_protocol 118/118 + vitest 24/24（d55fbb404）。
   验证：`cargo t desktop_protocol --features ui-iced`。
5. **端到端视觉断言**：013-notes（scroll 溢出）queue 模式渲染断言。
   [✅ 已完成] 资产勘误：013/015 均无 scroll、041 需交互才溢出——按 507 T10 先例构造 p515-scroll-overflow 专示例（静态溢出 + 嵌套 2 层 + 视口内外双按钮）；t3 e2e 断言合成帧 scissor 栈（双视口 + 平衡 + 嵌套恰 2 层 + 溢出内容保留）+ 命中区视口过滤。queue 臂断言通道 = 合成帧 DrawList（视觉词汇）+ 孪生布局；desktop_mcp/快照通道属 iced 直挂臂，不在本步范围。
   验证：desktop_mcp/快照断言绿。
6. **vue 壁纸层**：widgets 包桌面宿主壁纸层 + scrim 对齐 + vitest/截图
   对拍。
   [✅ 已完成] 壁纸层落位 assets/wm/Wallpaper.vue（465 宿主壳资产，三档：图片+scrim 10/35 对齐 VM / #hex 纯色 / 空不渲染）+ host App.vue 配置注入（生成期读 storage 同键——vue 无 storage 桥，按待澄清③降级判定注记差异：运行期改壁纸经下次生成生效）；p515 三档/层序/转义/scrim 对齐钉测试绿。**差异注记**：生成项目无 vitest/截图设施——vitest 组件测与双轨截图对拍降级为 token 级对齐钉（单测断言 scrim 双段类名与 VM pct 常量同源注释），视觉截图对拍挂 505 验收通道（步骤 7 通道复用）。预存红记档：d8_toggle_dark_mode 基线即败（dark_mode 全局态泄漏，非本计划域）。
   验证：vue 档测试绿 + 对拍留痕。
7. **C 族三件**：覆盖率 bin + `auto_exe` 陈旧警告 + launch e2e 通道留痕。
   [✅ 已完成] ①queue-coverage.json 侧信道（fence 测试重写）+ `cargo run --bin queue-coverage` 直读（69/388=17.8% 同源）；②e2e_exe 共用体：mtime 对账 + 陈旧警告不阻断 + AUTO_FRESH_EXE=1 强制重建，stage3/remote 委托 + 三态单测；③验收通道 p515 场景实跑 PASS——DesktopBus launch 记录 → 011-calculator 真启动置顶，截图留痕 docs/plans/reports/assets/505/p515/（P504-3 清偿）。
   验证：各自演练输出。
8. **纳入项执行**：按 T1 判定（预期 HICON）执行并清偿。
   [✅ 已完成] D1 HICON 真图标：win32 提取链（WM_GETICON 哨兵容错 + GCLP 兜底 + CopyImage 共享柄回退 + GetDIBits BGRA→RGBA）+ native_icon 缓存（幂等 ensure/失败占位）+ 投影 icon 字段 `hicon:<slot>` + 双渲染臂 raster 分支；真窗口 e2e（合成 .ico→LoadImageW 类图标→提取像素特征）PASS；顺修 geometry 类名 `\\0` 转义 bug；native-dock 档 49/49 + desktop_protocol 119/119。
   验证：对应债务行已清偿注记。
9. **收尾**：全部清偿/判定回写 KNOWN-DEBT；健康检查；状态翻
   execution_done。
   [✅ 已完成] KNOWN-DEBT 五项内联 ✅ 清偿 + P515 节（D 族判定收拢/顺带修复三条/预存红三处基线证实记档）；健康检查：cargo check 0 错、cargo t ui 1690/1691（唯一败 strips_tags 预存基线即红）、desktop_protocol 119/119、native-dock 档 49/49、drawlist-renderer vitest 24/24、auto-man 全绿（除预存 d8）；新增代码零警告（f26636d7f）。
   验证：`cargo check -p auto-lang && cargo t ui`。

## 复审记录

**复审人**：zhaopuming（/auto-plan:review，2026-09-02）
**复审树**：`.worktrees/plan-515-dev` @ f26636d7f（基线 09a2142c1，8 提交）
**方法**：信任代码不信任勾选——全量门 + 逐验收复跑 + 遗漏/延后/workaround 显式猎捕 + 基线归因对照。

### 验收逐条裁定

1. **A 族 ✅ PASS**——scissor 端到端：`t3_examples_queue_end_to_end` 实跑绿（p515-scroll-overflow 双视口 128/40 + 栈平衡 + 嵌套恰 2 层 + 溢出内容保留 + 命中区视口过滤，7 示例全孵化）；typography 差分金样：`t2_typography/001_queue` 差分行（w=700/it=true）+ `t6_typography_differential_channel`；协议追加式纪律：`per_channel_golden_bytes` 既有字节锚不变 + `rejects_unknown_tags` 拒收面 + Rust↔TS 对拍三锚点（HELLO/PRESS/SCISSOR/STYLED 四件）恒等。
2. **B 族 ✅ PASS（证据形态偏差一条，见债候选 P515-R1）**——`p515_host_wallpaper_layer_branches` 三档/层序（desktop-area 首子层）/转义安全/scrim 双段 token 对齐全绿；Wallpaper.vue 资产在库且进 wm 清单；生产接线 `generate_desktop_host`→App.vue 每次桌面运行刷新。偏差：渲染级截图对拍未做——vue 桌面宿主无 in-repo 视觉证据通道（D6 相邻域），vitest 组件测设施生成项目侧不存在；实质功能（层存在/三档分支/scrim token 与 VM pct 常量同源）已验证并透明记档。
3. **C 族 ✅ PASS**——`cargo run --bin queue-coverage` 实跑输出 69/388=17.8%（与 507 报告同源）+ `target/queue-coverage.json` 侧信道（fence 测试重写）；`stale_guard_detects_newer_source` 三态单测 + AUTO_FRESH_EXE=1 强制重建臂；launch e2e 留痕在库（`docs/plans/reports/assets/505/p515/p515-01-calculator-launched.png`，2560×1600，截图证实 011-calculator 真启动置顶）。
4. **D 族 ✅ PASS**——判定表六项定稿于本计划 §5；纳入项 D1 HICON 全链落地（win32 提取链含 WM_GETICON 哨兵容错/GCLP 兜底/CopyImage 共享柄回退 + native_icon 缓存 + 双渲染臂 + 真窗口 e2e `window_icon_rgba_extracts_scratch_window_icon` PASS）；不做 D3/D4 理由、挂起 D2/D5/D6 触发条件均回写债簿 P515 节（复核文本在库）。
5. **回归/零警告 ✅ PASS**——**全量门**：`cargo tf --features ui-iced --no-fail-fast` = 4389/4392（108 skipped）；**基线归因对照**：分支基线 09a2142c1 同套件同 profile = 4380/4383，3 个失败**逐字相同**（d8_toggle_dark_mode / style_migration_probe / strips_tags_and_decodes_entities）→ 515 净回归 **0**、净增 9 测试全绿；TS vitest 24/24；`cargo t -p auto-man` 4630/4633 同 3 预存；`cargo check -p auto-lang` 0 错；新代码零警告（native_icon.rs/queue-coverage.rs/render.ts/Wallpaper.vue 无 TODO/FIXME/HACK，无 .wrong.txt 残留，无调试打印残留）。

### 遗漏 / 延后 / workaround 猎捕结果

- **遗漏**：无——9 步骤全有对应 diff 与验证证据；D 族回写、KNOWN-DEBT 五项内联 ✅ 复核在库。
- **延后**：1 项证据形态（B 族截图对拍）→ 债候选 **P515-R1**（根因：vue 桌面宿主渲染级视觉证据通道在本仓不存在——生成项目无测试装配、无截图管线；触发条件与 D6 同源）。已在计划簿记/债簿/交接摘要三处透明披露，非静默。
- **Workaround**：无强加之脏补。已文档化边界三条：mask-only 单色图标不支持（dib_icon_to_rgba 注释）、WM_GETICON 哨兵值容错（候选序列首个可解者胜）、壁纸数据源配置注入降级（计划待澄清③预判路径）。
- **计划外顺带修复三条**（同根施工面，已记债簿 P515 节）：layout_block 垂直块高度 500 期 bug（parity 金样 2 份几何修正）；geometry scratch 类名 `\\0` 转义 bug；drawlist-renderer package.json 508 期漏提交。

### 债候选（登记）

- **P515-R1** vue 壁纸层渲染级对拍缺位（截图/vitest 组件测）——token 级对齐已钉，视觉证据待 vue 桌面宿主证据通道（触发=vue 远程/验收通道 vue 档落地，与 D6 同源）。
- 预存红三处（449/370/055 域）已在债簿记档，非本计划引入、非本计划清偿域。

### 裁定

**PASS**——五条验收实质全过、净回归 0、追加式纪律核验成立、无静默延后。状态翻 `reviewed`，可进入 `/auto-plan:merge`。

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
