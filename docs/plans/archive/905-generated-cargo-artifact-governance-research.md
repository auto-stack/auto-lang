---
plan_id: PLAN-905
status: archived
feature_name: 生成Rust工程与Cargo产物治理的有界实测和方案设计
author: [Codex]
created_at: 2026-11-01
updated_at: 2026-11-01
plan_revision: 1
current_step: 6
total_steps: 7
supersedes_spec_components: []
new_spec_components: []
touched_goals: []
affects: [auto-generation-core, generated-ui-delivery, auto-web-generation, auto-man-build, build-storage-research]
depends_on: [PLAN-890, PLAN-899, PLAN-901]
---

# [PLAN-905] 生成Rust工程与Cargo产物治理：有界实测与方案设计

## 0. 变更摘要

把原[Cargo研究交接](reports/cargo-artifact-sharing-research-handoff-2026-10-30.md)落实为正式研究Plan，依据[Design53](../design/53-independent-build-and-generated-cargo-governance.md)。本包完成实际来源/配置/容量基线、至少UI与Web两个真实生成消费者、有界复用/失效/并发/恢复实验、候选选择和经确认的有限治理实施合同输入；不以安装工具或只否定Kache宣称问题已解决。

当前drafting0/7，仅用户授权立项，未测量目录容量、未建树/运行构建/清理/改代码。该研究不是生产实现Plan：有新机制需求时，结论须给完整可审阅后继实施合同（不自动取号或开工）；已有机制若足够则须实证对用户确认的有限验收成立。研究交付与整个Cargo治理/M1出口分开。

## 1. 目标

- G1：核清“已共享什么、仍重复什么”，区分Auto语义缓存、生成源、Cargo target/incremental/PDB与sccache，不把其配置上限当实际占用。
- G2：同义输入验证稳定工程身份/不重写、真实失效、跨项目源码隔离和错误恢复；用完整物理/逻辑账及时间阶段数据选择方案。
- G3：明确producer/build/clean各owner、兼容环境边界、资源预算和可执行回滚，为近期有限M1给用户可确认的验收出口与实施合同。

非目标：生产代码改造/默认配置切换、合并worktree可写target、重做全部Design40 HIR/MIR/ABC缓存、复活Kache/改卷/安装全局工具、清用户或别的计划缓存、调整debug/优化/feature/样本来凑空间、全ACE/AG/所有应用或远端发布。

## 2. 架构方案

语言仓持唯一合同，因为共享生成机制在crates/auto-generation-core；Man实际Cargo编排、UI/Web生成及发行接口参与只读调查和固定版本实验。生产所有权不转移。两类真实消费者至少各一个：UI现有外部生成app与Web Rust服务，优先复用公开生成/发行夹具，不拿纯std小包替代实际工作集。

实验候选仅在自有fixture/生成副本与显式临时配置中运行，不改主仓/其他会话默认配置。比较现有共享workspace、稳定内容写入/成员身份、按兼容环境分组、隔离target配现有缓存等；不预定全局巨大workspace。相同target被多项目使用时核Cargo锁/等待/源身份，跨worktree可写状态继续隔离。

## 3. 技术栈

沿现有Cargo/rustc、Python/PowerShell、Git、auto-generation-core::rust_workspace/source_owner、UI/Web公开生成入口及Man owned进程执行。官方机制资料见Design53；具体Cargo key/JSON/target与build-dir支持由T-01按本机版本重核，不混用在线最新教程。

## 4. 需求分析与背景调查

2026-11-01用户要求把Cargo治理和独立Web检出问题分别立计划；当前只授权规划。原888/890等待已由交付解除，899/901/UI018已交付，接受现行profile/feature；原“20GB/工程”是用户历史观察，“50–60GB”是已取消Kache方案估算，均非本机测量或批准收益门。

语言HEAD b1a4871aa6a8d960130fc2e1aed8bf6dc4183c5a；UI6ab7f2def391eca584ebbbbbf375ef75a7f64ec9；Man cbbe6573e2d40e3237f52718dd02a3cd065d5cbd；Web ec9c4bac90fed62e6b651df7789153149e8a9473。开工重核，保其他docs WIP。已读docs/specs/overview.md、auto-cache/project.md、auto-test-case/build-storage-lifecycle.md和repository-boundaries/generated-ui-distribution.md；docs/specs/auto-generation-core/project.md当前不存在，不冒有现行专篇。

