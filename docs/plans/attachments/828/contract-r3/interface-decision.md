# PLAN-828 T-01 接口决策（D-01..03）与双函数真实竖切

冻结提交：`plan-828-dev@27aaa34d5`（T-01 修复轮 r4，发布代际联动残项；前序 `0b612fda1` / `de4cba702` / `23a6b35a0` / `3e60af2c8`，基面 `47894c2e6`，同树 `D:/autostack/.wt/lang-828/auto-lang`）。
本文按 §5.6 记录 T-01 的有界决策、真实证据与剩余边界；未回答条目将阻止依赖它的广泛改造。**已获独立复审闭合：T01-C01/C02/C03/C04（de4cba702 复审）；T01-C05 候选所有权模型获 0b612fda1 复审确认（原八行适配 8/8、上轮六失败全闭），其唯一残项（公开 recheck 发布未参与基面失效）已按窄修复指示闭合，待冻结根受影响面复审。**

## 连续不通过轮次的根因诊断（各轮先行对账，均已写入后落实）

- **C04（授信边界失效，23a6b35a0 复审提出）**：T01-C01 修复把共享 binder 的 function schema 放宽（`require: [name, body]`→`require: [name]`），因为单 body 接缝需要无 body callee 声明能**绑定**。但完整 verifier 的 ownership 走查（verify.rs `check_module`）仍依赖"binder 已拒绝 dangling body"这一前提——前提失效后 `bodies.get(u32::MAX)` 落空即 `continue`，缺 body 声明穿过完整校验领到 CheckedModule，native 按 def 索引 body 越界 panic。教训：只验证了局部路径（`verify_function` 对目标 body 已有越界拒绝），没有在前提改变后重审**共享授信边界**。修复原则：凭证边界自带规则、不继承 binder 前提。**de4cba702 复审：CLOSED**（五 profile+混合/未调用+成对边界）。
- **C05 第一轮（环境/候选的真实持有与发布点，23a6b35a0 复审提出）**：`recheck_function` 成功即把候选换入 env 槽位并**丢弃**被置换条目——"current"没有第二持有者，后置 verify/lower/link 拒绝时无从恢复；决策文档又把两种互斥语义并列为 T-02 可选项。真实发布点：session 的 root/source/revision 由既有 commit 单点发布（r23a 已闭）；env 槽位是第二个发布点，必须同样一次发布。
- **C05 第二轮（de4cba702 复审提出，0b612fda1 修复并获复审确认）**：八行事务控制映射出的真实缺陷——prepare 把候选 graft 进 env 后只返回旧条目 save：**成功 prepare 即已发布**（commit 前 env 就能发射候选 body）；**drop 不恢复**（候选留在 env）；**commit 不安装**（丢弃 save 即返回 Ok，交错时"提交成功"实际 body 是旧的）；**身份=名字+槽号**（同名同槽跨环境处置被接受）。根因：把"保存旧条目"当成了"不提前发布"——被保存的 current 不是公开可消费的当前环境，发布时点也不由 commit 控制。教训：候选必须自持新条目与 owner/基面，发布只存在于 commit 安装这一处。
- **C05 第三轮（0b612fda1 复审提出，本轮闭合）**：代际只绑定 `FnCandidate::commit` 的调用次数，未绑定**所有真实发布**——公开 `recheck_function` 成功即原地替换当前 body 却不推进 generation，旧候选仍通过基面检查覆盖新结果（复审反例：1 →prepare 2(base0)→ 公开 recheck 发布 3 → 旧候选 commit Ok 覆盖回 2）。根因：失效凭据必须挂在"成功对外发布当前语义"这一事实上，而非某个方法的调用计数。**修复（27aaa34d5）**：`recheck_function` **成功时** `generation+=1`——与 candidate commit 同属真实发布入口，参与统一基面失效；`recheck_swap`（prepare 共用的有界临时换入、返回前恢复）刻意不推进，失败 recheck 不发布，cancel/drop/close 不变。已盘点 source 内对当前函数槽位的对外发布入口恰为这两条。

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

| §5.7 行 | 代码对应（27aaa34d5） |
|---|---|
| Open session | `session.rs` `current/fns(+history)/env/context_revision`；prepare/失败/取消/迟到/foreign 不改 root（r23a close-controls 全绿保持） |
| PreparedEdit | `prepare_edit` 产出、外部持有至 commit/cancel/drop；closed 拒绝亦消费（r23a 证据） |
| ProgramSnapshot | `Arc<ProgramSnapshot>`；close 撤销 session 拥有引用→vacant 根（r23a F05b-C01）；无反向强持有 |
| PausedRun/帧 | `execute_program_steps/resume_program`；close/drop session 后旧链完成/取消（r23a close-adjacent） |
| Closed session | `close()/closed()`；run=Infra、prepare/edit=Err、commit/cancel=拒、无 owner 资源 close 即死 |
| store/lease | `persist_all_units/restore`+unit-span sidecar（F-10/F-11 已闭部分）；GC/lease/水位仍 OPEN |

〔T01-C05 修复·最终模型（0b612fda1 + 27aaa34d5 发布联动补全，取代 de4cba702 的 save 模型）〕**候选所有权与发布点**：
(a) **候选自持**：`FnCandidate` 拥有新 body 条目（move，非 clone）、所属环境身份 `env_id`（进程唯一计数器——同名同槽不是环境身份）、prepare 时基面 `base_generation`（由**每次成功对外发布**推进：candidate commit 与公开原地 `recheck_function` 成功；prepare 的临时换入/失败 recheck/取消/丢弃不推进）、槽位身份，以及候选自身冻结产物（`parts()`/`receipt()`——单 body 文档即候选工件）。事务全部有界状态=一个被 move 的条目+工件；`CheckedEnv` 无 Clone、全 env 深clone 禁止保持。
(b) **prepare 不发布**：`prepare_function_candidate` 在独占访问内做有界临时 graft（真单 body resolve/typecheck/emit/verify 都对着候选运行），**返回前恢复 current、候选随对象离开环境**。成功 prepare 后 env 逐字节服务已提交 current；任一阶段失败先恢复 current 再返回诊断（后置 verify 拒绝保持 current 的既有控制保留）。
(c) **commit=候选发布点**：按 `closed → owner(env_id) → 基面(generation) → 槽位身份` 校验后**安装自己持有的候选**并 `generation+=1`；过期基面=受控 `source.stale` 拒绝（旧候选不覆盖新结果，§5.4——该失效对**两个公开发布入口**统一生效：经 commit 发布或经公开 recheck 原地发布的新当前都使旧候选过期），跨环境=受控 `source.foreign` 拒绝（不修改任一环境）。session 在其单一提交点与 root/source/revision 一同调用（T-02/T-06 接线）。
(d) **取消/丢弃零 env 访问**：`cancel(self)` 与 drop 均不取环境引用——未提交候选**构造上不可能**修改任何环境的当前语义；迟到取消无回滚可写。
(e) **close 显式释放**：`CheckedEnv::close()` 清 fns/records、置 `closed`、幂等；close 后 recheck/emit/prepare/commit 均受控 `source.closed` 拒绝（cancel/drop 无 env 访问、恒安全）；保留 sigs/rec_index/profile/计数/generation 供关闭后诊断。预算照计 F-09。
已删除 de4cba702 引入的 `FnSlotSave`/`recheck_transactional`（前一提交新增、无其他消费者）；`recheck_function` 保留其文档化原地原语语义（复审确认为合法示范原语），其成功发布自 27aaa34d5 起参与统一基面失效。
**永久断言（`contract_828_transaction` 8 项）**：①–⑥收编 de4cba702 复审 transaction-controls.rs 的八行控制（同源码、同期望状态，适配新 API）——成功 prepare 不发布+drop 惰性、顺序取消→提交正控、双候选同基面（取消 A 提交 B→B）+迟到取消不回卷+过期基面 commit 受控拒、同名同槽跨环境 commit 受控拒且双环境不变（"foreign restore"行在新 API 无对应入口——取消无 env 引用，语义由该断言承载）、后置 verify 拒绝保持 current、close 释放+全入口拒绝；⑦⑧收编 0b612fda1 复审 publisher-controls.rs——公开 recheck 发布使旧候选 `source.stale` 过期且新 current 保持（generation 推进），失败 recheck 不发布、待提交候选仍合法提交。
**T-02/T-06 保留面**：session 的 root/source/revision/来源与环境槽位的**原子同点发布**接线、PreparedEdit 与 FnCandidate 的合并形态、迟到/foreign/closed 完整矩阵、三入口派发一致性——owner 不变，本接缝只交付其可依赖的最小事务前置。

