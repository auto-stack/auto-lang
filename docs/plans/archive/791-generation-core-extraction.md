---
plan_id: PLAN-791
status: archived
feature_name: 生成核心与 Auto 可执行模板适配拆分
author: [Codex]
created_at: 2026-10-16
updated_at: 2026-10-16
plan_revision: 1
current_step: 8
total_steps: 8
supersedes_spec_components: [docs/specs/auto-gen/project.md]
new_spec_components: [docs/specs/auto-gen-core/project.md]
touched_goals: [GOAL-003, GOAL-016]
affects: [auto-gen, auto-gen-core]
depends_on: [PLAN-784, PLAN-786]
---

# [PLAN-791] 生成核心与 Auto 可执行模板适配拆分

## 0. 变更摘要

新建auto-gen-core，将保护区段、输出路径/文件编排、通用错误与生成报告从Auto解释执行中拆出；现有auto-gen成为Auto数据/模板adapter及兼容门面，生产CodeGenerator真实调用新核心。消除这部分实现重复，但不冒称auto-man已独立。

## 1. 目标

纯生成核心可由无auto-lang/VM/UI的外部consumer调用；原Auto模板/CLI和legacy AutoGen API维持行为。非目标：把解释型模板改成文本替换、删除legacy API、全面重构auto-man包/锁/CLI或升级模板语法。

## 2. 架构方案

auto-gen → auto-gen-core；core不依赖auto-lang/auto-man/renderer。AutoInterpreter、AST Code、LoadedData留在auto-gen adapter。core通过小的renderer回调/trait拿到owned文本，不持有解释器或Shared<AutoInterpreter>。guard/error/报告类型只保留一个定义，用re-export或明确映射保持旧公开路径；不能由core反调具体Auto adapter。

## 3. 技术栈

Rust新workspace成员、现有GuardProcessor/GenError/GeneratorConfig/GenReport、临时目录与独立Cargo consumer。不预造插件进程协议或动态加载器。

## 4. 需求分析与背景调查

取材基线 auto-lang@`3e65540cbe6ac019d210318dc93d17accf4ebcc4`；[输入指纹](attachments/extraction-batch-20261016/inputs.json) 记录现行 Specs、代码和 781/784/785/786 归档合同的 SHA256。overview 是导航快照，以相关模块 Spec 和实际源码为准。
用户 2026-10-16 授权“建立下一批实施计划”；本轮只建立合同及更新设计/路线，不实施 Rust、不创建执行树、不更改旧仓 refs、不发布/推送。之后按确认的合同进入 work；不能把本次建计划算作实施授权或完成证据。
781/784/785/786 均已归档；旧仓主分支和所有恢复载荷继续保留。787/788 为 AC/AutoDown 实验与集成线，不是本批全局前置，实施时按实际 diff 复核共用 manifest/数据类型热点。
实际 template.rs 使用 AST Code/AutoInterpreter，不是普通字符串替换；data.rs 的 LoadedData直接含Shared<AutoInterpreter>。generator.rs混合数据加载、template render、guard合并和写文件；guard.rs和error.rs只需轻值/regex/标准库。lib.rs还保留deprecated AutoGen，auto-man/asset.rs另有可执行模板消费，不能误认为auto-gen抽取就消掉Man→语言全部依赖。依据auto-gen/project与Design37 E波。
与789共用workspace/777归属短写面，实施可并行，合流时重核；不占用790 native热点。792在三者合流后固定source闭包。

## 5. 详细设计

