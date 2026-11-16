---
plan_id: PLAN-1034
status: archived
feature_name: ACE worker源码快照与父子有界终态
author: [Codex]
created_at: 2026-11-13
updated_at: 2026-11-16
plan_revision: 5
current_step: 3
total_steps: 6
depends_on: [PLAN-1024]
supersedes_spec_components: [docs/specs/auto-test-case/architecture.md, docs/specs/auto-test-case/a2r-hir-certification.md, docs/specs/auto-test-case/a2c-hir-collections-certification.md, docs/specs/auto-test-case/a2c-hir-scalar-certification.md, docs/specs/auto-test-case/a2c-hir-record-text-certification.md, docs/specs/auto-a2x/python-stdlib-readiness.md]
new_spec_components: []
touched_goals: []
affects: [ACE]
---

# [PLAN-1034] ACE worker源码快照与父子有界终态

## 0. 变更摘要

2026-11-13 r4当前：r3复审发现内联1MiB额外收缩、70s整check硬截止超出设施能力、certify-c遗漏golden_path有界读取，以及把v2-native 1MiB计量当A2X证据。已删额外内联门、细分70s交换/3s收尾、补全文件输入并区分正式campaign入口。 本轮只改计划；drafting 0/6及原Task/AC/Case编号保留，无实现/测试/配置/canonical Spec改动或ACE执行。

承接[1024](1024-ace-m2-code-quality-and-refactoring-audit.md)的F10/F18/F19/F27，优先级**P1（F10/F19策略为P2）**。本轮仅立规划合同，drafting、0/6；未建树、未写实现/探针/fixture、未构建或运行ACE。用户当前仅授权复审与规划，不授权本包代码实施。真实动态影响由T-01反例界定，不把静态风险冒实际失败。

2026-11-16 r5阶段裁定：F10/F18/F19/F27全部A-required，先关闭原IPC-01..03及r4新增Python控制与对应独立oracle再供985/A；源码一次快照与hash、IO分类、三个父adapter和有界交换/清理是985证据可信基础，不能在不可信父侧上签A。原6任务/0完成/既定设计与历史保留，仅规划。

## 1. 目标

编译文本和报告SHA来自同一次有界读取；父侧在等待/双流收集/超时/清理均有受控终态；协议IO/EOF/编码及cap口径明确而不默改旧wire。

非目标：扩大语言/ABI/平台支持、默认切换、删除oracle、降低样本/规模/原条件、重写全ACE或接管其他Plan。限额/字节口径已在§5冻结，保既有样本/规模；不声称当前可绕过认证。reader共性抽取限实际兼容子集。

## 2. 架构方案

A2X一次有界bytes快照供解码/hash/compile；两个daily父adapter采用auto-tool-execution现生命周期的opt-in cap扩展（拟新增），沿现Windows Job/Unix进程组收尾，父请求送stdin。三line保各自framing/编码，本包不抽跨crate统一reader、不升级wire。

沿1024真实producer→consumer分层，恢复不授信，source/typed、guest错误/Resource/Cancelled/Infra分账。不因共用文件扩大范围。§5已固定策略及跨平台设施；work只选等价内部细节，不能改默认cap/协议相容或语言支持。

## 3. 技术栈

Rust、现固定组/原TestCase及Markdown。仅语言仓。后续唯一树D:/autostack/.wt/lang-1034/auto-lang，建议分支codex/plan-1034-dev，尚未创建；work/review/merge顺序使用同树，禁止链接。材料唯一D:/autostack/plan-artifacts/plan-1034/，创建即登记owner/用途/状态，当前无scope。

## 4. 需求分析与背景调查

授权：2026-11-13用户要求先复审1024，再逐发现分配到已有/新有界计划，明确优先级与1002关系，暂不改实现。无指定预算或自动施工授权。r1固定源Git=81d8fc03f10f4a7d179b0bc3a917602406ed7b91（历史）；r2历史取材Git=c2760324da25795594ab0cc01b581e3a306d2375，具体文件身份以本节SHA为准；1024 r2审查包与源码分开绑定，开工重核配置/输入/来源漂移。旧反例是静态证据，不是运行日志。

已读docs/specs/overview.md作导航；当前规则以module Specs为准，原始证据及测试入口如下：

| 已核源路径 | 字节SHA256 | 状态 |
|---|---|---|
| [experimental/ace/crates/a2x-worker/src/bin/a2x-worker.rs](../../experimental/ace/crates/a2x-worker/src/bin/a2x-worker.rs) | e368a4701c917e94444742fe19798616663e01c5b1ab9807ee680ec4a572e2e7 | 静态取材，非运行证据 |
| [crates/auto-lang-core/src/test_cases/ac_a2r_hir.rs](../../crates/auto-lang-core/src/test_cases/ac_a2r_hir.rs) | 5b6e456b7001bba4716485e0628c7944b5d7aa7faa1ac004eb6be18ae4e409e0 | 静态取材，非运行证据 |
| [crates/auto-lang-core/src/test_cases/ac_a2c_hir.rs](../../crates/auto-lang-core/src/test_cases/ac_a2c_hir.rs) | 0a7dbe916c97599074401dccf7d14eac01dd3a0418a171b75d775eb1e0968dbf | 静态取材，非运行证据 |
| [experimental/ace/crates/case-worker/src/main.rs](../../experimental/ace/crates/case-worker/src/main.rs) | 3bbb71456402bbdb80657c0f54b206fdf261c292fc40627e53bbe6d4d8004f91 | 静态取材，非运行证据 |
| [experimental/ace/crates/semantics-store/src/ipc.rs](../../experimental/ace/crates/semantics-store/src/ipc.rs) | 78cda8649d995d296b45843bb34c1c983adaf65008e8282e55684249adb93fa6 | 静态取材，非运行证据 |
| [experimental/ace/crates/native/src/bin/mn-collections-worker.rs](../../experimental/ace/crates/native/src/bin/mn-collections-worker.rs) | bfcc75ac5f2357d15967ef2fab634312039f5ece033e4960e5dee752bbda4f81 | 静态取材，非运行证据 |

| Spec来源 | SHA256 | 适用边界 |
|---|---|---|
| [docs/specs/auto-test-case/architecture.md](../../docs/specs/auto-test-case/architecture.md) | 2e37d29c0df993f95809bf262af6a53b4b53acebf353453dcfc3e1626bc89fe4 | 当前规则保持；开工重读核漂移 |
| [docs/specs/auto-test-case/a2r-hir-certification.md](../../docs/specs/auto-test-case/a2r-hir-certification.md) | a32d3555b5657cb72c2ae909ce2ac000110662ef5aa46e218f96c7895561581a | 当前规则保持；开工重读核漂移 |
| [docs/specs/auto-test-case/a2c-hir-collections-certification.md](../../docs/specs/auto-test-case/a2c-hir-collections-certification.md) | 94e00af47b2015bcf0290138d831ddc2b013755df09a51d636ce28c0286ef256 | 当前规则保持；开工重读核漂移 |

源码/Spec是实际规则依据。1024 M/S/D/L/A/N/U为保留账，不代Rust reader/nextest发现；同文件在途修改串行单写，正式合入来源重核，不用他Plan WIP当已交付依赖。


### r2来源差额与执行入口预检

当前取材c2760324da25795594ab0cc01b581e3a306d2375；r1关键文件无差额的证据可保留，实际有差额者已回读并按本轮合同修正。预检发现：r1既未定义one-shot/line各自的字节口径，也把auto-tool-execution泛称成已有带cap执行器；该库的run_command_capture实际无默认输出上限。

| 现有测试/补充实现入口 | SHA256 | 本轮用途 |
|---|---|---|
| experimental/ace/crates/a2x-worker/tests/conformance.rs | 0eba766d1bcc2b9bd2633570f0aa9c747e2d2712c2d2de72cc5c0774ff0c901e | 已存在；T-02在原模块扩展或接入，不冒已创建测试 |
| crates/auto-lang-core/src/test_cases/ac_a2r_hir.rs | 5b6e456b7001bba4716485e0628c7944b5d7aa7faa1ac004eb6be18ae4e409e0 | 已存在；T-02在原模块扩展或接入，不冒已创建测试 |
| crates/auto-lang-core/src/test_cases/ac_a2c_hir.rs | 0a7dbe916c97599074401dccf7d14eac01dd3a0418a171b75d775eb1e0968dbf | 已存在；T-02在原模块扩展或接入，不冒已创建测试 |
| experimental/ace/crates/core/src/link.rs | 7e19a7d49894d30a4d20297c36690559e5dc3dff70f15922f54fa1a361cda0d0 | 已存在；T-02在原模块扩展或接入，不冒已创建测试 |
| crates/auto-tool-execution/src/lib.rs | 7b367bea468594217c3f209828b3e68853f7f9272df497fe92ad901e4cd4aada | 已存在；T-02在原模块扩展或接入，不冒已创建测试 |

