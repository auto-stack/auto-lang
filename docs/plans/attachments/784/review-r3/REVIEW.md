# PLAN-784 final review R3

2026-10-15；**final pass**。R1 三项与 R2 一项 findings 全部 closed，无新增阻断项。T-01..T-06 完成，计划进入 **reviewed，6/7**；T-07 由 merge 收尾。本轮没有修实现、合并、发布规范或移除工作树。

## 审面与独立性

- plan_id: PLAN-784；plan_revision: 1；review_scope: final。
- reviewer_identity: 当前 Codex 会话，规划及前两轮复审作者，非本次实现提交实施者；独立读取代码、重放历史反例并选择补充反控。没有额外 agent 或外部 reviewer，不虚构身份。
- worktree_path: `D:/autostack/.wt/lang-784/auto-lang`；branch: `plan-784-dev`。
- reviewed_commit: `d2f12ca2e37cab568041f2b548f6dad29e516dc4`；base_commit: `7d51845ffcca4c886698caa947cbaca93bf7fdca`；R2 修复 diff 起点 `1994280805a524659ca3119ab9845e453a2cbc4a`。
- dependency_revisions: auto-ui `f7d92eef273c7f5592cce9d1612fa28ebe12d45e`，auto-man `51fafe82e8e0c4e66325ee14088eed8c3270e07e`。原检出仍在 master，UI 原有 Cargo.lock WIP 保持；准备树保持隔离。
- 主检出已前进到 `972dddf10d7e6bc213c217f4fdb911ea510d5f61`，相对本计划 base 仅提交 PLAN-781 簿记；本轮保留其他计划 WIP。没有把主检出 HEAD 前进误写为实现代码漂移。
- 前后 HEAD/clean、实现 tracked 全文件 SHA256、两原仓 refs/status/remotes/源文件与全部实际 payload 字节不变（含 v1、v2 旧代和本轮新代）。[results.json](results.json) 与最终 [fingerprint.json](fingerprint.json) 绑定证据。

## Readiness 与根因核验

已核对 work 在第三次送审前提供的两连败诊断和覆盖调整：旧模型将 refs 当作 bundle 元信息而未纳入状态；修复把完整 refs→objectID 放进状态身份，并将 bundle 实际 refs 覆盖检查接入创建、复用和 verify；D8 增加 refs 新增/移动/删除的真实恢复矩阵。

实际源码与交接描述一致；没有通过改 AC 或延后范围来关闭 finding。新增 state_schema=2 的快照并存，旧恢复点保留，不改写旧代快照。

## Finding 闭合表

| finding | 结论 | 本轮独立证据 |
|---|---|---|
| F784-R1-01 重复保全覆盖旧 WIP | closed | A→B 生成不同快照；A 全目录路径/字节 hash 保持并读回 A；第三次 B 复用、载荷 hash 与文件集合保持；原 D5/D7 再跑通过 |
| F784-R1-02 原 ignored 漂移漏检 | closed | tracked WIP、untracked、ignored 各自内容变更/删除，共 6 个独立反控均 exit 4；原文件恢复后复绿，快照字节不变 |
| F784-R1-03 规范失效链接 | closed | 冻结 SD 全部 5 个本地链接存在；原三处 plans 链接保持修正 |
| F784-R2-01 refs 变化仍复用旧 bundle | closed | 原 R2 新标签反例现生成新快照，bare fetch 实际恢复到独立提交；补测 tag 移动、删除及 branch 新增，全部恢复 refs 与当前完整映射一致；状态回到初始值时正确复用初始快照，旧快照字节保持 |

额外验证实际 refs oracle：在新临时仓，用**有效但缺 ref 的旧 bundle**替换新 bundle，同时更新 manifest 中 bundle SHA256，使简单 hash 检查可以通过。verify 与 preserve 仍分别 **exit 4**，明确报告缺 `refs/tags/oracle-extra`，全部 payload 保持不变。这证明新增覆盖检查确实作用于 bundle 内容，没有只验证 manifest/checksum。

