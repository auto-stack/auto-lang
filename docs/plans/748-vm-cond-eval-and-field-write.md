---
plan_id: PLAN-748
status: drafting               # drafting → executing → execution_done → reviewed → archived
feature_name: VM 模板条件求值/续体字段写/动态样式三断点修复（auto-musk PLAN-101 上游缺口）
author: [agent]
created_at: 2026-10-10
updated_at: 2026-10-10

# /auto-plan:review 结束时填写：
supersedes_spec_components:
  - docs/specs/auto-lang/vm/architecture.md
new_spec_components: []
touched_goals:
  - GOAL-007

affects: [auto-lang/vm]
current_step: 0
total_steps: 6
plan_revision: 1
---

# PLAN-748 — VM 模板条件求值/续体字段写/动态样式三断点修复

## 变更摘要

auto-musk PLAN-101（2026-10-09 实机）登记了三个 VM 渲染/执行引擎断点并
以 workaround 绕开（musk `docs/plans/101` §10），本计划在 auto-lang 侧
根修：

1. **模板 `if` 条件成员访问恒假**——`for` 循环行数据的 `if w.is_current`
   不渲染（裸 binding `if .collapsed` 正常；成员读在 text/attr 位正常）。
2. **handler 续体静态名字段写崩溃**——await/park-resume 后续体中对 JSON
   局部对象 `item.x = v` → `RuntimeError("execution terminated unexpectedly")`
   （段名挂外层 handler）；动态键 `item["x"] = v` 与同 receiver 正常。
3. **动态 style 字段引用半支持**——`style: .w.row_style` 在部分元素臂被
   静态提取丢弃（`extract_style` 只认字面量），绑定感知通道
   `extract_style_with` 未全元素收口。

修复后 musk 101 的绕开代码（mark 预拼 / JSON 克隆+动态键 / 静态样式串）
**回退为直读条件并双端对拍通过**（101 §10 收口候选）。

## 目标

1. 复现并定音三个断点的真实根因（master 代码上三处"看起来都有臂"，
   不许在未定音前打兜底补丁——renderer.rs:16457 "静默塌缩恒 false"
   历史教训）。
2. 断点①：模板条件求值对循环行数据成员访问、`.store.X`/computed 混合
   形态在**根件与子件两上下文**均返回真值（子件面嫌疑最大：
   P733-R2 预存红族同族）。
3. 断点②：handler 续体内静态名/动态键/循环绑定写三形态与无 await 对照
   组行为一致（成功或响亮的具体 Err），不再 Terminated。
4. 断点③：`style:` 动态字段引用全元素产出（静态臂收口
   `extract_style_with`），词表过滤至少保留 button 臂同款 WARN。
5. 跨仓回退对拍：musk PLAN-101 worktree（5b2fa9f）的绕开代码回退版在
   本计划构建上 probe 全绿；musk 侧 ui-parity 无新增红。
6. 既有回归零新增红（tv/tf/RC + daily 对拍；**master 预存红 P733-R2 六条
   为零增减基线，不是本计划修复对象**）。

### 非目标

- 不改 a2vue/web 生成器面（style "四形态"是 web 轨语义登记，VM 解释器
  通道归本计划，生成器不动）。
- 不修 P733-R2 预存红本体（子件 override `.store.X` 深层链如经 T-00
  定音与断点①同根则顺带修复并在 §9 说明；否则登记边界）。
- 不处理 auto-lang 主检出上的他人 WIP（媒体引擎 8 文件，见 §4 风险）——
  本计划 worktree 基面恒钉 master 已提交代码。
- 不动 park/resume 协议本身（Stakes/RC 敏感区，419/510/624/733 四轮
  修过；修复落在错误处理/求值侧，非弹栈侧）。

## 架构方案

