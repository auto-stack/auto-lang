# PLAN-814 R4：六项修复重点复验，needs_fix

2026-10-20。stage: review；review_scope: **phase / PLAN-814:r2:M814-A，重点R3-01..06**。reviewer_identity: 本聊天Codex，独立于实施上下文，未新增agent/checkout。B未迁runtime本身不作本轮缺陷。

## 冻结基线

- plan_id: PLAN-814；plan_revision: 2；reviewed_commit: `6418f99efe0af2432a652212e98d39b973515cd9`；base_commit: `203e2ce91302ff1ebd48cbd0a8365d76abb89009`；修复比较基线R3 `abe9c99002fe7bafb49a9f86c5867f68d796d65a`。
- worktree_path: `D:/autostack/.wt/lang-814/auto-lang`；branch: `codex/plan-814-dev`；dependency_revisions: 组内auto-down `c39ac7f9dacdc769fc4a5ad3b98ec896240b28e8`；主检出初始HEAD `cb2194c2461a203712bd601c7d314ab35544fec3`。
- 初查实施树clean；末查源码/HEAD/tree/依赖/洁净与hash见 [baseline-evidence.txt](baseline-evidence.txt)。原共享计划 [plan-before-review.md](plan-before-review.md)、[候选冻结](candidate-frozen.md)、[映射表冻结](work-mapping-frozen.md)、[告警归因冻结](work-warning-frozen.md)。
- spec_inputs见 [input-hashes.txt](input-hashes.txt)。计划SD表+候选全文冻结 [sd-frozen.md](sd-frozen.md)，SHA256 `031db8ec3b2d418ae906533af261231c98c1e15debfe30564c12c9de0d08c242`。候选头仍写abe9c9900，本轮根据主计划续4和候选§6明确绑定6418f99ef，不误用旧SHA授pass。

## 六项闭合表

| 原Finding | R4结论 | 已关闭部分及仍需处理的范围 |
|---|---|---|
| F814-R3-01 / P1 | **原行为问题 CLOSED** | 原host探针退出0，create_vm与run包装均装配frame；显式默认123/456及VM快照正控通过；只使用旧Parser路径的新进程可解析widget。原“漏装配”不复活。新增inventory隐式装配的合同偏移单列R4-01，不能以行为绿代替合同通过。 |
| F814-R3-02 / P1 | **PARTIAL / OPEN** | 原“空cache在生成后保存null”探针已过，auto-man Vue/Jet也已在load前注入。但None占位阻断后续Some、晚注入后保存给旧条目改盖新指纹的两项相邻反例失败，详下。 |
| F814-R3-03 / 原P1 | **源码清单与独立执行子项 CLOSED；build输入部分仍OPEN（余项P2）** | 非UI源码快照独立新锁+locked消费者5/5，旧auto-macros dev/optional dialect缺清单原例全部关闭。SDK build.rs仍扫描UI/Man源码并纳入实际BUILD_INPUTS，不满足原finding已列出的build输入owner闭包。编译阻断已消除，余项按构建边界P2记录，仍属合同内必修。 |
| F814-R3-04 / P1 | **CLOSED** | 旧autodown配置实际build成功，原native探针现在返回真实JSON；parse/serialize/text/find/insert/template/错误分支相邻正反控全部通过。 |
| F814-R3-05 / P2 | **PARTIAL / OPEN** | tt扩包修复及86项迁移映射真实有效，原两个代表性测试分别定向1/1。但tb及原ta/taa/t3仍只选旧包，映射表将这些原来包含迁出inline测试的档说成“否/非本族档”，未完整修复原档选择面。 |
| F814-R3-06 / P3 | **PARTIAL / OPEN** | 原两个lib门面格式检查退出0，当前正常构建的unexpected_cfgs为0；修复新增ui_cache测试和macro测试导入块仍未格式化，修复触面检查退出1。不能据“本批触面全部fmt过”的归因表记为全闭。 |

旧问题已关闭的子项明确保留，不因修复引入相邻缺口把原反例重新称为失败。未以总数或work的“全PASS”转述代替当前命令结果。

## 未闭范围及可回放证据

### R3-02：原空缓存例已修，身份转换仍会误用缓存

位置：`crates/auto-lang-core/src/database/ui_cache.rs:17..22`、`save`的当前指纹重盖分支。

work续3称改为RwLock升级语义，实际6418f99ef仍是`OnceLock<Option<String>>`且setter忽略重复set结果。独立新进程：

