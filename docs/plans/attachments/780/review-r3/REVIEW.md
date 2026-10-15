# PLAN-780 独立 final review R3

2026-10-15；**needs_fix**。上轮终止性与原坏输入反例均已通过；发现合法静态语法被新分隔检查误拒，以及豁免身份按后缀匹配造成的门禁假绿。不能合入。

## 冻结基线与审查范围

- plan_revision=1；review_scope=final。审查者为此前规划及R1/R2评审会话，未实施780，依据实际diff及独立输入复核。
- worktree=`D:/autostack/.wt/lang-780/auto-lang`；branch=`plan-780-dev`。
- reviewed_commit=`61eea96971c1290c745da4ff74e80795834aa9cb`；diff_base=`075a9b4ab6cb4284f3c259b348f2571322107fec`；上轮=`ee67ea5df5b7cf47a6eaea30bc798489e1ef6618`。
- auto-down只读兄弟=`c39ac7f9dacdc769fc4a5ad3b98ec896240b28e8`；锁/受管输入与最终复核见 [fingerprints.json](fingerprints.json)。
- auto-man/project Spec SHA256=`bb857003628365be87cec91912ab36c672388a943163d5fc3584fc44600e80c3`；新project-model Spec仍不存在。[规范增量冻结](spec-delta-reviewed.md)。

已先核验连续两轮失败后的诊断与设计调整：work明确零消费分支根因、遗漏终止性测试原因，加入永久12例边界/3正控、线程recv_timeout截止、separator VM oracle及异常清单。该交接具备复审条件；但实现还未满足全部不变量。

## F780-R3-01 / P2：新分隔检查破坏已冻结的注释/字面量容器支持

位置：`crates/auto-project-model/src/scanner.rs:1073`（check_separation）及1113（字面量标识符后跳过全部空白/注释）。check_separation只跳空格，不识别值后的注释，将`/`直接分类ValueSyntax。另一方面true/false/nil分支提前skip_ws_and_comments吞掉换行，后续分隔检查把下一键/值当作同行相邻语法。

独立进程先执行旧VM oracle再解析新模型，每例2秒护栏，9例中 **4pass/5fail，无超时**：

| id | 输入要点 | 旧臂 | 新模型 |
|---|---|---|---|
| 0 | `members: ["a" // comment\n, "b"]` | members=[a,b] | unsupported ValueSyntax `/`，members=None |
| 1 | `members: ["a" /* block */ , "b"]` | members=[a,b] | unsupported ValueSyntax `/`，members=None |
| 2 | `cfg: { a: true\nb: false }` | Ok | unsupported ValueSyntax `b`，raw值丢失 |
| 3 | `cfg: [true\nfalse]` | Ok | unsupported ValueSyntax `f`，raw值丢失 |
| 4 | `{ name: "ok", back: { project: "b" /* comment */ } }` | name=ok、back=b | 整文档unsupported，字段未提取 |
| 5..8 | 字符串尾逗号/数字换行/值前注释/布尔显式逗号 | Ok | static，通过 |

证据：[combinations.rs](combinations.rs)、[运行器](combinations.py)、[逐例旧新结果](combinations.json)。五个失败均为新模型分类错误，不是旧oracle失败或终止护栏触发。

这是R2修复的兼容性回归，非新增语言需求：T01冻结报告§1/§3包含字面量数组/对象与任意注释；当前work自己也以VM确认换行分隔合法。合法输入分类非static并不是完整兼容。影响AC02/03、V02/V05、T02/T03/T05。

修复应在保留终止保证下统一处理trivia与分隔边界，保留“是否经过合法换行”的信息；避免按字符串/数字/布尔走不同分隔语义。永久矩阵需覆盖注释在值前/后、逗号前/后、闭符前，及字符串/数字/builtin与array/object/top-object组合，并保留同行无分隔负控。无需把表达式求值搬入轻模型。

## F780-R3-02 / P2：豁免清单后缀匹配不能绑定输入身份

位置：`crates/auto-man/src/project_model_adapter.rs:516`、531。两类异常均以Path::ends_with匹配；坏输入清单仅文件名，模板清单也包含短后缀。目录不同的文件仍取得豁免。