```text
T-00 定音探针（干净 master worktree 重建 auto.exe——musk 实机用的是
     dirty 构建 v0.4.2-2730-g91ae002d3-dirty，主检出另有媒体 WIP，
     全部观测必须用干净基面重取）
  ├─ ① 三形态×两上下文条件矩阵（w.is_current / .store.X==w.id /
  │    computed+bare）×（根件/子件挂载），headless 快照 + VmBridge
  │    new_from_decls 双通道，先定罪到求值器臂/子件挂载/别名快照哪环
  ├─ ② resume FAILED 补 crash-ip 反汇编窗（镜像 vm_bridge.rs:2922
  │    首派形态，补 :1829 resume 路径）+ AUTO_VM_TRACE_OPS 末 50 指令
  │    + 非续体对照组 → 定音 intercept_error 残余 vs 帧 bp 协议
  └─ ③ style：静态臂→extract_style_with 收口（无条件腿，与①②并行）
T-01 断点②修复（按定音：intercept_error/engine.rs:11251 面或
     handler_codegen catch 目标面，红相 spike 先行）
T-02 断点①修复（按定音：eval_computed/store 别名展平挂载面或
     eval_condition_with_inner 补臂，红相矩阵先行）
T-03 断点③收口 + 词表 WARN 对齐
T-04 跨仓回退对拍（musk 101 workaround 回退版 probe + ui-parity）
T-05 全量门 + 提交 + 交接
```

## 需求分析与背景调查

### 授权

用户 2026-10-10 指示"起草修复计划吧"（PLAN-101 §10 三断点的 auto-lang
侧根修）。本阶段只起草；work 授权待用户确认计划后另行给出。

### 已取证事实（PLAN-101 实机，2026-10-09，auto.exe dirty 构建）

| 断点 | 症状 | 对照正常形态 |
|---|---|---|
| ① if 成员访问 | `if w.is_current` 恒不渲染（字面 id 重标后仍假） | `if .collapsed`（裸 binding）、`text .w.name`、`title: .w.path` |
| ② 静态名写 | 续体 `item.x=v` → Terminated（外层段名） | `item["x"]=v` 同 receiver 同续体通过 |
| ③ 动态 style | `style: .w.row_style` 不产出 | 静态串、`if 裸binding{全静态串}` |

### 代码定位（master @ 7bd28888b，调查代理只读实读）

基线 hash（SHA256 前 16）：engine.rs `93a73e6d2df5983e`、
aura_view_builder.rs `f986554d393ad093`、vm_bridge.rs `54487aad1ffb5191`、
vm/codegen.rs `139f1baea40fa564`、handler_codegen.rs `408566622fe0d7fb`。

- **断点①链路**：parser.rs:16390/16532 条件逐字保留（成员链显式拼点）
  → aura/extract.rs:1127 → `AuraNode::Conditional`；求值唯一权威
  `aura_view_builder.rs:12828 eval_condition_with`（iced renderer 不参与；
  vnode_converter 只是序列化）。成员访问臂 :12941→:12943
  `resolve_binding_path`、RHS 成员 :13048、`.store.X` :12996→
  `resolve_expr_to_value` Dot 臂 :12357（`Ident("store")` 特判）、computed
  兜底 :12930→`eval_computed` :12262。**单测
  `test_eval_condition_with_bindings`（:19019，含 `.filter=="active" &&
  todo.done==false` 复合形态）master 实跑通过**——求值器"看起来有臂"。
  `.store.X` 展平依赖 handler_codegen.rs:184 线程级名快照 + bridge
  每组件存档（PLAN-642）双通道；快照空时静默落空。
  **嫌疑排序**：(a) 子件渲染上下文 computed/store 面断裂（P733-R2 预存红
  族 `musk_vm_track_p053_1_widget_computed::*` master 实跑 FAILED 同族）；
  (b) 时序/数据面（101 自身竞争已另行修复）；(c) 求值器真缺臂（低）。
