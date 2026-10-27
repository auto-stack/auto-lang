# PLAN-828 r3 / 0b612fda1 受影响面复审

**needs_fix：上一轮六项事务失败已全部闭合；原 C05/P2 仅余公开原地发布入口的代际遗漏。** 不新增 finding ID，不重开 C01/C02/C03/C04/F05b。Plan 保持 executing0/11，暂不 T-02 全面换接。

受审 `plan-828-dev@0b612fda1ce004f0565a4f2e1057768f841a0f31`，基面 `de4cba7022a2dd9f2d03ee3dea91322a5088674c`，累计 T01 基面 `47894c2e6747c8c06032be7a3d40bf00b90a204d`。实施树 `D:/autostack/.wt/lang-828/auto-lang` 入口/收尾 clean。final 角色定向复审，非整份最终验收；reviewer=Codex /root 当前上下文，未实施本提交及前序 T01，未创建额外 agent。只写证据/主检出簿记，不修实施/合入/清树。

[Plan](plan-input.md)、[接口决策](interface-decision-input.md)、[SD](spec-delta-input.md)、[冻结输入](baseline.json)、[最终复核](final-state.json)。auto-down `c39ac7f9dacdc769fc4a5ad3b98ec896240b28e8` clean，本轮 Cargo 未消费该兄弟包；实际 manifest/配置/Spec/rlib 绑定见 baseline。Plan §§1–7、目标/profile/预算/阈值与 AC/V/C/SD/T/owner 保持。

## 旧控制已闭合，候选所有权重设计有效

新 FnCandidate 自持候选 ResolvedFn、产物、env_id 和 base_generation；prepare 临时换入后在成功/失败返回前恢复 current；commit 核 closed/owner/generation/槽后安装自己持有的新条目；cancel/drop 没有 env 参数。原 FnSlotSave/recheck_transactional 被删除。

[原八行输入](transaction-original.rs) 冻结原样保存。因旧 API 已删除，以 [适配版](transaction-controls.rs) 调用新 API，保留相同 literal 源码 `helper() return N; main→helper`、N=1/2/3/9 和冷编 HIR 状态 oracle：

| 原控制 | 本轮 |
|---|---|
| 顺序取消后合法提交 | PASS，current1→取消保1→提交2 |
| 成功 prepare 尚未提交 | PASS，候选产物2、current1 |
| drop 未提交候选 | PASS，current1 |
| 同基面双候选，取消 A→commit B | PASS，实际发布 B=3 |
| B 提交后迟到取消 A | PASS，保留3 |
| 同名同槽 foreign restore | 新模型不存在可传目标 env 的 restore；cancel 无 env 引用，原输入两环境1/9都保持，PASS；这是结构性消除错误操作，不伪称调用了已删 API |
| 同名同槽 foreign commit | PASS，source.foreign，两个环境不变 |
| closed 处置 | PASS，source.closed、函数数0、幂等 |

**适配重放8/8**，见 [raw](transaction-output.txt)。永久 transaction 六测试也覆盖旧八行及 candidate→candidate 的 stale 正反控，contract16/16。后置 verify 拒绝恢复保持。旧问题没有被改断言掩盖，本轮不否认上述真实进展。

## C05/P2 剩余：成功 recheck_function 未让旧候选失效

**原规则**：Plan §5.4/§5.7、R04 及 D03 要求旧候选不覆盖更新的当前结果；candidate 必须绑定真实发布基面。保留的公开 `recheck_function` 自身文档明确成功即替换 env 当前条目，供后续编辑/发射读取；当前没有禁止同一 CheckedEnv 使用两个公开入口的前提。因此“实际当前已更新”必须参与候选失效，而不能只数新 commit 方法调用。

**定位 / 确认原因**：

- [resolve.rs:803](D:/autostack/.wt/lang-828/auto-lang/experimental/ac-core/src/source/resolve.rs:803) 的公开 `recheck_function` 成功调用 recheck_swap 并丢弃旧条目，**没有更新 generation**。
- [adapter.rs:410](D:/autostack/.wt/lang-828/auto-lang/experimental/ac-core/src/source/adapter.rs:410) 只比较候选 base_generation 与当前 env.generation；[435](D:/autostack/.wt/lang-828/auto-lang/experimental/ac-core/src/source/adapter.rs:435) 只在 FnCandidate::commit 成功时递增。
- 所以新方法之间有 stale 保护，旧公开原地方法发布的新语义却没有进入同一失效轴。除显式 close（终态守卫）外，本轮已盘点 source 中对当前函数槽位的对外发布入口为这两条；prepare 的内部 swap 是恢复后返回的暂存，不应算发布。

**独立重放**：[publisher-controls.rs](publisher-controls.rs) `publisher_public_recheck_invalidates_old_candidate`，同一合法 Scalar 源、同名同槽同签名，真实公开入口，无私有字段修改：

