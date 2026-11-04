---
plan_id: PLAN-929
status: executing
feature_name: SG-C 全源码入口增量恢复统一与重复parser退役
author: [Codex]
created_at: 2026-11-03
updated_at: 2026-11-04
plan_revision: 2
supersedes_spec_components: [docs/specs/auto-syntax/source-frontdoor.md, docs/specs/auto-ac/function-incremental-source.md, docs/specs/auto-ac/closures-function-production.md, docs/specs/auto-av/closures-source-recovery.md, docs/specs/auto-a2x/incremental-and-cache.md]
new_spec_components: [docs/specs/auto-test-case/unified-source-syntax-admission.md]
touched_goals: [GOAL-002]
affects: [auto-syntax, auto-ac, auto-av, auto-a2x, auto-test-case]
depends_on: [PLAN-927, PLAN-928, PLAN-866, PLAN-889, PLAN-911, PLAN-913, PLAN-921]
current_step: 8
total_steps: 14
---

# [PLAN-929] SG-C 全源码入口增量恢复统一与重复parser退役

## 0. 变更摘要

消费927七profile基础与928复杂profile/嵌套单元，统一所有ACE实际Auto源码入口：cold、单函数/方法/lambda暖编辑、失败修复、hit/restart/恢复后的下一编辑。版本化接入来源/语法/声明环境，保真实未改函数生产0、恢复深重验和预算，删除/隔离手写重复生产parser并接严格父侧SYN验收。

本包最终关闭[裁定SYN-01..06](reports/ace-m1-architecture-completeness-decision-2026-11-03.md)，只为ARCH各门提供语法迁移后的证据；不冒整个M1通过。[机制审计](reports/ace-m1-architecture-gap-audit-2026-11-03.md)中SQ/INST/CT、CA-D、最终组合验收仍待立项。drafting r2、0/14；本轮只规划，不施工。

## 1. 目标

- 所有真实源码consumer统一消费版本化语法/CST/AST事实，兼容API不再含独立语法或隐式fallback。
- 对原支持的函数/方法/lambda单body编辑保未改P=0；恢复后第一次编辑也不得借M轴隐藏整源parse/resolve/环境重建。
- 声明/type/enum/签名变化正确传播，源字节未改的依赖者不能错误命中；无关函数生产0，实际目录/词法/准入与必要重验分账。
- 原资源语义、诊断事实、同PID在线HMR与AV/native/Rust/C适用认证不回退；SYN父侧准入能拒伪造/缺失/STALE证据。

非目标：不替换旧产品AutoVM/A2X，不删除Atom配置/受检HIR文本parser、不同领域grammar或测试参照；不补全语义求解器/用户泛型/CT源语法/公共ABI桥/全AJ。语法统一不是借机扩所有后端能力或豁免已承诺范围。

## 2. 架构方案

同一 `SourceProfile + SourceSnapshot + GrammarId/projection/schema + Decl/Unit身份` 驱动完整与片段解析，各consumer保留自己的执行/目标生命周期，不能在后端重解Auto语法。

语法会话拥有snapshot/CST/声明单元目录，原CheckedEnv/closures registry仍保语义凭证；候选源/语法/语义/产物随唯一提交点一起发布。失败/取消/stale/foreign/迟到不得更换current root、诊断或source身份。严格projection消费当前树，不能把CST缓存页当Checked凭证。

恢复分账：R可对源页、CST/目录、声明上下文/IR做必要结构/版本/依赖深重验；P（parse/project/resolve/typecheck/plan/bind/HIR/lower/encode/emit/write）不得先做再宣称恢复。纯恢复缺必要页具名拒绝或显式cold请求另跑，不能在P0车道静默降级；成功恢复必须保足下一次局部编辑的上下文。

## 3. 技术栈

927/928统一语法API，现有source CheckedEnv/候选、AV/closures/resident、native incremental/collections、A2xSession/ManagedStore、comptime/semantics源码adapter，AutoDown/TestCase严格父decoder与固定组。无第二store、独立测试引擎或guest执行器。

## 4. 需求分析与背景调查

r2复核main `f2214a65f`，本轮继续仅修订规划，不施工/建树/构建测试。原交付基线与以下初订输入保留，当前状态/写窗见[复核排期](reports/ace-927-929-plan-review-and-schedule-2026-11-03.md)；头状态与实际WIP分开核，不能自动扩大923–926合同。

授权仅本轮调查/正式规划/写面交接，不建树/施工/测试/派发。基面 `ec3e70414`；实际输入hash见审计§6。后继开发需读取D927/D928与所有实际consumer SHA，不把当前设计视为已存在接口。

源码核实：`core/source/adapter.rs`完整源入口；`av/src/session.rs` scalar暖CST；`core/source/closures_fn.rs` lambda/owner手写片段；`native/src/collections_incremental.rs`候选；`a2x-store/src/lib.rs`完整/片段/恢复路径；`semantics/src/adapter.rs`、`comptime-core/src/consumer.rs`使用core源码入口。直接lexer探针、source driver升级尝试序与生成文本source-map也须盘点。921纯恢复P0存在，但`ensure_env`可能在恢复后首次编辑整源重建并记env_rebuilds/M，不能由“恢复本身P0”推出新M1已闭；925/926承诺的新source恢复需一起核，而非重复施工。

已读Spec：auto-syntax、auto-ac/function-incremental-source/closures-function-production/native-source-diagnostics、auto-av/closures-source-recovery/resident-online-reload、auto-a2x/incremental-and-cache及正式认证、auto-test-case适用规则/诊断/成本归因、ABI分域。早期AX-B type片段暂不可用已被904后续交付覆盖，不重开旧包。

实施树（2026-11-04 开工建组）：`D:/autostack/.wt/lang-929/auto-lang` @ `plan-929-dev`（基面 master `03e9869fb`，组内含 auto-down 兄弟 detached `7c4d5cb`）。材料唯一`D:/autostack/plan-artifacts/plan-929/`（已登记 `.retention/policy.json` in-flight），按当前政策登记/merge摘要入计划并全删，所有外部输出路径逐项对账；主检出只做簿记。**实施授权（2026-11-04 /goal "完成计划929"）**：授权整链 work→review→有界修复→merge（auto-plan-goal 范式），含建树；§8 末"用户本轮不授权work"仅指 2026-11-03 规划轮。开工时点事实：PLAN-928 由另一活跃会话在途执行（lang-928 worktree，本日 09:05 开工 T-01），D928 尚不存在——本包按 §5.7 分层推进：①T-01 调用清单/冻结可先行；②T-02 起公共实现需 D928 稳定接口（不把在途分支当D）。主检出未提交项均属其他会话（930-933 规划簿记、examples/ui/015-notes、test/fixtures/reader-import-budget、scripts/tools/native-fixture/Cargo.lock），已具名上报不入本包、待其 owner 路由。

