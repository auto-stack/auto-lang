# PLAN-828 T-01 接口决策（D-01..03）与双函数真实竖切

冻结提交：`plan-828-dev@de4cba702`（T-01 修复轮 r2；前序 `23a6b35a0` / `3e60af2c8`，基面 `47894c2e6`，同树 `D:/autostack/.wt/lang-828/auto-lang`）。
本文按 §5.6 记录 T-01 的有界决策、真实证据与剩余边界；未回答条目将阻止依赖它的广泛改造。r3 T-01 复审五 finding（T01-C01..C05）已全部闭合，见各节〔修复〕标注。

## 连续两轮不通过的根因诊断（23a6b35a0 复审裁定，本轮先行对账）

- **C04（授信边界失效）**：T01-C01 修复把共享 binder 的 function schema 放宽（`require: [name, body]`→`require: [name]`），因为单 body 接缝需要无 body callee 声明能**绑定**。但完整 verifier 的 ownership 走查（verify.rs `check_module`）仍依赖"binder 已拒绝 dangling body"这一前提——前提失效后 `bodies.get(u32::MAX)` 落空即 `continue`，缺 body 声明穿过完整校验领到 CheckedModule，native 按 def 索引 body 越界 panic。教训：只验证了局部路径（`verify_function` 对目标 body 已有越界拒绝），没有在前提改变后重审**共享授信边界**。修复原则：凭证边界自带规则、不继承 binder 前提——完整模块凭证在自身走查内拒绝缺 body；binder 的放宽只服务函数级文档。
- **C05（环境/候选的真实持有与发布点）**：`recheck_function` 成功即把候选换入 env 槽位并**丢弃**被置换条目——"current"没有第二持有者，后置 verify/lower/link 拒绝时无从恢复；决策文档又把"保留换入槽位"与"影子槽回滚"并列为 T-02 可选项，与 §5.4/§5.7"仅成功提交替换当前语义"冲突。真实发布点：session 的 root/source/revision 由既有 commit 单点发布（r23a 已闭）；env 槽位是第二个发布点，必须同样一次发布。修复原则：被置换条目成为有界 save（move，非 clone），save 的处置即事务边界。

## D-01 受检环境与单 body 检查（F-01 前端接缝）

**决策**：受检环境 = 冷编 `resolve_env` 保留的 `CheckedEnv{program: ResolvedProgram, rec_index, sigs, trace_enabled, profile, resolve/check_walks, closed}`（ac-core `source/resolve.rs`）。单 body 检查 = `recheck_function(env, item, fn_index)`：签名漂移守卫（`fn_sig_of` 按冷编同一规则重建签名，任何差异=受控拒绝 `source.incremental`）→ 真 `resolve_fn`（环境 sigs/records 供全部名字查询）→ O(1) 槽位换入 → `check_body_in` 只查该 body（自递归见新条目；失败回滚旧条目）。发射 = `emit_function_body(env, fn_index, src)`：Emitter 对改函做真 body 计划，对其余函数做**无 body 声明脚手架**（types 段=全函数接口目录，declarations 段=改函带 body+其余无 body 声明，bodies 段仅改函），全环境供 callee/类型查询；单文档经同一 render/bind/reanchor 规则产出。

**关键发现（竖切实测）**：单 def 文档的跨函数 DefRef 要求 callee def 存在。
〔T01-C01 修复〕callee 以**无 body 声明脚手架**满足存在性：五个 profile schema 的 `function` 声明 `body` 放宽为可选（向后兼容——既有文档全部带 body），binder 对无 body 声明产出 `DefKind::Function{body: BodyId(u32::MAX) 哨兵}`。热路径发射**恰好一个 body 文档**；未改函数零 plan/零 body 渲染/零 bind。生产者产物即证据：`bundle.modules[0].bodies.len()==1`、receipt `real_body_plans:1, scaffold_fns:N-1`。
〔T01-C02 修复〕结构门挂到**真实生产者**：ac-core `body_work::capture_body_work` 观测窗，`record_resolve/typecheck/plan/bind` 分别挂在 resolve_detailed/recheck 的 resolve_fn 调用点、check_block 入口、plan_fn 入口、binder bind_body——按函数名记账，接口目录（types/无 body 声明）不计。门谓词：窗口内改函恰 +1/+1/+1/+1 且**其他函数全零**。真实旧 synth 管线在同一门下红；旧计数门（env_walk_stats+receipt）保留为第二轴。
〔T01-C04 修复〕**完整模块准入恢复**：`check_module` 的 function→body ownership 走查对缺 body/dangling def 发 `verify.missing-body` 受控拒绝——哨兵或缺引用的 def 拿不到整模块 CheckedModule，native 越界不可达。覆盖五 profile 缺 body 全文、混合文档（真 body+无 body callee）、未被调用的无 body 声明；不做 native 吞 panic/只查 entry/补空 body/恢复全 stub。函数级合法性保持：单 body 文档经 `verify_function`（目标 body 越界本就拒绝、callee 只读类型）依旧通过——同一放宽在函数级合法、在完整凭证拒绝，永久成对断言（`contract_828_admission_*`，含 pre-repair descriptor 拒绝对照与合法完整文档对照，旧 schema 冻结于 `probe/tests/fixtures_828/`）。

