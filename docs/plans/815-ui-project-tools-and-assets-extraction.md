---
plan_id: PLAN-815
status: executing
feature_name: UI 工程工具、生成入口与专属资源脱离 Man
author: [Codex]
created_at: 2026-10-19
updated_at: 2026-10-20
plan_revision: 2
current_step: 4
total_steps: 14
supersedes_spec_components: [docs/specs/auto-man/project.md, docs/specs/auto-lang/ui/overview.md, docs/specs/project-model/project.md, docs/specs/repository-boundaries/generated-ui-distribution.md]
new_spec_components: [docs/specs/auto-ui-tools/project.md, docs/specs/tool-protocol/project.md]
touched_goals: [GOAL-003, GOAL-007]
affects: [auto-ui-tools, auto-tool-protocol, auto-man/compat, ui-assets]
depends_on: [PLAN-809, PLAN-814]
---

# [PLAN-815] UI 工程工具、生成入口与专属资源脱离 Man

## 0. 变更摘要

完成R04工程工具半部：Man内Vue/Rust/Jet/Ark/Tauri等实际UI生成/脚手架、gallery/wm/shadcn等专属资产及其调用闭包迁到独立UI工具包，消费已交付I-PROJECT和814公开SDK/UI接口，不反依赖Man管理大库。同步交付轻I-TOOL执行协议与实际UI工具服务入口，供816消费，避免双向等待。包含真生产、旧门面兼容、资源/发行及失败证据。

不搬通用锁/resolver/Builder/模板算法，不把整个api_gen或http_service按文件名盲搬，不迁Git仓。I-TOOL不是另一个小Plan，使用809已交付的单源请求类型。

本次r2按M815-A真实阶段发布、M815-B完整交付组织，不增加Plan编号、不降低原最终AC。执行与审查修复并行规则见[阶段交付与并行审查修复合同](reports/staged-delivery-review-814-816.md)；阶段pass不等于整Plan交付。

## 1. 目标

- UI工程工具可脱Man实现独立生成/构建受支持工程，项目配置经唯一轻合同输入，不回头读取Pac/AutoConfig或主pac.at形成第二数据源。
- UI资产/示例模板/构建输入有唯一来源，旧路径只转发；既有语义与真实支持profile不因搬移丢失。
- I-TOOL数据/版本/能力/producer/资源/结果/错误/生命周期合同、实际工具入口及真实外部调用可用；CLI不需静态链接UI工具实现。
- 通用非UI服务/语言adapter与上层兼容的归属明确，815与816不都拥有同一个生成器/资源或重复执行设施。

## 2. 架构方案

新`crates/auto-ui-tools`（候选名T-01固定）→814公开UI/compiler/runtime/SDK、已合入auto-project-model请求与轻协议；不依赖auto-man/旧auto-lang兼容大包。新`auto-tool-protocol`只持轻请求/响应与版本能力/资源描述，复用I-PROJECT唯一类型和必要可序列化支持；无AST/VM/renderer/Man。

实际`auto-ui-tool`可执行入口由工具包拥有，消费版本化协议，独立打包。旧`auto_man::vue/rust_ui/...`等签名若需暂保，投影adapter在上层兼容承载；816把其与纯管理服务闭包分开。不能一面在管理服务normal闭包重导出UI工具，一面宣称管理服务已脱UI。

## 3. 技术栈

现有Rust生成器/脚手架、rust-embed资产、auto-gen-core/既有发布guard、814 UI公开包、809 I-PROJECT；协议使用有界、具名、版本化数据，不重新定义一套project/AST、runner或构建缓存。具体wire/framing/stdio传输在T-01整链probe后冻结。

## 4. 需求分析与背景调查

用户2026-10-20明确要求修订现有814/815/816，允许阶段交付及接口合入后放行下游、并行审查验证修复，不增加编号。原目标/最终AC/支持面不变；本轮只改主检出规划文档，不改实现树、canonical Specs或远端。 来源与当前hash见[本轮输入](attachments/sdk-tools-r2-20261020/inputs.json)。

