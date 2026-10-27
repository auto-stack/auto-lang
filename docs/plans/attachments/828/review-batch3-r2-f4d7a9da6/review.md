# PLAN-828 第三批 R2：原例通过，C03/C04 不变量仍未闭合

2026-10-22；review_scope=phase（第三批T-07/T-08/T-09受影响面，非T-10整份最终）；**needs_fix**。沿用原 finding ID：**B3-C01、C02、C05 原列具体错误 CLOSED；B3-C03/P1、B3-C04/P2 OPEN**。没有新增一套验收目标，也不要求重做已通过的 root/事务/单元派发机制。

## 冻结与方法

- 同树 `D:/autostack/.wt/lang-828/auto-lang`；`plan-828-dev@f4d7a9da6fb6b58a15a3149ee86583a7555daa86`；基面 `8a18db4503a6b30e53a0affa9cb17c4126d35e6d`。实施树入口及结束clean，无未提交实现。主检出交接HEAD=db363c72761a72d0a711819cd70482e4532d1fd0。
- 主Plan r3、interface决策和三项交付文档先冻结到本目录；baseline/final-state绑定520项源码/锁配置/Specs/Design43/冻结合同/技能，matches_baseline=true。auto-down=c39ac7f9dacdc769fc4a5ad3b98ec896240b28e8，probe未消费；文档与证据hash另见evidence-index。
- reviewer为本Codex审核上下文，未实施修复、未启代理，不编造未暴露身份。未改WT源码、canonical/ledger，未整档/合入/清树。
- 原六控制机械适配只有JudgeEdit.expected_install；生成器仍从本冻结worker抽取生产session_serve。原故障、输入和oracle保持。进一步核对合法命中、完整观测、实际执行与失败证据出口，不能将同一旧例绿等同完整不变量绿。

## 关闭表

| 原finding | 结论及证据边界 |
|---|---|
| B3-C01/P2 root归属 | CLOSED。归属tracker移到get_enveloped_opt成功之后、decode/header/rebind/canonical/span/verify之前；不再依赖采纳。原2控2/2，位置变化skip_span=3/adopted=0、root3→3→3、retired=1。另6次合法位置变化→restore→内容7/8回访→真实GC，逐轮root=3、结果正确；旗舰512真实存储轴1/1（246.78s）。未放宽采纳门，用户哨兵/required闭包旗舰保持 |
| B3-C02/P1 修订/目标 | CLOSED，在正式脚本每个请求均为成功编辑的边界内。父侧独立expected_install核准确目标及源名，阶段排除按该预期；revision核req_id。原stale/install各自红，clean绿；官方小夹具含4修订、异函数和历史回访绿。没有将“拒绝后继续成功的通用事务脚本”擅自算成本性能脚本新增支持面 |
| B3-C03/P1 实际载荷身份 | 原miss副本例CLOSED；完整不变量OPEN，并有合法命中新回归。见下节，不能只修一个reuse键就关闭 |
| B3-C04/P2 计时/观测/raw | 计时端点、init单列、已收到样本的字段扩充及成功raw落盘CLOSED；失败落盘与阶段wall/IO仍OPEN，见下节 |
| B3-C05/P2 原文档失实 | resident_weak真实参数、document→hit/miss→miss verify生产顺序、b1=11原三项CLOSED。C03/C04的“不可脱钩/成功失败同留/完整观测”完成措辞须随机制修正；不另加文档finding。coverage中的session::dispatched_identities须改成实际符号last_run_dispatched_identities（该笔误并不单独抬为阻塞P2） |

## B3-C03/P1 残项：观测仍可与实际消费脱钩，且误拒合法命中

归属T-08、AC-02/07、V-08/09、C02-05，违反原§5.5/5.8：实际未改payload身份/代际/内容保持、实际复制并执行时同门必红、合法内容命中允许复用。不增加语言或资源支持面。

### 1. 在run内部真正复制并执行，正式门仍绿

session.rs:1254–1265先枚举`self.current.code()`的Arc地址/代际，随后调用execute_program。这是包装层当前根盘点，不是从本次AV实际获取的payload采集。它也没有内容轴。父侧仅比较这份盘点，不能证明生成本次发布值的执行使用它。

`generate-consumer-copy.py`对**完整冻结session源**做审核临时投影：doc注释转普通注释以include；只把run末尾的一处`execute_program(&self.current,...)`替换成受控分支：深复制真实FunctionCode/MirFunction、同revision/maps重链接，断言未改main same_object=false/same_content=true，**实际execute_program(copied)**。新观测、字段、计数、全部其他函数体原样。冻结生产worker循环只将session导入换为投影；父侧仍是原生产`run_formal_session_judge`。没有手填身份、篡改回包或调用另一个gate。

