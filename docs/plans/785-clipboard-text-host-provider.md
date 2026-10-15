---
plan_id: PLAN-785
status: drafting
feature_name: 文本剪贴板宿主能力依赖倒置
author: [Codex]
created_at: 2026-10-15
updated_at: 2026-10-15
plan_revision: 1
current_step: 0
total_steps: 8
supersedes_spec_components: [docs/specs/auto-lang/vm/overview.md, docs/specs/auto-lang/ui/overview.md]
new_spec_components: []
touched_goals: [GOAL-003, GOAL-007]
affects: [auto-lang/vm, auto-lang/ui]
depends_on: [PLAN-784, PLAN-781]
---

# [PLAN-785] 文本剪贴板宿主能力依赖倒置

## 0. 变更摘要

把779的装配模式推广到一个实际迁移批次：文本剪贴板 native 2925/2926。core保留值编码与调用合同，UI侧提供现有OS实现；外部无UI类型provider可经真实VM读写。只迁文本两函数，不把整套clipboard/media一次搬走。

## 1. 目标

剪断选定两shim对ui::clipboard的直接调用；保持canonical名、ID、String/Bool编码、无能力错误和OS不可用行为。显式默认优先、兼容补空槽、实例覆盖及binding/proof规则与779一致。
非目标：文件/图像2934..2937、editor copy/paste、scroll/term/media、async或通用动态插件、换OS剪贴板库、搬仓、UI AST重构。

## 2. 架构方案

新增纯宿主能力接口（拟ClipboardTextProvider，名字T01核定）与NI实例槽；接口使用拥有型Rust文本/布尔或等价无UI类型，不泄漏VM字符串索引/借用缓冲。VM shim负责现有pop/add_string/rc_push_str_idx/Bool lane，adapter只委托ui/clipboard.rs现有实现。
进程默认/if-absent/实例快照复用779纪律，不能使用frame两函数只读标量的证据替代字符串ownership验证。

## 3. 技术栈

Rust、NativeInterface、既有宿主factory、现有clipboard/arboard后端。新增独立consumer使用公开API；不为了迁出把默认feature扩大到UI或引入第二个VM。

## 4. 需求分析与背景调查

取材：docs/specs/auto-lang/{vm,ui}/overview.md中779现行合同、Design37§4；版本见 [输入指纹](attachments/modularization-planning-20261015/inputs.md)。
实锚：vm/native_catalog.rs::NATIVE_CLIPBOARD_TEXT=2925、NATIVE_CLIPBOARD_SET_TEXT=2926；native.rs::shim_clipboard_text/shim_clipboard_set_text；ui/clipboard.rs::clipboard_get/clipboard_set；vm/host_capabilities.rs、engine.rs::AutoVM::new、ui/frame_provider.rs及lib.rs::install_ui_host_capabilities。
当前启用ui-clipboard时get不可用→空串、set不可用→false；未启用时两个shim返回VMError，**不能照搬frame缺席0**。getter当前add_string+rc_push_str_idx，setter pop_string_arg；保留编码与释放责任。ui-clipboard独立feature组合是否可单建须T01核实，不冒称所有feature组合已支持。
用户仅授权本轮建计划，实施待合同确认。此计划仅修改auto-lang，不改旧UI/Man仓或783实验。784完成后可准备；Rust生产接线等781最终验收合并、共享native/宿主热点释放后开始。783无需全结束，除非执行时扩界碰本计划接口。

## 5. 详细设计

T01先冻结当前调用入口/feature/错误/值编码/生命周期矩阵，登记哪些flags原本可建；不能用新增任意字符串长度或固定堆上限躲避ownership。若现有公开API不足，给出有界接线决策后再实施，不公开全部VM internals。
显式provider允许no-ui宿主注入文本能力；未注入时保留当前feature-disabled错误（诊断可定位2925/2926）。有UI默认能力时保留OS不可用empty/false；任何状态不得伪装成功写入。
强制替换只通过显式API；隐式安装同锁if-absent，全支持入口使用统一installer。构造期快照、NI Clone/merge、实例覆盖独立；不能覆盖其他native或改变reference proof机制。
新fixture拟 `test/fixtures/modularity/clipboard-provider-consumer/`，由T03创建，主workspace exclude+独立manifest；no-ui独立锁与UI开启外部消费都要实际验证。允许沿用779 workspace锁定rlib外部harness验证UI公开API，但其覆盖边界须明示，不把它写成新锁发行通过。
生产OS adapter保留现有函数与既有测试，不新增随意改写用户剪贴板的验收步骤；真实VM+可控provider验证值流/ownership，真实宿主factory验证默认adapter接入。跨仓发行前另查OS/feature/锁/patch闭包。

### 规范增量

