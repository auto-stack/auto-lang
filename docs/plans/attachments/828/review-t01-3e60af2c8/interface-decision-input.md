# PLAN-828 T-01 接口决策（D-01..03）与双函数真实竖切

冻结提交：`plan-828-dev@3e60af2c8`（基面 `47894c2e6`，同树 `D:/autostack/.wt/lang-828/auto-lang`）。
本文按 §5.6 记录 T-01 的有界决策、真实证据与剩余边界；未回答条目将阻止依赖它的广泛改造。

## D-01 受检环境与单 body 检查（F-01 前端接缝）

**决策**：受检环境 = 冷编 `resolve_env` 保留的 `CheckedEnv{program: ResolvedProgram, rec_index, sigs, trace_enabled, profile}`（ac-core `source/resolve.rs`）。单 body 检查 = `recheck_function(env, item, fn_index)`：签名漂移守卫（`fn_sig_of` 按冷编同一规则重建签名，任何差异=受控拒绝 `source.incremental`）→ 真 `resolve_fn`（环境 sigs/records 供全部名字查询）→ O(1) 槽位换入 → `check_body_in` 只查该 body（自递归见新条目；失败回滚旧条目）。发射 = `emit_function_body(env, fn_index, src)`：Emitter 对改函做真 body 计划，对其余函数做**签名脚手架**（`stub_resolved`：保留条目的字段级空桩——空 body/仅参局部/无 loop/无 intrinsic），全环境供 callee/类型查询；单文档经同一 render/bind/reanchor 规则产出。

**关键发现（竖切实测）**：单 def 文档的跨函数 DefRef 要求 callee def 存在——脚手架即 §5.6 允许的"接口目录遍历"。**emit 层新旧路径等价**（旧 synth 桩管线同样付脚手架费）；本接缝的真实收益在 resolve/typecheck 的按 body 定界——由环境内生产者计数器证明（`env_walk_stats`：冷基线 N/N，每次 recheck 恰 +1/+1）。§5.8 的 F-01 门按"缺必需观测即拒 PASS"语义实现：PASS 需 `BodyEmitReceipt{real_body_plans:1, scaffold_fns:N-1}` + 行走计数凭证；旧 `compile_program_parts` 入口不产生该凭证 → 同门必红（测试 `contract_828_frontend_single_body_matches_cold_compile` 同时演示两态）。

**证据（probe/tests/function_incremental_828_contract.rs，filter `contract_828_frontend`，5/5）**：
- C01-01 最小正控：改 helper +1→+2，单 body HIR 与冷编 `compile_parts_with` 的 d_helper **canonical dump 逐字节相等**；计数 3/3（恰一次 resolve+一次 typecheck）。
- C01-04 最小：`n + true` 真类型错拒绝（`source.type-mismatch`）且环境不中毒（后续合法 recheck 通过）。
- 签名漂移受控拒绝；caller 条目保留可复现（对 main 用原 item recheck+emit，dump 与冷编 d_main 相等）。

**边界与 T-02 责任**：scalar 自由函数面已接（receiver/method 路径仍整编）；session 热路径仍走 synth 桩管线（T-02 换接 `resolve_env/recheck_function/emit_function_body` 并绑定观测）；`stubify`/`synth_stub_walks` 计数随 T-02 移除。HIR 构造边界不变：仍经 Emitter→Atom→bind（未手写 HIR）；A07 函数 HIR 产物/恢复归 T-03/T-07（本接缝产出未验证 `SourceParts`，语义验证仍走既有 verify 入口）。

## D-02 无 merged 派发（F-02 执行载荷）

**决策**：T-05/06 实现方向冻结为——链接期计算**绑定表**（def 名→函数槽、每单元局部 const/type 索引→快照级interned解析、import 调用→槽位绑定），AV 三入口（`execute_program/execute_program_steps/resume_program`）共用同一 dispatch，经单元视图直接消费 `FunctionCode` 的不可变 `MirFunction`（局部 PC/槽/常量按单元局部索引+表解析），**不重建 merged 载荷、不重写未改指令**。持久 codec/公共接缝在 T-01 不变（观察 API 纯新增）。

