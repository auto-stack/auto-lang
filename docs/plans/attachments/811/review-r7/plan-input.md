---
plan_id: PLAN-811
status: execution_done
feature_name: 媒体宿主域、资源生命周期与跨端消费整体解耦
author: [Codex]
created_at: 2026-10-19
updated_at: 2026-10-19
plan_revision: 1
current_step: 8
total_steps: 11
supersedes_spec_components: [docs/specs/auto-lang/vm/overview.md, docs/specs/auto-lang/ui/overview.md, docs/specs/auto-lang/ui/design/image-surface-events.md, docs/specs/auto-man/project.md, docs/specs/repository-boundaries/generated-ui-distribution.md]
new_spec_components: [docs/specs/auto-media-core/project.md]
touched_goals: [GOAL-003, GOAL-007]
affects: [auto-media-core, auto-lang/vm/ffi, auto-lang/ui, auto-man/generated-media]
depends_on: [PLAN-779, PLAN-790]
---

# [PLAN-811] 媒体宿主域、资源生命周期与跨端消费整体解耦

## 0. 变更摘要

交付R03剩余的完整媒体领域边界：新建`auto-media-core`承接renderer-neutral图像资产/调度/缓存/会话及本地媒体索引/Range规则；VM的17项auto.image接口、HTTP数据面、Vue生成后端、Rust merged和真实Iced渲染共享同一数据与机制。UI只留纹理/事件投影，Man只留服务器/生成接线，旧公开路径薄兼容、原算法删除，含实际线程/票据/缓存退出和独立发行消费。

不迁终端（808）或Man项目/构建规则（809），不新增图像格式/播放功能，不搬Git仓。i18n、a2r直接frame、播放器/纹理renderer及其他runtime残边明确交给后续R04，不将本Plan完成冒称所有宿主/UI已拆完。

## 1. 目标

- `ui/image_pipeline.rs`的公共值、registry、priority/latest-wins、encoded/decoded cache、worker、session/URI/HTTP规则有独立非UI所有者；`ui/media_service.rs`的索引/不跟随链接/Range决策一并迁入同包具名模块，不再残留通用媒体服务在UI目录。
- 真实VM、HTTP与renderer/生成后端使用同一registry、票据、revision及共享字节，不出现“接口都通但各自维护一份资产”的假解耦。
- 保持17项签名/编号/值域/feature开关与旧stub语义，控制面不搬像素/绝对路径；生命周期、线程、背压、缓存预算和失败出口可观测。
- 复用已有设施，不新造HTTP服务器/进程协议/OS provider或codec；管理核心和UI生成器搬迁仍由809/R04/R06完成。

## 2. 架构方案

新`crates/auto-media-core`含`image_pipeline`和`media_service`等具名模块（最终内部分层T-01冻结），仅依赖实际codec/hash等基础包，不依赖auto-lang/VM/UI/Man/iced。原`auto_lang::ui::{image_pipeline,media_service}`保薄重导出同一类型身份与同一默认服务，真实消费者逐步改依公开core入口。

媒体机制本来不依赖UI，不机械包装成OS provider。VM改轻core/runtime服务调用及编解码，票据handle映射/上限/释放规则有明确owner，不能留成多份业务逻辑；无须为本Plan引入新的任意类型provider框架。默认进程共享service的原行为保持，局部owned runtime用于可控寿命及测试；若选择可替换服务，必须额外证明已发票据归原owner、不串两VM，不能让新实例ID误碰旧资源。

## 3. 技术栈

Rust、现有image JPEG/PNG/WebP codec、blake3、Arc共享bytes、Mutex/Condvar、有界channel/worker，原HTTP response/StreamPlan值类型；实际依赖版本/features从当前Cargo取材，避免引入第二套codec特性。Iced/GPU/axum/tokio/server socket adapter留各自消费方。

## 4. 需求分析与背景调查

授权：用户要求按建议规划两份大模块合同，与808/809并行；本轮仅auto-lang仓内文档起草，不执行、不创建agent/worktree、不搬仓/写远端。[总路线](reports/v06-extraction-execution-roadmap.md)、[本批边界](reports/grammar-media-parallel-order-810-811.md)、[来源hash](attachments/grammar-media-batch-20261019/inputs.json)为调度依据。

权威输入：[VM](../specs/auto-lang/vm/overview.md)、[UI](../specs/auto-lang/ui/overview.md)媒体文件服务/未迁边界、[ImageSurface事件](../specs/auto-lang/ui/design/image-surface-events.md)、[Man](../specs/auto-man/project.md)、[发行合同](../specs/repository-boundaries/generated-ui-distribution.md)。[图像管线设计](../design/autoui/image-viewer-pipeline.md)是历史设计来源，部分状态/性能指标与当前代码不等价；T-01对账，不把设计目标自动当现行实现或降低canonical合同。

实证锚点：`auto-lang/src/ui/{image_pipeline,media_service}.rs`已大体renderer-neutral；`vm/ffi/stdlib.rs`尚有直接UI类型/registry调用、全局256 ticket handles和HTTP快路径；`vm/native_catalog.rs`与名称绑定、`auto-man/src/{api_gen,rust_ui}.rs`发射字符串、Iced renderer/media notify都在真实消费。808的`attachments/808/handoff.md`余edge表确认media与i18n未迁，此清单仅作边界输入，不消费808未合入接口。