core接收模板输入描述、渲染回调和输出配置，负责现有输出命名、guard保留/冲突、dry_run/overwrite_guarded和单次生成报告。Auto端负责数据加载、库路径、Auto模板解析及执行错误映射，默认语义不变。保留当前报告内收集单模板错误、批次继续和顶层加载错误的阶段边界；不能顺手改成全批原子提交或全部fail-fast。
文件写入语义以当前支持为准：render/guard失败不能先覆写既有文件，dry_run不产生目录/文件；多模板部分成功的既有报告规则保持。不承诺当前未实现的路径沙箱/事务策略。
删除迁出guard/通用编排旧实现，legacy wrapper调用共同核心或既有Auto adapter，不为兼容再复制算法。T01须区分legacy AG_GUARD与新begin/end保护语法；若语义不同，以明确兼容策略承载，不能把不同规则误删为重复或改写旧输出。记录auto-man/Templates仍未迁及未来UI tool/core边界；依赖体积/性能只报实测。

### 规范增量

| ID | 操作 | target | before/after rule | 原因 | AC |
|---|---|---|---|---|---|
| SD-01 | add | docs/specs/auto-gen-core/project.md | 新增无解释器生成核心及render/写入错误合同 | 可单独消费 | AC-01/02/03 |
| SD-02 | modify | docs/specs/auto-gen/project.md | 明确Auto可执行模板adapter、兼容API与单一核心 | 纠正简单文本引擎描述，不改模板能力 | AC-02/04 |

## 6. 测试设计

实施、独立 final review、merge 分别交接，未指定外部模型/额外 agent。所有人使用本计划同一棵实施树；本轮只给拟定路径，不创建它。V 用例由 work 建成可执行 fixture；下表是合同而非已通过结果。
开发只运行受影响定向案例及必要 cargo check；先修完全部 findings、提交并完成 final review，merge 再在冻结提交执行裸 cargo t 和触面档。cargo tf 仅主检出批量到期回归，不能在每棵树运行。无相关 Rust/Schema 改动不跑 docs_gen；UI codegen 触面用 tu，不用改金样消红。

| V / owner | AC | 输入/入口 | oracle 与覆盖界限 |
|---|---|---|---|
| V01 / work | 01 | 新test/fixtures/modularity/gen-core-consumer，以fake renderer生成文件 | 独立fresh manifest编译运行，normal/build闭包不含auto-lang/UI/auto-man；fake文本真到文件 |
| V02 / work | 02/04 | 旧提交冻结的AutoCode/AutoFile/Atom、fstr_note、use库、多模板、legacy API/CLI | 输出字节、命名、report错误归属一致，不能用新输出再造expected |
| V03 / work | 02/03 | guard保留/冲突、模板render失败、加载失败、dry_run、overwrite、已有输出和第二模板失败 | 失败文件旧字节不变、dry_run零写入；批次继续/报告同旧合同，避免只测成功生成 |
| V04 / work | 04 | 原CodeGenerator生产调用链及legacy/asset消费者检查、normal dependency图 | 真实默认用core、无算法双份；未选Man模板路径仍登记为未迁，不伪报删边 |

拟实施树 D:/autostack/.wt/lang-791/auto-lang，branch codex/plan-791-dev。定向cargo test -p auto-gen-core及auto-gen模块测试、cargo check -p auto-gen；merge-owned裸cargo t和新crate/adapter定向全测，未触VM/trans不凭类比加tv/tt。

## 7. 验收标准

| ID | owner_stage | 可观察结果 |
|---|---|---|
| AC-01 | work | 无解释器核心独立可用且DAG正确，V01通过 |
| AC-02 | work | 现有Auto模板、legacy/CLI输出与错误阶段保持，V02/03通过 |
| AC-03 | work | guard/失败/dry_run反控成立；旧实现净删除，core不持解释器类型 |
| AC-04 | work | 生产默认接core，未迁消费者/模板资产清单及source映射可追溯 |
| AC-05 | review | 独立consumer/失败出口/真实去重与SD复审通过 |
| AC-06 | merge | 冻结门禁与规范/ledger/归档、guard清理完成 |

## 8. 执行步骤