1. 当前 `helper→1`，prepare 候选 `helper→2`，记录 base_generation=0；current 仍1。
2. 通过公开 `recheck_function` 成功把当前改成 `helper→3`；公开 emit 的 canonical HIR 与冷编3逐字节相等，确认真实新当前已安装。
3. 原候选2再 commit：期望受控 stale 拒绝、保持3。

实测：`after_direct_generation=0, old_commit_ok=true, current_is_new_direct=false, current_is_old_candidate=true`，旧候选把3覆盖成2。**FAIL**，见 [raw](publisher-output.txt)。

同文件两个合法对照通过：新 candidate commit→旧 candidate 被 source.stale 拒绝且保持3；旧 recheck 类型错误未发布→原候选仍可合法提交2。合计 **2pass/1fail、exit101**。这两个对照区分了真正发布、失败尝试及待提交候选；不是要求每次检查/prepare 都改变代际。

该项为**原 C05 的基面不变量残项**，范围只缩到现存两个公开入口的联动；不要求新语言能力/额外场景规模。当前 session 尚未换接 CheckedEnv，本轮没有声称产品 session 的 root/source/revision 已被污染。此前确认 recheck 为合法原语，保持其语义是正控；新增候选机制也须正确观察这个原语对当前状态的真实改变。

**关闭条件**：让所有成功对外发布当前语义的入口参与同一失效凭据；失败 recheck、纯 prepare 的临时 graft 与取消/drop 不发布。上例旧候选应拒绝且current3保持；两合法对照及原八行继续通过。可按实际实现更新代际或等价基面凭据，不能仅在文档禁止混用或删除旧公开能力。不要把递增放入 prepare 共用的临时换入步骤，破坏同基面双候选。

## 验证范围

| 定向选择 | 结果 / raw |
|---|---|
| contract / t01r | 16/16、4/4，[输出](contract-output.txt)，contract内transaction6是子集 |
| 原八行适配 | 8/8，[输出](transaction-output.txt)；原件保留并做哈希比对 |
| 发布入口联动 | 2pass/1fail，[输出](publisher-output.txt) |
| 旧 boundary / 额外 admission / 原关闭 | 11/11、2/2、9/9，[boundary](boundary-output.txt)、[admission](admission-output.txt)、[close](close-output.txt) |
| 主 session / r1 / core | 17/17、86/86、22/22，[session](session-output.txt)、[core](core-output.txt) |
| ac-core 六目标 | hir9/record21/scalar16/source_frontend14/text16/text_record18，全94过，[输出](ac-core-output.txt) |
| 四个修改 Rust 文件、diff | fmt0/diff-check0，[格式](fmt-output.txt)、[diff](diff-check-output.txt) |

无净新增 warning；probe四旧+shim旧doc warning不算新增。只运行定向目标，未整档t/tv/tt/tb/tu/taa/tf、docs_gen、正式性能/资源或旧62/798兼容全集；集合重叠不汇总唯一测试总数。命令 cwd=实施树；Cargo compiler-artifact 元数据选择精确 [库](linked-libraries.json)，不按 mtime 猜测：

    cargo test --locked --manifest-path experimental/ac-av-probe/Cargo.toml --test function_incremental_828_contract --test function_incremental_828_contract_t01r -- --test-threads=1 --nocapture
    powershell -NoProfile -ExecutionPolicy Bypass -File D:/autostack/auto-lang/docs/plans/attachments/828/review-t01-0b612fda1/run-controls.ps1 -Source transaction-controls.rs -Filter transaction_
    powershell -NoProfile -ExecutionPolicy Bypass -File D:/autostack/auto-lang/docs/plans/attachments/828/review-t01-0b612fda1/run-controls.ps1 -Source publisher-controls.rs -Filter publisher_

其余 replay：同 runner 的 boundary-controls.rs/boundary_、admission-controls.rs/admission_、close-replay.rs/close_；原样输入与脚本均冻结。

## 状态与下一步

C01/C02/C03/C04/F05b 的已测闭合保持；C05 旧六失败关闭、仅公开 recheck 发布联动仍 OPEN/PARTIAL，owner=T01。SD04 事务前置仍部分，无整份 SD/Plan PASS。SD01..06 SHA256 仍 `371a9298c285eb8a0d7309b77298ce9bdadb932e626cc82b7df3af7be3759623`，canonical/ledger 不改。

[最新 GLM 指示](glm-next-round.md) 仅要求修这个遗漏并保存三项正反控，不重做候选所有权设计。其他 F01/F02/09/10剩余/11剩余/12/14 及完整 session 原子发布/状态矩阵保持原 owner/OPEN。executing0/11；新冻结根受影响面复审通过后才 T02 全面换接。整份最终验收、merge-owned 冻结裸t+tv/必要consumer/触面门及Spec/ledger/归档/guard pending。连续失败历史保留；本轮已覆盖两个发布入口、临时准备、失败、取消/drop与close，不用逐样本补丁代替失效规则。
