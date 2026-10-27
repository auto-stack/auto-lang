# PLAN-828 第二批 R2＋T-07 三切片复审

2026-10-22，review_scope=phase；**needs_fix**。B2-C01/P1、B2-C02/P1、B2-C04/P2在受测边界 CLOSED；B2-C03/P2原例通过但不变量仍 OPEN。T-07 新增两项 P2：T07-R2-01（manifest 精确性/格式），T07-R2-02（required 引用未落地）。没有新增 P1。

## 身份和限制

实施树 `D:/autostack/.wt/lang-828/auto-lang`，`plan-828-dev@cfa67eeceb4555e55b978354d3fd7ab508e80664`；diff base `a66d9d9e54e4db1de90edc078ed7d23d161e00d5`，包含 T-07 的54c5f1d3d/bae9f15f4/429e244ee及四项修复。入口clean，测试构建时源码与冻结一致。Plan r3/executing0/11；主Plan、interface、Specs/Design43、技能、516项源码/配置的哈希与依赖见 `baseline.json`；auto-down=c39ac7f9dacdc769fc4a5ad3b98ec896240b28e8，本probe未消费。

本Codex审核上下文未实施本批；从生产代码、原样重放和新独立输入判断，不用work自报PASS替代复审；未启代理、未编造模型/会话身份。没有修改实施代码或canonical/ledger。

**结束发现并发T-08 WIP，不能称入口结束一致**：session.rs增加SessionCounters::snapshot，lib.rs注册formal_worker，并发av_probe_worker/formal_worker修改见 `concurrent-wip.diff`、`concurrent-wip-status.log`、`final-state.json`。原diff/HEAD源码不回滚、不覆盖。`final-state.matches_baseline=false`，结论仅绑定已构建的cfa，当前WIP未审。

证据时间/库边界：定向7目标与原反例构建完成11:28:47，原例重放11:29、T07探针11:31–32、投影反例11:32:48使用该冻结构建的rlib；原普通rlib直到11:34:33的cache-tier才被重编。lifetime控件11:34:11仍链接旧普通rlib。`control-binaries.json`、Cargo artifact、target-results、linked-libraries保留对应身份。`commit-component.log`的lib-test重编时已存在只增加计数访问器的WIP，且cache-tier后跑时已包含formal_worker，**这两项只作受影响代码未变的补查，不作为精确cfa冻结PASS证据**；其中新增WIP warning不归本批。停止进一步构建。五文件格式另从`git show cfa:<file>`导出重核，结果0，不以当前WIP格式代替冻结格式。

## 旧四项关闭审计

| Finding | 原反例 / 本轮机制 | 结论 |
|---|---|---|
| B2-C01 P1 | 全单元owner(schema+module)一致门；restore按会话owner而非decode占位符stamp。foreign原例拒绝，同owner替换43、恢复后编辑、六槽序/递归正控保持 | CLOSED（本次已测单模块owner边界） |
| B2-C02 P1 | 遍历实际Call-import消费点，构造可执行snapshot前必须有绑定；合法lower成Trace正控8，原mark_b合法ABC但未绑定的Call在link即拒 | CLOSED（当前MIR指令形态） |
| B2-C03 P2 | 原重排和真实源编辑反例已通过；新func_slots按payload裸func编号建立全局表，编号冲突时后插覆盖 | 原例关闭，不变量OPEN，见下 |
| B2-C04 P2 | dispatch以st.steps-seg_start核切片预算；原Some(1)完成、Some(0)零步；独立跨close旧链有限切片仍在累计6步fuel退出并释放root/code | CLOSED（当前步数/累计fuel与生命周期边界） |

`link-controls.rs`和`execution-controls.rs`从R1原样复制，8/8＋4/4，未改oracle。永久b2 13/13、T07 6/6、b1 11/11、session18/18（明确跳过两个第三批512）、contract22/22、t01r4/4、r1 86/86全部通过。集合重叠不加总。原关闭6＋3、core22、独立生命周期2通过。相邻负例说明套件绿不能代替完整机制关闭。

## B2-C03 / P2 残项：不同payload拥有相同生产槽号时投影再次串图

