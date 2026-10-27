# PLAN-814 R12：R11 修复后的完整 T-10 复审，needs_fix

2026-10-21；stage: review；review_scope: final；plan_revision: 3。
reviewer_identity：本聊天 Codex 独立复审上下文，未参与实施；未创建 agent 或新 checkout。实施摘要与关闭声明均重新对照实际代码及真实编译。

## 绑定与变更边界

- 原树 `D:/autostack/.wt/lang-814/auto-lang`，分支 `codex/plan-814-dev`。
- reviewed_commit `1bbf986208b2486e4e899dd58209aecacc3a0608`；tree `7de5a0e12a009f88aa310a07179b4db3e3ab683f`。
- 修复 base=R11 `49206c6992b177cc02e84f35be34a885a2ebd817`；B base=A 发布 `f6e07aceeeea3cb12698c747d61db5be43f7458c`。
- auto-down `c39ac7f9dacdc769fc4a5ad3b98ec896240b28e8`，实施/依赖树 clean；初末复核见 `input-hashes.json`。
- 相比 R11，仅 `crates/auto-ui-compiler/src/ui_gen/rust.rs` 改动（77+/7-），未改 Cargo、feature、选择配置、测试声明、SDK、runtime、发行驱动。R11 记录的 44 份源码/配置输入只有此文件变化。
- 主计划、handoff、SD 及十一份 canonical Spec 已冻结；canonical 版本均与 R11 一致，runtime Spec 仍为待新增。impact 元数据精确指向十一份现有 Spec 与新 runtime Spec，GOAL-003/007 保持。没有发布 canonical 或 ledger。
- `prior-artifact-drift.json` 为空：本轮 work 正确复制至 r11-closeout，未覆盖 R11 冻结证据。本轮所有再生均在新 review-r12。

## 两项 finding 核销

| ID | 结果与边界 |
|---|---|
| F814-R11-01 / P3 新增多余括号告警 | **CLOSED**。数字扫描的 if 多余括号已去掉；四包匹配检查 exit0，compiler 37→36，runtime 235 保持。原有其它同类告警不重开，生成探针本身 unused 等告警不算产品新增告警。 |
| F814-R9-01 / P2 比较类型保持 | **旧例关闭，整项仍 OPEN**。原 R11 七个失败组合均变绿，work 新增十个 feature 也全部实际编译通过；但透明分组只处理了“另一操作数”，比较字段自身加括号仍误走 truthy，算式类型仍由紧邻 token 决定，详见下表。 |

无新增独立 finding；以下是既有 AC-03/06、T-05/T-08/T-09 的 R9-01 同一不变量未闭。不能把“每个类型单例能编译”解释为表达式类型保持已经完成。

## 独立编译证据

真实链路为 Core Parser（UI session）→AURA→当前 RustGenerator→canonical runtime/SDK Rust 编译。46 个输入均成功解析/生成。Value prop 沿用既有合法 `Note=serde_json::Value` 外部类型别名；没有把探针缺少类型定义误判为产品错误。

复制 work 的 26 feature 和无 feature 正控后独立再生、逐项检查，**27/27 exit0**，包括原负数/浮点/左右位/prop/状态 int/bool/float=f32/字符串、嵌套分组另一操作数及 i64 宽度。这些原例现在确认关闭，旧类型身份 pin 也随默认正控再次编译通过。

新增九个代表组合，**1 pass / 8 fail**；共 35 feature 加默认正控，合计 **28 pass / 8 fail**。结果逐项记录在 `results.json`。

| fixture / 条件 | 实际生成及结果 |
|---|---|
| group-field-int：`(p.rank) > .limit`，limit:int | `(truthy(&(p["rank"]))) > self.limit`，bool 与 i32，E0308，exit101 |
| group-field-int-rhs：`.limit < (p.rank)` | `self.limit < (truthy(&(p["rank"])))`，i32 与 bool，E0308，exit101 |
| group-field-float：`(p.score) > 0.5` | bool 与 float，E0308，exit101 |
| group-field-float-rhs：`0.5 < (p.score)` | float 与 bool，E0308/E0277，exit101 |
| group-both-int：`((p.rank)) > ((.limit))` | 字段仍被 truthy 包裹，bool 与 i32，E0308，exit101 |
| prop-group-field：`(note.rank) > (.limit)` | `truthy(&(self.note["rank"]))` 与 i32，E0308，exit101 |
| float-arithmetic-lhs：`(.threshold + 0.5) < p.score`，threshold:float=f32 | `(self.threshold + 0.5) < p["score"].as_f64().unwrap_or(0.0)`，f32 与 f64，E0308，exit101 |
| prop-float-arithmetic：`(.threshold + 0.5) < note.score` | 同构 f32 与 f64，E0308，exit101 |
| float-arithmetic-rhs 正控：`p.score > (.threshold + 0.5)` | 按 threshold 声明型投影 `as_f64() as f32`，exit0。仅交换操作数位置，另一侧就错误改为 f64。 |

相邻原正控 `p.rank > .limit`、`.limit < p.rank`、`p.score > 0.5`、`0.5 < p.score`、`note.rank > .limit` 均过；增加透明分组不应改变这些条件的类型。算式正反控不是增加新的语言能力要求，而是现有支持的状态值加浮点字面量、比较和分组的组合。

### 原因与修复边界

