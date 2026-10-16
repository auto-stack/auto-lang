---
plan_id: PLAN-791
status: drafting
feature_name: 生成核心与 Auto 可执行模板适配拆分
author: [Codex]
created_at: 2026-10-16
updated_at: 2026-10-16
plan_revision: 1
current_step: 0
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
| [ ] | T-01 | work | 784/786交付、合同确认 | 冻结generator/data/template/guard/error及legacy/Man消费图、输出与错误阶段旧臂 | 02/04 |
| [ ] | T-02 | work | T01 | 新建crates/auto-gen-core、迁单一guard/error/报告和小renderer边界；cargo check -p auto-gen-core | 01/03 |
| [ ] | T-03 | work | T02 | 迁通用输出编排，接dry_run/guard/错误收集；加入失败文件不改写反控 | 02/03 |
| [ ] | T-04 | work | T03 | auto-gen保留解释器adapter并真实委托core；旧API兼容re-export，删除重复算法 | 02/04 |
| [ ] | T-05 | work | T04 | 外部gen-core-consumer、V01..04、metadata/777边界与既有Auto模板定向验证 | 01–04 |
| [ ] | T-06 | work | T05 | 净删/搬迁及剩余Man模板依赖收据、SD素材；冻结提交与完整交接 | 01–04 |
| [ ] | T-07 | review | T06 | 独立字节对照、失败出口与真实生产调用、规范冻结 | 05 |
| [ ] | T-08 | merge | final pass | 裸t+本crate/adapter定向门，沉淀/归档/guard清理 | 06 |

## 9. 复审记录

2026-10-16 stage=new | plan_revision=1 | outcome=pass（草案preflight） | next=合同确认后work | 0/8。当前可执行模板依赖被明确保留在adapter；未声称Man core解耦完成。

## 10. 待澄清事项

T01据实际CodeGenerator公开类型确定兼容re-export/错误映射，不要求提前决定插件ABI。若renderer生命周期迫使core持AutoInterpreter/AST或删除旧能力，先给具名依赖环修订，不以optional feature藏回向依赖。