实物：crates/auto-generation-core/src/rust_workspace.rs有get/resolve_rust_workspace_dir、shared_cargo_target_dir、try_ensure_shared_workspace/try_ensure_workspace_for、back_member_name和workspace_lock_identity；target选择受CARGO_TARGET_DIR、发现语言根/兄弟根与fallback影响，write_shared_cargo_config有fs::write。source_owner.rs校验受管来源/内容。UI rust_ui.rs约604/640使用共享workspace、约6239起显式target；Man http_service.rs约199起按项目锚调用Web同源出口。它们存在不代表已验证共享正确、写入稳定或隔离充分。

本次只读Get-Volume：D/C均NTFS、剩余约411.1/118.5GiB，随并发变化不作实施预算；未递归统计目录、未验证CoW或收益。主机当前空闲量和原会话“C仅1.6GiB”已不同，不沿用旧数。906会改变依赖来源，014会重排生成工具：实验冻结实际SHA/内容hash，改变输入则对应重测，不把它们设为无关全局前置。

## 5. 详细设计

### 5.1 基线与测量合同

T-01冻结版本、有效Cargo配置（Process/User/Machine wrapper和相关白名单，脱敏）、profile/features/triple/flags/jobs、环境输入/构建脚本、registry/锁、源码/生成器/SDK/resource身份与所有输出路径。先浅层盘点owner，再经批准的受控遍历计唯一文件数据、分配字节可得值、逻辑大小/共享误差；不跟随reparse，不修改权限。自有实验目录可测，未知/活动他方目录不扫描清理。

按源工程→生成源码/manifest/lock→依赖产物→本地应用→incremental/PDB/其他→旧缓存分账。总占用包括store与全部实验工程及临时/迁移峰值；目录求和不能证明物理共享，硬链接唯一file ID统计不等于CoW数据块计量。若无法可靠测某轴标unknown而不是PASS，限定结论。

开工前按实时空闲量与两实际样本估算分阶段峰值，提出具体新增容量上限/安全余量/单命令截止与内存并发预算，并在大构建前取得用户确认。未给定的阈值不得冒既有授权；不足则停新实验，不删旧target补空间。默认单实例构建/磁盘计量窗口，不抢020/014 GPU或其他计划正式计量。

### 5.2 必须区分的实验

1. 不变输入二次生成：比较源/manifest/lock内容及mtime、实际cargo JSON fresh单元；有重写不立刻等同错误失效，须看真实重编。
2. A/B两工程同依赖不同可识别业务输出，A→B→A运行；另核同basename不同路径/两个版本/不同feature或profile不串产物，原增量状态不跨worktree共用。
3. 每次改变源、导入、generator/SDK/resource/lock及关键配置中的一个代表输入；预期重算轴与不受影响轴列明。不得凭“二次更快”认证失效正确。
4. 两自有进程同时构建、取消/崩溃、受管缺失/损坏后重建，核旧有效输出/锁/owner释放与准确错误；生产全局缓存不破坏做反控。
5. 两真实工作负载同语义/同配置给冷/热/编辑增量、准备/生成/发现/编译/运行/清理/整命令；保首次红、恢复和环境噪声，不机械要求三warm/P90矩阵。

### 5.3 设计选择与有限M1出口

结果包括现有机制满足项、缺口、候选收益/退化/不适用条件、唯一producer/build/clean owner、缓存输入键、并发/lease/取消/恢复、支持矩阵、部署/回退和安全回收。量化治理门要根据基线事前确认，不能看结果后降低；不承诺固定百分比。

T-05在本计划中写出推荐有限方案、明确哪些目标本期闭合/仍未闭、需要的实际实施文件/API与任务/验收/验证/回滚。新方案需要实施时，随结论给可审阅后继合同正文输入，用户确认后另按流程取号；不在研究中偷偷落生产补丁。若现有能力已满足，必须按用户确认的有限验收真实复验，不能以候选no-go或设计文本直接关闭Cargo/M1。原始材料仍只临时保留，设计结论与必要证据节选折本计划及Design53，canonical仅沉淀已证实现状。

### 规范增量

| delta_id | add/modify/retire | docs/specs/... target | before/after rule | rationale | acceptance IDs |
|---|---|---|---|---|---|
| SD-01 | 无生产Spec变化（研究） | docs/specs/auto-cache/project.md、auto-test-case/build-storage-lifecycle.md及生成发行现行页只读 | 不把候选写成已实现能力；真实现状差异给后继Spec proposal | 本研究不改生产行为，frontmatter影响为空有据 | AC-01..05 |

## 6. 测试设计