用户授权下一批较大模块规划，当前只在auto-lang仓写计划/设计，不实施、不创建agent/树、不写远端。来源：[路线](reports/v06-extraction-execution-roadmap.md)、[批次](reports/sdk-tools-parallel-order-814-816.md)、[输入hash](attachments/sdk-tools-batch-20261019/inputs.json)。

权威：[Man](../specs/auto-man/project.md)、[UI](../specs/auto-lang/ui/overview.md)、[project-model](../specs/project-model/project.md)、[交付](../specs/repository-boundaries/generated-ui-distribution.md)。809/I-PROJECT已归档交付b819a602e，以当前Spec和实际同源类型/codec为准；814按本批r2阶段合同提供SDK/UI出口，不冻结其WIP。

实证：`auto-man/src/{vue,rust_ui,jet,ark,tauri,vue_shadcn,gallery_assets,wm_assets,gallery_cache,image_viewer,api_gen,tauri_backend,vscode,sidecar,http_service}.rs`及`assets/{gallery,wm,shadcn-ui}`；Vue/Rust等当前使用Pac/语言生成API，api_gen同时有非UI服务/媒体适配，http_service承载独立server命令；需逐符号而非整文件武断归属。`rust_ui.rs`还持792发行管线与producer/source表；原资产使用编译期embed和邻仓输入，必须迁完整来源。

### r2阶段启动门

- 809/I-PROJECT已交付，T-01/T-02/T-03及T-12可现在实施；**协议与真实窄端点不依赖814**。
- **M815-A**：T-01/T-03→T-12→T-13→T-14；实际协议/tool发布后放行816客户端。T-02可与协议工作分scope推进，但不提前搬走旧生产embed源。
- **M815-B**：T-04生成主体等M814-A+M815-A；T-05中依赖新runtime的真实build/run集成等M814-B，随后T-06..11完整收口。生成主体的独立审查/修复无需等runtime完整发布。
- 不复制I-PROJECT，不消费814 WIP，不用假backend提前报告生成/运行支持。

## 5. 详细设计

### 5.1 完整owner图与I-PROJECT

UI工程遍历/模板/脚手架/生成/资源发布依实际调用图迁入；主项目声明读取经809请求，mode/output_root/input_sources/dependency roots、sidecar/front/theme/backend/locale/env覆盖等按已合入规则保持。必要跨文件输入由请求/显式source adapter提供，不暗中重新求值主pac.at。动态配置留显式语言adapter，不把静态scanner当全部配置。

仅UI服务壳/媒体生成桥归UI工具；通用API提取、语言trans/server配置、非UI模板、compiler/PATH/Git/FS/Builder动作留语言/管理adapter或已有共享包，边界由T-01逐符号清单固定。若拆文件，单份算法/类型，不保两套server generator；shared helper只为真实消费者抽出且不建巨型common。

### 5.2 资源与生成产物

gallery/wm/shadcn、UI专用模板/样式/i18n/图标等源码与embed/include/生成schema、drawlist等外部构建输入有owner/source/许可证/定位清单；803既有通用templates/GHS/IAR资源不重新迁。默认/未知族、覆盖顺序/文件字节、主题/sidecar/外部backend/项目路径相对源根行为冻结。T-02纯资产阶段不动生产路径，完整接线T-04后删除原重复来源。

792受管发布/producer/lock/修剪/failure facility复用并扩至新工具，不能靠扩大allowlist漏资源或依赖父仓；非受管旧模板的既有破坏性失败窗口按已确认语义记录，不突然承诺所有生成文件原子事务。新协议的能力/版本拒绝须发生在副作用之前；真正发布中断的可恢复/保留行为按适用profile精确验证。

### 5.3 I-TOOL接口交接（A发布协议与窄端点，B补全UI能力）

覆盖协议版本/operation、能力与profile、请求身份/来源、I-PROJECT轻payload、producer版本/指纹、资源根/输出根、产物与诊断/错误、进度/日志/取消/退出及结果与运行进程寿命。请求只承载具名数据，不让AST/VMValue/裸指针/任意callback跨工具边界。若需要wire序列化，优先在数据owner提供同源codec，不复制请求结构再双向丢字段。

