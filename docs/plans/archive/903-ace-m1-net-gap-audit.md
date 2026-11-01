---
plan_id: PLAN-903
status: archived
feature_name: ACE M1净缺口审计与能力/证据责任收口
author: [Codex]
created_at: 2026-11-01
updated_at: 2026-11-01
plan_revision: 1
supersedes_spec_components: [docs/specs/auto-test-case/ace-m1-conformance.md, docs/specs/auto-test-case/project.md]
new_spec_components: [docs/specs/auto-test-case/ace-m1-gap-audit.md]
touched_goals: [GOAL-002]
affects: [auto-test-case, auto-ac, auto-mir, auto-a2r]
depends_on: [PLAN-862, PLAN-863, PLAN-864, PLAN-865, PLAN-866, PLAN-871, PLAN-881, PLAN-889, PLAN-891, PLAN-892, PLAN-894, PLAN-901]
current_step: 10
total_steps: 10
---

# [PLAN-903] ACE M1净缺口审计与能力/证据责任收口

## 0. 变更摘要

对当前M1登记、已批准适用规则、已交付能力和正式消费者逐格对账，区分“能力确缺”“能力已有但证据/入口未接”“登记/判定口径落后”“infra”“尚未实证”。交付净缺口、能力/判定owner、已立项与否、关闭oracle和后继顺序。审计完成不等于M1通过；本包不修改生产、判定器、binding/expected/golden/证书或批准规则/分母。

依据[ACE总路线](../design/strategy/auto-native-backend-evolution.md)、[M1适用性裁定](reports/ace-m1-scope-decision-2026-10-28.md)、[历史收口表](reports/ace-m1-m2-closure-roadmap-2026-10-29.md)及current Spec。本轮仅立项，drafting r1、0/10，未建树/编译/测试。902不构成整包前置。

## 1. 目标

- G1：冻结当前登记、正式suite及已批准适用性；原103行、扩展行和表外正式义务都有去向，目录/包装/新recipe数不冒覆盖分母。
- G2：结合真实producer、受检入口、独立oracle/证书、正式收据及必要窄重验，形成能力/证据/infra/规则/未定位分类，不机械统计raw gap。
- G3：每个非关闭项有能力owner、证据/判定owner、计划或具名候选、前置/写面、下一动作和独立关闭oracle；未定位有边界，不冒已缺或已完成。
- G4：输出M1正式签章还缺什么及顺序，与893慢测、899工具/profile和902/AX-C源码线分工；不因审计PASS或902新producer绿就声明旧路径迁移完成。

非目标：签M1全绿、修生产/前端/诊断/资源/旧a2r、改比较器/NA/accepted-diff/分母、翻binding token、改源/expected/oracle/golden/现有证书、新A2R切默认、场景工具施工、13项慢测归因、全M2/全生态回归、native loader/AJ/block增量/AACE。发现问题只提具名后继，不顺手修复或预占号。

## 2. 架构方案

`binding(ns,id,path)+正式suite/规则 → 当前source/producer/接口/收据 → 有效证据或必要原入口窄重验 → 分类/owner/关闭oracle → 计划摘要及review后current Spec`。审计不成为日常执行前置，不造runner/全仓目录/影响图/第二裁判。

每格身份至少(ns,id,path,profile,axis)，保raw token；a2r emit/golden与compile-run认证分轴，source与typed/扩展profile分开；892/902证据并列，不能覆盖旧a2r身份。分类表不是语言执行结果。

## 3. 技术栈

仅语言仓，复用AutoDown/TestCase、binding.tsv/SCENARIOS.md/正式suite、原父侧adapter/受控worker、固定cargo t与原PowerShell入口、标准库静态表格/hash分析。无新增依赖、通用checker、生成器或跨仓改动，无生产代码任务。

## 4. 需求分析与背景调查

### 4.1 授权与当前基面