## 验收映射

| AC / 任务 | 结果 | 本轮证据 |
|---|---|---|
| AC01 / T01/02/03/05 | pass | auto-ui 新代 `2c2ec22a85dcea56` 独立 bare fetch 恢复 11 refs/3 tags；auto-man `e143d51c828956ec` 恢复 29 refs/23 tags；两仓 tree 与原 tip 一致。UI 未提交 work 字节复制到独立恢复副本并校验相等，全部保留原文件/载荷 hash 相等 |
| AC02 / T01/02/03/05 | pass | 原仓与全部实际 payload 前后不变；自带永久 drill 8/8；独立旧快照保留、源文件6反控、refs矩阵与缺ref反控通过；三棵树 wt-guard clean、实现工作区 clean，原分支与 remote 不变 |
| AC03 / T04 | pass | 777 strict 在实现树以及主检出使用交付基线各重新运行一次：均 pass、0 violations/2 grandfathered。迁出清单/owner/消费者及在途重叠沿用冻结登记；脚本、基线、配置、Design37 和相关 Specs 与 R1 的 hash 相同。V04 读取失败控制沿用未变实现和既有证据，不重复运行 |
| AC04 / T06 | pass | 独立验收、历史闭合、遗漏/延后/workaround检查完成；SD-01 重读、冻结并绑定 SHA256；无未授权删除/迁移，无未关闭合同内 finding |
| AC05 / T07 | pending（merge-owned） | 规范沉淀/ledger/归档、目标仓准备树明确接管、auto-lang 执行树 guard 后清理尚待 merge |

独立 harness 共 **19/19** 结果记录通过（其中 permanent-drill 内含 8/8 控制），详见 [run.py](run.py)、[补充控制](extra-controls.py) 与 results；没有将内部子断言重复计入 19。原始 CLI 输出逐项保留为同目录日志。

strict 入口：

```text
python scripts/modularity-audit.py --root D:/autostack/.wt/lang-784/auto-lang --mode strict --output <review-r3>/audit-worktree.json
python D:/autostack/.wt/lang-784/auto-lang/scripts/modularity-audit.py --root D:/autostack/auto-lang --baseline D:/autostack/.wt/lang-784/auto-lang/.autoos/module-boundaries-baseline.json --mode strict --output <review-r3>/audit-main.json
两者 exit 0：strict verdict=pass violations=0 grandfathered=2
```

## 规范增量与边界

已冻结 [SD-01 原文](spec-delta-reviewed.md)，目标 `docs/specs/repository-boundaries/project.md`，SHA256 见 fingerprint。新增现行仓身份、legacy 保全、完整 refs 参与快照身份、旧代保留与治理规则，符合实际实现；未冒称 UI/Man 已迁出。

元数据确认：new_spec_components 为该精确路径；supersedes_spec_components 空；touched_goals 空有治理准备类解释，无需关闭语言/UI能力目标。主检出尚未发布该规范，本轮未写 canonical Specs 或 live ledger。

Category A：无 Rust 修改，本轮未运行 cargo t/tv/tf/docs_gen 或编译旧项目；不要求 merge 为本计划补跑 Rust 整档。diff-check、三树 guard、实现与保护对象最终指纹均通过。

## 下一步与未交付项

可进入 `/auto-plan:merge`：在本冻结提交上核对证据新鲜度，完成 T07 规范/ledger/归档、旧仓准备树和恢复载荷的明确交接，再 guard-clean 后清理 auto-lang 执行树。不得把旧仓准备树与恢复载荷当成本计划临时垃圾删除。未授权 push、默认分支更名/切换、生产代码树替换。

**review pass 不等于已合并或已交付。** 784 merge 完成后才满足后续计划对“784 完成”的依赖；785 仍须满足 781 宿主热点释放条件，786 按既定独立入场约束开展。
