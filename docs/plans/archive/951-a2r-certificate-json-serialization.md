---
plan_id: PLAN-951
status: archived
feature_name: HIR Rust证书严格JSON序列化修复
plan_revision: 1
author: [Codex]
created_at: 2026-11-06
updated_at: 2026-11-06
supersedes_spec_components: [docs/specs/auto-test-case/a2r-hir-certification.md]
new_spec_components: []
touched_goals: [GOAL-002]
affects: [auto-a2x, auto-test-case]
depends_on: [PLAN-892, PLAN-902, PLAN-904]
current_step: 5
total_steps: 6
---

# [PLAN-951] HIR Rust证书严格JSON序列化修复

## 0. 变更摘要

独立关闭941首例发现的F-941-01/D-941-1：原`CertDraft::to_json`在capabilities后缺逗号、末尾execution_config多逗号，产生不可严格解析的证书。仅修producer序列化、补真实JSON解析回归与原worker重验；不宽容坏JSON，不扩语言/后端能力，不替换legacy A2R的38格认证身份。

## 1. 目标

scalar、record-text、collections三种Rust证书均产生合法UTF-8 JSON，字段/枚举/完整actual/reference/toolchain身份、原预算及daily/cert分轴保持。空/非空capabilities、execution_config=null/有效canonical配置都以实际JSON decoder验证，不以typed draft合法或两次坏字节相等冒证据。

## 2. 架构方案

保留手写serializer及字段顺序，只修对象字段分隔。回归放既有`ace-a2x-worker` conformance test target，复用其已有serde_json依赖，显式注册新测试模块；不引入依赖或另建runner。显式原worker继续使用既有受管编译/执行。

## 3. 技术栈

`experimental/ace/crates/a2x-rust/src/cert.rs`；`experimental/ace/crates/a2x-worker/tests/{conformance.rs,cases/cert_json_951.rs}`（新模块）；现`cargo t ace-a2x`固定组及`--test-name cert_json_951`定向入口。

## 4. 需求分析与背景调查

授权：2026-11-06用户答复“授权独立修复并继续（推荐）”，明确允许本独立包work/review/merge及返回941；不再重问同范围授权。取材main `6e2abe9fe`，941检查点`91968fd40`。原scope的真实Rust证书hash `63a49eab9a4d8f9a0cf9b1bc3ee7947dcf481a1beaf8f28ac91b51484f52c8ad`，标准parser报line36 column3；C正控合法。原测试只验typed合法和重复序列化稳定。当前Specs要求严格证书，不授权parser修补。

编号簿起点949已被占用，取号前对账全部在途/归档文件，将落后游标提升到实际最大编号+1，再运行`scripts/new-plan.sh a2r-certificate-json-serialization`；脚本分配951（并发另包950被保留），next=952。其余main仅文档WIP，不纳本包提交。唯一实施树拟`D:/autostack/.wt/lang-951/auto-lang`、branch `plan-951-dev`；材料仅`D:/autostack/plan-artifacts/plan-951/`，开工登记owner，用完merge全删。941树继续保留，非本包实现/复审副本。

应用goal当前仍显示blocked，但用户已明确授权“并继续”；本包按该直接指令执行，不另建goal、不声称已由工具恢复active。

## 5. 详细设计

producer `CertDraft` → `to_json` → 原worker证书文件 → 严格JSON decoder。修capabilities字段后的逗号，最后execution_config字段不再带尾逗号；非空配置字符串继续沿原kv转义、原字段顺序不变。空数组/多项数组、三schema、null/带引号及换行的合法字符串参数在实际serde_json解析中保持原值。

### 规范增量

| delta_id | action | target | before/after | AC |
|---|---|---|---|---|
| SD-01 | modify | docs/specs/auto-test-case/a2r-hir-certification.md | 严格证书规则不变；明确typed合法/稳定序列化不足，输出须标准JSON解析并保持配置/能力值 | AC-01/02 |

## 6. 测试设计

| V | 输入/入口 | 独立可观察oracle/关闭条件 |
|---|---|---|
| V-01 | 新`cert_json_951`模块，三Rust schema与空/非空能力、null/collections有效配置 | serde_json实际解析原serializer输出；修前失败、修后通过；关键字段与原typed输入逐值相等 |
| V-02 | 同测试模块中带转义的配置/字符串、重复序列化 | 合法JSON且解码值不变，输出字节稳定；不只是字面标点匹配 |
| V-03 | 原a2x-worker对941同源Q4-001及独立reference真实certify | actual/reference各value:42，证书严格解析/完整文件hash可核；C证书正控仍合法 |
| V-04 | 最终冻结`cargo t ace-a2x` | 原完整职责组全部原条件执行，记录执行/排除/known red，不重跑无关全仓/全性能 |

