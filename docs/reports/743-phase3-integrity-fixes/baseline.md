# PLAN-743 Phase 3 修复基线（T-15）

> r3 合同：docs/plans/743-acc-bootstrap-hir-contract.md（plan_revision 3，T-15..21）。
> 外部复审：docs/reports/743-r2-quality-review-20261005/REVIEW.md（P743-R2-QA-01..06）。
> 本目录是 Phase 3 完整性修复的验证资料目录；反例原文只读引用。

## 激活与基线身份

| 项 | 值 |
|---|---|
| 实施基点 | `a0f8ddd443418fb32a67d308cbd4a0fd00f74835`（含 r3 合同的最新 v0.6-dev） |
| worktree | D:/autostack/.wt/lang-743/auto-lang，branch plan-743-dev（重建；r2 分支已删） |
| 主检出预检 | 0 WIP；并行线 ed2d00b90（NOTES-001 sqlite）不属 743，不在本 Phase 范围 |
| 上轮基线 | r2 reviewed_commit 5983f8aee（R3 pass 基线 a63660137→落地 8b44ae855） |
| Category | A（PYTHONUTF8=1 消除 harness 编码歧义；不跑 cargo/docs_gen；无外仓/链接） |
| 编号 | .next-id=744 未占用；741 r3/ABI/742/NOTES 线独立 |

## 复审反例 before 状态（worktree @ a0f8ddd44 实测，scan_module 直调）

### R2-QA-01 分类越权（三 fixture）

```
shadowed_host.at    -> native-runtime print        （本地 fn print 遮蔽宿主内建，应 local-fn）
                      native-runtime read_line IO  （本地 type IO 遮蔽宿主命名空间，应 type-qualified）
different_owner.at  -> implicit-self-method next   （.next() 在 Q 方法体内，next 属 P，应 unknown）
unrelated_dot_pipe.at -> implicit-self-method next ×2（自由函数内点/管道形态，无 owner，应 unknown）
```
根因：①NATIVE_NAMESPACES/BARE_NATIVES 判定先于本地符号表（遮蔽不生效）；
②_post_classify 用全模块方法名集合升级 dot/pipe，无 enclosing-owner 记录。

### R2-QA-02 证据未绑定不失效

当前 validate_decisions 只查证据文件存在、绑定另行遍历、二者无包含关系；
合法层追加仓内非扫描输入证据（如 README.md:1）而无绑定 → 严格 --check 仍 exit0
（复审 fixture 实测；当前真实 47 决定 evidence 全有绑定，audit.json 空缺口，故现状不误报）。

### R2-QA-03 resolved 族无决定引用仍宣称闭环

resolved 只查 disposition、ref 为空不报错；删除全部 resolved 引用后严格门仍
"覆盖闭环通过"（复审独立 fixture 实测）。

### R2-QA-04 字段类型错误抛 traceback

kind=[]、conclusion=None、evidence="not-a-list"、bound_input_hashes=["oops"] 等
合法 JSON 错误类型 → TypeError/AttributeError traceback（本基线实测 stderr 含
Traceback），非受控 ERROR[...] 定位拒绝。

### R2-QA-05/06（文档/生命周期域）

- stage-contract.md 阶段规则摘要仍写"R3 effect 只能收窄"，与正文两分规则冲突。
- acceptance-matrix/next-work-packages 的 CLI 仍写 ac-probe（741 r2 已更名
  auto-ac-prototype）。
- 归档态链接 ../reports/... 自 docs/plans/archive/ 解析为 docs/plans/reports/ 不存在；
  auto-acc/plans.md、auto-hir/plans.md 的 743 行状态过期（激活后为 r3 executing）。

## 修复目标映射

| 发现 | 任务 | 落点 |
|---|---|---|
| R2-QA-01 | T-16 | scan_module（本地符号优先 + enclosing-owner 记录）/ _post_classify + 正反例 |
| R2-QA-02/04 | T-17 | validate_decisions 类型前置校验 + 同条 evidence-binding + 参数化负例 |
| R2-QA-03 | T-18 | resolved/open 引用与适用规则 + 真实层核准 |
| R2-QA-05/06 | T-19 | 阶段摘要 SD-07、CLI 更名同步、链接/plans.md 状态、phase3 delta |