## 5. 详细设计

### 5.1 全入口与交付责任

| consumer/实际源码面 | 必须接通的路径 | 保留语义owner |
|---|---|---|
| core `source/{adapter,parser,closures_fn,closures_context}.rs` | 九profile完整/函数/方法/lambda、错误恢复、type/enum/签名事件、各compat公开API | 原resolve/typecheck/verify/capture/resource facts，不在parser重分析 |
| AV `session.rs`、closures与resident/cache模块 | scalar与closures冷/暖/失败/零编辑/restart/next-edit、同PID安全点更新 | 原函数凭证/registry/env/cell/旧链保活及有界回收 |
| native `incremental.rs`、`collections_incremental.rs`、`source_diagnostics.rs`、driver/worker | source-v1诊断、profile升级链、函数/集合/实际可用闭包入口、typed结果原身份 | 923 fault/Place与924 closure语义不接走，宽位型不冒Auto宽语法 |
| A2X `a2x-store/src/lib.rs`、worker | Rust/C各适用profile cold/hit/ranged/method、来源绑定恢复→下一编辑、目标版本隔离 | 925/926 C资源/runtime/发射与原Rust provider保持 |
| semantics/comptime/测试与工具adapter | 所有Auto源码解析调用统一；不误删Atom/config/模型输入解析器 | 796/817实际语义查询/CT执行合同，不冒普通前端SQ/CT接线已闭 |

T-01建立完整调用点清单（包含测试/工具/公开直接lexer与依赖树）；T-10最终逐条标记统一门面/已删除/隔离测试/不同语言合法保留，未判定调用点不能关退役门。迁移期间927/928残余列表合并于本表实例，不允许新隐式fallback。

### 5.2 增量、接口变更与资源语义

- body编辑支持集：自由函数、record static/view方法、含type/enum声明的模块、lambda单体与owner含未改lambda，以及顶层脚本的合成入口单元；目标P恰预期生产次数，未改逻辑单元parse/project/resolve/typecheck/HIR/plan/bind/lower/encode/emit/persist为0。无变化重放全部P0，真实link/execute与R/M单列。
- type字段/enum arm/函数或方法签名/声明增删/重排：目录与来源依赖正确更新，受影响消费者按实际依赖重验，无关函数生产0；prefix/空白/span漂移不改body语义身份。跨声明或词法不收敛可具名扩大语法重扫，不能用整模块语义重建当支持路径；不能因调用点没改就盲hit。通用meta/用户实例机制不足记录给SQ/INST后继，不伪称本包实现。
- 同一候选只提交一次；源、CST、目录、CheckedEnv/registry、目标产物和diagnostic revision一致。失败候选诊断绑定候选源，当前执行态不被替换；修复编辑正常继续，异常/infrastructure分态。
- AST模式/源序保真后，参数mode、移动/只读view/显式clone、共享捕获/写回/逃逸及cleanup仍由同一受检资源事实驱动。必须保原L04/L05=42、K02 creator退出、K06写回、T03/T05原source和真实fault/cleanup断言，不以只跑语法测试代替。

### 5.3 来源恢复、预算、时间

版本轴包括grammar/schema/compiler/scanner/runtime/projection/profile、source/context/decl/owner/caps/producer及后端版本。恢复读取无trusted位，CST/目录、上下文及凭证重新准入；source不匹配、跨owner、坏引用/页/版本、缺required、坏slot/布局都在使用前受控拒绝。原913等更严格的拒绝/miss分类不放宽。

恢复后下一次body编辑禁止整源parse/resolve/重建环境；若需持久化语法/声明环境，则由本包增加版本化required边/恢复顺序和预算，不把guest活env/cell/指针或Checked票据序列化。R的decode/deep-verify可非零，不能混入未改函数P；若合法会话仅cold入口被显式请求，whole成本如实报告，不能充restart/P0。

保所有原适用预算：syntax64MiB档、1MiB局部读取≤48KiB/复用≥99.99%/200轮；closures前端16MiB、source4MiB、父working-set相对冷基线≤128MiB；A2X稳态管理集≤48MiB及原store/prepare/IO上限，各target族合同更严格者优先。source/CST/AST/候选/解码瞬态/缓存同时存活计费；旧版本被调用或lease持有合法保活，最后owner/close后回收。不得因接CST擅放宽资源或缩样本。正式release原样本与整窗口P90≤2000ms，准备/冷构建/执行/清理分列；可行性失败needs_replan。

### 5.4 退役与严格父侧门

重复手写Auto源码lexer/parser从生产依赖中退役；可留无独立规则的公开兼容门面及明确test-only参照。不能改名搬到consumer、删检查脚本却留另一入口、用缓存结果藏重复解析；以实际调用图、依赖边、禁止旧实现编译/运行反控证明。旧产品parser、Atom配置/受检HIR reader及不同领域语法仍由各owner维护，本包不越权删。

新增版本化SYN报告/父decoder（拟schema名与字段在T-02冻结），绑定真实source/grammar/producer/target/revision、各P/M/R计数/预算/认证身份、原始结果和成功终态。缺字段/未知字段/错类型/伪计数/缺采样/错source/STALE/hash漂移拒绝；用真实生产反控，不能只靠报告自填0。新门与旧acceptance/2/原313及C新增门分列，不能把SYN绿冒ARCH总绿。

### 5.5 与923–926及927/928交接

927→928的稳定语法/AST接口是公共迁移前置；无需等待整包才能私有盘点，但最终交付必须含真实D927/D928。923 N故障/Place独立，929不改其语义；native collections接线取其稳定版本并重验原故障反例。924的闭包candidate/registry/worker共享S，按具体符号SHA取窗，不并写；其未交付时native closure认证pending，最终适用腿不允许省略。

925/926 C provider/runtime主体独立；`a2x-store/lib.rs`、方法/type-enum片段、来源manifest/next-edit是实际冲突面：C包先交其原功能与接口，929负责语法接管，若C包已补相同恢复能力则直接消费并验证而非重做。929最终C rt/coll认证等待实际功能与稳定源码接口可用；这只是适用腿的验收依赖，不给所有私有研究加D925/D926整包前置。

S：source/AST/候选/诊断/目录、全部session与语法adapter；H：公共driver/SDK/TestCase注册/父decoder/固定组/workspace manifest-lock/来源规则，H全文件单写。每窗含owner/revision/base+handoff SHA/schema/完整producer-consumer与construct-match/已验/STALE/pending/释放；正式采样错峰，不能持窗等认证。原Plan合同/进度不自动改写，实际接口漂移使受影响证据STALE。

### 5.6 r2首操作、真实计数与事务恢复

