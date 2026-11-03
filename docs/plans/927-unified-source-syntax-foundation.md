---
plan_id: PLAN-927
status: executing
feature_name: SG-A 统一源码语法基础与七profile受检前门
author: [Codex]
created_at: 2026-11-03
updated_at: 2026-11-03
plan_revision: 2
supersedes_spec_components: [docs/specs/auto-syntax/grammar-contract.md, docs/specs/auto-syntax/project.md, docs/specs/auto-syntax/snapshot-cst.md]
new_spec_components: [docs/specs/auto-syntax/source-frontdoor.md]
touched_goals: [GOAL-002]
affects: [auto-syntax, auto-ac, auto-test-case]
depends_on: [PLAN-787, PLAN-828, PLAN-881, PLAN-894]
current_step: 10
total_steps: 12
---

# [PLAN-927] SG-A 统一源码语法基础与七profile受检前门

## 0. 变更摘要

三包第一包：在现有通用scanner/CST/规则程序上统一Core、Scalar、Text、Record、TextRecord、EntryUnit、HostConsole七个源码profile的规则、严格AST投影、完整与片段入口，并切换其冷编译前门。后继928承担Collections/Closures及嵌套单元，929承担所有消费者暖编辑/恢复/重复生产parser退役与SYN最终验收。单包不签新M1。

依据[架构裁定](reports/ace-m1-architecture-completeness-decision-2026-11-03.md)、[实况审计/拆包及交接](reports/ace-m1-architecture-gap-audit-2026-11-03.md)及Design43 §22。drafting r2、0/12；仅规划，所有测试与预算探针尚未执行。

## 1. 目标

- 七profile全部既有合法语法/拒绝边界及诊断事实保真，不按后端复制语法。
- 配置规则驱动共享lexer/CST，Auto专属投影产同一AST；解析仅产语法事实，类型/资源/捕获仍由原语义层检查。
- 保持无依赖环的真实编译入口；完整源、自由函数/方法片段使用同一语法规则与坐标合同。
- 交928/929稳定的语法身份、单元目录、诊断、预算、计数与profile能力接口。

非目标：不实现新Auto语法/宽数值源码、不改IR/参数模式/ABI/ownership/求解器，不替代旧产品AutoVM parser，不在本包实现所有会话恢复或完整AJ；七profile以外仍为有期限的迁移路径，显式登记给928/929，不冒完成统一。

## 2. 架构方案

采用审计§3的默认无环方向：`runtime ← grammar ← core source(CST→AST) ← syntax-core兼容适配/消费者`。core AST/Span/Diagnostic/SourceProfile身份保留；把Auto专属投影迁入core新模块（拟`experimental/ace/crates/core/src/source/cst.rs`），syntax-core薄重导出，不引入 `core→syntax-core→core` 环。runtime/grammar不能依赖core/语义/VM。若中立model抽取确有优势，T-02先证明完整依赖图/类型身份等价，再记入本计划；不能两份AST或隐藏反向依赖。

语法包共享词法、声明/表达式规则，以版本化profile能力清单保留历史边界（如Core的while与Scalar的for、Unit结果位、不同text/record面）。不得一份“全接收语法”配后端各自猜profile；禁止runtime按Auto语言ID硬编码语法分支。投影的字面量解码/AST映射不是第二份独立表达式parser。

## 3. 技术栈

现有Rust ACE workspace、Atom grammar配置、ace-syntax-runtime/grammar/core、ace-core；AutoDown/TestCase与固定职责组。auto-atom/auto-val是中立数据依赖，不能因语法接入拖入旧AST/VM/Trans。不引入tree-sitter运行依赖，不新造测试调度器。

## 4. 需求分析与背景调查

授权：用户明确本轮核清M1机制、为统一源码语法正式立项、判断拆包并写验收/923–926交接；**仅规划，不施工/建树/运行测试**。不新建chat/agent，无指定模型或预算。
**2026-11-03 /goal 实施授权**：用户指令"完成计划927"——授权整链 work→review→有界修复→merge（auto-plan-goal 范式）。施工树 `D:/autostack/.wt/lang-927/auto-lang` @ `plan-927-dev`（基面 master `2f558fc93`，含 auto-down 兄弟 detached）；材料 `plan-artifacts/plan-927/`（已建并登记 .retention/policy.json）。

