---
plan_id: PLAN-852
status: archived
feature_name: ACE M1共同子集四路径测试语料与Rust金样独立认证
plan_revision: 2
author: [Codex]
created_at: 2026-10-26
updated_at: 2026-10-26
current_step: 8
total_steps: 8
supersedes_spec_components:
  - docs/specs/auto-test-case/project.md
  - docs/specs/auto-test-case/architecture.md
  - docs/specs/auto-test-case/profile-v1.md
  - docs/specs/auto-test-case/profile-v2-native.md
new_spec_components:
  - docs/specs/auto-test-case/ace-m1-conformance.md
  - docs/specs/auto-test-case/a2r-golden-certification.md
touched_goals: []
affects: [auto-test-case, auto-ac, auto-av, auto-lang-core]
---

# [PLAN-852] ACE M1共同子集四路径测试语料与Rust金样独立认证

## 0. 变更摘要

r2按用户本次授权收敛为**先交付测试底座/标量四路径基线**；全集分母和缺口保留，不等待823～826能力补齐，也不以M1全集通过为本计划结项前提。由用户接下来启动work，本轮不代其实施。统一交接见[共同合同](reports/ace-852-baseline-and-capability-validation-2026-10-26.md)。

为用户确认的M1方向建立文件式测试分母：同一Auto源码覆盖共享Typed MIR原生路径AC、ABC解释路径AV、旧AutoVM及a2r。
**日常a2r仅生成Rust并比较已认证金样，绝不逐例再编译运行Rust。** 真实Rust行为核验是独立认证流程，完成后按金样/依赖身份复用。

本轮交付为执行合同和[设计包](attachments/852/README.md)：58份Auto源码草案、5份宽标量typed入口草案、43份独立Rust行为参考、43份AutoDown接线草案，及Dyn/CTE/资源/增量/HMR/认证场景。不是这些用例已运行或M1已通过的声明；真实a2r金样尚未生成。

## 1. 目标

本计划必交付出口：完整语料/支持矩阵、真实标量四路径基线、通用接线、独立金样认证与日常快照、当前可用cache/会话验证及下游交接。未实现的源码/后端/HMR由能力owner另计划补齐；基础层PASS与M1成熟度PASS分轴，不以新增能力未就绪阻塞基础层交付。

一份源码、固定独立oracle、四路径独立判决。普通返回、trap、静态诊断、infra分态。覆盖标量、文本、Auto `type`结构类型、集合、Option/Result/有限enum、有限闭包、Dyn及portable数据；位型、资源、缓存与热重载纳入分母。

日常：AC/AV/AutoVM真实执行，a2r实际emit并比较已认证金样。独立认证：实际生成Rust与手写参考各自编译运行，分别符合固定oracle，收据可复用。语料交付与语言成熟度分别报告；不能以SKIP、缩小选择集合或四个不同程序冒称M1全绿。

非目标：实现缺失语言特性/native lowering、完整FFI、任意状态迁移、AJ/OSR或发布器；不改旧引擎规范来迁就测试，不把手写Rust参考作为a2r金样，不自动更新金样。

## 2. 架构方案

```text
共同Auto源文件 + 独立行为oracle
  ├─ AC：源码 → HIR → 共享Typed MIR → Cranelift → native执行
  ├─ AV：源码 → HIR → Typed MIR → ABC → AV执行
  ├─ AutoVM：同一源字节 → 旧生产引擎 → 实际值/诊断
  └─ a2r日常：同一源字节 → 真实Rust文本 → 已认证金样bytes比较
                                                ↕ 有效认证身份
独立认证：真实生成Rust == 待认证金样 → 编译运行 ──┐
         独立手写Rust参考 → 编译运行 ──────────┴→ 固定行为oracle → 认证收据
```

判决沿用auto-test-case父侧，worker不接收expected。认证是显式独立入口，普通`auto-case run`不得隐藏调用Rust编译器。快照结果与认证有效性分轴；四路径成熟度同时要求文本一致和有效行为认证。

## 3. 技术栈

Rust；现有AutoDown pinned parser、`autolang.case/2`及suite/report；`crates/auto-test-case`严格合同和比较。
真实adapter owner为`crates/auto-lang-core/src/test_cases/{mod,avm,a2r,ac_mn,ac_av,stale}.rs`；CLI为`crates/auto-lang/src/bin/auto-case.rs`。SDK不增加对实验编译器/Cranelift的静态反向依赖。

拟新增正式目录`crates/auto-lang/test/cases/ace-m1/{src,oracle,reference,...}`、`scripts/verify-ace-m1-852.ps1`、独立认证入口/证书；精确新模块和schema由T-01冻结。设计资产在attachments/852，不被正式suite发现。

预定唯一实施树`D:/autostack/.wt/lang-852/auto-lang`，分支`codex/plan-852-dev`；尚未创建。按AGENTS解析跨仓依赖，不建链接。实施、独立review使用同一树、不同上下文。

## 4. 需求分析与背景调查

2026-10-26用户明确要求修订852及823～826后自行开始852。r2的实施范围已获认可，无需再请求方向确认；本轮只改主检出合同，不创建worktree、不启动work/代理。原设计包预期与全部M1分母保留，新增能力实际绑定由下游验证phase消费交付后的底座完成。

授权：用户要求为前述M1特性设计源码测试，并用统一文件框架覆盖AC/AV/AutoVM/a2r；随后明确Rust运行行为验证应独立进行、完成一次后日常仅比生成代码。本轮据此起草合同/具体设计代码，未授权改语言规范、自动更新金样、写其他工作树、创建代理。没有指定模型、预算或日历承诺。

从[Spec overview](../specs/overview.md)定位；[输入hash收据](attachments/852/planning-inputs.json)记录实际路径/版本。关键事实：

1. 773/788框架已有；a2r/emit只做生成/比较，不能冒称Rust运行通过。
2. 818共享MIR原生路径仅i32/bool；770/771/782更丰富的HIR直达native不能代替共享MIR支持。
3. 820/821已有record/文本/owned；822～824在途，825/826是装载/编码优化，不自动增加源码语法。
4. 828/821有函数增量、事务安装与旧快照；818重链接/run不等于native在线热重载。
5. 旧int宽度、let、borrow、闭包提升可能不同；旧通过不是新语义规范。
6. 788 ac profile只core/scalar，probe注册集也严格。非标量正式绑定依真实扩面，不能虚构recipe绕过；Dyn共同源API未冻结。

