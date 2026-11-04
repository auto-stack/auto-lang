---
plan_id: PLAN-930
status: execution_done
feature_name: 生成工程交付回归与独立来源边界验收收口
author: [Codex]
created_at: 2026-11-04
updated_at: 2026-11-05
plan_revision: 1
current_step: 6
total_steps: 9
supersedes_spec_components: [docs/specs/repository-boundaries/generated-ui-distribution.md, docs/specs/repository-boundaries/standalone-language-build.md]
new_spec_components: [docs/specs/repository-boundaries/generated-delivery-regression.md]
touched_goals: []
affects: [generated-ui-consumer-driver, generated-delivery-controls, standalone-language-admission]
depends_on: [PLAN-906, PLAN-910, PLAN-918, auto-ui/PLAN-036]
---

# [PLAN-930] 生成工程交付回归与独立来源边界验收收口

## 0. 变更摘要

接收036/918正式结果，收口生成UI交付driver仍有的**V04两红与V05a一红**，建立对应固定组负控，并以真实来源分别重验语言L1/L2独立边界。语言持driver/准入/固定组主合同，若V05a实证为UI生成/投影问题则UI owner同组窄改，不在语言复制UI算法。

execution_done r1、6/9（2026-11-05）：三红全部真实闭合（基线 28 PASS/3 FAIL → 修复 41/41 全绿），新固定组 generated-delivery-controls 15/15，隔离 L1/L2 独立成立，SD-01..04 落树；未测面如实记录（UI shell-pack 过期红、jet/ark 环境门控保原 owner）。

## 1. 目标

- G1：driver按910真实成员/项目身份取dev产物，不用旧basename路径或模糊glob；保dev/delivery main.rs同编译相位/字节合同。
- G2：真实bundle A/B内随包tool可locked构建并在外部真实消费，V05a根因闭合；原V01..V06执行/负控/来源/主题义务守恒。
- G3：命名固定组承接便宜但区分错误的driver/身份/子失败负控，不另造tm/影响图/新的feature分叉。
- G4：当前冻结语言源的L1 Core与L2完整workspace在无UI/Web/Man兄弟检出下真实成立，原source/锁/错误恢复与918审计规则不削弱。

非目标：继续拆crate、整个Cargo/机器缓存治理、修改test feature政策、SG/ACE语法927–929、UI041状态/042 Reload、OS/Demo主题迁移、Web D-906-1发行平台或schema级ID集合、Man D-910-2派发、Tauri R-6、全部历史债/平台全绿。

## 2. 架构方案

冻结三仓source/tool身份→原producer生成dev和bundle A→明确dev成员身份/正确产物→A fresh/locked/tool→B不含target重定位/locked/tool→原wire/资源/失败与主题真实消费→收据/独立源边界。语言driver验证对象与终态，UI负责生成器/投影、Man负责真实CLI，职责不反转。

便宜控制检验成员身份/歧义拒绝、真实文件比较/坏source/子项失败传播；真实构建和端到端仍原显式driver执行，不拿合成控制绿抵真实A/B。独立语言边界沿906既有受管来源复制/consumer机制，不移动/隐藏用户真实仓。

## 3. 技术栈

Python现driver/固定组runner与unittest、Git/锁定Cargo metadata/build、现source_owner/generated_build与受管安装/交付清单；必要UI rust_ui.rs投影和原工具wire。原资源/来源/锁/生命周期与hash协议保持，不设计另一个缓存/发行引擎。

## 4. 需求分析与背景调查

已读auto-plan-new/verification-contract、docs/specs/overview、仓界/生成生命周期/生成交付/906独立边界与918准入Spec，905/906/910/918归档、036最终记录及[UI与语言接续设计](../../../auto-ui/docs/design/autoui/instance-state-and-reload-stage-one.md)。现Design53需同步归档与后继记录。观察语言HEAD aa6791649be2e2386c15b334d91fff74975cf684、UI 92ee2459a1058ddddc3fc3378ecec6585ffe726c、Man 2bbd9542c708c174c2ac71cd9c1f26811cc10c4c。

