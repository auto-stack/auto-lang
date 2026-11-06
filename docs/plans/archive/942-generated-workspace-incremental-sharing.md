---
plan_id: PLAN-942
status: archived
feature_name: 共享生成workspace增量收益验证与方案冻结
author: [Codex]
created_at: 2026-11-05
updated_at: 2026-11-06

plan_revision: 2
supersedes_spec_components: [docs/specs/repository-boundaries/generated-cargo-build-lifecycle.md]
new_spec_components: [docs/specs/repository-boundaries/generated-workspace-sharing-study.md]
touched_goals: []
affects: [generated-cargo-build-lifecycle, generated-workspace-sharing-study]
depends_on: [PLAN-910, PLAN-930]
current_step: 5
total_steps: 7
---

# [PLAN-942] 共享生成 workspace 增量收益验证与方案冻结

## 0. 变更摘要

承接 910 已交付的共享 target/flags/成员身份/内容收据，以「独立 workspace、已经共享 target」为 A，与「同兼容族集中 workspace」B 比较；必要的 C 逐阶段、逐调用匹配 B 的实际依赖/feature图、锁迁移及缓存历史；匹配成立才作分量归因，否则只报端到端差额和限制。不能复算 910 的历史收益。

本计划是**有界研究与决策合同**，不是生产切换合同。形成新增节省字节/比例、生成/编译周转、锁耦合、隔离/恢复和独立导出的证据，再决定是否另立生产实施计划。目标设计见 [Design53 §8](../design/53-independent-build-and-generated-cargo-governance.md)（共享生成 workspace r2）。

**当前授权：2026-11-06按四项复审意见修订设计/计划至r2，暂不实施。** 当前修订后待定向复审；原R1结论保留为历史，不代表r2已经通过。 T-01..07 全部未执行；未建研究 worktree/探针/缓存，未运行研究构建/测试。只读 `cargo t --list` 是入口核实，不是测试通过证据。

## 1. 目标

- G1：冻结真实 UI/Web 生成链和完整空间口径；证明 A 已共享 target，不能拿逐项目独立 target 作主基线。
- G2：取得 A/B 及必要 C 的冷态、稳态、编辑/锁变化周转账；逐阶段匹配才作分量归因，报告分配主口径和重复性界限，历史清理不计新收益。
- G3：通过真实应用和区分错误的控制，查明成员身份、来源隔离、锁/收据耦合、并发/恢复、运行 cwd/资源和独立导出边界。
- G4：冻结 go / conditional / no-go 结论及后继合同。研究完成仅代表结论可信，不代表新 workspace 已上线。

非目标：修改生产 Rust、切默认生成路径、迁移真实应用成员/锁、删用户旧 target、降 debug/incremental/profile/功能换空间、引入 sccache/Kache/新 CAS、合并跨 worktree 可写 target、重构 AutoCache、全盘清仓、Tauri/裸 a2r/ACE 工程治理、产品主题/语法线交付。

## 2. 架构方案

`auto-generation-core` 继续是共享生成设施唯一 owner，UI/Web/Man 保持现有域归属。研究优先通过既有 `AUTO_RUST_WORKSPACE`、`AUTO_GENERATED_TARGET_ROOT` 操纵**隔离副本**，不用新生产实现证明自己可行。

候选继续 910 平铺 `<basename>-<id8>` / `-back` 成员，集中 workspace.dependencies、锁及根配置；应用源仍属原仓。未来 cohort 至少区分 producer 检出/安装来源、工具链/target、flags/策略和 UI/Web 来源组合，不能每改源码就新建整个 workspace。默认物理路径、cohort 算法、安装 SDK 模式待研究冻结，本稿未承诺新环境变量。

保持服务单独可用、工作树可写隔离、来源验证和独立 delivery。如果需要改变整锁新鲜度、锁范围或发现方式，记录后继合同，不在本研究顺手改生产代码，也不靠省略校验获得更好数字。完整目标设计/布局/口径以 Design53 §8 为准。

## 3. 技术栈

现有 Rust/Cargo 生成入口与环境覆盖；Cargo JSON artifact/fresh、`cargo metadata`/`cargo tree`；Python/PowerShell；NTFS 卷+file-ID/分配字节测量。Cargo 内部布局不是稳定 API，不能猜 deps 文件名认领发布件。

规划环境：Cargo `1.98.0 (797e8a9bc 2026-08-05)`。在线文档可能领先，本机 target/build-dir 支持在 T-01 核实；外置 build-dir、wrapper cache 与全部实际输出要纳账，不能只量 target。

## 4. 需求分析与背景调查

### 4.1 授权与工作树

用户：「按这个方向先形成共享生成 workspace 的设计和计划，承接 PLAN-910，重点验证相比现有共享 target 还能节省多少空间，暂不实施。」

2026-11-06用户要求「按这四项复审意见修订 PLAN-942 和 Design 53，升到 r2，把对照、空间口径、收益归因和验证入口写清楚，暂不实施。」本轮授权修订942、Design53和相关跟踪指针，不另取号。当前只授权文档修订。研究实验、创建 worktree、生产/应用修改与缓存删除均未获权；未来执行须收到用户授权。用户未指定数字预算、收益阈值、模型或审查人员，本计划不虚构。

未来唯一研究 worktree：`D:/autostack/.wt/lang-942/auto-lang`，分支 `plan-942-dev`（按本仓AGENTS.md）。UI/Web/Man 主检出只读；所需应用/依赖/工具副本在统一 scope。若需跨 owner 改生产码，先形成独立获权合同，不给研究添加隐藏实施阶段。

### 4.2 规范与规划快照（执行前重绑）

已读 `docs/specs/overview.md`（部分仓地图为旧快照）、`docs/specs/README.md`、[生成生命周期](../specs/repository-boundaries/generated-cargo-build-lifecycle.md)、[生成发行](../specs/repository-boundaries/generated-ui-distribution.md)、[930守门](../specs/repository-boundaries/generated-delivery-regression.md)、[生成owner](../specs/auto-api-gen/project.md)、[auto-cache](../specs/auto-cache/project.md) 和 Design53。它们没有「全局生成 workspace 已启用」承诺。

| 来源 | 规划时身份 | 用途 |
|---|---|---|
| auto-lang master | `5f67374675e1ea9c5278e39d0dd24849faa7e8dc` | 当前共享生成实现；主检出其他 WIP 保全 |
| auto-ui | `e3f0933d2384921aa49df8007cf9730ad7723692` | Rust UI 生成/构建/发布/导出 |
| auto-web | `37d32c22d1a4e7744710bf8f02f6c74d5a96c897` | 服务生成及新鲜度 |
| auto-man | `03bb65ef7fb6181caa6a08d9752ac7d57c75516c` | 服务调度 |
| `rust_workspace.rs` SHA256 | `2a58165f169c0ceea023f16f9dd8bbcd8fc4837d41b7d7f5495691a5190165a2` | workspace选择/扫描/根清单 |
| `generated_build.rs` SHA256 | `1782b9a812866bcae884d6f14e8b1fff2a69fbf6fc2310b55b2534de026fe890` | context/flags/成员/锁/lease/收据 |
| 生成生命周期 Spec SHA256 | `efb4a768d14a405baaab42bef33895ca26ce0233a7ea88d459cf62b953ddfc35` | 现行合同 |

HEAD 不证明没有未提交改动。T-01 核 Git 状态、相关文件/锁/工具哈希与应用快照；来源改变则重绑受影响证据，不借并行计划的代码变化冒本计划收益。

2026-11-06取材快照：语言HEAD 5440503189a53554c25a7fa490ad1971d41774bd；上述两Rust源及生命周期Spec的SHA256与表一致。930真实driver SHA256=90036a35461193794b48f3a2ed021c407ae71039d28a587bfd33cf8652176c2d。重读consumer_env、main、V04：清除AUTO_RUST_WORKSPACE、按owner根target找/建工具、固定project-local发现；member_id8仍由canonical项目路径派生。执行前重绑，不以移动HEAD冒内容身份。

### 4.3 继承与已观察事实

- 910 已交付共享 target、flags 单源、同名隔离、写前比较、收据/lease/回收。其 UI 21.02→11.72GB、Web 3.19→1.744GB 是**独立 target 对共享 target**的历史结果，不是本计划新证据或阈值。
- 930 在 `docs/plans/archived/930-generated-delivery-regression-and-source-boundary-closure.md` 为 archived/delivered。接受 member.json 精确发现、dev/delivery 同源、真实 A/B 包消费与便宜固定控制的不同职责，不重复立项。
- 11-05 静态快照：20 个应用生成配置中 14 个指同一 target、3 个 worktree 来源、3 个其他；chat/database/solitaire 存在旧本地 target。未量空间/运行，不把改落点或删旧目录计新收益。
- 当前仓外仍 project-local workspace；根清单扫描平铺成员并推导 UI/Web 来源需求。集中后坏成员、失效 path 或缺 UI SDK 可能影响其他服务，不能只测两个好成员。
- UI/Web 收据使用**整个 workspace lock**身份；必须区分 receipt stale、生成文件重写和 Cargo 真重编，不能许诺集中后所有修改只影响单应用。

### 4.4 执行纪律

使用 `C:/Users/zhaop/.codex/skills/auto-plan-new/SKILL.md` 和 `references/verification-contract.md`，用户仓库规则优先。将来全部材料只在 `D:/autostack/plan-artifacts/plan-942/`，开工登记 owner/purpose/path/in-flight。资源不足具名阻塞，不缩样本或降配置；当前纯文档规划不建 scope。

## 5. 详细设计

### 5.1 已核实入口与研究处理

| 对象 | 已有生产路径/符号 | 研究范围/关闭条件 |
|---|---|---|
| workspace/来源 | `crates/auto-generation-core/src/rust_workspace.rs::{resolve_rust_workspace_dir,try_ensure_workspace_for,select_language_producer}` | 现有env控制scope副本；记录实际cwd/config/依赖来源 |
| target/身份 | `crates/auto-generation-core/src/generated_build.rs::{GeneratedBuildContext,claim_member_dir,WorkspaceGenerationGuard,TargetLease}` | 继承flags/成员/lease；A/B/C分别target，显式根先预建，不借别树target |
| UI | `auto-ui/crates/auto-ui-tools/src/rust_ui.rs::{generate_rust_ui_with_request,front_needs_regeneration,build_rust_ui_with_request,generate_rust_ui_delivery_with_request}` | 真实输入→成员→编译→发布/发现→行为；bytes/mtime/receipt分层观察 |
| Web/Man | `auto-web/crates/auto-web/src/api_gen.rs::{generate_api,backend_generation_is_fresh}`；`auto-man/crates/auto-man-runtime/src/http_service.rs::serve_project` | 真实生成/受管构建/HTTP响应，exit0不代替行为 |
| delivery | `test/fixtures/modularity/generated-ui-consumer/driver.py`；`scripts/tests/test_generated_delivery_controls.py` | 930原oracle只验旧路径基线；B候选由T-02提供scope适配器及来源负控，分别实际构建/移根/消费，见§5.1.1 |
| 清理 | `scripts/test-governance/build_storage.py` | 不扩生产准入；只研究scope自有副本，记录后继根/lease合同 |

