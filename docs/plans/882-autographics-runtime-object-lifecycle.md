---
plan_id: PLAN-882
status: drafting
feature_name: AutoGraphics有限运行对象身份与生命周期
author: [Codex]
created_at: 2026-10-29
updated_at: 2026-10-29
plan_revision: 1
supersedes_spec_components:
  - docs/specs/auto-graphics/scene-play-session.md
  - docs/specs/auto-graphics/auto-source-driver.md
new_spec_components:
  - docs/specs/auto-graphics/runtime-objects.md
touched_goals: [GOAL-008]
affects: [auto-graphics]
depends_on: [PLAN-879]
current_step: 0
total_steps: 10
---

# [PLAN-882] AutoGraphics有限运行对象身份与生命周期

## 0. 变更摘要

在879迁仓交付后的auto-ui实验源码上，交付用户Auto驱动的两维有限运行对象：从已接受叶模板spawn、平移/显隐、despawn、同槽新代重用与跨Run失效。运行对象与角色/相机同帧受检发布；不改编辑文档或旧profile。后续[883](883-autographics-trigger-events-and-round-restart.md)在本计划实际交付后接触发和新局。

## 1. 目标

- 对象真实存在于受检Run对象表，生命周期不是仅切换visible；销毁后身份及消费者资格消失，同槽复用获得新generation。
- 一套公开Auto合同覆盖2D/3D，复用现有单Host/Play/SDK/资源/GPU；两维最小项目可仓外运行并有真实内容/来源/owner证据。
- 保持角色、相机、整帧失败last-good、Stop/新Run与编辑隔离；预留883需要的几何/代次接缝。

非目标：触发事件/新局服务（归883）、G11 Gallery（归883）、880 G07/G08、完整游戏、动态solid/移动平台/自由刚体、生成角色/相机目标、运行时导入资源、动态组/父子拓扑/任意prefab、缩放/旋转/材质writer、动画/声音/HUD、UI/ACE融合、WASM/迁仓/发布。实例是非solid叶，旧静态角色/solid合同不扩大。

## 2. 架构方案

固定Editor Document及角色/solid/相机保持；独立Run对象槽表保存实例身份、模板、局部/世界位置和可见性。受信Auto管理生命周期与用户调用，用户Auto决定何时生成/移动/销毁；Host验证模板/资源/身份转移及整候选，准备真实GPU内容并共同发布，不做游戏业务。

编辑u64 ID、旧ordinal和新实例(run,slot,generation)三域不混用。显式新2D/3D组合profile（候选名称`autographics.objects2d/1`、`objects3d/1`，T-02冻结数字/签名），可带当前公共角色/相机。旧behavior/1..5、character/1、character3d/1及camera两profile装配/行为逐字节保持，未知behavior/6仍拒绝。

## 3. 技术栈

实现仓唯一为`D:/autostack/auto-ui`，根`experimental/autographics-auto-driver`，AGP只做必要消费触面。已核源码：`src/authoring/{behavior.rs,behavior/assembly.rs,play.rs,spatial.rs,document.rs}`、`host/{play_adapter.rs,viewport.rs}`、`api/behavior_v1.at`、`samples/authoring_play/main.at`、`tools/{prepare_sdk,verify_source,package_consumer,verify_plan879}.py`。

拟新路径：`api/runtime_objects_v1.at`、`src/authoring/runtime_objects.rs`、`tests/{runtime_objects_api,runtime_objects_host,runtime_objects_samples}.rs`、`samples/runtime-objects/{two,three}/`、本代identity/checker。T-02可做等价局部分解，不把新路径当已实现。无新物理库/SDK依赖，不修改`crates/*`生产UI。

## 4. 需求分析与背景调查

授权：用户2026-10-29告知879交付、880已开发，要求规划下一阶段；本轮仅Plan/设计/来源核验，不实施/建树/接管880/新建agent/远端操作。AG源码今后在auto-ui，中央Plan/规范知识暂仍auto-lang；Gallery在ag-examples，各完整游戏另仓。

