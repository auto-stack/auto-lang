# PLAN-828 r3 / de4cba702 受影响面复审

**结论：needs_fix。T01-C04/P1 CLOSED；T01-C05/P2 PARTIAL、仍 OPEN。** 不新增 finding ID；T01-C01/C02/C03、F-05b 既有闭合保持。T-02 暂不全面换接，Plan 保持 executing 0/11。

受审 `plan-828-dev@de4cba7022a2dd9f2d03ee3dea91322a5088674c`，基面 `23a6b35a0e94a6c948230df94c0e686129c2e8ae`，累计 T-01 基面 `47894c2e6747c8c06032be7a3d40bf00b90a204d`；实施树 `D:/autostack/.wt/lang-828/auto-lang` 入口及收尾 clean。review_scope=final 角色定向复审，非整份最终验收。reviewer=Codex /root 当前审查上下文，未实施本提交或前序 T-01 提交，未创建额外 agent。只写审核证据/主计划簿记，不修实施、不合入或清树。

[Plan 输入](plan-input.md)、[工作决策输入](interface-decision-input.md)、[SD 输入](spec-delta-input.md)、[冻结指纹](baseline.json)、[最终复核](final-state.json)。auto-down `c39ac7f9dacdc769fc4a5ad3b98ec896240b28e8` clean，本轮 Cargo 产物未消费该兄弟包；实际依赖 manifest/配置/Spec/库哈希记录于 baseline。Plan §§1–7、AC/V/C/T/SD 与计数未改。

## 关闭表

| Finding / 原规则 | 独立证据 | 结果 |
|---|---|---|
| C04/P1；AC02/05、C01-04、SD02 完整授信边界 | 原五 profile 缺 body 反例均受控拒绝；额外五 profile × 调用/未调用混合文本都拒绝；单 body 脚手架在函数级合法、同 bundle 完整校验拒绝 | CLOSED |
| C05/P2；AC04/06、§5.4/5.7、R04 当前/候选隔离 | 新 prepare 的后置 verify 拒绝会恢复原槽，顺序取消/继续与 closed 守卫过；但成功 prepare、drop、交错处置、同名 foreign 仍有实测失败 | PARTIAL / OPEN，原 finding 残项，不另拆六项 |
| C01/C02/C03 | contract/t01r、新生产者同窗额外旧管线控制、单 body/cold dump/非首位递归与本轮构建保持 | CLOSED 不重开 |
| F05b | r23a 原输入九控制原样重放通过 | 原已测关闭范围 CLOSED，未来 session 接新环境后仍须按影响复验 |

## C04：完整凭证边界已修复

[verify.rs:1167](D:/autostack/.wt/lang-828/auto-lang/experimental/ac-core/src/verify.rs:1167) 对每个 Function/Method 的不存在 body 发 `verify.missing-body`，不再依赖 binder 已拒绝的旧假设；五种公开完整 verify 入口均到此处，函数级 verify 仍只校验目标真实 body。修复位于授信边界，未在 native 吞 panic、补空 body 或恢复全 stub 生产。

原 [boundary-controls.rs](boundary-controls.rs) **11/11**，其中五个 `boundary_missing_body_*` 从失败转绿。每个仍从合法自身 profile 的冷编 Atom 文本删除 body 字段与 body 段，经公开 bind→完整 verify；修复前 descriptor 对同文本的拒绝及合法完整文本对照保持。

另加 [admission-controls.rs](admission-controls.rs) 两个独立测试：五 profile × 有调用/无调用的混合文档共十组均以 `verify.missing-body` 拒绝；helper/main 两个目标分别单 body 发射，函数凭证合法而完整凭证拒绝。实际产物仍恰一个 body。见 [原样输出](boundary-output.txt)、[额外准入输出](admission-output.txt)。没有将测试方法数量或内层组合数混算。

## C05：后置拒绝回滚有效，成功候选仍提前改变当前环境

**违反的仍是原合同**：prepare 只构造候选；仅成功 commit 替换当前语义；失败/取消/迟到/foreign 不覆盖当前。该规则见 Plan §5.4/5.7、§8 R04 及此前两轮交接，owner=T-01/D03。完整 session 的 C01/C02/C05 状态矩阵仍归 T-02/T-06，本轮只检查新公开候选接缝能否满足最小事务前置，不提前要求完整换接。

**已有实质修复**：`prepare_function_candidate` 真正运行 recheck→单 body emit→verify；若缺 return 等导致后置 verify 拒绝，先 restore 再返回。永久 `contract_828_transaction_post_verify_rejection_keeps_current` 实际通过。`CheckedEnv::close` 显式清 fns/records，关闭守卫与幂等已具备；“字段嵌在 session 内自然释放”的说法已撤销。