1028已由用户启动实施，本轮未写其计划或工作树；不要仅因其主检出frontmatter尚未更新就判断未开工。与它共享的计量结果在后续验证时绑定实际已合入SHA，本包不新增1028整包硬依赖。963/947等新源与本批修复共用位置时按正式合入基线串行重核，主检出其他文档WIP保留。


### r3补充依据与来源绑定

r3静态取材Git=6584e94d465dc1f8b2a1cd58344913de4d5ed18a；§4原r2 SHA表是历史快照。本轮重核原表及下列补充来源，当前字节身份见§9核验记录。以下是规则/代码证据，不是运行结果。

| 补充来源 | SHA256 | 作用 |
|---|---|---|
| crates/auto-test-case/src/admission.rs | 504616a6a1e22c9226c95e0265a1815b1a5d32d5b65ce5f65cf74a615a113b7f | 文档/inline/sidecar默认与硬限额，不能混指源码 |
| crates/auto-test-case/src/contract.rs | a8a8d4bbd5d1b4df46aaf7b77908d9c04582312f3398ccd0221ead3686ecd696 | 70000ms total档 |
| docs/specs/auto-test-case/profile-v2-native.md | 839355e7ccf00cf47b24301c02978e9c000cb73cda8b97656dabbde06f1cccc1 | 1MiB程序流与4MiB元数据/64/128MiB产物分离 |
| crates/auto-tool-execution/src/lib.rs | 7b367bea468594217c3f209828b3e68853f7f9272df497fe92ad901e4cd4aada | 跨平台生命周期已有，cap扩展是拟新增 |
| crates/auto-lang-core/Cargo.toml | def80d45d2369fa0d94aad5dc7a91938151c7ff8cbba8cc5930b29b3506baa9c | SDK不依赖ace-core |
| experimental/ace/crates/core/src/link.rs | 7e19a7d49894d30a4d20297c36690559e5dc3dff70f15922f54fa1a361cda0d0 | 非Windows containment stub拒绝，不能直接替换父执行器 |


### r3当前文件样本静态盘点

按git ls-files中.case.ad文件筛recipe字段ac-a2r/hir-*或ac-a2c/hir-*，逐source.file及expect.artifacts.file读取bytes（不运行reader/worker）：64个case文档，66个recipe字段匹配：Rust scalar7 / record-text9 / collections20；C scalar5 / record-text5 / collections20。所有source均为file，未解析/缺文件=0；计入重复引用的源码总11758B，最大533B，路径=crates/auto-lang/test/cases/a2c-hir-collections-926/src/N889-nested-variant.at，SHA256=4b6433b03528d986ef67edf9d7420409f8e7d64ebccafceadf9fc4d23da6e61f。最大单golden=28996B（a2c-hir-scalar-921/golden-C01/a2x_c_runtime.h），最大case golden总量=47224B（ace-m1/source-query-934/source-query-934.case.ad）。按排序case路径逐项记录recipes、source路径/大小/hash、artifacts路径/大小并以json.dumps(sort_keys=True,ensure_ascii=False)编码的清单SHA256=3b755fec800d8e3a70c064d832b89b817644bfe40ee2a52f06bfb17123588a7d。该词法盘点不是case/check引擎执行数，不覆盖动态生成formal负载；1MiB字节边界控制另由§5转义界和未来重放守恒。


### r4复审补充依据

基线Git=09119bd3cface39a8c794e54f9dacdd3f7dd4452，主检出D:/autostack/auto-lang，branch=master。本次按现Spec/代码重新构造反证；r3结论不作为证据。关键补充来源如下（SHA256为文件字节）：

| 来源 | SHA256 | 核查用途 |
|---|---|---|
| experimental/ace/crates/a2x-worker/src/bin/a2x-worker.rs | undefined | source分支优先级、certify-c第三文件golden_path |
| crates/auto-tool-execution/src/lib.rs | 7b367bea468594217c3f209828b3e68853f7f9272df497fe92ad901e4cd4aada | 3s per-path宽限及两轮cancel非总3s |
| experimental/ace/crates/a2x-worker/src/bin/a2x-formal.rs | 749fb39307dcae13db14c1f0b752cddc6a43f15b6f6ed699ccefc4d45392b06c | session API正式入口不同于worker JSON |
| experimental/ace/crates/a2x-worker/src/bin/a2c-formal.rs | aa90b3898385a16abbacc3d9697624613dea64c94b5b1c0a8324582217a38c1d | C正式入口不自动受JSON门 |
| docs/specs/auto-test-case/a2c-hir-scalar-certification.md | 6f052e5215baeef5ce67b4ed1a25c1fc07ab00a152c3b5702d7e393d0cabe564 | 同一C adapter的scalar责任 |
| docs/specs/auto-test-case/a2c-hir-record-text-certification.md | 86f5049f6d698590df6b630bf920ff2d02e642874fa0a1fc540f097ff50e24ff | 同一C adapter的record-text责任 |

本轮授权与来源（2026-11-16）：用户只要求重冻985/A验收合同、核1024及审计后继和1066的A/B边界，明确任务/关闭条件并更新A顺序；不启动实施，不新增Plan。读取主线107b23d0f69c6ad11d036dc37bed0b688493db30、当前Spec导航/相关模块合同、1024 F01..35/S01..13、1027/1028归档收据、1029..1038和1066当前合同及实际源码消费者。静态风险不是已经复现的bug；A行须真实修复后验证或可复核充分反证，未执行保持pending，不能以本轮分类当PASS。原资源/平台/性能/ignore与UI三波边界保持。

## 5. 详细设计

唯一范围=F10/F18/F19/F27。关闭行必须连接“发现→现代码不变量→反例→真实consumer→永久断言→修复提交”，承接登记不算关闭。T-01使§6全部case在原入口可运行。

数据/错误/责任：编译文本和报告SHA来自同一次有界读取；父侧在等待/双流收集/超时/清理均有受控终态；协议IO/EOF/编码及cap口径明确而不默改旧wire。 A2X一次有界bytes快照供解码/hash/compile；两个daily父adapter采用auto-tool-execution现生命周期的opt-in cap扩展（拟新增），沿现Windows Job/Unix进程组收尾，父请求送stdin。三line保各自framing/编码，本包不抽跨crate统一reader、不升级wire。 confirmed/candidate/rejected/unknown沿1024分开；充分反证需保源码/合同依据，不能删分母。当前测试与新增反例分别保完整身份、源hash、expected、条件；共享机制不共享被测oracle。


### r4执行合同（继承r2输入，以下决定优先）

**输入到消费路径。** 生产入口仍从AC_A2X_HIR_WORKER及可选AC_A2X_HIR_WORKER_SHA256选择预构建worker；负控stub仅测试setup提供并固定其hash，case文件不能任意指定exe。Rust/C和新Python父adapter各自调用真实run_check/manifest比较；one-shot main保argv或stdin整JSON，不能套line reader。

**状态、故障与责任。** 选择auto-tool-execution现跨平台生命周期中的显式opt-in cap扩展，现run_command_capture仍无默认cap；具体接线/拒绝ace_core依赖的原因已在下文决定。全局env负控测试隔离，不能修改用户环境。

**决定状态。** DEC-IPC-LIMIT按r4修订闭合，r3四项设计缺陷与关闭证据见§9；测试的小cap/短deadline不成为产品默认。
### r4限额与执行责任决定 DEC-IPC-LIMIT（复审修订后设计闭合）

状态：2026-11-13 closed/design-static-r4。下表是本Plan拟交付的新资源policy，非A2X旧限额或v2-native自动适用规则。r4删除无额外依据的source_text 1MiB门；保留的request8MiB、文件64MiB、输出/产物门是明确新增资源收缩，不能承诺旧无界实现接受的所有输入均保持。T-01只补实际设置/漂移与既定反控，不再任选数值。MiB=1048576字节，等于上限可进入下一准入步骤，超1字节即资源拒绝；格式正确与资源足够是分别判断。

