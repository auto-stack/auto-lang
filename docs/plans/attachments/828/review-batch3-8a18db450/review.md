# PLAN-828 第三批 T-07/T-08/T-09 独立复审

2026-10-22；`review_scope: phase`；**needs_fix：2 项 P1、3 项 P2**。第三批不能关闭，T-10 尚不具备进入条件。此前第一批及第二批已闭机制保留，不要求重做候选模型或单元派发。

## 冻结与证据边界

- 实施树 `D:/autostack/.wt/lang-828/auto-lang`，`plan-828-dev@8a18db4503a6b30e53a0affa9cb17c4126d35e6d`；批次基面 `cccb09b1f2b77e14cb410811eb4493247e2ed787`。包含 f7d1091b3、f2593bf75、8a18db450 全部提交。
- 主检出交接 d89e1dff36c13839d20ad35ae9f76088ab7c05d7。主 Plan r3 与接口决策先冻结到 plan-input/interface-input；四项交付文档另存同目录，未拿 worktree 中旧 Plan 替代主 Plan。
- 本 Codex 独立审核上下文未实施该批、未启代理；实际身份未暴露部分不编造。未改实施源码、canonical Specs 或 ledger，未执行整档/合入/清树。
- 入口、结束实施树 clean，HEAD/依赖/520 项源码配置与 Spec、Design43、冻结合同及技能输入完全一致，见 baseline.json / final-state.json（matches_baseline=true）。auto-down=c39ac7f9dacdc769fc4a5ad3b98ec896240b28e8，probe未消费。
- 审核范围为第三批完整已交付路径，包含其使用的既有 formal_worker 父侧 consumer；不因该文件早于本批提交而忽略正式入口的原合同缺口。旧 PASS 不覆盖新替换的正式测量 consumer。

## B3-C01 / P2：未采纳 A07 后忘记旧 root，破坏有界存储

**归属：T-07；AC-06/08、V-07、F-09/10/11。新 root 回收机制的相邻恢复出口缺陷。** 原合同及本批承诺每函数至多一个现行 HIR soft root，512 修订/GC 水位有界。

`hir-roots-controls.rs` 使用真实公开 warmup_persisted→restore→edit→GC；三函数 helper/main/tail，helper 返回7、main返回helper+2。原源码持久化后，只在 `return 7` 间增加合法空格，随后恢复；ABC 实际 admitted=3，而 A07 因当前 span 不同被安全拒绝，skip_span=3/adopted=0。再真实编辑7→8并成功持久化，soft root **3→4，GC 后仍4**，retired=0；违背三函数上界。无位置变化的合法正控 adopted=3/retired=1，root **3→3→3**。原始退出1，1/2通过，见 hir-roots-controls.log。

已确定原因：session.rs:1061 只在 HIR 成功采纳后设置 persisted_hir_cv；1524 的退役又只认这个 tracker。缓存的可采纳性与本项目拥有的旧 store root 被同一个变量混用，合法 span 拒绝出口丢失后者。新旗舰两次 restart 只断言 ABC 单元全部重验，未检查该拒绝出口，因而旗舰通过不能排除此反例。

**关闭 oracle**：分清已发布 root 的所有权与 untrusted HIR 能否采纳；原反例、无变化正控、代表性的多次位置变化恢复/内容回访之后，成功发布后每函数至多一个所属现行 root、GC 水位有界、当前结果正确、用户哨兵与 required 闭包不损。不能放宽 A07 span/semantic 门或强制采纳来掩盖泄漏。覆盖本项目/支持 profile，不要求清理不属于会话的用户行。

## B3-C02 / P1：正式 judge 未绑定当前修订与准确安装目标

**归属：T-08；AC-07、V-08/09、§5.5 的逐次 actual/current publication。原必需验收门缺口。**

judge-controls 中的 stale/install 两例调用真实 `run_formal_session_judge`。审核 worker 保留冻结生产 session_serve 循环、真实 warmup、真实编辑 helper7→8、真实 AV 执行结果10，只分别改变发布中的 revision 或 install_set：

| 控制 | 应有判决 | 实际 |
|---|---|---|
| 实际修订1却发布revision0 | 当前发布身份红 | gate=true、correct=true、structural=true、revisions=[0]、failures=[] |
| 改helper却发布install_set=[d_tail] | 目标不匹配红 | gate=true、correct=true、structural=true、failures=[] |
| 原循环正控 | 绿 | 绿 |

原因已由源码确认：formal_worker.rs:226–278 只核 req_id/value/ok；revision 仅入报告，不核期待进度；安装只核 `len()==1`，changed_fn 由回包自报并决定排除哪些函数的计数。JudgeEdit 无独立期望目标约束。安装集合的基数不证明安装身份，正确数值也不证明发布属于当前修订。

