---
plan_id: PLAN-802
status: drafting
feature_name: CodeEditor native 全族宿主能力迁移
author: [Codex]
created_at: 2026-10-17
updated_at: 2026-10-17
plan_revision: 1
current_step: 0
total_steps: 8
supersedes_spec_components: [docs/specs/auto-lang/vm/overview.md, docs/specs/auto-lang/ui/overview.md, docs/specs/auto-lang/ui/design/code-editor-session-endpoints.md]
new_spec_components: []
touched_goals: [GOAL-003, GOAL-007]
affects: [auto-lang/vm, auto-lang/ui]
depends_on: [PLAN-785, PLAN-790, PLAN-794]
---

# [PLAN-802] CodeEditor native 全族宿主能力迁移

## 0. 变更摘要

将22个code_editor native由VM直引UI迁为per-VM owned宿主接口，UI adapter委托现有editor/scroll/clipboard。真实no-ui VM可注入自定义editor，现有UI宿主自动装配保持。交付完整有界族的生产迁移，不只做pilot。

## 1. 目标

消除本族VM shim→ui::code_editor及ui::scroll实现边，保留固定ID/签名、栈编解码、错误/值降级、delta水位和编辑/文件/光标/滚动行为；跨界无rope/renderer/VM heap句柄。

非目标：迁editor引擎或iced、改变撤销/folding算法、修复所有产品问题、重写794 provider、迁term/media/dialog/dnd或宣称runtime已独立。a2r/merged直接UI臂保留同源语义，不强改为VM provider。

## 2. 架构方案

沿779/785/790/794三层装配：实例覆盖、构造期默认快照、UI兼容默认if-absent同锁补缺；后改默认不影响旧VM，NI Clone保provider、VM merge保目标provider，注入不改变binding_revision/proof规则。core仅owned值/状态/错误分类，UI adapter统一委托editor registry及scroll/OS clipboard。

UI全局registry仍单一事实源；per-VM provider隔离不承诺多租户UI registry。clone/替换/drop不清空UI文档，不持跨回调锁/借用；delta有副作用，必须恰好消费一次。

## 3. 技术栈

Rust host_capabilities、NativeInterface/AutoVM、code-editor/ui-clipboard feature、ui/code_editor/core及ui/scroll；独立no-ui consumer、真实adapter定向测试与必要实机交互，不新建RPC/编辑引擎。

## 4. 需求分析与背景调查

用户2026-10-17授权继续建立迁移合同。本轮仅规划，不执行Rust、不创建agent/树、不改canonical Spec/旧仓。初次取材`143aff0a258fdca8bda9a3b8068dc8a9cced623c`，规划期间复核至`2d246b4e86c1ebf5138f6a316c1896e56a5d35f8`，800已归档、799 executing；相关editor源码未被800改变，见[指纹](attachments/extraction-next-20261017/inputs.json)。785/790/794已归档，无799/800硬依赖，可与799/803并行；共享接线见[编排](reports/extraction-next-parallel-order-20261017.md)。

native_catalog.rs核对22项；native.rs含feature开/关分支，不能只迁菜单六项。701端点Spec冻结save原始rope字节、光标0基/char列、scroll投影与≤2s收敛；673 delta是destructive read，selection_len是bytes，fold入口是1基行，不能统一单位。

缺席并非全部报错：scroll_to关闭分支弹参数返false；load_file失败返-1，save失败返false；cut/copy/paste还受ui-clipboard约束。T01按全部实际分支冻结矩阵，不能照文本剪贴板族一刀切。9907归Env.track，不得占用。

## 5. 详细设计

完整集合：2910–2915、2919–2924、2932–2933、2939、9906、9908–9913，共22项。canonical名/签名不变，不分配新ID；catalog/codegen只作必要兼容适配。

T01逐项冻结参数顺序、位宽/编码、有效/缺key、feature-off、无provider、provider失败及UI默认。**无provider镜像原feature-off分支**；显式在位provider使no-ui也能供值。保存具体错误/返回值及弹栈行为，不把缺席当成功，UI不可用按原失败语义处理。

