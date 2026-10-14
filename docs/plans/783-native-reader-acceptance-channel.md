---
plan_id: PLAN-783
status: drafting
feature_name: native-reader-acceptance-channel
title: "原生 Reader 窗口的可重放行为验收通道"
author: [Codex]
created_at: 2026-10-14T00:00:00Z
updated_at: 2026-10-14T00:00:00Z
plan_revision: 1
current_step: 0
total_steps: 8
supersedes_spec_components:
  - docs/specs/auto-lang/ui/overview.md
new_spec_components:
  - docs/specs/auto-lang/ui/design/native-acceptance-channel.md
touched_goals: [GOAL-007, GOAL-014, GOAL-016]
affects: [auto-lang/ui]
depends_on: []
---

# [PLAN-783] 原生 Reader 窗口的可重放行为验收通道

## 0. 变更摘要

复用已有AutoUI HTTP MCP，为真实Reader独立VM窗口补齐split VM+HTTP编排、真实段落动作、当前帧布局可见性oracle与冷启动证据。承接001 T-26的原生验收前置，亦可为781提供原生几何oracle。框架通道成功与Reader混排恢复成功是分别验证的事实。

## 1. 目标

- 匹配CLI运行`auto run -r vm --server=vm`的真实Reader窗口及后端，隔离数据并验证实例身份。
- 驱动当前渲染树上的段落点击/字号与章节动作，经真实产品handler和API持久化，重启同一隔离库观察恢复结果。
- 以新鲜目标/viewport/祖先clip交集证明可见，missing/stale/隐藏/无动作均不能PASS；原始状态、几何、截图、文件hash和失败退出可重放。
- 非目标：通用OS键鼠平台、模拟native UI、替产品scroll到目标、绕过后端保存、Reader业务修复、自动关闭001或把框架成功当混排通过。

## 2. 架构方案

优先扩展`.agents/skills/autoui-verifier/scripts/test_vm_mcp.py`的现有client/进程编排设施，复用`acceptance_channel.py`生命周期做法；新增专用Reader驱动为有界consumer，不重写MCP协议。真实`autoui_action`选择后渲染VTree节点并发事件，`autoui_state/vtree/select_rect/screenshot`取得后验。

若普通布局text段落的点击或测量尚未由MCP覆盖，只补同产品事件分派与gated layout观测的最小通路；准确与用户事件路径对齐，不直接调用fixture把progress写成预期。T-01冻结动作/几何/帧新鲜性接口；新观测字段不能泄漏到正常产品UI。

## 3. 技术栈

Rust ui/mcp_server.rs、mcp_types.rs、iced renderer与必要layout collector；Python现有原生验收设施；真实AutoUI .at窗口/HTTP后端与仓内可重放结果。可见性采用窗口逻辑坐标，截图辅助核查。

## 4. 需求分析与背景调查

授权：用户2026-10-14要求auto-lang立项；三项原目标保持。当前仅设计/文档，未启动窗口、未运行行为检查、未创建agent。Reader现有业务代码只读，后续测试用临时独立库，不修改真实知识库或共享用户进程。

基线master @c171c1a09、Reader @29b5decf878ec2cdca5c6a39575e872a4597121d，冻结hash见 [立项证据](reports/reader-prerequisites-planning-20261014.md)；设计见 [共同设计](../design/autoui/reader-runtime-prerequisites.md)。依据 [UI Spec](../specs/auto-lang/ui/overview.md)、[Select Anything](../specs/auto-lang/ui/design/select-anything.md)；后者的中心点选择是采集合同，本计划视口oracle使用正面积交集，不修改其选择规则。

源码已有`autoui_action`（press/type/scroll等）、`autoui_vtree`、`SharedState::layout_bounds`和截图。部分非容器bounds只在F12 debug采集；MCP目前不能从工具名直接推断Reader段落可点击且几何新鲜。test_vm_mcp.py默认裸`run -r vm`，Reader必须`--server=vm`。acceptance_channel.py主要桌面宿主，不能直接当Reader独立窗口证据。

PLAN-783不依赖781/782：通道使用均匀内容正对照与有意错误负对照，验证自己能够准确区分状态；781新增能力以其独立fixture/后续Reader接入使用该oracle。若现有Reader均匀恢复失败，记录consumer defect；不改成直接注入PASS，可使用框架fixture交付通道，但必须明确Reader尚未满足的门及阻断001。

## 5. 详细设计

### 身份、动作与证据合同