```text
set_generator_fingerprint(None)
set_generator_fingerprint(Some(actual_compiler_fp))
expected=42114e3b7e7cc6db actual=None
exit=101
```

[None→Some原始结果](cache-none-upgrade.log.txt)。这是现有Some/None装配合同及work声称的升级路径，不要求新规模或新后端。

更关键的旧调用顺序含已有条目时：[晚注入反例](cache-late-save.log.txt)：先在未知指纹下保存/加载`unchanged.at`，随后装配F2、保存旧cache、再load，实际输出：

```text
late_identity=Some("F2") retained=1 dirty=false
exit=101: None-to-Some identity transition must invalidate old artifacts,
not restamp them as fresh
```

save仅把身份字段改成F2，没有让旧条目失效，因此给旧缓存补上了新的身份。与现行runtime Spec PLAN-698“一侧缺席也整缓存失效”相违。正控：[不同进程F1→F1](cache-same.log.txt)保留1条，[F1→F2](cache-upgraded.log.txt)失效0条均退出0，说明失败边界是晚装配/未知身份转换，不是全部load失效逻辑都坏。

修复：None不得阻断真实身份装配；已加载/已保存条目遇到身份转换必须失效或可验证地重建，不能仅重签旧内容。重放原空cache例、None→Some、旧条目+晚注入和跨进程版本正反控；同时保持Vue/Jet在首次dirty判断之前获得真实身份。

### R3-03：真实无UI源码消费通过，但build输入仍含禁止owner

[独立source探针](no-ui-source.py)复制真实非UI依赖源码及标准库/必要身份锁输入，根目录不含auto-lang、auto-ui-dialect/compiler/runtime、auto-man；独立consumer从无锁开始offline解析新锁，再`cargo test --locked`执行。

[原始结果](no-ui-source.log.txt)：`forbidden_source_present=False`，新锁成功，SDK消费者**5/5**，`NO_UI_SOURCE_FRESH_LOCKED_PASS`。不是沿原完整检出path构建，也不是只metadata。第三方编译缓存复用不替代源码：被测本地crate实际从快照路径构建，根锁副本仅为源码中声明的编译期identity输入，consumer自己的锁为新生成。

但原source/build归属检查尚未解决。SDK `build.rs:41`的`collect(root/crates)`未改；[实际生成的SDK构建输入摘录](sdk-build-inputs.log.txt)：

```text
("crates/auto-lang/src/ui/iced/renderer.rs", ...),
("crates/auto-man/src/automan.rs", ...),
("crates/auto-ui-compiler/src/ui_gen/vue.rs", ...),
```

这些输入写入`assembly_build_inputs.rs`并经SDK stdlib_assembly/manifest.rs的BUILD_INPUTS参与身份。无UI目录时可运行，不能证明在完整检出时SDK没有UI/Man构建输入边；本轮把两种性质分开记录。原R3-03已明确列该问题，未新增要求。

修复：收窄SDK真正拥有的构建输入/依赖身份来源，补无UI消费及UI/Man无关改动不进入SDK输入集合的负控；需要哪些外部轻能力/资源须具名，不回退到全crates扫描。

### R3-05：86项映射正确，原档位覆盖声明不正确

[实际nextest list](test-selection-list.json.txt)、[分析程序](test-selection-analysis.py)、[分析输出](test-selection-analysis.log.txt)：86/86真实名称与映射表逐项完全相等，无漏/多；SDK Rust转译和UI方言代表性测试各定向1/1。

但`.cargo/config.toml`的tb/taa/ta/t3仍只有`-p auto-lang --lib`。迁前master203e2ce91中`test_transpile_fn_single_function`是auto-lang内无额外feature门的inline测试，tb默认筛选并不排除它；迁后它位于SDK，运行旧包不会测试依赖crate的unit tests。86项parser/UI测试同理：迁前这些档位启用旧包默认UI，不能把它们统称为“非本族档”。

因此tt修复成立，但tb原例仍未修，ta/taa/t3的实际旧覆盖也应核对；tv的语料白名单、tu的ui_gen滤串则需按其真实筛选边界判断，不要求机械扩所有包。

修复：映射基面应追溯814迁包前203e2ce91，而不只取中途d905e796c。给原名→新owner→各别名/过滤/feature选择表，并修正实际遗漏。review使用list与定向断言，不提前运行整档。

### R3-06：门面与cfg已修，新增修复片段fmt仍失败

[原两门面检查](fmt-original.log.txt)为空、退出0；构建日志中`warning: unexpected`计数0。仍须关闭的具体新增片段：

