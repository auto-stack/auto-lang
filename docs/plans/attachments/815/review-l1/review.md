# PLAN-815 M815-L / T-16 复审 L1：needs_fix

2026-10-20，Plan r3；phase review，只审新增账本恢复，不授予整Plan reviewed。

- 提交：`feacf79e8a9c52575a6f64dae3e068a3cc3ae1c7`；基面 `abad97371f5443ea25b007fa34074b882062163f`；唯一树 `D:/autostack/.wt/lang-815/auto-lang`，分支 `codex/plan-815-dev`，进入/结束实现均clean。
- 主合同：主检出 `12edd6ca1ccb917d2bdc34d561264df018ba1c71` 的r3；完整输入、真实loader/模型/脚本SHA见baseline.json.txt。
- 上下文限制：同一/root会话参与过前期合同和旧守卫候选，不声称另一个独立会话/模型；本次从新恢复工具、真实store、实际候选和独立失败注入重建证据；未创建agent、未修实现。
- 生产ledger始终为原v57，SHA `c24b174ee3c0eba25555c0a31f0c988a0a465af880c99388bf0f537925b03038`，本轮零写入。

## 已确认的正控

重新生成候选SHA `d60bd3fa8349959c2da14a052b85f3a9c345cb497ce3483cd9964fb52c6604c5`及转换规范化JSON SHA `c35615e23cd50502da030b659493890cf0958e0a7c623d8c6f082ba072c153f2`与送审一致（转换磁盘文件自身SHA不同，哈希算法明确是sort_keys序列化，并非逐字文件）。

真实`SpecsStore::load`加载候选6区1098项，真实save再加载内容相等；PascalCase SectionType修正确认有效。阶段四条source-sha256逐一匹配当前来源，未见来源漂移。两个工具自检通过，脚本diff空白检查通过。上述正控不能覆盖以下失败路径。

## Findings（全部 OPEN）

### F-815-L1-01 / P1：命令行丢失拒绝/失败退出码

位置：`scripts/plan815_ledger_recover.py:661-666`，AC-10/11，V-L3。

`cmd_apply`返回2/1，但main忽略返回值，进程以0退出。真实子进程两例：错误source hash打印REFUSED却exit0；不存在loader触发FAILED/回滚仍exit0。自动化据退出码将恢复失败当作成功。`cmd_convert`即使BLOCKED也固定返回0；如dry-run将阻断作为可观察报告退出0需明确其语义，apply的生产失败必须非零。

修正：传播子命令结果到进程退出码，并以真实CLI子进程断言成功/拒绝/失败及文件不变；只直接调用cmd_apply断言整数不足以覆盖入口。

### F-815-L1-02 / P1：并发保护和失败回滚会覆盖外部写入

位置：`scripts/plan815_ledger_recover.py:451-505`（重点495/503-504），AC-10/12，V-L3。

source hash仅在首次read检查，此后转换/备份/临时写入后没有有效并发保护。在实际copy边界注入另一写者的v58内容，apply仍返回0并以候选覆盖；在写后探针期间注入外部更新再返回失败，except见hash不同即把原v57写回，覆盖外部内容。两例均观察到external_survives=false。旧deposit候选的无条件回滚同样需要纳入修复/反例范围，不能只改正常路径。

修正：建立适用于当前写通道的单写者/冲突检测；在替换与回滚前验证当前内容仍属于本次写入，发现外部更新停止并记录准确残留，不能恢复覆盖别人；补实际检查到写入间、写后验证/回滚间两个反例。备份是否同字节不是对目标文件的CAS。

### F-815-L1-03 / P2：转换前像与保护断言不能证明原始信息保留

位置：`scripts/plan815_ledger_recover.py:214-221,280-283,375-377`，AC-10/11，V-L1..3。

实际v57的8个related字段原值（3个字符串化列表、5个null）在赋新值后才记录original，转换表original写成新列表；如P656-1原`"['PLAN-656']"`被记成`["PLAN-656"]`。P014-1改名在deepcopy之前直接修改source，转换表没有任何id改名条目，protections比较的是已被改过的“原件”。另在候选删除一个未声明转换的原title，protections返回[]，因为只检查key仍存在的分支。

修正：保持source不可变，逐项记录真实旧值及缺失/存在区别、明确ID映射并验证重命名关系；保护同时拒绝未声明删除/增加/改值及顶层/section越界变化。需要可重放原始ID和字段的区分性断言，不能仅以完整备份存在替代逐项转换审计。

