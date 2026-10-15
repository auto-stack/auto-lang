---
plan_id: PLAN-786
status: archived
feature_name: AutoMan 展示字段读取收口与显式兼容适配
author: [Codex]
created_at: 2026-10-15
updated_at: 2026-10-16
plan_revision: 1
current_step: 7
total_steps: 7
supersedes_spec_components: [docs/specs/auto-man/project.md, docs/specs/project-model/project.md]
new_spec_components: []
touched_goals: []
affects: [auto-man, project-model]
depends_on: [PLAN-784, PLAN-780]
---

# [PLAN-786] AutoMan 展示字段读取收口与显式兼容适配

## 0. 变更摘要

把780模型从探针用于AutoMan内部一个有限生产面：name/title/title_zh读取与locale选择。统一只读入口与必要兼容策略，删除vue.rs内两个重复行循环；保留现有公开命令及显示结果。不是全面切换pac解析，也不宣称man已不依赖lang。

## 1. 目标

已冻结静态子集由ProjectDescriptor供展示字段；原先接受但模型不支持/归一不同的文本，通过具名兼容策略保留现有结果，不静默变空、不执行VM。原调用者保留locale、目录回退等职责。
非目标：shadcn/default_classes、npm_deps、exe_name、targets、dep/member声明门、registry/loader、pac.lock、全量Pac/VM adapter、跨仓/CLI路由。去重不改这些未选字段的实现。

## 2. 架构方案

新增auto-man内部只读模块（拟 `src/project_display_reader.rs`）；依赖现有auto-project-model，不增加auto-lang→man边。一次解析产出展示读取结果与内部路径分类（静态/具名legacy兼容/无值或坏输入原因），多个字段共享；不建另一套通用pac parser。
旧形态必需的文本兼容实现只保留一处，按key及空值策略读取，替代parse_pac_scalar/parse_pac_name的重复循环。生产不同时运行两臂做逐次判等选路；legacy对拍oracle仅用于测试冻结。

## 3. 技术栈

Rust；auto-project-model::parse_document、ProjectDescriptor.identity/display/provenance/duplicates/unsupported；auto-man现有locale选择及调用者。所有解析内存只读，无文件写入/网络/VM求值。

## 4. 需求分析与背景调查

依据project-model/project.md、auto-man/project.md的四名称/locale规则、Design37§5及780 T04 owner map。源码/Spec指纹见 [取材记录](attachments/modularization-planning-20261015/inputs.md)。
实锚vue.rs::parse_pac_scalar/parse_pac_name/parse_pac_title/parse_pac_display_title/parse_pac_display_title_in；model lib.rs::parse_document，from_scan.rs冻结归一。现有行扫描取首匹配，允许部分单引号/非结构化形态；parse_pac_name空串可Some，标题函数忽略空串。模型存在last-wins、trim、完整语法分类等区别，不能用正常样例绿把它们视为等价。rust_ui.rs::parse_pac_exe_name及toggle等不在本轮修改。
用户仅授权本轮建计划，实施待确认。仅auto-lang仓auto-man模块改动，不改旧仓或783实验。784完成、780已合流后可实施；不依赖785/781全结束，因为不改parser/VM/UI renderer。若执行时出现新同文件写入，T01记录热点并等释放，不靠并行覆盖。

## 5. 详细设计

T01冻结输入类别与旧输出；T02据此登记有限的model准入条件/legacy兼容理由和触发规则，附实际源码依据。静态无歧义的name/title/title_zh进入DTO路径；模型限制、重复键优先差异、空串/空白差异、旧引用形态等由明确兼容策略处理，不丢行为。
不能为方便把整个文件的unsupported误说成展示字段不存在；某些字段仍可识别时按已冻结策略处理。既有非法文本上旧扫描返回值/None的行为，同样在兼容矩阵中记录，未获同意不改为新错误或默认值。
内部分类须在单测/收据可观察；不新增用户stdout、warning噪声或全局env分支，不用隐式catch-all unwrap_or_default伪装成功。若无法用有限规则保持兼容且不复制parser，T02提交具名决策/最小修订，后续实施停在该项；不能擅自把重复键或空值语义改成Pac规范来通过验收。
locale继续由现有parse_pac_display_title[_in]决定，zh前缀优先title_zh否则title；name/目录兜底保留在原调用者。不得引入env缓存或并行set_var测试；验证用显式prefers_zh入口。
T03真实调用点接入，删除原两个独立行循环；兼容扫描只有一处，附有限使用理由。不要求删除本计划明确保留的所有其他文本副本。T04永久选定字段对拍与变异输入，工作完成前记录真实减少的重复实现、仍存兼容分支和后续退出条件，不把搬行数算净删除。
新增fixtures拟 `test/fixtures/modularity/project-model/display-reader/`，测试基线来自冻结的旧纯函数与既有调用结果；旧oracle只留测试，不在生产维持双套读取。784的清单须把本次兼容adapter登记到未来Man小包迁移，不永久绑定语言执行类型。

### 规范增量

| delta_id | 操作 | target | before/after rule | 原因 | AC |
|---|---|---|---|---|---|
| SD-01 | modify | docs/specs/auto-man/project.md | 三个选定字段统一只读入口，locale/回退不变；其他文本读取未迁 | 首批生产去重，守原行为 | AC01/02/03 |
| SD-02 | modify | docs/specs/project-model/project.md | 增加实际首个生产消费者及精确支持/兼容边界，撤销对应“生产全部未接线”的旧描述 | current-state如实沉淀，不扩大pilot语法 | AC01/02 |

touched_goals为空：内部读取组织与兼容治理，不宣称语言/UI能力目标关闭。

## 6. 测试设计

实施未指派；独立final reviewer在同树查验，拟D:/autostack/.wt/lang-786/auto-lang、branch plan-786-dev。以下V由T01/T04提供可执行样例与runner，不是已执行证据。