`rust.rs:8638` 的 `comparison_context` 只查字段紧邻的运算符，未透明跨过字段自身的 `(`/`)`；loop/prop 在 `:8920`、`:8995` 因而走条件位 truthy 分支。`comparison_operand_type`（`:8683`）虽然跨过另一侧的分组，但仍只取紧邻一个 token，向后扫描浮点算式得到 `.threshold`→f32，向前扫描相同算式得到 `0.5`→f64；没有取得同一操作数的完整类型。

应统一识别比较节点、透明分组和操作数表达式类型，再决定值投影；保留 state_types 的合法声明宽度、已知 shape 访问器与纯条件 truthy。不要再只对本表的括号/尾 token 打补丁。验收边界维持原合同，代表矩阵为 **字面量/声明值/已支持算式 × 字段及另一侧的分组 × loop/prop × 左右位 × int/float/bool/str**；类型错误及静默值语义都需有区分性 oracle。work 先完成全部修复、定向验证并提交，再送审；无需整档回归。

复现（在原 worktree，附件使用绝对路径）：

```text
cargo build --offline --locked --manifest-path D:/autostack/auto-lang/docs/plans/attachments/814/review-r12/Cargo.toml --target-dir target --bin r10-generate --bin r10-overlay
python D:/autostack/auto-lang/docs/plans/attachments/814/review-r12/replay.py
```

`replay.py` 保留每项真实退出码，不把预期失败伪装成编译通过；结果及所有源/产物/日志哈希见 `input-hashes.json`。后续 work 必须复制到新修复目录运行，保留本冻结件。

## 完整 T-10 对账及复用

| AC / 任务 | 结论及证据边界 |
|---|---|
| AC-01 / T-02,T-06 | **pass，限定复用**。R9 无 UI 物理源码快照 fresh+locked SDK consumer 5/5、四非 UI consumer 闭包与构建输入证明；R11 后 SDK/manifest/lock/输入规则与依赖未变，当前仅 UI RustGenerator 改动。没有声称本轮重新执行隔离构建。 |
| AC-02 / T-05 | 旧 API/type pin 本轮默认正控实际编译过；r3 例外/Parser 单源、provider43/43及六 feature 形态证据按原未变实现复用。**T-05 仍因已承载的生成路径缺陷不能完成**，不重开已经发布的 A 兼容装配事项。 |
| AC-03 / T-04,T-05,T-08 | **fail**，R9-01 如上。T-04 runtime 迁移/宿主闭环保持；本轮真实 overlay 八组合再次 mismatches0/exit0。R9 counter Vue/VM 的 0→-1→Reset→1 及 R11 runtime65定向结果按未变源码/配置/输入复用；RustGenerator 条件路径本轮已实际编译，不能用旧 e2e 绿掩盖当前失败。四生成器路径策略/canonical 模板/其它三生成器未变。 |
| AC-04 / T-06 | **pass，限定复用**。无新 SDK↔UI 余边或单源状态改动；R9 provider/frame/i18n/theme/mem_guard44及六provider43，R11 runtime65覆盖按未变代码/依赖复用。当前真实 strict 再次验证零逆向边。 |
| AC-05 / T-07,T-08,T-09 | 选择/发行所验 **pass**；规范关闭声明 **partial**。九档 R6 原→A strict 逐 ID、R8 A→B 挂载路径精确对账、R9 九档逐 ID/R10 t与tu实际枚举（5328/868 unique）保持；没有测试声明或选择配置变化，复用而未整档。R9 冻结792 driver24/24真实构建/重定位/负控与runtime独立consumer5/5依赖模板/producer/输入未变，限定复用；当前生成条件的失败已单独揭示。777 strict 本轮 exit0、violations0、grandfathered3，锚2f83c4270及豁免未变。 |
| AC-06 / T-10 | **needs_fix**。代码/格式/告警/遗漏与 SD 审查已完成；当前 R9-01 未闭，不授 pass。 |
| AC-07 / T-11,T-12 | **pending，merge owner**。本轮未运行整档、合入、Spec/ledger落库、归档或清树。 |
| AC-08..10 / T-13..15 | **保留 A 已发布边界**。R7 r3 审定及 T15 实际发布不因本轮 B 缺陷回滚，也不提前宣布 B 发布。 |

健康检查：四包 cargo check、手写改动 rustfmt、修复 diff whitespace 均通过；新改动没有 debug/allow 绕过。R11-01 新告警已消失，预存警告按其边界保留。

Spec delta：SD-01/03/04/08 与未变的 runtime/SDK/发行实现及 A 保留规则继续相符；SD-05/06/07 的 B 事实注记不引入合同变更。SD-02 新增的声明环境/宽度说明与已通过单例一致，但“完整类型感知/透明分组”仍与上述产物冲突，**不能审定为已实现规则**。T-09 需随同一修复同步准确正文；本轮不将其删减为默认 str 或授权取消原比较能力。before-hash 与十一 canonical 输入一致，目标集无需修订。

## 路由

outcome: **needs_fix**；status: **executing**；T-05/T-08/T-09 重开，T-10..12 保持未完成，完成 T-01/02/03/04/06/07/13/14/15，**9/15**。next_responsible_role: **work**，同原树一次修齐 R9-01 的比较节点/分组/表达式类型不变量并提交复审。

最终 pass 后由 merge 在新冻结提交上统一执行 **裸 t+tv+tu+tt** 与必要 docs/schema/book 门，再实际合入和出 B 发布收据。handoff 旧段落的“tt按需”不替代主合同的触面要求；tf 仍由主检出到期单实例，aavm 原用户豁免保持。