F-01/F-02 改动后的重放集合：r23a close-controls/close-adjacent（9/9）、62 例会话重放、r1 套件（86）+ contract（16）+ 兼容面；F-05b 行为按受影响面重放，不据旧 PASS 覆盖新代码。

## T-02 生产 session 热路径换接（85cbebea4，待受影响面复审）

**生产调用链（实施后）**：`warmup` = front_door → `compile_scalar_source`（冷，唯一整模块编译）→ lower/split/admit → ac-syntax 驻留态 + **`resolve_env` 保留 `CheckedEnv`**（槽序与声明序在 warmup 断言绑定；recheck 自带名字守卫为运行时后备）。`prepare_edit` = 快照编辑 → `incremental_reparse` → 单 decl 检测 → `project_decl_strict`（仅改函，增量 CST 投影进生产消费）→ **`prepare_candidate_document(env_core, item, idx, src)`**（recheck 真单 body resolve/typecheck + 单 body 发射无 body 脚手架；返回前恢复 current）→ FnEnv 接口门（防纵深保留）→ content-version → **hit=复用已验对（零重验证）** / **miss=`verify_function`+credential 门 → `lower_and_admit`**。`commit` = 守卫 → span/origin → `link_with_maps` → **`candidate.commit(env_core)` 与 root/source/revision 同点发布** → history/persist。`close` 显式 `CheckedEnv::close`。**热路径 synth 全 Program/stubify/stub_item/synth_stub_walks 已删除**（冷初始化与显式非快路径不变）。

**新增 API**：`prepare_candidate_document`（recheck+emit 事务、**不含验证**——生产消费者在"内容命中零重验证复用"与"miss 全链 verify/lower/admit"间自决；`prepare_function_candidate` = 其 + verify，T-01 合同不变）；`env_fn_names`（warmup 槽序绑定）。**hit 语义保持**：内容命中复用存储已验 (body, code) 对，零重验证/零重 lower/零重 admit（t04 契约不削弱）；verify_hir 计数移至 miss 路径调用方。

**接缝修复（换接实证发现）**：scoped 单 body 发射中，非目标函数的接口目录类型与无 body 脚手架声明携带保留环境自身源（旧源）的 span——源长度变化后 reanchor 越界（"span escapes the source"）。等长编辑（既有合同测试）未暴露，r1 编辑矩阵（间距/重写）击中。修复：scoped 模式下非目标函数的接口/脚手架 span 置零（惰性元数据、锚点不在当前源、从不消费——与旧零 span stub 同理）；整模块路径逐字节不变。

**永久断言（`contract_828_hotpath` 3）**：①真实 `s.edit()` 在生产者 body_work 窗口内单 body（改函恰 +1×4、其余全零；外层 stage/whole_module 计数零；synth_stub_walks 不存在；值真安装）——主 session 入口的接线证明，非仅直接调用接缝的测试；②同窗强制执行旧全 stub 管线（parse+清 stmt+`compile_program_parts`，planned_fns=3）同门红——区分性负控保留；③prepare 未提交保旧语义 + close 后 commit 拒 + revision 不动（T-02 新增/替换入口的真实成功/拒绝首例；完整迟到/foreign/closed 矩阵仍归 T-06）。**支持面**：非首位/递归/稳定 caller/bool/短路/条件 for/命名实参/intrinsic 由 r1 86+t01r+session17 在新热路径上全绿保持，未改任何语言规则。

## T-03 A07 函数级 HIR 工件层 + T-04 证据映射（4a083d53c，第一批待审）

**T-03 交付**：函数 HIR 工件作为新 A07 函数单元层接入共享 ManagedStore（`ac828/a07-fnhir/v1`——与 783 模块级 A07 显式版本隔离，不共享键/payload）。`fn_hir_codec.rs` 有界 typed-binary codec 覆盖 scalar 增量 body 面：全部长度/计数/索引受界检查，面外构造或超 cap=编解码 Err（永不 panic）。payload=typed body+依赖事实（无 span 内容版本、span 布局摘要、声明上下文指纹、caps 指纹、实际引用 intrinsic 符号）；**verifier 凭证不落盘**。
**生产**：`persist_all_units` 按函数内容寻址存放；`persisted_hir_cv` 追踪使会话**仅在新内容时产生新行**（`hir_artifacts_deposited` 只计新产物：warmup=2、每次编辑恰 +1）。**恢复消费（HIR 的真实 cold-vs-warm 分界）**：每行都是**不可信描述**——须通过有界 codec 解码 AND 在当前上下文重验（def 身份/caps/上下文指纹/经同一类型表 dump 的 canonical 内容等于新鲜冷编真值/span 布局摘要）后才成为 resident+history 对象（`restore_hir_adopted`）；任何 miss/不匹配跳过该工件（冷 body 本就正确，跳过只损失溯源不损失正确性；ABC 准入不受影响）。
**永久断言（`t03_a07_*` 2 项）**：①roundtrip——warmup 存 2、编辑恰 +1（仅变更单元产生新产物）、restore 采纳 2、后续内容命中按**身份**复用工件采纳的 body 对象；②篡改控制——同键预写垃圾行（store 无重写、会话自身存放冲突被容忍）后 restore 解码失败跳过（adopted=1、冷 body 继续服务、ABC 不受影响）+ codec 边界负控（空/坏魔数/截断）。
**V-03 依赖轴现状**：上下文指纹（声明集+签名，intrinsic 声明按体内容排除）+ intrinsic 符号清单（工件携带）+ caps 指纹（键轴）+ env_id（同名异环境拒，T-01 交付）。scalar 面无 CTE/inline，callee 只读接口——接口轴即上下文指纹。
**边界**：A07 函数 HIR 的深接（跳过恢复时的整模块 emit+bind+verify/lower——当前 restore 仍冷编前端作为校验真值源）归 T-07 恢复消费者；512 修订水位/GC 归 T-07；预算：新增驻留（cold_module+每函数 canonical dump）计入 F-09。

**T-04 证据映射（既有机器按 V/C 行核实，无新代码需求）**：单函数 lower=`hir_adapter::lower_function_unit`（恰一函数入 unit func0、其余 import；session miss 路径 `lower_and_admit` 消费，credential 门在前）；内存 hit 复用=session 内容寻址历史（`t04_content_addressed_reuse_hits_previous_revision`：hit 零重验证/重 lower/重 admit、对象身份断言）；内容历史+各自 mint 源=FnRecord.history（r1 F-828-R1-04/R3a 矩阵）；C02-02 单元局部 const/slot 独立=r1 矩阵+contract dispatch；"未改 lower/emit/decode/verify/load=0"=session t01 stage 断言。措辞边界：lower 入口取已验 bundle 文档而非 CheckedFunction 对象（credential 先行门控）——与任务语义等价，如实记录供复审核。

## 测试选择表（新目标）