| V | work owner | AC | 输入/入口 | 观察oracle与边界 |
|---|---|---|---|---|
| V01 | T01/T04 | 01/02 | 当前受管pac+最小单字段/三字段/标题缺失 | 选定旧函数与真实消费输出逐值相等；不是模型单测自证 |
| V02 | T02/T04 | 02 | 重复name/title、空/空白、嵌套同名、inline注释、单引号、转义、顶层对象 | model/兼容路径理由明确；输出保持冻结旧行为，未支持不假记DTO等价 |
| V03 | T04 | 02/03 | 显式locale zh/zh-CN/en/无中文标题 | title_zh→title规则及原name/目录兜底不变，无env竞争 |
| V04 | T04 | 02 | 动态值/插值、括号缺失、780坏输入/块注释控制 | 不执行VM，不写文件；错误/兼容不伪装静态成功，外部截止保护终止 |
| V05 | T04 | 01/03 | 所有选定调用者与非选字段控制 | 原两扫描循环删除且调用者实际用新入口；toggle/npm/exe/loader路径不偷换 |

开发：cargo test -p auto-project-model --locked；现有 `cargo test -p auto-man --lib pac_title_overrides_document_title --locked` 与 `cargo test -p auto-man --lib pac_display_title_locale_chain --locked`，加新reader族（T01登记新增精确filter）；cargo check -p auto-lang -p auto-man及触面fmt；复跑780定向corpus/consumer。禁止为本局部修复反复整档。final review通过后merge冻结提交裸cargo t；若实际改ui_gen才补tu，改book才tb，不触VM不tv，不触aavm不taa。若扩大改动先修订门禁映射，不用全档代替字段oracle。

## 7. 验收标准

| ID | owner_stage | 可观察结果 |
|---|---|---|
| AC01 | work | 选定真实调用接统一入口，DTO路径实际被使用；原重复循环已删除、必要compat唯一，V01/05 |
| AC02 | work | 全冻结输入及边界输出保持；路径理由可审计，动态不执行/误标静态，V01/02/04 |
| AC03 | work | locale/回退与未选字段保持，依赖无新增反向边，V03/05 |
| AC04 | review | 独立字段/变异/调用覆盖与去重审查，绑定提交与SD |
| AC05 | merge | 冻结门禁、SD/ledger、合流归档/批到期判定/guard-clean完成 |

## 8. 执行步骤

| 状态 | ID | owner_stage | 依赖 | 操作/产出及验证 | AC |
|---|---|---|---|---|---|
| [x] | T-01 | work | 784+780合流/合同确认 | 盘点真实调用者、旧输入输出oracle与精确测试filter；冻结V01/02/03表 | 01/02/03 |
| [x] | T-02 | work | T01 | 新project_display_reader只读入口，DTO/显式compat准入决策；不允许无界重写parser | 01/02 |
| [x] | T-03 | work | T02 | vue.rs三字段与locale读取接线、两个重复扫描循环收口；其他字段不扩界 | 01/03 |
| [x] | T-04 | work | T03 | V01..05永久对拍、变异/截止控制，780 scoped回归、check/fmt | 01/02/03 |
| [x] | T-05 | work | T04 | 提交实现+真实去重/兼容退出清单+SD素材，确认work验收完整 | 01/02/03 |
| [x] | T-06 | review | T05 | 独立逐AC、未授权语义差异与伪DTO路径扫描，SD冻结 | 04 |
| [x] | T-07 | merge | review pass | 冻结t门禁、知识沉淀、合流归档与guard-clean | 05 |

## 9. 复审记录

2026-10-15 stage=new | plan_revision=1 | outcome=pass（草案preflight） | next=784完成及确认后work，可与785不重叠面并行 | 0/7，无源码实现/测试/树。已将first-vs-last、空串与unsupported的差异列为必验而非默许行为变更；T02有界决策不允许静默缩合同。

2026-10-15 stage=work | plan_revision=1 | T-01 完成证据：依赖核验（784/780 均已归档）；调用者盘点+冻结旧输入输出 oracle（31 类含 first-vs-last/空值语义差异/嵌套与块注释藏行/冒号前空格/nil-astr 差异）+ DTO 准入与具名兼容矩阵 + 测试 filter 登记 + V01/02/03 冻结表 → [t01-frozen-oracle.md](attachments/786/t01-frozen-oracle.md)。worktree `D:/autostack/.wt/lang-786/auto-lang`，branch `plan-786-dev`，base `a1d5ed311`（主检出 tip；与 d0a454422 间仅 docs/plans 差异）。1/7。

2026-10-15 stage=work | plan_revision=1 | outcome=pass（work 验收完整，execution_done）| worktree=`D:/autostack/.wt/lang-786/auto-lang` | branch=`plan-786-dev` | base=`a1d5ed311` | code=`bde9d0523` | deps: auto-project-model@a1d5ed311（未改）；组内 auto-down 兄弟 detached@c39ac7f（仅 workspace 解析需要，非本计划依赖改动） | task_ids=T-01..T-05 | 证据见下方 T-02..T-05 行与 §9.1 | blockers=无 | next=review（T-06）

- T-02 证据：`crates/auto-man/src/project_display_reader.rs`（准入矩阵实现；兼容臂唯一冻结副本 legacy_scan，skip_empty 参数区分 title 族跳空续扫/name 首匹配两族空值语义；无 auto-lang 依赖=不执行 VM 的结构性证明）。
- T-03 证据：vue.rs diff −50/+36（parse_pac_scalar/parse_pac_name 两循环删除；生产调用点 `project_display_reader::read(&pac_content)` 一次读取；locale 链与 env 读取保留 vue.rs；shadcn/default_classes/parse_workspace_path/rust_ui/vscode/ark/jet 副本不动，由 v05 源码断言看守）。
- T-04 证据（命令=结果）：`cargo test -p auto-man --lib display_reader` 6/6 绿（V01 受管语料 170 文件 510 字段读取与冻结旧 oracle 逐值相等，model=496/compat=14；V02 31 类矩阵+8 fixtures 期望与路径分类冻结；V03 显式 prefers_zh；V04 780 dynamic/bad 7 文件+插值/包裹分类冻结+无写入+10k 行有界）；`cargo test -p auto-man --lib pac_title_overrides_document_title` 与 `pac_display_title_locale_chain` 1/1 绿（既有冻结断言不变）；`cargo test -p auto-project-model --locked` 30/30；`cargo test -p auto-man --lib project_model` 11/11（780 adapter 语料对拍不回归）；`cargo check -p auto-lang -p auto-man` 通过、触面文件零新警告；rustfmt 触面文件 clean。vue:: 模块整跑余红 2=预存（plan593 index_css golden / desktop_extra_app_roots，基线 a1d5ed311 stash 复现同红）；plan609_unresolved_dep 首跑红为并行 flake（基线与复跑均绿）。
- T-05 证据：提交 `bde9d0523`（tree clean）；oracle 修正记录：冻结表 #24「引号内尾逗号」实测无分歧（旧归一先剥逗号后剥引号，引号内逗号保留），已改 Model(Value)——期望错误由 oracle 对拍纠正，实现未动。