| 状态 | ID | owner_stage | 依赖 | 实施/产出/验证 | AC |
|---|---|---|---|---|---|
| [x] | T-01 | work | 784/786交付、合同确认 | 冻结generator/data/template/guard/error及legacy/Man消费图、输出与错误阶段旧臂（R1库臂已补：9c4780c54 replay旧码bless+14/14绿） | 02/04 |
| [x] | T-02 | work | T01 | 新建crates/auto-gen-core、迁单一guard/error/报告和小renderer边界；cargo check -p auto-gen-core | 01/03 |
| [x] | T-03 | work | T02 | 迁通用输出编排，接dry_run/guard/错误收集；加入失败文件不改写反控 | 02/03 |
| [x] | T-04 | work | T03 | auto-gen保留解释器adapter并真实委托core；旧API兼容re-export，删除重复算法 | 02/04 |
| [x] | T-05 | work | T04 | 外部gen-core-consumer、V01..04、metadata/777边界与既有Auto模板定向验证（R1库覆盖闭；R2基线干净树刷新后strict=pass闭） | 01–04 |
| [x] | T-06 | work | T05 | 净删/搬迁及剩余Man模板依赖收据、SD素材；冻结提交与完整交接（R1闭+R2基线闭，冻结561a61fc6） | 01–04 |
| [x] | T-07 | review | T06 | 独立字节对照、失败出口与真实生产调用、规范冻结（R3 pass，受审561a61fc6；merge-owned门禁pending） | 05 |
| [x] | T-08 | merge | final pass | 裸t+本crate/adapter定向门，沉淀/归档/guard清理（全过；见§9合并收据） | 06 |

## 9. 复审记录


2026-10-16 stage=merge | PLAN-791:r1 | outcome=pass（delivered） | 交付链（ff-only 落地，master tip=1120ae3b4）：311bd1241(T-01冻结)→f5ae117f3(core crate)→d0a0e98ee(adapter委托)→f461bbbfa(consumer门+777登记)→8c8945f05(SD素材)→9c8310a70(R1修复)→f20c11dc2(合并沉淀+ledger)→eead6dd58/3e1a9742c(并流复原+收据)→1120ae3b4(基线终锚)。**三轮 rebase over 并发 master 推进**（787/788/790 簿记→789 落地→790 落地）：第一轮 range-diff 7/7 全等；第二/三轮 `!` 仅含 union 解冲突（根 Cargo.toml exclude 三消费者并集、ledger 逐级 union v31→v32→v33、plans.md 双索引行并集、基线取最新锚后干净树重锚 43 边含 auto-lex/auto-gen-core 双方新边）与上下文漂移，代码提交保持 `=`。

| 检查点 | 证据 |
|---|---|
| prepared | reviewed_commit=561a61fc6（R3 pass）→rebase 等价映射（第一轮全等记录于[合并报告](reports/791-generation-core-merge.md)）；canonical diff=SD-01 新 auto-gen-core/project.md + SD-02 改写 auto-gen/project.md（R3 冻结 SHA 一致）；投影目标=ledger 六节中 designs/tests/reviews/reports |
| landed | master tip==1120ae3b4（--ff-only，无 merge commit）；canonical/specs/ledger/baseline/consumer fixture 均在 master；主检出 strict=pass 冒烟复验 |
| ledger_refreshed | .autoos/specs.json **v33**（789 五条∪790 五条∪791 五条=P791-D1/D2/T/R3/M；守卫式离线投影：无 store writer 可达、字节级备份、diff 证明仅 version 行+追加、goals/architecture 逐字不变、回读校验 IDs/内容/链接） |
| archived | docs/plans/archive/791-generation-core-extraction.md（本 git mv）+ attachments/791 review-r1..r3 入库 + 本收据提交（未遗留 untracked docs/） |
| cleaned | wt-guard 双树 clean → worktree auto-lang/auto-down 移除、branch plan-791-dev 删除（止于 1120ae3b4）、组目录 .wt/lang-791 移除 |