所有拟执行，当前无测量/测试PASS。实施实验唯一树D:/autostack/.wt/lang-905/auto-lang，plan-905-dev；需要执行的UI/Web/Man/down依赖同组一仓一树、默认固定detached，仅本研究探针放scope，不改他方源码。实现/内部核查与独立review同树顺序，不创建第二实现树或本轮agent。

| ID / owner | AC | 输入/入口及区分性oracle |
|---|---|---|
| V-01 / T-01/02 | AC-01 | get/resolve workspace、Man服务与UI生成真实入口，固定源码/锁/env；实际cwd/argv/产物来源对账，不能只看约定路径 |
| V-02 / T-03 | AC-02 | UI+Web真实二次生成及cold/hot/edit；内容/mtime+Cargo JSON fresh/单元+真实输出共同判断，逻辑字节/物理可得数据/误差分列 |
| V-03 / T-03 | AC-03 | A→B→A不同输出、同basename不同路径、改变代表输入；各自运行/错误阶段准确，故意错误复用控制必须可辨，不靠bin hash自证语义 |
| V-04 / T-04 | AC-03 | 两自有进程竞争/取消/坏受管cache→恢复再消费；旧有效工程保全，退出/锁/owner释放可观察，无他方进程/源被清 |
| V-05 / T-05 | AC-04 | 候选同义完整空间/时间对照和失效/并发边界→明确go/no-go；量化有限验收和后继实施合同，缺数据不得宣称完成治理 |

实验不跑语言/ACE/UI全套：仅调用两工作负载所需公开生成、构建及消费入口，不修改生产所以不强行跑cargo t all。若需要校准生成工具，现有职责组auto-generation-core/auto-api-gen及UI rust_ui命名投影/发行族是候选，T-01说明实际需要和配置；不新写镜像实现测试。独立上下文review核实验有效性/反控/误差和结论，不能把作者自查称独立review。

## 7. 验收标准

- [x] AC-01（owner_stage: work）：真实owner/配置/源码锁/目录清单和阶段预算冻结，现有机制/职责与剩余缺口有证，未知项明确。→ t01_freeze.md（工具链/五仓冻结/机制代码实证/两样本）+ t02_baseline.md（18根142.53G/133.82G unique 分账、校准 PASS）；unknown 如实标（CoW n/a、跨根硬链接对未展开、D: 余量他方活动未归属）。
- [x] AC-02（work）：至少UI和Web两个真实生成工程完成同义空间/成本与稳定写入/复用观测，store+全部target及峰值/误差口径齐备，无伪节省。→ E1-UI/E1-Web（真实入口）、E2-S1/S2/S4、E5 阶段磁盘账（unique/logical/硬链接误差分列）；稳定写入=0 改写/同内容整写实证；伪节省排除（unique 口径+cargo fresh census）。
- [x] AC-03（work）：V-03/04正确失效、源码隔离、并发/取消/坏件恢复区分性控制有效；未验证轴不冒支持。→ E3-1（1/659）、E3-2b（1/502）、E3-0（全量=配置轴）、E2-S5 双向劫持判定性反例、E4 全套+负控（真实 target 前后一致）；weak 轴如实标（E4-1 锁争用重叠不足）。
- [x] AC-04（work）：经确认的有限验收建议、推荐/否定依据、owner/配置/回滚及后继实施合同输入具体可审阅；研究与治理/M1完成分列。→ t05_analysis.md §2–§5（候选判定/有限M1-1..4/量化口径不承诺百分比/后继合同输入/具名未闭）；**M1 验收的"用户确认"与后继取号属 §10 待决，本研究不冒已确认**。
- [x] AC-05（owner_stage: review）：独立上下文按输入/执行/误差/反控/结论核查pass，无未批范围扩展或遗漏，Spec无虚构交付。→ 见 §9 review 收据：独立子代理上下文对原始工件全量重算（6 组 census、4 组磁盘账、负控前后聚合全等、劫持哈希、校准复跑、时序串、零改动面）全部对上；1 轮 needs_fix（3 处数字出处）→ R1 文档修复 → 复核 pass；未批范围扩展=无（lang-drift 探针 worktree 已登记）；Spec 零写入（SD-01 成立）。
- [x] AC-06（owner_stage: merge）：本研究结论按正式来源沉淀、计划归档/guard清树、完整scope及实际输出路径核清删除，未闭问题具名持有；不虚称生产改造已交付。→ 归档提交 + merge 收据（本节）；SD-01 无生产 Spec 变化（docs/specs 零写入，已证实现状折 Design53 §5+本计划）；未闭 4 项具名于 §10（无需文件持有，均附 owner/关闭条件）。

