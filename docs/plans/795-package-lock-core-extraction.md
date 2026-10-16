---
plan_id: PLAN-795
status: execution_done
feature_name: 包锁模型与校验核心抽取
author: [Codex]
created_at: 2026-10-16
updated_at: 2026-10-16
plan_revision: 1
current_step: 6
total_steps: 8
supersedes_spec_components: [docs/specs/auto-man/project.md]
new_spec_components: [docs/specs/package-lock/project.md]
touched_goals: [GOAL-003, GOAL-016]
affects: [auto-man, package-lock]
depends_on: [PLAN-786, PLAN-791]
---

# [PLAN-795] 包锁模型与校验核心抽取

## 0. 变更摘要

新建auto-package-lock，将pac.lock数据编解码、排序/查找/验证规则从携带语言执行/构建的Target中解耦。auto-man保留Target→轻投影、Git观测及兼容门面，生产锁路径真实调用新核心。此pac.lock与792的Cargo.lock为不同合同。

## 1. 目标

无VM/编译器/renderer的独立consumer能读写验证现有pac.lock；原LockFile/LockEntry公开构造与from_targets/verify等能力、TOML/JSON格式及错误阶段保持。非目标：重写pac.at、升级锁格式、下载/物化依赖、改变版本解析/构建器或假称整个Man core已独立。

## 2. 架构方案

auto-man → auto-package-lock → serde/toml/轻值及标准库；core不依赖Target、auto-lang/auto-gen/auto-man。输入为owned依赖投影及必要路径观测，Git命令与TargetKind筛选保留adapter。核心掌握单一序列化/校验算法；legacy门面若需保留原public字段/方法，可薄DTO转换，不复制规则，也不在外包类型上写非法inherent impl。

## 3. 技术栈

Rust新crate、现有serde/toml/JSON、Target和Git观察adapter、临时普通目录/小Git仓fixture与独立Cargo consumer；不建junction。

## 4. 需求分析与背景调查

取材 auto-lang@`9c773601abfdb25b80c08d06922d0ad50edc653a`；[本批输入指纹](attachments/parallel-extraction-20261016/inputs.json) 记录现行 Specs 与实际源码 SHA256。overview仅作导航，以模块Spec和真实代码为准。789/790/791均归档8/8；792为独立构建发行线，当前无交付假设。
用户要求“最好能有几个并行的计划”，本轮按该授权创建可执行合同和编排文档；未授权在本轮实施、创建agent/工作树、发布或修改旧仓refs。确认合同后各自work；本计划不依赖792完成，正式搬仓仍须满足整条路线退出条件。
787/788在途实验线不设为全局前置；实施者必须按实际diff重核类型/manifest共享面。全部实现/测试在本计划唯一组树，主检出只写计划簿记。
lock.rs导入Target；Target又使用transpile_c/Atom及构建/目录/Git逻辑。锁模型当前为LockEntry{name,version,url,commit,path}和LockFile{automan_version,dependencies}。from_targets只选Dep/Device并按name排序；verify目前检查缺项/版本及无commit本地路径存在性，不应借抽取宣传成完整Git内容或供给链验证。
load/save按.json扩展名选择JSON，否则TOML；get/verify取首次同名条目，from_target使用既有Git观测策略。new的automan版本来自auto-man，不得错误改成新core包版本。
主写lock.rs/Target轻adapter/新core，不改792的rust_ui生成或Cargo.lock解析、不改793诊断/794 native。共享根members、auto-man依赖声明与777规则短窗口串行，源实现可并行。

## 5. 详细设计

T01冻结现有公开API/格式/路径和Git观测行为，选择精确轻投影及兼容门面。旧LockFile/LockEntry字段构造、new/from_targets/load/save/verify/get/from_target保持；不要求旧类型与新core DTO强行成为同一Rust类型，但旧消费者不能被静默破坏。
Target→DTO和Git probing留上层；core接收调用方提供的automan版本，不依赖crate::AUTOMAN_VERSION。校验所需路径存在性可由标准库或显式observer承担，不能假装空path成功等于验证完整依赖来源。保持旧目标筛选/排序、重复项处理、错误聚合顺序及serde格式，不顺手硬化/放宽历史协议。
锁load解析错误/IO错误按旧阶段保留，save序列化失败不得先写新数据；现有写盘语义保持，不追加未批准的全局原子事务承诺。复用兼容算法必须真实接生产路径，不能留下两套LockFile::verify或序列化实现。未迁resolver/Target/build/exporter/可执行模板单独列清。

