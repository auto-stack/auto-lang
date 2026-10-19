---
plan_id: PLAN-810
status: execution_done
feature_name: 完整 UI 文法与默认编译入口解耦
author: [Codex]
created_at: 2026-10-19
updated_at: 2026-10-19
plan_revision: 1
current_step: 8
total_steps: 11
supersedes_spec_components: [docs/specs/auto-ui-dialect/project.md, docs/specs/auto-parser-api/project.md, docs/specs/auto-lang/frontend/overview.md, docs/specs/auto-lang/frontend/design/dialect-system.md, docs/specs/auto-lang/ui/design/scenario-dialect.md]
new_spec_components: [docs/specs/auto-ui-dialect/design/full-grammar.md]
touched_goals: [GOAL-003, GOAL-007]
affects: [auto-ui-dialect, auto-parser-api, auto-lang/frontend]
depends_on: [PLAN-799, PLAN-801, PLAN-806]
---

# [PLAN-810] 完整 UI 文法与默认编译入口解耦

## 0. 变更摘要

交付总路线R01：在806已交付msg/model基础上，将剩余完整UI结构/事件文法及递归调用闭包迁入`auto-ui-dialect`，主文件与被导入文件的默认UI解析全部消费新实现，Parser只保通用解析/推断能力及旧公开薄门面，UiDialect不再取回完整Parser。包含所有真实调用、语义副作用、独立消费、旧算法删除及发行输入闭包，不按parse函数逐个开小Plan。

本计划不迁UI AST/关键词所有权（R10），不搬renderer/UI生成器/Man工具或Git仓（R04/R08），不承诺本Plan后auto-lang基础闭包已完全不依赖UI文法（默认组合出口由R04处理）。

## 1. 目标

- widget/component/view/fragment、store、props/refs、watch/setup/expose/computed、on/actions/style等既有UI文法有单一轻实现，完整闭包按T-01真实调用图穷举，不以名称举例代替范围。
- Core expr/type/body/语义存储保持原所有者；UI语法不复制Pratt、类型推断、作用域、模块解析和语句算法。
- UI默认注册、Override→builtin→Append优先级、None回滚、Err恢复、递归重入、import/session传播及旧公开API保持；Core同名变量、参数`view`、task `on`等不被抢占。
- 编译/VM/现有UI生成路径可消费同一AST，源位置/诊断/词法前瞻/原始CSS与既有规则相符；不借迁移扩张文法或静默清理旧错误怪癖。

## 2. 架构方案

沿用`auto-ui-dialect → 轻解析合同/auto-ast/lex/diagnostics`，不新建一套Parser或UI AST镜像。上层`ParserBridge`实现具名最小能力；UI文法调用这些能力，而非获得Parser/lexer/TypeStore/infer_ctx引用。当前`ParseContext`不足以表达完整body/scope/raw-source等：T-01用有界完整路径probe冻结增量设计。

可采用UI伴随端口（建议在`auto-ui-dialect`定义`UiParseContext`，由ParserBridge实现）与薄上层默认adapter，避免向既有`ParseContext`强加全部必实现方法破坏外部consumer；需要通用Core能力时放在有明确Core含义的接口。最终布局T-01冻结，必须证明既有ExternalDialect/ParseContext实现可继续编译、对象安全及无实现反向边。不能用Any/downcast、opaque callback把完整Parser或整份UI算法藏回上层。

## 3. 技术栈

Rust现有轻解析crate、owned AST、场景/有作用域checkpoint、ParserBridge及CompilerSession。复用现有msg/model单一实现，不从源码字符串重解析整个文件，不新建另一套词法/推断存储。

## 4. 需求分析与背景调查

授权：用户确认按上一轮建议建立两份较大模块计划，可与808/809并行。本轮只规划auto-lang仓内实现，不执行、不创建agent/工作树、不搬仓/写远端。参见[总路线](reports/v06-extraction-execution-roadmap.md)、[本批并行合同](reports/grammar-media-parallel-order-810-811.md)、[来源快照](attachments/grammar-media-batch-20261019/inputs.json)。用户未指定实现/审查agent或预算。

