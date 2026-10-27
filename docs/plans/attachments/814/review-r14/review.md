# PLAN-814 R14：R13 修复后的完整 T-10，needs_fix

2026-10-21；stage: review；review_scope: final；plan_revision: 3。
reviewer_identity：本聊天 Codex 独立复审上下文，未参与实施；未创建 agent/新 checkout。实际代码、区分性正反控及冻结身份构成结论。

## 绑定

- 原树 `D:/autostack/.wt/lang-814/auto-lang` / `codex/plan-814-dev`。
- reviewed_commit `42a4ef792e522d0b30e81f42f6fb25e254148ac1`；tree `abce0e2b0af2406b0a8ceb1687b97359b5673b91`。
- 修复 base=R13 `feae28e0f16beba7637f81c9499a4ed62fe40282`；B base=A实际发布 `f6e07aceeeea3cb12698c747d61db5be43f7458c`。
- auto-down `c39ac7f9dacdc769fc4a5ad3b98ec896240b28e8`；初末实施与依赖 HEAD/clean 稳定。main HEAD、当前源码/Spec/计划/SD/证据指纹见 `input-hashes.json`。
- 修复仅 RustGenerator 一文件53+/5-，完整消费 <=/>=，补 ! 与逻辑组合类型传播；44份前轮源码/配置输入只有该文件变化。SDK、runtime、manifest/锁、测试声明、选择配置、发行驱动及其它三生成器未变。
- R13 冻结附件无漂移。work保留的未跟踪复审快照/历史证据按仓规和用户说明保留，不作为实施WIP或额外阻断，不入库、不清理。

## 已关闭的边界

**R13 两处具体缺口均 CLOSED**：原六个 <=/>= 失败及三个布尔取反失败全部实际编译 exit0；R9-R12原例保持。work 的62feature+默认正控共**63组编译全部通过**，overlay八组合运行mismatches0，原64个检查已独立重放全绿。

独立再测 **六比较符 ==/!=/</>/<=/>= × 两方向 × loop/Value prop=24组**，字段和另一操作数都有分组、另一侧为已声明int算式，**24/24实际编译exit0**。完整运算符消费和既有分组不再重开；R11告警与R10浮层/777/serde项也保持关闭。

## F814-R9-01 / P2 仍 OPEN：比较表达式本身的 bool 结果未传播

影响 AC-03/06、T05/T08/T09。仍为原类型保持finding，不增加新独立finding或新的语言能力范围。

`classify_operand_expr`（`rust.rs:8904`，逻辑递归`:8930`）可识别bool字面量/声明值/取反/逻辑组合，却没有比较表达式节点的结果类型。对 `(.limit > 3)` 剥壳后，`.limit > 3` 不匹配纯状态、数字或四则节点，返回 Unknown；调用端将其视为str。该表达式实际返回bool，外层未知字段因此被错误投影为as_str。

| fixture / 条件 | 当前真实产物及编译 |
|---|---|
| bool-state-comparison：`p.dirty == (.limit > 3)`，limit:int | `p["dirty"].as_str().unwrap_or_default() == (self.limit > 3)`，str与bool，E0277、exit101 |
| bool-comparison-rhs：`(.limit > 3) == p.dirty` | bool与str，E0277、exit101 |
| prop-bool-comparison：`note.dirty == (.limit > 3)` | `self.note["dirty"].as_str()...`与bool，E0277、exit101 |
| not-bool-comparison：`p.dirty == (!(.limit > 3))` | 内部Unknown沿!递归丢失，外层str与bool，E0277、exit101 |
| bool-comparison-logical：`p.dirty == ((.limit > 3) && .allowed)`，allowed:bool | 逻辑组合左子树Unknown，外层str与bool，E0277、exit101 |
| known-bool-comparison 正控：相同第一式，列表元素形状已知dirty:bool | `as_bool().unwrap_or(false) == (self.limit > 3)`，exit0 |
| known-bool-comparison-logical 正控：相同逻辑式，形状已知 | bool投影与bool逻辑结果，exit0 |
| logical-nested-control：`p.dirty == (!(!(.allowed && .other)))` | 声明bool/逻辑/双取反全保型，exit0 |
| prop-logical-rhs-control：`(!(.allowed || .other)) != note.dirty` | prop与声明bool逻辑结果，exit0 |

形状正控使用同一个真实Parser→AURA→RustGenerator→Rust编译链，证明比较结果参与外层比较与逻辑组合原本就是合法支持的语法，不是未知函数返回值或新的表达式规模要求。Value prop 的 `Note=serde_json::Value` 外部别名沿用原探针，不把缺类型的探针错误算产品缺陷。