T-01冻结适用平台、实际codec/framing、消息/输出/排队预算与单位、版本/未知能力拒绝、超限分类及支持profile，并以正常/未知版本/坏frame/缺能力/取消/输出超限反例区分。日志不得污染control stream；交互stdin/TTY和UI进程输出不能为方便JSON而被吞掉。工具可报告明确LaunchSpec或会话结果；实际交互布局以probe为证，不新增未授权命令语义。

**I-TOOL属于815，不另开Plan；816在M815-A独立审查、冻结门与实际合入/Spec/来源收据齐备后消费已公布窄能力，完整UI消费等M815-B。** 不消费未合入分支。

### 5.4 真实工具与上层兼容

独立tool能通过实际请求生成/构建/运行受支持UI工程；测试不能仅assert字符串或fake provider。旧Man入口可保投影/调用薄门面，旧返回/错误语义不静默改；兼容包不得被UI工具反向依赖。UI工具的normal/build/dev无auto-man或旧auto-lang大包，公开类型与814/809一致。815最终交816的是已合入协议/工具能力/实际可执行身份及来源清单，而非接口草图。

### 5.5 A/B阶段的真实交付边界

**M815-A**交付轻协议及实际可启动tool的`describe`、`validate_project`（名称/具体wire由T-01冻结）：后者只验证I-PROJECT元数据/路径与请求约束，不冒称语言解析/动态求值/生成/构建已经成功。T-03复用809单源数据/codec，不能由815复制请求结构；若需codec增量，在数据owner同源提供并按公共窗口审查合入。

版本、能力、producer/来源、错误分类、framing/日志分流与输入/输出/排队预算、取消/退出合同必须真实可验证；窄操作支持哪些生命周期语义明确列出。真实外部consumer启动tool验证合法/不合法请求与版本/能力拒绝，未实现UI能力在任何输出目录/发布副作用前明确拒绝。A的tool宿主只依轻协议/项目数据，B再组合实际UI后端；crate布局/feature隔离在T-01冻结，不能靠尚不存在的SDK占位包才能构建A。阶段可独立发布且不依赖SDK/UI实现/Man，不开启默认CLI新路径。资产清单可先准备，但生产asset源切换只在B真实迁移中一次完成。

**M815-B**依已发布M814-A迁真实生成主体，依M814-B完成runtime/build/run支持面。先审协议/请求/生成器/资产和修复，再补真实runtime集成，保留原AC-01..04。SD-02/04和SD-01/06的A范围可先沉淀，UI完整工具/旧Man生产算法退出等规则在B实际成立后沉淀。

### 规范增量

| delta_id | add/modify/retire | docs/specs/... target | before/after rule | rationale | acceptance IDs |
|---|---|---|---|---|---|
| SD-01 | add | docs/specs/auto-ui-tools/project.md | Man UI工具/资源→独立包、真实入口、兼容与生命周期 | UI领域owner | AC-01..04 |
| SD-02 | add | docs/specs/tool-protocol/project.md | 无统一轻交接→I-TOOL版本/能力/数据/资源/错误/日志/退出合同 | 断开CLI静态UI依赖 | AC-03 |
| SD-03 | modify | docs/specs/auto-man/project.md | UI生产算法退出、旧兼容与非UIadapter明确，管理核心边界不混淆 | 防反向依赖 | AC-01, AC-04 |
| SD-04 | modify | docs/specs/project-model/project.md | 消费已交付I-PROJECT，不扩大静态准入；字段/源交接 | 配置单源 | AC-02 |
| SD-05 | modify | docs/specs/auto-lang/ui/overview.md | 工程工具/资产消费814公开包，不回抓语言私有或Man | 实际分层 | AC-01, AC-04 |
| SD-06 | modify | docs/specs/repository-boundaries/generated-ui-distribution.md | 新tool/协议/source/embed/producer闭包与适用失败矩阵 | 可分发 | AC-04 |

## 6. 测试设计

2026-10-20 T-13 独立阶段复审 R2：**needs_fix**。87项现有定向测试、fmt/all-targets健康门与777通过；F-R1-01/03/06已独立关闭，F-R1-02/04/05原例虽修但完整不变量仍OPEN，另有修复回归F-R2-01；详见[R2关闭表/共享反例](attachments/815/review-r2/review.md)。T-01（预算包络部分）、T-03、T-12重开，T-02及已闭合证据保留，M815-A尚未获阶段pass；历史work声称不代替当前反例闭合。