| 通道 / 对象 | 冻结值与计量口径 | 依据 / 作用点 |
|---|---|---|
| A2X一次请求，argv与stdin两入口 | 原始JSON UTF-8字节≤8MiB=8388608，含空白/尾LF，不含传输EOF；stdin有界读取cap+1后即拒，argv先按OsStr借用长度准入再创建String；必须读完整个one-shot文档，不能把第一行当请求。 | 新策略，数值参照TestCase文档硬上限8MiB，但对象不同。不是8MiB源码上限；超限在serde解析前拒绝。 |
| source_text | 与source_path采用同一源码内容上限64MiB；另受整个请求8MiB约束，实际可内联大小取两者交集。解码后的原始UTF-8内容用于hash/compile；不再另加1MiB字段上限，不强制原可内联输入转文件。 | 新策略。TestCase inline1MiB是作者文档admission的边界，不能移为私有worker字段限制。1MiB+1的ASCII源码及其小于8MiB的请求可正常进入编译；64MiB源码不保证能装进8MiB JSON信封。 |
| source_path文件源码 | 原始内容≤64MiB=67108864；一次有界快照转UTF-8并hash/compile；metadata只早拒，仍以实际cap+1读防文件增长。 | 新worker资源policy，宽于当前TestCase sidecar硬上限4MiB，但不由产物64MiB推导为旧输入合同。原直接worker >64MiB内容将新增Resource/Infra拒绝；原source_path优先级保持，未选择的source_text不另作前置内容门。 |
| certify/certify-c的reference_path | 原始参考源码≤64MiB，同一次有界读取/UTF-8/原字节hash；不制造空参考或hash(empty)。 | 新policy，同一参考输入拒绝IO/编码/超限；数值不改正式oracle/样本，现已承诺样本须按真实certify入口重放。 |
| certify-c可选golden_path | 若未提供，保持证书该项None；若提供，实际UTF-8文件≤64MiB并有界读取/hash；缺文件、IO、坏编码、超量明确Infra，不能用.ok()退成“未声明”。 | 新policy，补r3遗漏的第三种文件输入。该文件仅显式认证的hash键材，不参与生产emit；daily父侧golden仍绝不传worker。 |
| Rust/C emit与Py emit的worker stdout | 原始响应JSON元数据≤4MiB=4194304，含尾LF；父侧收集过程中强制，早于JSON解析/trace完整响应复制。 | 新策略，采用v2-native IPC元数据档。该stdout只含protocol/recipe/identity/manifest/files元数据；生成源码文件不在stdout，不套程序输出1MiB门。 |
| daily worker stderr | 原始字节≤1MiB=1048576，父侧即时强制，不经lossy转换后计数。 | 新策略，采用现程序流资源档；cap+1立刻触发owned树终止与ERROR，不等待自然退出，不以截断文本成功。 |
| daily实际文件 | 单文件≤64MiB、单check总量≤128MiB=134217728（包括manifest），manifest/文件清单元数据≤4MiB；父侧有界读取、checked累加后hash/独立golden比较。 | 本包采用v2-native产物档作为A2X新策略。生产者报告bytes只作声明，父侧核真实字节、规范相对路径/唯一集合/域边界，不能信文件len后无界read。oracle本身仍受原TestCase准入，不传child。 |
| daily worker交换与清理 | exec_deadline=owned执行入口开始+70000ms，约束启动后的stdin发送/关闭、worker等待和双流收集；终止触发后共享一次cleanup_deadline=触发时刻+3000ms，含两轮kill/reap、reader/writer退出与guard兜底，所有等待取同一剩余量，不每路再加3s。成功必须含清理在70s内完成；超过70s后只准清理并ERROR。 | 新policy，选择70s交换档与3s总收尾档，非v2-native“整个check70s无宽限”合同。timeout/cap先保原失败因，再记清理成功或具名残留；实现不得以现工具库多轮TEARDOWN_GRACE自然相加冒单一3s。不宣称硬实时返回或OS调用可被中断。 |
| formal certify既有轴 | 保原compile120s、run30s及certify整体360s合同/既有父门和流上限；本Plan的daily70s与metadata4MiB不覆盖其目标程序输出规则。 | 原合同保留；既有ace-perf样本/规模/平台/ignore不变，不能用daily cap替代正式campaign。原目标stdout/stderr各1MiB仍是程序流。 |

同一policy各最大值不是“同时取最大也必过”：source_text先受request8MiB约束，64MiB是选中源码内容门；source_path和未选source_text的现优先级不变。1MiB最坏JSON源部分6291456B，最小信封6291474B，只证明这一边界在8MiB内，不证明A2X正式campaign已执行/必绿。TestCase默认文档2MiB/inline256KiB/sidecar1MiB及硬上限8MiB/1MiB/4MiB由原admission继续执行，不改作者格式。

#### 父执行器与跨平台选择（已决定）

选择在现auto-tool-execution的同一run_prepared/CaptureWorker生命周期中增加**显式opt-in有界捕获入口**（拟新增，包含per-stream cap、exec deadline和单一cleanup deadline选择，不是现有API），沿原Windows Job/Unix进程组与cancel/reap/pipe-poll实现。auto-lang-core仅增加对该轻量工具库的依赖，不引入ace-core编译器到语言SDK普通构建闭包；Rust/C及新Python三个父adapter共同消费这一入口。现run/run_command_capture等默认无截断合同保持，新cap只由本包调用方显式传入，不改变所有工具命令。

不选ace_core::link作为两个语言父adapter的依赖：它的capped executor在非Windows明确拒绝spawn，而且SDK目前没有ace-core依赖；worker certify内部原有该入口继续保持。本包不迁移certify、不新造第二套Job/进程组，不等待新增平台能力计划。

70s预算从调用该owned执行入口前创建，不能从case started机械倒推，更不能只传None deadline。父请求统一经已有ExecutionSpec.stdin_data的owned写入机制送完整JSON并关闭stdin，以免Windows argv长度门在8MiB请求前先失败；worker原argv入口继续兼容且同限额。预构建worker选择/可选SHA credential、显式cwd/env和WorkerExchange身份保留；绝不把case指定exe变生产能力。stdout/stderr在read和stop-drain两条路径均须在扩容前核cap，缓冲最多保存cap字节、计数checked/饱和保cap+1事实，首个溢出通知现调度循环立即终止自有树，不能无界drain后才判断。IO/reader panic/writer/kill/reap失败均独立Infra；不能用进程exit0覆盖。3s单一收尾截止必须传到异常/正常EOF收集、两轮cancel和Drop兜底；cleanup失败须记录未确认reap/PID/线程状态，不写“保证全收敛”，也不自动续宽限/隐藏后台reaper。原无限cap调用仍用旧合同，新增路径仅在同一生命周期内扩展。


#### 父侧阶段责任及相容边界（r4已决定）

父run_check的worker凭证读取、source staging、事后manifest/实际文件/golden读取比较是同步父阶段；有界字节读取、checked总量与阶段时间记录必做，能轮询处响应cancel，但本Plan的owned子进程设施不能硬截止一次阻塞的父文件IO。分别记录prepare、exchange、judge、cleanup及整check耗时，不再声称整个run_check≤70s。若产品将来要求全部父IO也受可抢占墙钟约束，需要另立受控执行边界，不在本Plan偷偷加第二个worker/违反daily单spawn。

当前auto-tool-execution的TEARDOWN_GRACE=3s用于多处stop_and_join/cancel、cancel_within含两轮终止；它只是可复用基础，不是已满足本Plan新单一3s清理预算的API。实施必须落实opt-in期限传播及结束状态，不可只给旧run加70s后写PASS。正常成功无截断；任何阶段IO或实际文件超限仍为ERROR。

相容结论：现64个文件case的静态大小均低于本policy，但不是旧worker接受集合的最大值证明。r4固定测试1MiB+1 ASCII inline可接收、选中path时忽略未选inline额外内容门；>8MiB请求、>64MiB选中文件、超流/产物以及超过交换deadline则是明确的新资源拒绝。它们不改变语言支持或变成syntax负例。取64MiB输入档是本Plan的保守资源策略选择，不冒原规范数值，也不声称“绝不影响所有历史合法输入”。

正式证据按入口分开：v2-native的gen796-1mi/v1归其原recipe；A2X load256/30采样/31修订、certify120s/30s/360s及其它已冻结campaign照原入口保留。a2x-formal/a2c-formal可直接调用session API，不自动经过本worker JSON/path门；既不能拿它们的绿证明IPC cap已覆盖，也不能给它们套daily70s。已有1MiB字节边界作为本包正确性控制保留，当前不声称它本来就是A2X正式规模。


#### 复审期间新合入的Python consumer（r4漂移复核决定）

1042现已正式合入同一a2x-worker，新增daily-emit-py和run-py；ac_a2py_hir也通过Command::output等待，属于相同F27父终态问题。T-02同一opt-in入口须接第三个父adapter，T-03同时核Py三个emit recipe与一个run recipe；本包不把它排除后再声称共享worker全部父入口有界。现64文档/66 recipe盘点只覆盖原Rust/C，不能冒新增Py动态case分母。

Py emit沿4MiB元数据响应档；run-py响应包含stdout/stderr和stdout_hex/stderr_hex四字段。保现PY_STREAM_CAP=1MiB/流及PY_RUN_DEADLINE=30s，父侧先核hex长度≤2MiB、解码后每流≤1MiB，以原始bytes判oracle。每raw byte最坏JSON文本6B+hex2B，两流≤16MiB；另留元数据≤4MiB（含键/分隔符/identity），因此run-py原始响应cap明确取20MiB=20971520B。不能将4MiB emit门或worker stderr1MiB门套程序流字段，更不能把程序NUL/坏UTF-8 lossily显示文本当byte oracle。父层须分别核20MiB整体、4MiB非流元数据、每流1MiB，解码前拒超量。元数据可按去除四个流值后的同一JSON编码计量，原identity/manifest/credential父判保持。