主检出有大量他方未提交修改，本轮只写新计划、其attachments和分配器.next-id，不整理/提交他方文件。

## 5. 详细设计

### 5.1 语料与共同入口

[cases.json](attachments/852/cases.json)/[matrix.tsv](attachments/852/matrix.tsv)固定B01～B20、T01～T08、R01～R10、L01～L07、V01～V07、K01～K06：43值例、9静态诊断、6trap。R类使用Auto `type`，不是新record关键字。各例有源hash、独立预期、状态和正式启用任务。

完整typed返回，不使用进程低8位；-21和128512是区分性正控。草案统一`fn main() int`，T-01核AutoVM/a2r真实入口返回；若需具名测试函数，统一改四路径、宿主调用，不只给一端改算法或加print。包装版本计入认证身份，不任意修补生成Rust。

List/Option/Result/enum/闭包拼写来自现有Auto示例，组合接受仍待T-01。宽标量typed参数/位型入口、Dyn/portable真实源API按实际已交付接口绑定；未交付项记录owner/接口需求和M1 gap，不先替语言owner冻结全部新API。typed fixture不能替最终源码证明。

诊断阶段/码逐后端绑定真实类型边界；普通trap不等于任意exit70，Rust编译失败/缺工具/timeout不满足Auto负例。

### 5.2 一次认证、日常快照

首次由真实a2r产生candidate，审阅后形成金样；认证先核actual全文件集等于金样，再编译运行actual和手写reference，两者各自符合固定oracle。不能直接编expected冒充actual，不能只编不运行。

日常只emit+bytes compare+认证身份核验，rustc/cargo/目标Rust进程spawn计数=0；不缓存本轮PASS。
认证键绑定Rust全文件内容、入口包装、固定行为oracle/独立参考文件hash、runtime/锁定依赖闭包、实际target/toolchain/生效flags。
**转译器变更但输出与上下文不变时复用认证，不因转译器版本号变化重编。** 生成器身份只留本轮来源审计。

金样内容变化使快照FAIL，只生成待审proposal；依赖或oracle漂移使受影响证书STALE，日常不自动认证。认证收据与金样可版本化保存、可跨检出审计，不能只藏在会被LRU回收的本机cache中。新增金样未认证，不能借“整个转译器曾认证过一次”通过。

负例在真实a2r前端拒绝时不需要生成可执行Rust金样；Rust编译失败不能当Auto拒绝。认证只证明冻结平台/依赖profile，非所有平台证明。

### 5.3 分母、并行与单写

逐行区分designed/source-bound/executed/unsupported/semantic-difference/infra，属于测试事实，不自创Plan状态。M1 required分母独立于selector；局部run不是整体通过。旧引擎缺能力如实登记owner，不自动新增永久双维护；改已议语义须带反例交用户决定，不为绿改expected。

T-01、旧引擎语法/入口探针、标量接线与认证工具可先做，不等822～826全体。集合/闭包/Dyn正式消费分别等D822/D823/D824（独立review、冻结门禁、合入、Spec/收据齐全）。825/826非语料反向前置，已交付版本须兼容。native扩面和在线loader由其独立owner交付。

本计划只改自身语料/测试层/最小append-only注册，不写MIR/ABC/AV实现，不复制他树WIP。contract/recipe registry/SDK adapter共有文件单写交接；拆分计划拥有归仓，按实际新owner适配，不恢复旧实现或复制runner。

### 5.4 资源与阶段验收

复用788/818 contained工具治理；认证器的Rust构建/运行预算、输出cap、子树清理在T-01测量冻结，不擅自拿旧70s上限承诺大型runtime冷编。首次认证耗时单列，普通编辑P95不含独立认证。

I01～I12观察生产计数、实际执行和对象代际。稳定会话代表编辑到父侧正确结果/诊断P95<1s；冷构建/任意规模不作此承诺。内存沿消费者各自量纲/预算，AV和native不虚构成同一额；类型扩面复测真实峰值和GC。重启exe不满足native在线HMR。

### 5.5 基础层结项与下游验证交接（r2）

**最低必需基线：B01=14、B04=42、B09=120及B02=-21四路径真实观测。** AC必须走共享MIR
原生链，AV走ABC；AutoVM实际执行；a2r真实emit与已认证金样比较，独立认证有实际Rust运行收据。
只编译/只有emit/缺证书不能满足此基线；负值不得截断。共同入口适配由T-01冻结，T-02实际接通。
此基线取自已交付标量能力，不等待822～826，不要求实现其新类型。

58个主要源码、5个typed入口及所有supplement必须都有能力/起点/证据或缺口记录；
未实现的Dyn源API、复合source/native、native在线装载不要求852实现，也不能用typed fixture假关闭源码行。
T-01对这些项产出明确owner/所需接口/关闭oracle；T-02只提升已绑定项并为其余保留设计资产，
不能把“先冻结全部新语言API”设为底座结项前提。

T-05至少消费已交付标量会话与cache路径验证fresh/hit/重启正确性和一个真实body编辑的结构观测；
对未交付复合/native在线接口保留M1缺口，不构造假的loader或以重启exe冒称HMR。
已有底座适用的身份/取消/工具治理缺陷仍是852阻断问题，不能泛化为范围外gap。

`acceptance`验852基础层合同；独立M1完整性汇总保留required全集，任何必需未支持/未认证/未绑定
都保持M1非通过。底座退出0不表示M1全集通过；输出/报告必须分别给出两种结论。

D852=本revision底座独立review、冻结门禁、实际合入、Spec/merge收据齐全。交付包包含runner/API/recipe/
证书schema及脚本版本、固定source/oracle身份、支持/缺口矩阵和新增binding的接入说明。
823～826仅其T-11等D852，原实现不等；852也不消费四计划WIP。共享测试合同由852 owner交接，
下游仅在本目录追加binding/证据，改registry/SDK adapter须拿测试窗口，关联核心合同同时拿核心窗口。
具体case映射和无环任务顺序见[共同合同](reports/ace-852-baseline-and-capability-validation-2026-10-26.md)。

