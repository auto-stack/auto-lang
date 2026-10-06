# PLAN-743 r6 合入后独立复审（2026-10-06）

**outcome: pass。上轮三个 P2 缺陷已修复；本轮未发现新的阻塞问题。**

## 基线、范围与独立性

plan_revision6，reviewed_commit `8b17e4de8851cdf37ccdce76a82f78422cf8bb15`，diff_base `bf899da0b2805b61e5e815a265438ea45c2e9e6d`。
未参与r6实施；本会话负责r5复审，因此自行重放反例/测试/真实制品，不以执行侧pass作依据。没有委派代理。
实施tree已收尾，按真实祖先链确认25c3eb5b3→d822de2cc→f8fc6b98a→3fe9b99c1→5e9e3d94b→8a912cf89及归档25373bc2a/38任务收口978a93d8c均落地。
创建本次专用detached审查tree固定HEAD，只读主检出并在临时fixture写反例，不建立实施分支。当前受管9输入、两个脚本、测试、canonical/manifest/manual/ledger与主检出逐字节一致。
Category A：只运行Python工具/文档检查，没有Cargo/native/docs_gen。主线自基点的fs.rs/stdlib.rs改动属于并行742/744（7b732dae4、9cfe7f65c）；743交付不触Rust、旧VM/A2X、741 profile或外仓，阶段设计不变。

## 本轮实测

- `python -B -m unittest discover -s scripts/tests -p test_acc_inventory.py`：**128/128 OK，46.266s**，tests.txt。
- 原r5 probe_edges原样重放：合法/额外绑定存在/恢复consumer_reads=21且CLI0；缺失文件、错类型duplicate先/后及合法duplicate先/后均consumer_reads=0、CLI1、无traceback（edge-results.json）。
- 补充6项记录控制：缺evidence/缺bound_input_hashes同ID前后两顺序均CLI1、consumer0；ID为数组/对象受控拒绝、无traceback（additional-record-controls.json）。
- 原r5类型参数反例：Meter.len→unknown-receiver、Meter()→unknown-bare-call；无绑定对照type-qualified/type-construction保留（classification-results.json）。
- 新增7组组合：普通/mut×自由函数/已观察方法四组，let/var及支持的`use other: Meter`三组；同名绑定不升级本地类型/宿主（additional-binding-controls.json）。历史探索样例`use "some_module" {Meter}`不是本工具声明支持的use形式；其输出仅保留探索历史，不把未承诺语法作为缺陷或通过证据。
- 原更早反例：mut/method/bare/method-bare不升级宿主；合法无冲突宿主保持native；manifest/source_identity四种错形受控拒绝，无traceback；invalid conclusion/stale binding/duplicate/invalid type的消费者均0（prior-counterexamples.json）。
- 真实strict：47条决定fresh、9输入、146unknown候选由7族声明闭环，CHECK-OK。双独立--write/--check及提交制品三方manifest/summary一致（只归一audit-only HEAD）；两个manual及原manual字节不变（generation.json）。
- 最终实际archive门：**38/38、全部P743-1..8指针、模块743归档行、canonical、Plan实际父目录链接、严格门均通过**（final-assertions.txt）。同名模块741行仍为r7 executing/active，未因743归档被覆盖。

## 全部验收映射