| 目标 | filter | 现数 |
|---|---|---|
| probe/tests/function_incremental_828_contract.rs | `contract_828_frontend` | 4 |
| 同上 | `contract_828_dispatch` | 3（identity_axes/unit_view_first_case/real_link_rejections） |
| 同上 | `contract_828_admission`（T01-C04 永久断言） | 3 |
| 同上 | `contract_828_transaction`（T01-C05 永久断言，收编八行控制+stale 基面+发布联动两控制） | 8 |
| 同上 | `contract_828_hotpath`（T-02 主入口换接：生产者窗口单 body/旧管线同窗红/prepare-close 接线） | 3 |
| probe/tests/function_incremental_828.rs | `t03_a07_`（T-03 A07 工件 roundtrip/篡改跳过） | 2 |
| 既有不变面 | `function_incremental_828`（19，含 t03 2） | — |
| probe/tests/function_incremental_828_contract_t01r.rs（收编 review-t01 frontend-controls；C02 控制采用生产者窗口门） | `t01_review_` | 4 |
| 既有不变面 | `function_incremental_828_r1`（86）/`function_incremental_828`（17）/core（22） | — |
| ac-core 内部（source 重构面） | `cargo test --manifest-path experimental/ac-core/Cargo.toml` 全 crate + 六 profile 目标 | — |

新公共 API（ac-core）：`source::{resolve_env/recheck_function/prepare_function_candidate/FnCandidate/CheckedEnv{close,closed,env_id,generation}/emit_function_body 已有}`、`body_work::{capture_body_work/BodyWork}`、verify 新诊断码 `verify.missing-body`/`source.closed`/`source.foreign`/`source.stale`；（probe core）：`ProgramSnapshot::dispatched_payload_same_object/same_content/unit_view/unit_binding_complete`、`UnitView`、`av::execute_program_units`。`FnSig` 为 pub(crate) 与 CheckedEnv.sigs 字段可见性一致，ac-core 0 警告。schema/授权/资源接缝无变化（F-09 预算轴不因此增减）。

## 回归面（本轮实测，全部定向）

boundary 独立控制 11/11、admission 独立控制 2/2（de4cba702 复审新增件原样重放）、r23a 关闭原样 9/9、contract 18/18（事务 8 项）、t01r 4/4、主 17/17、r1 86/86、core 22/22、ac-core 全 crate+六 profile 目标全绿（**名字/计数以复审 raw 为准：hir_verify 9 / record_hir 21 / scalar_hir 16 / source_frontend 14 / text_hir 16 / text_record_hir 18**）、scalar_projection_ 6/6、rustfmt 通过、ac-core 0 新警告（probe 4 旧 warning+shim 1 旧 warning 为既有，不算新增）。复审控制件重放（同输入/同期望状态）：0b612fda1 包 publisher-controls 3/3（原失败项"公开 recheck 发布使旧候选过期"转绿）+ transaction-controls 适配版 8/8；de4cba702 包 transaction 八行按同输入/同 oracle 收编为 `contract_828_transaction` 永久断言（冻结原件保留于复审包；其中"foreign restore"行在新 API 无对应入口——取消已无 env 引用，其语义由 foreign commit 拒绝+双环境不变断言承载）。

## 第一批复审 B1-C01..C05 一批修齐（14c929a5d，待受影响面复审）

冻结提交 `14c929a5d9d54a90106b8e62b058e98e75a28b9f`（同树 plan-828-dev，基面 d93a9e373）。五项一次闭合；上文 T-03 节所述 `ac828/a07-fnhir/v1` 编码与"canonical 比对即采纳"的恢复门自本轮起被 **v2 + 全语义准入**取代（历史描述保留，以本节为准）。

### 数据流/所有权表（恢复链与新增 owner，逐格对应当前代码）

| 环节 | 事实与 owner | 当前代码 |
|---|---|---|
| 候选 body 的坐标系 | 编辑提交后 `FnRecord.hir_body` 的 callee/type 引用是**候选文档自身 def/type 表**的槽位；warmup/恢复采纳的 body 是**冷模块表**的槽位。每记录携带 `producing_defs/producing_types`（id 文本、索引对齐） | `session.rs` FnRecord 字段；commit 在 `candidate.commit(env_core)` 消耗候选前从 `candidate.parts().bundle.modules[0]` 捕获 |
| 工件保存身份/事实 | codec **v2**：body 的 callee/type 引用是**工件局部引用表**（稳定 **def/type id 文本**、首用序）的索引；表随 payload 落盘。键/层 schema=`ac828/a07-fnhir/v2`（v1 行键不可达、永不原地改写）；intrinsics 事实=经驻留 env 按 def id 文本解析的实际读取符号集；span 摘要=body 全 span FNV | `fn_hir_codec.rs`：`normalize_fn_hir_body`（产表→工件局部）/`rebind_fn_hir_body`（工件局部→当前表，按 id 文本）/`encode|decode_fn_hir`（版本=2、表长界内检查）；`persist_all_units` 归一化后存放 |
| 恢复建立当前上下文 | restore 冷编当前源 → `cold_bundle`（域/requires/模块表）+ 驻留 `FnEnv`（接口/intrinsic 事实/body facts）即"当前上下文" | `restore()` → `warmup()` 构造；`session.cold_bundle` 取代旧 `cold_module`+`intrinsic_symbols`（后者删除） |
| 校验链（谁放行） | 五门顺序：①codec v2 界内解码+header（def/cv/caps/上下文指纹）②**重绑**（稳定 id 文本→当前冷表；未解析即跳过）③canonical 内容等值（经当前冷表 dump）④**实际解码 body 的 span 摘要**==当前真值==自报字段 ⑤**真实 `verify_function`**（当前冷 bundle，body 装回其 def 槽——owner/参数/绑定等全部规则）**且**声明 intrinsics==重绑 body 实际读取集。任何一门失败跳过该工件（冷 body 保正确；每门有 `restore_hir_skip:{gate}` 计数） | `session.rs` restore 采纳循环 |
| 校验后谁拥有 body | 采纳的 body `Arc` 成为 resident+history 对象（producing 表=当前冷表）；FnRecord/PreparedEdit/history 的既有拥有关系不变 | 同上 |
| hit 如何核适用性 | 内容命中（同 cv）外，先核**body-fact 读取记录**的现时性：消费者记录的 (owner/domain/def/version) 任一失配→**拒绝命中、强制走 miss 全链重验**（`body_fact_invalidated:{fn}` 计数在真实消费点）；remint 提交清除该消费者记录（scalar 检查不读 body） | `prepare_edit` staleness 门；`query_body_fact` 记录/`FnEnv::body_fact_current` |
| close 谁释放 | close 显式释放 `cold_bundle`（含全部 owned body 深副本）与 body_reads 记录；`resident_cold_hir_nodes()` 为可观测 oracle（close 后=0，无需 drop(session)）；外部 pin/PausedRun 合法寿命不变 | `session.rs close()`；B1-C05 控制 |

### 五项关闭行（根因→受影响路径→机制变化→原反例与合法正控→永久断言→剩余边界）