### 规范增量

| ID | 操作 | target | before/after rule | 原因 | AC |
|---|---|---|---|---|---|
| SD-01 | add | docs/specs/package-lock/project.md | pac.lock轻投影/格式/校验与边界，诚实限制 | 无语言核心可消费 | AC-01/02/03 |
| SD-02 | modify | docs/specs/auto-man/project.md | Target/Git adapter和锁core生产分工、旧API兼容 | 真实删边与剩余范围 | AC-02/04 |

## 6. 测试设计

实施者未另指派，独立final review与merge分别交接，不新建或指定额外模型。下表V用例由work实现并执行，不是本轮通过证据。开发及needs_fix只做定向验收与必要cargo check；最终代码提交、final review通过后，由merge在冻结提交跑裸cargo t及实际触面档。tf仅主检出批量单实例，不随各计划并发；无Schema/生成器改动不跑docs_gen。

| Case / owner | AC | 入口/输入 | observable oracle / 边界 |
|---|---|---|---|
| V01 / work | 01 | 新test/fixtures/modularity/package-lock-consumer，fresh manifest读写/验证 | 可独立构建运行，normal/build闭包无auto-lang/Man/gen/renderer，不复制rlib |
| V02 / work | 02 | 冻结旧TOML/JSON、空锁、Unicode/Windows路径、Dep/Device与其它kind、同名项/版本等原支持形态 | 字段/顺序/serde和错误规范投影同旧；new版本仍为automan_version而非core版本 |
| V03 / work | 02/03 | 缺项/版本错/本地路径缺失、坏格式/IO失败；真实普通目录和临时Git依赖 | 原判红阶段与聚合顺序一致；正控复绿，无source/输出副作用；不把路径存在当完整commit校验 |
| V04 / work | 03/04 | auto-man生产LockFile旧API/from_targets→save→load→verify，与core直接输入比较 | 旧消费者代码可编译；生产真实委托、算法单份，Git/Target不进入底层 |

拟树D:/autostack/.wt/lang-795/auto-lang，branch codex/plan-795-dev。定向新crate/auto-man锁测试及check；merge-owned裸t+两crate定向验收，不因“锁”字与792共享Cargo发行档。tf批量单实例；795%5=0落地后按现行规则移交regress。

## 7. 验收标准

| ID | owner_stage | 可观察结果 |
|---|---|---|
| AC-01 | work | 新锁核心无语言/Target依赖，独立consumer V01通过 |
| AC-02 | work | 旧API、格式、automan版本及校验错误保持，V02/03/04通过 |
| AC-03 | work | 真实单一规则实现、无假验证/失败覆盖，V03/04通过 |
| AC-04 | work | 生产锁流程委托，Target/Git adapter及未迁Man清单有来源/净删账 |
| AC-05 | review | 独立fresh consumer/旧API/负控及SD审查通过 |
| AC-06 | merge | 冻结门禁、规范/ledger/归档/guard清理及批到期交接 |

## 8. 执行步骤

| 状态 | ID | owner_stage | 依赖 | 操作/验证 | AC |
|---|---|---|---|---|---|
| [x] | T-01 | work | 786/791归档、合同确认 | 冻结lock.rs/Target公开API、serde/错误旧臂、Git与path行为，确定轻投影和版本来源 | 02/04 |
| [x] | T-02 | work | T01 | 新crates/auto-package-lock，模型/codec和无Target输入；cargo check新crate | 01/02 |
| [x] | T-03 | work | T02 | 迁排序/查询/verify单一规则，observer/路径负控；保留原错误阶段 | 02/03 |
| [x] | T-04 | work | T03 | auto-man lock.rs兼容委托及Target/Git adapter；旧API构造/方法真实编译 | 02/04 |
| [x] | T-05 | work | T04 | 外部consumer及V01..04、metadata/777边界；检查792写面未侵入（R1 修复轮：777 基线已在原树重锚，strict pass/0 violations，见 9 节修复记录） | 01–04 |
| [x] | T-06 | work | T05 | 净删/搬迁/未迁清单和SD素材，提交冻结代码与并行交接收据（R1 修复轮：基线重锚提交 edf982e52，冻结 HEAD 前移） | 01–04 |
| [ ] | T-07 | review | T06 | 独立真实锁流程/负控/公开兼容和SD冻结 | 05 |
| [ ] | T-08 | merge | final pass | 裸t+定向门、沉淀/归档/guard、regress到期交接；必要时792刷新source闭包 | 06 |