权威输入：[frontend](../specs/auto-lang/frontend/overview.md)、[方言体系](../specs/auto-lang/frontend/design/dialect-system.md)、[UI场景](../specs/auto-lang/ui/design/scenario-dialect.md)、[auto-ui-dialect](../specs/auto-ui-dialect/project.md)、[auto-parser-api](../specs/auto-parser-api/project.md)。799/801/806已合入，806归档记录字段仍有7/8而尾部归档/清理收据为8/8；此处依据实际交付`c4158b89e`与canonical Spec，不代修其簿记，也不把进度字段当唯一证据。807文本模块已交付；与本计划无新硬前置。

实证锚点（均相对`crates/auto-lang/src`）：`parser.rs`的`parse_widget_decl/parse_widget_use_block_inner/parse_watch_block_inner/parse_store_decl/parse_computed_block_inner/parse_component_fn_decl/parse_view_* /parse_on_events/parse_on_block/parse_actions_* /parse_style_* /parse_setup_* /parse_expose_*`及互调辅助；`dialect/ui.rs`两处分支仍`bridge.parser()`；`parser_bridge.rs`持上下文；`Parser::build_dialects/with_session`负责默认组合。UI结构区还有对Lexer直接save/restore/push_token/capture_raw_block，以及skip_check、errors/warnings回滚、临时scope绑定的访问，不能机械用当前checkpoint替换。

808终端、809管理核心仍修复/审查中，不是本Plan硬前置，严禁消费其WIP。公共manifest/777/792/测试注册按本批合同串行合流。其他AC/前端路线若改同一语义存储/Parser公共能力，按精确写面交接，不要求全工程停工。

## 5. 详细设计

### 5.1 闭合范围与算法归属

| 迁入UI文法模块（包含必要私有helper） | 保持Core/上层，仅经最小具名能力消费 |
|---|---|
| widget/component壳、props/refs/external-use、model/msg组合、watch/setup/expose、scene/UI声明语法 | 通用expr/type/body/语句解析、符号查找/推断、ModuleTracker、真实文件import加载 |
| store的UI声明、computed/object-vs-body决策、actions属性/处理器、raw style及style recipe | 通用对象或块的核心解析、原始源码区段读取与游标操作、局部语义事务 |
| view/fragment/节点、for/if/key/memo/link/style binding、UI on消息/stream订阅与内联handler声明构造 | Core task-on/函数/表达式、临时上下文与scope/绑定操作；代码生成/执行留既有消费者 |

`parse_store_stmt`等同名Core表达式与`parse_store_decl`声明必须按真实调用区分，不能按函数名整块搬错。对Core与UI共享的on/body辅助，归通用语义所有者、UI调用端口；不得仅保原UI算法在Parser再把端口命名为`parse_widget`。生成handler/lambda/fragment参数绑定相关语义决策归文法，底层绑定与scope存储留Core，保持单份算法。

### 5.2 端口可行性与事务语义

T-01必须产出逐方法的字段访问→能力→实际实现映射及编译probe，至少打通一个含model/props、computed、raw CSS、watch/on、fragment scope与view节点的生产解析路径；识别下列差异后才开始后续批量迁移：

- **前瞻**：806的`probe_next_kind`成功零副作用、失败按旧Lexer消费且丢弃错误；不能再以`checkpoint+advance`制造注释/docs/诊断副作用。多token save/restore和回推重消费另有契约，不能假定单token探测覆盖全部。
- **checkpoint与局部投机**：801方言checkpoint的诊断单调追加不变；computed先试object、失败转body时旧代码显式truncate局部errors/warnings，这是另一种具名Core事务，不能扩大到所有restore或泄漏投机诊断。cursor/prev/token缓冲、skip_check、上下文旗标、scope与临时绑定按路径冻结。
- **scope/资源**：不返回TypeStore/InferenceContext引用，不复制绑定表。绑定离开语义范围后行为与旧臂相符；错误/重入/嵌套路径必须审计实际栈状态，若旧臂存在泄漏/挂死则在基线具名列出，不偷偷修成新文法。影响迁出可行性的错误行为冲突先做具体重规划，不以绿例绕过。
- **raw CSS**：使用具名raw-block能力取得原始字节和正确后继位置，保持嵌套括号、字符串、注释、换行/Unicode；不能Token.text拼接还原CSS。源码借用不逃逸解析上下文，不用unsafe延寿或要求`'static`源码。