- **B1-C01/P1**：根因=采纳仅比 header+canonical（canonical 省略 owner/参数 index）+自报字段，intrinsics 字段无消费。受影响=restore 采纳→resident/history→后续内容命中复用（污染沿 hit 传播）。机制=⑤门真实 `verify_function` + intrinsics 交叉校验（上表）。原反例（越界/范围内错误 owner、越界 param index、伪造 intrinsics 全被采纳 adopted=2）现全部跳过（adopted=1，skip:verify/intrinsics 各 1）；合法正控 clean 与 nonzero_owner（owner=0/1）均采纳 2；被拒函数冷回退、编辑回环不再复活坏对（ptr_eq 断言）。永久断言=`t03_a07_artifact_admission_controls`（10 模式，含"verifier 自身拒绝哪些"的前提核验）。剩余边界=无（未以特判补字段；伪造 span/literal 由 ④/③ 门关闭）。
- **B1-C02/P2**：根因=只比自报 `span_digest` 与冷摘要。机制=④门对**实际解码 body** 求摘要，双等式（==冷真值==自报字段）。原反例（body.span=999000..999010 仍采纳、坏 span 进 history 命中）现 skip:span=1；合法非零 offset/未移位 roundtrip/原 span/reanchor 矩阵 r1 86 保持。剩余边界=无。
- **B1-C03/P2**：根因=FnEnv 只有接口/声明指纹事实，无 body-fact 查询→记录→失效链。机制=生产查询核心 `FnEnv::{set_body_fact,body_fact_of,body_fact_current}` + 会话 `query_body_fact`（真实入口计数 `body_fact_query:{callee}`）+ prepare 的 staleness 门（上表 hit 行）。探针=`t03_body_fact_query_dependency_invalidation`：已读 helper body 变化→main 同内容命中被拒、强制重验（invalidated=1、reuse_hit 不动、verify_hir+1）；remint 后 helper 再变不失效；重读后改**未读** tail 不失效（hit 保持）；只读接口轴 main 零阶段工作+对象身份保持。剩余边界=scalar 检查自身不读 body（无 inline/CTE 消费者——首版语言面即如此）；未来消费者经 `query_body_fact` 接入，不自报计数。
- **B1-C04/P2**：根因=持久化用 warmup intrinsic 槽表解释候选局部 DefId、恢复又把 body 塞进冷表 dump——两张局部表从未重绑（数组槽被当跨产物身份）。机制=codec v2 稳定 id 文本引用表（上表）；键/版本隔离 v1→v2，旧行不可达不覆盖、Conflict 仍不吞。原反例（mixed mark_a/mark_b 的 main 工件存错 intrinsics=["hir.test.mark_a"]、恢复仅采纳 2/3）现：存放侧 callee 表含 mark_b def id、intrinsics=["hir.test.mark_b"]（测试直读 store 行断言）；恢复采纳 3/3、结果=5、trace=[MarkA,MarkB] 不漂移；二次恢复仍 3/3；后续编辑/回环未改函数 HIR/code Arc 身份保持；plain 正控 3/3 保持。永久断言=`t03_a07_roundtrip_plain_and_mixed_intrinsic_adoption`。剩余边界=工件语义变更已按 v2 版本隔离（不迁移 v1 行——跳过只损失溯源）。
- **B1-C05/P2**：根因=close 清 env_core/store/fns/current 但不清 T-03 新增 `cold_module` 深副本（Vec<Body> 全 owned）。机制=close 显式置空 `cold_bundle`+清 body_reads；`resident_cold_hir_nodes()`（defs+types+各 body locals/places/exprs/blocks 计数）为该 owner 的定向可观测控制：close 前>0、close 后=0（无需 drop）、幂等；外部 pin 的快照 close 后仍正确执行（V-05）；r23a 旧 close 控制 6/6+3/3 原样重放保持。永久断言=`t03_close_releases_cold_bundle_owner`。剩余边界=close 保留的轻量诊断/修订元数据为合同允许（§5.7），不声称进程内存归零。

### 新 API 与测试选择（本轮增量）

- probe 新公共 API：`fn_hir_codec::{normalize_fn_hir_body, rebind_fn_hir_body}`、`FnHirArtifact.{callee_table,type_table}`；`function_queries::{BodyFactRead, FnEnv::set_body_fact/body_fact_of/body_fact_current/intrinsic_facts_of}`；`session::{query_body_fact, resident_cold_hir_nodes}`。删除：session `intrinsic_symbols`/`cold_module`（由 `cold_bundle` 取代）。
- 新测试目标 `probe/tests/function_incremental_828_b1.rs`（filter `t03_` 4 项）：admission 10 模式（收编复审 artifact-controls 同输入同 oracle，含 skip 门计数与"verifier 前提"核验）、roundtrip plain+mixed（收编 roundtrip-controls，含二次恢复/编辑回环身份/源与 trace 不漂移/store 行直读）、body-fact 失效探针、close 释放 oracle。
- 计数器新增：`restore_hir_verified`、`restore_hir_skip:{codec|header|rebind|canonical|span|verify|intrinsics|def}`、`body_fact_query:{callee}`、`body_fact_invalidated:{fn}`。

### 回归面（14c929a5d 实测，全部定向）

session 20/20（含 512 旗舰 111s）、contract 22/22、t01r 4/4、r1 86/86、b1 4/4、`t03_a07_` 2/2、r23a close-controls 6/6 + close-adjacent 3/3 原样重放、rustfmt 0 diff、警告集=HEAD 既有（SpanInfo/history_entry/shim doc + 旧测试 2 unused-mut，零新增；`cargo check --all-targets` 对照 HEAD 核验）。ac-core 本轮零改动。整档 t/tv/tf 等 merge-owned 门禁保持 pending。

## 第一批 R2 修复轮（599cc15af，待受影响面复审）

冻结 `599cc15afee9fb964a8a88a243c2210ea0597aa2`（基面 14c929a5d，同树）。R2 复审（B1-R2-01 P1 / B1-C03 残项 / B1-R2-02 P2）三项一次闭合；上轮已闭机制（五门准入、codec v2、cold_bundle 释放、12 控制）全部保留。

### 为何旧 oracle 漏掉 / 判据调整（连续两次非通过的机制诊断）

- **旧 oracle 只测非法性，不测同内容区分力**（R2-01）：上轮 admission 控制全部用"verifier 会拒的坏 HIR"（越界 owner/index/伪造事实）检验恢复门，未用"verifier 会收、但语义不同"的合法 HIR 检验内容键。canonical 是内容身份而非准入规则——两轴混同导致 `Place.local` 漏渲染多年未暴露。本轮判据：内容编码必须覆盖 hir 语义等值（`PartialEq`）全部字段，与准入门分离论证；控制必须含"合法但不同程序"的正控（x=3→y=3 真编辑 + 冷编对照 + 错误值差）。
- **依赖模型按消费者名挂账，产物无凭据**（C03 残项）：上轮把 body_reads 做成 per-consumer 可覆盖表 + remint 清除，等价于"消费者最新一次读取代表其所有缓存产物"——重读即刷新旧产物前提、兄弟产物互相污染。本轮判据：凭据属于**具体受检产物**（history 条目）；重读=读取结果，永不刷新旧凭据；命中核**目标条目自己的**快照；只有 stale 强制重验产生的**新对**可以替换同键旧对。

### 机制（对当前代码）

| 机制 | 规则 | 代码 |
|---|---|---|
| 内容身份 can3 | canonical 渲染 hir 语义等值全字段：body id/owner（按 def id **文本**，非表槽）/entry；locals id/name/role/slot index/mutable/param_mode/type；places id/kind/**local**/type/access；exprs id/类型/节点；blocks id/stmts；loops。跨表引用一律名字/形状（表无关）。`can3` 头使全部内容版本换键——R2 前写入的 HIR/ABC/sidecar 行键不可达（不覆盖、不吞 Conflict）。cosmetic（span、mutable_explicit）不进键 | `cache.rs canonical_hir_function` |
| 依赖归属 | history 条目 6 元组尾部自带 `Vec<BodyFactRead>` 冻结快照；`query_body_fact` 绑定消费者**当前条目**（同 callee 首读 wins，重读只返回新事实值）；命中核目标条目自己的快照，stale→拒绝命中→全链重验→**新对替换同键旧条目**（仅 remint 触达；内容命中提交不动缓存条目的 mint 基线/映射/mint 文本——R2 轮内曾因无条件覆盖破坏 hit 重锚，已修并回归 r1 86）；取消/丢弃不触碰任何条目；未读函数/接口 only 不受影响；恢复重建条目 deps=空（凭证不落盘） | `session.rs` FnRecord.history / query_body_fact / prepare_edit / commit / history_insert |
| 关闭终态 | `query_body_fact` 在 closed 会话直接 None——零结果、零计数、零状态 | `session.rs query_body_fact` 首行 |

