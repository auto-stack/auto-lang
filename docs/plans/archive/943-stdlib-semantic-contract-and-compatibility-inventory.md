---
plan_id: PLAN-943
status: archived
feature_name: 标准库语义合同与兼容性实册
author: [Codex]
created_at: 2026-11-05
updated_at: 2026-11-05
plan_revision: 1
supersedes_spec_components: [docs/specs/stdlib/project.md, docs/specs/stdlib/design/backend-assembly.md, docs/specs/stdlib/design/primitive-str-byte-len.md, docs/specs/goals.md]
new_spec_components: [docs/specs/stdlib/design/semantic-operation-contract.md]
touched_goals: [GOAL-002]
affects: [stdlib, ace]
depends_on: []
current_step: 7
total_steps: 7
---

# [PLAN-943] 标准库语义合同与兼容性实册

## 0. 变更摘要

建立真实公共 API→绑定→provider→consumer 的全库分母和首批可执行语义合同，保留已有行为差异；让旧装配 inventory 真正消费合同，给 ACE 接入提供可复用输入。

设计依据：[Design54](../../design/54-stdlib-semantics-and-ace-integration.md)，衔接 Design33/46/51。初稿为仅设计交接；本次 goal 已授权实施；实施与两项发现修复已交付，独立R3与最终门通过，现已落主线并归档；不切换默认引擎。

## 1. 目标

本包目标：建立真实公共 API→绑定→provider→consumer 的全库分母和首批可执行语义合同，保留已有行为差异；让旧装配 inventory 真正消费合同，给 ACE 接入提供可复用输入。

非目标：不改变既有 API 行为/默认引擎，不实现 ACE 调用，不修整个标准库，不统一 len/parse_int，不把 API 文件数量当已支持数量。

不改变原M1/M2 required/accepted-diff/NA、语料/oracle、既有资源预算及CAP进入门。缺口保留，不为通过删除子项。

## 2. 架构方案

复用当前统一source/Checked HIR、共享MIR/资源事实及query/store。A2X消费HIR，AV/native消费共享MIR；公共语义与物理实现分开，旧runtime只按实际合同复用。标准操作身份、兼容策略和manifest双指纹遵守Design54 §4–7。

依赖：无新计划前置；开工重核当前主线/Specs。

## 3. 技术栈

Auto公共源码、现有Rust装配/ACE workspace、既有TestCase与固定Cargo组。不得添加平行编译器、资源分析、store或自报PASS系统；新文件/接口为拟议，T-01先核真实consumer再定位置。

## 4. 需求分析与背景调查

从 `docs/specs/overview.md` 的stdlib/语言/ACE边界取材，读取current stdlib项目卡、assembly-manifest/backend-assembly、相关模块源码及Design33/46/51。读取基线HEAD `a0750086c`；部分设计在磁盘有其他会话WIP，确切输入hash见Design54 §12，不把WIP当正式D。

用户授权原文：“按这个方向编写……设计文档……拆分后续实施计划，先不施工。”本轮仅AutoLang设计/计划/索引簿记，未授权任何实现、外仓写入或自动续跑；没有预算/模型/agent指派。未来执行需要后续授权，不能以本draft推断批准施工。

已核路径（后继开工重核实际符号与版本）：

- `stdlib/auto`（现存；文件/目录）
- `crates/auto-lang-core/src/stdlib_assembly/loader.rs`（现存；文件/目录）
- `crates/auto-lang-core/src/stdlib_assembly/providers.rs`（现存；文件/目录）
- `crates/auto-lang-core/src/stdlib_assembly/validate.rs`（现存；文件/目录）
- `docs/specs/stdlib/project.md`（现存；文件/目录）
- `docs/specs/stdlib/design/primitive-str-byte-len.md`（现存；文件/目录）

边界：现有文档的历史数量/状态不自动当当前运行事实；`primitive-str-byte-len`与872绑定矩阵差异先由943按具体入口处理。旧math公开i64与ACE i32域分开；普通Rust/C能编译不证明Auto语义。

## 5. 详细设计

首批合同源拟新增 stdlib/contracts/operations.toml 与 compatibility.toml（名称/格式由 T-01 冻结）；语义 ID 独立于 native ID，包含公共拼写/可见性、宽度/单位、mode、错误、需求、兼容关系及适用 profile。inventory 适配器在真实扫描/校验路径消费它，逐项报告缺合同、冲突、未知和候选，不能仅新增无人读取的数据文件。公共源不可解析、非公开 helper、native-only 别名保留各自身份。

冻结种子：现有 byte_len 合同及文本 equality/clone 的真实公共绑定、int 纯 helper 的可见性、当前有限 List/Variant 操作、console 输出；math abs/min/max 的 i64 声明不映射成 i32。943 不承诺这些候选全部直接可用：逐项给可实施路径/缺口；945/946 的最终 required 表必须在实施前以 r2 或确认记录冻结，未解决语义行不可进入支持集。

### 执行合同与下一入口

输入→实际producer→存储/中间表示→consumer见上列路径及§6，未新增接线由T-01/T-02创建。开工先核主检出计划revision、正式依赖SHA与当前树HEAD，记录实际已闭/未闭项；T-01只允许有界前探与合同冻结，不能用整包重构解决未知。需改变公开语义/支持阈值/跨仓权限则修订Plan并向用户说明具体决定。

所有临时probe、日志、receipts、外部验证副本统一 `D:/autostack/plan-artifacts/plan-943/`，开工登记owner/用途/在途状态；本轮未创建。常规构建在唯一实施树target，外置target也只在此scope。留存结论最终折入本Plan；测试fixtures若为产品长期用例留在真实owner测试目录并明确T-02..05登记，不把临时探针散落仓根。

### 规范增量

### T-01 冻结确认（2026-11-05，保持 r1 验收范围）

执行授权继承本 goal；唯一实施树 `D:/autostack/.wt/lang-943/auto-lang`，`plan-943-dev`，base `a0750086c57c809a586884ac8b0519e8c162979b`；只读兄弟 auto-down `7c4d5cb`。建组入口默认落 WSL Git，完成后用 Windows `git worktree repair` 修复双方路径登记；未建链接。迁出用户临时文件 manifest SHA256=`224BE0F54B4DDC47E96290F57E332A86CEB3BAE28A11592A6CF5B44FC509EB04`。

位置冻结：`stdlib/contracts/operations.toml`、`compatibility.toml`，schema 1；真实消费点为 `stdlib_assembly::loader::scan_inventory`。首批17项包括 byte_len、3个i64 math、2个private math helper、text equality/clone、List.len、Result.is_ok、console.say、两种read_line及4个旧文本入口。字段涵盖版本/ID、签名/mode、单位/派发、所有权/错误/需求、profile/oracle、来源与后继owner。其余声明保留 missing_contract，失败层的API数保持未知，不归为已认证。