结果 `consumer-copy-controls.log`：原run正控绿；run内部实际副本执行却 **gate=true / identity=true / correct=true / failures=[]**（应红），1过1败。源码/hash/限定变换见consumer-source.json、session-consumer-projection.rs、worker-consumer-copy.rs、worker-consumer.json。投影编译的9项unused/dead-code警告来自审核仅用session部分API，非生产新增警告。

另一个公开路径交叉控制 `copyafterrun`：只先真实s.run一次再深复制并AV执行副本，用副本结果发布；未伪造字段，仍gate=true。两证据都表明“旧副本例因上一修订记录而红”是间接新旧对象启发式，尚未把证据绑定真正消费。仅修成“每次都强制原run一次”会留下此漏洞，也可能多执行guest副作用。

### 2. 未改main缺观测仍绿

审核omitmain模式保留真实编辑/执行，仅从ready及每次dispatched清单中移除d_main：ready.functions仍3，实际清单只2项。父侧检查相邻清单彼此相同，没有对完整独立函数目录/ready计数核准。结果 **gate=true / identity=true / failures=[]**。缺少主入口身份不能证明它未重建。原规则“缺必需观测不得出PASS”已有，不是新增协议威胁模型。

### 3. 原正式worker误拒合法同内容命中

不使用mutation worker：直接调用Cargo返回的**冻结生产av_probe_worker**，对三函数helper的`7`改成`7 `（合法纯body空白变化，结果仍9）。生产者确实记录 `reuse_hit:helper=1`、零lower/emit/decode/verify/load；revision/目标/结果全正确，仍被身份门判红：目标未换对象且“reuse_hit=0”。

确定根因在formal_worker.rs:378：读`counters_delta.get("reuse_hit")`，实际session.rs:2140按`reuse_hit:{changed_name}`生产。合法T-04内容命中当前对象，本就不应要求对象改变。原小夹具“8→9→8”历史回访可能换回旧对象，碰巧满足`target != previous`，不能覆盖复用当前同一对象。

`judge-identity-controls.log`三控0/3（分别是合法应绿被拒、缺观察应红误绿、copyafterrun应红误绿）；`consumer-copy-controls.log`1/2。`judge-hit-controls.log`真实8→9→8回访两控2/2，证明那个原旁路控制尚能红，并非所有门失灵。初版审核探针误读了全局reuse键，前提断言先失败；原输入和日志保存在judge-hit-initial.*，改用真实producer键并以官方worker重建反例，不把探针错误算额外产品finding。

**一次关闭条件**：修正合法hit判据；完整函数目录/required观测缺失须拒PASS；从真正执行consumer产生并与本次请求/快照/发布结果绑定身份、代际、内容证据，不能只读当前根或上次run列表。上述run内实际副本必须使同一正式总门红，原run/当前内容hit/历史回访/多修订各自绿。代表性共用普通/step/resume观测若被改动，按受影响面核验；不重做已闭核心执行器，不手填布尔值，不拿记录一次原执行遮盖实际副本结果。

## B3-C04/P2 残项：失败时仍不留证，阶段wall/IO缺口仍在

归属T-08/T-09、AC-07、V-08/09、原§5.5的“保留全部raw/失败、阶段wall/计数/IO、输入与二进制身份”。端点移到父侧检查后是有效修复，成功落盘也是实际交付，以下是原finding剩余范围。

### 1. 正式entry先assert成功才write_evidence

function_incremental_828_t08.rs中，在write_evidence前已assert failures空、all_correct、structural_ok、P95、gate_pass。任一正式门失败都会panic，绕过新保存逻辑。`write_evidence`函数本身可写红report，但entry未接上失败出口。

`generate-evidence-control.py`抽取冻结正式战役测试的**原函数体**，只重命名函数并由局部worker_exe选择同一真实循环的wrongvalue变体，所有30次真实编辑/值比较/判决/落盘顺序保持；外层catch_unwind在新审核专有输出目录检查证据。结果：30轮全记录“published -999 != independent expected”，第一个成功断言panic；**report_exists=false / raw_exists=false**，应留存断言失败（evidence-failure-controls.log，0/1）。正控官方永久t08已实际写出30行，故不是目录权限问题。

异常reply类型还直接return Err，调用方expect后也没有带已收集样本的报告；缺样本虽有failures消息，正式entry同样先assert而不会保存。关闭须覆盖代表性的值错/结构错/中断或异常回包，不能只调用写函数单测。

### 2. 两个总时长不是各阶段wall/IO