| 输入 | 已观察事实与限制 |
|---|---|
| test/fixtures/modularity/generated-ui-consumer/driver.py | SHA256 8304e0c3c8f75c73d83a17473e81007911e098d769bc9602472bb723729cc846；V04硬编码dev-run/rust-workspace/dev-run，两条dev shape/main字节断言因此需实证；公开参数--scratch/--skip-build-auto |
| 910规范/生成核心 | managed成员为basename-id8，项目锚/cwd无关、同basename隔离与context/receipt/lease已交付，不能为旧测试回退目录 |
| 036归档 | archived10/10，交付UI e06b65b2/Man 2bbd954/lang 2f558fc93；最终记录三具名原红V04×2/V05a，材料已删。V05a明确真bundle tool fresh locked构建，不能先断言只是测试问题 |
| 当前固定组 | auto-generation-core完整包、infra-fixed固定分组/缓存owner、web-delivery-controls四仓准入存在；未找到generated-delivery-controls，拟新建单一便宜模块组 |
| 918 | consumer/窄设施/基线有限交付；新manifest/源变化仍按owner重冻，不能代927/其他在途改动吸收未知边 |
| 927/928/929 | 语法/ACE独立合同，927 reviewed10/12、其余drafting；本包不改其语义/合同/实现树 |

本轮未重跑driver/metadata/audit或构建，三红来自归档收据、硬编码为静态实物。当前所有main源码/manifest/工具在T-01重新冻结，不以日期或HEAD数目替代一致性。036的tauri R-6、Web/Man其他债不因本包绿自动关闭。

## 5. 详细设计

### 5.1 启动与下一任务

实施授权待给。拟唯一组D:/autostack/.wt/lang-930/{auto-lang,auto-ui,auto-man,auto-web,auto-down}，语言分支codex/plan-930-generated-delivery；有实际改动仓各一树/分支，其他固定detached正式源，不建链接。实现/独立review同树冻结，不复制第二实现树，不借UI041/042或SG WIP。

T-01定位已有受管成员/receipt/metadata真实身份来源与准确输出，逐红最小复现并冻结错误/合法控制、V05a根因/owner、原完整case集合/原排除、准确工具/锁/命令/环境和资源。若超原生成交付契约需新后端/发行schema，具名重规划；不先改assertion/floor。原材料消失按最新源重测，不能索要不存在日志代替当前证据。

### 5.2 V04身份与V05a真实能力

成员定位优先消费现受管登记/receipt或真实metadata中的项目绑定，T-01确定具体字段，不复制Rust身份算法或新增相冲突命名约定。两个同basename不同根、错误owner目录、旧无身份成员、多个候选应区分/拒绝；不得glob第一个、仅以basename/mtime认领。main.rs只在实际选中正确文件后按原同源合同比较，确需允许不同元数据必须先有规范/语义证据，不能广泛删行归一化掩盖错误。

V05a冻结真bundle A先复现；若投影/锁/闭包/资源有真实生产缺陷，交对应UI/Man owner在同组最小修复和原负控，不从主检出补丢失文件/偷用旧exe。B重定位不含target，重建随包新tool并真实运行describe/validate_project/generate_ui及资源物化/协议错误门；A成功不可跳过B或反过来以B抵A。Web独立发行平台/schema不纳本包，必需闭包若牵涉其owner先给窄前置，不用跳过当通过。

### 5.3 固定控制入口与独立边界

拟新增scripts/tests/test_generated_delivery_controls.py及scripts/test-groups.toml固定组generated-delivery-controls（runner=python单一文件；work使其可执行）。只放真身份定位/产物比较/坏来源/子失败传播反控，不默认发起重Cargo/GUI、不编ACE，不构造新叶目录/影响图。原driver仍显式--scratch指定本计划scope，原V01..V06/工具wire/资源/主题义务不减；一项必需控制失败整体非0。

