---
plan_id: PLAN-815
status: executing
feature_name: UI 工程工具、生成入口与专属资源脱离 Man
author: [Codex]
created_at: 2026-10-19
updated_at: 2026-10-20
plan_revision: 3
current_step: 5
total_steps: 17
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

本次r3保留r2的M815-A真实阶段发布、M815-B完整交付组织，增加M815-L账本兼容恢复phase作为T-14c前置；不增加计划编号、不重做A代码、不提前归档。原r2按M815-A真实阶段发布、M815-B完整交付组织，不增加Plan编号、不降低原最终AC。执行与审查修复并行规则见[阶段交付与并行审查修复合同](reports/staged-delivery-review-814-816.md)；阶段pass不等于整Plan交付。

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

### r3阶段启动门

- 809/I-PROJECT已交付，T-01/T-02/T-03及T-12可现在实施；**协议与真实窄端点不依赖814**。
- **M815-A**：T-01/T-03→T-12→T-13→T-14；T-14a/b与canonical Spec已落地，T-14c等待M815-L的T-15→T-16→T-17完成；实际协议/tool发布后放行816客户端。T-02可与协议工作分scope推进，但不提前搬走旧生产embed源。
- **M815-B**：T-04生成主体等M814-A+M815-A；T-05中依赖新runtime的真实build/run集成等M814-B，随后T-06..11完整收口。生成主体的独立审查/修复无需等runtime完整发布。
- 不复制I-PROJECT，不消费814 WIP，不用假backend提前报告生成/运行支持。

### r3账本恢复授权与输入

用户2026-10-20提出“可以在本计划中加phase来恢复吗？”；本修订在815内增加M815-L，不另立编号。已授权的T-14收尾继续；本轮补成可审查合同，不把新增恢复实现/独立审查标为已完成。实现/审查/合入均沿用唯一树，auto-musk仅只读schema来源，不修改该仓或在线服务。

输入：`.autoos/specs.json` v57（1098 items）、原`.autoos/specs.json.bak-v57-815a` v56（1094 items），以及[store审计](attachments/815/t14-closeout/ledger-schema-audit.json.txt)和[原收尾核验](attachments/815/t14-closeout/recovery.md)；v56已有1060条旧item不兼容，v57新增4条也不完整，六section缺少store字段。当前恢复基线和schema完整SHA在审计附件，后续发现来源漂移必须重新冻结，不直接套用旧转换。canonical权威与ledger派生边界遵循`docs/specs/README.md`和`docs/design/autoplan-spec-ledger.md`。

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

### 5.6 M815-L：只恢复兼容投影，不重写知识

T-15先完成有界方案与真实loader可行性probe：逐字备份v57及可得历史备份、冻结hash/版本/schema、枚举缺字段/非法状态/重复ID/不支持类型，输出每一转换的ID、字段、原值、新值、依据及不可判定项。修复内容只限store结构、合法状态表示和本阶段四条来源增量；不改A/B行为、canonical Spec正文、历史业务结论、外仓代码或共享服务。

不得凭空补造历史完成时间或把active/delivered/缺状态等直接判为已验收。缺失时间、未知状态的表示及恢复方式由T-15提出明确候选、真实store往返probe与转换表，T-16独立核准后才落库；不能证明安全的记录须保留原始表示与完整恢复映射并阻断，不能删除/静默跳过。未知兼容字段保存在候选与逐字原始备份/映射中，必须说明store load/save对未知字段的行为，不能只测JSON能解析。

新增专用恢复工具`scripts/plan815_ledger_recover.py`（T-15建立；不是新通用知识系统），默认dry-run，在临时副本生成候选/逐项报告；只有T-16绑定候选与转换hash通过后，由T-17在正确workspace执行单写者恢复。保护先检查完整store schema、唯一ID、原始信息完整可恢复、canonical引用、全库计数及非目标字段不变；失败时真实原件逐字不变。备份不得覆盖，CAS/hash检测并发更新；写后真实store加载/读回核对失败要恢复精确原件并记录失败。原有已合规条目不得重新措辞或改状态。

