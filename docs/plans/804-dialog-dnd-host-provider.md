---
plan_id: PLAN-804
status: drafting
feature_name: 对话框与拖出宿主能力迁移
author: [Codex]
created_at: 2026-10-17
updated_at: 2026-10-17
plan_revision: 1
current_step: 0
total_steps: 8
supersedes_spec_components: [docs/specs/auto-lang/vm/overview.md, docs/specs/auto-lang/ui/overview.md, docs/specs/auto-lang/ui/design/shell-a2r-seams.md]
new_spec_components: [docs/specs/auto-lang/ui/design/dialog-dnd-host-capabilities.md]
touched_goals: [GOAL-003, GOAL-007]
affects: [auto-lang/vm, auto-lang/ui, auto-lang/host-adapters]
depends_on: [PLAN-779, PLAN-785, PLAN-790, PLAN-794]
---

# [PLAN-804] 对话框与拖出宿主能力迁移

## 0. 变更摘要

迁移dialog_open/dialog_save（2927/2928）及dnd_start（2938）三native：VM只编解码并调用owned宿主接口；OS对话框、父窗选择、拖出载荷解析与COM执行由adapter承担。同时合并VM/a2r重复对话框构建逻辑，保留两者已有父窗策略差异。

## 1. 目标

本族VM不直引rfd/raw-window-handle/Win32/COM/ui::native_dnd，真实no-ui VM可由自定义provider供值；UI宿主保留模态父窗、过滤/默认文件名、取消值、拖出输入线程及完成事件行为。三个ID/签名、字符串所有权/布尔编码保持。

非目标：迁拖入IDropTarget、所有OS能力/终端/媒体/diff、改变a2r默认父窗策略、修复旧HWND缓存策略或重写OLE。三函数迁出不代表整个UI运行时可搬仓。

## 2. 架构方案

在既有host_capabilities分别增加DialogHostProvider（open/save）与DndHostProvider（start），各自独立槽与feature装配，避免一种能力在位掩盖另一种缺席。仅跨界owned String/Bool；JSON仍是dnd现有wire输入，由唯一UI载荷parser解释，不复制为VM里的第二套parser。

两槽沿779/785/790/794装配：实例覆盖、构造默认快照、兼容if-absent同锁补缺；NI clone保provider、VM merge保目标实例，注入不改变binding_revision/proof。调用前释放槽锁，真实模态调用可重入，不跨回调持RwLock。Send+Sync约束provider自身，不授权VM/COM跨线程。

OS对话框实现放新上层host_adapters/dialog.rs（拟名），独立于ui renderer门；保留ui-dialog单开、不启用ui也能走原高层dialog路径。ui::dialog旧open/save薄委托同一实现、使用旧unparented策略；VM默认adapter使用旧parented策略。Dnd adapter留UI，仅Windows+native-dnd+ui安装，调用原start_drag。

## 3. 技术栈

Rust、现有rfd/raw-window-handle与Win32/OLE桥、NativeInterface/AutoVM；独立consumer、组件及真实窗口/OS交互证据。无新OS桥/RPC/后台拖拽线程。

## 4. 需求分析与背景调查

用户2026-10-17授权继续规划，并询问对799/801依赖。本轮只新建合同/设计路线，不执行Rust、不修复或合并他线、不创建agent/树、不切旧仓。取材主检出9932962bc157be13bcaa0520011c7c78082cb2fe，来源SHA及在途状态见[指纹](attachments/extraction-followup-20261017/inputs.json)。本计划不消费AST/方言API，没有799/801/803硬依赖。

802 latest readiness复审为needs_fix（V06实机证据未齐），生产实现4029dd705仍未合入；803也未合入，起草期间继续复审/修复。均不能按已交付接口消费。本计划与802共用NI/VM/统一安装和native.rs，但改不同族：可在独立树准备新文件/定向实现，共享安装/槽接线短窗口串行；推荐802先合，后合入者取并集、重核受影响证据，不设虚假的AST前置。

源码中dialog_parent及rfd构建在vm/native.rs，ui/dialog.rs另有近似open/save供a2r使用。两者过滤规则相同，但VM带父窗、a2r旧版本无父窗。ui-dialog只依赖rfd和raw-window-handle，不要求ui；不能为方便把adapter全藏在ui门后。Dnd文件头仍称专用STA线程，实际start_drag及shim明确要求收到按下的输入线程同步DoDragDrop；以实际实现与已落地合同为准，SD纠正文档陈旧描述。