输入：`docs/specs/overview.md`、`auto-graphics/{source-repository-and-sdk,overview,behavior-api,scene-document,scene-play-session,spatial-query-profile,game-character3d,runtime-game-cameras}.md`、[Design45§13](../design/45-autographics-wgpu-engine-and-game-prototypes.md#13-m2运行对象触发与新局事务2026-10-29拟议合同)、[运行设计](../design/autographics/m2-runtime-and-character-control.md)、[游戏分母](../design/autographics/m2-game-sample-matrix.md)。本轮不改canonical Specs。

879实际交付auto-ui master=`4adf2d037a1804741adc9c5a012f08cfa566439d`，规划时clean；旧语言`autographics-dev=f07b5ec8`仅历史/指引，不再建引擎实现树。879迁移702文件，695恒等/7声明修改，SDK681文件快照在driver/sdk/frozen，当前身份1387条，冷准备`--from-snapshot`已交付；SDK pin仍7b987ca8、两AG锁不变。工作输入完整指纹见[本轮基线](attachments/882/planning/baseline.md)。

当前限制有源码证据：旧api ag_valid仅ordinal范围，所有count/拓扑/资源冻结；Play::admit要求`4+22*N+camera`及固定identity/kind/parent/resource，Frame是静态Document派生；Descriptor绑定base hash/行数≤128；viewport只渲染Frame.objects，潜在mesh≤512。故不能删count校验、隐藏金币或复用旧ordinal来宣称动态生命周期。新扩展必须携带独立受检Run表和转移边界。

880用户报告已进入开发；现有唯一树`D:/autostack/.wt/lang-880/ag-examples`、codex/plan-880-dev存在，最初检查仍在6a223d9锚点且clean，计划簿记仍drafting0/10；规划收尾已有catalog/G07/G08/checks未提交WIP。仅记录分时观察，不推断暂停/未授权，不修改其合同/目录/锁，也不切它的875固定输入。

## 5. 详细设计

### 5.1 实施入口和先行可行性门

未来唯一实现树`D:/autostack/.wt/lang-882/auto-ui`、分支`codex/plan-882-dev`，从核实的auto-ui master（规划pin如上）建树，源码只合auto-ui/master；语言主检出仅Plan/知识簿记，不建可写auto-lang引擎树。新增目标AGENTS若有须读。每Plan每仓一树，同树顺序work/独立review/merge；无junction/symlink，guard后清理。

T-01核879实际切源与当前owner/base/SDK/锁/current identity。T-02在`attachments/882/work/runtime-objects-contract.md`先冻结公开函数、profile、整数token、对象输出/操作转移准入、实际计费与支持包络，再做真实Auto整路径检查：角色移动/归一/跳、相机、8槽对象创建/移动/销毁/同槽重用，及为883预留的“三物品同时收集”操作组合。不能只证明独立spawn不超限。

原hard limits不变：总节点128、32credits/callback、1000steps/slice、100000steps/callback、120resumes/250ms、用户256KiB/managed64KiB/combined320KiB、初始化30s/整tick2s/每轮8ms目标。879固定SDK不改。需要新profile专用紧凑动作快照/批量只读访问时，签名/价格由T-02先冻结且只在新profile生效；不把开多个callback或隐藏免费扫描当预算解决方案。若整组合不可行，保留计数/源长度/反例回new，不静默抬限或移Rust做玩法。

### 5.2 支持集、容量与身份

每Run只选择一个维度，单静态角色/静态solid与可选同维运行相机；最多8个预留运行槽。`base groups+leaves+reserved capacity <=128`，容量按潜在槽计，隐藏/未激活槽不偷减分母；3D潜在mesh按各槽可选模板的最坏完整mesh数核算，base+实例总潜在数≤512。T-02将regular/8槽/128稀疏/64密集及超限形状冻结，继承874/875空间场景分母，不承诺128个动态solid。

模板仅来自本Run接受的同维叶，公开注册使用受检base节点映射；未知/组/错维/不支持shape/非有限/越界拒绝。资源、kind、scale/yaw/pitch/材质冻结，可用当前支持的内建与静态GLB完整mesh；spawn提供根级世界xyz，2D z必须0，继承模板形状/资源而不继承父子依赖。模板本身仍为原静态节点（显隐由原接口决定），不能偷偷从Document删去以换预算。

运行实例不能被当成角色、静态solid或相机target；是非solid对象，882不让它阻挡角色。运行对象参与的是正式对象注册和有界几何描述，883据此做触发；旧ordinal查询不自动接受实例token。存活但隐藏的实例仍具有身份/几何，和销毁不同。

身份至少含Run、slot、正generation，Run/generation均整数传输，禁止浮点截断或低位别名；编码/整数边界T-02冻结，耗尽具名拒绝，禁止回绕。公开接口须显式携带或可验证绑定Run，不能仅一个每Run重置的slot数字；旧Run同值token无效。允许合法absence检查返回false，其他读写/二次销毁/错Run/旧generation为sticky first error，整callback Fault/闭VM，不返回伪成功。

spawn创建真实对象表项；despawn撤销该身份、后续读写及渲染/几何消费者资格；同槽新spawn是新代，不能恢复旧句柄。一次callback最多32条生命周期写操作（仍受共享32credits和VM步数限制），无无界操作队列；具体读写/创建/销毁价格由实际charge路径冻结。显示隐藏只改可见性，不撤销身份，不可作为despawn实现。

### 5.3 新帧与消费者合同

| Rule | 实际数据流/拥有者 | 必须验证/关闭条件 |
|---|---|---|
| R-01 | 接受Document/Registry→Descriptor模板→受信Auto对象表 | 模板/资源/维度/静态形状来自实际全mesh，非用户自报；运行时不导入新资源 |
| R-02 | Auto调用→有界操作/对象快照→Play准入 | 校验Run/tick/profile、唯一活slot/代次、count/容量、template/resource、有限值/2D域、合法出生/死亡/重用序列与generation高水位；错包在消费前拒绝 |
| R-03 | 受检实例＋角色＋camera→同Prepared→accept/publish | 对象集合和角色/相机同tick，prepare前后失败都保上一完整呈现/对象资格；不得只保像素却接受新实例 |
| R-04 | Run VM/对象描述/资源引用→Stop/Fault/关闭→最后owner | Editor合法共享资源可保留；实例表/VM/对象描述额外owner释放，外部合法pin按最后持有者释放；新Run不沿用旧代 |

新运行表不写Document/scene4/dirty/Undo，动态几何不冒充旧Descriptor静态行。渲染可复用模型/mesh资源，但对象身份不得伪装持久Object.id；若内部适配临时Object用于DrawCmd/MeshInstance，域边界与不持久化必须具名校验。完整候选必须验证受信Auto返回值及合法状态转移，不能仅信回传alive标记或重算位置替代Auto。

旧profile保持原输出帧及选项/装配字节，新分支只对显式身份启用；SourceSnapshot/区域来源/AST准入、panel/CLI/Play/consumer须一致识别。current identity升本代，879迁仓映射作为历史保留，不拿未来修改套879恒等声明。

### 5.4 公共工程、来源和资源

新samples两维各至少两套模板/位置布置：真实Auto生成、移动、隐藏→显示、销毁、同槽新生；检查新旧handle、变化后的实际合成内容与基数。3D含至少一个既有合法多mesh模板，不能只复制首mesh。一个用户参数/时间点改动使实际轨迹或生命周期改变，无专用Rust分支；root入口为公开`--open/--behavior/--behavior-api`，新selector在T-03建立，不能用测试harness作为用户启动依赖。

fresh own-SDK consumer从本Plan新source closure导出，快照冷准备、独立target、离线锁定构建、真实Auto及无窗口Host/GPU内容成立；PE32MiB/worker16MiB保持。regular/8槽和128稀疏/64密集做有限暖步资源测量，分报Auto/准入/prepare/对象表/源码/逻辑与实际内存；不重跑五进程全矩阵或承诺60Hz。超限guard拒绝无半帧，CPU错误不当无碰撞。

20轮创建/销毁/故障/Stop/新Run用weak owner＋实际上传/内容对照，模型合法共享无每次spawn重新解析/顶点全扫/静态mesh上传；内存随最大容量有界，非累计出生数增长。880只读固定875，不依赖新对象API。

### 规范增量

| delta_id | add/modify/retire | docs/specs/... target | before/after rule | rationale | acceptance IDs |
|---|---|---|---|---|---|
| SD-01 | add | docs/specs/auto-graphics/runtime-objects.md | 仅固定节点→新显式两维有限实例、代次/资源/错误/包络 | 公共生命周期 | AC-02..05 |
| SD-02 | modify | docs/specs/auto-graphics/scene-play-session.md | 静态Frame→新profile对象集合与角色/相机同事务，旧保持 | 消费与owner | AC-03/04 |
| SD-03 | modify | docs/specs/auto-graphics/auto-source-driver.md | 879迁仓代际→新对象身份/来源/CLI/consumer，SDK/旧源保持 | 真实入口 | AC-01/02/05 |

## 6. 测试设计

work自查→独立上下文最终review→merge冻结受影响门，同一auto-ui树；未指定模型/创建agent。按2026-10-28政策：只测变更相关、约1/10可复现抽样（seed/候选集/选择数在案），必要新AC直接正反例不可漏；全量AG里程碑档pending。OS输入/置顶/抢焦点自动化暂停；新内容用公开逻辑输入与无窗口GPU，历史OS证据只适用于未变路由，不把旧exe/截图冒新runner。

| Case | AC/不变量 | 入口/代表输入 | 可区分oracle | work任务/边界 |
|---|---|---|---|---|
| V-01 | AC-01/02 身份/准入 | 新api/assembly/Play，错profile/资源/slot重复/假generation/float整数/截断包 | 新合法全通；旧版本装配不变；反例在具名准入阶段拒绝无未受检消费 | T-02/03；静态旧/动态新双向 |
| V-02 | AC-03 真生命周期 | 两维模板spawn→隐藏/显示→despawn→复用、旧Run相同数字token | 实例表/画面/几何资格一致；隐藏仍alive，销毁不alive；旧token操作新对象真实红；generation耗尽拒绝不回绕 | T-03/04；8槽根叶、非solid |
| V-03 | AC-04 事务/owner | 当步移动+spawn/despawn+orbit，坏字段/超限/prepare前后失败，20轮Stop | 角色/camera/活集合全旧或全新；编辑hash/历史不变；最后weak owner释放且mesh上传不随birth数增长 | T-04；共享合法pins另测 |
| V-04 | AC-05 源码/实际内容 | 两维双布置/参数改动、多mesh、冻结对象输入或漏mesh真实变异 | 实际Auto轨迹/生命周期与像素敏感；仅日志/计数清零不能绿；还原正例绿 | T-04/05；非完整游戏 |
| V-05 | AC-01/05 消费/预算 | fresh包A→B、SDK/锁/identity controls、8槽/128wide/64deep及over-limit | 真离线运行/32credits及原步限；超限保last-good；缺件/partial/0selected/错runner拒绝 | T-02/05；非实时保证 |

新测试T-03/04建立：driver `cargo check --locked`、`cargo fmt -- --check`、`cargo test --locked --test runtime_objects_api --test runtime_objects_host --test runtime_objects_samples -- --test-threads=1`；现有runtime_camera_host/character3d/assembly/Play选受影响命名case（无原生窗口），checker实参/controls与报告集合T-02冻结。source验证必须含新identity `--controls`、`prepare_sdk --verify-only`、`verify_source`和独立consumer命令。AGP有修改才跑其受影响面，default UI manifest/锁/crates零功能diff；根语言无改动不跑根cargo t/tf/tv/tu/docs_gen，不跑全UI/全AG。

原始探针/输出/截图本地`D:/autostack/plan-artifacts/plan-882/`，attachments只提交md摘要/契约（命令/exit/数字/路径/hash）。复用证据说明代码/依赖/配置/输入未变；新运行表/消费者不能只引用旧套件绿。

## 7. 验收标准

| ID | owner_stage | 可观察通过条件 |
|---|---|---|
| AC-01 | work | 879实际源/SDK/锁/唯一auto-ui树和fresh闭包核实；880零写入 |
| AC-02 | work | 新profile/签名/资源与整数身份/整路径预算先冻结，V-01成立，旧版本保持 |
| AC-03 | work | 两维实例真创建/销毁/代次/存活显隐及跨Run拒绝，V-02 |
| AC-04 | work | 新对象/角色/相机事务、失败保旧、Stop/owner与资源有界，V-03 |
| AC-05 | work | 双布置/参数敏感、实际内容/多mesh/消费/预算与真实负控闭合，V-04/05 |
| AC-06 | review | 独立逐项核AC/V/SD、身份/状态转移/遗漏/fmt/新增warnings，merge门pending |
| AC-07 | merge | 冻结受影响抽样/新必需对照与复用审计通过，失败回work复审 |
| AC-08 | merge | 源码只落auto-ui，语言知识/ledger/索引/归档与guard清理，交883实际SHA |

## 8. 执行步骤

| ID | owner_stage | 依赖 | 实际位置/操作与产出 | AC | 状态 |
|---|---|---|---|---|---|
| T-01 | work | 用户启动；879已交付 | auto-ui唯一树/owner/base、SDK/两锁/当前代际核对，880边界 | AC-01 | [ ] |
| T-02 | work | T-01 | runtime-objects-contract/价格/身份/帧/支持集/整路径probe先冻结，失败回new | AC-02/05 | [ ] |
| T-03 | work | T-02通过 | 新Auto单元/SourceSnapshot/Play/registry与准入/选择器、V-01/02 | AC-02/03 | [ ] |
| T-04 | work | T-03 | Host/GPU对象适配/事务/owner、两维公共工程/真实反例、V-03/04 | AC-03/04/05 | [ ] |
| T-05 | work | T-04 | fresh consumer/资源测量/current identity/checker/fmt/提交/SD摘要，V-05 | AC-01..05 | [ ] |
| T-06 | review | T-05 | 同树独立最终review，绑定冻结提交；merge门pending | AC-06 | [ ] |
| T-07 | merge | T-06 pass | 冻结受影响门/未变复用审计；不跑全AG/窗口自动化 | AC-07 | [ ] |
| T-08 | merge | T-07 | auto-ui/master落地，记录实际源码SHA；不合旧autographics-dev | AC-08 | [ ] |
| T-09 | merge | T-08 | 语言主检出SD/overview/plans/ledger/索引、归档、handoff给883 | AC-08 | [ ] |
| T-10 | merge | T-09 | wt-guard clean后清树/分支/空组；保留必要exe/本地证据收据 | AC-08 | [ ] |

## 9. 复审记录

2026-10-29 new r1：`stage: new | plan_id: PLAN-882 | plan_revision: 1 | outcome: pass | next: 用户启动work后T-01→T-02`。879实际交付已核，880固定输入可并行；本合同先限8槽非solid根叶，T-02拥有身份/帧/价格及整路径可行性冻结，不预填未测资源余量。未实现任何函数/新树/SDK准备/构建/OS/GPU/agent/远端动作，所有AC/V/T/SD未执行。

2026-10-29 new合同预检：0..10节、唯一AC-01..08/V-01..05、分stage任务与SD覆盖已核；本批两Plan及五份路线文档109个本地链接均可解析、编号各唯一，880跟踪文件相对HEAD零diff。仅结构/路径/合同自查，不是实现或独立final review。

## 10. 待澄清事项

当前没有新增产品决策。8槽是第一版明确支持集，base+reserved仍≤128；不会借此关闭完整游戏的动态solid/任意拓扑等非目标。若角色/相机+对象/后继收集组合在原预算不可行，或须改SDK/旧版本/验收分母，先带反例回new由用户裁定；不静默抬限。883必须等本计划实际交付再冻结执行基线。