### 9.1 真实去重与兼容退出清单（T-05）

**真实减少的重复实现**：vue.rs `parse_pac_scalar` + `parse_pac_name` 两个完整行扫描循环删除；该两字段的行扫描算法在仓内从 vue.rs 副本归一为 reader 兼容臂单一冻结副本（算法本身保留是设计内兼容，非未竟去重）。LOC 记账（不把搬迁算净删除）：vue.rs −50/+36，新增 reader 216 行 + 对拍族 436 行 + fixtures 8 文件（净增为准入判定与永久对拍，非重复实现）。

**仍存的兼容分支（具名、有限、可审计）**：`CompatReason::{ParseError, HiddenOrPhantomLine, DuplicateKey, UnsupportedValue, LineFormMismatch}` 五类，语义与触发规则见 T01 冻结表 §3；受管语料触发面 14/510 字段读取。

**退出条件（后续计划承接，非本计划范围）**：(1) 旧形态语料（单引号/裸值/重复键/注释藏行）清退后，兼容臂可随 V02 矩阵同步收缩；(2) ark/jet/rust_ui/vscode/cmd_vue 的 parse_pac_name 独立副本迁往统一入口时，按 784 清单登记的 Man 小包迁移一并处理；(3) project-model 支持对象文档物理行号/包装形态归一后，HiddenOrPhantomLine 类可再细分。784 已登记的未来迁移清单覆盖 (2)。

### 9.2 SD 素材（供 review 冻结）

- SD-01（auto-man/project.md）：`project_display_reader` 统一入口三字段；locale/回退职责留调用者；其余文本读取（toggle 族/front 路径/各模块副本）未迁。
- SD-02（project-model/project.md）：首个生产消费者=auto-man 展示字段 reader；支持边界=静态 Str 字面量+准入谓词，兼容边界=五类具名 CompatReason；撤销「生产全部未接线」的旧描述（780 pilot 措辞）。

## 10. 待澄清事项

无需用户先选内部类型名。T02若证明必须改变展示结果、全面放弃动态兼容或引入VM求值，提交具体反例后修订/裁定，不能自行选择破坏兼容。该阻断只影响本reader计划，不要求781/783停工。

### 9.3 独立 final review R1（2026-10-15）

stage=review | review_scope=final | reviewer_identity=Codex（本聊天独立上下文，未参与实现/未委派） | plan_id=PLAN-786 | plan_revision=1 | outcome=needs_fix | worktree_path=D:/autostack/.wt/lang-786/auto-lang | branch=plan-786-dev | reviewed_commit=bde9d0523198f1a15bc908c62a5b18b59324e61a | base_commit=a1d5ed3114b163fa8a10c3d8985f5ae664bdad2c | dependency_revisions=auto-project-model tree b4a5de79bf608a897944fb794f16201c850e3d29，Cargo.lock SHA256 DAA795C5B9A39810836A5B8770D939B9175D8C506CA595415B57EC22CD96BC00 | spec_inputs=两目标 ec934b0006acb9b9838def82d0fcaf438ae0153f（内容hash见报告） | evidence=[R1完整报告](attachments/786/review-r1/REVIEW.md)、[独立反例](attachments/786/review-r1/counterexamples.rs)、[带10秒外部截止重放](attachments/786/review-r1/replay.ps1)、[780回归原始结果](attachments/786/review-r1/adapter.log) | SD_snapshot=[spec-delta.md](attachments/786/review-r1/spec-delta.md)，SHA256 0C8601B6B810EA5E526595711626721C9ED7C7DBACA28CF2D2EF9C9A8596D6DB（needs_fix，修复后重冻结） | next=同树work修复、提交后复审。

acceptance_results: AC01 pass（接线/去重）；AC02 fail；AC03 pass（locale/依赖/未选实现）；AC04 fail；AC05 pending merge-owned。reader 6/6、独立260组、模型30/30、既有title/locale各1/1、check通过；指定auto-man project_model回归10/11失败。最终提交语料178/534（model515/compat19），工作旧收据170/510未包含提交新增8 fixtures。

findings: F786-R1-01 P1（无关字段512层数组导致read全文解析栈溢出abort，旧oracle返回ok；32/128控制通过，外部截止重放同红）；F786-R1-02 P2（新增single-quoted-title fixture破坏780受管corpus严格身份门，旧VM拒绝不在允许例外内）；F786-R1-03 P3（新增vue兜底/locale行fmt不clean；新增reader两文件clean，区分原有格式漂移）。全部需修复，不因登记债务而延后。具体输入、断言、命令、修复范围见报告。

重开T02/03/04/05及依赖验收，保留T01；current_step=1/7、status=executing。历史工作完成声称保留为历史，不是当前readiness。无实现修复、合流或规范发布。pending_closeout_gates: final review通过后的冻结裸cargo t（merge-owned），SD/ledger/归档/批到期判定/guard-clean；本轮未运行整档。

2026-10-15 stage=work（R1 修复轮）| plan_revision=1 | outcome=pass（三项 findings 全闭，execution_done）| worktree=`D:/autostack/.wt/lang-786/auto-lang` | branch=`plan-786-dev` | base=`a1d5ed311` | code=`db71263b5`（叠加于 `bde9d0523`）| deps 不变（auto-project-model 未改；auto-down@c39ac7f 仅组内解析）| task_ids=T-02..T-05 重闭（T-01 复审保留）| blockers=无 | next=review（R2，绑定 `db71263b5`）

**finding 闭环表**：

