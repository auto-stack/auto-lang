# PLAN-814 独立复审 R1：needs_fix（就绪核查）

2026-10-20。用户请求审核已完成工作；实际交接仍明确承认未完成。本轮完成就绪核查及可直接核实的边界检查，不将不具备送审条件的 WIP 当成完整验收对象。

## 基线与独立性

- stage: review；review_scope: final（就绪核查提前返回，未授 final/phase pass）；reviewer_identity: 本聊天 Codex，上下文独立于现有实施记录，无另建 agent/checkout。
- plan_id: PLAN-814；plan_revision: 2；计划取自主检出当前未提交簿记版本，原文保存在 [plan-before-review.md](plan-before-review.md)。
- worktree_path: `D:/autostack/.wt/lang-814/auto-lang`；branch: `codex/plan-814-dev`。
- reviewed_commit: `03d983ae9a654d06963b2ad56a04a3adb3bf8e6e`；Git tree: `ebade961435379e85f5622f1b8bb314ecfcb0f1c`；base_commit: `203e2ce91302ff1ebd48cbd0a8365d76abb89009`。
- 主检出输入 HEAD: `0f390ecc4602592f82b74020599ddbca15fb6945`；未将其他计划的主检出 WIP 算入814交付。
- dependency_revisions: 组内 `auto-down` 为 `c39ac7f9dacdc769fc4a5ad3b98ec896240b28e8`；本轮未构建，不声称验证其运行闭包。
- `git worktree list --porcelain` 的路径/分支/SHA匹配上述登记。只读权限扩展后 `git -C <tree> status --short` 输出为空；`diff --quiet` 与 `diff --cached --quiet` 均退出0，无未提交实施改动。普通沙箱下工作树status报权限错误，未把错误当clean。
- 合同、基线清单、阶段合同及六个既有SD目标/目标账本的SHA256见 [input-hashes.txt](input-hashes.txt)。最终重查HEAD、依赖和洁净状态均未漂移，原始核查摘录见 [readiness-evidence.txt](readiness-evidence.txt)。

## Findings

### F814-R1-01 / P1 / OPEN：实施明示未完成且缺工作阶段必需交付

类型：原合同未完成/送审就绪缺口；任务T-02、T-03、T-13及最终T-04..09；AC-01..05、AC-08。

证据：计划仅T-01勾选，`current_step=1/15`；§9明确记载“树不可编译，明确标注不得送审/合入”。HEAD提交全文也写明 `TREE DOES NOT COMPILE` / `Do NOT merge; not reviewable`，并列出未迁根项。独立源读确认SDK `src/lib.rs:1855` 调用 `extract_autovm_result`，定义仍在上层 `crates/auto-lang/src/lib.rs:1875`；SDK `read_auto_lib` 仍被不存在的 `ui` feature门控（SDK lib.rs:1002，Cargo.toml没有该feature）。这不是本轮重新执行编译得到的诊断，编译失败结论来自交接自述并有源级缺项佐证。

`crates/auto-ui-compiler/src/lib.rs:1` 只有7行文档注释，runtime对应文件只有9行文档注释；每包只有manifest与该文件。四生成器/aura/实际runtime尚未迁入，不能满足M814-A的真实生成出口或B完整运行承诺。`crates/auto-vm/Cargo.toml:9` 仍依赖旧 `auto-lang`，未交付具名SDK消费者；独立SDK/UI消费者目录与 `m814-a-candidate.md`/最终handoff均未存在于本轮检查位置。SDK build.rs:32/125还扫描兄弟旧包的ui_gen源码，独立无UI checkout资源闭包未成立。

纠正：原树继续work，先修完T-02根项和消费者/资源闭包，完成T-03、T-13的当期正反控、真实产物消费与候选Spec子集并提交。仅申请M814-A审查时须明确phase与冻结SHA；若申请整Plan最终审查则还需原B任务完成。现有T-01进度保留，不将尚未完成的条目重复勾选或通过记债降格验收。

### F814-R1-02 / P1 / OPEN：SDK仍直接携带UI文法实现