观察到17个auto.image名字：2960–2975只覆盖其中16项，`thumb`在shim按名注册，**不能臆分第17个固定ID或改native ABI**，T-01核实实际注册/返回类型。当前request只pop宽高并原样返回handle，stats/session_stats也有现行语义，不能在迁移中偷偷补成新功能。image_pipeline采用2 decode+1 resize、队列8、registry512、会话32×256；media_service另有depth32及不跟随symlink/junction合同。两者扫描政策不同，不能为“去重”误合为同一策略。

无需等待808/809或810；涉及Man api_gen/rust_ui的媒体适配片段与809项目读取/792清单协调，不搬其生成器算法，不改变I-PROJECT。

## 5. 详细设计

### 5.1 完整能力与唯一类型/服务

| 子域 | 本Plan迁移/接线范围 |
|---|---|
| 17 native | queue/open/scan/request/retain/release/close/stats、open_session/snapshot/current_uri/names/navigate/request_view/close_session/session_stats/thumb；ID/参数pop/值型/缺席逐项冻结 |
| 资产机制 | AssetId/Key/Ticket/State/Lookup/Error、metadata/orientation/rendition、registry、TTL/references/过期/代次唤醒、缓存与预算、priority/latest-wins/worker |
| 图像会话 | 默认共享服务、host-owned session/路径/entry/handle表；scan/open/request/navigate/close和并发发布归属；返回URI/元数据，不输出像素或绝对路径 |
| 媒体出口 | 图像`/api/__auto/media/{id}/{revision}`HTTP值规则与merged像素查找；媒体文件index/resolve_root/content_type/StreamPlan/range规则；真实HTTP/文件流执行留adapter |
| 实际消费者 | VM/std HTTP快路由、Man api_gen/rust_ui媒体桥、Vue split的生成后端、Rust merged、Iced image renderer/ImageSurface事件 |

原路径重导出身份，不保第二份LazyLock registry/worker/session；Man或a2r生成代码不能各嵌一份库源造成同进程双实例。跨crate私有MediaAssetId构造等只开放必要安全观测/构造，不把所有内部状态升pub。未经支持的任意URI/路径不因抽包获得访问能力。

### 5.2 兼容与资源支持

- 现行`ui-iced`开关的注册/无此feature时行为保持；core能独立编译不等于给原VM新增图像能力。缺票/坏session/过期/关闭/队列满/解码失败有明确旧oracle，不能全部变成功或异常。
- 默认budget按实际实现核实（encoded64MiB、decoded256MiB、hard pin当前图允许临时越budget，软pin先淘汰）；1GiB编码文件/400MP上限是现行拒绝边界。检验阈值不必实际分配400MP图，以合法header/limit函数边界与小预算实压互证；不得宣称RSS永远≤256MiB或支持任意大图。旧设计额外目标仅在证实已适用时验收，差额具名登记。
- 默认进程共享handle/session兼容；retain/release/close、淘汰256 handle/512 registry/32 session/256 entry、TTL、双close、过期revision不得释放他人live资源。绝不把table.len归零当线程/bytes/GPU引用释放的全部证据。
- 核心owned runtime的shutdown/drop必须终止本runtime拥有的scan/open/decode/resize工作；**不是仅join前端句柄**。有界channel满、解码在途、close与scan/open并发、generation/revision更新、失败/取消的发布出口都需可控调度反例。外部close可保持异步返回，但其失效会话不得被迟到回调复活；剩余任务最终退出/引用释放可观测。默认进程常驻池与局部runtime寿命区分，VM drop不得误关共享全局池；不承诺中断不可取消的codec指令，只要求不再接新任务、旧工作受控收敛且关闭后不再发布可见结果。
- 不能在registry/VM/service锁内执行用户回调或等待自身worker，引入deadlock；工作线程完成/notify只改变公共媒体代次，UI构建/paint期不执行文件IO/解码或重入handler。

### 5.3 HTTP、文件流、渲染与事件

图像HTTP保持GET/HEAD/ETag/304/404/410/422/503、MIME/长度/缓存头与旧错误优先级，VM快路径仍先于用户路由，仅截获原媒体prefix、query处理按旧入口；非媒体路由不被抢。响应bytes仍Arc共享，HEAD无body；当前行为与文档若冲突先取证列明，禁止靠重写golden掩盖。

media_service索引深度/扩展名/自然排序/根路径优先级/不跟随reparse、token/路径隐藏及Range含边界/空文件/非法范围、实际部分流输出按现行Spec保留；不扩大文件可达面，不把多GB视频read_to_end。codec/播放器/音视频事件不在本Plan重写范围。

VM/Rust merged renderer对媒体URI先查同进程共享registry/decoded pixels，不绕HTTP/base64；Vue只从实际生成后端取媒体，不搬到浏览器另一套扫描/解码业务。ImageSurface loaded/error保持Ready且下一渲染拍再单发、handler形参投影、换src/卸载退订、A→B→A与50ms媒体代次唤醒；不得以registry Ready直接判已渲染/loaded。旧codec/EXIF/crop/rotate/尺寸策略和纹理引用释放由实际消费验证。