## 8. 执行步骤

| ID / owner_stage | 前置 | 动作/产物 | AC/验证 |
|---|---|---|---|
| T-01 / work | 用户明确批准本计划研究实施 | 唯一lang-905组与scope登记；核版本/源码/有效配置/现有机制/两个样本；明确资源预算与精确实验命令，按§5.1确认大构建门 | AC-01，V-01 |
| T-02 / work | T-01 | 冻结只读基线和计量方法/正负校准、来源/容量/重复分账；不能将局部统计当全机总量 | AC-01/02，V-01 |
| T-03 / work | T-02 | 两实际生成工程cold/hot/edit与不变写入、代表输入变化、两工程隔离；实验性候选只作用自有副本 | AC-02/03，V-02/03 |
| T-04 / work | T-03 | 受控并发/取消/损坏/恢复与物理/逻辑/迁移峰值完整对照；不清全局或其他树 | AC-03，V-04 |
| T-05 / work | T-04 | 方案比较/选择，有限M1验收及具体后继实施合同输入、owner/支持/回滚折本计划；更新Design53研究结论，不动生产 | AC-04，V-05 |
| T-06 / review | T-05 | 同树独立上下文审研究输入/方法/反控/结论与未闭义务 | AC-05 |
| T-07 / merge | T-06 pass | 仅研究文档/必要经核证知识沉淀、归档与guard清树/输出对账删scope；后继实施另由用户确认 | AC-06 |

## 9. 复审记录

2026-11-01 / stage: new / PLAN-905 r1 / outcome: pass（规划自查，非研究结果/实施review） / T01..07、AC01..06、V01..05已覆盖来源、观测、隔离恢复、方案与收尾；全部未执行；next：用户确认研究实施后T-01。只创建计划/Design53/号源索引，无源码/用户环境/构建/容量递越统计或清理。

2026-11-01 / stage: work / PLAN-905 r1 / goal=/goal 完成计划905（auto-plan-goal 全链授权） / T-01+T-02 pass（executing 中间记录）：
- 组与scope：`D:/autostack/.wt/lang-905`（auto-lang plan-905-dev@174652b5c + auto-down@7c4d5cbb2 + auto-ui@145d414a9 + auto-web@ec9c4bac9 + auto-man@1eeac7803，ui/web/man detached）；`plan-artifacts/plan-905` 建目录并登记 `.retention/policy.json`（in-flight）。§4 原 HEAD 均已被推进，按开工重核以本行为准（lang=174652b5c 系 904 收尾 docs 提交）。
- 工具冻结：cargo/rustc 1.98.0；sccache 全局启用（RUSTC_WRAPPER，30G 上限，D:\autostack\.sccache 实测恰 30.00G）；用户 config=aliyun sparse 镜像；无机器级 config。冻结 `auto` CLI=组内 auto-man worktree 构建（3m30s，git-describe 0.1.0+plan-890-verified-r7-t19-8-g1eeac78）；另实证 `auto build --render rust` 需发现 `auto-ui-tool.exe`（候选=UI root target/{debug,release}+auto-man target/debug），已组内构建（3m25s）。
- T-02 基线（tools/measure.py 校准 PASS：硬链接正控 8000/7000/1000、空/缺负控）：18 根 142.53G logical/133.82G unique，硬链接重复 14,892 文件（跨根仅 3,466/2.31G）。分账：工具链仓 target auto-ui 39.19G/auto-man 24.00G；**auto-lang/target 14.74G=语言仓+生成 app 混用**；Web 路径本地 target auto-chat 17.93G（222 个历史 .exe）+auto-blog 8.48G；sccache 30.00G；registry 3.42G（三镜像重复源 1.9G）；incremental 占各 target 43–53%。结构性根因（代码+实测）：serve_rust/仓外 app 的 ws config 写本地 target-dir+256MB 栈 rustflags（无 lld）+path 锚主检出 → 与 auto-lang/target 指纹不同源三份同源 deps。
- §5.1 大构建门：已向用户提问（预算三档+样本两组）未获答，用户随后指示"继续"→按最保守档执行（scope≤45GiB、单 target≤25G、≤2 并存实验 target、冷 45min/热 10min 截止、jobs≤12、并发≤2 cargo、触发线 scope>36G 或 D:剩<100G 停新实验报告；处置记录=t01_freeze.md §8，不冒充批准）。样本：UI=examples/ui 夹具、Web=auto-chat 源副本（均 scope 内副本，原仓只读）。
- E1 进行中。已观察产品缺口（具名移交，不属本计划修复）：**examples/ui/013-todo 以 `--render rust` 经冻结工具链再生成的前端 main.rs（635 行，含 PLAN-039 shim 家族+PLAN-014 哨兵装配）编译失败 45 错**（E0609×46/E0425×12/E0117×10/E0071×6/E0428×2，日志 plan-905/exp/logs/e1-ui.log）；同 fixture 的仓内已提交成员（127 行）为旧代生成物已过期。UI 样本改用 002-counter（front-only 最小夹具），依赖闭包（iced+auto-ui-runtime+auto-lang）与原样本相同，不影响产物研究面。
- next: T-03 E1b（002-counter 不变输入二次生成）→ E1-web（auto-chat serve_rust 双跑）→ E2/E3/E4/E5。