类型：原AC-01/AC-08禁止边未清除；任务T-02（关联T-01可行性机制的覆盖边界）。

证据：`crates/auto-lang-core/Cargo.toml:31` 无条件normal依赖 `auto-ui-dialect`；`src/parser.rs:13307..13365` 的声明/节点包装直接调用 `auto_ui_dialect::grammar::*`；`src/dialect/ui.rs:51..84` 也直接调用同一实现。因此即使关default-features，SDK的normal源码闭包仍含UI文法实现。这是r2 §1/§5.1明确禁止的实现依赖，不是允许暂留的轻UI兼容AST/TokenKind。无需依赖某个运行输入就可核实该manifest边。

纠正：在原合同范围内实现上层显式方言装配及旧API兼容，解除SDK到UI文法实现的正常依赖，给出无UI源码的独立SDK消费及旧默认UI正控。若旧固有方法/字段兼容与该边界无法同时满足，按T-01要求提交具体needs_replan，不用改写“纯SDK”定义消除finding。

## 验收与任务覆盖

| AC | 本轮结果 | 依据/边界 |
|---|---|---|
| AC-01 | fail | F814-R1-01/02；真实SDK消费未交付，normal闭包仍含UI文法实现 |
| AC-02 | partial | 轻模块迁移/再导出已有历史记录；当前不可编译WIP不能证明整链旧API、显式组合与单状态；V-02未重放 |
| AC-03 | fail | 新compiler/runtime仍空实现；四生成器/真实产物消费未完成 |
| AC-04 | partial | 迁移已开始；SDK a2r_std.rs:1231/1237仍有cfg(ui)的旧frame直连，provider/寿命/退出矩阵未完成，不将暂时cfg掉的分支视为清偿 |
| AC-05 | fail | 测试选择/发行/无UI checkout及消费者尚无当前完整交接；t/tf已扩包不等于tv/tt/tb/tu等实际选择面完成 |
| AC-06 | partial | 本轮独立就绪核查完成；完整代码/格式/新增warning/debug/遗漏与SD验收尚未进入，不能勾T-10 |
| AC-07 | pending | merge-owned，未运行/未合入/未沉淀/未归档清理 |
| AC-08 | fail | M814-A同样缺真实SDK/compiler/consumer和V-06证据 |
| AC-09 | pending | T-13尚未完成，无阶段冻结候选；未执行T-14阶段pass |
| AC-10 | pending | merge-owned阶段发布，未实际合入或出发布收据 |

历史T-01的probe与5303绿仅是早期切片自述；不复用为当前03d983ae9的通过证据。本轮未因未完成实施反复启动整套最终验收；未运行Cargo、docs_gen、UI实机、777/792、fmt或整档测试，也未修复产品代码。未检查的完整高风险API/状态/资源组合均pending，理由是工作阶段必需项明确未完成。

## Spec delta与后置门

现有SD-01..08提案已原文冻结在 [sd-frozen.md](sd-frozen.md)，SHA256 `9df295eedc0ac214ecc761aa1ce1d1b467814316ed89d9b011a4774d2c81846f`。既有目标六路径存在；三个new_spec_components是待新增目标，GOAL-003/007真实存在，frontmatter路径映射保留。提案目前描述目标，没有按当前A候选冻结可发布的规则正文/版本/API范围，不能批准作为已实现现状。F814-R1-02解决前SD-01的“纯SDK”声明不成立。未改canonical Specs/live ledger。

pending_closeout_gates: 独立代码/验收/Spec pass后，merge owner才在同一冻结输入上统一裸cargo t及实际触面tv/tu/tt（根据最终diff确定其余门，包括迁移测试基建的aavm触面）。docs_gen/tb仅实际触发时执行；tf仅主检出到期单实例。当前未授权以WIP进入合入门。

## 路由

outcome: needs_fix；Plan保持executing、current_step=1/15；T-01保留完成，其余checkbox本来未勾，无需伪造回退。next: 原树work完成上述必需项、定向验证并提交后，按A阶段或最终范围重新交接。未合入、归档或清理。
