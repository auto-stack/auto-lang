# PLAN-784 final review R2

2026-10-15；**needs_fix**。R1 三项 finding 已关闭；新增 **F784-R2-01（P2 / blocking）**。保留 T-01/T-04，重开 T-02/T-03/T-05，executing 2/7。本轮仅复审，不修代码、不合并、不清理树。

## 审查基线

- plan_id: PLAN-784；plan_revision: 1；review_scope: final。
- reviewer_identity: 当前 Codex 会话，规划/R1 复审作者，非本实现提交实施者；依据源码、独立恢复、自选反例重建结论。没有额外 agent 或独立外部 reviewer，不虚构身份。
- worktree_path: `D:/autostack/.wt/lang-784/auto-lang`；branch: `plan-784-dev`。
- reviewed_commit: `1994280805a524659ca3119ab9845e453a2cbc4a`；base_commit: `7d51845ffcca4c886698caa947cbaca93bf7fdca`；修复 diff 的前序提交为 `3f084979dcabf65e363e19027df9dff0c4b6e0b9`。
- dependency_revisions: auto-ui `f7d92eef273c7f5592cce9d1612fa28ebe12d45e`，auto-man `51fafe82e8e0c4e66325ee14088eed8c3270e07e`；两仓原检出仍在 master，准备树仍在 codex/v06-extraction，未做远端操作。
- 实现 tracked 全文件 hash、HEAD/clean、两原仓 refs/status/remotes/源文件和全部实际 payload hash 在复审前后不变；含旧 v1 与新 v2 载荷。证据：[results.json](results.json)、最终 [fingerprint.json](fingerprint.json)。
- 本次重新冻结 [SD-01](spec-delta-reviewed.md)，目标 `docs/specs/repository-boundaries/project.md`，SHA256 见 fingerprint。主检出尚未发布该规范；本轮没有修改 canonical Specs 或 ledger。

## 上轮 findings 闭合

| finding | 结果 | 独立证据 |
|---|---|---|
| F784-R1-01 重复保全覆写旧 WIP | closed | 原 A→B 控制按新布局复放：生成不同 snapshot，A 目录所有字节保持且读回 A；第三次 B 重跑复用，payload 文件集合及 hash 不变。新模型的 refs 完整性另记 R2-01，不混为旧 A→B 未修 |
| F784-R1-02 ignored 漂移漏检 | closed | tracked WIP、untracked、ignored 各自内容变更/删除，共 6 个独立反控均 exit 4；恢复原字节后复绿，备份 hash 始终不变 |
| F784-R1-03 三个规范失效链接 | closed | 冻结 SD 全部 5 个本地链接存在，三处 plans 链接已修正 |

代码检查确认同 ID 快照先验完整性再复用，新快照 staging 写入并自检后落位；本轮重跑永久 drill 7/7，通过损坏 bundle 拒绝且不覆写等既有控制。不能用该 7/7 代替下面的新状态反例。

## 新 finding：F784-R2-01 · P2 / blocking

**快照身份遗漏 refs，仅新增标签时仍成功复用旧 bundle，无法恢复新标签。**

类型：R1 快照模型修复后的遗漏；对应既有 AC01/AC02、V01/V03，影响 T-02/T-03/T-05。不是要求新增备份类型，原合同已要求全 refs bundle 和标签可恢复。

源码：`scripts/legacy-preservation.py:187–216` 的 compute_state 仅包含 HEAD/status/WIP/untracked/ignored；`:325–339` 在 state 相同后校验旧 bundle 和原 manifest 并直接 reuse，没有比较当前 refs。`:360` 只在新快照创建时冻结 refs。因此 HEAD 与工作字节相同而 refs 发生变化时，不会更新快照。

独立反例（仅全新临时仓）：

