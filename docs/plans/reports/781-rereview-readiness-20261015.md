# PLAN-781 再次送审：入口核查 needs_fix

stage: review
review_scope: final-readiness（未进入完整最终验收）
reviewer_identity: 本次 Codex 审查上下文；非本轮实施者；未委派 agent。
plan_id: PLAN-781
plan_revision: 2（当前文件值，见下方合同版本说明）
outcome: needs_fix
worktree_path: D:/autostack/.wt/lang-781/auto-lang
branch: plan-781-dev
reviewed_commit: 3d7817a246fa9409bb001bc0dd0d8717940e8108
base_commit: 491e84afbdcbfa3e05755596f5fb8d5187ba2f3a（与当前 master 的 merge-base）
main_checkout_commit: 972dddf10d7e6bc213c217f4fdb911ea510d5f61
dependency_revisions: auto-reader@41c5dd3d2f7fc8631011376d42c82aa183edae49，D:/autostack/.wt/reader-781e/auto-reader，branch=plan-781e-meta-index。

## 裁决依据

用户明确请求再审，已核对主检出当前 Plan、既有实现树和真实提交。Plan 当前 `status: executing / current_step: 21 / total_steps: 26`，T-22、T-23 未勾选；阶段 E 最新 work 交接也明确 next 为 T-22/T-23。并非仅簿记落后：本轮提交未改 `crates/auto-lang/src/tests/plan781_budget_http_tests.rs`，仍是原先发请求即 drop socket、等 300ms、assert ping=200 的测试；没有 W-04/slowwrite/entered 握手实现。

按照 auto-plan-review 的 readiness 要求：存在已知未完成 work-owned 验收时直接退回 work，不再运行一次完整 final-review 周期。因此本次没有 cargo、原生窗口或浏览器重跑，没有把新测试总数冒充最终 pass；保留已经完成的任务与全部历史记录。

## 上轮 findings 的当前状态

| finding | 本次核查 | 结论边界 |
|---|---|---|
| F-781-R01 | 有 34bb33090 + ab65017c0 修复和 W-01 双端证据记录 | 保留 T-19 完成进展；尚未在最终冻结提交独立重放，不新增 pass |
| F-781-R02 | 有 e455c1d16 归属过滤/寻址修复和 W-02 双端证据记录 | 保留 T-20 完成进展；尚未重新独立验证整个不变量 |
| F-781-R03 | 3d7817a24 driver 改全章 oracle；抽查 D:/p781-t26/out1,out2 原始 JSON：TXT 1347 章及冷重启 1347 章、MD 1346 章，W-03 精确报告第 900 章 sha-mismatch | 明显超出旧抽样覆盖，保留 T-21/24/25/26 进展；Reader 新架构及新反例仍须最终独立复核 |
| F-781-R04 | 原缺口仍在；T-22 明文未实施，HTTP 测试相对 base 零差异 | **open，阻断 AC-17（及原 AC-12/14）** |

R03 两份报告 CLI SHA256 均为 `226954ac30fe4b43cd0ef1ff992ba750c44e9ab035978f03154201a2c9758046`。本次仅检查报告内容，未重新验证该二进制构建归属，不将其记录为本审查运行结果。

## 退回 work 的具体事项

1. **T-22 / AC-17 / F-781-R04**：在真实 LegacyHttp 长忙请求上，以 entered 握手确认执行已开始；完成 deadline、断连、服务关闭三路，断言取消后无新副作用、单终态、task/scope/许可归还。服务最终还能 ping 与杀掉整个进程均不能代替该要求。
2. **T-23 / AC-18**：补四 finding 的反例→不变量→修复→永久用例→原始结果关闭表，冻结所有仓提交、CLI/配置/fixture hash、受影响定向验收及规范候选。T-23 旧文字中的 tu/th 全档需按 2026-10-14 裁定留给 merge，不在 work 或再次送审时执行整档；required 定向验收仍由 work 完成。
3. **合同/Spec 版本收口**：阶段 D/E 新增多仓实现、AC-15..19 后，frontmatter 仍为最初 plan_revision=2；保留用户裁定与历史批准，按语义合同变更递增 revision，不能伪造既往审批。SD-08/09 仍是待冻结摘要，未完整表达新增 byte_len/a2r 与 Reader meta 索引的持久知识目标；T-23 应明确各仓 canonical Spec 增量、边界、目标版本及 AC 映射。§10 旧“781 不实施、001 承接”的历史路由需标明已被阶段 E 裁定覆盖，避免当前合同相互矛盾。

## 基线与健康核查

auto-lang 实现树在核查开始/结束均 clean、HEAD 未变；auto-reader tracked 文件 clean，但有未跟踪目录 `s5-probe-/`、`s5b-/`、`s5c-/`、`t07-probe-/`，本次未删除。下次冻结时应说明其证据/临时产物归属。主检出 783/784/KNOWN-DEBT 等在途改动未纳入本次审查。

spec_inputs（本次只读，SHA256）：

- docs/specs/auto-lang/ui/design/scroll-child-anchor.md = FDDA3E3F584985148FAF23B5EC816793E868A74D8C174B89C9FDF48E4736C711
- docs/specs/auto-lang/vm/design/handler-execution-budget.md = F7AEC9B7B9896BB9CEA7A3CEAE0BF96B10BA488FCC12DF5D64345D125CF20A6D
- 主检出 Plan 核查前 SHA256 = 5F2C6A033E8A0BF62DE671021FEAE4B4D1CBB6CDEE54698877182A3FE11629A1

acceptance_results: AC-17 fail（required oracle 缺失）；AC-18 partial（未收口）；其他 AC 未在本次重新授 pass。
pending_closeout_gates: 全部 merge-owned 门禁仍 pending；最终审查通过后按实际触面在新冻结提交执行（lang 日常/VM/UI codegen/trans/HTTP 与 Reader 自身规则分别核定）。本次未执行整档。
next: work，在同一 781 树完成 T-22/T-23 与合同收口，再提交 execution_done；不合入、不归档、不修实现。