### 5.1.1 真实候选导出与输出隔离（F-942-R1-03）

1. 930原V01..V06语义/平台条件保留，在scope复制其所需语言/UI/Man等真实依赖布局；显式指定scratch子目录、cwd、工具和全部target/build-dir。driver推导的owner根必须是scope副本，不能直接在主检出运行后假定--scratch隔离所有输出。--skip-build-auto不证明两件工具身份；先核auto CLI/UI tool绝对路径、hash和来源，禁止陈旧工具/主检出回退。
2. T-02提供拟新增candidate-delivery适配器（可集成workspace_study.py）：正式生成到B准确根，定位唯一member.json并核role/project_path、成员清单/main.rs/sidecar/资源及输入相位，再走正式delivery入口。若入口按Auto源再生成而非复制Rust成员，如实标“同源再生成”，以同一冻结输入/工具、准确B成员与delivery字节/资源对账证明等价，不称生产导出器直接读取B。
3. 完成B来源/内容对应后，实际移根，剥离研究生成根/开发根覆盖，完成原有locked构建/运行、包内工具/资源验证。开发阶段设置B环境，独立消费阶段才剥离；消费输出仍显式放scope。旧driver的project-local V04通过不能抵B候选。
4. 来源负控：实际隔离移走B成员、篡改其有效生成内容、指定错B根/另一合法App成员，必须在对应来源/同源断言非0，不能自动修复B或回退旧路径继续绿；保实际路径/hash，恢复合法B再验。不能只比文本而跳过真实构建/行为。
5. 只改scope研究脚本，不改生产driver/CLI/生成器、不减oracle。T-02在重构建前冻结每条命令的exe/argv/cwd/env增删、输入hash、全部输出、退出传播及清理owner；存在主检出默认输出或未知输出不进入T-03。原入口无法完成B链须具名能力差额，不伪造成功。

### 5.2 对照、样本与操作（F-942-R1-01/04）

| 臂 | workspace/锁策略 | 构建根 | 条件 |
|---|---|---|---|
| A 主基线 | 各应用独立，保当前合法解析/锁和自然演化 | `<scope>/arms/A/target`，臂内共享 | 保910共享target，不为C预升级A |
| B 候选 | 同兼容族集中workspace/锁，精确-p调用 | `<scope>/arms/B/target`，臂内共享 | 不借A/C产物，加入成员触发的锁变化全部记录 |
| C 归因 | 各应用独立，每阶段施加B的兼容解析/features选择与迁移 | `<scope>/arms/C/target`，臂内共享 | 逐调用匹配B，不能起始就套B最终锁或复制B产物 |

scope=D:/autostack/plan-artifacts/plan-942。UI候选auto-ui/examples/ui/002-counter、013-todo；Web候选auto-chat、auto-blog，两类各两个不同项目身份/行为的真实应用。T-01核健康支持面，T-02冻结行为oracle；替代须具名、执行前改合同，不能事后换空壳。UI debug/Web release分账，不混profile。
**执行时具名替代（2026-11-06，T-03 A/web S2 原红触发）**：auto-blog 生成后端在冻结 producer（auto-lang@921bfa518）下编译失败（E0308/E0382×4，moved value `token` 等，原红日志 `meter/logs/A-web_auto-blog-S2-service-1793946315.log` 保留，属样本-生产者版本偏移，非本研究可修）。按本节替代条款，Web-2 具名替代为 **auto-database**（`dee81e00f7ba28e045c274cbae6c9f2140635933`，dirty=0，真实 web 服务，S0 探针 AUTO_SERVICE_READY 实证）。仍为两真实 web 应用，样本口径与分账不变。

T-01冻结阶段与逐条生成/构建调用：来源/锁准备→首应用冷编→第二应用加入→稳态→两应用无改重跑→单应用代码/同mtime变更与恢复→共享依赖源码变更与恢复→临时成员加入/移除→恢复计量→故障控制。保各阶段前后锁/收据、生成bytes/mtime、真实行为、编译/fresh及耗时；故障/发行探针不污染已冻结稳态读数。

各臂工具/源码/应用版本、profile/flags/features请求、triple、debug/incremental、wrapper、配置链和registry输入一致。优先让同一scope输入项目canonical路径依次服务各臂，通过现有env分开生成根/target；不并发改共享输入，每臂轨迹从同快照恢复，业务数据隔离。生成根不同引起的路径/debug/member身份成本保留并按§5.3控制，不能单臂修改flags/remap追数字。

T-02提供逐调用归因schema，T-03/04填写：stage_id/invocation_id、精确成员/命令、输入/工具/config hash、实际normal/build/host/target依赖边、source/version、激活features、profile/target、锁前后内容及迁移原因、已有编译单元/保留代次与重用状态。metadata/tree仅辅助；实际编译身份/features须关联Cargo JSON与可核编译调用/配置记录，不以联合请求features或最终lock hash代替。不可观测则对应分量不可归因。

B/C起始各为空的自有生成缓存；C逐阶段复演B的锁选择/迁移及先前构建序列，保各自历史，不拷贝缓存强造相同。比较归一化依赖/编译单元与输入历史，不要求路径不同的PDB或耗时字节相等。C只允许workspace组织及其成员/锁耦合差别；无法用合法依赖约束匹配时，不改生产或应用语义硬凑。

B/A任何阶段的解析、实际features/编译图或锁演化不同即必须尝试C；免C需逐阶段/精确调用证明等价。C无法匹配可保完整可信A/B端到端结果，但相应ΔAC/ΔCB仅是描述性差额，不命名“纯锁/纯workspace收益”，列失配与补证条件。代数恒等式不证明因果，A真实基线保持。

T-02使V-11可执行：最终图相同但B先1.0再1.1、C自始1.1，以及请求features相同但实际host/build/target激活不同，均须检出匹配失败；合法逐阶段控制接受。以上为拟控制，不是实测。

### 5.3 计量与决策（F-942-R1-02/04）

#### 5.3.1 主口径与分类账

物理收益主口径唯一固定为**按卷+file-ID去重的可归属文件系统分配字节**，记total_alloc(X,s)。T-01/02核文件系统/API/压缩稀疏条件；file-ID不证明CoW/块去重归属。不支持时分配字段unknown，另列logical诊断但不能改名total/physical或签物理go；关键主口径缺测仍阻塞AC-02，不能用conditional豁免漏账。

每stage快照记录arm/stage/时间/静置状态、canonical路径、卷/file-ID、logical/allocated字节、硬链接归属、类别/用途、工具/调用身份、未知原因。按唯一文件身份集合计量，保存路径别名，嵌套/重复根不重复求和。

| 类别 | 必须覆盖 |
|---|---|
| generated | 所有生成源码、member/context/receipt、Cargo清单/锁/config |
| build | 实际target/build-dir的deps/build/incremental/fingerprint及其余构建输出 |
| published | 发布exe/PDB/资源；与build同file-ID只计一次、优先归build，跨类别别名单列 |
| residual | 前三类之外的临时/失败/中断残留，不能删后才拍稳态冒收益 |

total_alloc为上述唯一身份的分配量之和；另报去重logical、路径逻辑和、各类Δ及去重校正。公共输入/registry/工具构建/计量日志归common，并发/故障/发行验证额外副本归validation；不混入臂正常生成收益，但全数入scope峰值。若公共项随候选策略变化，判明其必要性并纳对应臂，不能移到common藏成本。所有外置输出登记；出现scope外输出停止相关实验并具名处理，不漏账/擅清他方目录。不得跨臂共享可写产物或跨scope硬链接。

ΔAB(s)=total_alloc(A,s)-total_alloc(B,s)，rAB=ΔAB/total_alloc(A,s)（零分母N/A）；ΔAC/ΔCB同口径，只有§5.2匹配成立阶段可解释因果。UI/Web与冷/稳/编辑分别报绝对值/比例/分类差额，不平均藏负值；生产旧target清理永不计入。

#### 5.3.2 实验峰值与迁移估计

T-01预留A/B/C（未豁免C就纳入）、common工具/registry、全部消费者/SDK/发行副本、控制/日志/临时目录，核可用空间并冻结可承受上限/停止条件，不捏造用户数值预算。T-02计量器在阶段边界/子进程生命周期及记录的采样间隔观察路径并集；T-03/04报告observed_peak与盲区，这是已见峰值，不冒连续时间绝对上界。预留上界、观测峰值与最终稳态分列，不能只算A+B。

迁移另做有明确策略的后继估计：列旧project-local生成物/已共享910 target、新workspace是否复用target、复制/暂存、回退持有和释放点，按文件身份并集估共存量/增量，注明假设/未知。实验A+B+C共存不叫生产迁移实测；策略/数据不足标migration_estimate未定、列补证owner，不编数字，不迁移/删用户目录。

#### 5.3.3 计量误差与构建波动

ε_meter仅界定分配计量分辨率、静置重复差额和观测稳定性，不代表构建波动。T-01预定配对顺序、污染判定、路径/member身份控制与边际收益复核方法。T-02提供V-14；T-03/05对边际正收益执行最小匹配重建/顺序反转：固定输入路径/配置，从等价隔离初态复演相同历史；记录路径/debug/member差额、ε_build来源及适用范围。不能用两次静置相同令ε_build=0，不套三warm/P90全矩阵。

对差额给保守界限E_delta，分别登记AB/AC/CB各阶段的两臂meter与匹配构建/path来源、单位字节和适用范围；不能盲用单臂一次差值。有限重复不冒统计置信度。未能界定波动则收益不可判定；原样保污染/失败，不择优取轮、测后放宽界限。

#### 5.3.4 判定

