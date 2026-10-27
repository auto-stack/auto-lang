# PLAN-828 r3 / 27aaa34d5 受影响面复审 PASS

**本轮受影响面 PASS；T01-C05/P2 CLOSED。无新增 finding。** C01/C02/C03/C04/F05b 既有已测闭合保持，当前 D01..03 接缝/候选前置可交 T-02 热路径换接。Plan 按用户交接保持 executing0/11，不授整份 reviewed、不授权 merge；F01/F02 等完整任务仍未完成。

受审 `plan-828-dev@27aaa34d565b11facab37008d42c6e509cc3e13d`，基面 `0b612fda1ce004f0565a4f2e1057768f841a0f31`，累计 T01 基面 `47894c2e6747c8c06032be7a3d40bf00b90a204d`。实施树 `D:/autostack/.wt/lang-828/auto-lang` 入口/收尾 clean；final 角色定向复审，非整份最终验收。reviewer=Codex /root 当前审查上下文，未实施该提交及前序 T01，未创建额外 agent。只读实施/执行定向检查及写主计划簿记，未修代码/合入/清树。

[Plan 输入](plan-input.md)、[决策输入](interface-decision-input.md)、[SD 输入](spec-delta-input.md)、[冻结指纹](baseline.json)、[最终复核](final-state.json)。auto-down `c39ac7f9dacdc769fc4a5ad3b98ec896240b28e8` clean，Cargo未消费该兄弟包；实际依赖 manifest/库/配置/Spec身份见 baseline。Plan §§1–7和任务/验收语义未改。

## C05 关闭证据

修复落在 [recheck_function](D:/autostack/.wt/lang-828/auto-lang/experimental/ac-core/src/source/resolve.rs:812) 的成功公开发布出口：recheck_swap 返回成功才推进 generation；失败不推进。候选 prepare 仍直接调用不推进代际的内部 recheck_swap，临时换入后恢复当前；FnCandidate::commit 仍核 owner/base/closed/槽、安装自身候选后推进代际。没有把临时检查误当发布，也没有修改已通过的候选所有权设计。

原 [publisher-controls.rs](publisher-controls.rs) **逐字节不变重放3/3**：

- 当前1→prepare候选2(base0)→公开recheck发布3：`after_direct_generation=1, old_commit_ok=false, current_is_new_direct=true, current_is_old_candidate=false`。旧候选不再覆盖新当前。
- 新候选commit发布3也使旧候选以source.stale拒绝，当前保持3。
- 失败的公开recheck不发布、不改变当前，原候选仍能合法提交2。

见 [raw](publisher-output.txt)。原失败转绿，两个合法对照保持；永久 contract 新增对应两项，连同既有 candidate 间 stale 控制覆盖三个判据。

[八行事务适配控制](transaction-controls.rs) 原样8/8：成功prepare不发布、drop保持当前、顺序取消/提交、同基面双候选、迟到取消不回卷、同名跨环境commit拒绝、无env参数的取消保持双环境、closed守卫。见 [raw](transaction-output.txt)；已删除旧restore API的含义/适配记录沿用原件，不伪称调用已删除入口。C05 的先前六失败及最后发布联动残项均在当前真实 API 与相同状态 oracle 下闭合。

已盘点当前语义写入/处置：公开原地recheck和candidate commit都更新基面；失败/prepare临时换入/取消/drop不发布；close由终态守卫约束。候选仍有界move一个新条目、无全env clone，未改body不因本次修复重建。当前 session 尚未接入新环境，本 PASS 不替代未来 session 的原子发布/完整状态矩阵。

## 冻结根上的定向验证

| 选择 | 结果 / raw |
|---|---|
| publisher 原三控制 | 3/3，[输出](publisher-output.txt) |
| 八行适配 | 8/8，[输出](transaction-output.txt) |
| 永久 contract / t01r | 18/18、4/4，[输出](contract-output.txt)；transaction8已含在contract18中，不重复相加 |
| 原 boundary / 额外准入 / 原关闭 | 11/11、2/2、9/9，[boundary](boundary-output.txt)、[admission](admission-output.txt)、[close](close-output.txt) |
| session / r1 / core | 17/17、86/86、22/22，[session](session-output.txt)、[core](core-output.txt) |
| ac-core 六目标 | hir9/record21/scalar16/source_frontend14/text16/text_record18，全94过，[输出](ac-core-output.txt) |
| 本轮两修改 Rust 文件 / diff | fmt0/diff-check0，[格式](fmt-output.txt)、[diff](diff-check-output.txt) |

无净新增 warning；probe四旧及shim旧doc warning保持，不作为新增。未整档t/tv/tt/tb/tu/taa/tf、docs_gen、正式性能/资源或旧62/798兼容全集；不冒充work摘要全复现。集合有重叠，不汇总唯一总数。Cargo精确产物/链接库与测试二进制哈希已冻结，原三控制/八适配/其他旧输入与上轮逐字节相等。

复跑 cwd=实施树：

    cargo test --locked --manifest-path experimental/ac-av-probe/Cargo.toml --test function_incremental_828_contract --test function_incremental_828_contract_t01r -- --test-threads=1 --nocapture
    powershell -NoProfile -ExecutionPolicy Bypass -File D:/autostack/auto-lang/docs/plans/attachments/828/review-t01-27aaa34d5/run-controls.ps1 -Source publisher-controls.rs -Filter publisher_
    powershell -NoProfile -ExecutionPolicy Bypass -File D:/autostack/auto-lang/docs/plans/attachments/828/review-t01-27aaa34d5/run-controls.ps1 -Source transaction-controls.rs -Filter transaction_

其他原样目标仍用同runner的boundary-controls.rs/boundary_、admission-controls.rs/admission_、close-replay.rs/close_，选择数见表和raw。

## 验收 / Spec / 下一责任

AC04/06、§5.4/5.7、D03/R04 的**本轮候选/发布接缝子集PASS**，C05关闭；AC02/05完整授信及单body/真实门等既有子集保持。SD04的该子轴通过，SD01..06 SHA256仍 `371a9298c285eb8a0d7309b77298ce9bdadb932e626cc82b7df3af7be3759623`。未授整个SD或Plan最终PASS，未改canonical/ledger。预先规范增量仍需T09按最终真实实施回写/最终复审，不将接缝证据写成全链路已交付。

下一责任是 **work/T-02**，按[当前GLM指示](glm-next-round.md)把生产session热路径换接到既有单body接口，保留原语法/profile、完整入口、生产者观测和事务/关闭不变量。本轮没有启动实施或发送其他任务消息。T01完整任务不因定向子集PASS自动打勾，按用户保留executing0/11；F01/F02仍OPEN，A07/依赖、F09、F10/F11剩余、F12正式资源/性能和F14仍归原owner。

整份AC09最终验收、merge-owned冻结裸t+tv/必要consumer/实际触面门、Spec/ledger/归档/guard仍pending。本次不是phase landing/merge批准；下一轮代码变化须绑定新冻结根按受影响面复审。