初订取材 `ec3e70414`，r2复核main `f2214a65f`；输入路径+SHA256在审计§6。已读Spec overview/README、auto-syntax全部合同、source七profile/资源合同、函数增量与native诊断/AutoDown合同；Spec是当前行为依据，设计是target-state。`source/{lexer,parser,ast}.rs`分别876/3567/598行（调查快照），不能把旧手写实现改名为共享平台即称统一。原syntax-core投影依赖core，只支持有限core/scalar；Auto后缀/record/Unit/text诊断需要真实扩展，T-02先探针而非假定现成全可用。

一手架构资料与边界见审计§5；仅借鉴分层/依赖与结构复用，不声称成熟编译器必须用某种parser。旧auto-lex的checkpoint/转义与旧parser规则可作取材，旧错误行为不是绝对oracle。

实施树（尚未创建）：`D:/autostack/.wt/lang-927/auto-lang`，分支`plan-927-dev`，全生命周期唯一；材料开工后仅`D:/autostack/plan-artifacts/plan-927/`，登记owner/用途/可删缓存，merge摘要入本计划后删scope。主检出只做计划簿记。

## 5. 详细设计

### 5.1 支持与入口合同

| profile | 必保面（完整既有Spec/语料优先于此摘要） |
|---|---|
| Core / Scalar | 整数边界、bool、全部已有算子/结合性/短路、条件循环/控制流、换行与命名实参源码序；同拼写跨profile的接受/拒绝保持，现有ASCII标识符边界不因支持Unicode文本而扩大 |
| Text | str/Str/char、UTF-8/多行与CRLF/NUL/转义/Unicode标量、全部既有方法和类型词参数；文本资源规则不在parser重做 |
| Record / TextRecord | type/字段/名义引用、static/view方法、receiver、view/mut/move拼写、构造源码序、字段/方法后缀、owned/text组合与T03/T05规则 |
| EntryUnit / HostConsole | main三形态/脚本/尾值/Unit结果位；合法/非法return与void位置、print/eprint实参源码序及宿主声明，语义拒绝仍交原阶段 |

`parse_program/parse_program_with`保兼容返回类型，七profile路由至通用CST。`parse_function_fragment`原本只收free function，方法片段接口需明确owner/receiver/范围，不可偷改free-only合同；新统一片段适配器由T-07使能，原调用方在929迁移。所有片段必须完整消耗，偏移保持原源绝对UTF-8 byte span；不授权范围/半token/半方法拒绝。直接公开lexer的消费者须登记，不能旁路计量。

CST error/missing可为编辑器保留，但严格投影不得产AST成功/Checked凭证。现有source.lex/syntax/unsupported/literal-range及文本码的stage/code/span/稳定message/顺序保真；语法错误与语义错误的原阶段不挪动，不能靠字符串解析重构原因。正常调用可返回原单诊断兼容面，结构化完整诊断单源保存。

### 5.2 资源、身份与迁移

T-02先做最小整链：有owned文本的record+方法→共享CST→AST→真实原语义检查→AV/native/Rust适用入口，并测增量片段与失败修复；证明无依赖环、无重扫描投影、预算可行。包内只做合法的既有语义，不要求C新能力未交付时冒C认证。

Grammar/schema/compiler/scanner/runtime/projection/profile能力版本均入语法结果与依赖身份；CST恢复不能带trusted标志。七profile语法语义变更必须换键，不能仅改文件名。位置变动单列，不使未改body内容身份无谓改变。

沿既有预算：通用syntax流式work-owned≤64MiB档、1MiB中部单body编辑真实读取≤48KiB/块复用≥99.99%及200轮fresh等价保留；当前适用消费者的4MiB源前门、closures前端16MiB等不擅增（闭包整链由928/929证明）。工作集按真实同时存活source/CST/AST/候选/decode计费，不能多模块重复使用同一授权。深链parse/project/drop/recovery走有界工作栈，无新固定句法深度上限；预算不足是资源态，不是语法不支持。若可行性不能达标，T-02记录具名阻断并needs_replan，不能缩样本或放宽门。

