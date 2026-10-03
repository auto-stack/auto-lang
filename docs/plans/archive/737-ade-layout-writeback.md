---
plan_id: PLAN-737
status: archived               # drafting → executing → execution_done → reviewed → archived
feature_name: ade-layout-writeback
author: [zhaopuming]
created_at: 2026-10-03
updated_at: 2026-10-03

# /auto-plan:review 结束时填写：
supersedes_spec_components: []
new_spec_components: []
touched_goals: []             # 引用 docs/specs/goals.md 的 GOAL-NNN

affects: [auto-lang/ui]       # autodown_editor（观测面）/ iced renderer（派发表+栅栏）/ mcp_server（栅栏发送）
current_step: 5
total_steps: 6
---

# [PLAN-737] ade-layout-writeback

## 变更摘要

跨仓缺陷修复单（消费方 jade-edit PLAN-037 T-02 解锁门）：PLAN-732 已交付的
原生 wikilink 供给在真实 merged 生产窗口不可激活。插桩实测**推翻缺陷单的
"DocLayout 写回脱节"假设**——布局写回每帧健康（w=876、blocks=8、links=5、
press 逐块命中正确），真实根因两个，均已修复：

1. **R-1（主根因）合成通道丢激活**：MCP `editor_drag`（`__mcp_drag_ade`，
   renderer.rs ~17731）直调 `core.handle_input`（057 keyed 寻址、不经
   widget），只消费 `focus_changed`/`text_changed`——PLAN-732 新增的
   `DocOutput.link_activated` 在该通道被整条丢弃。插桩铁证：点击门
   `same=true` 18 次恒成立、widget.publish（ACTIVATED）0 次。真实事件泵
   （widget update 臂）路径无此缺陷（六环语料 13/13 绿的原因）。
2. **R-2（伴生）合成动作"入队"语义**：MCP 动作经 channel→60fps 泵→
   update 异步应用，工具立即返回——探针 press→立即读状态与泵竞速，
   focus 带映射系统性偏移（缺陷单"DocLayout 退化/focus 带"实为该读数
   伪影：插桩证明布局健康，本侧复现 block0 首中 y=51 漂移）。

修复后实机验证（真实 jade 应用 merged 窗，工作树 binary）：块带映射 8 块
全对齐（block0@y6=真实位置），jade 探针 N2（Goals 真实点击→打开，
nav_seq 恰 +1）、N3（目标页）、O1..O5（双实例 origin 隔离全绿）PASS。

## 目标

- G1 ✅：根因定谳（R-1/R-2 如上；"DocLayout 写回脱节"假设被插桩否定）。
- G2 ✅：修复——真实 merged 窗 editor_drag 语义完整点击经 core 门激活并
  到达 VM handler（nav_seq +1、页面打开）；SD-01/K 条款零弱化（真实事件
  泵路径零改动；`__mcp_click` press-only ghost 负例语义不变）。
- G3 ✅：13/13 plan732 零回退；新增 2 个真实节拍回归测试锁定。
- G4 ✅：ECONNRESET 定谳（见「详细设计」§4）。
- G5：merge 后 master 重建 debug binary，交付回执注明新指纹。

## 架构方案

供给链（实勘口径）：lower 期 `autodown_editor_sync`（renderer.rs）→
`sync_external` → `rebuild_with`；布局写回唯一写点 `render_frame`
（widget.layout/draw 每帧调用）；MCP 读数=core 直读；点击通道
`__mcp_drag_ade` 直调 `core.handle_input`；`autoui_editor_state.text`
=core `emit_document`。

## 需求分析与背景调查

见「变更摘要」与缺陷单全文。关键实证节拍（AUTO_ADE_TRACE 插桩，
master@7b93bfcc5 复现 → 修复后复验）：

```
+0.000s sync UNREGISTERED（首帧 no-op——sync 先于 DocEditor::new 注册，回执⑤口径复证）
+0.008s rebuild_with（次帧重降层补内容——SD-01 ⑤注意点按设计工作）
+0.461s sync REBUILT → render_frame WRITE w=876 blocks=8 links=5（布局写回每帧健康）
PRESS (30,6) hit=Some(0) links=5 pend=Some(0)（press 逐块命中正确）
RELEASE gate same=true ×18 → ACTIVATED publish ×0（R-1 铁证）
```