### 三项关闭行

- **B1-R2-01 P1**：根因=canonical 漏 `Place.local`（及 locals name/index/param_mode、expr/block id、body owner/entry），`x=3→y=3` 同键命中旧代码（增量 3 vs 冷编 1）。机制=can3 全字段+换键隔离。原反例：真实 edit 增量==冷编==1、零 hit、HIR/code 双换新对象；同根恢复反例（合法但写目标不同/入口不同的工件）在 canonical 门被拒（skip:canonical=1、adopted=2、冷 body 服务）；owner/param 篡改现在也提前到 canonical 门拦截（门分配更新，verify 前提断言仍证 verifier 拒绝力）；值正控（3→4）冷编一致。永久断言=`t03_r2_assignment_target_change_is_new_content_not_old_code`、`t03_r2_assignment_value_change_positive`、`t03_r2_semantic_wrong_target_or_entry_not_adopted`、admission 控制门表更新。剩余边界=无（字段清单以 hir PartialEq 为准成文；r1 86 全矩阵回归确认无 cosmetic 误失效）。
- **B1-C03 残项**：根因=依赖挂消费者名（重读原地替换、remint 全清）。机制=产物归属（上表）。原反例 1（重读后同内容 edit）invalidated=1、旧凭据不被刷新；反例 2（B remint 后回 A）A 缓存对不被复用、fresh pair 替换；正控=A 的读取不泄漏给 B（B 命中不受 helper 变化影响）、未读函数变化零失效、接口 only 零阶段/身份保持、重验后新产物可记新事实、重读返回新值但凭据不刷新。永久断言=重写的 `t03_body_fact_query_dependency_invalidation`（完整矩阵）+ 收编 `t03_r2_reread_does_not_refresh_old_dependency`、`t03_r2_history_retains_its_own_body_dependencies`。剩余边界=scalar 检查不读 body（无 inline/CTE），未来消费者经 query 接入。
- **B1-R2-02 P2**：根因=新查询入口漏 closed 守卫。机制=首行拒绝。原反例：close 后 query 返回 None、`body_fact_query` 计数不动、零状态；open 正控与既有终态（run/Infra、close 幂等、cold_nodes=0）保持。永久断言=body-fact 测试尾段。

### 回归面（599cc15af 实测，全部定向）

session 20/20（含 512 旗舰 202s——can3 换键后 hit 机制全矩阵仍绿）、b1 9/9（原 4+新增 5）、contract 22/22、t01r 4/4、r1 86/86（曾因 hit 提交覆盖缓存条目红 12 项——修复后全绿，证明 r1 矩阵对该接缝的区分力）、r23a close-controls 6/6 + close-adjacent 3/3 重放、rustfmt 0、警告集比 HEAD 少一（删除 dead associated history_entry；余为 HEAD 既有 SpanInfo/2 unused-mut/shim doc）。ac-core/AV core/解析零改动。整档与 merge-owned 门禁保持 pending。

## 第一批 R3 窄修复轮（a66d9d9e5，待窄复审）

冻结 `a66d9d9e54e4db1de90edc078ed7d23d161e00d5`（基面 599cc15af，同树；R2/前两轮机制全部保留）。

**B1-R3-01 / P2 关闭行**：根因=can3 的 Call binding 渲染只有 param/arg，遗漏 `moved`（hir PartialEq 明确语义字段、codec v2 真实承载）——verifier 会接受的"仅 .move 标记不同"合法 HIR 与当前源码同内容版本，恢复采纳后 resident 变为 true。修复=canonical 升 **can4**：绑定渲染 param/arg/`moved`（`.move` 标记 `!`），并补 Binary 的 overflow 策略渲染（当前单态 Trap，渲染使未来新增变体无法从该缝隙进入内容身份）；格式标记换键，R3 前全部 store 行不可达即隔离（不覆盖/不吞 Conflict）。原反例：moved true 变体现在被 canonical 门拒绝（skip:canonical=1、adopted=2、resident 保持冷源 false）；字段策略经全量复核——Body{owner 按名,entry,loops}/LocalDef{id,name,role,index,mutable,param_mode,ty}/Place{id,kind,local,ty,access}/Expr{id,ty,各 kind 全参含 Call{callee 名+签名,eval_args,bindings{param,arg,moved}},Binary 含 overflow}/Block{id,stmts}，cosmetic 仅 span/mutable_explicit。永久断言=`t03_r3_call_binding_moved_distinguished_at_restore`（收编 R3 canonical-controls 反例，含 PartialEq 前提与采纳/门计数）；`t03_r3_dependencies_cancel_and_late_do_not_refresh_history`（收编取消/迟到依赖正控）。剩余边界=scalar profile 工件面不含 owned Str/.move 调用点（语言能力不属本计划）；字段策略以 hir PartialEq 为唯一权威，未来新增语义字段随格式标记换键。

**回归面（a66d9d9e5 实测，全部定向，按任务单不跑 512/整档）**：b1 11/11（原 9+新增 2）、session 18/18（显式跳过两个后批 512）、contract 22/22、t01r 4/4、r1 86/86、cache_tier 20/20（798 共享 canonical 调用方在 can4 换键下全绿）、r23a close 6/6+3/3 重放、rustfmt 0、警告≤HEAD 既有。ac-core/AV core 零改动；整档与 merge-owned 门禁保持 pending。

## T-07 剩余轴三切片（54c5f1d3d / bae9f15f4 / 429e244ee，与第二批复审并行实施；待第三批受影响面复审）

与第二批 T-05+T-06 复审并行（用户连续实施裁定；复审收据明确将这些提交排除在其覆盖外，绑定冻结 a66d9d9e5）。三切片各自独立提交：

- **切片 A（V-07/C05-02/03 回收矩阵，54c5f1d3d）**：可观测访问器 `resident_weak` / `PreparedEdit::code_weak`（仅诊断，不改拥有关系）。永久控制 `function_incremental_828_t07`（新目标）：外部 pin 的旧 snapshot 保持可运行、最后 pin 释放即回收（Weak 观测，非 drop(session)/history_len）；未 pin 的退役 history 对在任意淘汰序下真实回收（死亡计数 ≥ 总内容-容量-1 的序无关 oracle）；外部 pin 跨 close() 保活其代、最后 owner 释放即死；候选的未安装对恰在消费点死亡（cancel/闭态 commit 拒绝），正常 commit 安装后存活；**哨兵不变**——共享 store 中的无关用户行在 40+ 修订浸泡、restore 与再持久化后逐字节保持。
- **切片 B（F-11 manifest required 引用，bae9f15f4）**：`persist_manifest` 将 manifest 注册为 store root（project 级、kind session-manifest、幂等累积）；`restore` 在任何 warm 准入前**消费** roots——某注册 root 须在本 owner 身份下解信并精确描述当前程序（entry/caps/每函数内容版本）才 warm（`restore_manifest_matched`）；注册了 roots 但无一匹配（陈旧源/垃圾 root）→ 回冷，冷路径重存匹配 manifest 自愈（`restore_manifest_rejected`）；**无**注册 roots = 局部/旧式 store，按单元内容寻址路径照旧服务（`restore_manifest_absent`，r19a/r21a 手工种子家族保持）。门不加信任，只拒 warm。
- **切片 C（F-09 驻留盘点，429e244ee）**：`resident_inventory()`（函数数/每函数 HIR body 节点/冷 bundle 节点/有界 history 条目——§5.5 驻留轴的计数面）；真实峰值字节归因归 T-08 正式测量。控制：水位收敛（超容量后更多编辑不动水位——淘汰真实运行）；close() 后全部归零。persist 失败非回滚路径保持结构证据（安装先于持久化，错误只记计数器——不存在回滚代码），释放矩阵由 T-06 事务测试+切片 A 承载。

