---
plan_id: PLAN-899
status: executing
feature_name: ACE组织收尾、历史probe清退与行号级调试
author: [Codex]
created_at: 2026-10-31
updated_at: 2026-10-31
plan_revision: 2
current_step: 2
total_steps: 7
supersedes_spec_components: [docs/specs/auto-test-case/build-storage-lifecycle.md]
new_spec_components: []
touched_goals: []
affects: [experimental/ace, test-infrastructure]
depends_on: [PLAN-896, PLAN-898, PLAN-891, PLAN-900]
---

# [PLAN-899] ACE组织收尾、历史probe清退与行号级调试

## 0. 变更摘要

r2依据900/891已交付、901在途的真实基线修订原计划，不另取号。ACE已是独立workspace，现有30成员（原r1的29成员加891交付的typed-entry-runtime）、单Cargo.lock及ace-*包名；本包不是再次统一workspace，而是清理实际无长期职责的旧probe、修正当前说明、成员dev/test保行号且非成员无调试，并安全核清旧缓存。

保留独立workspace、实现职责、语言隔离与全部有效测试/oracle/正式样本；四个原no_std运行时加891新typed-entry运行时的panic-abort合同全部保护。debug=0改行号会增加成员调试信息，不把这一项宣传为减量。

r2实施已获用户 /goal 授权（2026-10-31，见§4.1与§9收据），当前executing、2/7：T-01审计与T-02 profile/README已在lang-899树交付（代码提交416060511）；T-03/T-04等U901（901仍executing未合入master），届时续work。审计结论：本轮19 bins+2 fixtures全有真实职责或消费者，零可删；两处断链/落后文档（verify-av脚本旧bin名、KNOWN-DEBT旧路径）归T-03原子修复。

## 1. 目标

- G1：单workspace及现有依赖方向保持，活跃README/manifest/工具说明与30成员和当前固定组一致，不再把语言all描述为包含ACE。
- G2：一次性研究工具有据退出，有效worker、ABI oracle、资源负控、原始样本producer及长期诊断保留或按职责迁名，实际消费者不破损。
- G3：ACE dev/test成员debug=line-tables-only、非成员debug=false；full-debug成员debug=2可用，原release/syntax-release及各运行时专用profile保持。
- G4：main残留旧缓存按精确owner/祖先/活动/证据守卫核清；真实回收与完成必要构建后的稳态分列，无法证明安全的scope具名保留，不承诺固定节省量。

非目标：并入语言workspace/搬出experimental、大crate化、改算法/ABI/schema/正式门限或禁用慢测、缩样本/规模、拍平ACE内部feature、代901重做语言feature退出、共享可写target、清其他树/全局Cargo或sccache、修改别的活动Plan、AG/UI产品、远端发布。

## 2. 架构方案

实现根保持experimental/ace/{crates,tests,grammars,docs}，普通target与target-batch保持原用途。普通语言测试不隐式编ACE；显式ACE消费仍通过原worker及父侧判定，不复制runner/计量器。默认成员保持当前5个轻包，新增运行时不进入普通unwind测试编译形态。

保留fixture/oracle与长期诊断，不以probe/bench文件名、年龄、耗时、归档状态或rg零命中决定删除。改名只改工具定位和活跃调用，不改cteworker/1等协议、recipe/身份字段、原始样本schema与历史证据。

本任务L1，沿896现有架构与Spec。只有实物核查发现必须改变隔离/ABI/支持范围才回报修订，不自动扩成架构重做。

## 3. 技术栈

复用Rust/Cargo/nextest、固定cargo t组、现有PowerShell验收/父分析器、scripts/test-governance/{toolchains.json,prepare_tools.py,build_storage.py}。不创建动态目录、叶ID/影响图、新测试runner或缓存框架。

## 4. 需求分析与背景调查

### 4.1 授权、来源、worktree

