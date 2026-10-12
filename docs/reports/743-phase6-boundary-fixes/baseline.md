# PLAN-743 Phase 6 修复基线（T-34）

> r6 合同：docs/plans/743-acc-bootstrap-hir-contract.md（plan_revision 6，T-34..38）。
> 外部复审：docs/reports/743-r5-quality-review-20261006/REVIEW.md（P743-R5-QA-01..03）。

## 激活与基线身份

| 项 | 值 |
|---|---|
| 实施基点 | `bf899da0b2805b61e5e815a265438ea45c2e9e6d`（含 r6 合同的最新 v0.6-dev） |
| worktree | D:/autostack/.wt/lang-743/auto-lang，branch plan-743-dev（重建；r5 分支已删） |
| 主检出预检 | 0 WIP；r5 reviewed 基线 ce61e5e6d→落地链至 9111c21e5 在案 |
| Category | A（PYTHONUTF8=1；不跑 cargo/docs_gen；不占号；741/742/ABI 树不动） |
| 计数 | 激活时 26/38；本 Phase 后目标 37/38（T-38 留 review/merge） |

## 复审反例 before 状态（worktree @ bf899da0b 实测）

### R5-QA-01 缺失绑定文件仍入消费者（CLI 三段式 + 消费者层）

完整层 + 被族引用决定追加同条绑定文件 → exists exit0 / 删除 exit1 / 恢复 exit0（CLI
三段式全对）。缺口在消费者层：`not f.is_file()` 分支只置全局 stale 不置 record_bad，
该记录仍留在 validated_by_id 被族读取（复审 consumer_reads=21 实证）。

### R5-QA-02 错类型同 ID 绕过全局唯一性

类型非法记录 `continue` 在 seen_ids 去重之前——bad-first/bad-last 两种顺序下，合法
同 ID 记录仍进 validated_by_id 被族消费（CLI 均 exit1 受控，非假绿；违背 AC-21/25
"重复整组作废"承诺）。仅可辨识的非空字符串 ID 参与全局去重（非法 ID 类型不 hash）。

### R5-QA-03 已知参数绑定让位于本地类型

```
probe.at  （type Meter + type Other + fn demo(Meter Other)）：
  type-qualified   len  receiver=Other   ← Other 是参数绑定，应 unknown
  type-construction Other               ← 同上，应 unknown
对照（无参数绑定）：Meter.len=type-qualified、Meter()=type-construction（合法保留）
```
根因：_post_classify qualified 分支 local_types 先于 known_bindings；bare 分支
enum/type 构造亦先于 known_bindings——SD-09 要求可观察绑定先于宿主/类型升级。

### 前置：主线 5 条 stale

主线 741 P5/P6 相继改动 auto-ac/project.md；T-36 对 MD-203/205/301/302/304 逐条
审定重绑（不机械换 hash），随后严格门绿。

## 修复目标映射

| 发现 | 任务 | 落点 |
|---|---|---|
| R5-QA-01/02 | T-35 | validate_decisions：全局 ID 去重前置（可辨识字符串 ID 全覆盖）+ 缺失绑定 record_bad + 整组作废语义测试 |
| R5-QA-03 | T-36 | _post_classify bare/qualified 均绑定先于本地类型/宿主升级 + 正反例 |
| R4-QA-04 续 | T-37 | final_assertions v3（38 任务/真实链接/计数）+ SD-10 冻结核对 |
