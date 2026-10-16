---
plan_id: PLAN-785
status: archived
feature_name: 文本剪贴板宿主能力依赖倒置
author: [Codex]
created_at: 2026-10-15
updated_at: 2026-10-16
plan_revision: 1
current_step: 8
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
| [x] | T-01 | work | 784+781交付/热点释放+确认 | 冻结feature/调用/encoding与ownership矩阵，bounded接线决策和反控输入。[✅ 已完成：附件 attachments/785/t01-freeze-matrix.md——784 已归档；781 未合入但热点重叠实测=engine.rs 单文件不相邻 hunk（781 触 L39/3055/13024，785 触 L816/1240），裁定开工+登记 final review 前同步 master 残余义务；{ui-clipboard} 单开实测不可建（E0433）→单体 shim 后新可建（缺席错误语义，已登记）；786 归档零重叠] | 01/02/03 |
| [x] | T-02 | work | T01 | host_capabilities/NativeInterface/AutoVM接文本接口、快照/覆盖和两shim，core实现直引清零。[✅ 已完成：commit 64c72d20c——ClipboardTextProvider（owned String 出/&str 入）+进程默认槽4 API+NI实例槽（Clone保留）+AutoVM::new快照+实例API；两 shim 单体化（缺席=原 feature-disabled 错误文本逐字）；core 直引 `ui::clipboard::` 清零（plan785 剪断锚常驻）] | 01/02 |
| [x] | T-03 | work | T02 | 新独立consumer及V01..04，字符串释放/失败与双feature形态实际门。[✅ 已完成：test/fixtures/modularity/clipboard-provider-consumer/（独立 workspace、no-ui）10/10——UTF-8 精确参数/相邻读写ownership/缺席矩阵（错误文本逐字）/优先矩阵/identity+merge；UI 形态经 workspace 锁定 rlib 重放（779 R3 法，attachments/785/ui-form-replay/）10/10 exit0，frozen_source_unchanged=true——独立 UI 锁解析漂移（wgpu-hal）如实登记不冒称（779 同款既有债）] | 01/02 |
| [x] | T-04 | work | T03 | UI clipboard adapter接全部受支持宿主；V04/05、保留779/未迁族。[✅ 已完成：ui/clipboard_provider.rs ArboardClipboardProvider（纯委托既有函数+plan418 既有测试不动）+优先关系单测；六处宿主接线（统一 installer/run_file_dynamic_ui_inner/VmBridge×2/auto CLI main/autovm_persistent 经统一）；plan779 4/4+779 fixture 6/6+clipboard 族 21/21 全绿（未迁 files/image/editor 族零改动）] | 03 |
| [x] | T-05 | work | T04 | 定向encoding/ownership/contract/入口回归+check/fmt，无整档。[✅ R2 重闭 2026-10-16：缺席锚改构造式（engine.rs new_inner+cfg(test) new_without_host_snapshot），in-crate shim 缺席测试零全局槽操作；R2 混合反例命令集（clipboard_provider+frame_provider+host_capabilities+plan320_tests 21测 threads=16）30/30 绿+合并提交 10/10，原过滤 30/30；日常档 plan785 6/6（含新 hammer）/plan779 4/4；fixture no-ui 10/10+UI 重放 10/10（head=c49ca102a）；check 0 错+触面 rustfmt 清 | 01/02/03 |
| [x] | T-06 | work | T05 | 实现提交、永久案例与覆盖记录、SD素材，确认所有work项闭合。[✅ R2 重闭 2026-10-16：修复 commit cb5fa865b + master 同步 merge c49ca102a（781 重查仍未合入、R08-C 只触 ui/iced/view_tree 区，不相交保持）；永久组合反例=absence_anchor_survives_concurrent_implicit_installs（构造式缺席×普通高层入口并发，架构级钉面）；诊断/覆盖调整表见 §9 R2 记录；merge-owned 门禁保持 pending（冻结提交裸 cargo t+tv，T-08） | 01/02/03 |
| [x] | T-07 | review | T06 | 独立final，不能以默认0/空串/跳过OS代替注入证据 | 04 |
| [x] | T-08 | merge | review pass | t+tv冻结门、SD/ledger/归档合流/清理。[✅ 已完成：rebase 全等 2431eb300（range-diff 3/3 =；crates/test 与受审提交零差异；映射 64c72d20c→d03ea0cd8/012c89b99→dea54b85f/cb5fa865b→2431eb300）；SD-01/02 沉淀 c8f697009（源 hash 与 R3 冻结 delta 吻合）；冻结门禁 裸 cargo t 5371/5371（72.2s）+ tv 163/163（5.17s）；合入收据 ecce1dc8c ff-only 落 master；ledger v28→v29 受管离线投影（+P785-D1/D2/T/R3/M，五护栏过）；归档+清理收据见 §9 合并记录] | 05 |

## 9. 复审记录

2026-10-15 stage=new | plan_revision=1 | outcome=pass（草案preflight） | next=前置交付及合同确认后work | 0/8，无实现/测试/树。已明确frame与字符串能力差别；T01仅核定接线机制，不允许修改既有缺席/编码合同来消红。

2026-10-16 stage=work（T-01..T-06 收口 → execution_done）：stage=work | plan_id=PLAN-785 | plan_revision=1 | worktree_path=D:/autostack/.wt/lang-785/auto-lang（组内兄弟 auto-down detached@c39ac7f） | branch=plan-785-dev | base_commit=340ab95fd（master，含 786 归档；786 仅动 auto-man 零重叠） | code_commit=64c72d20c | dependency_revisions=auto-down 主检出 c39ac7f（依赖仓只读）；PLAN-781 未合入——热点重叠实测裁定见 attachments/785/t01-freeze-matrix.md §4（engine.rs 同文件不相邻 hunk，781 触 L39/3055/13024 vs 785 触 L816/1240；native_catalog.rs 785 零改动），准予开工+残余义务：**785 final review（T-07）前同步 master，若 781 已合入则吸收并复跑受影响门禁；若 engine.rs 冲突按 781 语义优先解** | task_ids=T-01..T-06（6/8；余 T-07 review/T-08 merge） | evidence=T01 冻结矩阵+接线决策+反控输入（attachments/785/t01-freeze-matrix.md：{ui-clipboard} 单开实测不可建→单体 shim 后新可建且缺席错误语义、{ui}/{ui,ui-clipboard} 预存 8 错与本计划无关、.at 面零触点）；T02..T05 实现+门禁：fixture no-ui 10/10 + UI 锁定重放 10/10（attachments/785/ui-form-replay/ui-consumer.json，frozen_source_unchanged=true，源 sha256=77c58ae5…）+ in-crate plan785 5/5 + plan779 4/4 + clipboard 族 21/21 + host_capabilities 1/1 + 779 fixture 6/6 + check 三形态 0 错 + 触面 rustfmt 清 | 覆盖记录：V01=fixture v01×2（stdout 实值+低层双活+UTF-8 精确+不串线）；V02=v02×3（临时 owned String 相邻读写/空串/None/false 不计成功）；V03=v03×2+in-crate shim 直调（错误文本逐字/no-ui 高层 Err/ui 补缺成功差分/裸 VM 零输出）；V04=v04×2+in-crate 高层优先+adapter 单测（显式不被覆盖/空槽补缺/实例覆盖独立）；V05=fixture v05+in-crate merge 保 provider（2925/2926 钉扎、邻族不撞、注入不 bump revision、Clone/merge 不丢、覆盖 shim 端到端） | 真实 OS adapter 证据=纯一行委托+plan418 既有 roundtrip（headless 自跳过）+UI 重放读侧成功（内容不定差分 oracle；红线遵守：全程未新增任何写用户剪贴板的验收步骤；首轮断言失败时 panic 消息意外回显真实剪贴板内容，已从日志净化） | 预存发现登记=KNOWN-DEBT F-785-P01（{ui}/{ui,ui-clipboard} 预存不可建+779 文档化 ui 复跑命令已烂）/F-785-P02（examples/ui_container.rs 预存 E0765 使全仓 cargo fmt 不可跑+仓级 fmt 漂移）；独立 UI 锁解析漂移沿 779 R3 同款登记（跨仓发行依赖身份验证，不冒称发行通过） | blockers=无（work-owned AC01..03 全闭；AC04 归 review、AC05 归 merge；merge-owned 门禁显式 pending：冻结提交裸 cargo t+tv——本计划改 VM 未触 ui_gen/aavm/trans，不 tu/taa/tt；tf 仅批量规约） | next_responsible_role=review（T-07 独立 final：重点=781 同步义务执行情况、缺席/编码合同逐字、覆盖矩阵对 V01..05、SD-01/02 冻结）

2026-10-16 stage=review | review_scope=final | reviewer_identity=本聊天 Codex（独立于实施上下文，无子代理） | plan_id=PLAN-785 | plan_revision=1 | outcome=needs_fix | worktree_path=D:/autostack/.wt/lang-785/auto-lang | branch=plan-785-dev | reviewed_commit=64c72d20c2e081b563760c1c4fe38761ccc6d00b | base_commit=340ab95fd6b7871f492b65f721f4cef56b9f4c11 | acceptance_results=AC01 pass/AC02 partial/AC03 pass/AC04 fail/AC05 pending | findings=F-785-R1-01（P2，新增测试未统一隔离进程默认槽，6 项 libtest 并行重放第2轮“缺席=无 provider”失败）；F-785-R1-02（P2，fixture :498 新增 unused shim_clipboard_text 导入） | evidence=[R1 完整报告](attachments/785/review-r1/review.md)，no-ui consumer 10/10、785/779/adapter 定向 nextest 10/10、UI 同源 consumer+独立控制13/13、check 两包 exit0；原始并行失败/冻结 SD/依赖指纹见同目录 | dependency_revisions=781仍未合入，master相对base无本计划生产代码差异，本树本地锁指纹独立记录，返工须履行原“final前同步master”义务并重查781；auto-down见baseline.txt | spec_inputs=vm/ui overview SHA256见冻结 [SD](attachments/785/review-r1/spec-delta.md) | pending_closeout_gates=最终复审通过后的冻结提交裸cargo t+tv、SD/ledger/合流归档/批到期判定/guard-clean；本轮未跑整档 | next=work在原树修复提交后复审。T-05/06 重开，T-01..04 保留完成，current_step=4/8；旧✅收据仅为历史声明，不覆盖R1缺口。
2026-10-16 stage=work（R1 修复轮 → 再交 execution_done）：stage=work | plan_id=PLAN-785 | plan_revision=1 | worktree_path=D:/autostack/.wt/lang-785/auto-lang | branch=plan-785-dev | base_commit=64c72d20c（R1 复审基线） | code_commit=**012c89b99**（F-R1-01/02 修复）+ **0ac6e2ff5**（master 同步 merge，义务履行） | dependency_revisions=auto-down c39ac7f 不变；master@7c1eb2753（相对 base 仅 docs/experimental 差异）；PLAN-781 重查=仍未合入（树 HEAD 26fcb89f4；R07 增量仅 ffi/stdlib.rs+native_catalog.rs+plan781 测试——与 785 触面不相交性保持，依赖未交付事实不变） | task_ids=T-05/T-06 重闭（6/8；余 T-07/T-08） | findings 闭合=F-785-R1-01：不变式修复（非仅反例）——根因=多个测试文件各自本地 OnceLock 序列化锁域不相交，写槽/装配/缺席断言测试并行互踩；修复=host_capabilities 落 crate 级 cfg(test) 共享锁 test_default_slot_lock，全部 9 个相关测试接线持有（plan785×3、plan779×2+helper、clipboard_provider、frame_provider、host_capabilities absent），save/restore 纪律保留；受影响兄弟面排查=779 族/两 adapter/host_caps 单测全部纳入同域（bridge 构造与高层入口经 installer 的 if-absent 双写 frame+clipboard 均在锁内）；反例重放=复审原命令修复前本轮首跑即复现红→修复后 30/30 绿（合并提交再 10/10）；组合 13 测同进程 15/15 绿；F-785-R1-02：fixture 未用导入已删，两形态编译零本计划新增警告 | evidence=check --locked --offline 0 错；plan785 5/5+plan779 4/4+clipboard 21/21；fixture no-ui 10/10；UI 锁定重放 10/10 exit0（attachments/785/ui-form-replay/ui-consumer.json：head=0ac6e2ff5、frozen sha=1ab9ee3e、frozen_source_unchanged=true） | blockers=无（AC02/AC04 的 R1 缺口已闭，待独立复审确认） | next_responsible_role=review（T-07 R2：重点=F-R1-01 锁域闭合完整性（是否有第 10 个写槽测试漏网）、30 轮重放复跑、两形态警告清零、R1 义务履行核验）

2026-10-16 stage=review | review_scope=final | reviewer_identity=本聊天Codex（独立于实施、沿用R1复审上下文、无子代理） | plan_id=PLAN-785 | plan_revision=1 | outcome=needs_fix | worktree_path=D:/autostack/.wt/lang-785/auto-lang | branch=plan-785-dev | reviewed_commit=0ac6e2ff5ced743974ce274bb0fa8cf424c14453 | base_commit=64c72d20c2e081b563760c1c4fe38761ccc6d00b（修复比较；原生产diff基面340ab95fd） | dependency_revisions=auto-down c39ac7f不变；master@7c1eb2753已吸收、同步义务履行；781仍未合入、树HEAD=5bc7e58ad；当前master@1cb4009eb无本计划生产/已跟踪配置变化 | acceptance_results=AC01 pass/AC02 partial/AC03 pass/AC04 fail/AC05 pending | findings=F-785-R1-02 closed（两形态无该未用导入）；F-785-R1-01原6项反例30/30绿，但完整隔离未闭（加入普通高层入口plan320七测后，20项组合第14轮19pass/1fail，同:55“缺席=无provider”失败，普通run_with_capture经installer隐式写槽却不持共享锁） | evidence=[R2报告及闭合表](attachments/785/review-r2/review.md)、原始14轮日志/重放脚本/两形态consumer/冻结SD见同目录；no-ui10/10、UI+独立控制13/13、触面fmt/diff-check通过；生产/依赖/配置未变的R1 check及779外部证据显式复用 | spec_inputs=vm/ui overview指纹未变，冻结[SD](attachments/785/review-r2/spec-delta.md) | pending_closeout_gates=最终复审通过后的冻结提交裸cargo t+tv及SD/ledger/合流归档/批到期判定/guard-clean；本轮未跑整档 | next=原树work修复完整隔离，T-05/06重开、current_step=4/8。已连续两次final非通过，下次交接先附失败诊断/覆盖调整表，明确间接宿主装配路径及永久组合反例，不以原6项或13项通过替代完整闭合。
2026-10-16 stage=work（R2 修复轮 → 再交 execution_done；两连败诊断/覆盖调整表按 R2 route 要求附此）：
**诊断/覆盖调整表**：
1. **为什么九个具名测试共锁仍漏普通高层入口**：R1 修复模型=协作式测试侧锁，只有知道自己在写槽的测试参与；普通程序测试（plan320 族）不知道 run_with_capture 会写槽——execute 在 run_autovm_capture spawn 的独立 VM 线程上执行 install（lib.rs:694 起），写槽对调用方不可见。枚举**测试集合**永远≠覆盖**写者集合**。
2. **中间方案（六写 API 入生产互斥域+TLS 重入）为何废弃**：实测死锁——持域测试线程（high_level 跨 run_with_capture 持窗）join 该 spawn 线程，而 spawn 线程的 install 要取域 → 持域者→join→取域环（hammer+high_level 对偶 rc=124 两现；看门狗栈实录：等待者全部卡在 execute→install→enter_slot_domain）。生产写 API 回到 R1 原状，watchdog 已移除。
3. **采用的完整隔离**（R2 处方 (a)+(b) 混合）：缺席断言锚=**构造式**（cfg(test) AutoVM::new_without_host_snapshot——engine.rs new_inner(flash,ram,snapshot) 重构，生产 new() 行为零变）；**全局**缺席时序合同=外部 fixture 独立进程（no-ui 无任何安装路径；v03 经真实高层入口 Err 文本逐字；两形态门禁常驻）；测试侧共享锁保留但语义收窄——持有者仅限**清/写全局槽**的测试（跨操作原子），普通程序测试零配合豁免（其唯一槽写 if-absent 补缺对一切非缺席断言无害）。
4. **永久组合反例**：in-crate `absence_anchor_survives_concurrent_implicit_installs`（构造式缺席线程 × 普通高层入口 hammer 线程——若缺席锚退回全局时序实现即随机变红，R2 证据即其红形态）；外部 fixture no-ui v03 两形态门禁。
5. **检测为什么此前漏掉**：R1 重放过滤集只含自愿持锁的 6 具名测试——无非协作写者入池；R2 加入 plan320 后非协作写者立即复现。教训入册：隔离验证的过滤集必须包含不知情写者（普通程序测试），本轮重放口径已升级为混合集。
6. **覆盖调整**：重放=混合集（slot 家族+plan320_tests，21 测 threads=16）30 轮+原过滤 30 轮+合并提交 10 轮；日常档 plan785 含 hammer（6 测）。
stage=work | plan_id=PLAN-785 | plan_revision=1 | worktree_path=D:/autostack/.wt/lang-785/auto-lang | branch=plan-785-dev | base_commit=0ac6e2ff5（R2 复审基线） | code_commit=**cb5fa865b**（R2 修复）+ **c49ca102a**（master 同步 merge） | dependency_revisions=auto-down c39ac7f 不变；master@f73350a3f（相对 7c1eb2753 仅 docs/plans 差异零 crates 冲突）；PLAN-781 重查=仍未合入（树 HEAD 3dca8a39c，R08-C 只触 ui/iced/view_tree 区，与 785 触面不相交保持，不宣称依赖交付） | task_ids=T-05/T-06 再闭（6/8） | findings 闭合=F-785-R1-02（R2 已确认 closed）；F-785-R1-01 完整不变式本轮闭：生产写 API 未动（与 R1/原始一致），隔离架构重排为构造式缺席锚+独立进程全局合同+参与者窄锁 | evidence=混合集 30/30+原过滤 30/30+合并提交 10/10；plan785 6/6+plan779 4/4；fixture no-ui 10/10；UI 锁定重放 10/10（ui-consumer.json head=c49ca102a、frozen sha=1ab9ee3e、frozen_source_unchanged=true）；check --locked --offline 0 错；触面文件 rustfmt 清 | blockers=无 | next_responsible_role=review（T-07 R3：重点=诊断表核验、混合反例重放复跑、构造式锚与 fixture 合同分工边界、hammer 时长（日常档 ~2.7s））

2026-10-16 stage=review | review_scope=final | reviewer_identity=本聊天Codex（独立于实施、沿用R1/R2复审上下文、无子代理） | plan_id=PLAN-785 | plan_revision=1 | outcome=pass | worktree_path=D:/autostack/.wt/lang-785/auto-lang | branch=plan-785-dev | reviewed_commit=c49ca102abb198006ef1737a87522e37292a2c99 | base_commit=0ac6e2ff5ced743974ce274bb0fa8cf424c14453（原生产diff基面340ab95fd） | dependency_revisions=auto-down c39ac7f不变；master@f73350a3f已吸收，当前main@67b44e040仅Plan/附件差异；781仍未合入、树HEAD=3dca8a39c，后续若吸收其生产改动须受影响复审及新冻结门禁 | acceptance_results=AC01/02/03/04 pass，AC05 merge pending | findings=F-785-R1-01完整关闭（构造式缺席锚+独立进程公开入口全局合同+参与者窄锁；非仅原例关闭）；F-785-R1-02保持closed；无新增阻断项 | evidence=[R3报告及闭合表](attachments/785/review-r3/review.md)：原过滤7/7；R2混合反例21项×30轮全绿（每轮20s截止，无死锁）；no-ui10/10；UI同源fixture+独立控制14/14（含额外2线程×20次逐次expect成功/实值输出）；779外部6/6；check两包0错；触面fmt/diff-check通过；生产new()仍传true，cfg(test)绕过无外部API，无生产写域/TLS/watchdog残留；HEAD/clean稳定 | spec_inputs=vm/ui overview指纹未变，冻结[SD](attachments/785/review-r3/spec-delta.md)，hash及依赖/config指纹见baseline.txt/报告 | pending_closeout_gates=merge在冻结提交裸cargo t+tv，再SD/ledger/合流归档/批回归到期判定/guard-clean；本轮未跑整档 | next=merge；T-07完成，reviewed 7/8。本pass只表示代码/验收/规范审查通过，merge-owned门禁通过前不得合入或宣称交付完成。
### T-08 合并收据（2026-10-16，stage=merge，completion_kind=delivered）

- **prepared**：R3 pass @c49ca102a（reviewed_commit，plan_revision 1）；冻结 spec-delta（review-r3/spec-delta.md，vm AB378E0B…/ui BB13D724…改前指纹逐字吻合）；交付链 rebase 全等 2431eb300→SD 沉淀 c8f697009→收据 ecce1dc8c。
- **landed**：master tip = ecce1dc8c（`git merge --ff-only plan-785-dev`，零 merge commit）；old→new 映射 64c72d20c→d03ea0cd8、012c89b99→dea54b85f、cb5fa865b→2431eb300，`git range-diff` 三提交全等 `=`（安全改写证明）；`git diff c49ca102a 2431eb300 -- crates/ test/` 零差异。
- **merge-owned 冻结门禁**（交付提交 c8f697009）：裸 cargo t **5371/5371 PASS**（1548 skipped，72.2s）；cargo tv **163/163 PASS**（5.17s）；未触 ui_gen/aavm/trans（不 tu/taa/tt）。
- **ledger_refreshed**：`.autoos/specs.json` v28→v29（Regime B **受管离线投影**——HTTP 探测 store 不在线；五护栏：字节备份 `.autoos/specs.json.p785-bak`/脚本改写/作用域 diff（其余条目逐项相等，恰 +5）/schema 六节校验/回读验证）；条目 P785-D1、P785-D2（designs）/P785-T（tests）/P785-R3（reviews）/P785-M（reports）——最低四节面满足，全部带 file+tags。
- **archived**：本文件 git mv 至 docs/plans/archive/，status=archived，8/8；SD canonical 在 master（vm/overview 30f59c0e…/ui/overview ab0aefe9…）；合入报告 docs/plans/reports/785-clipboard-provider-merge.md 随分支落库。
- **cleaned**：wt-guard 双树 clean → worktree/分支/组目录移除（见下补记）。
- **观察项**：auto.exe 未随合并重建（本计划行为面零变，非必需；需要现值可 `cargo build -p auto`）；KNOWN-DEBT 中 F-785-P01/P02 条目仍在工作区（该文件同时携带 786 会话未提交记录——归属其簿记会话，随下次文件级提交落库，计划 §9 已有完整描述，无信息丢失）。
- **批量回归**：785 % 5 == 0 → **到期**，已交接 /auto-plan:regress（主检出单实例）。

## 10. 待澄清事项

默认方案无需用户先选trait命名；若T01证明保持错误/ownership/宿主行为需要改变公共接口或扩大媒体/线程模型，停在具名决策工件，修订本合同后再实施。独立UI锁解析问题不得默默记为通过，按V04边界与后续发行验收处理。