Py parent也用70s交换/单一3s清理；run-py保spawned_python=1、execution_claimed=true及完整python_identity，Py emit仍spawned_python=0/execution_claimed=false。只允许父spawn预构建worker，run recipe的原目标Python腿不是违规额外spawn；Rust/C emit的spawn0合同不变，不能删Python真实执行以过门。

pyrun::resolve_python目前版本探测仍用无界Command::output；在本Plan同族修复中改用已存在的受管capped runner，所有解释器候选共享一次30s探测期限、stdout/stderr各1MiB（新探测策略，沿现执行档），不逐候选重置；解释器内容hash流式读取，不一次缓存整个binary；读取失败明确Infra，不退成路径字符串hash。原实际run腿30s不变；错误probe/真实执行/父交换清理分别记因，不伪造解释器凭证。平台仍按当前正式准入（非Windows实际runner若原拒绝继续拒），不借本包扩Linux支持。

新增边界控制：单流1MiB NUL加hex可达8MiB、双流可达16MiB，连同合法小metadata应被run-py父接受；另测20MiB响应多1、每流多1、metadata多1、版本探针挂起/持续双流、错身份、emit冒run及清理失败。只做小cap机制/序列化等界测试，不运行大输出正式campaign。保持既有十种recipe与条件，原Rust/C静态样本及Py实际入口分别记发现/执行数。

#### 三种line协议兼容决定（已决定）

本包保留各crate的协议入口，不新增跨crate统一reader/公共wire版本；只在原reader修正IO与TooLong/EOF的结果区分，共性抽取成本留1038。这些line cap均按原始帧字节计量，case/sema的lossy UTF-8输出最多为原字节的3倍，不得对转换后String重新套raw cap而误拒原可接受输入；这不是固定堆驻留上限。纯EOF不产生请求；IO Err必须具名Infra终止当前不完整请求，不能当正常EOF或接受前缀，先前已完成请求回执不重写。

| 原adapter | 原cap / LF口径 | EOF与编码冻结 |
|---|---|---|
| case-worker | SESSION_MAX_LINE_BYTES=8MiB，不计分隔LF | 保无LF最终非空帧可解析；保from_utf8_lossy后JSON解析，不能本包静默改strict UTF-8。 |
| semantics-store ipc | MAX_LINE=4MiB，不计分隔LF | 同保非空EOF帧及lossy UTF-8；IO错误不再折正常终态。 |
| mn-collections-worker | LINE_CAP=5657941字节，包含LF | 保必须尾LF、serde_json::from_slice严格字节准入；无LF/坏编码拒绝，不能跟另两者统一为宽松。 |

A2X one-shot则是完整JSON+EOF，不是上述line协议。JSON/UTF-8/字段格式坏为请求协议错误；文件IO/UTF-8/源或请求限额/输出cap/超时/身份失配等在父check一律Status::Error（Infra，保Resource子因），永不满足语言负例或冒语法不支持。对于有效UTF-8源码，语言诊断继续由原编译器前门产生。


**范围与执行门。** 当前仍只规划；T-01/02/03分别按本节和§6执行，T-04逐Case核证，不把正确性留给merge。G1002只约束改写新作者格式/相关接线，不约束原入口反例。本轮§5 r4修订后的设计决定已闭合，新增策略是本Plan的拟交付合同；本轮不含实现授权，后续work按既定合同实施，不把关键边界再留到实现中任选。

### A-EXIT-20261116：本包在重冻985/A中的阶段（当前优先）

本包所列原发现/Case全部为A出口必须关闭的正确性/可信证据输入，**不改变已冻结语义、预算或测试规模**。A取原T-01→T-02/03实现/反证与永久例→T-04独立复审→T-05正式合入/Spec来源；T-06按原规约收尾归档，不能把仅分支绿当正式D。可用已审落主切片准备消费，但985/A最终必须绑定真实代码/输入/oracle/目标/工具/原条件及结果。当前草案/0完成保持；本轮没有证明任何发现已修复。

实现若发现主线已有修复，须保原反例/合法邻例、真实入口/事实和充分反证，经原review确认后关闭，不能为了“做修复”重复改代码，也不能用未复现/组总绿当反证。文档/G1002只在实际改写作者格式或相应adapter接线时消费；原入口/Rust回归不等1066全量迁移。UI三波只约束图形新增支持，已有非图形AV/AC/A2R及原C/Py支持腿/错误/资源回归不取消。
**本包绑定**：F10/F18/F19/F27 → 原IPC-01..03及r4新增Python控制 → T-01/02/03 → T-04/05正式来源 → 985 T-A03/A04。原因：源码一次快照与hash、IO分类、三个父adapter和有界交换/清理是985证据可信基础，不能在不可信父侧上签A。关闭谓词：沿r4 DEC-IPC-LIMIT已定门：真实选中源/hash同一快照；Rust/C/Py父侧错误不充语言PASS，短deadline/stub与正常worker控制、单次收尾/产物准入/Python字节流分别观察；不冒整个run_check硬截止。

### 与1002及其他计划的依赖

- 本包调查、原入口回归与核心修复/研究不以1002为硬前置。只依赖1024审计交接；不等待1002全语料迁移或985终验。
- G1002=1002 T-03四例七检查及负控通过，T-04独立复审且读取/比较接口已正式合入并绑定SHA。只有新作者格式迁移或相关document/IR/adapter接线重构须等待G1002；Rust/旧case回归可先交付。
- 1002不反向等待本包全完。若本包真实反例影响其七行基线，只对受影响行具名先修/同源重跑，不抹真实红。
- 1017同路径计量/执行基线先协调冻结输入/构建身份，不等待整包、不擅跑ace-perf。能力计划只消费已审已合入修复，其M2义务不自动通过。

### 规范增量

| delta_id | add/modify/retire | docs/specs target | before/after rule | rationale | AC |
|---|---|---|---|---|---|
| SD-01 | modify（拟议） | docs/specs/auto-test-case/architecture.md | 现规则/支持域保持；补充以下修复后的明确合同：明确reader EOF/IO/编码与输入限额单位，不默改既有wire；错误身份父侧独立拒绝。 | 对应本Plan具体Case及真实consumer；不是候选已实现声明 | AC-02/03/04 |
| SD-02 | modify（拟议） | docs/specs/auto-test-case/a2r-hir-certification.md | 现规则/支持域保持；补充以下修复后的明确合同：daily Rust emit只spawn预构建worker；按r4新worker资源policy，70s交换/3s共用收尾且同步父IO单列，golden仍父判。 | 对应本Plan具体Case及真实consumer；不是候选已实现声明 | AC-02/03/04 |
| SD-03 | modify（拟议） | docs/specs/auto-test-case/a2c-hir-collections-certification.md | 现规则/支持域保持；补充以下修复后的明确合同：C daily按同一r4新worker资源policy与拟新增opt-in cap入口，保自身recipe/fileset、父独立golden及原formal轴。 | 对应本Plan具体Case及真实consumer；不是候选已实现声明 | AC-02/03/04 |
| SD-04 | modify（拟议） | docs/specs/auto-test-case/a2c-hir-scalar-certification.md | C scalar daily同步r4 worker资源policy/交换清理与父判，保原certify/四文件及spawn0。 | 共享C adapter不只影响collections | AC-02/03/04 |
| SD-05 | modify（拟议） | docs/specs/auto-test-case/a2c-hir-record-text-certification.md | C record-text daily同步r4 policy/父判，保各profile/证书和原reference轴。 | 补齐同一consumer的规范增量 | AC-02/03/04 |
| SD-06 | modify（拟议） | docs/specs/auto-a2x/python-stdlib-readiness.md | 同一worker新consumer接父有界捕获；Py emit4MiB/run20MiB响应与每流1MiB分域，保显式执行/解释器身份和原30s腿，版本probe有界。 | 1042合入差额，不得误拒现程序输出或漏第三父入口 | AC-02/03/04 |

本轮不改canonical Specs。frontmatter是拟议影响，review按实际交付收敛；touched_goals空，工程质量不擅扩产品目标。

## 6. 测试设计

拟执行完整受影响组：cargo t ace-a2x ace-native-worker ace-native ace-semantics ace-adapters auto-test-case auto-tool-execution。T-01按实际调用/修改冻结必要消费者并集，不能以单反例代完整职责组。 本包新增ace-adapters核实际语言父裁判，ace-native覆盖其line worker；本设计将扩展auto-tool-execution，必须加其同名完整组；不改core/link，不据此追加ace-source。目标测试名/新fixture在T-01核原目录后使其可运行，当前均未创建。原样本/规模/平台/ignore/正式性能轴保持；merge复用同身份有效证据，不追加全仓/ace-perf。