本包七profile冷前门必须真正切换；Collections/Closures只允许显式登记的迁移分支，928接管。其他会话手写片段调用点由929逐项清零，不能静默新增长期fallback。

### 5.3 写面与交接

公共S：`core/src/source/{mod,ast,lexer,parser,adapter}.rs`、新cst模块；`syntax-{runtime,grammar,core}`真实接口。927先冻结core AST/Span/Profile与syntax-core重导出接缝；923 native N/fault、924闭包语义、925/926 C provider/runtime不归本包改写。

923当前3/12且N有WIP，925 current_step=2且私有C runtime WIP，924/926尚draft：无整包等待，私有grammar/case可准备。若925/926要改parser/AST/方法片段，双方具体符号串行交接；公共S不并写。928可私有研究，不在927未释放时并写core投影。H面workspace manifest/lock、测试注册/SDK/driver全文件单写，记录owner/revision/base+handoff SHA/schema/全部consumer/construct-match、有效/STALE/未验及释放；不能持窗等待慢认证。详见审计§4。

### 5.4 r2入口请求、资源结果与片段授权

统一入口的逻辑请求至少绑定`source/snapshot身份+revision、SourceProfile/能力版本、入口kind、授权range、当前owner/父单元、grammar/projection/schema与词法checkpoint/规则入口`，逻辑结果包含无损CST/目录、语法AST或惰性body引用、绝对位置/完整诊断及真实工作/资源观察。具体Rust类型由T-02冻结；完整模块、free fn、方法与lambda等kind不是四套语法实现。profile是来源合同，不从后端名、文件名或源码内容猜；911的profile升级尝试序与诊断归属保持，升到更宽profile不得偷偷扩大原请求能力或复用错profile结果。

对有上下文的会话编辑，range必须对应当前revision目录授权的语法单元/规则边界及完整scanner状态（mode/mid token/原始起始行），不得把字符串/注释内看似`fn`的子串当合法声明，也不得把半token/CRLF中点/闭包半体按默认mode重启。保持旧`parse_function_fragment(src,range,profile)`签名的孤立语法兼容结果**不是会话授权**，不能单凭该AST安装候选；真实编辑入口必须验证base/范围/owner/上下文。方法必须使用真实owner/receiver上下文，不拼假free fn或重写实参顺序。片段前缀不得整源重parse来获取上下文，使用已受检目录/checkpoint或具名非快路径；所有重扫如实计量。

T-02必须先冻结**typed成功/源码诊断/资源耗尽/stale-foreign/基础设施失败**结果接缝；旧Diagnostic返回API可以作兼容投影，但正式ACE入口必须保原typed分类，不能仅存全局last-error或由message反推。特别是`RtError::ResourceExhausted`穿过projection/adapter/worker/父侧后仍是资源态，不改为`source.lex/syntax/unsupported`或a2x语法拒绝；有原语法错误时仍按原stage/code/span/顺序。T-06/08接实际producer，V-08/09由T-09使能。

### 5.5 r2预算适用性与交窗凭据

| 预算/观察 | 适用对象与本包责任 | 关闭条件 |
|---|---|---|
| syntax work-owned≤64MiB原档 | 原787流式固定负载/条件保持；不是所有source profile统一输入上限 | 显式传入调用方预算，不沿`Budget::default().work_bytes=256MiB`冒正式授权；超限前置拒绝，最大parse工作深度是可配置资源额度而非语言unsupported |
| 1MiB / ≤48KiB / ≥99.99% / 200轮 | 原固定局部编辑样本与分母完整保留，不泛化成任意巨大声明编辑的必然比例 | 原case逐项通过；新增text/record代表例冻结输入/编辑/影响集，单列真实读量/生产与峰值，不能以新增case“不适用”删旧样本 |
| 16MiB前端 / 4MiB源 / 父WS增量128MiB | 912/913 closures等原适用消费者；927基础机制需证明可组合，928/929交实际闭包链 | 不把4MiB扩为其他profile新语法限制，不擅增原cap；current/候选/历史pin/CST/AST/解码同时存活先预留后物化 |
| A2X≤48MiB等各族预算 | 沿各族原负载/prepare/store/IO合同，统一前端不得借syntax独立额度额外占用 | T-02冻结所有权/生命周期与峰值观察，T-10接适用真实consumer，受影响门不放宽 |