merge-owned 冻结门禁（@1120ae3b4 终态，789/790 相继改 auto-lang 源码后重跑）：裸 cargo t **5326/5326 绿**（108.8s；此前轮 5382 与 5318 三次全绿 + 两次满池瞬态均归因预存 state_file 锁时序 flake：solo 3/3 绿、PLAN-044/738 谱系、本计划零 auto-lang 触面）；定向 7 套件绿（lib11+v02_14+v03_8+core12+doc5）；consumer 4 绿+闭包 PASS；fmt --check=0；boundaries strict=pass（grandfathered 2 不变）。tv/tt/tb/tu/taa 未触面不适用。

部署物检查：auto-lang 仓无运行时生产工件（backend 二进制/web bundle/daemon 属 auto-musk 域），纯库/测试面无可重建项。批量回归到期判定：**DUE**（790%5==0 已落地，last_covered=786）——790 合入已向 /auto-plan:regress 交接，本计划不重复触发；收据待批量档更新 last_covered≥791。

2026-10-16 stage=review | review_scope=final | reviewer_identity=本聊天Codex（未参与实施，无额外agent；R1/R2同一独立审查上下文） | plan_id=PLAN-791 | plan_revision=1 | outcome=pass | worktree_path=D:/autostack/.wt/lang-791/auto-lang | branch=plan-791-dev | reviewed_commit=561a61fc694f6f1a0a493300bf6385c8e11f96dd | base_commit=e4ac72a6265c85cdadeedbabc7a0c6fd93948e48 | dependency_revisions=auto-down@c39ac7f9dacdc769fc4a5ad3b98ec896240b28e8，Cargo.lock未变 | spec_inputs=两卡片hash及冻结delta与R2一致 | acceptance_results=AC-01..05 pass，AC-06 pending | findings=F-791-R2-01 closed，R1两项保持closed，无新finding | evidence=[R3报告](attachments/791/review-r3/review.md)，新冻结头strict exit0/verdict=pass violations=0 grandfathered=2；41声明边与豁免不变，baseline仅base_revision重绑；diff-check exit0，worktree clean；R2独立50项定向+4consumer及fmt等证据因实现/测试/锁/配置未变明确复用，未重复跑 | pending_closeout_gates=merge-owned裸cargo t+core/adapter定向全测、规范/ledger沉淀、归档/合入与guard清理 | next=merge执行T-08，门禁通过前不得合入或宣称交付完成。T-07完成，7/8；review未修改实现/发布规范/合入/归档/清理。前两轮needs_fix及work声明均保留为历史。


2026-10-16 stage=work(needs_fix修复R2) | plan_id=PLAN-791 | plan_revision=1 | findings=F-791-R2-01 → 闭合 | worktree_path=D:/autostack/.wt/lang-791/auto-lang | branch=plan-791-dev | base_commit=e4ac72a62 | code_commit=561a61fc6（新冻结，worktree clean） | task_ids=T-05/T-06重开后闭合 | evidence=R1修复改crates/auto-gen/Cargo.toml字节（EOF空行删除）而777冻结基线未刷新→strict stale_baseline；按复审要求在干净树b54cc4a2a上显式--refresh-baseline（base_revision重绑b54cc4a2a1b9，41边/合法边不变：auto-gen→auto-gen-core、auto-gen-core→auto-val；grandfathered 2不变：auto-man/auto-gen→auto-lang，R5退出），strict verdict=pass violations=0 grandfathered=2，未跳过/放宽陈旧判定；基线提交561a61fc6。R1两项闭合事实与R2报告一致，未重复改动。 | blockers=无 | next_responsible_role=review（R3）