- **断点②**：错误唯一抛出点 `engine.rs:2794`（`drive_handler_segment`
  内 `StepResult::Terminated`；**fn_name 是段名非死点函数**）。SET_FIELD
  实现 :6566-6808 **无任何路径返回 Terminated**（成功或具体 Err）——
  真实错误被 try 拦截后 catch_pc 坏跳 / 帧弹穿 / ip 出界，与 PLAN-705
  已修族（"深帧 try 经 park/resume 后 catch 不生效→ip 出界"，
  engine.rs:11273-11286）同症状族。SET_FIELD（:6566）与 SET_ELEM（:6406）
  引擎臂同构（弹 3 nv + stake 结算），codegen 层 :7129-7396（Dot→
  SET_FIELD / Index→SET_ELEM；静态路径多 `SET_GENERIC_FIELD` 推断分支
  :7255）。字符串池 pinned，resume 后失效假设可排除。嫌疑：静态写路径
  receiver 求值序列与续体栈布局交互 → SET_FIELD 返回 Err → 续体+嵌套
  帧+try 组合触发 705 族残余。
- **断点③**：`style:` prop 仅对象字面量形态走 `StyleBinding`（parser
  :15864）；`extract_style`/`extract_string` 只认 `Expr::Str` 字面量，
  动态 Expr 静默 None——仍有静态臂的转换器：tabs/center/video/
  progress/spacer/checkbox（:1893/:3307/:8118/:8265/:8326/:11950）；
  `extract_style_with`（:14080, Plan 057）已接 row/col/scroll/container/
  input/textarea，**button/div/span/command 已是绑定感知**（:10901/
  :10678）——故 musk"style 不产出"证据同样需干净构建复验。第二丢弃面：
  `Style::parse` 词表过滤静默丢（button 臂有 446-U7 WARN :10910，其余臂
  连 WARN 都无）。
- **733 先例**（archive/733，三腿修复 0b5c8758d/481bcb0b9）：调用返空族；
  验证=矩阵单测 `plan733_fn_call_semantics_tests` + daily 回归与 master
  基线逐名对拍（预存红零增减）+ tv 全量 + 跨仓 probe-g15.mjs（musk 零
  改动）。本计划照搬其"先 bounded 调查定音再动刀"流程。
- **预存红基线（master）**：P733-R2 六条（`musk_vm_track_p053_1/p053_4/
  p053_6/p054` widget_computed 族 + `widget_computed_store_arg_helper_chain`
  /`widget_computed_passthrough_survives_reeval`）——修前修后零增减，
  若 T-00 定音与之同根则一并修复并改判基线。

### 风险与约束

- **主检出 WIP**：auto-lang master 工作区有未提交改动 8 文件
  （aura_view_builder/renderer/session/view/video_uplink/mpv + 020-music-
  player 示例，2026-10-10 媒体引擎方向，与本三断点无关）——**他人 WIP，
  不归本计划**；worktree 基面钉 master 已提交代码，rebase 前须与该 WIP
  owner 协调（向用户上报，不静默包含/丢弃）。
- **敏感度**：Stakes/RC（419/510/624/733 四轮修复区）——改动落错误
  处理/求值侧，禁碰弹栈协议；renderer/aura_view_builder 补丁密度极高，
  禁打静默兜底补丁（恒 false 塌缩史）。
- 观测约束：musk 全部实机证据来自 dirty 构建 + 当日 master 正被活跃
  开发，**T-00 干净基面重取是硬门槛**，不得直接引用 101 的观测定罪。

## 详细设计

### T-00 定音探针（判定产物：定音报告，含每断点根因层 + 修复路线）

1. **干净基面**：`git worktree add D:/autostack/.wt/lang-748/auto-lang -b
   plan-748-dev master`（或 new-wt-group.sh）→ 仓内 cargo 构建 debug
   auto.exe，固定 `AUTO_EXE`。
2. **断点①矩阵**：最小 .at 语料（fixture widget：根件 + 子件各一），
   条件形态 `{w.is_current / .store.x==w.id / computed && w.id}` × 挂载
   上下文 `{根件/子件}`，headless 快照（`ui/headless/renderer.rs` 通道）
   + `VmBridge::new_from_decls` 双通道取真值；对照 ARM：
   `musk_vm_track` 预存红测试在干净基面的失败面逐条记录。