同一共享payload只记一次物理总账，同时纳入所有适用预算域各自核验；不能借多域重复授权扩大可分配空间。候选失败/drop/close及最后pin释放按真实owner释放，缺采样或“只算容量不算同时存活”不能判绿；P90≤2000ms仍沿原各正式样本/窗口，不新增统一三warm门。

公共窗口必须在开写前填`文件/符号、原owner及当前WIP、稳定base+handoff SHA、输入/输出schema、所有producer/reader/construct-match、验证/STALE/pending、释放条件`。当前有原owner WIP的符号先稳定交接；未占用的私有grammar/case与明确释放S符号可先行。927七profileAPI释放给928；对925/926的方法/模式/type片段，旧兼容门面只调用新事实，接窗后不得再新增独立语法分支。H文件全文件单写，完整矩阵见[复核§4](reports/ace-927-929-plan-review-and-schedule-2026-11-03.md)。

### 规范增量

| delta_id | add/modify/retire | docs/specs/... target | before/after rule | rationale | acceptance IDs |
|---|---|---|---|---|---|
| SD-01 | modify | docs/specs/auto-syntax/grammar-contract.md | core/scalar有限规则→七profile通用表达能力与版本化能力清单 | 规则单源 | AC-01/02/03 |
| SD-02 | add | docs/specs/auto-syntax/source-frontdoor.md | 新增真实CST→AST冷/片段入口、上下文授权/typed失败/依赖图/预算与迁移边界 | 可供后继消费 | AC-02/04/05/06 |
| SD-03 | modify | docs/specs/auto-syntax/snapshot-cst.md | 补七profile资源/增量结构合同，保原预算/恢复不变量 | 防回退 | AC-07/08 |
| SD-04 | modify | docs/specs/auto-syntax/project.md | 明确七profile已接与复杂profile/会话仍待迁移的实况 | 不冒全面统一 | AC-06/09 |

## 6. 测试设计

以下V均为新要求/待执行，T-09使能文件化`crates/auto-lang/test/cases/ace-m1/source-syntax-927/`（新目录）和独立oracle；不改原用例expected/分母。旧手写parser仅作迁移差分之一，冻结语料与人工核对的结果/诊断也是独立oracle，不能两个同源实现互证冒正确。