2026-10-16 stage=review | review_scope=final | reviewer_identity=本聊天Codex（复审上下文未参与实施，无额外agent） | plan_id=PLAN-791 | plan_revision=1 | outcome=needs_fix | worktree_path=D:/autostack/.wt/lang-791/auto-lang | branch=plan-791-dev | reviewed_commit=b54cc4a2a1b95631c9e4726985615d66ba116c7e | base_commit=e4ac72a6265c85cdadeedbabc7a0c6fd93948e48 | dependency_revisions=auto-down@c39ac7f9dacdc769fc4a5ad3b98ec896240b28e8，Cargo.lock及边界指纹同R1 | acceptance_results=AC-01 pass, AC-02 pass, AC-03 pass, AC-04 partial（strict失败）, AC-05 fail, AC-06 pending | findings=F-791-R1-01/02已关闭；新增F-791-R2-01(P2，manifest末尾空行修复导致strict基线过期) | evidence=[R2报告与原始证据](attachments/791/review-r2/review.md) | pending_closeout_gates=merge-owned裸cargo t+core/adapter定向全测+规范/ledger/归档/guard清理 | next=原树按777规约核对并刷新基线、strict正控复验后提交重审；T-05/T-06重开，4/8，其余已验证任务保留。50项定向+consumer4项通过，fmt/diff-check=0；首次sccache端口环境失败已在本命令禁用wrapper后重现通过，不改工程配置。


2026-10-16 stage=work(needs_fix修复) | plan_id=PLAN-791 | plan_revision=1 | findings=F-791-R1-01+F-791-R1-02 → 闭合 | worktree_path=D:/autostack/.wt/lang-791/auto-lang | branch=plan-791-dev | base_commit=e4ac72a62 | code_commit=b54cc4a2a（新冻结，worktree clean） | dependency_revisions=auto-down@c39ac7f不变 | task_ids=T-01/T-05/T-06重开后闭合（T-02/03/04保留） | evidence=F-01：**9c4780c54 detached replay worktree（旧实现）**载入新v02+fixtures，UPDATE_GOLDENS=1由旧码bless use_line_filtered金样（"rendered body line"无尾换行、use行滤除），旧码复跑14/14绿后wt-guard clean移除；三条新冻结case=非空lib_files定义不可见（断言Undefined variable: greet+零文件）、模板use行滤除+非空lib_paths惰性（金样）、缺库文件=顶层TemplateLoadError+哨兵字节不变+零新文件。F-02：fmt两crate --check=0；Cargo.toml EOF空行删；金样机械从common/mod.rs迁入v02文件内（仅v02编译）6条dead_code全消；空金样树改.commit化.empty-tree标记（fresh-checkout正确性，autofile/autocode/missing_template/lib_files_invisible四case）。规范措辞：legacy模板执行留adapter解释器、仅guard feed单一定义在core。定向终态（新码，无rebless）：v02=14+v03=8+core=12+auto-gen lib=11+doc=5=50绿；consumer=4+闭包PASS；本crate触面零新警告。R1指出的work收据"16lib"计数错误确认（实为11），本轮已按实测数报告。 | blockers=无 | next_responsible_role=review（R2）

invariant闭包：F-01原例=缺库文件顶层Err；同因臂=非空lib_files成功加载但定义不可达（报告内错）、use行滤除（渲染层）、lib_paths惰性（配置层）——四态全部有断言；旧码replay证明期望非新码再造。F-02原例=6警告+fmt+EOF；同因臂=common中仅v02消费的helper编译进v03（拆分修）、空金样目录git不可跟踪（.empty-tree标记修，超出原例的同类隐患）。