本次没有删除任何M1 required特性；改变的是852基础设施交付出口，完整语言/native/HMR实现和最终M1验收
另由能力包负责。823/824类型扩面、825/826性能/格式验证不反向扩大852实施范围。

### 规范增量

| delta_id | add/modify/retire | docs/specs/... target | before/after rule | rationale | acceptance IDs |
|---|---|---|---|---|---|
| SD-01 | add | docs/specs/auto-test-case/ace-m1-conformance.md | 无统一分母→同源四路径/差异/真实源码边界 | 防不同程序冒称一致 | AC-01/02/05 |
| SD-02 | add | docs/specs/auto-test-case/a2r-golden-certification.md | emit不证明运行→独立认证、日常只快照 | 用户速度裁定 | AC-03/04 |
| SD-03 | modify | docs/specs/auto-test-case/architecture.md | 补认证拓扑/身份/父侧判决/工具治理 | 不隐藏重编译或新增第二runner | AC-03/04/07 |
| SD-04 | modify | docs/specs/auto-test-case/project.md | 导航新语料、认证与缺口 | 现状准确 | AC-05/08 |
| SD-05 | modify | docs/specs/auto-test-case/profile-v1.md、profile-v2-native.md | 保持旧接受/拒绝集；新增合同显式版本化/导航 | 不宽松忽略未知字段 | AC-04/06 |

### 规范增量补记（R1 修复后，merge 执行的具体沉淀文本）

- **SD-01/02（add，工作树已备稿）**：`docs/specs/auto-test-case/ace-m1-conformance.md`
  与 `a2r-golden-certification.md` 全文入正式 Spec；内容=R1 修复后事实（严格收据验证、
  toolchain 构建期轴日常核查、wrapper 单源 `oracle/wrapper-main-v1.rs`、双管并发排水
  与溢出杀树、G02 周期CIM ancestry采样（非无损进程创建计数），worker维护观测单列）。
- **SD-03（modify，architecture.md 追加节"ace-m1 认证拓扑"）**：在现有 adapter 拓扑
  节后追加：认证器为独立 bin（`ace-certify`，`auto-lang-core::test_cases::ace_certify`），
  不属 suite runner 派发面；日常 face（snapshot scope）=emit+byte-compare+严格收据
  验证+周期ancestry采样（非无损计数），认证 face 仅显式入口；报告新增 `build-rustc-identity`
  toolchain 轴（build.rs 捕获，零 spawn）。
- **SD-04（modify，project.md 导航行）**：模块清单追加 ace-m1 语料
  （`crates/auto-lang/test/cases/ace-m1/`：binding.tsv 分母登记/58主源+43参考+5typed/base 套件/
  证书 registry）与 verify 脚本六主 scope及 receipts诊断面、`ace-certify` 入口、证书 schema `ace.m1.cert/1`。
- **SD-05（modify，profile-v1/v2 各加导航注）**：v1/v2 合同不变（接受/拒绝集保持，
  未知字段仍拒）；各加一行导航：外部资产身份合同（收据/wrapper/binding）以
  `ace.m1.cert/1`、`ace.m1.binding/1` 独立版本化，不进 case/suite 解码器。
- **touched_goals 为空的说明**：852 是测试底座交付，不改写 roadmap 目标条目本身；
  M1 语言/native 目标的推进归属 822～826 与 native 扩面/在线 loader 能力包，852 只
  为其提供分母登记与验证入口（D852 交接），故不登记 touched_goals。

## 6. 测试设计

[源码/参考](attachments/852/README.md)与[补充合同](attachments/852/SCENARIOS.md)为固定分母：W01～05、D01～09、E01～03、C01～02、R01～06、I01～12、G01～08。supplement各行在T-01 binding表绑定AC、owner_stage=work、实际入口/状态、独立oracle；不能只放文档算已运行。

| AC/不变量 | 实际通道 | 区分性输入/oracle | 边界 |
|---|---|---|---|
| AC-01同源 | Auto源→四路径 | source hash同一；128512/-21不截断；只改一端source反控非绿 | 允许宿主包装，不改算法 |
| AC-02语义 | 值/静态诊断/trap | B04/B20/B16成组；固定expected；旧i64/let/borrow差异保留 | 不把旧行为自动变规范 |
| AC-03认证 | 实际Rust/手写reference各自执行 | G01/06/07，actual全文件集、typed结果、输出、工具身份 | 不是全平台证明 |
| AC-04速度 | 日常emit/比较/证书核验 | G02/03零编译运行spawn；G04/05非绿 | 独立认证单列 |
| AC-05分母 | selector/required与能力矩阵 | 零选择、缺行、仅emit或skip必需项不能M1绿 | 局部诊断非整体 |
| AC-06协议 | parser/registry/identity/stale | 未知字段/recipe、坏hash/收据、缺工具、错实际集合 | 不宽松解析 |
| AC-07资源 | 工具子树/真实会话/GC | R/I反控、截止杀树、未改工作=0、旧代最后释放 | native接口缺失保持gap |

未指定人员或模型，遵循同树独立上下文review，实施停止写入。本轮不创建代理。复审逐例核oracle，不能只数43个Rust文件。

## 7. 验收标准

| ID | owner_stage | 可观察验收 |
|---|---|---|
| AC-01 | work | 完整58主要源码+5typed入口及supplement清单/身份/能力起点可追踪；B01/B02/B04/B09四路径基线实际通过；其余明确证据或M1 gap/owner，不要求尚未交付能力全绿 |
| AC-02 | work | 43值例/9诊断/6trap预期逐例审阅；已绑定项按真实阶段/typed值判决；未绑定/旧语义差异保留owner，不擅改预期或冒认typed为源码 |
| AC-03 | work | B01/B02/B04/B09真实a2r金样独立编译运行认证，actual/reference各自匹配固定oracle；其他已支持代表项按实际认证，无支持者保留gap，不等待集合/闭包/Dyn源码或native扩面 |
| AC-04 | work | 日常仅emit/快照；G02/03零编译运行spawn；内容/依赖漂移只影响相关证书；G04～08负控非绿 |
| AC-05 | work | 852基础层结项与M1全集完整性结论分开；required全集不因selector/未支持被删除，未认证/差异/infra阻止M1绿但合法范围外gap不阻塞底座交付 |
| AC-06 | work | v1/v2/recipe/stale现状兼容；新增协议按T-01显式决定，有正反例 |
| AC-07 | work | 已交付标量会话/cache至少有fresh/hit/重启/body编辑真实观测，认证工具有期限/输出cap/杀树负控；未交付复合及native在线接口有精确owner/关闭oracle，不用重启冒充HMR |
| AC-08 | review | 独立逐AC/SD/分母审查，绑定revision/原始证据，遗漏/延后扫描；merge-owned门禁pending明确 |
| AC-09 | merge | 冻结提交按实际触面门禁；Spec/ledger/归档/guard清理有收据；不借本计划宣称M1已绿 |

