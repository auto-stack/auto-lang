# 53 · 独立构建边界与生成Cargo产物治理

状态：目标设计与交付记录；2026-11-06。905研究、906独立边界、910有限Cargo治理、918准入/边界规则、UI036包发行/输入与[930生成交付回归及独立来源验收](../plans/archived/930-generated-delivery-regression-and-source-boundary-closure.md)已交付归档。[942共享生成workspace增量研究](../plans/archive/942-generated-workspace-incremental-sharing.md)为drafting r2、0/7；§8已按四项复审意见修订，待定向复审，仅设计与计划，暂不执行研究或实施。旧目标/研究测量按各自时点保留，不因归档数量冒全机Cargo问题已解决。

承接[Design40](40-compiler-artifacts-and-cache-lifecycle.md)、[Design50](50-autoweb-service-framework-and-repo-boundaries.md)、[Design52](52-single-compile-profile-consolidation.md)。统一的是来源、依赖与构建生命周期的可验证边界，不预定全局共享可写target，也不把源码分仓当独立构建已完成。

**2026-11-05 规划增量（11-06续订r2）**：930 已交付归档，当前守门见[generated-delivery-regression](../specs/repository-boundaries/generated-delivery-regression.md)；下文 §7 保留 11-04 立项时背景。§8「共享生成 workspace」最初为r1，现按四项意见修订为r2，配套 [PLAN-942](../plans/archive/942-generated-workspace-incremental-sharing.md) drafting/r2，待定向复审。用户仅授权形成及修订设计与计划，**暂不实施，也不运行研究构建**。910 的既有共享 target 是新对照的基线，不能再次认领其约 44%–45% 历史收益。

## 1. 两个独立目标

| 目标 | 主责/计划 | 完成边界 |
|---|---|---|
| 生成Rust工程/Cargo产物治理 | 语言侧持[PLAN-905](../plans/archive/905-generated-cargo-artifact-governance-research.md)，Man/UI/Web按实际producer/consumer参与调查 | 本期有界实测、方案选择、有限验收及可执行后继合同输入；研究完成不自动代表Cargo问题/M1已闭 |
| 语言仓不依赖Web兄弟检出 | 语言侧持[PLAN-906](../plans/archive/906-standalone-language-web-boundary.md)，Web/UI/Man参与必要接收 | Core独立消费和完整语言workspace分别证明；不能只改Core的轻合同path就冒全仓独立 |

二者没有无关全局先后；共享source_owner/rust_workspace/SDK投影/锁/发行入口改动须短窗单写。906改变依赖身份时，905沿新SHA更新对应观测，不混算收益；UI014结构/AG020产品在各自合同执行，不追加全线前置。

## 2. Cargo治理：从已存在机制出发

实际代码已有auto-generation-core::rust_workspace（目录解析/成员命名/共享workspace/target/lock身份）、source_owner（受管来源校验），UI rust_ui和Web/Man服务调用共同消费。TOML/config/environment及具体cwd会影响真实落点，已有机制≠已证明跨项目隔离、稳定性及物理减量。

905先用至少一个真实UI生成工程、一个真实Web生成工程，测完整依赖/应用/调试/增量/历史缓存账；再比较稳定写入/稳定成员身份、分兼容环境workspace、独立target配编译缓存等候选。没有证据不重造auto-cache，不预定一套巨型workspace，不合并多worktree可写fingerprint/incremental。Kache迁移已由用户取消，本设计不复活它，也不自行改文件系统。

缓存/产物复用必须绑定源与导入、generator/SDK/runtime/resource、lock/feature/profile/triple/flags/toolchain及build-script/环境实际输入；未变输入不重写失效，真实变更不错误复用。支持多项目/多工作树/版本并发、取消/崩溃与恢复；清理只能针对受管自有可重建内容，活跃产物/源/用户数据不误删。收益按store+所有实验工程的完整占用及迁移峰值核，不只看单目录或逻辑求和。

## 3. Web依赖边界：必须先冻结完整闭包