3. **断点②探针**：resume FAILED 路径补 crash-ip + 反汇编窗
   （vm_bridge.rs:1829 镜像 :2922 形态，含 disasm 窗口 20 指令）；
   三臂语料（静态写/动态键写/循环绑定写）× `{有 await, 无 await}` 对照；
   `AUTO_VM_TRACE_OPS=1` 抓续体末 50 指令。定音：Err 被谁吞、catch_pc
   还是帧展开、是否 705 族残余。
4. **断点③核验**：干净基面跑 musk 101 选择器行（`style: .w.row_style`
   形态最小化）确认丢弃是否复现于干净构建；清点静态臂清单。
5. **判定纪律**：每断点输出"根因层（文件:符号）+ 修复路线（A/B 案）+
   回归面"；证据不足时扩探针不猜。

### T-01 断点②修复（红相先行）

按定音落点二选一：
- **路线 A（Err 吞没面）**：`intercept_error`（engine.rs:11251）残余 +
  handler_codegen catch 目标生成；修"错误发生在 try 同帧被调 RAM 帧且经
  park/resume"组合的 catch_pc 错位。
- **路线 B（帧协议面）**：段驱动嵌套 CALL 帧 bp 协议（624 族 push/pop
  对称性）。
红相 spike 语料（705 形状）：handler 内 await HTTP → resume 后静态写 +
  动态键写 + 循环写三臂断言段 Completed(Ok)；测试挂
  `plan748_engine_resume_field_write_tests`。

### T-02 断点①修复（红相矩阵先行）

按定音落点二选一：
- **路线 A（子件挂载面）**：computed 表 / store 别名快照在
  DynamicComponent 子件构建时的挂载（vm_bridge `new_from_decls:741`
  一带 + handler_codegen.rs:184 快照机制）；若与 P733-R2 同根则合并
  修复并改判预存红基线（§9 说明 + KNOWN-DEBT 条目更新）。
- **路线 B（求值器补臂）**：`eval_condition_with_inner` 具体形态补臂
  （可能性低，改动最小）。
单测扩 `test_eval_condition_with_bindings` 矩阵：成员真值/RHS 成员/
  store 混合 × 根件/子件。

### T-03 断点③收口

静态臂转换器（tabs/center/video/progress/spacer/checkbox）
`extract_style` → `extract_style_with` 机械收口；`Style::parse` 词表
丢弃在非 button 臂补 446-U7 同款 WARN；`tests/style_parity.rs` +
038 minesweeper 动态样式 e2e 回归。

### T-04 跨仓回退对拍

musk PLAN-101 worktree（.wt/musk-101/auto-musk @ 5b2fa9f）起两 variant：
1. **workaround 版**（现状）：发送闭环 + 选择器 ✓ probe 全绿（基线）；
2. **回退版**（patch 临时还原：`if w.is_current` 直读 + `item.x=v`
   静态写 + `style: if w.is_current {...}`）：本计划 auto.exe 下必须
   同等全绿——这是三断点修复的权威验收。
锁 AUTO_EXE 指向 748 worktree 构建；musk 侧 ui-parity check 无新增红。

### T-05 全量门与收尾

`cargo test` 受影响套件（tv 3731 / tf 3560 / RC 生命周期 /
plan702/705/707/711 segment 面 / plan733 矩阵 / musk_vm_track——
预存红零增减）+ auto-lang `scripts/ui-parity.mjs run --plan 748`（如
有对拍 case）+ daily 回归与 master 基线逐名对拍。提交 + §9 记录 +
KNOWN-DEBT-AND-RISKS.md 条目更新（三断点销账/新边界）。

## 测试设计