2026-10-16 stage=review | review_scope=final | reviewer_identity=本聊天 Codex（独立于实施交接上下文，未另派agent；模型/session ID未暴露） | plan_id=PLAN-791 | plan_revision=1 | outcome=needs_fix | worktree_path=D:/autostack/.wt/lang-791/auto-lang | branch=plan-791-dev | reviewed_commit=0cd3e3e9f7e448fc95a1ef61b13761ac02d7e65e | base_commit=e4ac72a6265c85cdadeedbabc7a0c6fd93948e48 | dependency_revisions=auto-down@c39ac7f9dacdc769fc4a5ad3b98ec896240b28e8（其余锁/规范指纹见报告） | acceptance_results=AC-01 pass, AC-02 partial, AC-03 pass（仅承诺的render/guard写前边界，不扩展事务保证）, AC-04 pass, AC-05 fail, AC-06 pending | findings=F-791-R1-01（库兼容/加载失败oracle缺失）、F-791-R1-02（新增6 warnings与fmt/diff-check失败） | evidence=[R1完整报告](attachments/791/review-r1/review.md) | pending_closeout_gates=merge-owned裸cargo t+core/adapter定向全测+规范/ledger/归档/guard清理，均未运行/完成 | next=work在原树补齐两项、提交后重审；T-01/T-05/T-06重开，current_step=3/8。历史work完成声明不代表当前复审通过；已通过测试和迁移证据保留。复审未修改实现、未merge/归档。

2026-10-16 stage=new | plan_revision=1 | outcome=pass（草案preflight） | next=合同确认后work | 0/8。当前可执行模板依赖被明确保留在adapter；未声称Man core解耦完成。

2026-10-16 stage=work进入 | 用户指令"/auto-plan:work 791"=实施授权；worktree D:/autostack/.wt/lang-791/auto-lang branch plan-791-dev（计划§6拟codex/plan-791-dev，按仓规约AGENTS.md改为plan-791-dev，语义无差）；base=e4ac72a62（crates/与取材基线3e65540cb零diff）。

T-05[✅] gen-core-consumer外部探针（独立workspace+根exclude，仅依赖auto-gen-core）：4测绿（fake真落盘/guard/dry_run+失败反控/legacy feed字节）；verify-deps.sh闭包断言PASS（core→auto-val+叶子，零auto-lang/auto-man/auto-vm，版本后缀匹配防检出路径误报）。777边界：domains登记auto-gen-core=man-ecosystem，基线21ff040308e8刷新（新增auto-gen→auto-gen-core、auto-gen-core→auto-val合法边，strict=pass，grandfathered 2不变）。V04：生产链CodeGenerator.generate→auto_gen_core::generate真委托；guard.rs/error.rs净删、feed_guarded/capture_block/generate_one/resolve_output_path无第二份；auto-man零改动仍依赖legacy API=未迁登记；check -p auto-gen/-p auto-man/-p auto全零错。
T-06[✅] SD素材：docs/specs/auto-gen-core/project.md（新，含冻结语义5条+红线+诚实债登记）、auto-gen/project.md改写（纠正"简单文本替换"错误描述、adapter+兼容门面定位、单一core定义、未迁收据）；INDEX/ledger沉淀留merge。净删收据：src级 error.rs/guard.rs整文件迁移（0改动逐字）、generator.rs 371→179行净编排删generate_one/resolve_output_path、lib.rs删feed_guarded+capture_block（-104行）。冻结提交0cd3e3e9f（6提交：9c4780c54/b8b439304/6ee5123eb/21ff04030/0d8eafb05/0cd3e3e9f），worktree clean。
T-02[✅]/T-03[✅] crates/auto-gen-core（auto-val+regex+thiserror，零auto-lang）：error/guard逐字迁移+feed_guarded_legacy单一定义（legacy字节语义）；generate.rs编排（命名/guard/dry_run零写入/单模板错收集/批次继续/报告）+TemplateRenderer边界（owned RenderedTemplate）；12单测绿（含预存空文件guard合并加尾换行旧语义钉）。
T-04[✅] CodeGenerator真实委托core（AutoTemplateRenderer=TemplateEngine+共享LoadedData解释器；duration保整调用语义）；error.rs/guard.rs删除、7类型re-export、legacy gen_one调core::feed_guarded_legacy删重复算法；DEBUG eprintln清理；regex/thiserror依赖移除。**冻结V02金样+V03全套零rebless全绿**（11+8+11lib+12core+5doc）；auto-man check零错误。commits: 9c4780c54/b8b439304/6ee5123eb。