接口使用具名语义操作，禁止字符串method总分发和UI通用对象句柄：text/delta返回owned String，光标/选择/查找使用原标量或owned DTO，操作返回原值形/错误分类；不暴露CodeEditorCore/scroll intent/rope/ClipboardOp。具体方法划分T01登记，全22项必须有映射。

delta每次只取一次队列，保持revision/水位/JSON；text不得顺带清队列。edit负offset、反区间、越界或非UTF-8字符边界返false，成功进入原人机统一delta队列。光标0基行/char列、选择bytes、fold入口1基保持。save直接UI rope→文件，无全文VM字符串往返，BOM/EOL职责按701保持。

editor滚动三项由UI adapter委托editor_scroll_handle及原controller/intent，不误转为794通用scroll参数协议。scroll_to true仅入队，未绑定丢弃、同帧投影、后续clamp及≤2s用户滚动收敛不变。文件IO/OS clipboard留UI实现，复用现有剪贴板逻辑，不复制系统桥或状态。

装配覆盖install_ui_host_capabilities、高层run/解释器/config/REPL/back-proxy漏斗、VmBridge两构造与CLI相关入口，显式默认恒胜；原始AutoVM仍只快照，不回接UI。共享接线/777规则用精确片段/边，不增加ui整模块allow。

### 规范增量

| delta_id | add/modify/retire | docs/specs target | before/after rule | rationale | acceptance IDs |
|---|---|---|---|---|---|
| SD-01 | modify | docs/specs/auto-lang/vm/overview.md | editor直引UI→22项owned provider、缺席矩阵/装配/寿命 | core本族删边 | AC-01/02/03 |
| SD-02 | modify | docs/specs/auto-lang/ui/overview.md | editor未迁→adapter单一registry与宿主装配，其他族仍未迁 | 不夸大解耦进度 | AC-03/04 |
| SD-03 | modify | docs/specs/auto-lang/ui/design/code-editor-session-endpoints.md | shim直引→provider后VM/a2r仍同源，单位/IO/收敛不变 | 保持消费者合同 | AC-02/03 |

## 6. 测试设计

拟唯一树`D:/autostack/.wt/lang-802/auto-lang`、branch `codex/plan-802-dev`。work→同树独立final review→merge，无额外agent或模型指定。新fixture由T04/T05创建，不是本轮已执行证据；UI交互用autoui-verifier现有技能/工具，不能用fake代替真实adapter。

| Case / owner | AC / 不变量 | 输入、实际入口 | oracle与覆盖边界 |
|---|---|---|---|
| V01 / work | 01/02 真VM全族 | 新test/fixtures/modularity/code-editor-provider-consumer/Cargo.toml，无UI feature；22项互异参数及可区别provider值，经真VM/AutoTask | ID/栈顺序/返回形全对；metadata闭包无renderer，源shim无UI引用；同类型参数交换被trace抓到 |
| V02 / work | 02 兼容矩阵 | 旧feature-off/缺key/无provider/失败/可用格；负/大offset、多字节和空文档 | 具体值/错误文本/栈效应对旧臂；scroll_to关闭=false、load=-1、save=false等不统一成VMError；9907不变 |
| V03 / work | 02 delta/ownership | 多次edit→delta→空delta，穿插text，原buffer销毁/换provider/VM复用 | revision/内容/FIFO同旧臂，空队列保水位，delta恰好调一次；buffer释放后VM字符串有效 |
| V04 / work | 03 装配/寿命 | 两VM互异provider、构造后改默认、NI clone/VM merge、显式默认对兼容安装、替换drop | 无串线/覆盖，binding_revision/proof不变，锁不跨回调，资源释放且旧VM保持快照 |
| V05 / work | 02/03 真adapter | 真editor多字节编辑/光标/选区/undo/redo/fold；临时文件load/save含CRLF及失败 | 实际rope/选择/delta/磁盘字节改变且与原UI函数同源；save无全文VM字符串；不只断言fake计数 |
| V06 / work | 03 scroll/clipboard | 真实editor scroller/renderer drain，cut/copy/paste走OS桥且保存恢复剪贴板 | 入队/投影/可见偏移/clamp符合701；未绑定反控；copy不删/cut删/paste插，内容正确；必要实机未跑是缺口，不能标pass |
| V07 / work | 04 剩余边/交付 | 777审计、22映射/feature/装配覆盖、792代表生成consumer | 本族零core→UI边，其他族仍登记，a2r臂未误删；闭包改变按792 driver重验受影响fresh/locked/重定位和失败保护 |