| 首批族 | 实际来源与决定 | 后继进入边界 |
|---|---|---|
| byte_len | str.rs.at::str_byte_len(str)->int；catalogue1526；str.at无此声明且有既有解析失败；UTF-8 bytes | 944/945需真实公共绑定；legacy合同保留 |
| len/length | 872项目卡与catalogue/engine：静态1500 bytes、CALL_SPEC scalars、属性170/内联 scalars；a2r bytes/JS UTF-16差异保留 | primitive旧文档不能全局覆盖具体派发；不入无差异认证 |
| math | math.at公开abs/min/max参数返回均i64；square/cube为非pub的int body | scalar-i32不是i64公共API；private helper不绕可见性，945仍需关闭公共纯算法缺口 |
| text equality/clone | equality是语言操作；clone.at是Clone spec；ac_rt_text_eq/ac_rt_own_clone只证明候选物理入口 | 945未接线，不把spec/staticlib/私有ABI冒充公共source支持 |
| List/Result | list.at泛型层有既有解析缺口；result.at方法与ACE有限Variant分账 | 946仅沿已有有限域，不推断任意泛型 |
| console/EOF | io.at say/read_line/read_line_opt；813空行Some("")、EOF None；AV console/1有独立授权合同 | 947 say未映射console/1；File/Task/Stream不自动支持 |
| parse_int/async | VM parse_int/to_int同1516、失败裸i32零，a2r Result差异保留；async.at Task/Msg声明与backend-assembly的VM park-resume/Rust Tokio actor-drain独立 | 未证明生命周期等价；948持有差额，不做异步转同步或资源转整数 |

945/946/947的输入是这些冻结差额。四个ACE producer当前均无公共stdlib装配接线，unwired/missing_capability/unknown不算supported；945最小集合不足由其T-01据差额解决，943不新增公共API或删除差额。既有required/accepted-diff/NA、语料、oracle与阈值不变。

以下是**拟议**delta，review按实际范围定稿，merge才写canonical Specs；本轮不写current Spec。

| delta_id | add/modify/retire | docs/specs target | before/after rule | rationale | acceptance IDs |
|---|---|---|---|---|---|
| SD-01 | modify | `docs/specs/stdlib/project.md` | 增补操作语义、兼容差异和实际支持分类，不把计划候选写成已支持。 | Design54/本包实际边界 | AC-01..04 |
| SD-02 | modify | `docs/specs/stdlib/design/backend-assembly.md` | 增补操作语义、兼容差异和实际支持分类，不把计划候选写成已支持。 | Design54/本包实际边界 | AC-01..04 |
| SD-03 | add | `docs/specs/stdlib/design/semantic-operation-contract.md` | 增补操作语义、兼容差异和实际支持分类，不把计划候选写成已支持。 | Design54/本包实际边界 | AC-01..04 |

| SD-04 | modify | `docs/specs/stdlib/design/primitive-str-byte-len.md`、`docs/specs/goals.md` | 将旧 len 全局描述纠正为具体派发；目标仍部分达成，仅增加合同出处。 | 872 实际绑定与本包行为守卫 | AC-01..04 |

## 6. 测试设计

全部是待执行设计，非当前证据。首批实际运行限既有已认证Windows x64；有界T-01只做§6列出的最小输入与对应合法/非法近邻，预算沿各原recipe，不临时添加新性能阈值或做全库压力。新增fixture由T-02..05接到真实entrypoint，T-01把下表的概念输入落实成当前合法语法/入口，原不支持域不改成伪正例。

| Case ID | AC/invariant | 输入/设置 | 真实入口或拟议接线 | 独立可观察结果 | owner/coverage |
|---|---|---|---|---|---|
| V-01 | AC-01 | 同名 len 的静态/动态入口及旧 primitive 描述 | 实际 inventory + production 绑定核查 | 分别记录单位/派发；冲突带来源，不全局归一 | work/T-05；仅本包冻结域 |
| V-02 | AC-02 | 真实合同副本改 byte_len 单位/同宽签名、删除 ID | 装配 inventory 合同校验（T-02/03 接线） | 非法项非零具名；未改副本通过，不能仅 grep 文本 | work/T-05；仅本包冻结域 |
| V-03 | AC-01 | 一个损坏层与一个含内联方法层 | scan_inventory/parse_layer | 失败仍计分母，内联方法不漏；不将 candidate 算 actual | work/T-05；仅本包冻结域 |
| V-04 | AC-03 | math.min 的 i64 与源码 i32 profile | 首批支持表 + 实际 source 前门前探 | 保持 i64 声明；不能仅 cast 后声称已支持 | work/T-05；仅本包冻结域 |

验证入口：`cargo t stdlib`。组名已从本轮 `cargo t --list` 核验；work按实际改动确认完整职责组与必要下游并集，最终review后冻结提交沿同并集核门，身份/输入未变证据可复用。若真实修改 legacy parser/VM 绑定，按实际触面追加 parser/vm；本包不改 ACE producer，不能为合同数据自动跑全部 ACE。

新用例必须进入所选固定完整组；若现有组无法覆盖新合同包，T-05明确登记新固定职责组并在review前冻结，不靠过滤的零选择假绿。原ignore/平台/性能/认证条件不减；不按计划机械运行全ACE或全仓。周期`cargo t all`仅由主检出due门负责。列发现/执行/条件排除数，准备/运行/清理/总时间与子终态；编译错误、超时、未执行不算语义反例PASS。

实现者自查不能替代final independent review。本轮未指派独立agent；未来review以独立上下文在同一唯一worktree核每条AC、遗漏/延后/workaround、warning/fmt/debug及Spec delta，绑定revision/SHA。复审期间停止实施写入，发现问题交回同树修复。

## 7. 验收标准

| ID | owner_stage | 可观察标准 | 核验 |
|---|---|---|---|
| AC-01 | work | 全库实际 API/provider/consumer 分母可重算，失败和未知保留；调用路径与旧语义逐行有来源。 | T-01及T-05；§6对应输入与独立观察 |
| AC-02 | work | 操作合同被真实 inventory 校验消费；ID/签名/单位/mode 冲突具名拒绝，合法相邻对照通过。 | T-02及T-05；§6对应输入与独立观察 |
| AC-03 | work | 945/946/947 所需首批符号表与差额冻结；无 i64→i32 偷换、len 全局改义或旧 accepted-diff 丢失。 | T-03及T-05；§6对应输入与独立观察 |
| AC-04 | work | 旧公共 API 行为不变，定向反例与完整 stdlib 组通过；语义差异不伪报跨端一致。 | T-04及T-05；§6对应输入与独立观察 |
| AC-05 | review | 独立核§6反例/合法对照、所有AC、遗漏/延后、warning/fmt/debug及Spec delta；结论绑定冻结revision/SHA | T-06独立复审记录；不信勾选/父exit0 |
| AC-06 | merge | 实际Spec落地、代码来源一致、归档与scope/树清理对账完成；不得把正确性留到merge | T-07实际收据/guard及路径核验 |

## 8. 执行步骤

预定实施树 `D:/autostack/.wt/lang-943/auto-lang`，分支 `plan-943-dev`；未创建。只在后续获实施授权且依赖就绪后以 `bash scripts/new-wt-group.sh lang-943 --branch plan-943-dev` 建组，开工核唯一树/仓/分支/base/HEAD。无junction、无跨树可写target。

