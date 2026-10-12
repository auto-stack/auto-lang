# PLAN-738 独立复审 R10（2026-10-10，pass → reviewed；R9 必修闭合定向复审）

- stage: review | plan_id: PLAN-738 | plan_revision: 3（验收合同不变）
- outcome: **pass**——P738-R9-01/P738-R9-02 在修复合并树 `41b4d4be5` 上闭合确证；门禁全绿（含本轮补跑的正式生成服务链）；无新引入问题。修复合并树可落地，**可重启 merge**。
- reviewed_commit: `41b4d4be52723ec2462e5ab511d3ea8e588dd38e`（worktree `D:/autostack/.wt/lang-738/auto-lang`，plan-738-dev，入场/结束 clean，只读复审）
- base_commit: 修复父 `19973b089`（R9-I/R9-II 复审对象）；历史链 R8 reviewed `3713337d9` / SD `641f66a5f` / SD 沉淀 `adfb7a5d1`
- independence: 全新上下文 R10 agent，未参与本计划任何实现/复审。
- 复审范围：R8 已对全 AC/SD pass（@3713337d9）；R9-I/R9-II 已确证主链修复机制有效 + 两项 reviewed 树既有必修；本轮定向 = R9-01/02 闭合 + 修复合并树回归面 + 一致性，不重复全 AC 审计。

## 1. P738-R9-01 闭合确证（lock 读错误 fail-closed）

### gate 侧（rust_ui.rs `backend_generation_is_fresh`）

代码链逐级核证（`git show 41b4d4be5` + worktree 实读 L4020-4060）：

1. `classify_lock_read`（L4054-4060，纯函数）：`Ok(bytes)→Some(bytes)`；`Err(NotFound)→Some(Vec::new())`（空字节=absent 哨兵）；`Err(_)→None`（权限/IO 不可核验）。旧 `.ok().map(hash).or_else(absent)` 的 fail-open 路径已删除。
2. `current_lock` 映射（L4026-4035）：`classify_lock_read(...).map(|bytes| 空→"absent"、非空→fnv1a)`——`None` 不进 `map`，读错误传播为 `current=None`。
3. 调用链（L4040-4047）：首物化绑定臂 `(Some("absent"), Some(materialized))` 仅在 `current=Some` 时命中；`current=None` 落入 `lock_freshness(recorded, current)`。
4. `lock_freshness`（L4079-4084）：`(Some(a), Some(b)) => a == b, _ => false`——**任意 recorded × current=None 一律 false（保守陈旧）**。
5. R9-01 原反例（recorded="absent" × PermissionDenied → 旧 absent/absent 假新鲜）现判 **false**：单测断言在案（`classify_lock_read_fail_closed` L5188 `!lock_freshness(Some("absent"), None)`；真值表 L5076 同向、L5072 bound×absent、L5074 unbound×absent=合法新鲜、L5077 None/None=false）。
6. 既有语义保留：R5 一次绑定状态机 `review738_r5_lock_binding_state_machine` 1/1（首物化 fresh+绑定、漂移/删除/缺字段拒绝）；R6-02 绑定写失败保守陈旧（L4042-4044）。

### 生成侧（api_gen.rs `generate_rust_server` L1068-1082）

`Ok→fnv1a`；`Err(NotFound)→"absent"`；**`Err(e)→return Err("workspace Cargo.lock unreadable: {e}")`**——拒写收据（收据不得携带不可核验 lock 身份）。旧 `unwrap_or_else(|_| absent)` 吞错路径已删除。

### 实跑

- `cargo test -p auto-man --lib classify_lock_read_fail_closed -- --test-threads=1` = **1/1 ok**
- `cargo test -p auto-man --lib freshness -- --test-threads=1` = **2 passed**（`assembly_freshness_truth_table` + `lock_freshness_truth_table`）+ 1 failed=`test_shell_pack_lib_freshness`（**预存分诊见 §4**）
- `cargo test -p auto-man --lib review738_r5_lock -- --test-threads=1` = **1/1 ok**

## 2. P738-R9-02 闭合确证（四格式点收敛）

- 修复 hunk（lib.rs:119 注释对齐 / renderer.rs:24761 launcher match / engine.rs:9840+9852 pop 表达式）**token 级核证**：lib.rs 与 engine.rs 去空白 token 流与父提交**逐字节一致**；renderer.rs 唯一 token 差异 = `catch_unwind(AssertUnwindSafe(build),)` 尾随逗号移除（rustfmt 尾逗号规范化，AST 等价）。
- `rustfmt --edition 2021 --check`（1.9.0-stable）五个触面文件（lib/renderer/engine/rust_ui/api_gen）**全部干净 exit 0**——格式收敛机械证明，覆盖 R9 报告全部四点且不扩大范围。
- 触面 hunk 零新编译警告（touch 强制重编后 warning 均在 hunk 之外，为 master 继承存量）。