1. 每run创建reader001-review-*隔离目录，记录CLI path/hash/version、框架commit、Reader commit/源码hash、数据目录、MCP endpoint、前/后端pid与实际启动参数；启动就绪验证项目身份与空/预期隔离库，已有端口实例不能混用。
2. 所有动作发现自当前VTree，原始段落key/正文与节点匹配后经生产事件派发；节点可点击性、ack与随后存储/状态变化都需要证据。不调用fixture注入“保存成功/已恢复”终态。隔离种子只能建立前提。
3. 几何与实例/渲染代际绑定：action前后、重启前后记录freshness；未测量、帧未更新、节点失效、隐藏/零尺寸都不可判visible。测量pane真实viewport，目标rect须与pane及祖先有效clip有正面积交集。transform/scroll translation只转换一次，区分布局坐标与屏幕可见坐标。
4. 检查恢复阶段只能观察/等待，禁止`autoui_action scroll`、调用scroll helper或直接设置offset；可在建立测试保存前提时滚动，但两阶段明确隔离记录。
5. 真实冷启动=停止并reap本run拥有的进程，再用同库重新启动，保留新pid/实例identity；hot reload/只跑Init/截图看见窗口不能代替。收尾不得按进程名杀共享auto.exe；超时有控制失败及归属明确的残留记录。
6. required病例任何FAIL/SKIP/缺证据导致驱动非零；汇总读取真实执行病例，不以banner证明覆盖。保留raw MCP结果/动作trace/state/几何/截图与磁盘hash，报告原生backend而非headless。

### 规范增量

| delta_id | add/modify/retire | docs/specs/... target | before/after rule | rationale | acceptance IDs |
|---|---|---|---|---|---|
| SD-01 | add | docs/specs/auto-lang/ui/design/native-acceptance-channel.md | 无Reader独立窗口可重放合同 → split编排、实例/动作/新鲜裁剪几何/冷启动/失败收尾与覆盖边界 | 原生行为证据可信 | AC-01/02/03/04 |
| SD-02 | modify | docs/specs/auto-lang/ui/overview.md | 仅零散MCP/桌面设施 → 通道入口、生产默认观测开销/门控和真实consumer状态 | 消费者可以复验 | AC-04 |

如需扩展既有select-rect工具，只加互操作元数据/观测字段，不改变中心包含语义；新的read-only观测API及gating在T-01冻结并加入SD-01候选。不修改canonical Specs直至merge。

## 6. 测试设计

角色未指定分离agent/model；普通独立review为final，review暂停同树实现写入。下表所有V项owner_stage=work；T-02~05使拟病例可执行，本轮没有运行证据。

| Case | AC/不变量 | 真入口/输入 | 区分错误实现的oracle | 覆盖边界 |
|---|---|---|---|---|
| V-01 | AC-01/实例隔离 | Reader split VM+HTTP，A/B不同库和endpoint | pid/项目/源码/库身份一致；旧实例/错误endpoint明确拒绝，不能错用已开窗口 | 独立Windows native VM |
| V-02 | AC-02/真实保存 | 导入小型均匀书，当前VTree段落press | 实际handler/API成功且progress磁盘内容/ID/段key一致，失败ack不报保存 | 不用fixture终态注入 |
| V-03 | AC-02/冷恢复 | 同库真实关闭→重启，均匀书中段/字号设置 | 新pid、规范state保留，目标正文与pane裁剪交集>0；观察阶段没有补滚动 | 当前Reader正对照；混排通过由001验收 |
| V-04 | AC-03/假几何 | 框架fixture的目标完全在pane外/隐藏/零尺寸；错node/stale frame/无bounds | 必须FAIL/invalid，采集器layout rect非零仍不能冒visible | 负例显式构造并证明真实变异 |
| V-05 | AC-03/裁剪 | 目标部分可见、pane有嵌套包装/祖先clip；滚动后新帧 | 几何正确正面积交集，双重offset/只看屏幕外框错误被区分 | ordinary scroll-pane，不managed |
| V-06 | AC-04/失败传播 | 无动作、断后端、窗口启动失败、超时、带一个FAIL/SKIP的required结果 | 整体非零，raw失败与汇总对应，本run子进程reap，无共享进程被杀 | 驱动级真实失败，不mock全PASS |
| V-07 | AC-03/与781互操作 | 新anchor fixture或明确在pane外的mixed Reader | 通道按实测分类；若产品未接781则诚实FAIL而非通道不支持/修正位置 | 框架可见性与业务恢复分别归属 |

纯driver/文档更改按AGENTS Category A仅跑上述目标验证与Python语法检查，禁止cargo t/docs_gen。若T-03需要改Rust UI/MCP测量，worktree执行`cargo check -p auto-lang`+裸`cargo t`；碰VM/native补tv，UI生成补tu，其余按触面，不碰aavm不taa。真实原生窗口与MCP回放始终required，不以单测/headless替代。