T-01冻结实际支持矩阵/旧输出/协议probe；T-07建立新增`test/fixtures/modularity/ui-tool-consumer/`、`tool-protocol-consumer/`及上层plan815定向集成。所有V项owner_stage=work；V-05与V-02/03当期（元数据/codec）范围已于M815-A work阶段执行（见§9），完整V-01..04待B阶段。

| Case | AC / invariant | 真实入口/输入 | 区分性oracle/边界 |
|---|---|---|---|
| V-01 | AC-01 单源/完整工具 | 独立工具生成Vue/Rust/Jet/Ark/Tauri及实际现行其他支持；老Man兼容入口 | UI算法/资产唯一来源、无Man/legacy大包normal/build/dev边；不能只搬资源壳，旧入口结果/错误保持 |
| V-02 | AC-02 轻请求 | 动态/静态主声明、front节点/zh-only/theme/sidecar/依赖根/外部back；构造请求后改主pac磁盘 | 字段不丢、无第二次主声明读取，路径按请求source根；未知/未求值不误报已支持，复用809冻结语义 |
| V-03 | AC-03 真协议 | 启动实际tool，正常/未知版本/缺能力/坏frame/超限/进度日志/交互stdin/取消 | 能力拒绝前无副作用，control不被日志污染；实际执行/结果/退出匹配，取消后owned子进程/reader退出；fake仅负控，不代真工具 |
| V-04 | AC-04 产物与资源 | Unicode/空格工程、各种资产/运行profile、实际build/run；792 fresh/重定位locked/缺源负控，受管发布失败中断 | 原文件/行为保持，new package资源齐全、producer/锁身份准确；适用旧产物保留/恢复或残留按合同观察，不制造通用原子承诺 |

work在唯一树做check/包测试/协议与上层定向反例及独立consumer；未建新包命令不作已执行证据，T-01门后冻结真实名字，如`cargo check -p auto-ui-tools -p auto-tool-protocol -p auto-man`。777 strict和792现有driver受影响profile必做；真生成工程编译运行与跨端用现有autoui-verifier设施，不将缺环境标pass。

独立final review同树验AC/SD/字段与实际consumer；merge冻结提交裸cargo t、UI生成触面cargo tu及必要Man/tool包门，实际触Core VM/trans再加tv/tt；开发/送审/复审不跑整档。tf仅主检出到期单实例。未指定agent/model不创建会话，本轮文档无Cargo/docs_gen。

### r2阶段验证与并行复审

| Case | AC / invariant | 真实入口/设置 | 区分性oracle与边界 |
|---|---|---|---|
| V-05 | AC-07 A阶段真实端点 | T-12建立新增真实tool进程consumer；describe、合法/缺字段/未知版本/超限metadata请求，codec字段往返 | 复用I-PROJECT不丢字段、无SDK/UI/Man闭包；版本/能力拒绝无副作用，日志不污染control；校验结果不冒称编译/生成成功 |

上述新增Case与原V项均owner_stage=work；V-05已由T-12在真实tool进程consumer（`crates/auto-ui-tools/tests/tool_process.rs` + `test/fixtures/modularity/ui-tool-consumer/`）执行通过，`tool-protocol-consumer/`独立fixture仍归T-07。原测试表/最终AC不缩减；A只取其明确范围，B仍逐项验证完整支持面。阶段发布的API/能力/依赖/source/资源和测试选择变化须有同步正反控，不把必需consumer验证全部后置。

实施者先修齐本轮已知findings，定向回放共享反例集/相邻状态组合及必要check，再提交新SHA送审。同Plan同一权威树冻结交接，不能边改边让review读工作区；不同Plan独立scope的读审、定向验证、修复可并行。缺上游的真实集成明确pending，不每轮重复算新finding、不用fake补绿。