**关闭 oracle**：父侧从批准输入/已固定会话建立逐请求修订和目标的独立预期，在当前发布门核对；两例必红、真实合法多修订/内容命中仍绿，拒绝与缺样本不能悄悄缩小序列。ready 恒定身份可预先核，变化的 actual 仍须在每次计时窗内。无需增加语言支持面或臆造密码学协议。

## B3-C03 / P1：实际复制执行载荷仍获正式结构门 PASS

**归属：T-08；AC-02/07、C02-05、V-09、§5.8 F-02。原必需反控未进入正式 consumer。**

`judge_actual_copied_payload_dispatch_must_fail` 不是篡改一个布尔值：在真实生产编辑成功后，对实际所有 FunctionCode/MirFunction 深复制、重新 link_with_maps，并让 **AV 真正 execute_program(copied)**。审核在执行前断言未改 main 的 `dispatched_payload_same_object=false`、same_content=true；返回值仍10、真实阶段计数不变。正式 judge 却 **gate=true / correct=true / structural=true / failures=[]**。

这正是合同规定必须检出的“结果正确但复制未改 payload”机制。formal_worker 的结构判据只有阶段和 whole-module 计数及安装集长度，没有消费实际派发对象 identity/generation/content 轴。既有进程内 identity oracle 是有效的，但其单测通过不等于正式父侧门已使用它。

**关闭 oracle**：观测挂在真实执行 consumer，并把未改对象/代际/内容的前后证据纳入同一正式结构判决；上述实际副本执行必红，原单元复用正控绿。不能改成仅复制未执行包装对象、手填零计数/恒真身份字段、另一个不影响正式总门的测试。覆盖既有普通/step/resume消费合同，修改共用观测时重放其代表性兄弟入口，不重新设计已通过的核心派发。

审核 mutation 方法：generate-worker-variants.py 从冻结 worker 抽取完整 session_serve，只有脚本中明示的模式选择、发布字段变体/执行副本/反控值变化；不改 WT。所有模式编译为同一内容的审核 exe，按自身文件名选择模式；hash见 worker-variants.json，原件hash见 worker-variant-source.json。用同一个生产父侧函数判决，非另造模拟 gate。clean、whole-counter、wrong-value 三控均按预期；stale/install/copy 三控失败，judge-controls.log 为3过3败。此方法验证门对明确故障的区分力，不声称可防任意恶意 worker。

## B3-C04 / P2：计时提前结束，必需 observations/raw 未交付完整

**归属：T-08/T-09；AC-07、V-08/09、§5.5。原测量/证据要求，不是新扩展。**

1. formal_worker.rs:226 在 recv/反序列化后立即取 `verified`；之后才检查 reply类型、req_id、结果、结构、计数差分。报告的 edit_to_verified_ms 实际排除了父侧这些必要校验。原合同结束点为父侧校验当前结果/诊断并发布，不能把同一请求的必要验收移出窗。该静态顺序足以证明端点不符；不据此推断补齐后必超1秒。
2. EditResultMsg 有 worker_edit_us、完整 counters、changed_fn/install_set、diagnostics，但 JudgeSample/raw_jsonl 丢弃这些，只保留总时长/聚合数/长度；没有约定的阶段wall/IO、输入与二进制/配置身份、初次装载单列证据。未保留完整回包与失败上下文，不能复核每个阶段/目标。
3. 正式永久测试在191行产生raw，只检查行数随后丢弃；末尾只打印两条汇总，并未打印或落盘raw。interface-input:260“raw_jsonl全留在测试输出”与实际相反。审核独立保存了此次已有字段的30行raw及report，但不替正式入口补交仍缺的字段和失败记录。

**关闭 oracle**：必要父侧判定完成后才截结束时间；单列ready/恢复和输出报告成本；正式路径可留存完整成功/失败观测（含缺样本/异常回包的失败上下文）、所需阶段wall/计数/IO、输入/worker构建及配置身份；保留30样本和失败控制的可重放证据。可以采用现有observations通道或轻量结构，不要求通用平台。反控必须让同一正式总门红，不能只校验 JSON 行数。

## B3-C05 / P2：T-09 “真实 API/生产路径”文本仍有失实

**归属：T-09；AC-09/10、SD-01/05/06。收尾文档缺陷。**

- successor-interface-contract.md:21 写 `resident_weak(name, gen)`，实际 session.rs:1154 接口只有 name，观测当前驻留对。照文档编写后继消费者不能调用。
- spec-delta-r3-text.md 的 SD-01 把热路径写成 prepare_candidate_document→prepare_function_candidate→miss/hit；实际 session.rs:1994/2131 是前者返回候选文档后先判hit/miss，miss才直接 verify_function。session 未调用 prepare_function_candidate。不能把另一个公开接缝说成生产consumer。
- coverage-manifest.md:56 的 b1=13 与冻结实际选择11不符；正式raw/observations已交付与可选分类还须按C04纠正。

