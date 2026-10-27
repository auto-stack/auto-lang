# PLAN-824 T-07 R1 needs_fix 修复闭合表（repair round 1）

- stage: work（needs_fix 修复轮）; plan_revision: 7（不变）
- worktree: D:/autostack/.wt/lang-824/auto-lang; branch: plan-824-dev
- 修复基线: reviewed_commit 2093675731 → 修复提交 **6a68240e8**（11 文件，+1198/−97）
- 依据: [review.md](review.md)（R1 三项 P1 + unused import）；修复在唯一实施树内完成，未合入 master。

## 闭合表（finding → invariant → 修复与永久用例 → 当前结果 → 余隙）

| finding | 不变量 | 修复与永久回归（真实测试入口） | 当前结果 | 余隙 |
|---|---|---|---|---|
| F-824-R1-01 标量叶子描述符/载荷错配被接受（`take_as(I32)` 返回 `Ok(Bool)`） | 未信任内层字节在授予类型化访问前逐节点校验；好 checksum 不洗白结构 | `portable.rs::expect_leaf_kind`：8×8 描述符 kind × 载荷 tag 等价门（每个标量节点解码处，先于 rollup）；永久用例 `av_dynamic_824_wire_scalar_kind_equivalence`（harness，含 review 原样重放：wire[16] TY_BOOL→TY_I32 + 重算 FNV → Invalid type；record/List/活跃 variant payload 下同形控制；合法位保真往返 + 错型 take 源保留） | harness 27/27 绿 | 无 |
| F-824-R1-02 恢复无法校验当前字段/payload 类型；空容器/不活跃 arm 的名义依赖漏检 | 恢复对**完整递归类型身份**做当前依赖准入，与值节点是否到访无关 | binding 携带类型：`RecordBinding.fields: Vec<(&str, Rc<DynType>)>`、`ArmSig.payload: Option<Rc<DynType>>`；根 tydesc 一次性全量准入 `admit_tydesc`（迭代工作list + `ty_ident` 逐对计费；下游节点类型为根子树且 rollup 拒绝不一致）；`portable_bindings_of` 从当前类型表记忆化解析（Rc 共享 + MAX_DYN_NODES 总预算，坏表 Err）；永久用例 `av_dynamic_824_admission_type_only_dependencies`（空 List<gone-owner/Gone> → unknown-type；不活跃 arm payload 同；登记后恢复）、`av_dynamic_824_admission_changed_field_and_payload_types`（同名改型 i32→bool / variant payload 改型 → layout-mismatch；同 schema 新位号 77 仍恢复）、`av_dynamic_824_semantics_restore_revalidates_current_field_types`（**真实 adapter** portable_bindings_of 轴） | harness 27/27、semantics 21/21 绿 | 无 |
| F-824-R1-03 类型 DAG 展开绕过工作/内存计量（721B ABC → 262143 节点 ≈20MiB 仅计 8 步） | 类型分析/展开/比较在过量工作与分配**之前**受预算并计费，分态+取消+清理 | `av.rs::resolve_dyn_type` 返回 (树,节点数)，展开超 MAX_DYN_NODES 即中止 → `Resource(DynTypeNodes, "av.resource.dyn-nodes")`；三条 Dyn 指令把节点数计入 st.steps（跨 max_steps → InstrSteps）；`mir_verify::dyn_whitelisted` (idx,depth) 记忆化（指数路径 → 线性，语义等价：结构环在任意上下文重现）；`contract.rs` deep_size 计入 Dyn 类型描述符树（8MiB 值数据线）；`dynamic.rs::validate_type` 类型节点上限（DynPackError::code 返回实际 code）；永久用例 `av_dynamic_824_resources_type_dag_expansion_bounded_and_charged`（15 层 65535 节点=成功且 steps≥65535、16 层 131071 / 17 层 review 原形 → DynTypeNodes、紧 step 预算 → InstrSteps） | resources 12/12 绿；deep_size 计量后 resources 既有 peak/边界控制全绿 | 无（同源 harness `av_dynamic_824_budget_nodes_freeze` 顺带迁至 pack 侧类型上限，freeze 侧值节点上限改用嵌套 List 控制覆盖） |
| 附带：probe/tests/av_dynamic_824_incremental.rs:14 unused import（test target 零警告主张失实） | 零警告健康门 | 删除 `verify_mir` 导入；cargo check --all-targets 两 manifest 0 警告，fmt --check 零漂移 | 绿 | 无 |