实现与final review同树顺序进行。无授权spawn/chat，复审由当前会话重新读取冻结diff/AC与真实证据，并明确同一上下文的限制，不冒称独立agent。merge冻结完整组只跑一次；此小serializer修复不改变编译/运行性能算法，不另重采正式P90。

## 7. 验收标准

| ID | owner_stage | 条件 |
|---|---|---|
| AC-01 | work | 三schema、空/非空能力、null/有效配置的真实JSON解析均通过且字段保真，修前红证据保留 |
| AC-02 | work | 既有worker完整实际认证+独立reference仍正确，证书完整可读，C正控不退，schema/预算/legacy38守卫不改 |
| AC-03 | review | 冻结SHA逐AC/SD/遗漏/健康复审通过，明确同会话复审限制 |
| AC-04 | merge | 冻结ace-a2x职责组与必要消费者门通过，SD主线读回、线性合入归档、全部材料/guard清理可核 |

## 8. 执行步骤

| ID | owner_stage | 内容 | 状态 |
|---|---|---|---|
| T-01 | work | 建专属树/scope，实际serializer解析回归修前红 | [x] |
| T-02 | work | 修serializer并补三schema/配置/能力/转义字段保真，定向回归通过 | [x] |
| T-03 | work | 原worker实际认证重放/C正控，准备SD，提交冻结实现 | [x] |
| T-04 | review | 同树重新核AC/SD/反例/健康，提交绑定final verdict | [x] |
| T-05 | merge | 冻结完整ace-a2x组、必要消费者，线性合入/Spec读回/归档 | [x] |
| T-06 | merge | 所有输出对账、scope全删、双仓guard后清树分支组，回941 | [ ] |

## 9. 复审记录

2026-11-06 work entry：确认树D:/autostack/.wt/lang-951/auto-lang / branch plan-951-dev / base 1e85cb756，auto-down只读58ea33f。scope已在全局plan-artifacts/.retention/policy.json登记；T-01回归测试已写入原conformance target，原producer尚未改，定向红门运行中。

2026-11-06 stage:new | PLAN-951 r1 | outcome:pass | 用户明确授权独立修复并继续941 | next:work T-01；不改941原验收/目标。


2026-11-06 work T-01/T-02：`cargo t ace-a2x --test-name cert_json_951 --out .../red-json` exit1，2/2实际失败于标准serde_json解析（expected comma/close，line34 col3），冷编译35.11s、整命令37.260s；raw results SHA256 `63c27b33299616a077224563edc84216abb9d34796bf14011c33af95f19e621e`。修正producer字段分隔后同入口`.../green-json` exit0，2/2实际通过、8组typed组合（3schema/null与集合配置/空多能力/转义）与各自坏分隔符负控，整命令3.902s；raw results SHA256 `72d991b0e6004ba6e442b72d3caf1f0152defe6a3c39b3045f452cb30b5bc9f3`。`rustfmt --edition 2021 --config skip_children=true --check <cert.rs> <cert_json_951.rs>`与`git diff --check` exit0。实现及SD冻结提交`6d21078da10ecf92bf3407183542fb817d469a63`，4文件123行新增/3行删除，无Cargo/lock/语言/预算/旧guard变更。T-03原worker release准备中，尚未ready for review。

冻结门选择：`ace-a2x`完整组涵盖被改producer及所有真实Rust/C worker认证/文件集/共享store消费者；根adapter日常只emit字节、不读取本证书serializer，本包未改其合同/接线，故不另跑根/全仓/全正式性能。正式时间/样本/ignore条件保持，merge一次完整组；targeted修前红与修后绿不替代最终门。


2026-11-06 stage:work | PLAN-951 r1 | outcome:pass | worktree:D:/autostack/.wt/lang-951/auto-lang | base:1e85cb756 | code_commit:6d21078da10ecf92bf3407183542fb817d469a63 | T-01..03完成。`cargo build --locked --offline --release --manifest-path experimental/ace/Cargo.toml -p ace-a2x-worker --bin a2x-worker` exit0（126s，log无warning）。从941提交91968fd40取Q4源/ref blob到本scope/q4-inputs（非本机未跟踪残留，源SHA256仍827926839c09c494915fa161b7e4689294aaf4171af82fb24325df413d4f7b9d）；既有worker+process_capture.capture依次显式certify/certify-c，每次完整actual+独立reference编译运行，stdout各value:42、exit0，Rust1723.31ms、C15407.97ms，原360s墙内。941 EvidenceScope原严格JSON reader原样接受两证书，独立校源/ref hash并流式核3个actual源文件，schema/recipe/oracle42均符；原daily/旧38守卫未动。Rust cert hash `79833213b7ce060b6c46a75f5b4d4c7a6bce1f9da14ddb7ce18ce372c8a6a5cc`，命令收据hash `5795dc4c43af290dd05b580f4963c6ccac42ef0e9b74dcd1bf1e2cae64d888ba`；C正控cert hash `a54d5d4a52f18b9bd28384e2bebbaef238dd24e274628a527598a0518a2da6d1`、命令收据hash `44de514f4a9860c377cb511803a3b0c36a40bc9fb35c75a428db335feac6693e`。全部原始收据/actual/reference/exe仅本scope，默认构建target仅专属树内；无其他外部安装/消费目录。work-owned AC01/02已闭，无开放工作内finding，最终ace-a2x完整门归merge，next:final review。