| Case ID | AC / work owner | 区分性输入/设置 | 实际入口 | 独立关闭oracle | 当前状态 |
|---|---|---|---|---|---|
| IPC-01 | AC-02 / T-02 | source_path读A后文件变化/二读故障的受控Reader反控；source_text邻例 | read_source→emit→父侧身份/manifest准入 | 实际编译bytes的SHA精确匹配；读错明确infra，不报空hash成功 | 拟实施，未运行 |
| IPC-02 | AC-02 / T-02 | 上限±1、多行JSON、EOF、半行、错UTF-8、Reader IO错 | one-shot main与三个原line reader | 合法历史输入保持；前置限量；IO不冒正常EOF；framing/编码按r4表保持 | 拟实施，未运行 |
| IPC-03 | AC-02 / T-02 | 预构建stub worker挂起、双流超量、子孙持管道、正常返回 | ac_a2r_hir/ac_a2c_hir真实父adapter→现owned executor | 交换受限且ERROR；自有子树/reader清理结果和失败残留具名；不额外spawn编译器，golden/manifest仍父判 | 拟实施，未运行 |

代表性路径/失败状态不冒全笛卡尔覆盖。发现/执行/原条件排除、准备/冷编译/发现/运行/清理、首次红/环境归因/重放绿分别记；父exit0不代子终态。当前未运行，无产品PASS。

角色：后续获授权work实施；review独立核证（无指定模型/另起agent授权）；merge收尾。使用同一实施树，本轮规划自检不冒实施review。


### r4具体输入与状态断言（全部待实施）

IPC-01用固定UTF-8源fn main() int { return 42 }加恰一个尾LF（28字节，SHA256=a1458aa7834a132d416cdc044693771f9719ca5febaad7df69fcd8f08534fc5d），分别source_text与source_path；请求其他字段/nonce/recipe继续由原adapter构造，不猜新wire。可控Reader在首次read后若再读返回不同文本或Err，并记录读次数，独立SHA只对实际编译bytes算一次。

IPC-02的line输入用固定cap C测试C-1/C/C+1（分别保含LF/不含LF的原计量口径），纯EOF、半行、非法UTF-8 JSON原字节hex=7b2278223a22ff227d0a及Reader在若干字节后IO Err；one-shot用合法多行JSON加EOF，不能因首行可解析就提前接受。

IPC-03测试stub具名模式：正常、永不退出、持续stdout、持续stderr、父退出而孙进程持管道、返回畸形JSON；stub和所有PID记录在本计划scope，不能误杀同机进程。

| 原Case ID / 子面 | 实际设置/适用状态 | 独立关闭断言 |
|---|---|---|
| IPC-01 | source_text/path、UTF-8错/读错、返回错SHA | text/compile/SHA同一次快照；父侧继续独立拒错身份/manifest，读错不是hash(empty)成功 |
| IPC-02 | line worker三种旧reader、one-shot stdin/argv、原合法最大case | EOF/IO/TooLong/encoding分开；原支持framing和含LF计量保持，按r4表保持，无wire升级 |
| IPC-03 | 两个真实父adapter、双流/期限/子孙/reader/正常golden | 交换deadline与单一cleanup截止分账；cap超量ERROR；自有树/reader退出经确认或具名cleanup失败，无rustc/cargo额外spawn；正常字节与golden不变 |

| 原Case ID | 拟新增永久测试名（当前不存在） | 既有源码/目标落点 | 使其可运行的owner |
|---|---|---|---|
| IPC-01 | audit1034_ipc_01 | [experimental/ace/crates/a2x-worker/src/bin/a2x-worker.rs](../../experimental/ace/crates/a2x-worker/src/bin/a2x-worker.rs) | T-01固定设置，T-02新增函数/接线，T-03核真实发现/执行 |
| IPC-02 | audit1034_ipc_02 | [experimental/ace/crates/case-worker/src/main.rs](../../experimental/ace/crates/case-worker/src/main.rs) | T-01固定设置，T-02新增函数/接线，T-03核真实发现/执行 |
| IPC-03 | audit1034_ipc_03 | [crates/auto-lang-core/src/test_cases/ac_a2r_hir.rs](../../crates/auto-lang-core/src/test_cases/ac_a2r_hir.rs) | T-01固定设置，T-02新增函数/接线，T-03核真实发现/执行 |

一个Case可需多个同族函数，以上名字是最小锚，不代全部状态。优先扩已有test根/私有cfg(test)模块；若新建test文件，须在其真实conformance根或Cargo target接线，不能只创建文件然后把rg属性数当发现。T-03必须在实际固定组结果中核各锚点被发现/执行；平台/工具条件排除具名列出，不计PASS。


### r4决定对应的补充断言（未来实施）

| 原Case | 冻结默认与负控 |
|---|---|
| IPC-01 | text保1MiB±1成功进入编译及request交集边界，path/reference/golden分别64MiB±1；实际文件增长越过cap的受控Reader必须被cap+1读取拒绝。64MiB大默认只对纯reader/计数门验证，不调用编译器；完整case另跑真实小源，不降formal规模；真实小源42仍走hash/compile/manifest全链。 |
| IPC-02 | request原字节8MiB±1（用合法JSON空白调尺寸），最坏转义1MiB inline、1MiB+1 ASCII inline与多行JSON；原path优先并存字段控制；三line各自C-1/C/C+1、是否计LF、半帧EOF/lossy或strict及IO在前缀后失败，逐原入口保持表中差异。 |
| IPC-03 | 两个真实adapter分别用显式小caps（如stdout128B/stderr64B）及短deadline（如200ms）触发持续双流、不读stdin、挂起、子退出孙持管道、IO/错JSON/错身份等；小值只属测试配置，生产固定stdout4MiB/stderr1MiB/exec70000ms和单一cleanup3000ms。测试另给小cleanup预算，观察首次因/自有PID及reader/writer/guard总收尾，失败残留具名；超70s绝不成功，不能让每线程再续3s。 |
| IPC-03 | 另核实际产物64MiB/128MiB及manifest4MiB的等界/多1有界读取、错误宣称长度/清单逃逸/额外文件；用受控读取/计数小预算验证机制，正常64个跟踪case文档及Rust/C各scalar、record-text、collections六种Rust/C recipe及四种Py recipe的golden/source/identity逐项重放，平台未运行不计PASS。 |

有界入口落在auto-tool-execution现测试根，必须追加其完整固定组；两个adapter均核新调用，不用helper单测代替真实父裁判。若实施修改共享工具库影响SDK消费者，只补真实受影响生命周期消费者，不把ACE正式性能挂日常门。

本轮A阶段交接：IPC-01..03及r4新增Python控制的全部原合法/拒绝/失败后恢复与资源断言是A必需；原AC-01..06范围不减，独立复审/正式来源和当前A/B分类由985接收控制核，不另建Plan状态或测试调度器。当前均未执行。

## 7. 验收标准

- [x] AC-01 / work：T-01冻结真实源、支持/资源边界与前置，落实r4 DEC-IPC-LIMIT的源码对称门、默认/计量/平台/阶段与错误类，所有§6反例及邻例可在实际入口运行，不私改expected。
- [x] AC-02 / work：r4具体输入/每项状态断言及§6实际测试落点均落实；§6每行独立oracle已取得；缺口修复或有充分反证，修前/修后及同族边界证据齐，原义务不减。
- [x] AC-03 / work：消费者、原测试、恢复/重试/失败责任保持，完成适用门、Spec准备及1002依赖分流，未闭不冒PASS。
- [x] AC-04 / review：逐项核r4独立预期、后续使用闭环与具名决定，不能仅信组绿；逐AC/Case/代码/Spec独立核验、遗漏/延后/workaround及健康/来源检查通过，结论绑定revision/冻结提交。
- [ ] AC-05 / merge：review后冻结并集门/正式合入、规范沉淀及归档真实完成。
- [ ] AC-06 / merge：所有输出路径对账，摘要折Plan，scope删除或具名持有，guard clean后清树/分支。

## 8. 执行步骤

| task / owner_stage | 依赖 / 状态 | 操作、交付与验证 / AC |
|---|---|---|
| T-01 / work | [x] 1024交接、未来实施授权及当前源 | 按§5 r4 DEC-IPC-LIMIT重核源漂移、64个case样本、1MiB±1控制及A2X真实formal入口；建立Rust/C/Py三父adapter的可信stub/短deadline、原OS Python正常及缺失负控和三line差异控制，记录协议/recipe/source/manifest独立预期。 AC-01  A-EXIT-20261116：本行按A-required原Case闭合；当前分类不是完成证据。 |
| T-02 / work | [x] T-01反例设置与r4既定决定 | 先接有界一次快照/hash与Py版本probe，再在auto-tool-execution同生命周期增加opt-in cap并接Rust/C/Py三父adapter的stdin/70s交换及单一3s cleanup/产物有界准入；原三line分别修IO分类，不抽新跨crate reader、不改默认无界工具调用。 AC-02  A-EXIT-20261116：本行按A-required原Case闭合；当前分类不是完成证据。 |
| T-03 / work | [x] T-02 | 对Rust/C/Py三adapter跑IPC全部状态和正常old suite/golden；记录PID/reader收尾/错身份及spawn计数，不能只测替身helper或以timeout父kill代case终态。 AC-02/03  A-EXIT-20261116：本行按A-required原Case闭合；当前分类不是完成证据。 |
| T-04 / review | [x] T-01..03完成、冻结提交 | 独立核AC-01..03、§6/SD/漂移、遗漏延后绕行及债务；失败回同树work不缩scope；AC-04  A-EXIT-20261116：本行按A-required原Case闭合；当前分类不是完成证据。 |
| T-05 / merge | [ ] T-04 pass | 核冻结并集且复用同身份证据，正式合入、规范/归档；正确性不留merge；AC-05  A-EXIT-20261116：本行按A-required原Case闭合；当前分类不是完成证据。 |
| T-06 / merge | [ ] T-05 | 所有实际输出对账、摘要/删除/持有登记；wt-guard clean后清树/分支；AC-06  A-EXIT-20261116：本行按A-required原Case闭合；当前分类不是完成证据。 |