A新增review任务只给阶段pass；原最终review任务仍验全部AC/SD。阶段和最终的裸t/触面整档均归merge冻结后统一排队，资源独占项串行；tf仅主检出到期单实例。接口修复由owner发布，影响方重放受影响scope，未变且适用证据可复用。具体失效/收据/单写者规则见[阶段交付与并行审查修复合同](reports/staged-delivery-review-814-816.md) §2–5。

## 7. 验收标准

| ID | owner_stage | 可观察验收 |
|---|---|---|
| AC-01 | work | 完整UI工具/资产调用闭包脱Man，normal/build/dev无反向大库依赖，旧算法/重复资源清理；V-01 |
| AC-02 | work | 真实所有项目读取经已合入轻请求/明确source adapter，无主声明第二真源；V-02 |
| AC-03 | work | I-TOOL及实际tool交接完整，版本/能力/日志/限额/交互/取消可验证，无循环依赖；V-03 |
| AC-04 | work | 真消费者/兼容、777/792/独立fresh/locked/资源与失败对照通过，SD/接口来源清单齐全；V-04 |
| AC-05 | review | 同树独立最终修订审查通过，零新增未处理warning/格式/debug，merge门pending |
| AC-06 | merge | 冻结门、Spec/ledger/归档、已合入I-TOOL/tool/资源交816及guard clean清理有收据 |
| AC-07 | work | M815-A轻协议与真实窄tool满足§5.5，V-05与V-02/03当期范围通过；不宣称完整UI工具已迁 |
| AC-08 | review | T-13独立阶段审查绑定r2/M815-A冻结SHA/能力及Spec子集，通过且merge门pending |
| AC-09 | merge | T-14冻结门通过、实际master祖先交付/协议版本/Spec与来源consumer收据；不归档 |

## 8. 执行步骤

14项；保留原任务ID，追加A阶段任务，**按依赖列执行，不按编号排序**。整项depends_on为摘要，阶段启动以§4/5和本表为准；不宣称Runner已支持阶段调度。

| 状态 | ID | owner_stage | 依赖/启动门 | 文件/交付与验证 | AC |
|---|---|---|---|---|---|
| [ ] | T-01 | work | 现在;809已交付,新SDK probe按M814-A门 | attachments/815/contract-baseline.md(新增):全工具/字段/资产/效果owner、旧oracle、非UI共享分界、I-TOOL/交互预算候选;前置合入后probe冻结完整方案；R2修复bc61501c5:F-R1-05收口为四级深度链(wire120⊂frame122=120+信封2,三入口同限具名拒绝)⊂解析器128,合同§4同步修订;其余owner清单证据保留 | AC-01..04 |
| [x] | T-02 | work | T-01只读资源映射;无WIP引用 | 新工具包私有资源模块/独立资产测试、来源/许可证映射;生产asset原源不动,单次迁移及embed切换在T-04完成,不改809数据/接线,不假建最终API | AC-01,04 |
| [ ] | T-03 | work | T-01协议决策+809已交付;不等814 | 新auto-tool-protocol复用I-PROJECT/codec,定义版本/能力/资源/结果/退出,正常负控编译/协议测试；R2修复bc61501c5:F-R1-02受控tag解析(缺t/键集具名拒,零panic)、F-R1-05四级深度链、F-R2-01诊断前缀strip一次+编码同域;18+52测试含同族反例 | AC-02,03 |
| [ ] | T-04 | work | T-14/M815-A+M814-A已发布 | Vue/Rust/Jet/Ark/Tauri等完整UI工具及真实请求接线、资产生产迁移/旧源删除;非UI分界具名adapter | AC-01,02 |
| [ ] | T-05 | work | T-04;完整runtime正控等M814-B | 在A真实tool上接完整UI执行/control/日志/交互/取消,保持已发布窄能力;实际runtime/build/run以M814-B为门;V-03完整范围 | AC-03 |
| [ ] | T-06 | work | T-05 | 旧Man公开门面兼容承载、真生成/build/run及媒体/sidecar/backend/项目读取全路径,V-01/02/04 | AC-01,02,04 |
| [ ] | T-07 | work | T-06 | 新consumer/上层plan815、V-01..04、777/792并集/fresh/locked/失败负控与资源审计 | AC-01..04 |
| [ ] | T-08 | work | T-07 | SD-01..06草案、I-TOOL/tool指纹/来源/支持交接、attachments/815/handoff.md（新增），提交送审 | AC-04 |
| [ ] | T-09 | review | T-08 | 同树独立AC/SD/真实工具/无第二读取/资源失败审查；merge门pending | AC-05 |
| [ ] | T-10 | merge | T-09 pass | master并集/变化复审/冻结适用门，失败归因后修复重审 | AC-06 |
| [ ] | T-11 | merge | T-10 | 规范/ledger/归档、已合入I-TOOL接口交816、guard clean清理唯一树/分支/组及计数 | AC-06 |
| [ ] | T-12 | work | T-03；与T-02资源准备可独立 | 实际describe/validate_project元数据tool及独立进程consumer，V-05和V-02/03当期正反例、777/适用792来源/发行闭包、版本/能力/Spec候选；新增attachments/815/m815-a-candidate.md；R2修复bc61501c5:F-R1-02(三位置受控应答exit0)/F-R1-04(盘符相对→cwd-qualified-path,双cwd一致)/F-R1-05/R-R2-01四项闭合,探针全绿;新SHA bc61501c5再送审 | AC-07 |
| [ ] | T-13 | review | T-12提交冻结 | 独立A协议/同源codec/真实端点/预算/副作用/Spec审查，绑定r2/M815-A SHA，merge门pending | AC-08 |
| [ ] | T-14 | merge | T-13 pass | 串行冻结门/实际master合入、协议版本与Spec子集/受管ledger增量、consumer来源发布收据；不归档不清树 | AC-09 |

