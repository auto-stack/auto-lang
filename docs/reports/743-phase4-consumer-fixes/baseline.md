# PLAN-743 Phase 4 修复基线（T-22）

> r4 合同：docs/plans/743-acc-bootstrap-hir-contract.md（plan_revision 4，T-22..27）。
> 外部复审：docs/reports/743-r3-quality-review-20261005/REVIEW.md（P743-R3-QA-01..04）。

## 激活与基线身份

| 项 | 值 |
|---|---|
| 实施基点 | `1d8fba15c7f8685e670f3bc8a398198895331c72`（含 r4 合同的最新 v0.6-dev） |
| worktree | D:/autostack/.wt/lang-743/auto-lang，branch plan-743-dev（重建；r3 分支已删） |
| 主检出预检 | 0 WIP；上轮 reviewed 基线 8e8f6f19b（r3 合入后独立复审基线） |
| Category | A（PYTHONUTF8=1；不跑 cargo/docs_gen；无外仓/链接；不占新号） |
| 并行边界 | 741/742/ABI 检出独立；.next-id=744 未占用 |

## 复审反例 before 状态（worktree @ 1d8fba15c 实测）

### R3-QA-01 消费者组合路径 traceback（完整层 + 族引用决定 evidence=17）

```
exit 1 + Traceback: TypeError: 'int' object is not iterable
```
decisions_by_id 从原始 decisions 建索引（仅查 id 为 str），被类型校验淘汰的记录
仍可被族适用性检查消费 → rd.get("evidence", []) 返回 17 → 迭代崩溃。

### R3-QA-02 hash/subject 旁路（完整层 + hash 变体）

族引用决定的 evidence 只指 lib.rs:1874（不含族路径），族路径仅出现在
bound_input_hashes → 严格 --check --require-decisions **exit 0**（旁路确认）。
subject-only 变体同理（代码 `covers = ... or rd.get("subject") == fp`）。

### R3-QA-03 导入/参数遮蔽

```
import_shadow.at   -> print=imported-symbol ✓；但 process/List/IO/File 四个
                      qualified 调用仍 native-runtime（use other: IO, List,
                      File, process 遮蔽未传入 qualified 判定）
parameter_shadow.at -> IO.read_line()=native-runtime（IO 是 Meter 参数，应 unknown）
```

### R3-QA-04 归档生命周期

归档态 T-21 无完成标记/r3 尾段"尚未实施"起草语残留（激活提交已补 T-21 勾选与
计数 21/27，本 Phase 标注历史段消歧）；ledger P743-3.file 指向活动路径（重开期间
可解析，最终归档后需改指 archive/ 路径）——最终 merge 对全部 P743-* 指针/计数/
active-archive/索引做逐项断言（断言清单随 verification.md 交付）。

## 修复目标映射

| 发现 | 任务 | 落点 |
|---|---|---|
| R3-QA-01 | T-23 | decisions_by_id 只收类型校验通过的记录；族引用被淘汰记录→定位拒绝 |
| R3-QA-02 | T-24 | 适用性=evidence 文件集合（移除 bound/subject OR 旁路） |
| R3-QA-03 | T-24 | imported/参数/局部绑定名传入 qualified 判定，冲突即 unknown |
| R3-QA-04 | T-25/27 | SD-08 提案 + 历史段消歧 + 最终指针断言清单（merge 执行） |