- [x] **T-01｜owner_stage: work｜AC-01**：以现行 Specs/源码取实际分母与冲突清单；核 len 各拼写/派发、byte_len、int/i64、EOF、parse_int、async 生命周期；在本 Plan §5 冻结首批符号与单位/错误决定，若改变旧兼容承诺则停依赖施工等待裁定。
- [x] **T-02｜owner_stage: work｜AC-02**：新增版本化操作/兼容合同数据及有界解码、唯一 ID/签名/单位/mode 检查；沿现有装配模型记录来源，不新建全语言类型系统。
- [x] **T-03｜owner_stage: work｜AC-03**：把真实 inventory/provider 校验接入合同；列公共声明、内联方法/ext、别名及失败层分母；校验同名不同派发不被合并。
- [x] **T-04｜owner_stage: work｜AC-04**：为后继四端/宿主输出明确的支持、缺能力、未接线、未知表和逐项 owner；首批符号的 caller/producer/宽度/源码合法性与独立 expected 一并冻结。
- [x] **T-05｜owner_stage: work｜AC-04**：补缺 ID、同宽错签名、语义错单位、解析失败不删行、用户同名合法对照及旧行为守卫，跑完整职责组并核子终态。
- [x] **T-06｜owner_stage: review｜AC-05**：在同一实施树以独立上下文执行 `/auto-plan:review`，逐AC/反例与真实diff核验，具名记录债务候选；实际通过才进入reviewed。
- [x] **T-07｜owner_stage: merge｜AC-06**：按 `/auto-plan:merge` 核来源/落Spec/归档；证据摘要折入Plan，逐命令实际输出路径对账，删统一scope（具名持有除外），刷新retention；`wt-guard` clean后移除树/分支，实际完成后记账。

T-01可执行前置：无新计划前置；开工重核当前主线/Specs。 只有取得后续实施授权后才运行；依赖未齐时至多做明确隔离的只读准备，不把准备计为下游交付。T-02..05依次依赖T-01冻结结果；T-06在work侧AC全闭后，T-07在review pass后。任务与AC引用多对多：T-05覆盖AC-01..04，不以单行对应限制验证。

## 9. 复审记录

2026-11-05 goal 开工预检：用户以“/goal 完成计划943”及“继续完成计划943”授权 work → independent review → bounded repair → merge/archive/cleanup 全链；原 draft 的“未授权执行”仅为历史状态。当前主线 `a0750086c57c809a586884ac8b0519e8c162979b`，无 943 实施树或分支。发现主检出存在归属未确认的代码路径未跟踪文件：`examples/ui/015-notes/tests/pnpm-lock.yaml`、`scripts/tools/native-fixture/Cargo.lock`、`test/fixtures/reader-import-budget/{book-10mib.md,book-10mib.txt,book-bad.bin,book-small.txt,book-tiny.txt}`。依 work 技能主检出预检/goal stop condition 暂停开工，未改动这些文件、未创建实施树或运行测试；等待用户确定移出暂存或交所属会话处理。

`stage: goal | plan_id: PLAN-943 | plan_revision: 1 | round: 1 | from: preflight | to: work | status: drafting | outcome: waiting | worktree: not_created | commit: a0750086c57c809a586884ac8b0519e8c162979b | next: user`

2026-11-05 draft handoff：`stage: new | plan_id: PLAN-943 | plan_revision: 1 | outcome: pass（draft合同比较完整，未授权执行） | next: 等待用户实施授权；随后 work/T-01。`

本轮只做文档静态预检与固定组名称核验；实施、产品验证、独立final review、Spec沉淀、merge/archive/cleanup全部未执行。current_step=0/7。非完成声明。

实际文档预检：Design54及六份Plan的本地链接、现存源码路径、0–10章节、七任务/六AC、Spec delta映射、drafting/0-of-7及943–948编号唯一性通过；`.next-id=949`。已核所列固定组名；相关跟踪文档`git diff --check`通过。该检查不证明任何拟议功能、产品测试或独立final review已通过。



2026-11-05 work 完成（T-01..05，r1）：`stage: work | plan_id: PLAN-943 | plan_revision: 1 | outcome: pass | worktree_path: D:/autostack/.wt/lang-943/auto-lang | branch: plan-943-dev | base_commit: a0750086c57c809a586884ac8b0519e8c162979b | code_commit: bc9aa48c6a7b50762697e23fa466619375a4f2ae | dependency_revisions: auto-down=7c4d5cb (read-only) | task_ids: T-01..05 | blockers: none | next_responsible_role: final independent review`

- AC-01：实扫122层/77模块，922个声明或失败占位行（其中39失败层，API数量未知）；1403条原生名称/别名，1114条真实绑定、285条未知、4条resolved-unbound；37条provider模块声明保留其声明身份，不冒称逐符号实际支持。类型内方法、ext、enum/spec方法、类型/字段/alias均来自实际AST；来源层及canonical native名可联结，用户同名不套标准操作ID。
- AC-02：schema1两表17对经实际scanner消费；256KiB/1024项有界解码、未知字段/schema、缺/重复ID、政策孤儿、同宽i64→u64错签名、mode、单位、派发、可见性、原生identity均具名拒绝，非法输入不生成assembly fingerprint。未改副本通过。String→str/i32→int为具体已知表示映射，不使用旧normalize_host_type的i64全归int行为。
- AC-03：§5表及机器合同冻结首批入口/宽度/独立expected、四producer支持分类/host/差额与945..948 owner。显式source前门前探：合法int min近邻通过 `auto.source.scalar-i32.draft`；保留i64签名的同构输入拒绝 `source.unsupported`。首次probe因 `} return` 缺换行而语法失败（非语义反例PASS），修正夹具换行后通过；原始与重放日志均保留。
- AC-04/V-01..03：永久tests进入stdlib固定职责组，包含872既有六族守卫及新A中😀=8 bytes、e+U+0301=3 bytes、NUL不截断、泛型receiver CALL_SPEC len=3的实际源码守卫。首次Val参数调用被前端拒绝，改用合法泛型调用（该入口真实生效），未改产品行为。EOF/parse_int/async旧合同保留，未声称各端一致。
- 验证：`cargo t stdlib --out D:/autostack/plan-artifacts/plan-943/work-final` exit0，发现171（147执行+24既有ignored），147/147通过，其他4007不在职责选择内；准备/发现16.949s、执行3.495s、命令总20.482s、清理由runner完成。`work-final/results.json` SHA256=5cdd094851a93062fd90fdf8fb422a730331da95292929f24b916ed21f2fe19e。
- 首次完整组141执行=137绿/4红；实际修复为原生String/i32别名校验、错误夹具语法、原本漏列的trans/rust/semantics.rs与semantics/walk.rs指纹输入，以及失败层合同签名核验接入实际最终native interface；无原公共API实现改动。修复定向79执行78绿/1别名遗漏，补NATIVE_ID_ENTRIES逐名列入后6项合同族通过。后续完整组147执行146绿/1非法Val夹具，合法泛型修复单项通过；work-final完整147/147如上。最后原生identity拒绝补充的定向单项通过（`native-identity-repair/results.json` SHA256=63bce4449b1c3740614406e4cbf553576c8ef0be10aba61a52612a6c79cf73ab），其他证据输入不变复用；末尾TOML仅清理空行，冻结提交后的并集门由merge负责。
- 证据：`inventory.json` SHA256=2538327df3e2291c002ec3459af267cd0ca516418e6ae5c1c1c1479f31b65355；source-profile-replay.log SHA256=e8d196bc7ff6e7e58db68b4ae27b694672424c8d44d78ad59dd5adc61454af57。source-profile源码/target明确位于统一scope，构建从实施树发起；未跑全ACE/全仓。正常target与driver target在实施树。
- 健康：所改Rust文件rustfmt --check通过，git diff --check通过；无新增debug打印（测试有可选机器盘点输出）。编译日志现存core lib-test145警告、facade32警告来自既有宏/未用导入/旧测试等；无本包新增contracts/loader警告，未把全仓历史警告当已清偿。旧738测试曾默认覆盖tracked矩阵报告，已改为临时目录或显式AUTO_STDLIB_MATRIX_OUTPUT_DIR；原报告恢复，后续输出在scope。首次固定组driver默认落树内target/test-groups，收尾须搬入scope并删除，不留散落证据。
- Spec delta已在worktree准备并随提交冻结：SD-01..04，含新semantic-operation-contract、具体派发纠正与GOAL-002出处（目标状态仍部分达成）。暂无在合同内未完成项/延期/workaround；旧parser/公共入口/后继接线缺口在§5与Spec具名保留。