## 详细设计

### D-1 wikilink 激活合成通道派发表（R-1 根修）

- **注册表**：renderer.rs `ADE_LINK_DISPATCH: OnceLock<Mutex<HashMap<
  String, LinkCallback<IcedMessage>>>>`，键=storage key 归一形。
- **装配**：`convert_view_messages` AutodownEditor 臂（每次脏帧重建）以
  **与 widget `on_link` 同一 LinkCallback 闭包**（消息构造单源：Typed 双
  Str → from_dynamic/encode_payload）注册——无第二套事件名/载荷拼装。
- **消费**：`__mcp_drag_ade` 抬起腿放行的 `link_activated` →
  `ade_link_dispatch_message(sk, act)`（查表前经 `normalize_payload_key`
  幂等归一——MCP 载荷 sk 为裸键，探针 MISS 实录后定谳）→ `Task::done`
  消息，排在其臂既有 `__noop` 之前。未注册（未挂载/泛型轨）None=零派发
  （既有 keyed 通道语义）。
- **契约边界**：真实事件泵路径零变化（widget 仍经自身 on_link 发布，
  C-04 结构性实例绑定不动摇）；本表是 MCP 合成通道的 keyed 寻址补充
  （与 `__mcp_key`/`__mcp_drag_ade` 既有寻址模型同域），生命周期与编辑
  壳 core 注册表同键同域（同 key 覆写、无 dispose 面——core 注册表同样
  无过期）。O 臂（edA/edB 双 key）实测各发各来源零冒领。

### D-2 合成动作「已应用」栅栏（R-2 根修）

- `SharedState::send_action_applied(handle, msg)`：`__mcp_*` 合成事件在
  `ActionTarget::Event.event` 尾附 `|ack=<id>` → 发送 → 限时轮询回执
  （1s 上限、5ms 步；**轮询期不持锁**——回执与本句共用同一 mutex，持锁
  轮询自锁死）。非合成事件（handler 名直派：press/type_text/键盘）保持
  既有异步语义零改动。超时不报错（动作仍在队列，最终应用——尽力同步）。
- 应用侧：update 入口 pre-block 统一剥离 `|ack=`（臂匹配零改动）；
  **noop 臂回执**（动作臂回发的 `__noop|ack=<id>` 排在其 Task 队列尾，
  消费即"该动作含 Task 级联已全部应用"——link 消息派发先于 noop）+
  无 noop 臂（scroll/drag/pen/click 无焦点路径）臂尾 `mcp_ack_apply`
  直执回执。回执复用 fixture ack 表（`next_fixture_id`/`finish_fixture`
  /`take_fixture_ack`——同 id 空间、同 64 容量纪律）。
- 8 个 `__mcp_*` 臂全覆盖（resize_col/click/key/drag_ade/scroll/drag/
  pen + noop 回执点）；解析失败路径不回执（工具侧 1s 超时兜底，非热点）。

### D-3 常驻观测面（AUTO_ADE_TRACE=1；零门控开销）

- `ade_trace`（core.rs，一次性 env 检查）。
- `RELEASE gate (x,y) pend=(block,lo,hi,target) same=`（点击门核对结果
  ——本缺陷家族的定位锚）。
- `mcp_drag link_activated target= anchor= dispatched=`（派发腿结果）。
- `link_dispatch MISS sk (table keys=…)`（键失配定位——诊断期立功）。

### D-4 ECONNRESET 定谳（G4）

- **无 panic 路径**：press 链 `hit_test_strict` 位置索引天然在界、
  blocks 访问全 `get/get_mut`、release 腿只读 layout+blocks（锁序单一，
  Mutex 无重入）；`render_frame` 的 `panic!("render covers every block")`
  被 walk+防御补尾覆盖（None⟺hidden，构造不可达）。密集扫描实测：单轮
  102 press + 39 release 直调，多轮复跑应用存活零退出；trace 无 panic
  痕迹。