| finding | invariant | 受影响路径/状态 | 修复与永久用例 | 当前结果 | 残余 gap |
|---|---|---|---|---|---|
| F786-R1-01（P1 深度 abort） | 非选字段的任意文本不得使展示读取进程级失败；准入/解析须对代表性结构化/坏输入安全 | `[]`/`{}` 族任意深度（数组/对象/节点嵌套同因——递归位点 scan_value_positions/parse_node/AtomParser）；`()` 参数表迭代不受影响 | read() 前置迭代括号深度筛（字符串/转义/行·块注释感知；只数 `{}`/`[]`；阈值 64=实测 128 过/512 abort 之下成对余量），超限整体 `Compat(DeepNesting)` 且不做模型解析；永久回归 `display_reader_v04_deep_nesting_process_isolated`（自执行子进程重放复审反例 512/1024 层 + 10s 外部截止；修复前子进程 0xC00000FD 即红）+ `display_reader_v04_deep_nesting_boundary_and_awareness`（64 Model/65 DeepNesting 边界、节点嵌套 512、字符串/注释括号不计数、oracle 三方一致） | reader 族 8/8 绿（进程隔离探针子进程干净退出 0） | 阈值 64 为包络判定非语法限制（未加未经批准的语法约束）；模型侧递归深度治理留待 project-model 后续计划 |
| F786-R1-02（P2 780 consumer 10/11） | 不同 oracle 的负例集合必须隔离；语料合同身份精确 | 全部 8 个 display-reader fixtures（`.pac.at` 后缀命中 780 walker `git ls-files *.at`+`ends_with("pac.at")`）；单引号形态旧 VM 拒绝而模型归 unsupported，落 template-rejected 例外表外 | fixtures 改名 `.pac.at`→`.at` 脱离受管语料身份；未放宽 780 例外表/未目录豁免 | staged（walker 可见）状态复跑 `project_model` 11/11；V01 语料回到 170 文件（510 读取 model=496/compat=14） | 无 |
| F786-R1-03（P3 触面 fmt） | 本次新增行须 rustfmt clean，检查边界如实 | vue.rs name 兜底链（3311-3315）与 locale 测试断言（10008-10022） | 逐行改为 rustfmt 建议形态；预存漂移（vue.rs 100+ 处、ark.rs 等）不扩大重排 | 触面区域 rustfmt --check 无差异；新增两 reader 文件全文件 clean | 预存全文件漂移不在本计划范围（记录于 KNOWN-DEBT 由复审裁量） |

**R1 验证收据（命令=结果，全在 `db71263b5` 工作树）**：`cargo test -p auto-man --lib display_reader` 8/8（含进程隔离深度回归）；`cargo test -p auto-man --lib project_model` 11/11（staged fixtures 生效）；`cargo test -p auto-man --lib pac_title_overrides_document_title` 1/1、`pac_display_title_locale_chain` 1/1；`cargo check -p auto-lang -p auto-man` exit 0、触面零新警告；rustfmt 触面 clean。merge-owned 门禁（裸 cargo t 等）保持 pending，由 final review 通过后在冻结提交执行。工作收据 170/510（model=496/compat=14）与提交态一致（R1 语料隔离后 walker 与 V01 同为 170）。

**SD 素材增补**：SD-02 的支持边界描述须含深度包络（`DeepNesting` 兼容类与 64 阈值），consumer 值等价准入（数值/nil astr 谓词）不得概括为全量静态 Str——已纳入 §9.2 供 R2 重冻结。

### 9.4 独立 final review R2（2026-10-16）

stage=review | review_scope=final | reviewer_identity=Codex（与R1同一独立复审聊天、未参与实现、未委派） | plan_id=PLAN-786 | plan_revision=1 | outcome=needs_fix | worktree_path=D:/autostack/.wt/lang-786/auto-lang | branch=plan-786-dev | reviewed_commit=db71263b5b9b7f9451c6e588fef31765accbba3d | base_commit=a1d5ed3114b163fa8a10c3d8985f5ae664bdad2c | repair_diff_base=bde9d0523198f1a15bc908c62a5b18b59324e61a | dependency_revisions=auto-project-model tree b4a5de79bf608a897944fb794f16201c850e3d29、Cargo.lock与scanner内容hash均未变（见报告） | spec_inputs=两目标规范 ec934b0006acb9b9838def82d0fcaf438ae0153f，hash未变 | evidence=[R2完整报告](attachments/786/review-r2/REVIEW.md)、[组合反例及结果](attachments/786/review-r2/counterexamples.log)、[10秒外部截止重放](attachments/786/review-r2/replay.ps1) | SD_snapshot=[R2规范增量](attachments/786/review-r2/spec-delta.md)，SHA256 29C7E73E90FC41C045C79AEF98AEDFF3B36EAF1D6A82B93234A4F54A7ADA97BB（needs_fix，修复后重冻结）。起止实现树clean、HEAD不变。

闭环：F786-R1-02 CLOSED（提交态project_model 11/11，8负例改名但直接reader验证保留，严格允许表未放宽）；F786-R1-03 CLOSED（原指出的vue触面行已按formatter修改，新reader两文件check通过）；F786-R1-01原始plain512反例已闭、全不变量仍OPEN，由F786-R2-01续接。

新增/续接：F786-R2-01 P1——深度筛在LF结束字符串，与实际模型跨行字符串语义不同；`extra: ["hello\nworld", <512层数组>]` 隐藏真实括号，read仍栈溢出abort（0xC00000FD）。32/128跨行控制、plain512/nodes512/转义换行512均exit0。F786-R2-02 P2——新增进程测试超时只assert，无kill/wait；panic Drop Child不回收进程，未形成子进程生命周期截止。此项为明确源码失败路径检查，未伪称正常测试已复现挂起。详细反例、位置与修复边界见报告。

acceptance_results: AC01 pass、AC02 fail、AC03 pass、AC04 fail、AC05 pending merge-owned。重跑reader8/8、780 adapter11/11、既有title/locale各1/1、check通过、diff-check通过；独立6组合中multiline512栈abort，runner exit1。未运行整档、未修实现、未merge/发布规范/清树。

重开T02..05，保留T01，current_step=1/7 / executing；成功闭合证据保留。连续两次final未通过，下一次readiness必须先提供诊断/覆盖调整表：为何单行深度样例遗漏跨行词法差异、如何使前置筛对真实parser保守可靠、如何证明超时子进程终止回收。不能仅修一条反例再次声称全闭。pending_closeout_gates: 最终审查通过后的merge-owned冻结裸cargo t及SD/ledger/归档/批到期判定/guard-clean。

