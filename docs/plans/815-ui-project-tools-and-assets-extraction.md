---
plan_id: PLAN-815
status: drafting
feature_name: UI 工程工具、生成入口与专属资源脱离 Man
author: [Codex]
created_at: 2026-10-19
updated_at: 2026-10-19
plan_revision: 1
current_step: 0
total_steps: 11
supersedes_spec_components: [docs/specs/auto-man/project.md, docs/specs/auto-lang/ui/overview.md, docs/specs/project-model/project.md, docs/specs/repository-boundaries/generated-ui-distribution.md]
new_spec_components: [docs/specs/auto-ui-tools/project.md, docs/specs/tool-protocol/project.md]
touched_goals: [GOAL-003, GOAL-007]
affects: [auto-ui-tools, auto-tool-protocol, auto-man/compat, ui-assets]
depends_on: [PLAN-809, PLAN-814]
---

# [PLAN-815] UI 工程工具、生成入口与专属资源脱离 Man

## 0. 变更摘要

完成R04工程工具半部：Man内Vue/Rust/Jet/Ark/Tauri等实际UI生成/脚手架、gallery/wm/shadcn等专属资产及其调用闭包迁到独立UI工具包，消费已交付I-PROJECT和814公开SDK/UI接口，不反依赖Man管理大库。同步交付轻I-TOOL执行协议与实际UI工具服务入口，供816消费，避免双向等待。包含真生产、旧门面兼容、资源/发行及失败证据。

不搬通用锁/resolver/Builder/模板算法，不把整个api_gen或http_service按文件名盲搬，不迁Git仓。I-TOOL不是另一个小Plan，也不消费809未合入请求类型。

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

用户授权下一批较大模块规划，当前只在auto-lang仓写计划/设计，不实施、不创建agent/树、不写远端。来源：[路线](reports/v06-extraction-execution-roadmap.md)、[批次](reports/sdk-tools-parallel-order-814-816.md)、[输入hash](attachments/sdk-tools-batch-20261019/inputs.json)。

权威：[Man](../specs/auto-man/project.md)、[UI](../specs/auto-lang/ui/overview.md)、[project-model](../specs/project-model/project.md)、[交付](../specs/repository-boundaries/generated-ui-distribution.md)。809 I-PROJECT当前仍review修复，handoff列主声明再读、动态/前端字段、sidecar/依赖根等跨文件差额；以其最终合入Spec为准，不冻结其WIP。814为本批新合同。

实证：`auto-man/src/{vue,rust_ui,jet,ark,tauri,vue_shadcn,gallery_assets,wm_assets,gallery_cache,image_viewer,api_gen,tauri_backend,vscode,sidecar,http_service}.rs`及`assets/{gallery,wm,shadcn-ui}`；Vue/Rust等当前使用Pac/语言生成API，api_gen同时有非UI服务/媒体适配，http_service承载独立server命令；需逐符号而非整文件武断归属。`rust_ui.rs`还持792发行管线与producer/source表；原资产使用编译期embed和邻仓输入，必须迁完整来源。

### 开工门

- T-01可立即只读完成工具/数据/资源/效果映射，研究历史输出与I-TOOL合同。
- T-02只做独立资产/纯清单的代码可现在在唯一树推进：建立私有资源模块/来源映射与定向测试，不提前搬走仍被旧生产embed使用的文件；中间阶段可只读映射原源，但不称其已可独立发布。不得改809生成器接线、不依赖未合入I-PROJECT/814类型、不创建最终API假实现；本步骤不能单独标整个工具模块完成或阶段合入。
- **T-03起协议类型/工具生产接线等809和814整体合入**。T-01任何涉及新接口的编译probe同样等门；不cherry-pick审查分支，不用临时镜像DTO绕等待。

## 5. 详细设计

### 5.1 完整owner图与I-PROJECT

UI工程遍历/模板/脚手架/生成/资源发布依实际调用图迁入；主项目声明读取经809请求，mode/output_root/input_sources/dependency roots、sidecar/front/theme/backend/locale/env覆盖等按已合入规则保持。必要跨文件输入由请求/显式source adapter提供，不暗中重新求值主pac.at。动态配置留显式语言adapter，不把静态scanner当全部配置。

仅UI服务壳/媒体生成桥归UI工具；通用API提取、语言trans/server配置、非UI模板、compiler/PATH/Git/FS/Builder动作留语言/管理adapter或已有共享包，边界由T-01逐符号清单固定。若拆文件，单份算法/类型，不保两套server generator；shared helper只为真实消费者抽出且不建巨型common。