**仍未成立的机制（源码确认）**：

- [adapter.rs:346](D:/autostack/.wt/lang-828/auto-lang/experimental/ac-core/src/source/adapter.rs:346) 将候选换入原 env；[成功出口:361](D:/autostack/.wt/lang-828/auto-lang/experimental/ac-core/src/source/adapter.rs:361) 直接返回旧条目的 FnSlotSave，未先恢复当前槽。返回后 env 已可经公开 emitter 读到候选。
- FnSlotSave 只拥有被移出的旧 body，没有候选 payload、所属环境身份或基面/代际。drop 会丢弃旧 body，候选留在 env。
- [commit:926](D:/autostack/.wt/lang-828/auto-lang/experimental/ac-core/src/source/resolve.rs:926) 只检查 closed，随后 Ok；它不安装自己对应的候选，也不检查 owner/槽或基面。
- [restore:906](D:/autostack/.wt/lang-828/auto-lang/experimental/ac-core/src/source/resolve.rs:906) 只比槽号与函数名，匹配便把旧条目写回；同名同槽不是环境身份，也不能区分已提交的新修订。

**独立最小输入与实测**：[transaction-controls.rs](transaction-controls.rs)，原文均为 `helper() int { return N }` 与 `main→helper`，使用真实 parse/resolve/prepare/emit；canonical HIR 与对应冷编逐字节比对。N=1 是当前基面，A=2、B=3，另一环境=9；签名与槽号相同。不是手造 payload/受信标记，所有交错均是同线程显式顺序，无调度假设。

| 控制 | 期望 | 实测 |
|---|---|---|
| 顺序 prepare A→显式取消→prepare A→commit | 取消保留 1，合法提交变 2 | PASS，作为合法控制保留 |
| closed 后 dispose/emit、幂等 | 受控拒绝、env 函数数 0 | PASS |
| 成功 prepare A，尚未 commit | 候选产物=2；当前仍=1 | 候选产物=2，但从 env 发射的当前已=2，FAIL |
| prepare A 后直接 drop 返回值 | 未提交候选丢弃，当前仍=1 | 当前仍=2，FAIL |
| 同时持有 A/B：取消 A→commit B | B 提交则实际=3；或受控拒绝已不适用基面 | B commit 返回 Ok，实际却=1，FAIL |
| A/B 都存活：先 commit B→再迟到取消 A | 已提交 3 保持，不被旧取消覆盖 | 取消返回 Ok，回滚为 1，FAIL |
| 把 A 的 save restore 到同名同槽的另一环境 | 拒绝，另一环境保持 9 | 返回 Ok，另一环境变成第一环境旧 body=1，FAIL |
| 把 A 的 save commit 到同名同槽的另一环境 | 拒绝 foreign | 返回 Ok，FAIL；本控制不声称该调用写了另一环境 |

实际 **8 测试=2pass/6fail，exit101**，见 [raw](transaction-output.txt)。这些是同一当前/候选/owner/基面不变量的代表性出口，不报六个新增 finding。新机制原先只在文档中存在，本轮已经成为可调用 API，所以可实测其错误；**当前 session 尚未接 CheckedEnv，不声称已复现 session 的 root/source/revision 被污染，也不把 F05b 原 session 的 foreign/late 守卫重开。**

## 为什么原 11 绿仍不能关闭 C05

原 11 中，五缺 body 例负责 C04，其他是派发/旧管线与旧 `recheck_function` 的示范。`boundary_post_recheck_verify_rejection_leaves_mutated_env_demonstration` **有意断言原地 recheck 后 env 仍保留被后续 verify 拒绝的 body**，它通过代表旧原语事实保持；它从未调用本轮新增 `prepare_function_candidate`，不是事务闭合 oracle。永久新 post-verify 回滚测试补上了一个真实出口，但没有覆盖成功 prepare 返回时的 current。

永久 `contract_828_transaction_cancel_then_continue` 在创建 B 前已经取消 A，证明的是顺序取消后继续，未证明两个候选同时存活的基面/处置；foreign 样本用另一环境的 `main` 对本环境 `helper`，只证明名字不同拒绝，未检验相同名字/槽号的真实跨环境。这是本轮绿色断言与合同间的明确缺口。

工作诊断已经正确识别“env 槽位是第二发布点”，但实现把“保存旧条目”当成“不提前发布”。被保存的 current 无法作为公开当前环境继续消费，发布时点也未被 commit 控制。下一轮应先修候选所有权与发布模型，再收编状态断言，不能只加几个拒绝分支或把 save 的丢弃解释成授权提交。