用户此前提出ACE组织、probe清退与行号配置目标；r2会话明确“核查并更新899”仅授权规划修订；2026-10-31用户以 /goal 完成计划899 授权全链实施（work/审/有界修复/merge，含建树），本节与§9首条goal收据记录该授权。当前不从“OK”推导删缓存或联系其他会话授权。主检出已有其他docs/计划/号源WIP，保全，不代提交。

已读docs/specs/overview.md、auto-ac/project.md、auto-test-case/build-storage-lifecycle.md，896/900最终记录、891当前交付与901合同及其实际提交。知识冲突优先最新用户仓规与已交付源码；Spec尚记29包/4运行时是落后快照，列入本包必要沉淀，不倒写成已测。

实施唯一树D:/autostack/.wt/lang-899/auto-lang，分支plan-899-dev（2026-10-31已建，基面master@03797707b）；组内兄弟auto-down@7c4d5cb(detached)与auto-web@c48924f(detached，只读依赖——ACE经auto-lang-core→auto-web-contract路径依赖需组内../auto-web解析，实测缺它cargo全workspace解析失败)；实现、测试、独立复审同树顺序。main仅计划/规范簿记及merge阶段经当前主检出清理器证明安全的精确旧缓存处置，不在main实现/跑产品测试。

### 4.2 本次只读基线（2026-10-31，开工重核）

| 实物/输入 | 观察及使用限制 |
|---|---|
| main | 03797707b0e86cbb3bd48e88a9282de5c5c0cab2；891已归档交付并清理，不再是r1草案。898、900均已交付，不保留“898正在执行”旧前置 |
| ACE根 | 30个显式成员，5个default-members；dev/test debug=0，尚无package.*调试override；full-debug=2，release=false，syntax-release=true，runtime-dev/release panic=abort |
| 新运行时 | crates/typed-entry-runtime，staticlib；native/src/typed_runtime.rs通过runtime-release+指定triple/target构建和来源hash消费；不能漏出profile/ABI保护范围 |
| 901 | 唯一树lang-901/auto-lang HEAD fe825a67e4832a8186b419b8daa6655df5218143，当前tracked diff为空（仅当次快照，不保证后续不变）；计划头仍executing1/8，但已有6677bf7bb、fe825a67e实现提交，不能按计数认定“尚无代码” |
| 实际901交集 | 6677bf7bb改experimental/ace/tests/conformance/Cargo.toml、scripts/test-groups.toml、toolchains.json及Core ac_av/collections/ace_certify等；语言test-*转发被移除，ACE自身batch/真实code-editor仍保。树与main另有891分歧，不能把整段diff都归因901 |
| CTE worker | comptime/src/bin/cte_probe_worker.rs，正常worker不是一次性打印探针；comptime tests/CARGO_BIN_EXE、conformance host、Core ac_cte.rs、toolchains.json与verify-ac-cte-817.ps1均实际引用 |
| AV样本producer | av/src/bin/av_runtime_825_bench.rs与av_wire_826_bench.rs，代码标10 warmups/30正式samples、失败行也输出。scripts/verify-av-{825,826}.ps1与av-{825,826}-bench-report.ps1等父分析链需完整追踪，不得因名称旧而删除/减样本 |
| mem_stage_probe | av/src/bin/mem_stage_probe.rs。KNOWN-DEBT 798 R12-04仍记公开前端分阶段内存诊断，旧引用路径已落后；该未闭归因使它成为保护候选，不能以静态消费者少为由清退 |
| ABI/资源fixture | abi/cside/layout_probe.c由contract_layout_admission.rs真实引用；native/tests/fixtures/collections_runtime_probe.rs由collections_resources_tier.rs真实编译消费，必须保留 |
| 活跃README | experimental/ace/README.md仍写all含ACE并引用已退役attachments/896摘要；需改为当前语言all排除ACE及归档计划内摘要，不重写历史合同 |
| 清理设施 | build_storage.py已存在exact target、--retired-source祖先owner、lease/活动/reparse/证据守卫；不应为899另造宽泛删除器。scripts/ace-cargo.ps1在main已不存在，不能用旧入口起跑 |