**原规则**：§5.3局部坐标与跨函数逻辑引用独立于其他函数长度/顺序，C02-02/03来源正确，F-04身份不可重写；本轮修复声称图随拥有payload的代码对象选择。

**可重放输入**：`projection-controls.rs::projection_distinct_admitted_functions_sharing_producer_slot_cannot_cross_maps`。两份真实合法源码，A顺序helper/main/tail，B顺序main/helper/tail。各自warmup、ABC准入均成功，owner完全相同；取B的helper（实际生产slot=1）与A的main（实际生产slot=1）及tail公开link，helper绑定图+100、main图+200。无修改MIR/ABC/span、无不可信构造器。它不是要求session把声明重排作为body快路径，而是检查公开checked linker对其接受的合法单元组合：可正确投影，也可有依据地拒绝不可表示的组合。

**期望/实测**：helper trap应159..164，或link在发布前受控拒绝该歧义；实际link Ok，普通与step→resume(None)均返回 **259..264**。同owner不能证明裸数值生产槽全局唯一。

**已证实原因**：`core/src/program_snapshot.rs`的func_slots循环，对每条span做`insert(sp.func,si)`，重复键静默覆盖；`av::map_span`再按此结果取图。新注释“two units can never share one”没有验证/构造保证。

**关闭**：来源选择绑定实际执行函数/代码对象，或在受检link明确验证并拒绝无法表示的歧义；不能靠输入名单/两源码hash特殊分支，也不能改写共享payload。保留原R1两种重排正控、六槽序值、不可变身份、副本反控、旧pin映射。此为原C03不变量的修复残项，沿用ID，不另增finding。

## T07-R2-01 / P2：manifest目录不是精确集合，格式字段未消费

**原规则**：F-11/T-07、V-04/07及本轮冻结接口文档要求注册root在当前owner/entry/caps下**精确描述每函数内容版本**，坏manifest不能授权warm；持久化格式要有版本边界。本轮代码自身定义format=1。

**输入/实测**：`t07-controls.rs`从真实persist生成的合法envelope读出manifest，保持owner/entry/caps，仅①三条units中用helper替换main条目（重复helper＋遗漏main）；②独立把format改999。通过公开put_enveloped以新key存入正确owner envelope，移除旧session-manifest roots，只注册该mutant root；unit行仍是合法原件。两种输入都 **matched=1/rejected=0**，缺项反例仍admitted=3。期望rejected=1、走冷回退；完整原件的正控matched=1/admitted=3。

**已证实原因**：`session::manifest_describes_program`先filter_map（坏条目会被静默丢弃），再以长度相等＋listed每项能在expect里找到作为“exact”；未验证唯一性或双向覆盖，因此重复能补齐长度。`format`未读取，未知版本按v1理解。

**风险边界**：当前ABC/HIR独立验证仍保护值正确，未报为任意代码授信或P1；缺陷是F-11选择门/目录证据失真。

**关闭**：严格解析已支持格式、完整且唯一的def→cv集合，坏/重复/缺项/未知格式走原冷回退；合法条目顺序变化、matched/absent/rejected自愈和旧手工partial store保持。把两个反例永久收编，补一组合法重排正控，不能仅增加原始数组长度检查或忽略非法行。

## T07-R2-02 / P2：manifest只成为额外root，没有required数据引用

**原规则**：T-07/F-11的manifest required引用/驱动恢复、V-07真实GC和共享生命周期；Design40/783的required边保持被选根的必需数据，provenance不强留非必需上游。不是新增自有GC实现的要求。

**路径与合法正控**：`persist_manifest`的units只写def/name/cv，`put_enveloped(...,edges=vec![],Some(project))`，再register_root为session-manifest。每个工件也因Some(project)获得独立soft根。`t07_review_live_manifest_required_references_survive_real_gc`实际触发store GC时live=10/deleted=0，warm仍admitted=3；这证明soft根保活，**不证明manifest required边存在**。

**区分性反例**：`t07_review_manifest_as_required_root_survives_retiring_individual_soft_roots`保留真实session-manifest root，仅使用store公共remove_roots(project,"soft")退役独立soft根，然后run_gc(high=1,low=0)。low=0是783自身生命周期控制用法，用于回收全部dead候选、不允许删live闭包；不是提高本Plan生产限额。实际retired_soft=10、deleted=9、live=1，manifest仍可完整读取，但恢复 **admitted=0**（冷编服务）。若manifest真是required目录根，当前必需ABC/来源闭包不应只因独立soft根退役而丢失。