同文件串行；跨计划调度见1024分配表。草案stage:new pass不授权开T-01构建/探针。

## 9. 复审记录

2026-11-13 stage:new / r1 / outcome:pass（可审阅规划，非产品/独立实施review） / next:获实施授权后work。new-plan.sh中央顺序取号，源/Spec路径与组存在，Task/AC/Case/SD明确；drafting 0/6。未建树/材料、改实现/测试/配置、构建/运行、提交/合入/tag/归档。


2026-11-13 stage:new / r2 / outcome:pass（执行合同预检，可从T-01开始；非代码/独立实施review） / next:后续实施授权和当时基线重核后work。r1既未定义one-shot/line各自的字节口径，也把auto-tool-execution泛称成已有带cap执行器；该库的run_command_capture实际无默认输出上限。 已补§4真实差额、§5边界/故障设置、§6完整输入/独立oracle/目标落点、§8前三项定制操作、SD具体规则；原AC/Case/T-ID及6任务/drafting0进度保留。 DEC-IPC-LIMIT：T-01列request字节、source UTF-8原始字节、stdout、stderr及wall/deadline五个单位与作用点，盘点当前跟踪case和1MiB正式负载/JSON转义膨胀。源/请求新限额、daily响应cap或新兼容拒绝若无现合同依据，先给具体上限与相容决定，不暗设1MiB统一门；该决定只阻相关cap改动，F18同源读取可独立推进。测试使用显式小cap/短deadline作机制负控，不把小测试值写成产品默认。 该具名决定在T-01关闭前不进入依赖T-02改动；不是把支持判断留给任意实现。

r2所读源Git=c2760324da25795594ab0cc01b581e3a306d2375；r1已存在的源码差额：

本Plan r1已钉源/Spec文件无字节差额；额外读取的现测试/设施见§4。

本次只写本Plan，无实现/测试/配置/Spec/执行改动；未建树/材料/运行/取号/提交/合入。1028不在修改集合，同机其他计划WIP保留。


2026-11-13 r2目标文档终检：本批11份计划原66个任务/AC编号和29个Case ID保留；本Plan原3个Case不删减。共核114条当前源/Spec/测试入口hash，本批26个拟新增测试锚点只作未来T-02落点，未创建/未执行。各Plan 0–10章节、围栏、本地链接、实际组名、r2具体输入/断言、原任务顺序/owner_stage及drafting0进度通过。1028计划字节hash与本轮入场一致，本会话未写其实施树；并发其他计划WIP保留。结束HEAD=9a7674009725913f06f44b8176ae20c8aad2c455，c2760324→结束HEAD的本轮源/Spec/组范围diff为空，无相关源码WIP。git diff --check与新文档空白检查通过；这些均为静态规划检查，不是ACE功能/资源/性能PASS。T-01真实源/输入/预期及下一步已明确；T-02仍须原T-01反例/决定完成。 DEC-IPC-LIMIT为具名T-01决定，关闭前不修改其依赖部分；不是新增用户批准的限额/支持承诺。


### r3设计核清记录

2026-11-13 stage:new / revision:3 / decision:DEC-IPC-LIMIT closed/design-static。依据§4补充Spec/代码区分原合同与新策略；静态盘点64个case文档/66个recipe匹配与文件大小，未运行case解析器/worker，未对formal性能做成功断言。独立算术：8MiB=8388608、1MiB最坏JSON=6*1048576+18=6291474、native含LF cap=(4<<20)*4//3+(64<<10)=5657941。原r2把设施选择留T-01的记录为历史，现选定跨平台轻量库opt-in扩展；cap API尚不存在，不能写成已可直接调用。当前HEAD=6584e94d465dc1f8b2a1cd58344913de4d5ed18a。3 Case/6 AC/6任务保留，drafting 0/6，T-01未整项完成。只更新本Plan；未建树/写fixture/构建/运行ACE或性能/改canonical Spec/提交/合入。

本轮静态来源复核：本Plan原§4来源均重算SHA256，原来源字节与r2相同。本轮三Plan共绑定39个去重来源（源码/Spec/组配置）；已于r3静态终检重核，无来源漂移，历史r2 hash不冒当前执行证据。


r3静态终检通过（2026-11-13）：三份文档0–10章节、r3/drafting 0/6元数据、原18个Task与18个AC及11个Case编号、表列/围栏/空白、本地链接和职责组实际名称核对通过；39个去重来源SHA256与本轮冻结快照一致。git diff --check通过，experimental/ace、crates、scripts无源码diff；未执行ACE测试/构建或把静态检查记作功能/资源/性能PASS。当前只关闭设计决定，未来实施与独立代码复审义务保留。


### r4目标复审与同轮修订记录

stage: review；review_scope: targeted-draft（r3执行前设计合同）；reviewer_identity: 本会话Codex；independence_limit: 与起草同一会话，未另起独立上下文/agent，本轮从现Spec、实际源码和独立构造反证重建裁定，不以r3闭合声明作证。outcome: r3 needs_replan；用户已授权直接修订，现改为r4并复核设计项，非final实现pass、不将状态改reviewed。

reviewed_plan_r3_sha256=a0abba554448667cae4afa4932c37286fed052e21a8b863b148e983c91eae902；source/base_commit=09119bd3cface39a8c794e54f9dacdd3f7dd4452；worktree_path=D:/autostack/auto-lang（计划主检出），branch=master；未创建实施树。reviewed_contract_r4_sha256=b2d14be26a40d1a78cb06e488c389a9fef95214cf7ed261eede1d56529155ee1（LF UTF-8，§0–8含frontmatter）；proposed_spec_delta_sha256=62e739ff36eebca1c79959be171e4b181881a527966c63e22bfb90de12991cc9（§5“规范增量”至§6前，LF UTF-8）。原§9 r1–r3收据原样保留但其相容/截止/资源结论已被本次具名修订替代。

| finding / severity | r3合同问题与可复核证据 | 同轮修订 / 关闭oracle与范围 |
|---|---|---|
| R-1034-01 / P1 | TestCase inline硬上限1MiB不适用于未写该约束的私有worker；ASCII源1048577B、最小JSON1048595B<8MiB，r3拒inline而同内容path可进。64个小case不能证明全域相容。 | 删除额外字段1MiB门，text/path共64MiB内容门且text另受8MiB请求；明确保留门的新增资源收缩、路径优先级和line raw/lossy口径。IPC-01/02；AC-01/02/03。 |
| R-1034-02 / P1 | run_prepared没有硬截止父run_check里的同步fs IO；TEARDOWN_GRACE=3s被多处及两轮cancel使用，r3不能同时承诺整个check70s无宽限与复用旧设施即满足。 | 70s交换与触发后一次3s cleanup截止，成功含收尾必须在70s内；同步父阶段有界字节/单列时间，收尾失败具名残留。新opt-in须传播同一cleanup截止至guard，非现成API。IPC-03/AC-01/02/03。 |
| R-1034-03 / P2 | certify-c还读取golden_path，r3只限定source/reference；当前read_to_string(...).ok()既无界又把声明读取失败降成None。 | golden若未声明保None，声明则64MiB有界UTF-8/hash且IO/超限Infra；只属认证键材，绝不进daily生产。IPC-01/02/03与显式certify-c反控；AC-02/03。 |
| R-1034-04 / P2 | gen796-1mi/v1是v2-native计量证据；A2X load256及a2x-formal/session API不必通过worker JSON。r3把1MiB转义算式当A2X正式路径相容证据过强。 | 1MiB只记字节边界控制；原A2X campaign、30采样/31修订及120/30/360s分入口保留，未运行不记PASS。§4/5/6、AC-03/04。 |
| R-1034-05 / P2 | 共享ac_a2c_hir覆盖scalar、record-text、collections三recipe，r3规范增量仅列C collections，不能完整沉淀新policy。 | 新增C scalar/record-text两Spec目标与SD-04/05；六recipe正常/身份回归保留。§5/6、AC-03/04。 |
| R-1034-06 / P1（源漂移） | 1042已合入第三Py父adapter及run-py四流字段；4MiB不能容纳两流各1MiB的text+hex，版本探针还使用无界output。 | 第三父接同一bounded入口；Py run整体20MiB/metadata4MiB/原始流1MiB分账，probe共享30s/每流1MiB；原run/emit、身份/平台保持。IPC-01/02/03，SD-06，AC-01/02/03/04。 |