## 8. 执行步骤

| ID | owner_stage | 依赖、触面和结果 | 验证/关联AC |
|---|---|---|---|
| T-01 | work | [x] 真语法/入口探针；冻结最低标量基线与通用报告/认证合同；全分母分类已有/未绑定/差异，未交付源API只登记owner/需求/关闭oracle；预算及版本化策略落contract-decision，不等823～826 | contract scope和实际入口正反控；AC-01/02/05/06。[✅ 已完成] 探针四路径基线全真值（B01=14/B02=-21/B04=42/B09=120，exit含符号/result十进制/emit bytes）；B14/B16负例通道实测；认证wrapper实测（rustc 0.12–0.19s/例）+认证键/复用/预算冻结；binding.tsv 103行+verify-ace-m1-852.ps1 contract scope PASS（篡改反控FAIL）；决策见[contract-decision](attachments/852/contract-decision.md)，实现提交38c651163 |
| T-02 | work | [x] T-01后接通B01/B02/B04/B09同源四路径，审阅58源码/43参考/5typed设计；提升已绑定项，其余保留设计和明确M1 gap；不实现新语言、复合native或loader | source scope，完整typed返回及真实语言诊断/infra分类；AC-01/02/05。[✅ 已完成] 全58例观测探针；正式语料 crates/auto-lang/test/cases/ace-m1（58源+43参考byte-identical）；base套件34例67检查全绿（gate 0）+cargo测试2绿+负例oracle红；差异行如实入binding（B08=10/B11=-7 vs oracle7、T01/T02/V03/R07旧运行时、K02/K05旧语法、T04=false等）；a2r emit检查按任务边界归T-03；提交5e6e5ceaa |
| T-03 | work | [x] T-02后真实a2r生成审阅金样，建立普通emit快照与认证身份核查；通用增量binding按已交付能力接线，不等下游新family | snapshot scope，G02/03零编译运行spawn；AC-04/05/06。[✅ 已完成] 24金样冻结（标量10+复合14，byte-exact a2r输出）；a2r检查26绿/全套件91绿；verify snapshot scope PASS（进程spawn=0结构守卫+G04金样漂移红+证书两轴报告）；复合伪Rust（List<int>/int不可编译）→认证gap登记owner=a2r复合lowering；提交7fcebf511 【R1修复重关闭：F-852-R1-01快照严格收据验证（oracle绑定登记值/flags/deps/observed/文件轴+build期toolchain轴；四篡改变体复放双红、对照绿）+G02真实进程计数（CIM ancestry，认证面0 spawn，worker维护构建单列）+snapshot零构建】 【R2修复重关闭：快照严格收据=类型化精确键集schema+default-toolchain轴比对；采样描述准确化】 |
| T-04 | work | [x] T-03后显式独立认证入口，contained编译运行基线actual/reference，保存证书；仅实际支持代表项继续认证，未支持复合项有gap即可 | certify独立scope，G01/04～08；AC-03/04/06/07。[✅ 已完成] ace-certify入口（挂起态Jobcontained rustc，120s/30s墙/1MB cap/scratch清理）；10证书入库（actual==golden+wrapper与手写reference各符合固定oracle）；G08二次运行零编译复用；G04篡改金样FAIL且证书不动；G05证书STALE轴核查；提交40b607727 【R1修复重关闭：严格validate_receipt（结构破损=REJECTED含--force不覆盖；文件漂移=STALE重认证）+完整身份（rustc -vV构建期轴/wrapper单源入库include_str/冻结flags+deps）+流治理双独立读线程+溢出杀树+新格式10证书-f真入库（旧证书曾被*.json ignore静默拦截未入库）】 【R2修复重关闭：R2-A身份绑定实际启动编译器（contained -vV，nightly复放记1.90-nightly，shim测试证伪构建常量冒充）+R2-B错型/未知字段逐层拒绝（双消费者）+R2-C清理守卫覆盖身份捕获】 |
| T-05 | work | [x] T-02及T-04后绑定已交付标量cache/会话的fresh/hit/重启/body编辑和工具资源负控；复合/在线native缺口登记owner；交付下游T-11所需binding入口及D852清单 | lifecycle scope，原始计数/身份/结果/治理证据；AC-02/05/07。[✅ 已完成] session-lifecycle.case.ad（B09同源：av warm120/hot120/体编辑720/跨进程编辑持久720+cache_hit_abc；mn fresh120/edit+重启720+cache_hit_coff=2）；认证器墙钟杀树负控真实rustc挂起程序2.17s绿；I01/02/05/06 scalar-subset证据+保留缺口；I08/复合native gap登记；提交b19c6b2d8 【R1修复重关闭：墙钟/流治理负控改#[ignore]显式入口由certify scope驱动；lifecycle scope零编译】 【R2修复重关闭：维持R1证据（会话/流治理未变，R2确认）】 |
| T-06 | work | [x] T-03～05后验852最低基线/框架/证书及完整gap清单，分别报告底座验收与M1完整性；SD/交接说明齐全，不以未交付能力阻塞底座，也不将其算M1 PASS | acceptance scope与定向新增测试家族；AC-01～07；无整档/隐式认证。[✅ 已完成] acceptance scope编排五scope全绿（冻结头4b3c9b254）；两轴判定：底座PASS / M1 75/252语料单元绿+29/40场景行开放+owner阻断清单（822=24、823=6、a2r复合lowering、native扩面38、语义差异86）；基线四证书在场；新Spec草案2份入worktree；提交77176a7f2+4b3c9b254 【R1修复重关闭：acceptance不再调用certify（缺/坏证书=显式缺口红，复放验证零编译）；SD-03/04/05具体沉淀文本+touched_goals理由；rustfmt/diff-check清零；修复提交a19e5597f】 【R2修复重关闭：SD计数修正58/43/5+采样措辞准确化+二次失败诊断入档】 |
| T-07 | review | [x] 同树独立上下文逐AC/SD/固定oracle核验，不自修，出revision-bound结论（R5 pass；冻结提交与证据见复审记录） | AC-08，merge-owned裸t及触面档pending |
| T-08 | merge | [x] 最终审查后在冻结提交跑实际Rust触面门禁，Spec/ledger/归档/guard清理 | AC-09；纯设计资产不跑cargo t/docs_gen。[✅ 已完成] 裸cargo t 3190/3190绿（触面=纯test-cases面）；SD-01..05落地+ledger v95；ff-only落地4a81d2ebc；归档+guard清理见9节收据 |