`stage: goal | plan_id: PLAN-943 | plan_revision: 1 | round: 2 | from: work | to: review | status: execution_done | outcome: pass | worktree: D:/autostack/.wt/lang-943/auto-lang | commit: bc9aa48c6a7b50762697e23fa466619375a4f2ae | next: review`


### 2026-11-05 独立 final review R1：needs_fix

`stage: review | review_scope: final | reviewer_identity: independent sub-agent /root/review_943（与实施上下文分离；同一唯一实施树；未暴露model/session ID，不虚构） | plan_id: PLAN-943 | plan_revision: 1 | outcome: needs_fix | worktree_path: D:/autostack/.wt/lang-943/auto-lang | branch: plan-943-dev | reviewed_commit: bc9aa48c6a7b50762697e23fa466619375a4f2ae | base_commit: a0750086c57c809a586884ac8b0519e8c162979b | dependency_revisions: auto-down=7c4d5cbb28d2093cc2e5dfb76d3104984a8dfe9c（只读） | pending_closeout_gates: 修复提交后受影响复审；final pass后merge-owned cargo t stdlib冻结完整并集；AC-06仍pending | next: work in same tree`

审查技能源 `C:/Users/zhaop/.codex/skills/auto-plan-review/SKILL.md` SHA256=`3d9263331895dd09cff5015783d9fc21b8be8441505d4101d52f8197d6d2a1f7`。入口/末尾HEAD相同，实施树clean；main在其他会话推进到61207576，但四份现行Spec从a0750086至main无差额，未动其他会话docs WIP。本轮只写本Plan与具名债务候选，不改实现、不merge。

| AC → task | 结果 | 实际核验及边界 |
|---|---|---|
| AC-01 → T-01/T-03/T-05 | pass（原分母证据保留） | 读真实scanner/AST提取、生产NativeInterface、schema3及inventory原始数据；失败层保留、alias逐名记录，provider目录未被当执行支持。独立合同族6/6重放含failed-layer/内联/spec/user-homonym合法对照。 |
| AC-02 → T-02/T-03/T-05 | fail | F-943-R1-01：删除可选native证据可绕过byte_len实际签名核验；未解析候选也无独立逻辑签名防漂移约束。 |
| AC-03 → T-01/T-03/T-04 | pass（冻结边界保留） | 检查17操作/政策、math.at实际i64公开签名/private helper、4producer未接线分类及owner；源前门合法int/非法i64探针及原始重放日志可追溯。未声称公共math已接ACE；len/parse_int/EOF/async旧差额均保留。 |
| AC-04 → T-04/T-05 | partial | 实际变更无公共API执行实现修改；独立872族6/6（含Unicode动态派发）通过；原完整stdlib147/147、24既有ignored证据可复用，但合同本身存在F-943-R1-01。修复后的最终固定并集仍由merge负责。 |
| AC-05 → T-06 | partial | 已独立审所有AC、实际调用路径、反例/合法控制、遗漏/延期/workaround及Spec；存在合同内缺陷，不能给final pass。 |
| AC-06 → T-07 | pending | merge-owned实际Spec沉淀/归档/清理未执行。 |

**F-943-R1-01｜P1｜original requirement gap｜AC-02/AC-04，T-02/T-03/T-05。** 违反原§7“签名/mode冲突具名拒绝”和T-02有界校验消费合同的要求，不能登记债务后当已完成。

- 真实输入/路径：复制当前stdlib到统一scope；修改`operations.toml`的`text.byte_length`：删除`native = "auto.str.byte_len"`，并将该项`returns = "int"`改成`"float"`，保持compatibility/units/dispatch等不变。`loader::scan_inventory` → `contracts::validate` → `audit_providers` → `stdlib_assembly_fingerprint(..., Vm)`。
- 预期：保留已冻结byte_length逻辑签名及1526实际provider证据；错返回应具名拒绝，删除佐证不能降低校验强度。观察：`CONTRACT_INVALID=[]; fingerprint_ok=true`。仅改返回、不删除native时，实际producer具名拒绝：`text.byte_length: signature/mode conflicts with native auto.str.byte_len (auto_lang_core::vm::ffi::stdlib::strings_json::shim_str_byte_len)`。
- 已定位原因：str.rs.at现为失败层，`validate`跳过其声明签名；`audit_providers`只进入`Some(native)`，而compatibility政策仅独立保存unit/dispatch，无签名/mode或绑定证据约束。故缺native将原本可独立核的首批身份静默降级。不是新公共API要求，也不要求修parser。
- 同因候选：`list.length`（失败层、native=None）由`[] -> int`改为`[Move str] -> float`后，也为`CONTRACT_INVALID=[]; fingerprint_ok=true`。该项原本source签名未验证可以保留unknown，但逻辑合同/政策应阻止静默自改，且应明确signature_unverified，不能将未知当实际声明证明。本支仅要求冻结逻辑合同一致性和验证等级，不新增泛型支持承诺。
- 合法对照：原合同通过；`auto.str.byte_len`改为同callee别名`Str.byte_len`通过；错native `auto.str.parse_int`因无独立producer签名被拒绝；单独错返回被拒绝。现有永久8类mutation未涵盖“删除证据+错签名”组合，故现有6/6不足证明该不变量。
- 关闭oracle/最小scope：真实disk scan及fingerprint对byte错返回、删除已声明native证据及组合具名拒绝；未解析candidate逻辑签名/mode漂移被独立政策拒绝，同时原版/合法别名/user-homonym/失败层unknown对照仍通过。新增永久用例进stdlib职责组，复审只重放共同不变量/原有合同族与必要siblings；不得每个finding附全库/全ACE门。

独立执行证据（全部在`D:/autostack/plan-artifacts/plan-943/`，merge折叠后可删）：

- `cargo t stdlib --test-name stdlib_assembly::contracts::tests:: --out .../review-contracts` exit0；实际selected=6、executed=6、excluded=0、errors=0；整命令31.46s。`review-contracts/results.json` SHA256=`b55a0c2f66c6418e06c63c14e402f60b8c44d37aa30bd67154e1d5e646b2317d`。未误把nextest输出中未选ignored行算已执行。
- `cargo t stdlib --test-name plan872_builtin_semantics_tests --out .../review-legacy` exit0；实际6/6、excluded0；含A中😀=8bytes、combining=3bytes、NUL、泛型CALL_SPEC=3及既有绑定/解析等守卫。
- 独立额外probe以本树当前`libauto_lang_core-1cf3bd112d292aa6.rlib`为依赖，用rustc构建到scope，运行`review-neighbors.exe`/`review-sibling.exe`；没有写实施代码。首次直接rustc缺native链接搜索路径而link失败（未算产品FAIL/PASS），补现有native搜索路径后成功。`review-neighbors.rs` SHA256=`d6e363e2edafa7045b2a38976aa57eae86da1c9efcdf385f8653bac89d0803cf`；`review-neighbors-combination.log` SHA256=`7e76c5c1fd125c12a5e91839a409753fa15578f3cd019fdb0524b639c0f6ffda`；`review-sibling.rs` SHA256=`4bcc76d7dd5dd59e87915e52e4d50807a291b229ad80c1acac97298b2f3a85ea`；`review-sibling.log` SHA256=`454bde8e2e1bae9d32e604a28e3bd3ed271cea5e204b32600374c8db1dbd2a09`。组合日志中的`missing_native`标签对应删除native+错返回，输入源码为准。
- `rustfmt --check --edition 2021`对本包8个已改Rust文件exit0，`git diff --check base HEAD` exit0。无新增debug语句；测试盘点println为具名机器证据。日志中的既有core145/facade32 warnings已明确盘点，本包contracts/loader未新增warning；未宣称历史warning全部清偿。

