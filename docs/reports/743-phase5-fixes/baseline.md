# PLAN-743 Phase 5 修复基线（T-28）

> r5 合同：docs/plans/743-acc-bootstrap-hir-contract.md（plan_revision 5，T-28..33）。
> 外部复审：docs/reports/743-r4-quality-review-20261005/REVIEW.md（P743-R4-QA-01..04，
> 含冻结 SD-09 提案与 counterexamples.json）。

## 激活与基线身份

| 项 | 值 |
|---|---|
| 实施基点 | `39a4f91339ac9744578cf2064c9731090469e960`（含 r5 合同的最新 v0.6-dev） |
| worktree | D:/autostack/.wt/lang-743/auto-lang，branch plan-743-dev（重建；复审临时 detached 树已由复审方清理） |
| 主检出预检 | 0 WIP；r4 reviewed 基线 af3c8040e→落地 898530d57 链在案 |
| Category | A（PYTHONUTF8=1；不跑 cargo/docs_gen；不占号；741/742/ABI 树不动） |
| 计数 | activating 22/33；本 Phase 后目标 32/33（T-33 留 review/merge） |

## 复审反例 before 状态（worktree @ 39a4f9133 实测，scan_module 直调）

### R4-QA-01 参数遮蔽不完整（六 fixture）

```
mut_parameter.at         -> native-runtime read_line IO   （demo(mut IO Meter)——mut 名提取得空串）
method_parameter.at      -> native-runtime read_line IO   （Meter.demo(IO Other)——方法签名未提取）
bare_parameter.at        -> native-runtime print          （自由 fn demo(print Other)——bare 路径缺绑定检查）
method_bare_parameter.at -> native-runtime print          （方法体内 print 参数同理）
usual_parameter.at       -> unknown-receiver read_line IO （对照：普通参数正确）
conflict_free.at         -> native-runtime print/read_line（对照：无冲突宿主正确）
```
根因：①签名提取只走自由 fn declarations、忽略 methods；②mut 参数名取 toks[0].replace
("mut","") 得空串；③裸调用分类 BARE_NATIVES 先于 known_bindings。

### R4-QA-02 manifest 形状崩溃

合法 JSON 但 source_identity=null/[]、顶层 []/1 → AttributeError/TypeError traceback
（source_identity=None 实测：`'NoneType' object does not support item assignment`）。

### R4-QA-03 消费者索引语义边界

非法 conclusion/过期绑定的记录仍入 validated_by_id 被族消费（CLI 最终 exit1 靠全局
错误，不靠可信边界）；重复 ID 仅后者淘汰、先插入者未经完整语义复检。三例 CLI 都
正确拒绝——承诺的可信消费边界未兑现，非假绿。

### R4-QA-04 归档链接与断言缺口

归档计划 6 处 `../reports/` 链接自 archive/ 解析断裂（audit.json 行 156/172/183/723/
840/955）；final_assertions 未检查 Markdown 链接、任务计数不含 r3 表格行历史已修但
仍需覆盖 33 任务/归档态链接。

### 并行主线 fresh 状态（T-28/T-30 前置）

主线 741 P4 落地又改 auto-ac/project.md → 当前主检出严格门正确拒绝 5 条 AutoAC Spec
绑定过期（MD-203/205/301/302/304）。T-30 逐条审定重绑（不机械换 hash）。

## 修复目标映射

| 发现 | 任务 | 落点 |
|---|---|---|
| R4-QA-01 | T-30 | 签名提取覆盖方法+mut 名修正+bare/qualified 共享绑定优先 |
| R4-QA-03 | T-29 | validate_decisions 完整语义/唯一性后才入可信索引（重复整组作废） |
| R4-QA-02 | T-31 | manifest 顶层/source_identity 形状门（受控 ERROR 零 traceback） |
| R4-QA-04 | T-32 | SD-09 冻结核对 + final_assertions v2（33 任务/链接/计数） |
| 5 条 stale 重审 | T-30 | 对照 741 P4 diff 逐条审定后重绑 |