用户同意现在规划M1净缺口审计，并确认“本轮只做规划”。本轮允许本计划取号/文档、893修订及路线更新；不建树、不编译/测试/认证/慢采样、不改其他实施合同/WIP，不创建agent/chat或联系他方。main读取75223fc47，他方未提交改动保留。

891已archived，最终10/10、delivery eb6e45ef2在main；header旧5/10不表示未完成。889/892/894和前序source包均已归档，901已archived9/9。899 executing r2、2/7，lang-899@80d8bb694已接U901；902由用户确认开工、lang-902树已存在。893本轮r3、0/12。T-01重新冻结实际HEAD/owner，不把当前快照当后续窗口授信。

### 4.2 静态调查与证据限制

| 来源 | 已观察事实 / 使用限制 |
|---|---|
| docs/specs/overview.md、auto-test-case/{ace-m1-conformance,a2r-golden-certification}.md | 852底座、四路径、两轴及862 NA-4/后续绑定。早段103行/旧路径等有历史残留，不能据此丢后续义务 |
| binding.tsv SHA256 d9f05da057dfd658df09a7725aa391bc4c9e14993e13e9846724160915ff5d7a | 当前131行=91 corpus+40 scenario，均12列，四路径共524登记格。原103+862入口10+866常驻3+863 console15；新增登记不自动扩大原M1适用范围，须按批准合同分别核 |
| 2026-10-28 M1裁定与862 Spec | 已批准NA-1..3、R-A..F及后续NA-4，旧n/a、diff-*、认证token尚混；不能自行补新NA或接受差异 |
| 当前原源登记 | T03/T05 ac/av均ok，L/V14 native ok:P889-source，W5 native ok:P891-typed，R07/R08 ok:P881；旧路线称待实施是过期快照 |
| 残余线索 | B17/18/19 native及legacy B18 diff:pending；E01/E02等scenario-tooling、R02 av-alloc-fault、R03 av-821-gen；a2r B16/E03解析、R07 untyped、6项golden仍登记。不是本轮实测后的净能力缺口 |
| scripts/verify-ace-m1-852.ps1 | contract仍读attachments/852/cases.json等；acceptance窄ok词表/旧认证计数不等现行批准规则完整裁判。未运行脚本，不据静态问题宣称所有scope不可用 |
| scripts/test-groups.toml | case-ace-m1-{base,tr-native,collections-864,closures-865-source,entry-862,method-881,resident-866,native-collections-889,typed-native-891,console-863,text-source-894}真实注册；所有suite/check要核，不从binding或文件数猜完整义务 |
| 892/902新HIR A2R | 892 scalar已交付，902 record/text在途，新recipe只能并列记，不自动替旧a2r或改原M1分母 |

同时读取auto-mir/{record-method-lowering,native-typed-entry,native-collections-variants}.md、auto-a2x/project.md、auto-a2r/project.md、相关源/oracle/adapter。旧收据只证其冻结范围；旧raw按merge清理不要求复活，需要当前证据就原入口定向重验。

## 5. 详细设计

### 5.1 登记、适用与证据三者分开

冻结登记、suite/case/check、source/oracle/reference/证书hash、工具/profile/语义/SDK和Plan版本。原103与新增28行分来源/批准scope，表外正式suite也有去向；不能擅自纳入或排除原M1。corpus:R03和scenario:R03是两条义务。每格留raw token、规范状态候选、规则/范围、producer/recipe/入口、独立oracle/证据/条件、有效性、能力owner、判定owner、计划/下一步。

n/a只有批准规则才规范化；accepted-diff保原观察且不等ok，R-E解析不能转accepted。金样绿/认证缺按原批准两轴处理，不隐式certify/写证书。不明确的范围或规则保持待决定，不改原表。登记单元格数、适用M1格数、实际已执行格、clean行及缺口族数分列；同缺口影响多格记录映射，不把格数当Plan数。

### 5.2 净缺口分类