2026-10-16 stage=work（R2 修复轮，含二次失败诊断）| plan_revision=1 | outcome=pass（R2 两项 findings 全闭，execution_done）| worktree=`D:/autostack/.wt/lang-786/auto-lang` | branch=`plan-786-dev` | base=`a1d5ed311` | code=`322719893`（叠加于 db71263b5/bde9d0523）| deps 不变 | task_ids=T-02..T-05 重闭（T-01 保留）| blockers=无 | next=review（R3，绑定 `322719893`）

### 二次失败诊断表（R2 要求，先诊断后修复）

| 诊断项 | 为什么之前的检查漏掉 | 本轮如何建立/证明 |
|---|---|---|
| 单行深度样例为何遗漏跨行词法差异 | R1 的前置筛字符串语义抄自我对 scanner 的**局部阅读**（且选了"遇 LF 断"的错误保守），而对抗性测试只造了单行字符串/转义换行/平面 10k 行——没有任何用例把**真实 LF 放进字符串后再接深层括号**。缺的不是更多深度样例，而是"前置筛 vs 真实 lexer 词法"的差分视角 | 逐函数核对真实词法（parse_dq_string 跨 LF、take_bracket_extent/take_arg_list 串内跳过跨 LF、skip_to_entry_end 行界但线性、value-position scan_string 跨 LF）→ 前置筛取**最宽字符串解释**（两种引号皆跨 LF、转义感知）：欠计数危险、过计数只多走兼容臂（安全）；未终止字符串吞剩余文本的安全性按 lexer 行为逐路证明（线性消费或先 Err，均无递归下钻）。反例形态（multiline+512）进程内与进程隔离双向冻结 |
| 前置筛如何对真实 parser 保守可靠 | ——（同上：靠读码对齐而非测试对齐，未覆盖"筛必须过近似 lexer"这一不变量） | 同族组合矩阵：multiline-32（包络内 → Model 准入控制，不收缩）/65/512（DeepNesting）、CRLF 跨行、`\"`/`\n` 转义转换、单引号跨行（过近似最宽）、未终止字符串（→ParseError 安全路径，括号不计也不 abort）、平面 64/65 边界、节点 {} 嵌套 512、字符串/行注释/块注释内括号不计数；进程隔离子进程重放平面 512/1024 + 跨行 512 |
| 超时子进程终止回收如何证明 | R1 测试的截止臂只 `assert!` 时间——panic 时 `Child` 被 drop 而无 kill/wait，管道未消费；且没有**确定性**的挂起控制来行使该路径（正常子进程从不超时，失败路径只被源码检查发现） | 探针重构为 `ChildGuard` 生命周期守护：成功/失败/超时/轮询错误/panic-Drop 全路径 kill+wait 回收，管道诊断收集进结果；`ProbeOutcome::TimedOut{reaped_success}` 携带回收后状态。确定性挂起控制：子进程睡 30s、父侧 2s 截止 → 必须 TimedOut 且回收状态为被 kill 的失败态（若 kill 失效则 wait 阻塞至 30s 后断言红，自身可证伪） |

### R2 finding 闭环表

| finding | invariant | 受影响路径/状态 | 修复与永久用例 | 当前结果 | 残余 gap |
|---|---|---|---|---|---|
| F786-R2-01（跨行字符串绕过深度筛） | 前置筛词法必须**过近似**真实 lexer 的字符串范围；欠计数=绕过包络=abort | 跨行 `"`/`'` 字符串后接深层 `[]`/`{}`（R2 反例 multiline+512）；CRLF；串内转义；未终止串吞尾 | 字符串扫描改跨 LF 直到闭引号（转义感知，双引号皆跨行）；未终止安全性按 lexer 行为证明（线性/先 Err）；永久用例=进程内同族矩阵 + 进程隔离探针新增 multiline-512 原样重放 | reader 8/8（含 2.12s 挂起控制时序）；R2 反例进程内 DeepNesting 且 oracle 一致 | 模型侧递归深度治理（非本计划范围，后续 project-model 计划承接） |
| F786-R2-02（截止测试不终结子进程） | 外部截止必须保护**子进程生命周期**：任何路径 kill+reap+诊断保留 | 超时臂/轮询错误臂/panic 路径/管道消费 | `ChildGuard`（terminate_and_reap 幂等 + Drop 兜底）+ `run_child_probe`（全路径回收、诊断收集）+ 确定性挂起控制（30s 子进程 vs 2s 截止，TimedOut+非成功回收态断言） | 挂起控制在套件内 2s 内确定性通过；探针诊断进失败消息 | 无 |

**R2 验证收据（命令=结果，`322719893` 工作树）**：`cargo test -p auto-man --lib display_reader` 8/8；`project_model` 11/11；`pac_title_overrides_document_title` 1/1、`pac_display_title_locale_chain` 1/1；`cargo test -p auto-project-model --locked` 30/30；`cargo check -p auto-lang -p auto-man` 0 error、零触面新警告；rustfmt 两 reader 文件 clean。V01 收据 170/510（model=496/compat=14）不变。merge-owned 门禁保持 pending。

### 9.5 独立 final review R3（2026-10-16）

stage=review | review_scope=final | reviewer_identity=Codex（本独立复审上下文，未参与实现、未委派） | plan_id=PLAN-786 | plan_revision=1 | outcome=needs_fix | worktree_path=D:/autostack/.wt/lang-786/auto-lang | branch=plan-786-dev | reviewed_commit=322719893262352ae1f25d894a833571c044e434 | base_commit=a1d5ed3114b163fa8a10c3d8985f5ae664bdad2c | repair_diff_base=db71263b5b9b7f9451c6e588fef31765accbba3d | dependency_revisions=auto-project-model tree b4a5de79bf608a897944fb794f16201c850e3d29；Cargo.lock/scanner hash与R2未变（见报告） | spec_inputs=两目标 ec934b0006acb9b9838def82d0fcaf438ae0153f、内容hash未变 | evidence=[R3报告/闭环表](attachments/786/review-r3/REVIEW.md)、[反例结果](attachments/786/review-r3/counterexamples.log)、[带10秒截止的重放](attachments/786/review-r3/replay.ps1) | SD_snapshot=[R3规范增量](attachments/786/review-r3/spec-delta.md)，SHA256 01907A32F42B13B93A447377CD6418979664999A5977FEB695A86679B8B0547F（needs_fix，修复后重冻结） | next=同树work修复并提交后复审。起止实现树clean、HEAD不变。

