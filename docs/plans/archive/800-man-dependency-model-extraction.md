---
plan_id: PLAN-800
status: archived
feature_name: Man 依赖模型与路径解析核心解耦
author: [Codex]
created_at: 2026-10-16
updated_at: 2026-10-17
plan_revision: 1
current_step: 8
total_steps: 8
supersedes_spec_components: [docs/specs/auto-man/project.md]
new_spec_components: [docs/specs/package-resolver/project.md]
touched_goals: [GOAL-003, GOAL-016]
affects: [auto-man, package-resolver]
depends_on: [PLAN-780, PLAN-786, PLAN-795]
---

# [PLAN-800] Man 依赖模型与路径解析核心解耦

## 0. 变更摘要

新建auto-package-resolver，将auto-man/resolver.rs中依赖名称/路径视图、包路径候选/搜索及依赖内模块判定抽成轻核心；ExecutionMode、ModulePath/ModuleResolver和pac读取留在上层adapter。既有AutoManResolver入口真实委托，不仅新增另一套闲置DTO。

## 1. 目标

独立无语言实现消费者可按当前规则解析已解析依赖与模块路径；保留Dependency/AutoManResolver公开调用、模式选择、路径优先级与错误阶段。非目标：全面切换pac.at至轻静态解析、修复legacy扫描语义、依赖下载/版本求解、Target/build迁移或删除语言resolver trait。

## 2. 架构方案

auto-man(adapter) → auto-package-resolver(名称/路径/观察规则)；新包不依赖auto-lang/auto-ast/VM/Man/renderer/执行模板。语言AST ModulePath/PathPrefix与ExecutionMode在Man适配边界转换；核心只见owned/borrowed轻名称、segments、路径及必要文件观察。模式标签对核心不透明，解释和默认/alias语义留语言适配层，不复制第二套ExecutionMode解析器。
780的ProjectDescriptor/dep声明继续是源配置模型，本计划只承载已解析的依赖视图/路径规则，不再造pac parser或竞争schema。auto-package-lock继续拥有锁格式/验证，不能复制进resolver core。

## 3. 技术栈

Rust新workspace成员、std路径与文件观察、现有auto-val轻字符串（确需才依赖）；auto-man现有resolver/pac读取adapter、独立consumer和真实临时普通目录，不创建junction。

## 4. 需求分析与背景调查

取材主检出 auto-lang@`30d3e1d2a011e2a34243bb8a2b141259af721edc`；[输入指纹](attachments/model-decoupling-20261016/inputs.json)记录相关Specs、AST全目录和Man resolver代码SHA256。以Spec为现行合同、源码为事实，不以账本或旧勾选替代验收。
用户2026-10-16明确授权建立“前端模型解耦”和“Man依赖模型解耦”两份实施计划，并要求明确与792/794并行边界及合入顺序。本轮只建计划和更新设计/路线，不执行Rust、不创建agent/工作树、不发布Spec、不操作旧仓。后续确认对应合同再进入work。
789/793/795均已合入归档；本计划无792/794交付硬依赖。取材时792为execution_done（有needs_fix复审记录，实际由该线继续修复，不冒称通过），794为executing；最终实际状态须work重新核对。其他796/797/798实验线不设全局前置，只按真实写面协调。
实际resolver.rs导入语言ModuleResolver、ExecutionMode及AST ModulePath/PathPrefix。prepare_env/load_pac_at是legacy行扫描：mode/use/dep:等并非780完整静态模型；find_package_path、find_module_in_dep及ModuleResolver::resolve/exists/search_paths混有不同存在性/候选规则。resolve可能返回尚未存在路径，exists不是其同义检查；不能“顺手统一”。Pac/Super/None当前委托语言FilesystemResolver，Dep由Man规则查找。
Dependency公开含ExecutionMode，get_dependencies返回HashMap引用；兼容层不能静默改成新类型或维护两个不同步的权威注册表。原API的std/相对/依赖/search优先关系及错误候选集需逐入口冻结。
本计划不依赖799；始终通过auto_lang旧兼容路径使用语言类型，799前后均应可编译。与在途工作按[并行与合流合同](reports/model-decoupling-parallel-order-20261016.md)执行：主要实现可并行；共享manifest/导出/777基线短窗口串行。若多项同时ready，推荐792→794→799→800；这不是depends_on，先完成者可以先合，后合入者承担同步与受影响复审/冻结门，不能等待792未完成的接口或扩大其职责。
新crate或source身份影响792交付闭包时，先合入者给出包/路径/接口清单，后合入者按实际已交付接口更新清单并重放相关fresh/locked/重定位与资源保护控制。792若尚未交付，只做兼容接线、不得伪造其不存在的验收证据。