## 已运行的定向验证

| 选择 | 结果 / raw |
|---|---|
| contract 13（frontend4/dispatch3/admission3/transaction3）、t01r4 | 13/13、4/4，[输出](contract-output.txt) |
| 原边界控制 | 11/11，[输出](boundary-output.txt)，C05 旧原语示范的含义见上 |
| 额外准入 | 2/2，内含十个混合组合及两个目标成对验证，[输出](admission-output.txt) |
| 新候选事务 | 2pass/6fail，[输出](transaction-output.txt) |
| 原关闭控制 | 9/9，87 filtered 不计通过，[输出](close-output.txt) |
| session / r1 / core | 17/17、86/86、22/22，[session](session-output.txt)、[core](core-output.txt) |
| ac-core 六目标 | hir9/record21/scalar16/source_frontend14/text16/text_record18 共94，均过，[输出](ac-core-output.txt)；工作文档个别名字/计数对应有误，以本 raw 为准，不另立 finding |
| 本轮五个修改 Rust 文件、diff | fmt0/diff-check0，[格式](fmt-output.txt)、[diff](diff-check-output.txt) |

四条既有 probe warning 与 shim 旧 doc warning 保持，无新 private_interfaces。没有运行整档 t/tv/tt/tb/tu/taa/tf、docs_gen、旧62/798兼容全集或正式性能资源门；不把工作摘要冒充本轮实测。集合有重叠，不合计“唯一总测试数”。

重放 cwd=实施树，先按本轮 [Cargo 元数据](cargo-artifacts.jsonl) 精确选择 [rlib](linked-libraries.json)：

    cargo test --locked --manifest-path experimental/ac-av-probe/Cargo.toml --test function_incremental_828_contract --test function_incremental_828_contract_t01r -- --test-threads=1 --nocapture
    powershell -NoProfile -ExecutionPolicy Bypass -File D:/autostack/auto-lang/docs/plans/attachments/828/review-t01-de4cba702/run-controls.ps1 -Source boundary-controls.rs -Filter boundary_
    powershell -NoProfile -ExecutionPolicy Bypass -File D:/autostack/auto-lang/docs/plans/attachments/828/review-t01-de4cba702/run-controls.ps1 -Source transaction-controls.rs -Filter transaction_
    powershell -NoProfile -ExecutionPolicy Bypass -File D:/autostack/auto-lang/docs/plans/attachments/828/review-t01-de4cba702/run-controls.ps1 -Source admission-controls.rs -Filter admission_
    powershell -NoProfile -ExecutionPolicy Bypass -File D:/autostack/auto-lang/docs/plans/attachments/828/review-t01-de4cba702/run-controls.ps1 -Source close-replay.rs -Filter close_
    cargo test --locked --manifest-path experimental/ac-core/Cargo.toml --test hir_verify --test scalar_hir --test text_hir --test record_hir --test text_record_hir --test source_frontend -- --test-threads=1

## 当前下一步与 Spec 边界

C04 已满足此前完整授信关闭条件；SD02/05 的该子轴通过，不授完整 SD 或整份 Plan PASS。C05 的 SD04 事务子轴仍 FAIL/PARTIAL；SD01..06 SHA256 仍 `371a9298c285eb8a0d7309b77298ce9bdadb932e626cc82b7df3af7be3759623`。没有修改 canonical/ledger、目标/profile/P95/预算、原任务和阶段 owner。

按 [最新 GLM 指示](glm-next-round.md)，同树仅修 C05：让候选真正拥有待提交的新条目，当前环境在 prepare 返回前恢复或保持，commit 校验 owner/基面后才安装对应候选；drop/cancel 不改当前，旧取消不能覆盖新提交。保留有界 move、单 body 工作边界和已有后置失败回滚。实现可采用等价事务表示，不能用全环境 clone 或全程串行取消一切候选缩减原会话合同。

这只是 T-01 最小接缝的修正，session 的完整状态矩阵仍留 T-02/T-06，关闭/预算仍保留原 F05b/F09 责任。连续失败的计数不因修复轮命名重置；下一次先以状态表对账实现和断言，再提交精确关闭表。修后冻结根先受影响面复审，通过后才 T-02 全面换接；其余 F01/F02/09/10剩余/11剩余/12/14 OPEN 不变。整份 AC09 最终复审、merge-owned 冻结裸 t+tv/必要 consumer/实际触面门及 Spec/ledger/归档/guard 均 pending。