T-01不另取Plan号；决策工件提交同一Plan。若只能靠暴露完整Parser/复制推断/改变默认文法达成，停止依赖实施并提出needs_replan，不退化为只迁易搬函数。

### 5.3 生产入口、兼容与恢复

`dialect/ui.rs`可保兼容类型与薄注册adapter，所有UI规则在新模块，不调用`ParserBridge::parser()`；若该私有escape hatch没有其他调用则删除，若发现另有非UI调用须逐项归属且不得留下UI逃逸。Parser现有公开parse_*方法保同签名薄委托，内部全部调用也真实改消费。

默认`Scenario::UI`与普通Core、显式外部方言注册传播均保原规则。主文件、跨文件use、导入再导入及独立session各使用同份文法，普通模块路径解析继续留Core。UI专属use块归UI；字符串级`scan_use_statements`不是UI文法，保留语言所有者，不借迁移重写全模块加载器。View/On TokenKind本期保兼容基础AST，R10再退出。

既有msg/model算法和ExternalSubset政策保持；新UI代码复用它们，不能复制第二份声明parser。诊断类别/文本/位置/顺序及恢复进度按旧臂逐面冻结；对已知无界EOF怪癖用隔离进程+外部截止证明基线并标明确切范围，不要求测试进程自身挂住，也不宣称错误恢复已经完善。

### 5.4 独立消费与发行

独立manifest可实现受限真实端口驱动新UI grammar，normal/build/dev闭包无auto-lang/VM/renderer/auto-man；另用真实Parser/default import路径验证上层语义，不把toy Context通过当生产等价。777精确登记合法边，792受管source/依赖声明与受影响fresh/locked同步维护；原闭包含auto-lang是组合现状，不能伪称SDK已纯Core。新增算法迁出行数与净删除分别披露。

### 规范增量

| delta_id | add/modify/retire | docs/specs/... target | before/after rule | rationale | acceptance IDs |
|---|---|---|---|---|---|
| SD-01 | modify | docs/specs/auto-ui-dialect/project.md | msg/model轻文法→完整UI文法、默认adapter与精确剩余边 | 单一领域实现 | AC-01, AC-03, AC-05 |
| SD-02 | add | docs/specs/auto-ui-dialect/design/full-grammar.md | 记录完整语法/调用图、Core能力、投机/raw/scope、支持及失败边界 | 可交接契约 | AC-01..04 |
| SD-03 | modify | docs/specs/auto-parser-api/project.md | 既有公开端口兼容；新增通用能力如有则记录其独立语义，不把UI端口塞入Core包 | 防逆依赖/错误事务混用 | AC-02 |
| SD-04 | modify | docs/specs/auto-lang/frontend/overview.md | UI算法从Parser退出，旧公开薄门面、默认/import装配事实 | 现状可审计 | AC-01, AC-03, AC-05 |
| SD-05 | modify | docs/specs/auto-lang/frontend/design/dialect-system.md | 无完整Parser逃逸的UI注册/派发、原checkpoint/优先级保持 | 解析入口解耦 | AC-02, AC-03 |
| SD-06 | modify | docs/specs/auto-lang/ui/design/scenario-dialect.md | UI领域完整文法owner与Core对照、同份AST消费 | 保护场景边界 | AC-03, AC-04 |

## 6. 测试设计

T-01冻结旧臂输出、错误/副作用及支持矩阵；T-07建新增`crates/auto-lang/src/tests/plan810_full_ui_grammar.rs`和`test/fixtures/modularity/ui-grammar-consumer/`。下列均owner_stage=work，当前只是未来验收合同，不宣称已经运行。

