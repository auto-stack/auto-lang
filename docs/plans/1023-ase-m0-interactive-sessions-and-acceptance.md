---
plan_id: PLAN-1023
status: reviewed
feature_name: ASE M0交互会话原型与联合验收
author: [Codex]
created_at: 2026-11-12
updated_at: 2026-11-19
plan_revision: 2
current_step: 9
total_steps: 10
depends_on: [PLAN-1022, PLAN-968, PLAN-969, PLAN-970, PLAN-980, PLAN-981]
supersedes_spec_components: []
new_spec_components: [docs/specs/ase/m0-interactive-sessions.md]
touched_goals: [GOAL-002, GOAL-007, GOAL-010]
---

# [PLAN-1023] ASE M0交互会话原型与联合验收

## 0. 变更摘要

### 2026-11-19 · 980/981 后继接口

会话按 981 已交 SOURCE/会话和 980 已交 UI 宿主基础核对；剩余交互或资源面按实际使用项消费 [1071](1071-autoui-w1-av.md) 与 [1075](1075-age-av-content-and-fusion.md)。ASE 会话及产品验收沿本包合同。

r2，drafting，0/10；按用户指令只更新合同，不启动实现。基础调度/执行/会话、980 STATE/MIN-A-UI及981事件A已交；仍等1022正式frame，且**Node真实保留会话与UI状态接线未被证明**。新增981为实际所用事件接口来源owner，不等待其整包B收口。保原10任务、AC与全部业务/资源门，新增区分历史重放的V-07..10；M0不因A出口自动完成。路线：[ASE M0](../design/strategy/ase-m0-delivery-roadmap.md)。

## 1. 目标

单原生宿主支持两个存活应用会话；Counter A点击3次、B点击7次，切回各保留原值。同app双会话模型独立，关闭重开为0，旧事件不写新会话；取消/关闭后受管资源收敛。汇总各上游交付来源与证据并作M0实际裁决。

不含Clock/后台任务、checkpoint、Todo/服务/持久化、2D/3D、完整中文IME/复杂弹层、网络沙箱、A2R/Native发行或全Gallery。限制为研究profile，后续目标仍保留。

## 2. 架构方案

普通Node源model/msg/on与text/button→980/981共享Schema与唯一verify→公共执行/会话owner持有状态→宿主typed事件选择当前实例与无参handler→同模型读回状态增量→`ui_apply_*`提交UI→1022 frame。ASE只管理应用/session/frame映射与关闭代次，UI副本仅用于显示，不成为业务状态权威；Rust不代算count、不手写翻译handler，不将事件日志重放后数字一致冒真实保留会话。整条公共接线目前是本包必须先证的目标，不是已存在API。

## 3. 技术栈

复用1021/1022消费层、980 `ui/ace_node_desktop.rs`的UI-FFI/队列/显示owner与公共Node宿主，970 `ace_session`/`session_bridge`、所用968调度及981事件A语义。候选会话绑定/观察口在T-01按公开接口冻结；不假称当前有任意Node常驻API。拟新增`auto-ui/crates/auto-ui-runtime/tests/ase_m0_sessions.rs`，不重复实现语言检查、执行器、通用调度或renderer，不要求970所有CLI/MCP消费迁移。

## 4. 需求分析与背景调查

2026-11-17用户授权只更新ASE路线和1022/1023，明确已交接口、剩余依赖及真实会话验证；1020/1021归档保持，**先不启动实现**。本轮不建树、不改源码/fixture/Spec、不运行构建/复验/GUI/探针、不提交/合入/发布；没有自动续跑、模型/工期/token预算。其他owner在途/暂停决定保留。

取材[Specs overview](../specs/overview.md)、[M2-A认证](../specs/ace-migration/m2-certification.md)、[UI已交§6/12](../specs/ace-migration/m2-autoui.md)、[970会话规范](../specs/ace-migration/m2-sessions.md)、[ASE冷加载现状](../specs/ase/m0-headless-loading.md)及[源码重放规范](../specs/auto-test-case/ace-source-only-replay.md)。规范/原owner正式收据为交付依据，整包executing与未勾历史项不代接口缺失；相反，A阶段签章也不证明Node常驻会话已接入。本包拟议Spec仅在真实验收后沉淀。

读取auto-lang HEAD=`f60266a99f53ca154d8caa717519aecbae2baef3`、auto-ui HEAD=`69e64d584f5eb62116604dba8d2da708acec9ab3`及并发文档WIP，开工重核。Spec SHA256前16：m2-certification=`045340642830e6b0`、m2-autoui=`bf01874c6fa2e8ba`、m2-sessions=`1e3b1376a56aa8d2`；源码已核`ace_state_bridge.rs::{dispatch,refresh}`与`ace_node_desktop.rs::NodeDesktopBridge`，后者持`Option<AceStateEngine>`。这是静态来源，不是本包运行证据。