L1用外部语言SDK consumer真实API/VM消费与错/缺源拒绝；L2在仅语言来源的隔离快照执行锁定metadata、完整workspace build与明确受影响普通职责，不删除成员/optional/dev依赖或复制UI/Web实现。输入与历史有效证据身份完全未变可有据复用，否则重验；L1不冒L2，metadata不冒build。必需registry源可用条件如实记，不宣称冷依赖全离线。

918 self-test/strict/locked metadata按本包真实改面核；需重冻只在干净已提交组/参与源、审完全部新增边后进行，未知边不吸收、其他在途owner变化不顺手洗绿。若依赖未正式交付改动，保持阻点，不把语言语法/ACE全线变成本包前置。

### 5.4 测试与回滚纪律

consumer环境与工具hash明确；036记录的RUSTC_WRAPPER缺失/空串差异只作为待核本机事实，可按实际子进程env显式隔离并记录，不改机器配置/共享缓存。准备/编译失败与产品失败分列，不把包装exit0当子build成功。原jet/ark v0.7环境门控/平台条件保留，不新增无关平台义务。

修复前后保持旧交付失败保旧、恢复可再消费；只删除本次owned staging/target/进程，测试源码/SDK/输出显式统一scope。需要改被测source时旧证据按输入失效，final review后冻结必要并集一次。

### 规范增量

| delta_id | add/modify/retire | docs/specs/... target | before/after rule | rationale | acceptance IDs |
|---|---|---|---|---|---|
| SD-01 | add | docs/specs/repository-boundaries/generated-delivery-regression.md | 三具名原红/旧身份验证→正确成员/真tool/固定负控及全链支持 | 长期守门 | AC-01..04 |
| SD-02 | modify | docs/specs/repository-boundaries/generated-ui-distribution.md | V04/V05实际来源/正确形状与原wire/恢复义务 | 不弱化交付契约 | AC-01/02 |
| SD-03 | modify | docs/specs/repository-boundaries/standalone-language-build.md | 当前L1/L2/输入身份/错误与复用边界证据 | 独立构建持续可验 | AC-04 |
| SD-04 | modify | docs/specs/repository-boundaries/delivery-admission-and-boundary-policy.md（真实差额）；UI对应build页如实际投影差额 | 本包owner/冻结基线与旧外仓债交接，不扩大准入 | 保918边界 | AC-05 |

metadata只列本仓目标，跨仓Spec由实际owner差额冻结，无真实规则变化不虚构新能力，不写旧JSON。

## 6. 测试设计

以下全未执行，T-01绑定真实fixture/参数/原ID集合，T-02..05使新负控/全链可运行。work内部自查、review同组独立上下文冻结验证；review不修代码。真实A/B不能由合成unit green替代。