### 5.2 资源与生成产物

gallery/wm/shadcn、UI专用模板/样式/i18n/图标等源码与embed/include/生成schema、drawlist等外部构建输入有owner/source/许可证/定位清单；803既有通用templates/GHS/IAR资源不重新迁。默认/未知族、覆盖顺序/文件字节、主题/sidecar/外部backend/项目路径相对源根行为冻结。T-02纯资产阶段不动生产路径，完整接线T-04后删除原重复来源。

792受管发布/producer/lock/修剪/failure facility复用并扩至新工具，不能靠扩大allowlist漏资源或依赖父仓；非受管旧模板的既有破坏性失败窗口按已确认语义记录，不突然承诺所有生成文件原子事务。新协议的能力/版本拒绝须发生在副作用之前；真正发布中断的可恢复/保留行为按适用profile精确验证。

### 5.3 I-TOOL接口交接（本Plan整体交付）

覆盖协议版本/operation、能力与profile、请求身份/来源、I-PROJECT轻payload、producer版本/指纹、资源根/输出根、产物与诊断/错误、进度/日志/取消/退出及结果与运行进程寿命。请求只承载具名数据，不让AST/VMValue/裸指针/任意callback跨工具边界。若需要wire序列化，优先在数据owner提供同源codec，不复制请求结构再双向丢字段。

T-01冻结适用平台、实际codec/framing、消息/输出/排队预算与单位、版本/未知能力拒绝、超限分类及支持profile，并以正常/未知版本/坏frame/缺能力/取消/输出超限反例区分。日志不得污染control stream；交互stdin/TTY和UI进程输出不能为方便JSON而被吞掉。工具可报告明确LaunchSpec或会话结果；实际交互布局以probe为证，不新增未授权命令语义。

**I-TOOL属于815，不另开Plan，816只在815整体review+merge后消费。** 本合同无phase merge；816可先完成与协议无关的执行设施工作。这样815不等816接口，816不依赖815 WIP，形成有向交接。

### 5.4 真实工具与上层兼容

独立tool能通过实际请求生成/构建/运行受支持UI工程；测试不能仅assert字符串或fake provider。旧Man入口可保投影/调用薄门面，旧返回/错误语义不静默改；兼容包不得被UI工具反向依赖。UI工具的normal/build/dev无auto-man或旧auto-lang大包，公开类型与814/809一致。815最终交816的是已合入协议/工具能力/实际可执行身份及来源清单，而非接口草图。

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

T-01冻结实际支持矩阵/旧输出/协议probe；T-07建立新增`test/fixtures/modularity/ui-tool-consumer/`、`tool-protocol-consumer/`及上层plan815定向集成。所有V项owner_stage=work、尚未运行。

| Case | AC / invariant | 真实入口/输入 | 区分性oracle/边界 |
|---|---|---|---|
| V-01 | AC-01 单源/完整工具 | 独立工具生成Vue/Rust/Jet/Ark/Tauri及实际现行其他支持；老Man兼容入口 | UI算法/资产唯一来源、无Man/legacy大包normal/build/dev边；不能只搬资源壳，旧入口结果/错误保持 |
| V-02 | AC-02 轻请求 | 动态/静态主声明、front节点/zh-only/theme/sidecar/依赖根/外部back；构造请求后改主pac磁盘 | 字段不丢、无第二次主声明读取，路径按请求source根；未知/未求值不误报已支持，复用809冻结语义 |
| V-03 | AC-03 真协议 | 启动实际tool，正常/未知版本/缺能力/坏frame/超限/进度日志/交互stdin/取消 | 能力拒绝前无副作用，control不被日志污染；实际执行/结果/退出匹配，取消后owned子进程/reader退出；fake仅负控，不代真工具 |
| V-04 | AC-04 产物与资源 | Unicode/空格工程、各种资产/运行profile、实际build/run；792 fresh/重定位locked/缺源负控，受管发布失败中断 | 原文件/行为保持，new package资源齐全、producer/锁身份准确；适用旧产物保留/恢复或残留按合同观察，不制造通用原子承诺 |

work在唯一树做check/包测试/协议与上层定向反例及独立consumer；未建新包命令不作已执行证据，T-01门后冻结真实名字，如`cargo check -p auto-ui-tools -p auto-tool-protocol -p auto-man`。777 strict和792现有driver受影响profile必做；真生成工程编译运行与跨端用现有autoui-verifier设施，不将缺环境标pass。