回归（各切片时点，全部定向）：t07 6/6、b1 11/11、session 18/18（显式跳两个第三批 512）、contract 22/22、t01r 4/4、r1 86/86、cache_tier 20/20、r23a close 6/6+3/3、rustfmt 0、警告≤HEAD 既有。

## 第二批 needs_fix 四项一批修齐（cfa67eece，待受影响面复审）

冻结 `cfa67eeceb4555e55b978354d3fd7ab508e80664`（基面 a66d9d9e5→经 T-07 三切片至 429e244ee；B2 修复在其上）。复审包 [review-batch2-a66d9d9e5](../review-batch2-a66d9d9e5/review.md) 的四项关闭行：

- **B2-C01/P1 owner 上下文**：根因=link 仅按 `c.name()==target`+结构签名选 callee——异 owner 单元（同名同签名、合法 ABC 准入）被链接执行（43）。机制=`link_with_maps` 要求**单一 owner 上下文**：所有单元的准入 owner（schema+module）必须一致，外来单元在受检链接即拒。配套 owner 保真修复：`verify_abc` 本按"调用方授权"重 stamp owner，但 restore 曾把 decode 占位符当期望 owner——现按**会话 owner 上下文**（MIR_SCHEMA+模块 id）stamp，恢复单元与新铸单元同域，owner 门对合法流一致、对外来单元有实义（此修复前 r1 恢复-编辑链接被门误伤，已回归证明）。正控：同 owner 替换 43、普通链接、递归全保持；反控：foreign owner 拒绝且 install/dispatch=0。
- **B2-C02/P1 调用闭包**：根因=非 `fn:` import 存 None 绑定，可执行 snapshot 构造与执行前不核实际 Call 闭包——已准入未闭合调用先产生 MarkA 再 Infra。机制=link 末核**每单元指令实际调用的 import** 必须有链接期绑定（真实前端把测试标记 lower 为 Trace 指令，Call-on-import 是外来载荷形状）；未调用的 import 元数据保持 dispatch 期准入违规语义。三入口均无法先执行 guest 副作用后报错。正控：合法 Trace 程序绑定完备、值 8。
- **B2-C03/P2 来源投影**：根因=`map_span` 用 payload span.func（产生模块槽位）直接索引 per-code-slot maps——代码集重排后串图（64..69→33..33、125..130→225..230）。机制=snapshot 链接期构建 `func_slots`（payload func→执行槽，由各单元自身 span 表导出），`map_span` 经投影取"拥有该 payload 的代码槽"的图；payload 不改写、对象身份不变、结果 span.func 保持 payload 稳定身份轴。正控：真实源码编辑（reuse_hit=1 非恒等图）后同 code+maps 重排，trap 坐标不变；synthetic 偏移图各归其主。
- **B2-C04/P2 分段预算**：根因=resume 把切片预算当累计阈值比较（`st.steps >= budget`）——重复 Some(1) 永不前进。机制=`dispatch` 记录 `seg_start`，预算按段内增量比较；累计硬限额（max_steps/fuel/frames/slot/trace）仍跨段。Some(0) 零指令干净暂停（`PausedRun::step_count` oracle）；连续 Some(1) 完成六步程序。

收编：`function_incremental_828_b2`（13 测试=复审 link 控件 8 全部同输入同 oracle + Some(0)/硬限额兄弟 + execution 控件 4）+ session.rs 组件内 `component_link_failure_keeps_all_installed_state_then_accepts_good_candidate`（真实生产 commit；root/source/revision/env 代际/全 records 图像（history+deps+origin/map/canonical/mint）/旧 trap 全等；good 候选随后可提交）。

回归（cfa67eece，全部定向）：b2 13/13、r1 86/86、b1 11/11、session 18/18（跳两个第三批 512）、t07 6/6、contract 22/22、t01r 4/4、core 22/22+组件 1/1、cache_tier 20/20、r23a close 6/6+3/3、rustfmt 0、警告=HEAD 既有。剩余边界：owner 门当前按"单模块单 owner"合同（多模块语言能力不属本计划）；abc wire 不携带 owner 权威（由准入调用方 stamp——restore 现与会话一致）；整档与 merge-owned 门禁保持 pending。

## T-08 脚手架（8be89bf5d，并行于第二批 R2 复审实施；正式测量后置）

冻结 `8be89bf5d208f6f36f65aa22b79eea0a9a5953ff`（基面 cfa67eece）。**范围边界（用户裁定）**：本切片交付 F-12 的协议/骨架与全部 V-09 门轴，正式测量战役（冻结 273575B 负载、≥30 修订、P95<1000ms 判决、双线内存预算）在第二批 R2 通过后于已审执行核心上执行——本切片不产正式数字。

- **常驻 worker `session-serve`**（av_probe_worker 新子命令）：一进程一会话——`init`/ready 握手（恒定 worker 身份=session id，按 §5.5 一次核验；初始值由父侧独立期望判定）、编辑流（每次发布携带已验证当前值、worker 内部分相计时（仅归因）、累计计数快照（父侧取差）、worker 线内存峰值采样）、`close` 终止。worker 不做判决。
- **父侧 judge**（formal_worker.rs）：测量窗=编辑**入队**→父侧验证发布（IPC 在窗内，§5.5 语义）；独立期望（ready 值+每修订）；结构门=未改函数零阶段工作/零整模块轴/安装集恰一（重扫为证据轴单列——有界窗口可含相邻 decl，非门）；双门=P95<gate_ms ∧ 全对 ∧ 结构 ∧ 完整，**gate_ms=0 必红**。worker 中途死亡→记录"缺样本"并红完整性门，绝不静默缩小战役；`raw_jsonl` 全样保留。
- **控制（function_incremental_828_t08，3 项，真 worker 二进制跨进程）**：小 fixture 判决系列全绿（身份/双内存线/raw 完整）；零门/错期望/流中断三轴各自单独红门；ready 错期望在任何编辑前拒绝。

回归：t08 3/3、b1 11/13(b2)、session 18（跳两个 512）、contract 22、t01r 4、fmt 0、零新警告。R2 复审收据已注明本并发 WIP 不在其覆盖内；后续新冻结须覆盖全部实际修改（本提交已入库满足）。


## 第二批 R2 三项一批修齐（3090f2326，待受影响面复审 R3）

冻结 `3090f2326f3595a69132c80bee3a2891c5f41b76`（基面链 a66d9d9e5→cfa67eece→8be89bf5d[T-08 脚手架]→本提交；R2 收据要求的"新冻结覆盖全部实际修改"满足——树含 T-08 脚手架已入库提交）。