**已证实原因/边界**：生产manifest完全没有required edges，也没有指向工件ID的目录；当前warm仍重新按冷源算key查各行。所有历史内容独立root的积累还使普通GC“全活”而无法证明稳定回收。本轮不把独立soft根存在本身报错，也不声称已测512磁盘收敛或用户未注册行有永久保活特权；反例仅检查保留manifest根的必需闭包。

**关闭**：按既有shared-store API建立并消费可验证的当前目录/required引用闭包（至少恢复必须的ABC及来源，HIR按既定再生/引用分类，不强制一律required），区分当前必需数据、旧root和可回收历史的责任。永久控制同时保留soft根正控、manifest-only required根GC正控、合法恢复结果/来源、共享其他project的合法root/用户哨兵。不能仅依赖每工件永久soft根掩盖缺边，也不能修改783全局GC语义或把回冷正确值当作required保活PASS。

## T-07 三切片覆盖与未宣称范围

| 切片 | 结论 |
|---|---|
| A Weak观测/回收矩阵 | 已测PASS：pin旧root运行与最后释放、退役history真实code死亡、候选消费、close保活与最后释放；独立补fuel/trap旧PausedRun退出。原哨兵浸泡/恢复字节相等通过，但没有触发GC，不能单独证明存储GC合同 |
| B manifest roots/恢复 | matched/absent/rejected及自愈原例PASS；精确性/格式与required图FAIL（上两项）。完整F-11不可关闭 |
| C resident_inventory | 计数观测/相同body形状下超cap后水位与close归零PASS。只计函数数、当前HIR body节点、cold_bundle节点、history条目；不是所有驻留字节/查询图/索引/旧代峰值的完整账本，F-09双线真实峰值仍T-08，不能以该计数替代 |

本请求涵盖新增三切片；初始T-07两个512旗舰负载未在本次重跑，明确pending后续受影响最终验收，未借历史值冒称本轮PASS。T-08正式worker/IPC和T-09整份清单/SD仍未审。

## 复现与证据

`run-controls.ps1`选择本次Cargo artifact的明确rlib（AllLibraries只增加所需直接依赖），不随机选库；从本目录执行：

```powershell
powershell -NoProfile -File ./run-controls.ps1 -Source link-controls.rs
powershell -NoProfile -File ./run-controls.ps1 -Source execution-controls.rs
powershell -NoProfile -File ./run-controls.ps1 -Source projection-controls.rs
powershell -NoProfile -File ./run-controls.ps1 -Source t07-controls.rs -AllLibraries
powershell -NoProfile -File ./run-controls.ps1 -Source lifetime-controls.rs
powershell -NoProfile -File ./run-targets.ps1
```

projection当前0/1（exit101）；T07独立2pass/3fail（exit101，3fail归2finding）；原link8/execution4/lifetime2全绿。`target-results.json`保留7目标args/exe/hash/真实exit；所有永久目标退出0。`close-review.log`6、`close-adjacent.log`3、`core-lib.log`22通过。freeze、diff、五冻结Rust格式0的证据保留；最后freeze FAIL归并发WIP，不伪装clean。`reviewed.diff`为冻结a66→cfa完整diff，`frozen-*`为cfa五文件原件。

## 规范与路由

SD-03/04/05保持原语义：不可变单元派发的已测值/身份和原子性正控保持；C03来源不变量与F-11仍失败，不能授SD04/05完整已实现状态。SD01/02第一批已审子轴不重开，SD06与F09正式双线归后续。规范增量全文冻结/hash随收据保留，未写canonical或live ledger。

下一责任 **work**：同树一批处理原B2-C03残项及T07-R2-01/02；按[下一轮任务单](glm-next-round.md)收编控制。新冻结需包含/明确处置现有并发T-08修改，再复审受影响面；不重做已通过owner/调用闭包/候选所有权/预算机制。Plan r3/executing0/11，整份最终与最终PASS后merge-owned裸t＋VM/compiler tv等门仍pending；未整档/docs_gen/合入/归档/清树。