拟新脚本`powershell -File scripts/verify-ace-m1-852.ps1 -Scope contract|source|snapshot|certify|lifecycle|acceptance`各Scope由以上任务建立，目前不存在、未运行；certify始终独立显式操作，acceptance只核已有证据不暗中重认证。
R1独立复审 needs_fix：T-01/T-02保留完成，T-03～T-06因F-852-R1-01～04重开；历史完成声明保留作审计，不代表当前可合入。标量与旧引擎基线不等822～826全体；新增family未就绪保留M1 gap，不阻塞852基础层交付。关闭测试计划不关闭M1缺口；M1全绿需要所有required真实满足。

2026-10-26 stage:work | PLAN-852 r2 | work启动 | worktree: D:/autostack/.wt/lang-852/auto-lang | branch: codex/plan-852-dev | base_commit: 2e09547ee | 主检出crates/test无已跟踪WIP（仅未跟踪Cargo.lock与fixtures产物） | next: T-01真语法/入口探针与合同冻结。

## 9. 复审记录

**PLAN-852:r2 合并收据（stage: merge | outcome: pass）**

- `prepared`：reviewed 基线 826e3472c（R5 pass，实现冻结 d3b4fe90e）；SD-01..05 全落地（新规范 2 + architecture/project/profile-v1/v2 导航增量）；projection 目标 .autoos/specs.json（designs2/tests1/reviews1/reports1，最低四类全满足，条目带 file+tags）；交付提交 6f94eaa23（reviewed 树的纯文档/投影后代，实现与依赖未变）。
- `门禁（merge-owned，冻结基线执行）`：裸 `cargo t` 首跑 1 FAIL+14 TIMEOUT（worker 冷 release 构建+nextest 并行 cargo 锁竞争；失败用例单跑 27.5s PASS），预热各 worker+补组内 auto-man 兄弟仓（schema_drift 自述需仓库布局）后**全跑 3190/3190 passed / 0 failed**（673 skipped=feature 门控面）；触面=纯 test-cases 面，无 VM/trans/book/ui_gen/aavm 生产路径，无需额外触面档；门禁顺带再生的 7 个 plan796/818 turnaround 指标报告已还原（非本计划交付物）。
- `landed`：分叉=master 侧 3 个他计划 docs 提交；按会话既有同步模式一次 merge master 入分支（不重写已审提交，无需 range-diff），`git merge --ff-only codex/plan-852-dev` → **master tip = 4a81d2ebc = 分支 tip**（landing 未创建 merge 提交）；主检出冒烟：新规范/语料/脚本在场 + ledger v95 + contract scope PASS。
- `ledger_refreshed`：**Regime B 守卫离线投影**（HTTP store 探测无监听）；字节级备份 .autoos/specs.json.bak-p852-merge；原文件含历史不规则序列化（纯往返不纯），改用**文本级外科插入**：+117/-1 行（5 条目+version 94→95），未触条目 JSON 语义逐节相等+六节 schema+回读核验；channel 如实记为 guarded offline projection。
- `archived`：git mv → docs/plans/archive/852-...，status: archived，current_step 8/8；计划+attachments（review-r1..r5 证据链）+设计包均已在 master 版本控制内。
- `cleaned`：wt-guard 四仓全 clean（auto-lang/auto-ui/auto-down/auto-man，reparse 扫描）→ `git worktree remove --force`（auto-lang 因未跟踪 probe-t01 本地 scratch 与 target/ 构建产物拒绝普通移除；持久证据=master 上 review-r1..r5 报告与本收据，guard 通过后按规程 --force）→ 分支 codex/plan-852-dev 已删（was 4a81d2ebc）→ 组目录 D:/autostack/.wt/lang-852 已删；`git worktree list` 无 852 残留。
- `批量回归到期判定`：收据 docs/plans/.last-batch-regression.json = last_covered_plan_id 850 @ 2026-10-26T00:02Z（<48h）；其后落地计划 852/853/854/822R12 均非 5 的倍数 → **不到期**，不触发 /auto-plan:regress。
- `生产工件观察`：auto-lang 本工作流无部署面（无 daemon/web bundle/常驻进程消费本仓二进制）；verify/auto-case/ace-certify 为开发工具，release 产物未重建——无运行中生产进程受影响。
- **最终计数**：任务 8/8（T-01..T-08 全 [x]）；plan_revision 2；completion_kind: delivered；M1 完整性明确未主张（75/252 语料路径单元绿，缺口按 owner 在 binding.tsv 登记，归 822~826/能力包）。