- **B2-C03 残项关闭行**：诊断=旧 oracle（R1 重排/真实编辑控制）全绿但不变量未闭的根因——R1 的 `func_slots` 以**裸生产 func 号**为全局键，前提"一个 snapshot 的所有单元共享一个生产坐标空间"不成立：真实同 owner 单元各自从自己的单函数 bundle lower，不同函数可占同一生产槽号（审核反例 helper/main 各占 slot1），insert 后项覆盖前项→串图。机制=**帧的执行槽投影**：`ModuleView::map_span`（默认恒等，模块级视图发布自身坐标）+ `UnitView` 经本 snapshot 的 `span_maps[执行槽]` 映射——trap 构造点在 dispatch 内用**当前帧的槽**（唯一坐标）取图，payload func 号不再参与键控（结果 `func` 字段保留 payload 稳定身份轴）；全局 `func_slots` 表与外层 `map_span` 包装删除（三入口单一映射点，无双映射）。原反例：共享生产槽的两真实单元 159..164 正确（普通+分步）；R1 重排/真实编辑/六槽序/不可变 payload/旧 pin 保持；Un 臂局部 `v` 遮蔽修正。收编 `projection_distinct_admitted_functions_sharing_producer_slot_cannot_cross_maps`。
- **T07-R2-01 关闭行**：根因=匹配器无 format 校验、行解析静默跳过（filter_map）、多重集包含比较允许重复掩盖缺项。机制=严格目录判等：format 必须==1（未来格式拒 warm 不解读）；每行必须解析出 def_id/cv 字符串（坏行拒该候选，绝不静默跳过）；listed 多重集去重后须与期望**集合精确相等**（重复即非精确目录；合法重排仍匹配）。原反例：重复 helper 掩盖缺 main→rejected=1；format 999→rejected=1；合法重排→matched。收编审核 t07-controls 五控制+重排正控。
- **T07-R2-02 关闭行**：根因=manifest edges 恒空、units 无工件目录、每行 project soft 根即保活——非 manifest required 闭包；退役 soft 根后 GC 删尽恢复所需。机制=**manifest required 边闭包**：单元 ABC 行与 span sidecar 改 store-internal（project None）并在 persist 时解析工件 id（put 成功即 id；Conflict 经 `get_enveloped_with_id` 解析幸存行）作为 manifest 的 `RequiredData` 边（含 repair 行路径）；manifest 以边+显式 root 发布（register 先行）；**superseded manifest root 在发布点退役**（旧 manifest 及其边随之消亡→被取代修订的单元载荷可被真实 GC 回收——稳定水位不靠全历史 rooting）；HIR 工件保持 soft 根（可再生缓存，soft 正控来源）。原反例：退役全部 soft 根+真实 GC→manifest 保留、admitted=3、值/来源正确；live GC 同样 3/3。新增：superseded 载荷被真实 GC 回收（deleted>0）且共享 project 下带独立 root 的用户行逐字节幸存。
- 回归（3090f2326，全部定向）：b2 14/14（含新投影控制）、t07 8/8、b1 11/11、r1 86/86、session 18/18（跳两个 512）、contract 22/22、t01r 4/4、t08 3/3、core 22/22+组件 1/1、cache_tier 20/20、r23a close 6/6+3/3、rustfmt 0、警告=HEAD 既有。剩余边界：manifest 边集=当前恢复闭包（ABC+sidecar）；HIR 分类为可再生缓存（soft），其全量保活策略随 F-10/预算轴在 T-08 正式测量/第三批复审中再核。


## T07-R3-01 窄修复（b846f45dc，待窄复审）

冻结 `b846f45dc10abb580136ff97f0562db7c1f62e41`（基面 3090f2326）。**身份/版本设计一行**：required-边 manifest 的**键**取 format=2（body 的目录语义不变——format 字段仍为 1；匹配器按 body 判等从不按 key 读取）；旧无边行（键 format=1）经其 root id 只读、仍可 gate warm restore，在新图注册时按既有先注册后退役时序退场；同 body 不同边集=不同 artifact 身份，永不共键。冷回退→写新键→旧根退役→下次恢复自愈为闭环。旧行不覆盖、不吞 Conflict、不要求清缓存。
- 关闭行：原 4 失败出口全绿——partial 旧 store 冷回退自愈且再恢复 warm=3；旧行上直接 warmup_persisted 成功；完整旧闭包先 warm admitted=3 再重开不 Conflict；warm 读后真实同内容空白编辑 reuse_hit=1 且 persist_error=0（新 required 图发布）。正控：新图幂等重开+同内容编辑 persist_error=0。新增全路径：legacy→warm 读→同内容编辑→旧根退役→真实 GC→再恢复 admitted=3 值=9。
- 收编：`function_incremental_828_mig`（6 项，R3 审核 migration-controls 同配方同 oracle + 全路径测试）；旧 R2 required-root/退役 GC/用户哨兵控制保持。
- 回归（b846f45dc，全部定向）：mig 6/6、t07 8/8、b2 14/14、b1 11/11、r1 86/86、session 18/18（跳两个 512）、contract 22/22、t01r 4/4、t08 3/3、core 22/22+组件 1/1、cache_tier 20/20、r23a close 6/6、fmt 0、警告=HEAD 既有。剩余边界：无（键身份策略已在 783 store 合同内：同 layer+key 单身份、artifact_id 含边集）。


## T07-R3-01 残项修复（cccb09b1f，待窄复审 R5）

冻结 `cccb09b1f2b77e14cb410811eb4493247e2ed787`（基面 b846f45dc）。**更正上一轮的错误声明**（"同 body 不同边集永不共键/剩余边界无"）为有证据的当前设计：持久身份=**实际 body 指纹 + 规范化 required 闭包指纹**。

- **身份设计（一行）**：`persist_manifest` 将边集规范化一次（按 (artifact_id, ref-kind) 排序去重），同一规范化集**既用于实际发布的 edges 也用于键的 `required-fp` 分量**（与 body 的 fp 并列；format=2 分量继续隔离旧无边行）——同闭包任意枚举序幂等同键；闭包目标变化（如缺块后重建的 primary sidecar 新 id）取新键；旧行只读不覆写、Conflict 不吞、新根注册成功后才退役旧根。
- **诊断**：固定版本轮换（1→2）只区分"旧空边配方 vs 新配方"两种离散身份；同一新配方内不同 required 目标（审核反例：真实缺块→store recover→合法源码位置变化→primary 重建 id 变化，fresh reference store 逐字节证明 manifest body 不变）仍共键——store 按含边 artifact 身份正确拒绝第二身份。每个固定版本号都会复发；身份必须绑定实际引用集合。
- **关闭矩阵**：R4 rebuilt-closure 两控制收编——同源缺 primary 正控冷自愈+再 warm 3/3；位置变化反例（body 逐字节等、重建 primary id 确实不同）restore 成功、新图独立身份发布、再 warm 3/3。区分控：同闭包经真实 publisher 重开幂等（单 manifest 根、persist_error=0 跨重开+编辑）。R3 五旧迁移控制与永久 mig6、T07 required 根/用户行/退役 GC 控制全部保持。
- 回归（cccb09b1f，全部定向）：mig 9/9、t07 8/8、b2 14/14、b1 11/11、r1 86/86、session 18/18（跳两个 512）、contract 22/22、t01r 4/4、t08 3/3、core 22/22+组件 1/1、cache_tier 20/20、r23a close 6/6、rustfmt 0、警告=HEAD 既有。剩余边界：键身份=body+规范化闭包双指纹，783 store 合同内（同 layer+key 单身份、artifact_id 含边集）；RefKind 字符串往返采用 Debug 名匹配（四变体穷尽）。


## T-08 正式测量战役（f7d1091b3，待第三批受影响面复审）

冻结 `f7d1091b358386b1b28f85eabb5e75414a40adce`（基面 cccb09b1f）。V-08 双门在冻结负载上经**真实常驻 worker + 父侧 judge** 落地：