### 任务证据（work 轮）

- **T-01 冻结**：临时探针测试在 579f6dec0 旧臂捕获 golden（探针即删）：TOML pretty（`automan_version` 先、`[[dependencies]]` 块、反斜杠 path 字面量单引号形态 `'deps\zeta-lib'`、尾换行）、JSON pretty（2 空格、无尾换行、空锁 `[]`）、错误文本（读失败带 AutoPath 统一化路径 `Z:/...`、JSON/TOML 解析阶段前缀）、`G-PATHSTR` 证实 `deps\sub\pac.lock.json`→统一化 `deps/sub/pac.lock.json` 选 JSON、verify 混入 App kind 目标被过滤（单错误臂输出）。轻投影定型：`LockEntry` 五字段 owned String 投影 + `ResolvedDependency{name,version}`；版本来源=调用方（auto-man 传 `AUTOMAN_VERSION`=0.1.0，core 永不代填）。
- **T-02/T-03 核心 Crate**：`crates/auto-package-lock`（model 112/verify 58/codec 54 生产行）依赖仅 auto-val+serde/serde_json/toml+std；`LockedEntryView` 四字段只读视图 + `find_entry`/`sort_dependencies`/`verify_dependencies` 单一规则；codec 保持 AutoPath 统一化路径语义与 save 先序列化后建目录再写盘次序。`cargo check`/`clippy --all-targets` 零警告。
- **T-04 门面**：旧公开 `LockFile`/`LockEntry` pub 字段构造与 `new/from_targets/load/save/verify/get/from_target` 全保持（serde derives 保留）；规则零复制——`get`/`verify` 经 `LockedEntryView` 零 DTO 委托，`load`/`save`/排序经 core；Target 投影（Dep/Device 过滤）与 `get_git_commit`（`git rev-parse HEAD`）留 adapter。
- **T-05 验证**：core 单测 14 + 合同 11（V02/V03：格式冻结/空锁/Unicode/Windows 路径/统一化选 JSON/大小写敏感/错误阶段/路径正负控（真目录过、文件与缺失红、有 commit 跳过）/聚合并序/重名首条/save 建目录失败反控不覆写）、门面 9（V04：golden TOML/JSON 相等、from_targets kind 过滤+排序+未物化复绿、真 Git 临时仓 commit==HEAD、委托错误文本逐字）、consumer 4+闭包 PASS（V01：闭包=package-lock-consumer→auto-package-lock→auto-val，无 auto-lang/man/gen/ui/vm）。777 strict（**R1 纠正**：首轮回执误记"唯一红=config_drift、重锚归 merge"——独立实跑实为两项红 `config_drift_since_baseline`+`stale_baseline`，且 work-owned 边界闸门不得留 merge；已在 R1 修复轮于原树重锚闭合，见 9 节）：新边 auto-man→auto-package-lock、auto-package-lock→auto-val 均 allowed 且已入基线。792 写面零侵入（diff 触面：module-boundaries/根 members+exclude/auto-man Cargo+lock.rs/新 crate/新 fixture/specs 两件）。**预存红归因**：`rust_ui::tests::review738_r5_lock_binding_state_machine`/`review751_empty_lock_identity_matrix` 在基线 579f6dec0 裸复现（stash 验证，非本计划引入；792 域）。
- **T-06 净删/搬迁/未迁清单**：旧 lock.rs 253 行（生产 173）→ 门面 179 生产行（投影/委托/Git 观测）+ 核心 224 生产行（模型/规则/codec 单一定义，含视图 trait 与文档）；规则实现单份（无第二套 verify/序列化）。**未迁**（仍携语言栈留 auto-man）：resolver、Target 全家、builder、exporter、可执行模板、git/index/pull、AUTOMAN_VERSION 戳。SD 素材：`docs/specs/package-lock/project.md` 新增 + `docs/specs/auto-man/project.md` 分工节与模块行改写；INDEX/specs.json/plans.md 双索引归 merge。冻结代码提交=11ddc162e。