- 一过性 "fetch failed"（本侧 1 次）：与**同端口僵尸应用竞争**相关
  （诊断 harness 固定端口复用所致；换随机端口后未再现）。定谳：
  **非 panic、非本修复根因族，探针环境相关性**——jade 侧两次 ECONNRESET
  与其探针 60s 窗口/端口环境对齐；非阻塞登记，复验时若再现按
  环境排查（AUTO_ADE_TRACE/AUTO_SCHED_DIAG 双观测面已常驻）。

### D-5 N4/P2 残余定谳（消费方域，非本计划缺陷面）

- **N4（别名页）**：供给链全通（SCHED-DIAG 证明 update 收到
  `OpenWikiLink s别名页 s`；trace `dispatched=true`；单次点击实测走到
  jade handler 的 missing-confirm 分支——`create_confirm_open: true`）
  ——**jade `resolve_wiki_link`（其 back/wsys.at；四级解析归消费方后端，
  SD-01 §1 明文本层不做）未把 别名页 解析到 AliasTarget.ad**。jade
  复验时按其 Vue 轨行为对齐其 fixture/解析面。
- **P2（脏源点击）**：探针在块 0 键入 XYZ 后点击块 0 自己的
  [[Goals]]——冻结契约 U-02/`plan732_local_edit_deactivates_until_rebuild`
  （K 条款）语义=被编辑块链接暂态失活直至外部真变化 rebuild；自回显走
  PLAN-057 回声守卫不重建。探针前提与冻结契约相抵，非回归（换非编辑块
  的链接点击不受影响——块级失活粒度）。

## 测试设计

- `plan737_mcp_drag_single_point_full_click_dispatches_wiki_activation`
  （renderer.rs）：生产节拍全链——首帧 sync no-op（UNREGISTERED）→注册→
  次帧 sync 重建→render_frame 布局写回→单点 press+release（__mcp_drag_ade
  同构直调）→门放行激活 payload 断言→**裸键**派发（键幂等归一）→
  IcedMessage 解码断言（OpenWikiLink + 双 Str 逐值）+ 归一形键直查 +
  未知键零派发。
- `plan737_press_only_still_never_dispatches`：press-only ghost 语义
  阴性锚（K-04 冻结负例不因派发表弱化）。
- 零回退：plan732 13/13、scoped `cargo t autodown` 79/79、ui-iced 单开
  feature 编译干净（新测试模块 `cfg(all(test, autodown, code-editor))`
  门控）。

## 验收标准

- AC-01 ✅：根因定谳在案（R-1/R-2 + 插桩证据 + ECONNRESET 定谳）。
- AC-02 ✅：真实 merged 窗（探针 N 臂形态）editor_drag 单点完整点击激活
  （nav_seq +1、页面打开）；块带映射逐块对齐。N2/N3/O 组 PASS 实证；
  N4 残余=消费方别名解析（D-5，非供给面）。
- AC-03 ✅：plan732 13/13 + scoped autodown 79/79 零回退；复审裸
  cargo t 见复审记录。
- AC-04 ✅：新增真实节拍回归测试 2/2 绿（修复前红态=探针 N 臂两轮本侧
  复现 + 插桩 ACTIVATED×0 铁证）。
- AC-05：merge 后 master 重建 binary + 回执指纹（T-06）。

## 执行步骤

- T-01 工作树 + 插桩诊断 [✅ 已完成]
  证据：AUTO_ADE_TRACE 节拍实测（详见「需求分析」节）；根因 R-1/R-2
  定谳；"DocLayout 写回脱节"假设否定（布局写回每帧健康、press 逐块
  命中正确）。