**首编辑观察必须从新进程恢复请求开始**：restore成功→立即提交真实body/方法/lambda/脚本单元编辑→执行并比较独立oracle；另一路restore→首个坏编辑→修复→第二个有效编辑。中间不允许dummy edit、ensure_env、整源parse/resolve、提前提取目标AST或先访问所有body作预热；快路径状态/producer计数在restore前安装并跨首操作保留。source SHA/目录校验可以做，但真实parse/project/resolve/clone/发射等无论发生在R、M、prepare、decoder回调或错误早退都计实际producer attempt；失败attempt也不能默认0。

恢复R允许按原合同decode/deep-verify声明/IR/语法页，这些有真实读量与工作，不要求其I/O=0；它不得解析源码、重新投影全部AST或重新lower来计算恢复键。恢复后的暖编辑中，未改单元的body物化/解析/语义/发射/持久化为0，未改目标artifact payload额外读写亦为0；共享只读元数据、源hash、目录/坐标重锚与有界词法重扫按M/I/O单列，不能把全body遍历/AST深clone改名元数据。这里的未改artifact指函数语义/后端片段等单元payload；最终文件集装配、模块链接、manifest发布可有真实整体输出，须单列装配/链接及其字节、I/O、峰值并纳总预算，不能借装配重新parse/lower/print未改片段。T-02冻结每真实调用点/阶段与观察窗口，父侧核缺计数/Default-0/出窗工作，不只信worker自填字段。

来源恢复不强制把完整AST作为可信镜像持久化：可以用现913声明上下文+受检单元交叉验证、版本化CST/目录与按单元惰性body，或等价机制。但源hash相等不免context/owner/capture/type依赖重验；913 dump原本不含body，不得补trusted body/pointer/PASS跳验证。首编辑应按当前单元context选择合法parse入口，不能为补上下文扫描全部未改函数。

语法/CST/目录/context/IR/目标片段的发布必须形成**同一次提交的required闭包**：先验证/写齐immutable子项及清单→取得保活→注册新root→按原协议退役旧root，最后返回安装回执；中断不能产生“新source/grammar根+旧registry/目标片段”的半可恢复状态。各具体seam故障必须证明可见root仍闭包完整、当前旧态可用或具名拒绝，清故障后下一次真实restore/编辑成功。资源/guest执行与checkpoint时序仍由原owner合同，不将该发布协议冒新guest热重载。

旧来源锚缺grammar/projection/profile能力身份时不能被当新格式P0恢复；版本隔离或明确迁移请求沿原协议处理，迁移确需生产则报告显式cold/repair成本，不能隐式fallback再签restart。合法历史版本被lease/pin保活的页不为通过预算盲删。V-12..14由T-09使能，T-02/08提供真实恢复/发布入口。

### 5.7 r2预算域、typed结果及交接前置

| 门/域 | 范围与观察窗口 | 本包必验 |
|---|---|---|
| syntax原64MiB/1MiB局部/48KiB/99.99%/200轮 | 原固定case/输入/编辑与条件保持，不泛化为任意巨声明编辑 | 927/928原证据按输入判有效/STALE；新增复杂case另冻实际影响集/峰值，不能删原分母 |
| closures原源/context/manifest与前端/恢复 | 源≤4MiB、单manifest≤4MiB、声明context≤4MiB、单wire≤8MiB、单元≤4096、恢复namespace≤128MiB，前端池≤16MiB及原code/value/verify cap；父WS相对冷基线≤128MiB | 原限制按913/912适用入口读前/物化前检查，原诊断/资源分类不改；新增CST/目录/两代候选/pin/解码暂存都计真实同时存活，不把这些限制扩大成其他profile新增语法限制 |
| A2X原≤48MiB/各族prepare-store-IO | 原各负载/目标族与受管工作集定义保持 | 不能拿另一目标cache或syntax预算抵扣授权；source restore与首编辑整个窗口采集，旧runtime共享与新候选峰值按真实所有权归账 |
| 正式时间/控制/typed失败 | 原样本/规模/平台/资源组，release端到端P90≤2000ms；源码错/资源/取消/infra分态 | 不先预热移窗，不吞`ResourceExhausted`为source错误，缺采样/假0/缩预算/spike等反控；不新增统一三warm方案 |

同一共享payload物理总账一次，但各适用预算域都验证自身占用；不能多域重复预留变相增空间。close/取消/失败候选与最后pin均核实际release与工作集收敛，不只比较history长度。原P/M/R报告schema若不能表达当前事实，T-02先版本化冻结并由T-09父decoder严格接入，不靠message或全局last-error回填。

前置分三层：①调用清单/schema/私有case研究可先；②T-02实际整链及公共迁移需927/928稳定接口和对应consumer符号窗；③最终AC-08/09/11适用native/C认证必须已有924/925/926实际能力与稳定交付输入，不得pending/条件skip冒PASS。frontmatter的公共语法依赖不等于给私有准备增加923–926整波门；也不豁免最终必需腿。

具体S交接为`core AST/typed outcome/上下文入口`927→928→929，`closures_fn/context/registry`与924按符号串行，`A2xSession/method/type-enum/source-origin/ensure_env`与925→926按稳定接口交接。925/926若先实现同一恢复能力则本包只接语法事实并重验，不重复施工；未解决的最终普通前端接缝归本包。native allocator/Place/env语义仍923/924所有。H全文件单写；每窗均列owner/revision/base+handoff SHA/schema/全部producer-reader-construct-match/有效-STALE-pending/验证和释放条件，见[复核§4](reports/ace-927-929-plan-review-and-schedule-2026-11-03.md)。

### 规范增量

| delta_id | add/modify/retire | docs/specs/... target | before/after rule | rationale | acceptance IDs |
|---|---|---|---|---|---|
| SD-01 | modify | docs/specs/auto-syntax/source-frontdoor.md | 九profile统一机制→所有真实consumer/冷暖恢复与退役证据 | 单一正式入口 | AC-01/02/10 |
| SD-02 | modify | docs/specs/auto-ac/function-incremental-source.md | 补九profile语法事实/声明变更与完整P/M/R轴 | 不藏生产 | AC-03/04/05 |
| SD-03 | modify | docs/specs/auto-ac/closures-function-production.md | 保原capture语义，补统一语法单元/候选/失败恢复 | 保912闭环 | AC-03/06/07 |
| SD-04 | modify | docs/specs/auto-av/closures-source-recovery.md | 补单次来源发布required闭包/语法目录版本、无预热首编辑与原深重验 | 恢复不先生产 | AC-05/07 |
| SD-05 | modify | docs/specs/auto-a2x/incremental-and-cache.md | Rust/C来源恢复与下一编辑共用语法/环境，移除重复解析 | 去掉恢复后全源重建 | AC-02/03/05/08 |
| SD-06 | add | docs/specs/auto-test-case/unified-source-syntax-admission.md | 新增SYN门schema/typed失败/真实P-M-R-I/O/严格父侧与独立oracle反控 | 可签语法门，不冒ARCH | AC-09/11 |