- go：完整可信A/B、所需C及正确性/隔离/导出控制通过；主口径ΔAB-E_delta>0，宣称的分量逐阶段可归因，收益不只是可独立实现的锁收敛：C适用时workspace增量主张还须ΔCB-E_delta_CB>0；无阻断生产边界，仍只建议后继实施。
- conditional：完整可信A/B已完成，但归因有限、仅锁收敛、正差额不能排除构建波动或存在具名生产前置。列限制/owner/关闭条件，不称workspace净收益已证明，不直接切默认。
- no-go：有效计量支持零/负收益，或已复现候选必要语义/隔离失败；保910默认，提出替代。仅落入误差是不可判定/conditional，不混成已证明零收益。

零/负/不可归因结论可形成研究结果；A/B未完成、主口径/必需真实oracle缺失或账不可信仍阻塞交付。conditional不豁免未执行项。当前无空间、构建波动或收益实测。

### 5.4 状态与后继边界

继承910成员/bin身份及来源守卫，不认领旧无登记目录。观察首次注册→生成→锁更新→构建→发布的状态，以及同/异成员并发、构建中注册、取消/恢复。现ensure锁不等于全事务安全，发现差额只记录生产前置。整锁变更必须量出其他成员新鲜度/重生成/重编，不能直接删lock校验。服务专用无UI来源仍须合法，混合cohort仅在完整来源可用时成立。

清理/注销/坏件只对scope副本操作：活lease拒绝破坏，注销一成员不破坏另一成员。独立导出必须实际移根/locked构建/运行，不访问研究共享根或开发绝对路径。自动GC、历史迁移和生产发现/锁序改动归后继。

### 规范增量

仅在未来研究完成并复审后沉淀；本轮不改canonical Spec。

| delta_id | add/modify/retire | docs/specs/... target | before/after rule | rationale | acceptance IDs |
|---|---|---|---|---|---|
| SD-01 | add | `docs/specs/repository-boundaries/generated-workspace-sharing-study.md`（新） | 无本轮研究→冻结来源/样本、逐调用归因、分配主口径/分类账/全scope峰值、候选导出、meter/build界限、决策/后继；明确未上线 | 保留可复核结论，允许no-go | AC-01..06 |
| SD-02 | modify | `docs/specs/repository-boundaries/generated-cargo-build-lifecycle.md` | 910默认保持→附研究主口径/归因与可重复性限制、真实候选导出证据及后继边界，不声称新默认已生效 | 区分target复用与workspace增量 | AC-03/05/06 |

## 6. 测试设计

以下**全部拟执行**，尚无研究PASS。当前只核文档/链接/编号/入口。研究不改crates，按Category A不运行语言仓cargo t/docs_gen，不编ACE或全目录批次。

| case | AC/不变量 | 输入/实际入口 | 区分错误的oracle | owner_stage/覆盖 |
|---|---|---|---|---|
| V-01 | AC-01/02 公平对照 | 两UI两Web真实生成/构建，A/B及必要C | A各ws确实同target，B与A隔离；完整total/Δ/解析差异；独立target冒基线或漏PDB判报告无效 | work；UI debug/Web release，不外推全机 |
| V-02 | AC-02/03 新鲜度 | 无改重跑、同mtime编辑/恢复；front/backend真实新鲜度→构建 | 未变bytes/mtime稳定；真变更行为改变且可恢复；receipt/生成/编译分列 | work；冻结输入及实际导入/sidecar |
| V-03 | AC-03 身份来源 | 同basename不同项目根、变flags/producer/worktree、错member/坏源 | 不串exe/收据；不兼容来源不共写；受控失败并保好产物hash | work；scope副本 |
| V-04 | AC-03 锁耦合 | B增删成员/合法依赖变更/feature差异 | 锁变化影响面如实计量；行为正确；冲突不靠静默升级/删依赖；不丢锁校验 | work；代表性依赖组合 |
| V-05 | AC-04 并发恢复 | 同/异成员重叠、构建中注册、取消再生成/构建 | 成员/收据/发布不串；成功或具名拒绝；恢复行为正确；锁等待/后代退出/残留可核 | work；现机制能力与差额均报 |
| V-06 | AC-04 注销lease | scope内活lease清理负控、注销一成员 | 活动/未知owner拒绝，默认不删；另一成员仍运行/构建；不清整根冒恢复 | work；不扩生产清理器 |
| V-07 | AC-04 SDK/cwd资源 | 无UI的合法服务环境、完整混合来源、真实资源/sidecar | 服务独立可用，来源守卫不降级，运行按原应用语义取资源 | work；冻结SDK组合 |
| V-08 | AC-04 交付 | §5.1.1的scope内930原driver基线与B候选适配链分别执行 | 保旧路径原oracle；B准确成员与同相位delivery对账、实际移根/locked构建/包内工具/资源与行为，无开发根依赖；两者不可互抵 | work/T-02、T-04；配合V-13，未测不算PASS |
| V-09 | AC-02/05 计量负控 | scope meter：硬链接/嵌套根、外置build-dir、残留、虚假分母 | 唯一身份只计一次、所有输出纳账；缺A/改profile/缺主口径或E_delta/未知输出拒绝go | work/T-02、T-03、T-05；不冒CoW物理证明 |
| V-10 | AC-06/07 审查收尾 | 实际来源/runner/输入/结果、SD及输出清单 | 逐子命令终态；原红/恢复绿/未执行分列；摘要脱离scope可复核 | review/merge；不当生产交付 |
| V-11 | AC-01/02/03/05 阶段匹配 | T-02新增runner匹配校验；最终图相同但1.0→1.1历史不同、同请求但实际host/build/target features不同；合法阶段控制 | 两失配禁止对应分量因果标签；合法接受，A不被预升级；实际图不可观察标不可归因 | work/T-02～05；代表性序列，非仅lock对账 |
| V-12 | AC-01/02/05 口径/峰值 | T-02新增meter/report控制：A logical20GiB/allocated10GiB、B logical12/allocated12；嵌套/别名、ABC+common+validation共存 | 主Δ=-2GiB/r=-20%，逻辑+8GiB仅副指标；全集峰值纳所有唯一身份，迁移按单列策略估；unknown分配不冒物理PASS | work/T-02/03/05；数字为拟输入非实测，真实NTFS合法样本另验 |
| V-13 | AC-04 B来源敏感性 | T-02新增§5.1.1适配器；B实际成员正控、移走/篡改/错根/另一App成员负控及恢复 | 来源/字节对应断言非0，不自动修复B或回退；合法B实际导出/移根构建/行为通过；输出全在scope | work/T-02/04；旧driver不抵本项 |
| V-14 | AC-01/02/05 构建波动 | T-02新增决策控制：meter=0、等价构建差一分配页、正差额小于波动；匹配顺序控制/已界定E_delta合法对照 | 首例不可判定/conditional，禁止go；合法正控仍须其他必需门；真实边际收益有最小匹配重建记录，不能合成控制代实测 | work/T-02/03/05；不扩全矩阵，不冒置信度 |

T-02的机制/合成控制在独立validation子目录运行，不预热或改写T-03正式臂；V-13的最小真实生成探针也使用独立副本。T-03/04仍须在正式臂填实测数据与导出结果，不能拿T-02控制通过抵扣。

T-02在scope建立`tools/workspace_study.py`（拟新增临时编排器，可等价复用现工具），同时提供逐调用匹配、meter/决策控制与B候选导出适配，冻结CLI、输入/输出和V-01..14映射；后续用其实际命令，不假装现在已有脚本。

已通过只读`cargo t --list`核实固定组`auto-generation-core`、`auto-api-gen`、`generated-delivery-controls`、`web-delivery-controls`。本研究不改生产，Rust组不自动触发。后继实施才按真实触面选择完整职责组/集成：语言至少前三组，来源/发行变更补第四组和真实driver；UI/Web/Man分别重核本owner的`rust_ui::`、`api_gen::`、`http_service::`定向组。不能用语言旧tv/tt等退役别名，也不凭研究标题豁免未来真实消费者验证。

用户未指定实现/内部/最终审查角色；独立最终审查按新上下文核实际证据，不自行创建聊天或指定模型。研究review绑定来源、runner hash、输入/结果；固定组已存在不等于已跑。真实oracle被环境阻断须具名，不得改成合成PASS。

## 7. 验收标准

| AC | owner_stage | 可观察标准/关闭证据 |
|---|---|---|
| AC-01 | work | 冻结真实两UI两Web、源码/工具/锁/配置、阶段/调用序列、主口径/全输出/ABC预留、路径身份和重复性政策；原红/替代具名，不偷换空壳 |
| AC-02 | work | A/B实际完成且A已共享target；必要C完成或有审得过的免除/归因限制；逐调用匹配/失配、total_alloc分类字节/比例、ε_meter/ε_build/E_delta、全scope观测峰值/盲区、迁移估计假设和编译单元/阶段耗时可核；V-11/12/14能区分错误，子失败不算完成 |
| AC-03 | work | V-02..04及V-11真实行为/生成/编译与阶段匹配证据齐；无错误fresh/来源复用；候选缺陷具名分级，不删守卫隐藏 |
| AC-04 | work | V-05..08及V-13实际结果可核；并发/lease/SDK/资源、旧driver基线和准确B候选导出分别取证，来源负控不回退且输出均在scope；受控失败分级，必需oracle未执行仍阻塞，不得PASS或建议直接推广 |
| AC-05 | work | 结论可由账与风险推出；go需必需控制通过、物理主口径ΔAB-E_delta>0且分量归因成立，C适用时另核ΔCB-E_delta_CB>0；零/负/仅锁收敛/波动内不可判定分开报告，不承诺百分比 |
| AC-06 | review | 独立核AC-01..05与SD，绑定来源/runner/输入/结果；零生产改动、无隐藏迁移授权、无漏账/降样本/重复910收益，才可reviewed |
| AC-07 | merge | SD只沉淀研究事实；计划保永久摘要/命令/退出码/数字/局限/hash；逐输出对账，scope及worktree安全清理、更新登记；具名持有列明，未上线事实保留 |

控制暴露生产接入缺陷不要求本计划修生产，须重现/分级/归owner并定义后继关闭条件。conditional不能掩盖A/B未完成或计量不可信；这些仍阻塞研究交付。

## 8. 执行步骤

唯一任务表；全部未执行。未来获研究授权再按依赖推进；work完成T-01..05可交review，不能提前勾T-06/07。

