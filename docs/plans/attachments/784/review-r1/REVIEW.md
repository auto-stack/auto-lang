# PLAN-784 final review R1

2026-10-15；结论 **needs_fix**。审查 revision 1；保留 T-01、T-04 已完成，重开 T-02、T-03、T-05，计划回到 executing，2/7。本轮不修实现、不合并、不移除工作树。

## 审面与独立性

- review_scope: final；reviewer_identity: 当前 Codex 会话（曾编写规划，未实施本次提交），依据实际源码、独立恢复和自选反例重新验证；未创建额外 reviewer/agent，也不冒称外部复审。
- worktree: `D:/autostack/.wt/lang-784/auto-lang`；branch: `plan-784-dev`。
- reviewed_commit: `3f084979dcabf65e363e19027df9dff0c4b6e0b9`；base_commit: `7d51845ffcca4c886698caa947cbaca93bf7fdca`。
- auto-ui: `f7d92eef273c7f5592cce9d1612fa28ebe12d45e`；auto-man: `51fafe82e8e0c4e66325ee14088eed8c3270e07e`。两原检出均保持 master；UI 原有 Cargo.lock 未提交改动保持。
- 复审前后全部 tracked 实现文件 SHA256、HEAD、clean 状态一致；两旧仓 refs/remote/status、保全源文件和实际恢复载荷 hash 一致，见 [results.json](results.json)。源规范/配置指纹、最终 HEAD/status、三树 guard、diff-check 见 [fingerprint.json](fingerprint.json)。
- [冻结 SD-01 原文](spec-delta-reviewed.md)，SHA256 见 fingerprint；目标为 `docs/specs/repository-boundaries/project.md`，主检出尚无此文件，本轮未发布规范或修改 ledger。new_spec_components 路径正确；touched_goals 为空有治理类解释；supersedes 为空合理。

## Findings

### F784-R1-01 · P1 / blocking · 重跑保全会覆写已冻结的未提交载荷

原需求缺口；影响 AC01/AC02，T-02/T-03/T-05。`scripts/legacy-preservation.py:193–208` 对既有 payload 只检查 bundle；随后 `:231–242` 对同一路径直接 copy/write，`:302` 覆写 manifest。bundle 的 reuse 并不保护 WIP 恢复点。

独立反例：临时仓 f.txt 产生未提交版本 A → preserve → 改成未提交版本 B → 使用相同 payload-dir 再 preserve。实际 exit 0（action=reuse），原 `.work` SHA256 从 `305b9ce03a8aa14b723a454ef5b8736832e741ca2c3c5d211486441f725f883d` 变为 `216f5a612c5b360a2165d60ff1c68083d7c778be03eb0fe4346f68bb1055cdfb`，旧 manifest 同时丢失。A 不在 Git bundle 内；重跑因此破坏此前承诺的恢复点。此反例仅在新建临时仓运行，真实保全载荷未改变。

修复：同快照幂等重跑应保持既有载荷；输入发生变化时，在任何覆盖前显式拒绝，或写独立、不可覆盖的新快照并明确交接。检查既有 manifest、WIP/untracked/ignored 与 bundle 的一致性，失败出口不能先破坏旧快照。保留 A→B 控制及原有同 tip 重跑、bundle 损坏拒绝控制，验收“旧快照始终可恢复”，不能只断言 exit 或 reuse。

证据：[dirty-first.log](dirty-first.log)、[dirty-second.log](dirty-second.log)、results 中 `preserve-must-retain-prior-snapshot`。可重放入口 [run.py](run.py)，每次创建新 UUID scratch，勿对真实旧仓执行 preserve。

### F784-R1-02 · P2 / blocking · verify 漏检已保留 ignored 文件的原文件变化

原需求缺口；影响 AC02/V02，T-03/T-05。`:400–403` 校对 tracked WIP 的源文件；`:404–409` 对 untracked/ignored 只校对备份 hash，没有校对原文件。ignored 内容变化不会改变 porcelain status。

独立反例：临时仓将 private.cfg 加入 ignore，preserve --ignored-keep private.cfg，建立 legacy 锚，随后只改原 private.cfg 内容。verify 实际 exit 0、`ok: true`，应检测原字节漂移并非零退出。本次真实 ignored 保留文件经 reviewer 自行逐个比较均匹配；问题是交付校验工具存在假阴性，不能用其通过记录证明 V02 全部成立。

修复：对所有明确保留的原文件统一检查存在性和 hash；内容变化/删除须报告并非零退出，备份本身仍保持。纳入 ignored 与 untracked 的不变正控、内容变更/删除反控，保留 tracked WIP 现有控制。

证据：[ignored-verify.log](ignored-verify.log)、results 中 `verify-original-ignored-drift`、run.py 的最小构造。

### F784-R1-03 · P3 / closeout · SD-01 三个证据链接不可解析

收尾缺口；影响 AC04/SD-01，T-05。新规范第 33、57、58 行使用 `../plans/...`，从 `docs/specs/repository-boundaries/` 解析到不存在的 `docs/specs/plans/...`；应指向 `../../plans/...`。Design 37 和模块边界门链接有效，不属于此 finding。

修复链接并执行全部本地链接存在性检查；新提交后重新冻结 SD。证据：[spec-links.json](spec-links.json)、冻结 SD 原文。

## 验收映射

| AC | 任务 | 结果 | 独立证据及边界 |
|---|---|---|---|
| AC01 | T-01/02/03/05 | partial | 真实两 bundle 分别独立 bare fetch，冻结 refs 8/26、旧 tags 2/22、HEAD tree 全等；UI 未提交 work 字节及全部保留文件 hash 匹配。重复保全破坏旧快照的反例失败，R1-01 未闭合 |
| AC02 | T-01/02/03/05 | partial | 原仓/实际 payload 前后不变；三组树 guard clean；原自带 drill 4/4、两真实 verify 通过。但独立 WIP 重跑保护、ignored 漂移反例失败，R1-01/02 未闭合 |
| AC03 | T-04 | pass | 迁出 owner/资源/生成物/测试/消费者与 781/783/785/786 重叠逐表核对；实现树 strict 及主检出使用交付 baseline strict 均 exit 0；V04 读取失败控制收据及777实现拒绝路径核对。没有以环境下依赖身份显示差异臆判 strict 失败 |
| AC04 | T-06 | fail | 本次已完成逐项审查并冻结 SD；上述 blocking 与链接项尚待 work 修复，不签发 pass |
| AC05 | T-07 | pending | 合入/规范沉淀/ledger/归档、准备树接管与执行树 guard 后清理均属 merge；本轮未做 |

原自带 drill、真实恢复和两个 strict 通过，不能覆盖两个独立反例失败。Category A：本轮没有运行 cargo t/tv/tf/docs_gen 或编译；无 Rust 整档门禁。`modularity-audit.py` 为只读盘点，非编译。

## 下一步

在原 plan-784-dev 树修复三项并提交；更新 T-02/T-03/T-05 收据与反例覆盖后再交 review。保留原真实恢复载荷，不以覆盖备份方式关闭 finding。复审需绑定新提交、重跑受影响控制，重新冻结 SD；通过后再由 merge 完成 T-07。此前两个真实旧仓的现存备份已验证可恢复，无需重新建仓或清空主分支。
