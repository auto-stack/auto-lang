---
plan_id: PLAN-734
status: archived
feature_name: api-contract-and-generation-integrity
author: [agent]
created_at: 2026-10-03
updated_at: 2026-10-03
plan_revision: 1
current_step: 8
total_steps: 8
supersedes_spec_components:
  - docs/specs/stdlib/project.md
  - docs/specs/stdlib/design/http-server.md
  - docs/specs/stdlib/design/backend-assembly.md
  - docs/specs/auto-man/project.md
  - docs/specs/auto-lang/trans/overview.md
  - docs/specs/auto-lang/vm/back-proxy.md
new_spec_components:
  - docs/specs/stdlib/design/api-transport-contract.md
  - docs/specs/auto-man/design/api-generation-integrity.md
touched_goals: [GOAL-003]
affects: [crates/auto-lang/src/api, crates/auto-lang/src/vm/codegen.rs, crates/auto-lang/src/vm/ffi/http_server.rs, crates/auto-lang/src/vm/ffi/stdlib.rs, crates/auto-lang/src/back_proxy.rs, crates/auto-lang/src/trans/rust.rs, crates/auto-man/src/api_gen.rs, crates/auto-man/src/vue.rs, crates/auto-man/src/rust_ui.rs, crates/auto-man/src/tauri.rs, crates/auto-man/src/tauri_backend.rs]
---

# [PLAN-734] API 契约与生成代码一致性：参数校验、真实执行与失败诊断

## 0. 变更摘要

接续 [Design 33](../design/33-stdlib-runtime-and-http.md) 阶段 D2a。727 客户端文件传输、729 服务端下载已交付，730 服务端上传仍在规划/执行队列。本计划提前规划**普通 JSON API 的共同契约和真实实现来源**：VM HTTP、生成 Rust HTTP、back-proxy、VM merged 与 Tauri IPC 在受支持数据面上对拍；不把 HTTP status/header 强行包装到进程内函数调用。

关键修复是消除“接口生成/业务体转译失败却退回模板或启动旧产物”，建立有源位置、实现来源、参数/响应种类和后台能力的共同记录。参数缺失/坏值不变成0或空串；普通 int 不被资源id误判；业务对象有error字段不因此变成500；文件/上传/流保持专属协议。

本期不是完整生产服务器认证，不接入 TLS/HTTP2/3/通用 WebSocket，不移植 back-proxy 传输到 Axum，不实现任意外部 SSE handler 生成，不改变 CPU 调度。也不提供完整 OpenAPI/全 stdlib 装配 manifest；这里的 API 契约记录只服务 API 消费/诊断，其他符号覆盖另案。

## 1. 目标

- G1：从 api.at 的解析/类型及实际实现得到版本化 `ApiContract`，各适配器消费同一参数、返回种类、实现来源和能力分类，禁止不同消费者各用contains字符串猜类型。
- G2：普通JSON请求绑定遵循现有“path > body > query”与精确类型校验；HTTP错误一致、IPC/merged在其原生调用边界拒绝坏值/不可用能力，合法数据语义一致。
- G3：有真实业务体/已解析委派时执行该实现；无法解析/转译/定位实现则生成失败并中止相关build/run，不能换成CRUD模板、空桩或旧生成版本。
- G4：明确 JSON/显式HTTP Response/SSE/FileResponse/Upload 的边界；async终值也按声明类型检查，不把资源句柄、错误字段或序列化失败当成功JSON。
- G5：生成产物与当前契约及实现指纹绑定，失败不发布可启动的半成品；前端生成错误也不能只warn后继续，手写TS提供者保持显式来源。
- G6：真实VM/Rust/HTTP客户端、merged/back-proxy和Tauri command dispatcher验证受支持子集；不以golden或直接Rust函数调用冒充传输链闭合。

只修改本仓；不改 auto-musk/AutoUI 应用业务来掩盖生成缺陷。对已工作的类型与成功返回保持兼容；需要调整的错误处理与旧静默fallback行为在本合同/SD中显式提出，不批量删测/缩验收。

## 2. 架构方案

```text
api.at + resolved imports/impl → ApiContract（类型身份、源位置、实现来源、指纹）
  ├─ HTTP 参数绑定计划 + JSON/显式响应分类 → VM/Rust/back-proxy adapters
  ├─ 函数参数/数据分类 + 能力门 → merged / Tauri IPC
  └─ TS 调用绑定 + response kind → HTTP/IPC client
       生成期间任一错误 → Err诊断 → 停止该API build/run，不替代实现
       全部成功 → immutable generation bundle + ready记录 → 启动校验
```

拟新增 `crates/auto-lang/src/api/contract.rs`（normalized contract/validator/返回和传输能力分类）与 `api/diagnostic.rs`（结构化生成诊断），由 `api/mod.rs` 导出给 auto-man。纯契约/校验不能依赖 Axum、Tokio、VM或Tauri；宿主资源依旧由其专属执行模块承担。VM codegen/session发布从本session构建的契约，不新增跨session覆盖式全局表。

执行来源采用 `CompiledBody / ResolvedDelegate / ExplicitProvider / Scaffold` 等明确标签，标签不能代替实现证明。生成器输出 `Result` 与诊断，旧 `TargetGenerator::generate -> String` 可保留给兼容/脚手架调用，但相关运行入口必须走checked API；错误不能通过字符串注释或空返回隐藏。