## 6. 测试设计

由T-09新增`crates/auto-lang/test/cases/ace-m1/unified-source-syntax-929/`、独立oracle与原源引用；全部未执行。原NA/accepted规则、typed位型轴、a2r金样/独立认证身份保持。Q4用同字节source，在各目标原批准适用面各对独立oracle；闭包/HostConsole等不适用某目标的面如实列原规则，不能因此省各自适用AV/native场景。

| case | AC | 输入/实际入口/work任务 | 断言、反控与覆盖边界 |
|---|---|---|---|
| V-01 | AC-01/02 | T-03..07，各consumer调用清单，九profile完整/片段及公开API | 真实路由统一规则/AST事实；运行时禁旧生产函数反控必红；Atom/不同领域与旧产品parser合法保留 |
| V-02 | AC-03/08 | T-03..06，自由函数/方法/lambda/owner/含type-enum的单body编辑及脚本合成入口编辑，zero-hit | 每单元真实P恰目标集合、未改0；源码读取/目录/R单列，强制整源parse、AST深clone或兄弟重验的反控必红 |
| V-03 | AC-04 | T-07，type字段/enum arm/签名/声明增删与空白/前缀漂移 | 受影响消费者重验或正确拒绝，无关函数P0；文本未改callee接口变更不可盲hit；fresh与增量结果/诊断等价 |
| V-04 | AC-05 | T-02/08，fresh→hit→进程退出→restart→第一次局部编辑，AV/native/Rust/C适用来源路径 | restart源生产P0、R深验非零且合法；next-edit未改P0/whole0；把ensure_env重建移M轴或缺页静默cold必须红 |
| V-05 | AC-05/07 | T-08，source/context/grammar/schema/owner/caps/unit身份/required页损坏，各配合法近邻 | 消费前受控拒绝/miss按原合同，零错误凭证/调用；无锚不伪恢复，合法当前非零身份接受 |
| V-06 | AC-06 | T-03/04/07，失败暖编辑→修复、cancel/foreign/stale/迟到，坏语法/语义/infra分别 | 诊断绑定候选源且stage/code/span/message/顺序保真，旧运行态不被错替换，修复后fresh等价，不查last-error冒本次事实 |
| V-07 | AC-07/08 | T-03..06/09，原T03/T05、L04/L05=42、owned view/clone、K02/K06/S02与原fault | 模式/资源/捕获facts单源、每出口/失败原子/最后清理，不能AST对拍或只数日志冒执行资源正确 |
| V-08 | AC-08 | T-03/09，resident H01-H03同PID正在运行时编辑 | chain_entries=1、旧在途链/后续新调用各对版本oracle、最后旧代回收；重启/重入/仅暂停恢复替代必须红 |
| V-09 | AC-07/09 | T-02/09/11，原syntax/closures/A2X预算、深链/200编辑/正式负载与close/最后lease | 原峰值/读取/工作栈/资源/P90≤2000ms保持；预算前置，引用在用保活、最后drop收敛；缩预算/spike/漏采样反控 |
| V-10 | AC-09/11 | T-09/11，真实Q4/适用认证与父schema篡改：错类型/缺键/未知键/错源/过期SHA/假0/坏结果 | 严格父侧拒伪报告，各目标独立工具链真实compile/run，认证与日常分轴；各required门单列，未执行不冒绿 |
| V-11 | AC-10 | T-10，生产依赖/调用图与禁止旧实现编译反控 | 再引旧符号或改名复制旧规则被结构门发现；兼容门面只调统一链，测试参照不入产品构建 |
| V-12 | AC-03/05/06/09 | T-02/03..08/09，新进程restore后立即首个真实body/方法/lambda/脚本编辑；另一路首编辑失败再修复，禁止任何dummy edit/ensure_env/提前目标AST访问 | 从restore前连续观察到执行结束，首编辑whole0/未改P0；失败attempt真实计数且current未被替换，修复等fresh；把预热或rebuild移R/M/出窗必须红 |
| V-13 | AC-03/05/07/09/11 | T-02/08/09，恢复deep-verify合法R读取，暖编辑强制未改body读取/ASTclone/隐藏producer；显式资源不足、缺计数/采样 | R读取如实非零但不源码生产；暖未改artifact payload额外I/O0且所有真实P0，伪元数据walk与Default-0被父侧拒；资源态穿新入口/worker不变语法错误，原预算边界有合法近邻 |
| V-14 | AC-05/06/07/11 | T-02/07/08/09，新语法/context/IR发布每seam故障、错版本旧锚/required缺页/同源异owner，清故障后下一restore及编辑 | 只安装单次提交的闭包版本；旧root/pin合法保持或明确拒绝，无半可恢复新态/旧语义混入；显式cold成本不冒restartP0，真实恢复深验与后续局部编辑成立 |

职责固定组：`cargo t ace-syntax ace-source ace-semantics ace-comptime ace-artifacts ace-lowering ace-av ace-native ace-a2x ace-adapters auto-test-case infra-fixed infra-ace-m1`；必要集成并集为`case-ace-m1-base case-ace-m1-native-diags-911 case-ace-m1-collections-864 case-ace-m1-closures-865-source case-ace-m1-shared-mutable-909 case-ace-m1-closures-fn-912 case-ace-m1-closures-recovery-913 case-ace-m1-resident-866 case-ace-m1-method-881 case-ace-m1-entry-862 case-ace-m1-console-863 case-ace-m1-text-source-894 case-ace-m1-native-collections-889 case-ace-m1-typed-native-891 case-a2r-hir-record-text-902 case-a2r-hir-collections-904 case-a2c-hir-scalar-921`及实际923–926注册组。拟新增`case-ace-unified-source-929`由T-09注册；不是本轮已存在组。受影响正式性能沿`ace-performance`原样本/规模/条件和资源组，不为“多组”重复运行相同样本，不机械要求全仓三轮；最终冻结职责/必要集成并集只跑有效所需证据。

实现/内部自查/独立review在唯一实施树顺序交接，独立review不自行修代码，返work后受影响证据失效；未指定人员/模型、不派发。测试清单区分实际执行/原条件排除/未验，慢认证/正式采样与其他计划错峰。

## 7. 验收标准

SYN门最终映射（927/928交分项证据，929按真实consumer复核）：

| M1门 | 本包关闭AC | 必需来源 |
|---|---|---|
| SYN-01 | AC-01/02/10 | 九profile/全部调用清单、单规则事实、重复生产parser退役 |
| SYN-02 | AC-03/04/06 | fresh与body/声明/错误修复逐次等价、实际依赖重验 |
| SYN-03 | AC-06/07 | 无损CST/严格失败、候选事务、取消迟到不覆盖 |
| SYN-04 | AC-05/07/09 | restart深重验→下一编辑、版本/预算与资源峰值 |
| SYN-05 | AC-03/04/07 | 全真实P/M/R生产边界，未改函数与保留lambda为0 |
| SYN-06 | AC-08/11 | 适用四端真实执行/认证与严格父准入、禁止旁路反控 |

