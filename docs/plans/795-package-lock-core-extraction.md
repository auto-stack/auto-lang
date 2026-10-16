---
plan_id: PLAN-795
status: executing
feature_name: 包锁模型与校验核心抽取
author: [Codex]
created_at: 2026-10-16
updated_at: 2026-10-16
plan_revision: 1
current_step: 0
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
| [ ] | T-01 | work | 786/791归档、合同确认 | 冻结lock.rs/Target公开API、serde/错误旧臂、Git与path行为，确定轻投影和版本来源 | 02/04 |
| [ ] | T-02 | work | T01 | 新crates/auto-package-lock，模型/codec和无Target输入；cargo check新crate | 01/02 |
| [ ] | T-03 | work | T02 | 迁排序/查询/verify单一规则，observer/路径负控；保留原错误阶段 | 02/03 |
| [ ] | T-04 | work | T03 | auto-man lock.rs兼容委托及Target/Git adapter；旧API构造/方法真实编译 | 02/04 |
| [ ] | T-05 | work | T04 | 外部consumer及V01..04、metadata/777边界；检查792写面未侵入 | 01–04 |
| [ ] | T-06 | work | T05 | 净删/搬迁/未迁清单和SD素材，提交冻结代码与并行交接收据 | 01–04 |
| [ ] | T-07 | review | T06 | 独立真实锁流程/负控/公开兼容和SD冻结 | 05 |
| [ ] | T-08 | merge | final pass | 裸t+定向门、沉淀/归档/guard、regress到期交接；必要时792刷新source闭包 | 06 |

## 9. 复审记录

2026-10-16 stage=new | plan_revision=1 | outcome=pass（合同preflight） | next=合同确认后work | 0/8；本轮未运行Rust或探测网络Git。

2026-10-16 stage=work | PLAN-795 | plan_revision=1 | 入场执行 | worktree=D:/autostack/.wt/lang-795/auto-lang | branch=plan-795-dev（repo 命名规约覆盖合同拟称 codex/plan-795-dev；PLAN-726 组布局含 auto-down 兄弟 detached@c39ac7f） | base_commit=579f6dec0 | 前置验证：786/791 已归档；inputs.json 指纹复核 lock.rs=407127b5/target.rs=a483fa96/auto-man project.md=f12b0b40/module-boundaries=82718ad2 四项全等 | 用户显式授权 /auto-plan:work 实施。

## 10. 待澄清事项

T01若旧公开构造与拆包类型规则冲突，采用保持旧API的薄DTO门面并明确转换；不得直接删方法/改字段。发现旧锁校验弱点只登记后续行为修复，不以抽取名义改格式或宣布安全升级。