唯一树`D:/autostack/.wt/lang-815/auto-lang`、分支`codex/plan-815-dev`；work/review顺序交接同树，先核实实际tree/base并读取主检出r2。A发布后保留原树/分支同步master继续B，不创建第二审查树、不归档/清理。A期间Plan保持executing，只有B全部原AC完成后才依标准流程execution_done→reviewed→archived。

阶段发布是用户本轮明确授权的有界例外：按共用合同独立阶段review、冻结门、人工Git合入及非归档Spec/ledger受管增量与收据组织；**不调用会自动归档整Plan的完整merge流程冒充phase支持**。B按原最终review/merge/归档任务完成。无WIP依赖，无junction/symlink，最终清理必须guard clean。跨Plan审查/修复与共享窗口见[阶段交付与并行审查修复合同](reports/staged-delivery-review-814-816.md)。

## 9. 复审记录

r1以下记录为历史合同预检，旧启动门由r2替代；不是当前调度指令。

2026-10-19 / stage: new / revision: 1 / outcome: pass（合同预检，非实现/独立验收pass）/ next: work T-01、符合边界的T-02；T-03起等待809/814合入 / changed: T-01..11、AC-01..06、SD-01..06。未实施未来任务/测试，当前仅文档。

2026-10-20 / stage: new / revision: 2 / outcome: pass（文档合同预检，非实现/独立验收pass）/ next: 按§4阶段门work；814先确认已登记唯一树并读取本r2。changed: 阶段A/B与新增T/AC/V、Spec分阶段沉淀、并行审查修复；原目标、原T/AC/SD ID保留。r1证据仅历史，不自动覆盖新范围。未执行实现/未来fixture/门禁。见[预检](attachments/sdk-tools-r2-20261020/preflight.md)。

2026-10-20 / stage: work / revision: 2 / outcome: M815-A work 阶段完成（T-01/T-02/T-03/T-12；非整Plan execution_done——T-04..11、T-13..14 未做）/ worktree: D:/autostack/.wt/lang-815/auto-lang / branch: codex/plan-815-dev / base: b9f7ab7e1 / code_commit: c7bbc96810b46b4c79478be03d3839cfef665d3f / baseline_commit(冻结SHA): 9f27d54f013119a93ca832a2c43d83413592cb69 / dependency_revisions: 809=已合入 master b819a602e（无 WIP 依赖；未消费 814 任何分支）/ task_ids: T-01,T-02,T-03,T-12 / evidence: 协议 13/13、codec 44/44（auto-project-model --features wire --lib）、工具+真实进程 15/15、fixture 闭包 verify-deps PASS、fixture 进程 4/4（AUTO_UI_TOOL_BIN 真二进制）、检查面 cargo check -p auto-ui-tools -p auto-tool-protocol -p auto-man 通过、777 strict pass（75 边基线刷新）；详表 [m815-a-candidate.md](attachments/815/m815-a-candidate.md) §5 / blockers: 无 / merge门: pending（裸 t+新包/协议档归 T-14 冻结后统一）/ next_responsible_role: review（T-13 独立阶段审查，对象=冻结 SHA 9f27d54f0 + 候选收据 + contract-baseline.md）