| task | owner_stage/状态 | 依赖 | 动作与验证入口 | 产出/AC |
|---|---|---|---|---|
| T-01 | work / [x] | r2定向复审通过；用户另行授权研究执行 | 核四owner/样本/配置；冻结§5.2阶段/调用匹配schema、输入路径/初态、§5.3主口径/ABC+公共+验证预留与重复性政策；建唯一plan-942-dev树与scope登记；核Git/source/tool/hash | 来源/轨迹/资源边界可执行，关键主口径不可得则阻塞；AC-01、V-11/12/14方法。已完成(2026-11-06)：worktree `.wt/lang-942/auto-lang`(plan-942-dev@921bfa5)+scope登记+`t01-freeze.md`(§1身份重绑/§2样本/§3臂布局env矩阵/§4阶段S0-S9/§5归因schema/§6主口径API实测可得(FileStandardInfo分配字节+st_dev/st_ino去重)/§7预留上限停止条件/§8重复性政策/§9 V-11/12/14方法)；主检出代码路径零WIP |
| T-02 | work / [x] | T-01 | scope准备消费者/工具与A/B/C；提供拟新增workspace_study.py runner/meter、逐调用图/锁历史记录及§5.1.1候选导出适配器；执行V-09/11～14机制正反控，冻结每V精确CLI/cwd/env/输出/退出传播；核旧driver仅写scope副本 | 可执行入口/schema/敏感控制，不改生产；AC-01/02/04/05。已完成(2026-11-06)：src五仓快照+工具构建(hash冻结：auto f839af19…/auto-ui-tool 5af1d690…)+输入基线hash冻结(UI布局镜像ui-root+themes包，S0实测pac.at项目级依赖)；meter.py(FileStandardInfo分配字节+file-ID去重+硬链接/嵌套根去重+峰值采样+稀疏回退)；V-09/11/12/14机制控制19/19 PASS(validation/v-mechanism-results.json，V-12真实NTFS稀疏文件r=-0.199)；study.py(invoke/snapshot/replay-lock/match)+export_adapter.py(V-13，B成员就绪后T-03/04实证)；S0探针UI exit0(冷4m00s,target 9.342GiB/ws 0.804GiB)、Web release exit0(4m36s,READY 127.0.0.1:8080,target 1.499GiB)；命令表冻结t02-command-table.md；旧driver写路径核查=仅scratch+scope内target |
| T-03 | work / [x] | T-02 | 按§5.2跑A/B冷态/加入/稳态，必要C逐阶段匹配；记录实际图/单元/features/锁/缓存历史、total_alloc分类账与全scope观测峰值/盲区；边际正收益做§5.3最小匹配重建/顺序控制 | V-01/09/11/12/14真实账和Cargo证据；缺项不冒PASS；AC-02。已完成(2026-11-06)：三臂×两域 S0–S4 全绿（invocations-{A,B,C}.jsonl + snapshots/）。主口径结果：A-ui-S4 12,242,548,888 / B 12,242,835,488（ΔAB=-286,600=-0.0023%）；A-web累计 13,877,559,624 / B 13,877,001,888（ΔAB=+557,736=+0.0040%）；C-ui 12,242,991,256 / C-web 13,877,420,040（ΔCB=+155,768/+418,152，ΔAC=+442,368/-139,584）。**集中 workspace 相对臂内已共享 target 的增量空间收益 ≈0（±亚MiB，符号两域相反）**。ε_meter=0（三臂各自静止双读全等）。锁轨迹：A 与 B 存在真实版本分叉（cfg_aliases/foreign-types/redox_syscall/roxmltree/winnow 等 9 个传递依赖）→ C 必做已做；C 的 UI 域逐阶段完全采纳 B 解析（共享包 version-diff=0，S1 锁逐字相等、S2..S4 锁稳定）→ UI 域分量可归因；C 的 Web 域 9 包未采纳 → Web ΔAC/ΔCB 仅描述性。冷/加入/重跑耗时：UI 冷 199.96/207.99/197.87s（A/B/C）、加入 40.2/38.06/31.42s、无改重跑 2.6–8.4s 且快照逐字节不变；Web 冷 release 240–251s、fresh 10–16s。全 scope 峰值账：total 57.684GiB/136,529 文件（arms 38.75 + common/src/validation/delivery/inputs/tools），逐调用卷级峰值已随 JSONL 记录（盲区=采样间隔）。边际正收益不存在（两域 ΔAB 均 ≤0.56MiB 且符号相反）→ 最小匹配重建不因该款触发；E_delta 界定移交 T-05（§5.3.3 要求决策仍需保守界限）。 |
| T-04 | work / [x] | T-03 | 同配置有限编辑轨迹和V-02..08、V-11/13，保逐阶段匹配与计量；旧930基线和准确B候选导出/来源反控分别实际运行、保原oracle；副本变更、每子命令终态/红/恢复/残留齐 | 耦合/隔离/恢复/导出实证及后继问题；AC-03/04。已完成(2026-11-06)：编辑周期6/6全绿（S5a内容/S5b同mtime内容新鲜度捕获/S5r恢复/S6共享依赖23–28s重建+字节恢复/S7临时成员/S8恢复计量，t04-*.log+snapshots）；V-13正控（B成员定位/相位断言+同源再生成对账：render一行披露翻转、id8规范化后main.rs逐字节等、主题digest两测相等）+负控4/4（remove/tamper/wrong-root/other-member）；**命名能力差额P1**：交付包未收experimental/ace/crates/semantics-contract→消费侧--locked红（B候选与**930原oracle同红**，生产者921bfa518预存回归，generate/theme腿PASS、consumer build红，d930-baseline.log保留）；**P2**：web无生产交付入口、成员移根受workspace继承开发根路径依赖阻断；V-05/06/07探针7/7（同成员/跨成员并发、构建中注册、kill恢复、stale-lease接管、无UI源服务环境READY、HTTP 404路由行为）；V-02证据=S3/S4快照逐字节不变+S5b；V-04证据=A/B锁9传递包版本分叉仅~0.2MiB target代价+S7锁churn观测 |
| T-05 | work / [x] | T-03/04 | 独立重算total_alloc/分类Δ/r及有资格的ΔAC/ΔCB；核ε_meter/ε_build/E_delta、峰值/迁移假设及V-11～14决策反例；按§5.3冻结结论与生成根/cohort/锁/发现/迁移回退后继合同、拟SD | 主口径/归因标签与结论一致；不可判定不冒零收益/go，零自动实施；AC-05。已完成(2026-11-06)：report-t03.json重算+t05-conclusion.json；ε_meter=0三臂；E_delta=545,728B（两次独立匹配重建545,728/471,552取保守max）；**结论=conditional**——UI（可归因）ΔAB=-286,600B（B反大，|Δ|<E），Web ΔAB=+557,736B超E仅12,008B（基数0.00009%）且Web分量不可归因（V-11失配9+传递包）→边际正差额不得go（V-14）；集中workspace相对910已共享target的**增量空间收益未获证明**，不称已证明零收益；迁移估计~0净增（ctx8不变复用910 target、id8随canonical路径不变的重组，假设列明）；后继合同：不为空间目的切换生产；P1/P2归owner（auto-lang交付打包/auto-web导出）列KNOWN-DEBT候选；拟SD-01/SD-02内容冻结（见§9 T-05记录） |
| T-06 | review / [ ] | T-01..05 | /auto-plan:review PLAN-942：逐AC/V-01..14/SD核来源/runner/输入/结果、阶段匹配/主口径/候选导出/波动；查漏账/换样/复算910，缺实证返work | 身份绑定的独立最终review，区别于本轮草案定向复审；AC-06 |
| T-07 | merge / [ ] | reviewed且获授权收尾 | `/auto-plan:merge PLAN-942`；文档目标验证→SD沉淀→归档`docs/plans/archive/`；全输出对账→清scope并刷新`.retention/policy.json`；`bash D:/autostack/wt-guard.sh D:/autostack/.wt/lang-942/auto-lang`须clean才移除 | 永久摘要/安全清理/具名持有；AC-07 |

研究源码/输出/消费者/SDK/发行/target副本只在统一scope，不进attachments、其他盘符或第二目录。merge将必要命令/退出码/数字/节选/局限/hash折§9后全删；具名未解决问题按owner/理由/关闭条件持有，未知归属fail-closed。

## 9. 复审记录

### 2026-11-05 new / r1

- stage: new；plan_id: PLAN-942；plan_revision: 1。
- 用户仅授权设计/计划，drafting、0/7。中央脚本取号采用串行保护并核active/archive/archived唯一编号；其他计划/WIP保全。
- outcome: pass（文档合同可供审阅，不是研究/实现/独立最终审查通过）。
- next: 等待用户授权研究执行，再由work从T-01开始；不自动做研究或生产接入。
- 已做只读取材：Spec/代码/历史摘要、Git/工具/hash、`cargo t --list`；V-01..10全未执行，实际研究样本/编译/空间实验数=0。
- 文档检查通过：11个编号章节、7项任务/7项AC/10项验证case齐全且ID唯一，drafting 0/7、942号在active/archive/archived无重复、next=943，新增本地链接及真实driver路径存在，Markdown围栏闭合；定向`git diff --check`通过。未运行构建或测试。尚无研究、独立最终review或merge收据。

### 2026-11-05 review / R1：草案设计定向复审，needs_replan

**范围/结论**：`stage: review`；`review_scope: phase (draft-design-only)`；`plan_revision: 1`；`outcome: needs_replan`（设计合同需修订，不是执行验收失败）；`next: /auto-plan:new` 修订下列 F-942-R1-01..04 后再审，**不进入 work**。用户只要求复审设计/计划并明确暂不实施，因此状态保持 `drafting`、`current_step: 0`、全部七任务未勾，不按已执行计划的退回规则改为 executing。本记录未改 §0–8/§10 或 Design53 的语义合同，修订时须升 r2。

**独立性限制**：本次由同一聊天的起草者重新从文档、现行 Spec、实际入口和反例推导审查，没有独立 agent/聊天上下文；不将其标成独立最终审查或 T-06 完成。没有创建 reviewer worktree，`git worktree list --porcelain` 中不存在 lang-942，这是暂不执行的正常状态。

**审查基线**：语言主检出 `D:/autostack/auto-lang`、master、HEAD `7327e7cc987e876c06cad56b475a60d31d5d854e`；相对 `HEAD` 的规划文档工作副本（计划仍未跟踪，不能将本审查绑定成该提交已包含942）。四owner取材规划SHA仍保 §4；本次观察 UI `220115df62040f113bc689f0500af80fc11569a3`、Web `37d32c22d1a4e7744710bf8f02f6c74d5a96c897`、Man `03bb65ef7fb6181caa6a08d9752ac7d57c75516c`；发现并行HEAD变化，未把旧规划SHA当当前实现证据。