Spec核验：SD-01..04目标/GOAL-002身份正确，len具体派发与i64宽度、未接线分类/差额符合源码；现拟议文字的普遍“逻辑签名/mode冲突均拒绝”受F-943-R1-01影响，修复后须重核对应段及冻结delta hash，不可凭此R1授权发布。冻结`review-spec-delta.patch` SHA256=`e5fb6a8b0a3daf743aedea85646a64ce47cf2b04c8ace9a5827eb838696555a`。现行Spec输入SHA256：project=`d144494d7e3b5e2d913c76bfeae5c568ab8e3a6e6057517831ffdea1de2db512`；backend-assembly=`0038af77d5e5b2e98c6f09743193d63c43ec3191b4485e7d0d9159fd5f350657`；primitive-str-byte-len=`5376cd3671cbcddd8c367dab4b4e94bf73ecbea57b78aa68d3428ee84aa2e69e`；goals=`efb763125ff4e590dd1df715a95b6ceb3bd1be50115da70485f804eea8ce48f2`。不写legacy ledger。

遗漏/延期/workaround审查：未发现偷偷删旧required/accepted-diff/NA或API实现行为变化；候选/失败层及后继owner缺口已保留。唯一合同内开放finding为F-943-R1-01；T-02/T-03/T-05及其依赖证据重开，T-01/T-04保留完成，current_step=2/7，status=executing。T-06/T-07仍未完成。后续修复不重启无关完整campaign。



`stage: goal | plan_id: PLAN-943 | plan_revision: 1 | round: 3 | from: review | to: work | status: executing | outcome: needs_fix | worktree: D:/autostack/.wt/lang-943/auto-lang | commit: bc9aa48c6a7b50762697e23fa466619375a4f2ae | next: work`

2026-11-05 bounded repair cycle 1：`stage: work | plan_id: PLAN-943 | plan_revision: 1 | outcome: pass | worktree_path: D:/autostack/.wt/lang-943/auto-lang | branch: plan-943-dev | code_commit: f05b28eadc2862cb1a6a0e0c1929bff3fac7bc06 | dependency_revisions: auto-down=7c4d5cb (read-only) | task_ids: T-02/T-03/T-05 | blockers: none | next_responsible_role: final independent re-review`

F-943-R1-01关闭候选：

| finding→不变量 | 受影响分支/永久回归 | 当前观察/边界 |
|---|---|---|
| 可选证据删除不能降低合同约束 | compatibility独立保存parameters/modes/returns与所需native；missing_native、missing_native_wrong_return、byte_return_float、原native_identity | scanner出CONTRACT_INVALID，fingerprint Err；合法原版/Str.byte_len同callee别名仍接受 |
| 未解析candidate不能伪造签名或冒称checked | candidate_signature/candidate_mode，list.length及相同结构candidate；operation_proofs | 逻辑签名/mode仍与政策核对；实际层失败时明确policy_only_signature_unverified，byte_len另有producer_signature_checked_semantics_unverified；不新增公共绑定或修parser |

最小验证：`cargo t stdlib --test-name contracts:: --out .../r1-repair` 7/7通过，结果SHA256=23ad017df7d9ff3959f8265c3655923ec41d828b1d45c81e9beb481dc1d9b03b。复用R1独立probe原始输入源码，只改输出目录并重新链接实际新rlib：repair-neighbors.log SHA256=0f1e2d7391586881efacb0448f51de8c01a30b54710d970303197a0f0a015ffe（原版/合法alias两项fingerprint true；错返回/错native/缺native+错返回三项false）；repair-sibling.log SHA256=aecac8a81ee096c46d6bb0abe963daf4cf27fe794137a897e85d9f8dc14da433（Move str→float候选拒绝）。重放链接准备曾缺windows.0.52.0.lib搜索路径/显式toml依赖，均非产品语义失败；补齐当前依赖路径后成功，不计为原语义反例。

原提交bc9aa48c6、修复提交54c2234d2在同树rebase到主线61207576b（UI053 parser递归guard）后对应06a0f7d7f、f05b28ead；`git range-diff a0750086c..54c2234d2 master..HEAD`两项均`=`，无冲突。rebased冻结提交再跑`cargo t stdlib --test-name contracts:: --out .../r1-rebased` 7/7通过；实际122层/39失败分母未缩。原AC01/03与旧行为证据输入未变复用；最终完整stdlib并集保持merge-owned pending。Spec增量补充政策签名防漂移/所需native证据和逐操作证明等级，无验收/scope/revision变更。所有work-owned开放项已修复，等待独立核验。

`stage: goal | plan_id: PLAN-943 | plan_revision: 1 | round: 4 | from: work | to: review | status: execution_done | outcome: pass | worktree: D:/autostack/.wt/lang-943/auto-lang | commit: f05b28eadc2862cb1a6a0e0c1929bff3fac7bc06 | next: review`


### 2026-11-05 独立 final re-review R2：pass

`stage: review | review_scope: final | reviewer_identity: independent sub-agent /root/review_943（同R1独立审查上下文，与实施分离；无公开model/session ID） | plan_id: PLAN-943 | plan_revision: 1 | outcome: pass | worktree_path: D:/autostack/.wt/lang-943/auto-lang | branch: plan-943-dev | reviewed_commit: f05b28eadc2862cb1a6a0e0c1929bff3fac7bc06 | base_commit: 61207576b7676148499854f9df4b2d019a476972 | dependency_revisions: auto-down=7c4d5cbb28d2093cc2e5dfb76d3104984a8dfe9c（只读） | findings: F-943-R1-01 closed; no new findings | pending_closeout_gates: merge-owned cargo t stdlib完整冻结并集，AC-06实际Spec落地/归档/清理 | next: merge`

审查范围为R1 finding及共同不变量、修复回归/合法控制与修订Spec；复用R1未被修复改变的AC-01/03、872旧行为与原完整stdlib证据，不重跑全库/全ACE。实施写入在审查期间暂停，入口/末尾f05b28ead及clean状态一致；唯一树、分支、只读依赖经重新核验。审查技能及Spec输入沿R1原hash不变；四份现行Spec在base/main无并发差额。主线parser递归guard为本包rebase输入差额：两提交range-diff均`=`，实际inventory分母重算仍122层/77模块、39失败层、922声明/失败占位行。