2026-11-06 stage:review | review_scope:final | reviewer:当前会话（与实现同上下文，未冒称独立agent；重新从冻结diff/原始证书/实际parser终态重建结论） | PLAN-951 r1 | outcome:pass | reviewed_commit:6d21078da10ecf92bf3407183542fb817d469a63 | base:1e85cb756 | worktree/branch:D:/autostack/.wt/lang-951/auto-lang / plan-951-dev | dependency:auto-down58ea33f。AC01=PASS：定向2test/8组合真实解析，修前原producer两test红，修后绿；双次坏JSON相等的历史伪证据被替换为标准parser独立oracle。AC02=PASS：重新读取两份现场cert，用标准json对象钩子拒重复key；Rust22键、C21键，source/ref/actual三文件hash及oracle42与原始命令独立核对，Rustnull配置仍原typed语义。生产diff仅字段逗号/最后kv末尾分隔，原字段序/转义/identity/typed验证保持；真实worker没有绕过producer。AC03=PASS：改动仅4个声明路径，target注册确实被固定组发现执行；未改旧guard、分母、预算/样本/ignore；无遗漏/未经授权延后/掩盖债，fmt与diff健康均绿，树clean。SD01=PASS：canonical文件`docs/specs/auto-test-case/a2r-hir-certification.md`增量确切对应输出可解析与字段保真，sha256 `53c96a30ecc6482852921c3c624322d4cce6898c532994a62cdad6c45f29c759`。最终完整ace-a2x职责并集pending merge；不凭本包签M1。next:merge。


2026-11-06 stage:merge | PLAN-951 r1 | prepared/landed/canonical_verified:pass | reviewed6d21078da→delivery **fec8a9602413847c681c6944d490d7a39e18e272**，rebase onto master a3a42d74b，`git range-diff 1e85cb756..6d21078da master..HEAD`为1/1 `=`，实现/依赖/SD字节相同；主检出`git merge --ff-only plan-951-dev`到该delivery，无merge commit、未动他方文档WIP。最终冻结`cargo t ace-a2x --out D:/autostack/plan-artifacts/plan-951/final-ace-a2x` exit0、selected=executed=183、excluded=0、not_run=0、183终态全ok，整命令168.602s（准备编译2.94s另列），raw results SHA256 `841cf25e0cec3af3b255a767209a5d10ab0bd7aa9ad4c168154dca93c046cf3d`，source_state b9f159a12ec03847c412c0227fe0fbb4d92bde06f8120645936fa30fab195d44。scope内原始证据摘要已折本计划；共享变更只在证书serializer，完整Rust/C认证及store职责原用例均真实执行。canonical主线读回sha256仍`53c96a30ecc6482852921c3c624322d4cce6898c532994a62cdad6c45f29c759`。guard双仓clean。T-06 cleanup待实际完成；不以本包通过冒M1通过。

输出路径对账：专属build仅951树`experimental/ace/target/`与` scripts/full-regression-driver/target/`；原group测试a2x-out*/store等均在该树的experimental/ace/target下；手工Q4源码/ref副本、完整cert/actual/reference/exe、meter/stdout/stderr及固定门收据全部在scope。唯一外部原用例路径`%TEMP%/a2c-det/{a,b}`为2026-11-03已存在的通用determinism共享缓存（创建时间实查，原`a2c_scalar_semantics::c_fileset_identity_and_determinism`使用，本次lastwrite 2026-11-06 01:34:02），不属951新建专属材料，保留原缓存、未迁移/删除它；无951第二套target/SDK/安装根。主产品auto/UI无代码改动与重启义务；主检出既有预构建a2x-worker不自动视为已更新，941消费修复前须在其树重建原worker并重新认证，不复用旧二进制PASS。

batch due：本包951非5倍，最新receipt covered940、timestamp2026-11-05T17:15:51Z，未超过48h；950尚未交付，故本包不触发批回归，不叠加全仓门。

## 10. 待澄清事项

无新用户决定；发现超出序列化修复的语言/资源缺陷另具名，不塞本包。