[组件/Store实例规范](../../../auto-ui/docs/specs/runtime/component-and-store-instances.md)为隔离语义输入，不直接使用旧VmRef。970已交，但参考求值域为scalar int/bool、cap64携带表/源prelude及受限声明集；其持久化op-log用于显式恢复，不是任意Node模型支持。980已交按钮/状态，但`AceStateEngine::dispatch`追加历史ops，`refresh`构造完整`ComponentScenario`重新观察；该实现可作业务oracle，不能证明本计划驻留。1022公共frame仍未交。原r1“970草案/接口未交”的背景由本节取代，留§9历史。

## 5. 详细设计

### 最小正式进入门

frontmatter为来源owner并集，新增981仅针对实际所用事件A/语言域，不新增981整包或全B波等待。当前进入表：

| 依赖 | 已交事实 | 尚待关闭的实际门 |
|---|---|---|
| 1020/1021 | 历史基线、真实无窗口执行，已归档 | 不重开；1021当前兼容复验由1022完成，变化才定向再验 |
| 1022 | 尚未实施 | 同宿主公共frame/导航/关闭正式交付，**这是T-01实施的直接等待** |
| 967/968/969/970 | 公开FFI/执行/调度/会话隔离/关闭，已交 | T-01验证所用Node行为能进入保留会话；970 scalar域不自动扩到Node |
| 979＋980 STATE/MIN-A-UI | Node语法/普通行为、状态观察、实际UI-FFI、typed队列 | T-01验证每事件只执行本次handler、状态权威/增量读回及UI绑定；历史重放路径不满足 |
| 981事件A＋980消费 | 无参unit消息/命名或匿名选择与拒绝边界已交 | 本包只选Inc/Dec/Reset；payload/参数执行及逐项诊断等B面不纳入M0，也不作为全局前置 |

对已交能力核repo/SHA/Schema/SDK/host ABI/实际入口与范围，不重新实现。对缺接线必须区分“ASE薄消费适配”（本包）与“公共执行/Node语义能力”（980/981等原owner）；后者先核既有B职责，有缺口则具名提出修订/后继责任，不能擅改上游或重开归档970。

### 真实会话与UI状态先证（T-01必闭）

1022正式交付后，T-01最多比较两种公共接线策略，以普通Node Counter、一个新事件、隐藏/切回和一次关闭/重开先走全链。选用970会话/已交持有模型仅是候选，不假设把Node源码直接塞进scalar Eval就能成功。领域行为必须沿共同Schema/verify和编译器产生的handler，不允许Rust手写`count += 1`或将原handler人工翻译成REPL表达式。

先证须记录并冻结实际producer/consumer、公开观察口和以下owner关系：

| 对象/动作 | 必须成立的规则 | 失败时处理 |
|---|---|---|
| 编译/模型 | 固定源与ABI身份下，初次会话创建持有已验程序/模型；隐藏/切回不重新编译、分配或初始化业务模型 | 源/ABI显式变化可具名重建；普通切换发生重建则先证失败 |
| 新事件 | 实际UI队列带session/instance/generation的宿主元数据，调用普通源中该无参handler；此前已成功事件不再次执行 | 不把宿主identity编码为新guest参数，失败重试不能重放已成功事件 |
| 状态与显示 | 状态权威在受检执行服务的会话owner；读回值及增量经`ui_apply_*`落对应frame，显示镜像只读消费 | 不能只从Rust缓存/历史ops算出相同数字；无公开状态绑定则列具体缺口 |
| 切换与关闭 | 源不变时激活原session/model；close撤输入/呈现、释放最后owner；reopen新代次只初始化一次且为0 | 原session只是名字不变而内部重建也算失败；普通关开不做checkpoint恢复 |

T-01必须以V-07/08的机制观测区分真持有与历史重放；准备两个“最终数字仍相同但机制错误”的负控。观察可复用公开trace/生命周期钩子或在真实adapter边界加测试计量，准确位置由T-01冻结，不使用私有VM字段，不用wrapper自增计数替实际编译/初始化/handler执行。若没有可观察的公共全链或只能重放，不得通过先证、不得进入T-02以后，输出needs_replan＋源/接口/owner/关闭oracle；不要求用户为普通等价实现重新确认。

本profile最多两个活Session，第三个请求具名`session.limit`并保持现有两个不变；不自动淘汰、不承诺休眠。顺序测试A/B以及同app双上下文，每轮最多两个；关闭后释放名额。一次关闭必须取消当前输入捕获、撤呈现、释放会话租约与owned结果；持久数据不在本包。

A=3/B=7通过真实按钮事件达到；切换保留同一会话模型，不合并源命名空间。关闭A后重新打开种子0；同app A1/A2分别3/7。事件身份包含session及generation，旧队列事件在关闭/重开后明确拒绝、不能写入新0模型；测试从真实adapter队列注入迟到事件，不用自造布尔标志代表生产拒绝。

Counter源以规范node编写，消息固定无参`Inc/Dec/Reset`，int状态经已交int槽显示；不为显示计数器引入尚未支持的f-string文本叶，不改成Rust业务。当前Event-A带参/载荷为安全拒绝，保持其零状态/UI效果；若将来扩带参才按原EVENT-B子面交付，不借本包宣称完整事件已支持。