| finding / AC | R2核验与结论 |
|---|---|
| F-943-R1-01 / AC-02 → T-02/T-03/T-05 | **closed/pass**：compatibility独立保留parameters/modes/returns和必须保留的native；逻辑签名/mode漂移与所需证据删除均具名拒绝。原生名字必须解析到政策绑定的同一实际ID，再核实际producer签名/绑定；合法同callee别名接受。不能通过删native降级。 |
| 同因candidate证明等级 / AC-01/02 | **pass**：`list.length=policy_only_signature_unverified`、`math.min=source_signature_checked_semantics_unverified`、`text.byte_length=producer_signature_checked_semantics_unverified`；仅政策一致不冒称source/实现语义验证，保持未接线与缺能力。 |
| AC-01/03 → T-01/T-03/T-04 | **pass，复用R1+身份重核**：公共API宽度、private可见性、len分派及旧accepted-diff/EOF/async边界未改。parser guard重算未缩分母。 |
| AC-04 → T-04/T-05 | **pass（work-owned正确性）**：修复只涉及合同验证/证据等级、政策与Spec，不修改公共API执行实现。R1旧行为6/6及原147/147完整组证据保留；R2针对修复7/7及额外控制通过。新冻结提交的最终完整stdlib并集仍须merge实际执行，未算本轮已绿。 |
| AC-05 → T-06 | **pass**：独立复审已覆盖R1全AC与本次受影响共同不变量、合法/非法近邻、遗漏/延期/workaround、健康及Spec；r1/SHA绑定如上。 |
| AC-06 → T-07 | **pending（merge owner）**：不得在最终并集门、Spec落地、归档/路径清理前宣称交付完成。 |

实际独立执行：`cargo t stdlib --test-name contracts:: --out D:/autostack/plan-artifacts/plan-943/r2-review-contracts` exit0，selected=7、executed=7、excluded=0、errors=0，整命令5.044s。`r2-review-contracts/results.json` SHA256=`e82b544cf31160f1e3ff9239e149da0482793414330f559304558ec66951aa9b`。永久mutation覆盖原R1反例及删除证据、删除证据+错返回、byte错返回、candidate签名/mode，合法alias和证明等级控制通过。

额外独立近邻（输入→真实disk scan→fingerprint，基于新rlib重链，输出全部在统一scope）：原版与`Str.byte_len`合法别名fingerprint=true、诊断空；仅删除native→false且`missing/unexpected native evidence required by compatibility policy`；删除native+float返回→false且具名逻辑签名/缺证据诊断；list.length仅returns int→float→false；text.equal保留参数/返回、仅首参数View→Move→false（避开arity/shape不合法的无关拒绝），均命中`logical signature/mode conflicts with compatibility policy`。三种proof等级上述值有独立断言。`r2-neighbors.rs` SHA256=`e1c20e766e960008a2167d97fe4088a09b66b79ef1229c4dbbc120ab13033053`；`r2-neighbors.log` SHA256=`342b7e9d147f6aaf0f125c8c05d06ac8638ffbc965cc24f4aed9e79047dab450`；`r2-review-inventory.json` SHA256=`ece83087c23c00dbf87ce4f45499371c1c66707897716194eb9fdb3128404888`。这些输入只检验冻结有限合同，不新增公共source/泛型/跨端支持承诺。

健康/遗漏：修复两Rust文件`rustfmt --check --edition 2021` exit0，`git diff --check 61207576b HEAD` exit0；无新增debug、合同外scope缩减/延期/workaround或ledger写入。既有warning盘点/边界沿R1保留，本修复未新增warning。R1唯一in-contract finding已关闭，无其他开放项阻止reviewed。

Spec SD-01..04已定稿核验：本次新增政策签名/所需native证据与operation_proofs文字符合真实消费/拒绝路径和验证等级；其他增量同R1，精确目标路径和GOAL-002元数据有效。冻结全部Spec diff `r2-review-spec-delta.patch` SHA256=`5d08282af74245ed0caff106c85b34d82b99fe24839e767790e74985fa156156`，替代R1未获通过的delta；当前canonical Specs未由review发布。语义验收范围/AC/plan_revision仍为1，修复是原不变量关闭而非改约。

T-06完成，status=reviewed、current_step=6/7；T-07/AC-06保持未完成。冻结受影响最终并集仅`cargo t stdlib`（stdlib assembly/政策/语义守卫职责）；没有本包parser/VM执行语义或ACE producer修改，不扩全ACE/全仓。merge完成冻结完整并集及实际收尾前不得落地或宣称delivered。



`stage: goal | plan_id: PLAN-943 | plan_revision: 1 | round: 5 | from: review | to: merge | status: reviewed | outcome: pass | worktree: D:/autostack/.wt/lang-943/auto-lang | commit: f05b28eadc2862cb1a6a0e0c1929bff3fac7bc06 | next: merge`

2026-11-05 merge checkpoint / needs_fix：最终冻结`cargo t stdlib --out .../merge-gate`父子exit0，148/148执行通过、24既有ignored（发现172），准备/发现1.310s、执行4.834s、总6.182s；results SHA256=cdae9ebf56e9355216fa4698f50e5ea961994cb6994f7c3068e02e1c44df6354。guard clean后已ff-only将f05b28ead落master（旧61207576b）；未归档/删树。随后收尾审计比对filtered/full两份inventory名称set发现F-943-M1-02，因此停止收尾并返回同树work，不能以已合入代替完成。

**F-943-M1-02 / P1 / AC-01,T-03/T-05 / original requirement gap**：audit_providers使用`id_to_name().into_values()`，但该反汇编tooltip投影按HashMap遍历每ID仅保留任意一个名称，额外registry/catalogue无法补齐仅final interface存在的别名。filtered/rebased盘点1407行、merge盘点1398行（如Math.abs/JSON.stringify/auto.auto.str.byte_len与Math.max/auto.fmt.eprintf互有漏列）；922声明/失败行与122层/39失败未变。不能把该随机行数当完整可重算native分母。独立review_943已只读确认同因；最小修复为只读完整name表accessor、audit逐名消费、单ID三个非catalogue/registry别名与生产全集具名set/id/bound/producer及重复排序守卫。无需改resolve/register/执行语义，不扩大到全VM/parser/ACE。

`stage: goal | plan_id: PLAN-943 | plan_revision: 1 | round: 6 | from: merge | to: work | status: executing | outcome: needs_fix | worktree: D:/autostack/.wt/lang-943/auto-lang | commit: f05b28eadc2862cb1a6a0e0c1929bff3fac7bc06 | next: work`



2026-11-05 bounded repair cycle 2：`stage: work | plan_id: PLAN-943 | plan_revision: 1 | outcome: pass | worktree_path: D:/autostack/.wt/lang-943/auto-lang | branch: plan-943-dev | base_commit: f05b28eadc2862cb1a6a0e0c1929bff3fac7bc06 | code_commit: 72ac9e7b341bd53de92cb53c9ecaf0716371a5a3 | task_ids: T-03/T-05 | blockers: none | next_responsible_role: final independent affected review`

F-943-M1-02关闭候选：`NativeInterface::registered_names()`只读枚举整个name_to_id键集；audit合并该全集与catalogue/registry并经BTreeSet排序，不调用tooltip的有损反向表。永久`all_registered_aliases_survive_one_native_id`对同6800三个专属名称逐名断言id/bound/无伪造producer，生产interface每个名字核id/bound/producer，重复audit序列完全相等。未修改注册/解析/派发/tooltip行为；只有库存元数据消费与Spec说明，固定完整职责组保持stdlib，无需扩大VM/parser/ACE。

