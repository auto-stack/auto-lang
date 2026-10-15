# PLAN-780 独立 final review R4

2026-10-15；**needs_fix**。R3全部原反例通过，异常清单身份已闭合；分隔检查仍把块注释内换行当语法分隔，既误拒合法输入，也使旧VM拒绝的输入获得static等价标志。尚不能合入。

## 基线与身份

review_scope=final；plan_revision=1。审查者为规划及R1–R3评审会话，未实施780；根据实际diff、冻结合同与独立旧新控制重建证据。

- worktree=`D:/autostack/.wt/lang-780/auto-lang`；branch=`plan-780-dev`。
- reviewed_commit=`3ab22eeb7b50dcbdc012cc2ce0a7aede9cc5a4dc`；base=`075a9b4ab6cb4284f3c259b348f2571322107fec`；R3=`61eea96971c1290c745da4ff74e80795834aa9cb`。
- auto-down只读兄弟=`c39ac7f9dacdc769fc4a5ad3b98ec896240b28e8`；依赖锁/受管源码/最终状态见[fingerprints.json](fingerprints.json)。
- Spec源auto-man/project SHA256=`bb857003628365be87cec91912ab36c672388a943163d5fc3584fc44600e80c3`；project-model新Spec仍不存在；[规范增量冻结](spec-delta-reviewed.md)。

本轮work已提供R3根因及设计调整：统一trivia、保留分隔状态、按容器区分规则、精确完整路径清单及永久正反控制。复审已核验这些变更，并独立检查新增分支；无需改变revision1合同。

## F780-R4-01 / P2 / blocking：块注释的物理换行被当成语法分隔

位置：`crates/auto-project-model/src/scanner.rs:1133–1141`，check_separation的块注释分支遇`\n`设置passed_break=true。旧VM将整个块注释视为trivia，内部换行不形成元素/键值对分隔。源码行号应累计，但分隔状态不能因此改变。

独立探针对每例先调用生产旧臂AutoConfig→Pac，再调用新模型归一化；每个子进程2秒截止，无超时。三项违反既有合同：

| 新矩阵id | 输入 | 旧VM oracle | 新模型 |
|---|---|---|---|
| 0 | `back: { project: "b" /*multi\nline*/, extra: 1 }` | Ok，external_backend=b | unsupported ValueSyntax逗号，external_backend=None |
| 1 | `back: { project: "b" /*multi\nline*/ extra: 1 }` | Err，Expected infix operator | **static=true**，external_backend=b，unsupported为空 |
| 2 | `members: ["a" /*multi\nline*/ "b"]` | Err，Expected infix operator | **static=true**，members=[a,b]，unsupported为空 |

证据：[separator-matrix.rs](separator-matrix.rs)、[运行器](separator-matrix.py)、[逐例旧新结果](separator-matrix.json)。JSON的exit101来自独立断言失败，不是超时。输入中的`\n`表示实际换行。

性质：R3分隔兼容修复的遗漏/回归；原反例闭合，完整trivia不变量未闭合。违反AC02/03、V02/V05、T02/T03/T05及T01冻结的字面量容器/注释支持；尤其旧臂报错时不得声称static等价。不要求轻模型实现表达式或扩大规模。

修复应区分源码行数与语法换行事件：块注释内部换行仅更新位置；注释外的真实换行、行注释终止、逗号分别按实际VM规则处理。永久矩阵需覆盖array/object/top-object、字符串/数字/builtin、单行/跨行块注释，并成对验证“真实换行”与“仅块注释内换行”。保留已有终止性、同行相邻拒绝及旧反例。

## R1–R3闭合与独立扩展

| 审面 | 结果 |
|---|---|
| R1独立输入 | 12/12通过，六项具体反例继续闭合 |
| R2终止控制 | bounds6/6，无超时，终止项保持关闭 |
| R3注释/builtin换行 | 原combinations9/9通过；完整不变量partial，延续R4-01 |
| R3完整身份 | bad/template影子原门均exit101，contract_pass=true；源码完整仓根相对路径等值匹配，关闭 |
| missing/原意外损坏required | 原门均exit101，contract_pass=true |

新增分隔矩阵16例：12例与旧oracle一致、3例为上述阻断、1例为安全拒绝的边界差异。**id6** `members: ["a", /*c*/, "b"]` 旧臂接受，新值层AtomParser拒绝第二个逗号；这是探测到的双逗号/注释混排限制，没有假static。双逗号并非T01冻结最小语法的必需项，本轮不将它列为第二个阻断或要求扩张支持面；后续Spec应如实保留该限制，不宣称所有VM逗号混排均受支持。矩阵`probe_pass`仅表示探索性oracle一致，不直接等于最终AC判定。

矩阵通过面含#行注释、单行块注释、显式逗号、实际CRLF、嵌套builtin换行、非法先行逗号/同行相邻及对象换行后逗号反控；归一化字段逐项相等或不假static。覆盖新增trivia分支，不以原九例通过代替完整审查。

## 验收映射与门禁

| AC→Task | 结论/证据 |
|---|---|
| AC01→T02/T03/T05 | pass；consumer3/3，normal deps无compiler/man/VM/UI |
| AC02→T01/T02/T03/T05 | partial；全受管170正常面、原反例通过，R4-01阻断 |
| AC03→T02/T03/T04/T05 | partial；路径/owner/生产读取零切换通过，语法分类仍阻断 |
| AC04→T06 | 独立审计完成，final未pass |
| AC05→T07 | merge-owned pending |

本轮实跑model29/29、consumer3/3、adapter11/11、separator oracle2/2；正常corpus **170=157 static+9 dynamic+2 template+2 both**。check(auto-lang+auto-man)、model及adapter/consumer格式、diff-check通过；生产pac.rs/vue.rs/rust_ui.rs/asset.rs/target.rs/auto-lang/src diff为空。见[runs.jsonl](runs.jsonl)、[supplemental-runs.jsonl](supplemental-runs.jsonl)及各命名log。未发现本次新增未处理警告，不声称全仓历史警告已清零。

复放（所有构建/执行定位Plan树，主检出仅存证据）：

```text
python docs/plans/attachments/780/review-r4/run.py
python docs/plans/attachments/780/review-r4/replay.py --implementation D:/autostack/.wt/lang-780/auto-lang
python docs/plans/attachments/780/review-r4/bounds.py
python docs/plans/attachments/780/review-r4/combinations.py
python docs/plans/attachments/780/review-r4/gate-negative.py
python docs/plans/attachments/780/review-r4/gate-malformed.py
python docs/plans/attachments/780/review-r4/gate-shadow-bad.py
python docs/plans/attachments/780/review-r4/gate-shadow-template.py
python docs/plans/attachments/780/review-r4/separator-matrix.py
```

前八步通过，最后探索矩阵exit1（上述三阻断例+一个非阻断边界差异）。独立截止子进程均由父进程负责回收。临时index仅注入测试子进程，真实index不变；辅助代码在ignored scratch，不修改实现。

## 交接

保留T01/T04，重开T02/T03/T05，T06未pass/T07待办，**executing 2/7**，revision1不变。下一轮work先纠正“物理换行=语法分隔”的根因与覆盖假设，再补完整正反矩阵并提交复审。既有合同仍有效，无需自动needs_replan。

merge-owned裸cargo t在最终review通过后于冻结提交执行；不触VM/codegen，无tv/tu/tt/taa触面门。本轮未跑任何整档、未发布canonical Spec/ledger、未合入/归档/清理。