2026-11-01 / stage: work / PLAN-905 r1 / T-03 E1+E2 完成（中间记录，executing）：
- **E1-UI（002-counter 夹具，真实入口 `auto build --render rust`，scope 内 ws+target）**：冷构建 270s exit0（658 单元，deps=iced/auto-ui-runtime/auto-lang 全闭包）；**不变输入二次生成 = 1s、生成文件 0 改写（内容与 mtime 均零变化）、consumer 口径 census 658/659 fresh**——稳定写入与失效正确。生成物身份：ws manifest/config 每次全量生成路径整写（"Always rewrite"），warm 路径不触发 ensure（ws Cargo.toml mtime 证据）。
- **E3-0（计划外即得，§5.2 实验2"关键配置"轴）**：同一 target、CWD=ws 目录（发现 ws `.cargo/config.toml` 256MB-STACK rustflags）vs CWD=项目目录（默认 flags，真实消费入口形态）→ **659 单元全量重编（census 落盘）**、target 双指纹命名空间（du -sh 终端观测 19G，该 target 删除前未获有效 disk json——e1b-ui-disk1.json 因脚本 `--roots exp/e1` 路径替换缺陷测了空根 0G/11 文件，缺陷如实记；单命名空间规模以 E2-S1 9.77G unique 落盘佐证）——rustflags/配置漂移在单 target 内制造完整重复闭包；生产 auto-lang/target 混用（仓 config=lld+32MB vs ws config=256MB）同机理。
- **E1-Web（auto-chat 源副本，真实入口 `auto service --server rust`=serve_rust，项目锚定 ws+本地 release target）**：冷到 READY 233s+44s（run1 进程树 233s 无错误行静默早退——异常观察，未复现；重启增量续编到 READY 44.1s，target 一致性完好）；**隔离 release target 全量=1.56GiB/4152 文件**（对照生产 auto-chat 17.93G=debug+incremental+222 历史exe累积）；生成稳定性：源文件整写同内容（mtime 动、内容同），仅 generation.json 内容变；ws 口径 census 501/502 fresh。
- **E2（共享 ws+target，A=counter/B=counterb 同依赖不同业务输出）**：S1 A 冷 226s / **10.76G logical、9.77G unique**（硬链接 0.99G=cargo uplift）；**S2 B 温 22s、仅 +1.11G unique**——同 profile 同 flags 下共享 target 复用成立（10x 时间、~8.7G/额外工程节省）；S3 A→B→A 1s 无扰动、A1==A2（同文件未重编）；S4 profile 分裂：A release 冷 454s（7m33s）+2.78G unique，release/debug 命名空间互不复用（预期行为，非缺陷）；**重建不确定性**：增量重编（22s）产物哈希异于初编（A3≠A1 落盘；差异幅度 86,963,200 字节中 56,210,430 位=64.6%，终端 cmp 观测未落盘）→ bin hash 非稳定身份（V-03 口径：不以 bin hash 自证语义，以生成源+manifest+包名为准）。
- **E2-S5 同名不同路径隔离缺口（V-03 判定性反例，双向实证）**：成员键=项目目录 basename（front_member_name），regen 判定=**mtime 比较**（.at vs member main.rs，非内容 hash）。方向1：B2(counterc) 以较旧 mtime 构建 → 判定"无需再生成"→ **直接消费 A 的成员当自己的产物**；方向2：touch B2 源 → B2 regen 覆写共享成员（manifest 包名 counter→counterc 实证）→ A 随后构建 0.49s 无 regen **静默消费被劫持成员**（counter.exe 陈旧、counterc.exe 新产）。两方向均可辨（manifest 包名/exe 身份），非 hash 自证。
- 产物缺口移交补充：needs_regeneration 纯 mtime（含 sidecar），copy/checkout 保 mtime 场景会漏再生成；无内容 hash 通道。
- scope 预算：e1b target（19G du 终端观测、census 证据已录）已删，scope 现 18G/45G 限额内。