readiness诊断表已检查，但“最宽字符串解释即安全”的论证不成立：跳过更多文本会少计真实括号；`skip_to_entry_end` 本身线性，不代表返回外层parser后不会递归。F786-R2-01原双引号跨行反例已闭，完整不变量由F786-R3-01续接；F786-R2-02 CLOSED（ChildGuard各出口回收，确定性30s/2s挂起控制通过）。R1-02/03闭合保持。

findings: F786-R3-01 P1——`extra: 'abc\nignored: 0\ndeep: <512层数组>` 被前置筛当作到EOF字符串，真实scanner的unsupported路径在行界结束后恢复外层解析，仍栈溢出abort（0xC00000FD）；单引号后续闭合、f-string、call三种组合亦同红，32/128控制与plain512/multiline512均通过。F786-R3-02 P3——新测试997行unused reaped_success、888/892行diagnostics字段dead_code共三警告；非测试cargo check不覆盖，须修复并保留诊断/回收断言。均为合同内缺口，登记不批准延后。

acceptance_results: AC01 pass、AC02 fail、AC03 pass、AC04 fail、AC05 pending merge-owned。本轮reader8/8、780 adapter11/11、title/locale各1/1、model30/30、check/rustfmt/diff-check通过；独立8组合中4个栈abort、runner exit1。sccache端口os10013通过本次进程RUSTC_WRAPPER=''绕过，未更改配置。未跑整档、未修实现、未merge/发布规范/清树。

重开T02..05及依赖验收，保留T01与未受影响接线/locale/去重证据，current_step=1/7、status=executing。下次readiness须以实际parser重入路径论证括号上界，不能再以局部线性/更宽跳串推断安全；永久进程隔离重放本轮与R1/R2反例，清除新增test构建警告。pending_closeout_gates: final pass后的merge-owned冻结裸cargo t、SD/ledger/归档/批到期判定/guard-clean。

2026-10-16 stage=work（R3 修复轮，修正 R2 诊断错误）| plan_revision=1 | outcome=pass（R3 两项 findings 全闭，execution_done）| worktree=`D:/autostack/.wt/lang-786/auto-lang` | branch=`plan-786-dev` | base=`a1d5ed311` | code=`079b3b8d4`（叠加于 322719893/db71263b5/bde9d0523）| deps 不变 | task_ids=T-02..T-05 重闭（T-01 保留）| blockers=无 | next=review（R4，绑定 `079b3b8d4`）

### R2 诊断错误修正（R3 要求）

R2 诊断表的论断「skip_to_entry_end 行界但为线性跳过，无递归位点 ⇒ 未终止字符串吞尾安全」是**错误的**：skip_to_entry_end 只在**局部**线性，它返回后外层 parse_document 继续解析后续条目，其中的深层结构照常递归下钻——局部无递归 ≠ 整条 read 路径安全。同理「更宽的字符串跳过=更保守的上界」不成立：更宽的跳过区域会**藏住可达解析状态会数的真实括号**。R1（遇 LF 断→闭引号误判）、R2（跨 LF→未终止吞 EOF）两轮反例同根：任何"按字符串语义跳区域"的上界都需要精确预测各解析路径的字符串行为，而各路径语义互不相同（parse_dq_string 跨 LF、skip_to_entry_end 行界、take_bracket_extent 跨 LF）——该方向不可靠。

**R3 终形上界**：只跳注释（行 `//`、`#` 非 `#[`、块 `/* */`——这些在任何可达解析状态下都不会成为结构括号），**其余文本一律计数（含字符串内括号）**。可靠性论证：解析器只在注释外、字符串外的文本上做结构下钻；计数集 ⊇ 结构括号集 ⇒ 只会过计数（多走兼容臂，值行为不变），永不欠计数（绕过包络→abort）。过计数代价：字符串内嵌套括号 >64 的输入走兼容臂（受管语料字符串内括号为个位数，V01 收据 170/510 不变）。

### R3 finding 闭环表

