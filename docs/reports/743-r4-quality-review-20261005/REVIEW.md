# PLAN-743 r4 合入后独立复审（2026-10-05）

Outcome: **needs_fix**。

落地状态：2026-10-06补齐上次因审批超时未执行的文档落地；在主检出v0.6-dev（写入前HEAD 0501b0f274088c05bb1b7c5a5fd9a98f4791c835）激活同一743为r5/executing，追加Phase5，当前22/33。复审基线及证据仍绑定2026-10-05的r4提交，不把文档落地日期当成新的实现复审。

绑定r4合同 `1d8fba15c7f8685e670f3bc8a398198895331c72` 和合入后HEAD `c865adf66a75099df07113b053eb9d19e92986a3`；对照SD-08、当前canonical Specs和实际源码，不以旧pass/勾选为证据。复审者本轮未实现743工具修复。按用户此前明确授权重启同ID r5 Phase5；不占新号、不改产品/canonical/ledger。

## 实际通过与范围

在独立detached工作树 `D:/autostack/.wt/lang-743/auto-lang`（上述HEAD）重放，Python3.14.2、PYTHONUTF8=1，Category A：不运行cargo/docs_gen。

- `python -B -m unittest discover -s scripts/tests -p test_acc_inventory.py -v`：83/83，46.587s，见[完整日志](tests.txt)。
- `python -B scripts/acc_inventory.py --root . --output docs/reports/743-acc-hir-contract --check --require-decisions`：exit0，47条决定全部新鲜、9输入、146 unknown组/7族，见[严格门](strict-check.txt)。这些合成反例不证明当前47条决定已经失效。
- `python -B docs/reports/743-phase4-consumer-fixes/final_assertions.py .`：真实archived27/27、全部P743-*历史指针、模块索引及canonical通过，见[原断言](final-assertions.txt)。旧r3的缺/错类型traceback、subject/hash替代证据、普通自由函数参数/导入遮蔽已修复，见[旧反例重放](previous-r3-counterexamples.json)。
- 临时目录双生成字节一致；audit-only HEAD归一后与提交manifest一致；summary一致，manual原字节未改，见[确定性与delta](determinism-and-delta.json)。SD-08两个冻结块与canonical一致。旧audit脚本关于phase3 delta的历史布尔不能作为r4 delta结论。
- `git diff --check`、合入范围/历史检查；r4包含完整27个任务，旧计数与ledger问题已修复。18哈希重绑定对应Spec变化，并非整体重刷结论；后续合成负例独立隔离。

## P743-R4-QA-01 [P2] 参数遮蔽不完整，错误升级宿主

`scripts/acc_inventory.py:450–479`签名处理仅遍历自由函数declarations，忽略methods；mut参数名取toks[0]再删mut得到空名。裸调用分类（约370行）没有在BARE_NATIVES前检查known_bindings。四个既有语法内反例均错成native-runtime：`demo(mut IO Meter)`内`IO.read_line()`、方法`demo(IO Other)`内同调用、自由函数/方法的`print`参数调用`print()`。见[四反例与正例](counterexamples.json)、[可运行重放器](reproduce.py)。普通自由函数IO参数仍unknown、无冲突宿主仍native是对照。mut为真实Auto语法（auto/lib/engine.at:85等），不是新增语法要求。其他探索性for/单行绑定不加入本Phase必做范围。

修复：准确提取普通/mut参数及方法签名，裸/qualified判据共享绑定优先规则，冲突unknown；不要求完整resolver。回归导入、局部类型、普通参数及无冲突native正例；逐项审定真实unknown漂移，不固定计数、不机械更新hash。

## P743-R4-QA-02 [P2] 合法JSON错误形状导致检查崩溃

`scripts/acc_inventory.py:988–1008`解析后直接`.get`及给source_identity索引赋值，没有对象形状门。在完整有效人工层（对照exit0）上仅替换manifest为`[]`、`1`、source_identity为`[]`/`null`，四例exit1且AttributeError/TypeError traceback；见[复现结果](counterexamples.json)。不是故意损坏人工层导致混合错误。

修复：在访问/归一前验证容器形状，受控ERROR/非零/no-traceback；保持audit-only HEAD忽略及完整重新生成比较，无输入文件写入、副作用或空对象兜底假通过。

## P743-R4-QA-03 [P2] 消费者索引尚未达到完整语义/唯一性验证

`validate_decisions`约706–756、828–842行：类型非法会continue，但非法conclusion/过期绑定只设置全局错误，仍进入validated_by_id；重复ID的后者不入索引，先插入者未淘汰。family消费者随后读取该记录。这违反AC-21及已沉淀SD-08“完整类型/语义校验”的边界。

重要限制：**三例最终CLI都正确拒绝（exit1），没有复现错误CHECK-OK或新增崩溃**。调用观察器仅转发原`_evidence_file`参数/返回值，以唯一`:910`后缀定位；原CLI另行无注入执行。valid对照consumer_reads21、CLI0；invalid_conclusion/stale_binding/duplicate各consumer_reads21、CLI1；invalid_type为0、CLI1，见[counterexamples](counterexamples.json)与[观察器源代码](reproduce.py)。此项是已承诺的可信消费边界未兑现，非伪造绿色验收。

修复：逐记录完成所有验证/全局唯一性后建索引，重复整组淘汰；或失败后完全停止族消费。负例断言消费者不读取/贡献覆盖；两个重复顺序和合法层正例俱全。

