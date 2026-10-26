---
plan_id: PLAN-852
status: drafting
feature_name: ACE M1共同子集四路径测试语料与Rust金样独立认证
plan_revision: 1
author: [Codex]
created_at: 2026-10-26
updated_at: 2026-10-26
current_step: 0
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

为用户确认的M1方向建立文件式测试分母：同一Auto源码覆盖共享Typed MIR原生路径AC、ABC解释路径AV、旧AutoVM及a2r。
**日常a2r仅生成Rust并比较已认证金样，绝不逐例再编译运行Rust。** 真实Rust行为核验是独立认证流程，完成后按金样/依赖身份复用。

本轮交付为执行合同和[设计包](attachments/852/README.md)：58份Auto源码草案、5份宽标量typed入口草案、43份独立Rust行为参考、43份AutoDown接线草案，及Dyn/CTE/资源/增量/HMR/认证场景。不是这些用例已运行或M1已通过的声明；真实a2r金样尚未生成。

## 1. 目标

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

List/Option/Result/enum/闭包拼写来自现有Auto示例，组合接受仍待T-01。宽标量typed参数/位型入口、Dyn/portable真实源API先冻结再补src/W*.at、D*.at；typed fixture不能替最终源码证明，未补保持required gap。

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

### 规范增量

| delta_id | add/modify/retire | docs/specs/... target | before/after rule | rationale | acceptance IDs |
|---|---|---|---|---|---|
| SD-01 | add | docs/specs/auto-test-case/ace-m1-conformance.md | 无统一分母→同源四路径/差异/真实源码边界 | 防不同程序冒称一致 | AC-01/02/05 |
| SD-02 | add | docs/specs/auto-test-case/a2r-golden-certification.md | emit不证明运行→独立认证、日常只快照 | 用户速度裁定 | AC-03/04 |
| SD-03 | modify | docs/specs/auto-test-case/architecture.md | 补认证拓扑/身份/父侧判决/工具治理 | 不隐藏重编译或新增第二runner | AC-03/04/07 |
| SD-04 | modify | docs/specs/auto-test-case/project.md | 导航新语料、认证与缺口 | 现状准确 | AC-05/08 |
| SD-05 | modify | docs/specs/auto-test-case/profile-v1.md、profile-v2-native.md | 保持旧接受/拒绝集；新增合同显式版本化/导航 | 不宽松忽略未知字段 | AC-04/06 |

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
| AC-01 | work | 58源码和supplement无遗漏/重复；入口/hash/expected/能力状态可追踪；已支持子集同源执行，缺能力精确gap |
| AC-02 | work | 43值例、9诊断、6trap独立审阅，按真实阶段映射；位型/资源不被单一返回值替代；不擅改语义 |
| AC-03 | work | 对实际已支持的标量及复合代表完成真实a2r金样认证；尚不支持项保留required gap、不伪造证书；独立入口有真实行为证据 |
| AC-04 | work | 日常仅emit/快照；G02/03零编译运行spawn；内容/依赖漂移只影响相关证书；G04～08负控非绿 |
| AC-05 | work | required四路径分母独立于selector；unsupported/差异/infra/未认证阻止相应M1全绿；语料交付和成熟度分别报告 |
| AC-06 | work | v1/v2/recipe/stale现状兼容；新增协议按T-01显式决定，有正反例 |
| AC-07 | work | 已绑定会话/资源/认证工具有真实负控及治理；不可用native在线等精确登记owner；不扩AJ或伪装重启 |
| AC-08 | review | 独立逐AC/SD/分母审查，绑定revision/原始证据，遗漏/延后扫描；merge-owned门禁pending明确 |
| AC-09 | merge | 冻结提交按实际触面门禁；Spec/ledger/归档/guard清理有收据；不借本计划宣称M1已绿 |

## 8. 执行步骤

| ID | owner_stage | 依赖、触面和结果 | 验证/关联AC |
|---|---|---|---|
| T-01 | work | 真语法/入口探针；冻结case→四路径→观察→gap、未决源API、证书schema/独立命令/工具预算；写contract-decision | 新脚本contract scope核清单/负控；AC-01/02/05/06 |
| T-02 | work | 审阅58源码/43参考，提升已绑定src/case/suite；补已冻结W/D源码；缺语言能力保留gap、不改引擎 | 未来verify脚本source scope，逐例raw值/诊断；AC-01/02/05 |
| T-03 | work | 真实a2r生成、审阅金样；既有emit快照+认证核查/报告；普通run不编译Rust | 未来snapshot scope；G02/03 spawn反控；AC-04/05/06 |
| T-04 | work | 独立显式认证入口；contained编译运行实际Rust和独立参考，父侧固定oracle比较并保存收据 | 未来certify scope；G01/04～08；AC-03/04/06/07 |
| T-05 | work | E/C/R/I绑定已交付真实接口；共享文件单写交接，native loader等缺失保持M1 gap；不复制底层实现 | 未来lifecycle scope，原样计数/身份/峰值；AC-02/05/07 |
| T-06 | work | acceptance核全集分母/实际选择/零选择反控/证书；SD草案、差异owner和handoff；无整档测试 | 定向`cargo test -p auto-test-case <新增家族>`及`cargo test -p auto-lang --no-default-features --features test-cases --lib <新增家族>`；T-01建真实名称；AC-01～07 |
| T-07 | review | 同树独立上下文逐AC/SD/固定oracle核验，不自修，出revision-bound结论 | AC-08，merge-owned裸t及触面档pending |
| T-08 | merge | 最终审查后在冻结提交跑实际Rust触面门禁，Spec/ledger/归档/guard清理 | AC-09；纯设计资产不跑cargo t/docs_gen |

拟新脚本`powershell -File scripts/verify-ace-m1-852.ps1 -Scope contract|source|snapshot|certify|lifecycle|acceptance`各Scope由以上任务建立，目前不存在、未运行；certify始终独立显式操作，acceptance只核已有证据不暗中重认证。
下一执行入口T-01；本轮0/8。标量与旧引擎基线不等822～826全体；新增family按D依赖局部阻塞。关闭测试计划不关闭M1缺口；M1全绿需要所有required真实满足。

## 9. 复审记录

- stage: new；PLAN-852 r1；outcome: pass（设计交接，非执行/独立最终review）；next: work/T-01。
- 静态预检见[preflight.json](attachments/852/preflight.json)：计数、唯一ID、文件身份/参考/绑定检查；不作为语法或语义执行证据。
- Auto/Rust编译运行、实际a2r金样/认证、正式suite接线及执行门禁均未进行。
- 用户速度裁定进入AC-03/04、T-03/04、G01～08；不存在日常逐例Rust编译任务。

## 10. 待澄清事项

以下由T-01有界调查，现有范围授权继承，不因需调查而自动要求重新授权：

1. 共同具名入口/完整typed返回，特别是当前a2r对`fn main() int`的实际形态。
2. Dyn/portable共同源API、宽标量位型通道；冻结前不发明Auto语法。
3. 旧int/let/borrow/闭包语义差异；改已议语义时携具体反例交用户决定，不能为绿改expected。
4. native在线装载/切换接口与owner；不以重启exe替代，不由测试计划实现。
5. 认证独立伴随合同优先；必要report/decoder扩展显式版本化，保持原严格接受/拒绝规则。