验证方法：rg符号/逐入口读取/Spec对照；Python -B -仅内存算式和文档结构/hash检查。Pratt两方向分别得到rule_valid=true/token_valid=false与rule_valid=false/token_valid=true；N=5000000时wire上界65001024<67108864、受控请求同驻留下界280000000>268435456；ASCII源1048577B的最小信封1048595B<8388608。以上是设计反证，不是运行decoder/compiler/worker，未分配巨包，未运行ACE/构建/正式性能。

acceptance_results：AC-01/02/03及其T-01/02/03实现/动态oracle=pending，AC-04独立实施代码review=pending；本次仅其设计边界/Case/责任/Spec-delta逐项核清。AC-05/06 merge/archive/清理=pending。原6任务/6AC及本Plan Case分母保持，drafting 0/6，不把本轮核清勾为T-01全部完成。所有scope内复审 finding已落实到r4合同，未新增实现债务豁免或以1002迁移代正确性。

依赖快照：1024 r2 reviewed 4/5（SHA256=4834e56b888635bcb68f82bab88d61da39edbdb2c8897dee2f084960641825e5）；1002 r7 drafting 0（SHA256=751c438a33864cd333d19c6aa198c3cfa86aa88cb4974472fb5ac3b3d0358dfa，主检出未提交文档身份）。G1002仍只约束新作者格式/相关接线，不要求本包旧入口修复等待其全迁移。当前源码/Spec/配置以本轮54个去重源字节hash绑定；历史receipt不冒当前产品验证。next：未来获实施授权后在原计划专属树执行T-01反例设置与基线复核，按r4合同进入T-02；不得照r3已撤销限制施工。


补充来源绑定（其余见三Plan §4/既有来源差额表）：

| 来源 | SHA256 |
|---|---|
| docs/specs/auto-a2r/collections-hir-backend.md | 7cffe09c0ae08b86ba200dc742f30020633f01ddb094a70384ad46ef0385597f |
| scripts/test-groups.toml | e8b55f0eeecc01bb4eb2c5d672a11c2113ee00a39aae8842d81b3f650f6c7f2f |


复审期间并发合入1042 Python通道：首次来源冻结时HEAD=6584e94d465dc1f8b2a1cd58344913de4d5ed18a，现按正式合入HEAD=09119bd3cface39a8c794e54f9dacdd3f7dd4452重核相关差额；不能将初始HEAD与后合入源混写成同一提交。contract只新增Py recipe注册/门，conformance新增Py模块及排序，ace-a2x组新增Python包，原数值cap与原R/C recipe保持。其余已绑定来源按hash逐项复核；本会话不修改/接管他Plan实现。

| 来源漂移 / 新消费者 | 当前SHA256 |
|---|---|
| crates/auto-test-case/src/contract.rs | de83e05a610ca64c14e138c76fd9a7beb18ff25243e5eeb0710d3c0dfbc0af0a |
| experimental/ace/crates/a2x-worker/tests/conformance.rs | 32fa9de0918496bf9d851c427936848602b8a7de0d2d180e307f3795604f35db |
| scripts/test-groups.toml | 1b432bb63abff7fef0222d18db33246b7ec25f15a4015c2393f0f4cb1a84b3ce |
| crates/auto-lang-core/src/test_cases/ac_a2py_hir.rs | c175327e5a4e92d98227b03140df2407e0effbabd326a0d614aa4b3085bfb3b6 |
| docs/specs/auto-a2x/python-stdlib-readiness.md | 930ab8b2fb6eb202f95afa2218aa4530836260d6bd83de411dbf97d2fcd0aa90 |
| experimental/ace/crates/a2x-worker/src/pyrun.rs | edad642fcc60ba21b337341ef29e7d3a9fa0f516d218dd463701bf4237276e18 |
| experimental/ace/crates/a2x-python/src/lower.rs | 37c47e24a989b9561abaf925f90fdfee2042cf252fc744a4f11f00abfec2f3c6 |


r4静态终检（2026-11-13）通过：54个相关源/Spec/配置及2个依赖文档与漂移重核后的绑定一致，结束HEAD=09119bd3cface39a8c794e54f9dacdd3f7dd4452。三Plan的0–10章节、r4/drafting 0/6、原18任务/18AC/11Case、表列/围栏/空白、本地链接、实际固定组，以及§0–8合同SHA和拟议Spec delta SHA逐项核对通过；git diff --check通过，当前实现路径无WIP。本会话只写这三份计划，并发1042等正式合入差额已重核，不将它们说成本会话实现。设计finding的合同纠正已落实，运行oracle、T-04独立代码验收与merge门仍pending；未执行ACE/构建/性能，不签产品或最终实现pass。

2026-11-16 stage:new/revision | PLAN-1034 r5 | outcome:pass（规划分类/静态合同，非产品或A出口PASS） | F10/F18/F19/F27/原IPC-01..03及r4新增Python控制明确A；6任务/勾选/0完成与既定设计保持；源/结果不得以旧审计自报替代 | next:原owner未来获实施指令后按现合同执行；当前仅规划，不建树/写代码/运行测试/合入/清理/恢复目标。

本轮规划静态预检（非产品PASS）：18份规划/路线/债务文档；13份Plan共92个原任务，任务ID/勾选/状态/完成数/整包依赖保持。985本轮162父行分类索引唯一、F01..35唯一且全覆盖（20条A正确性、6条A约束/证据、2条已交复用、7条B）；新增本地链接/锚点、章节、表格、阶段/任务/原Case映射与行尾核对通过，阶段图34节点36边无环，1066的A终验不再依赖B-only格式/项目/250编译任务。仅验收合同冻结；真实矩阵/adapter/新控制及全部新产品结果仍待原work任务，未运行/签A。未改源码/测试资产/canonical Specs/取号计数器，未实施/建树/构建/窗口/合入/归档/清理或恢复目标。


### 1034 work 收据（2026-11-16）

stage: work | plan_id: PLAN-1034 | plan_revision: r5 | outcome: pass（work-owned T-01..T-03 完成）
worktree: D:/autostack/.wt/lang-1034/auto-lang | branch: plan-1034-dev | base: bdb8fe428200377006bbd847e7075af9d2ee9179
code_commit: 761b6e949（18 文件 +2495/-135）| dependency_revisions: sha2=0.10 新增（白名单入列）；auto-lang-core 增 auto-tool-execution（计划 §5 既定）
next_responsible_role: review（AC-04；统一回归已按冻结组并集执行，见下）

**T-01** 源漂移重核：12/16 文件与 r4 绑定 hash 一致；4 漂移均为他计划加法（1002 authoring inline payload、
conformance 新 case 挂载、组配置、link.rs 测试路径），不触 F10/F18/F19/F27 面。IPC-01 固定源 28B
SHA256=a1458aa7834a132d416cdc044693771f9719ca5febaad7df69fcd8f08534fc5d 逐字节复核一致。
case 样本现值：123 文档/127 recipe 匹配（a2r 9+18+40 / a2c 10+10+40；1002 document 兄弟与 981 并入），
全部远低于 §5 policy；r3 的 64/66 为历史快照。反例设置冻结于 plan-artifacts（t01-freeze.md）。

**T-02 交付**：
- worker 输入门：REQUEST 8MiB（stdin take(cap+1) 整文档、argv OsStr 长度准入、serde 前拒绝，
  a2x.request-{limit,io,utf8}）；read_source 一次 cap+1 快照（同 bytes hash+decode+compile，删双读
  TOCTOU）；certify/certify-c reference+golden 同门（golden 未声明 None，声明缺/坏/超具名 Infra）；
  A2X_TEST_FAULT 六模式故障缝（hang/flood-stdout/flood-stderr/bad-json/wrong-identity/orphan）；
  pyrun 探针改受管 capped runner（共享 30s、每流 1MiB、流式 hash、读失败 Infra）。
- auto-tool-execution：opt-in `run_bounded`（BoundedPolicy：exec/cleanup 双期限、per-stream cap；
  StreamCap/ExecDeadline/Infra 三态 + CleanupReport 具名残留）；capped CaptureWorker read/stop-drain
  两路扩容前核帽、计数饱和 cap+1；`cancel_within_absolute` 两轮 kill/reap 共享绝对截止。实现中修复
  真缺陷：正常收尾 wait() 已消费消息后二次 stop_and_join 把已收线程误报 panicked（flood 管道破裂
  自退 0 的两种时序均覆盖）。既有 run/run_command_capture 合同零改动。
- 三父 adapter（ac_a2r/ac_a2c/ac_a2py）：新 a2x_exchange 模块——stdin 请求、ProcessTree、70s/单一3s、
  stdout 4MiB（run-py 20MiB）/stderr 1MiB；ArtifactBudget（manifest 4MiB/单文件 64MiB/总量 128MiB
  checked 累加）；run-py 分域（hex≤2MiB、解码≤1MiB、非流元数据 4MiB 解码前拒）。TEST-ONLY env
  覆盖串行锁，生产默认冻结。