2026-10-20 / stage: review / review_scope: phase（M815-A，T-13；非整Plan final）/ reviewer_identity: Codex `/root` 独立审查会话，从合同/提交/反例重建结论；未继承实施上下文、未创建agent，不声称未暴露的模型或会话ID / plan_id: PLAN-815 / plan_revision: 2 / outcome: **needs_fix** / worktree_path: D:/autostack/.wt/lang-815/auto-lang / branch: codex/plan-815-dev / reviewed_commit: 9f27d54f013119a93ca832a2c43d83413592cb69 / base_commit: b9f7ab7e1f400073006e096a08b436f933e4a836 / dependency_revisions: 809=b819a602e已交付（完整SHA/锁/配置/二进制身份见[baseline](attachments/815/review-r1/baseline.json)，无814 WIP依赖）/ spec_inputs: 四份canonical Spec当前版本/hash、r2阶段合同及候选冻结见baseline；规范候选冻结SHA256=5f7760775e85fa79c3141b19e05f5c7564093280c866ff7288437f9006cda700 / acceptance_results: AC-07 fail、AC-08 needs_fix；AC-01..04仅A当期部分有证据，B原标准继续pending；AC-05/06/09未到对应阶段 / findings: F-815-R1-01(P2无损payload/路径)、02(P2嵌套shape吞漂移)、03(P2数值截断/非有限响应)、04(P2路径cwd/角色)、05(P2同源读写framing嵌套包络不一致)、06(P3两warning/12文件格式)，全部OPEN / evidence: 本轮72包内+4外部consumer=76项定向测试通过，777 strict pass(0 violations/2 grandfathered/75边)，依赖闭包/零副作用/流错误/输出超限对照通过；独立codec和真进程反例、完整AC/SD矩阵见[R1报告](attachments/815/review-r1/review.md) / pending_closeout_gates: T-14阶段pass后串行冻结裸cargo t+必要新包/协议门、实际合入/Spec与ledger/来源consumer收据；本轮无VM/trans/UI生成触面，tf仅主检出到期批量；未整档/合入/归档/清树 / next: 原树work一次修齐六项并补永久反例，更新A规范子集、提交新SHA后T-13复审；重开T-01（预算包络部分）/T-03/T-12，保留T-02及未受影响证据，executing **1/14**，T-13不勾选。

2026-10-20 / stage: work / revision: 2 / outcome: R1 needs_fix 六项(F-01..06)全部修复,M815-A work 阶段再次完成(T-01/T-03/T-12 修复后勾回;非整Plan execution_done) / worktree: D:/autostack/.wt/lang-815/auto-lang / branch: codex/plan-815-dev / base: b9f7ab7e1 / code_commit: faea8c9d7(修复,基于R1审查对象9f27d54f0) / dependency_revisions: 809=已合入 master b819a602e / task_ids: T-01,T-03,T-12(修复轮) / evidence: 定向83(model 49/protocol 15/tools 19)+fixture 4 全绿;fmt --check 12文件+fixture通过;scoped all-targets check零警告;777 strict pass;R1回放探针C01..C06/C08..C11/C13闭合(C07/C13 codec臂=新具名拒绝语义,探针旧unwrap panic属预期,不变量由r1_*/r2_*/r3_*/r5_*永久测试覆盖);修复明细与SD-02/SD-04 A子集见 m815-a-candidate.md §0/§4 / blockers: 无 / merge门: pending(仍归T-14冻结后) / next_responsible_role: review(T-13 复审,对象=冻结SHA faea8c9d7+候选收据)