## 5. 详细设计

T01形成源声明→模式适配→已解析依赖视图→路径候选→语言trait的责任表，冻结所有公开方法/构造与三个路径入口（resolve、resolve_with_prefix、exists）各自行为。旧不确定HashMap候选枚举只归一其无序部分比较，不把搜索优先序也归一掉。
核心抽取find_package_path、依赖内file.at/mod.at候选与歧义判定，以及能不依赖语言的命名/相对/search路径规则；生产AutoManResolver薄委托。Pac/Super/None仍由上层语言FilesystemResolver处理，不复制其语法/根定位。Mode保持旧枚举身份与默认优先关系；核心不得用unknown mode静默选择别的后端。
load_pac_at当前扫描算法与警告/缺文件行为仍留adapter原样，包括未找到依赖的当前处理；不在拆分中新增网络、物化或执行动态配置。若未来改成780源模型，另立显式兼容合同；这里不扩大其生产消费域。
采用借用投影/重建视图或受控同步等具体办法由T01论证，必须有add_dependency同名替换后立刻解析正确的反控；不可同时缓存两份权威map。公共Dependency/AutoManResolver旧方法及返回类型保持；新核心可有独立DTO，不要求旧语言类型下沉。
歧义、not-found、路径构造和source位置/阶段按旧规则保持，未承诺路径安全沙箱、canonicalize、commit验证或全生态跨平台支持；不借抽取加入新限制。未迁Target/build/Git/动态配置与CLI列清。

与792的窄接线例外：本计划后合入且新包确在选定交付闭包中时，允许在其已交付source清单/依赖投影处补包/producer身份和对应fixture；不改rust_ui/automan的生成/交付策略、CLI、staging/admission/失败保护算法。若新包不在该闭包内，记录metadata证据即可，不无端将整个Man栈打入UI包。需改算法时先具名协调修订，不能留无所有者的漏包修复。

### 规范增量

| delta_id | 操作 | target | before/after rule | 原因 | AC |
|---|---|---|---|---|---|
| SD-01 | add | docs/specs/package-resolver/project.md | 轻依赖视图、路径候选/存在性/错误规则和无语言边界 | 独立可消费 | AC-01/02/03 |
| SD-02 | modify | docs/specs/auto-man/project.md | 现有AutoManResolver委托核心；模式/AST/pac扫描留adapter，780/795所有权不变 | 真实解耦与未迁边界 | AC-03/04 |

## 6. 测试设计

work、独立final review、merge分别交接；未分配额外agent或模型。各角色使用本计划同一棵实现树，以下新fixture由work创建并运行；本次建计划不算测试证据。work先完成全部修复、定向验收与必要cargo check，提交后送final review；整档门后置merge冻结提交，tf仍主检出到期批量单实例。Category A本轮禁止cargo整档/docs_gen。