**关闭 oracle**：在最终修复代码上重新逐项绑定真实符号、参数、miss/hit顺序、case选择与raw入口，具体Spec delta与后继合同一致。修文档描述真实API，不为迎合错误文档新增无用参数或重复验证；修订观测文本后核SD05/06仍是proposed，由merge沉淀。

## 实测通过的范围与未授予的结论

| 面 | 本次独立结果 |
|---|---|
| 新完整存储轴512/真实GC/两次restart/用户哨兵 | 1/1，270.88s，full-storage-512.log；不包含C01的合法A07拒绝出口 |
| 正式冻结负载30修订 | 全部数值正确，现有gate绿；P50=339.7531/P95=351.5037/P99=362.5712ms；受C02–04限制，不能授正式双门PASS |
| 同798双线预算口径 | A=211644416B≤234881024B，B=0≤134217728B；worker峰值210206720B/父侧2400256B；formal-budget-verdict及frontend-attribution保存 |
| permanent mig/t07/b2/b1 | 9/9、8/8、14/14、11/11 |
| session/contract/t01r/r1 | 18/18（显式跳3个512，新的另跑）、22/22、4/4、86/86 |
| 原迁移/缺块闭包/GC目录独立控制 | 5/5、2/2、5/5；R5已闭保持 |
| T08脚手架 | 3/3；正式战役由独立保存raw的wrapper另跑1次 |
| close继承控制 | 96/96（原include链，包含r1及关闭相邻控制；有重叠，不加总宣传） |
| 7个改动probe Rust文件格式/批次diff-check | 0/0；shim只改注释，整文件格式漂移为已登记既有债，不声称全部8文件rustfmt绿 |

正式数值记录见 formal-samples.jsonl / formal-report.json / formal-review.log / formal-worker-identity.json；A/B方法沿用批准的798残差口径，不因B=0额外改变预算要求。这里证明的是本次采样数值，不是无限负载的普遍承诺。官方测试生成raw但不输出是C04，审核额外保存不将该缺口自动关闭。

未重跑旧两个512、core/ac-core全套、798八套全体；不把work报告作为本次复现。前批未变核心机制的既有证据仅在baseline锁定的输入边界延续，第三批新机制由上述直接控制覆盖；T-10整份逐AC验收仍pending。当前早已needs_fix，不执行merge-owned整档来重复确认未完成的验收。

AC-01..05受影响合法/拒绝/派发子集保持，AC-06/08因C01为partial，AC-07因C02–04为fail，AC-09/10仍pending且T09文字受C05阻塞。SD01/05/06为needs_fix；SD02/03/04已通过的核心不重开，但不授整份delta最终pass。既定Spec目标路径及GOAL-002元数据保留，未沉淀canonical。

## 关于两项“非必须扩展”的裁定

冻结主性能场景已有30样本，超过既定场景的额外性能战役可后继；函数前中后、槽/行长/调用递归的既定功能覆盖仍由原矩阵负责，不能因纵向30样本而删除。**observations/raw中§5.5明确要求的内容不是可选扩展**。专用漂亮报告或固定落盘CLI可后继，但必须有可留存、可复核的原始成功/失败证据；当前纯内存生成后丢弃不满足此要求。shim既有dead-code/格式债保持归原owner，不要求本批全仓格式化。

## 复现与下一步

先在冻结WT按cargo-artifacts中的同名目标重建，再由本目录执行：

```powershell
powershell -NoProfile -File ./run-controls.ps1 -Source hir-roots-controls.rs -AllLibraries
python ./generate-worker-variants.py
powershell -NoProfile -File ./build-worker.ps1 -AllLibraries
powershell -NoProfile -File ./run-controls.ps1 -Source judge-controls.rs -AllLibraries
powershell -NoProfile -File ./run-targets.ps1
powershell -NoProfile -File ./run-storage.ps1
```

前两组含当前应红控制，分别退出1；不能将外层脚本继续执行的exit0当测试通过。build脚本退出码/worker hash、Cargo确定的exe/rlib选择和日志均保留。formal-review.rs 使用PLAN828_REVIEW_WORKER（Cargo返回的worker路径）与PLAN828_REVIEW_EVIDENCE（本目录）保存实际report/预算；可用同一run-controls重放。复现脚本使用Windows绝对路径是当前仓约定，并非新增产品API。

同树work一次修齐C01..05，先给各组“根因→代表性兄弟出口→机制→原红例/合法正控→余界”关闭表，再冻结包含全部修改的单一提交，第三批受影响面复审后才T-10。无需逐finding送审，也不撤销已闭前批机制。Plan保持r3/executing0/11；最终审查通过后merge-owned裸t+tv/必要consumer触面档、tf到期主检单实例及Spec沉淀/归档/guard仍pending。
