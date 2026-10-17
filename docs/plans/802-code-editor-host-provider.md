---
plan_id: PLAN-802
status: executing
feature_name: CodeEditor native 全族宿主能力迁移
author: [Codex]
created_at: 2026-10-17
updated_at: 2026-10-17
plan_revision: 1
current_step: 6
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
| [x] | T-01 | work | 785/790/794已合、合同确认 | 22 项旧臂/缺席/单位/弹序/方法粒度冻结：[t01-frozen-matrix.md](attachments/802/t01-frozen-matrix.md)（19 方法↔22 native 全映射；scroll_to 唯一值缺席臂；cut/copy/paste Option<bool> 组合特征臂；catalog/codegen 零改动裁定） | 01/02/03 |
| [x] | T-02 | work | T01 | host_capabilities CodeEditorHostProvider（19 方法）/NativeInterface 槽+Clone+merge/AutoVM 快照+实例覆盖/22 shim 单态化委托；`cargo check` default+no-default PASS、定向 27/27（[收据](attachments/802/t06-work-receipt.md)§二）；R2 格式门过（`b32d4df7d`） | 01/02 |
| [x] | T-03 | work | T02 | 新 ui/code_editor_provider.rs（真注册表 adapter，剪贴板实现级第二门控返 None）+ 四漏斗装配（install_ui_host_capabilities/run_file_dynamic_ui_inner/VmBridge 两构造）；compat 单测 PASS；R2 格式门过 | 03 |
| [x] | T-04 | work | T03 | 新 no-ui consumer fixture（7 测试）+ in-crate plan802（7 测试）；施工期发现预存局部槽≥16 限制（P802-D1，非本改面）；fixture/测试 R2 格式门过 | 01/02/03 |
| [x] | T-05 | work | T04 | **R2 修复轮**：驱动 oracle 重写（clamp 按 Reader 实测几何对拍、偏差 0.200px/Reader 结构化必验门禁/滚轮 Reader 直读+tick 下一拍捕获身份/诚实描述无同帧声明），三态 mock 自检先行（错误上界与缺 Reader 均正确拒绝），冻结基线实机必验 12+辅助 1 全 PASS exit 0；R1 轮定位并修复预存 VM 轨 bind 缺口（`ad1427e33`）；V07 777 口径 52→23/a2r 臂未动/792 闭包不变 metadata 证明（[收据](attachments/802/t06-work-receipt.md) R2/R1 轮） | 03/04 |
| [x] | T-06 | work | T05 | 22映射/剩余边/净删与SD收据齐（[t06-work-receipt.md](attachments/802/t06-work-receipt.md) 含 R1/R2 修复轮）；冻结代码 `b32d4df7d` 送审（worktree clean，实机二进制 SHA256 与证据 JSON 一致）；merge 门 pending | 04 |
| [ ] | T-07 | review | T06 | 同树独立反控/AC/SD审查，merge门保持pending | 05 |
| [ ] | T-08 | merge | final pass | 同步master、受影响复审/冻结门、Spec/ledger/归档/wt-guard清理 | 06 |

## 9. 复审记录

2026-10-17 / stage: new / PLAN-802 revision: 1 / outcome: pass / next: work（合同确认后）。仅合同预检通过，T/AC未实施；与799/800/803主写面分离，见[预检](attachments/extraction-next-20261017/preflight.md)。

2026-10-17 / stage: work / PLAN-802 revision: 1 / T-01 完成：22项旧臂矩阵冻结（attachments/802/t01-frozen-matrix.md），取材 1dd123361；worktree 拟建 `D:/autostack/.wt/lang-802/auto-lang`、branch `plan-802-dev`（AGENTS.md 仓规命名——计划 §6 原拟 `codex/plan-802-dev` 系起草方前缀，路径/唯一树不变，非合同要素）。next: T-02。

2026-10-17 / stage: work / PLAN-802 revision: 1 / outcome: pass（work-owned 全完成，送 final review） / worktree `D:/autostack/.wt/lang-802/auto-lang` / branch `plan-802-dev` / base `1dd123361` / code `4029dd705` / deps: auto-down sibling `c39ac7f` detached 只读 / task_ids: T-01..T-06 / evidence: [t06-work-receipt.md](attachments/802/t06-work-receipt.md)（fixture 7/7 + in-crate 7/7 + 定向 27/27；缺席矩阵逐字面值；真 controller/arboard 组件验证；vm/native.rs `crate::ui` 52→23 本族生产边清零；792 闭包不变 metadata 证明；a2r 臂未动）/ blockers: 无阻塞；如实登记 1 条件缺口（V06 live iced 窗口消费链未跑实机，补验条件已记）+ 1 施工期预存发现（局部槽≥16 限制→KNOWN-DEBT P802-D1）/ merge 门 pending：裸 `cargo t`+`cargo tv`（未触 ui_gen 故无 tu；无 aavm 触面）由 merge 在冻结提交执行 / next_responsible_role: review（T-07 同树独立反控/AC/SD 审查）。

## 10. 待澄清事项

无需等待799或800。T01负责每项错误/栈矩阵和方法粒度，不能借未定义删除native。真实OS/renderer必要证据受环境阻塞时记录缺口/条件，不能把模拟改称真实验收。