### F-815-L1-04 / P2：schema镜像接受真实store拒绝的输入

位置：`scripts/plan815_ledger_recover.py:115-158,464-501`，AC-10/11，V-L2..3。

负控把候选version改为-1、一个tags改为[7]，schema_errors仍为空，而真实loader拒绝(integer7 expected string)。检查只认int/list外壳，没有u64范围、列表元素类型等完整store要求，也没有六section精确集合/ID与type配对检查；apply的真实loader还是可选，且只认退出码，不核对真实文档区/项身份。mirror不能作为完整前检证明。

修正：补足真实schema边界并让生产路径不能跳过所需typed验证/区项核验；给负u64、越界、bool、非字符串列表、区缺失/重复/错配提供合法对照与实际typed负控。不是要求修改auto-musk schema，也不是增加产品支持面。

### F-815-L1-05 / P2：合并两checkpoint且改变28个阶段字段，version仍57

位置：`scripts/plan815_ledger_recover.py:323-340`与`docs/plans/attachments/815/t15-recovery/report.md:89`，AC-10/12，Plan §5.6。

当前候选将兼容恢复与四条815语义upsert合成一步（28条stage字段变更），source/candidate version均57；彩排再执行deposit因已含四条而no-op，整个真实变更没有递增。合同要求恢复与四条upsert两个checkpoint、各次版本递增。报告请reviewer核准“恢复不递增”不能自行覆盖r3合同；即便结构恢复获准不增，已实施的阶段语义upsert也不能被幂等跳过掩盖。

修正：按当前合同拆分/记录并正确递增，或明确提出语义合同变更走修订；不能用本轮review pass静默豁免。重新生成候选、转换与彩排收据，旧d60/c356证据受影响。

### F-815-L1-06 / P2：实际恢复命令绑定了错误source SHA

位置：`docs/plans/attachments/815/t15-recovery/report.md:74`（§4 source表同错），AC-11/12。

命令写`c24b174ee3c0eba25555c0a31c0f988a0a465af880c99388bf0f537925b03038`，实际冻结是`c24b174ee3c0eba25555c0a31f0c988a0a465af880c99388bf0f537925b03038`；命令必拒绝，又受F-01影响会假exit0。这是发布指纹校对缺陷，不是账本新漂移。

修正：由冻结manifest生成命令/收据，校验三hash及stage/schema/loader身份，真实CLI以完全相同参数在副本彩排。候选与转换变化后全部重新绑定。

## AC / V 覆盖结论

| 项 | 结果 | 说明 |
|---|---|---|
| AC-10 / T-15 | fail | 可加载正控过；保留审计/错误类型/并发与版本路径未闭 |
| AC-11 / T-16 | needs_fix | 不核准生产候选hash或T-17 |
| AC-12 / T-17 | pending | merge-owned真实恢复/upsert/回滚收据，本轮禁止生产写 |
| V-L1 | partial | 实际loader/6区1098项往返过，rename与删除保护失败 |
| V-L2 | partial | PascalCase/当前候选过，类型负控错误接受 |
| V-L3 | fail | CLI失败传播、并发与回滚反例均未通过 |
| V-L4 | partial/pending | 四ID/sourcehash检查过；幂等入口、两个checkpoint和版本收据须按F-05重验 |
| 原A AC-07/08及R4 | preserved | 未变代码/精确Spec阶段pass保留；本轮非整A重新验收 |
| 原B AC-01..06 | pending | 原计划B任务与上游814依赖不提前放行 |

## 证据与下一步

可直接重放`review_probe.py`（入口为原树脚本，所有apply仅指向target临时副本），结果见results.json.txt、extra-results.json.txt、probes.log.txt、self-tests.log.txt；指纹见baseline.json.txt。测试总数不冒称完整语义覆盖。没有执行整档cargo t/tv/tt/tb/tu/taa或docs_gen；此前batch未完成收据仍独立pending，不在复审中重跑。

回原唯一树work一次修齐六项、补永久真实CLI/并发/前像/schema/版本对照，更新闭环表与r3候选/转换/命令后提交新SHA，再送T-16。T-15重开，T-16/17及T-14仍不勾；Plan executing **5/17**。不合入、不改生产ledger/canonical Spec、不归档不清树。