| finding | invariant | 受影响路径/状态 | 修复与永久用例 | 当前结果 | 残余 gap |
|---|---|---|---|---|---|
| F786-R3-01（未闭合引号绕过深度筛） | 深度上界必须是解析器结构括号集的**超集**；不得从局部线性跳过推断整条 read 路径安全 | 未闭合单引号（`extra: 'abc`）/更晚闭合单引号/f-string（`` ` ``）/调用内未闭合双引号 + 后续深层条目——skip_to_entry_end 行界重入路径四形态（R3 复现 4/8 probe abort） | max_bracket_depth 删除全部引号处理，只跳注释、其余全计；永久用例=进程内四形态矩阵（32 层 Model 准入控制/65/>512 DeepNesting+oracle 三方一致）+ 进程隔离子进程四形态 512 重放 + **复审反例原样字节**复制为持久 fixtures（`r3-*.at` 8 文件）逐文件复验 | reader 9/9（r3 文件复验测试 8/8 文件全过）；R1/R2 原反例（平面/跨行 512）继续在隔离探针重放 | 无（模型侧递归深度治理仍留 project-model 后续计划） |
| F786-R3-02（新增测试 3 条编译警告） | 新增代码零警告（含 lib-test 编译面；派生 Debug 不算字段读取） | run_child_probe 轮询错误臂 unused `reaped_success`；ProbeOutcome 两 variant 的 `diagnostics` 字段仅构造未读 | 探针 match 臂解构并真实使用全部字段（失败断言/panic 消息携带诊断）；轮询错误臂诊断串包含回收态 | `cargo test -p auto-man --lib --no-run` 对 display_reader 文件零警告提及 | 无 |

**R3 验证收据（命令=结果，`079b3b8d4` 工作树）**：`cargo test -p auto-man --lib display_reader` 9/9（2.12s 含挂起控制时序）；`project_model` 11/11；`pac_title_overrides_document_title` 1/1、`pac_display_title_locale_chain` 1/1；`auto-project-model --locked` 30/30；lib-test 编译对触面文件零警告；`cargo check -p auto-lang -p auto-man` 0 error；rustfmt 两 reader 文件 clean；V01 收据 170/510（model=496/compat=14）不变。merge-owned 门禁保持 pending。

### 9.6 独立 final review R4（2026-10-16）

stage=review | review_scope=final | reviewer_identity=Codex（本聊天独立复审上下文，未参与实现、未委派） | plan_id=PLAN-786 | plan_revision=1 | outcome=needs_fix | worktree_path=D:/autostack/.wt/lang-786/auto-lang | branch=plan-786-dev | reviewed_commit=079b3b8d412ed7e7c1abdaade66c732f7281a454 | base_commit=a1d5ed3114b163fa8a10c3d8985f5ae664bdad2c | repair_diff_base=322719893262352ae1f25d894a833571c044e434 | dependency_revisions=auto-project-model tree b4a5de79bf608a897944fb794f16201c850e3d29、worktree Cargo.lock/scanner hash与R3不变（main lock不同，全部测试在plan树） | spec_inputs=两目标 ec934b0006acb9b9838def82d0fcaf438ae0153f，hash不变 | evidence=[R4报告/闭环表](attachments/786/review-r4/REVIEW.md)、[反例与结果](attachments/786/review-r4/counterexamples.log)、[10秒截止重放](attachments/786/review-r4/replay.ps1) | SD_snapshot=[R4规范增量](attachments/786/review-r4/spec-delta.md)，SHA256 03A9A31EC65846BC2049693EFBE121E02FE08B4DF0954E7B9639A529031C3A1C（needs_fix，不批准发布） | next=同树work修复提交后复审。起止实现树clean、HEAD不变。

R3 readiness诊断/闭环表已核查：“计数集是结构括号超集”不能推出嵌套上界——字符串里的注释起始符可能隐藏后续真括号，字符串里的闭括号还会抵消真实深度。R3-01旧8个反例重放全过，但完整安全不变量仍OPEN，由下列两项续接。F786-R3-02 CLOSED（lib-test触面零新警告，字段实际用于诊断）；R2生命周期、R1语料/格式闭合保持。

findings: F786-R4-01 P1——`extra: "/*"` 后接512层数组，前筛误吞EOF而真实parser解析深数组，0xC00000FD；数组内字符串 `"//"` / `"#"` 后同一行512层数组也abort。F786-R4-02 P1——512层数组每层先放字符串 `"]"`，前筛的真实开括号被串内闭括号逐层抵消（max=1），仍栈abort；无注释标记，修前项不能闭本项。旧oracle均name=n/title=ok；32/128同族和plain512为控制。具体字节/位置/修复边界见R4报告。

acceptance_results: AC01 pass、AC02 fail、AC03 pass、AC04 fail、AC05 pending merge-owned。reader9/9、780 adapter11/11、model30/30、既有title/locale各1/1、check、两reader文件fmt及diff-check通过；R3独立8/8通过；R4独立9组=5控制通过+4栈abort，runner exit1。未跑整档、未修实现/merge/发布规范/清树。

重开T02..05及依赖验收，保留T01及未受影响接线/locale/去重证据；status=executing、current_step=1/7。下一readiness须先修正上界论证及覆盖矩阵，同时覆盖注释标记藏结构与串内闭括号抵消，永久进程隔离重放历史与本轮反例；不得只修反例文本后再次声明全闭。pending_closeout_gates: final pass后的merge-owned冻结裸cargo t、SD/ledger、合流归档、批到期判定、guard-clean。

2026-10-16 stage=work（R4 修复轮，上界论证终形）| plan_revision=1 | outcome=pass（R4 两项 findings 全闭，execution_done）| worktree=`D:/autostack/.wt/lang-786/auto-lang` | branch=`plan-786-dev` | base=`a1d5ed311` | code=`bb3cd34b7`（叠加于 079b3b8d4/322719893/db71263b5/bde9d0523）| deps 不变 | task_ids=T-02..T-05 重闭（T-01 保留）| blockers=无 | next=review（R5，绑定 `bb3cd34b7`）

### 上界论证终形（R4 要求修正，覆盖 R1..R4 全部反例族）

**为什么 R3 上界仍不成立**：R3「只跳注释」仍做了一类词法判定——在无字符串上下文的情况下识别 `//`、`#`、`/*`。字符串内的注释标记（`extra: "/*"`、`["//", …]`、`["#", …]`）让筛子按注释跳区域：块注释形态吞掉后续多行、行注释形态吞掉本行剩余，两者都藏住真实括号（F786-R4-01）。更根本地，**计数型上界（superset of bracket characters）不能证明嵌套上界**：字符串内的闭括号参与 max-depth 计算会把每个真实开启层层抵消（`["]", ["]", …` 真实嵌套 512、旧筛 max=1，F786-R4-02）——复审原话"added closing brackets can reduce the measured maximum"。

**R4 终形**：`bracket_open_count = 文本中 '['+'{' 字符总数`，**零词法判定、永不递减**。论证：解析器结构下钻（scan_value_positions 的 scan_object/scan_array、parse_node 子节点、AtomParser 结构化字面量）按**嵌套层**递归——元素迭代（loop）、下钻才递归，且每下钻一层必消费一个被解析为 opener 的 `[`/`{` 字符（已核实 scan_array/scan_object 为元素循环+按层递归）。故**真实递归深度 ≤ 开启括号字符总数**：闭括号不参与计数，不可能削减上界（R4-02 根除）；注释/字符串上下文只会**虚增**计数（其中 `[`/`{` 被算入而解析器不下钻），过计数仅多走兼容臂、值行为不变（R4-01 根除）；零词法判定 ⇒ 「筛子与某条解析路径的字符串/注释语义不一致」这一反例族**结构性不存在**（R1 遇 LF 断 / R2 跨 LF / R3 行界重入 / R4 注释标记与闭括号抵消，四轮同根）。阈值 64 不变（实测 128 通过/512 abort 之下成对余量；受管语料全部 ≤64 开启）。

### R4 finding 闭环表