| Case | AC / invariant | 输入、真实入口 | 区分性断言及覆盖边界 |
|---|---|---|---|
| V-01 | AC-01 文法全集 | 真实Parser UI场景，widget/component/store/view/fragment、嵌套节点、props/refs/watch/setup/expose/computed/actions/style/on/stream/内联handler交叉样例 | AST结构/ToNode/ToAtom/源位置与冻结旧臂一致，逐调用图每个生产入口命中新实现；轻consumer无语言闭包，旧public构造/签名可编译 |
| V-02 | AC-02 事务/前瞻 | computed合法object与失败转body、注释分隔named payload、坏字符/无界EOF隔离反例、raw CSS括号/注释/多字节 | 区分成功/失败probe消费；局部诊断回滚仅在原位置生效，checkpoint仍诊断单调；raw字节/后继token正确；已有806永久反例重放 |
| V-03 | AC-02 scope/寿命 | fragment/component参数、model/for绑定；同名shadow、嵌套on与Core body、错误后续声明、跨ctx/过期checkpoint | 支持路径的binding/skip_check/flags/scope入出与旧oracle一致；非法checkpoint受控拒绝；源码drop后owned AST仍可用，端口不持悬空引用；合法深嵌套不被新限额拒绝 |
| V-04 | AC-03 默认派发 | 普通Core中同名变量、view参数/task-on；UI默认、Override/Append、NotClaimed后继续Core、两session及真实嵌套use-import | Core正控合法，UI AST/优先级保持；None游标恢复/Err方言不丢；导入文件使用同份新文法，不因新建Parser退回旧算法 |
| V-05 | AC-04 真消费者 | 既有代表UI examples、含外部use/组件/模型绑定等交叉样例，经Parser→实际VM及Vue/Rust/Jet/Ark现有生成器 | 保持已有支持矩阵的执行/产物结果，错误按原支持面；不通过改golden或跳过困难backend判pass。重型UI实机只选真实触面，使用现有autoui-verifier设施，不重复全画廊整档 |
| V-06 | AC-05 清源/发行 | 轻consumer fresh/locked、777 strict、792真实受影响交付profile及非法反向依赖/缺源负控 | 一份UI算法/AST身份，旧Parser无UI算法/escape；新包或源码未漏入交付，负控必红 |

work用`cargo check -p auto-ui-dialect -p auto-parser-api -p auto-lang`、轻包相关定向测试、`cargo test -p auto-lang --lib plan810`及新增consumer manifest；真实生成器/VM命令由T-01据当前入口冻结、T-07建立可重放选择，不伪造不存在flags。相关806/801反例及新增病例必须work完成。777用`python scripts/modularity-audit.py --mode strict --no-cargo-check`；792沿`python test/fixtures/modularity/generated-ui-consumer/driver.py`的现有受影响消费门，不新建发行器。

未指定实现/审查agent/model；不创建其他会话。独立final review使用同一树、独立上下文核AC/SD与反例，pass仅表示代码/合同审查。merge在冻结提交跑裸`cargo t`+`cargo tv`；因改变完整语法对生成产物的实际触面，最终统一跑`cargo tu`，触及trans源再加`cargo tt`。开发/送审/复审不触发整档；tf主检出到期单实例批量。复用必须证明源码、依赖、配置、输入适用性；缺环境为pending。当前文档规划不运行Cargo/docs_gen。

## 7. 验收标准

| ID | owner_stage | 可观察标准与证据 |
|---|---|---|
| AC-01 | work | 完整UI文法/必要helper单源迁入轻包，所有真实生产入口及旧公开门面消费它，闭合图无未授权遗留；V-01 |
| AC-02 | work | 无完整Parser/语义存储逃逸；通用能力与既有公开端口兼容，scope/事务/前瞻/raw/源码寿命按冻结合同；V-02/03 |
| AC-03 | work | UI默认/显式注册/Core对照/跨文件及嵌套import恢复全部保持；V-04 |
| AC-04 | work | VM和既有生成器真实代表消费者按现行支持面通过、错误对照与原始产物可审计；V-05 |
| AC-05 | work | 旧算法/逃逸退出、轻fresh/locked与777/792实测通过，资源/支持/未迁AST与组合剩余边及SD草案齐备；V-06 |
| AC-06 | review | 同树独立AC/SD/遗漏/已知怪癖审查绑定最终修订，格式/新增warning/debug问题清零；merge门pending明确 |
| AC-07 | merge | 最终冻结门、规范/ledger、归档/公共接口及资源交接、wt-guard clean后清理均有实际收据 |

## 8. 执行步骤

共11项，按“有界设计冻结→完整文法迁移→生产接线与单源清理→验证→审查合流”组织，每行唯一权威任务。