拟新命令：`python scripts/tools/test_native_reader_acceptance.py --auto-bin <plan-cli> --app-dir <Reader工作位> --data-dir <reader001-review-目录> --out-dir <证据目录>`；新driver由T-02创建后冻结参数及命令，此处不是已经可执行的测试证据。

## 7. 验收标准

- [ ] AC-01: owner_stage=work；V-01真实split实例身份、独立数据和启动错误拒绝可观察，双实例不串线。
- [ ] AC-02: owner_stage=work；V-02/03通过真实保存/冷启动链路取得结果；如现有consumer失败，通道须以等价框架正对照证明可用且明确记录Reader实际失败，不把其目标授PASS。
- [ ] AC-03: owner_stage=work；V-04/05/07正负对照区分新鲜、失效、隐藏、剪裁与视口外目标；恢复后测试不补scroll，截图与几何来自同一窗口。
- [ ] AC-04: owner_stage=work；V-06失败非零/required缺证据不pass、owned进程收尾、raw证据和冻结CLI/driver命令完整；正常产品默认行为与观测门控不回归。
- [ ] AC-05: owner_stage=review；在固定commit/CLI/consumer基线上独立重放通道正负与启动/终止路径，审SD-01/02，真实Reader覆盖及未通过门分别记录。
- [ ] AC-06: owner_stage=merge；实际规范沉淀、归档/守卫清理及通道交付receipt，给781/001可用入口；不宣称001混排/长文已通过。

## 8. 执行步骤

拟树D:/autostack/.wt/lang-783/auto-lang、branch=plan-783-dev；main记录plan，worktree实现/验证。不另建review树。Reader仅只读consumer，在其既定app工作位用专用库运行；如该位正在写入，等其冻结实际commit而不擅自切分支/reset。

- [ ] T-01: owner_stage=work；依赖无；核对现有MCP/action/layout脚本、Reader事件树与split生命周期，新增docs/plans/reports/783-channel-decision.md（动作/帧identity/裁剪/门控/复用边界）；有界定位缺口，冻结新工具/参数形状。AC-01/02/03/04。
- [ ] T-02: owner_stage=work；依赖T-01；复用test_vm_mcp.py client与owned进程设施，新增scripts/tools/test_native_reader_acceptance.py；支持split、严格实例身份、隔离库、就绪条件和raw证据；跑V-01/06，不依赖固定sleep假就绪。AC-01/04。
- [ ] T-03: owner_stage=work；依赖T-01；仅当现有API不足，在ui/mcp_server.rs/mcp_types.rs/iced布局collector补最小gated观测与生产事件动作覆盖；加入真实帧/clip/失效单测与V-04/05，正常产品默认不打开测试入口。若无Rust改动，记录复用证据。AC-03/04。
- [ ] T-04: owner_stage=work；依赖T-02/03；创建test/fixtures/native-reader-acceptance/（新），跑真实Reader导入/段落press/保存/冷启动的V-02/03；保留磁盘与MCP证据；consumer失败另外列出，不能驱动写终态替代。AC-02。
- [ ] T-05: owner_stage=work；依赖T-04；完整V-01~07及required变异/失败传播、owned进程收尾验证；目标窗几何/截图人工核查，按触面跑相应门；V-07无781时用显式在pane外目标作负对照，不创建环依赖。AC-01/02/03/04。
- [ ] T-06: owner_stage=work；依赖T-05；冻结CLI/framework/Reader/driver hash、原始结果/病例总数与SD候选、具体用户命令；Reader成功/失败/未测门分开记录，交execution_done。AC-04。
- [ ] T-07: owner_stage=review；依赖T-06；独立final重放工具/产品正负、current/stale frame与失效节点，核实际窗口及SD-01/02；通道全部required通过才reviewed。AC-05。
- [ ] T-08: owner_stage=merge；依赖T-07；实际merge/规范/归档/守卫清理与到期批量门；交付receipt到781/001原生验收路径。AC-06。

## 9. 复审记录

2026-10-14：stage=new | plan_id=PLAN-783 | plan_revision=1 | outcome=pass（立项合同可评审，非通道或Reader实现PASS）| baseline/inputs=立项证据 | changed_tasks=T-01~08 | acceptance=AC-01~06 | next=用户确认执行范围后work，优先T-01核复用/最小缺口；drafting，0/8。

## 10. 待澄清事项

没有须用户提前选择的目标缺口。T-01负责普通text段事件、布局freshness与frame identity的具体可实现形式；不因旧select-rect能返回结构就假定可见性oracle已闭合。通道可独立完成，但Reader混排仍有F-12且等待781接入；001 T-26只在使用交付通道完成其实际行为验收后勾选。