2026-10-26 stage:review | review_scope:final | reviewer_identity:R1～R4同一独立复审上下文，未参与work，无代理 | PLAN-852 r2 | outcome:pass (R5) | worktree_path:D:/autostack/.wt/lang-852/auto-lang | branch:codex/plan-852-dev | reviewed_commit:826e3472c04b4e20b5db7fea5e8c109d694eb2db | implementation_commit:d3b4fe90e（本次HEAD与其852代码/脚本/Spec草案无差异） | base_commit:0cc4e2f7e808ed4f3396db954546cc4e05d682b7 | dependency_revisions:auto-ui b4fe0000e56cddaf7d3cf70e348431f2258aacde / auto-down c3464d0a0d36e4f8a7988b0c0c26448fc52361b4 | spec_inputs/frozen_delta:[最终规范增量](attachments/852/review-r5/spec-delta-reviewed.md)及同目录冻结草案/hash | acceptance_results:AC01～08 pass；AC09 merge-owned pending | findings:F-852-R4-01及前序所有in-contract findings按R1～R5证据链闭合，无未决阻断 | evidence:[R5复审报告](attachments/852/review-r5/report.md)、R4原始数组反例双拒、独立生产validator直达105错型/嵌套反控全拒+10有效对照绿、Rust精确定向schema1绿、正常snapshot97检查/10证书绿、fmt/diff-check绿；未受影响oracle/identity/stream/cleanup/lifecycle证据按修订/依赖/配置/输入未变复用 | pending_closeout_gates:merge在冻结提交跑裸cargo t并按最终实际触面决定必要档；tf仅主检出单实例批量档到期执行；本次无整档 | next:T-08 merge执行门禁、Spec/ledger/index沉淀、归档与guard清理。reviewed不等于可跳门禁合入，更不等于M1全集通过。T-07关闭/current_step7；plan_revision2不变，保留所有历史失败/修复声明作审计。

2026-10-26 stage:work | PLAN-852 r2 | needs_fix R4修复闭环（五送审）| worktree: D:/autostack/.wt/lang-852/auto-lang | branch: codex/plan-852-dev | code: R4=d3b4fe90e(冻结头，终验六scope绿) | F-852-R4-01闭合（全字段类型谓词清单而非单例补丁）：case/module/identity.actual/golden 补 -is [string] 标量守卫（阻断 PS 集合左操作数过滤语义）；验证块提取为共享 Invoke-ReceiptValidation + 新 receipts 直达面（无suite跑/零spawn，矩阵用）；双消费者同矩阵=22标量字段×5错型109项两器同拒（唯一受纳=created_at正整数，合同合法且Rust同受）；R4精确反例（case/module单元素数组）经完整日常快照REJECTED+有效对照绿（probe-t01/r4-*）；诊断回应"为何类型审计止于大小写/个别字段"：R3 修复按 R3 报告列举的边界逐项补而非按字段清单全量盘点——本轮改为字段清单驱动（scalar_paths×wrong_types 程序化矩阵），并以 assert 锚定替换消除前两轮"替换静默落空"（R3 SD-02 段与注释本轮断言式修正并 grep 复核）| 其余证据保留 | merge门禁pending归merge | next: T-07五审

2026-10-26 stage:review | review_scope:final | reviewer_identity:R1～R3同一独立复审上下文，未参与work，无代理 | PLAN-852 r2 | outcome:needs_fix (R4) | worktree_path:D:/autostack/.wt/lang-852/auto-lang | branch:codex/plan-852-dev | reviewed_commit:0cc4e2f7e808ed4f3396db954546cc4e05d682b7 | base_commit:545b727dd17b6c19232a744b01e231ad5f83f8ef（修复87ec33640） | dependency_revisions:auto-ui b4fe0000e56cddaf7d3cf70e348431f2258aacde / auto-down c3464d0a0d36e4f8a7988b0c0c26448fc52361b4 | spec_inputs/frozen_delta/acceptance_results:[R4报告](attachments/852/review-r4/report.md) | confirmed_closure:R3 produced_by未知字段双消费者均拒绝；schema定向1测试绿/格式绿 | findings:F-852-R4-01(P2，同一严格类型不变量残余：case/module改单元素数组，Rust拒绝exit1但日常snapshot仍10证书STRICTLY valid/exit0；-cne集合过滤不等于标量类型检查)；SD-02资源段未改/BUILD_TOOLCHAIN_IDENTITY注释残留旧身份描述 | acceptance_results:AC01/02/03/05/07沿用pass，AC04/08 partial、AC06 fail、AC09 merge-owned pending | pending_closeout_gates:最终pass后裸cargo t+实际触面档，tf仅主检出批量档；本轮无整档 | next:work原树补全双消费者逐字段类型审计/反控及SD确切残留文字后提交；仅T-03/T-06重开/current_step4，保留其他闭合证据，plan_revision不变，不合并归档清理。

2026-10-26 stage:work | PLAN-852 r2 | needs_fix R3修复闭环（四送审）| worktree: D:/autostack/.wt/lang-852/auto-lang | branch: codex/plan-852-dev | code: R3=87ec33640(冻结头，终验六scope绿) | F-852-R3-01闭合：日常快照验证器补 produced_by 精确键集+字符串类型（R3反例 produced_by.unknown_field 复放REJECTED）；同不变量边界全修——键比较全改大小写敏感，hash 断言 -cnotmatch 小写hex，created_at 无符号（负值拒绝），文件轴比较 -cne；复放矩阵=produced_by unknown/missing/null/根键大小写Schema/大写hash/负created_at 六控件全红+有效对照绿（probe-t01/r3-*，工作树本地）；Rust 侧矩阵扩 produced_by 四变体（invalid_receipts 1测试绿）；ace_certify.rs 头注释与 SD-02 资源段残留"默认取构建期身份"旧描述对齐实际（身份恒为实际启动编译器 contained 实测）| 其余：T-01/02/04/05 与 R2 证据保留未动 | merge门禁pending归merge | next: T-07四审