### 5.4 发行与剩余owner

core normal/build/dev无auto-lang/VM/UI/Man/iced，独立manifest fresh解析+locked构建；必要codec/build输入/资源的777/792 source清单及生成应用依赖/重定位随本Plan维护。旧公开路径兼容但生成代码实际能消费新包，不以字符串assert替代编译运行。

本Plan负责image_pipeline和media_service全部非renderer机制，不承接i18n/直接frame等不相关族。T-08余edge报告将其逐项交给R04运行桥/默认组合包，R04须在自己的合同中验收，不能丢弃或声称本Plan后全R03/UI已完成。纯算法复制、长期公共类型和资源来源都清楚，不把移文件算全生态净删。

### 规范增量

| delta_id | add/modify/retire | docs/specs/... target | before/after rule | rationale | acceptance IDs |
|---|---|---|---|---|---|
| SD-01 | add | docs/specs/auto-media-core/project.md | UI目录的中立媒体机制→独立单源类型/共享服务、资源/寿命/支持合同 | 可独立消费 | AC-01..05 |
| SD-02 | modify | docs/specs/auto-lang/vm/overview.md | 17 native及HTTP UI直连→非UI服务/编解码、开关及stub冻结 | VM脱UI类型 | AC-01, AC-02, AC-04 |
| SD-03 | modify | docs/specs/auto-lang/ui/overview.md | 图像/媒体索引/Range机制迁core、renderer及剩余owner真实列明 | 不虚报全UI独立 | AC-01..05 |
| SD-04 | modify | docs/specs/auto-lang/ui/design/image-surface-events.md | 规则保持，registry/代次owner改core、事件与渲染拍留UI | 保护loaded/error | AC-04 |
| SD-05 | modify | docs/specs/auto-man/project.md | 媒体生成/HTTP接线消费新core；保留PLAN-809已落地的auto-man-core规则源与语言效果adapter分工，UI工具等剩余owner如实列明 | 对齐当前canonical的809/R04边界 | AC-04, AC-05 |
| SD-06 | modify | docs/specs/repository-boundaries/generated-ui-distribution.md | 新core及codec/source闭包进入受影响交付profile | 独立可分发 | AC-05 |

## 6. 测试设计

T-01冻结实际支持/资源/错误矩阵和旧臂；T-07建立新增`crates/auto-lang/src/tests/plan811_media_runtime.rs`、`crates/auto-man/tests/plan811_media_consumers.rs`及`test/fixtures/modularity/media-core-consumer/`。以下V项owner_stage=work，均为拟执行的验收，不是本轮证据。

| Case | AC / invariant | 输入与真实入口 | 区分性结果／覆盖边界 |
|---|---|---|---|
| V-01 | AC-01 全集/同一资产 | 17真实VM注册调用及旧公开路径、core独立consumer；两VM/default service→HTTP→merged renderer | 名称/实际ID/参数pop/返回型/feature关语义与旧臂相同；同URI/ticket在不同消费者命中同一资产与类型身份，request旧stub不被升级；无UI闭包 |
| V-02 | AC-02 票据/会话 | retain/release/close/双close、超cap淘汰、TTL/坏票/过期revision、32+1会话与256+1 entry、close时populate受控暂停 | 引用owner正确、关闭不复活/不回灌另一会话、handle失效与返回值有旧oracle；不用相同数值但不同service的假票据证明隔离 |
| V-03 | AC-03 并发/资源 | 小预算cache、hard/soft pin、8队列背压、旧generation迟到；满resize channel与decode在途时shutdown/drop，scan/open关闭竞争 | latest-wins旧结果不可覆盖、队列/cache服从实际预算含hard pin例外；有外部截止及超时清理；实际worker/scan退出、弱引用/文件/bytes释放可观测，不只caller返回/计数归零 |
| V-04 | AC-03 支持/格式 | JPEG/PNG/WebP、EXIF/rotate/crop、坏头/缺文件/不支持格式；1GiB与400MP边界函数/metadata，真实小图decode，media_service索引/reparse/Range边界 | 同旧bytes/像素/错误分类；合法临界值可准入，超限具名拒绝；保Range与索引策略，测试不创建worktree内symlink/junction、不强行分配极大图 |
| V-05 | AC-04 真生产出口 | VM HTTP与生成Rust HTTP真实本地TCP，GET/HEAD/ETag/错票/非媒体用户路由；Vue split、VM/Iced、Rust merged ImageSurface loaded/error/换源/卸载 | 真实状态/头/流bytes与冻结oracle，非媒体路由仍可用；merged不走HTTP/JSON像素；Loaded晚于渲染拍且单次，坏票error单次，A→B→A不串、空闲媒体完成唤醒；不能只mock/断言生成字符串 |
| V-06 | AC-05 独立发行 | 新consumer fresh/locked、792实际生成工程fresh/重定位locked，缺包/非法反向边负控、算法调用审计 | 单份codec/registry/资源来源，编译运行真消费者、负控红；原UI机制实现退役，剩余非媒体edge逐项有owner |