T-05记录20轮开/切/关后受管Session/任务/图形目标真实创建/释放计数与owner走势；共享字体/宿主缓存单列允许有界驻留，不要求RSS/物理显存归零。原生事件和宿主计量不能侵入ACE私有字段；公共观察缺失交原owner限定补接口。无界guest/强进程故障隔离不属于该最小profile，不借取消按钮声称已具备。

### 规范增量

| delta_id | add/modify/retire | docs/specs/... target | before/after rule | rationale | acceptance IDs |
|---|---|---|---|---|---|
| SD-01 | add | docs/specs/ase/m0-interactive-sessions.md | 两页文字→普通Node真实会话/状态权威与增量、代次/上限/关闭与M0已验格；记录实际机制及拒绝重放边界 | 数字相等不能代驻留证明 | AC-01..06 |
| SD-02 | modify | auto-ui: docs/specs/runtime/ase-m0-frame-consumption.md（1022创建） | text消费→已验证基本按钮事件/resize/owner与陈旧拒绝边界 | UI适配单源 | AC-02..05 |

### 首次入口与统一树

唯一组`D:/autostack/.wt/lang-1023/{auto-lang,auto-ui}`，分支`codex/plan-1023-dev`，每仓唯一树，显式来源锁与依赖解析，不建链接。材料`D:/autostack/plan-artifacts/plan-1023/<repo>/`开工即登记；原生输入/GPU同机单实例。不修改OS/Gallery产品源码与默认入口。

## 6. 测试设计

测试与oracle以下均为拟议、未执行；新增测试/CLI入口在对应实施任务创建，不能在创建前报告PASS。实现与内部检查由执行者承担，最终独立review采用独立上下文、同一权威树；本轮未指定模型/人员，不创建agent/chat。

语言仓只按真实触面选择完整固定组及必要消费者，参考现存`ace-av`、`ace-source`、`ace-artifacts`，不为单一外部消费者机械跑`cargo t ace`全并集或`cargo t all`。若改ACE的源码/共享语义须补相应完整组与下游理由；纯ASE消费者不修改ACE时以新增ASE独立目标为主。UI只跑新增/受影响命名目标与公共接缝必要集成，按本仓入口，不沿用语言仓退役别名；原生/GPU/焦点单实例，自动注入与真实用户输入分账。工作迭代按反例/完整职责面，最终review后冻结提交验证受影响并集，身份未变证据可复用。

r2补充：实际改公共执行/会话时选完整`ace-facade`及受影响`ace-av`等职责组；只做UI/ASE消费时不机械重跑全部ACE。正式压力仅显式`cargo t ace-perf`，不附加本包；Windows不跑Linux/WSL腿，具名排除归专用Linux环境、不充PASS。以下V-07..10为本revision必需补充，T-01先创建最小可执行验证/真实观察口，T-06固化完整目标；当前全未执行。

| Case ID | AC / invariant | 输入/真实入口 | 独立oracle与边界 |
|---|---|---|---|
| V-01 | AC-01 接口准入 | T-01核1022正式frame、已交状态/无参事件/会话/关闭/所用调度及具体Node接线 | 不因970已归档或A出口自动通过；缺Node绑定具体具名，不再把已交基础写为草案 |
| V-02 | AC-02 状态保留 | 原生按钮A×3、B×7→切回 | 显示3/7且受检会话模型对应；换宿主顺序/应用名成立；结合V-07/08证明不是重放/补数 |
| V-03 | AC-03 同app隔离 | A1×3/A2×7；关闭A1→重开 | 3/7独立，A1重开0而A2仍7；真实会话身份/模型owner核齐 |
| V-04 | AC-04 陈旧与限额 | 关闭后实际旧代次事件；第三会话；释放再开 | 旧事件拒绝/新0不变；第三具名拒绝且现有状态不变；释放名额后可打开 |
| V-05 | AC-05 呈现/释放 | 有效resize/焦点转移/20轮开关 | 命中当前frame、旧输入不串；owned资源计数收敛，共享缓存单列 |
| V-06 | AC-06 联合边界 | 1020–1022收据/当前身份＋本包结果 | M0-BASE/LOAD/TEXT/SESSION/CLOSE每门实际来源；缺证据pending，不从plan勾选反推完成 |
| V-07 | AC-01/AC-02/AC-03 真会话 | 固定Node源/ABI，开A/B→事件→隐藏/切回→同app双会话→关开 | 记录真实程序/模型owner与session/generation、编译/初始化及逐handler执行。切回不增加编译/业务初始化且不重复旧handler；A/B独立；关开仅新代次初始化一次为0。以真实边界trace/资源观测为准，不以外层id恒定代证 |
| V-08 | AC-01/AC-02 判别力 | 控制1强制回到初值＋重放历史ops，控制2每次切回重建模型并补回相同count | 控制的最后3/7仍正确，但V-07实际初始化/执行计量必须红；真实保留路径绿。控制进入同一adapter/observer，不用假PASS字段或只对测试桩计数 |
| V-09 | AC-03/AC-04/AC-05 状态到UI/释放 | 单事件→受检owner状态差量→UI提交，另session无事件；重复/错代/关闭后事件，注入一次失败 | 实际值与ui_apply目标/值、frame对应；错代零业务/UI效果，不污染另一session；失败按公共原子/错误合同不重复先前成功事件；最后owner释放与20轮受管资源收敛 |
| V-10 | AC-01/AC-06 事件支持边界 | 正常无参Inc/Dec/Reset，另取当前Event-A拒绝的带参/载荷近邻 | 正常由普通Node handler执行；不支持输入具名拒绝且零状态/UI效果，不把B的逐项arity/type等诊断冒已支持，也不为此等完整EVENT-B |