| case | AC/不变量 | 输入/入口与owner_stage=work任务 | 可区分错误实现的断言/边界 |
|---|---|---|---|
| V-01 | AC-01/03 | T-04/05，七profile既有全部源码/错误语料；shared parse完整/片段 | 接受/拒绝集合、AST全部字段/源码序/span一致；反控改关键字/优先级必须改变真实结果，不仅改变元数据 |
| V-02 | AC-02 | T-03，manifest/实际依赖图/重导出consumer | 编译无环、AST/Span/Profile同一类型；core不得依赖syntax-core，runtime无语义/旧VM |
| V-03 | AC-03/04 | T-04/06，UTF-8/CRLF/多行/转义跨页、坏转义/未闭合/越界literal | 无损字节一致；诊断stage/code/span/message/顺序按冻结合同；分辨resource与syntax，坏树严格不成功 |
| V-04 | AC-04/05 | T-06/07，方法/命名参数/字段/脚本尾值/片段多余token | AST保存owner/receiver/源码求值序；完整消耗与range授权，不能把方法伪造free fn再误认语义 |
| V-05 | AC-06/09 | T-08/09，真实七profile冷入口/semantics adapter及禁调用手写冷parser反控 | 真实结果/诊断正确；强制旧路或偷偷重复tokenize必须让结构门红；复杂profile仍登记迁移，不冒通过 |
| V-06 | AC-07/08 | T-02/10，1MiB局部编辑、64MiB原流式/深链档、连续200编辑及失败修复 | 保原实际读取/复用/预算，投影/回收不爆栈；预算precheck先于物化，缩预算反控具名拒绝 |
| V-07 | AC-07/09 | T-07/09/10，首例语法产物fresh/hit/进程退出-restart/body-edit、grammar/projection/schema失配与合法近邻 | 真实持久化读回/重验后继续局部编辑，版本换键/受控拒绝；本包语法产物恢复不冒929全编译来源P0，不能把旧CST/AST缓存当凭证 |
| V-08 | AC-04/05/06 | T-02/07/08/09，新typed入口：错误profile、旧revision/owner、字符串或注释内伪fn、半token/CRLF/错误checkpoint，各配当前目录授权合法近邻 | 真实会话编辑先准入，孤立parse成功不授权安装；绝对span/全消耗/receiver与profile保真，不为定位前缀整源parse |
| V-09 | AC-03/04/08/09 | T-02/04/06/08/09，同一合法源显式缩work_bytes/parse工作栈、缺预算或峰值观察、坏转义近邻 | 正常预算成功；资源拒绝先于大物化，typed资源态穿过worker/父侧不变源码错误；坏转义仍原诊断，禁止沿默认256MiB偷过64MiB档或隐藏候选峰值 |

固定职责组（从`scripts/test-groups.toml`只读核对，未运行）：`cargo t ace-syntax ace-source ace-semantics ace-comptime ace-lowering ace-av ace-native ace-a2x ace-artifacts ace-adapters auto-test-case infra-fixed`；影响共享source/AST，不能只测parser。T-10按冻结真实触面选择必要集成组：`case-ace-m1-base case-ace-m1-entry-862 case-ace-m1-console-863 case-ace-m1-method-881 case-ace-m1-text-source-894 case-a2r-hir-record-text-902`，C scalar/925组以实际已注册为准。拟新增固定组`case-ace-source-syntax-927`由T-09注册。原正式性能/重内存/平台条件保留，性能走`cargo t ace-performance`中受影响已命名样本，P90≤2000ms，不叠全仓三轮；需改样本选择时先冻结实际case，不能静默减原门。

实现/内部自查/独立review按角色顺序在同一树进行，用户未指定人或模型；本轮不派发。冻结提交后复用未失效证据，列实际执行/原条件排除/未执行，正式采样与923/925错峰。

## 7. 验收标准

| AC | owner_stage | 关闭条件 |
|---|---|---|
| AC-01 | work | 七profile全支持/拒绝清单和V-01全等，未扩/缩原语义或分母 |
| AC-02 | work | V-02真实依赖图无环，通用runtime中立，公共类型/API兼容 |
| AC-03 | work | V-01/03配置驱动scanner/grammar与字面量合同成立，无硬编码语言旁路 |
| AC-04 | work | V-03/04/08/09严格投影与完整诊断/typed资源失败保真，不重复分析类型/ownership或铸Checked |
| AC-05 | work | V-04/08函数/方法片段range/owner/profile/revision/checkpoint/EOF/绝对span及会话授权合同实际接通 |
| AC-06 | work | V-05七profile真实冷入口切新链，复杂/会话迁移残余逐项交928/929，无静默fallback |
| AC-07 | work | V-06/07真实局部工作/版本/恢复准入可观察，未改生产与重扫分账 |
| AC-08 | work | V-06/09原预算/深链/样本及适用域保持，typed失败资源态、真实峰值与无泄漏/候选回滚成立 |
| AC-09 | work | V-05/07正式文件case/oracle/计数反控与固定组/consumer验证齐，交接含SHA/STALE |
| AC-10 | review | 独立核AC-01..09与SD/遗漏/原语义/健康，revision-bound pass，必要债务有owner不冒本包关闭 |
| AC-11 | merge | 已复审Spec实况沉淀、交付/archive、材料路径对账/摘要/删除及guard清树收据，原M1未冒签 |