| 状态 | ID | owner_stage | 依赖 | 实际文件/动作、交付与验证 | AC |
|---|---|---|---|---|---|
| [x] | T-01 | work | 799/801/806实际交付 | [✅ 已完成] `attachments/810/contract-baseline.md`：迁移闭包（基面 parser.rs 13223–17909 + 3 自由函数）、逐方法触面→端口映射、事务/前瞻/raw/scope 合同冻结；probe 以全量迁移形态完成（§5 实测结论+勘误：expect_ident 留 Core、probe_next_token 上抛、advance_peek_kind 具名化） | AC-01..05 |
| [ ] | T-02 | work | T-01 | R1 重开：保留 UiParseContext/ParserBridge 实现；修复成功 raw 投机的快照生命周期及公开/直连入口清理（F810-R1-01），区分 scene/subresource 与 stream 的旧前瞻 prev 状态（F810-R1-04）；定向验证见 [复审报告](attachments/810/review-r1/REVIEW.md) | AC-02 |
| [x] | T-03 | work | T-02 | [✅ 已完成] widget/store/scene/props/use/setup/expose/watch/computed 及依赖 helper 迁 `grammar/{widget,store,scene,helpers}.rs`（约 4.9k 行，脚本 `scripts/plan810_migrate.py` 留档）；806 声明经 declarations 单源直调；Parser 留同签名薄委托 | AC-01,02 |
| [x] | T-04 | work | T-03 | [✅ 已完成] view/fragment/component/节点/for/if/key/memo/link/style 绑定/事件族/actions/style_recipe/timer/routes/menubar/toolbar 迁 `grammar/{view,actions}.rs`；on-events/arrow 事件文法留 Core（共享），Core store 表达式/parse_store_stmt 未搬错 | AC-01,02,04 |
| [x] | T-05 | work | T-04 | [✅ 已完成] dialect/ui.rs 重写全端口薄 adapter；`ParserBridge::parser()` escape hatch 删除（全仓零调用）；plan778 4 绿（None 回滚/Err 传播/matches 门）、Core 对照（fn view 参数/on 回退）plan810 v04 绿 | AC-03 |
| [ ] | T-06 | work | T-05 | R1 重开：旧算法/escape 删除成果保留；修复 F810-R1-01/04 并补永久反例；补真实 style None、Core/session/store 护栏及 scope/错误/凭据状态断言（F810-R1-03），完成触面 fmt/新增 warning 清理（F810-R1-06） | AC-01..04 |
| [ ] | T-07 | work | T-06 | R1 重开：本轮 plan810 13/13、轻 consumer locked 7/7、806 18/18、408 20/20、051 25/25；777 strict 实红 stale_baseline，修 F810-R1-02；按 F810-R1-03 补 V-01..06 可追溯矩阵及缺失 oracle，更新真实发行/负控收据 | AC-01..05 |
| [ ] | T-08 | work | T-07 | R1 重开：修 F810-R1-05 的 SD-01..06 冲突/错标，随代码修复重冻 delta/hash；按 F810-R1-02/03/06 更新无虚假通过的 handoff、卫生门及精确提交/证据；merge-owned 门仍 pending | AC-05 |
| [ ] | T-09 | review | T-08 | 同一树独立按AC/SD复核全集、语义事务/错误怪癖及真实消费者；保留merge-owned门pending | AC-06 |
| [ ] | T-10 | merge | T-09 pass | 合流master并集、受影响内容复审，最终冻结提交统一§6门；失败修复后重新绑定审查/重冻，不跳过 | AC-07 |
| [ ] | T-11 | merge | T-10 | canonical Spec/ledger沉淀、归档、已合入接口与发行来源交接，guard clean再清理树/分支/组，核对终态计数 | AC-07 |

handoff：唯一`D:/autostack/.wt/lang-810/auto-lang`，分支`codex/plan-810-dev`；work确认实际base/组内兄弟依赖，同树顺序用于work/review/merge，不在master改代码、不建junction/symlink。与811无业务硬依赖；与808/809仅公共manifest/777/792/测试/装配精确片段协调，见本批报告。

## 9. 复审记录

2026-10-19 / stage: new / PLAN-810 revision: 1 / outcome: pass（合同起草与文档预检，**非实现/独立验收pass**）/ next: work T-01 / changed: T-01..11、AC-01..07、SD-01..06。仅规划，未执行任何未来fixture或Cargo门；实际来源/hash与合同预检见本批附件。