| AC | owner_stage | 关闭条件 |
|---|---|---|
| AC-01 | work | V-01全调用点清单闭合，九profile各真实入口统一，范围不减 |
| AC-02 | work | V-01冷/片段/adapter/工具/driver及来源身份同规则，兼容API不含另一语法 |
| AC-03 | work | V-02/12/13真函数/方法/lambda/脚本入口增量、zero-hit及未改P0，真实attempt/暖I/O/隐藏whole/深clone反控红 |
| AC-04 | work | V-03声明/type/enum/签名影响精确传播，增量与fresh等价，无关函数P0 |
| AC-05 | work | V-04/05/12..14纯恢复P0且无预热首编辑不整源重建，单次发布闭包/版本/依赖深重验与合法近邻 |
| AC-06 | work | V-06/12/14首编辑失败/修复/取消/迟到/发布故障事实与事务一致，诊断和typed资源分类保真 |
| AC-07 | work | V-05/07/09/13/14原资源各适用域/两代候选pin峰值/最后owner清理与凭证信任、真实P/M/R/I/O分账成立 |
| AC-08 | work | V-02/07/08适用AV/native/Rust/C真实运行、资源、typed与在线HMR不回退 |
| AC-09 | work | V-09/10/12/13原正式样本/P90/内存门及独立认证齐，无首编辑预热/出窗；未验/NA/accepted严格区分 |
| AC-10 | work | V-11重复生产parser退役/隔离，实际依赖与反控闭合，未越权删不同语言/旧产品parser |
| AC-11 | work | V-10/13/14版本化父侧SYN准入与全部SYN-01..06映射齐，拒漏计/伪资源分类/混合提交恢复；与923–928交接SHA/STALE/适用腿证据完整，ARCH余账不冒关闭 |
| AC-12 | review | 独立逐AC-01..11/SD/遗漏/健康revision-bound pass，未完成具名在合同内不得延期冒过 |
| AC-13 | merge | 已审Spec实况/交付/archive/全部输出路径材料对账删除/guard清树收据；M1总签章仍待其余ARCH/原门/C门 |

## 8. 执行步骤