**证据（probe/tests/function_incremental_828_contract.rs）**：
- `contract_828_frontend`（4）：C01-01 单 body HIR 与冷编 canonical dump 逐字节相等+计数 3/3；C01-04 真类型错拒绝且环境不中毒；签名漂移受控拒；caller 条目保留可复现。（旧文"5/5"为笔误，据 raw 纠正为 4。）
- `contract_828_admission`（3）：五 profile 缺 body 拒绝+旧 descriptor 对照+合法对照；混合/未调用拒绝；脚手架函数级合法+同 bundle 完整凭证拒绝（成对边界）。

**边界与 T-02 责任**：scalar 自由函数面已接（receiver/method 路径仍整编）；session 热路径仍走 synth 桩管线（T-02 换接 `resolve_env/recheck_function/emit_function_body` 并绑定观测）；`stubify`/`synth_stub_walks` 计数随 T-02 移除。HIR 构造边界不变：仍经 Emitter→Atom→bind（未手写 HIR）；A07 函数 HIR 产物/恢复归 T-03/T-07（本接缝产出未验证 `SourceParts`，语义验证走 `verify_function`/完整 verify 入口）。

## D-02 无 merged 派发（F-02 执行载荷）

**决策**：T-05/06 实现方向冻结为——链接期计算**绑定表**（def 名→函数槽、每单元局部 const/type 索引→快照级interned解析、import 调用→槽位绑定），AV 三入口（`execute_program/execute_program_steps/resume_program`）共用同一 dispatch，经单元视图直接消费 `FunctionCode` 的不可变 `MirFunction`（局部 PC/槽/常量按单元局部索引+表解析），**不重建 merged 载荷、不重写未改指令**。持久 codec/公共接缝在 T-01 不变（观察 API 纯新增）。

**实测现状（F-02 量化，producer 报告）**：`ProgramSnapshot::dispatched_payload_same_object/same_content`——未改函数的派发载荷今天**对象身份红**（每快照 merged 副本），且竖切发现**内容也漂移**：改函增长使 merged 类型/常量空间重索引；**单元载荷**（不可变 FunctionCode 内的 MirFunction）内容恒等。代码级身份轴绿：未改函数跨快照同一 `Arc<FunctionCode>`；强制同内容副本结果仍正确但身份门红（§5.8 C02-05 雏形）。

**〔D-02 前置已补〕实际消费接缝与双函数首例**：`av.rs` `ModuleView` trait（fn_at/const_at/zero_of/type_is_bool/resolve_callee/entry）——三公开入口与单元原型共用同一 `dispatch`（MergedView=今行为逐点保持）。`ProgramSnapshot::unit_view()`/`UnitView` 按单元局部索引消费各函数**原始不可变 MirFunction**；`execute_program_units` 原型入口双/三函数程序结果正确且执行路径消费快照不可变载荷；`unit_binding_complete()` 为绑定拒绝控制。**原型的真实边界**：UnitView 当前在**派发时按 import 名查槽**（`fn:<name>`→单元槽位），链接期预计算绑定表仍是 T-05/06 的冻结方向——原型验证的是"单元局部载荷可直接派发"这一可行性，不是最终绑定机制；默认三入口仍走 MergedView，`execute_program_units` 是原型入口而非默认路径。**证据**：`contract_828_dispatch`（3）：identity_axes（身份轴绿/副本负控红/派发轴如实红）+ unit_view_first_case（本轮修正：基线在 edit **前**捕获，跨修订真实身份+copy 实际执行负控）+ real_link_rejections（真实 link 构造点缺 target/错签名拒绝——公开绑定唯一产源）；`t01r` 收编 4/4。merged 派发轴当前红仍是 F-02 的量化现状（T-05/06 换默认路径后翻绿）。

## D-03 所有权对应（§5.7 表 ↔ 当前代码）

| §5.7 行 | 代码对应（de4cba702） |
|---|---|
| Open session | `session.rs` `current/fns(+history)/env/context_revision`；prepare/失败/取消/迟到/foreign 不改 root（r23a close-controls 全绿保持） |
| PreparedEdit | `prepare_edit` 产出、外部持有至 commit/cancel/drop；closed 拒绝亦消费（r23a 证据） |
| ProgramSnapshot | `Arc<ProgramSnapshot>`；close 撤销 session 拥有引用→vacant 根（r23a F05b-C01）；无反向强持有 |
| PausedRun/帧 | `execute_program_steps/resume_program`；close/drop session 后旧链完成/取消（r23a close-adjacent） |
| Closed session | `close()/closed()`；run=Infra、prepare/edit=Err、commit/cancel=拒、无 owner 资源 close 即死 |
| store/lease | `persist_all_units/restore`+unit-span sidecar（F-10/F-11 已闭部分）；GC/lease/水位仍 OPEN |

