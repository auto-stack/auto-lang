# PLAN-814 T-14 / R7：按 r3 合同复审，M814-A phase pass

2026-10-20 / stage: review / review_scope: **phase / PLAN-814:r3:M814-A / T-14** / outcome: **pass**。
reviewer_identity: 本聊天 Codex 复审上下文，独立于本批实现；未创建 agent 或额外 checkout。本次只给 A 阶段验收结论，整 Plan 保持 executing，B 及实际 A 发布未完成。

## 冻结与授权

- plan_id: PLAN-814；plan_revision: **3**；reviewed_commit: `256613936b6a5e9baea450e5c545899ee7a661dc`；tree: `98252f78e85c5a2339cdfc4e584210f8ee068099`；base_commit: `203e2ce91302ff1ebd48cbd0a8365d76abb89009`。
- worktree_path: `D:/autostack/.wt/lang-814/auto-lang`；branch: `codex/plan-814-dev`；dependency_revisions: 组内 auto-down `c39ac7f9dacdc769fc4a5ad3b98ec896240b28e8`；初末 HEAD/tree、依赖与 clean 状态见 [baseline-evidence.txt](baseline-evidence.txt)。
- 主检出起始 HEAD `b65d0e963bbf39f97de5c1d6531997f6f21c760b`。用户本轮明确要求“T-14 按 r3 合同重审”、绑定上述提交、主计划、候选/SD、r6-closeout，并指定已闭项的复用及 merge-owned 门禁。该请求与主计划 §5.3/§9续7/§10 一起构成当前审核的授权范围，不再沿用 R6 的“r2、待用户批准”判定。
- 主计划已为 r3，§5.3 完整落文两个窄增量，§5.2/V-02/AC-02 与 SD 同步；未扩大不同版本共装支持，未砍旧 API，未把完整 runtime 提前列为 A 能力。AC-09/T-14 任务行遗留的 r2 绑定标签在本次簿记改为 r3，属于引用纠正，不改变验收含义或再次升 revision。
- [受审主计划](plan-before-review.md)、[原候选](candidate-frozen.md)、[当前候选及历史勘误](candidate-reviewed.md)、[历史装配论证](assembly-background-frozen.md)均冻结。候选 §8 明确当前 r3/256613936，早批 SHA、已删除 feature/旧告警延期叙述保留为历史，不作为当前发布事实。
- spec_inputs: [input-hashes.txt](input-hashes.txt)。当前详细设计/§5.3/SD 及候选一起冻结于 [sd-frozen.md](sd-frozen.md)，SHA256 `87e766f7b29d892099bd6a38f90f2c2b4d86b56062c5c80b1dccbb6964049691`。未改 canonical Specs 或 live ledger。

## R4-01 在 r3 下的闭合

**F814-R4-01 CLOSED，限定本次 r3，不抹去 r2 下的历史不通过。**

| r3 约束 | 实际实现 / 复验 | 结论 |
|---|---|---|
| 允许上层登记轻表/函数指针，启动登记不调用工厂或 UI 业务/I/O | 三项 submit 分别传 `ui_grammar` 函数指针、`plan814_ui_compat_install` 函数指针，没有调用表达式；SDK 只 collect。inventory 锁定版本及注册机制与 R5/R6 已核对者相同。[来源定位](assembly-source-locations.txt)；原 host 探针先断言新进程无 frame provider，再调用旧入口。 | pass |
| 具名入口显式优先，未装配才用登记来源 | 上层安装器首句 set_ui_grammar，compiler 三入口经 assemble_ui_grammar；SDK ui_grammar 先读 OnceLock，缺席才 iter；宿主 hook 已设置时直接调用 hook 并 return。[原 host](host-original.log.txt) 与 [显式覆盖/VM 快照/autodown](host-autodown.log.txt)。 | pass |
| 旧 Parser 直达 UI 默认行为保持 | 只经旧 Parser::from(...).with_session(UI) 的新进程解析 widget，并断言 WidgetDecl；[Parser-only](parser-only.log.txt)。 | pass |
| 多当前来源同一文法实现，不依赖顺序选择语义 | 上层和 compiler 都登记 `auto_ui_dialect::ui_grammar`；该单一构造器只组装本 crate 真实文法函数指针。没有新增多版本并存承诺。 | pass |
| 纯 SDK 无 UI 实现/提交，缺席拒绝保持 | 实际复制完整 normal/build/dev 本地源码闭包，UI/旧包/Man 目录不存在；独立消费者生成新锁后 locked 执行5测试，包括 UI 门面明确诊断拒绝、Core parse/VM/trans；[完整无 UI 源码结果](no-ui-source.log.txt)。 | pass |
| UiParseContext/UiGrammar 归轻端口，文法实现仍单源，旧路径保身份 | parser-api 只有独立 opt-in 子 trait 与函数表；dialect::grammar re-export 同一 trait，crate 根再导出原路径；未把 trait 方法并入普通 ParseContext，也未搬文法实现进 SDK。实际纯 SDK/上层/compiler 消费均编译通过。 | pass |