恢复全库与815四条upsert分两个可核查checkpoint：先验证兼容恢复，再按R4四份canonical来源受管upsert固定P815-A-D1/T/R/REL，已有项更新而不重复追加；各次version递增、实际提交/hash记录清楚，不冒称跨文件/跨仓事务。重新生成发布收据，验证后关闭原T-14；不重复合入已是master祖先的代码。

规范影响：本phase只恢复现有派生ledger与审计资料，无新增产品/API/业务规则，**无新canonical Spec增量**；原SD-01..06及A已审子集保持，若probe需要改变规范/知识生命周期则停止并单列修订，不在修复中迁移知识体系。

## 6. 测试设计

2026-10-20 T-13 独立阶段复审 R4：**pass（仅M815-A阶段）**。F-815-R3-01已独立关闭，全部历史finding全闭、无新增；本轮14项wire测试与model all-targets/fmt通过，生产实现/依赖/配置/二进制与R3一致，按hash复用其余已闭验证证据。见[R4阶段报告](attachments/815/review-r4/review.md)与[精确已审A规范增量](attachments/815/review-r4/spec-delta-reviewed.md)。T-01/02/03/12/13完成，Plan保持executing5/14；T-14冻结门/实际合入/来源发布收据pending，未宣称整个815或M815-A已发布。

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

### r3恢复验证（M815-L，待T-15建立可执行证据）

所有L用例work owner=T-15；实现者自检只表示可送T-16，独立审查另上下文同树进行，未指定模型或新增agent。T-16未完成前禁止生产账本写入，T-17负责真实发布/回滚证据。

| Case | AC / invariant | 真实入口/设置 | 区分性oracle/覆盖边界 |
|---|---|---|---|
| V-L1 | AC-10 可加载且无历史丢失 | 专用恢复工具dry-run、v57真实副本、store `SpecsDocument` serde/`SpecsStore::load` | 六section与全部原ID仍在、原值可逐项恢复，合规项原字段不变；实际typed load成功，JSON parse不能代替 |
| V-L2 | AC-10 未知状态/时间不造假 | 合法状态正控、active/delivered/缺状态、缺时间/错类型/重复ID/未知字段对照 | 转换报告与审定映射一致，无依据即阻断；未知字段经load/save的保留边界显式核对，不能误报原有知识已完成 |
| V-L3 | AC-10/12 失败与并发安全 | 临时副本注入候选校验失败、读回失败、源hash漂移、备份同名不同字节、写失败 | 必须真实触发目标分支；旧文件hash不变或按合同恢复精确字节；并发别人的内容不得被回滚覆盖；原备份不覆盖 |
| V-L4 | AC-12 阶段投影幂等 | 恢复后四个固定ID首次upsert、重复执行、部分已有/内容冲突 | 每ID恰好一条、来源hash匹配canonical/审查/交付，非815条目保持；重复无新增、不虚增version；部分状态须明确对账 |

Category A：若只改专用脚本/临时fixture/ledger/文档，不另跑cargo t/docs_gen；使用实际loader与上述反例。若任何Rust生产代码变化则回到相应scope复审和merge门，不因恢复phase豁免。到期批量tf/tt/tb沿用独立主检出单实例收据，不冒充恢复验证。

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
| AC-08 | review | T-13的r2/M815-A R4 pass绑定原冻结SHA/能力及Spec子集，r3按未变来源继承该阶段证据；不覆盖新增恢复phase，其独立审查见AC-11 |
| AC-09 | merge | T-14冻结门通过、实际master祖先交付/协议版本/Spec与来源consumer收据；ledger经AC-12核验后才完整发布；不归档 |
| AC-10 | work | M815-L转换与真实typed loader验证通过，全部旧记录/原值可恢复，合规项不变、不补造历史结论；V-L1..3 |
| AC-11 | review | T-16独立恢复审查绑定r3/恢复提交/候选及转换hash，逐项核对V-L1..4与未知项，零未决；实现自检不代替 |
| AC-12 | merge | T-17按审定hash在正确workspace单写者恢复并完成固定四项受管upsert；备份/version/加载/读回/幂等/回滚收据齐备；再关闭T-14；V-L3..4 |