## 5. 详细设计

T01冻结三项参数/栈/feature/缺席矩阵、装配入口和线程/事件链。Dialog无provider保旧feature-off VMError文本/栈效应；在位provider取消或不可用返回空串。Dnd无provider、坏payload/空payload/已有拖出重入返false。原start_drag进入会话后无论落下还是取消，返回true表示执行过会话，完成效果另由on_dnd_finished报告，**不能把cancel映射为false或把true当drop成功**。

DialogHostProvider接原始filter/default_name，公共过滤语义只在单一OS实现维护：逗号/分号/空格/tab分割、去前导点、空项丢弃、无扩展不加filter；路径返回to_string_lossy和空串取消保持。搬出原dialog_parent及测试，ui::dialog与VM adapter只选择不同parent策略，不保留两份构建/过滤/父窗算法。旧a2r公开路径可编译且parent差异诚实保留。

Dnd provider接owned JSON，复用parse_payload_json的text/files/virtual_files、base64与裸反斜杠重试规则；实际DoDragDrop仍在输入线程阻塞，释放会话/载荷沿旧实现。完成通道、session代号、发起者路由和事件消费保持；不得改成spawn/async提前返true。拖入/窗口注册生命周期仍原owner，不趁本轮重构。

统一宿主入口补两槽，明确ui-dialog-only（无ui）的高层兼容安装，原始AutoVM仍按既有provider模式只快照，不暗装UI。显式default/instance恒胜兼容默认。与802/799/801的共享窗口、792source闭包及合流责任见[本批编排](reports/extraction-followup-parallel-order-20261017.md)。

### 规范增量

| delta_id | add/modify/retire | docs/specs target | before/after rule | rationale | acceptance IDs |
|---|---|---|---|---|---|
| SD-01 | add | docs/specs/auto-lang/ui/design/dialog-dnd-host-capabilities.md | 三native owned边界/降级/线程/完成与父窗策略 | 不把模态/重入当普通异步调用 | AC-01/02/03 |
| SD-02 | modify | docs/specs/auto-lang/vm/overview.md | VM直调OS/UI→两独立provider/实例装配 | 本族删边 | AC-01/02 |
| SD-03 | modify | docs/specs/auto-lang/ui/overview.md | adapter归属/feature-only路径和真实拖出线程 | 纠正陈旧线程叙述/如实登记剩余族 | AC-03/04 |
| SD-04 | modify | docs/specs/auto-lang/ui/design/shell-a2r-seams.md | dialog共享实现、VM/a2r旧parent策略不变 | 去重与兼容分开 | AC-02/04 |

## 6. 测试设计

work→同树独立final review→merge，无额外agent/模型指定。拟唯一树D:/autostack/.wt/lang-804/auto-lang，branch codex/plan-804-dev；work登记实际base/HEAD/feature。新fixture由T04建立，本轮未运行。涉及UI交互使用autoui-verifier现有工具；OS模态/拖出证据另用可复跑的真实窗口步骤，不冒充MCP控件测试。

| Case / owner | AC / 不变量 | 输入/真实入口 | oracle与边界 |
|---|---|---|---|
| V01 / work | 01 no-ui真供值 | 新test/fixtures/modularity/dialog-dnd-provider-consumer；真VM三ID，互异raw filter/default/JSON与Unicode返回 | 参数/返回与trace一致，原buffer销毁后VM字符串有效；无UI/OS feature也可显式供值，闭包无renderer/rfd；非假adapter直测 |
| V02 / work | 02 缺席/取消/形态 | 无provider、两槽只装一个、feature-off、取消、坏/空JSON、重入、text/files/virtual_files载荷 | 具体值/错误/栈效应与旧臂一致；缺席dnd=false，进入后取消=true且完成effect=none；不混淆会话与成功 |
| V03 / work | 02/03 装配/重入寿命 | 两VM、default变更、NI clone/merge、显式选择遇兼容安装；provider回调再访问/设置槽 | 无覆盖/串线/deadlock，锁不跨回调，drop释放；binding/proof规则未变；Send+Sync不据此宣称跨线程COM |
| V04 / work | 02/03 真dialog | Windows真实UI程序open选文件/save选目标/两者取消；空/混合filter、Unicode临时目录；ui-dialog-only高层入口；a2r旧路径 | 返回路径/空串正确，VM窗口在模态结束后可操作、parent策略同旧；a2r仍原策略；组件mock不能替代真窗口与ui-dialog-only编译/接线 |
| V05 / work | 03 真拖出 | 真实输入线程启动，拖出文本/现有文件/虚拟文件至测试接收窗口，并取消一次 | 接收内容/落地bytes及完成事件归发起者；调用线程ID相同，返回在会话结束之后，资源/拖旗标复原；不能用直接调用COM构造器替代 |
| V06 / work | 04 单一性/合流 | 源码清单/777/792、802实际合入差异、旧a2r调用 | native本族无rfd/Win32/UI直引，对话框构建唯一；其他provider未回退；792闭包/构建输入变化按其真实入口更新并重放受影响fresh/locked/重定位/保护控制 |

