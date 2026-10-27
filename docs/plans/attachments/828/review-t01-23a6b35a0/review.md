# PLAN-828 r3 / T-01 修复受影响面复审

结论：**needs_fix**。T01-C01/C02/C03 本轮 CLOSED；新增 **T01-C04/P1**（完整模块准入回归），以及 **T01-C05/P2**（D-03 事务决策未满足既有合同）。T-02 暂不全面换接。Plan 保持 executing、0/11；F-05b 既有关闭范围保持 CLOSED。

受审 `plan-828-dev@23a6b35a0e94a6c948230df94c0e686129c2e8ae`，修复基面 `3e60af2c8aa2c287633f4700b377714e94c4ff4f`，累计 T-01 基面 `47894c2e6747c8c06032be7a3d40bf00b90a204d`。实施树 `D:/autostack/.wt/lang-828/auto-lang`，入口和收尾 clean。review_scope 为 final 角色的定向复审，非整份最终验收；reviewer 为 Codex /root 当前审查上下文，未实施两个 T-01 提交，未创建额外 agent。主检出其他在途文档改动保留；本轮只写本报告/探针及 Plan 簿记，不修实施、不合入。

输入：[Plan](plan-input.md)、[接口决策](interface-decision-input.md)、[SD](spec-delta-input.md)、[冻结指纹](baseline.json)、[收尾复核](final-state.json)。依赖 auto-down `c39ac7f9dacdc769fc4a5ad3b98ec896240b28e8` clean；本轮 Cargo 产物清单没有消费该兄弟包，不把它算兼容验证。实际本地依赖 manifest、配置、Spec、schema、rlib 身份见 baseline。Plan 的 §§1–7、任务 ID/计数和验收语义未改。

## 旧 finding 关闭情况

| ID | 本轮证据 | 结论 |
|---|---|---|
| T01-C01/P2 | 删除 owning ResolvedFn 的 clone/stub_resolved；Emitter 仅 plan/render 改函 body，其余为无 body 声明。原三函数反例现只生成/绑定 1 body，合法冷编 dump 及非首位递归对照保持 | CLOSED；新增完整模块准入问题另列 C04，不复活旧反例 |
| T01-C02/P2 | resolve/typecheck/plan/bind 真实入口的窗口观测，改函 +1/+1/+1/+1、其余 0。独立把真 seam 和真旧 synth 都放进同一窗口，测得 helper=2/2/2/2、main=1/1/1/1，同门红 | CLOSED；覆盖已测 scalar 自由函数同步路径，不授正式性能/worker 观测门 PASS |
| T01-C03/P2 | FnSig 与 CheckedEnv.sigs 可见性一致，本轮两 workspace 构建不再出现 private_interfaces；四条旧 probe warning 与 shim 旧注释 warning 不算新增 | CLOSED |

上一轮冻结独立 frontend 原样重放为 **3pass/1fail**，输出保留：[raw](original-frontend-output.txt)。剩余失败明确仍读取旧 `env_walk_stats + receipt` 谓词；该谓词本来就无法覆盖额外管线，新实现没有让旧谓词获得新含义。本轮用新真实生产者门重放相同额外工作后确实红，见 [独立探针](boundary-controls.rs) `boundary_actual_old_work_in_same_window_is_detected`；收编 t01r 4/4。这里关闭的是原工作不变量，不把旧弱谓词的红误报为新门仍漏检，也不隐藏原样结果。

## T01-C04 / P1：公开完整文档接受无 body 函数，产生无效 CheckedModule 并使后端 panic

**既有规则**：Plan §5.2/§5.6、AC-02/05、C01-04 及 SD-02 要求完整输入入口/全部适用规则保留，函数级接缝不能削弱完整校验。CheckedModule 仍是完整模块后端的授信边界。该项为本轮修复回归，owner=T-01/D-01，不是新增产品需求。

**定位与原因（源码确认）**：

- [descriptor.rs:1253](D:/autostack/.wt/lang-828/auto-lang/experimental/ac-core/src/descriptor.rs:1253) 在公开 binder 中允许缺少 body，填 `BodyId(u32::MAX)`；五种既有 descriptor 同时取消 body 必填。
- [verify.rs:1167](D:/autostack/.wt/lang-828/auto-lang/experimental/ac-core/src/verify.rs:1167) 对不存在 body 直接 continue，依赖“binder 已拒 dangling body”这个如今失效的前提，因此无 body 函数可取得 CheckedModule。
- 后端按完整凭证索引 [native.rs:641](D:/autostack/.wt/lang-828/auto-lang/experimental/ac-core/src/native.rs:641) / [native.rs:2472](D:/autostack/.wt/lang-828/auto-lang/experimental/ac-core/src/native.rs:2472)，索引 4294967295 越界。