906立项时观察至少三条活跃边（现已按906收口）：Core→auto-web-contract、facade auto-lang→auto-web、auto-api-gen→auto-web；Core的http_service_config.rs整包再导出轻合同，该合同同时含严格解析、hash、CORS/Host/可信代理等Web政策，不能未经分类全搬回语言。

906先冻结L1（Core/API独立消费）与L2（语言workspace完整普通构建）清单、直接/传递/optional/build/dev依赖和当前SDK/生成/测试调用。候选：

1. 只将语言宿主真正需要的中立端口/数据交给合适基础owner，Web纯策略/算法留Web；显式adapter保持。旧公开类型路径/序列化/配置语义的兼容须真实证明。
2. 已有受管固定发布/离线依赖渠道若可承接，按固定来源消费Web轻包/必要组合，明确冷源可获得性；不能依赖开发机绝对path、未发布版本或global cache冒fresh可用。
3. facade/generator剩余Web实现边改为实际域owner装配，必要compat adapter按版本明确退出。不能悄悄删workspace成员、optional包或测试以凑“单仓”。

以上仅候选，不承诺复制整Web实现、搬全部政策、改ABI或另建仓。方案涉及破坏公开API/跨owner归属/删除兼容承诺/需要远端发布时，906 T-02给具体迁移合同供用户确认，未确认不进入依赖实施。保留一个真实类型owner；不复制同名Rust类型/政策，也不删除links冲突守卫来容忍同名双source。

**T-02 冻结（2026-11-01，探针 P-A/P-B/P-C 全绿）**：906 选中候选1+3组合（C1）——`auto-web-contract` 包逐字迁语言仓（Web workspace 去成员、auto-web path 翻转回指语言单源；links/类型身份/hash/严格策略随包不变）；facade 与 auto-api-gen 的 auto-web 边清零（facade 公开 Web 再导出面四仓零外部消费者、Man/UI/生成代码均 auto_web 直连；`auto_lang::http_service_config` 改指 core 再导出，RateLimiter/ServiceRuntime 留 Web）；http e2e 家族迁组内独立探针工程显式装配 Web host；source_owner/交付投影/安装脚本按新源身份接线。C2（合同留 Web+固定渠道）有据否决：registry 未发布、无 git remote、vendor 复制=第二套实现且不解决实现边。负控实证：未打补丁清单树 metadata 红于 `auto-web\crates\auto-web-contract\Cargo.toml` 缺失；C1 正控全 workspace 无 Web 兄弟 `cargo metadata --offline` 绿。完整设计/兼容差额/回退见 [PLAN-906 §5.5](../plans/archive/906-standalone-language-web-boundary.md)。

## 4. 验证和退出

两个计划都用专用worktree。外部验证在统一plan-artifacts scope隔离，无Web/UI/Man兄弟目录、无相关env/compiled-in fallback、保必要registry输入；不得移动/隐藏真实用户仓来制造环境。L1 Core API/VM正反消费与L2根Cargo metadata/build/普通固定组实际读取路径分列。服务装配仍显式消费Web，跨仓新组合fresh/locked/重定位/错误恢复和原子测试义务保持。

905只读/实验结果、906实施结果均需独立上下文review；源码/日志/生成工程/外置target只在计划scope，merge必要摘要折计划后全删，唯一具名未解决问题可持有。canonical在docs/specs，legacy specs.json只读，应用派生索引。

