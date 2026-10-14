# PLAN-776 最终独立复审 R12（2026-10-14）

**outcome: needs_fix。F776-R11-01 部分闭合。** 五 setter 现在均使用真实参数实际调用，共享 RC2 矩阵与变异快照断言通过。上一轮明确要求的逐 setter RC1/RC2/错误 receiver 矩阵仍未补齐：另外四个 setter 仅测试 RC2；最后释放后虽调用 reap，却没有回收断言。生产修复保持 R11 的 pass，未发现新运行时缺陷。不再重跑完整最终验收周期，针对已知缺口返回 work。

## 基线

- stage: review；review_scope: final；plan_id: PLAN-776；plan_revision: 1。
- reviewer_identity/limitations: 当前 Codex 审查上下文，独立于此次测试修复实施，延续 R11 审查；未委派子 agent，不声称额外模型独立性。
- reviewed_commit: `d0fa9e5e2d0b50924698a01c591541f6e8744c8c`；repair_diff_base: `38ad38abaf7d0fc40adacbb775d3f0258aaa53a7`；base_commit: `5e21561d2c303a02319ff7a29e4607933e09a38f`。
- worktree_path: `D:/autostack/.wt/lang-776/auto-lang`；branch: `plan-776-dev`；实施树 clean。
- dependency_revisions: auto-down `c39ac7f9dacdc769fc4a5ad3b98ec896240b28e8`，clean。
- spec_inputs: [input-hashes.json](../attachments/776/review-r12/input-hashes.json)。七个 canonical Spec、decision 和拟议 SD 与 R11 相同；冻结 [reviewed-spec-delta.md](../attachments/776/review-r12/reviewed-spec-delta.md)，SHA256=`D5730778FA8EE21A005D6F934DBCD7C1BAFD49ED3B75EAA7B7548CC5E57D9CDB`。
- 修复 diff 仅一个测试文件 +137/-27；全部生产代码、依赖、测试配置、R11 独立探针输入未改。因此复用 R11 生产15格矩阵、原UAF重放、66专项和126守卫证据；本轮重新运行变化的测试并审查其实际调用和断言。

## F776-R11-01 剩余范围（P2，原 finding 延续）

旧未使用函数指针表已删除。header 正确压 key/value 池字符串，text/html 使用真实 body 并验证 Content-Type，bytes 使用 ListData<i32> [72,105]；四块都实际调用 shim。`settle` 验证 RC2→释放返回槽→RC1、reap 后快照字段一致。因此“未调用另外四个 setter”的原缺口已关闭。

| 提交内场景 | status | header/text/html/bytes |
|---|---|---|
| RC2共享、丢弃返回值、跨reap、实际mutation | 已覆盖 | 本轮已补齐 |
| RC1独占、释放返回值后回收 | 已覆盖 | 缺失 |
| 非凭据receiver错误返回、保留外部份额 | 已覆盖 | 缺失 |
| 最后全局份额释放后确认回收 | 有存活量断言 | 只调用release/reap，没有回收断言 |

位置：`plan776_http_return_tests.rs:2743` 的 RC1 和 `:2791` 的错误分支仍仅调用 status；`:2824` 的 settle 是单一 RC2 路径；`:2855` 最后 release/reap 后无断言。这不是要求所有语言组合穷举，而是 [R11报告](776-return-resource-review-r11.md) §3 已明确的五 setter 三场景验收。R11 的独立探针能证明生产行为，不能替代明确要求提交的正常回归测试。

修复只需扩展原测试：为 header/text/html/bytes 各补 RC1 和错误 receiver 两场景，使用已有合法参数，断言 owned 份额与存活/回收；在共享 settle 的最后释放后增加 heap不存在或存活量回基线断言。既有 status 重复链可保留，混合链不是本轮新增阻塞要求。保留已通过共享测试与全部生产修复，不必重写实现或弱化断言。

## 验收、规范与门禁

- AC01/03/04/05/07/08：沿用 R11 pass 及其代表覆盖/已冻结生成支持面边界；不把 742 真实业务体或 P776-D3/D4 另域债务记为完成。
- AC02/06：生产行为 pass；提交内回归矩阵 partial。AC09：fail，T07/T08 尚未完成既定矩阵。
- SD01..07 持续规则与 R11 一致；spec-impact 七个 supersedes 路径、新规范路径及 GOAL-003/016 保持；不发布 canonical 或 ledger。后续更正证据覆盖范围，不改变 plan_revision。
- actual：`cargo nextest run -p auto-lang --lib --features test-http-e2e plan776_r10_setter_single_release_shared -j1 --no-fail-fast` **1/1**，0.212s，0 timeout；[原始日志](../attachments/776/review-r12/setter-test.log)。通过证明已写分支有效，不证明缺失分支存在。
- 新测试文件 rustfmt --check（edition2021，skip_children=true）、`git diff 38ad38aba..HEAD --check` 通过。生产代码未改；未新增编译警告，仓库既有警告保留归因。
- 本轮未跑整档 t/tv/th/tt/tu/taa/tf。work 本轮提前运行的裸 t 不能替代 final pass 后 merge-owned 门禁；后置裸t、tv、HTTP与相关生成验证，以及主检出到期batch tf、Spec沉淀/归档/wt-guard仍 pending。

next: work，同一树补 F776-R11-01 剩余8格和最终回收断言、提交并定向复审。重开 T07/T08，current_step=6/8，status=executing；T01..06和全部已证子项保留。审查未修实施代码、合入、归档或清树。