**可重放输入/路径**：五个 `boundary_missing_body_*`。每个从自身 profile 的真实 `fn main() int { return 1 }` 冷编文本开始；先断言完整原文 bind+verify 成功；再删除 `; body: "b_main"` 和 `body b_main {...}`，其余身份/类型/声明不变。相同缺失文本用修复前同 profile descriptor 断言拒绝；随后真正经公开 `bind_document → verify/verify_scalar/verify_text/verify_record/verify_text_record → find_entry → lower_object`。没有手造受信 HIR 或修改私有字段。

期望：完整文档在 bind 或完整 verify 受控拒绝；不得授完整凭证。实测：Core、Scalar、Text、Record、TextRecord 五路全部授信，native 全部越界 panic；探针捕获该 panic 后，因完整 verify 应拒绝的断言失败。**5 个失败对应同一个不变量**，不拆成五项。见 [source](boundary-controls.rs)、[raw](boundary-output.txt)、[旧 descriptor 固定输入](old-scalar-i32.atom)。没有声称执行了产品 CLI 或证明安全利用。

**关闭 oracle**：保留函数级无 body callee 的合法控制及单 body 工作边界；五 profile 的合法完整文档继续通过，缺失 body 的完整文本受控拒绝，拿不到完整 CheckedModule。也覆盖“真 body + 无 body callee 声明”的混合完整文档和不调用的无 body 声明，不能只修空模块/入口函数或在 native 里吞掉 panic。可选择隔离上下文绑定或在完整授信边界明确拒绝缺失，具体机制由 work 决定；不能靠重新生成全部 stub body 退回 C01。

## T01-C05 / P2：D-03 仍允许候选替换当前环境，没有冻结事务首例

**既有规则**：Plan §5.4、§5.7 规定仅成功提交替换当前语义；§8 原 R-04 与上一轮任务单已要求 candidate/current 基面、prepare 隔离和最小事务首例。owner=T-01/D-03；完整 C01/C02/C05 状态矩阵仍在原 T-02/T-06，不提前要求全量落地。

**文档证据**：[interface-decision.md:42](D:/autostack/auto-lang/docs/plans/attachments/828/contract-r3/interface-decision.md:42) 明确写“prepare 不改 env 槽位之外的任何 session 状态”“候选验证失败时 recheck 的 env 槽位换入保留”“若需严格回滚……选择留给 T-02……两语义都不提前发布”。这把既有隔离规则变成可选项；文档也没有候选环境的基面/持有者/成功发布接缝及对应首例。禁止全 env clone 是正确约束，但本身不是事务机制。

**原语实测，非 session 污染结论**：独立 `boundary_post_recheck_verify_rejection_leaves_mutated_env_demonstration` 先验证原 helper；将 body 改为 `var x = 2` 无 return，recheck 成功后 emit，真实 `verify_function` 拒绝。再从同 env 发射，即使用原 source 参数，仍是被拒 body；这确认后置失败不会自动恢复槽位。该示例自身 PASS，说明可用于构造事务负控；`recheck_function` 原地修改本身并非未承诺的 API bug。**当前 session 尚未接 CheckedEnv，本轮没有声称已复现 session 的 root/source/revision 污染。**缺陷是 T-02 将要执行的决策与当前合同冲突且必要首例未做。

**关闭 oracle**：选定并实现有界的单槽事务/候选机制，明确候选与当前基面；失败、取消、迟到/foreign/closed 不发布，成功 commit 一次发布相应环境/根/source/revision/来源。T-01 只先做已登记的同基面双候选取消后合法继续、一次 recheck 成功但后置 verify/lower/link 拒绝的真实最小事务首例，证明 current 保持；完整矩阵留原 owner。close 明确撤销新环境所拥有的 body 引用，不能以“字段在 session 内”推导调用 close 自动 drop；预算仍属 F-09。禁止全环境深 clone 的要求保留。

## D-02 与其他受影响面

新 ModuleView/UnitView 确实经共享 dispatch 直接读取原 FunctionCode 的局部类型/常量/指令；本轮独立观察已过非首位自递归、互递归、bool/i32 局部空间、真实长度编辑后的非首位 trap 全 span、旧 pin、缺 import/错签名实际 link 拒绝。main→helper 的新接缝可行性已具备有界证据。