独立final review同树验AC/SD/字段与实际consumer；merge冻结提交裸cargo t、UI生成触面cargo tu及必要Man/tool包门，实际触Core VM/trans再加tv/tt；开发/送审/复审不跑整档。tf仅主检出到期单实例。未指定agent/model不创建会话，本轮文档无Cargo/docs_gen。

## 7. 验收标准

| ID | owner_stage | 可观察验收 |
|---|---|---|
| AC-01 | work | 完整UI工具/资产调用闭包脱Man，normal/build/dev无反向大库依赖，旧算法/重复资源清理；V-01 |
| AC-02 | work | 真实所有项目读取经已合入轻请求/明确source adapter，无主声明第二真源；V-02 |
| AC-03 | work | I-TOOL及实际tool交接完整，版本/能力/日志/限额/交互/取消可验证，无循环依赖；V-03 |
| AC-04 | work | 真消费者/兼容、777/792/独立fresh/locked/资源与失败对照通过，SD/接口来源清单齐全；V-04 |
| AC-05 | review | 同树独立最终修订审查通过，零新增未处理warning/格式/debug，merge门pending |
| AC-06 | merge | 冻结门、Spec/ledger/归档、已合入I-TOOL/tool/资源交816及guard clean清理有收据 |

## 8. 执行步骤

11项；T-01现在只读，T-02纯资产可先实施，T-03起等809+814合入；无phase merge。

| 状态 | ID | owner_stage | 依赖/启动门 | 文件/交付与验证 | AC |
|---|---|---|---|---|---|
| [ ] | T-01 | work | 现在只读；新接口probe等门 | attachments/815/contract-baseline.md（新增）：全工具/字段/资产/效果owner、旧oracle、非UI共享分界、I-TOOL/交互预算候选；前置合入后probe冻结完整方案 | AC-01..04 |
| [ ] | T-02 | work | T-01只读资源映射；无WIP引用 | 新工具包私有资源模块/独立资产测试、来源/许可证映射；生产asset原源不动，单次迁移及embed切换在T-04完成，不改809数据/接线，不假建最终API | AC-01,04 |
| [ ] | T-03 | work | T-01方案+809/814已合入 | 新auto-tool-protocol复用I-PROJECT/codec，定义版本/能力/资源/结果/退出，正常负控编译/协议测试 | AC-02,03 |
| [ ] | T-04 | work | T-03 | Vue/Rust/Jet/Ark/Tauri等完整UI工具及真实请求接线、资产生产迁移/旧源删除；非UI分界具名adapter | AC-01,02 |
| [ ] | T-05 | work | T-04 | 真auto-ui-tool执行入口/control/日志/交互/取消按协议打通，复用现有执行facility；V-03 | AC-03 |
| [ ] | T-06 | work | T-05 | 旧Man公开门面兼容承载、真生成/build/run及媒体/sidecar/backend/项目读取全路径，V-01/02/04 | AC-01,02,04 |
| [ ] | T-07 | work | T-06 | 新consumer/上层plan815、V-01..04、777/792并集/fresh/locked/失败负控与资源审计 | AC-01..04 |
| [ ] | T-08 | work | T-07 | SD-01..06草案、I-TOOL/tool指纹/来源/支持交接、attachments/815/handoff.md（新增），提交送审 | AC-04 |
| [ ] | T-09 | review | T-08 | 同树独立AC/SD/真实工具/无第二读取/资源失败审查；merge门pending | AC-05 |
| [ ] | T-10 | merge | T-09 pass | master并集/变化复审/冻结适用门，失败归因后修复重审 | AC-06 |
| [ ] | T-11 | merge | T-10 | 规范/ledger/归档、已合入I-TOOL接口交816、guard clean清理唯一树/分支/组及计数 | AC-06 |

树`D:/autostack/.wt/lang-815/auto-lang`、分支`codex/plan-815-dev`；同树work/review/merge，work记录actual base/兄弟依赖。不得借809/814 WIP提前生产接线，分支不叠加其送审代码；树内禁junction/symlink，遵守guard。

## 9. 复审记录

2026-10-19 / stage: new / revision: 1 / outcome: pass（合同预检，非实现/独立验收pass）/ next: work T-01、符合边界的T-02；T-03起等待809/814合入 / changed: T-01..11、AC-01..06、SD-01..06。未实施未来任务/测试，当前仅文档。

## 10. 待澄清事项

无现在需用户重复批准的范围；T-01负责工具/非UIadapter界面、公开兼容与wire/交互预算的有界probe。若不能保旧公开支持、需复制I-PROJECT/放宽协议或迁其他仓，具体needs_replan。不得用“资源已搬”替代完整UI工具交付。