## P743-R4-QA-04 [P3] 六处归档报告链接失效，最终断言漏检

实际归档Plan行156/172/183/723/840/955用`../reports/`，应从archive父目录用`../../reports/`。23个链接查得6断链，见[audit](audit.json)。原final_assertions仍通过；27/27和P743-*指针确实正确，不重新报旧计数/指针错误。激活移回plans目录会暂时使六链接可用，但不代表归档问题修好。

修复：新Phase5最终断言从实际父目录检查Markdown链接/围栏排除；模拟及真实archive均验，保留r4历史helper。33/33、metadata、模块导航、全部P743-*历史指针一并核对。

## 逐项验收与Spec delta

| AC | 本轮结论/证据 |
|---|---|
| 01 | 部分：真实清单严格通过；参数反例误判QA-01 |
| 02 | 部分：83测试通过；manifest形状QA-02、受控检查未完整 |
| 03–07 | 通过：人工层47条、设计/阶段/负例/后续合同文件与strict重放；不等同实现native/ACC |
| 08 | 部分：历史工作流真实闭环，新的needs_fix不由旧pass覆盖 |
| 09–13 | 已测试范围通过：旧负例和fresh47成立；消费者边界另外由AC21衡量 |
| 14–15 | 部分：历史交付保留；新遮蔽QA-01和归档QA-04 |
| 16–17 | 已覆盖字段通过：错误类型受控、证据同条绑定；不替代全记录语义/消费者验收 |
| 18 | 部分：CLI正确拒绝无效引用；QA-03消费者仍读非法记录 |
| 19 | 通过：当前CLI/主打目标命名等r3修正保留 |
| 20 | 部分：旧计数历史已修复；最终链接仍缺口 |
| 21 | 未通过：QA-03非法语义/stale/重复仍被消费者读取 |
| 22 | 通过：旧subject/hash证据替代反例拒绝，真实fresh层通过 |
| 23 | 未通过：QA-01 mut/方法/裸调用参数遮蔽不完整 |
| 24 | 部分：83全绿、27/27/ledger/SD-08一致；六链接QA-04 |

无删除子项、未经授权延期、捕获异常假绿等新增证据。SD-08正文沉淀一致但实现未全兑现；SD-09为现有边界的具体化，见[冻结提案](proposed-spec-delta-phase5.md)、[哈希](delta-hashes.json)。本轮不修改canonical或derived ledger；用户授权激活造成P743-3..6旧archive指针暂缺，r5 merge须恢复全部，模块导航先指active。

## 交接

同一[Plan743](../../plans/743-acc-bootstrap-hir-contract.md) r5/executing，当前完成22/33；T23..27重新待验，旧实施记录保留为历史；新增T28..33。工具修复、新增产品测试及Spec沉淀尚未执行。本报告及证据独立于实施者旧pass，不接受“合同就绪”充当新阶段完成。

重放：`python -B docs/reports/743-r4-quality-review-20261005/reproduce.py <repo> <不存在的临时输出目录>`。测试只在临时fixture写入，调用现有工具/测试factory；不修改产品文件。证据原始字节hash见[source manifest](source-manifest.json)。

## 并行主线更新核对（2026-10-05草稿准备时的历史记录）

准备草稿期间主线合入741 r4，最新HEAD `7b9c6948c1465e87a6f60404510c30c712e1c7bc`。743工具/测试/计划/SD08/auto-acc Spec均未改变，本轮反例证据仍适用；共享债务、auto-hir导航草稿从最新主线重建，保留741销账。当前strict正确拒绝5条AutoAC Spec绑定过期决定（MD-203/205/301/302/304），见[current-main-strict](current-main-strict.txt)。这属于外部Spec变更后的正常freshness护栏，不新增为743缺陷；T28/T30需逐条重新审定绑定及结论，不能直接替换hash。前文47fresh仅对应独立复审基线，不代表最新主线仍全绿。

## 2026-10-06补齐落地

用户指出743复审失败后未激活/未写计划，明确要求补齐。核对原复审工具/测试/auto-acc与HIR stage Spec/受管制品/原r4计划哈希均未改变；唯一历史差异为并行741的ledger投影，完整保留当前741 r5计划/债项和导航。未重新运行83测试或声明新的行为pass；本轮是既有needs_fix证据和修复合同的文档落地，不实施Python修复，不发布canonical或live ledger，不占新ID。

活动[Plan743](../../plans/743-acc-bootstrap-hir-contract.md)记录全部QA01..04及T28..33/AC25..28/SD09；T23..27当前验收重开，旧r1..r4实施与复审收据保留。草稿校验副本只属历史，实际落地校验见[activation validation](activation-validation-20261006.json)。5条AutoAC Spec stale决定继续由T28/T30重新审定，不能机械刷新hash。

SD09草稿从CRLF统一为LF后，内容保持不变；本轮校验确认旧哈希对应CRLF字节，delta-hashes.json已同步落库LF字节哈希。SD08受审副本与原冻结哈希仍完全一致。

## 上次复审临时检出收尾（2026-10-06）

证据已入仓；上次遗留的detached D:/autostack/.wt/lang-743/auto-lang和空组已清理，无实施分支创建/删除，其他工作树未动。wt-guard.sh仍缺失，bash/WSL也因VHDX路径不可用报错；删除前已做精确路径/原HEAD/git clean及全树ReparsePoint等价检查，沿用既有收据的安全扫描方案。见[cleanup receipt](cleanup-receipt-20261006.json)。