实施 `unit_view_first_case` 的 `before=s.current()` 实际位于 edit 之后，原测试单凭该断言不能证明跨修订身份。本轮另在 edit **之前** pin、确认根不同而 main 的真实单元 payload 同一；复制该 payload 后实际从 copy snapshot 执行，值相同但指针不同，独立控制通过。该测试时点应在永久回归修正，不另立运行时缺陷。完整 AV 生产者身份门、链接期预计算绑定、三默认入口切换和 C02 全矩阵仍属 T-05/06；UnitView 目前按 import 名运行时查槽，默认仍 MergedView，不能把原型首例说成 F-02 CLOSED。现有公开绑定只由 link 构造，实测缺 target/错签名确实在此拒绝；不将单独 vacant 检查等同完整绑定反控。

## 实际验证与边界

| 选择 | 结果 / 输出 |
|---|---|
| contract / t01r | 6/6、4/4，exit0；[raw](contract-output.txt)，其中 dispatch 2 是 contract 6 的子集，不重复相加 |
| 上轮独立 frontend 原样 | 3pass/1fail，exit101；旧谓词局限如上，[raw](original-frontend-output.txt) |
| 本轮独立 boundary | 6pass/5fail，exit101；5 fail 均 C04，D03 原语示范单列为通过，[raw](boundary-output.txt) |
| r23a 原关闭输入 | 9/9，87 filtered 不计通过，[raw](close-output.txt)；F05b 不重开 |
| session 两目标 | 主17/17、r1 86/86，[raw](session-output.txt) |
| AV core lib | 22/22，[raw](core-output.txt) |
| ac-core 六个定向目标 | hir9/scalar16/text16/record21/text_record18/source_frontend14 全过，[raw](ac-core-output.txt)；它们未覆盖新缺 body 文本，不抵消反例 |
| 修改的九 Rust 文件 / diff | rustfmt check skip_children exit0、diff-check exit0；[fmt](fmt-output.txt)、[diff](diff-check-output.txt) |

复跑入口（cwd=实施树）：

    cargo test --locked --manifest-path experimental/ac-av-probe/Cargo.toml --test function_incremental_828_contract --test function_incremental_828_contract_t01r -- --test-threads=1 --nocapture
    powershell -NoProfile -ExecutionPolicy Bypass -File D:/autostack/auto-lang/docs/plans/attachments/828/review-t01-23a6b35a0/run-controls.ps1 -Source boundary-controls.rs -Filter boundary_
    powershell -NoProfile -ExecutionPolicy Bypass -File D:/autostack/auto-lang/docs/plans/attachments/828/review-t01-23a6b35a0/run-controls.ps1 -Source close-replay.rs -Filter close_
    cargo test --locked --manifest-path experimental/ac-core/Cargo.toml --test hir_verify --test scalar_hir --test text_hir --test record_hir --test text_record_hir --test source_frontend -- --test-threads=1

先按 cargo-artifacts.jsonl 为本提交选择精确 rlib，再编独立探针；[库哈希](linked-libraries.json)。最初从 ac-av workspace 选非成员 ac-core tests 在执行前受控拒绝，原始输出保存于 [范围选择记录](ac-core-workspace-selection.txt)，改用 ac-core 自身 manifest 后上表全部实际运行。未跑整档 t/tv/tt/tb/tu/taa/tf、未重复旧62/798兼容全集，不声称复现 work 全部摘要；scope 集合有重叠，不合计为独立总覆盖。

## 收敛与下一步

本次为连续第二次 T-01 定向不通过，不因更名修复轮清零。旧三项有实质改善；新增 C04 的根因是只验证“局部路径能工作”，却把完整 binder 的先验约束移除；C05 则是把应当在 T-01 定下来的事务约束留成两个互斥选项。下一轮先核消费者/授信边界与候选状态矩阵，再一批修齐，不能只给五个样本打补丁或继续增加绿色总数。

具体工作见 [最新 GLM 指示](glm-next-round.md)，它取代旧基线任务单的当前执行入口。SD-01..06 SHA256 仍 `371a9298c285eb8a0d7309b77298ce9bdadb932e626cc82b7df3af7be3759623`；SD-02/05 因完整准入失败，SD-04 候选事务仍部分，不授整份 delta PASS。其余 A07/依赖、F09、F10/F11 剩余、F12 正式资源门、F14 保持 OPEN 与原 owner；canonical/ledger 不改。

仅记录 needs_fix，T-01 未勾，执行0/11。新冻结提交受影响面复审通过后才进入 T-02 全面换接；整份最终 AC09、merge-owned 冻结裸 t+tv/必要 consumer 与实际触面门、Spec/ledger/归档/guard 均 pending。记债不批准缩面或延期。