work：`cargo check -p auto-media-core -p auto-lang -p auto-man`、`cargo test -p auto-media-core`、`cargo test -p auto-lang --lib plan811`、`cargo test -p auto-man --test plan811_media_consumers`、新consumer manifest；原media worker/session/HTTP/Range/事件相关定向测试。T-01记录真实app命令/driver、平台及工具环境，不虚构flags；跨端实机使用既有autoui-verifier脚本，当前必需环境不足为pending，不把未测标pass。HTTP用OS临时端口及有界退出，不改全服务watch/daemon设施。

777：`python scripts/modularity-audit.py --mode strict --no-cargo-check`；792沿现有`python test/fixtures/modularity/generated-ui-consumer/driver.py`受影响实际交付门。生命期/真消费者的必要定向证据必须work完成，不能全部后置给merge。无agent/model指定，不创建其他会话。

review同树独立上下文审AC/SD与原始反例，merge在冻结提交统一裸`cargo t`+`cargo tv`（VM触面）；若实际改ui_gen则加`cargo tu`、trans则`cargo tt`。Man生成器包的必要测试/真实媒体TCP控制也纳入最终适用证据；代码/依赖/配置/输入未变可说明复用，否则重放受影响项。tf仅主检出到期单实例批量，开发/复审/送审不跑整档。本轮纯规划不跑Cargo/docs_gen。

## 7. 验收标准

| ID | owner_stage | 可观察标准与证据 |
|---|---|---|
| AC-01 | work | 非UI媒体机制/数据/媒体文件服务单源，17项真实VM及旧路径同型同service，feature/stub保持，闭包无UI；V-01 |
| AC-02 | work | 票据/引用/失效/关闭/并发会话与cap owner正确、跨消费者同资产，关闭无复活；V-02 |
| AC-03 | work | 调度/cache/预算/格式与Range原支持保持，局部runtime有实际shutdown/drop/scan/worker退出与失败清理证据；V-03/04 |
| AC-04 | work | 真实HTTP/Vue split/VM/Iced/Rust merged及ImageSurface渲染后事件通过，不用mock/生成字符串代替；V-05 |
| AC-05 | work | 独立fresh/locked、777/792/生成工程消费通过，UI旧算法退役、非媒体余edge/资源与SD草案完整；V-06 |
| AC-06 | review | 同树独立AC/SD/生命周期/真消费者审查绑定最终修订，零新增未处理warning/格式/debug；merge门pending明确 |
| AC-07 | merge | 冻结门、规范/ledger、归档/发行交接及guard clean后清理均有实际收据 |

## 8. 执行步骤

共11项，五阶段完成整个媒体模块；不再把票据DTO/单个native/HTTP头分别立小Plan。

| 状态 | ID | owner_stage | 依赖 | 实际文件/动作、交付与验证 | AC |
|---|---|---|---|---|---|
| [x] | T-01 | work | 原provider/支持合同已交付 | 新`attachments/811/contract-baseline.md`：image_pipeline/media_service/17 native/HTTP/生成/renderer完整读写图；旧oracle/预算/实际ID与thumb按名注册、当前设计差额、service owner/scan关闭/shutdown满channel有界probe、真消费者命令矩阵；阻塞点先决策 | AC-01..05 |
| [x] | T-02 | work | T-01 | 新`auto-media-core`及轻consumer骨架，迁公共类型/registry/metadata/rendition/HTTP规则，UI路径薄重导出同型同service；check/V-01/04 | AC-01,03 |
| [x] | T-03 | work | T-02 | 迁cache/queue/latest-wins/worker/scan/session，建立冻结owned runtime及默认共享门面；必要关闭/迟到发布/锁序有界修复不改变支持输入，V-02/03 | AC-02,03 |
| [x] | T-04 | work | T-03 | media_service全部索引/Range纯规则迁同包，保旧路径/根优先/路径隐藏/链接规则；codec与图像阈值定向V-04 | AC-01,03 |
| [x] | T-05 | work | T-04 | VM stdlib17项/ticket映射及HTTP适配消费core，移除UI类型与业务规则重复；保持开关/注册和用户路由，真VM/HTTP定向 | AC-01,02,04 |
| [x] | T-06 | work | T-05 | Iced renderer/notify与Man api_gen/rust_ui媒体生成/HTTP消费同一core；不搬809项目读取/生成算法；真生成编译与事件/渲染/流输出对照 | AC-04 |
| [x] | T-07 | work | T-06 | 建plan811/Man consumer/独立manifest，V-01..06含真实三形态和资源退出；777/792并集/受影响fresh/locked与负控 | AC-01..05 |
| [x] | T-08 | work | T-07 | SD-01..06草案、净删/迁出/剩余owner与资源清单，新`attachments/811/handoff.md`绑定修订/环境/输入/原始结果；提交送审 | AC-05 |
| [ ] | T-09 | review | T-08 | 同树独立完整媒体AC/SD/close-vs-scan/满channel/真消费者及旧路径审查，保留merge门pending | AC-06 |
| [ ] | T-10 | merge | T-09 pass | 公共装配/792/777并集合流后必要复审，最终冻结提交运行§6适用门；失败归因修复再审重冻 | AC-07 |
| [ ] | T-11 | merge | T-10 | canonical Spec/ledger沉淀、归档/来源接口交接，guard clean再清理唯一树/分支/组，核对终态计数 | AC-07 |

