---
plan_id: PLAN-737
status: executing               # drafting → executing → execution_done → reviewed → archived
feature_name: ade-layout-writeback
author: [zhaopuming]
created_at: 2026-10-03
updated_at: 2026-10-03

# /auto-plan:review 结束时填写：
supersedes_spec_components: []
new_spec_components: []
touched_goals: []             # 引用 docs/specs/goals.md 的 GOAL-NNN

affects: [auto-lang/ui]       # autodown_editor / iced renderer（具体路径诊断后定）
current_step: 1
total_steps: 6
---

# [PLAN-737] ade-layout-writeback

## 变更摘要

跨仓缺陷修复单（消费方 jade-edit PLAN-037 T-02 解锁门）：PLAN-732 已交付的
原生 wikilink 供给在真实 merged 生产窗口不可激活——**DocLayout 写回与可见
文档脱节**。本计划：①根因诊断（真实应用节拍插桩实测）；②修复真实 merged
窗内 `DocLayout.blocks/links` 与可见渲染文档一致性；③ECONNRESET 崩溃路径
定谳（顺手项）；④新增真实节拍回归测试锁定；⑤merge 后重建 master debug
binary 并在回执注明新指纹（jade 侧以 AUTO_EXE 默认路径复跑探针做 T-02
复验，N 臂全 PASS = 解锁项⑤）。

## 目标

- G1：根因定谳——真实 merged 生产窗口中 `AutodownEditorCore.layout`
  （DocLayout.blocks/links）为何与 core.blocks（可见渲染文档）脱节。
- G2：修复——真实 merged 窗内 DocLayout.blocks/links 必须与可见渲染文档
  一致（块 rect 逐块、链接区间随 render_frame 写回）；不弱化 SD-01
  （docs/specs/auto-lang/ui/design/autodown-wikilink.md）与已冻结 K 条款。
- G3：13/13 既有 plan732 测试零回退；新增真实节拍回归测试锁定布局-内容
  一致性。
- G4：ECONNRESET 定谳：密集 editor_drag 期间应用偶发退出是否 panic、
  是否同根因；同根因一并修，否则另案登记。
- G5：交付回执（docs/reports/）注明新 binary 指纹；jade T-02 复验路径
  （其仓只读消费，本计划不动 jade-edit 仓）。

## 架构方案

供给链（已实勘，master@7b93bfcc5）：
- lower 期 `autodown_editor_sync`（renderer.rs:26303 generic / :27572 VM
  轨）→ `sync_external` 差分回写 → `rebuild_with` 重建 core.blocks。
- 布局写回唯一写点 = `render_frame`（core.rs:1810，DocLayout 落
  :2632）；调用点 = widget.layout() / widget.draw()（widget.rs:221/:347）。
- MCP 读数 = core 直读（mcp_server.rs tool_editor_state：text=
  emit_document、focus=focused_block）；点击通道 = `__mcp_drag_ade`
  （renderer.rs:17682）press/dragged/released 逐点直调
  `core.handle_input`，点击门读 `self.layout`（core.rs:1529/:1038）。
- 真实应用节拍关键面：脏帧快门（renderer.rs:23517 `!dirty` 返回缓存
  Element）下 lowering（→sync）只在 view_dirty 帧跑；widget.draw() 每次
  重绘都调 render_frame 写回布局——「内容 rebuild 之后 render_frame 是否
  再跑、以何 viewport_w 跑」为诊断主疑点（缺陷单方向 a/b；方向 c
  hit_test_strict 在退化布局下的放大行为一并核）。

修复原则（约束）：不改 SD-01 契约面（语义/命中模型/点击门/载荷/origin/
验证面口径），只修「布局写回与可见文档同步」的时序/一致性；不动
jade-edit 仓。

## 需求分析与背景调查

- 消费方契约：jade-edit PLAN-037 T-02 解锁门五项回执，①..④ 已 pass
  （本仓 master d5215b477，13/13 独立复跑全绿）；⑤ live 核验失败。
- 交付参照：docs/reports/p732-wikilink-supply-receipt.md（⑤注意点：
  "autodown_editor_sync 首帧 no-op——次帧重降层补内容"；六环语料测试
  显式两段降层模拟同节律——真实应用节拍未被既有测试覆盖）。