| Case ID | 不变量/真实输入 | 入口与区分性oracle | owner_stage |
|---|---|---|---|
| V-01 | AC-01；正确项目、同basename不同根、错owner/旧成员/歧义 | 新控制调用driver实际定位逻辑及真实dev生成，绑定正确member；错误显式拒绝，不选glob第一项 | work |
| V-02 | AC-01；V04 dev shape/main同源与单字节反控 | 实际dev/交付文件和metadata，合法形状/字节绿、错误成员或篡改非0；不是删除原assertion | work |
| V-03 | AC-02；V05a/V05b包内tool locked构建、外部协议/资源/错误与恢复 | 真A/B source/锁/tool exe hash与实际I-TOOL消费，坏帧/未知版本/缺资源/恢复原义务，禁止主检出补源/借exe | work |
| V-04 | AC-02/03；原V01..V06与新便宜控制必需项守恒 | 原driver全ID/退出码/receipt对账，新cargo t generated-delivery-controls；删case/子项失败/伪summary整体非0，合成绿不替build | work |
| V-05 | AC-04；隔离L1、完整L2与UI/Web/Man不可见、缺/错源 | 现language-sdk-consumer与冻结仅语言快照的metadata/build/职责；真实消费/缺源拒绝，无隐藏用户仓/删成员/旧target | work |
| V-06 | AC-05；source/owner/新依赖边/陈旧与dirty负控 | 原918 self-test/strict/锁定metadata及本包变更身份，未知边不随刷新合法化；原其他债/SG合同不被关闭 | work |
| V-07 | AC-02/05；生成/构建/恢复/清理失败、wrapper/工具来源 | 真子进程终态/旧交付字节/owned资源，失败非0、合法恢复后再次真实消费；环境事实与代码结论分开 | work |
| V-08 | AC-06；三红/全chain/独立源/原负控与SD | 同组独立final review绑定源/工具/产物，不能照收据摘要或测试数判pass | review |
| V-09 | AC-07/08；冻结必要职责、跨仓落地/规范/归档/全输出 | 被测==交付身份，canonical/路线/具名债实证读回、guard与材料/资源对账 | merge |

明确现命令：python test/fixtures/modularity/generated-ui-consumer/driver.py --scratch <scope/consumer>（--skip-build-auto仅有已冻结实构工具才可用）；cargo t auto-generation-core（该库真改时）、cargo t infra-fixed（分组实现真改时）、cargo t web-delivery-controls（共享四仓consumer真改或合同选择需要时），新generated-delivery-controls由T-03注册。不要为纯driver/docs跑所有语言/ACE/AG；UI生成/投影真改跑对应完整职责/必要consumer，由UI owner选t/tu单进程/必要ti；Man保真实CLI/安装调用面。

L2完整workspace build是本包独立边界门，不等于cargo t all；原批门到期仍由主检出单实例/现用户裁定，不自动串入本计划。列准备/冷编译/发现/运行/清理与实际执行/原条件排除、首红/环境重放/未测。原样本/负控/门控不能减。

## 7. 验收标准

| ID | 可观察关闭条件 | owner_stage |
|---|---|---|
| AC-01 | V04两项读取正确成员/真实dev shape与main字节合同，同basename/错owner/歧义拒绝有反控，V-01/02 | work |
| AC-02 | V05a根因实证闭合、A/B随包tool真实构建/消费，原V01..V06必需项/错误恢复不减，V-03/04/07 | work |
| AC-03 | 新固定组便宜控制可显式运行、失败整体传播，无tm/动态图/feature分叉，V-04 | work |
| AC-04 | 当前隔离语言L1/L2分别真实成立或身份未变证据有据复用，不删源/成员/借产物，V-05 | work |
| AC-05 | 918/036/source/审计/环境与其他owner/SG边界准确，三红未实证不闭、未知边不洗绿，V-06/07 | work |
| AC-06 | 独立final review逐AC/遗漏/workaround/健康/SD pass，绑定冻结源，V-08 | review |
| AC-07 | 冻结门/实际跨仓合入、canonical/路线/债务与归档同源读回，V-09 | merge |
| AC-08 | 全实际输出/owned资源对账，摘要折Plan后删scope/自有树、guard clean、持有具名，V-09 | merge |

## 8. 执行步骤