handoff：`D:/autostack/.wt/lang-811/auto-lang`，分支`codex/plan-811-dev`；唯一实现树供work/review/merge顺序使用，work记录actual base/兄弟依赖。与810无业务硬依赖，与808/809保持具名片段边界；不在worktree内创建任何junction/symlink，链接反例用既有只读fixture或可控metadata假体并注明覆盖限制。

## 9. 复审记录

2026-10-19 / stage: new / PLAN-811 revision: 1 / outcome: pass（合同文档预检，**非实施/独立验收pass**）/ next: work T-01 / changed: T-01..11、AC-01..07、SD-01..06。当前未执行未来测试或Cargo；来源hash/取号/预检见本批附件，不以历史设计的Implemented标签替代本Plan真实生产证据。


2026-10-19 / stage: work / PLAN-811 revision: 1 / outcome: pass（work 完成，**非独立验收pass**）/ worktree: D:/autostack/.wt/lang-811/auto-lang / branch: codex/plan-811-dev / base: ab08955c2 / code: 0f2ab56a9（T-02..T-07 十提交）/ 依赖: auto-down 兄弟 detached c39ac7f9（零源改） / task_ids: T-01..T-08 / evidence: [handoff](attachments/811/handoff.md)§3（core 30/30、plan811 7/7、Man 3/3、777 strict pass、792 25/25、back_proxy th 媒体面 4/4、parity PASS；预存红×3 基线重放归因；V-05 浏览器实机=环境 pending 如实登记） / blockers: 无 / next: review T-09（merge 门 pending：冻结提交裸 cargo t + tv）

2026-10-19 / stage: review / review_scope: final / reviewer_identity: 当前 Codex 独立复审上下文（本会话未实施；具体模型/会话 ID 未暴露） / PLAN-811 revision: 1 / outcome: **needs_fix** / worktree_path: D:/autostack/.wt/lang-811/auto-lang / branch: codex/plan-811-dev / reviewed_commit: 0f2ab56a991e5c1ca7ac599ba2ca0bd68bf4f488 / base_commit: ab08955c241f6a824091c53d3b02894ca17e193b / dependency_revisions: auto-down c39ac7f9dacdc769fc4a5ad3b98ec896240b28e8 / spec_inputs与冻结SD: [baseline](attachments/811/review-r1/baseline.json)、[SD原件](attachments/811/review-r1/spec-delta-input.md) / acceptance_results: AC01 partial、AC02 fail、AC03 fail、AC04 fail、AC05 partial、AC06 fail、AC07 merge pending / findings: F811-R1-01(P1) scan/open线程未纳入shutdown/drop，三次Drop后扫描继续变更registry；F811-R1-02(P2) 已关闭registry新票据仍可发布Ready、session仍准入；F811-R1-03(P2) V05真实三形态/渲染后事件与生成媒体HTTP/流输出未完成，不能移交R04代替验收；F811-R1-04(P3) 六个新增/门面文件格式红 / evidence: [完整R1/反例/AC-SD映射](attachments/811/review-r1/review.md)，独立core构建与30/30测通过仍不覆盖这些缺口，现有792收据已冻结但未冒记本轮实跑 / pending_closeout_gates: 最终审查pass后冻结裸cargo t+tv、最终并集与必要受影响发行验收、规范/ledger/归档/guard清理 / next: 同原树work修复及补必要oracle、提交后复审；executing **3/11**，保留T02/04/05，重开T01/03/06/07/08。未改语义合同、实施源码或合入；当前未就绪，历史work pass不代表现状完成。


2026-10-19 / stage: work / PLAN-811 revision: 1 / outcome: pass（R1 修复完成，**非独立验收pass**）/ worktree: D:/autostack/.wt/lang-811/auto-lang / branch: codex/plan-811-dev / base: ab08955c2 / code: 33fc23208（R1 修复 c66ca5a6a+33fc23208，冻结实绩 12 提交）/ 依赖: auto-down 兄弟 c39ac7f9 / task_ids: T-01/03/06/07/08 重开后再闭合 / evidence: [fix-r1.md](attachments/811/fix-r1.md)（F01/02 复审探针重放全绿+34/34；F03 四形态真机收据 9/9+7/7+6/6+13/13；F4 复审口径 --check exit 0；预存缺口三项具名 R04：merged 事件分发/sweep 静止边界/031 生成后端破损基线归因）/ blockers: 无 / next: review T-09（R2；merge 门 pending：冻结提交裸 cargo t + tv + 777 strict 复核）