- `auto-lang-core/src/database/ui_cache.rs:371`本次新增的std::fs::write调用格式；
- `auto-macros/tests/debug_expand.rs:3`、proc_macro_tests/value_macro_tests本次新增`use auto_lang_core as auto_lang`导入顺序。

[修复触面rustfmt原始结果](fmt-repair-scope.log.txt)退出1。只要求整理新增/改动片段，未把其他未改的历史样式差异变成新范围。work告警表以d905e796c中途基线比较296条，不足以证明814全迁移相对原master零新增告警；本轮仅确认原unexpected_cfgs类已清，未泛称所有229个当前告警都由本Plan新增，也未要求清理所有历史告警。

## 新发现 F814-R4-01 / P2：修复采用隐式链接装配，未与原合同及Spec delta对账

分类：修复设计合同偏移/规范增量缺口；任务T-02/T-03/T-13，AC-02/08/09。

源码：SDK lib.rs:1313附近`UiCompatInstaller`收集/遍历与`UiGrammarSource`首次来源装配；auto-lang lib.rs:311/318与compiler lib.rs:31的inventory submit。行为实测是成功的（原host与Parser-only正控均通过），**本finding不声称当前Windows样例发生随机链接失败**。问题是交付合同r2 §5.2/V-02及AC-02仍明确要求“不靠inventory偶然性”“无隐式链接装配”；当前设计说明以“上层被链接即其submit生效”作为直达入口的装配来源，没有获准改变该要求的修订。显式set_ui_grammar接口存在，不代表旧路径实际采用了显式组合。

此外UiParseContext/UiGrammar迁入auto-parser-api，与canonical `auto-parser-api/project.md`和`auto-ui-dialect/design/full-grammar.md`“UI伴随端口不入Core包”的现状规则相异。对象安全opt-in和旧路径re-export值得保留，但“孤儿规则只有这一条合法路”的work说明不能代替设计/规范对账。本轮复审已补frontmatter的auto-parser-api、runtime缓存及full-grammar导航目标；实际规范delta正文尚缺，候选主表仍写已删除ui-dialect feature、autodown延期，末尾增量未把A实际API/Spec patch完整冻结。

下一步：落实符合既有合同的明确装配来源并保持旧API/类型/状态；若确需保留此隐式机制或改变既有边界，按计划的needs_replan流程提出具体合同变更及证据，不能在本次review中默降AC。为实际trait归属/缓存时序补精确A子集SD正文、目标版本/路径与理由。此处未修改canonical Specs、live ledger或语义合同。

## 证据适用范围与路由

本轮原三探针 [host](host-original.log.txt)/[cache](cache-original.log.txt)/[autodown](autodown-original.log.txt)全通过；[相邻宿主默认/VM快照与autodown操作](host-autodown-edges.log.txt)全通过；[仅Parser直达](parser-only.log.txt)通过。探针源码/依赖冻结在[probes](probes/)及[probe-Cargo.lock.txt](probe-Cargo.lock.txt)，分别新进程执行，未修改产品实现。

已保存work的[792收据](work-792-receipt.json.txt)25项全绿，来源为组目录scratch-consumer，主计划所写实现树内路径不正确。收据未含冻结source/producer SHA集合，本轮仅作为work回执，不把它泛化为当前全部迁移/修复的独立pass；没有重复冷构建/完整792矩阵。其余未受本轮影响的历史证据按原边界保留，不把先前pending改成通过。

acceptance_results：AC-08 **fail**（R3-02/03build/05/06及R4-01）；AC-09未通过；AC-10 merge-owned pending。AC-01的独立SDK源码/编译执行子面、AC-02的原host行为、AC-04的autodown原行为分别通过所列定向证据，不代表全Plan原AC-01..05/B整体通过。

outcome: **needs_fix**。原R3-01/04行为findings CLOSED；R3-02/03/05/06 OPEN（具名子项已闭）；R4-01 OPEN。保持executing，重开T-02/T-03/T-13、current_step=1/15，T-01保留完成，历史证据不抹除；T-14不勾、T-15不放行。

next：同一原树work一次修完具名剩余边界与合同/Spec对账，回放本轮共享正反控和实际档位映射，再提交冻结A候选。pending_closeout_gates：阶段独立pass后merge owner统一冻结裸t及实际tv/tu/tt等触面门、实际master合入/Spec子集/受管ledger/API来源发布收据；A不归档不清树。B沿原任务继续。此次未跑整档、未修实施、未合入/发布或清理；tf仍主检出到期单实例。