work仅定向provider/装配/OS反控、`cargo check -p auto-lang`、新consumer manifest test；ui-dialog-only用`cargo check -p auto-lang --no-default-features --features ui-dialog`验证。真OS证据未齐不得execution_done或pass，环境缺口保留blocked项，不降低V04/05。最终review后merge冻结裸cargo t+tv；实际修改ui_gen才加tu。tf主检出到期单实例，work/review不跑整档。本轮Category A不跑Cargo/docs_gen。

## 7. 验收标准

| ID | owner_stage | 可观察结果 |
|---|---|---|
| AC-01 | work | 三项生产shim只编解码/委托，两槽无UI/OS实现依赖，V01通过 |
| AC-02 | work | 固定ID/栈/错误/取消/feature-only、字符串寿命和旧a2r路径保持，V02/03/04通过 |
| AC-03 | work | 真实模态/拖出线程/完成/资源、装配隔离证据完整，V03–05通过 |
| AC-04 | work | 单一对话框实现、剩余边/迁位净删、共享合流/792及SD素材完整，V06通过 |
| AC-05 | review | 同树独立逐项AC/SD/遗漏复审通过，绑定提交，merge门pending |
| AC-06 | merge | 冻结t/tv及必要触面门、Spec/ledger/归档、guard清理完成 |

## 8. 执行步骤

| 状态 | ID | owner_stage | 依赖 | 路径/交付与验证 | AC |
|---|---|---|---|---|---|
| [ ] | T-01 | work | 既有provider交付、合同确认 | native.rs/dialog_parent、ui/dialog.rs/native_dnd.rs、NI/VM/组合入口：旧矩阵/线程/取消及802共享窗口决策 | 01/02/03 |
| [ ] | T-02 | work | T01 | host_capabilities两provider/槽、NI/VM快照/clone/merge、三shim，check及定向V01/02 | 01/02 |
| [ ] | T-03 | work | T02、共享接线窗口 | 新host_adapters/dialog.rs、UI dnd adapter、ui/dialog薄门面和宿主装配，删除VM OS实现；V03/feature-only | 02/03/04 |
| [ ] | T-04 | work | T03 | 新独立consumer及旧a2r/OS组件/装配反控，V01–04；记录filter/fixture实际命令 | 01/02/03 |
| [ ] | T-05 | work | T04 | V04真模态+V05真拖出，V06 777/792/802合流验证，必要证据必须齐 | 03/04 |
| [ ] | T-06 | work | T05 | 来源/删边/去重/剩余族/SD收据，提交冻结代码送审 | 04 |
| [ ] | T-07 | review | T06 | 同树独立AC/SD和OS证据审查，merge门pending | 05 |
| [ ] | T-08 | merge | final pass | 同步master、受影响复审/冻结门、沉淀/归档/wt-guard清理 | 06 |

## 9. 复审记录

2026-10-17 / stage: new / PLAN-804 revision: 1 / outcome: pass / next: 合同确认后work。无799/801硬前置；802未交付的provider接口不作基础，本族沿已交付模式增量实现，共享写面短窗口串行。[本轮预检](attachments/extraction-followup-20261017/preflight.md)。

## 10. 待澄清事项

无外部决策阻塞。T01核实真实OS证据取得路径；若环境不能完成必验，记录条件/缺口，不重复802的组件证据替代实机问题。此次只建计划，不运行OS对话框/拖拽，也不操作真实用户文件。