## 重放与受影响面（check → finding 映射）

- review 反例重放：三项原例全部以**永久测试**形式重放（非一次性 probe）；旧 probe 二进制因 binding API 变更不再编译，其重放职责由上表测试承担。
- av_dynamic_824 全族（最终代码 6a68240e8）：first-case 4/4、semantics 21/21、admission 1/1、resources 12/12、cache 9/9、incremental 5/5、baseline 6 入口、autodown 3/3、同源 harness 27/27；verify-av-dynamic-824.ps1 REQ 钉住新 close-oracle 用例（semantics/resources/harness 三处）。
- 兼容六族（共享 contract/av/mir_verify 触面）：821 first-case 4 + cache 7 + 语料 3、822 first-case 5 + 语料 3、798/820 语料（--skip turnaround）14、817 语料 5、818 语料（--skip turnaround --skip rounds_failure）18 —— **语义面全绿**。
- 时序门负载归因：acceptance 期间他方会话 Plan 856 merge 串行门（plan-856/merge-r1/gates.py, cargo test --locked --test-threads=1）持续占机；两次失败轮替出现于不同测试（818 P95 1022ms≥1000ms 门 / 818 session-wall / 798 formal_turnaround），全部时窗 1.7–2.5× 基线——负载伪影，非本 diff 影响（818/798 路径不含 Dyn 指令/Dyn 值，dyn_whitelisted 仅 Dyn 操作数触发）。安静机器复跑见下。

## 已核预存边界（非本次引入，如实保留）

- `read_tydesc` 递归（未改动基线代码）：1MiB 主线程 ~242 深溢出（stash 基线对照：241 过/245 溢，与修复后完全一致）；≥2MiB 线程 256 深安全（libtest/生产线程语境，resources 255 深测试绿）。本次修复把新增准入遍历全部迭代化，未加深该边界。
- docs/reports/plan818/turnaround-*.json 为测试再生计时工件，每轮 compatibility 运行会改动（预存行为，提交中已还原）。

## 余项与门禁

- compatibility 时序门终局：**818 两门安静机器复跑 PASS**（turnaround_body_edit_gate、rounds_failure_liveness_and_caps）；798 formal_turnaround_gates 同提交隔离跑 PASS（164s），聚合形态红（edit P95 1308ms ≥ 1000ms）——**stash 基线对照同形态更红（P95 1715.6ms）**，属预存机器边缘条件（当日 P50 844-931ms、离群样本 1.3-2.4s 拖 P95；本日 acceptance#2 高负载下该族反而绿过），非本 diff 回归（修复代码路径不含 Dyn 指令/Dyn 值）。merge 冻结门禁复跑时按机器状态判定。
- 过程事故（已恢复，如实记录）：验证期一次基线 stash 对照中，因 worktree 树已 clean（修复已提交）而 `git stash pop` 误弹共享 stash 栈顶的他方条目（plan-822 unowned WIP），在 collections_session.rs 半套用出 UU 冲突；立即 `git reset --hard 6a68240e8` 复位，stash 栈 5 条原样核对无损，本计划无未提交内容损失。教训：共享 stash 栈的多 worktree 仓禁用 stash 做基线对照（后续用临时 worktree/分支代替）。
- merge-owned 门禁全程 pending（裸 cargo t + 触面 tv + 新 workspace 有限合同档）；tf 仅主检出批量档。
- 823 并行编号冲突（id5/AVAD v4/opcode）仍归 merge 统一裁定。