定向`cargo t stdlib --test-name contracts:: --out .../alias-repair` 8/8 PASS（SHA256=c4c8e937e416bdcbaa9bdcfdcfb1f375510b19730486c39644df92d349dbcbb1）；进一步三别名/id/bound/producer完整断言后`--test-name all_registered_aliases --out .../alias-control-final` 1/1 PASS（SHA256=8aa8b00c8c5f6153e43fac1fbb0fc2e5d2b4be88bce5e8173961024136e93126）。rustfmt --check含native.rs通过，git diff --check通过。R2的签名防降级/单位/宽度/合法source/旧行为证据未受该只读枚举改动影响，明确复用；完整stdlib冻结门在R3通过后由merge重跑，前一merge-gate不覆盖新源码。source支持集、预算与plan_revision未变。

`stage: goal | plan_id: PLAN-943 | plan_revision: 1 | round: 7 | from: work | to: review | status: execution_done | outcome: pass | worktree: D:/autostack/.wt/lang-943/auto-lang | commit: 72ac9e7b341bd53de92cb53c9ecaf0716371a5a3 | next: review`


### 2026-11-05 独立 affected final review R3：pass

`stage: review | review_scope: final | reviewer_identity: independent sub-agent /root/review_943（与实施上下文分离，同一唯一树；未暴露model/session ID） | plan_id: PLAN-943 | plan_revision: 1 | outcome: pass | worktree_path: D:/autostack/.wt/lang-943/auto-lang | branch: plan-943-dev | reviewed_commit: 72ac9e7b341bd53de92cb53c9ecaf0716371a5a3 | base_commit: f05b28eadc2862cb1a6a0e0c1929bff3fac7bc06 | cumulative_diff_base: 61207576b7676148499854f9df4b2d019a476972 | dependency_revisions: auto-down=7c4d5cbb28d2093cc2e5dfb76d3104984a8dfe9c（只读） | findings: F-943-M1-02 closed; F-943-R1-01 remains closed; no new findings | pending_closeout_gates: merge-owned new frozen cargo t stdlib完整并集（预期149执行/24既有ignored，必须按实际结果记账），AC-06归档/清理 | next: merge`

只复核F-943-M1-02共同不变量、同因近邻和Spec补充，不重开R2已闭的签名/模式/单位/宽度/源码合法性及旧行为证明。已核唯一树/分支/当前SHA/clean及只读依赖，实施写入暂停；入口/末尾均72ac9e7b。技能源沿R1记录未变。f05b28ead已合main但未归档，不把历史落地代替本次修复验证。

| finding / AC → task | 证据与结论 |
|---|---|
| F-943-M1-02 / AC-01 → T-03/T-05 | **closed/pass**：`registered_names()`只读遍历完整name_to_id.keys；audit合并全名、catalogue、registry、源码声明后用BTreeSet去重排序，不再依赖每ID单名的tooltip反向投影。逐名resolve/get/contract仍取最终真实接口，不改执行语义。 |
| AC-01/04/05 → T-03/T-05/T-06 | **pass**：独立永久case验证同6800三个专属别名逐名保留、ID/bound/producer事实，生产接口全部显式名被报告，重复audit序列相等；额外两份新建production接口验证HashMap seed不同也不丢全名/不改排序或事实。 |
| AC-02/03及AC-04其余面 | **pass，复用R2适用证据**：修复仅只读枚举与盘点消费，逻辑合同/签名证明等级/旧API执行实现未变；R2合法/非法合同近邻、宽度/len差异、原完整组证据保留。前merge-gate148/148是旧f05b28ead证据，不能覆盖新冻结源码。 |
| AC-06 → T-07 | **pending**：新完整stdlib门、ff落地/实际归档/路径收尾由merge完成。 |

独立执行：`cargo t stdlib --test-name all_registered_aliases --out D:/autostack/plan-artifacts/plan-943/r3-review-alias` exit0，selected=1/executed=1/excluded=0/errors=0，整命令2.449s；结果SHA256=`abf730c447d52fa58333ad543f6f0265a46ca4757a4c60aed60e83ae2296927f`。检查了永久断言，既核手写三名字oracle，又逐生产注册名核ID/bound/producer，并比较整个序列，未以计数代替完整集合。

独立同因近邻使用本树新rlib，创建两份新production接口，对两个显式名集合及最终审计(name,id,bound,producer)序列进行全等断言，并逐名核最终事实、严格排序无重复。观察：`fresh_interface_explicit_names=1022 audited_union_rows=1146 sorted_unique=true complete=true identity_bound_producer=true cross_instance_sequence_equal=true`。这里1146为**空源码fixture**的registry/catalogue/interface并集，不是全库API/源码声明分母；实际全库表由merge单独盘点。所有输出在scope；`r3-fresh-interfaces.rs` SHA256=`7f0e8099e8a72a410976a9383f3034cc3a401523f7d937f0a3495cf1e2d75b1f`，日志SHA256=`cb4ed87ac14cf835dc28adedd0b2dc0192c0ff1bb69b6e29b4ef5bf354ba6d98`。无需重复跑全VM/parser/ACE。

健康与Spec：`rustfmt --check --edition 2021`覆盖本次三个Rust文件（含native.rs）exit0，`git diff --check f05b28ead HEAD` exit0；没有新增debug或执行路径副作用，既有warnings沿前轮具名边界保留。新增Spec两句准确说明完整注册集合与同ID事实；不改变source支持、原accepted-diff/NA、预算或revision。current主线semantic-operation-contract输入SHA256=`f648097847ab9f36a9fe06579155bd14e5d156e50b374939ec9aa539050b1733`（R2版本），其他四份Spec输入沿R1不变。本包累计SD-01..04冻结diff `r3-review-spec-delta.patch` SHA256=`6fc26b5c337b8bdb86f6f6b0cf29898ae99f65ab0214421652abb7110046efea`，替代旧delta身份；metadata路径/GOAL-002仍准确。不发布canonical Spec、不写legacy ledger。

遗漏/延期/workaround扫描无新发现；F-943-M1-02原漏列机制已整体移除，三专属别名证明不依赖catalogue/registry偶然补漏，fresh-interface近邻覆盖随机迭代因子。T-06完成、reviewed/current_step=6/7；T-07保持未完成。merge写所有权交回，须在本次冻结SHA完成完整stdlib并集与实际收尾后再报告delivered。



`stage: goal | plan_id: PLAN-943 | plan_revision: 1 | round: 8 | from: review | to: merge | status: reviewed | outcome: pass | worktree: D:/autostack/.wt/lang-943/auto-lang | commit: 72ac9e7b341bd53de92cb53c9ecaf0716371a5a3 | next: merge`

2026-11-05 merge prepared/landed/canonical_verified：R3 final pass已绑定r1/72ac9e7b；新完整固定门`cargo t stdlib --out D:/autostack/plan-artifacts/plan-943/final-gate`父子exit0，发现173=149执行+24既有ignored，149/149 PASS，not_run=[]、errors=[]，既有ignore/资源条件保留。准备/发现1.157s，执行4.994s，总6.190s；runner清理完成。results SHA256=116a1005f81271408458ebd7fa642a884acbb8fab72f45d130d8642590628dab。

最终实扫122层/77模块、922声明/失败占位（39失败保留API未知）、1431原生名称/别名=1142真实bound+285unknown+4resolved-unbound；37模块provider声明保持声明身份；17操作及逐项proof未冒称语义认证。final-inventory.json SHA256=0a47674a98872ef9a4d9822731c870ae75a55e9ff0c8c07c3c79d57373d942d7。前轮1398/1407等native行数为有损投影历史，已由F-943-M1-02修复取代，不能用于当前分母。完整同ID全别名、最终id/bound/producer及稳定set/序列均由永久case和独立R3核验。

