# PLAN-777 r1 — 独立最终复审 R1

> 日期：2026-10-15；outcome：**needs_fix**；review_scope：final。
> 实施树：`D:/autostack/.wt/lang-777/auto-lang`，branch `plan-777-dev`。
> reviewed_commit：`97df2f99785be53fd8a1a6ef70124725d304d576`。
> base_commit：`17bc8431457848b040bffa5c261e13b057771559`；实现提交 `0b5e79950381af5c7d486efc42e29bb8e315f6fe`。
> 评审会话参与过777规划/就绪核查，未参与本次脚本实现；本结论从提交、源文件与重新执行的正反控重建，不转述work的pass。没有另建评审工作树或调用其他agent。
> 使用 auto-plan-review；技能、依赖、输入与冻结增量指纹见 [fingerprints](fingerprints.jsonl)。新版用户AGENTS优先于工作树中的历史规则；Category A没有运行Cargo测试/编译或任何整档。

## 1. 验收结论

| AC | 结果 | 实际证据 / 缺口 |
|---|---|---|
| AC-01 | partial / 未通过 | 普通双扫稳定、输入hash与锁hash不变；但 `--output` 可以覆盖本次读取的配置，违背只读/另存要求（F03） |
| AC-02 | fail | 原有窄豁免/撤销/过期/宽豁免控制通过；多target表与workspace继承依赖漏报、optional激活被误grandfather（F01/02/04） |
| AC-03 | partial / 未通过 | 资产、根外5消费者、已有21条责任与启发式限制有记录；语法/AST、build调度和CLI责任缺失，且跨根required path一律禁用的规则与目标架构冲突（F05/06） |
| AC-04 | 审计已执行，未授予pass | 代码、反例、健康、规范增量已审；发现6项需修复，后续重新绑定新提交复审 |
| AC-05 | pending / merge-owned | 尚未沉淀Spec、投影、合入、归档或清理；不得借needs_fix执行这些动作 |

T-01..04的原work记录保留为历史；因上述缺口重新打开。Plan返回 executing，当前0/6；这不表示已实现的有用部分被删除，而是相关task的完整验收还没有成立。

## 2. 已通过的独立验证

- `python scripts/modularity-audit.py --help`：exit 0。
- `python scripts/modularity-audit.py --self-test`：7/7。逐项读过assertions，不把这个总数当全部不变量成立。
- 当前冻结提交两次 `--mode observe`：exit 0，stdout SHA-256同为 `8bbb1fe2c253087287491c6492c876934dad9ccd1768d6d995f53d663e2076f2`。
- 真实仓 `--mode strict`：exit 0，0 violation / 2 grandfathered；metadata对照36=36。这只证明当前实际输入通过，下面合法变体仍能触发假绿/误红。
- 扫描前后所有受管文件SHA-256一致，Cargo.lock哈希一致；实施树始终clean。`git diff --check 17bc84314..HEAD`通过。
- 真实源规模：2052个受管Rust文件/945894物理行；auto-lang/src 733/710556，UI五目录239/353361。包含测试/生成等，不是生产SLOC。
- 唯一target表的新增core→UI边被正确点名并exit 2（C01-control），是多个target反例的有效阳性对照。

原始运行信息：[primary-runs](primary-runs.jsonl)；完整观察报告：[a](observe-a.jsonl)、[b](observe-b.jsonl)、[strict](strict.jsonl)。第一次review helper结束时遇到Windows临时Git对象只读清理失败；修正的是本评审helper的清理逻辑，随后重跑全部控制，未修改被审实现。

## 3. Blocking findings

### F777-R1-01 [P1] 多个target依赖表错误复用最后一个table

- 归类：原合同缺口；AC-02，T-01/T-03/T-04。
- 源码：`scripts/modularity-audit.py:349–354`。构造列表只保存section/target_key，读取时却使用外层循环最后留下的target_sec。
- C01-multiple-target：Windows正常依赖fixture-ui-kit，Unix正常依赖fixture-lang-sdk。Cargo独立metadata确认两个声明均存在；审计报告却把SDK同时放到两个target，并漏掉UI。
- 实际：`strict verdict=pass / exit 0`。预期：保留每个target自己的依赖表，点名Windows core→UI禁向边、exit 2。不是不支持语义解析的问题，是已承诺manifest表读取错误。
- 修复方向：每个target按自己的table读取；normal/build/dev各表都不可互相覆盖。增加多个target及混合kind的永久正反控。