| ID | 动作/关闭结果 | 依赖 / AC | owner_stage |
|---|---|---|---|
| T-01 | 授权后唯一组/正式源与tool/准确命令/原ID/资源冻结，三红最小复现、V05a根因/owner和成员身份决定 [✅ 已完成 2026-11-05] 五仓组 lang-930（lang plan-930-dev@95d7ddfd8、ui plan-930-ui-dev@2a830d6c 基、man/web/down detached）；工具链 cargo/rustc 1.98.0；基线 driver 复现 **28 PASS/3 FAIL**（V04×2=910 id8 命名 stale、V05a=E0433 autodown_core 投影缺口）与 036 三红逐一吻合；consumer env RUSTC_WRAPPER 显式置空串（036 机器事实隔离，commit 95d7ddfd8）；T-01 证据 plan-artifacts/plan-930/auto-lang/t01-baseline-evidence.md | 授权待给；AC-01..05 | work |
| T-02 | driver正确成员/项目身份与V04 shape/字节比较，错源/歧义/同basename敏感性反控 [✅ 已完成 2026-11-05] commit 41ccccc3b：driver 加 canonical_key/resolve_front_member（消费受管 member.json：schema/role/project_path 恰一匹配，缺失/错owner/旧无身份/多候选具名拒绝，不复刻 id8 哈希）；V04 两腿按实际成员解析（fix 验证成员 dev-run-83e95a71 命中、dev/交付 main.rs 逐字节同源绿）；负控自检同basename不同根/错owner/旧布局/多候选全拒 | T-01；AC-01 | work |
| T-03 | 新便宜控制模块/固定组、子失败/必需项守恒，原全链入口/材料路径准确 [✅ 已完成 2026-11-05] commit 2e2fe40a0：scripts/tests/test_generated_delivery_controls.py（15 case：真身份定位/产物比较/坏来源/子失败传播）+ test-groups.toml 注册 generated-delivery-controls；`cargo t generated-delivery-controls` 15/15 PASS（含发现完备性自钉=删case判红、子失败整体非零传播） | T-02；AC-03/05 | work |
| T-04 | 按真实根因修V05a；必要UI/Man owner窄参与，A/B tool真实构建/协议/资源/恢复及原主题consumer [✅ 已完成 2026-11-05] UI commit b76483aa：DeliveryProjectionSpec 增 dep_rewrites（依赖行改写），PROJECTION_AUTO_UI_TOOLS 剥 tools 对 runtime 的 `features=["autodown"]` 请求（原形/投影形二选一校验、漂移判红、幂等）；新增单测 plan930_tools_dep_rewrite… 过，rust_ui::tests 57 过/1 既有红（shell-pack 过期，非本包）；顺闭 UI 910 过期测试 generate_rust_ui_out_of_repo_lands_project_local（按 member.json 登记定位）；重建 auto-ui-tool.exe；driver fix 验证 **41/41 全绿**（V04×2、V05a/b 82s/90s、V05c..h、V06a..e 全过，jet/ark 环境门控，receipt delivered_tool sha256=f9e084695…） | T-01/03；AC-02 | work |
| T-05 | 原全driver并集、当前隔离L1/L2与必要固定组/审计负控；健康/SD/三红与owner交接，execution_done6/9前置 [✅ 已完成 2026-11-05] 隔离快照 iso-lang（仅语言来源、无兄弟/无target/无.git）：L1 language-sdk-consumer 5/5、缺源负控（抽走 parser.rs→E0583 具名红 exit101、恢复→绿）、L2 metadata --locked exit0 + build --workspace --locked exit0（94.8s）；审计 modularity-audit self-test 16/16 + strict pass violations=0（本包零 manifest 变更→基线不 stale）；SD-01..04 落语言仓（commit a627ce130）；新固定组 15/15 | T-04；AC-01..05 | work |
| T-06 | work全部证据/source/环境/未测/SD与复用理由冻结，execution_done6/9 [✅ 已完成 2026-11-05] 证据：plan-artifacts/plan-930/{auto-lang/{t01-baseline-evidence.md,baseline-scratch/,fix-scratch/（driver-receipt.json 41/41）,iso-lang/},auto-ui,auto-man}；未测面如实记录（UI shell-pack 过期红、jet/ark 环境门控、Web/Man 其他债保原 owner）；work 全 AC 自查通过，无 open in-contract finding | T-05；AC-01..05 | work |
| T-07 | 同组独立上下文final review，finding回work/失效相应证据 | T-06；AC-06 | review |
| T-08 | 冻结必要职责并集，各实际仓落地/规范/路线/具名债务实证/归档读回 | T-07 pass；AC-07 | merge |
| T-09 | 全输出路径/进程/登记对账、摘要折Plan删scope/自有树产物，各树guard clean后移除/分支/组；archived9/9 | T-08；AC-08 | merge |