## 9. 复审记录

2026-10-16 stage=work | PLAN-795 | plan_revision=1 | outcome=pass（R1 修复轮：F-795-R1-01 闭合，T-05/T-06 重闭，6/8） | worktree=D:/autostack/.wt/lang-795/auto-lang | branch=plan-795-dev | base_commit=579f6dec0 | code_commit=edf982e52（=11ddc162e 实现 + 基线重锚提交） | dependency_revisions=与 R1 受审一致（auto-down@c39ac7f9；审计内 cargo metadata --locked 45 声明边零差异复核通过，实现/依赖未变） | task_ids=T-05、T-06（重闭；T-01..T-04 证据保留） | finding_closure=F-795-R1-01：finding=777 strict 两项红（config_drift+stale，基线未绑定四锚定输入）→ invariant=work-owned metadata/777 闸门须在送审冻结 HEAD 上 strict=pass → affected=仅 .autoos/module-boundaries-baseline.json → fix=原树（干净受管）`--refresh-baseline`，差异审查=base_revision 3e1a9742c→11ddc162e、config_sha256 重绑、+2 条 member 边（man→auto-package-lock、auto-package-lock→auto-val）、规则/边零删改、grandfather 仍 2 条 R5 边不变 → results=提交 edf982e52 后冻结 HEAD 复跑 strict **pass/0 violations**，grandfathered=2 不变，new_allowed=[]（新边已入基线） → remaining=无 | evidence_reuse=按 R1 裁定"只改基线、依赖/配置经审计复核不变"复用 R1 行为证据（core25+门面9+consumer4=38 pass、fresh 探针 4 组、闭包、fmt），未重跑行为面 | blockers=无 | merge-owned pending（不变）：裸 cargo t + 双 crate/consumer 定向验收 + fmt 复核 + specs.json/INDEX/plans.md 沉淀 + 归档/guard 清理 + tf 到期交接（795%5==0）；777 基线已不再是 pending 项 | next_responsible_role=review（T-07 重审，建议聚焦 F-795-R1-01 复验与基线 diff 审查）

2026-10-16 stage=review | review_scope=final | reviewer_identity=本聊天 Codex（实施以外交接上下文，未派 agent，具体模型/session ID未暴露） | plan_id=PLAN-795 | plan_revision=1 | outcome=needs_fix | worktree_path=D:/autostack/.wt/lang-795/auto-lang | branch=plan-795-dev | reviewed_commit=11ddc162ecd839b98cee748e3d2c27cfd05df301 | base_commit=579f6dec0941fb711664679a9cb8c29fa71bad20 | dependency_revisions=auto-down@c39ac7f9dacdc769fc4a5ad3b98ec896240b28e8；其余依赖/锁/规范 SHA256 见 [fingerprints.json](attachments/795/review-r1/fingerprints.json) | spec_inputs=canonical auto-man project.md f12b0b4098450025b918847e7e2dfc29b07619a16574bc341b5e62a3f9ad8ba8；冻结 SD-01/02 [spec-delta.diff](attachments/795/review-r1/spec-delta.diff) | acceptance_results=AC-01 pass、AC-02 pass、AC-03 pass、AC-04 partial、AC-05 fail、AC-06 pending | findings=F-795-R1-01(P2，T-05/06)：777 strict 独立实跑 exit1，两项违例 config_drift_since_baseline + stale_baseline；新增两依赖边 allowed、metadata45边零差异，但基线未绑定本次四个锚定输入；不得留 merge 补 work-owned 边界验收。791 R2→R3 同类缺口亦要求先修复提交 | evidence=[R1完整报告](attachments/795/review-r1/review.md)；core25+门面9+consumer4=38 pass，fresh独立探针4组pass，依赖闭包PASS，变更Rust文件fmt及diff-check通过；入场/退场HEAD一致且实现树clean | pending_closeout_gates=merge-owned裸cargo t+两crate/consumer定向验收+fmt复核、规范/ledger/索引、归档/guard清理、795%5=0批回归交接；全部未执行整档/收尾 | next=work在原树核对并显式刷新777基线、不放宽规则与豁免，提交后strict复绿及受影响复审；T-05/T-06重开，current_step=4/8。历史work完成声明保留为历史，本轮未修改实现或合入。