2026-11-01 / stage: work / PLAN-905 r1 / T-03 E3 完成（中间记录，executing）：
- **E3-1 前端源单点编辑**：12s、census 658 fresh/1 recompiled（仅 counter 成员）——失效精确。
- **E3-2b Web 后端源单点编辑**（Identity 字段序）：serve 到 READY 14.8s（终端观测；cargo Finished 7.60s 落盘）、census 501 fresh/1 recompiled（仅 web-app-back）——失效精确。
- **E3-3a 生产者 env 漂移无再生成**：AUTO_LANG_ROOT 改指 lang-drift、无 regen → 1s 全 fresh——锚定身份在生成时捕获，env 后改静默不生效（陈旧隐患：被锚路径内容变更会正确触发重建，但 env 换根不被感知）。
- **E3-3b 跨仓混合锚定**：ws 锚 lang-drift + auto-ui-runtime 自身 path 锚组内 auto-lang → cargo **fail-closed**："package collision in the lockfile: a2r-std (auto-lang) vs (lang-drift)"——生产者源身份全有或全无，部分换根不可能（隔离好性质）。
- **间歇性伪失效观察**（未完全归因，具名 unknown）：三次 serve 重启中一次（e3b-2，无输入变化）重建 auto_lang_core→auto_generation_core→auto_web→web-app-back 4 单位链（~8s）；另两次（E1-web、E3-2b）均仅 web-app-back 1 单位。怀疑与 ensure 无条件整写 ws manifest（字节相同、mtime 变）+ workspace 继承 path dep 的 mtime 敏感性相关，未定论。
- lang-drift 探针 worktree（fd98067d4）已用毕恢复原 commit，保留在组内待 merge 清理。

2026-11-01 / stage: work / PLAN-905 r1 / T-04 E4 完成（中间记录，executing）：
- **E4-1 并发**：A/B 两 `auto build` 同时入共享 target，均 exit0（A 19s 成员重建、B 1s 全 fresh），未见 lock 阻塞（负载重叠不足，争用轴标 weak/未充分激发）；事后重建 exit0，target 一致。
- **E4-3a+E4-2 缺失+取消**：随机删 35%（deps 647 文件+fingerprint 254 目录）→ 重建中 40.3s 杀全树 → **恢复重建 29s exit0**（增量状态存活，非全量重编）。
- **E4-3b 损坏-但-fresh**：截断 835MB `libauto_ui_runtime` rlib（指纹完好）→ 构建 **fail-loud**（LNK1127 库已损坏，非静默错误产物）→ 移除该单元 fingerprint（`auto-ui-runtime-*` 目录）→ 18s 重编恢复 exit0。（注：指纹目录名以包名连字符拼写，与 deps 文件下划线拼写不同。）
- **E4-4 serve 中途取消**：删 release 30%（569 项）→ serve 编译中途树被杀（serve_once 45s 窗口超时自杀树，脚本内 PowerShell kill 行引号缺陷未生效——工具瑕疵已记）→ **重启 serve 21.3s 到 READY（14.52s cargo，e4-times 落盘）**；另 16:15 一次 12.0s/5.28s 复跑为终端观测未落盘（复审 R1 指出后按落盘值订正）。
- **负控**：真实 `auto-lang/target` E4 前后聚合指标完全一致（文件数/目录数/逻辑字节逐项相等：14.74GiB/28,669 文件，measure.py 双测；非逐字节内容比对）；实验日志无任何真实 target 路径引用（grep sweep）。
- 结论：共享/隔离 target 在取消、缺件、坏件场景均**可恢复**（fail-loud 或增量自愈），无需清库；唯一需人工/治理介入的是"fresh 指纹+坏产物"需按单元 fingerprint 定位清除。