修复边界：比较节点本身的结果为bool，应与取反、逻辑节点在同一既有表达式类型传播规则中递归处理，保持运算符/分组/调用边界、shape、声明宽度及纯条件truthy。应按既有AST节点/类型环境核对表达式闭包；不要只替换本表字符串、把Unknown一律当bool，或把合法bool子表达式削减为默认str。六比较符现已通过，后续保持其回归正控。

## 本轮结果与复现

106个输入均成功解析/生成；95feature+默认正控共 **96组实际编译：91 pass /5 fail**。新增33个代表组合中28过/5败；原63组全过。overlay八组合另计运行检查，mismatches0/exit0。结果逐项见 `results.json`，原始日志指纹和保留错误块见 `input-hashes.json`。

初跑新增夹具与历史bool-comparison重名，造成探针污染；已停止该次runner、改为独立名称bool-state-comparison并从头重放。**该次结果全部废弃**，不用于finding/计数。最终work的73个历史.at输入全部逐字节保留，历史默认正控本轮通过；setup内已加重名断言。此更正没有写实施代码或覆盖前轮冻结快照。

在原worktree重现，先将本目录复制至新work证据目录，勿覆盖冻结件：

```text
cargo build --offline --locked --manifest-path D:/autostack/auto-lang/docs/plans/attachments/814/review-r14/Cargo.toml --target-dir target --bin r10-generate --bin r10-overlay
python D:/autostack/auto-lang/docs/plans/attachments/814/review-r14/replay.py
```

replay逐项保存真实退出码；runner正常结束不代表全部编译通过。四包cargo check、手写改动rustfmt、diff whitespace通过；改动段无新增告警（compiler36/runtime235保持）。当前777 strict exit0、violations0/grandfathered3。未运行整档t/tv/tt/tb/tu/taa。

## 完整 T-10 / SD 对账

| AC / task | 当前结论与证据边界 |
|---|---|
| AC01 / T02,T06 | **pass，限定复用** R9无UI物理源码SDK fresh+locked5/5、四非UI consumer及BUILD_INPUTS证明；R13后工作树SDK/依赖/锁/配置及输入未变。 |
| AC02 / T05 | 原API/typepin本轮默认正控再次编译通过；r3 inventory例外、单源Parser/VM、六feature/provider证据按未变实现复用。T05因所承载生成路径仍开放，不重开已发布A事项。 |
| AC03 / T04,T05,T08 | **fail，R9-01上述子表达式类型缺口**。T04 runtime保持；本轮实际overlay八组合通过；R9 counter Vue/VM同源三状态、R11 runtime65按未变SDK/runtime/脚本/示例/配置复用。其它三生成器与import策略/模板未改，新Rust条件路径已实际编译核验。 |
| AC04 / T06 | **pass，限定复用** provider/frame/i18n/theme/mem_guard44、六provider43及R11 runtime65；宿主/寿命/状态实现未变，本轮strict没有逆向实现边。 |
| AC05 / T07,T08,T09 | 选择/发行所验**pass**，SD02 **partial**。R6原→A九档strict逐ID、R8 A→B实际挂载、R9九档逐ID/R10 t与tu枚举（5328/868 unique），按未变测试声明/配置复用；不认证通用unique-leaf回退。R9冻结792真实driver24/24与runtime独立consumer5/5，按未变工作树producer/模板/manifest/锁/输入复用；不以旧e2e掩盖当前反例。777本轮strict绿。 |
| AC06 / T10 | **needs_fix**；完整代码/验收/规范审查已做，R9-01未闭，不能授pass。 |
| AC07 / T11,T12 | **merge pending**；本轮未实施修复/合入/实际B发布/沉淀canonical或liveledger/归档/清树。 |
| AC08..10 / T13..15 | A阶段R7/r3审定与T15实际发布边界保留，不因B缺陷回滚。 |

十一canonical输入均与R13一致。SD01/03/04及A/r3保留规则、SD05/06/07事实注记与未变实现相符；SD02六符/取反/逻辑的已测边界成立，但“完整表达式类型”仍遗漏bool比较子树，需随同一修复同步。SD08已在主计划修复记录承认按当前`de5e9d4a395f956e7847bcfb5e731d55f35acfde`发行Spec增补并保留M815-A/M816-A，R13并发对账继续有效；不使用旧3e1版覆盖当前全文。十一现有Spec+新增runtime与GOAL003/007影响元数据正确；本轮无canonical写入。

outcome: **needs_fix**；status: **executing**；T04保持闭合，重开T05/T08/T09，T10..12未完成；实际完成T01/02/03/04/06/07/13/14/15，**9/15**。next_responsible_role: **work**，同原树完成既有比较子表达式bool类型传播及同节点组合检查，定向验证、提交后再审。

最终pass后merge在新冻结提交统一 **裸t+tv+tu+tt** 与必要docs/schema/book，主库并集变化由T11复审，再实际B发布/Spec沉淀及收据；tf仍主检出到期单实例，aavm原用户豁免保持。