2026-10-16 stage=work | plan_id=PLAN-791 | plan_revision=1 | outcome=pass（work-owned 6/6 任务完成，ready-for-review） | worktree_path=D:/autostack/.wt/lang-791/auto-lang | branch=plan-791-dev | base_commit=e4ac72a62 | code_commit=0cd3e3e9f（冻结，worktree clean） | dependency_revisions=auto-down兄弟detached@c39ac7f（组内解析序，只读） | task_ids=T-01..T-06 | evidence=冻结V02金样11+V03失败语义8零rebless全绿；core单测12；consumer探针4+闭包断言PASS；module-boundaries strict=pass（基线21ff040308e8）；cargo test -p auto-gen全绿（16lib+11+8+5doc）；check -p auto-gen/auto-man/auto零错零新警告；clippy两crate零警告 | blockers=无 | next_responsible_role=review（T-07）

readiness：AC-01✓（V01+闭包）、AC-02✓（V02金样+V03，含四项冻结怪癖原样保留）、AC-03✓（失败反控+旧实现净删+core零解释器类型）、AC-04✓（生产默认真委托+未迁清单+source映射=specs两卡+基线边）。review/merge-owned未标记完成：T-07独立复审、T-08裸cargo t+本crate/adapter定向门+沉淀/归档/guard清理。债候选移交review沉淀KNOWN-DEBT（该文件当前有他session未提交登记，避撞）：①AutoFile/AutoCode数据源与lib_files作用域错配（仅Atom可渲染，CLI路径预存红，已spec登记）；②autogen_tests.md Test4期望陈旧已对齐（档外红自f955fa533）；③fstr_note共享解释器路径无效（spec登记）。

T-01[✅] 消费图：auto-gen消费方=auto-man/exporter/{iar,ghs}.rs+builder/ninja/builder.rs（均legacy deprecated AutoGen/Mold）；asset.rs是自有AutoInterpreter不经auto-gen；CLI走CodeGenerator。阶段边界：顶层Err=data/lib文件加载失败；report内单模板错=TemplateLoad/TemplateSyntax/Io，批次继续。**冻结怪癖（旧臂，提取不得改变）**：①AutoFile/AutoCode数据与lib_files定义经eval后对模板不可见（作用域错配，仅DataSource::Atom经merge_atom可渲染——CLI AutoFile路径今天全红）；②fstr_note在共享解释器渲染路径无效（DataLoader解释器恒'$'）；③无guard输出丢尾换行（filter join），guard合并输出保留尾换行；④legacy feed用join("\n")无尾换行、mold名仅剥内嵌".at."。V02=11测+goldens、V03=8测在旧码上全绿（commit 3734a3847前序），goldens由旧码生成未rebless。附带：autogen_tests.md Test4期望陈旧（解释器bool现渲染true非1；f955fa533迁移起档外红，cargo t/tf只跑-p auto-lang不可见）已对齐=债候选。全套cargo test -p auto-gen 16+11+8+5doc 绿。

## 10. 待澄清事项

T01据实际CodeGenerator公开类型确定兼容re-export/错误映射，不要求提前决定插件ABI。若renderer生命周期迫使core持AutoInterpreter/AST或删除旧能力，先给具名依赖环修订，不以optional feature藏回向依赖。

## 11. spec-sync 回写记录（v1 惯例）

- 2026-10-16 PLAN-791 归档回写：SD-01 新增 docs/specs/auto-gen-core/project.md；SD-02 改写 docs/specs/auto-gen/project.md；auto-gen/plans.md 与 auto-gen-core/plans.md 各加一行；docs/specs/overview.md 工具链表 auto-gen 行改述+auto-gen-core 新行；docs/specs/goals.md GOAL-003/GOAL-016 加 791 引用；scripts/spec-index.py GROUPS 登记+INDEX.md 再生；docs/specs/plans.md 双索引加 791 行；ledger v33 P791-D1/D2/T/R3/M；合并报告 docs/plans/reports/791-generation-core-merge.md。