- 三 line reader：case-worker/sema IO 错具名不折 EOF 不收前缀（无LF终帧/lossy/raw 计量保持）；
  mn over-cap（含 LF）与 missing-LF 拆分具名、严格 from_slice 保持、冻结值 5657941 自检。

**T-03 交付**：audit1034 永久测试 26 例全绿——IPC-01 9（含 1MiB+1 内联无字段门）+ py probe 1 +
IPC-02 11（三 reader 通道）+ IPC-03 设施 6 + 父面 6（真实 run_check + 预构建 worker 故障缝：deadline/
stdout cap/stderr cap/bad-json/错身份/孙持管道，PID 记录、cleanup_completed 事实、无 rustc/cargo 额外
spawn）+ run-py 分域 1。全部 ≤1.3s。

**验证记录（本会话实跑）**：cargo t 冻结组并集两轮（首轮 run-t6mf2uz_ / 修白名单后冻结版 run-7runm5ea
+ 组级复跑）——ace-a2x 297 PASS、ace-native-worker 43 PASS、ace-native 295 PASS（首轮 3 例并集负载
闪红，隔离与冻结版均绿=环境归因）、ace-semantics 292 中 291 绿+1 既有红（PLAN-1062 恢复的
cache-roundtrip.document.case.ad 缺 status: executed，master 既有）、ace-adapters 95 中 94 绿+1 既有
abi 红（source_parent_oracle_failure_rejudges——**pristine 主检出同测试复现失败**，master 既有，归
abi authority 链 owner）、auto-tool-execution 49 PASS（冻结版并集 1 例 816 计时敏感闪红，组内重放绿
=环境归因）、auto-test-case 151 PASS（复用首轮证据，fmt 未触及）。old suite case 六组重放 6 FAIL=
golden 滞后（actual 含 PLAN-967/975 runtime 新内容 vs PLAN-902 期 goldens；actual 产物经新有界路径
完整产出并被父侧有界读取比较，失败仅落 golden 侧；归 case 组 owner，不充 PASS 不代修）。证据：
plan-artifacts/plan-1034/{t01-freeze,t02-implementation-notes,t03-golden-drift-triage}.md。

AC-01/02/03 达成（证据如上，§6 每行独立 oracle 落到具体测试）；AC-04 review、AC-05/06 merge 待续。
Spec delta SD-01..06 维持拟议，按实际交付待 review 收敛。


stage: goal | plan_id: PLAN-1034 | plan_revision: r5 | round: 1 | from: work | to: review | status: archived | outcome: pass | worktree: D:/autostack/.wt/lang-1034/auto-lang | commit: 761b6e949 | next: review
（/goal 授权链：work pass → review；既有红三处具名归因不充 PASS，见 work 收据）


### 1034 T-04 final review（2026-11-16）

stage: review | review_scope: final | reviewer_identity: 本会话 ZCode（与实施同会话——局限性已声明；结论自工件独立重建：冻结提交 diff 逐文件核、常量/SHA 独立重算、33 个永久测试在冻结提交上重放、不采信执行方摘要）
plan_id: PLAN-1034 | plan_revision: r5 | outcome: **pass** | worktree: D:/autostack/.wt/lang-1034/auto-lang | branch: plan-1034-dev
reviewed_commit: 761b6e949 | base_commit: bdb8fe428200377006bbd847e7075af9d2ee9179 | dependency_revisions: sha2 0.10（白名单随 boundaries 治理入列）；auto-lang-core←auto-tool-execution
spec_inputs: SD-01..06 六目标均存在且 hash 稳定（architecture cd870bd2…、a2r-hir a32d3555…、a2c-coll 94e00af4…、a2c-scalar 6f052e52…、a2c-rt 86f5049f…、py-readiness 930ab8b2…）
acceptance_results: AC-01 pass / AC-02 pass / AC-03 pass（映射与证据见下）；AC-04 本记录；AC-05/06 merge-owned pending
pending_closeout_gates: merge 冻结并集复用（run-7runm5ea 绑定提交内容）；归档/清树/guard
findings: 无阻塞项。两条非阻塞观察已记录——(a) IPC-03 父测试在预构建 worker 缺失时 skip+eprintln（不充 PASS；组工具声明保证运行）；(b) run-py 解码 1MiB 门被 hex 2MiB 门算术蕴含（测试注明，防御性双门保留，符合合同"分别核"）。
evidence:
- 冻结值核验：a2x_exchange.rs 常量 70_000ms/3_000ms/4<<20/1<<20/20<<20/64<<20/128<<20 与 worker 8<<20/64<<20 与 §5 DEC-IPC-LIMIT 逐项一致（grep 实读）。
- IPC-01 固定源 28B SHA256 独立重算=a1458aa7… 与计划冻结一致。
- 永久测试复审重放（冻结提交）：设施 6/6（0.45s）+ worker/line-reader 20/20（0.05s）+ 父面 IPC-03/分域 7/7（0.68s）。
- 新用例计时：26 个新增全部 ≤0.75s（<10s 门）；无隐式日常构建（预构建工具经组声明、te-child 经 CARGO_BIN_EXE、release 构建 5m16s 属工具准备并计量）；全程 dev/test profile。
- diff 卫生：+2495/-135 无 TODO/FIXME/workaround 标记；Rust adapter 零新增 Command::new（唯一 spawn=ExecutionSpec worker）；既有入口合同未改（run/run_command_capture 仅加不可达 CapExceeded 臂）；golden/case 文档零改动（未私改 expected）。
- 既有红归因复核：abi source_protocol 在 pristine 主检出复现失败（99.6s）=master 既有；sema boundary 红=PLAN-1062 恢复文件缺 status 标记（master 既有）；case 六组=golden 滞后（actual 含 PLAN-967/975 runtime 新内容，runtime 源本包零改动）；ace-native 3 例与 816 计时 1 例=并集负载闪红、隔离/组内重放绿（环境归因）。均不充 PASS、不判本包。
- 单一收尾截止独立核验：cancel_within_absolute 与三线程 stop 均从同一 `at` 取剩余量；实现中自发现并修复的二次 join 误报缺陷有对应触发场景覆盖（flood 管道破裂自退 0 时序）。
next: merge（T-05/T-06）


stage: goal | plan_id: PLAN-1034 | plan_revision: r5 | round: 2 | from: review | to: merge | status: archived | outcome: pass | worktree: D:/autostack/.wt/lang-1034/auto-lang | commit: 761b6e949 | next: merge


### 1034 T-05 merge 收据（2026-11-16，PLAN-1034:r5）

stage: merge | plan_id: PLAN-1034 | plan_revision: r5 | outcome: pass（分段 checkpoint 见下）
prepared: reviewed 761b6e949（r5 final pass）+ docs-only 后裔；SD-01..06 于 worktree 落盘
landed: **master=140c1a626=plan-1034-dev（--ff-only，无 merge commit）**。rebase 两次（master 期间前移：981 落地 bd0698890、980 docs 至 714366aaa），range-diff 全 `=` 等价；映射链 761b6e949→62ba5e4ad→7db528b93（实现）、aefe8a3e3→b46889546→140c1a626（沉淀）。第二次 rebase 仅叠 980 docs 两提交（980-ace-m2-autoui.md/m2-autoui.md），代码零变化，冻结门证据绑定的代码树不变。
canonical_verified: 六目标在 landed 提交内（HEAD grep 实证 §「worker IPC 读取与输入限额单位（PLAN-1034）」等六节齐备）；architecture.md 工作树差异=1062 既有 WIP（保留未动）。landed hash：a2r-hir 31779745…、a2c-coll 32aae23d…、a2c-scalar 2f02b419…、a2c-rt f168016c…、py-readiness 695a26c3…、architecture(HEAD) 21129c88…
merge-owned 冻结组门（rebased 基线 b46889546 代码树，run-v2if6oid）：ace-a2x 297 PASS、ace-native-worker 43 PASS、ace-native 295 PASS、auto-test-case 151 PASS、auto-tool-execution 49 执行 48 绿+1 既有 816 计时例负载闪红（隔离 5.05s 绿+整组 49/49 重放绿，环境归因）、ace-semantics 291 绿+1 既有（1062 恢复文件）、ace-adapters 94 绿+1 既有（abi，pristine master 复现取证）。三既有红具名不充 PASS（同 review 记录）。

## 10. 待澄清事项

DEC-IPC-LIMIT经r4修订后无关键设计未决：text不再额外限1MiB；8MiB整请求、64MiB选中源码/reference/golden及输出/产物门是明确新资源policy，不能称对旧无界接口全域无影响。70s只约束owned交换且另有单一3s收尾；父同步IO无硬截止承诺，误差/清理失败残留不得记PASS。现样本及各formal入口的未来动态重放仍欠，未运行不充证据；状态drafting 0/6，仅格式接线按G1002。