沿用原门禁控制方式：复制真实index，测试子进程设置GIT_INDEX_FILE，加入一个实际存在的额外required输入；真实index/源码不变。两次运行均 **cargo exit0**：

| 控制 | 未获批准的输入 | 实际计数 |
|---|---|---|
| [bad影子](gate-shadow-bad.json) | `scratch/review780-r3/shadow/bad-missing-brace.pac.at`，内容`name: [\n` | 171=157 static+9 dynamic+2 template+3 both |
| [template影子](gate-shadow-template.json) | `scratch/review780-r3/shadow/fixtures/dynamic-expressions.pac.at`，内容`name: unknown_symbol\n` | 171=157 static+9 dynamic+3 template+2 both |

脚本：[gate-shadow-bad.py](gate-shadow-bad.py)、[gate-shadow-template.py](gate-shadow-template.py)；完整原门日志为同名.log。两个控制均应非零，审查脚本以exit1表示合同失败。模板控制是未定义变量导致旧臂拒绝，并不声称其内容是缺括号语法错误。

原`unexpected-required-pac.at`反控已通过，但R2-F02的完整身份不变量仍未闭合。影响AC02/03、T03/T05。清单需按统一仓根相对路径**精确匹配完整身份**并校验预期分类/诊断，拒绝同名和同后缀影子；不要只加本次scratch路径特判。永久负控覆盖两个异常桶。

## 旧项闭合、验收与证据

| 项 | 本轮结论 |
|---|---|
| R1六项具体反例 | 12/12重放通过，格式通过；继续保留 |
| R2-01 原四超时及正常控制 | bounds6/6，无超时；源码前进路径核验通过。终止性项关闭，新的兼容性回归单列R3-01 |
| R2-02 原意外坏输入 | 原反控cargo exit101，通过；完整身份约束partial，延续为R3-02 |
| 缺失required | 原门cargo exit101，contract_pass=true |

model28/28、consumer3/3、adapter8/8、separator oracle1/1；正常corpus **170=157 static+9 dynamic+2 template+2 both**。check(auto-lang+auto-man)、model及adapter/consumer格式、diff-check通过。normal consumer依赖无compiler/man/VM/UI；T04字段owner和生产入口未切换证据不变。见 [runs.jsonl](runs.jsonl)、[supplemental-runs.jsonl](supplemental-runs.jsonl)、各命名log。已有全仓警告不宣称本轮清零；未发现本次新增未处理警告。

| AC→任务 | 结果与依据 |
|---|---|
| AC01→T02/T03/T05 | pass；consumer及normal deps运行证据 |
| AC02→T01/T02/T03/T05 | partial；正常全受管面和旧控通过，R3两项阻断 |
| AC03→T02/T03/T04/T05 | partial；owner/路径/无生产切换通过，合法分类及门禁阻断 |
| AC04→T06 | 独立审计完成，final未pass |
| AC05→T07 | merge-owned pending |

复放顺序（在主检出调用，脚本中的构建/测试均定位Plan树）：

```text
python docs/plans/attachments/780/review-r3/run.py
python docs/plans/attachments/780/review-r3/replay.py --implementation D:/autostack/.wt/lang-780/auto-lang
python docs/plans/attachments/780/review-r3/bounds.py
python docs/plans/attachments/780/review-r3/gate-negative.py
python docs/plans/attachments/780/review-r3/gate-malformed.py
python docs/plans/attachments/780/review-r3/combinations.py
python docs/plans/attachments/780/review-r3/gate-shadow-bad.py
python docs/plans/attachments/780/review-r3/gate-shadow-template.py
```

最后三项本提交预期exit1；前三类旧控制通过。辅助代码与临时index仅在ignored scratch运行；没有修改实现、真实index、canonical Spec或ledger。

## 交接

保留T01/T04，重开T02/T03/T05，T06未通过、T07待办，**executing 2/7**，revision1不变。下一轮work需扩展trivia/分隔组合与精确身份矩阵，闭合全部finding，提交后复审。既有合同仍有效，无需自动needs_replan。

未运行任何整档t/tv/tu/tt/taa/tf，未merge/归档/清理；最终review通过后由merge在冻结提交执行裸cargo t及Spec/ledger/合流/批回归到期判定/guard-clean。生产读取未切换，本轮发现影响pilot验收。
