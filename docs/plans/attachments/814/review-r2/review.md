# PLAN-814 修复复审 R2：needs_fix

2026-10-20。stage: review；review_scope: final（就绪核查及受影响编译复验，未授final/phase pass）；reviewer_identity: 本聊天Codex，延续独立复审上下文，未参与实施、未新增agent或checkout。

## 冻结基线

- plan_id: PLAN-814；plan_revision: 2；worktree_path: `D:/autostack/.wt/lang-814/auto-lang`；branch: `codex/plan-814-dev`。
- reviewed_commit: `edb78b2efbdcadb1994d4065b9cb341b3d71497f`；tree: `9573d70b1e0eb68dfcd576615860410d37f18b60`；base_commit: `203e2ce91302ff1ebd48cbd0a8365d76abb89009`；上轮受审: `03d983ae9a654d06963b2ad56a04a3adb3bf8e6e`。
- 主检出输入HEAD: `abad97371f5443ea25b007fa34074b882062163f`；主检出最新计划原文见 [plan-before-review.md](plan-before-review.md)。实施采用soft-reset重组提交；本轮直接比较旧、新对象，不假设旧提交是新HEAD祖先。
- dependency_revisions: 组内auto-down `c39ac7f9dacdc769fc4a5ad3b98ec896240b28e8`。工作树初查clean，末次HEAD/依赖/洁净复核见 [baseline-evidence.txt](baseline-evidence.txt)。
- spec_inputs及合同/基线/阶段合同SHA256见 [input-hashes.txt](input-hashes.txt)。[sd-frozen.md](sd-frozen.md)与R1相同，SHA256 `9df295eedc0ac214ecc761aa1ce1d1b467814316ed89d9b011a4774d2c81846f`。未改语义合同，revision仍2。

## 已修与未修的边界

| Finding | 本轮状态 | 直接证据及下一步 |
|---|---|---|
| F814-R1-01 / P1 | **PARTIAL，仍OPEN** | 原不可编译子项已关闭：SDK现有`extract_autovm_result`定义（lib.rs:2220），read_auto_lib已去旧门；独立`cargo check --locked --offline -p auto-lang-core -p auto-lang`退出0、22.49s。其余交付仍缺：auto-vm Cargo.toml:9依赖旧auto-lang；新compiler/runtime文件仍仅7/9行文档注释；A独立consumer/候选/实际生成与发行证据未交付；SDK build.rs:32/125仍扫描兄弟UI生成器。先完成T-02/T-03/T-13，再申请具名A阶段复审；整项最终审查仍需B原任务。 |
| F814-R1-02 / P1 | **OPEN，未关闭** | SDK Cargo.toml:31无条件依赖auto-ui-dialect；dialect/ui.rs:51等仍调用完整grammar，parser.rs:883..885仍按Scenario::UI默认装配UiDialect。本轮工作记录也明确该项未处理。应按r2 §1/§5.1解除实现边并保持旧API正控，不能把“归lang-core”视为已获准降低无UI文法实现要求。 |

编译修复证据为本轮真实命令，原始输出 [review-r2-check.log.txt](../review-r2-check.log.txt)。本轮未把work所述SDK1711/日常5294作为独立复验，也未重跑整档。`cargo check`通过仅关闭编译子项，不证明消费者、UI运行或单状态合同。

a2r frame未完成的后果仍可直接从源码核实：SDK `a2r_std.rs:1229/1235` 使用不存在的ui feature门，:1241/:1246的反向门使两个公开读值走固定0；上层启用ui不会自动使依赖SDK具备该feature。当前frame provider仍未接入，不能当作旧UI帧观测已保持。本轮check直接报告这些`unexpected_cfgs`（原始日志:50起），以及其他迁移后的feature门告警；不以忽略告警/仅退出0授health pass。此项属于R1-01已有frame清边范围，不新增相同finding。其他依赖/UI告警尚未逐项归因，未把全部告警都算作本Plan新回归。

## 就绪、合同与验收

当前共享计划仍为executing1/15，仅T-01勾选；最新work记录明确保留auto-vm/frame和F02未处理，T-02未勾。因此本轮依据review技能的就绪规则，不启动另一轮完整最终验收。T-03/T-13未完成也不能授M814-A阶段pass。

contract-baseline §9/§10.3把UI文法当作允许进入SDK的轻实现，但r2计划§1/§5.1及V-01/V-06明确禁止**UI文法实现**；域表的导航分组不能覆盖已约定验收。当前计划无批准的新语义修订；该解释不作为关闭F02的依据。可行性若无法同时满足旧API与纯SDK，应由work提出具体重规划，不默改AC或清除finding。

acceptance_results: AC-01/03/05/08 **fail**（上述未完成边界）；AC-02/04/06 **partial**（可编译性已恢复，其余API/状态/资源、健康/SD完整审查pending）；AC-07/09/10 **pending**（后续review/merge-owned操作）。未重开已闭的根项编译缺陷，T-01既有进度保留；T-02/T-03/T-13和其余原本未勾的任务保持未勾，current_step=1。

Spec delta原稿未变、仍未形成绑定A候选的可发布规则正文/API/资源范围，不授SD pass。原R1的输入freeze和未检查风险矩阵仍仅作范围记录；源码/依赖/配置有变化，本轮不泛化复用旧行为证据。完整API/全局与TLS/退出/四生成器实际消费/777/792/fresh-locked/格式/debug扫描仍pending，原因是work-owned必需交付明确未完成。没有以记债批准延期或缩面。

pending_closeout_gates: 将来独立代码/验收/Spec pass后，由merge owner在最终冻结输入上统一裸cargo t及实际触面tv/tu/tt等；docs_gen/tb/taa按最终触面和仓库时序裁定；tf仅主检出到期单实例。本轮未运行整档、未修实现、未合入/沉淀/归档或清理。

outcome: **needs_fix**；next: 原树work完成本轮已知剩余项及定向正反控，提交完整A候选或完整B交接后再审。第二次非pass已记录，下一次交接需明确根因与覆盖调整：UiDialect/旧Parser兼容如何同时满足纯SDK边界、frame provider如何保持旧读值、实际compiler消费者和发行证据如何覆盖；仅新测试总数或“SDK闭包纯净”的摘要不构成就绪。