2026-10-19 / stage: review / review_scope: final / reviewer_identity: 当前Codex独立复审上下文（本会话未实施；具体模型/会话ID未暴露） / PLAN-811 revision: 1 / outcome: **needs_fix（R2）** / worktree_path: D:/autostack/.wt/lang-811/auto-lang / branch: codex/plan-811-dev / reviewed_commit: 33fc23208fb6e7b22f4b57b7101efaa553b4c8f1 / base_commit: ab08955c241f6a824091c53d3b02894ca17e193b / dependency_revisions: auto-down c39ac7f9dacdc769fc4a5ad3b98ec896240b28e8 / spec_inputs与冻结SD: [baseline](attachments/811/review-r2/baseline.json.txt)、[有效SD](attachments/811/review-r2/spec-delta-input.md) / acceptance_results: AC01 partial、AC02 fail、AC03 fail、AC04 fail、AC05 partial、AC06 partial、AC07 merge pending / findings: F811-R2-01(P2) open通过关闭检查后暂停、shutdown完成再恢复，仍插入有效session；F811-R2-02(P2) scoped-thread超时panic仍join，100ms反控1500ms后才返回，满resize/在途的受控oracle仍缺；F811-R1-03继续OPEN，merged收据明确未断言loaded/error、实际一直loading，静止绑定等原合同项仍移交R04 / closure: R1顺序原例/三次scan退出已通过，R1-04格式CLOSED（十文件check exit0）；core34/34通过 / evidence: [完整R2/原例闭合/交错与截止反控/收据核对](attachments/811/review-r2/review.md) / pending_closeout_gates: 最终pass后冻结裸cargo t+tv、最终并集与受影响发行验收、规范/ledger/归档/guard清理 / next: executing **3/11**，保留T02/04/05，重开T01/03/06/07/08；第二次final未过，下次work先补报告§7诊断与覆盖/设计调整表，原树修复提交后复审。登记不批准延期/缩面；未改实施、canonical/ledger或合入。


2026-10-19 / stage: work / PLAN-811 revision: 1 / outcome: pass（R2 修复完成，**非独立验收pass**）/ worktree: D:/autostack/.wt/lang-811/auto-lang / branch: codex/plan-811-dev / base: ab08955c2 / code: 4a1d8fcf3（R2 修复单提交；累计 13 实施提交）/ 依赖: auto-down 兄弟 c39ac7f9 / task_ids: T-01/03/06/07/08 二次重开后再闭合 / evidence: [fix-r2.md](attachments/811/fix-r2.md)（R2-01/02 复审探针全绿+core 38/38；R1-03 全矩阵：VM 12/12、Vue 8/8、merged 11/11、生成 HTTP 13/13，收据含 provenance；792 修复树重跑 25/25）/ blockers: 无 / next: review T-09（R3；merge 门 pending：冻结提交裸 cargo t + tv + 777 strict 复核）

2026-10-19 / stage: review / review_scope: final / reviewer_identity: 当前Codex独立复审上下文（未实施；具体模型/会话ID未暴露） / PLAN-811 revision: 1 / outcome: **needs_fix（R3）** / worktree_path: D:/autostack/.wt/lang-811/auto-lang / branch: codex/plan-811-dev / reviewed_commit: 4a1d8fcf3a72fcb6a524b642669ff0340dabcf5d / base_commit: ab08955c241f6a824091c53d3b02894ca17e193b / dependency_revisions: auto-down c39ac7f9dacdc769fc4a5ad3b98ec896240b28e8 / spec_inputs与冻结SD: [baseline](attachments/811/review-r3/baseline.json.txt)、[SD](attachments/811/review-r3/spec-delta-input.md) / acceptance_results: AC01/02 partial、AC03 fail、AC04 fail、AC05/06 partial、AC07 merge pending / closure: R2-01受控关闭竞态CLOSED，R1顺序/三次scan退出及Weak正控过；修改两文件格式过、core38/38 / findings: F811-R3-01(P2) 新typed通知仅按src合并不同handler，真窗口同URI loaded A0/B1、error A1/B0，换handler后B0；F811-R3-02(P2) merged无独立媒体唤醒，无应用timer挂载Ready图片2s后loaded仍0；F811-R2-02继续OPEN，当前helper100ms及时红但自有线程/bytes未回收，running≥9仅证明至少6等待不足以证8槽满 / evidence: [完整R3/真实Iced窗口正反控/旧例闭合/准确helper反控](attachments/811/review-r3/review.md) / pending_closeout_gates: 最终pass后冻结裸t+tv、最终并集/受影响发行、规范ledger/归档guard清理 / next: executing **3/11**，保留T02/04/05、重开T01/03/06/07/08；原树work补订阅身份/无timer/owner清理与准确通道状态矩阵，修复提交后复审。原合同未改，登记不批准延期/缩面；未改实施或合入。


2026-10-19 / stage: work / PLAN-811 revision: 1 / outcome: pass（R3 修复完成，**非独立验收pass**）/ worktree: D:/autostack/.wt/lang-811/auto-lang / branch: codex/plan-811-dev / base: ab08955c2 / code: 27d3387af（R3 修复单提交；累计 14 实施提交）/ 依赖: auto-down 兄弟 c39ac7f9 / task_ids: T-01/03/06/07/08 三次重开后再闭合 / evidence: [fix-r3.md](attachments/811/fix-r3.md)（R3-01 复审 typed-consumer 四行全绿；R3-02 无 timer 探针 5/5；R2-02 双负控+占用仪表+scan 门；core 40/40；五 harness 全矩阵重绑本提交）/ blockers: 无 / next: review T-09（R4；merge 门 pending：冻结提交裸 cargo t + tv + 777 strict 复核）

