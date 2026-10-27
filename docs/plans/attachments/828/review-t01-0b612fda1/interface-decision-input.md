# PLAN-828 T-01 接口决策（D-01..03）与双函数真实竖切

冻结提交：`plan-828-dev@0b612fda1`（T-01 修复轮 r3，仅 C05；前序 `de4cba702` / `23a6b35a0` / `3e60af2c8`，基面 `47894c2e6`，同树 `D:/autostack/.wt/lang-828/auto-lang`）。
本文按 §5.6 记录 T-01 的有界决策、真实证据与剩余边界；未回答条目将阻止依赖它的广泛改造。**已获独立复审闭合：T01-C01/C02/C03/C04（de4cba702 复审）；T01-C05 已按其残项指示修复，待冻结根受影响面复审。**

## 连续不通过轮次的根因诊断（各轮先行对账，均已写入后落实）

- **C04（授信边界失效，23a6b35a0 复审提出）**：T01-C01 修复把共享 binder 的 function schema 放宽（`require: [name, body]`→`require: [name]`），因为单 body 接缝需要无 body callee 声明能**绑定**。但完整 verifier 的 ownership 走查（verify.rs `check_module`）仍依赖"binder 已拒绝 dangling body"这一前提——前提失效后 `bodies.get(u32::MAX)` 落空即 `continue`，缺 body 声明穿过完整校验领到 CheckedModule，native 按 def 索引 body 越界 panic。教训：只验证了局部路径（`verify_function` 对目标 body 已有越界拒绝），没有在前提改变后重审**共享授信边界**。修复原则：凭证边界自带规则、不继承 binder 前提。**de4cba702 复审：CLOSED**（五 profile+混合/未调用+成对边界）。
- **C05 第一轮（环境/候选的真实持有与发布点，23a6b35a0 复审提出）**：`recheck_function` 成功即把候选换入 env 槽位并**丢弃**被置换条目——"current"没有第二持有者，后置 verify/lower/link 拒绝时无从恢复；决策文档又把两种互斥语义并列为 T-02 可选项。真实发布点：session 的 root/source/revision 由既有 commit 单点发布（r23a 已闭）；env 槽位是第二个发布点，必须同样一次发布。
- **C05 第二轮（de4cba702 复审提出，本轮修复）**：八行事务控制映射出的真实缺陷——prepare 把候选 graft 进 env 后只返回旧条目 save：**成功 prepare 即已发布**（commit 前 env 就能发射候选 body）；**drop 不恢复**（候选留在 env）；**commit 不安装**（丢弃 save 即返回 Ok，交错时"提交成功"实际 body 是旧的）；**身份=名字+槽号**（同名同槽跨环境处置被接受）。根因：把"保存旧条目"当成了"不提前发布"——被保存的 current 不是公开可消费的当前环境，发布时点也不由 commit 控制。教训：候选必须自持新条目与 owner/基面，发布只存在于 commit 安装这一处。

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

| §5.7 行 | 代码对应（0b612fda1） |
|---|---|
| Open session | `session.rs` `current/fns(+history)/env/context_revision`；prepare/失败/取消/迟到/foreign 不改 root（r23a close-controls 全绿保持） |
| PreparedEdit | `prepare_edit` 产出、外部持有至 commit/cancel/drop；closed 拒绝亦消费（r23a 证据） |
| ProgramSnapshot | `Arc<ProgramSnapshot>`；close 撤销 session 拥有引用→vacant 根（r23a F05b-C01）；无反向强持有 |
| PausedRun/帧 | `execute_program_steps/resume_program`；close/drop session 后旧链完成/取消（r23a close-adjacent） |
| Closed session | `close()/closed()`；run=Infra、prepare/edit=Err、commit/cancel=拒、无 owner 资源 close 即死 |
| store/lease | `persist_all_units/restore`+unit-span sidecar（F-10/F-11 已闭部分）；GC/lease/水位仍 OPEN |