## 3. 回归面（41b4d4be5 worktree 实跑，2026-10-10 串行）

| 门禁 | 结果 |
|---|---|
| `cargo t plan738` | **73/73**（42.5s，含 12 方法×双拼写矩阵、t12 三角、p4 探针） |
| `cargo tv` | **162/162**（2.4s；**cb_web_mime PASS@1.2s**——merge 假红已修） |
| `cargo test -p auto-man --lib api_gen -- --test-threads=1` | **44 passed / 1 ignored**（1.7s，含真实生成消费者腿） |
| 正式生成服务完整链 `--features test-http-e2e http_e2e_plan738 -- --ignored` | **1/1 @ 42.29s**（生成→收据→实编→serve→ready→业务→stdlib-only 失效→再生→重建复验）——R9-I 报告明确要求「重跑正式生成服务链」、work 交接以合同面一致为由省略，本轮**补跑确证**（rust_ui.rs lock 门为该链 ready 消费路径，41b4d4be5 独有语义变化，不可沿用 19973b089 收据） |
| 三 crate check（`-p auto-lang -p auto-man -p auto`） | **exit 0 零 error**（存量 warning 保留，触面 hunk 零新增） |
| worktree 状态 | 入场/结束 **clean**（touch 仅 mtime 未触内容） |
| 修复 diff 零 debug 残留 | 新增行 grep `dbg!/println!("[/[DEBUG/todo!/unimplemented!` **零命中**（renderer.rs 的 eprintln 为 plan-453 既有行随缩进移动） |
| `git diff 19973b089 41b4d4be5 -- docs/specs/` | **空**——SD 沉淀（adfb7a5d1 九文件）未被扰动 |
| 合同面（stdlib_assembly/、native.rs、native_catalog.rs、native_registry.rs、stdlib/） | `git diff --quiet` = **逐字节一致**——R9-II「两树合同面一致」结论未被修复扰动；VM/契约门禁不重复全跑的省略依据经本轮机械复核成立 |

## 4. 预存红分诊（非本修复引入）

- `test_shell_pack_lib_freshness`（freshness 族内）：R3 起在案（R4 记录同名同 panic 复现；R9 交接「shell_pack=R3 在案预存」）。机制分诊：该测试对拍 **auto-os/shell 外部仓**生成物（`generate_shell_pack_lib_sources(&pack_dir)`）vs 入库物 `crates/shell-pack/src/lib.rs`，与本修复 lock 门零交集；生成器函数不在修复 diff 中；红因 = 跨仓资产漂移（PLAN-036 域），属外部仓同步问题。

## 5. 观察项（非阻塞，登记不批准延期）

- **P738-R10-01（P4 观察）**：R9-I 修复要求的负例矩阵中 bound(hash)×读错误(None) 组合无显式单测断言（语义由 `lock_freshness` 单一 `_ => false` 臂覆盖，unbound×读错误反例已有显式断言）；「恢复可读后证明能收敛」无专门测试（机制保证 = 读路径纯函数零状态写入，恢复后自然回到正常真值表）。测试完备性小缺口，语义已闭合，建议 merge 顺手补一条断言。
- **P738-R10-02（P4 观察）**：gate 侧真实 0 字节 Cargo.lock 会被归入 "absent" 哨兵（`bytes.is_empty()→"absent"`）。语义可辩护（空 lock=零依赖信息、非空化即触发陈旧、cargo 实际不产空 lock），方向非 fail-open；设计意图已在注释说明。
- **P738-R10-03（流程记录）**：work 交接以「合同面两树一致」为由省略 R9-I 明确要求的服务链重跑——省略理由覆盖面（VM/合同面）不含 rust_ui.rs lock 门（服务链 ready 消费路径）。本轮补跑闭合，不构成缺陷，记录以警示后续交接的省略理由须与触面逐项核对。

## 6. 判决

**pass**。覆盖链：R8 全 AC/SD pass（@3713337d9）→ R9-I/R9-II 主链修复机制确证 + 两项 reviewed 树既有必修 → **R10 两项必修在修复合并树 41b4d4be5 闭合确证 + 门禁绿（含补跑服务链）+ 无新引入问题**。修复合并树（41b4d4be5）可落地。

- state: **reviewed**（current_step=7，R8 口径恢复：T-01/T-09..T-14 完成，T-02..T-08 由 Phase 3 承接）
- next: **重启 merge**（`/auto-plan:merge PLAN-738`，授权后）——SD 沉淀已在树（adfb7a5d1）、ledger/Design33/索引、wt-guard+worktree 清理；R9-I 的 HTTP 全档未完成段（45/102 停跑）随 merge 后批量回归（`/auto-plan:regress`，到期判定已触发：last_covered_plan_id=740，2026-10-08T11:20Z）收口。
- 不合入/不归档/不删 worktree；禁 tf（批量回归档归 regress）。