## 8. 执行步骤

| T | owner_stage | 前置/真实写面 | 产出/验证与关联AC | 状态 |
|---|---|---|---|---|
| T-01 | work | 主线/Spec/923–926实际SHA，无需整包D | 全七profile/调用点/语法拒绝/预算/窗口冻结，输入hash漂移对账；AC-01/06/09 | [x] 2026-11-03：12/12 审计输入 SHA256 零漂移（master 2f558fc93）；冻结清单 `plan-artifacts/plan-927/t01-inventory-freeze.md`（七profile谓词/四入口全部生产调用点/诊断码与门控拒绝面/六族预算锚/写面实况：923Δ公共S=0仅test-groups+16行、925Δ=0、公共窗空闲）|
| T-02 | work | T-01，私有语法与最小whole-path探针 | 5.2/5.4/5.5可行性、typed outcome/片段上下文/显式预算/无环与合法失败近邻；失败先needs_replan不减门；AC-02/04/08 | [x] 2026-11-03：typed outcome（Ok/Diag/ResourceExhausted/StaleForeign/Infra）+ FrontRequest/显式预算 + 步1预算前置检查（1字节预算→ResourceExhausted 探针绿）；whole-path `probe_whole_path_text_record`（record+owned text+方法→AST 与旧全等）绿。checkpoint `5eaabff42` |
| T-03 | work | T-02，core/syntax-core/manifest H短窗 | 迁投影与兼容重导出/依赖边；V-02，cargo t ace-source ace-syntax；AC-02 | [x] 2026-11-03：投影住 core/src/source/cst/（frontdoor/kinds/lex/project/shift）；syntax-core 薄重导出接缝（frontdoor_reexport_927.rs：同类型断言+typed前门经接缝）；依赖图无环（core 不依赖 syntax-core，runtime/grammar 无语义/旧VM）；cargo t ace-source PASS、ace-syntax-artifacts 31/31（修 T-04 结束行约定的 EOF 双计）。conformance 298/298 |
| T-04 | work | T-02/03，syntax scanner/grammar/codec | 文本/char/转义/后缀通用机制与版本反控；V-01/03；AC-01/03/08 | [x] 2026-11-03：scanner 跨行 token 记结束行（SCANNER_ABI /2）；program codec/schema v3（+pratt postfix_tail）；PARSER_ABI /2（cut 深度语义）；pratt WantOp postfix-tail 续读（KIND_POSTFIX，`f().len()` 调用后点后缀链）；char 转义/char-multi/unterminated span 逐字节对齐旧 lexer；尾随未解析 token=ERROR 叶。差分 567/567 绿 |
| T-05 | work | T-04，grammar资产/profile清单 | 七profile完整规则/拒绝面，复用组合规则；V-01；AC-01/03 | [x] 2026-11-03：auto-source.at 七 profile 全编译；str/char/affix 字面量与 move/mut 参数 mode 全域结构化（投影层按旧 Raw 臂消息拒绝）；host-console 脚本形态（旧 is_entry_unit 含 HostConsole 谓词对齐）；差分 567/567（含 70 冻结 edge）绿 |
| T-06 | work | T-03..05，core新cst模块 | 严格AST/诊断/typed资源分类/来源投影，V-03/04/09；AC-04/08 | [x] 2026-11-03：迭代帧机投影（调用栈序修正：receiver 值在实参下；path-assign path-seg 收集）；classify_word ret 位 record 收任意 ident（旧 fn_item_tail 名义分支）；type-item 成员序列全镜像（field/member_type/duplicate/空 record）；期望游走=区域首叶起的全树后缀叶列（跨恢复区域边界）；旧 continue-'outer 怪癖保真（leading-dot 空调用/空 ctor）；EOF 期望错误（function name/`{` 等）；保留 token/尾随叶分类。差分+edge 全绿 |
| T-07 | work | T-06，parser薄门面/片段新接口 | 同规则完整源/free fn/方法片段、上下文授权与计数/identity；V-04/07/08；AC-05/07 | [x] 2026-11-03：FrontKind::{FreeFnFragment, MethodFragment}（range/owner/receiver 上下文）+ parse_fragment_front + parse_function_fragment_via_cst 旧签名桥 + shift.rs 全 AST 绝对坐标平移；free-fn 片段差分（AST 全等+绝对 span+多函数/坏区间反控）与方法 owner/receiver 恢复测试绿。旧 `parse_function_fragment` 保持 free-only 原合同不动（调用方 929 迁移）；会话编辑准入（V-08 的 revision/checkpoint 授权）由 T-09 接实际 producer |
| T-08 | work | T-07，source adapter/semantics消费者S窗 | 切七profile冷入口/登记剩余路径给928/929，V-05；AC-06 | [x] 2026-11-03：`parse_program_with` 七 profile 路由统一 CST 链（Collections/Closures=显式登记迁移分支走旧手写 `parse_program_with_handwritten`；差分 oracle 列亦用之）；SyntaxWork 计量保真（note_tokenize/record_whole/note_parse 按投影结果补记）。切链暴露并修复：restore 丢 expr_anchor（e18 `||` 后行操作数）、一元操作数 same-line 规则步（e19）、int-literal same-line 只绑负号分支、program 顶层 ListPlain（同线相邻声明）、`[]` 标点/非 ASCII Raw(NonAscii) 词法合同、HostConsole 脚本面+console 保留名+expr-stmt 门（旧 is_entry_unit 谓词）、旧 stmt head 起始 token 分支（c03）。ace-core conformance 300/300 + 差分 567/567 + 70 edge 全绿 |
| T-09 | work | T-05..08，文件化case/父侧/H窗 | 使能V-01..09/独立oracle/typed父准入/反控与固定组，AC-04/05/07/08/09 | [x] 2026-11-04：固定组 `ace-source-syntax-927`（567差分+70edge+片段，oracle=真 handwritten 列）+ 文件化 `case-ace-source-syntax-927`（case-worker/contract 双侧 `auto-source@<profile>` 路由 + 七 profile lossless oracle）7/7 全绿；**D-927-1 已修**（Save 携 pending 内容；连带修复被自比掩盖的 postfix 键错位与 checkpoint 双重计数）债务已关。typed 会话准入 producer 接线仍显式交 929 |
| T-10 | work | T-09，冻结提交及正式采样错峰 | §6完整职责/必要消费者、原预算/性能/深链及交928/929，AC-01..09 | [x] 2026-11-04：冻结提交链至 `70d8f62f0`；固定组全绿：ace-syntax（31/31 artifacts）/source/semantics（附件环境位补齐）/comptime（静机）/lowering/a2x/av/adapters/native/runtime/native-worker/test-case/infra-fixed + case-ace-m1-{base,entry-862,console-863,method-881,text-source-894}、case-a2r-hir-record-text-902、case-ace-source-syntax-927（7/7）+ ace-source-syntax-927；ace-performance 见 §9 采样收据；928/929 交接=迁移分支显式登记（parse_program_with_handwritten/collections-or-closures slug/片段 producer 929）。ace-host 不在本包 §6 清单（3 红为基座/环境归因，已登记） |
| T-11 | review | T-01..10完成，停止实现写入 | 独立逐AC/SD/遗漏与健康复审，AC-10 | [ ] |
| T-12 | merge | T-11 pass与后续合入授权 | Spec沉淀/交付/归档/所有输出路径清理，AC-11 | [ ] |