## 8. 执行步骤

17项；保留原任务ID，追加A阶段与M815-L恢复任务，**按依赖列执行，不按编号排序**。整项depends_on为摘要，阶段启动以§4/5和本表为准；不宣称Runner已支持阶段调度。

| 状态 | ID | owner_stage | 依赖/启动门 | 文件/交付与验证 | AC |
|---|---|---|---|---|---|
| [x] | T-01 | work | 现在;809已交付,新SDK probe按M814-A门 | attachments/815/contract-baseline.md(新增):全工具/字段/资产/效果owner、旧oracle、非UI共享分界、I-TOOL/交互预算候选;前置合入后probe冻结完整方案；R3独立确认bc61501c5的wire120/完整frame122/两读入口及解析器防线一致，F-R1-05 CLOSED；其余owner清单证据保留 | AC-01..04 |
| [x] | T-02 | work | T-01只读资源映射;无WIP引用 | 新工具包私有资源模块/独立资产测试、来源/许可证映射;生产asset原源不动,单次迁移及embed切换在T-04完成,不改809数据/接线,不假建最终API | AC-01,04 |
| [x] | T-03 | work | T-01协议决策+809已交付;不等814 | 新auto-tool-protocol复用I-PROJECT/codec,定义版本/能力/资源/结果/退出,正常负控编译/协议测试；R4确认7045e20fa的F-R3-01关闭：helper完整wire直接解码、双路径合法对照和目标分支/具名错误断言有效；14项wire复跑通过，R3其余未变证据复用 | AC-02,03 |
| [ ] | T-04 | work | T-14/M815-A+M814-A已发布 | Vue/Rust/Jet/Ark/Tauri等完整UI工具及真实请求接线、资产生产迁移/旧源删除;非UI分界具名adapter | AC-01,02 |
| [ ] | T-05 | work | T-04;完整runtime正控等M814-B | 在A真实tool上接完整UI执行/control/日志/交互/取消,保持已发布窄能力;实际runtime/build/run以M814-B为门;V-03完整范围 | AC-03 |
| [ ] | T-06 | work | T-05 | 旧Man公开门面兼容承载、真生成/build/run及媒体/sidecar/backend/项目读取全路径,V-01/02/04 | AC-01,02,04 |
| [ ] | T-07 | work | T-06 | 新consumer/上层plan815、V-01..04、777/792并集/fresh/locked/失败负控与资源审计 | AC-01..04 |
| [ ] | T-08 | work | T-07 | SD-01..06草案、I-TOOL/tool指纹/来源/支持交接、attachments/815/handoff.md（新增），提交送审 | AC-04 |
| [ ] | T-09 | review | T-08 | 同树独立AC/SD/真实工具/无第二读取/资源失败审查；merge门pending | AC-05 |
| [ ] | T-10 | merge | T-09 pass | master并集/变化复审/冻结适用门，失败归因后修复重审 | AC-06 |
| [ ] | T-11 | merge | T-10 | 规范/ledger/归档、已合入I-TOOL接口交816、guard clean清理唯一树/分支/组及计数 | AC-06 |
| [x] | T-12 | work | T-03；与T-02资源准备可独立 | 实际describe/validate_project元数据tool及独立进程consumer，V-05和V-02/03当期正反例、777/适用792来源/发行闭包、版本/能力/Spec候选；R4全部finding全闭，AC-07阶段work通过；A规范采用review-r4/spec-delta-reviewed.md，绑定7045e20fa，来源/真实进程等未变证据按hash复用 | AC-07 |
| [x] | T-13 | review | T-12提交冻结 | R4独立M815-A阶段pass，绑定r2/7045e20fa、精确A规范增量及验证复用清单；无开放finding，merge-owned冻结门/实际发布收据pending；不授予整Plan reviewed | AC-08 |
| [ ] | T-14 | merge | T-13 pass；剩余ledger/发布终验等T-17 | 串行冻结门/实际master合入、Spec子集已完成；受管ledger增量 blocked：v57四条815记录不完整，v56已有1060条历史记录及六section不兼容store schema。原件保持不变；脚本已补全量schema拒写与回滚守卫，四条完整草稿与恢复方案见 attachments/815/t14-closeout/recovery.md。发布条件尚未全部满足；不归档不清树 | AC-09 |
| [ ] | T-15 | work | r3合同；原A代码不变 | T-16/L2 needs_fix（1082bde52）：L1-01/04/06独立CLOSED；L1-02部分写失败残留257字节、L1-03前像/删除声明不全仍OPEN；L1-05代码双checkpoint已闭但§7收据残留；新增L2-01最终指纹不符仍exit0。真实CLI57→58→59/1098项往返正控通过，修齐后新SHA再送审，详 attachments/815/review-l2/review.md | AC-10 |
| [ ] | T-16 | review | T-15冻结提交/候选/映射 | 同树独立审查新增恢复phase（不重做未变A代码R4），逐项批准安全转换与加载/保留/失败反例，merge真实写入pending | AC-11 |
| [ ] | T-17 | merge | T-16 pass；单写者窗口 | 审定输入上恢复兼容账本，受管upsert四条815来源项并做真实加载/读回，记录版本/hash/备份/幂等与失败收据；交回T-14关闭发布；不归档不清树 | AC-12 |