| 类别 | 必需证据与关闭者 |
|---|---|
| 已有且正式接线 | 同source/profile真实consumer+独立oracle和有效来源；仅typed指令或Spec标题不足 |
| 能力已有，证据/入口缺 | 指生产API/已有测试及具体缺adapter/观测/binding，由具名H候选关闭，不能只写“补证” |
| 真能力缺口 | 最小合法源/操作在权威前门或真实执行链拒/错，列独立期望/原因边界/S-N-R owner，不凭陈旧gap认定 |
| 登记/裁判落后 | 证据与批准规则充分但token/旧脚本未表达，列后继重绑定/判定修订范围，本包不施工 |
| infra/失效证据 | 缺工具/旧路径/错身份/条件/坏收据等保首红，不算语言负例成功、能力不存在或NA |
| 批准NA/accepted-diff | 精确规则ID/适用范围/重入条件和原观察，不擦历史或冒新执行PASS |
| 未定位/待决定 | 缺哪项观察、owner/下一最小探针或规则决定和影响范围，不冒已缺/已闭 |

一格可有多问题，不为互斥计数丢信息。没有有效证据可保持未实证，但须owner/具体下一动作；本包目标是完整诚实的账，不是把所有M1缺口修绿。

### 5.3 必要窄重验与原入口约束

先源/Spec/API/有效冻结证据，必要时才选原正式case或原测试。用专属树原cargo t相关case/职责组；组内原生导航只作定位，不冒完整职责组PASS。父侧独立oracle/源码/profile相符才能复用。B17..19任意非零不是正确负例，R02/R03仅exit70/读gen字段不是故障或stale保护，场景需实际trace/events/layout/恢复；最终值不能冒真函数增量/未改生产0/资源通过，缺轴明确pending。

旧852脚本材料路径/acceptance问题记infra/判定发现；不在本包修脚本或复制M1裁判。被破损scope无替代有效证据时保未实证并给后继；其他正式入口仅同源/oracle/适用范围等价且理由可审才补证。无界全套/全部13慢测不是默认审计门。编译准备与实际运行/条件/首红/重放/cleanup分列；显式认证和正式性能只有明确适用/授权窗口才跑，不把它们塞日常审计。

### 5.4 后继顺序与责任

建议默认①登记/证据及批准适用性归一的接线候选；②原负例诊断/资源故障代际的实证能力缺口；③旧a2r解析/未定型发射、金样与判定；④具备全部适用义务后独立M1正式验收。具体前置/独立写面优先，不制造全局语义串行。

每非关闭项具能力+判定owner、已立项/未立项、前置/最小输入/期望/入口、独立关闭oracle、写面和顺序。未立项项标规划责任=ACE路线/具名能力包，不虚构他方会话已接任务。902→AX-C为新Rust能力线，不自动替旧a2r收口；数值源码/多模块只保未编号候选。规则需新NA/accepted时保持pending并交用户决定，审计可诚实交付，M1不能因此判绿。

### 5.5 并行、唯一树与材料

| 面 | 903范围与交接 |
|---|---|
| P903文档/静态对账 | 可与902/899/893并行，只读正式基面；不改他方合同/树 |
| S/N/R生产 | source/verify/资源/MIR/native/旧Trans/新Rust provider只读，不跟进修复，不消费902WIP |
| H合同/固定组/脚本 | 原入口只读运行，不改binding/cert/expected/golden/注册。必要改接口/观测/判定另立或修订合同、取符号窗 |
| 893 | 13项慢测/未来接入归893，M1逐格审计归903，可引用正式证据，不互等整包或重复归因 |
| 899/902 | 899正式工具/profile变化使受影响证据stale；902只列在途新producer，二者非静态审计全局前置 |
| 运行窗口 | 窄重验与902认证/性能、899重建、893慢采样错峰，不停他方进程、不持H等机器窗 |