拟完整入口（T-06创建）：在UI唯一树`cargo test -p auto-ui-runtime --features ace-bridge --test ase_m0_sessions -- --test-threads=1`。现有参考目标`ace_node_desktop`、语言`auto-lang-ace-host/tests/t970_sessions.rs`/固定组`ace-facade`只按实际改面选；已有参考绿不代本包Node整链。1022的`ase_m0_text`与当前`cargo t ase-m0`有效证据可复用，输入/SDK/ABI变才补受影响完整目标。真实GUI入口/命令与观察口由T-01冻结，不虚构当前有新测试或常驻Node API。保20轮、原生人工/自动分账，零选择/缺环境不PASS。

## 7. 验收标准

| ID | owner_stage | 验收与方法 |
|---|---|---|
| AC-01 | work | 1022及已交基础来源核齐，T-01真实Node会话接线先证通过；V-01/07/08/10 |
| AC-02 | work | 两Counter普通源经ACE模型/真实按钮保留3/7，实际机制拒绝历史重放/切回重建；V-02/07/08 |
| AC-03 | work | 同app双会话隔离、同owner切回、关开新代次为0且不动另一会话；V-03/07/09 |
| AC-04 | work | 错代/关闭事件零业务与UI效果、第三请求拒绝不破旧态、释放名额再开；V-04/09 |
| AC-05 | work | 有效resize/焦点/20轮真实owner与受管资源收敛，状态提交到正确frame；V-05/09，原生/自动分别验 |
| AC-06 | work | 联合来源/实际支持域/未支持项对齐，无legacy fallback，事件A/B不混；V-06/10。不提前称正式M0落地 |
| AC-R | review | 独审普通Node→编译/真实会话→handler→状态增量→UI与关闭的producer/owner，复核V-07/08对重放的判别力及所有原门，不以前包勾选或只看3/7代证据 |
| AC-D | merge | 本包受审来源落地、Spec/归档/清理齐后才标M0正式完成；不签ACE/AGE里程碑 |

## 8. 执行步骤

