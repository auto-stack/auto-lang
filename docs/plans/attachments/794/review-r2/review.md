# PLAN-794 独立最终复审 R2

2026-10-16；stage=review；review_scope=final；**outcome=pass**。该结论通过代码、验收及规范增量审查，合入门禁与实际交付仍pending。

## 基线与修复范围

- reviewer_identity：当前Codex独立审查上下文，未参与实施；延续R1，未另建agent/工作树。
- plan_revision=1；worktree=`D:/autostack/.wt/lang-794/auto-lang`；branch=`codex/plan-794-dev`。
- reviewed_commit=`d027e5f617a3d33bb8b1897779569020f886f0e2`；初始base=`8b97d3c9bc9c86cb4d8ebde4a5d7856554e83b9c`；R1受审基线=`91974d8f8d826a9cdb3ea6a50038a518bc02c4e9`。
- auto-down组内兄弟仍为`c39ac7f9dacdc769fc4a5ad3b98ec896240b28e8`。入场/结束HEAD一致，受管实现clean；本轮临时补控文件已移除。
- R1→R2仅3文件：模块边界baseline重锚、plan794测试去一个mut、VM探针查询刷新/失败传播/三个新控制。没有生产代码、依赖声明、UI生成器、renderer或状态机变更。
- 实读输入hash：[implementation-inputs.json.txt](implementation-inputs.json.txt)、[spec-inputs.json.txt](spec-inputs.json.txt)。主检出其他计划与ledger的既有WIP保持；本轮只写794复审附件、计划簿记与债务关闭记录。

## Findings closure

| Finding | 结果 | 独立证据 |
|---|---|---|
| F-794-R1-01 / P2 | CLOSED | strict实跑exit0，0 violations、2条既有窄grandfather、ownership完整。baseline只变base_revision与core_to_ui文本统计；declared_edges及配置/豁免不变；新base=7a566c89b7fe88a8c79e6ffbd5b28ba4ee9e4031，为受审HEAD祖先。未用放宽规则消红。[报告](modularity.json.txt) |
| F-794-R1-02 / P2 | CLOSED | 旧确定性反例现exit0，失败点击返回False且不读取旧成功标签。[旧例重放](old-counterexample.log)。真实Driver的6项独立控制全过：首轮点击失败、pending/pending/applied、pending后点击失败、首读applied、持续pending超时拒绝、永久负控自身通过。[源码](oracle-controls.py)、[结果](oracle-controls.log)。原VM探针实际60/60；旧57个case一个未删，上轮旧key换绑反例现samples=['invalid:not_found']。[日志](ui-vm.log)、[原始报告](vm-report.json.txt) |
| F-794-R1-03 / P3 | CLOSED | 126行移除无用mut；定向9/9通过，输出无plan794测试或adapter文件告警。存量其他文件告警未冒称全部清零。[targeted.log](targeted.log) |

查询修复每轮重按只读Q按钮，未重发Go入队；点击失败立即返回，样本顺序记录。真实重入队控制本轮只采到`applied`，不冒称直接采到了pending；入队置Pending由未变状态机及既有注册表测试证明，独立pending/pending/applied控制验证了新循环确实重新查询三次。新查询路径的持续失败、后续失败和成功退出均检查，不能由原来单个旧例闭合代替。

上轮非编号链接红项也已关闭：`wt-guard`入场及本轮VM/补控运行后均clean，未重建Vue生成目录或引入链接；最终结果见[wt-guard.log](wt-guard.log)。不在review移除工作树。

## AC与验证映射

| AC | tasks | 结果 | 证据 |
|---|---|---|---|
| AC-01 | T01..03/T05 | pass | 外部no-ui consumer5/5；R1独立源码八native控制在当前基线重放通过；剪断结构锚包含在9项定向测试 |
| AC-02 | T01/T03..05 | pass | 当前ID/arity/缺席/精确参数定向测试；真实VM60/60；F02旧例与失败出口补控通过 |
| AC-03 | T02/T03/T05 | pass | 实例/默认优先/Clone/merge定向通过；独立32轮全部shim在位/缺席、空句柄、拒绝与状态、两种to形态，栈/heap/pool份额及健康计数回基线 |
| AC-04 | T04..06 | pass | 薄adapter与原renderer生产代码未变，真实VM60/60；按下述指纹复用R1 Vue59/59；未迁族清单/有效strict/guard通过 |
| AC-05 | T07 | pass | 三finding关闭，无新增合同内遗漏/延期/workaround；SD01..04冻结复核通过 |
| AC-06 | T08 | pending | merge-owned冻结门禁、规范/ledger/归档/guard清理与实际合流未执行 |