| finding | invariant | 受影响路径/状态 | 修复与永久用例 | 当前结果 | 残余 gap |
|---|---|---|---|---|---|
| F786-R4-01（字符串内注释标记藏括号） | 深度上界不得依赖任何词法判定（注释/字符串/转义），任何上下文误判都可藏住可达解析状态会数的括号 | 字符串内 `/*`（吞多行）后接深层条目；`["//", …]`/`["#", …]` 同行深数组——三种形态 R4 复现 abort | 删除全部注释/字符串识别，只数开启括号；永久用例=进程内 quoted-marker 四形态（32 控制 Model/65/512 DeepNesting）+ 进程隔离子探针重放 + **R4 反例原样字节** fixtures（`r4-quoted-*.at`）逐文件复验 | reader 9/9；18 个反例文件（r3×8+r4×10）全过 | 无 |
| F786-R4-02（字符串闭括号抵消深度） | 计数上界不得被闭括号削减——闭括号（任意上下文）不参与上界计算 | 每层数组以字符串 `"]"` 开头的嵌套（真实 N 层、旧筛 max=1）——R4 复现 abort（32/128/256/512 族） | 同上（开启括号总数，永不递减）；永久用例=cancel-depth 进程内 32(Model)/65 边界 + fixtures `r4-cancel-depth-{32,128,256,512}.at` + 探针重放 | 同上 | 无 |

**R4 验证收据（命令=结果，`bb3cd34b7` 工作树）**：`cargo test -p auto-man --lib display_reader` 9/9（2.12s 含挂起控制）；counterexample 文件复验 18 文件（name=n/title=ok、oracle 三方一致、32 控制 Model、>64 全 DeepNesting）；`project_model` 11/11；title/locale 各 1/1；`auto-project-model --locked` 30/30；lib-test 编译触面零警告；check 0 error；rustfmt clean；**V01 收据不变（170/510，model=496/compat=14）——开启计数上界未影响任何受管文件的 Model 路径**。merge-owned 门禁保持 pending。

### 9.7 独立 final review R5（2026-10-16）

stage=review | review_scope=final | reviewer_identity=Codex（本聊天独立复审上下文，未参与实现、未委派） | plan_id=PLAN-786 | plan_revision=1 | outcome=pass | worktree_path=D:/autostack/.wt/lang-786/auto-lang | branch=plan-786-dev | reviewed_commit=bb3cd34b78610ddbdc39d50e682eec7174896892 | base_commit=a1d5ed3114b163fa8a10c3d8985f5ae664bdad2c | repair_diff_base=079b3b8d412ed7e7c1abdaade66c732f7281a454 | dependency_revisions=auto-project-model tree b4a5de79bf608a897944fb794f16201c850e3d29；worktree Cargo.lock/scanner SHA256与R4不变（见报告） | spec_inputs=两目标 ec934b0006acb9b9838def82d0fcaf438ae0153f、内容hash不变 | evidence=[R5报告/AC/闭环表](attachments/786/review-r5/REVIEW.md)、[R4原反例重放](attachments/786/review-r5/r4-replay.log)、[独立边界结果](attachments/786/review-r5/counterexamples.log)、[10秒截止runner](attachments/786/review-r5/replay.ps1) | SD_snapshot=[R5冻结规范增量](attachments/786/review-r5/spec-delta.md)，SHA256 69D8D31BA80D43B86B84A93AADE32095B79EBA6EC91D937BCF028E1E83C4DA22 | next=merge（T07）；起止实现树clean，HEAD不变。

F786-R4-01/02 CLOSED：只计全部开启括号字符、零词法判定/零递减，字符串内注释标记与闭括号均无法削减上界；原R4九组全部exit0。R1/R2/R3安全反例族闭合保持（R3独立八组全过）；R3新测试警告、R2子进程生命周期、R1语料/格式全部保持CLOSED。本轮诊断与上界论证核查通过：document/value scanner结构下钻消耗opener、兄弟迭代；结构化调用在AtomParser前由分类器拒绝，文档参数表迭代采集。64阈值为已验证消费包络，不宣称模型任意线程栈/无限规模安全。

acceptance_results: AC01/02/03/04 pass；AC05 pending merge-owned。reader9/9（含18文件及2s挂起控制）、780 adapter11/11、model30/30、既有title/locale各1/1、check/rustfmt/diff-check通过；独立R3 8/8、R4 9/9、R5 13/13（object/nodes/mixed/flat/string/comment的64/65边界+512参数控制）全部exit0。旧接线/locale/依赖检查复用理由：本轮未改相关源文件或依赖；所有必需定向验收已在新提交重跑。无新增未闭合同项；兼容/迁移退出清单与SD均已复核。

T06完成，status=reviewed、current_step=6/7；T07保持未完成。pending_closeout_gates: merge-owned冻结提交裸cargo t、SD两目标/索引/ledger沉淀、合流归档、主检出批回归到期判定、guard-clean清理。未触ui_gen/VM/aavm/book，故无tu/tv/taa/tb触发。本轮未跑整档、未修实现、未merge/发布规范/归档/清树；review pass不等于交付完成。

2026-10-16 stage=merge | plan_revision=1 | outcome=pass（delivered，7/7）| delivery_commit=`240281c96`（master ff-only；rebase 哈希映射 6/6 range-diff 全等：bb3cd34b7→18416ac2b 等，见收据）| review=R5 final pass 绑定 bb3cd34b7（spec-delta SHA256 69D8D31B…）| checkpoints：**prepared**=SD-01（auto-man/project.md f12b0b4098450025）+SD-02（project-model/project.md 27d9fd39ef486a38）按冻结措辞落 worktree、spec-index 无差异；**landed**=裸 cargo t 5364/5364 绿（108.4s，冻结基线 bb3cd34b7；触面仅 auto-man+fixtures，无 tv/tt/tb/tu/taa）→ rebase master → ff-only；**ledger_refreshed**=.autoos/specs.json v25→v26（Regime B 受管离线投影，五护栏：字节备份/脚本 RMW/作用域 diff 零扰动/schema 校验/回读 P786-D1/D2/T/R5/M），通道诚实记录="guarded offline projection, no store writer reachable"；**archived**=docs/plans/archive/786-*.md + 收据报告 docs/plans/reports/786-display-reader-merge.md + 本提交入库（无未跟踪 docs 遗留）；**cleaned**=wt-guard clean → worktree/分支/auto-down 兄弟/组目录移除（见下）。批量回归到期判定：**未到期**（786%5=1；收据 781@2026-10-15T03:55Z 距合入 <48h）。生产工件观察：auto-lang 无登记的运行态生产进程/发布重启协议；`auto` CLI 分发构建属后续发布流程。

completion_kind: delivered