2026-10-19 / stage: work / PLAN-810 revision: 1 / outcome: pass（work 自有 AC-01..05 完成，送独立 final review）/ worktree: `D:/autostack/.wt/lang-810/auto-lang` / branch: `codex/plan-810-dev` / base: `ab08955c2` / code: `c8183ff73`→`605ed28d1`→T-06/07 提交→T-08 备稿提交（见 git log；review 绑定最终提交） / dependency_revisions: auto-parser-api/auto-ui-dialect 均在本树内，无跨仓 / task_ids: T-01..T-08 / evidence: contract-baseline §5/§6（17 定向族全绿 + plan810 13 + 轻 consumer 7 + verify-deps×2 + 777 strict pass；792 driver 后台收据回填 handoff）/ blockers: 无 / next_responsible_role: review（T-09，同树独立核 AC/SD/反例；merge-owned 裸 t/tv/tu 保持 pending——handoff §5）。执行期勘误已回填 contract §5（expect_ident 留 Core 等 5 项，均在合同授权范围内："Core与UI共享的on/body辅助归通用语义所有者、UI调用端口"）。

2026-10-19 / stage: work / PLAN-810 revision: 1 / outcome: needs_fix 修复轮回送复审（R1：F810-R1-01..06 全闭合）/ worktree: `D:/autostack/.wt/lang-810/auto-lang` / branch: `codex/plan-810-dev` / base: `ab08955c2` / code: R1 修复 `55ce34200`→`0f2995e5f`→toy 端口面补齐（review 绑定分支 HEAD；初轮 `ccd024d72` 见 review-r1 基线）/ dependency_revisions: 无跨仓 / task_ids: T-02/T-06/T-07/T-08 重开再闭合 / evidence: review-r1/REVIEW.md 六 findings ↔ handoff §7 逐项修复表（plan810 20/20 含 r01 快照生命周期/r04 prev 对照/三层 import 链；四后端代表定向绿；777 strict pass+负控 15/15；fmt 净） / blockers: 无 / next_responsible_role: review（T-09 第二轮，同树绑新 HEAD）。

2026-10-19 / stage: review / review_scope: final / reviewer_identity: 当前 Codex 独立复审上下文（精确模型/会话 ID 未暴露，未创建 agent）/ PLAN-810 revision: 1 / outcome: **needs_fix** / worktree_path: `D:/autostack/.wt/lang-810/auto-lang` / branch: `codex/plan-810-dev` / reviewed_commit: `ccd024d7208caa14fb2f71d71b8d256d2ba74f82` / base_commit: `ab08955c241f6a824091c53d3b02894ca17e193b` / dependency_revisions: auto-down `c39ac7f9dacdc769fc4a5ad3b98ec896240b28e8`，本树/兄弟 clean，Cargo.lock 与 Spec 指纹见 [baseline](attachments/810/review-r1/baseline.json) / acceptance_results: AC-01/03/04 partial，AC-02/05/06 fail，AC-07 merge-owned pending / findings: **F810-R1-01..06 OPEN**（成功快照残留、777 基线过期实红、强制 oracle 空壳/缺口、scene 前瞻 prev 漂移、SD 冲突、fmt/新增 warning）/ evidence: [完整报告/AC/反例与修复要求](attachments/810/review-r1/REVIEW.md)、[可重放反例](attachments/810/review-r1/probes.rs)、[观测结果](attachments/810/review-r1/probes.log)、[冻结 SD](attachments/810/review-r1/spec-delta.patch) / pending_closeout_gates: 裸 t/tv/tu（触 trans 再 tt）、master 并集影响复核、规范/ledger/归档/wt-guard 清理；本轮未执行整档 / next: **原树 work 修完整 findings 后提交复审**，不合入、不清理。重开 T-02/T-06/T-07/T-08，保留 T-01/T-03/T-04/T-05 既有成果，`executing 4/11`；此前 work pass/777 绿为历史声明，以本轮实测结论为当前状态。

## 10. 待澄清事项

当前无需用户额外决定的阻塞项。T-01负责有界端口可行性与语义冻结，不能先复制Parser再补边界。若需破坏旧公开API、改变既有错误/文法/支持面、迁UI AST或改变上层默认组合包归属，应具体needs_replan；不能以部分函数迁出替代本Plan完整交付。