- [x] T-01｜work｜1022正式交付及已交基础来源；后续仍需实施指令｜核唯一树/scope/来源，最多两机制普通Node→真实会话→UI增量整链先证；冻结公共adapter/实际观察口并创建最小V-07/08/10，证明历史重放与切回重建负控红；AC-01、AC-02。缺能力记录owner/consumer/关闭oracle并停T-02以后，本轮不执行。[✅ 2026-11-19：进入门全闭——1022已归档（auto-lang `archive/1022-ase-m0-text-browser-and-embedding.md`，delivery baa223ba7/auto-ui b1364eed）；`/goal 完成计划1023` 为实施授权。树=组 `D:/autostack/.wt/lang-1023/{auto-lang@baa223ba7, auto-ui@b1364eed, auto-web@26d085f, auto-down@72b0970}`（auto-web/auto-down 为 auto-ui path 依赖兄弟，detached 钉各 master，与1022四树同构）；scope=`D:/autostack/plan-artifacts/plan-1023/{auto-lang,auto-ui}/`。策略两比较后冻结 **S-B**（模块头具名）：S-A 双开显示车道被拒——980 `ui_apply_*` 钩子定向进程全局 current（唯一 setter `ui_open`，无公开切换口），隐藏会话差量会串写；S-B=每会话一 `NodeDesktopBridge`/`AceStateEngine` 常驻为业务权威，UI 车道只为选中会话开（切换=关出向+开目标，980 已交 reopen 语义），ASE 关闭释放引擎 owner、重开新代次种子0。adapter=`auto-ui crates/auto-ui-runtime/src/ui/ase_m0_sessions.rs`（两活会话上限 `session.limit` 具名拒、同 app 双上下文、evidence JSON）；观察口=loads/ui_lane_opens/clicks_offered/dispatches（真实调用边界：load/open/push_event/dispatch_log）。最小验证 `tests/ase_m0_sessions.rs`（commit `codex/plan-1023-dev` auto-ui 树）：V-07 min 两会话 3/7 真保留（loads钉2、dispatches=10、切回零重编译零重执行、隐藏会话引擎权威可读）452ms；V-08 min 负控1（切回重放历史事件：loads每切回+1、3次点击历史累计6次执行）+负控2（切回重建+种子补数：loads+1、count=3零执行撑值）最终数字均仍正确而观察口红 169ms；V-10 min Inc/Dec/Reset 真实轨迹+带参载荷 981 边界具名拒零状态效果+按钮目标越界具名拒 106ms（三例全部≤10s；prep=ase_m0_text_window build+--prep-only，日常档零 rustc；跑前机检无竞争 cargo/rustc）。]
- [x] T-02｜work｜T-01整链先证通过｜创建拟Counter源/A/B描述，Node无参Inc/Dec/Reset、int槽显示；共同Schema/verify，不手写handler或Rust代算；AC-02、AC-06，V-10。[✅ 2026-11-19：fixtures `experimental/ase-m0/fixtures/counter-{a,b}/main.at`（`CounterA`/`CounterB` 异根名异文字、同 Schema 无参 Inc/Dec/Reset + `model count int = 0` + on 复合赋值；按钮声明序冻结 0=Dec/1=Reset/2=Inc；无 f-string）+ README §PLAN-1023 消费描述；auto-lang commit `85b7816b7`。验证：`p1023_t02_fixture_counter_sources_load_and_verify`（auto-ui commit `90175197`）——两 fixture 经公共面装载零错误（共同 Schema/verify）、双会话各 Inc 得 1 互不影响、loads=2、文字叶真值各归各源（A 中文/B ASCII），226ms ≤10s；handler 全部为普通 Node on 复合赋值，零 Rust 代算。]
- [x] T-03｜work｜T-02｜接公共保留会话与tab/frame绑定、权威状态读回，同app隔离/关闭重开；AC-02、AC-03，V-02/03/07/08，不能直接把AceStateEngine重放称驻留。[✅ 2026-11-19：窗口组件 `AseM0SessionsWindow`（app 打开行/会话 tab 行选中高亮+ASE 级关闭按钮/frame 区渲染选中会话显示模型+业务按钮→`push_event` 真实 typed 队列，tick 20ms 受控步进，帧计数 `ASE_M0_SESSIONS_FRAMES`）；代次语义冻结=app 会话序号（每次 open 递增，同 app 双上下文各自成代）。`p1023_t03` case：同 app A1×3/A2×7 隔离、ASE 关闭释放 owner（engine_states=None、closed 记录 gen1）、重开=新会话新代次（gen3）种子0且 Inc 真实执行、A2 跨关开恒7、窗口 view 纯渲染冒烟，257ms。真保留由引擎常驻+loads/dispatches 观察口证（V-07/V-08 负控判别），非 AceStateEngine 重放。]
- [x] T-04｜work｜T-03｜代次队列/状态增量到ui_apply/关闭owner/两会话上限；AC-03、AC-04，V-04/09，禁止错代业务与UI效果。[✅ 2026-11-19：`p1023_t04` case——单事件=恰一个 int 槽 diff apply（applied 轨迹 kind/slot/值与显示一一对应）、隐藏会话车道 applies 零污染（before/after 对账）；第三活会话具名 `session.limit` 拒（errors 留名、现有两会话值/live/loads 全不动）；关闭后迟到事件经真实 adapter 队列注入（`push_late_event` 定向旧车道）→late_dropped 计数、新代次模型零业务（count=0）零 UI（applies 不动）效果；释放名额后同 app 再开成功。305ms。]
- [x] T-05｜work｜T-04｜有效resize/焦点、原生输入和20轮资源计量，关联会话/模型/呈现实际owner；AC-05，V-05/09，不冒完整IME/DPI/沙箱。[✅ 2026-11-19 CPU 面：`p1023_t05` 20轮 开→点→切→关 计量——常驻会话 a2 零重建（loads 不因切换增长）零重执行（dispatches_of 恒7）值恒7（每轮车道重提交引擎真值"7"）；每轮 a1 新开/关闭各一次记账；精确对账 loads=21/clicks=27/dispatches_total=27/ui_lane_opens=41（新增累计 dispatches 观察口，step 边界计数，关闭会话不丢走势）；结尾 live 清空、21 条 closed 记录（owner 全释放），1.56s。resize/焦点/原生输入的 OS 级注入归 T-06 窗口档（example exe+native_dock::win32 注入面），本任务不冒 IME/DPI/沙箱。]
- [x] T-06｜work｜T-05｜创建完整ase_m0_sessions并固化V-01..05/07..10，检查真实边界负控敏感性；受影响text/headless完整目标与必要共享组按身份/触面选，未变有效证据复用。[✅ 2026-11-19：完整目标 `cargo test -p auto-ui-runtime --features ace-bridge --test ase_m0_sessions -- --test-threads=1`（冻结入口）——9 CPU case 全绿 2.97s（V-01=接口准入经T-01来源冻结+T-02共同Schema装载；V-02=双counter A×3/B×7 双序成立 471ms；V-03=t03 同app隔离/关开 292ms；V-04/09=t04 增量目标/limit/迟到 110ms；V-05/09=t05 20轮计量 1.29s；V-07/08/10=最小集 253/181/103ms），各例≤10s；排窗档 `AUTO_UI_ASE_M0_E2E=1` 单实例——真窗两会话生命周期＋**OS级resize（SetWindowPos/WM_SIZE）与焦点转移（最小化-恢复）raw user32注入**命中当前frame零错误串扰，证据断言 b=7 保留/重开 gen2 种子0/loads=3/dispatches_total=10/frames≥15，3.22s；真鼠标/键盘注入归手工档（与自动注入分账，1022 同界）。入口 `examples/ase_m0_sessions_window.rs`（--app/--script open|select|close|click/--evidence/--prep-only）。受影响面：`ase_m0_text` 7/7 CPU 绿（1022 窗口档证据身份未变复用）、语言仓 `cargo t ase-m0` 组 PASS（worktree 实跑，fixtures 拷贝含 counter 新增）；`ace_node_desktop`/`ace_state_bridge` 模块零改动，980 既有证据复用。负控敏感性=V-08 双控在同观察口红、真实路径绿（T-01 冻结，非测试桩计数）。]
- [x] T-07｜work｜T-06｜核1020–1022正式来源与本包联合矩阵，给M0候选裁决/缺项；AC-06，V-06，不标里程碑正式完成。[✅ 2026-11-19：来源实核（worktree 磁盘+提交身份）——1020/1021/1022 均归档在 `docs/plans/archive/`；Specs：`docs/specs/ase/m0-readiness.md`（1020）、`m0-headless-loading.md`（1021）、`m0-text-browser.md`（1022 SD-01，delivery baa223ba7）、auto-ui `docs/specs/runtime/ase-m0-frame-consumption.md`（1022 SD-02，b1364eed）。联合矩阵（每门实际来源，缺证据即 pending）：**M0-BASE**=1020 基线与接口就绪（归档）✓；**M0-LOAD**=1021 解析+无窗口 AV 冷加载（归档+当前 `cargo t ase-m0` PASS 实跑）✓；**M0-TEXT**=1022 两页文字浏览器同宿主 frame（归档+本包树 `ase_m0_text` 7/7 复验绿）✓；**M0-SESSION**=本包两活会话/引擎常驻真保留/同app隔离/两上限（T-01..T-06，9 CPU+1 排窗 e2e 全绿）✓；**M0-CLOSE**=本包 ASE 关闭释放 owner/重开新代次种子0/迟到拒绝/20轮收敛（T-03/04/05）＋1022 轮换关闭面 ✓。**M0 候选裁决：五门全部有实际来源证据，M0 候选成立**；正式完成不在本任务标（AC-D 归 merge：受审落地+Spec/归档/清理齐后）。缺项（如实，不阻塞 M0 候选）：真鼠标/键盘注入与真人走查档 pending（1022 同界）；resize/焦点为 OS 级注入档已验、IME/DPI 完整面/网络沙箱/A2R 发行不在 M0 域；`ui_apply_*` current 热路径限制双开车道（S-B 冻结已记录，公共多车道切换口属 980 owner 后继）。]
- [x] T-08｜work｜T-07｜告警/格式/无调试输出，§9折命令/结果/资源/hash与拟议Spec交接；AC-01..06。[✅ 2026-11-19：健康检查——新增三文件（ase_m0_sessions.rs 模块/测试/example）`cargo check --tests --examples` 零新告警（仅仓基线既有）、rustfmt --check 全绿、零 dbg!/println!（eprintln 仅 CLI 诊断与计时证据行，1022 同款）。拟议 Spec 交接：SD-01 草案=auto-lang `docs/specs/ase/m0-interactive-sessions.md`（commit 23f9524ca）；SD-02 草案=auto-ui `docs/specs/runtime/ase-m0-frame-consumption.md` 增交互会话消费节（commit f24aca0e）。两 worktree clean。命令/结果/资源/hash 全折 §9 work pass 收据。]
- [x] T-09｜review｜T-08｜独立重建M0 required，逐链核真实状态owner/编译初始化与handler观测、V-07/08负控及原反例/资源；AC-R，不能以最后数字相同免检。[✅ 2026-11-19 final review **pass**（独立子代理上下文，无实施记忆）：diff 面=auto-ui 5 文件/lang 4 文件全对账，980/981/1022 模块 diff 为空；producer/owner 链逐环节码读（load→engine 常驻→step→dispatch→diff apply→current 热路径→关闭释放，S-B 冻结理由在 hook 代码核验）；V-07/08 判别力成立——逐变异具名失败断言（重建+重放→loads/dispatches 红；直连 load 绕计数→dispatch_log 红；种子补数→dispatches_of(a)==Some(3) 红）；冻结 HEAD f24aca0e 复跑 9/9 CPU 2.63s（全部 case ≤10s，最快 95ms/最慢 1.15s）+ 排窗 e2e 3.1975s（frames≥15、b=Int(7) 保留、重开 gen2 Int(0)、loads=3、dispatches_total=10、零错误；fmt 后重跑绑定 reviewed 提交）+ ase_m0_text 7/7 复验 0.84s；机检零竞争。AC-01..06 全 pass（证据映射见 §9 收据）。发现 F-1（非阻塞：负控经真实 bridge/engine 构造而非变异 adapter，判别量仍为真实边界——后续复用可考虑变异 adapter 形态）、F-2（非阻塞：SD-01 归档引用前瞻，T-10 归档后成立，merge 落地时核对）。SD-01/SD-02 草案 pass。限制：未重跑语言仓 ase-m0 组（lang diff 仅 fixtures+文档且 6 case 直接消费验证）、真鼠标/真人走查档 pending（已记录接受边界）。]
- [ ] T-10｜merge｜独审pass｜冻结相关完整目标/必要组，SD-01/02正式落地、归档、全输出对账折摘要删scope；用`pwsh -NoProfile -File D:/autostack/wt-guard.ps1 <每仓工作树>`确认clean后清树；AC-D，齐后才记M0正式完成。