2026-10-26 stage:review | review_scope:final | reviewer_identity:R1/R2同一独立复审上下文，未参与work，无代理 | PLAN-852 r2 | outcome:needs_fix (R3) | worktree_path:D:/autostack/.wt/lang-852/auto-lang | branch:codex/plan-852-dev | reviewed_commit:545b727dd17b6c19232a744b01e231ad5f83f8ef | base_commit:17a959ba097dd7fd247171f7ce43568c75b937ba（R2修复对比，实质修复81b474145） | dependency_revisions:auto-ui b4fe0000e56cddaf7d3cf70e348431f2258aacde / auto-down c3464d0a0d36e4f8a7988b0c0c26448fc52361b4 | spec_inputs/frozen_delta/acceptance_results:[R3报告](attachments/852/review-r3/report.md) | confirmed_closures:R2原始反例全闭合（nightly身份、flags/deps/根未知字段、scratch清理、全改动Rust格式）；精确定向schema/identity-cleanup各1测试绿；旧stream/隐式认证/oracle审阅证据复用 | findings:F-852-R3-01(P2，R2-B共享严格schema仍未闭合：produced_by新增未知嵌套字段，Rust拒绝exit1但PS5.1 snapshot称10证书严格有效exit0；同类类型/键集一致性须机制闭合)；SD-02资源段与代码注释仍有旧默认identity描述须对齐 | acceptance_results:AC01/02/03/05/07 pass、AC04/08 partial、AC06 fail、AC09 pending | pending_closeout_gates:最终review pass后merge-owned裸cargo t+实际触面档；本轮无整档 | next:work修复日常消费者与SD/comments并提交，重放受影响不变量；T-03/T-06重开，T-01/T-02/T-04/T-05保留完成，current_step=4；plan_revision不变，不合入/归档/清理。

2026-10-26 stage:work | PLAN-852 r2 | needs_fix R2修复闭环（三送审）+二次失败诊断 | worktree: D:/autostack/.wt/lang-852/auto-lang | branch: codex/plan-852-dev | code: R1=a19e5597f, R2=81b474145(冻结头，终验六scope绿) | findings闭合：R2-A身份绑定实际启动的编译器（contained -vV 捕获，CLI>env>PATH 单一规则；identity_source+default_identity_at_certify 入证书；nightly 精确复放=RUSTC=nightly 认证书 identity=1.90.0-nightly、default=stable；exe-shim 测试证伪构建期常量冒充；复用校验对 identity 漂移→STALE 不复用）/R2-B类型化精确键集 schema（未知字段/缺失/错型元素逐层拒绝，无 filter_map 宽松；Rust+PS 双消费者一致；flags null 项、external [null,17]、未知根字段、字符串 exit 四变体 PS 复放红+Rust 内存矩阵绿）/R2-C清理守卫覆盖身份捕获（缺失 --rustc 复放 exit2 且 --scratch 目录确认移除）/stale.rs+全改动集 rustfmt 清零、SD-04 计数修正 58主源+43参考+5typed、进程观察措辞改采样描述 | 二次失败诊断：见下段 | merge门禁pending归merge | next: T-07三审

**二次失败诊断（R1→R2 同类机制复发根因）**：两次失败共享同一模式——"便利默认"侵蚀证据合同。R2-A 机理：R1 为满足日常零 spawn 把 identity 默认取构建期常量，将"运行时实际启动的编译器"与"构建期工具"两个语义实体合并为一个身份轴——设计层合并而非测试缺失，故 R1 新增校验在错误前提下自洽通过；正确分解=显式认证面实测真实身份（允许 spawn），日常面仅比对 default 轴（零 spawn）。R2-B 同型：filter_map(as_str) 宽松解析惯例把"错型元素"默认成"无元素"，与"未知即拒"规约冲突。覆盖调整（已落地）：(1) 身份类不变量必须有"证据主体=被记录主体"反例（nightly 双工具链复放+shim 改写 -vV 复放）；(2) 严格 schema 以错型/未知输入矩阵测试（null/number/string-exit/unknown-field，双消费者）；(3) 不得以性能/便利合并语义不同的身份轴。

2026-10-26 stage:review | review_scope:final | reviewer_identity:R1同一独立复审上下文，未参与work，无代理 | PLAN-852 r2 | outcome:needs_fix (R2) | worktree_path:D:/autostack/.wt/lang-852/auto-lang | branch:codex/plan-852-dev | reviewed_commit:17a959ba097dd7fd247171f7ce43568c75b937ba | base_commit:4b3c9b254487f81322670fb62cc6b7953718cadb（修复对比；原merge-base 6c02d13628f7c6fcd31555226621ebc2c1fea9b7） | dependency_revisions:auto-ui b4fe0000e56cddaf7d3cf70e348431f2258aacde / auto-down c3464d0a0d36e4f8a7988b0c0c26448fc52361b4 | spec_inputs/frozen_delta/acceptance_results:[R2报告](attachments/852/review-r2/report.md)及同目录hash/可重放脚本/原始输出 | findings:F-852-R1-01部分关闭但R2-A(P1实际nightly1.90编译却记录stable1.98且日常PASS)/R2-B(P2 flags和deps错误类型被过滤、未知字段忽略)未关闭；F-852-R1-02隐式认证移除确认；F-852-R1-03双流/溢出/墙控3绿，但R2-C(P2 identity失败scratch残留)未关闭；F-852-R1-04交接补齐、stale.rs新增格式仍失败 | acceptance_results:AC02/05 pass，AC01/04/07/08 partial，AC03/06 fail，AC09 pending | evidence:有效/旧篡改正反控、实际双工具链新认证+显式--rustc对照、PS5.1 snapshot97绿但误身份仍绿、原128KiB反例已绿、定向治理3测试绿；58源码43参考+9诊断6trap逐例静态审阅 | pending_closeout_gates:最终review pass后merge-owned裸cargo t+实际触面档；本轮无整档 | next:原树work修复并提交后再审，保留已关闭反例/不重复全量；第二次最终复审非pass，下次交接须有共享身份/严格schema机制诊断与覆盖调整。T-03～06重开/current_step=2，plan_revision不变；此前重关闭声明为历史记录，不代表当前可合入。PS7初始probe错误已校正，非产品finding；正式复放为文档指定PS5.1。
2026-10-26 stage:review | review_scope:final | reviewer_identity:当前Codex独立复审上下文（本会话未参与work，无代理） | PLAN-852 r2 | outcome:needs_fix | worktree_path:D:/autostack/.wt/lang-852/auto-lang | branch:codex/plan-852-dev | reviewed_commit:4b3c9b254487f81322670fb62cc6b7953718cadb | base_commit:6c02d13628f7c6fcd31555226621ebc2c1fea9b7（原work基面2e09547ee） | dependency_revisions:auto-ui b4fe0000e56cddaf7d3cf70e348431f2258aacde / auto-down c3464d0a0d36e4f8a7988b0c0c26448fc52361b4 | spec_inputs/冻结SD/逐AC结论:[R1报告](attachments/852/review-r1/report.md)及同目录hash/原始输出 | findings:F-852-R1-01(P1证书完整身份/严格收据核查缺失，篡改仍reused且snapshot PASS)、F-852-R1-02(P1 acceptance和日常测试隐式认证/重编)、F-852-R1-03(P2双管阻塞+输出上限/工具身份治理不完整)、F-852-R1-04(P2格式与冻结交接/SD待完) | acceptance_results:AC01/02/08 partial、AC03/04/06/07 fail、AC05 inspection pass、AC09 pending | pending_closeout_gates:最终复审pass后由merge在冻结提交执行裸cargo t及实际触面档；本轮无整档 | next:work在原树修复T-03～T-06并提交，定向反控后复审；T-07未通过，不合并/归档/清理。current_step按仍完成2项重算，plan_revision不变；原[✅ 已完成]文字为历史work声明，由本记录覆盖当前状态。