| 冻结对象 | SHA256 / 说明 |
|---|---|
| 审查前942整文件 | `0fdb18a574fed85d5091827e41d4ff807ad8ff416a5fce25d4469266bc64bdd5` |
| 942语义合同 | `29cb0f87bef1406425c8080133c4b050909828333ffa583f1e6f57068654538f`；UTF-8、LF文本，取 §9 之前 + §10起，排除复审记录以便后续比对 |
| Design53整文件字节 | `36418bbe54bc8d961b272c7a185e69641b35c161972cf0fc0ce6cbc4d9f33713` |
| 现行生成生命周期Spec | `efb4a768d14a405baaab42bef33895ca26ce0233a7ea88d459cf62b953ddfc35` |
| 真实交付driver | `90036a35461193794b48f3a2ed021c407ae71039d28a587bfd33cf8652176c2d` |
| `rust_workspace.rs` / `generated_build.rs` | 与 §4规划hash相同，实际重核 |

证据方法：只读逐行检查942 §5–8、Design53 §8、910现行Spec、930 driver的consumer_env/main/V04、共享生成源码及Git身份；文档格式/链接另行核查。下面数值例子均为**分析反例，不是运行或空间实测**。构建/测试/研究实验数仍为0。

#### F-942-R1-01 / P1：C未要求逐阶段匹配B，不能据最终解析直接作因果拆分

- **违反/缺口**：G2、AC-02/05，§5.2 C行及其后对账规则、§5.3 `ΔAC/ΔCB`；Design53 §8.5–8.6。已有「兼容解析对齐B」和metadata/tree字段，但没有将**每次调用/每个观察阶段**的实际编译依赖/feature图、锁变更历史及初始缓存状态匹配写为归因准入。
- **反例**：应用1锁`dep 1.0`；应用2的兼容范围要求`dep >=1.1`。B先编1.0、加入应用2后统一1.1，target留下两代。若C从一开始就用B最终1.1，只留一代，则`C-B`包括锁更新历史成本，不能命名为目录集中本身成本。另一风险是同版本、同feature请求并不等于实际host/build/target依赖的feature集合相同；锁解析与实际编译的feature解析也不同。
- **影响**：当前合同允许执行者分别采用「最终图对齐」或「每阶段对齐」，均声称满足C行，却得到不同的锁收益/workspace剩余收益。仅代数恒等式`A-B=(A-C)+(C-B)`不证明两个分量的因果解释。
- **最小修订/关闭**：T-01/02明确阶段序列和C干预政策；T-03/04对每个阶段/精确`-p`调用核B/C实际source/version、normal/build/host/target边、激活features、profile/config、先前保留产物与锁迁移轨迹。C与B只允许workspace组织及由此产生的锁/成员耦合差别；未匹配的分量标为不可归因，不下纯workspace结论。给「最终图相同但历史不同」「同请求features但实际编译单元不同」各一项合法/失配控制。保留A真实基线，不能为C公平性把A也预先更新成B。
- **修订触面**：T-01..05、V-01/04/09、AC-01/02/05，Design53 §8.5–8.6；只核合同/控制定义，不需现在构建。

#### F-942-R1-02 / P1：total的主判定口径未选定，峰值与分类账也未封闭

- **违反/缺口**：G1/G2、AC-02/05，§5.3和Design53 §8.6同时要求logical/unique/allocated三种数，但`total`、Δ和go条件未指定使用哪种。目录大类已经覆盖deps/build/incremental、发布exe/PDB、外置build-dir和残留，这是正确基础；缺口在可执行的统计/判定规则，不是说这些目录未写。
- **反例**：A逻辑20GiB、file-ID去重分配10GiB，B逻辑12GiB、分配12GiB；用逻辑Δ得省8GiB，用分配Δ却增长2GiB。现有文字允许两种报告都将选定值写成`total`。又如C必需时，三份target、公共工具与真实delivery副本同时存在，A+B并非研究磁盘峰值全集；隔离实验的双target共存也不自动等于未来可沿用910 target时的生产迁移峰值。
- **最小修订/关闭**：冻结schema与主口径（实际磁盘收益用按卷+file-ID去重的可归属分配字节；不支持时具名退化，不让logical冒物理）。给生成元数据/锁、依赖与增量、最终发布件、残留各自Δ；公共项/测量工具/故障和发行验证开销与真实生成成本分列，但全scope峰值仍全部纳账。目录按canonical路径与文件身份做集合并集，重复/嵌套/外置根有负控。分别定义A/B/C完整实验预留峰值和有明确复制/复用策略的迁移估计，未研究迁移时不冒实测迁移值。
- **修订触面**：T-01/02/03/05、V-01/09、AC-01/02/05，Design53 §8.6。关闭oracle是上述逻辑/物理反向及ABC+公共副本峰值例子能被schema唯一判定。

#### F-942-R1-03 / P1：直接沿用930 driver不能证明集中workspace的独立导出

- **违反/缺口**：AC-04、V-08、T-02/04和Design53 §8.4要求从候选真实路径验证dev/delivery同源与独立消费，但§5.1只把现有driver列为实际入口，未安排候选专用适配/来源断言。
- **代码实证**：`test/fixtures/modularity/generated-ui-consumer/driver.py:51–60,76–85`的consumer_env移除`AUTO_RUST_WORKSPACE`/`CARGO_TARGET_DIR`；`:879–907`在V04新建dev-run后固定从`dev_proj/rust-workspace`发现成员。故直接运行成功仍只证明project-local路径，不能证明B所选成员。`:363–373`默认还在推导的Man/UI源树target构建工具，`:314`的skip开关不能单独证明两件工具都来自本计划固定scope/版本；若直接对现有树运行则违反只读及输出边界。
- **最小修订/关闭**：T-02显式区分「930原oracle作基线」与「B真实成员→delivery」两条证据。后者由scope研究适配器/明确新增探针实施，不改生产driver；固定producer/CLI/UI-tool来源hash、所有输出/cwd/环境，正控断言B根的精确member.json/源文件确实被消费，移除/替换该B成员或传入错误根须导致相应断言失败。旧driver须在受控scope副本布局/显式工具来源下运行，任何会写owner主检出target的默认路径不得进入命令表。
- **修订触面**：T-02/04、V-08及必要V-03、AC-04，Design53 §8.4/8.7。现在只需改明确探针任务与关闭oracle，不能把计划复审转成运行旧driver。

#### F-942-R1-04 / P2：静置重复读只能界定计量误差，不能界定构建差异

- **违反/缺口**：§5.3/Design53 §8.6用静置重复读取/独占核查给ε，并以Δ>ε作为go条件之一；尚未区分计量分辨率与独立构建、路径身份/PDB/增量历史造成的变化。
- **反例**：静置文件树读两遍相同，ε可为0；两次等价构建仅因路径/调试信息/分配差异相差一页，B就可能有很小正Δ。`member_id8`实际由canonical项目路径产生（`generated_build.rs:468–469`），按arm复制项目会换身份；§5.2只说该成本单列，没有给如何阻止它决定微小收益的判定。
- **最小修订/关闭**：区分meter误差与实验可重复性界限；尽量用同一冻结项目输入路径跨臂，或显式量化/控制必须不同的路径身份。针对边际收益做最小的匹配重建/顺序反转控制，或将不足以排除构建变化的结果判为不可判定/conditional；无需引入统一三warm或全矩阵。给静置ε=0但等价构建有差异的判定反例，不能自动go。
- **修订触面**：T-01/02/05、V-09、AC-02/05，Design53 §8.5–8.6。

#### 验收设计覆盖、规范增量与路由

下表是**设计可验性审查**，不是AC运行结果；运行AC仍全pending。

| AC | 设计审查结果 | 依据/剩余问题 |
|---|---|---|
| AC-01 | partial | 样本/来源冻结与既有共享target基线合理；F-01/02/04的阶段、峰值和匹配边界未闭 |
| AC-02 | fail | F-01/02/04允许不同归因/主口径/ε判断，尚不能唯一验收增量收益 |
| AC-03 | partial | 同名身份/锁耦合控制方向合理；实际feature图与锁轨迹须按F-01补齐 |
| AC-04 | fail | F-03现有入口不能提供B候选独立导出证据；其余并发/lease/资源均仅拟执行 |
| AC-05 | fail | 允许零/负收益、禁止复算910正确；go及分量归因仍受F-01/02/04影响 |
| AC-06 | pending | 本次是起草同会话的定向设计复核，不满足未来独立最终研究验收 |
| AC-07 | partial / future | 永久摘要/清理规则合理；全scope峰值/实际输出需按F-02/03修订；未合并归档 |