## 9. 复审记录

stage: work | plan_id: PLAN-1023 | plan_revision: r2 | outcome: pass | worktree: D:/autostack/.wt/lang-1023/{auto-lang,auto-ui}（依赖兄弟 auto-web@26d085f/auto-down@72b0970 detached 钉 master）| branch: codex/plan-1023-dev（两仓同名）| base: auto-lang baa223ba7 / auto-ui b1364eed | code_commit: auto-lang 23f9524ca（T-02 85b7816b7＋SD-01 23f9524ca）、auto-ui f24aca0e（T-01/T-02测试/T-03/T-04/T-05/T-06/SD-02 七提交）| dependency_revisions: 980/981/1022 公共面经 worktree 树内源码核验（ace_node_desktop.rs/ace_state_bridge.rs/ase_m0_frame.rs 零修改）| task_ids: T-01..T-08 全完（8/10；T-09 review、T-10 merge 待）| evidence: 见下折录 | blockers: 无 | next_responsible_role: review（独立上下文，AC-R）

**work pass 证据折录（2026-11-19，goal 授权 `/goal 完成计划1023`；进入门=1022 已归档核验＋本 goal 为实施指令；master 蓝图 .at 删除 WIP 经用户裁定"先不处理照常开工"，两 worktree 从已提交 HEAD 开工不触碰）**

