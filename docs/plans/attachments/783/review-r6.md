# PLAN-783 r1 · independent final R6 · pass

2026-10-16（Asia/Shanghai）。本次独立Codex复审上下文，未参与实施；未派生代理或联系其它聊天。
**代码、定向验收与Spec增量审查通过；merge-owned完整closeout仍pending。**

## 冻结基线

- 同树 `D:/autostack/.wt/lang-783/auto-lang`，branch `plan-783-dev`，受审commit
  **aa8c666caa793891c9085efa3f8675ef8f6fe7d1**；本轮base
  `4ca6f3a0f6c2415d693a887a6e19b3a760fc128d`；原base
  `ec934b0006acb9b9838def82d0fcaf438ae0153f`。进出clean、HEAD未漂移。
- main HEAD `a37557764d1b258ff00bbfb8de5ca05602940065`，Plan r1回写前SHA256
  `C756ABD94FD713BF0A7891D53216D243C8FD73BF22792CD1C32CFC7F81BE12D8`。
- ac-core tree `bccb77d30131182129213f234cdcc63f40855fcc`、auto-cache tree
  `c4524240cf1a0e6c50a9176fe673cdf2527076e9`、probe lock blob
  `93582090320f0b07fd6c12d83c15f867ec1d3366`与R5相同；聚合脚本blob
  `078a9270e5be7f16fb1f5573c031865935798940`。
- canonical Spec SHA256：auto-ac/project
  `9336DFD87258DBBB037F2A94C97C40BB7D71F4CC346F51172C7D880E2C9363A8`；auto-cache/project
  `FA52F8DF1F5D8FB9671AE3762DE6EC2CA39EAAD607DE07411E51D28F23DBA8A2`；auto-hir/stage-contract
  `581D0C22408DFCB18667009158ECC364D08AE76F5C4CE3D8EFEB6DCBE9A62C5C`，均未变。
- [冻结SD](review-r6-spec-delta.md) SHA256
  `EB210819336C0D80D1A4590155E6E59B42F65AD9B0B040CE951FB2C538875080`。
  auto-plan-review skill SHA256沿用已核版本
  `0D68BA05276AFF05A143C8F27010EE02B0A9BDED14C27A0133D3A774C20F0CF3`。

## 复验与R5闭合

`RUSTC_WRAPPER=''`，运行 `scripts/verify-ac-artifact-probe.ps1 -Scope acceptance`：
**PASS，18族93 passed、0 failed、1白名单ignored**，含R1–R5五层回归，
[原始输出](review-r6-acceptance.txt)。work收据写17族，实际含新增R5家族为18族，
通过数93一致，以原始摘要为准。cargo fmt --check、git diff --check均通过。

| R5 finding | 原例与完整不变式验证 | 结论 |
|---|---|---|
| F-783-R5-01 | 预算锁内计staging+既有CAS.tmp+本次双副本；失败清理仍在锁内；原文反例重编译当前库及独立清理受阻兄弟见下 | closed |
| F-783-R5-02 | 逐行对比提交JSON六场景与人读表，GC428/621/632及22趟77删73重叠一致；restart219与hot271比较方向正确；peak明确为成功预留上界 | closed |
| F-783-R5-03 | 两处unused项已删除，完整验收无探针新增warning；仅未改shim-metadata:213基线unused_doc_comments | closed |

R5独立源码**原文不改**，使用[既有重放入口](replay-review-r5.ps1)重新链接本轮冻结库：
[输出](review-r6-prior-counterexamples.txt)。第一次3MiB写入耗尽重试后staging=0、残留CAS.tmp=0；
第二次在途staging+CAS.tmp=**6,291,552≤8,388,608**，最终成功。