唯一树`D:/autostack/.wt/lang-815/auto-lang`、分支`codex/plan-815-dev`；work/review顺序交接同树，先核实实际tree/base并读取主检出r3。A发布后保留原树/分支同步master继续B，不创建第二审查树、不归档/清理。A期间Plan保持executing，只有B全部原AC完成后才依标准流程execution_done→reviewed→archived。

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

2026-10-20 / stage: review / review_scope: phase（M815-A/T-13，R3；非整Plan final）/ reviewer_identity: Codex `/root`，延续R1/R2独立上下文，未继承实施自检、未创建agent/第二树 / plan_id: PLAN-815 / plan_revision: 2 / outcome: **needs_fix** / worktree_path: D:/autostack/.wt/lang-815/auto-lang / branch: codex/plan-815-dev / reviewed_commit: bc61501c58d45dde28ee2c739425dbc3c911b38c / base_commit: b9f7ab7e1f400073006e096a08b436f933e4a836 / repair_base: faea8c9d727df1ab2b16d45fed95c0f345c48180 / dependency_revisions: 809已发布，完整SHA/锁/配置/当前Spec版本与候选hash见[R3 baseline](attachments/815/review-r3/baseline.json)，无814 WIP / spec_inputs: 原候选和contract已冻结；[R3精确A规范校对稿](attachments/815/review-r3/spec-delta-reviewed.md)采用已冻结wire120/frame122及实际错误分类，hash见final-baseline；未发布canonical/ledger / acceptance_results: 上轮四项F-R1-02/04/05与F-R2-01全部独立CLOSED，早先F-R1-01/03/06保持CLOSED；AC-02/03当期实现行为通过，AC-07回归验证仍有唯一缺口、AC-08 needs_fix；B原项及merge项pending / findings: 唯一OPEN=F-815-R3-01(P2，新回归单测的helper返回完整wire却再包入request，所有负控提前在request/unknown wire键失败，合法Value也通过同一拒绝断言；非实现代码回归) / evidence: 91包内+4外部consumer=95个reported pass，fmt/all-targets零warning、777 strict pass/闭包通过；独立验证wire120/完整frame122精确边界、两读路径、7组路径双cwd、三位置坏payload受控应答后流继续、诊断双侧精确准入；详见[R3报告/当前关闭表](attachments/815/review-r3/review.md) / pending_closeout_gates: 阶段pass后T-14串行冻结裸cargo t+必要新包/协议/consumer门、实际合入/SD/ledger/发布来源收据；当前无VM/trans/UI生成触面，tf仅主检出到期批量；未整档/合入/归档/清树 / next: 原树仅修该单测输入构造/合法对照/目标分支断言并提交；实现/依赖/配置/输入不变时下轮按hash复用本轮代码关闭及规范证据，限定测试缺口复审；T-01预算确认完成、T-02保留，T-03/T-12未完成，executing **2/14**，T-13不勾选。