| T | owner_stage | 前置/真实写面 | 产出/验证与AC | 状态 |
|---|---|---|---|---|
| T-01 | work | 私有调查可先，927/928及923–926实际状态 | 调用清单/生命周期/版本/预算/所有写窗和原正式样本冻结，AC-01/11 | [x] 2026-11-04：五面只读盘点于冻结基面 `03e9869fb`；主件 `plan-artifacts/plan-929/t01-call-inventory-master-03e9869fb.md`（52 入口/全部生产+测试调用点/退役预分类/不同语言保留）+ 余项件 `t01-freeze-versions-budgets-windows-samples.md`（版本轴/六域预算原值/写窗实况/正式样本清单）。923-926/927 全部已归档交付、边界符号具名；**D928 写窗+稳定接口 SHA 具名 pending**（另一会话在途，按 §5.7②属 T-02 前置，不把在途分支当D） |
| T-02 | work | T-01、927/928稳定接口，最小store/candidate整链 | 冷→restart→无预热首编辑/失败修复、嵌套单元/发布seam可行性；typed父schema/恢复数据/预算域/真实计数与I/O设计，失败needs_replan；AC-03/05/07 | [x] 2026-11-04：D928 落地（`6b429502f`，基面 ff 至 `37503a3f5`）；实现 `UnitCatalog::from_restored` **零生产惰性恢复**（身份-only headless 单元+字节级深重验+`RestoreError` 八类使用前拒+类型感知嵌套检查）+ Option 化惰性物化（未改单元跨恢复→首编辑 P=0 保持 headless）+修复 to_manifest module_sha 陈旧；8 用例套件 `source_restore_929_t02.rs` 全绿（零生产恢复/无预热首编辑恰目标物化/失败零变异修复/7 类伪造拒绝/foreign+stale+零编辑重放/跨编辑链续/嵌套 owner⊇λ）；`cargo t ace-source`+`case-ace-source-composite-928` PASS（worktree `1f029a4a4`）。**设计冻结**：SYN 报告 schema=`ac-source-syn-report/1`（身份/P-M-R-IO 四轴/预算域/认证终态，父 decoder 拒伪五规则，字段骨架见 plan-artifacts 草案，T-09 接线时终冻）；恢复数据=`ac-source-units/1` 清单+惰性 body+RestoreError 分类；真实计数=ProductionMeter（produced/reused/discarded 分账）+StageAttempts（whole/per-unit 双轴）；预算域六域原值不变。消费者 store/会话集成（ensure_env 消除的 a2x/AV/native 落地）归 T-03..T-08 |
| T-03 | work | 927/928稳定接口，T-02，AV/core S窗 | scalar/closures候选与常驻session切统一语法；V-01/02/06/08，AC-02/03/06/08 | [x] 2026-11-04（worktree `404ccba21`）三面迁移：①closures 片段生产者→统一前门桥（新增 `parse_closure_fragment_via_cst` 合成宿主入口+`parse_function_fragment_via_cst`，closures_fn.rs edit_body 两处切接）；②常驻 scalar session 切共享 `profile_grammar` 缓存（**auto-scalar.at 退出生产依赖**；暖建 ast 视图复用统一 `core_program` 投影=零二次投影；暖编辑变更 decl 走 `FreeFnFragment` 统一片段前门；`decl_names` 本地名序替代 outline spec 命中——统一语法 decl 注册在顶层分派规则）；③`front_door_admit` 深度度量改共享扫描器供给（语法 token 名+拼写分类，LevelChain/F798 语义原样）+mem_stage_probe lex 面同迁。验证：`ace-av`/`ace-source`/`ace-syntax` 全组 PASS+闭包 912/913/865/909+composite-928 绿（清 `%TEMP%\av798-session-*` 后；**av798 跨运行污染处置余账→T-09**）；av crate 存 928 基线 fmt/警告漂移 6 文件（非本包触面，复审健康项具名） |
| T-04 | work | T-02，native稳定923/924消费接口窗 | native cold/collections/闭包实际入口与source-v1诊断；V-01/02/06/07，AC-02/03/06/08 | [x] 2026-11-05（worktree `8365fe856`）：native 唯一残留 legacy 调用点 collections 暖编辑片段→`parse_function_fragment_via_cst`；冷入口/closures 面 928 已统一（parse_program_with 九 profile 路由）、924 闭包前端经 T-03 核心侧统一；source-v1 诊断随 compile 链自动统一。**计量保真修复**：fn 片段桥补 `note_parse`（真实函数名）——统一前门不做 per-fn 计量，collections 结构门（改函恰 1、无关 0）初判红后修复归绿。ace-native+case-889/911 PASS；911 P90 body-edit 首红 2560ms→隔离复跑绿（环境页面文件归因，同 928 先例） |
| T-05 | work | T-02，Rust与925/926稳定A2X接口窗 | A2xSession cold/method/type-enum/ranged统一与原provider不回退；V-01/02/07，AC-02/03/08 | [x] 2026-11-05（worktree `77eb17eae`）：a2x-store **六处 fragment 生产调用全部切统一桥**（ranged 旧片段身份校验 ×2、候选片段 ×2、暖 body/method 编辑 ×2）——新增 `parse_method_fragment_via_cst`（合成 type 宿主+占位字段满足 record 面最少字段约束、receiver 从语法/owner 从调用方、shift.rs i64 核带符号回绝对坐标）；a2c_rt 28/28、ace-a2x + case-902/921 全绿。925/926 provider/runtime 面零触碰（TargetProvider 之后全 SemanticView）。**ensure_env 消除归 T-08**（需持久化 env/目录设计，from_restored 已就绪） |
| T-06 | work | T-02，semantics/comptime/工具adapter窗 | 剩余Auto源码reader切共享门面，严分Atom/模型输入；V-01/02，AC-01/02/08 | [x] 2026-11-05（worktree `b6edf15d9`）：case-worker `auto-source@<slug>` 语法臂切 `profile_grammar` 共享缓存（进程 OnceLock，消除线程本地第二份裁剪编译；`profile_from_slug` 导出）；auto-core.at/interface-mini-v1.at = **不同领域语法合法保留**（测试用例 DSL，具名登记）；semantics/comptime/core CLI 均经统一路由（928 后无独立分支）。auto-test-case 组绿。至此**生产依赖中重复 Auto 源码语法编译清零**（T-10 只剩 cfg 隔离+结构门） |
| T-07 | work | T-03..06，来源目录/CheckedEnv/registry | type/enum/签名依赖、失败与迟到事务、完整诊断；V-03/06，AC-04/06 | [ ] |
| T-08 | work | T-07，来源manifest/codec/store/会话S窗 | 语法/上下文原子required闭包/版本隔离与深准入，恢复后首编辑去整源重建；V-04/05/12..14，AC-05/07 | [x] 2026-11-05（worktree `0234f3a22`+fmt `3e810a3c2`）：**ensure_env 整源重建退役**——新 `core/source/decl_env.rs`（`core-decl-env/1` 确定性声明面清单：签名/参数 mode/receiver/记录形状/VT 池；严格解码；stub-body `CheckedEnv` 装配，sigs 按 work_key owner 限定）+ AXD1 存储层（幂等发布、只读豁免）+ ensure_env 改为深验装配（profile/caps/source-hash 绑定+池越界受控拒）；**恢复→首个 ranged/method 编辑 whole 轴=0、parse 只计片段面**（a2x_decl_env_929：MSVC 产物 value:23 正确+清单缺失具名拒）；decl_env_roundtrip（字节全等回环/零 parse 装配/五类篡改拒）；rt_restore 4/4、store_cache 5/5（计数按新工件更新：persist 5→6、quota 头窗、settle 后总量断言沿测试自身 WAL/trash 纪律）；ace-a2x 真实失败清零（full_corpus 首红隔离绿=负载环境）；ace-source PASS。原子发布闭包六步沿 anchor 幂等提交点（V-14 seam 面归 T-09 文件化负控） |
| T-09 | work | T-03..08，case/父decoder/注册H窗 | 使能V-01..14文件化oracle/SYN报告严格准入及无预热/I/O/原子发布/Q4/资源/HMR/预算反控；V-07..10/12..14，AC-07/08/09/11 | [ ] |
| T-10 | work | T-03..09，调用清单闭合后 | 重复手写lexer/parser生产退役、兼容门面/测试隔离与负控；V-11，AC-01/10 | [ ] |
| T-11 | work | T-10，冻结提交、正式采样错峰 | 职责/必要集成并集与真实正式时间/内存/认证，AC-01..11 | [ ] |
| T-12 | work | T-11 | SYN-01..06收据、旧门/C/ARCH余账与实际交接摘要，无总M1冒签；AC-11 | [ ] |
| T-13 | review | T-01..12完成，停止实现写入 | 独立逐AC/SD/遗漏/健康最终复审，AC-12 | [ ] |
| T-14 | merge | T-13 pass及后续授权 | Spec沉淀/交付/归档/所有输出目录清理，AC-13 | [ ] |

下一入口T-01私有清单，T-02最小恢复整链在迁移前完成；公共实现需927/928稳定接口与所有实际owner释放，不把在途分支当D。用户本轮不授权work。

## 9. 复审记录