唯一拟实施树D:/autostack/.wt/lang-903/auto-lang，分支plan-903-dev，本轮未建。work/自检/独立final review同树顺序，无agent/chat。开工登记D:/autostack/plan-artifacts/plan-903/ owner/用途/在途；JSON/临时账/日志/输出/外置target全在该scope，正常target留树。摘要/数字/命令/hash/残余折叠计划，review后沉淀current Spec，不散落attachments/项目目录；merge核全部实际输出、删scope/树构建物，只有具名未解决问题可hold。不写legacy specs.json。

### 规范增量

| delta_id | add/modify/retire | docs/specs/... target | before/after rule | rationale | acceptance IDs |
|---|---|---|---|---|---|
| SD-01 | add | docs/specs/auto-test-case/ace-m1-gap-audit.md | 历史快照→revision绑定的当前适用性/能力与证据/owner/下一步 | 当前知识，不是运行目录 | AC-01/02/03/04 |
| SD-02 | modify | docs/specs/auto-test-case/ace-m1-conformance.md | 原103及片段→来源范围/双轴/审计导航和待验收边界 | 不改已批准规则/分母/producer | AC-01/02/05 |
| SD-03 | modify | docs/specs/auto-test-case/project.md | 过期统计→实际审计范围/来源/后继入口 | 不冒M1/M2完成 | AC-05/07 |

## 6. 测试设计

本包是审计，不新增测试框架；静态分析/原入口窄重验在后续work，本轮无执行证据。下列V由任务使可复核，不虚构已有专用checker。

| V | AC/invariant | 输入/原入口 → 独立可观察关闭oracle | owner_stage |
|---|---|---|---|
| V-01 | AC-01守恒 | 原binding/正式suite/规则；临时对账漏scenario/console、混两namespace R03控制 → 自检/独立复核必须拒不守恒，当前524登记格及表外义务有去向，适用范围另账 | T-01/02 work |
| V-02 | AC-02规则/双轴 | NA-1..4/R-A..F与旧token，把R-E误accepted或认证缺冒emit失败/认证PASS控制 → 依批准规则拒/待决定，保原观察，无隐式certify/证书改写 | T-02 work |
| V-03 | AC-03能力/接线 | 已交付T03/T05、L/V14、W5/方法行对旧gap，原源/正式consumer → 仅有效对应源证据关闭，typed/新producer不冒source/旧路径；未验证标未实证 | T-03/04 work |
| V-04 | AC-03证据/infra | B17..19/R02/R03/E-I场景/旧脚本路径；错诊断、任意非零、失效hash、缺观测控制 → 不冒语言/资源成功，给类别/owner/下一探针，重放绿保首红 | T-03/04 work |
| V-05 | AC-04后继 | 每非关闭格的owner/计划/前置/oracle/窗口；无owner或拿他包非目标充关闭 → 自检/review拒，去重数可回溯，不虚构授权接受 | T-05 work |
| V-06 | AC-05保护与声明 | 生产/源/oracle/expected/binding/cert前后hash/diff → 不变；903审计PASS与M1未签章分列，来源漂移stale，待裁定/未实证不变绿 | T-01..06 work |

对照控制只在scope复制/构造审计输入，不改共享源/工具；现有checker不支持时用独立解析/抽查及逐格来源核验，报告实际方法，不为一次审计造通用checker。窄运行选已有case-ace-m1相关组；发现必须改代码则needs_replan，不暗增施工。角色未指定人/模型，独立上下文final review不等自查，单一权威树顺序；无agent/chat。纯文档不跑无关cargo t all/docs_gen；已有窄运行保真实命令/条件/失败/收尾摘要。

## 7. 验收标准

