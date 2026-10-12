# PLAN-743 r5 合入后独立复审（2026-10-06）

**needs_fix：3项P2既有约束未完全兑现。** 上轮四类反例均修好；真实归档门也正确，不重新挂旧缺陷。
plan_revision5，reviewed_commit `9111c21e53ccd9a81554bfd4764c540fdfdcad21`，diff_base `ad1fe269c21bc9021220645b52a32301e680f321`。本会话不参与r5实施；读取源码/实际Spec并自构公开校验/扫描反例，不以执行者pass为依据，没有委派代理。

## 基线与实际通过

- 用户发起检查时merge尚在进行；先在clean plan-743-dev@437a6b2f9243e35e5e23ab9ce6f005712aa89f7e运行**110/110单测，54.351s**。[日志](tests.txt)。合入后比对当前scanner和测试SHA与该检出完全一致，见[基线](source-manifest.json)，所以不重复整族；新增反例在合入后的当前同一代码上重放。
- 当前落地d64dc0f5c修复、f609e916b最终断言、454825ac5 SD09、8f2a021d6逐条重绑、27803ef74归档、638f25df9任务收口、9111c21e5收据均在祖先链。
- 原4类反例重放全部符合：mut/method/bare/method-bare不升级native；错误JSON形状ERROR非零无traceback；原invalid conclusion/hash变更/有效duplicate/单独错类型的consumer读取0；正例CLI0。[原反例重放](prior-counterexamples.json)/[输出](prior-replay.txt)。
- 严格47条决定全部fresh；归档真实**33/33、全部P743指针、模块导航、canonical、实际链接**通过，[真实终态门](final-assertions-main.txt)。5条AutoAC决定在merge按741P5实际diff逐条重审后重绑，没有当过期绑定为工具bug。
- TEMP双生成和提交制品三方一致（HEAD仅审计归一）、manual原字节不变，[确定性](generation.json)。当前真实人工47条不因合成反例被宣称失效。
- Category A纯Python调研/文档范围：本轮没有运行Cargo/native/docs_gen；未改旧parser/VM/A2X/auto/lib/741 profile/外仓。

## P743-R5-QA-01 [P2] 绑定文件不存在仍进入族消费者

`validate_decisions`绑定校验的`not f.is_file()`分支只置全局stale后continue，没有置record_bad，后续仍将这条记录放进validated_by_id。仅hash不匹配分支已修对。
受控三段式：真实绑定文件存在CLI0/consumer_reads21；删除该额外仓内绑定文件后CLI1/stale，但**consumer_reads仍21**；恢复原字节再次CLI0/21。新增文件和删除均在临时fixture；其余合法完整层保持不变。[反例源码](probe_edges.py)/[结果](edge-results.json)。
当前CLI正确拒绝，未复现CHECK-OK或崩溃；失败的是已沉淀SD09“新鲜度通过才入索引、invalid/stale不贡献族覆盖”的边界。AC16/18/21/25，T23/29。
修复：所有绑定缺文件、hash不匹配等失效情况都令本记录退出可信索引；或完整人工层失败后停止族消费。永久测试断言消费者0及有效/恢复正例，不能只断言CLI非零。

## P743-R5-QA-02 [P2] 错类型重复记录绕过全局唯一ID

类型错误先continue，seen_ids/invalid_ids去重发生在其后。所以同ID组中有一条evidence=17的记录时，无论前插还是后插，其合法同ID记录仍进可信索引，被resolved族读取21次。对照：两条都字段合法的重复组在前后两种顺序均已正确consumer0。CLI三种坏数据都受控非零、无traceback；没有把最终拒绝误报为假绿。[完整组合结果](edge-results.json)。
SD09和AC21/25要求全局唯一、重复整组作废，不豁免另有字段错误的同ID记录；T23/29。
修复：全局ID去重覆盖所有可辨识的非空字符串ID，不让字段类型淘汰跳过重复组识别；或校验失败后停止全部族消费。不能将非法ID列表/对象直接hash；错误类型仍受控定位。永久锁定两种顺序、缺字段/错类型+重复及真正合法正例。

## P743-R5-QA-03 [P2] 已知参数绑定仍让位于本地类型

`_post_classify`qualified分支先`recv in local_types`再检查known_bindings/import；bare分支先local enum/type构造再检查known_bindings。宿主IO/print优先已修，但刚沉淀的SD09明确“裸调用与qualified调用在宿主/类型升级前均检查可观察绑定”。
同模块声明type Meter与type Other，`fn demo(Meter Other)`内的`Meter.len()`仍为type-qualified；`Meter()`仍为type-construction，而此处Meter是参数绑定，归属不能证明应unknown。去掉该参数的同样调用是type-qualified/type-construction合法对照。[源码/分类结果](classification-results.json)/[运行器](probe_classification.py)。枚举和import探索数据附录不升级为必做新语法；本finding仅依已有普通参数/type/调用形式成立。
AC15/23/26、T24/30；当前真实清单旧宿主13及146族严格通过，不宣称全仓实际数据已出现相同错误。
修复：bare/qualified均先检查已有可观察绑定冲突，再升级本地类型/构造/宿主；不确定身份保留unknown。保持无冲突本地type、宿主、合法当前owner、前几轮参数/导入/多行保护；不实施完整resolver。