API产物按generation_id保存完整、不可变bundle；源/实现指纹、契约版本、支持能力和ready状态作为启动凭据。具体目录/指针或等价启动闸门由T-01冻结，不承诺对多个文件逐个rename就是原子事务，不创建symlink/junction。

## 3. 技术栈

- 现有 Auto parser/type/source-span、API模型和serde_json，现有a2r与auto-man生成链；不引入第二套Auto解析器/通用JSON Schema引擎。
- VM默认HTTP Axum0.8；729已验证的生成Rust轨是Axum0.7，版本无关契约/错误数据共享，本地adapter各自装配HTTP对象。
- 当前TypeScript HTTP/IPC客户端与Tauri v2命令；测试使用实际锁版本的Tauri test feature/MockRuntime command dispatcher，避免启动webview。SDK测试API变化由T-01核实。
- 真实TCP、源码/依赖指纹缓存的生成实编、现有VM/转译分级门禁；port=0，异步等待用gate和截止时间。

## 4. 需求分析与背景调查

### 4.1 来源、版本、授权

起草基线 `32f41d211`（与本HTTP任务无关的733执行簿记）；729归档:r1，delivery=`050e2ee90`，规范已落盘。730起草:r1 0/9，暂无其完成/review/merge证据；本计划不能把上传视为已交付。734取号提交=`1cabfeb89`；活跃/归档max733，修正滞后counter后exclusive取734，不覆盖731/732/733。