### F777-R1-02 [P1] workspace继承path使用了member基点

- 归类：原合同缺口；AC-02，T-01/T-03/T-04。
- 源码：`scripts/modularity-audit.py:361–371`。继承workspace.dependencies后，仍以member拼path。
- C02：根声明 `fixture-ui-kit = { path = "crates/fixture-ui-kit" }`，member声明 `{ workspace = true }`。目标manifest实际存在，Cargo oracle正常解析到UI包；工具却得到 `unresolved(crates/fixture-lang/crates/fixture-ui-kit)`。
- 实际：extraction_gap被写入报告，但strict是pass/0，UI域规则被绕开。预期：按声明来源用正确基点，点名实际新增禁向边；不可把锁缺失的允许gap扩大为合法manifest解析错误的假绿。
- Cargo的继承path按workspace根处理，见[官方实现](https://doc.rust-lang.org/stable/nightly-rustc/src/cargo/workspace/parser/mod.rs.html)。本次另有本机Cargo1.98的真实oracle，非仅文档推断。
- 修复方向：保留path来源；覆盖继承的normal/build/dev、target内继承及直接member path对照。

### F777-R1-03 [P1] 报告输出可以覆盖审计输入

- 归类：原合同缺口；AC-01，T-01/T-03/T-04。
- 源码：`scripts/modularity-audit.py:1501–1505`。直接写用户给定路径，没有与输入域做冲突检查。
- C03：在独立临时fixture调用 `--mode observe --output <repo>/.autoos/module-boundaries.toml`。
- 实际：exit 0；配置hash改变，TOML配置被整份JSON报告覆盖。前/后hash与原配置在controls里，原始实施树未被改。
- 预期：写入前拒绝输出与输入同一文件的情况，受管Rust、manifest、配置、冻结基线等输入保持原字节；stdout和安全独立输出仍正常。显式refresh只准写合法的基线目标，不得借目标参数覆盖其他输入。
- 修复方向：统一在写入前检查规范化目标与实际输入的别名/重合；永久正反控证明拒绝发生在输入被写之前。不是要求审计器控制调用者在shell中另行重定向stdout。

### F777-R1-04 [P2] 新增禁向性质被当成存量豁免

- 归类：原合同不变量的补充反例；AC-02，T-01/T-03/T-04。
- 源码：`scripts/modularity-audit.py:812–815,878–895`。基线key只用from/to/kind/target，丢失optional和scope等影响规则判定的字段。
- C04：optional=true的core→UI声明在规则下合法；它有供退出跟踪的owner note。冻结后只改成optional=false。
- 实际：新出现的非optional禁向依赖被grandfather，strict pass/0。预期：合法optional控制保持pass；改变成不被允许的必选依赖时判红，不能靠同名note扩大存量豁免。
- 修复方向：豁免绑定冻结时实际已存在的禁向状态/规则与足够的声明身份，而非仅包名对。补optional激活、scope变化及同target存量控制；不擅自扩大规则到所有optional边。

### F777-R1-05 [P2] 全部调研责任覆盖的声明不成立

- 归类：原合同遗漏；AC-03，T-02/T-04。
- 源码：`.autoos/module-boundaries.toml:189`起ownership表。
- 调研§4.1/4.2的AST与parser/Dialect是拆仓主焊点，§5.2还明确build调度和auto CLI归属。现有21条没有这些项的迁移owner/退出阶段；6个man split条目实际重复覆盖UI生成/服务，不能替代缺失的build/CLI类。
- C05列出parser.rs、ast.rs、dialect.rs、dialect/ui.rs、auto-man/src/builder、auto/src/main.rs，均确实存在，但都不在责任表；检查全表evidence也未找到覆盖这些类别的分组责任。成员域分类为cli/lang-monolith不等于迁移责任合同。
- 修复方向：可以逐项或明确分组登记，无须每个文件单独一行；须对照调研各类别建立覆盖表，并在漏一类时有失败控制，不能只验证已登记path存在。

### F777-R1-06 [P2] 跨根path一律optional是未经设计支持的新限制

- 归类：规则与现有目标设计冲突；AC-02/03，T-02/T-03/T-04。
- 源码：`.autoos/module-boundaries.toml:102–109`的 `out-of-root-edges-must-be-optional`，注释将其归因Design37§6。
- Design37§2/§6允许UI依赖独立语言SDK，开发使用显式组内path/patch，发行固定版本/revision；没有规定跨根normal/build依赖必须optional。optional本身也不解决发行来源/路径可用性。
- C06复制生产规则，构造允许方向的UI→独立SDK，显式开发path正常可由Cargo解析。实际审计只因external+nonoptional而exit 2，误称new_forbidden_edge。
- 修复方向：移除或限定这条未经支持的通用约束；按已同意的依赖方向/身份/发行职责做检查。若要新增“所有跨根path必须optional”的产品政策，需要独立的设计决策与契约修订，不能把它写成已存在规则。

## 4. 反例重放与覆盖边界

```text
python D:/autostack/auto-lang/docs/plans/attachments/777/review-r1/replay.py --implementation D:/autostack/.wt/lang-777/auto-lang
```

当前helper总体exit 1（预期：合同控制失败）；7个控制结果为1个阳性对照通过、6个合同控制失败。**不是**把被审CLI的假绿exit 0当成功。输入、变异前/后状态、返回码、具体边、Cargo声明oracle与ownership核查保存在[controls](controls.jsonl)；主工具自己的7项自测仍保持通过，两组计数分开。

所有反例都是对原冻结fixture的独立临时副本操作；新增边未提交，从而不由“stale manifest”掩盖实际判红能力。C01/C02/C06的Cargo oracle只在这些临时副本运行metadata，允许产生副本的临时lock；工作树实际Cargo对照始终--locked，原锁未变。没有Rust compile/test，也没有修改其他工作树。

复审边界：是metadata/文本候选/配置/基线检查，未要求整个Rust语义分析、所有合法Cargo扩展或全生态consumer运行。原候选启发式限制明确、6类分类控制已通过。UI→language观察配置当前只看auto_lang::前缀，不覆盖大量crate::ast/vm路径；这一观察范围必须明确，未把它臆造为完整架构依赖图。

## 5. 记录/规范勘误与知识增量

- work的t04-summary.md写2045/945837，但它引用的observe与strict原始JSON均是2052/945894；前者恰少fixture新增的7个Rust文件/57行。请同步摘要，不能改原始报告凑数。
- script头部及METHOD.strict仍写“脏树判红”，实际行为是strict允许脏树并审当前边集、refresh拒脏树；work备注已说明后者。统一help/报告/注释/Spec提议，使使用者不依赖不存在的门。当前允许审未提交新增边本身不违背原合同，无需为了纠正文字改变该行为。
- 全文与diff健康检查无格式问题；CLI打印属于正常结果，不是stray debug。没有触及生产Rust或Cargo manifest，Category A有效。
- SD-01目标与范围正确，现行Spec hash为 `3800856129eb9b5bd742fef9ea2aa0c94f0db780ab2a8ba01fc7ae8965dde49b`，主检出与实施树相同。具体工具行为尚未通过，所以增量只能partial，不得沉淀“严格门已可用”的现状。
- 原Spec delta已冻结于[spec-delta-reviewed.md](spec-delta-reviewed.md)，对应SHA在fingerprints。supersedes仍为docs/specs/design/repository-layout.md，new_components/touched_goals为空：仓库审计入口变化，没有新语言/产品目标，不虚构Goal。

## 6. 路由与剩余关口

next：同一 `lang-777` 树交work修F01..06，补永久反例与覆盖对账、更新摘要/方法说明，提交后再独立review。保留这次接受控制与原7项自测；修复后在新提交运行有区分力的脚本检查，不能以新增baseline/豁免让反例变绿。

merge-owned：Category A无t/tv/tt/tu/taa等Rust整档门；仍有T-06的真实Spec/投影/合流/归档、批回归到期判定与guard-clean清理，均pending。最终审查未通过前不得合入，本review不运行tf或发布canonical知识。