2026-10-20 / stage: work / revision: 2 / outcome: R3 单项(F-815-R3-01 测试缺陷:负控双重包装致断言空洞)修复完毕,M815-A work 阶段第四次完成(T-03/T-12 修复后勾回;实现与 bc61501c5 一致零改动) / worktree: D:/autostack/.wt/lang-815/auto-lang / branch: codex/plan-815-dev / code_commit: 7045e20fa(仅 wire_tests.rs) / dependency_revisions: 809=已合入 master b819a602e / task_ids: T-03,T-12(R3测试修复轮) / evidence: model 52(含修后 r2_value_object_without_tag_is_a_named_error:双路径合法对照必须解码+逐例 value 分支具名断言)/protocol 18/tools 21 定向全绿;fmt+scoped all-targets check 零警告;R3 报告确认四项代码 finding CLOSED、实现零改动可复用其验证证据 / blockers: 无 / merge门: pending(归T-14) / next_responsible_role: review(T-13 复审,对象=冻结SHA 7045e20fa;按 R3 路由可复用已闭代码证据、限定复审测试修复)

2026-10-20 / stage: review / review_scope: phase（M815-A/T-13，R4；非整Plan final）/ reviewer_identity: Codex `/root`，延续R1-R3独立上下文，未继承实施自检、未创建agent/第二树 / plan_id: PLAN-815 / plan_revision: 2 / outcome: **pass（阶段）** / worktree_path: D:/autostack/.wt/lang-815/auto-lang / branch: codex/plan-815-dev / reviewed_commit: 7045e20fa71ba437bc80075f17d9aace755f5abd / base_commit: b9f7ab7e1f400073006e096a08b436f933e4a836 / repair_base: bc61501c58d45dde28ee2c739425dbc3c911b38c / dependency_revisions: 809已发布，完整SHA/来源/锁/配置/Spec与候选hash见[R4 baseline](attachments/815/review-r4/baseline.json)，无814 WIP / spec_inputs: [精确已审A规范增量](attachments/815/review-r4/spec-delta-reviewed.md)继承R3规范正文、更新本轮绑定，替代候选§4旧概括，冻结hash见final-baseline；canonical/ledger未写 / acceptance_results: AC-07/08阶段pass；AC-01..04仅A当期范围通过，原完整B项与AC-05/06/09仍pending / findings: F-815-R3-01独立CLOSED，F-R1-01..06和F-R2-01保持CLOSED，无新增finding / evidence: 相对R3唯一变更wire_tests.rs，生产实现/其他测试/manifest/依赖锁/构建配置/canonical Spec输入及真实tool二进制身份完全一致；本轮14/14 wire测试(含合法/非法控制及目标错误分支)与model all-targets零warning/fmt通过，R3未变81项定向测试、真实进程/路径/深度/诊断/资源/闭包/777证据按hash复用，不冒称本轮全部重跑；详见[R4报告](attachments/815/review-r4/review.md) / pending_closeout_gates: T-14在冻结输入上串行裸cargo t+必要新包/协议/consumer门，实际master阶段合入/精确A规范与受管ledger/祖先与来源consumer发布收据；当前无VM/trans/UI生成触面，tf仅主检出到期批量；未整档/合入/归档/清树 / next: merge owner推进T-14，代码/依赖/配置/输入如变化则按影响重审；完成T-01/02/03/12/13，Plan保持executing **5/14**，不标整体reviewed、不提前放行下游按已发布A消费。