源码中的“r2 约束下待批准”注记是该冻结提交在批准前的历史状态，本次由已批准 r3 主计划及收据明确覆盖，不要求为改历史注记另造实现 SHA。历史论证附件中的旧不可兼得/未批准段落同理，仅为背景；当前规则以主计划 r3 §5.3 为准。

## 独立重验及 r6-closeout 对账

实施者 [r6-closeout/probes-r3.log](../r6-closeout/probes-r3.log) 与 [pure-sdk-r3.log](../r6-closeout/pure-sdk-r3.log) 为摘要收据，已核对其命名范围；本次不只依摘要的 PASS/0 tests 片段授通过，另执行：

1. `cargo build --offline --locked --manifest-path .../review-r5/probes/Cargo.toml --target-dir <原树>/target --features auto-lang/autodown --bins`：[完整构建输出](probe-build.log.txt)，退出0。[二进制hash](probe-binary-hashes.txt)。
2. 新进程 `plan814-review-r3-probes host`：启动时尚无默认 frame；旧 create_vm/run 均装配并输出42：[输出](host-original.log.txt)，退出0。
3. 新进程 `parser_only`：[输出](parser-only.log.txt)，退出0。原公开路径、同一 Parser 状态与真实 widget AST 保持。
4. 新进程 `host_autodown_edges`：显式 frame 123/456 与 VM 快照、真实 autodown parse/serialize/find/text/insert/template/错误分支：[输出](host-autodown.log.txt)，退出0。
5. 原 [no-ui-source.py](../review-r5/no-ui-source.py) 在冻结原树运行：独立无 UI 源码、新消费者锁、locked 消费 **5/5**，不是只 metadata 或 doc-tests 0项：[输出](no-ui-source.log.txt)。
6. 独立 `ui-package-consumer` 的 `cargo test --offline --locked --manifest-path ... --target-dir <原树>/target` **4/4**：[输出](ui-consumer.log.txt)。覆盖真实 Vue 生成出口以及公开 style/lucide/fingerprint 面；不将这4项说成四后端完整运行验收。

当前重验与实施者摘要一致。compiler 三个入口的显式时序还经源码核对；没有为通过测试注入第二份 Parser/VM 或修改生产实现。

## 复用证据、验收矩阵及边界

受审实现、tree、依赖、Cargo 锁/清单、别名与 nextest 配置与 R6 完全相同。R6 冻结输入34件全部 hash 未变：[复用输入核验](reused-inputs-check.txt)；六份 Cargo/测试配置相同：[配置核验](reused-config-check.txt)。复用不依赖总数口头自述，也不把旧结论跨新实现授予。

| AC / Case / 任务 | 本次结论与证据 | 复用边界 |
|---|---|---|
| AC-01/V-01、AC-08/V-06；T-01/02/13 | A 纯 SDK、Core 执行、非UI trans 与公开 VM 面通过；本次无UI源5/5，R5闭合 build 输入禁止项0。 | 原 non-UI consumer/语言单源及 A owner 图未变；不授 B 所有 runtime 最终能力。 |
| AC-02/V-02；T-02/13 | r3 的显式/登记/缺席及端口归属通过，R4-01全部关闭。 | 既有公开字段/trait/callback pin 和外部方言行为沿未改输入复用，不新增兼容退役范围。 |
| AC-03/V-03 A子集；T-03 | 真实 compiler 公开算法/生成路径及 A 当前真实 runtime 归属保持，外部消费者4/4。 | 四生成器迁移代码与既有映射不变；完整新 runtime/renderer/真实工程跨端最终矩阵仍属于 B，tu 整档归 merge。 |
| AC-04/V-04 A子集 | R5闭合 host/autodown、SDK轻端口保持；本次额外重验宿主覆盖/快照及 autodown。 | 不把 B 生命周期/renderer迁移提前通过。 |
| AC-05/V-05 A子集；T-03/13 | R6 strict proof 九档逐旧ID显式核销、删除/同名替换负控及18件持久清单继续有效。[R6报告](../review-r6/review.md)、[严格输出](../review-r6/strict-selection-proof.log.txt)。 | 仅复用独立 strict proof，未认证工作脚本的唯一叶名泛化；tu869描述符/867唯一ID历史重复保持。 |
| R3-02/03（R5闭合） | 缓存升级/晚装配失效、SDK named build 输入与无UI清单执行保持关闭。 | 函数体/依赖/构建配置未变；本次无UI消费另重验，不重跑已闭缓存全部反例。 |
| R3-06（R6闭合） | 默认五包 check 的迁移新增0/减少7、六文件fmt、Drop两单测与跨crate四控制沿原证据复用。 | 不宣称全仓零warning，不把本次 autodown 特性构建 warning 数代替同配置差集。 |
| 777/792 A交付准备 | 777域基线/资源和受管投影结构在此冻结SHA未变，沿原 work验收/历次审查记录保留；25/25为已识别 work证据。 | 本轮未声称独立重跑完整792，更不把准备收据当实际master A发布。冻结后实际发布/来源/consumer读回归T-15。 |
| AC-08、AC-09 / T-14 | **M814-A phase pass；T-14完成。** | 仅 r3+256613936+冻结候选/SD；不代表整Plan AC-01..07全部完成。 |
| AC-10 / T-15 | **pending / merge-owned** | 回归门和实际 A 合入/版本来源/知识/consumer读回收据未执行，不记通过。 |