2026-10-19 / stage: review / review_scope: final / reviewer_identity: 当前Codex独立复审上下文（未实施；具体模型/会话ID未暴露） / PLAN-811 revision: 1 / outcome: **needs_fix（R4）** / worktree_path: D:/autostack/.wt/lang-811/auto-lang / branch: codex/plan-811-dev / reviewed_commit: 27d3387af988fd7f88b3cc392e043fb98bb3026e / base_commit: ab08955c241f6a824091c53d3b02894ca17e193b / dependency_revisions: auto-down c39ac7f9dacdc769fc4a5ad3b98ec896240b28e8 / spec_inputs与冻结SD: [baseline](attachments/811/review-r4/baseline.json.txt)、[SD](attachments/811/review-r4/spec-delta-input.md) / acceptance_results: AC01 partial、AC02/03/04 fail、AC05/06 partial、AC07 merge pending / closure: R3-01身份矩阵CLOSED，同步无timer原例、R2-01关闭准入竞态及原scan/Weak继续通过；core40/40、两文件格式过 / findings: F811-R4-01(P2) resize_backlog空闲shutdown后确定性下溢panic，send/recv/Stop计数不能作精确占用；F811-R4-02(P2) shutdown_started令二次关闭0ms返回时registry仍开放，新ticket与转换仍准入；F811-R3-02继续OPEN，with-task无timer仍loaded0，loaded回调改绑Ready B后B0 / evidence: [完整R4/旧例闭合/生产核心与七窗口重放](attachments/811/review-r4/review.md) / spec_delta: SD05已对齐主检出809已迁auto-man-core事实，未改AC/语义revision；原work五收据实际记4a非自述27，需等价绑定说明 / pending_closeout_gates: 最终pass后冻结裸t+tv、最终并集/受影响发行、规范ledger/归档guard清理 / next: executing **3/11**，保留T02/04/05、重开T01/03/06/07/08；原树work按报告诊断修复计数/关闭完成/双入口与回调后调度，提交后复审。未改实施、canonical或合入，不批准延期/缩面。


2026-10-19 / stage: work / PLAN-811 revision: 1 / outcome: pass（R4 修复完成，**非独立验收pass**）/ worktree: D:/autostack/.wt/lang-811/auto-lang / branch: codex/plan-811-dev / base: ab08955c2 / code: 53f1111a2（R4 修复单提交；累计 15 实施提交）/ 依赖: auto-down 兄弟 c39ac7f9 / task_ids: T-01/03/06/07/08 四次重开后再闭合 / evidence: [fix-r4.md](attachments/811/fix-r4.md)（R4-01/02 复审重放全绿：backlog 不下溢、重复关闭完成语义；R3-02 余量：async-no-timer a=1、cascade a=1 b=1；core 42/42；六 harness 提交后实跑重绑 53f1111a2；792 重跑 25/25）/ blockers: 无 / next: review T-09（R5；merge 门 pending：冻结提交裸 cargo t + tv + 777 strict 复核）

2026-10-19 / stage: review / review_scope: final / reviewer_identity: 当前Codex独立复审上下文（本会话未实施；具体模型/会话ID未暴露） / PLAN-811 revision: 1 / outcome: **needs_fix（R5）** / worktree_path: D:/autostack/.wt/lang-811/auto-lang / branch: codex/plan-811-dev / reviewed_commit: 53f1111a2923a2bcca0096f973eb714a94147a06 / base_commit: ab08955c241f6a824091c53d3b02894ca17e193b / dependency_revisions: auto-down c39ac7f9dacdc769fc4a5ad3b98ec896240b28e8 / spec_inputs与冻结SD: [baseline](attachments/811/review-r5/baseline.json.txt)、[SD](attachments/811/review-r5/spec-delta-input.md) / acceptance_results: AC01/02/04 pass、AC03 fail、AC05 partial、AC06 fail、AC07 merge pending / closure: R4-02重复关闭完成语义CLOSED，R3-02同步/async无timer与回调换图CLOSED；R3-01身份、原open竞态/scan/Weak保持；core42/42、plan811七测、Man三测、HTTP13、777 strict与格式过 / findings: F811-R4-01(P2)空闲下溢旧例闭但完整仪表仍OPEN，三次真实在途关闭后backlog=1而workers=0/pending=0/registry已关；send前预留量含阻塞发送者，不能直接证明8槽满；F811-R5-01(P3)新增并发关闭测试unused Ordering导入 / evidence: [完整R5/在途关闭生产反例/旧例闭合/AC-SD](attachments/811/review-r5/review.md) / spec_delta: 当前SD05继续保留809已迁事实，未改AC/语义revision；work六收据当前SHA匹配，HTTP现存exe与旧hash不同已独立13项重放重绑 / pending_closeout_gates: 最终pass后冻结裸t+tv、最终并集/777/受影响发行、canonical/ledger/归档guard清理 / next: executing **4/11**，保留T02/04/05及本轮已闭T06，重开T01/03/07/08；原树work修复准确占用/Stop后Work结算与真实满通道oracle，清新增warning，提交后复审受影响项；不重开已闭事件，不以饱和减/事后归零代替oracle。未改实施/canonical/live ledger或合入，未批准延期/缩面。