`wt-guard`在landing前报告clean；master先61207576b→f05b28ead、后f05b28ead→72ac9e7b均ff-only，无merge commit；与R3冻结源码/配置/只读auto-down依赖一致。所有canonical工作副本`git hash-object`与72ac9e7b的Git blob逐一相等（主检出换行可与实施树raw SHA不同，内容来源按Git blob核）：

- docs/specs/stdlib/project.md SHA256=590459e1657951bcd0c09c73b309b5fa4af0c6aa03a2de0fad1eb7d3e850e659 git_blob=6be3b98a05f75c637b9224fdb60dfa77a1df2563
- docs/specs/stdlib/design/backend-assembly.md SHA256=4ec34035399560f697301ddbd418a1c0d4640ec263d04ecf00d2d027a00b672b git_blob=c54429c881262957d4aec2d3410af7a3218bab4d
- docs/specs/stdlib/design/primitive-str-byte-len.md SHA256=367db3dcad01be7e08e28eb6708a0bcb4f1d80179f61125e386af820e02a6dd0 git_blob=4447ff0584d7baa888f4f34f99a1b8a58c173a78
- docs/specs/stdlib/design/semantic-operation-contract.md SHA256=2d44c8304d5f8022b1f53fb046ecb1227e457f08067758bf09e8fde7344f675f git_blob=ae477b77068d3b55b672a6a8d236f77fb19fd07f
- docs/specs/goals.md SHA256=b58eed597abe4da67094fdc735f8cfbf5be60088162bf824fbd1f7e944002172 git_blob=3f9d78f97053c22b226949b807d75029da22e655

归档交付类型`completion_kind: delivered`；T-07清理尚待下文实际收据完成，保留实施树直到归档提交。其他会话主检出docs草稿/WIP原样保留，只有本Plan与本Plan债务段进入簿记提交，源码/Spec路径无未提交改动。未写legacy specs.json/应用索引。

路径对账：本Plan正常构建在专属树target、scripts/full-regression-driver/target；所有source-profile/独立rustc probe/exe/pdb、验证日志/矩阵与合同副本在统一scope。首次driver的target/test-groups/run-tjp34w1d输出已迁至scope/initial-fixed-group，9文件/1,454,731 bytes，逐项SHA256一致（manifest在scope）；旧738报告写入已恢复且永久改为临时目录/显式scope输出。无外部安装/消费副本或其他盘符输出。永久测试使用的临时fixture由tempfile自动回收。独立fix-943-preflight-temporaries scope由用户具名持有7原始文件，不随本Plan删除；这是开工安置，不是本Plan构建留存。

部署观察：PATH上的auto为`D:/autostack/auto-man/target/debug/auto.exe`，UTC构建时间2026-11-05 11:19:39，早于交付，尚未重编译/重启；本包没有auto-lang release/auto.exe或web bundle发布任务，auto-man等外仓未修改。当前交付是源码/SDK合同，不宣称运行中工具已热更新。批量due核验：.last-batch-regression.json覆盖937、timestamp2026-11-05T12:21:00Z，本次943非%5且收据未超48h，窗口内未合入新的%5语言Plan，无需再跑cargo t all；已有known_reds_only历史不当成本包PASS。



外部source-profile前探的精确narrow输入（宽近邻把全部int注解换为i64），用`ace_core::source::compile_scalar_source(src, module_path, &[])`观察；int接受scalar-i32，i64拒绝source.unsupported。仅证明源码域边界，不宣称std.math调用/执行已支持：

```auto
fn min(a int, b int) int { if a < b { return a }
 return b }
fn main() int { return min(3, 5) }
```

2026-11-05 cleaned / delivered：archive提交4742ec1741f9123dd792e803acf8e816bc7e332e已在master；实际archive gate核status/location/本地链接/跟踪/72ac9e7b祖先通过。两实施树立即移除前guard均clean，auto-lang 23,262文件/5,854,123,838 bytes、只读auto-down 1,250文件/26,384,862 bytes随worktree remove删除；plan-943-dev已git branch -d，空lang-943组已删除，未回收构建产物。

统一scope删除前无活跃进程，路径恰为D:/autostack/plan-artifacts/plan-943，guard clean；预清理测量4,174文件/1,747,250,930 bytes，新增一份清理数量清单后全scope共4,175文件一并删除（约1.75GB）。首次包含Force的组合删除命令被自动策略拒绝，未执行任何部分；完成独立路径/进程/guard核查后，以显式literal路径、不带Force的Remove-Item -Recurse成功清除。实际Test-Path/exists核scope不存在，树/组/分支均不存在。代码/临时输出/安装根路径已逐项对账，无其他盘符或外部消费根残留。用户持有的fix-943-preflight-temporaries仍保留7原文件和manifest，owner/原因/关闭条件在登记中，未以归档释放。

证据摘要与所有必要命令、exit、关键计数、区分输入/结果及原始哈希已折入本Plan §9；原scope不再是可用留存件。权威登记D:/autostack/plan-artifacts/.retention/policy.json刷新为deleted-at-merge，并记录最终Plan内容hash；初期误写到D:/autostack/.retention/policy.json的本任务两条登记已撤回、迁入正确索引，其他旧条目保留。scope内probe/安装验证副本/target/exe/pdb均已删除，无本Plan具名未解决问题持有。

AC-01..06最终均已闭合，T-01..07=7/7；F-943-R1-01与F-943-M1-02各经独立复审/永久反例关闭。主检出本Plan实现/Spec/归档范围干净；预先存在的其他会话docs草稿/WIP保留并未混入本次提交，未声称全仓这些文档也已提交。源码交付commit=72ac9e7b341bd53de92cb53c9ecaf0716371a5a3，canonical identity及149/149门见上，陈旧auto-man工具观察仍如上；未擅自部署/跨仓重建。

`stage: merge | plan_id: PLAN-943 | plan_revision: 1 | outcome: pass | completion_kind: delivered | delivery_commit: 72ac9e7b341bd53de92cb53c9ecaf0716371a5a3 | archive_path: docs/plans/archive/943-stdlib-semantic-contract-and-compatibility-inventory.md | canonical_verified: SD-01..04 | cleanup: tree/branch/group/scope removed; registry refreshed | pending_merge_gates: none`

`stage: goal | plan_id: PLAN-943 | plan_revision: 1 | round: 9 | from: merge | to: done | status: archived | outcome: pass | worktree: removed | commit: 72ac9e7b341bd53de92cb53c9ecaf0716371a5a3 | next: done`

## 10. 待澄清事项

2026-11-05 用户答复：“它们是临时产物，移到独立暂存目录后继续”。已逐文件迁出至 `D:/autostack/plan-artifacts/fix-943-preflight-temporaries/`（独立于本计划交付材料），7/7 SHA256 一致，原路径与大小/哈希见该目录 manifest.json；登记为用户临时产物持有，不随 943 merge 删除。主检出 examples/scripts/test 路径已无未提交文件，开工阻塞解除。943 材料 scope 已建立并登记，work/T-01 开始。其他会话 docs WIP 保留，不纳入本计划提交。

- 首批符号/合法source/profile/接口位置已在§5/机器合同冻结；缺能力留具名后继进入条件，不构成943待决事项。
- 完整CAP仍守Design46 A/B/C1–C3门；本包无权提前完整求解或以候选字段宣称已实现。
- 实施授权与临时产物安置问题已关闭，work完成；当前无等待用户决定的阻塞。未指定预算/模型/角色不虚构约束；跨仓若后续需写代码，由对应 owner 另建合同与授权。
