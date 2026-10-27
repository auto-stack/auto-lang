# PLAN-814 R15：完整 T-10 复审 pass，merge-owned 门禁 pending

2026-10-21；stage: review；review_scope: final；plan_revision: 3。
reviewer_identity：本聊天 Codex 独立复审上下文，未参与实施；未创建agent或新checkout。结论依据已提交代码、独立原例/相邻正反控、实际编译及明确边界的旧证据复用。

## 冻结身份

- 原唯一实施树 `D:/autostack/.wt/lang-814/auto-lang` / `codex/plan-814-dev`。
- reviewed_commit `6f1cac8cbe618e509bf85d5c000fa0c26f35df6e`；tree `f98330d91b665566876d36a330361f770503fd81`。
- 修复base=R14 `42a4ef792e522d0b30e81f42f6fb25e254148ac1`；B base=A实际发布 `f6e07aceeeea3cb12698c747d61db5be43f7458c`。
- auto-down `c39ac7f9dacdc769fc4a5ad3b98ec896240b28e8`；初末实施/依赖HEAD、tree、clean稳定。main HEAD、源码/配置/当前十一Spec、计划/SD/handoff与证据指纹见 `input-hashes.json`。
- 本修复只改RustGenerator41+/3-：补比较子表达式的bool结果识别。前轮44份源码/配置输入仅此文件变化；未改SDK、runtime、Cargo/锁、测试声明、档位选择、发行驱动或其它三个生成器。
- R14冻结附件无漂移；work的108个既有.at输入在本轮逐字节保持，再生只在review-r15。用户说明的未跟踪复审/历史证据按仓规保留，不作为实施WIP或额外阻断，不入库、不清树。

## F814-R9-01 CLOSED：原例、完整节点传播及修复回归核销

R14五个失败组合全部真实编译exit0：`p.dirty == (.limit > 3)`、反向、Value prop、取反包裹、比较结果再进入逻辑组合。新`has_top_comparison`在当前合法下形条件中将六种比较节点结果归为bool，和既有分组/取反/逻辑递归共同传播，不再把这些bool结果降成字符串。

- work原97feature+默认正控，共 **98组编译全部exit0**；overlay八组合另计运行mismatches0/exit0，原99个检查独立重放全部通过。
- R14的六比较符×两方向×loop/prop **24/24**再次通过，字段与另一操作数分组/声明int算式保持。
- 独立新增六种比较结果×两方向 **12/12**：比较的操作数含已声明int算式，结果嵌入&&或取反后再与未知字段比较；另Value prop双比较+逻辑或+取反、字符串内多个普通比较符两项正控通过。
- 既有known-shape bool/int/string、未知loop/Value prop、负数字面量/浮点、声明宽度f32/i64、双侧分组、算式方向、取反/逻辑/比较子树及纯条件truthy原例均保持。旧API/type身份pin随默认正控本轮再次编译通过。

**本Plan受验编译集合112/112通过**（原98+新增14），没有未闭的Plan内finding；R11新增告警项、R10浮层/777/serde、R8/R9其它项及R5/R6/R7已闭边界保持。

## 探索性字符串转义诊断：单独登记既有债务，不冒称绿

原始总矩阵实际为 **115次编译：112 pass /3 fail**；125个输入均成功解析/生成。三个失败为本轮额外的escaped quote诊断（`r15-string-escaped`、`-rhs`、`-known`），不是上述受验集合的通过项，原始退出码全部保留。

输入 `p.label == "a\"<b"` 在未知字段和已知字符串shape中都被AURA下形成非法Rust字面量 `"a"<b"`，均E0765（未闭合字符串）、exit101。因此已知shape正控也失败，不能据此把异常归为R9-01的值类型投影不保持，或把该探针当作已证明的合法条件字符串给新比较分类器。

已核实 `crates/auto-ui-compiler/src/aura/extract.rs` **从A实际发布f6e07acee到当前整文件逐字节未变**，git blob均为 `c0f7ab6f1e076308fecce29d143a9ec700025a90`；`:443` 的 `Expr::Str(s) => format!("\"{}\"", s.as_str())` 直接拼接已解码内容，丢失引号转义。A文件原件冻结为 `baseline-aura-extract.rs`。没有在旧提交重新执行完整二进制，不虚报A探针运行。

登记 **D814-R15-01（非阻断债务候选）**：AURA条件字符串再序列化的引号转义，以及未来修复此上游后，下游各字符串扫描器的转义规则需一并核查。分类理由不是“预存便可忽略”：本Plan合同是分包后保持已支持的生成行为，R9-01修复对象是可合法下形条件的比较类型保持；本组在迁移前已发布owner中就无法正确下形，且known-shape同样失败，属于另一条未改的字面量编码能力，不是借登记将本Plan未完成类型规则延期。没有新增语法限制或声明这些输入本来不合法，也没有承诺本Plan已修复所有既有生成器缺陷。若后续改变AURA编码/扫描器，需单独验证并使受影响review失效；当前pass不覆盖未来修复。