R3-01..06 与 R4-01 当前全部 CLOSED，无新阻断/未批准延期或缩面；R6 needs_replan 作为 r2 历史保留，本次不是倒改其结论。

## 审定的规范增量

现行9份 Spec 的源 hash 见 input-hashes；两个旧“不入 Core 包”约束由已批准 r3 的精确窄增量解决，其 canonical 写回仍由 T-15 执行。

| A增量 / 目标 | 审定规则与限制 |
|---|---|
| SD-01 add `docs/specs/language-sdk/project.md` | 真实 SDK owner/API/feature、无UI实现normal/build/dev与无UI源码消费；公开函数表装配及SDK零UI提交；R3-02指纹None→Some、首个Some优先、按对象出生身份保存。load仅在身份不同或仅一侧缺席时失效，None/None不因身份比较本身失效，不能将历史“一侧缺席”摘要误写成两侧均缺席也拒绝。 |
| SD-02 add `docs/specs/auto-ui-compiler/project.md` | compiler实际生成/分析/资源owner、三入口显式装配、同一dialect工厂、foreign build指纹None；不反依赖上层运行包。 |
| SD-04/05 modify `docs/specs/auto-lang/project.md` / `frontend/overview.md` | 上层兼容与默认组合、同一Parser/VM/旧路径；r3只登记轻表的窄例外、显式优先/直达兜底/纯SDK缺席诊断。 |
| r3 modify `docs/specs/auto-parser-api/project.md` / `docs/specs/auto-ui-dialect/design/full-grammar.md`（并同步project导航） | 前者允许独立UI opt-in轻端口定义但无文法实现依赖，普通ParseContext不强加UI能力；后者定义归属改parser-api、真实文法仍dialect、旧路径再导出保持类型身份。 |
| SD-06/07 A子集 modify `docs/specs/auto-lang/vm/overview.md` / `ui/overview.md` / `runtime/overview.md`相关事实 | SDK轻宿主端口、上层仍承载真实runtime、compiler已迁owner与缓存身份时序；不写成新runtime迁移完成。 |
| SD-08 modify `docs/specs/repository-boundaries/generated-ui-distribution.md` | A实际新包图/受管源码资源与测试选择、fresh/locked消费；阶段发布收据须绑定真实master祖先与来源。 |
| SD-03 / `docs/specs/auto-ui-runtime/project.md` | **B pending**；本次不得因frontmatter列了该目标就发布完整runtime事实。 |

frontmatter 的 supersedes_spec_components/new_spec_components 为全 Plan目标，路径准确；touched_goals=GOAL-003/007保持。merge仅按A已验事实沉淀，不将全Plan目标一次性宣称完成。附带的历史论证不覆盖r3正式正文，当前候选 §8 已明确其优先级。

## 交给 T-15 的边界

next_responsible_role: **merge / T-15**。Plan 保持 **executing5/15**，T-01/02/03/13/14完成；T-15与B任务未勾，不改成reviewed或归档。

pending_closeout_gates：

- 在冻结提交上统一执行 **裸 cargo t + tv + tu + tt**，并按真实docs/schema/book触面补必要门。它们本轮均未执行。
- 本阶段 diff 触及 `tests/aavm2_m4.rs` 与 lib.rs 的 aavm消费常量/辅助，不能以R6 taa编译/list当运行兜底；merge须按仓库AAVM触发/作用域及fold前兜底规则安排适用 **taa**，遵守机器单实例/资源纪律。
- tf 只在主检出按到期条件单实例批量回归；不是本阶段复审门。没有要求t3里程碑档。
- 任一门失败须归因；代码变化后在新冻结SHA复审受影响项，再跑受影响门，不能由这份pass覆盖后续实现。
- 冻结门通过后才实际A合入、受管Spec/ledger/index、版本/来源/consumer读回发布收据；A保留原树继续B，不归档或清树。

本轮仅定向探针/消费者及证据对账，未改实施代码，未执行整档/合入/实际发布/规范沉淀/归档/清树。**阶段 pass 表示代码、验收与规范子集审查通过；A 尚未交付。**