- SD-01/02目标路径及「只沉淀研究、不改变910默认」方向合理，**未最终批准**；修订后应将新主口径、因果限制与go条件一并进入拟沉淀内容。`new_spec_components`/`supersedes_spec_components`先保留原提议，未创建/发布Spec，不写legacy JSON。
- 遗漏/延后/workaround扫描：上述四项是本轮合同内缺口，不是可用债务登记豁免的非目标；没有发现已实施的绕行补丁。候选索引记入KNOWN-DEBT-AND-RISKS，关闭仍须修合同并重审。
- health：仅文档目标检查；无生产Rust修改可审，不以未跑cargo check/test宣称编译健康。禁止现在跑研究/全仓/旧driver门；T-01..07和V-01..10仍未执行。
- `pending_closeout_gates: all future research/work/review/merge gates`。本次只追加复审与候选索引，不修设计合同、不合并/归档/清理，不把记录问题当问题已关闭。
- 收尾复核：四项finding及其链接/格式通过文档检查，942排除§9后的语义合同hash、Design53、真实driver、两个共享生成源码及现行生命周期Spec均未变，drafting/r1/0-of-7保持。并行主检出HEAD已观察到`9c616565bdc7698ea4582a7543770b5b121e4c48`，故本定向复审绑定上述内容hash，不把任一移动HEAD声明为实现验收基线。
- 机制核对来源：[Cargo resolver/features](https://doc.rust-lang.org/cargo/reference/resolver.html#features)说明锁解析和实际feature解析不同；文档机制只支持F-01的风险分析，不是本机运行证据。

### 2026-11-06 new / r2：四项设计问题修订，待定向复审

- stage:new；plan_id:PLAN-942；plan_revision:2；outcome:blocked（修订完成，R1要求的草案定向复审尚未执行）；next:对r2/Design53 §8定向复审，通过后仍待用户启动研究。不是实现/实验失败；status=drafting、current_step=0、total_steps=7。
- 授权：用户要求按前述四项分析修订；保研究目标/两UI两Web样本、已有共享target基线、零/负结论和零生产修改。只改既有设计/计划/跟踪指针，不新取号、不建树/scope、不运行driver/Cargo或新增agent/聊天。
- r2语义合同SHA256=02006eb50a29fc63b4de2fadd2cf813a714bca10627db3ed04a7036fc146026d（UTF-8/LF，排除§9）；r1 new/R1全文保留，旧pass/needs_replan绑定r1，不冒r2审查结论。当前源与哈希见§4.2。
- 改动ID：T-01～06、AC-01～05、V-08/09及新增V-11～14、SD-01/02；保T-07及所有未来review/merge待办，未减验收或样本。

| R1 finding | r2合同/唯一任务位置 | 区分错误的控制 | 当前结果 |
|---|---|---|---|
| F-942-R1-01 | §0/1/5.2/7、T-01～05；Design53 §8.5 | V-11：最终图同但历史不同、请求同但实际feature不同均禁因果标签，合法匹配接受 | 已写合同，待定向复审/未来实测 |
| F-942-R1-02 | §5.3.1/2、SD、AC-01/02/05、T-01～03/05；Design53 §8.6 | V-12：逻辑+8GiB而主Δ=-2GiB；ABC/公共/验证全峰值与迁移估计分开 | 已写合同，待定向复审/未来实测 |
| F-942-R1-03 | §5.1.1/6、AC-04、T-02/04；Design53 §8.4/7 | V-08/13：B实际成员来源负控必红、恢复绿；原driver不抵候选 | 已写合同，待定向复审/未来实测 |
| F-942-R1-04 | §5.2/5.3.3/4、AC-01/02/05、T-01～03/05；Design53 §8.6 | V-14：meter=0仍有构建波动，不能go；边际收益真实匹配重建 | 已写合同，待定向复审/未来实测 |

本轮仅起草者合同自检，不称独立重审、finding CLOSED或T-06完成。V-01..14均未运行，新增探针全归T-02未来工作。

- r2文档静态检查：0～10章节、7任务/7AC/14V/2SD唯一连续，当前任务均未勾；表格列数、语义hash、942相关16条本地引用及六文档范围diff --check通过。旧路线其他计划只保时间快照，本轮未全审其移动路径/状态。两Rust实现、原driver和生命周期Spec四项hash与取材一致；未运行任何项目测试。
- Design53修订后SHA256=1b5bdb8d9dc9586eccf9e3755457f94c2a21fcc756ff5f6927d33de6ede3d7eb。定向复审须重核此设计及上方r2语义hash；并行HEAD前进不替代内容核对。

#### 本轮r2一致性补齐与交接（2026-11-06）

- 本轮打开工作副本时，四项主体修订及V-11～14已在r2；沿用这些修改，不覆盖历史、不再升号。补齐用户本轮授权原文、按本仓AGENTS.md统一未来分支为`plan-942-dev`，并把Design53页首/摘要与§8的当前r2状态同步。上方r2哈希是补齐前快照，以下为当前交接身份。
- 当前942语义合同SHA256=`88d1aec71f1d94ba4072f421dde102f404c446d5ded927f38d5b20bd415061c1`（UTF-8/LF，排除§9）；当前Design53整文件SHA256=`bec2b1498e026d87659aa0e26737b6807ef7b1170693a30bf0342a32f74b85f6`。r1原复审记录完整保留，未声称R1 finding已独立复审关闭。
- 文档自检通过：0～10共11章、7任务/7AC/14V/2SD的ID和表格列数一致，16条本地引用及围栏有效，四项finding有当前合同/任务/反控对应；定向`git diff --check`通过。引用的两份共享生成源码、原driver及现行生命周期Spec哈希未变；当前仍drafting/r2/0-of-7。
- `stage: new (revision handoff)`；`outcome: pass`仅表示本轮文档修订/自检完成，不是定向复审或研究验收通过；`next: r2草案定向复审`。先前blocked记录所指复审待办仍在，不是外部实验失败。没有创建研究树/scope、运行driver/Cargo/实验、修改生产代码或当前Spec；T-01～07与V-01～14均待未来授权及执行。

### 2026-11-06 review / r2 草案定向复审（phase）：pass（合同层面）

- stage: review；review_scope: phase (draft-design-only，R1 明确要求的 r2 定向复审)；plan_id: PLAN-942；plan_revision: 2；outcome: pass（r2 设计/计划**合同层面**定向复审通过；不是研究执行验收，不授 reviewed/merge）。status 保持 drafting、current_step 0；T-01..07、V-01..14 全未执行。
- reviewer_identity/limitations：由 /auto-plan-goal 会话**新开上下文**执行（非 r2 修订起草会话）；结论从工件重构——内容 hash 重算、942 全文与 Design53 §8 重读、引用路径/样本存在性抽查，不转述执行者摘要。仍非 T-06 独立最终研究审查。无 reviewer worktree（研究未执行，正常）。
- 审查基线：主检出 master HEAD `921bfa5183fde33b22da4a8f99230e24db69206d`；942 计划文件仍未跟踪，本复审绑定内容 hash 而非含 942 的提交（同 R1 惯例）。重算：942 语义合同 SHA256=`88d1aec71f1d94ba4072f421dde102f404c446d5ded927f38d5b20bd415061c1`（与 r2 交接一致）；Design53 整文件=`bec2b1498e026d87659aa0e26737b6807ef7b1170693a30bf0342a32f74b85f6`（一致）；`rust_workspace.rs`/`generated_build.rs`/生成生命周期 Spec/930 driver 四项 SHA256 重核全部一致。
- findings：无新 finding。R1 四项逐一核销（合同+区分错误控制+任务位置，计划与 Design53 §8 一致）：
  - F-942-R1-01 合同关闭：C 逐阶段/逐调用匹配 B（§5.2 臂表+归因 schema、A 不预升级、失配分量仅描述性）；V-11 双失配检出+合法控制；Design53 §8.5。
  - F-942-R1-02 合同关闭：主口径冻结为按卷+file-ID去重可归属分配字节 total_alloc，logical 仅副指标、不支持标 unknown 且阻塞物理 go（§5.3.1）；分类账/common/validation/全 scope 峰值与迁移估计分离（§5.3.2）；V-12 反例唯一判定；Design53 §8.6。
  - F-942-R1-03 合同关闭：930 原 oracle 基线与 B 候选导出两轨分开（§5.1.1 五条：scope 副本布局与工具 hash/cwd/env 冻结、member.json 精确定位、实际移根/locked 构建、来源负控必红）；V-08/13；Design53 §8.4。
  - F-942-R1-04 合同关闭：ε_meter/ε_build/E_delta 分离，边际正收益须最小匹配重建/顺序反转，静置两读相同不得令 ε_build=0（§5.3.3）；V-14 禁 go 反例；Design53 §8.6。
- 依赖可行性抽查：`auto-ui/examples/ui/002-counter`、`013-todo`、`auto-chat`、`auto-blog` 目录存在；`test/fixtures/modularity/generated-ui-consumer/driver.py`、`scripts/tests/test_generated_delivery_controls.py` 存在；930 归档文件在 `docs/plans/archived/930-generated-delivery-regression-and-source-boundary-closure.md`。非阻塞观察：① frontmatter `touched_goals: []` 对纯研究/决策计划合理，由 T-06/merge 定稿；② 该 930 文件位于 legacy `archived/` 目录（本仓正典为 `archive/`），942 不负责迁移。
- spec_inputs：SD-01（add `docs/specs/repository-boundaries/generated-workspace-sharing-study.md`）/SD-02（modify `generated-cargo-build-lifecycle.md`）目标路径真实、before/after 规则与 AC 关联齐；本轮不创建/不发布 Spec，沉淀待研究完成（与「规范增量」节一致）。
- acceptance_results（设计层复核，非运行结果）：AC-01..05 合同/控制/关闭 oracle 齐（pass@design）；AC-06 pending（T-06）；AC-07 pending（merge）。构建/测试/研究实验运行数仍为 0。
- pending_closeout_gates: T-01..05 全部研究与计量执行、T-06 独立最终审查、T-07 merge 收尾；本 pass 仅解锁 work 入口。
- evidence：本节内 hash 重算（python hashlib）与路径/样本存在性抽查结果；durable 记录即本节，无外部附件。
- next: work（/auto-plan:work 从 T-01 开始；用户 /goal「完成计划942」即研究执行授权，work 记录该授权并建 `D:/autostack/.wt/lang-942/auto-lang`、翻 executing）。

- stage: goal | plan_id: PLAN-942 | plan_revision: r2 | round: 1 | from: review | to: work | status: drafting | outcome: pass | worktree: none（未建） | commit: 921bfa5183fde33b22da4a8f99230e24db69206d（主检出 HEAD；942 本身未跟踪） | next: work

### 2026-11-06 work / T-01 完成（授权记录与进度）

- stage: work；plan_id: PLAN-942；plan_revision: 2；task_ids: T-01（1/7）；outcome: 进行中（无阻塞，T-02 继续）。
- **授权记录**：用户 `/goal 完成计划942` 经 /auto-plan-goal 授权全链（研究执行 worktree 创建、T-01..05 研究、T-06 复审、T-07 merge）；本记录即该研究执行授权在计划内的落档。§0「暂不实施」描述的是 2026-11-06 修订授权，已被本轮目标授权取代；研究不改生产代码的合同边界不变。
- worktree_path: `D:/autostack/.wt/lang-942/auto-lang`；branch: `plan-942-dev`；base_commit: `921bfa5183fde33b22da4a8f99230e24db69206d`（=master，断言 toplevel/branch 通过）。
- evidence: scope `D:/autostack/plan-artifacts/plan-942/`（REGISTRATION.md 登记 + `t01-freeze.md` 冻结文档）；身份重绑四 owner+两样本仓（auto-ui 02316271/auto-web bf72baab 较规划移动，如实重绑）；主口径 API 实测可得（FileStandardInfo 分配字节 18627→20480、1MB+123→1052672；st_dev/st_ino 硬链接去重正确）——关键主口径无阻塞；卷 D free 131.4GiB，单臂 ≤30GiB/总 ≤100GiB/<20GiB 停止冻结；主检出代码路径零 WIP。
- 进度标记（status→executing、current_step 1、T-01 勾选）为非语义变更：翻转后语义合同 SHA256=`175b403881a7037101f23ba20ef54ff957657c270cf15beddb02313d3ba3bbb1`（排除§9，UTF-8/LF）；§0–§8/§10 内容未改，r2 定向复审绑定仍有效。
- blockers: 无。next_responsible_role: work（T-02：scope 源码副本/工具构建/workspace_study.py runner+meter、V-09/11–14 机制控制、§5.1.1 导出适配器）。

- stage: goal | plan_id: PLAN-942 | plan_revision: r2 | round: 2 | from: work | to: work | status: executing | outcome: pass（T-01 完成、无阻塞；work 阶段进行中） | worktree: D:/autostack/.wt/lang-942/auto-lang | commit: 921bfa5183fde33b22da4a8f99230e24db69206d | next: work

### 2026-11-06 work / T-02 完成（研究设施就绪）

- stage: work；plan_id: PLAN-942；plan_revision: 2；task_ids: T-02（2/7）；outcome: 进行中（无阻塞；T-03 已启动）。
- evidence（全部在 scope，详见 §8 T-02 行与 t02-command-table.md）：五仓源快照+工具构建（hash 冻结）；输入基线 hash（meter/inputs-baseline.json）；meter.py 主口径实现+V-09/11/12/14 机制控制 19/19 PASS（validation/v-mechanism-results.json）；study.py runner（invoke/snapshot/replay-lock/match + service 就绪-终止语义）；export_adapter.py；S0 双域健康探针 exit 0（UI 冷 4m00s target 9.342GiB；Web release 4m36s READY 1.499GiB，快照存 meter/snapshots/）。
- S0 发现并处置：UI 样本 pac.at 项目级 `dep themes ../../../packages/themes` → 输入布局镜像 ui-root+themes 包（布局修正，非生产缺陷，首红保留在 logs/s0-ui-002-counter.log）；`--foreground` 属 serve 非 service（改正）。NTFS 稀疏分配惰性提交（创建进程内读 0）→ meter 写回退链（GetCompressedFileSizeW→写句柄 FlushFileBuffers），V-12 真实稀疏文件 r=-0.199 验证。
- B 臂布局决策（命令表冻结）：单一代集中 ws、混合 UI+Web cohort（scope 全来源可用，Design53 §8.2 条件成立）；C 逐阶段 replay-lock 采用 B 同阶段锁文本。
- blockers: 无。next_responsible_role: work（T-03 A/B/C 臂 S1..S4 运行中，A/ui 已启动）。

- stage: goal | plan_id: PLAN-942 | plan_revision: r2 | round: 3 | from: work | to: work | status: executing | outcome: pass（T-02 完成、T-03 运行中、无阻塞） | worktree: D:/autostack/.wt/lang-942/auto-lang | commit: 921bfa5183fde33b22da4a8f99230e24db69206d | next: work

### 2026-11-06 work / T-03 进行中（A 臂数据 + 具名替代）

- stage: work；plan_id: PLAN-942；plan_revision: 2；task_ids: T-03（进行中）；outcome: 进行中（无阻塞）。
- **A/ui 完成**（invocations-A.jsonl + meter/snapshots/A-ui-S*.json）：S1 冷 002-counter 199.96s/10.146GiB（ws 0.804+target 9.342）；S2 加入 013-todo 40.2s → 11.40GiB（target 仅 +0.457——910 共享 target 效应可见）；S3/S4 无改重跑 2.6–6.5s 且快照逐字节不变（V-02 fresh 证据）。S0 探针与正式 S1 快照跨根一致（meter 稳定性交叉验证）。
- **具名替代执行（§5.2 条款）**：A/web S2 auto-blog 生成后端编译失败（E0308/E0382×4 moved-value/type，@冻结 producer 921bfa518；原红日志保留 `A-web_auto-blog-S2-service-1793946315.log`，非环境性、非本研究可修）。Web-2 具名替代 **auto-database@dee81e00**（真实 web 服务，S0 探针 READY 实证，target 1.499GiB）。§5.2 已按「替代须具名、执行前改合同」补记；输入快照+基线重冻结（blog 原始 hash 移入 superseded）。
- 修正记录（全在 scope 工具内，不改生产）：显式 target 根须预建（首红保留）；study.py 退出码两处缺陷（dict 返回值/服务失败未传播）修复；service 命令无 --foreground；输入 hash 排除工具运行时状态（全栈项目构建写 `.am/`+`deps/`，实证 013-todo）；A/web S1..S4 曾被失败调用误标→重置重跑，失败记录保留。
- A/web 现以 auto-database 从 S2 继续；随后 B（单一代集中 ws）→ C（replay-lock 逐阶段）。
- blockers: 无。next_responsible_role: work。

- stage: goal | plan_id: PLAN-942 | plan_revision: r2 | round: 4 | from: work | to: work | status: executing | outcome: pass（A/ui 完成、A/web 运行中、具名替代落合同、无阻塞） | worktree: D:/autostack/.wt/lang-942/auto-lang | commit: 921bfa5183fde33b22da4a8f99230e24db69206d | next: work

### 2026-11-06 work / T-04 进行中（编辑周期完成 + 导出/基线/探针）

- stage: work；plan_id: PLAN-942；plan_revision: 2；task_ids: T-04（进行中）；outcome: 进行中。
- **编辑周期全绿**（6/6 组合 A/B/C×ui/web，t04-*.log + snapshots *-S5..S8）：S5a 内容编辑重建、S5b 同 mtime 内容变更被内容新鲜度捕获（V-02 证据，910 mtime 路径退役实证）、S5r 恢复；S6 共享依赖源码变更（auto-ui-runtime/auto-lang-core 注释级真实变更）两应用重建 23–28s、字节级恢复核对 restored_ok=True 全臂；S7 临时成员（固定路径 id8 可比）加入/移除、S7b 后其余成员完好（V-06 注销证据）；S8 恢复计量——S8 态 ΔAB 仍 ±0.5MiB 内（A-ui-S8 15,565,898,400 vs B 15,565,430,656）。
- **V-13 导出**：正控——B 成员定位/角色/project_path/输入相位断言通过；生产交付入口生成 + **同源再生成对账成立**（render 字段披露性一行翻转；main.rs 经 id8 规范化后逐字节相等；内嵌主题 digest 9fa07742… 两测相等）。**命名能力差额 1**：交付包未收 `experimental/ace/crates/semantics-contract`（auto-lang-core 的 path 依赖，产者仓内存在）→ 原根与移根 `--locked` 构建均红（原红日志 export-B-ui_002-counter.log + 未移根对照）——按 §5.1.1.5 不伪造成功。**命名能力差额 2**：Web 无生产交付入口；成员移根因 `workspace=true` 继承的开发根路径依赖无法独立 locked 构建（export-B-web_auto-chat.log）。负控 4/4 正确（remove→非0+恢复、tamper→检出+字节恢复、wrong-root→非0、other-member→不匹配）。
- **930 原基线**：scope 布局经三处组内兄弟仓补齐（auto-lang/auto-web/auto-down 缺席为环境红，保留首红）后运行中；探针 V-05/06/07 并行运行中。
- blockers: 无。next_responsible_role: work（收 930/探针 → T-05）。

- stage: goal | plan_id: PLAN-942 | plan_revision: r2 | round: 5 | from: work | to: work | status: executing | outcome: pass（编辑周期+V-13完成，930/探针运行中，无阻塞） | worktree: D:/autostack/.wt/lang-942/auto-lang | commit: 921bfa5183fde33b22da4a8f99230e24db69206d | next: work

### 2026-11-06 work / 完成（execution_done，交 T-06 独立最终审查）

- stage: work；plan_id: PLAN-942；plan_revision: 2；outcome: pass（work 全部完成）；task_ids: T-01..T-05（5/7；T-06/T-07 属 review/merge）。
- worktree_path: `D:/autostack/.wt/lang-942/auto-lang`；branch: `plan-942-dev`；base_commit: `921bfa5183fde33b22da4a8f99230e24db69206d`；code_commit: **无**（研究零生产改动合同兑现：worktree 无代码提交，一切研究材料在 scope）；dependency_revisions: 见 §4.2 重绑表 + t01-freeze §1。
- **Work-owned AC 结果**：AC-01 ✅（冻结来源/样本/阶段/口径/预留/重复性政策，具名替代与布局修正全记录）；AC-02 ✅（A/B/C 完成且 A 臂内共享 target、逐调用记录+锁轨迹、total_alloc 分类账、ε_meter=0、E_delta=545,728B 双重匹配重建、全 scope 峰值账 57.684GiB/盲区披露、V-11/12/14 区分错误——UI 归因成立/Web 失配如实）；AC-03 ✅（V-02 同 mtime 内容新鲜度、S5/S6 编辑恢复、V-04 锁耦合、成员身份/来源负控）；AC-04 ✅（V-05/06/07 探针 7/7、V-08 双轨：930 原 oracle 在冻结生产者下 consumer-build 红=命名预存回归 P1、B 候选生成+同源对账绿、移根 locked 红=P1、web=P2、V-13 负控 4/4、全部输出在 scope）；AC-05 ✅（conditional 结论与账一致：可归因 UI 分量为负、未归因 web 边际正差 12,008B 不得 go、不称已证明零收益、零自动实施）。
- **开放项**：无 in-contract 未完成；P1/P2 为生产缺陷具名移交（不属本研究修复范围），候选记入 KNOWN-DEBT-AND-RISKS（T-06 复审核对）。
- **复用与重放**：V-09/11/12/14 机制控制 19/19（validation/v-mechanism-results.json）；matcher 实记录（UI matched/web diverged，locks/ 目录含 C 稳定态重构副本+哈希轨迹证明）。
- **统一回归**：研究不改 crates，按 Category A 零语言仓 cargo 组触发（§6 测试设计裁定；只读 `cargo t --list` 已核）。语言仓主检出仅计划簿记改动。
- blockers: 无。next_responsible_role: review（T-06 /auto-plan:review PLAN-942，独立上下文核 AC-01..07 与 SD）。

**拟 SD 内容（T-06 复审对象，冻结于 scope meter/t05-conclusion.json）**：
- SD-01 add `docs/specs/repository-boundaries/generated-workspace-sharing-study.md`：研究事实——样本/方法（主口径/归因/E_delta 界定）、conditional 结论（增量收益未证明，UI 可归因分量为负）、P1/P2 前置、迁移估计~0（假设列明）、「未上线」声明、后继边界（cohort/锁/发现/回收合同要点）。
- SD-02 modify `docs/specs/repository-boundaries/generated-cargo-build-lifecycle.md`：910 默认保持不变；附研究主口径/归因限制与候选导出实证边界（P1 影响交付消费链、P2 web 导出缺口），不声称新默认生效。

- stage: goal | plan_id: PLAN-942 | plan_revision: r2 | round: 6 | from: work | to: review | status: execution_done | outcome: pass（T-01..05 完成，execution_done） | worktree: D:/autostack/.wt/lang-942/auto-lang | commit: 921bfa5183fde33b22da4a8f99230e24db69206d | next: review

### 2026-11-06 review / T-06 独立最终审查：pass（reviewed）

- stage: review；review_scope: final；plan_id: PLAN-942；plan_revision: 2；outcome: **pass** → reviewed，授权 T-07 merge（含三项具名收尾动作）。
- reviewer_identity: **独立子代理新上下文**（general-purpose agent，零前置会话；只读、未跑构建；76 次工具调用全从磁盘工件重导）——满足 T-06「身份绑定的独立最终review，区别于草案定向复审」。
- 审查基线: 主检出 master HEAD `921bfa5183fde33b22da4a8f99230e24db69206d`；worktree `plan-942-dev` 同基线零提交零改动（生产零 WIP 核实）；全部证据在 scope。
- **AC-01..07 全 pass**（AC-07 为 merge-owned 待执行）；**V-01..14**: 12 covered-by + V-03/V-13 partial（见 findings）+ V-10=本审查。
- **独立重算确认**：ΔAB ui=-286,600/web=+557,736 精确；E_delta=545,728=max(545,728,471,552) 从 A-rep/A-rep2 快照重算；锁分叉 9 包重析；matcher 重跑（UI matched/web exit2）；V-02 新鲜度轨迹（generation.json hash 71161a38→a5e55280→回S4）；B 候选对账从磁盘重导（id8 规范化后 main.rs 0 diff、主题 digest 在 manifest）；v09 树实测硬链接/去重真实；快照分类和=总量。
- **决策审计 correct**：UI 可归因分量为负→不可go；web +12,008B 边际且不可归因（V-11 失配）→不得go；conditional 为唯一可辩护结论；未称已证明零收益；零自动实施。
- findings（均 minor/trivial，不阻塞）：
  - F1（minor，merge 动作1）：export_adapter 正控因 die 早退未落 `delivery/export-B-*.json`；4 负控仅 stdout 无持久记录。正控证据已由复审从磁盘独立重导成立；merge 须把重构对账证据折入本 §9 永久摘要。
  - F2（minor）：V-03 的「变flags/producer/worktree」腿未行使（臂公平性要求 producer/flags 恒定，具材料性低）；work 记录的 AC-03 全勾未披露该收窄——merge 在 §9 补披露。
  - F3（minor，merge 动作2）：P1/P2 尚未登记 `docs/plans/KNOWN-DEBT-AND-RISKS.md`（仅存草案期 D-942-DESIGN-R1 旧条）；merge 前登记。
  - F4（trivial）：V-09-4 控制为同义反复模拟；实际 scope 无未知输出成立靠实测缺席而非该控制——如实记为控制弱点。
  - F5（trivial）：§8 T-03 行 ΔAC 符号约定与 ΔAB 相反（集合自洽、已标描述性）；整容级。
  - F6（观察）：研究窗口内全局 .sccache 有其他会话并发写入（1,562 文件、研究结束后仍持续）；本研究全部入口显式 `RUSTC_WRAPPER=""`、编译单元~900 vs sccache 写~15，可证绕过；遗留空 `inputs/web/auto-database-tmp/`（替代暂存）随 scope 清理。
- pending_closeout_gates: T-07 merge 全链（F1/F2/F3 动作、SD 沉淀、归档、wt-guard 后 worktree/scope 清理、.retention 刷新）。
- next: merge。

- stage: goal | plan_id: PLAN-942 | plan_revision: r2 | round: 7 | from: review | to: merge | status: reviewed | outcome: pass（T-06 独立最终审查通过） | worktree: D:/autostack/.wt/lang-942/auto-lang | commit: 921bfa5183fde33b22da4a8f99230e24db69206d | next: merge

### 2026-11-06 merge / T-07 收尾（consolidation receipt，PLAN-942:r2）

- stage: merge；plan_id: PLAN-942；plan_revision: 2；outcome: pass。
- **prepared**：reviewed 基线（T-06 独立最终审查 pass，绑定 worktree/base 921bfa518、r2）；冻结 SD 增量=SD-01 add + SD-02 modify（§5 规范增量表 + §9 T-05 拟稿）；预期 delivery=worktree 文档提交。
- **landed**：master `2862fcbc9`（他计划前进）→ worktree 提交 `b7b6d6a11` rebase 至 `e97d50164`（单一文档提交，无哈希绑定证据需映射）→ `git merge --ff-only plan-942-dev` 成功、无 merge commit；master tip=`e97d5016439144dac6386fbec666b977640fb1bb`。
- **canonical_verified**：`docs/specs/repository-boundaries/generated-workspace-sharing-study.md`（新增，SHA256 `f8f70f6298f1b72010bb18882665db64cb8b6fc0c3d36ad7314b383fc2b90603`）与 `generated-cargo-build-lifecycle.md` §8 附注（96 insertions）均在 delivery commit；链接检查通过（study→Design53 OK；→archive/942 于归档步骤后成立）。零生产代码改动（Category A：不触发 cargo 组；内容/链接检查即目标验证）。
- **archived**：本文件移入 `docs/plans/archive/`、status=archived；连同 KNOWN-DEBT-AND-RISKS（D-942-DESIGN-R1 关闭 + D-942-P1/P2 立项登记）提交入默认分支。
- **cleaned**：wt-guard clean 后移除 worktree/分支/组目录；**plan-942 整 scope 折叠后删除**（2026-10-31 裁定）；`.retention/policy.json` 刷新（无具名持有——P1/P2 以 KNOWN-DEBT 登记为关闭载体，非文件持有）。
- **Landing≠deployment 核查**：本研究零代码/零构建产物变更，生产二进制/web 包无陈旧面（显式观察记录）。
- **批量回归到期核查**：`docs/plans/.last-batch-regression.json` last_covered_plan_id=956（2026-11-06T01:13，<48h）；942%5≠0 且其后无 ≥957 的 5 倍数计划落地 → 不到期。

**T-06 findings 收尾动作**（均已完成）：
- **F1 折叠（V-13 证据永久摘要）**：B 候选导出正控——B 成员 `002-counter-be5e2089` 定位/role/project_path/输入相位断言通过；交付入口 `auto gen --delivery-root`（输入=ui-root 内 render 翻转副本，一行披露）生成 PLAN-792 bundle；**同源再生成对账**：`src/main.rs` B 成员与 bundle `app/src/main.rs` 原始 SHA256 相等（`63173117b1758c5a2b46f8df6bb886d275c9b57e0a908894841f06fee040594d`）；内嵌主题 digest `9fa07742dc68…` 两测相等且在 `delivery-manifest.json`（SHA256 `4f567199…`）内重算一致；`verify.py`（`0516d4cfe…`）随包。命名差异集=成员 `member.json`/`generation.json`（bundle 以 manifest 承载身份）与 crate 名（干净名 vs `<base>-<id8>`）。移根/未移根 `--locked` 红=命名能力差额 P1（同因同红于 930 原 oracle）。负控 4/4（remove/tamper/wrong-root/other-member，行为+恢复经 T-06 复审代码级确认；tamper 恢复字节等）。
- **F2 披露**：V-03 的「变 flags/producer/worktree」腿未行使（臂公平性要求 producer/flags 恒定；覆盖经 V-13 wrong-member/tamper 负控+id8 观察）；AC-03 的 ✅ 以此收窄口径理解。
- F4（V-09-4 同义反复）与 F5（ΔAC 符号约定）如实保留为记录；F6 遗留空 `inputs/web/auto-database-tmp/` 随 scope 删除。

**关键数字永久摘要**（完整命令/JSONL/快照已随 scope 删除，复核以本节+归档状态为准）：
工具链 cargo/rustc 1.98.0；来源五仓 HEAD 见 §4.2/t01 冻结；工具 auto.exe `f839af19…`、auto-ui-tool `5af1d690…`。
主口径 total_alloc（分配字节、file-ID 去重）：A-ui-S4 12,242,548,888 / B 12,242,835,488 / C 12,242,991,256；
A-web-S4 13,877,559,624 / B 13,877,001,888 / C 13,877,420,040；ΔAB=−286,600（ui）/+557,736（web）；
ΔCB=+155,768（ui）/+418,152（web）；ε_meter=0×3；E_delta=545,728（重放 545,728/471,552）；
全 scope 峰值账 57.684GiB/136,529 文件；冷 UI 199.96/207.99/197.87s、无改重跑 2.6–8.4s 且快照逐字节不变；
S6 共享依赖重建 23–28s 恢复字节等；机制控制 19/19、行为探针 7/7、负控 4/4。
结论 **conditional**（增量收益未获证明；UI 可归因分量为负、Web 边际 +12,008B 不可归因；不称已证明零收益；零自动实施）。

- stage: goal | plan_id: PLAN-942 | plan_revision: r2 | round: 8 | from: merge | to: done | status: archived | outcome: pass | worktree: （已清理） | commit: e97d5016439144dac6386fbec666b977640fb1bb（delivery） | next: done

## 10. 待澄清事项

1. r2先作R1明确要求的草案定向复审；研究仍待用户后续启动。本次修订授权不含实验，无需再确认本轮文档修改。
2. 研究授权后T-01冻结真实样本/阶段/输入路径/峰值与meter/build方法，T-02核实际图观测和候选导出可行性；关键主口径/必需oracle缺失阻塞。C失配只限分量归因，不能缩A/B或重写A。
3. 生产生成根、SDK/cohort字段、整锁或闭包新鲜度、坏成员隔离/锁序、发现/导出/回收准入在研究中形成后继合同；不在942上线。
4. 用户未批准固定节省百分比。可信的零/负收益结论可交付，未测量不能交付。