| AC | owner_stage | 可执行关闭条件 |
|---|---|---|
| AC-01 | work | V-01：登记/正式suite/原103与扩展来源逐项有去向，适用依据与raw总量分开，缺重/跨namespace不冒通过 |
| AC-02 | work | V-02：批准NA/accepted/双轴可追溯，不扩规则/改分母/翻binding，超范围待裁定 |
| AC-03 | work | V-03/04：每格能力/证据/判定/infra/未实证证据链明确，不凭旧token认已有或缺失，不弱化负例/场景/资源oracle |
| AC-04 | work | V-05：非关闭项能力+判定owner、已立项否、前置/最小源/独立oracle/下一动作/顺序齐；未定位有边界，不虚构任务接受 |
| AC-05 | work | V-06：可核审计摘要/SD候选及并行后继边界；原生产/源/expected/golden/binding/cert不变，903完成与M1剩余门分列 |
| AC-06 | review | 同树独立上下文逐AC-01..05/V/SD/遗漏/延期/适用证据/健康核验，revision/SHA绑定；M1尚红不否定诚实审计，审计绿不等M1绿 |
| AC-07 | merge | 冻结必要文档/原入口证据范围核验，真实合入/SD来源读回/归档/guard和所有材料路径清理，10/10真实，未解决项具名交接 |

## 8. 执行步骤

10项=6 work+1 review+3 merge，0/10。902/893/899非整包前置；后续用户指定实施后T-01，必要窄运行另取实际窗口。