| 验证 | 命令/方法 | 期望 |
|---|---|---|
| V01 定音探针 | T-00 探针套件（仓内 test + headless 快照） | 三断点根因层判定报告；干净基面复现成功；非续体对照组结论 |
| V02 红相 spike | `plan748_engine_resume_field_write_tests` + 条件矩阵单测 | 修复前红（绑定干净基面对应形态）、修复后绿 |
| V03 全量回归 | `cargo test`（tv/tf/RC/segment/plan733/musk_vm_track） | 零新增红；预存红六条零增减（若同根修复则改判并说明） |
| V04 跨仓对拍 | musk 101 workaround/回退双 variant probe + ui-parity | 双 variant 同绿；musk 无新增红 |
| V05 daily 对拍 | daily 回归 vs master 基线逐名 | 零增减 |

## 验收标准

| ID | 可观察结果 | 验证 |
|---|---|---|
| AC-01 | 干净基面复现三断点（或证伪并改判——dirty 构建/WIP 污染情形须报告） | V01 |
| AC-02 | handler 续体静态名/动态键/循环绑定写三形态与无 await 对照组行为一致，无 Terminated | V02/V03 |
| AC-03 | 模板条件成员访问（含子件上下文、store/computed 混合形态）求值真值正确 | V02/V03 |
| AC-04 | `style:` 动态字段引用在静态臂元素产出；词表丢弃有 WARN | V03 |
| AC-05 | musk 101 回退版（直读条件）在本计划构建跨仓 probe 全绿 | V04 |
| AC-06 | 全量零新增红；预存红零增减（或同根改判记录在案） | V03/V05 |
| AC-07 | 禁静默兜底补丁：三修复点均可在单测中观测到真值行为差异（非"看起来有臂"） | review 绑定代码+测试 |

## 执行步骤

| 完成/任务 | 依赖 | 文件/符号与产出 | 验证 | AC |
|---|---|---|---|---|
| [ ] T-00 | 无 | worktree lang-748（plan-748-dev，基 master 7bd28888b）；干净构建 AUTO_EXE；三断点定音探针套件 + 判定报告 | V01 | 01 |
| [ ] T-01 | T-00 ②定音 | intercept_error/catch 目标或帧协议面修复 + spike 测试 | V02/V03 | 02 |
| [ ] T-02 | T-00 ①定音 | 子件挂载面或求值器臂修复 + 条件矩阵单测（含 P733-R2 同根判定） | V02/V03 | 03 |
| [ ] T-03 | T-00 ③核验 | extract_style_with 收口 + WARN 对齐 | V03 | 04 |
| [ ] T-04 | T-01～T-03 | musk 101 双 variant 跨仓对拍（AUTO_EXE 锁 748 构建） | V04 | 05 |
| [ ] T-05 | T-04 | 全量门 + 提交 + §9 记录 + KNOWN-DEBT 更新 + owned 清理 | V03/V05 | 06,07 |

## 复审记录

### new 阶段交接（草稿准备完成）

- stage: new
- plan_id: PLAN-748
- plan_revision: 1
- outcome: pass
- next: work（待用户确认计划后授权；主检出 WIP 8 文件的 owner 协调
  为 work 前置面，向用户上报）
- changed_tasks: T-00～T-05
- changed_acceptance: AC-01～AC-07
- evidence: 本节§需求分析（双调查代理文件:行号定位 + 101 实机观测 +
  master 单测/预存红实跑）；基线 hash 与 733 先例。
- review_authority: 独立 auto-plan:review 复验定音报告与修复证据。

## 待澄清事项

1. **主检出 WIP owner 协调**（owner=用户/WIP 所属会话）：master 8 文件
   媒体引擎 WIP 与 aura_view_builder.rs/renderer.rs 同文件——748 work
   前需确认 WIP 归属与 rebase 策略（本计划 worktree 独立，日常不冲突；
   收尾 rebase 时点是协调点）。
2. **P733-R2 同根判定**（owner=T-00）：若断点①定音与子件 override
   `.store.X` 预存红同根，修复范围并入 T-02 并改判基线；异根则登记
   边界与后续计划指针（同 733 对 computed 返空面的处理先例）。
3. **dirty 构建污染面**（owner=T-00）：101 的三项观测若部分在干净基面
   不复现（WIP 引入或已修），按实际定音收缩范围——计划允许 T-00 后
   修订任务/验收（revision 递增），不预设结论。