新增worker_edit_us与worker_edit_to_result_us，逐函数生产计数/diagnostics/对象地址保留，input/worker指纹及init_ms也已交付。但session/worker仍未采集并输出合同所列生产阶段wall与IO观测。成功真实raw和完整JudgeSample/Report schema可直接核对：只有edit总时长、edit→result总时长和次数，无法得到resolve/typecheck/lower/emit/admit等阶段时长或产物IO。现有observations::Obs有span机制，但正式session路径未接用；不要求一定采用该API，只要求真实观测满足原口径。

**一次关闭条件**：正式entry在任何判决assert/退出前保存本次完整report、已有raw及失败上下文，确保失败目录是本次产物而非上轮文件；异常中断也留下准确收据。补齐相应阶段wall/计数/IO与口径，零IO明确记录来源，初始化/输出报告另报，必要父侧校验仍在窗内。复跑原失败控制及成功30轮，保存证据再判总门；专用漂亮报告/全场景性能扩样仍非新增必需项。仅挪write_evidence位置仍未关闭阶段观测部分。

## 本轮实际通过的验证

| 验证 | 结果 |
|---|---|
| 原judge-controls / 原hir-roots | 6/6、2/2（同输入、同oracle，expected_install机械适配） |
| 六次恢复位置变化/内容回访/GC | 1/1，逐轮root=3、结果正确 |
| 完整存储512/GC/两restart/pin/用户哨兵 | 1/1，246.78s；旧另两个512未重跑 |
| 官方t08四项（含正式30轮及原零门/错期望/缺样本） | 4/4，成功report+30行raw在formal-official；不把此绿当负控完整 |
| permanent mig/t07/b2/b1 | 9/9、8/8、14/14、11/11 |
| session/contract/t01r/r1 | 18/18（skip三512、新的另跑）、22/22、4/4、86/86 |
| 独立migration/缺块重建/GC目录 | 5/5、2/2、5/5；前批已闭保持 |
| close继承控制 | 96/96（include链有重叠，不相加宣传唯一覆盖数） |
| 五个改动Rust文件rustfmt / diff-check | 0/0；生产定向构建零新告警 |

正式本次P50/P95/P99=**188.1045/195.8526/200.6638ms**，init=1277.116ms；worker峰值210231296B/父侧22327296B；官方硬预算断言A≤224MiB、B≤128MiB通过，日志A=201/B=0（显示整数MiB）。这些是本次采样/批准798归因口径，不扩为普遍资源承诺。正式门仍受C03/C04限制。官方raw/报告保留了当前已实现字段，并不自动补齐缺字段或失败出口。

未重跑全部798/core/ac-core套件，不把work总数冒充本轮复现；其未变核心证据按输入指纹边界延续。新run字段只存数值、不增加Arc所有者，close原控制也已重放。T-10整份验收/Spec整体通过及merge-owned裸t+tv/必要触面consumer、tf到期主检单实例/沉淀/归档/guard均pending。

## 复现与下一步

在冻结WT按Cargo artifact所列9个integration target定向构建（见build-stderr/cargo-artifacts），随后本证据目录：

```powershell
python ./generate-worker-variants.py
powershell -NoProfile -File ./build-worker.ps1 -AllLibraries
powershell -NoProfile -File ./run-controls.ps1 -Source judge-controls.rs -AllLibraries
python ./generate-consumer-copy.py
powershell -NoProfile -File ./build-consumer-copy.ps1 -AllLibraries
powershell -NoProfile -File ./run-controls.ps1 -Source consumer-copy-controls.rs -AllLibraries
python ./generate-adjacent-workers.py
powershell -NoProfile -File ./build-adjacent.ps1 -AllLibraries
# PLAN828_REAL_WORKER取cargo-artifacts中av_probe_worker.executable
powershell -NoProfile -File ./run-controls.ps1 -Source judge-identity-controls.rs -AllLibraries
python ./generate-evidence-control.py
# PLAN828_FORMAL_EVIDENCE设置本次新的审核目录，不能拿既存成功报告掩盖失败缺文件
powershell -NoProfile -File ./run-controls.ps1 -Source evidence-failure-controls.rs -AllLibraries
```

原judge6与root2退出0；identity/consumer-copy/失败落盘控制退出101，不能拿外层脚本后续命令的exit0当全部通过。Cargo产物路径/hash、原始脚本/变换hash/源码/输入/输出均保留，不依赖实施者TEMP验证台。

下一责任同树work：**只修C03/C04剩余机制及其完成措辞**，一批冻结再复审，PASS后T-10。先提交紧凑“实际consumer→观测→publication”和“失败→证据→退出”关闭表，覆盖合法当前hit与缺观察这两个兄弟出口，避免按旧例追加特殊分支。Plan保持r3/executing0/11，不重新打开C01/C02已闭机制。SD01原路径更正通过，SD05/06受C03/C04完成声明限制，全部仍proposed、不沉淀canonical。