2026-10-17 / stage: review / review_scope: final / PLAN-802 revision: 1 / outcome: needs_fix（readiness） / reviewed_commit: 4029dd705a1f3d4e39ddbffbb6c5f426c402c2a6 / finding: F802-R1-01（P2，AC-03/V06 必需 live iced 滚动消费与≤2s收敛验证未完成；controller 测试人工 note-back 不能替代） / evidence: [R1报告](attachments/802/review-r1.md) / T-05、T-06 重开，executing 4/8；T01..04保留 work 声明，完整 AC/SD 复审 pending / Git status 受工作树访问问题未确认，未声称 clean / merge-owned 裸 t/tv 与收尾保持 pending / next: 原树 work 补齐必验证据再送审。

2026-10-17 / stage: work（R1 修复轮） / PLAN-802 revision: 1 / outcome: pass（F802-R1-01 闭合，重新送审） / worktree `D:/autostack/.wt/lang-802/auto-lang`（git status clean，`git rev-parse --show-toplevel` 已核验）/ branch `plan-802-dev` / base `1dd123361` / code `ad1427e33`（=4029dd705 + renderer VM 轨 bind 一行修复，diff 1 文件 +12/−1）/ deps: auto-down sibling `c39ac7f` detached 只读 / 闭合：V06 实机探针（[v06-live/](attachments/802/v06-live/)，MCP+Win32 真滚轮）8/8 PASS exit 0——程序化 150 命令值直达无中间值、越界 clamp 7541（=8020−479 精确）、真滚轮回填 0.713s≤701 2s、未绑定反控、ScrollStateReader 观测；实机过程中定位**预存** VM 轨 bind 缺口（render_dynamic_view 臂自 PLAN-629 未绑注册表→scroll_to 写端点 VM 轨恒 no-op；旧 shim 同队列，非 802 引入）并修复（ad1427e33，镜像 29480 desktop 臂既有 bind）/ 回归：plan802+code_editor_natives+plan794+plan728 21/21 PASS / 施工期发现（P802-D1 局部槽≥16 预存限制）维持 KNOWN-DEBT / merge 门 pending：裸 `cargo t`+`cargo tv` 由 merge 在 ad1427e33 冻结基线执行 / next_responsible_role: review（T-07 完整 AC/SD 复审，含 R1 finding 闭合核验）。

2026-10-17 / stage: review / review_scope: final / PLAN-802 revision: 1 / outcome: needs_fix / reviewer: 当前独立审查会话（沿用R1上下文） / worktree: D:/autostack/.wt/lang-802/auto-lang / branch: plan-802-dev / reviewed_commit: ad1427e3356097f91f97d3b8f76eea1e44ec5e42 / base: 1dd123361a75aa4b683376dcb2049299122724c9 / deps: auto-down c39ac7f9dacdc769fc4a5ad3b98ec896240b28e8 / HEAD及clean前后核验通过 / evidence: [R2完整报告与反控](attachments/802/review-r2.md) / 独立consumer 7/7、定向27/27通过；F802-R1-01部分闭合，新增F802-R2-01（V06错误clamp和Reader失败均可exit0，原trace不支持同帧直达声明）、F802-R2-02（新增Rust格式红）开放 / AC01/02功能pass、03/04 partial、05 fail、06 pending / T02..06重开，executing1/8 / SD快照与输入hash见报告，canonical/ledger未写；merge-owned裸t/tv及收尾pending / next: 原树work修复；第二轮non-pass，下次需附诊断与全不变量闭合表再送审。

2026-10-17 / stage: work（R2 修复轮） / PLAN-802 revision: 1 / outcome: pass（F802-R2-01/02 闭合，重新送审） / worktree `D:/autostack/.wt/lang-802/auto-lang`（clean）/ branch `plan-802-dev` / code `b32d4df7d`（=ad1427e33 + 四新增文件 rustfmt，1 提交 4 文件 +52/−31；实机二进制由此源构建，SHA256 d916ff74… 与证据 JSON 一致）/ **诊断（R2 要求）**：R1 版驱动把三类必验写成了便利断言——clamp 上界用数值区间（150<v<99999）而非几何推导，故任意错误上界（如 2000）通过；Reader 观察用子串包含且 record() 未并入 ok_all，故 `{}` 也 exit0；"命令值直达"是从异步采样 trace 过度推断（trace 实为多轮 0 后突变 150，只能证命令值收敛，不能证同帧）。根因是把"有一份正向样本"当成了"oracle 能拒绝错误行为"。**闭合表**：(a) clamp 上界 = Reader pane 实测 content−viewport、±1.0px 对拍（本次 7540.800，偏差 0.200px）+ 复采稳定 + Reader 可见偏移同值；(b) Reader 观察结构化必验（按键命中/数值齐全/静止点与读端点 ±1.5px 一致），失败进非零退出，截图降为显式辅助；(c) 程序化项只声明"命令值收敛+无中间落点"，同帧声明撤回；用户滚动计时改 Reader pane 直读（单跳）+ 观察节拍计量 + **tick 身份**（滚轮由下一读回拍捕获，无漏拍），预算=通道 2s+实测量子（1.243s），实测 1.336s；(d) 三态 mock 自检先行——valid 过、错误上界拒、缺 Reader 拒（`--selftest`，判定逻辑与实机共用同一 run_oracle 路径）；(e) 格式：四处新增文件 rustfmt 全过。**区分**：旧例闭合=R1 缺口（实机证据）已交付；全不变量闭合=oracle 经负控证明会拒绝错误行为；修复回归=定向 27/27 + fixture 7/7 + fmt check 全绿。实机必验 12+辅助 1 全 PASS exit 0（[证据](attachments/802/v06-live/v06-evidence.json.txt)）。/ merge 门 pending：裸 `cargo t`+`cargo tv` 由 merge 在 b32d4df7d 执行 / next_responsible_role: review（T-07；第三轮，含 R1/R2 finding 闭合核验）。
