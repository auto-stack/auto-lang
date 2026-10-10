# PLAN-738 归档后补充复查 R11（2026-10-10）

- stage: review；plan_id: PLAN-738；plan_revision: 3。
- outcome: **needs_fix（归档后发现后续修复项）**。R9 两项必修已闭合，但 R10-02 的空 lock 观察经真实生成/消费反例升级为 P2。不回退归档状态、不重开原计划、不修改实现。
- reviewed_commit: `9ff34ae0d0b52959de85949eed279e18ce832d7e`（已落地）；主检出入场 `95d5d3297c49042b292cd0ac891b198f62ea0aa1`。
- base_commit: R10 reviewed `41b4d4be52723ec2462e5ab511d3ea8e588dd38e`；R9 修复前 `19973b0899f27267db15a39669c124579f41a505`。
- dependency_revisions: auto-down=`895f8d0f9355c9f5ec3ce8fca268bdb768395846`；本轮 Cargo.lock 哈希见 baseline（根 lock 未入库，不能假定与 R10 相同）。
- 范围/独立性：本上下文曾执行 R5/R9 复查，未实施 R9 修复；本轮为归档后定向复核，不冒称全新上下文全 AC 独立复审。隔离 detached 验证 worktree，真实生成器/复用门探针，临时测试结束恢复原文件逐字节一致。

## 已确证闭合的修复

`41b4d4be5`、落地 `9ff34ae0d`、主检出 `95d5d3297` 的 crates 树完全相同（Git tree `7dcf8ea8e832307695b44423e66fe3a57005d3d0`）；stdlib、Cargo.toml、.cargo、.config 亦相同。落地提交是主检出的祖先；不是因原 worktree 消失就假定合入。

- **R9-01 闭合**：真实生成流程中，Cargo.lock 为目录引起非 NotFound 读错误，未绑定/已绑定收据均被复用门拒绝；生成器返回 `workspace Cargo.lock unreadable`；恢复原可读内容后重新通过。未绑定读错误恢复为真正缺失也重新通过，门本身不修改收据。生成器重新生成时先重建成员目录，因此不要求错误后旧收据仍存在。
- **R9-02 闭合**：恢复临时探针后，对 lib.rs、renderer.rs、engine.rs、api_gen.rs、rust_ui.rs 执行 rustfmt 1.9.0-stable（edition 2021、skip_children=true）检查，exit 0；diff --check 通过。
- R10 正常服务全链 **1/1、42.29s** 是已有历史证据；其 reviewed 源码与落地源码逐字节一致。本轮没有重跑或另报服务通过，也不把未冻结一致的根 lock 当成已复验依赖。

## P738-R11-01（P2）：空 lock 的生产者/消费者身份不一致

锚点：`crates/auto-man/src/rust_ui.rs:4030`，`api_gen.rs:1076`。

生成器对所有成功读取的字节计算 FNV，包括零字节文件；复用门却用 `bytes.is_empty()` 将成功读取的空文件与 NotFound 哨兵合并。通过正式 `generate_api` 写收据，再直接调用生产 `backend_generation_is_fresh`，得到：

```text
read-error generator+unbound+bound+recovery PASS
empty-lock first_fresh=true first_receipt="absent"
           regenerated_receipt="811c9dc5" regenerated_fresh=false
```

即：生成时无 lock，随后创建空文件，门返回 fresh 但未绑定实际身份；在这个同样的空文件上重新生成成功，收据写 `811c9dc5`，业务与 lock 均未改变，门仍返回 stale。R10-02「空文件归 absent 可辩护」没有对照生成端，遗漏了可复现的身份不一致。此处不是权限读错回归，也不宣称非空正常 lock 路径失效。

修复要求：仅 **NotFound** 表示 absent，成功读取（包括空字节）保持统一的实际内容身份，其他错误不可核验。宜共用显式分类/身份逻辑，避免用空 Vec 同时表示读取成功和缺失。补正式生成/消费测试：absent→空文件的一次绑定、空文件生成后的新鲜度、后续非空漂移和删除拒绝，并保留本轮已通过的读错误/恢复矩阵。不得把空文件不一致当作 D3b 豁免。

反例：`738-review-r11-probe.py`；测试段日志：`738-review-r11-evidence.txt`。最终探针真实选择 1 项，**0 passed / 1 failed、1.40s、cargo exit 101**，失败断言为两端 lock 身份应相同（`absent` vs `811c9dc5`）；不是测试环境错误。初次探针对生成器保留旧收据的错误假设已纠正，不作为发现依据。

## 验收与收尾边界

| 验收 | 本轮结论 |
|---|---|
| AC-01..05 | 未新增全量复验；继承 R8/R10 历史记录，源码一致性已核对 |
| AC-06 | 空 lock 内容身份不一致，后续修复必要 |
| AC-07 | 正式生成收据与实际消费门的空 lock 对拍失败 |
| AC-08 | 两项 R9 健康修复闭合；完整 HTTP 验收仍有待办 |
| SD-05 | 一次绑定/内容身份规则保留，修实现；不降低规范 |
| SD-01..04/06..07 | 本轮未提出规范变更 |

**HTTP 全档仍未完成**：归档收据明确 R9 串行 th 为 45/102，57 项未跑；R10 将它交给 regress。主检出 `.last-batch-regression.json` 仍是 2026-10-08、`covered_commit=3c2f3c347`，legs 仅 tf/tt/tb，不能证明此次落地或未跑 HTTP 项已经验收。这是已登记的验证尾项，不是新发现的 HTTP 代码回归；timed-frame 先前单项绿不能替代完整 th。后续须完成串行 th、逐名归因并写新收据。

本轮在确定性反例成立后不重复全档 t/tv/tt/th，不跑 tf/taa/tu/docs_gen，不外推全库通过。现有一次绑定状态机 scoped 单测另行复验，结果见 baseline。

next：原 PLAN-738 保持 **archived / delivered**；按仓库流程另立后续修复合同及 worktree，修复 P738-R11-01 并复验正式服务链；完成已经移交的完整 HTTP 分诊及到期批量回归。补充记录与债务登记不代表批准延期。