2026-10-16 stage=new | plan_revision=1 | outcome=pass（合同preflight） | next=合同确认后work | 0/8；本轮未运行Rust或探测网络Git。

2026-10-16 stage=work | PLAN-795 | plan_revision=1 | 入场执行 | worktree=D:/autostack/.wt/lang-795/auto-lang | branch=plan-795-dev（repo 命名规约覆盖合同拟称 codex/plan-795-dev；PLAN-726 组布局含 auto-down 兄弟 detached@c39ac7f） | base_commit=579f6dec0 | 前置验证：786/791 已归档；inputs.json 指纹复核 lock.rs=407127b5/target.rs=a483fa96/auto-man project.md=f12b0b40/module-boundaries=82718ad2 四项全等 | 用户显式授权 /auto-plan:work 实施。

2026-10-16 stage=work | PLAN-795 | plan_revision=1 | outcome=pass（work-owned T-01..T-06 全闭合，6/8） | worktree=D:/autostack/.wt/lang-795/auto-lang | branch=plan-795-dev | base_commit=579f6dec0 | code_commit=11ddc162e | dependency_revisions=auto-val@579f6dec0(主仓同基)、serde/serde_json/toml 走 workspace 版本 | task_ids=T-01..T-06 | evidence=AC-01: consumer 4/4+闭包 PASS（verify-deps.sh，闭包仅 auto-val）；AC-02: golden TOML/JSON 三面钉死（core/门面/consumer），旧 API 编译+行为保持（门面 9/9）；AC-03: 单一规则实现（sort/find/verify/codec 各一份于 core，门面零复制），错误阶段/文本/聚合并序冻结，路径存在≠commit 校验（有 commit 跳过路径检查负控）；AC-04: 生产路径真实委托（from_targets/load/save/verify/get 全走 core），净删/搬迁/未迁清单见任务证据节 | blockers=无 | merge-owned pending：裸 cargo t + 双 crate 定向验收（cargo test -p auto-package-lock 全档 + cargo test -p auto-man --lib lock:: + consumer manifest 门禁 + verify-deps.sh）+ cargo fmt --check（work 已过单文件，merge 复核）+ 777 基线重锚（--refresh-baseline，干净树）+ specs.json/INDEX/plans.md 沉淀 + 归档/guard 清理 + tf 到期交接（795%5==0，落地触发） | next_responsible_role=review（T-07）

#### work 轮附记

- 环境注记：sccache 默认端口 4226 落 Windows 保留段（4144-4243）启动失败；本会话改 SCCACHE_SERVER_PORT=52117 绕过（保留段清单 netsh 复核）。不影响仓内产物。
- 预存红（非本计划）：auto-man `rust_ui::tests::review738_r5_lock_binding_state_machine`、`review751_empty_lock_identity_matrix` 在基线 579f6dec0 stash 裸复现红（792 域）；"lock" 字面过滤器会误触达它们，定向门禁须用 `lock::` 前缀过滤。
- 触面档判定：diff 未触 auto-lang crates/、trans、book、ui_gen、aavm——tv/tt/tb/tu/taa 均零触发；merge 门=裸 t+上述双 crate 定向。

## 10. 待澄清事项

T01若旧公开构造与拆包类型规则冲突，采用保持旧API的薄DTO门面并明确转换；不得直接删方法/改字段。发现旧锁校验弱点只登记后续行为修复，不以抽取名义改格式或宣布安全升级。