| Case / owner | AC | 输入/生产入口 | oracle、边界与反控 |
|---|---|---|---|
| V01 / work | 01 | 新test/fixtures/modularity/package-resolver-consumer，用普通目录与轻依赖输入 | fresh manifest真实resolve/歧义/缺项测试；normal/build闭包无auto-lang/AST/Man/VM/UI |
| V02 / work | 02 | 旧resolver冻结std、./../、已声明依赖、额外search、Dep/其它前缀，file.at与mod.at单独/同时/均无 | 每入口路径、优先级、错误类别和文本投影保持；歧义不能成功，resolve与exists差异不抹平 |
| V03 / work | 02/03 | legacy pac mode/use/dep:、无pac、坏mode、同名dependency覆盖及两模式consumer | prepare_env和旧类型/返回引用兼容；替换后立即查新路径/新mode，不出现stale shadow map；无新增扫描/执行/网络回退 |
| V04 / work | 03 | 真AutoManResolver::resolve/resolve_with_prefix/exists、ModuleResolver trait与799前/后旧AST兼容路径 | 生产入口实际委托单一规则，mode/PathPrefix留adapter，核心无语言impl；780/795无算法复制 |
| V05 / work | 04 | 与792交付闭包对账及794八native接口不变检查 | 只有实际闭包受影响才更新source/manifest并重验792相关入口；不为无关锁/资源强建依赖 |

拟树D:/autostack/.wt/lang-800/auto-lang，branch codex/plan-800-dev。开发core/auto-man resolver定向测试和check，必要外部真实目录验证均work-owned；merge-owned裸t+两crate定向门。未改VM/trans/ui_gen不类比强加tv/tt/tu；若扩大触面先按规则补相应档。800%5=0合入后移交主检出regress，不能本树并行跑tf。

## 7. 验收标准

| ID | owner_stage | 可观察结果 |
|---|---|---|
| AC-01 | work | 新核心无语言/AST/Man实现依赖，独立consumer V01通过 |
| AC-02 | work | 原公开类型/模式、路径/候选优先序与错误阶段保持，V02/03通过 |
| AC-03 | work | 既有resolver真实委托、无两份权威map/路径算法；adapter/源模型/锁core所有权清楚，V03/04通过 |
| AC-04 | work | 未迁项/净搬迁与来源清单、并行合流证据和SD素材完整，V05触面闭合 |
| AC-05 | review | 独立目录/旧API/模式/负控及SD审查，绑定真实提交依赖 |
| AC-06 | merge | 冻结门禁、Spec/ledger/归档/guard清理和800批回归到期交接完成 |

## 8. 执行步骤