**实测现状（F-02 量化，producer 报告）**：`ProgramSnapshot::dispatched_payload_same_object/same_content`（merged 视图生产者自身报告）——未改函数的派发载荷今天**对象身份红**（每快照 merged 副本），且竖切发现**内容也漂移**：改函增长使 merged 类型/常量空间重索引，后续未改函数的 merged 指令内容随之变化；**单元载荷**（不可变 FunctionCode 内的 MirFunction）内容恒等。代码级身份轴绿：未改函数跨快照同一 `Arc<FunctionCode>`；强制同内容副本（`as_ref().clone()` 重建快照）结果仍正确但身份门红——负控可检出（§5.8 C02-05 雏形）。

**证据**：`contract_828_dispatch_payload_identity_axes`（filter `contract_828_dispatch`，1/1）覆盖代码身份正控、强制副本负控、派发轴当前红/单元内容绿的实测记录。T-05/06 落地后该测试的派发轴断言翻绿即为 C02-01 最小正控的最终形态（当前测试如实断言红——F-02 OPEN 的量化）。

## D-03 所有权对应（§5.7 表 ↔ 当前代码）

| §5.7 行 | 代码对应（3e60af2c8） |
|---|---|
| Open session | `session.rs` `current/fns(+history)/env/context_revision`；prepare/失败/取消/迟到/foreign 不改 root（r23a close-controls 全绿保持） |
| PreparedEdit | `prepare_edit` 产出、外部持有至 commit/cancel/drop；closed 拒绝亦消费（r23a 证据） |
| ProgramSnapshot | `Arc<ProgramSnapshot>`；close 撤销 session 拥有引用→vacant 根（r23a F05b-C01）；无反向强持有 |
| PausedRun/帧 | `execute_program_steps/resume_program`；close/drop session 后旧链完成/取消（r23a close-adjacent） |
| Closed session | `close()/closed()`；run=Infra、prepare/edit=Err、commit/cancel=拒、无 owner 资源 close 即死 |
| store/lease | `persist_all_units/restore`+unit-span sidecar（F-10/F-11 已闭部分）；GC/lease/水位仍 OPEN |

F-01/F-02 改动后的重放集合：r23a close-controls/close-adjacent、62 例会话重放、r1 套件（86）+ contract（5）+ 兼容面；F-05b 行为按受影响面重放，不据旧 PASS 覆盖新代码。

## 测试选择表（新目标）

| 目标 | filter | 现数 |
|---|---|---|
| probe/tests/function_incremental_828_contract.rs | `contract_828_frontend` | 4 |
| 同上 | `contract_828_dispatch` | 1 |
| 既有不变面 | `function_incremental_828_r1`（86）/`function_incremental_828`（17）/core（22） | — |
| ac-core 内部（source 重构面） | `cargo test --manifest-path experimental/ac-core/Cargo.toml`（15+11+6）+ ac-syntax `scalar_projection_`（6） | — |

新公共 API（ac-core `source`）：`resolve_env`/`recheck_function`/`CheckedEnv`/`env_fn_count`/`env_note_cold_walks`/`emit_function_body`/`BodyEmitReceipt`/`SourceParts.planned_fns`；（probe core）：`ProgramSnapshot::dispatched_payload_same_object/same_content`。schema/授权/资源接缝无变化（F-09 预算轴不因此增减）。

## 回归面（本轮实测，全部定向）

ac-core 15+11+6、scalar_projection 6/6、contract 5/5、r1 86/86、主 17/17、core 22/22、兼容面 cache20/mir8/abc15/binding4/vertical3/case5、r23a close-controls 6/6、rustfmt 通过；`resolve_with` 重构为 `resolve_detailed` 共享体，行为面由上述全套绿证。