- T-02 修复设计定稿 [✅ 已完成] D-1 派发表 + D-2 栅栏 + D-3 观测面。
- T-03 修复实现 [✅ 已完成]
  renderer.rs：ADE_LINK_DISPATCH 注册表 + convert_view_messages 装配 +
  `__mcp_drag_ade` 消费腿 + `__mcp_*` pre-block ack 剥离 + noop 臂回执
  + mcp_noop_with_ack/mcp_ack_apply；core.rs：ade_trace + RELEASE gate
  观测 + normalize_payload_key pub(crate)；mcp_server.rs：
  send_action_applied + 8 合成通道发送点切换（非合成事件零改动）。
- T-04 真实节拍回归测试 [✅ 已完成] 2 测试（见「测试设计」），2/2 绿。
- T-05 门禁 [✅ 已完成] cargo check（autodown/ui-iced/默认三 feature
  集）零错；plan737+plan732 15/15；cargo t autodown 79/79；复审裸
  cargo t 见复审记录。
- T-06 复审 → merge → master 重建 binary → 回执 → worktree 清理 [✅ 已完成] merge 59ff4e66f；master binary v0.4.2-2702-g59ff4e66f SHA256 43355F6536FC36CCBA14D8155544CCA40F537C76280FC6D820F95A7CEE628F43（回执 docs/reports/p737-ade-layout-writeback-receipt.md §4，交付件自检 N0..N3 PASS）；wt-guard clean、worktree/分支/组目录全清。

## 复审记录

### merge 收据（2026-10-03）

- merge 59ff4e66f（分支 plan-737-dev 5d967320e..b88a2e3a2 基面 ccc3a2f05）。
- master 重建 binary + 指纹 + 交付件 jade 探针 --phase native 自检 N0..N3
  PASS（N4=消费方域，回执 §5）。
- KNOWN-DEBT：P737-D1（MCP current-thread runtime 结构弱点/残余复位）/
  P737-D2（键域双形态归一口）登记在案。
- 本件归档 docs/plans/archive/（终态）。

2026-10-03 独立复审（本会话，缺陷单授权的执行+复审+merge 全程）：**pass**。

- Checklist audit：AC-01..05 逐条对照实证（AC-01 插桩证据在案；AC-02 实机
  N2/N3/O 组 PASS + SCHED-DIAG 消息到达证明；AC-03 15/15+79/79+裸 cargo t
  14 预存红与 master 基线逐名 diff 全等 + 2 flake（state_file 锁/
  plan730_commit_target_matrix）隔离复跑绿=零新红；AC-04 新增 2 测绿、
  修复前红态=探针两轮本侧复现+ACTIVATED×0 铁证；AC-05 待 merge 后重建
  binary 回执）。
- 遗漏/延后/workaround 扫描：无静默延后——N4/P2 消费方侧残余与 ECONNRESET
  基础设施债均明文登记（D-4/D-5 + KNOWN-DEBT 登记）；无契约弱化（真实事件
  泵路径零改动、press-only ghost 负例与拖选/多击/修饰键负例语义全部保留，
  plan732 13/13 为证）。
- Health check：三 feature 集编译零错零新增 warning（告警集与 master 同
  基线）；cargo fmt --check 我方三文件零 diff（仓内 28 处 fmt diff 全部
  预存于 examples 等，与本改动无关）；无 stray debug print（诊断输出全部
  收敛到 AUTO_ADE_TRACE env 门控观测面）。
- Spec impact：supersedes_spec_components=[] new_spec_components=[]
  touched_goals=[]（缺陷修复，无谱系面变更；SD-01/K 条款零弱化）。
- 复审补强（R-1 复审发现，已落实为第二提交 b88a2e3a2）：栅栏轮询初版在
  current-thread tokio runtime 上同步阻塞 accept/IO 线程，放大既有
  ECONNRESET 偶发家族（jade 旧 binary 亦两见=非本次引入）——工具执行整体
  下沉 spawn_blocking 后复验 N0-N3+O 全绿。
- 结论：合入。N 臂解锁判定权在 jade 复验（其契约 §1 重绑规则）；供给侧
  证据链完整。

## 待澄清事项

- N4/P2 消费方侧残余（D-5）——jade 复验时对齐；非本计划缺陷面。
- ECONNRESET 环境相关性（D-4）——观测面已常驻，再现时按 trace 排查。