| 状态 | ID | owner_stage | 依赖 | 路径/交付与验证 | AC |
|---|---|---|---|---|---|
| [x] | T-01 | work | 780/786/795归档、合同确认 | resolver.rs所有公开方法/模式/观察/legacy pac读取责任表；冻结V02/03旧臂、DTO/单一map方案及共享窗口 | 01/02/04 | [✅ 已完成] 冻结面+责任表+DTO/单一map方案见 attachments/800/t01-freeze.md；指纹commit 30d3e1d2a→master b0d6d4aab 关键输入零漂移（仅根Cargo+5行他线）；780/786/795归档核验通过 |
| [x] | T-02 | work | T01 | 新crates/auto-package-resolver、轻输入/路径候选与歧义规则；cargo check新crate | 01/02 | [✅ 已完成] std-only 新 crate（DependencyEntry<M>/PackageResolverCore<M>/PackagePathNotFound）；`cargo check -p auto-package-resolver` 通过；workspace member/exclude 已登记 |
| [x] | T-03 | work | T02 | 抽独立resolve/exists/search规则，保留各入口差异；旧语言前缀与mode留Man adapter | 02/03 | [✅ 已完成] find_package_path/find_module_in_dep/resolve/exists/module_search_paths 全入核心，三入口差异冻结（resolve 无盘臂/exists 无 package.at 臂/歧义不得成功/错误文本逐字）；核心单测 14/14 绿 |
| [x] | T-04 | work | T03 | AutoManResolver生产薄委托、旧API/get_dependencies和替换可见性保持；删除原路径算法，check/定向测试 | 02/03 | [✅ 已完成] adapter 薄委托（trait 四方法一行委托；Pac/Super/None 继续走 FilesystemResolver）；`Dependency=DependencyEntry<ExecutionMode>` 别名保调用面；auto-man 内路径算法 grep=0；`cargo test -p auto-man --lib resolver::` 38/38 绿（1 预存 ignore，含 4 新 plan800 例） |
| [x] | T-05 | work | T04 | 新外部consumer、V01..05、metadata/777精确边界；799两种兼容接线及792条件触面核查 | 01–04 | [✅ 已完成] package-resolver-consumer 7/7 + verify-deps.sh PASS（闭包=仅核心）；module-boundaries 登记 man-ecosystem+audit 零违规；792 work 时点未合入、review-r1 时点已合入——经核对 rust_ui DELIVERY_CRATES 10 项不含 auto-man 且闭包无本包边→不入 792 交付闭包、无清单接线义务（详见 t06 §6 merge 注记）；794 diff 面零 auto-lang/VM/UI 文件；799 兼容=旧 ast 路径身份不变；证据全录 attachments/800/t06-work-receipt.md |
| [x] | T-06 | work | T05 | 净删/搬迁/未迁和合流收据、SD素材，提交冻结代码、work项闭合 | 01–04 | [✅ 已完成·r1] r0 收据+SD 素材随 `5c1f43b5d`；review-r1 needs_fix（F01 错误映射描述失实/F02 新核心未过 rustfmt）已修——四处文档对齐真实边界（无 adapter 映射、legacy 调用点静默忽略）、新文件全量 rustfmt+adapter 定点修 7 处（余 26 处标记行经比对全部逐字存在于 master=历史搬运）；r1 修复提交 `c50e7a1c6`，三面定向复验全绿；SD01 新 hash `8BF942D5…`；明细 t06 §6 |
| [x] | T-07 | review | T06 | 同树独立真实路径/模式/负控/单一实现及SD冻结 | 05 |
| [x] | T-08 | merge | final pass | 同步/必要复审、冻结门/条件792重验；规范/ledger/归档/guard，800到期批回归交接 | 06 | [✅ 已完成] R2 pass 绑定 c50e7a1c6；rebase over 792/787/788/798 线（range-diff 2/2 全等，映射 5c1f43b5d→50c3a955c、c50e7a1c6→02b11c4b4；SD02 按含 792 章节的当前 main 增量并集落位）；冻结门禁全绿：裸 cargo t 5335/5335（首跑 plan484_024 满档负载 flake——solo 绿于 worktree/master 双侧、零 auto-lang 触面，P793 同 flake 在案先例；有据重跑过）+ 核心 14/14 + adapter resolver:: 38/38 + consumer 7/7 + verify-deps.sh PASS；delivery 02b11c4b4 ff-only 落 master；792 重验=DELIVERY_CRATES 10 项不含 auto-man、闭包无本包边（R1/R2 已核，main 代码面未再变）；SD-01/02 canonical + 双 plans.md 行 + GROUPS/INDEX 再生；ledger v38→v39 guarded offline projection（P800-D1/D2/T/R2/M）；归档 archive/800-…；wt-guard clean 后工作树/分支/auto-down 兄弟/组目录清理；800%5=0 批回归到期交接 /auto-plan:regress |

## 9. 复审记录

2026-10-16 stage=new | plan_revision=1 | outcome=pass（合同preflight） | next=合同确认、共享写面释放后work | 0/8。不以792/794/799为交付硬前置；当前未实施或执行Rust。

2026-10-16 stage=work | PLAN-800 | plan_revision=1 | outcome=pass | worktree=D:/autostack/.wt/lang-800/auto-lang | branch=plan-800-dev（计划拟名 codex/plan-800-dev 按仓规范改 plan-800-dev，见 t01-freeze.md §5） | base_commit=b0d6d4aab | code_commit=5c1f43b5d | dependencies=auto-down 兄弟 detached c39ac7f（只读，autodown-core path 解析） | task_ids=T-01..T-06 | evidence=核心 14/14、adapter resolver:: 38/38（1 预存 ignore）、consumer 7/7+闭包 PASS、check 两 crate 绿、audit 零违规、收据 attachments/800/t06-work-receipt.md | blockers=无 | next=review（T-07 同树独立复审；merge-owned：裸 cargo t + 两 crate 定向门 + consumer 门 + 800 到期批回归交接）

work-owned readiness：AC-01..04 全闭合（V01..05 证据绑定 5c1f43b5d）；无未结 in-contract finding；旧 API/模式/路径行为断言零改动通过；merge-owned 门显式 pending（非 work 关闭项）。