- 缺陷现象（jade 侧 18c278a + 本侧 2026-10-03 探针复跑实测，binary
  v0.4.2-2650-gd5215b477）：
  1. 真实 jade 应用 `auto run -r vm`（JADE_WORKSPACE fixture + MCP）打开
     9 块文档，editor_drag 单点（SD-01 §6 认可的语义完整点击通道）密集
     网格全点击零激活（nav_seq 恒 0），应用存活。
  2. press-only 纵扫 focus 带退化：jade 侧编辑前全 0、key 一字符后粗带
     {3: y2..44, 4: y47..359}；本侧复现 block0 首中 y=51——均不反映真实
     9 块文档；text 读数正确（core 内容对、layout 错）。
  3. 上游侧同代码六环语料（UserInterface 生产事件泵 + 显式两段降层）
     全绿——缺口特属真实应用渲染/同步路径。
  4. 附加：密集 editor_drag 扫描期间应用进程偶发退出（jade 侧 MCP
     ECONNRESET 两次，非每现）。

## 详细设计

（T-01 插桩诊断后回填：根因结论 + 修复设计；当前为假设清单）
- H-1 布局冻陈：内容 rebuild（sync 真值）后 render_frame 未再执行（快门
  /重绘停摆）或以异宽 viewport 执行——布局滞留旧形态。
- H-2 快门缓存 Element 复用帧（!dirty）不跑 lowering，sync 拿陈旧
  value；与 ui_epoch/memo 命中交互放大。
- H-3 hit_test_strict 严格口径在退化布局下的单带放大（非根因、伴生面）。

## 测试设计

- 回归锁定（新增）：贴近真实节拍的最小测试——生产 view 同步挂载多块
  文档（首帧 no-op + 次帧重降层 + 缓存/快门节拍），断言
  `AutodownEditorCore::link_regions()` 非空且 rect 对齐文本行、
  editor_drag 语义完整点击经 core 门激活。当前预期该测试在真实形态下
  先红（13/13 旧测仍绿——未覆盖此节拍/渲染路径）。
- 零回退：`cargo nextest run -p auto-lang --lib --features autodown
  plan732` 13/13；`cargo t autodown` scoped 档与 master 基线逐名全等。
- 门禁（fix-test-tiering 分级）：改动属 Category B 局部 Rust——
  `cargo check -p auto-lang` + `cargo t autodown`；复审裸 `cargo t`；
  不触 ui_gen 则不跑 tu。

## 验收标准

- AC-01：根因定谳记录在案（本计划「详细设计」节 + 插桩证据），含
  ECONNRESET 是/否 panic、是否同根因。
- AC-02：真实 merged 窗（探针 N 臂形态）DocLayout.blocks/links 与可见
  文档一致：block 0 链接 `[[Goals]]` 在 editor_drag 单点完整点击下激活
  （nav_seq +1）；多块文档块 rect 逐块对齐。
- AC-03：13/13 零回退：plan732 全组 + scoped autodown 档与 master 基线
  逐名全等；复审裸 cargo t 零新红。
- AC-04：新增真实节拍回归测试绿（修复前先红复现）。
- AC-05：merge 后 master 重建 debug binary（cargo build -p auto），交付
  回执注明版本串+SHA256 新指纹。

## 执行步骤

（原子任务：精确文件路径 + 确切操作 + 验证命令；每步完成后追加
[✅ 已完成] 一行证据）

- T-01 工作树 + 插桩诊断：lang-737 工作树内加 env 门控观测
  （sync_external/rebuild_with/render_frame/widget.layout/draw 节拍），
  重编译 binary 复跑 jade 探针 N 臂，捕获真实节拍 → 根因定谳
  （含 ECONNRESET panic 排查：__mcp_drag_ade 链 press/dragged/released
  直调在空 links/异常 block rects 下的越界/unwrap 面）。
- T-02 修复设计定稿 + 计划回填（本文件详细设计节）。
- T-03 修复实现（预计 core.rs / widget.rs / renderer lowering 内；
  插桩临时观测移除或转常驻 debug 面，遵循零 warning）。
- T-04 真实节拍回归测试（先红后绿锁定）。
- T-05 门禁：cargo check -p auto-lang；cargo t autodown；复审裸
  cargo t（主检出单实例）。ui_gen 未触则 tu 不跑。
- T-06 复审（/auto-plan:review）→ merge → master 重建 binary → 交付
  回执（指纹）→ worktree 清理（wt-guard 先行）。

## 复审记录

（/auto-plan:review 后回填）

## 待澄清事项

- ECONNRESET 两次在 jade 侧偶现、本侧暂未复现——T-01 定谳。