stage: goal | plan_id: PLAN-1023 | plan_revision: r2 | round: 1 | from: blocked(drafting r2) | to: review | status: execution_done | outcome: pass | worktree: D:/autostack/.wt/lang-1023/{auto-lang@23f9524ca, auto-ui@f24aca0e} | commit: lang 23f9524ca / ui f24aca0e / plan 10bcae61c | next: review

stage: review | review_scope: final | reviewer: 独立子代理（general-purpose，全新上下文，无实施记忆；限制=未重跑 lang ase-m0 组、真鼠标/真人走查档 pending、机检为开始时快照） | plan_id: PLAN-1023 | plan_revision: r2 | outcome: pass | worktree: D:/autostack/.wt/lang-1023/{auto-ui, auto-lang} | branch: codex/plan-1023-dev（两仓） | reviewed_commit: auto-ui f24aca0e / auto-lang 23f9524ca（复跑后树仍 clean，证据绑定 reviewed 提交） | base: b1364eed / baa223ba7 | dependency_revisions: 980/981/1022 模块 diff 为空（ace_node_desktop/ace_state_bridge/ase_m0_frame 零改动），S-B 冻结理由经 hook 代码核验 | spec_inputs: SD-01=docs/specs/ase/m0-interactive-sessions.md（23f9524ca）、SD-02=auto-ui docs/specs/runtime/ase-m0-frame-consumption.md 增量（f24aca0e）——两草案均 pass | acceptance_results: AC-01..06+AC-R 全 pass（映射：AC-01=归档核验+hook 码读+V-01/07/08/10；AC-02=v02 434.9ms+v07min 231.3ms+v08min 153.5ms；AC-03=t03 owner释放/gen3种子0+e2e gen2 Int(0)；AC-04=t04 session.limit/late_dropped 零效果/hook_close 结构性清队列；AC-05=t05 精确对账+e2e OS resize/焦点；AC-06=五门矩阵在盘核验+v10 载荷/越界具名拒；AC-R=逐变异具名失败断言） | pending_closeout_gates: merge owns——冻结受影响并集复验已毕（9 CPU+e2e+ase_m0_text 7/7 本审重跑），SD 正式落地、归档、wt-guard 清树；F-2 归档引用核对 | findings: F-1 非阻塞（负控经真实 bridge/engine 构造，判别量真实边界，无需整改）、F-2 非阻塞（SD-01 归档引用前瞻，T-10 后成立） | evidence: CPU 9/9 2.63s（t02 223.7ms/t03 236.3ms/t04 103.5ms/t05 1.152s/v02 434.9ms/v07 231.3ms/v08 153.5ms/v10 95.2ms，全 ≤10s）；e2e 3.1975s frames≥15/live b Int(7)×7次/重开 gen2 Int(0)/loads=3/dispatches_total=10/clicks=10/零错误；prep=example build 32.71s+--prep-only exit0（日常档零 rustc）；机检零竞争 cargo/rustc | next: merge