2026-10-16 stage=review | review_scope=final | reviewer=当前独立审核对话（未参与实现，无额外agent） | PLAN-800 | plan_revision=1 | outcome=needs_fix | worktree_path=D:/autostack/.wt/lang-800/auto-lang | branch=plan-800-dev | reviewed_commit=5c1f43b5dfebbc5fdffa891046a343ccd902ef79 | base_commit=b0d6d4aab50bfcb137d55b0f88e9f2bd82b8f644 | dependency_revisions=auto-down c39ac7f9dacdc769fc4a5ad3b98ec896240b28e8 | acceptance_results=AC01/02/03 pass、AC04 partial、AC05 fail、AC06 merge pending | findings=F800-R1-01 SD错误映射描述与实现不符、F800-R1-02新增核心格式门失败 | evidence=[R1审核收据](attachments/800/review-r1/REVIEW.md)（含Spec冻结hash及独立反例） | pending_closeout_gates=裸t/两crate/consumer/同步/Spec与ledger/归档guard/批回归到期交接 | next=work修复提交后同树定向复审。重开T06/T07，保留T01..05已通过证据；current_step=5/8。792现已合入，当前10crate交付闭包不含auto-man，新core不入闭包，无新增交付接线义务。

2026-10-17 stage=work | PLAN-800 | plan_revision=1 | outcome=pass（r1 修复交接） | worktree=D:/autostack/.wt/lang-800/auto-lang | branch=plan-800-dev | base_commit=b0d6d4aab | code_commit=c50e7a1c6（r0 实现 5c1f43b5d + r1 修复） | dependencies=auto-down c39ac7f（只读不变） | task_ids=T-06（重开→闭合） | findings_closed=F800-R1-01（四处文档对齐真实边界：无 adapter 映射、legacy load_pac_at `if let Ok` 静默忽略；按审核裁定不加无消费方映射代码）；F800-R1-02（新核心/consumer fixture 全量 rustfmt exit 0；adapter 定点修 7 处新代码，余 26 处 rustfmt 标记行内容比对全部逐字存在于 master=历史搬运不格式化） | evidence=复验全绿：核心 14/14、adapter resolver:: 38/38（1 预存 ignore）、consumer 7/7、verify-deps.sh PASS；修复明细+新 SD 冻结 hash（SD01 `8BF942D5…`、SD02 不变）见 attachments/800/t06-work-receipt.md §6 | blockers=无 | next=review（同树定向复审 T-07：绑定 c50e7a1c6；merge-owned 门保持 pending——裸 cargo t + 两 crate 定向门 + consumer 门 + 800 到期批回归交接；merge 时 SD 须按含 792 章节的当前 master 合并）

work-owned readiness（r1）：F800-R1-01/02 两 finding 闭合、无未结 in-contract 项；受影响面定向复验通过；行为零变化（P2 纯文档、P3 纯格式）。

2026-10-17 stage=review | review_scope=final | reviewer_identity=当前独立审核对话（未参与实现、无额外agent） | PLAN-800 | plan_revision=1 | outcome=pass | worktree_path=D:/autostack/.wt/lang-800/auto-lang | branch=plan-800-dev | reviewed_commit=c50e7a1c670c695766dfd81dcd572b446a99a31c | base_commit=b0d6d4aab50bfcb137d55b0f88e9f2bd82b8f644 | dependency_revisions=auto-down c39ac7f9dacdc769fc4a5ad3b98ec896240b28e8 | spec_inputs=main auto-man AA785B1D…/SD01 8BF942D5…/SD02 8FCA0393…（完整指纹与冻结副本见R2收据） | acceptance_results=AC01..05 pass、AC06 merge-owned pending | findings=F800-R1-01/02 closed，无新阻断项 | evidence=[R2最终复审收据](attachments/800/review-r2/REVIEW.md)：核心14/14、resolver筛选38 passed/1预存ignored、consumer7/7、闭包/新文件格式/独立反例全通过；adapter规范化前后字节一致，残余格式全为历史代码 | pending_closeout_gates=同步/必要受影响复审、冻结裸cargo t+两crate+consumer、Spec/ledger/索引/归档/guard清理、800批回归到期交接 | next=merge。T07完成，7/8；reviewed只表示代码/验收/规范审查通过，merge-owned门通过前不得合入或宣称交付完成。