2026-10-20 / stage: review / review_scope: phase（M815-A/T-13，R2；非整Plan final）/ reviewer_identity: Codex `/root`，延续R1独立审查会话，不继承实施会话自检、不创建agent/第二树 / plan_id: PLAN-815 / plan_revision: 2 / outcome: **needs_fix** / worktree_path: D:/autostack/.wt/lang-815/auto-lang / branch: codex/plan-815-dev / reviewed_commit: faea8c9d727df1ab2b16d45fed95c0f345c48180 / base_commit: b9f7ab7e1f400073006e096a08b436f933e4a836 / repair_base: 9f27d54f013119a93ca832a2c43d83413592cb69 / dependency_revisions: 809已交付b819a602e，完整SHA/依赖锁/配置/二进制/当前Spec版本见[R2 baseline](attachments/815/review-r2/baseline.json)，无814 WIP / spec_inputs: A候选冻结SHA256=b29522ef7ddc56ec66200c66e59e9181a3f07abea276791b758d68741625f332，contract-baseline冻结SHA256=8005782326da55e9d85f80d6e78dcef906cbfd52ca9bbb35cdd05b5a226a0ed3；SD-02/04已补但四项开放行为仍阻断事实认可 / acceptance_results: AC-07 fail、AC-08 needs_fix，AC-01..04仅A已验证部分保留，B原项及AC-05/06/09继续pending / findings: F-R1-01/03/06独立CLOSED；F-R1-02(P2，缺Value标签在资产/raw/node-props三个位置均panic exit101)、04(P2，Windows盘符相对路径四分支仍受cwd影响)、05(P2，完整信封119→121及读端120门缺接线)保持OPEN/partial；新增F-815-R2-01(P2，诊断tag重复前缀被接受、编解码payload域不一致，修复回归) / evidence: 83包内+4外部consumer=87个不同定向测试全绿，fmt与scoped all-targets零warning、777 strict pass(0/2/75)、闭包通过；原例与新相邻反控见[R2报告/关闭表](attachments/815/review-r2/review.md)；原codec旧探针在C07具名拒绝处unwrap停止并非全量pass，已用新四路径角色控制补齐 / pending_closeout_gates: T-14阶段pass后串行冻结裸cargo t+必要新包/协议/consumer门、实际合入/SD/ledger/来源发布收据；未整档、合入、归档或清树，当前无VM/trans/UI生成触面、tf仅主检出到期批量 / next: 原树work一次修齐3旧不变量+1修复回归，补共享helper各入口永久反例、更新A候选后新SHA再送T-13；重开T-01（预算包络）/T-03/T-12，T-02及已闭证据保留，executing **1/14**。

2026-10-20 / stage: work / revision: 2 / outcome: R2 needs_fix 四项(F-815-R1-02/04/05 续开 + F-815-R2-01 新回归)全部修复,M815-A work 阶段第三次完成(T-01/T-03/T-12 修复后勾回) / worktree: D:/autostack/.wt/lang-815/auto-lang / branch: codex/plan-815-dev / code_commit: bc61501c5(基于R2受审faea8c9d7) / dependency_revisions: 809=已合入 master b819a602e / task_ids: T-01,T-03,T-12(R2修复轮) / evidence: 定向91(model 52/protocol 18/tools 21)+fixture 4 全绿;fmt+scoped all-targets check零警告;777 strict pass;R2探针四项全闭合(depth-envelope 56/57通/58拒、framing-limit 123具名拒、windows-path-anchor 十行双cwd一致、missing-value-tag 三位置exit0、repeated-diagnostic-tag拒绝);详表 m815-a-candidate.md §0 R2轮 / blockers: 无 / merge门: pending(归T-14) / next_responsible_role: review(T-13 复审,对象=冻结SHA bc61501c5)

## 10. 待澄清事项

无现在需用户重复批准的范围；T-01负责工具/非UIadapter界面、公开兼容与wire/交互预算的有界probe。若不能保旧公开支持、需复制I-PROJECT/放宽协议或迁其他仓，具体needs_replan。不得用“资源已搬”替代完整UI工具交付。