下一执行入口仅T-01：先比审计输入hash与实际owner，未领公共窗前只作私有调查；T-02证明前不开始大面积迁移。用户本轮不授权work。

## 9. 复审记录

- 2026-11-03，stage:new / revision:2 / outcome:pass（仅规划复核）/ next:work（待后续实施指令及实际写面就绪）。本轮按用户要求补实入口授权/typed资源态、同文lambda身份与真子体复用、首编辑禁止预热、恢复发布与预算适用性、923–926逐符号交接，详[复核及完整排期](reports/ace-927-929-plan-review-and-schedule-2026-11-03.md) PR-01..08。保留原AC/T编号、总步骤和0进度；新增V例均待使能执行，r1不是本r2实现证据。下列r1为初订历史。

- 2026-11-03，stage:new / revision:1 / outcome:pass（规划合同可交接）/ next:work（仅在后续明确实施指令下）。路径、依赖环、profile边界、V→AC→T、原预算与923–926交接已做文档静态核对；实现/测试/独立实施复审未执行。未把文档核对当AC通过，全部任务未勾。


- 2026-11-04，stage:work / plan_id:PLAN-927 / revision:r2 / outcome:pass（work-owned T-01..T-10 主体完成，T-09 case-worker 路由与 T-10 集成组收尾在途）/ worktree:`D:/autostack/.wt/lang-927/auto-lang` @ `plan-927-dev`（base `2f558fc93`，HEAD 见 §8 T-10）/ 证据：差分 567/567+70 edge+片段差分（ace-core conformance 300/300，`cargo t ace-source-syntax-927`）；固定组绿名单见 T-10 行；墙钟发现与处置：CST 规则机 debug 档大源慢一个量级→test 档 O2+CteDriver 预热（cte 300ms deadline 合同恢复）；`!call` 一元算子经 callee 分析丢失→修复（is-digit C 链回归清零）。/ blockers：无阻塞用户决策；余项=case-worker 七 profile 路由+.case.ad 文件化（T-09 尾）、case-* 集成组跑齐（T-10 尾）、comptime 并行负载归因复跑。/ next_responsible_role: work（收尾后 review）