材料唯一D:/autostack/plan-artifacts/plan-930/，按auto-lang/auto-ui/auto-man身份分区登记owner/用途/在途。外部消费者/隔离语言快照/SDK/发行A-B/日志/显式target均在此，不用driver默认scratch或/tmp、其他盘散落第二证据根；普通构建在专属树。merge逐项对账实际所有输出/安装/产物/进程，命令/退出码/关键值/来源hash与残留条件折Plan后删scope/自有树产物；仅具名未闭持有有owner/理由/关闭条件，必需面不能借持有充PASS。未知/活动/他方不盲删、移除先wt-guard clean，不清910旧持有/共享target，不写旧JSON。

## 9. 复审记录

2026-11-04 / stage:new / PLAN-930 r1 / outcome:blocked（仅规划，实施授权待给） / 已核036最终归档三原红、driver旧成员路径与真V05a入口、906/910/918规范及现固定组；全部未重跑。next：授权后T-01冻结三仓源、复现/owner/资源；不假定V05a只是测试漂移，不重开918或接管927–929。

2026-11-05 / stage: work | plan_id: PLAN-930 | plan_revision: r1 | outcome: pass（T-01..T-06 全链，execution_done 6/9） | worktree: D:/autostack/.wt/lang-930/{auto-lang,auto-ui,auto-man,auto-web,auto-down} | branch: lang plan-930-dev / ui plan-930-ui-dev | base_commit: lang f4062cca6 / ui 2a830d6c | code_commit: lang 95d7ddfd8→41ccccc3b→2e2fe40a0→a627ce130；ui b76483aa | dependency_revisions: man 2bbd954 / web 3de5524 / down 7c4d5cb（detached） | task_ids: T-01..T-06 | evidence: 基线 driver 28 PASS/3 FAIL（三红复现）；fix driver 41/41 全绿含 receipt；generated-delivery-controls 15/15；audit self-test 16/16 + strict pass；L1 5/5 + 缺源具名红/恢复绿；L2 metadata+build exit0；SD-01..04 落树；plan-artifacts/plan-930/（t01-baseline-evidence.md + baseline-scratch + fix-scratch/driver-receipt.json + iso-lang） | blockers: 无 open in-contract finding；UI 既有 shell-pack 过期红 + jet/ark 环境门控 + 其他外仓债保原 owner，不阻塞 review | next_responsible_role: review（T-07 同组同树冻结 final review）

静态预检pass（2026-11-04）：0–10章节、9T/8AC/9V/4SD身份唯一连续，任务/AC的owner_stage与引用齐；本号跨活动/归档唯一、号源单调未回退，本地引用/尾空格检查通过。045–047三批恰覆盖B3十二例、无重复/漏项。仅文档一致性，不是新测试/探针或独立final review；全部实现/运行仍待授权work。

## 10. 待澄清事项

1. 本轮暂不实施；三红真实根因T-01有界冻结，公开契约/支持扩大需具体重规划，不能删断言/降低分母。
2. 041/042/OS及Demo主题批不是本包整项前置；930只验证真实生成/独立源，UI算法仍UI owner、Web/Man其他债留原owner。
3. 基线若因其他计划已变，不自动替其重冻；本包仅接已交付来源并按918审全部差额，dirty/未知边不能洗绿。
4. 原平台/jet-ark环境门控和资源条件保留，L1/L2不冒全语言测试/全部产品通过；环境wrapper差异只用可复核子进程策略处理，不改用户全机配置。