| delta_id | 操作 | target | before/after rule | 原因 | AC |
|---|---|---|---|---|---|
| SD-01 | modify | docs/specs/auto-lang/vm/overview.md | 2925/2926经文本能力接口，编码/feature错误/ownership与优先关系明确 | 剪core实现依赖 | AC01/02 |
| SD-02 | modify | docs/specs/auto-lang/ui/overview.md | UI默认adapter实际宿主注册路径，剩余clipboard/editor族未迁 | 装配不旁路、不外推全族完成 | AC03 |

## 6. 测试设计

实施者未指派；独立final review同树验收；拟树D:/autostack/.wt/lang-785/auto-lang，branch plan-785-dev。T03使下列fixture/新用例可执行；当前均为规划，不是已通过证据。

| V | work owner | AC | 输入/入口 | oracle及反例 |
|---|---|---|---|---|
| V01 | T03 | 01/02 | fake A/B经真实VM clipboard.text()/set_text；高层与原始双活VM | 不同文本与Bool到stdout；set收到精确UTF-8参数，实例不串线 |
| V02 | T03 | 02 | 空串、中文/emoji、相邻连续读写；provider返回临时owned String | VM后续使用原文本仍正确，不借用已释放缓冲；String/Bool lane不漂移 |
| V03 | T03 | 02 | 无UI+无provider；UI能力不可用；显式返回失败 | 原feature-disabled VMError；有能力不可用empty/false；失败不计成功 |
| V04 | T03/T04 | 02/03 | UI开/关×显式默认/空槽×高层/原始VM；重复/并发安装 | 显式默认不被覆盖，空槽兼容补缺；实例覆盖不污染默认，779历史控制仍过 |
| V05 | T04 | 01/03 | NI merge/重注册与名字/ID查找；宿主factory+adapter | 2925/2926不撞邻族，proof/revision规则不变，core两shim无UI实现直引 |

开发只做cargo check -p auto-lang -p auto与定向clipboard/新plan族、consumer门、必要生命周期控制；final review先验收，merge在冻结提交跑裸cargo t+tv（改VM）、本计划未改ui_gen则不tu，不触aavm不taa。tf仅按批量规约，不在工作树跑。

## 7. 验收标准

| ID | owner_stage | 可观察结果 |
|---|---|---|
| AC01 | work | 两shim实现依赖倒置，外部无UI类型consumer真实非空值流，V01/05 |
| AC02 | work | UTF-8/owned生命周期、返回编码、缺席/失败及装配优先矩阵通过，V01..05 |
| AC03 | work | 受支持真实宿主自动注入，显式不被覆盖；未迁族/feature限制明示，V04/05 |
| AC04 | review | 独立新正反控及779回归、逐AC审查绑定提交/依赖，SD冻结 |
| AC05 | merge | 冻结提交门禁通过后规范沉淀/合流归档/批到期判定/guard-clean |

## 8. 执行步骤

| 状态 | ID | owner_stage | 依赖 | 操作/验证产出 | AC |
|---|---|---|---|---|---|
| [ ] | T-01 | work | 784+781交付/热点释放+确认 | 冻结feature/调用/encoding与ownership矩阵，bounded接线决策和反控输入 | 01/02/03 |
| [ ] | T-02 | work | T01 | host_capabilities/NativeInterface/AutoVM接文本接口、快照/覆盖和两shim，core实现直引清零 | 01/02 |
| [ ] | T-03 | work | T02 | 新独立consumer及V01..04，字符串释放/失败与双feature形态实际门 | 01/02 |
| [ ] | T-04 | work | T03 | UI clipboard adapter接全部受支持宿主；V04/05、保留779/未迁族 | 03 |
| [ ] | T-05 | work | T04 | 定向encoding/ownership/contract/入口回归+check/fmt，无整档 | 01/02/03 |
| [ ] | T-06 | work | T05 | 实现提交、永久案例与覆盖记录、SD素材，确认所有work项闭合 | 01/02/03 |
| [ ] | T-07 | review | T06 | 独立final，不能以默认0/空串/跳过OS代替注入证据 | 04 |
| [ ] | T-08 | merge | review pass | t+tv冻结门、SD/ledger/归档合流/清理 | 05 |

## 9. 复审记录

2026-10-15 stage=new | plan_revision=1 | outcome=pass（草案preflight） | next=前置交付及合同确认后work | 0/8，无实现/测试/树。已明确frame与字符串能力差别；T01仅核定接线机制，不允许修改既有缺席/编码合同来消红。

## 10. 待澄清事项

默认方案无需用户先选trait命名；若T01证明保持错误/ownership/宿主行为需要改变公共接口或扩大媒体/线程模型，停在具名决策工件，修订本合同后再实施。独立UI锁解析问题不得默默记为通过，按V04边界与后续发行验收处理。