2026-10-20 / stage: merge(T-14, M815-A 阶段发布,非整Plan交付) / revision: 2 / outcome: M815-A 发布完成——冻结门全绿、master 合入、Spec/ledger 受管增量、发布收据齐备;Plan 保持 executing、唯一树/分支保留已同步 master 继续 B / worktree: D:/autostack/.wt/lang-815/auto-lang(分支 codex/plan-815-dev,已 ff 至合入位) / delivery_commit: e609caf1af511b96832f511ab9d1fa6c0c11f563(master 祖先验证通过) / reviewed: 7045e20fa(R4 pass) / gates: 裸 cargo t 5303/5303(worktree冻结位;ac_case/probe_adapter/plan499 前两轮失败=并发负载flake,隔离重跑全过,diff触面auto-lang/man/vm=0文件)+fixture闭包PASS+fixture进程4/4+777 strict pass;tv/tt/tu零触发 / deposit: SD-01-A/02-A/04-A/06-A canonical(按review-r4/spec-delta-reviewed.md);ledger v56→v57(scripts/plan815a_ledger_deposit.py,P815-A-D1/T/R/REL,备份bak-v57-815a,读回核对);spec-index重生成50projects;收据 attachments/815/m815-a-release.md(+json,按库惯例*.json不入库留盘) / 816放行: 本收据+R4 pass+冻结门齐备,M816-A 可消费协议版本1的 describe/validate_project 与 auto-ui-tool 二进制;不得静态链接工具实现 / next_responsible_role: work(M815-B:T-04 等 M814-A 发布;Plan 不归档不清树)

2026-10-20 / stage: work(T-15) / revision: 3 / outcome: 恢复方案/候选数据/反例验证完成,送 T-16 独立恢复审查;生产账本零接触(live v57=c24b174e… 全程逐字核验) / worktree: D:/autostack/.wt/lang-815/auto-lang(feacf79e8:plan815_ledger_recover.py+deposit脚本PascalCase修正) / 关键发现: SectionType 线格式=PascalCase——真实 SpecsStore::load 把 snake section_type 经 serde(other) 静默滤成空文档(load"成功"0 sections),python 级 JSON 校验不可见;工具与 deposit 校验均已修正 / 候选(hash绑定,待T-16): source=c24b174e…/candidate=d60bd3fa…/conversions=c35615e…(12140条,含四条M815-A增量按收尾冻结稿)/blocked=0(--ruling reviews/P014-1=P014-1-R 为T-15建议裁定,T-16 可改判后重生成) / 证据: attachments/815/t15-recovery/(report.md/freeze逐字备份/audit/convert-final/probe源码);V-L1负控live拒/正控候选6区1098项、往返零差异、未知字段真实丢弃实证、V-L3自检含真实探针失败回滚;T-17全链彩排(apply+loader+四条幂等)临时副本通过 / blockers: 无(生产写入仅待T-16核准hash) / next_responsible_role: review(T-16 绑定 r3/feacf79e8/候选与转换hash,逐项核对V-L1..4与§7裁定清单)

2026-10-20 / stage: review / review_scope: phase（M815-L/T-16，L1）/ reviewer_identity: Codex `/root`；同会话参与过前期合同/旧守卫候选，本轮从新恢复实现/真实loader/独立反例重建，不声称另会话final reviewer，未创建agent / plan_id: PLAN-815 / plan_revision: 3 / outcome: **needs_fix** / worktree_path: D:/autostack/.wt/lang-815/auto-lang / branch: codex/plan-815-dev / reviewed_commit: feacf79e8a9c52575a6f64dae3e068a3cc3ae1c7 / base_commit: abad97371f5443ea25b007fa34074b882062163f / dependency_revisions: store schema c124c844…、真实loader 791c204c…、live v57 c24b174e…、stage输入与脚本完整SHA见[baseline](attachments/815/review-l1/baseline.json.txt) / spec_inputs: M815-L无新canonical业务增量，原r3元数据与A/R4精确子集保持，[增量核验](attachments/815/review-l1/spec-delta-reviewed.md) / acceptance_results: AC-10 fail、AC-11 needs_fix、AC-12 merge pending；V-L1/2 partial、L3 fail、L4 partial/pending；原未变A代码R4 pass保持，B原任务pending / findings: F-815-L1-01(P1 CLI拒绝/失败exit0)、02(P1并发写入与回滚覆盖外部内容)、03(P2转换表8处前像失真/ID改名未记/未声明删除漏检)、04(P2 schema负u64/非字符串列表错放及typed验证可跳过)、05(P2恢复+28处阶段变更合并且version仍57违反r3双checkpoint)、06(P2恢复命令source SHA拼错)全部OPEN / evidence: 候选d60bd3fa…/转换规范化c35615e2…可重现，真实store6区1098项load/save内容相等，两工具自检通过；独立真实CLI与临时副本注入反例已复现，详[L1报告与反例](attachments/815/review-l1/review.md) / pending_closeout_gates: 新候选完整恢复审查通过后T-17单写者恢复/受管upsert/版本与真实读回收据，T-14关闭；旧batch tt归因/tb另待主检出独占窗口，本轮未整档 / next: 原树work一次修齐六项补永久真实CLI/并发/前像/schema/版本对照，新提交及候选hash后再T-16；重开T-15，T-16/17/T-14不勾，executing **5/17**，生产ledger/canonical未改，不合入不归档不清树。