- stage: goal | plan_id: PLAN-929 | plan_revision: r2 | round: 11 | from: work | to: work | status: executing | outcome: in-progress（T-07/T-08 完成 8/14） | worktree: `lang-929` @ `plan-929-dev`（代码至 `0234f3a22`） | commit: 0234f3a22 | next: work（T-09 case/父decoder/注册）
  - T-07（`decl_events_929_t07`）：references_name 扩全语法引用面（注解嵌套+全部名字叶）；签名事件必达/dependents 已物化-only/前缀漂移身份携带/**fresh≡增量清单全等**。T-08：decl-env 声明面持久化+stub 装配（913 checked_env_from_parts 先例推广）；过程修复：fld/mth/arm 编码组名残留、sigs owner 限定键、只读发布豁免、发布幂等（restart 零新增）；quota 测试按自身 settle 纪律核总量（被拒 put 的 WAL/trash 瞬态=既有会计行为，具名记录）。
  - 用户指令"继续后续，直到计划全部完成"——授权 T-07..T-14 全链连续执行。

- stage: goal | plan_id: PLAN-929 | plan_revision: r2 | round: 10 | from: work | to: work | status: executing | outcome: in-progress（T-04/05/06 完成 6/14） | worktree: `lang-929` @ `plan-929-dev`（代码至 `b6edf15d9`） | commit: b6edf15d9 | next: work（T-07 声明/签名依赖传播）
  - 三面交付：T-04 native（`8365fe856`，collections 暖片段统一+计量保真 note_parse 修复）；T-05 a2x（`77eb17eae`，六处 fragment 统一+method 合成 type 宿主桥+shift i64 核）；T-06 工具（`b6edf15d9`，case-worker 共享 grammar 缓存）。**生产依赖重复 Auto 语法编译清零**。
  - 过程修复三个真问题：①统一前门片段无 per-fn 计量→结构门红（补 note_parse）；②统一语法 `static fn` 仅 type 块合法→method 桥合成宿主；③record 面最少一字段→占位字段。911 P90 首红隔离绿=环境归因（同 928 页面文件先例）。
  - 串行五组全 PASS：ace-native/ace-a2x/case-902/case-921/auto-test-case。

- stage: goal | plan_id: PLAN-929 | plan_revision: r2 | round: 9 | from: work | to: work | status: executing | outcome: in-progress（T-03 完成 3/14） | worktree: `lang-929` @ `plan-929-dev`（代码 `1f029a4a4`+`404ccba21`） | commit: 404ccba21 | next: work（T-04 native 面）
  - T-03 交付（用户"继续完成计划"指令恢复 goal）：closures 片段生产者统一前门桥；常驻 session 共享 profile grammar（auto-scalar.at 生产退役）；admit 探针/内存探针共享扫描器。过程发现并处置两个真差异：统一语法 decl 注册在顶层分派规则（outline spec decl-rule 不命中→decl_names 本地名序）；常驻面 CoreKinds 828 分类法依赖 type-ann 规则名（统一语法 param 直接绑 ann-ty）→暖建/暖编辑投影全部改走统一链（core_program/FreeFnFragment），kinds 面整体退役。
  - 验证：ace-av/ace-source/ace-syntax 全 PASS+912/913/865/909+composite-928 绿；av798 污染清后复绿（处置归 T-09）；av crate 928 基线 fmt/警告漂移 6 文件具名（非本包触面）。
  - 环境教训：固定组运行器单实例锁（`target/test-groups/active`）——并行两个 `cargo t` 会互斥失败，需串行。

- stage: goal | plan_id: PLAN-929 | plan_revision: r2 | round: 8 | from: work | to: work | status: executing | outcome: in-progress（T-02 完成 2/14） | worktree: `lang-929` @ `plan-929-dev`（基面 ff `37503a3f5`，代码 `1f029a4a4`） | commit: 1f029a4a4 | next: work（T-03 AV/core 会话切统一语法）
  - **D928 落地**（19:00：merge pass `4f7dc8c16`、交付 `6b429502f`、归档+清树收据齐全）→ §10 恢复入口执行完毕：基面 ff、交接面读取（冷入口九 profile 统一/清单 codec/UnitCatalog/候选事务移交 929 恢复消费）、T-01 STALE 增量件 `t01-delta-d928.md`（fragment 三 API 仍 legacy=本包迁移面；929 侧环境余账三项）。
  - T-02 交付：`from_restored` 零生产惰性恢复整链+8 用例套件全绿+受影响组 PASS；设计冻结（SYN schema/恢复数据/计数轴/预算域）记入 §8 T-02 行。测试自证抓到并修复两处真问题：嵌套合法性（Fn⊇λ）初版误拒、to_manifest 陈旧 module_sha（928 遗留，编辑后导出即错）。
  - 用户裁定沿袭"①层研究+定期复查"至 D928 落地后自动转入施工（四草案已作 T-02 输入消费）。

- stage: goal | plan_id: PLAN-929 | plan_revision: r2 | round: 7 | from: user | to: work | status: executing | outcome: in-progress（①层研究四草案完成；D928 仍未落地，继续定期复查） | worktree: `lang-929` @ `plan-929-dev` | commit: master `757b4aef1` | next: user→work（D928 落地即 §10 恢复入口）
  - 用户裁定更新（17:02 AskUserQuestion）：由"纯等待"改"**①层研究+定期复查**"。产出四草案（plan-artifacts/plan-929/，均标注 DRAFT 待 T-02/T-09/T-10 真链冻结，不构成冻结决定）：`t02-draft-syn-report-schema.md`（SYN 报告 schema 提案 ac-source-syn-report/1：身份/P-M-R-IO 计数/预算域/认证终态/父 decoder 拒伪规则）、`t02-draft-restore-nextedit-options.md`（恢复→首编辑去整源重建：913式上下文持久化 vs 版本化CST/目录+惰性body 两选项+a2x/AV/native 各面建议+T-02 最小整链验证清单）、`t09-draft-v-case-skeletons.md`（V-01..14 逐 case 骨架：输入/断言/必红反控/依赖使能）、`t10-draft-retirement-negctl.md`（退役三道结构门：cfg 编译期不可达+调用图断言+负控自证，改名复制/缓存藏解析检测）。
  - 928 观察：16:36-17:07 仍 9/12（T-09 formal V faces 已在 worktree `d8dbe56d3`，master 簿记待 T-10 段）。

- stage: goal | plan_id: PLAN-929 | plan_revision: r2 | round: 6 | from: user | to: user | status: executing | outcome: waiting（轮内长等待检查点：D928 未落地，928 健康推进） | worktree: 保留 `lang-929` @ `plan-929-dev` | commit: master `757b4aef1`（929 无变化） | next: user（继续等待循环至 D928 落地，然后按 §10 恢复）
  - 10:09-14:28 轮内轮询（~10min 块）：928 会话持续活跃，进度 2/12→8/12（T-03 collections surface、T-04 语料 162/162、T-05 候选事务/pin、T-06 decl units/typed change events/CatalogManifest、T-07 冷入口统一切换 consumers green、T-08 迭代walker/深链/200编辑/末主回收；一致性 312→315/315；worktree 现 `5d7759586`）。master 另有第三方会话提交（`c7bf9aa1f` AGE 命名等）。预计 T-09..T-12+review+merge 还需 1.5-3h。无失速（未见 >90min 无活动）。

- stage: goal | plan_id: PLAN-929 | plan_revision: r2 | round: 5 | from: user | to: user | status: executing | outcome: **blocked**（三连续等待轮到期，D928 未落地） | worktree: 保留 `lang-929` @ `plan-929-dev` | commit: master `757b4aef1`（无变化） | next: user
  - 10:09 第三次复查：928 仍 executing 2/12，archive 无 928。按 auto-plan-goal 等待轮上限转 blocked。**解除条件（=§10）**：master 出现 PLAN-928 merge 交付 + `docs/plans/archive/928-*.md` → `/goal resume`，恢复入口：rebase lang-929 基面 → 读 D928 交接 SHA → STALE 增量刷新 T-01 冻结件 → T-02。检查点全部保留可续。

- stage: goal | plan_id: PLAN-929 | plan_revision: r2 | round: 4 | from: user | to: user | status: executing | outcome: waiting（挂起轮 2/3：D928 未落地） | worktree: 保留 `lang-929` @ `plan-929-dev` | commit: master `757b4aef1`（无变化） | next: user（D928 落地归档后按 §10 恢复；连续第 3 个等待轮将按 caps 转 blocked 收据）
  - 10:08 复查：928 仍 executing 2/12（距上轮 1 分钟，无变化）；archive 无 928。继续挂起。

- stage: goal | plan_id: PLAN-929 | plan_revision: r2 | round: 3 | from: user | to: user | status: executing | outcome: waiting（挂起轮 1/3：D928 未落地） | worktree: 保留 `lang-929` @ `plan-929-dev` | commit: master `757b4aef1`（无变化） | next: user（D928 落地归档后按 §10 恢复）
  - 10:07 复查：928 仍 executing 2/12，worktree `3219cad1e`+5 WIP 文件活跃推进中；archive 无 928。按用户裁定继续挂起，无工作无额外检查。

- stage: goal | plan_id: PLAN-929 | plan_revision: r2 | round: 2 | from: work | to: user | status: executing | outcome: waiting（用户裁定挂起等 D928） | worktree: `D:/autostack/.wt/lang-929/auto-lang` @ `plan-929-dev` | commit: 03e9869fb（树无代码提交；T-01 两冻结件在 plan-artifacts） | next: user（D928 落地归档后 /goal resume，入口动作见 §10）
  - T-01 完成（1/14，勾选+证据入 §8）。依赖墙复查：928 会话活跃推进至 2/12（worktree `3219cad1e`、master 簿记 `d0f67f75a`，其 T-02 可行性门已过——closures profile 走统一 frontdoor、sealed-body-handle 机制已定）。AskUserQuestion 三选一，用户选"挂起等 D928（推荐）"：不空转、不接管、不把在途分支当D；恢复条件与入口动作固化 §10。检查点保留（实施树/冻结件/retention）。

- stage: goal | plan_id: PLAN-929 | plan_revision: r2 | round: 2 | from: work | to: work | status: executing | outcome: in-progress（T-01 完成 1/14） | worktree: `D:/autostack/.wt/lang-929/auto-lang` @ `plan-929-dev` | commit: 03e9869fb（树尚无代码提交，纯盘点轮） | next: work（T-02 被 D928 阻塞 → 用户决策）
  - T-01 勾选：余项件冻结版本轴/预算域（六域原值不放宽）/写窗实况（D923..927 全归档+边界符号，D928 在途具名）/正式样本清单。**T-02 依赖墙浮出**：公共实现链 T-02..T-11 全部需要 D928 稳定接口（§5.7②），928 会话活跃在途（当日 09:38 WIP）；本包自身无替代路径（计划明文"不把在途分支当D"）。按 auto-plan-goal 停止条件提交用户决策：等待 928 交付 / 授权另行路由 / 重新排期。

- stage: goal | plan_id: PLAN-929 | plan_revision: r2 | round: 1 | from: start | to: work | status: executing | outcome: in-progress | worktree: `D:/autostack/.wt/lang-929/auto-lang` @ `plan-929-dev`（基面 master `03e9869fb`，auto-down 兄弟 detached `7c4d5cb`） | commit: 03e9869fb | next: work（T-01 调用清单冻结）
  - 2026-11-04 09:41 /goal 实施授权：用户指令"完成计划929"——授权整链 work→review→有界修复→merge（auto-plan-goal 范式），记录于 §4 与本收据，后续各 stage 继承。
  - 依赖事实（具名）：PLAN-927 已交付归档（master `5d5054d5d`/`03e9869fb`）；**PLAN-928 由另一活跃会话在途执行**（lang-928 worktree 有当日 09:38 未提交 WIP，绝不并发触碰）→ D928 不存在，本包按 §5.7 分层：T-01 先行，T-02 起需 D928 稳定接口，届时不把在途分支当D。
  - 簿记：plan-artifacts/plan-929/ 已建并登记 retention（in-flight）；顺带修复 plan-928 scope 登记被并发写覆盖丢失（按其 §9 收据补回），并具名上报 plan-927 空目录+陈旧 in-flight 条目待其会话收尾、主检出其他会话未提交项（见 §4）。
  - T-01 进展（调用清单已冻结）：4 只读 agent 盘点五 consumer 面于冻结基面，主件 `plan-artifacts/plan-929/t01-call-inventory-master-03e9869fb.md`（52 入口 API 总账+生产/测试调用点+退役预分类+不同语言保留清单）。关键事实：7 profile 整源已 927 CST；**Collections/Closures 整源与全部 fragment 解析（parser.rs:167/239/335）仍 legacy，CST 替代入口零生产调用**；a2x-store `ensure_env`（lib.rs:1268→1779-1806，env_rebuilds 计数）=恢复后首编辑整源重建具名靶点；av `front_door_admit`（driver.rs:1627-1640）直连 lexer；两条裸语法栈（av/session.rs:552-567、case-worker/syntax.rs）待 T-02 判定共享机制。T-01 余项：版本轴/预算域/写窗（927 部分可冻，928 写窗 pending D928）/原正式样本冻结。

- 2026-11-03，stage:new / revision:2 / outcome:pass（仅规划复核）/ next:work（待后续实施指令及实际写面就绪）。本轮按用户要求补实入口授权/typed资源态、同文lambda身份与真子体复用、首编辑禁止预热、恢复发布与预算适用性、923–926逐符号交接，详[复核及完整排期](reports/ace-927-929-plan-review-and-schedule-2026-11-03.md) PR-01..08。保留原AC/T编号、总步骤和0进度；新增V例均待使能执行，r1不是本r2实现证据。下列r1为初订历史。

- 2026-11-03 stage:new / revision:1 / outcome:pass（规划可交接）/ next:work（待后续实施指令与实际稳定接口）。静态核对调用点/恢复隐藏工作/退役范围/V→AC→T及923–926窗口；实现/正式测试/独立实施复审未执行，全部任务未勾。927/928及适用C/native腿依赖仍待真实交付。

## 10. 待澄清事项

**D928 依赖挂起（用户裁定，2026-11-04）**：T-02..T-11 公共实现链被 D928（PLAN-928 交付）阻塞；928 由另一活跃会话在途（本日开工，2/12）。用户裁定**挂起等 D928**，不空转、不接管、不把在途分支当D（未设期限）。**恢复条件**：master 出现 PLAN-928 merge 交付+归档提交（`docs/plans/archive/928-*.md` 且其 §9 merge pass）后 `/goal resume`——恢复入口动作：①worktree 基面 rebase 到 928 后 master 并读 D928 交接（其 §5.5/归档收据的 S 窗与稳定接口 SHA）；②按 §5.5"实际接口漂移使受影响证据STALE"刷新 T-01 冻结件（调用清单对 `cst/units.rs`/frontdoor 扩面做增量 diff）；③进入 T-02。检查点保留：实施树 `lang-929` @ `plan-929-dev`（无代码提交）、`plan-artifacts/plan-929/` 两冻结件、retention in-flight。

无待用户重新批准的语义/资源放宽。T-02需证明持久化语法/声明环境能在原预算内支持恢复后局部编辑，不能把整源重建挪到M轴、source hash未改就跳依赖，或用cold冒restart。失败先needs_replan。最终ARCH组合门、SQ/实例/CT与CA-D另有具名待立项owner，本包只交SYN与受影响机制重验，不自动扩包或宣称新M1完成。