〔T01-C05 修复〕**D-03 冻结唯一事务机制（撤销"两语义都不提前发布"的可选并列）**：有界**单槽事务**——
(a) `recheck_swap`（`recheck_function` 的核心）成功时把被置换的 current 条目 **move** 进 `FnSlotSave`（每打开事务恰一个 `ResolvedFn`，`CheckedEnv` 无 Clone、禁止全 env 深clone 的约束保持）；候选 graft 在 env 槽位，"current"唯一持有者即该 save。`recheck_function` 保持旧语义（丢弃 save=即发布），供已验收的冷/recheck 路径；新事务入口为 `recheck_transactional`。
(b) `prepare_function_candidate(env, item, fn_index, src, path)`：recheck→emit→`verify_function` 单事务；**任一阶段失败先 restore 再返回诊断**——后置 verify 拒绝不再保留换入槽位（复审示范原语 `boundary_post_recheck_verify_rejection_leaves_mutated_env_demonstration` 所证缺陷就此闭合，永久断言 `contract_828_transaction_post_verify_rejection_keeps_current`：拒绝后同 env 发射原 body=冷编原文且 verify 过）。
(c) 处置显式：`save.restore(env)`=取消/回滚（身份守卫：槽名不符=foreign 拒绝；closed=拒绝）；`save.commit(env)`=一次发布（closed=拒绝）。session 侧在**同一提交点**调用 commit 与 root/source/revision 发布（T-02/T-06 接线）；迟到/foreign/closed 完整矩阵、PreparedEdit 与 save 的合并形态仍归 T-02/T-06 原 owner。
(d) `CheckedEnv::close()`：显式释放 env 拥有的 body 条目（fns/records 清空、`closed` 置位、幂等），不以"字段在 session 内"推导 drop 即释放；close 后 recheck/emit/prepare/commit/restore 均受控 `source.closed` 拒绝；保留 sigs/rec_index/profile/计数供关闭后诊断。最小首例：`contract_828_transaction_cancel_then_continue`（同基面双候选：A 取消→current 保持冷编原文、B 提交→发布 B；foreign restore 拒绝）、`contract_828_transaction_close_releases_owned_entries`（close 释放+全入口拒绝+幂等）。预算照计 F-09。

F-01/F-02 改动后的重放集合：r23a close-controls/close-adjacent（9/9）、62 例会话重放、r1 套件（86）+ contract（13）+ 兼容面；F-05b 行为按受影响面重放，不据旧 PASS 覆盖新代码。

## 测试选择表（新目标）

| 目标 | filter | 现数 |
|---|---|---|
| probe/tests/function_incremental_828_contract.rs | `contract_828_frontend` | 4 |
| 同上 | `contract_828_dispatch` | 3（identity_axes/unit_view_first_case/real_link_rejections） |
| 同上 | `contract_828_admission`（T01-C04 永久断言） | 3 |
| 同上 | `contract_828_transaction`（T01-C05 最小首例） | 3 |
| probe/tests/function_incremental_828_contract_t01r.rs（收编 review-t01 frontend-controls；C02 控制采用生产者窗口门） | `t01_review_` | 4 |
| 既有不变面 | `function_incremental_828_r1`（86）/`function_incremental_828`（17）/core（22） | — |
| ac-core 内部（source 重构面） | `cargo test --manifest-path experimental/ac-core/Cargo.toml`（六 profile 目标 94 测）+ ac-syntax `scalar_projection_`（6） | — |

新公共 API（ac-core）：`source::{resolve_env/recheck_function/recheck_transactional/FnSlotSave/prepare_function_candidate/CheckedEnv{close,closed}/verify_function 已有}`、`body_work::{capture_body_work/BodyWork}`、verify 新诊断码 `verify.missing-body`/`source.closed`；（probe core）：`ProgramSnapshot::dispatched_payload_same_object/same_content/unit_view/unit_binding_complete`、`UnitView`、`av::execute_program_units`。`FnSig` 为 pub(crate) 与 CheckedEnv.sigs 字段可见性一致，ac-core 0 警告。schema/授权/资源接缝无变化（F-09 预算轴不因此增减）。

## 回归面（本轮实测，全部定向）

boundary 独立控制 11/11（五缺 body 反例红→绿；D-03 示范原语 PASS）、r23a 关闭原样 9/9、contract 13/13、t01r 4/4、主 17/17、r1 86/86、core 22/22、ac-core 全 crate+六 profile 目标（hir9/scalar21/text16/record14/text_record16/source_frontend18）全绿、scalar_projection 6/6、rustfmt 通过、ac-core 0 新警告（probe 4 旧 warning+shim 1 旧 warning 为 23a6b35a0 既有，不算新增）。