- stage: goal | plan_id: PLAN-927 | plan_revision: r2 | round: 1 | from: work | to: work | status: executing | outcome: in-progress | worktree: D:/autostack/.wt/lang-927/auto-lang @ plan-927-dev (HEAD 39dcdc64f) | commit: 39dcdc64f | next: work（唯一入口=D-927-1：列表机 restore/trivia 修复→去3 skip 复验→T-09 勾→T-11 review）

- 2026-11-04（轮2），stage:work / plan_id:PLAN-927 / revision:r2 / outcome:pass（work-owned T-01..T-10 全数完成）/ worktree:`D:/autostack/.wt/lang-927/auto-lang` @ `plan-927-dev` HEAD `70d8f62f0` / 证据：D-927-1 修复（Save pending 内容化）+ 差分 oracle 复真（handwritten 列——T-08 切链后 corpus 曾 new-vs-new 自比，复真后暴露并修复 postfix-tail 键错落 core-ops）+ checkpoint 行号 T-04 约定双重计数修复；`case-ace-source-syntax-927` 7/7、差分 567/567+70edge、conformance 300/300、§6 全组绿（ace-performance 采样收据见本节末）/ blockers：无 / next_responsible_role: review（T-11 独立复审）
- 2026-11-04（轮2补记），stage:work / 正式采样收据：`cargo t ace-performance`（T3_MILESTONE=1 4h 档，静机）**59/60 绿，唯一红 `mn.t11.refused`=D-927-2 基座既有**（ZZ927LEGACY 回退手写 parser 复跑同红 + base 编译本就不过 + 301,387B rev0 负载新旧 AST 逐字节全等——三重证据，非 927 回归；已登记 KNOWN-DEBT）。cte_turnaround 全系绿（含 127s 重腿）/ 交接：928=Collections/Closures 迁移分支（`parse_program_with_handwritten` + `collections-or-closures` slug）；929=片段 producer/会话准入 + 散落手写调用点清零

## 10. 待澄清事项

无需要用户重新裁定的语义范围。本包沿已批准支持/拒绝、预算与P90门；具体通用scanner/Pratt扩展及兼容类型导出由T-02有界探针决策并记录，若不能满足则needs_replan，禁止自行延期或改门。语义查询/实例/CT接线与CA-D/最终ARCH总验收仍为独立待立项职责，详审计，不由927隐式承担。
