# PLAN-780 独立 final review R2

2026-10-15；结论 **needs_fix**。原 R1 的 12 个独立控制全部通过，但本次新增边界控制发现两个阻断项。生产读取尚未切换；不能合入。

## 审查身份与冻结面

审查者参与拆分规划及 R1 审查，未实施780；依据实际差异和独立输入重建证据，不以 work 勾选作证明。plan_revision=1，review_scope=final。

- worktree：`D:/autostack/.wt/lang-780/auto-lang`；branch：`plan-780-dev`。
- reviewed_commit：`ee67ea5df5b7cf47a6eaea30bc798489e1ef6618`。
- base_commit：`075a9b4ab6cb4284f3c259b348f2571322107fec`；R1 tip：`6aa07b630939966ac793c9f4cf7b07060330d736`。
- 只读兄弟 auto-down：`c39ac7f9dacdc769fc4a5ad3b98ec896240b28e8`。
- auto-man/project Spec SHA256：`bb857003628365be87cec91912ab36c672388a943163d5fc3584fc44600e80c3`；新 project-model Spec 尚不存在。
- 锁、受管输入及最终复核见 [fingerprints.json](fingerprints.json)。规范增量见 [spec-delta-reviewed.md](spec-delta-reviewed.md)。

## 阻断 findings

### F780-R2-01 / P1：值位扫描器不能保证前进，短输入可永久循环

`crates/auto-project-model/src/scanner.rs:1092` 的未知字符分支返回 None 而不消费字符；`scan_array`（1162起）重复调用后仍停在同一位置。`scan_object`（1121起）在缺冒号分支（1149）continue 也没有进度保证。注释说交给 AtomParser 报错，但控制流无法退出到该解析器。

独立子进程逐项执行，2秒截止后由父进程终止并回收自己的子进程：

| 输入 | 结果 |
|---|---|
| `members: ["front", "back"]` | 正控通过 |
| `back: { project: "back" }` | 正控通过 |
| `members: [1 * 2]` | 超时；旧臂已返回 members=["2"] |
| `members: [1 : 2]` | 超时 |
| `members: [1}` | 超时 |
| `back: { @: 1 }` | 超时 |

证据：[bounds.rs](bounds.rs)、[bounds.py](bounds.py)、[bounds.json](bounds.json)。记录中的 exit1 是父进程终止超时子进程，不是模型返回的错误。2秒仅是审查护栏，不新增吞吐或规模合同。算术表达式应明确拒绝或分类，不要求轻模型执行它。

影响 AC02/03、V03/05，重开 T02/03/05。修复需保证循环每步消费输入、退出或返回错误/unsupported，并永久覆盖数组/对象/顶层对象中的未知token、错配闭符、缺冒号及表达式；用外部截止控制保护终止性测试。

### F780-R2-02 / P2：双臂拒绝豁免仍未绑定预期坏输入

`crates/auto-man/src/project_model_adapter.rs:474` 对任意 `(Err(_), Err(_))` 直接计入 both_rejected，无输入身份或预期诊断约束。439附近的 read_error 检查已修复缺失/编码错误，但重命名分类桶没有收紧异常范围。

独立控制复制真实 Git index，仅在测试子进程设置 GIT_INDEX_FILE，加入额外 required 文件 `scratch/review780-r2/unexpected-required-pac.at`（内容 `name: [\n`，不属于批准的坏fixture），调用原 `project_model_corpus_comparison_v02`：实际 cargo exit0；计数由170=157+9+1+3变为171=157+9+1+4。新增损坏输入自动获得豁免，门禁假绿。真实index及源码未修改。

证据：[gate-malformed.py](gate-malformed.py)、[gate-malformed.json](gate-malformed.json)、[gate-malformed.log](gate-malformed.log)。控制脚本以 exit1 报告合同失败。

影响 AC02/03、T03/05。需明确有限的预期坏输入/模板例外，并约束输入身份及预期stage/诊断；意外或被损坏的 required 输入必须失败/待调查，不能仅把本次 scratch 路径加入白名单或降低总数。R1-F04 的具体 I/O 反例关闭，例外范围不变量仍未完成。

## R1 闭合与验收矩阵

| 原项 | 本轮独立结果 |
|---|---|
| R1-01 嵌套值位变量 | 原反例通过；新增扫描器终止性回归见 R2-01 |
| R1-02 重复 dep path | 末声明覆盖、重复及行号保留，通过 |
| R1-03 未知子树/args | children/raw_args 哨兵可恢复，通过 |
| R1-04 读取失败 | missing/UTF8/empty 区分及 missing-required 原门反控通过；例外范围见 R2-02 |
| R1-05 wrapper | RootWrapper unsupported、原节点保留，通过 |
| R1-06 格式 | standalone consumer 格式通过 |

| AC | 结论 |
|---|---|
| AC01 | pass：consumer及normal依赖闭包无lang/man/VM/UI |
| AC02 | partial：冻结静态正常面与原反例通过，终止性/意外坏输入门禁未过 |
| AC03 | partial：路径/owner/生产读取不变通过，坏输入诊断仍阻断 |
| AC04 | 独立审计完成，final未pass |
| AC05 | merge-owned pending |

## 实测与重放

model27/27、consumer3/3、adapter7/7；正常corpus170=157 static相等+9 dynamic+1 template_rejected+3 both_rejected。check（auto-lang+auto-man）、model/adapter/consumer格式及diff-check通过；生产 pac.rs/vue.rs/rust_ui.rs/asset.rs/target.rs 与 auto-lang/src 未切换。证据见 runs.jsonl、各命名log及 consumer-deps.log；仓内已有警告不在此宣称已全部清零。

原12个控制源码不变，12/12通过；missing-required 原门cargo exit101，反控 contract_pass=true。新增终止控制2pass/4timeout；新增损坏required反控 contract_pass=false。执行顺序：

```text
python docs/plans/attachments/780/review-r2/run.py
python docs/plans/attachments/780/review-r2/replay.py
python docs/plans/attachments/780/review-r2/gate-negative.py
python docs/plans/attachments/780/review-r2/bounds.py
python docs/plans/attachments/780/review-r2/gate-malformed.py
```

后两项预期在本冻结提交 exit1；bounds依赖replay生成的当前库定位。所有构建与测试在Plan树，独立辅助代码/临时index在其ignored scratch；真实index未改。无修复代码、canonical Spec/ledger发布、合入、归档或worktree清理。

## 交接

保留独立核实的T01/T04，重开T02/T03/T05；T06未通过、T07待办，**executing 2/7**，revision1不变。连续两轮 final needs_fix，下一次 work 应先补根因分析及覆盖矩阵/设计调整，再闭合全部finding；不能只追加旧反例。当前未要求改合同，故不自动needs_replan。

最终review通过后，merge阶段在新冻结提交执行裸cargo t；本轮未运行任何整档t/tv/tu/tt/taa/tf，merge-owned门禁仍pending。