## 逐AC与Spec增量

| AC | 结果 | 任务 | 证据/说明 |
|---|---|---|---|
| AC-01 | partial | T24/30 | 真实9输入盘点通过；类型同名分类QA03 |
| AC-02 | partial | T23/29 | CLI错误受控，但可信消费者组合QA01/02 |
| AC-03 | pass | T03 | inventory/人工47决定/主体去向文档 |
| AC-04 | pass | T04 | S0..S6/Checked与target契约及741接口边界 |
| AC-05 | pass | T04 | pass模板/X1..X9/effect-trap顺序契约 |
| AC-06 | pass | T05 | 10锚点+双桥候选+Gen1/2/3，未宣称实现 |
| AC-07 | pass | T05 | 两张无编号后续候选/依赖与26–36估算，范围未扩大 |
| AC-08 | partial | T27/33 | 历史r5复审/merge真实，不能覆盖本轮新反例 |
| AC-09 | partial | T24/30 | 旧多行/宿主误判回归通过；类型绑定优先QA03 |
| AC-10 | partial | T23/29 | 非法输入CLI拒绝通过；类型错误duplicate组未整体剔除 |
| AC-11 | pass | T11 | R3效果/数值/runtime trap/顺序合同保持 |
| AC-12 | pass | T12 | 10锚点/候选owner及单向依赖，无抢号 |
| AC-13 | pass | T30/33 | 当前strict47fresh；P5外部Spec逐条重绑理由保留 |
| AC-14 | pass | T14 | r2历史/冻结/归档收据保留 |
| AC-15 | partial | T24/30 | owner与宿主正例通过；本地类型与绑定同名QA03 |
| AC-16 | partial | T23/29 | 同条绑定旧三段式通过；额外绑定文件缺失仍被族读取QA01 |
| AC-17 | pass | T31 | decision/family字段类型受控，无traceback |
| AC-18 | partial | T23/29 | 旧适用性拒绝正确；invalid/stale的消费边界QA01/02 |
| AC-19 | pass | T19/25 | canonical R3/trap/CLI/锚点一致 |
| AC-20 | pass | T21 | 历史进度/归档身份/索引/收据真实 |
| AC-21 | fail | T23/29 | 缺失绑定与错类型duplicate继续consumer_reads21 |
| AC-22 | pass | T24 | evidence-only适用性正反三段式与真实七族 |
| AC-23 | fail | T24/30 | 参数同名本地类型误升级type-qualified/type-construction |
| AC-24 | partial | T26/27 | 原测试/终态通过；本轮组合缺口未覆盖 |
| AC-25 | fail | T29 | 旧invalid/stale/有效duplicate消费0，新增两类仍21 |
| AC-26 | partial | T30 | 原四参数反例通过；SD09类型升级前绑定优先未全兑现 |
| AC-27 | pass | T31 | manifest/source_identity错误形状CLI1/no traceback |
| AC-28 | partial | T33 | 110tests/strict/确定性/归档33与SD09通过，行为缺口QA01..03 |

SD09正文沉淀与提案逐规则一致；实现上述3边界未完全兑现。r6仅具体化既有约束，冻结[SD10提案](proposed-spec-delta-phase6.md)，supersedes=docs/specs/auto-acc/project.md，new/touched=[]，auto-hir阶段/pass无变化。本次review没有改canonical或live ledger。

## 交接

继承用户此前“有问题激活同743并追加phase”的明确授权：r6/executing，新增T34..38，重开T23/24/26/27/29/30/33（旧执行文字保留为历史），完成26/总38。T25/31/32以及已过的原反例/归档机制保持完成。新phase按实际38任务验收，旧helper33常量只是r5历史。
next `/auto-plan:work`：T35处理QA01/02，T36处理QA03，T37复验/冻结，T38独立review与最终merge/归档/guard。不改生产Rust，不占新ID。本会话未新增/移除worktree；旧实施树由本轮merge作者清理，741/742/ABI保留。
复现：`PYTHONUTF8=1 python -B probe_edges.py <reviewed-repo-root> <new-TEMP-output-dir>`；`python -B probe_classification.py <reviewed-repo-root> <TEMP-result.json>`。只在隔离fixture写数据；观察器原参数/返回值不变，以tagged evidence的族消费调用计数取证，另跑无注入CLI。全部历史结果与当前源码hash冻结，旧pass不覆盖r6。

活动合同的真实父目录链接、38唯一任务/完成26、两模块743导航已核对；canonical/产品脚本/冻结输入hash保持复审基线。本次只提交743文档白名单，保护741等并行WIP。原始测试/控制台日志保留字节，文本格式门不改写证据输出空格。