stage: goal | plan_id: PLAN-1023 | plan_revision: r2 | round: 2 | from: review(entry) | to: merge | status: reviewed | outcome: pass | worktree: D:/autostack/.wt/lang-1023/{auto-lang@23f9524ca, auto-ui@f24aca0e} | commit: reviewed f24aca0e/23f9524ca（复跑后 clean） | next: merge

- **接线与观察口（T-01 冻结）**：S-A 双开显示车道被拒（980 `ui_apply_*` 钩子定向进程全局 current，唯一 setter `ui_open`，隐藏会话差量会串写——具名记录，公共多车道切换口属 980 owner 后继）；**S-B 选中车道+引擎常驻**采用。观察口=loads/ui_lane_opens/clicks_offered/dispatches（真实调用边界：load/open/push_event/step；dispatch_log 同源）。
- **测试（本会话实跑，`--test-threads=1`，全部 ≤10s）**：`cargo test -p auto-ui-runtime --features ace-bridge --test ase_m0_sessions`——9 CPU case 2.97s 总墙钟（v07min 253ms/v08min 181ms/v10min 103ms/t02 263ms/t03 292ms/t04 110ms/t05 1.29s/v02 471ms/e2e 档跳过记 pending 不充 PASS）；排窗档 `AUTO_UI_ASE_M0_E2E=1` 单实例——window e2e 3.22s（真窗两会话生命周期＋OS 级 SetWindowPos resize＋最小化-恢复焦点注入，frames≥15、b=7 保留、重开 gen2 种子0、loads=3、dispatches_total=10、零错误）。准备档=`cargo build -p auto-ui-runtime --features ace-bridge --example ase_m0_sessions_window`＋`--prep-only`（ui980 provider 预构建，日常档零 rustc）；跑前机检（Get-CimInstance）无竞争 cargo/rustc。**受影响面**：`--test ase_m0_text` 7/7 CPU 绿 0.81s（1022 窗口档证据身份未变复用）；语言仓 `cargo t ase-m0` PASS（worktree 实跑，含 counter fixtures 拷贝）；`ace_node_desktop`/`ace_state_bridge` 零改动、980 既有证据复用。负控敏感性=V-08 双控（重放/重建补数）同观察口红、真实路径绿。
- **资源计量（T-05）**：20 轮 loads=21/clicks=27/dispatches_total=27/ui_lane_opens=41 精确对账；结尾 live 清空、21 条 closed 记录（owner 全释放）；共享 provider 缓存单列非会话资源。
- **联合矩阵（T-07）**：M0-BASE(1020)/LOAD(1021+ase-m0 组)/TEXT(1022+复验)/SESSION(本包)/CLOSE(本包+1022) 五门实际来源证据齐——**M0 候选成立**；正式完成归 AC-D（merge），真鼠标/真人走查档 pending 如实记。
- **拟议 Spec**：SD-01=auto-lang `docs/specs/ase/m0-interactive-sessions.md`（23f9524ca）；SD-02=auto-ui `docs/specs/runtime/ase-m0-frame-consumption.md` 交互会话消费节（f24aca0e）。
- **待 review/merge owner 的门**：冻结受影响并集（ase_m0_sessions 完整目标＋必要时 ase_m0_text 复验）与 Spec 正式落地；统一回归不在本 handoff 前置（review/merge owns）。

stage: new/revision；PLAN-1023 r2（2026-11-17）；outcome: blocked（1022未交，真实Node会话与UI状态连接仍待先证；已交基础不再列“尚为草案”）；next: 未来1022正式交付并获实施指令后，T-01整链先证/V-07/08/10，go才进入T-02。0/10，所有V-*、联合验收、review/merge未执行，本轮不签M0。

历史r1（2026-11-12）：当时因1022与基础接口未齐而blocked，0/10；原静态检查通过不充实施证据。旧读取基面lang `7b2f2e959`/UI `ad9c05da`及“970草案”是当时背景，当前合同由r2 §4–8取代。r2保留T-01..10/全部原AC/3与7/关开0/两会话上限/20轮，新增V-07..10绑定原AC，981仅作实际消费事件接口来源，不新增全局等待。

r2静态预检（2026-11-17）：0–10章节、原任务/AC标识与零进度守恒，V-01..10各有输入/producer→consumer/独立结果及覆盖边界；V-07/08专门区分最终数字相同的重放与重建。三文档25处本地链接、前置身份/阶段DAG及格式检查通过，1020/1021归档与已读Specs未改。先证、负控、资源/窗口测试、独立实现复审均未执行，本轮不把本合同当真实会话接线已交付。

## 10. 待澄清事项

剩余直接前置是1022正式frame；实质未决是固定普通Node模型如何由公共会话持有、增量执行和绑定UI。T-01必须明确970参考标量域能否合法承接所用Node行为，不能手工改写源业务；不能承接时给具体最小能力/原owner/已有任务或待修订合同，保持needs_replan而非降低AC。M0无参事件不等完整EVENT-B或981整包；确需新增公共语义时先修对应owner合同，不擅改他会话或重开归档970。