## 9.5 合入收口记录（PLAN-800:r1，merge stage）

- **prepared**：reviewed 基线 c50e7a1c6（R2 pass，plan_revision 1）；canonical diff=SD-01 新
  docs/specs/package-resolver/project.md + SD-02 auto-man spec 增量节（rebase 后按含 792
  章节的当前 main 并集落位，非整文件覆盖）；delivery 候选=rebase 后 tip。
- **landed**：rebase over master 143aff0a2（792/787/788/798 线合入后），`git range-diff`
  2/2 全等（`5c1f43b5d→50c3a955c`、`c50e7a1c6→02b11c4b4`）；master `git merge --ff-only
  plan-800-dev`，tip=delivery commit `02b11c4b4`，零 merge commit。冲突面零（Cargo.toml
  /auto-man spec/manifest 自动并集正确，双侧 section/member 逐项核验）。
- **frozen_gates**（02b11c4b4 上，worktree）：裸 `cargo t` **5335/5335**（1 slow；首跑红
  plan484_024_charts_streaming_recompute=满档并行负载 flake——solo 复跑绿于 worktree 与
  master 双侧、本计划零 auto-lang 触面、P793 同 flake 在案先例，归因后有据重跑过）；
  `cargo test -p auto-package-resolver` 14/14；`cargo test -p auto-man --lib resolver::`
  38 passed/1 预存 ignored；consumer fixture 7/7 + verify-deps.sh PASS。无 VM/trans/book/
  ui_gen 触面→不跑 tv/tt/tb/tu。
- **792 重验**：master DELIVERY_CRATES 10 项不含 auto-man、全部闭包清单无
  auto-man→auto-package-resolver 边（R1/R2 复核；R2 后 main 代码面未变，本次 landing
  仅新增不被其消费的成员）——本包不入 792 交付闭包，无清单/锁接线义务。
- **ledger_refreshed**：`.autoos/specs.json` v38→v39 **guarded offline projection**
  （Regime B，无 store writer；沿用 P787/P792/P794 先例）：备份 v38-800；新增
  P800-D1/D2（designs）、P800-T（tests）、P800-R2（reviews）、P800-M（reports）五条目，
  均带 file/tags；范围守卫=diff 仅 P800 条目+版本号；回读校验 IDs/六节完整。
- **spec_deposit**：SD-01/02 随 delivery 入库；`docs/specs/package-resolver/plans.md` 新建
  （800 行）+ `docs/specs/auto-man/plans.md` 追加 800 行；`scripts/spec-index.py` GROUPS
  工具链登记 package-resolver + INDEX 再生（38 项目，余 ungrouped 仅预存
  repository-boundaries）。
- **archived**：`git mv` → docs/plans/archive/800-man-dependency-model-extraction.md，
  status=archived，8/8；本收据与 attachments/800/**（含 review-r1/r2 材料）随终态提交
  入库 master。
- **cleaned**：wt-guard 双树 clean（auto-lang/auto-down 均无 reparse point）→
  `git worktree remove` lang-800/auto-lang + `git branch -d plan-800-dev`（was 02b11c4b4，
  已落地）+ auto-down 只读兄弟移除 + 空组目录 `rmdir D:/autostack/.wt/lang-800`——
  零残留（2026-10-17 回填验证）。
- **batch_regression**：800%5=0 → 到期，交接 `/auto-plan:regress`（主检出单实例；
  收据 docs/plans/.last-batch-regression.json 由该技能落盘）。

## 10. 待澄清事项

T01具体轻投影/借用设计须保持旧HashMap引用API及mode身份；若需修改既有source配置识别/不兼容规范路径，应先具名修订。不能把legacy扫描弱点偷偷修成新的pac协议，也不能为避环复制语言AST/模式枚举解析。