来源SHA256：experimental/ace/Cargo.toml=94667adad88661cc9e22c5145d5b14c1642fc0a77f67444cc4a3b161a38d6e6a；ACE Cargo.lock=5B2A4A69487241C53DA5A1C24F156DED93E040178E20C5AC4F3959BFC4D34073；test-groups.toml=F618C6C584C142019571756B60F4F60BF907B170CB279EE6241490C45DEB182D；toolchains.json=45C270E8BE53F2B5E8BAE898F9C724C830BE4B66E790B97CC0247ECE2CFF1BDB；build-storage Spec=8DC45792DD3210B575BE37A033C0EA7015F6D8D1584CCED891E123FAF40ED573。

旧11目录experimental/{ac-artifact-probe,ac-av-probe,ac-case-worker,ac-core,ac-cte-probe,ac-mir-native,ac-semantics,ac-syntax,auto-a2x,auto-abi-probe,auto-mir-lower}本轮均仍存在，顶层无Cargo.toml；不止顶层target：ac-av-probe/probe/target、ac-core/test-support/target及runtime/text、probes/data-resources、ac-mir-native/runtime/collections仍有残留目录。git ls-files核ac-av-probe/ac-core/ac-mir-native旧路径无跟踪项，但这不证明未跟踪内容都可删。

r1“51.8GiB”是旧只读逻辑统计，不是当前占用或可释放承诺；本轮只列浅层路径，未递归遍历大缓存、未重测容量/唯一物理数据、未查进程cwd/env，不宣称闲置可删。新ACE当前跟踪835文件（不含构建物），成员/源码数量不作为测试分母。

### 4.3 与901的实际并行边界

| 类别 | 899处理规则 |
|---|---|
| 可先做的独立段 | 用户授权后T-01只读审计、旧缓存来源/保护盘点，以及T-02独立ACE profile/当前README修订，可在独立lang-899树推进；不借901 WIP、不改其材料/target |
| 共享写面 | CTE改名会改toolchains.json，必要Core ac_cte.rs、固定组/prepare_tools/verify接线；901正修改相同设施/相关语言接口，T-03这些写入及最终组合门等U901正式落地后再做，接受其实际版本 |
| profile与测量交集 | profile变更使重新构建身份/体积/时长变化，不能把901旧计量认证为899新profile；同树不共享target，正式性能/峰值/大构建窗口须可归因，撞窗口不停止他方进程 |
| 终态 | T-04最终完整常规ACE/桥接/新运行时消费者依赖稳定901组合；不靠899独立profile提交就宣布全计划完成。U901不是T-01/02调查/独立修改的全局前置，但T-03共享接线与T-04是明确等待点 |

因此899仍可作为独立段候选推进，但不是完全无交集的并行计划。当前只修订，不安排任何实施或跨会话消息。

## 5. 详细设计

### 5.1 清退与命名审计

T-01清点tracked源码、Cargo自动发现bin、fixtures、README、脚本的静态/动态调用、CARGO_BIN_EXE/默认路径/env/tool身份、当前债务和正式验证链。一次性记录真实路径、长期职责、有效消费者/未解决问题、保留/改名/迁长期测试/删除、理由及精确验证入口；不是运行前置目录，不把所有工具一概保留当完成。

当前分类：CTE worker、两AV benchmark、C ABI/collections fixture保有效职责；mem_stage_probe保留为诊断候选并核现行债务是否仍需。没有任何文件在本轮被认定“可立即删除”；具体删除项必须T-01给使命结束/真实等价替代/无未解决问题及无必要消费证据。动态调用和手动文档入口也算消费者，rg零命中不够。