本轮独立补充[源码](review-r6-controls.rs)、[输出](review-r6-controls.txt)：
预先持住同一CAS.tmp，允许读写但拒绝删除/rename，使失败出口的清理确实受阻。
失败后残留**3,145,776**；随后3MiB写入受ResourceExhausted拒绝，小写入成功，
实际临时总量仍3,145,776。释放句柄后recover删除2个orphan（6,291,552字节），
tmp=0，普通3MiB写入恢复成功。这验证了“清理失败但残留必须吃配额”的路径，
而不是仅依赖恰好清理成功。崩溃残留6MiB→大请求拒绝/小请求通过的永久兄弟也在本轮验收通过。
重放方法：`replay-review-r5.ps1 -InputName review-r6-controls`，只创建独占临时数据。

报告发布[四组控制](review-r6-gate-controls.txt)也重放：valid exit0并发布；invalid-report、
no-output、publish-blocked均exit1且旧报告hash不变。这是validator测试，不能替代真实closeout。

本轮阅读全部差异、预算计量/锁序/失败释放路径、新回归断言与SD；没有发现合同内开放阻断。
先前R1–R4的准入、布局、纵线、保活、namespace及GC状态矩阵结论，在相应代码/依赖/输入不变、
本轮重跑相关验收及所有历史永久回归的前提下复用；修改的预算和脚本注册路径按本轮重新验证。
没有将旧例闭合当作整条失败不变式的证明。无新的遗漏、未批准延期或绕过验收的补丁。
固定tmp的生产并发风险、pack/调度/真实DLL代际等D-783-A..E仍是明确的探针外后续工作。

## AC、Spec与交接

| AC | 结果与证据 |
|---|---|
| AC-01 / T-02 | pass；lib14、identity4、admission7及R1/R2准入控制通过；A00–12合同/实际codec子集未漂移 |
| AC-02 / T-03 | pass；layout6及历史边界回归通过，128MiB/页读/热点预算断言保持 |
| AC-03 / T-04 | pass；native_resume3、invalidation5及当前来源/持久信任回归通过 |
| AC-04 / T-05 | pass；reachability4、concurrency3、recovery4、gc_budget4+1白名单ignored、store_boundary5及历史回归/本轮独立失败组合通过 |
| AC-05 / T-06 | pass；perf2、提交六场景30样本合同通过，报告与JSON对齐、峰值含义清楚；冻结再生留merge |
| AC-06 / T-07 | pass；dep_audit5通过，SD-01..05与canonical输入相容、生产入口零切换 |
| AC-07 / T-08 | pass；独立final审查、零开放阻断、commit/依赖/SD绑定完成 |
| AC-08 / T-09 | pending；完整closeout、知识沉淀、归档合入与清理未执行 |

supersedes：`docs/specs/auto-ac/project.md`、`docs/specs/auto-cache/project.md`、
`docs/specs/auto-hir/stage-contract.md`；new：`docs/specs/auto-ac/artifact-contract.md`、
`docs/specs/auto-ac/artifact-probe.md`。SD-01给登记/身份合同，SD-02给运行/预算/限制，
SD-03给实验导航，SD-04给复用差距，SD-05给持久HIR重验/provenance规则。
现有目标版本见上述指纹，无冲突；touched_goals=[]合理，独立实验不扩大语言或自举闭包。
canonical Specs和live ledger本轮未改，merge按冻结SD沉淀。

Plan设置 **reviewed 8/9**，T-01..08完成，T-09 pending；next_responsible_role=merge。
当前总diff无crates/**或ac-core源码改动，Category A，严禁根cargo t/触面档/docs_gen；
本轮未执行它们。**合入前仍须在上述冻结commit执行完整probe closeout（512churn+
30样本perf再生/冻结来源与发布验证）**。本轮没有重跑该merge-owned载荷，也不以work
历史458s/30样本收据或mock控制替代。门禁若失败且修复代码，必须新提交、受影响复审后重跑。
随后由merge技能负责Spec/索引沉淀、归档合入、批量回归到期判定与guard清理。
本轮未修改实施代码、未merge、未归档或移除工作树；review pass不表示交付收尾完成。
