---
plan_id: PLAN-786
status: drafting
feature_name: AutoMan 展示字段读取收口与显式兼容适配
author: [Codex]
created_at: 2026-10-15
updated_at: 2026-10-15
plan_revision: 1
current_step: 0
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
| [ ] | T-01 | work | 784+780合流/合同确认 | 盘点真实调用者、旧输入输出oracle与精确测试filter；冻结V01/02/03表 | 01/02/03 |
| [ ] | T-02 | work | T01 | 新project_display_reader只读入口，DTO/显式compat准入决策；不允许无界重写parser | 01/02 |
| [ ] | T-03 | work | T02 | vue.rs三字段与locale读取接线、两个重复扫描循环收口；其他字段不扩界 | 01/03 |
| [ ] | T-04 | work | T03 | V01..05永久对拍、变异/截止控制，780 scoped回归、check/fmt | 01/02/03 |
| [ ] | T-05 | work | T04 | 提交实现+真实去重/兼容退出清单+SD素材，确认work验收完整 | 01/02/03 |
| [ ] | T-06 | review | T05 | 独立逐AC、未授权语义差异与伪DTO路径扫描，SD冻结 | 04 |
| [ ] | T-07 | merge | review pass | 冻结t门禁、知识沉淀、合流归档与guard-clean | 05 |

## 9. 复审记录

2026-10-15 stage=new | plan_revision=1 | outcome=pass（草案preflight） | next=784完成及确认后work，可与785不重叠面并行 | 0/7，无源码实现/测试/树。已将first-vs-last、空串与unsupported的差异列为必验而非默许行为变更；T02有界决策不允许静默缩合同。

## 10. 待澄清事项

无需用户先选内部类型名。T02若证明必须改变展示结果、全面放弃动态兼容或引入VM求值，提交具体反例后修订/裁定，不能自行选择破坏兼容。该阻断只影响本reader计划，不要求781/783停工。