| 来源 | 使用的规则/事实 |
|---|---|
| `docs/specs/overview.md`、`stdlib/project.md`、`stdlib/design/backend-assembly.md` | 多后台与API调用拓扑独立；merged直接返回函数值，没有HTTPstatus/header。 |
| `docs/specs/stdlib/design/http-server.md` §3/§4.1/§8.1 | 参数按名及来源优先级、错误和显式Response；目标描述与当前执行有冲突须核对。 |
| `docs/specs/stdlib/design/http-handler-async-lifecycle.md` | owner段执行、~T metadata门、scope和取消；本期不更换等待调度。 |
| `docs/specs/stdlib/design/http-server-files.md`、`auto-lang/trans/design/http-file-response-lowering.md` | 729typed文件协议、HTTP专属表示、不支持面；生成轨与VM Axum版本不同。 |
| `docs/specs/auto-lang/vm/back-proxy.md` | session各自路由，普通JSON/流/媒体路径有各自约定，不能当另一个VM HTTP。 |
| `docs/specs/auto-man/project.md`、`auto-lang/trans/overview.md` | API生成入口/手写提供者；project卡仍有“route A未建/空桩”旧描述，与现行源码相冲突。 |
| `docs/specs/auto-lang/types/design/type-representation.md`、`auto-lang/vm/architecture.md` ADR-22、`trans/rust.rs::rust_type_name` | 类型身份与存储/传输宽度需分开；Rust的int映射i64，VM当前参数cast_i32不是语言全局32位契约。 |
| [PLAN-730](730-http-server-upload-ingress.md) §5/§10 | upload typed资源/receipt和受限能力；仅为依赖合同，最终规范须T-01再读取。 |
| [Tauri test官方文档](https://docs.rs/tauri/latest/tauri/test/index.html) | test feature提供mock runtime与get_ipc_response/command dispatcher；它不是实际webview/E2E桌面认证。 |

用户本轮明确授权“730后下一步、提前规划计划”；授权只读调查和本仓计划/设计簿记，不实施/部署/修改其他计划状态，无预算或自动实施授权。

**前置：730 reviewed且merged后执行734**（729已满足）。730与本期同触API类型分类、生成器、VM响应编组/错误与入口；不能并行改这些核心文件。T-01核对730最终合同及两文件规范，语义不同则修订734，不复制旧假设。

### 4.2 本次代码证据（静态，实施期复现）

| 文件/符号 | 观察 | 本期处理 |
|---|---|---|
| `auto-man/api_gen.rs::generate_api` | full parse失败后lenient提取；提取失败/空结果可warn并Ok；body转译失败可fallback模板。 | 区分无API、显式提供者、真正解析/转译失败，真实实现不替代。 |
| 同文件`generate_rust_server/generate_api_rs` | 先写Cargo/types，再准备handler；委派/内联/模板分别决策。 | 先检查计划/全部实现，再发布完整产物。 |
| `auto-man/vue.rs`、`rust_ui.rs`、`tauri.rs` API入口 | 存在吞Err或warn后继续；Rust split生成失败仍调用start_api_server。 | error贯穿到build/run，阻止旧产物冒充当前契约。 |
| `api/targets/{typescript,tauri,axum}.rs` | 分别编组path/body/meta/返回值；通用TS HTTP有path模板未插值、参数直接按method归类的风险。 | 共用参数计划/response种类；T-01实际调用点trace，不能只修未消费的generator。 |
| `http_server.rs::push_typed_string_arg/bind_api_args_by_name` | string整数parse_i64后cast_i32；body JSON直接marshal，未逐声明类型校验；unknown type归Str。 | 检查整数范围与声明形态，不截断/默默默认；unknown不当str。 |
| 同文件`marshal_handler_value/json_value_reply` | 资源表数值探测有多分支；JSON串以`{\"error\":`开头被设500；无法序列化时可能200空body。 | 声明响应种类先分类；错误用typed控制流，不猜用户载荷。 |
| `vm/codegen.rs` merged导出缺失、`ffi/stdlib.rs::shim_warn_api_noop` | 部分API缺实现可告警后返回null/继续。 | 当前选择的merged实现不可用应可定位失败，不假成功。 |
| `api/types.rs::ApiEndpoint::body`注释 | 仍称生产未用，现行auto-man已try_transpile_body。 | 随代码修正事实说明及Spec delta，不以旧文档隐藏缺陷。 |

有已约定Spec与源码差距：str“原样文本”vs当前JSON路径、?T缺失404vs不同adapter的null、void/异步复杂终值及旧meta别名。T-01列“规范/代码/消费者/兼容影响”表，**不从源码事实推导自动修改已约定规则**；本期仅提交SD提案。必要wire语义改变须有具体revision/用户裁定后再执行，不能静默批量统一。

## 5. 详细设计

### 5.1 ApiContract与支持子集

最小信息：schema_version、module/endpoint稳定身份、source span、method/path、参数声明/来源/required/default、resolved type、async与响应kind、实现来源/指纹、各transport支持/拒绝原因。记录不含token/cookie/本地绝对文件根；指纹覆盖业务实现及实际依赖源，不只hash接口签名。ready不得引用未验证的实现。

声明kind至少区分普通JSON、显式Response、SSE/Iter、FileResponse、730 UploadRequest/Receipt；命中依据类型身份/解析结构与实际符号，不以`contains("FileResponse")`或“数值在资源表”决定。假同名类型/用户同名函数和普通int资源碰撞必须反例。

首期普通JSON数据：str/Str、bool、int、i64、有限float/double、可选值、数组、已解析命名record及其有界嵌套组合。未知/未解析/宿主资源类型按transport诊断，不降级String或JSON。int范围按类型语义与现有Rust映射核对冻结，不能以VM参数cast_i32作为语言全局32位限制；超32位合法整数不得截断，也不能为对拍擅自缩窄Rust输入。i64完整范围跨Rust/VM，TS超过安全整数范围使用明确字符串codec或显式拒绝，不能静默精度丢失；T-01冻结选择和签名，不误推广到整语言整数改造。generic/enum/循环类型是否可用以已解析能力判定，不临时伪造字段。

每个VM session持immutable契约/参数计划，不把同名函数的元数据覆盖到其他back-proxy session；缺metadata不走新的猜测型legacy fallback。现存无metadata旧入口的支持限制在矩阵明示，不借本期移除全部Builder。

### 5.2 参数与调用边界

- HTTP：path > JSON body field > query；按声明解析/范围/形态验证后marshal，缺required/错类型/溢出/NaN/Infinity为400（含param），服务编组故障500。optional/default仅按声明处理；未声明参数不自动当meta。
- 保留已有单str whole-body和meta opt-in的兼容事实，但由显式ParameterSource标记，不让它们吞UploadRequest或普通缺参。raw text、JSON string/object whole-body的边界须有fixture，旧meta名别名差异需T-01兼容表。
- body结构化对象/数组先验证字段型/必要项/深度/大小，不能仅`as_i64().unwrap_or_default()`；默认沿用现有body上限，不放大。重复query、未知字段、JSON null vs absent、optional默认顺序于T-01明确冻结，合法已支持调用不改值。
- 生成HTTP client准确插值并encode path，参数不在path/body/query重复发送，meta由宿主提供，PATCH/DELETE带body等按契约而非单纯method猜。端点已有`/api`前缀不能再加一次。错误状态与payload保留，不以空值替代。
- merged直接传Auto函数参数/返回值，不伪造HTTP 200/400；无实现加载失败或typed运行错误。IPC按command参数/序列化规则调用；metadata/文件流等transport-only能力按矩阵处理，不引入万能JSON RPC包装。

### 5.3 返回与错误（有限收敛）

JSON返回按声明验证再序列化，sync/~T终值共同分类。合法业务record含`error`/`status`仍是数据；VM执行失败、编组失败或明确Response才决定错误status。普通int在任何client/response/iterator资源id表碰撞时也返回其数值，SSE/File/Upload必须声明门+typed资源登记共同验证。

框架HTTP错误保留兼容`error`消息字段，可加kind/param/endpoint/request_id，不泄露内存地址/鉴权信息。bad argument 400、不可用transport 501或装载诊断、内部执行/编组500。生成期错和缺实现应非零build/run，不能变成上线后永远500的“已生成成功”handler。729已有诊断handler作为兼容基线纳入checked generation时处理，不能把修订后的约定冒称729原行为。

本期不强行重订str/optional/void的全部wire规则，也不更改显式Response业务状态、SSE事件schema。T-01依据canonical与消费者决定可兼容的共同JSON编码；发现要求改变已约定wire时先修订/裁定，不以对拍测试的新期望代替授权。对于未闭合的成功响应语义，只能标“不一致/尚不支持”，不能宣称所有类型跨形态完全等价；验收受支持子集需先冻结。

### 5.4 真实实现、checked generation与scaffold

每个参与运行的endpoint必须选定可验证实现：a2r实际编译业务体；解析到的明确委派函数签名；或项目明确提供且适用于目标transport的实现。手写`src/back/api.ts`是前端提供者，不自动等于Rust server实现。`AUTO_A2R_BODY=0`不能把真实业务体切到模板而假称同源。

解析失败/遗漏注解端点/转译失败/委派缺失/unsupported返回为结构化诊断：源文件+位置、endpoint、transport、阶段、reason及实际next action。strict解析不能在lenient扫描悄悄丢业务体；仍保留lenient用于明确声明-only/scaffold或手写提供者定位，输入模式必须显式。

CRUD模板只允许**显式脚手架模式**创建初始源码，标scaffold，不能被运行/发布入口当实际业务实现；普通API默认strict。T-01选择现有初始化入口或窄配置挂点，不增加默许fallback开关。无API项目仍可正常构建；被消费者引用的API缺源、同名route冲突、提取失败不得当“无API”略过。

如当前a2r对正常业务的分支return/async调用存在缺口，仅对实现本计划fixture与已有支持契约必要的真实lowering做有限修复；不靠文本替换改变业务、不做大范围transpiler重构。超范围缺口提出needs_replan及明确不可用诊断，不减少本合同验收。

### 5.5 生成发布与启动

先解析/验证/计划/生成全部backend与client到隔离临时bundle，成功校验源指纹后发布ready；启动只接受与当前源/依赖/transport匹配的ready generation。错误不覆盖有效旧bundle，但当前build/run非零且不启动它；已有运行实例的热更新失败只能明确保持上一版本，不当新版本成功。

`generate_api` Err在vue/tauri/rust_ui各调用点必须传播。只warn、`let _`忽略、catch后start_api_server、旧`.api_functions`继续消费都要有负向测试。诊断生成错误不影响无API项目及明确外部provider；执行来源标签与消费指纹不一致应拒绝。

本期生成ready记录仅描述API，不宣称stdio/io/http全部stdlib实现manifest。新产物路径/缓存以generation/source hash命名，不创建链接，清理只处理本计划自己创建的资产。

### 规范增量

以下仅提案，起草不改canonical；730落地后按最新Spec合并增量。

| delta_id | add/modify/retire | docs/specs/... target | before/after rule | rationale | acceptance IDs |
|---|---|---|---|---|---|
| SD-01 | add | `docs/specs/stdlib/design/api-transport-contract.md` | 分散参数/返回判断 → 共同API类型/来源/能力、HTTP与函数调用边界、支持子集 | 普通API可验证契约 | AC-01..06 |
| SD-02 | modify | `docs/specs/stdlib/design/http-server.md` | 按名规则有实现差距 → checked范围/形态和错误；明确返回kind与str/?T/void目标/现状差异 | 不从旧草案推定全面一致 | AC-02/03/06 |
| SD-03 | modify | `docs/specs/stdlib/design/backend-assembly.md`、`docs/specs/stdlib/project.md` | 分散路径覆盖 → API契约消费者/能力矩阵，文件/上传/流保留专属边界 | 不把使用同api.at当同传输 | AC-01/06/07 |
| SD-04 | add | `docs/specs/auto-man/design/api-generation-integrity.md` | parse/fallback/warn继续 → strict真实实现、显式scaffold、structured诊断、完整bundle/指纹与启动门 | 防错误实现或旧产物成功 | AC-04/05 |
| SD-05 | modify | `docs/specs/auto-man/project.md` | route A未建旧说明/文件特例 → 当前已用body及本期checked generation/错误传播 | 更正事实并记录交付边界 | AC-04/05/07 |
| SD-06 | modify | `docs/specs/auto-lang/trans/overview.md` | API内联/各端适配未统一 → checked签名/真实body/async终值与生成消费规则 | 同源实编而非golden | AC-03/04/06 |
| SD-07 | modify | `docs/specs/auto-lang/vm/back-proxy.md` | session局部route、参数/返回特路 → session局部契约/checked绑定与kind；流媒体仍独立 | 多session不串metadata | AC-01/02/03/06 |

## 6. 测试设计

### 6.1 支持与负向矩阵

最小同源fixture覆盖 sync/~T 标量、optional、record/array、branch return、明确委派、有状态调用，以及文件/上传/流分类。T-01按现行Spec冻结全部用例；复杂~T若目前降级null必须复现后真正marshal或明确重规划，不能标green。

| 族 | 必须观察 |
|---|---|
| 参数 | path/body/query同名冲突、URI字符/Unicode、多个path、PATCH/DELETE payload、required缺失/null/default、错误shape、声明整数边界±1/跨32位合法值/i64极值/浮点非有限；HTTP精确400不执行业务，正确args跨端一致。 |
| 返回 | str含JSON字面字符串、false/0、null/optional、record/array/~record、void、record首字段error、int碰Response/Iter/File/Upload id、用户同名类型；无伪500/空200/资源误判。 |
| 实现 | 行为明显不同于CRUD的算术/分支、有状态计数、db明确委派；修改body不改签名，实际返回及implementation hash都变化。 |
| 生成失败 | parser错误/遗漏注解、类型未解析、a2r失败/缺委派/禁用real body、引用API但缺源、输入能力非法；非零且定位，旧bundle存在也不能spawn旧服务。 |
| 发布 | 中途写入故障/源在生成期间变化/client生成失败；无ready半成品、旧bundle可保留但不能冒称新成功，no-API/明确TS provider/scaffold路径正确。 |
| 传输 | VM HTTP、实际生成Rust HTTP、back-proxy session、merged真实Auto调用、实际生成Tauri command dispatcher；只比共同函数数据，HTTP额外比status/header，IPC错误是reject而非HTTP包装。 |
| 专属能力 | 729/730正向HTTP下载上传及non-HTTP拒绝回归；既有publisher SSE分类/消费不变，不用JSON序列化stream/handle。 |

HTTP TS生成client须真正执行请求并验证path/body/错误，不仅检查字符串。Tauri用真实生成command、invoke注册和官方test MockRuntime走dispatcher，不能直接调Rust函数冒充IPC；它不证明webview/安装包行为。两个同名endpoint的back-proxy session并存，无schema/return种类串台。

### 6.2 实施门禁（起草不跑）

在 `D:/autostack/.wt/lang-734/auto-lang`：

- `cargo check -p auto-lang`、`cargo check -p auto-man`；开发 `cargo t plan734`、`cargo t api::`、`cargo test -p auto-man api_gen:: -- --test-threads=1`，入口传播测试按其实际族补充。
- 新HTTP族 `http_e2e_plan734`，`cargo nextest run -p auto-lang --lib --features test-http-e2e --test-threads=1 http_e2e_plan734`；生成backend、TS client、Tauri测试fixture实际build/run，缓存绑定source/deps hash，不跑shell裸cargo全池。
- 复审裸 `cargo t` + `cargo tv` + `cargo tt` + `cargo th --test-threads=1`；改`ui_gen/**`消费点才加`cargo tu`，不改则不跑。a2r-std执行核心不在本期，必要触及则补该crate scoped/全crate门禁。
- 不触aavm，不跑taa；不改文档生成器/schema/语法参考，不额外docs_gen。tf仅merge到期后主检出单实例 `/auto-plan:regress`。
- 旧015-notes/017-chat/023-realworld/027-file-manager代表契约 scoped生成/HTTP、729和730族回归；**不能**将未支持旧demo批量换为scaffold以保持测试绿。新增红与同命令基线逐名对照，零新增确定性红；有明确不可用能力报告，不缩验收。

新报告 `docs/plans/reports/734-api-{decision,binding,generation,parity,verification}.md`：实际基线/revision、每AC/SD证据、错误诊断原文、生成指纹/启动负测、各模式真正执行入口与限制。

## 7. 验收标准

| ID | 可观察交付 | 验证/期望 |
|---|---|---|
| AC-01 | 一个versioned contract被实际消费者使用、typed种类/session身份正确 | 同源契约输出、consumer调用图/单测；别名/假同名/两session反例，没有contains/id替代类型身份。 |
| AC-02 | 参数按Spec来源/类型/范围校验，客户端编组一致 | §6.1参数表VM/Rust/back-proxy wire、TS实际调用；坏args400且业务次数0、整数不截断、metadata不当JSON字段；IPC/merged按原生边界正确拒绝。 |
| AC-03 | ordinary JSON与typed错误/Response/资源明确区分，sync/~T返回正确 | 业务error字段200、执行/编组故障500、int碰id不变、record/array async不降null；成功编码按T-01冻结Spec/兼容合同，未知类型不伪成功。 |
| AC-04 | 所有运行API有可验证真实实现；生成失败不fallback | 非CRUD行为fixture与源码body修改实编；parser/type/a2r/委派/能力负测非零+源位置；scaffold显式且不能当运行实现。 |
| AC-05 | 完整生成bundle与当前源指纹绑定，错误传播阻止旧启动 | backend/client失败、写入故障/源变动/旧bundle存在等负测；spawn次数0、无ready半成品，noAPI/provider例正常；所有运行入口传播错误。 |
| AC-06 | 五调用形态受支持数据面真实对拍 | VM HTTP、生成Rust HTTP、back-proxy、merged和Tauri MockRuntime dispatcher运行证据；合法值/状态变更一致，TS实际HTTP执行，限制逐项明示。 |
| AC-07 | 730与既有JSON/SSE/文件能力兼容、Spec增量可沉淀 | 代表契约及729/730回归无新增确定性红；文件上传流不走JSON/fallback；所有SD证据绑定最终revision，不宣称新增部署/流支持。 |

## 8. 执行步骤

### T-01：依赖基线、兼容裁定与最小五形态原型 [✅ e67e46bba]

- 前置730独立review通过并merge；读取其最终Spec/receipt及729文件契约，记录master基线。
- `bash scripts/new-wt-group.sh lang-734 --branch plan-734-dev`；auto-down兄弟只读，禁止链接。
- 调查`api/{types,mod,targets/*}.rs`、`auto-man/api_gen.rs`及`vue/rust_ui/tauri`全部运行调用点、`vm/codegen.rs::emit_api_host_call/emit_api_http_call`、`http_server`binder/return、back-proxy局部签名；区分实际消费者与参考generator。
- 复现fallback/旧产物启动、整数截断/error载荷与资源碰撞；建立canonical/源码/消费者兼容表（str/Str/?T/void/meta/int/i64及存储/传输宽度）。冻结支持子集/参数默认与重复query/响应矩阵、generation bundle机制、scaffold挂点、真实Tauri test dispatcher及generated TS runner。
- `734-api-decision.md`和最小同源普通scalar五模式原型；无法闭合验收或需改变已约定wire时明确revision/裁定，不靠仅golden/直接函数替代dispatcher。覆盖全部AC/SD设计可实施性。

### T-02：normalized contract、diagnostic与checked接口 [✅ 4a2eb9c74]

- 依赖T-01。新`api/contract.rs`/`api/diagnostic.rs`，修改`api/{types,mod}.rs`、`api/targets/mod.rs`、必要source span提取；保留旧String返回接口仅compat/scaffold，运行消费者转checked。
- 类型身份/参数来源/响应kind/能力/实现源与指纹单源；纯验证可被VM和auto-man调用，unknown不能降Str；修正已失实body字段注释。
- 验证`cargo t api::`、`cargo t plan734`；普通/文件/上传/流与未知/别名/同名反例，diagnostic位置完整；AC-01/04/07，SD-01/03/04/06。

### T-03：VM/back-proxy参数与响应编组 [✅ d8a93d2b4+9d341b7e9]

- 依赖T-02。修改`vm/codegen.rs`、`vm/ffi/{http_server,stdlib}.rs`、`back_proxy.rs`；必要marshal纯helper归contract，VM资源执行保留宿主模块。
- session-local contract绑定，checked input和i64 marshal；response声明kind先分派、ordinary JSON不猜error字段/id、序列化失败不是空200；sync/~T合法终值一致。
- merged缺实现定位失败，不能warning+null；legacy无metadata接口边界明示。不是更换owner/CPU调度，不改所有FFI资源表。
- 验证`cargo t plan734`、VM/back-proxy真HTTP、merged真实调用/双session；AC-01/02/03/06，SD-01/02/07。

### T-04：auto-man真实实现与生成原子发布 [✅ 0015ec764]

- 依赖T-02。修改`auto-man/src/api_gen.rs::generate_api/generate_rust_server/generate_api_rs/try_transpile_body`及委派/提供者检测；纯generation计划可新`auto-man/src/api_contract.rs`。
- strict mode解析/实现/能力前置检查，真实body或明确delegate生成；错误返回Result而不warn/fallback，scaffold与TS provider显式。全部产物先进immutable bundle，ready+源/实现hash最后发布，失败不覆盖/冒用旧bundle。
- 验证`cargo test -p auto-man api_gen:: -- --test-threads=1`与真实Rust build/run；非CRUD/分支/有状态/缺实现/解析和FS故障/source变动负测；AC-04/05，SD-04/05/06。

### T-05：HTTP/IPC适配器与实际客户端 [✅ 437683bee + R1 a5284dd84]

- 依赖T-02/03/04。修改`api/targets/{typescript,tauri,axum}.rs`、`api_gen.rs`两套TS/client/handler生成；T-01确认实际UI消费者需改时才窄触`ui_gen/{vue,rust}.rs`。
- 共用绑定来源和checked型，正确path编码/prefix/PATCH-body/meta，JSON和HTTP-only资源分离；Tauri command按真实backend执行sync/async、typed reject，保留原生函数数据语义。
- 必要a2r修复只在`trans/rust.rs`真实lowering（T-01定位）；unsupported不是默认值或模板；生成服务Axum0.7与VM0.8分别适配，不通过版本混用强转。
- 验证TS实际HTTP、生成backend/command实编+Tauri dispatcher，scoped/golden+tu按触面；AC-01/02/03/06/07，SD-01/03/06。

### T-06：build/run错误传播与旧产物启动门 [✅ 75dabeda8]

- 依赖T-04/05。修改`auto-man/src/{vue,rust_ui,tauri,tauri_backend}.rs`所有API生成消费者、必要初始化/构建挂点；尤其generate Err后start_api_server、`let _`与旧`.api_functions`。
- 当前有API消费者时错误非零且spawn0；无API项目/明确外部provider正常。启动验证ready与当前source/transport，watch失败诚实保留旧运行版并报失败，不标成功reload。
- 验证auto-man入口单测和最小project实际CLI/run失败退出码、spawn sentinel、原产物hash；AC-04/05，SD-04/05。

### T-07：五形态对拍、资源分类与兼容矩阵 [✅ 438d22c47+2850e7d24+R1 a5284dd84+R2 2b116e4cd]

- 依赖T-03..06。新`crates/auto-lang/src/tests/plan734_api_contract_tests.rs`，修改`src/tests.rs`；auto-man新/现有integration test生成fixture；新`examples/http_server/api_contract/{README.md,pac.at,src/back/api.at}`，按T-01格式。
- 完成§6.1全部表，真实生成Rust/TS/Tauri test runner、merged和back-proxy session，不为测试手写替代业务；729/730HTTP正向与非HTTP拒绝回归；015/017/023/027代表契约只验证本期消费面，不做全UI画廊负载。
- 验证串行`http_e2e_plan734`及生成/TS/IPC运行入口；报告`734-api-{binding,generation,parity}.md`每格记录真入口/返回/status/hash/诊断；覆盖全部AC/SD。

### T-08：门禁、独立review与规范交接 [✅ 2850e7d24——门禁+六报告；review=下一阶段]

- 依赖T-01..07。按§6.2分级门禁、warning/format/debug/未批准延期扫描，`734-api-verification.md`绑定最终代码revision/全部AC/SD；新红与基线逐名对比，禁止用skipped/退回scaffold清账。
- `/auto-plan:review`独立验证真实执行和Spec冲突裁定；准备SD-01..07沉淀稿，merge再更新canonical/ledger/Design33与索引，归档不重开729/730。
- 清理前`bash D:/autostack/wt-guard.sh D:/autostack/.wt/lang-734/auto-lang`及兄弟仓guard clean；tf仅到期main单实例。全AC/SD闭合才可标reviewed/archived。

## 9. 复审记录

### 合并收据 PLAN-734:r1（2026-10-03，/auto-plan:merge）

- stage: merge | plan_id: PLAN-734 | plan_revision: 1 | outcome: pass
- **prepared**：reviewed 基线 6ebc85f7a（R2 pass）→ rebase 到 master（前移：735 已并，
  规范目标零冲突）→ 新链 12 提交（range-diff 逐对全等——安全改写证明；
  a5284dd84→036312096/2b116e4cd→2a024d0ae/6ebc85f7a→04fea48b2）→ canonical 沉淀
  SD-01..07（新 stdlib/design/api-transport-contract.md、auto-man/design/
  api-generation-integrity.md；改 http-server.md §14、backend-assembly.md、stdlib
  project.md、auto-man project.md、trans overview.md、vm/back-proxy.md）+ plans
  索引两行 + ledger P734-1(designs)/P734-2(designs)/P734-7(reviews)/P734-8(reports)
  （docsha 绑定）+ INDEX 再生 → 投影后裔 **delivery commit ec0eae41f**（实现/依赖与
  reviewed 状态零改动）。
- **landed**：主检出 `git merge --ff-only plan-734-dev` → tip=ec0eae41f（无 merge
  提交）；冒烟 `cargo t plan734` 7/7 + `cargo tv` 162/162。
- **ledger_refreshed**：`.autoos/specs.json`（tracked，worktree 内 upsert + git 提交）
  P734-1/2/7/8 回读在档；INDEX 再生（26 projects）。
- **archived**：`docs/plans/archive/734-api-contract-and-generation-integrity.md`
  （git mv；status archived；completion_kind: delivered）。
- **cleaned**：见下一提交（wt-guard + 工作树/branch/组目录移除后回填）。
- 部署观察：landing 非部署——本计划无本机常驻生产进程消费（auto 桌面壳未运行）；
  无 auto build 产物待重建。
- 批量回归到期判定：734 % 5 = 4 ≠ 0；回执 last_covered_plan_id=732 且回执
  时间戳 2026-10-03（<48h、其后仅 734/735 合并）——**未到期**。

### 复审 R2（2026-10-03，/auto-plan:review）

- stage: review
- plan_id: PLAN-734
- plan_revision: 1
- outcome: pass
- reviewed_commit: 6ebc85f7a（R1 回工 a5284dd84 + R2 补修 2b116e4cd + 报告 6ebc85f7a；
  分支累计 12 提交 e67e46bba..6ebc85f7a）
- base_commit: 6dd609ed9
- dependency_revisions: 同 R1
- spec_inputs: 同 R1
- acceptance_results: AC-01..07 全 pass（F-1 位点 grep 七文件全零；F-3 dispatcher 实测绿；
  F-2 报告修正；AC-03/AC-06 既有在案裁定维持）
- findings: R1 三项全修复复验；R2 补修一项——`vm_value_to_json` Value::I64 臂在 T-08
  提交时意外丢失（提交信息与 diff 不符经 `git log -S` 揭示；664 值域 e2e 红的根因），
  恢复后 tv 162/162。该丢失本身验证了"声明与工件核对"复审面的必要性。
- evidence: 终局门禁——裸 t 1492/1495（3 预存 musk）、tv 162/162、tt 1877/1880（同 3 预存）、
  th 串行 100/101（1 预存 corpora_data_face；plan707 flake/sse_chain 并行端口竞争在案）、
  plan734 7/7、plan730 e2e 14/14、api_gen 43+1（含新 Tauri fixture 锁+dispatcher 两测）、
  a2r-std 100 全。contains 生产位点：7 文件全零（2 测试断言位 tests mod 内可留）。
  规范增量 SD-01..07 目标路径/前后规则与实现一致，沉淀稿 merge 阶段落 canonical。
- next: merge（/auto-plan:merge）——SD 沉淀 + ledger + Design33 索引 + archive + wt-guard 清理。
- 独立性：R2 与实施同会话——从工件重建（grep 计数/git -S 考古/门禁复跑）。

### 复审 R1（2026-10-03，/auto-plan:review）

- stage: review
- plan_id: PLAN-734
- plan_revision: 1
- outcome: needs_fix
- reviewed_commit: 2850e7d24（分支 9 提交 e67e46bba..2850e7d24；worktree clean）
- base_commit: 6dd609ed9
- dependency_revisions: 730 已 merged/archived（canonical 已落）
- spec_inputs: http-server.md/http-server-files.md/http-server-uploads.md/backend-assembly.md/
  a2r-std project.md/trans overview.md/back-proxy.md/auto-man project.md
- acceptance_results: AC-01 **partial**（F-1）/AC-02 pass/AC-03 pass（D7 兼容回退在案）/AC-04 pass/
  AC-05 pass/AC-06 **partial**（F-3）/AC-07 pass
- findings:
  - **F-1 [P2]** AC-01 声明"53+ contains 全部退役"，但 **8 处生产位点残留**：
    `api_gen.rs` :1158（endpoint_body_params）/ :2422（db 委派文件臂）/ :2859（主文件臂）/
    :2949/:2978（上传臂）/ :3703（main.rs HEAD 自动化）；`typescript.rs` :312（FormData 参数
    剔除）/ :427（返回 response 直传）。两处（:5000/:5096）在 tests mod 内为断言——可留。
    这些位点消费 api 元数据裸名串，`MyFileResponse` 假同名反例会错误命中——与契约身份
    分类合同直接冲突。
  - **F-2 [P3]** parity 报告"既有 729/730 分支本体经同一 ResponseKind 判定改写"表述过强
    ——F-1 位点即未改写。F-1 修复后同步修正报告表述。
  - **F-3 [P2]** AC-06 的 Tauri MockRuntime dispatcher 运行证据缺失（P734-D6）——
    dev-dep 已加编译通过但 fixture 未落地。计划 §6.1 明确要求"用真实生成 command、
    invoke 注册和官方 test MockRuntime 走 dispatcher"。
  - 接受项（非阻塞，已如实记录）：D4/D5 债、D9 兼容回退（二分定位证据在 verification
    §3.3）、AC-03 的 plain int 反例由 id 空间分离承载。
- evidence: 门禁复现——`cargo t plan734` 7/7、`http_e2e_plan734` 串行 4/4、
  `cargo t plan730` 26/26、api_gen 41+1、a2r-std 100 全（R1 重跑）；
  contains 计数 grep 输出（7 文件中 5 清零、2 文件共 8 生产 + 2 测试断言）；
  代码审计——validate_body_value 接线/错误前缀退役/序列化失败 500/
  generation.json+新鲜度门/8 入口 `?` 传播全部在位。
- next: 回工修复 F-1（机械化：8 位点改 ResponseKind/is_upload_param）+ F-2（报告修正）+
  F-3（Tauri MockRuntime fixture）→ 复审 R2。
- 独立性：与实施同会话——从工件重建（grep 计数/代码审计/门禁复跑），未采信实施自述。

### 工作交接（2026-10-03，/auto-plan:work）

- stage: work
- plan_id: PLAN-734
- plan_revision: 1
- outcome: pass
- code_commit: 2850e7d24（分支 plan-734-dev，master 基线 6dd609ed9；11 提交
  e67e46bba..2850e7d24，worktree D:/autostack/.wt/lang-734/auto-lang 保留待审）
- task_ids: T-01..T-08 完成（T-07 Tauri MockRuntime fixture 登记 P734-D6）
- evidence: 六报告（decision/binding/generation/parity/verification +
  binding 补充），docs/plans/reports/734-api-*.md；门禁面——plan734 7/7、
  plan730 26/26、api:: 36/36、a2r-std 100 全、auto-man api_gen 41+1、
  VM HTTP e2e 4/4（参数校验/i64 全域/error 字段/plain int）、裸 cargo t
  1492/1495 + tv 162/162 + tt 1877/1880（3 预存 musk p053 族）、th 99/101
  （2 预存：corpora_data_face merge-base 实测 + plan707 flake 在案）——零新增
  确定性红。
- blockers: 无阻塞项。债：P734-D4（stdlib i32-lane 泛型转换）、P734-D5
  （run_with_capture 测试基建非确定挂死）、P734-D6（Tauri MockRuntime
  fixture）、P734-D7（D9 iterator/Response 臂兼容回退——Plan 326 int 声明
  wire 契约，6 e2e 二分定位后裁定保留注册表制；strict 分派保留于
  Upload/File 臂）。
- next: /auto-plan:review PLAN-734。
- 执行注记：复审驱动三项真修正——is_upload_param s-expr 双族（VM 签名侧
  Display 是 s-expr，裸名精确匹配致上传路由失识别/100-continue 泄漏）、
  back_proxy fn_meta primary 半段分类 + 参数面补充（501 守卫恢复）、
  vm_value_to_json I64 臂缺失（664 值域回归根修）。tauri 2.12 dev-dep
  （test feature）已加并编译通过（E7）。AC-06 的 Tauri 腿以生成串测试+
  E7 依赖证据部分覆盖。

### 起草交接（2026-10-03）

- stage: new
- plan_id: PLAN-734
- plan_revision: 1
- outcome: blocked
- blocker: 730尚无review/merge后的最终接口/Spec；相关核心文件按顺序实施。
- next: 730 reviewed+merged → T-01最终接口/兼容与支持子集冻结 → /auto-plan:work PLAN-734。
- changed_tasks: T-01..T-08（新）
- changed_acceptance: AC-01..AC-07（新）
- 完整规划已写出，blocked表示实施依赖未满足，不要求用户补充信息或阻止730推进。T-01发现wire约定变化再按合同裁定，不预授权批量重写成功返回。
- 本次只计划/设计簿记，未创建734worktree、未跑Cargo、未改canonical或其他计划进度；交接检查通过：11章节、8任务/7AC/7SD覆盖、现有Spec路径、新增链接、唯一编号与diff。Design索引既有513计划链接失效，未新增且不在本期修订范围。

## 10. 待澄清事项

- 不缺用户规划输入；730落地后T-01拥有具体兼容/SDK测试机制与真实实现原型。对于str/raw、optional404/null、void、meta和i64 TS编码中的Spec冲突，需要源/消费者证据与明确增量，不静默裁定整个API语义。
- 普通JSON输入/返回子集以约定Spec和T-01原型冻结，不能因现存复杂终值缺陷缩小AC。若需超范围VM值/类型系统重构，needs_replan，而不是Unknown→String或null。
- Tauri test证明command dispatcher/codec，不证明真实webview、权限/插件或安装包；SSE/文件/上传的非HTTP限制保留，不承诺完整透明RPC。
- 后续优先候选：D2b部署配置/支持等级与运维验证（监听、CORS/鉴权配置、代理/TLS边界、可重复负载/日志/关闭），以及D3标准库目标装配manifest；CPU阻塞/抢占另案裁定。本期不能据此称“完整生产HTTP能力已完成”。