2026-10-20 / stage: work(T-15,L1 修复轮) / revision: 3 / outcome: review-l1 六项(F-L1-01/02/03/04/05/06)全部修复,M815-L 恢复方案/候选/反例验证再次完成,送 T-16 复审;生产账本零接触(live v57=c24b174e… 逐字未动) / worktree: 1082bde52(plan815_ledger_recover.py 重写:锁内CAS写通道+受护栏回滚/退出码传播/双checkpoint/前像忠实/完备镜像+强制身份探针;deposit 脚本回滚加护栏) / 候选(hash 由 summary 程序化绑定): source=c24b174e…→A d571dbe3…(v58,结构恢复+P014-1-R 裁定)→B 33830b65…(v59,四条增量);conversions=0a1e1cbb…(12163 行,含 checkpoint 标记) / 证据: 自检真实 CLI 全分支通过;真 .exe 探针全链彩排 57→58→59/1098 项/四增量唯一/改名在场;探针自身修为 serde 线格式(Pascal)——身份核验并抓出其 as_str 缺陷;review-l1 探针绑定旧单候选接口,反例由自检等效覆盖、下轮按新接口重绑(报告 §0 注明) / blockers: 无 / next_responsible_role: review(T-16 复审,绑定 r3/1082bde52/新候选与转换 hash)

2026-10-20 / stage: review / review_scope: phase（M815-L/T-16，L2）/ reviewer_identity: Codex `/root`，同会话参与前期合同/旧守卫，按新修复/真实loader/独立反例重建，不声称另上下文/model，未创建agent / plan_id: PLAN-815 / plan_revision: 3 / outcome: **needs_fix** / worktree_path: D:/autostack/.wt/lang-815/auto-lang / branch: codex/plan-815-dev / reviewed_commit: 1082bde52e41d602409ead78a9753277feb6b3f1 / repair_base: feacf79e8a9c52575a6f64dae3e068a3cc3ae1c7 / dependency_revisions: live v57 c24b174e…/schema c124c844…、新loader/实现/候选及输入完整SHA见[L2 baseline](attachments/815/review-l2/baseline.json.txt) / spec_inputs: M815-L无新canonical业务规则，原r3元数据和A/R4已审子集保持；[增量核验](attachments/815/review-l2/spec-delta-reviewed.md) / acceptance_results: AC-10 fail、AC-11 needs_fix、AC-12 merge pending；A/B候选与转换hash重现、真实CLI57→58→59/6区1098项/往返通过 / findings: L1-01/04/06 CLOSED，L1-02(P1)旧外部更新控制已闭但本次部分写失败留下257字节且被错认外部更新、L1-03(P2)旧related/rename/title控制已闭但213条缺status记null/24个legacy删除未声明/section变异漏检保持OPEN；L1-05代码双checkpoint CLOSED而当前收据§7旧不递增规则残留P3；新增F-815-L2-01(P2相邻路径)最终真实probe后外部更新导致指纹不符仍applied=true/exit0 / evidence: [L2报告与两轮未闭诊断](attachments/815/review-l2/review.md)，含真实锁/fd部分写失败、真实CLI最终更新及旧例重放；两工具self-test与scoped diff-check通过 / pending_closeout_gates: 新恢复代码/转换复审通过后T-17真实恢复/upsert/备份/版本/读回收据，T-14关闭；旧batch未完成单独待独占窗口，本轮无整档 / next: 原唯一树work按两轮诊断一次修齐剩余不变量与收据，补永久相邻失败控制，新SHA与hash后再T-16；T-15重开，T-16/17/T-14不勾，executing **5/17**，生产ledger/canonical未改、不合入不归档不清树。

