# PLAN-828 T-01 接口决策（D-01..03）与双函数真实竖切

冻结提交：`plan-828-dev@23a6b35a0`（T-01 修复轮；首轮 `3e60af2c8`，基面 `47894c2e6`，同树 `D:/autostack/.wt/lang-828/auto-lang`）。
本文按 §5.6 记录 T-01 的有界决策、真实证据与剩余边界；未回答条目将阻止依赖它的广泛改造。r3 T-01 复审三项 P2（T01-C01/02/03）已在本轮闭合，见各节〔修复〕标注。

## D-01 受检环境与单 body 检查（F-01 前端接缝）

**决策**：受检环境 = 冷编 `resolve_env` 保留的 `CheckedEnv{program: ResolvedProgram, rec_index, sigs, trace_enabled, profile}`（ac-core `source/resolve.rs`）。单 body 检查 = `recheck_function(env, item, fn_index)`：签名漂移守卫（`fn_sig_of` 按冷编同一规则重建签名，任何差异=受控拒绝 `source.incremental`）→ 真 `resolve_fn`（环境 sigs/records 供全部名字查询）→ O(1) 槽位换入 → `check_body_in` 只查该 body（自递归见新条目；失败回滚旧条目）。发射 = `emit_function_body(env, fn_index, src)`：Emitter 对改函做真 body 计划，对其余函数做**签名脚手架**（`stub_resolved`：保留条目的字段级空桩——空 body/仅参局部/无 loop/无 intrinsic），全环境供 callee/类型查询；单文档经同一 render/bind/reanchor 规则产出。

**关键发现（竖切实测）**：单 def 文档的跨函数 DefRef 要求 callee def 存在。
〔T01-C01 修复〕callee 现以**无 body 声明脚手架**满足存在性：五个 profile schema 的 `function` 声明将 `body` 放宽为可选（向后兼容——既有文档全部带 body），binder 对无 body 声明产出 `DefKind::Function{name, ty, body: BodyId(u32::MAX) 哨兵}`（函数级验证只读 callee 类型、从不解引用哨兵 body；整模块文档恒带真 body）。热路径发射**恰好一个 body 文档**：types 段=全函数接口目录（允许项），declarations 段=改函带 body + 其余无 body，bodies 段仅改函；未改函数零 plan/零 body 渲染/零 bind，`stub_resolved` 深拷贝已删除。生产者产物即证据：`parts.bundle.modules[0].bodies.len()==1`、receipt `real_body_plans:1, scaffold_fns:N-1`（脚手架=无 body 声明）。
〔T01-C02 修复〕结构门挂到**真实生产者**：ac-core 新增 `body_work::capture_body_work` 观测窗，`record_resolve/typecheck/plan/bind` 分别挂在 resolve_detailed/recheck 的 resolve_fn 调用点、check_block 入口、plan_fn 入口、binder bind_body（`b_<name>`）——按函数名记账，接口目录（types/无 body 声明）不计。门谓词：窗口内改函恰 +1/+1/+1/+1 且**其他函数全零**。真实旧 synth 管线在同一门下红（其 stub body 经历完整 resolve/typecheck/plan/bind，被生产者记账）；旧计数门（env_walk_stats+receipt 缺失即拒）保留为第二轴。

**证据（probe/tests/function_incremental_828_contract.rs，filter `contract_828_frontend`，5/5）**：
- C01-01 最小正控：改 helper +1→+2，单 body HIR 与冷编 `compile_parts_with` 的 d_helper **canonical dump 逐字节相等**；计数 3/3（恰一次 resolve+一次 typecheck）。
- C01-04 最小：`n + true` 真类型错拒绝（`source.type-mismatch`）且环境不中毒（后续合法 recheck 通过）。
- 签名漂移受控拒绝；caller 条目保留可复现（对 main 用原 item recheck+emit，dump 与冷编 d_main 相等）。

**边界与 T-02 责任**：scalar 自由函数面已接（receiver/method 路径仍整编）；session 热路径仍走 synth 桩管线（T-02 换接 `resolve_env/recheck_function/emit_function_body` 并绑定观测）；`stubify`/`synth_stub_walks` 计数随 T-02 移除。HIR 构造边界不变：仍经 Emitter→Atom→bind（未手写 HIR）；A07 函数 HIR 产物/恢复归 T-03/T-07（本接缝产出未验证 `SourceParts`，语义验证仍走既有 verify 入口）。

## D-02 无 merged 派发（F-02 执行载荷）

**决策**：T-05/06 实现方向冻结为——链接期计算**绑定表**（def 名→函数槽、每单元局部 const/type 索引→快照级interned解析、import 调用→槽位绑定），AV 三入口（`execute_program/execute_program_steps/resume_program`）共用同一 dispatch，经单元视图直接消费 `FunctionCode` 的不可变 `MirFunction`（局部 PC/槽/常量按单元局部索引+表解析），**不重建 merged 载荷、不重写未改指令**。持久 codec/公共接缝在 T-01 不变（观察 API 纯新增）。

**实测现状（F-02 量化，producer 报告）**：`ProgramSnapshot::dispatched_payload_same_object/same_content`（merged 视图生产者自身报告）——未改函数的派发载荷今天**对象身份红**（每快照 merged 副本），且竖切发现**内容也漂移**：改函增长使 merged 类型/常量空间重索引，后续未改函数的 merged 指令内容随之变化；**单元载荷**（不可变 FunctionCode 内的 MirFunction）内容恒等。代码级身份轴绿：未改函数跨快照同一 `Arc<FunctionCode>`；强制同内容副本（`as_ref().clone()` 重建快照）结果仍正确但身份门红——负控可检出（§5.8 C02-05 雏形）。