| AC | 结果 | 任务 | 当前证据或明确复用理由 |
|---|---|---|---|
| AC-01 | pass | T02/T24/T30 | 9输入实际盘点、类型/绑定组合与真实扫描制品重生成 |
| AC-02 | pass | T02/T23/T29 | CLI受控拒绝和内部consumer读取0两层验证 |
| AC-03 | pass | T03 | 47人工决定fresh，完整主体清单与能力四态保持 |
| AC-04 | pass | T04 | stage-contract及bootstrap主体设计无diff，已有S0..S6/Checked/target审查证据复用 |
| AC-05 | pass | T04 | 既有pass模板/X1..X9/effect/trap/顺序合同无diff，复用r5逐项审查 |
| AC-06 | pass | T05 | 10锚点/双桥/Gen1..3文档无diff，复用r5审查 |
| AC-07 | pass | T05 | 后续无编号候选/依赖/估算保持，无抢号或实现范围扩大 |
| AC-08 | pass | T38 | 本轮独立检查38/38归档/链接/全部P743指针及提交祖先 |
| AC-09 | pass | T24/T30 | 128测试内旧多行/宿主分类回归+独立类型冲突控制 |
| AC-10 | pass | T23/T29 | 旧非法JSON/非法记录及两个顺序错类型duplicate受控，无traceback |
| AC-11 | pass | T11 | R3 effect/数值/运行时trap/顺序合同无diff，复用r5 |
| AC-12 | pass | T12 | 锚点owner与单向工作包依赖无diff，复用r5 |
| AC-13 | pass | T30/T38 | 真实strict47fresh；manual字节未变，未机械重绑 |
| AC-14 | pass | T14 | 所有历史合同/收据保留，当前revision6 source/hash冻结 |
| AC-15 | pass | T24/T30/T36 | owner/正常宿主正例及参数类型同名bare/qualified控制 |
| AC-16 | pass | T23/T29/T35 | 额外绑定文件存在/删除/恢复：CLI0/1/0，consumer21/0/21 |
| AC-17 | pass | T31/T35 | 128整族类型/字段反例与JSON形状拒绝，旧负例独立重放 |
| AC-18 | pass | T23/T29/T35 | 非法/陈旧/重复记录无族消费者读取；resolved适用性旧门回归 |
| AC-19 | pass | T19/T25 | 既有R3/CLI/锚点规范保持，本轮SD10只具体化三边界 |
| AC-20 | pass | T21/T38 | 归档38/38、6条review+P743-1/2真实指针/模块归档态/收据验证 |
| AC-21 | pass | T23/T29/T35 | 缺失绑定及错类型重复整组在新反例中consumer0 |
| AC-22 | pass | T24 | 证据适用性旧测试回归通过、146unknown候选/7族声明真实闭环 |
| AC-23 | pass | T24/T30/T36 | 普通/mut自由函数/方法+let/var/支持的use绑定在本地类型升级前阻断 |
| AC-24 | pass | T26/T27 | 当前128测试、严格门、确定性、规范及实际归档门独立运行 |
| AC-25 | pass | T29/T35 | 无效结论/hash漂移/合法及错误类型duplicate两顺序均consumer0 |
| AC-26 | pass | T30/T36 | 原四参数控制及新增7组类型绑定冲突；无冲突本地类型/宿主保留 |
| AC-27 | pass | T31 | manifest []/1、source_identity []/null均CLI1且无traceback |
| AC-28 | pass | T33/T38 | r5历史不删，本轮128/strict/确定性/38归档新证据覆盖当前实现 |
| AC-29 | pass | T35 | 原QA01/02两组复现正负控制全部符合，invalid ID/缺字段受控回归 |
| AC-30 | pass | T36 | 原QA03 bare/qualified未知，7新组合通过；未实现完整resolver |
| AC-31 | pass | T37/T38 | 全部新证据+源码/SD10冻结+范围审查+真实archive/all指针/guard收口 |

## 规范增量与遗漏检查

SD-10 modify `docs/specs/auto-acc/project.md`的三条规则均实际沉淀且与代码吻合：缺失绑定文件和hash漂移都淘汰记录；所有可辨识字符串ID去重发生在字段类型门之前，重复整组无效；bare/qualified都在本地类型/enum/宿主升级前检查可观察绑定。
冻结原提案在reviewed-spec-delta-phase6.md，完整SHA与Spec/源版本见source-manifest.json；supersedes仅auto-acc/project.md、new/touched=[]。没有新增阶段、编译能力、resolver或SD11。review不修改canonical/live ledger。
风险表保留了r6激活时的待修描述，本review按上述独立验证事实追加三个P743-R5-QA债项的清偿记录；before原文保留，不改变741及其它债项。
所有原任务和AC31实际收口；未发现未批准的延期/替代实现。128测试健康无新增警告/调试输出，原分类和人工层仍是有限词法观察，未宣称AC/ACC已经实现清单能力。
模块plans.md的archive/字段采用本仓现有目录标记惯例；实际Plan定位由P743-*指针和归档Plan完成，本轮没有将目录标记误报为断链。

## 结论与下一步

保持PLAN-743 archived、revision6、38/38；追加本次pass记录，不激活、不追加修复Phase。证据文档提交即可，本轮不重新合入实现/沉淀Spec，不改其它计划。
审查tree按精确路径/clean Git/无ReparsePoint扫描清理（规定wt-guard.sh本机缺失，沿用r1..r6已记录等价guard），详见cleanup.txt。741/742/ABI工作树不动。