work命令：`cargo check -p auto-lang`，新consumer `cargo test --manifest-path test/fixtures/modularity/code-editor-provider-consumer/Cargo.toml`及定向provider/editor测试（T04记录实际filter/选择数），UI adapter与交互记录。无UI/feature-off和UI-host证据分开，不假定所有feature组合原本都能编译。final review通过后merge冻结跑裸`cargo t`+`cargo tv`；实际改ui_gen才追加单进程`cargo tu`；tf归主检出批量单实例，work/review不整档。本轮Category A不跑cargo/docs_gen。

## 7. 验收标准

| ID | owner_stage | 可观察结果 |
|---|---|---|
| AC-01 | work | 22项生产shim无UI直引，独立no-ui VM能供值，V01通过 |
| AC-02 | work | ID/栈/缺席错误、单位/IO/delta/ownership保持，V02/03/05通过 |
| AC-03 | work | 显式装配、实例/clone/merge寿命、真UI/scroll/clipboard可验，V04–06通过 |
| AC-04 | work | 迁位/净删/剩余边、792触面及SD证据齐全，V07通过 |
| AC-05 | review | 独立AC/SD及遗漏扫描通过，绑定提交/feature/依赖，merge门pending明确 |
| AC-06 | merge | 冻结t/tv及实际触面门、Spec/ledger/归档和guard清理完成 |

## 8. 执行步骤

| 状态 | ID | owner_stage | 依赖 | 已核对路径/交付与验证 | AC |
|---|---|---|---|---|---|
| [ ] | T-01 | work | 785/790/794已合、合同确认 | vm/native.rs/native_catalog.rs、ui/code_editor/core和701Spec，全22项旧臂/feature/栈/装配及端口清单 | 01/02/03 |
| [ ] | T-02 | work | T01 | vm/host_capabilities及NativeInterface/AutoVM，owned provider/全部shim委托，check/定向V01/02 | 01/02 |
| [ ] | T-03 | work | T02 | 新ui/code_editor_provider.rs（拟名）、统一安装及实际宿主入口，复用editor/scroll/clipboard，V04 | 03 |
| [ ] | T-04 | work | T03 | 新no-ui consumer、VM/UI测试V01–05，delta/drop/真load/save/undo/fold及负控 | 01/02/03 |
| [ ] | T-05 | work | T04 | V06真UI滚动/OS剪贴板，V07 777边界/792触面，共享窗口串行 | 03/04 |
| [ ] | T-06 | work | T05 | 22映射/剩余边/净删与SD收据，提交冻结代码送审 | 04 |
| [ ] | T-07 | review | T06 | 同树独立反控/AC/SD审查，merge门保持pending | 05 |
| [ ] | T-08 | merge | final pass | 同步master、受影响复审/冻结门、Spec/ledger/归档/wt-guard清理 | 06 |

## 9. 复审记录

2026-10-17 / stage: new / PLAN-802 revision: 1 / outcome: pass / next: work（合同确认后）。仅合同预检通过，T/AC未实施；与799/800/803主写面分离，见[预检](attachments/extraction-next-20261017/preflight.md)。

## 10. 待澄清事项

无需等待799或800。T01负责每项错误/栈矩阵和方法粒度，不能借未定义删除native。真实OS/renderer必要证据受环境阻塞时记录缺口/条件，不能把模拟改称真实验收。