- **`formal_campaign.rs`**：从冻结 273575B/115 函数负载生成器构造判决脚本（修订 n 在 fn_037 首乘数处与 n-1 不同、独立期望值），≥30 真实 body 编辑；计时窗=编辑**入队**（父侧，含 IPC）→父侧校验发布；ready 值独立判定；初始化排除单列；`raw_jsonl` 全样保留；双线内存=worker 峰值随每次发布 + 父侧峰值包裹全程。
- **永久断言 `t08_formal_campaign_v08_dual_gate_30_revisions`**：30 修订全对、结构门绿（未改函数零阶段工作/零整模块/恰一安装/修订）、**P95<1000ms**、raw 完整、p50≤p95≤p99。实测（本轮，f7d1091b3）：**n=30，p50=371.8ms、p95=451.2ms、p99=533.7ms，worker 峰值 198MB、父侧峰值 2MB**。计数入 F-09 预算证据（198MB 为进程私有字节采样峰值，含驻留 HIR/查询图/历史/候选/索引——按 §5.5 归因法；非门，最终字节峰值预算判决随第三批/merge-owned 收口）。
- 回归：t08 4/4（脚手架 3+正式 1）、mig 9/9 保持、check 0 错误。剩余边界：≥30 样本已满足 V-08 下限；正式报告落盘路径/多场景扩样（函数前中后/槽位/控制流变化）与 observations 汇报面归 T-09 收口与第三批复审；512/GC/双线预算完整证据归属第三批 T-07/T-09 原 owner。


## T-07 完整 512/GC/存储轴 + 双线预算判决（f2593bf75，待第三批受影响面复审）

冻结 `f2593bf757880c21b5a392519f66dae3b8e6f2d9`（基面 f7d1091b3）。T-08 收据移交的三个原 owner 残项收口：

- **诊断（512 存储轴为何此前不可收）**：`persist_all_units` 对未改单元每修订全量重编码+幂等重放（115 函数×512 修订≈5.9 万次 put，实测 >12 分钟未完成）——违背 F-11 changed-only 意图；且 A07 HIR 行按内容版本累积 soft root、从不退役，512 修订线性增长（R2 收据遗留的"HIR 全量保活策略再核"项）。
- **修复一（changed-only 持久化）**：`FnRecord.persisted_unit_edges=(cv, abc_id, sidecar_id)`——本会话已按该 cv 发布过的单元在后续 persist 中**零 store 流量**（记录的工件 id 直接重入 manifest required 边）；restore 后的新会话无追踪器，做一次幂等全量后建立；外部删行经 manifest put 的 TargetMissing 可观察暴露（fail-closed，不静默）。
- **修复二（有界 HIR soft root）**：发布新 cv 行时退役该函数被取代 cv 的 soft root（每函数至多一个现行 root）；内容回访（同键 Conflict）重挂幸存行（幂等）；restore 采纳成功置 `persisted_hir_cv` 追踪器，restart 后首次取代仍正确退役。计数 `hir_soft_root_retired`/`hir_soft_root_rerooted`。HIR 是可再生缓存，退役只影响未来 restore 命中率（miss 走冷编真值），不影响正确性。
- **永久断言 `t07_v07_full_storage_axis_512_edits_gc_restart_sentinel`**（V-07 完整形态：fresh persisted 会话在冻结 115 函数负载上）：512 真实 body 编辑（changed-only 持久化）逐修订独立期望全对、安装集恰 fn_037；每 64 修订真实 GC——累计 deleted>0、**live-set 水位 ≤ 初始闭包+8**（不随编辑数增长）、**soft root 数 ≤ 函数数**；第 171/342 修订两次中途 warm restart（manifest matched、115 单元全部重验准入）后续跑；pin 的修订 0 快照全程可运行（值=基线期望）、释放即死（Weak）；用户哨兵行在全部 GC 后逐字节幸存；驻留 history ≤ 32×115 且 <513。实测 262.6s（changed-only 生效；修复前预估 >25min）。
- **双线预算判决（硬门进战役测试）**：`av_probe_worker frontend-attribution` 子命令=冻结负载前端编译瞬态**隔离**测量（新进程、PeakSampler 5ms，与 798 bench_turnaround A 线同口径）；`t08_formal_campaign_v08_dual_gate_30_revisions` 内断言 **A ≤ BUDGET_FRONTEND_TRANSIENT_PEAK_BYTES(224MiB)** 且 **B=worker_peak.saturating_sub(A) ≤ BUDGET_WORKER_PRIVATE_PEAK_BYTES(128MiB)**（缺归因整峰计 B 的保守语义由 saturating_sub 之前的 A 测量缺失场景承担——子命令失败即测试失败）。**实测（本轮）：A=201MB(≈191.7MiB) ✓ / B=0MB ✓（worker 峰值 194MB < A 归因，饱和差 0）/ 战役 p50=188.9ms、p95=342.6ms、p99=352.5ms、父侧 2MB**。
- 回归（f2593bf75..8a18db450，全部定向）：t07 8/8、mig 9/9、lib 组件 1/1、b1 11/11、b2 14/14、contract 22/22、t01r 4/4、r1 86/86、cache_tier 20/20、core 22/22、主套件 18+**三件 512 全绿一次**（soak/旗舰/完整存储轴 455.9s）、t08 4/4（战役+双线）、798 面 mir 8/abc 15/binding 4/vertical 3/native_admission 11/native_trace_parity 1/case_binding 5、r23a close 6/6+3/3 重放、rustfmt 0、probe 零告警。


## T-09 收口（8a18db450，第三批冻结，待第三批 T-07+T-08+T-09 受影响面复审）

冻结 `8a18db4503a6b30e53a0affa9cb17c4126d35e6d`（基面 f2593bf75）。四项交付：

- **映射清单**：`docs/plans/attachments/828/coverage-manifest.md` 全量 r3 回写——V-01..09 / C01-01..05 / C02-01..05 / C05-01..04 逐 case 映射实际测试入口（文件::测试名）、区分性 oracle 与交付版本；批次 finding（B1/B2/T07-R*）→ 永久断言映射；旧消费者兼容证据（828 面+798 面+r23a 关闭面）；r2 开放项 2/3 关闭、开放项 1 如实登记为非 required 边界。
- **SD 落文**：`docs/plans/attachments/828/spec-delta-r3-text.md`——SD-01/03/04（add）新 spec 文件全文 + SD-02/05/06（modify）对现行文件的精确增改段；proposed，merge 沉淀。
- **旧消费者兼容/后继接缝**：`successor-interface-contract.md` r3 回写——按真实 API 更正 r2 失实表述（`link_with_maps` 单元派发为生产路径、HISTORY_CAP 有界淘汰非"永不逐出"、FnEnv body-fact 核心替代不存在的 `interface_unchanged`/`credential_current`、manifest format=2 双指纹、有界 HIR soft root）；817（CTE 查询→body-fact 失效轴+783 store 身份）、818（共享 MIR/native 一致性→lower/OwnerScope/dispatched_payload_same_content）、820–824（语言面扩展→auto-scalar 体系+单元派发+canN 换键）、825（装载效率→驻留水位/分段预算基线）、826（紧凑 ABC→wire 版本升级身份律）、827（源码基线→load256/战役框架扩展）逐计划接缝落文。
- **告警清理**：SpanInfo 未用导入（session.rs）、CloseOp/InitOp 未用导入（av_probe_worker，T-08 脚手架）、两处 unused-mut（主测试文件）、shim doc 注释告警（classify.rs match 臂 ///→//，R3 复审点名项）——probe/ac-core/av-core 全零告警；shim-metadata 另 4 个 dead-code 告警为 master 既有债（828 零触及面，登记 KNOWN-DEBT 候选归 shim owner）。
- 边界：多场景扩样（函数前中后/槽位/控制流变化的战役变体）与 observations 汇报面仍为 r3 非必须的扩展面（V-08 下限 n=30 已满足）；正式报告落盘路径未建（raw_jsonl 全留在测试输出）——两项登记给第三批复审裁定是否后继。