## 本轮检查与复用边界

四包cargo check、手写触面rustfmt、diff whitespace均通过；新改动段无新增告警，匹配compiler36/runtime235保持（预存告警按历轮归因保留）。实际777 strict exit0、violations0/grandfathered3，锚及窄豁免未变。实际overlay八组合mismatches0/exit0。没有运行整档t/tv/tt/tb/tu/taa，也不将work自测整档结果当merge冻结门收据。

| AC / tasks | 完整T-10结论及证据 |
|---|---|
| AC01 / T02,T06 | **pass**，限定复用R9物理无UI源码SDK fresh+locked5/5、四非UI consumer闭包及BUILD_INPUTS单源证明；工作树SDK/依赖/锁/配置/输入未变，本修复只改UI RustGenerator。 |
| AC02 / T05 | **pass**，旧API/typepin本轮实际编译；r3 inventory登记/启动只注册、显式优先、缺席臂及Parser/VM单源边界未动，沿用既闭R5/R6/R7证明与R9六feature/provider43证据。 |
| AC03 / T04,T05,T08 | **pass**，本轮112组有效编译、旧反例/控制、六符24矩阵及比较结果节点补验如上，R9-01关闭；当前overlay八组合真运行通过。R9 counter Vue/VM同源三状态与R11 runtime65按未变runtime/SDK/示例/双端脚本/配置限定复用；其它三生成器、canonical import策略/模板与旧e2e两真实工程构建证据未变，复用但不冒称本轮重跑。三项转义探索诊断按上述独立债务如实保留。 |
| AC04 / T06 | **pass**，provider/frame/i18n/theme/mem_guard44、六provider43及R11 runtime65按未变宿主/寿命/状态实现复用；当前strict再次确认零逆向实现边。 |
| AC05 / T07,T08,T09 | **pass**，R6原→A九档strict逐ID、R8 A→B真实挂载路径、R9九档逐ID/R10 t与tu枚举（5328/868 unique）按未变测试声明/别名/选择配置复用；不认证通用unique-leaf回退。R9冻结792真实driver24/24（fresh/locked重定位/负控）与runtime独立consumer5/5，按未变工作树producer/模板/manifest/锁/输入限定复用。777本轮strict绿；SD/来源/兼容边界审定如下。 |
| AC06 / T10 | **pass**，同树revision绑定代码、全部AC对账、遗漏/规范/健康检查完成；无Plan内未闭finding，merge门明确pending。 |
| AC07 / T11,T12 | **pending，merge owner**；不是本轮已完成的交付。 |
| AC08..10 / T13..15 | A阶段R7/r3审定与T15实际发布边界保持，不授未来B合入收据。 |

证据复用经过源/依赖/配置/输入比对，不因历史“全绿”标签自动继承。本轮已变的Rust条件路径重新实际编译；尚未执行的未来门禁不引用旧work结果冒充。

## 规范增量审定

冻结 `sd-frozen.md` 与handoff/主计划r3绑定上述提交。十一canonical输入与R14相同：SD01 SDK单源能力、SD02 compiler/canonical路径和本次已实现的类型传播、SD03新增runtime职责/feature/装配/状态/资源、SD04 compat依赖与旧API、SD05/06/07事实注记、SD08发行模板/投影增量，与当前已验实现及A/r3保留规则相符。上述受验表达式类型行为审定，不把三项独立AURA转义诊断写成已支持能力。

SD08旧before3e1与当前blob `de5e9d4a395f956e7847bcfb5e731d55f35acfde` 的并发对账沿用R13，work已在主计划明确：merge须以当前全文增补B规则，保留M815-A/M816-A章节，不用旧A全文覆盖。十一现有Spec目标、新增`docs/specs/auto-ui-runtime/project.md`及GOAL003/007影响元数据正确。未改canonical或liveledger；落库时执行日志/测试计数留来源收据，canonical以已审职责/语义/边界为正文，不追加新的能力承诺。

## 状态与下一步

outcome **pass**；status **reviewed**；T05/T08/T09保持完成，**T10完成**；完成T01..10、T13..15，共 **13/15**。T11/T12仍pending。next_responsible_role: **merge**。

merge须在主库并集/冲突处理及受影响复审后，冻结最终提交，执行 **裸cargo t + tv + tu + tt** 和必要docs/schema/book门；红门归因，改码后受影响复审并重新冻结。`tf`仅主检出到期判定单实例，aavm按原用户豁免。门禁通过后方可实际B合入、Spec/ledger受管沉淀、发布/consumer读回收据、归档及guard clean清理。**reviewed不等于B已交付**。

本轮未改实施、未合入、未发布Spec/ledger、未归档/清树。复现脚本为本目录`replay.py`，使用本目录manifest/lock及原树target；修复或后续调查先复制探针目录，保留本冻结件。原始115次编译的三项探索失败不得从收据中隐藏。