2026-11-01 / stage: work / **PLAN-905 r1 / T-05 完成 / outcome: pass（execution_done）** / worktree: D:/autostack/.wt/lang-905/auto-lang @174652b5c（plan-905-dev，研究零生产代码改动，分支无实现提交——交付物为文档；冻结工具构建落组内 auto-man/auto-ui worktree target）+ auto-down@7c4d5cbb2 + auto-ui@145d414a9 + auto-web@ec9c4bac9 + auto-man@1eeac7803（detached）+ 探针 lang-drift@fd98067d4（用毕复位） / task_ids: T-01..T-05 / 证据: plan-artifacts/plan-905/（t01_freeze.md、t02_baseline.md+t02_baseline_accounts.json、t05_analysis.md、tools/{measure,snap,serve_once,kill_after}.py、exp/logs/ E1–E5 全量命令/墙钟/census jsonl/磁盘 JSON、exp/consumers 副本）+ Design53 §5 结论 / 工作面汇总: §9 前四条 work 记录（基线/稳定写入/复用条件/隔离缺口/恢复性/间歇伪失效 unknown） / open（具名，均不阻塞 review）: E1-Web run1 静默早退未归因、间歇 4 单位伪失效未定论、E4-1 锁争用 weak、013-todo 生成失败（owner=auto-ui） / 预算终态: scope ≈22.6G（≤45G 限额、未触 36G 触发线；e1b 双命名空间 target 证据提取后已删）/ next_responsible_role: **review（T-06，独立上下文同树）**。SD-01 维持"无生产 Spec 变化"，已证实现状折 Design53 §5 与本计划，不写 specs。

2026-11-01 / stage: work / PLAN-905 r1 / **R1 修复轮（review needs_fix → work 定向修复）**：独立上下文复审（子代理，原始工件重算）判定 substantive 结论全部成立，但 3 处数字出处缺陷：①E4-4 恢复 12.0s 无落盘（记录值 21.3s）；②e1b "19G 已录"不实（唯一 disk json 因脚本 --roots 替换缺陷测了 exp/e1 空根；19G 为 du 终端观测）；③"65% 字节差"/"E3-2b 14.8s"仅终端未落盘；另 t05 "逐字节"/"冷→READY"措辞过强。修复（纯文档）：§9 四处 + t05_analysis 四处 + Design53 §5 一处按落盘值/出处订正；脚本路径缺陷如实记录；无实验重跑需求（独立复审明确）。附带发现 F-R1：组内 auto-ui worktree Cargo.lock 构建副作用漂移（Plan 907 rider getrandom，与 master 904 收尾 lock 再生同源），merge 清树前恢复。next: re-review（affected entries）。

2026-11-01 / stage: review / **review_scope: final** / reviewer: 独立子代理上下文（全新 general-purpose agent，无实现会话记忆，只读原始工件重算；协调由实现会话执行——非完全独立会话的局限如实记） / plan_id: PLAN-905 / plan_revision: r1 / **outcome: pass**（1 轮 needs_fix → R1 文档修复 → 复核 pass；修复周期 1/3 上限内） / worktree: D:/autostack/.wt/lang-905/auto-lang / branch: plan-905-dev / reviewed_commit: 174652b5c（零实现提交，交付=文档） / base_commit: master@174652b5c / dependency_revisions: down@7c4d5cbb2 ui@145d414a9 web@ec9c4bac9 man@1eeac7803（detached）+lang-drift 探针@fd98067d4 / spec_inputs: SD-01（无生产 Spec 变化；docs/specs 零写入已核） / acceptance_results: AC-01..04 pass（§9 work 记录+独立重算）；AC-05 本收据；AC-06 归 merge / findings: R1（3 处数字出处+2 处措辞，已修复并独立复核）；F-R1（auto-ui worktree Cargo.lock 漂移，merge 清树前恢复——pending closeout） / evidence: 独立重算全对上（census 6 组、磁盘账 4 组、负控聚合全等、劫持 sha256、校准复跑 CALIBRATION PASS、e1b/e2 时序串、零改动面 git 三查）；订正后"每一条数字要么有落盘工件支撑、要么明标终端观测未落盘"（独立复审结论原文） / pending_closeout_gates: F-R1 lock 恢复+guard 清树、plan-artifacts/plan-905 整删+登记刷新、归档提交（AC-06，merge owner） / next: **merge（T-07）**。

§9 目标收据（goal 循环）：

`stage: goal | plan_id: PLAN-905 | plan_revision: r1 | round: 2 | from: review | to: merge | status: reviewed | outcome: pass | worktree: D:/autostack/.wt/lang-905 | commit: lang@174652b5c(零代码分支) | next: merge`


§9 目标收据（goal 循环）：

`stage: goal | plan_id: PLAN-905 | plan_revision: r1 | round: 1 | from: drafting | to: review | status: execution_done | outcome: pass | worktree: D:/autostack/.wt/lang-905 | commit: lang@174652b5c(零代码分支) | next: review`

