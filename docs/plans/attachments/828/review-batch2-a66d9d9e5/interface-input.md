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