| T | owner_stage | 前置 | 实际工作/产物与验证 | AC | 状态 |
|---|---|---|---|---|---|
| T-01 | work | 后续实施授权 | 核唯一树/主线/owner，冻结当前实际行/所有suite/hash、原103扩展范围、工具/规则/收据，V-01/06；本轮静态数不冒新执行证据 | AC-01/05 | [x] 审计§1：基线 b1a4871aa/树 lang-903（含 auto-down 7c4d5cb + auto-web ec9c4ba detached 兄弟）；binding hash d9f05da0… 复核一致；131=91+40、原103+U10+H3+O15 守恒（脚本负控 3/3 拒）；11 固定组注册核；24 金样/10 证书/78 源计数；manifest 239 文件 hash 存 scope | 
| T-02 | work | T-01 | 逐格批准适用/规范状态候选/a2r双轴，原token不改；V-01/02，范围未知待决定，冻结窄重验清单/预算/窗 | AC-01/02 | [x] 审计§2/§3：每路径 131 格 token 普查（守恒和已验）；NA-1..4/R-A..F 现格位映射；bare n/a 54 格待裁（G-DEC-1）；38 格 ok+cert-gap 等价双轴；R-E 不转 accepted、cert 缺不冒 emit 失败控制实录 |
| T-03 | work | T-02 | 追producer/source/typed/profile/原oracle及证据身份，分类已有/接线/能力/infra/未实证；V-03/04，旧脚本问题单账 | AC-03 | [x] 审计§4/§5：七类 237/38/73/32/11/1/73/59 对平 524；19 族逐格归账（owner/oracle/下一动作）；旧 852 脚本三重红、attachments oracle 不入库、892 套件无固定组、B16 vs B17-19 登记不一致等发现单账 |
| T-04 | work | T-03；必要运行窗 | 必要原入口窄重验/同身份证据复用，V-03/04，保首红/条件/cleanup；无法验证保未实证和下一探针，不改生产/判定/源 | AC-03 | [x] 审计§10：census+3 负控（REJECT×3）；base 组当前 HEAD 窄重验 97/97 PASS（wall 365s；首红=组缺 auto-web 兄弟，补 detached 后绿，首红保留）；B18 avm 保未实证+下一探针具名；其余族冻结收据按身份复用 |
| T-05 | work | T-02..04 | 逐格净缺口族/双owner/计划或未立项/前置/oracle/顺序；902并列/AX-C后继/893分工；V-05 | AC-04/05 | [x] 审计§5/§6/§8：19 族双 owner/已立项与否（893 drafting r3、904 drafting r1 在册；scenario 工具/native loader/console/AV fault 等未立项具名）；顺序①裁定→重绑定②实证缺口③旧a2r④M1验收；893/899/902/904 分工边界 |
| T-06 | work | T-01..05 | hash/diff保护、AC/V/SD/摘要/残余/健康自检提交、树clean→execution_done，V-06；无raw散落 | AC-01..05 | [x] 提交 504e4acbf（3 spec 文件 +210/−8，树 clean）；diff 仅 docs/specs/**，binding/生产/金样/证书零改动、hash 前后一致；SD-01..03 就位；无 raw 散落（scope=D:/autostack/plan-artifacts/plan-903/） |
| T-07 | review | T-06冻结SHA/同树 | 独立重核全账、抽查真实producer/oracle与批准范围/首红/未实证；finding同树修复重绑，pass仅reviewed | AC-06 | [x] 独立上下文（general-purpose subagent，非实施者）终审 504e4acbf：8 组核查全 PASS——worktree/分支/clean、diff 仅 3 spec 文件、binding blob 两 commit 同一、**自研独立重算** 131/91/40/12列/U10/H3/O15 + av ok*=113/avm diff*=31/a2r golden=6 全符、census 负控 3/3 REJECT、results.json gate_passed+97/97 PASS 解析、5 项事实抽查（11 固定组+902 组、852 脚本 attachments 依赖+主检出无 cases.json+worktree 无 attachments、24 金样/10 证书/78 源、892 套件无固定组、裁定件 NA/R 全在）、SD-01..03 diff 核（无批准规则改动）、UTF-8 无损；AC-01..05 全 pass；唯一 finding P3（F-SCN-2 悬空引用）→ 同树修复 18bac254b + 全 F-ID 定义复核无悬空 |
| T-08 | merge | T-07 pass | 冻结文档/源身份及受影响原入口证据核验，纯审计不附全仓/慢测门，来源改变只重核受影响结论，实际合入 | AC-07 | [x] worktree clean+wt-guard clean；master b1a4871aa 未动（rebase no-op）；`git merge --ff-only plan-903-dev` → master=18bac254b 线性无 merge commit；canonical 读回：三 spec blob master≡delivery（a0df39b9/8cb67325/d8e5cdef）；纯文档不附全仓门（内容/UTF-8/引用检查=review 已核） |
| T-09 | merge | T-08 | SD-01..03沉淀/来源读回、archive delivered、路线/残余owner交接，不冒M1已绿，不写legacy specs.json | AC-07 | [x] SD-01（ace-m1-gap-audit.md 新增）/SD-02（conformance 登记节+导航节）/SD-03（project 852 节）全部随 delivery 18bac254b 落 master 并 blob 读回一致；残余 owner=G-DEC-1..5 交裁定（账本§6）+19 族后继（§5/§8）；未写任何 legacy specs.json；审计 PASS≠M1 绿在账本头部/§9 与本计划双处声明 |
| T-10 | merge | T-09 | 摘要/命令/数字/hash折叠计划，全输出/hold对账，fresh wt-guard clean后清树/分支/组/plan-artifacts并登记，真实10/10 | AC-07 | [x] 三树 fresh wt-guard 全 clean 且 plan-903-dev==master 已并入→worktree×3 移除、分支删（was 18bac254b）、组目录 rmdir；plan-artifacts/plan-903 整删（census脚本/manifest/README；摘要已折§8/§9）；登记表刷 archived/cleaned/无holds；输出对账：auto-case 临时目录 runner 自清、无外置 target/第二证据目录/散落输出；无具名 hold |

## 9. 复审记录

2026-11-01 stage:new | PLAN-903 r1 | outcome:pass（规划合同，非M1/实现验收） | 授权=本轮只规划。静态核131×12/91 corpus+40 scenario、四路径token、已交付889/891/894及902在途、旧852脚本依赖/口径、固定组/源码与893899窗口。无编译/测试/认证/性能/建树/agent/chat，无生产/binding/expected/证书/current Spec修改。903由scripts/new-plan.sh排他锁取号，下一号904，不预占后继。

next: 后续用户指定work T-01，先基面/适用冻结再逐格追证。所有V待执行，drafting 0/10；最终903 pass只证明完整诚实审计与责任，M1正式验收另项。

静态收尾：编号章0..10齐，T10/AC7/V6/SD3唯一且引用映射无未定义ID，本地链接解析通过，903唯一/下一号904；binding原hash复核未变。未修改生产/源/oracle/expected/证书/固定组或current Spec，893/903均未建实施树；899/902等他方树与WIP保留。

2026-11-01 stage:work | PLAN-903 r1 | outcome:pass（work 6/6，execution_done） | worktree D:/autostack/.wt/lang-903/auto-lang @ plan-903-dev | base b1a4871aa | code_commit 504e4acbf（3 spec 文件 +210/−8，树 clean） | 依赖：auto-down detached 7c4d5cb、auto-web detached ec9c4ba（组内兄弟） | task_ids T-01..T-06 | 证据：审计账本 docs/specs/auto-test-case/ace-m1-gap-audit.md（SD-01，§1 基面/§2 守恒普查+3负控/§3 规则与双轴/§4 七类对平524/§5 19族/§6 待决/§8 顺序）；census_binding.py 正常 exit0 全守恒、负控 drop/mix/short 全 REJECT exit1；`cargo t case-ace-m1-base` PASS 97/97 checks exit0 wall365s（auto-case 真实执行，结果 target/test-groups/run-hfc3cy9o/results.json；首红=组脚本未建 auto-web 兄弟→a2r-actor-tests 依赖 auto-web-contract 路径解析失败，补 detached 兄弟后绿，首红保留记 F-INFRA-1 注记）；binding hash 前后 d9f05da0… 一致；diff 仅 docs/specs/**；B18 avm 保未实证+下一探针具名（avm/run × src/B18.at，avm 腿未接线任何现行 suite，构造新 case 超静态审计授权） | blockers 无（G-DEC-1..5 为账内待决项交裁定，非本包阻塞） | next: review（T-07 独立复审，revision r1 绑 504e4acbf） | scope D:/autostack/plan-artifacts/plan-903/（README 登记+脚本+baseline-freeze manifest 239 hash）

关键数字（对平 524）：①已有接线 237；①+双轴 ok+cert-gap 等价 38；⑥批准NA 73（NA-4 已入表 30+NA-1/2/3 候选 43）；⑥accepted 32；④登记/裁判落后 11；⑤infra/失效 1（B18 avm）；②③缺口 73；⑦待决定 59（bare n/a 54+NA-3 扩展确认 5）。审计 PASS≠M1 绿：M1 剩余门=§8 顺序①..④。

stage: goal | plan_id: PLAN-903 | plan_revision: r1 | round: 1 | from: new | to: work | status: execution_done | outcome: pass | worktree: D:/autostack/.wt/lang-903/auto-lang | commit: 504e4acbf | next: review

2026-11-01 stage:review | review_scope: final | reviewer: 独立上下文 subagent（general-purpose，非实施会话；工件重建，未采信执行者摘要） | PLAN-903 r1 | outcome:pass（final） | worktree D:/autostack/.wt/lang-903/auto-lang @ plan-903-dev | reviewed_commit 504e4acbf（finding 修复后树头 18bac254b） | base b1a4871aa | 依赖 auto-down 7c4d5cb / auto-web ec9c4ba detached | spec_inputs: ace-m1-conformance/a2r-golden-certification/project/裁定件 2026-10-28/binding.tsv d9f05da0… | acceptance: AC-01..05 pass（独立重算+负控+results.json 解析+行级抽查）；AC-06 本复审 pass；AC-07 merge-owned | findings: R1-01 P3 F-SCN-2 悬空引用→已修 18bac254b 并全 ID 复核 | evidence: 审计账本 §1..§10 + census_binding.py（scope）+ run-hfc3cy9o/results.json（97/97）+ subagent 报告 8 项 PASS 全录 | pending_closeout_gates: merge 冻结文档/源身份核验、实际合入、SD 沉淀读回、归档、guard+清理（T-08..T-10） | next: merge

stage: goal | plan_id: PLAN-903 | plan_revision: r1 | round: 2 | from: work | to: review | status: reviewed | outcome: pass | worktree: D:/autostack/.wt/lang-903/auto-lang | commit: 18bac254b | next: merge

2026-11-01 stage:merge | PLAN-903 r1 | outcome:pass（delivered；cleaned 见下条） | worktree D:/autostack/.wt/lang-903/auto-lang @ plan-903-dev | delivery_commit 18bac254b（=reviewed 504e4acbf + 其 P3 finding 单行文档修复，docs-only descendant，实现/依赖零变） | base b1a4871aa | checkpoints: prepared=reviewed r1 pass 绑 504e4acbf+冻结 SD-01..03；landed=master `git merge --ff-only plan-903-dev`→18bac254b 线性（无 merge commit，master 未动无需 rebase）；canonical_verified=三 spec 路径 blob master≡delivery（ace-m1-gap-audit a0df39b9、conformance 8cb67325、project d8e5cdef；sha256 34e35f4b/2d67ea47/b737c0ee）；archived=docs/plans/archive/903-ace-m1-net-gap-audit.md status:archived 本提交 | batch-regression due-check：未到期（903%5=3≠0；.last-batch-regression 收据 2026-10-30T15:25:49Z，距今约33h<48h；902 系其前合入） | landing-not-deployment：纯文档落地，无生产二进制/web bundle 消费本 delta——无 stale 产物需重建（既有 852 脚本 infra 红状态不因本落地改变，账本 F-INFRA-1 在案） | 主检出他方 WIP 保留未动：docs/design*、docs/plans/reports/*、893/904 草案、test/fixtures/reader-import-budget/（非本计划，交其 owner 路由） | next: T-10 清理收尾


2026-11-01 stage:merge(cleaned) | PLAN-903 r1 | outcome:pass（10/10 真实） | checkpoints: cleaned=三 worktree fresh guard clean→移除（auto-lang/auto-down/auto-web）、分支 plan-903-dev 删除、组目录删除、plan-artifacts/plan-903 整删、登记表刷新（archived/cleaned/holds=[]）；证据折叠完成（census 守恒+负控3拒、base 组 97/97 PASS wall365s、blob 读回三 hash、binding d9f05da0… 前后一致，全文见 §8/§9 各收据） | 主检出状态：本计划产物全部入库（18bac254b+98ce5723a）；他方 WIP/未跟踪件保持原样 | 待裁/后继：G-DEC-1..5 交用户（账本§6），19 族后继（账本§5/§8）——非本计划未决，不冒 M1 已绿

stage: goal | plan_id: PLAN-903 | plan_revision: r1 | round: 3 | from: review | to: merge | status: archived | outcome: pass | worktree: (removed, group lang-903 cleaned) | commit: 98ce5723a | next: done


## 10. 待澄清事项

| D | owner/关闭任务 | 未闭合边界 |
|---|---|---|
| D-01 | T-01/02核原103/新增28及表外suite正式适用依据 | 不自行纳入/排除M1，保待决定；524仅登记总量 |
| D-02 | T-03/04核旧852脚本/材料问题及各格可用证据入口 | 不修脚本/弱化判定，不凭缺raw否认交付；无等价证据保持未实证并给后继 |
| D-03 | T-04/05核899身份变化和902/893运行窗 | 不消费WIP、不争载、不复用失效证据，不持H等机器窗 |
| D-04 | T-02/05需新增NA/accepted或M1范围时交用户决定 | 不自改规则/分母；可诚实交付审计，不能签M1或把pending转绿 |