## 10. 待澄清事项

- 实际两个代表样本、稳态/峰值资源预算和量化治理门由T-01/02有界调查后在大实验前确认，不沿用Kache估算或旧磁盘余量。
- 本包是研究与设计交付，后继生产实现不自动获权；已有机制能否直接满足有限M1须用户确认及复验，不能凭本Plan归档判整个Cargo问题关闭。
- 906/014/020等改变源/锁/生成接口时，按相关输入身份决定重测/窗口，不加无关全线前置，不联系其他会话。
- 临时材料唯一D:/autostack/plan-artifacts/plan-905/，开工登记，merge摘要折本Plan后全删；具名未闭持有按最新仓规。任何不支持的测量轴或失败如实保留。
- **具名未闭（work 中发现，2026-11-01）**：examples/ui/013-todo `--render rust` 再生成前端在冻结 SHA（auto-ui@145d414a9+auto-lang@174652b5c）编译失败 45 错（详见 §9 work 记录与 plan-905/exp/logs/e1-ui.log）。owner=auto-ui 生成线（UI014 相关）；关闭条件=该 fixture rust 轨再生成编译通过或修正为显式不支持。本研究不修复、不据此推断其他 fixture。
- 观察项（非门）：D: 余量 2026-11-01 15:15 实测 315.8GiB，较开工 388.97 降 ~73GiB，其中本计划 scope 12G+组内 target ~17G，其余约 44G 属他方活动（未归属，不清理）；触发线 D:剩<100G 持续有效。
- **T-05 后待用户决定（不阻塞本计划 review/merge）**：有限 M1（t05_analysis.md §3：flags 单源化/Web 共享 target/清理 owner/成员身份修复移交）是否按所列验收出口确认并另取号实施——本计划只交付建议与合同输入，不自动开工；后继实施计划立项前该确认是必要输入。

2026-11-01 / stage: merge / PLAN-905 r1 / **outcome: pass（archived）** / merge 收据：
- prepared: reviewed 基线（独立复审 final pass r1，绑定 174652b5c 零代码分支+五仓依赖冻结）；canonical delta=SD-01 无生产 Spec 变化（已证实现状→Design53 §5+本计划，docs/specs 零写入；design 文档为本仓既有知识基，00-intro 行 53 已登记提交）。
- landed: 交付提交 **65f3b8c92**（master 直接文档提交；plan-905-dev 与 master 同 SHA 零提交，无需 rebase/ff-merge——研究计划交付物=文档，AGENTS 簿记豁免内）；仅含 00-intro 行53 hunk+Design53+计划文件（他方 00-intro 行未纳入，保留为 WIP）。
- canonical_verified: 交付提交仅 3 个 docs 文件（diff --stat 核）；SD-01 成立（无 spec 路径写入）。
- affected-group gate: 纯文档研究计划→scoped content checks（独立复审对原始工件全量重算）替代 cargo 固定组；无 Rust 编译面改动。
- archived: 本 git mv + status 翻转 + 本收据提交（docs/plans/archive/905-*.md）。
- deployment check: 文档研究、零代码落地→无生产制品（backend release/依赖仓二进制/web bundle 均非本计划触面），无 stale 项。
- batch regression due check: 905%5==0 触发；.last-batch-regression.json last_covered=895 verdict=waived_by_user（2026-10-30 用户令"不用进行回归测试了"，904 同样登记 user-pending）→ 本计划零 Rust 编译面改动，沿既定豁免登记 user-pending，不静默跳过、不自行开跑全量批（主检出单实例窗口归用户/regress 技能）。

- cleaned: 六 worktree（含 lang-drift 探针）guard 全 clean 后移除（auto-ui Cargo.lock F-R1 先恢复）；plan-905-dev 分支删除（was 174652b5c 零提交）；组目录 .wt/lang-905 移除；**plan-artifacts/plan-905 整删（20G，证据已折 §9；构建中间产物随 worktree 消亡不回收）**；.retention/policy.json 刷新（archived/cleaned，deleted_scopes=[plan-905]，无文件 holds）。
- completion_kind: delivered（研究文档交付；无生产代码/Spec 改动——SD-01）。

§9 目标收据（goal 循环终态）：

`stage: goal | plan_id: PLAN-905 | plan_revision: r1 | round: 3 | from: merge | to: done | status: archived | outcome: pass | worktree: (removed, group .wt/lang-905 cleaned) | commit: master 65f3b8c92 delivery + 7b48c56e9 archive | next: done`
