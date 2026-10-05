# PLAN-743 Phase 2 修复基线（T-07）

> r2 合同：docs/plans/743-acc-bootstrap-hir-contract.md（plan_revision 2，T-07..T-14）。
> 本目录是 Phase 2 修复的验证资料目录（新路径，无外链）；复审反例原文在
> docs/reports/743-quality-review-20261005/（只读引用，不改动）。

## 激活与基线身份

| 项 | 值 |
|---|---|
| 激活提交（实施起点） | `c6e4e568994941883b8c1b6b19dff703feb081cd`（docs(plan): review Plan 743 and reactivate Phase 2 quality fixes） |
| 实施基线 | 同上（v0.6-dev，clean，plan_revision=2） |
| worktree | D:/autostack/.wt/lang-743/auto-lang，branch plan-743-dev（重建） |
| 上轮 reviewed（r1） | fa3abe476（外部复核基线）；原 r1 内部 R2=d7013b290→落地映射 9abc87acc |
| Category | A（不跑 cargo/tv/taa/tf/docs_gen；不新建外仓/链接） |
| Python | 3.14.2，标准库 |

## 并行范围核对（T-07 要求）

- PLAN-741 r3（Phase 3）由 741 线自行推进，本计划不改 741 交付物；
  743 对 741 事实/验收可用性的依赖在 T-12 逐条标注。
- 744/745 编号未被占用（.next-id=744，本 Phase 不取新号）。
- 无 ABI 线工作在本仓落地（auto-native-abi-rfc 仍为 Draft，未冻结字段）。

## 复审反例 before 状态复现（本 worktree @ c6e4e5689 实测）

### QA-01 跨行字符串泄漏（multiline-string.at，scan_module 直调）

```
unknown-bare-call pretend receiver=None line 3
```
`pretend()` 完全处于跨行字符串字面量内（第 2–4 行），却被计为调用候选——
mask 状态机在换行处将 string 状态恢复为 code。

### QA-02 方法名优先于接收者身份（same-name-method.at，scan_module 直调）

```
native-runtime len receiver=x      line 7   ← x 为变量，Meter 亦声明 len
native-runtime new receiver=Meter  line 6   ← Meter 是本地 type，非宿主
```
真实仓同病：`docs/reports/743-acc-hir-contract/source-manifest.json` 中
`CG.new`（codegen.at:4168 ×5）与 `Ar.new`（a2r.at:4654 ×1）均被标 native-runtime。

### QA-03 人工层完整性缺口（三反例，复审实测，本 Phase 修复后以测试锁定）

- unbound decision（非空决定/空绑定）→ 当前 exit0；
- missing evidence 文件 → 当前仅 WARN 仍 CHECK-OK；
- 空 decisions 集 → "0 条全部新鲜"。且 `--require-decisions` 旗标尚不存在
  （argparse unrecognized arguments）。

### QA-06 当前输入过期状态（真实报告目录实测）

`--check` 当前 CHECK-FAIL **9 项**：MD-203/205/301/302/304/406/409/421 共 8 条决定
的 9 处绑定引用 741 r2 更新后的 auto-ac/project.md 与 auto-hir/project.md。
扫描源码（9 受管输入）未漂移；陈旧决定门按设计生效，本 Phase T-12 逐条审定后重绑。

## 修复目标映射（QA → 任务）

| 发现 | 任务 | 主要落点 |
|---|---|---|
| QA-01 跨行泄漏 | T-08 | acc_inventory.py::mask_comments_and_strings + fixtures/tests |
| QA-02 分类越权 | T-08 | acc_inventory.py::scan_module/_post_classify + 调用分类断言 |
| QA-03 人工层门 | T-09 | validate_decisions/main + --require-decisions + 负例 |
| QA-04 trap/effect 边界 | T-10 | auto-acc-bootstrap-contract.md §2/R3/阶段表 + SD-05 提案 |
| QA-05 锚点/候选对账 | T-11 | acceptance-matrix.md/next-work-packages.md + SD-04 提案 |
| QA-06 输入新鲜度 | T-12 | manual-decisions.json 逐条审定重绑 + 新观察覆盖 |
| QA-07 勾选簿记 | （激活提交已完成校正）| 计划文件 T-05/06 历史勾选已由 r2 合同修正 |