## 10. 待澄清事项

无现在需用户重复批准的范围；T-01负责工具/非UIadapter界面、公开兼容与wire/交互预算的有界probe。若不能保旧公开支持、需复制I-PROJECT/放宽协议或迁其他仓，具体needs_replan。不得用“资源已搬”替代完整UI工具交付。

2026-10-20 / stage: merge-reconcile(T-14) / outcome: **blocked（ledger_refreshed）** / correction_of: 上述T-14完成记录仅验证JSON/ID，未验证store schema，不能作为受管发布完成证明 / verified: reviewed7045e20fa与delivery e609caf1a均为master祖先；生产实现与R4一致，四份canonical Spec已落地，原裸t/fixture/777执行证据保留 / blocker: v56已有1060条旧item与六section不满足store schema，v57新增四条815也不完整；无可直接恢复的合规v56备份 / action: 原树修复plan815a_ledger_deposit.py（全量前检、工厂字段、SHA、ID、幂等、备份/回滚），临时反例全过；真实账本拒写且SHA未变；完整目标条目仅为草稿 / receipt: [收尾核验与恢复方案](attachments/815/t14-closeout/recovery.md)及schema审计、完整输入manifest / next: 待授权另立全库恢复计划，恢复后再执行815受管增量与发布终验；阶段R4 pass保留，T-14重开、executing **5/17**（r3新增三项恢复任务），不归档不清树、不把代码合入等同于完整发布。到期主检出批量回归另行记实际结果。

2026-10-20 / stage: new / plan_revision: 3 / outcome: pass（恢复phase合同预检，非恢复实现/审查pass）/ authorization: 用户要求在815内加phase恢复，沿用原唯一树、不新增计划编号 / changed: M815-L、T-15..17、AC-10..12、V-L1..4；T-14剩余部分依赖T-17，current_step 5/17 / evidence: 原v56与v57全库schema审计，原A代码/依赖/二进制零漂移及master祖先证明 / inherited: r2/T-13 R4仅A代码及精确Spec子集pass保持；新增恢复schema/脚本/转换证据不继承该pass / spec_delta: 无新增canonical业务规则，原A/B SD保持 / next: work T-15有界probe/转换决策与dry-run，提交后独立T-16，再由T-17恢复及T-14终验；本轮尚未修改生产ledger。不得把历史“发布完成”当作当前调度。

2026-10-20 / stage: work-preparation / phase: M815-L / partial_evidence: 原树写入守卫候选 `2dc142df0`（全量schema前检、工厂字段、ID/来源、幂等与回滚反例通过），尚未合入master；不是T-15完整完成，也不是T-16独立审查pass / next: T-15补真实typed loader probe、全库转换决策/候选与V-L1..4后送审；生产ledger仍为原v57且SHA未变。

2026-10-20 / stage: regress / outcome: incomplete（独占窗口不足，未推进.last-batch-regression.json）/ covered: 4d7c5f581 / tf: 完整6270/6270通过 / tt: 5671/5675通过，3fail+1timeout，隔离归因pending；不自动统称负载flake / tb: 中断未完成，后续预检见其他cargo t故拒绝启动 / evidence: [批量进度](attachments/815/t14-closeout/batch-status.md)与原始日志 / next: 独占主检出完成tt归因/重跑+tb后才写成功批量收据；与M815-L的T-15..17分开验收。