1. 干净 master 首次 preserve，快照 `f475982e5588e9c9`。
2. 用当前 tree 建立独立历史提交 `a19a072663ada83222d0fc05077429198a84897f`，建立标签 `archived-extra` 指向它；HEAD 和工作区保持不变。
3. 同 payload 再 preserve，实际 **exit 0 / action=reuse / snapshot 仍为 f475982e5588e9c9**。
4. 在独立 bare 仓 fetch 本次返回快照的全部 bundle refs；`refs/tags/archived-extra` 不存在，无法按本次成功收据恢复该标签。旧快照 hash 保持，说明并非旧快照被覆盖，而是新 refs 完全未被保全。

证据：[refs-before.log](refs-before.log)、[refs-after.log](refs-after.log)、results 中 `refs-only-new-tag-retained`（唯一失败行），完整可复放入口 [run.py](run.py)。输入生成及 bundle 实际恢复均独立于实现自带 drill；输出不是模拟失败。

修复方向：将影响 bundle 内容的完整 `refs → object ID` 纳入状态身份或在复用前显式比较、拒绝漂移；若接受新状态，建立独立完整快照，保留既有快照。新快照的 bundle 实际可恢复 refs 应覆盖冻结清单，而不是仅凭旧 manifest/hash 自证。检查快照复用所依赖的状态与 manifest 字段是否一致，避免只为新增标签打补丁。

补齐同 HEAD 下 refs 新增、移动、删除的控制，至少包含一条原 bundle 不可达的提交与标签；断言实际 bundle 恢复结果、原快照字节不变及返回值。若采用拒绝策略，须非零明确报告、不声称当前 refs 已保全。现有真实两仓快照 refs 齐全，本 finding 不表示已经损坏或丢失真实旧仓数据。

## 验收与复用边界

| AC | 结果 | 证据 |
|---|---|---|
| AC01 / T01/02/03/05 | partial | 当前真实 UI 快照 efe2102a435580ae：独立恢复 11 refs/3 tags；Man 快照 1567f76ca3ecb7cb：29 refs/23 tags；两者 commit tree 等同。UI WIP 从 payload 复制读回 hash 一致，所有保留源/备份 hash 一致。refs-only 新反例失败 |
| AC02 / T01/02/03/05 | partial | 两原仓和全量 payload 前后不变，永久 drill7/7、独立 A→B、源文件6反控、三树 guard clean；但 refs 变化的重跑错误 reuse，R2-01 未关闭 |
| AC03 / T04 | pass（复用 R1 证据） | R1 两 strict、迁出清单及 V04 检查继续适用；基线、777脚本/配置、清单、Design37、project-model/auto-man 规范 SHA256 全部与 R1 相同，主检出仍为同一 base，fingerprint 逐项记录。修复未改此面，故不重复运行 |
| AC04 / T06 | fail | 逐项 review 与 SD 冻结已执行，R2-01 尚未闭合，不能签发 pass |
| AC05 / T07 | pending | 规范沉淀/ledger/归档、目标准备树交接、执行树 guard 后清理由 merge 完成 |

规范影响元数据仍合理：supersedes 空；new_spec_components 为精确 repository-boundaries 路径；touched_goals 空有治理类说明。新 SD 的快照现行行为规则需随 R2-01 修复重新核对并冻结；不通过修改规范弱化全 refs 保全要求。

Category A：未运行 cargo t/tv/tf/docs_gen，未编译旧项目。diff-check、三树 wt-guard 通过，最终实现 HEAD 未漂移、工作区 clean；没有未处理的新 Rust warning 面。

## 下一步

在同一 plan-784-dev 树修复 R2-01、补回归并提交，更新 T02/03/05 收据，再交 final review。两次 final 均出现缺口，下次 readiness 先给出根因与覆盖调整：明确快照身份、manifest、bundle 与当前仓状态的对应关系，覆盖 refs 与文件两类状态，不能只补本轮单一标签例。修复后保持 R1 三项闭合。

待 final pass 后再进入 T07；无 Rust 整档门禁，但文档/规范沉淀、准备树接管及安全清理仍 pending。当前保全收据中“784 完成”的后续入场表述不能替代 final pass 和 merge。