2026-10-26 stage:work | PLAN-852 r2 | outcome:pass（work-owned T-01~T-06全完成，execution_done送审） | worktree: D:/autostack/.wt/lang-852/auto-lang | branch: codex/plan-852-dev | base: 2e09547ee | code_head: 4b3c9b254 (commits 38c651163/5e6e5ceaa/7fcebf511/40b607727/b19c6b2d8/77176a7f2/4b3c9b254) | deps: auto-ui b4fe000 / auto-down c3464d0 (组内detached只读) | tasks: T-01~T-06 | evidence: 底座=base套件36例97检查绿+cargo测试5绿（含负例oracle/金样漂移/认证新鲜+篡改/墙钟杀树）+六scope verify全绿（contract/source/snapshot/certify/lifecycle/acceptance，冻结头实测）；基线B01/B02/B04/B09四路径真实值+10证书；M1完整性=独立轴（75/252语料单元绿，owner阻断如实登记binding.tsv）| in-contract findings: 无未决 | merge-owned门禁pending: 裸cargo t+触面档（test-cases面已定向绿，全档归merge在冻结提交执行）；SD-03/04/05对既有spec的修改与ledger沉淀归merge | blockers: 无 | next: review（T-07同树独立上下文）

2026-10-26 stage:work | PLAN-852 r2 | T-04+T-05完成 | worktree: D:/autostack/.wt/lang-852/auto-lang | branch: codex/plan-852-dev | codes: 40b607727 / b19c6b2d8 | tasks: T-04, T-05 | evidence: 10证书+G03~G08正反控绿；lifecycle scope PASS（6检查真实观测+墙钟杀树）；base套件97/97 | blockers: 无 | next: T-06验收与两轴报告

2026-10-26 stage:work | PLAN-852 r2 | T-03完成 | worktree: D:/autostack/.wt/lang-852/auto-lang | branch: codex/plan-852-dev | code: 7fcebf511 | tasks: T-03 | evidence: 24金样（oracle/m1_*.rs）+a2r 26检查绿+全套件91绿+snapshot scope PASS+G04漂移红测试 | blockers: 无 | next: T-04独立认证

2026-10-26 stage:work | PLAN-852 r2 | T-02完成 | worktree: D:/autostack/.wt/lang-852/auto-lang | branch: codex/plan-852-dev | code: 5e6e5ceaa | tasks: T-02 | evidence: base套件67/67绿（auto-case gate 0）+ cargo ace_m1 2测试绿 + 负例oracle红 + verify source/contract scope PASS；全58例观测报告 probe-t01/obs-report.json（工作树本地）；差异行登记 binding.tsv | blockers: 无 | next: T-03金样生成审阅+emit快照+认证身份

2026-10-26 stage:work | PLAN-852 r2 | T-01完成 | worktree: D:/autostack/.wt/lang-852/auto-lang | branch: codex/plan-852-dev | base: 2e09547ee→6c02d1362 | code: 38c651163 | deps: auto-ui b4fe000 / auto-down c3464d0 (组内detached只读) | tasks: T-01 | evidence: 探针报告 probe-t01/（工作树本地）+[contract-decision](attachments/852/contract-decision.md)；contract scope PASS+反控FAIL | blockers: 无 | next: T-02接通B01/B02/B04/B09四路径正式语料

2026-10-26 stage:new | PLAN-852 r1→r2 | outcome:pass（底座交付合同可进入work，非实现验收） | authority:用户明确先修订、随后自行实施852 | changed:§5.5、AC-01/02/03/05/07、T-01～06的底座/M1出口及D852无环交接；任务8/当前0、drafting、全部语料/预期保留 | next:work/T-01，无需再次方向确认 | evidence:[共同合同](reports/ace-852-baseline-and-capability-validation-2026-10-26.md)、[修订预检](attachments/852-phase-integration-20261026/preflight.json)。本轮不代启实施。

- stage: new；PLAN-852 r1；outcome: pass（设计交接，非执行/独立最终review）；next: work/T-01。
- 静态预检见[preflight.json](attachments/852/preflight.json)：计数、唯一ID、文件身份/参考/绑定检查；不作为语法或语义执行证据。
- Auto/Rust编译运行、实际a2r金样/认证、正式suite接线及执行门禁均未进行。
- 用户速度裁定进入AC-03/04、T-03/04、G01～08；不存在日常逐例Rust编译任务。

## 10. 待澄清事项

r2明确：第2/4项新源API/native在线接口未交付时只登记owner和需求，不阻塞852基础层；第1项最低标量共同入口及第5项测试/认证合同仍是本计划必须解决的工作决策。需要改变语言语义的具体差异另行裁定，不以泛化待澄清阻塞全部基础实现。

以下由T-01有界调查，现有范围授权继承，不因需调查而自动要求重新授权：

1. 共同具名入口/完整typed返回，特别是当前a2r对`fn main() int`的实际形态。
2. Dyn/portable共同源API、宽标量位型通道；冻结前不发明Auto语法。
3. 旧int/let/borrow/闭包语义差异；改已议语义时携具体反例交用户决定，不能为绿改expected。
4. native在线装载/切换接口与owner；不以重启exe替代，不由测试计划实现。
5. 认证独立伴随合同优先；必要report/decoder扩展显式版本化，保持原严格接受/拒绝规则。