2026-10-19 / stage: work / PLAN-811 revision: 1 / outcome: pass（R5 修复完成，**非独立验收pass**）/ worktree: D:/autostack/.wt/lang-811/auto-lang / branch: codex/plan-811-dev / base: ab08955c2 / code: 2ffb87755（R5 修复单提交；累计 16 实施提交）/ 依赖: auto-down 兄弟 c39ac7f9 / task_ids: T-01/03/06/07/08 五次重开后再闭合 / evidence: [fix-r5.md](attachments/811/fix-r5.md)（R4-01 余量：occupancy-r5 探针三试 after_shutdown=0、backlog≥8 语义可区分；R5-01：test cfg 零 warning；core 43/43；六 harness 重绑 2ffb87755；792 重跑 25/25）/ blockers: 无 / next: review T-09（R6；merge 门 pending：冻结提交裸 cargo t + tv + 777 strict 复核）

2026-10-19 / stage: review / review_scope: final / reviewer_identity: 当前Codex独立复审上下文（本会话未实施；具体模型/会话ID未暴露） / PLAN-811 revision: 1 / outcome: **needs_fix（R6）** / worktree_path: D:/autostack/.wt/lang-811/auto-lang / branch: codex/plan-811-dev / reviewed_commit: 2ffb87755e482fb39f01123d28e3f82fa3e373c1 / base_commit: ab08955c241f6a824091c53d3b02894ca17e193b / dependency_revisions: auto-down c39ac7f9dacdc769fc4a5ad3b98ec896240b28e8 / spec_inputs与冻结SD: [baseline](attachments/811/review-r6/baseline.json.txt)、[SD](attachments/811/review-r6/spec-delta-input.md) / acceptance_results: AC01/02/04/05 pass、AC03 partial、AC06 fail、AC07 merge pending / closure: F811-R4-01占用残留/下溢CLOSED（原在途反例三次backlog0/worker0/registry终态），F811-R5-01新增warning CLOSED（test cfg check零warning）；原重复关闭/准入/scan/Weak及七窗口继续过 / findings: 仅F811-R2-02(P2)满通道fixture继续OPEN，核心初跑42/43，唯一红peak_running=8；随后单用例3/3绿但源码12次尝试/accepted≥8不能保证running≥9和8槽满的前置条件，不以重跑取绿冒充稳定验收 / evidence: [完整R6/初次红与三次绿原件/源码诊断/闭合表](attachments/811/review-r6/review.md)；plan811七测/Man三测/HTTP13/777 strict/格式过，六work收据当前SHA核对；HTTP现存hash变化已新跑重绑 / pending_closeout_gates: 最终pass后冻结裸t+tv、最终并集/777/受影响发行、canonical/ledger/归档guard清理 / next: executing **5/11**，保留T02/T03/T04/T05/T06，重开T01/T07/T08；原树work仅修有界受控造载/真实满状态与诊断，提交后受影响复审，不重开已闭生产计数/关闭及事件实现。未改实施/canonical/live ledger或合入，未改合同/revision，未批准延期或缩面。

2026-10-19 / stage: work / PLAN-811 revision: 1 / outcome: pass（R6 修复完成，**非独立验收pass**）/ worktree: D:/autostack/.wt/lang-811/auto-lang / branch: codex/plan-811-dev / base: ab08955c2 / code: 04f7446fd（R6 修复单提交；累计 17 实施提交）/ 依赖: auto-down 兄弟 c39ac7f9（零源改）/ task_ids: T-01/T-07/T-08 六次重开后再闭合 / evidence: [fix-r6.md](attachments/811/fix-r6.md)（F811-R2-02 fixture 前置条件：正控改有界补入——预算24/墙钟20s/被拒offer重试/观测到满即止/失败报告accepted与双峰值及末采样；新增未满判别负控——8项上限按算术running≤8、backlog≤7且正常收敛Ready归零；满态shutdown增worker_count=0与真终态仪表0断言。验证：正控16绿（单跑+钉核6副本/2核+并行8副本+编译负载下模块面44/44 @74.13s），零warning、rustfmt过；test-only触面lib产物不变，消费者收据维持R6绑定）/ blockers: 无 / next: review T-09（R7；merge 门 pending：冻结提交裸 cargo t + tv + 777 strict 复核）

## 10. 待澄清事项

无现在需用户先答的阻塞。T-01负责默认共享service与owned runtime边界、thumb实际注册和旧设计差额；不允许通过关闭feature、取消真实消费平台或全用fake使验收变绿。若需新增native ID/改变公开语义、缩小格式/资源支持、扩大到相邻仓或播放器重写，具体needs_replan；原本未实现的策略/优化不自动加入本Plan，只清偿本次边界必要的关闭/资源所有权问题。