〔T01-C05 修复·最终模型（0b612fda1，取代 de4cba702 的 save 模型）〕**候选所有权与发布点**：
(a) **候选自持**：`FnCandidate` 拥有新 body 条目（move，非 clone）、所属环境身份 `env_id`（进程唯一计数器——同名同槽不是环境身份）、prepare 时基面 `base_generation`（env 每次成功 commit +1）、槽位身份，以及候选自身冻结产物（`parts()`/`receipt()`——单 body 文档即候选工件）。事务全部有界状态=一个被 move 的条目+工件；`CheckedEnv` 无 Clone、全 env 深clone 禁止保持。
(b) **prepare 不发布**：`prepare_function_candidate` 在独占访问内做有界临时 graft（真单 body resolve/typecheck/emit/verify 都对着候选运行），**返回前恢复 current、候选随对象离开环境**。成功 prepare 后 env 逐字节服务已提交 current；任一阶段失败先恢复 current 再返回诊断（后置 verify 拒绝保持 current 的既有控制保留）。
(c) **commit=唯一发布点**：按 `closed → owner(env_id) → 基面(generation) → 槽位身份` 校验后**安装自己持有的候选**并 `generation+=1`；过期基面=受控 `source.stale` 拒绝（旧候选不覆盖新结果，§5.4），跨环境=受控 `source.foreign` 拒绝（不修改任一环境）。session 在其单一提交点与 root/source/revision 一同调用（T-02/T-06 接线）。
(d) **取消/丢弃零 env 访问**：`cancel(self)` 与 drop 均不取环境引用——未提交候选**构造上不可能**修改任何环境的当前语义；迟到取消无回滚可写。
(e) **close 显式释放**：`CheckedEnv::close()` 清 fns/records、置 `closed`、幂等；close 后 recheck/emit/prepare/commit 均受控 `source.closed` 拒绝（cancel/drop 无 env 访问、恒安全）；保留 sigs/rec_index/profile/计数/generation 供关闭后诊断。预算照计 F-09。
已删除 de4cba702 引入的 `FnSlotSave`/`recheck_transactional`（前一提交新增、无其他消费者）；`recheck_function` 保留其文档化的原地原语语义（复审确认为合法示范原语）。
**永久断言（`contract_828_transaction` 6 项，收编 de4cba702 复审 transaction-controls.rs 的八行控制——同源码、同期望状态，适配新 API）**：①成功 prepare 不发布+drop 惰性（候选工件=新 body、env current=基面）；②顺序取消→提交正控（取消保基面、提交恰发布自己+generation=1）；③双候选同基面（取消 A 提交 B→B）+迟到取消不回卷+过期基面 commit 受控拒（`source.stale`）；④同名同槽跨环境 commit 受控拒（`source.foreign`）且两个环境 current 均不变（取消无 env 引用、结构上不可触他环境）；⑤后置 verify 拒绝保持 current；⑥close 释放+commit/recheck/emit 拒绝+幂等。
**T-02/T-06 保留面**：session 的 root/source/revision/来源与环境槽位的**原子同点发布**接线、PreparedEdit 与 FnCandidate 的合并形态、迟到/foreign/closed 完整矩阵、三入口派发一致性——owner 不变，本接缝只交付其可依赖的最小事务前置。

F-01/F-02 改动后的重放集合：r23a close-controls/close-adjacent（9/9）、62 例会话重放、r1 套件（86）+ contract（16）+ 兼容面；F-05b 行为按受影响面重放，不据旧 PASS 覆盖新代码。

## 测试选择表（新目标）

| 目标 | filter | 现数 |
|---|---|---|
| probe/tests/function_incremental_828_contract.rs | `contract_828_frontend` | 4 |
| 同上 | `contract_828_dispatch` | 3（identity_axes/unit_view_first_case/real_link_rejections） |
| 同上 | `contract_828_admission`（T01-C04 永久断言） | 3 |
| 同上 | `contract_828_transaction`（T01-C05 永久断言，收编八行控制+stale 基面） | 6 |
| probe/tests/function_incremental_828_contract_t01r.rs（收编 review-t01 frontend-controls；C02 控制采用生产者窗口门） | `t01_review_` | 4 |
| 既有不变面 | `function_incremental_828_r1`（86）/`function_incremental_828`（17）/core（22） | — |
| ac-core 内部（source 重构面） | `cargo test --manifest-path experimental/ac-core/Cargo.toml` 全 crate + 六 profile 目标 | — |

新公共 API（ac-core）：`source::{resolve_env/recheck_function/prepare_function_candidate/FnCandidate/CheckedEnv{close,closed,env_id,generation}/emit_function_body 已有}`、`body_work::{capture_body_work/BodyWork}`、verify 新诊断码 `verify.missing-body`/`source.closed`/`source.foreign`/`source.stale`；（probe core）：`ProgramSnapshot::dispatched_payload_same_object/same_content/unit_view/unit_binding_complete`、`UnitView`、`av::execute_program_units`。`FnSig` 为 pub(crate) 与 CheckedEnv.sigs 字段可见性一致，ac-core 0 警告。schema/授权/资源接缝无变化（F-09 预算轴不因此增减）。

## 回归面（本轮实测，全部定向）

boundary 独立控制 11/11、admission 独立控制 2/2（de4cba702 复审新增件原样重放）、r23a 关闭原样 9/9、contract 16/16（事务 6 项全部按新 API 通过）、t01r 4/4、主 17/17、r1 86/86、core 22/22、ac-core 全 crate+六 profile 目标全绿（**名字/计数以复审 raw 为准：hir_verify 9 / record_hir 21 / scalar_hir 16 / source_frontend 14 / text_hir 16 / text_record_hir 18**）、scalar_projection_ 6/6、rustfmt 通过、ac-core 0 新警告（probe 4 旧 warning+shim 1 旧 warning 为既有，不算新增）。de4cba702 复审的 transaction-controls.rs 八行控制已按同输入/同期望状态收编为 `contract_828_transaction` 六项永久断言（冻结原件保留于复审包；其中"foreign restore"行在新 API 无对应入口——取消已无 env 引用，其语义由 foreign commit 拒绝+双环境不变断言承载）。