官方研究依据（2026-11-01查阅，实施仍核本机Cargo版本）：[build cache](https://doc.rust-lang.org/cargo/reference/build-cache.html)、[configuration](https://doc.rust-lang.org/cargo/reference/config.html)、[dependency overrides](https://doc.rust-lang.org/cargo/reference/overriding-dependencies.html)。这些是机制依据，不是本机收益或候选可行性证明。

## 5. PLAN-905 有界实测结论（2026-11-01 交付，研究非实施）

基线与全部实验证据折[PLAN-905](../plans/archive/905-generated-cargo-artifact-governance-research.md) §9；要点：

1. **现状判定**：Web 消费者路径（serve_rust/仓外 app）逐 app 本地 target 整编 auto-lang 闭包（生产 auto-chat 17.93G、auto-blog 8.48G，与 auto-lang/target 14.74G 三份同源 deps 互不复用）；rustflags 三源分裂（语言仓 config=lld+32MB、生成 ws config=256MB-STACK、裸默认）使共享 target 内部再裂双命名空间（实测 659 单元全量重编，census 落盘；单命名空间规模以 E2-S1 9.77GiB unique 佐证）。UI 路径 env 钉共享 target 但与语言仓自身构建混用同一目录（owner 混合）。
2. **复用成立的条件（实证）**：同 profile 同 flags 下，第二 UI 工程 22s/+1.11G（10x 时间）、Web release 跨消费者入共享 target +0G/6.8s 仅编译成员；取消/缺件/坏件均可恢复（fail-loud 或增量自愈，坏件需按单元 fingerprint 定位清除）；跨仓混合锚定 cargo fail-closed（好性质）。
3. **选定方向**：生成消费者统一共享 target + flags 单源化（§2 候选 B，按 profile 自然分 namespace）；不取内容寻址重建、不动 sccache、不合并多 worktree 可写增量。有限 M1 验收出口与后继实施合同输入见 PLAN-905 §9 T-05 记录（**后继实施不自动获权**）。
4. **移交缺口（不属本研究修复）**：生成成员键=项目 basename+mtime 判定 regen → 同名工程互劫持（双向实证）；needs_regeneration 无内容 hash；examples/ui/013-todo rust 轨生成编译失败（owner=auto-ui/UI014）。

## 6. Cargo有限治理交付接受（PLAN-910 archived9/9；2026-11-02）

[910实施合同](../plans/archive/910-generated-cargo-build-governance.md)接受906交付后的lang4571f55e9/Web847527e/UI98cbcd2/Man7f30eb8来源组合，开工再冻结当前版本。有限范围：生成构建上下文/flags单源与受管target归属、同basename成员及内容失效修复、并发/取消/恢复、精确闲置缓存预览回收。语言根工具target、跨worktree可写状态、只读SDK/独立delivery分界；不承诺不同feature/profile的合法变体消失，不改debug/incremental凑收益。

有限出口为真实UI/Web两类同义A/B消费者的正确隔离/复用、共享稳态unique字节严格低于独立总和、完整物理误差/迁移峰值/周转账及可回滚清理，再经独立review/实际合入。不是固定百分比或全盘历史清仓。906具名发行/布局/审计红按是否阻塞实际必需oracle判定，不以既有归因代替门通过。

UI007已完成整棵示例归位与首批主题，并接受910四仓版本、target/成员/receipt/配置合同和移根规则；910本身没有搬权威示例/改主题。后继031/033的新来源仍按910重新生成和验证独立消费，不恢复旧basename或mtime准入。910已实际交付（lang f665ce113/UI01f99fd/Web3de5524/Man67add3c），现知识以[生成生命周期](../specs/repository-boundaries/generated-cargo-build-lifecycle.md)为准；UI007亦已归档完成移根和首批，后继031/033及OS053按各自合同接受。D-910-1准入与D-906-3审计差额交918，Web/Man具名债及910缓存持有分owner保留；这些限制不抹成全产品已绿。

## 7. 036/918后继：真实回归对象与独立来源守门（2026-11-04）

036最终交付保留三项具名原红：792-driver V04×2（仍硬编码rust-workspace/dev-run，与910 basename-id8成员冲突）与V05a（真实bundle A内tool locked构建/815投影面）。[930](../plans/archived/930-generated-delivery-regression-and-source-boundary-closure.md)先重现/定位与冻结owner，修正确实过时的验收对象，并在必要UI owner窄改面修真正投影缺陷；不能为旧测试把生成目录回退，不能只删main.rs同源断言或以包B抵包A。

新固定generated-delivery-controls拟承接便宜身份/失败传播反控，真实原V01..V06、工具wire/资源/主题与A/B构建仍单独实际执行。语言L1 SDK、L2完整workspace隔离源分别核，不隐藏真实仓/删成员/借旧产物；本包新变化按918干净source/审边规则维护，其他语法计划变化不自动重冻洗绿。所有新探针均未执行。

UI041状态实例、042 Reload研究、045–047教学B3与930独立落约；真实OS产品主题线仍在OS各仓合同，不成为这些框架/语言守门的全局前置。Tauri R-6、vue-render外部发行渠道、Web发行/ID集合schema及Man派发等保持原owner，未由930自动关闭。

## 8. 共享生成 workspace：第一代目标设计 r2（2026-11-06；修订后待定向复审，未执行）

### 8.1 目标和现状边界

目标是把**同一兼容构建环境**内多个应用生成的 Rust 成员纳入 Auto 管理的 workspace，集中成员、Cargo.lock、workspace.dependencies 与构建策略，验证相对「独立 workspace + 已共享 target」的**额外**空间收益。源码项目和仓库保持独立；Rust 生成目录是可重建状态，不成为新的源码 owner。

现状以[generated-cargo-build-lifecycle](../specs/repository-boundaries/generated-cargo-build-lifecycle.md)和实际代码为准：

- `auto-generation-core::rust_workspace::resolve_rust_workspace_dir` 已支持 `AUTO_RUST_WORKSPACE` 权威覆盖，框架内共享、仓外 project-local；可先用现有覆盖做有限候选实验，不先重写生成器。
- `GeneratedBuildContext::resolve/cargo_env` 已把同 producer/flags 的生成消费者送入共享 target；`WorkspaceGenerationGuard`、`TargetLease`、`member.json`、内容收据、写前比较和带 id8 的 bin 已存在，必须继承。
- UI `front_needs_regeneration` 和 Web `backend_generation_is_fresh` 仍读取**整个 workspace lock** 身份。集中锁会扩大受影响范围，不能默认等同更少生成或重编。
- 11-05 的 20 个现存应用配置只读快照为：14 个同一 target、3 个 worktree target、3 个其他落点；3 个应用还存在旧本地 target。配置与目录存在不等于实际运行或占用测量，不把改落点/删除旧目录算作新 workspace 收益。

第一代候选只覆盖已验证的 UI Rust 前端、Web Rust 服务后端及 Man 调度入口。不覆盖所有裸 a2r 工程、测试临时 Cargo 工程、Tauri `gen/back/rust`、工具仓 target、跨 worktree 可写增量、完整 AutoCache/CAS 改造、动态链接或全机清仓。

### 8.2 候选布局与兼容环境

逻辑布局（**不是已安装路径或已批准的新默认配置**）：

```text
<Auto 受管生成根>/<cohort-id>/
  Cargo.toml / Cargo.lock / .cargo/config.toml
  .generation.lock / <workspace 归属记录（拟新增）>
  <basename>-<id8>/              # 前端；沿用 910 平铺成员布局
  <basename>-<id8>-back/         # 后端；沿用 910 成员/包/bin 身份
  target/                      # 逻辑示意；也可仍指向 910 受管 target
```

继续平铺成员以复用现有扫描器，不为排版新增 `apps/` 层。生成根的开发/SDK 默认物理落点、与 910 target 的绑定和回收准入由 942 研究冻结；本稿不写用户缓存配置，不在只读 SDK 下写状态，不新增第三套 blob/索引系统，也不假定 AutoCache GC 已能安全处理 Cargo 树。

`cohort-id` 是稳定的**兼容来源族**，不是每次构建输入内容 hash。至少区分 producer 检出/安装身份（工作树不合并）、工具链/host-target、flags/构建策略和 UI/Web 所需来源组合。源码、生成器或某应用 lock 的每次内容变化应记录并由正确失效路径处理，不能一改代码就另建整个 workspace 来回避增量。具体字段、兼容判定和演进规则由研究决定；不能仅复用当前仅含 producer 路径/flags 的 ctx8 冒充完整 cohort。

同 cohort 的兼容应用默认集中是**目标态**；环境覆盖仍须权威且验证来源。服务专用环境没有 UI SDK 时继续独立可用；只有完整来源组合兼容才研究 UI/Web 混合 workspace。不能为「全局唯一」迫使所有服务安装 UI 或删除不同 SDK/版本的隔离。

### 8.3 依赖、锁与配置

成员显式声明实际依赖并使用 `workspace = true` 继承。共享根统一依赖来源和版本策略；不引入一层包含所有依赖的大 crate，也不为复用一律启用全部 features。`-p <精确成员包>` 是日常构建入口，不默认 `--workspace` 编译全部应用。

集中锁必须保留各应用允许的版本范围、sidecar 依赖和公开行为。冲突无法兼容时，分兼容环境或具名拒绝并保留旧产物，不能静默升级、删依赖或放宽 links/source_owner 守卫。根级 patch/profile、Cargo config 发现链、运行 cwd、资源与 sidecar 相对路径和 wrapper 必须逐项对账，不能把搬目录引起的配置漂移计为收益。

新成员加入或锁更新的三个效果分开记录：生成收据变陈旧、重新生成字节、Cargo 实际重编。第一代研究先保持 910 整锁失效，量出它对其他应用的影响；不静默省略 lock 校验。如证据要求按成员依赖闭包判新鲜，则把完整来源/feature/build-script 失效合同作为后继实施前置，不在本研究中发明未经验证的替代 hash。

### 8.4 所有权、并发和导出

原项目路径仍是应用身份/输入/运行目录的锚，basename+id8 和 `member.json` 不变；新共享根须记录 owner、来源、成员注册和格式版本。旧 project-local 生成物不自动认领、搬移或删除；移根后的发布件发现必须能准确映射成员，不能继续猜 `<project>/rust-workspace`。

现有 ensure 锁只覆盖扫描和根清单/config 写入，不能据此宣称整段生成/构建/发布事务已串行安全。研究须观察成员首次注册、同成员并发、Cargo.lock 写入、构建期间另一成员加入、取消和恢复；失效旧成员不得毒化无关应用。若现有锁范围不足，提出明确的登记/发布/锁序后继合同，不在研究主线顺手改生产实现。

应用注销不删除仍被其他成员使用的依赖，活动 lease 阻止破坏性清理。研究只对自身隔离副本做删除/故障实验。全局预算 GC、自动迁移历史项目和全盘旧 target 回收均为后续工作，不能靠清空共享根完成验收。

独立导出分两条证据：930原driver保原oracle验证旧project-local路径；942 T-02另提供scope候选适配器，正式生成到B，准确定位唯一member.json/role/project_path及该相位的生成源码/sidecar/资源，再经正式delivery入口生成包，实际移根、locked构建、包内工具/资源与应用行为验证。B真实内容必须与交付产物同源对账；若delivery入口从Auto源再生成，标明“同源再生成”，不能谎称它读取B生成成员。

候选负控必须实际移走/篡改B成员或给错根/另一App成员，来源/内容断言非0且不能自动修复B或回退旧路径；合法恢复再验。消费阶段剥离开发根，开发阶段保B环境。930 driver目前清除AUTO_RUST_WORKSPACE且V04固定project-local发现，它通过不证明B；固定控制也不替代真实包构建。

所有探针、driver依赖布局、工具及target/build-dir/消费者输出在plan-942 scope。driver推导的owner根必须指向scope副本，--scratch或--skip-build-auto都不能单独证明隔离/工具身份；T-02先核两件工具绝对路径/hash及每命令cwd/env/全部输出。不改生产driver/CLI/生成器，原入口不满足B候选时具名差额。执行合同见[942 §5.1.1、V-08/13](../plans/archive/942-generated-workspace-incremental-sharing.md)。

### 8.5 对照设计：只核新增收益

| 臂 | workspace/锁 | target | 回答的问题 |
|---|---|---|---|
| A（主基线） | 各应用独立；保当前合法锁及解析策略 | 应用间已共享，910 策略 | 今天已有共享 target 的真实成本 |
| B（候选） | 同一兼容族集中 workspace/锁 | 臂内共享；与 A 物理隔离 | 集中后的端到端新增收益/代价 |
| C（归因控制） | 各应用仍独立；逐阶段/精确调用匹配B的兼容解析、实际features与锁/缓存历史 | 臂内共享；与 A/B 隔离 | 有多少收益仅来自锁/版本收敛，而非 workspace 集中 |

A/B必须实际执行。B/A任一阶段的解析、实际feature/编译图或锁迁移不同，C必须尝试；免C需逐阶段/逐成员精确调用对账，不只看最终Cargo.lock。C不能起始就套B最终锁，须从空的自有缓存复演相同锁选择/迁移及构建历史，不复制B产物。A保当前真实合法锁/历史，不为归因预先升级。

归因记录包含stage/invocation、命令/成员/输入/工具/config身份、normal/build/host/target依赖边、source/version、实际激活features/profile/target、锁前后及缓存保留代次。metadata/tree和请求features不能单独证明实际编译图，需关联Cargo JSON与可核调用/配置记录。无法合法匹配或观测时，相应ΔAC/ΔCB仅描述性差额，不下纯锁/纯workspace结论；完整可信A/B端到端结果仍可保留，代数拆分不证明因果。V-11专测“最终图相同但历史不同”及“请求features相同但实际编译图不同”，合法控制也须接受。

真实样本至少包含两个不同项目身份的 UI Rust 应用和两个 Web Rust 服务：先冻结当前健康的 `auto-ui/examples/ui/002-counter`、`auto-ui/examples/ui/013-todo` 候选，以及 `auto-chat`、`auto-blog` 服务路径。某样本不支持/原红时具名说明并在合同中冻结等价真实替代；不能事后换成空壳或删原 oracle。以两类工作集分别核算，混合 cohort 仅在来源完整时另测，不把 UI/Web 不同 profile 强行合并。

每臂使用同一套冻结工具/源码/应用快照、profile（UI debug、Web release）、flags、features 请求、target triple、debug/incremental、wrapper、配置发现链和 registry 输入。只有 workspace/解析策略为自变量；版本分叉是被测事实，按 B/C 单独归因。

测首次加入、第二及后续应用加入、无修改重跑、单应用编辑、共享依赖变更、成员加入/移除和有限编辑周转；不只测冷启动。观察编译单元/fresh、生成bytes/mtime、锁/收据、真实行为、阶段耗时及峰值/稳态。T-01预先冻结轨迹/恢复、输入路径与污染政策，优先同一canonical项目路径跨臂顺序运行，env区分生成根，业务数据隔离，不并发编辑共同输入。无法消去的生成路径/debug/member身份差额要计量与控制，不能仅称“单列”就默认不影响微小收益；详942 §5.2/5.3。

### 8.6 空间账与决策规则

**主口径固定**：total_alloc(X,s)为阶段s臂X按卷+file-ID去重的可归属文件系统分配字节。另列去重logical和路径逻辑和，不能临时选较好数字作total。文件系统/API不支持该归属时标unknown，不把logical或file-ID假称CoW/物理块证明；主口径缺测阻塞物理收益验收。

分类账为generated（源码/清单/锁/配置/收据）、build（全部实际target/build-dir、deps/build/incremental等）、published（exe/PDB/资源）、residual（临时/失败残留）。每文件身份一次归属，跨类/嵌套/硬链接保别名与去重校正，build/published共享优先归build，不能各目录简单相加。公共源/registry/工具/计量与验证额外副本分别记common/validation；若随候选策略变化是必要成本则纳对应臂，不能移出隐藏。主/副口径、每类Δ及未知/漏账需同时披露。

ΔAB=total_alloc(A)-total_alloc(B)，rAB除以total_alloc(A)（0时N/A）；ΔAC/ΔCB只在§8.5逐阶段匹配成立后解释为分量收益。UI/Web及冷/稳/编辑分开，不平均藏负值；历史旧target清理不进新增公式。V-12的分析输入A逻辑20GiB/分配10GiB、B逻辑12GiB/分配12GiB，主Δ必须为-2GiB（-20%），逻辑+8GiB仅副指标，不能判物理节省。

**两类峰值分开**：实验预留和观测包括A/B/必要C、全部common/validation、SDK/工具/发行/临时副本的文件身份并集；测量记录采样间隔/盲区，已见峰值不是绝对上界。迁移是后继策略估计，明确是否复用910 target、复制/暂存/回退持有和释放；不把A+B+C实验共存叫生产迁移实测。策略/数据不足则估计未定并列owner，不能迁移/清理用户目录取得数字。T-01冻结可承受上限/停止条件，不虚构用户数值预算。

**两类误差分开**：ε_meter来自计量分辨率/静置读取，只能说明读数；ε_build来自等价输入/路径身份及锁/缓存历史下的构建重复/顺序控制。边际正收益须最小匹配重建或顺序反转，不能静置两读相同就令构建波动为0，不套三warm/P90。给各阶段AB/AC/CB分别适用的保守E_delta及两臂meter/build/path来源、单位和范围；有限重复不冒统计置信度，无法界定则不可判定。V-14控制要求meter=0而构建波动覆盖微小正差额时禁止go。样本/污染保留，不测后择优或放宽界限。

| 结论 | 必要判断 |
|---|---|
| go | 可信完整A/B和所需C、必需正确性/隔离/导出门通过，ΔAB-E_delta>0，分量有因果资格，C适用时另需ΔCB-E_delta_CB>0以排除仅锁收敛，无阻断生产边界；仅建议后继 |
| conditional | 完整可信A/B已做，但归因有限、仅锁收敛、波动内正差额不可判定或有具名生产前置；不声称已证明workspace净收益 |
| no-go | 有效计量支持零/负收益或已复现必要语义/隔离失败；保910默认 |

没有预设节省比例，零/负/有限归因结论可交付；A/B未完成、关键主口径/必需真实oracle缺失或账不可信仍阻塞，不以conditional掩盖。当前所有数值反例均为设计输入，不是实验结果。完整可执行schema/任务见942 §5～8。

### 8.7 计划分解与当前授权

[PLAN-942](../plans/archive/942-generated-workspace-incremental-sharing.md)只承担有界研究和决策：冻结基线/工作集 → A/B/C 对照 → 周转与负控 → 冻结收益、风险和后继实施合同 → 独立复审与文档沉淀。2026-11-06按R1四项问题修订为r2，修订后待定向复审；本轮只有文档修改，所有研究任务未执行；不创建 worktree、不运行 Cargo 构建/测试、不迁移/清理应用。

未来研究获授权后，唯一 worktree 为 `D:/autostack/.wt/lang-942/auto-lang`，全部探针/外部消费者副本/target/日志在 `D:/autostack/plan-artifacts/plan-942/` 按 arm/owner 登记。研究不修改生产 Rust/配置，不新建长期缓存服务。后继生产实施在单独 Plan 确认具体生成根、兼容键、根锁/收据、调用方与迁移/回退，避免无收益也为完成实现任务而强行落地。

机制依据（2026-11-05 查阅；在线文档可能领先本机 Cargo 1.98.0，实施研究须核实支持）：[workspace](https://doc.rust-lang.org/cargo/reference/workspaces.html)、[resolver](https://doc.rust-lang.org/cargo/reference/resolver.html)、[configuration](https://doc.rust-lang.org/cargo/reference/config.html)、[build cache](https://doc.rust-lang.org/cargo/reference/build-cache.html)。这些说明机制，不证明本机额外收益。


## 9. 调试策略C生产落地（target-state，2026-11-10）

1001已交治理/对照研究，用户决定采用C；[PLAN-1003](../plans/archive/1003-generated-debug-policy-c-production.md)负责正式生产默认，不将研究scope配置冒已上线机制。保910共享target/成员身份/lease、优化/断言/栈/增量/release，不采用集中workspace来凑节省。

单一中立策略从`auto-generation-core::rust_workspace`与`GeneratedBuildContext`贯通UI/Web/Man、SDK/delivery、实际Cargo单元/发布收据/进程。默认成员line-tables-only、非成员依赖debug=false；Cargo真实成员集合决定分类，delivery中的vendored SDK可能变成员，不假称所有path依赖均被去debug。生成根每次ensure重写，配置落点/用户显式覆盖/完整debug恢复由1003 T-01最多两候选决定；禁止写用户全局祖先config、误继承旧package override或靠环境标签宣称有效。

策略及有效覆盖须参与可验证新鲜度/发布身份；文件write-if-changed、last-good失败保全、活动lease和他人target保护不退。四真实入口/移根SDK和成员行号、原默认/C同源净收益/误差/成本、迁移共存/回退/旧非增量变体显式治理均须真实验收。1001只清incremental，不能借此删旧deps/PDB；不自动清全局缓存。研究−51%不是1003承诺值。

1003正式交付→OS068最终C配置下同源重建/完整RQ终验→989治理+RQ/渠道联合核查；OS068清单/有界修复可先，1003不等OS068或两个M2。D-1001-R7仅在1003正式生产交付后关，当前仍OPEN。本文是目标架构追加，不改canonical现状、不实施。


## 10. Web成员独立交付与恢复（target-state，2026-11-11）

[1006 r1](../plans/1006-web-generated-member-independent-delivery-and-recovery.md)是D-942-P2有限生产合同，auto-web领域主责、auto-generation-core中立支撑，Man只接公开CLI。中央登记沿Web既有890/990模式，不新建Web号源；本轮仅立约、不实施。上节1003等待态是当时target-state记录，当前1003已正式交付、P1/R7已闭，事实以其[生产Spec](../specs/repository-boundaries/generated-debug-policy-production.md)为准；不重做C落地或集中workspace研究。

现有`generate_api`写开发workspace成员，Web SDK安装搬走整个SDK组；目标另交明确export-only入口（拟`auto service-export --dir <project> --delivery-root <bundle>`，尚不存在），包内成员/SDK/资源/锁/有效debug策略单一可核。依真实成员role和source owner选择，不靠basename/glob或环境改写重定向；不得从主检出fallback补依赖、复制UI/Man算法或扩中立层成业务框架。原开发生成/服务启动行为保持。

发布用owned staging与既有最小提交机制：坏源/错SDK/缺资源/失败提交保全last-good，私有用户数据不入包，资源越界/reparse拒绝。公开CLI→普通Auto真实业务→仓外fresh/locked→移根HTTP/错误/持久数据独立读回才关闭P2；仅SDK安装、预编译backend或200空响应不够。真实chat/database与1003依赖闭包/C策略须纳支持面，未来ACE M2不是本计划前置。

跨渠道恢复唯一对象合同见[恢复清单r1](../plans/reports/split-refactor-recovery-objects.md)：UI独立包、语言/Man/Web SDK、Windows RQ包组+Todo数据、首版Web A/B成员包。Web无历史导出包，按同一正式导出器生成有实际版本/资源差异的两个真实源包，验同schema替换/失败保全/一致备份恢复，不冒历史发行兼容。包原子提交不等数据库事务，代码/资源与外置数据分开，恢复后须业务和独立磁盘读回。

顺序为1006实际交付→989 r8 E-03解除，989按恢复合同与真实source/命令/schema核其余E门→T-02冻结→T-03～05验证。当前1006只有草案、实际候选绑定未齐；对象定义不等工件hash已产出/恢复已验。989 T-01准备可在1006实施时并行，Linux/ACE/AGE另线；1003/OS068不重做。本target-state不改canonical现状、不打tag或部署。