沿r1拟将cte_probe_worker改为ace-comptime-worker，但须在T-03接受901后同步Cargo自动发现/显式bin、CARGO_BIN_EXE、worker发现/default path/env与来源hash、toolchains、verify-ac-cte-817.ps1及活跃帮助。保持cteworker/1、AC_CTE_CASE_WORKER及父侧判定不变；若外部受支持旧名字确需过渡，给具名兼容/退出条件，不保两份实现。AV旧编号bench可按长期职责命名，但样本/schema/父分析器与原入口迁移必须原子闭合，不能为避验证直接放弃r1的命名清理目标。

当前README/manifest改为实际长期职责；历史experimental/ace/docs/*/contract-decision.md和归档计划保原历史身份，不全仓替换历史probe字样。KNOWN-DEBT只更新当前必要定位/交接，禁止顺手把内存问题标已闭。

### 5.2 调试profile及运行时

ACE根[profile.dev]/[profile.test]显式debug=line-tables-only，两者package.*设debug=false；仅非workspace依赖受通配override。full-debug成员保2，恢复依赖完整调试须明确调整override，不能声称单个环境变量能覆盖通配设置。runtime-dev继承新dev行号并保panic=abort，runtime-release/syntax-release及普通release的debug/优化/断言/溢出/incremental不变。

保护现有collections/data/text/trace以及891新增typed-entry运行时（具体五包由T-01再核清单和真实消费者），仍走原runtime-dev/runtime-release+triple。不得把no_std静态库误塞普通测试unwind构建、改默认成员或把核心protocol测试关掉。profile成本/体积在同环境单列，增加行号可能增大文件，不预设总体节省。

### 5.3 精确空间收尾

候选仅§4中实际旧Cargo缓存根。T-01列每个嵌套target并给各自迁移前祖先manifest证明；一个祖先不能默认证明所有嵌套root。无manifest历史证明、含源码/报告/未解决问题或活动引用的目录保守拒绝，不为能删而扩清理器allowlist。

复用build_storage.py默认preview；确认精确owner、祖先和当前lease/进程cwd/argv/env/依赖引用后才--apply --idle-confirmed。不能以“没cargo.exe”或无跟踪文件代替占用核查；不杀别的会话进程。main清理必须由main的有效脚本/真实main路径重新预览，不能拿lang-899树结果认证主检出旧缓存。

实施树完成必要构建后的稳态按profile/deps/incremental/PDB/其他分账；main历史缓存回收独立记唯一文件数据/失败/保护项，不把逻辑长度求和冒物理释放。现存旧scope已被其他收尾删掉时记录already_absent，不重建假缓存补“回收”正控。只删除核实为空的旧父目录，未知残留不盲删。

### 5.4 暂存、收尾与恢复

开工创建并登记D:/autostack/plan-artifacts/plan-899/，owner=auto-lang/PLAN-899、用途及在途状态明确。除本计划文件外所有审计表/探针/脚本副本/日志/样本/收据/发行或外部消费者副本均在该scope；正常target留唯一worktree，确需外置的计划target只能在scope登记。不能另建attachments、.wt/evidence、其他盘符散落根；现有工具输出路径不符合时显式指定，必要最小输出参数适配在T-03列明，不默许旧默认泄漏。

merge把必要命令/退出码/关键数字/代码或日志节选折叠计划本身，逐项核对实际命令/日志中全部输出路径，核归属、链接、活动引用及唯一依赖后删除整个plan-899 scope及worktree构建物。仅具名未解决问题可保留，必须owner+理由+关闭条件和实际路径入计划及.retention/policy.json；不再最近5个/14天/exe永久保留，不写旧specs.json。没有实施树的本轮规划不创建证据scope。

改名/配置回退采用本计划自身提交，旧缓存删除只影响可重建物；重建入口/源码/lock保留。不能为了回退还原已移除语言test-*或倒退901公开合同；不reset/stash他方WIP。

### 规范增量

| delta_id | add/modify/retire | docs/specs/... target | before/after rule | rationale | acceptance IDs |
|---|---|---|---|---|---|
| SD-01 | modify | docs/specs/auto-test-case/build-storage-lifecycle.md | ACE成员30/五运行时实际保护；dev/test成员0→行号、依赖false；full-debug/专用profile真实覆盖 | 用户调试要求及891新基线 | AC-01/02 |
| SD-02 | modify | docs/specs/auto-test-case/build-storage-lifecycle.md | 补最终probe/worker长期职责、精确旧缓存处置、当前统一暂存和merge即删政策 | 防误删有效oracle/诊断，保899结果边界 | AC-03/04/06 |

## 6. 测试设计

所有以下未执行。本轮没有cargo metadata/list/build/test，也没有清理preview/apply。work先给必要完整组并集及理由；review独立上下文在同一权威树核AC/删除/延期/SD/健康；实现者自检不冒独立终审。本次不创建agent/新聊天，不指定模型。

| ID | AC / 入口 | 可区分错误实现的oracle | owner_stage / 边界 |
|---|---|---|---|
| V-01 | AC-01；cargo metadata --manifest-path experimental/ace/Cargo.toml --no-deps --format-version 1 --locked；普通语言固定组静态/真实准备核 | 30基线成员减具名退役项、同workspace_root；typed-entry成员不丢，语言普通入口无ACE worker准备；明确default成员保持 | work；不生成永久动态目录 |
| V-02 | AC-02；受控verbose build及test-no-run的代表成员/依赖、full-debug、runtime-dev/runtime-release | 实际rustc参数和必要对象调试信息：成员行号/依赖无调试、full-debug成员2、五运行时abort/profile不变；不能只读TOML或看Cargo exit0 | work；正式release-only消费者不证明dev ABI已测 |
| V-03 | AC-03；cargo t ace、cargo t ace-runtime；新增typed-entry的真实native/宿主消费者；逐项删除/替代账 | 所有必需原ID/子断言/样本/条件守恒；CTE改名从父侧prepare到真实IPC成功，缺worker/坏路径/坏envelope仍拒绝；835文件数不是测试分母 | work；profile触及完整ACE常规，原ignore/平台限制保留，真实typed消费者必要时显式ace-native/ace-host组 |
| V-04 | AC-03；受影响verify-ac-cte-817.ps1、verify-av-{825,826}.ps1及父bench-report脚本/原命名原子 | 10 warmups/30正式样本与全失败行保留、父分析器原schema/分母一致；缺行/错身份/错误worker负控真实失败。仅改名亦须完整原子，不以--help替代 | work；必要cargo t ace-performance仅实际触面；不减样本，不把常规绿代替专项 |
| V-05 | AC-04；build_storage.py精确preview→保护/活动核→apply→后验及原守卫控制 | 活动/源码/证据/reparse/无历史owner拒绝；main实际回收/已不存在/保护/失败分列，实施树稳态独立；无假空目录“释放” | merge；如当前活动无法证清，具名hold该scope，不绕过守卫 |
| V-06（r2新增） | AC-01/03；接受901最终组合后Core adapter/工具准备、CTE父侧默认发现和显式env覆盖 | 使用901新常驻接口、准备命令不再请求退役语言test-*；能运行真CTE，缺worker具名失败，无旧exe fallback；显式ACE与普通语言隔离保持 | work；不复写901 feature逻辑，不全量UI/AG |

固定组实物存在ace/ace-runtime/ace-comptime/ace-av/ace-native/ace-host/ace-adapters/ace-batch/ace-performance（T-01接受新基线再核）。profile变更须完整常规ACE及运行时必要消费者，不只几条小测试。正式计量样本/阈值/Job/输出预算/串行条件保持；未运行专项只有在源码/依赖/工具/相关profile/条件身份均有效时可复用，profile变化影响的旧计量不能冒新证据。

列用例枚举/实际执行/原条件排除；准备/冷编译/发现/运行/清理/整命令分列。首次红、修复和环境重放分列；真正新增失败须修，既存红按同身份对照具名，不以归档笼统免责。普通语言只跑受影响必要消费者，不追加全AG/全应用或每步cargo t all；周期回归主检出due规则独立。

## 7. 验收标准

- [ ] AC-01（owner_stage: work）：现有30成员单workspace/五运行时职责、普通语言隔离保持；当前README/manifest/入口不再误述旧all/结构/附件，V-01/06有证。
- [ ] AC-02（work）：dev/test成员行号、非成员无调试，full-debug成员2可显式用；原release/syntax-release/runtime abort与真实typed-entry消费者合同保持。V-02不只清单证明。
- [ ] AC-03（work）：所有删除/改名逐项审计，使命结束或长期替代有证；有效CTE/AV样本/诊断/ABI/资源义务保持；V-03/04/06实际终态及负控成立，无缺项/空选/减样本。
- [ ] AC-04（owner_stage: merge）：main精确旧缓存安全处置或具名合理保护、失败/已不存在如实记录；回收及完整构建稳态分列，无他树/证据误删。V-05证明，不要求未知目录硬删。
- [ ] AC-05（owner_stage: review）：独立上下文逐AC/删除/遗漏/延期/workaround审查pass；零本包新增未处理warning/格式/debug输出，既存边界具名，SD真实可沉淀。
- [ ] AC-06（owner_stage: merge）：冻结版本及必要有效验证落地、canonical Spec回写、archive归档、guard清树和统一scope所有输出路径对账/删除完成；仅具名未解决持有，应用派生索引，不写specs.json。

## 8. 执行步骤

| ID | owner_stage / 依赖 | 路径、动作与交付 | AC / 关闭证据 |
|---|---|---|---|
| T-01 | work / 用户批准r2实施 | 以§4 main/891基线建唯一lang-899树、登记plan-899；审计实际probe/worker/父分析链/债务，旧main精确嵌套target来源与保护项；冻结共享写面和具体验证命令 | AC-01/03/04；V-01基线和逐项判定，无工具先删再找替代。[✅ 已完成] 树lang-899/auto-lang@03797707b+兄弟auto-down/auto-web(detached)；scope plan-artifacts/plan-899登记（.retention/policy.json auto-lang/PLAN-899）；审计表t01_audit.md：19 bins+2 fixtures全有真实职责/消费者（cte worker 7类消费点、mem_stage_probe为R12-04改进机会复现工具保留、AV双bench/863console/layout_probe/collections fixture有效）→零可删；断链2处（verify-av-825.ps1:144旧名av_runtime_bench、verify-av-826.ps1:103旧名av_wire_bench）+落后文档2处（KNOWN-DEBT R12-04旧路径、README four/attachments/all失实）归T-03；主检出16个嵌套target根全在f1493989e(=1795038c9^,HEAD祖先)有manifest证明、0跟踪文件、dir /AL无junction、~49.5G du逻辑量（不作回收承诺）；T-06冻结命令build_storage.py --retired-source f1493989e先preview后apply |
| T-02 | work / T-01；不依赖U901共享接线 | experimental/ace/Cargo.toml及当前README落成员行号/非成员false、full-debug说明和30成员/五运行时；不动锁版本/默认成员/真实能力 | AC-01/02；V-02代表实际参数；最终证据待T-04稳定组合，测量避901窗口。[✅ 已完成] 提交416060511：dev/test根profile=debug="line-tables-only"+两者[profile.*.package."*"]debug=false（通配只匹配非成员依赖）；V-02实测rustc实参（plan-artifacts/plan-899/v02_*.log）：dev成员ace_syntax_grammar/runtime=line-tables-only而auto_atom/auto_val/blake3/serde等依赖无debuginfo；test成员test目标line-tables-only、blake3强制重编仍无debuginfo；full-debug成员=2依赖仍ABSENT；五运行时（含891 typed-entry、text-runtime crate名ac_text_runtime）runtime-dev=staticlib+panic=abort+line-tables-only、runtime-release=panic=abort+opt3+无debug；V-01复跑cargo metadata --locked=30成员/5默认/同workspace_root/零警告；README修all排除ACE（实测all=34语言组0 ACE组）、五staticlib、行号说明（明确为增量不宣传减量）、归档896计划引用 |
| T-03 | work / T-01；共享写入必须U901 | 先接受901正式交付并同步本树，依审计表改CTE/benchmark工具定位与bin/fixtures/manifest/README、Core ac_cte、toolchains/固定组及verify消费者；有据删一次性工具；必要输出参数统一scope | AC-01/03；V-03/04/06；无cteworker/1语义/样本变更，901私有实现不被覆盖 |
| T-04 | work / T-02/03 + 稳定901组合 | 冻结提交跑完整常规ACE、ace-runtime及必要typed/adapter/CTE/bench完整原子；记首红/修复/重放、计数/分段时间/稳态，SD提议 | AC-01..03；V-01..04/06，不用旧profile结果替新结果 |
| T-05 | review / T-04 | 同树独立上下文逐AC、删除与反控/健康审查，核SD/债和当前清理候选来源 | AC-05，revision-bound证据；需修回原树，未来事项不预勾 |
| T-06 | merge / T-05 | 冻结有效门与合入，main每精确旧target即时preview/活动/持有核→apply/后验；SD01/02沉淀，真实回收与保护摘要折计划 | AC-04；V-05；来源变化重preview，不停他方任务 |
| T-07 | merge / T-06 | git mv至docs/plans/archive、置archived；摘要折叠；统一scope/全部散落实际路径核清及删除/具名hold；各树即时wt-guard clean后移除树/分支/空组 | AC-06；最新政策不再最近5个，legacy ledger只读 |

当前下一步是U901（901正式合入master）后续work T-03共享接线与T-04最终门（2026-10-31核查：901仍executing@fe825a67e未合入，T-03/T-04为明确等待点）。不联系901会话或向其发消息；以正式代码/计划实际交付结果作为接收依据。

## 9. 复审记录

2026-10-31 / stage: new / PLAN-899 r1 / outcome: pass（当时草案自查，非实现验收） / 原提交内容blob 08f9bce775fdf2181d3c664204ea13483c7f4c58；原基面ee0a1059e、29成员/4运行时、51.8GiB旧逻辑统计、898在途与最新5个保留说明均为历史。r1未实施、无完成项，本次没有抹去执行/审查证据；§0–8/10由r2替代为唯一当前合同。

2026-10-31 / stage: new（revision） / PLAN-899 r2 / outcome: pass（规划合同核查，不是独立实施review） / 用户授权仅核查修订；已核891新增成员/typed runtime、901实际提交共享写面、CTE/样本/诊断/ABI消费者与旧嵌套缓存目录；未编译、测试、清理。T01..07与AC01..06保持稳定ID，新增V-06及共享前置，SD01/02扩清当前元数据和暂存政策；next: 用户确认r2实施后T-01/02，可独立段推进，T-03/04等U901。仅修改本计划，无号源/产品/canonical/WIP改动。

2026-10-31 / stage: work / PLAN-899 r2 / T-01+T-02 / outcome: blocked（T-03/T-04外部前置U901未满足；T-01/T-02自身pass）/ worktree: D:/autostack/.wt/lang-899/auto-lang（组内兄弟auto-down@7c4d5cb、auto-web@c48924f detached只读依赖）/ branch: plan-899-dev / base: 03797707b / code_commit: 416060511 / deps: auto-down 7c4d5cb、auto-web c48924f（Cargo.lock未动）/ task_ids: T-01,T-02 / 证据: plan-artifacts/plan-899/{t01_audit.md, v02_dev_syntax-grammar.log, v02_test_syntax-grammar.log, v02_full-debug.log, v02_runtime-dev.log, v02_runtime-release.log, v02_runtime_4runtimes.log}；V-02 rustc实参全五运行时/成员/依赖/full-debug覆盖，零cargo警告（text-runtime 4条memcmp/memcpy符号签名lint为既存边界，profile无关）/ blockers: T-03共享接线（toolchains.json、test-groups.toml、Core ac_cte.rs、verify接线）与T-04最终门需U901=901正式合入master（2026-10-31仍executing@fe825a67e）；解除动作=901合入后同一目标续work T-03 / next_responsible_role: work（U901后）

2026-10-31 / stage: goal | plan_id: PLAN-899 | plan_revision: r2 | round: 1 | from: work | to: work(暂停于U901) | status: executing | outcome: waiting | worktree: D:/autostack/.wt/lang-899/auto-lang | commit: 416060511 | next: user（U901=901合入master后resume）

2026-10-31 / stage: goal | plan_id: PLAN-899 | plan_revision: r2 | round: 2 | from: work(暂停) | to: work(暂停) | status: executing | outcome: waiting（等待轮1/3） | 核查：master=edeb3f343（仅本计划簿记增量）无901合入；901仍executing@fe825a67e。用户续令为"等待901合入后resume"，未批准改序。无可做的合同内独立段（§4.3独立段T-01/02已毕）。本轮零工作零检查除门禁状态重核 | next: user（U901后resume续T-03）

2026-10-31 / stage: goal | plan_id: PLAN-899 | plan_revision: r2 | round: 3 | from: work(暂停) | to: work(暂停) | status: executing | outcome: waiting（等待轮2/3） | 核查：master=603681b74（仍仅本计划簿记）无901合入；901仍executing。门禁状态与round 2一致，无新证据，无合同内可推进项 | next: user（U901后resume续T-03；若下轮仍闭则goal置blocked）

2026-10-31 / stage: goal | plan_id: PLAN-899 | plan_revision: r2 | round: 4 | from: work(暂停) | to: blocked | status: executing | outcome: blocked（等待轮3/3触发上限） | 核查：master=49b0f3e45 无901合入；901 worktree lang-901/auto-lang 仍在@fe825a67e、计划仍executing；本计划检查点完好（plan-899-dev@416060511、scope已登记、T-01/02证据在案）。三轮连续等待无新证据，按goal循环上限置blocked。精确解除条件：PLAN-901正式合入master（U901）后 /goal resume 本目标→同树续work T-03（CTE改名原子接线+verify-av断链修复）；或用户明示批准"899先行于901基线"合同修订后按修订推进 | next: user

## 10. 待澄清事项

1. T-01已结：19 bins+2 fixtures全有真实职责或消费者（cte worker 7类消费点、AV双bench为正式样本producer但父脚本断链、mem_stage_probe为R12-04改进机会复现工具、ABI/资源fixture被真实消费）——本轮零可删，具名断链/落后文档修复归T-03。后续若审计再发现可删项仍须逐项使命结束证明。
2. CTE/bench新bin名与受支持旧消费者兼容已按T-01冻结触面清单（t01_audit.md §D：7类CTE消费点+verify-av两处断链），T-03接受901后原子接线；协议/env/样本身份不擅变。必要外部消费者超出既有边界时先呈报具体合同差额。
3. 旧main缓存来源已由T-01证明（16根全锚定f1493989e，无junction，无源码/证据混入抽查）；字节/进程引用/lease活锁由T-06 preview+守卫机检，历史51.8GiB与当前du~49.5G逻辑量都不作可回收承诺。
4. 若用户要并入语言workspace或移动experimental，须另审依赖/ABI/profile/默认成员与验收，这不属于r2等价整理。
5. 【等待用户/U901】T-03/T-04为901前置等待点：2026-10-31核查901仍executing（worktree fe825a67e，master=03797707b未含901）。解除动作：等待PLAN-901正式合入master后，以同一 /goal 目标resume续work T-03；或用户明示改序（如授权899先在当前基线做T-03再rebase 901）则属合同修订需另行确认。不联系901会话。