**〔D-02 前置已补〕实际消费接缝与双函数首例**：`av.rs` 引入 `ModuleView` trait（fn_at/const_at/zero_of/type_is_bool/resolve_callee/entry）——三公开入口与单元原型共用同一 `dispatch`（MergedView=今行为逐点保持，`typed_zero` 移除）。`ProgramSnapshot::unit_view()`/`UnitView` 按单元局部索引消费各函数**原始不可变 MirFunction**（blocks/slots/PC/consts/types 单元本地），跨函数调用经链接期名字表（`fn:<name>`→槽位）解析；`execute_program_units` 原型入口执行双/三函数程序结果正确且执行路径消费的对象即快照持有的不可变载荷（C02-01 正控的执行路径形态）；`unit_binding_complete()` 为绑定拒绝控制（无单元快照拒绝单元派发；分发点 Import 未解析→受控 Infra 为纵深防御）。
**证据**：`contract_828_dispatch` 2/2（identity_axes + unit_view_first_case：值正确、merged/unit 双视图一致、未改函数单元载荷跨编辑同分配）；`t01r` 收编 4/4。merged 派发轴当前红仍是 F-02 的量化现状（T-05/06 换默认路径后翻绿）。

## D-03 所有权对应（§5.7 表 ↔ 当前代码）

| §5.7 行 | 代码对应（3e60af2c8） |
|---|---|
| Open session | `session.rs` `current/fns(+history)/env/context_revision`；prepare/失败/取消/迟到/foreign 不改 root（r23a close-controls 全绿保持） |
| PreparedEdit | `prepare_edit` 产出、外部持有至 commit/cancel/drop；closed 拒绝亦消费（r23a 证据） |
| ProgramSnapshot | `Arc<ProgramSnapshot>`；close 撤销 session 拥有引用→vacant 根（r23a F05b-C01）；无反向强持有 |
| PausedRun/帧 | `execute_program_steps/resume_program`；close/drop session 后旧链完成/取消（r23a close-adjacent） |
| Closed session | `close()/closed()`；run=Infra、prepare/edit=Err、commit/cancel=拒、无 owner 资源 close 即死 |
| store/lease | `persist_all_units/restore`+unit-span sidecar（F-10/F-11 已闭部分）；GC/lease/水位仍 OPEN |

F-01/F-02 改动后的重放集合：r23a close-controls/close-adjacent、62 例会话重放、r1 套件（86）+ contract（6+4）+ 兼容面；F-05b 行为按受影响面重放，不据旧 PASS 覆盖新代码。
〔D-03 前置决策补齐〕recheck→候选流转（T-02 接线语义，本轮冻结）：(a) `recheck_function` 成功=**候选 HIR 就绪**，session.prepare 内再走 `emit_function_body`+`verify_function`+lower/admit 产 `PreparedEdit`；prepare 不改 env 槽位之外的任何 session 状态——候选验证失败时 **recheck 的 env 槽位换入保留**（环境反映"最近一次合法检查"，候选失败只拒安装不回滚环境；下次编辑在此基础上继续，env_walk_stats 如实记账）；若需严格回滚失败 recheck，T-02 可先 recheck 到影子槽——该选择留给 T-02 按计数门裁定，两语义都不提前发布。(b) 取消/迟到/foreign：现有 commit/cancel_prepared 守卫不变（r23a 已闭）；CheckedEnv 生命周期=session（open 状态拥有），close 释放 fns/history/store 时 env 一并释放（env 在 session 内，无独立所有权）。(c) commit 一次更新：现有 commit 单点更新 root/revision/span_maps ✓。(d) **禁止全 env clone**：CheckedEnv 无 Clone（derive 已移除），T-02 不得以整环境复制抵消 F-01——增量编辑经 recheck_function 原地演进。

## 测试选择表（新目标）

| 目标 | filter | 现数 |
|---|---|---|
| probe/tests/function_incremental_828_contract.rs | `contract_828_frontend` | 4 |
| 同上 | `contract_828_dispatch` | 2 |
| probe/tests/function_incremental_828_contract_t01r.rs（收编 review-t01 frontend-controls；C02 控制采用生产者窗口门） | `t01_review_` | 4 |
| 既有不变面 | `function_incremental_828_r1`（86）/`function_incremental_828`（17）/core（22） | — |
| ac-core 内部（source 重构面） | `cargo test --manifest-path experimental/ac-core/Cargo.toml`（15+11+6）+ ac-syntax `scalar_projection_`（6） | — |

新公共 API（ac-core）：`source::{resolve_env/recheck_function/CheckedEnv/env_fn_count/env_note_cold_walks/emit_function_body/BodyEmitReceipt/SourceParts.planned_fns}`、`body_work::{capture_body_work/BodyWork}`；（probe core）：`ProgramSnapshot::dispatched_payload_same_object/same_content/unit_view/unit_binding_complete`、`UnitView`、`av::execute_program_units`。〔T01-C03〕`FnSig` 提为 pub(crate) 与 CheckedEnv.sigs 字段可见性一致，本轮 ac-core/probe 构建 0 新警告。schema/授权/资源接缝无变化（F-09 预算轴不因此增减）。

## 回归面（本轮实测，全部定向）

ac-core 15+11+6、scalar_projection 6/6、contract 5/5、r1 86/86、主 17/17、core 22/22、兼容面 cache20/mir8/abc15/binding4/vertical3/case5、r23a close-controls 6/6、rustfmt 通过；`resolve_with` 重构为 `resolve_detailed` 共享体，行为面由上述全套绿证。