原工作树实际执行（SCCACHE_SERVER_PORT=42917）：

```text
cargo t scroll_host_provider                          9/9 pass
cargo test --manifest-path test/fixtures/modularity/scroll-provider-consumer/Cargo.toml
                                                      5/5 pass
cargo test --manifest-path ... --test review_r1         R1独立源码/ownership控制2/2 pass
python <R1 query-oracle-control.py> <当前vm_probe.py>     exit0
python <本轮oracle-controls.py> <当前vm_probe.py>         6项pass
AUTO_BIN=<原树CLI> P781_T05_OUT=<原树target本轮目录>
python examples/capability-tests/p781-scroll-child-anchor/tests/vm_probe.py
                                                      60/60，exit0
python scripts/modularity-audit.py --mode strict       pass/0 violations/2 grandfathered
rustfmt --edition 2021 --config skip_children=true --check <唯一改动Rust测试文件>
                                                      exit0
git diff --check <初始base> HEAD                       exit0
Git Bash wt-guard.sh <原树>                            clean/exit0
```

consumer结果见[consumer.log](consumer.log)，源码/ownership结果见[source-ownership.log](source-ownership.log)，使用[R1保存的原补控源码](../review-r1/probes.rs)，临时复制到consumer/tests后运行并仅移除该文件。未修改受审实现。

双端验证沿用本会话已读autoui-verifier技能。新VM报告保留真实offset变化、目标正面积交集、首绑/换绑/失锚、双pane与嵌套隔离、managed拒绝、已applied后用户滚动不回拉等原case；新oracle没有降低这些断言。当前CLI SHA256=`f9690405a4b66c727f4001db2d2647713f9b44ba852c500d94ffd69230cb90c9`，与R1真实CLI完全相同。

明确复用R1 [Vue59/59证据](../review-r1/vue.json.txt)及[执行日志](../review-r1/ui-vue-vmback.log)：R1→R2没有生产或Vue源/fixture/探针变更，当前根Cargo.lock、Cargo/nextest配置、模块规则以及CLI/Vue探针hash均与R1相等；变化仅VM探针、Rust测试mut和审计baseline。所复用形态仍为Vue前端+VM API后端，不新增“Rust backend已验证”声明。生成目录已被work清理；没有重新安装依赖或产生新的Vue资产，故复用已记录的同源执行证据，不把不存在的生成目录写成新构建通过。本轮目视复核新VM最终截图；双端没有新增视觉实现变化。

## Spec delta与后续门禁

四份canonical目标Spec与R1实读hash全部相同；[本轮冻结SD](spec-delta.md)与R1字节相同，SHA256=`84d2b2776faba28671437fbc91e708986408adb1240057639689923cca4c5bac`。SD01..04的owned能力边界、缺席/装配优先关系、四装配点、单源registry/renderer与781完成oracle仍准确；T01表9903旧弹序文字已同步勘定结果，历史说明保留。supersedes四个真实路径、new空与GOAL-003/007保持合理。未发布canonical Spec或修改live ledger。

遗漏/延后/workaround扫描未发现新项；R1开放结论保留为历史，当前三项均CLOSED。Plan更新为`reviewed 7/8`，T07完成，T08仍pending。

**pending_closeout_gates**：merge须在本次冻结代码（或经验证实现/依赖等价的收口后代）执行裸`cargo t`+`cargo tv`，再沉淀Spec/ledger、实际合流、归档与guard清理；ui_gen零触面不触发tu，tf仍按批量到期规则在主检出单实例执行。本轮未运行任何整档或docs_gen、未修实现、未合入、未归档，不宣称交付完成。
