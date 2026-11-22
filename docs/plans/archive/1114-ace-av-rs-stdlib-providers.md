---
plan_id: PLAN-1114
status: archived
feature_name: "ACE AV/RS 的 *.rs.at 标准库"
author: [Codex]
created_at: 2026-11-20
updated_at: 2026-11-22
plan_revision: 1
current_step: 7
total_steps: 7
depends_on: [PLAN-1113, PLAN-959, PLAN-1043, PLAN-972, PLAN-973, PLAN-974, PLAN-975]
supersedes_spec_components: [docs/specs/stdlib/design/context-source-assembly.md, docs/specs/ace-migration/m2-denominator.md]
new_spec_components: [docs/specs/stdlib/design/av-rust-provider-coverage.md]
touched_goals: [GOAL-002, GOAL-003]
affects: [stdlib, auto-av, auto-abi, ace-m2-build]
---

# [PLAN-1114] ACE AV/RS 的 *.rs.at 标准库

## 0. 变更摘要

Rust 路线 AV/RS 的完整交付。交 `IF-ACE-AV-RS-STDLIB/1`，以真实标准库逐操作对账，作为本路线启动 A2R 实施的前置。 各批依赖和交接见 [Design 60](../design/60-ace-rust-route.md)。

## 1. 目标

使普通 AV 项目能显式选择、构建和使用与公共 `.at` 契约一致的 Rust 来源标准库。按当前支持合同闭合 AV/Rust 必需操作，包括现役 `*.rs.at` 迁移以及文件不存在但规范已有 Rust provider 义务的操作。公共 Auto 算法仍共享，默认 Auto/native 行为按既定标准库合同验证。

## 2. 架构方案

公共模块 `.at` + `PackageAssemblyContext(backend=AV, provider=Rs, target)` → 1043 装配和受检绑定 → `.rs.at` 内的 Auto 适配声明/体按实际调用进入 1113/967 Rust shim → AV 运行 → provider 自己管理资源。`#in sys` 处理平台条件。

## 3. 技术栈

`stdlib/auto/`、`stdlib/ace/`、`experimental/ace/crates/core/src/source/packages/provider_sources.rs` 与 `binding.rs`、`experimental/ace/crates/abi`、ACE SDK、公共同源 Case。

## 4. 需求分析与背景调查

[1043 装配](../specs/stdlib/design/context-source-assembly.md)与 [967 AC-LOAD](../specs/ace-migration/m2-host.md)已有选源与有限真调用。`stdlib/auto/platform/io.rs.at` 当前位置只有 `io_impl_layer_present()`，而 `stdlib/auto/str.rs.at` 含旧 `#[rs]` Rust 目标体。两类文件都要按实际角色分类。[标准库覆盖设计](../design/stdlib/backend-provider-coverage.md)、[959](archive/959-ace-m2-denominator.md)、972–975 和 1064 为分母依据。

本轮取材基线为主检出 `4438d4933baa09ee4c6727111e2de07ff6cabfa0` 及活动 Plan 工作区文本。用户授权检查 AV 路线并规划 Rust 路线，顺序为 AV/RS（含 `*.rs.at`）→ A2R 替换 v0.5。本次授权仅含语言仓路线和计划文档。实施前按 AGENTS.md 确认计划与实际依赖版本。

## 5. 详细设计

工作起点按操作生成公共 API → 源/选源 → ABI → 已编译 Rust 实现或共享 Auto 体 → 运行 oracle 对照。全量扫现有 15 个 stdlib .rs.at 资产（time、str、storage、sqlite、redis、rand、platform/io、math、list、json、inline、image、frame、file、env），把公共能力缺文件的 required 行一并纳入。逐条分类为共享 Auto 体、既有 Rust 实现可通过受检 adapter 复用、需迁旧 #[rs] 体或需新增 Rust 实现、显式外部/领域提供者、具名非支持面。保旧 A2R 文件发现和语义兼容到切换；如需同时维持两套消费，T-01 在已存在装配机制内确定源码过渡方式，冻结合法选择和冲突负控，继承唯一公共 API。

按 core/text/math/collections，I/O/fs/env/time/net，JSON 与其他数据三批交付。存储/Redis/SQLite/图像/帧等按实际规范与已支持项目判定，显式第三方依赖独立锁定并实测适用行，领域 API/图形验收由原 owner 签。每批独立交公开操作、标准库源、provider 身份、expected 与错误/资源事实。当某操作有共享 Auto 算法时记录其最终实际原语调用来源。默认路径去掉 Rs/C 工件后仍按原规范真跑。

### 规范增量

| ID | 操作 | 规范目标 | 交付后规则与理由 | 验收 |
|---|---|---|---|---|
| SD-01 | 新增 | `docs/specs/stdlib/design/av-rust-provider-coverage.md` | 新增逐操作来源、平台与实际支持矩阵 | AC-01/02/03 |
| SD-02 | 修改 | `docs/specs/stdlib/design/context-source-assembly.md` | 补标准库生产入口实际选择与混合来源约束 | AC-02/03 |
| SD-03 | 修改 | `docs/specs/ace-migration/m2-denominator.md` | 追加 AV/Rust 操作及原义务映射 | AC-01/03 |

## 6. 测试设计

以下为拟执行验证。各任务先定位现有同义 Case 或实现正式入口和必要反例，再记录准确命令、独立预期、实际阶段和产物身份。公共语料采用 1078 的唯一特性源。平台及条件执行范围见 [Design 60](../design/60-ace-rust-route.md)。

| Case | 验收 / work 任务 | 真实入口和区别性断言 |
|---|---|---|
| V-01 | AC-01/02 / T-02 | 真实 `auto.str` 公共字节长度、核心集合/数学经 AV Rust provider 验返回值；`A中😀\0` 长度 9。保持 Auto 源不变，改受选 Rust 实现/适配源码可观察结果/工件身份变化，改未选 `.py.at` body 保持身份。 |
| V-02 | AC-02 / T-03 | 真实文件/控制台与 loopback I/O 按适用义务独立断言 exact bytes、EOF、错误和关闭；跨 provider/代次误交 handle 在执行前拒绝，旧会话仍可使用合法 handle，正确 session 关闭无泄漏。 |
| V-03 | AC-02/03 / T-04/05 | 公共 JSON 值和坏输入、适用显式外库操作真实运行；缺点名 Rust 来源、错签名或撤销权限具名拒绝；移去可选工件运行默认 Auto 近邻。重定位 SDK 核依赖和所选来源，原 AV 语料与 legacy 仍可选有效实现。 |

`cargo t ace-stdlib ace-av ace-abi ace-m2-build case-av-language-parity`；改装配或兼容文件时补 `ace-source`、`stdlib`，按 AV 阶段选择混合组的共享/AV 检查。构建工具只在既有显式 build-lane 执行，日常组使用已准备的工件。 完整受影响固定组合并入最终复审后的冻结验证，记录应跑、实跑、原条件排除和未跑数，以及准备/执行/清理时间。独立 review 查实际源码、Case 和真实生产入口。

## 7. 验收标准

- [x] **AC-01** · owner_stage=`work`。每一原资产/公共必需操作有逐原映射、实际来源、owner、平台和独立验收，旧义务守恒。
- [x] **AC-02** · owner_stage=`work`。各批公共 API 真实执行并核错误/资源/授权；V-01–03 的选择、调用与关闭可区分错误来源。
- [x] **AC-03** · owner_stage=`work`。本版适用的 AV/Rust required 无未验行；仓外预编译消费成功且默认 Auto 路径成立，交完整矩阵和接口。
- [x] **AC-04** · owner_stage=`review`。逐项独立复验 AC-01–03、缺口归属及 Spec 增量，结论和证据绑定合同版本及提交。
- [x] **AC-05** · owner_stage=`merge`。冻结门通过后落主、沉淀已审规范与接口、归档及清理；核发布给实际消费者的来源身份。
  - 证据：冻结门零新红（见 §9 门禁收据）；ff-only 落主 3904aeb0e（range-diff 全等映射）；canonical 三路径哈希在落主提交核验（av-rust-provider-coverage e38a5804…/context-source-assembly 004c0876…/m2-denominator 3000aad6…）；主检冒烟 avrs 12/12；消费来源身份=SDK 分发测试（provider DLL+ABC+manifest 闭包，1113 链）＋陈旧部署观察记录（PATH auto.exe=11-19 构建，早于本计划，未重建——发布不在授权范围）。

## 8. 执行步骤

实施入口为 `D:/autostack/.wt/lang-1114/auto-lang`、分支 `plan-1114-dev`，基于实施时主检出的正式输入。按需在当前工具中登记 T 项再开工；首任务 T-01 的输入和待决项见 §4、§5、§10。材料统一放 `D:/autostack/plan-artifacts/plan-1114/` 并登记。

- [x] T-01 冻结操作分母与文件迁移方式
  - owner_stage=`work`。对账 959、972–975、1064、1063 与实际文件、现有语义和平台；记录旧 #[rs] 的归属和验证入口，给每个真实缺口指定本 Plan 任务或现有域 owner（AC-01）。
  - 证据：架构裁定 D-1114-01..04 + 逐操作矩阵 + 具名边界 B-01..B-11 落于 worktree `docs/specs/stdlib/design/av-rust-provider-coverage.md`（SD-01 目标文件）；15 个旧资产全分类，旧 `stdlib/auto/*.rs.at` 镜像原样保留（旧义务守恒），缺口归 T-02..T-05 或域 owner（image/frame=980/OS064、TLS=974r2、Linux=1063）。
- [x] T-02 交核心值和集合
  - owner_stage=`work`。落公共 text/math/list 等最小生产批、V-01，证明已选 `.rs.at` 经受检 Rust 来源实际产生结果（AC-01/02）。
  - 证据：`stdlib/ace/avrs/{str,math}` + `avrs_stdlib_1114::v01_*` 三例（`A中😀\0`=9 真分派、选中编辑翻身份/值、未选 py 稳定、契约面锚、具名拒）；`ViewI32ToI32` 帧道补齐（T-11 家族 i32 孪生）。
- [x] T-03 交 I/O 和系统操作
  - owner_stage=`work`。按正式 primitive、FFI frame 与 1113 接口完成 V-02，含操作级错误、取消/释放观察（AC-02）。
  - 证据：`stdlib/ace/avrs/{console,file,env,time,net}` + `v02_*` 五例（文件精确字节/EOF/-8、console 计数 oracle、env -203 哨兵、owned clock 代次/关闭句柄安全、真回环 echo）；完成口形归 M2Build 执行器（B-10）。
- [x] T-04 交数据和显式外部模块
  - owner_stage=`work`。接 975 等实际公共出口及适用外库，补 JSON/错误和显式依赖 V-03，域外义务获得原 owner 的正式接收（AC-02/03）。
  - 证据：`stdlib/ace/avrs/{json,rand}`（serde_json=1.0.151 钉版显式外库）+ `v03_*` 四例（坏输入具名 -1、缺源/错签名/撤权三类具名拒、移除全部可选工件后默认 Auto 近邻同 byte 面）；sqlite/redis/storage 沿 967/975 已交 lane 复用（矩阵 X/S 行）。
- [x] T-05 验 AV/RS 标准库工程分发
  - owner_stage=`work`。按矩阵执行剩余 required 行，装配/重定位/默认 Auto 回归具备独立断言，准备 `IF-ACE-AV-RS-STDLIB/1`（AC-01–03）。
  - 证据：`compose_avrs_provider`/`build_avrs_provider` 拆分（选择→分发同身份）；`ace-m2-build/tests/avrs_project_1114.rs`（工程构建→运行 209137→SDK 打包→重定位→删开发缓存→迁移后运行）；接口冻结表落覆盖文档；矩阵批 1–3 全行 + B-01..B-11 边界齐备。
- [x] T-06 独立复审
  - owner_stage=`review`。核 AC-01–04，回放关键正反例、实际入口和结果，记录遗漏、延期及替代实现，绑定 revision 和提交。
  - 证据：final 复审（needs_fix→修复→复验 pass）双段记录见 §9；F-1114-R1/R2 修复于 commit 8418597d9（纯文档增量，零代码漂移=测试证据有效）；F-1114-R3 记 §10（域 owner 签收关闭条件）。
- [x] T-07 验证冻结交付与归档
  - owner_stage=`merge`。执行冻结受影响组，核 AC-05，落主并交接，折叠证据后按 guard 清理并完成路径对账。
  - 证据：冻结并集 run-c44l9666（§9 收据，零新红）；wt-guard clean 后 ff-only 落主 3904aeb0e；canonical 核验 + 主检冒烟；归档/清理收据见 §9 末条。

## 9. 复审记录

2026-11-20，stage=new，PLAN-1114 r1。计划已起草，实施确认和 §4–5 所列前置交付待办。outcome=blocked（work 尚未授权），next=确认后进入 T-01。T-01–07、AC-01–05 均待执行。

2026-11-22，stage=goal，PLAN-1114 r1（/goal 完成计划1114 授权整链）。授权记录：用户 `/goal 完成计划1114` = work+review+有界修复+merge 一体授权；worktree `D:/autostack/.wt/lang-1114/auto-lang`（base f4b3688bc，分支 plan-1114-dev）；材料区 `D:/autostack/plan-artifacts/plan-1114/`（已登记 owner-registry.md）。主检出 `.wt/lang-1095/` 为他人误建目录（非本计划所有，仅上报不处置）。round=1，from=work，to=review，status=executing→execution_done 待冻结组，outcome=pass（T-01..T-05 完成见 §8 证据），next=T-06 独立复审。

2026-11-22，stage=work，PLAN-1114 r1。work 交接（readiness）：work-owned AC-01/02/03 全过（§8 逐任务证据）；无未闭 in-contract 发现。受影响验证：新增 13 例全绿（12×ace-abi avrs_stdlib_1114 + 1×ace-m2-build avrs_project_1114，热径 <5s，时序表见 plan-artifacts owner-registry.md）；固定组 ace-m2-build 41/41 PASS；固定组 ace-abi 340 执行 28 红——**全数为主检出同族预存红**（source_foreign 家族，`foreign.authority.runtime-or-emitter` 环境归因，主检 85 跑 29 红[28+1 perf]与本树 28 逐族一致，抽样 4/4 主检复现；PLAN-1016 裁定 ::performance:: 另计）；ace-av 受影响子集（console/foreign/1041/1043/probe972）65/65 PASS——ViewI32ToI32 帧道增量无回归。统一冻结并集（ace-stdlib ace-av ace-abi ace-m2-build case-av-language-parity + 改动面 ace-source）= T-07 merge 门，work 不冒。代码身份：worktree plan-1114-dev @ 8d3d0ddbc（3 commits：e249faae3/cfe9e1487/8d3d0ddbc）。outcome=pass，status=execution_done，next_responsible_role=review（T-06）。

stage: goal | plan_id: PLAN-1114 | plan_revision: r1 | round: 1 | from: work | to: review | status: execution_done | outcome: pass | worktree: D:/autostack/.wt/lang-1114/auto-lang | commit: 8d3d0ddbc | next: review

stage: review | review_scope: final | reviewer: 本会话（实施会话内复审——独立性限制已声明；结论从工件重建：git diff 面检、矩阵逐资产对账、冻结提交重跑） | plan_id: PLAN-1114 | plan_revision: r1 | outcome: needs_fix | worktree_path: D:/autostack/.wt/lang-1114/auto-lang | branch: plan-1114-dev | reviewed_commit: 8d3d0ddbc | base_commit: f4b3688bc | dependency_revisions: 依赖全 archived（1113/959/1043/972–975/1064；1063 在途不阻塞） | spec_inputs: context-source-assembly.md / m2-denominator.md / backend-provider-coverage.md / Design 60 / m2-host.md AC-LOAD | acceptance_results: AC-01 pass（矩阵 15 资产全覆盖、`stdlib/auto` 零改动=旧义务守恒、边界 B-01..B-11）；AC-02 pass（V-01/02/03 复审重跑 13 例全绿，错误/资源/授权来源可区分）；AC-03 partial（接口冻结表✓、默认 Auto 近邻✓、仓外 SDK 重定位✓；SD-02/SD-03 增量未备=缺口）；AC-04 本审执行 | pending_closeout_gates: T-07 冻结并集（ace-stdlib ace-av ace-abi ace-m2-build case-av-language-parity + ace-source）| findings: **F-1114-R1**（minor，AC-03）SD-02（context-source-assembly 补生产入口实际选择与混合来源约束）与 SD-03（m2-denominator 追加 AV/Rust 行及原义务映射）未在 worktree 备好；**F-1114-R2**（minor，AC-01）矩阵未记 `stdlib/auto/platform/io.rs.at` 占位资产归属（§4 点名的 16 号资产）；**F-1114-R3**（process/low）image/frame 域外义务的"原 owner 正式接收"跨会话不可得——B-03 归属已记，正式签收属 980/OS064 owner 会话职责 | evidence: 复审重跑 12+1 全绿（v01-core 3.02s / v01-contract 1.24s / v01-neg 0.16s / v02-file 1.45s / v02-console 1.35s / v02-env 1.37s / v02-handles 1.87s / v02-closed 1.31s / v03-json 6.69s / v03-neg 1.34s / v03-grants 6.79s / v03-default 0.35s / p1114-sdk 1.31s）；diff 26 文件 +1351 行，stdlib/auto 0 改动；测试纪律核：新例行为+时序表在案、无 >10s 用例、日常不触发前置编译（分例选模+摘要键控构建复用）| next: work 有界修复（R1/R2）→ re-review

2026-11-22，re-review（修复复验，同 reviewer/基线链）。修复核验：F-1114-R1 ✓（SD-02 生产入口节 + SD-03 delta-2026-11-22 落 worktree，内容核过；frontmatter supersedes/new 与实际一致）；F-1114-R2 ✓（io.rs.at 占位行入矩阵 L77）；F-1114-R3 ✓（§10 记录+关闭条件）。8d3d0ddbc→8418597d9 纯文档零代码漂移=全绿测试证据有效。outcome=**pass（final）**，status=reviewed，reviewed_commit=8418597d9，next_responsible_role=merge（T-07；冻结并集待其执行）。

stage: goal | plan_id: PLAN-1114 | plan_revision: r1 | round: 2 | from: review | to: merge | status: archived | outcome: pass | worktree: D:/autostack/.wt/lang-1114/auto-lang | commit: 8418597d9 | next: merge

2026-11-22，stage=merge，PLAN-1114 r1。prepared：master 漂移至 0046408c5（他方会话落主，与本分支**零文件重叠**）；plan-1114-dev rebase 到 master 后 `git range-diff` 4/4 全等（补丁等价证明）：e249faae3→41323fdf5、cfe9e1487→4417957af、8d3d0ddbc→8fd757be7、**8418597d9（reviewed）→3904aeb0e（delivery candidate）**。冻结受影响组并集（ace-stdlib ace-av ace-abi ace-m2-build case-av-language-parity）在重定基基线 3904aeb0e 执行（见下方收据）。批量回归到期检查：1114%5≠0，回执 2026-11-21T04:20Z（<48h）→ 不到期。

**冻结门禁收据（基线 3904aeb0e，run-c44l9666）**：ace-stdlib PASS 2/2；ace-m2-build PASS 41/41（含 avrs_project_1114）；ace-av 1583 执行 13 红——逐名与 master 0046408c5 完全一致（av_method_modes/av_method_owned×5/av_owned_first_case/av_owned_recipe/av_test_batch::c_data_profile/component_event_sessions_1106::v02b/resource_915×3/av-core layout——master 漂移引入的预存红，非 1114 面）；ace-abi 340 执行 28 红——source_foreign 族预存（同前 work 收据的主检复核）；case-av-language-parity 529 执行 527 PASS/162 SKIP(AV-ONLY)/2 红——`av.parity.1093.explicit-c-library`/`c-header-spelling-rejected`＝主检出逐名复现（同 2 例；`foreign.prepared` C 闭包缺席=1093 源未入库已知债务、预热具名跳过）。**判定：零新增红（known-reds-only），门禁对 1114 通过**；预存红保持显式红、不充绿。

2026-11-22，stage=merge，PLAN-1114 r1（续）。landed：master==3904aeb0e（ff-only，无 merge commit；rebase 映射见 prepared 条）。canonical_verified：三 canonical 路径与源哈希在落主提交核验（SD-01 新增 av-rust-provider-coverage.md 114 行 e38a5804…；SD-02 context-source-assembly.md 125 行 004c0876…；SD-03 m2-denominator.md 427 行 3000aad6…）；主检落主后冒烟 avrs 12/12 全绿（5.9s）。archived：本文件移入 docs/plans/archive/ 并提交（completion_kind=delivered）。cleaned：wt-guard 二次 clean → worktree/分支/组目录删除；plan-artifacts/plan-1114/ 整目录删除（证据已折叠本文件）；.retention/policy.json 刷新（PLAN-1114 deleted-at-merge, no holds）。部署观察：PATH auto.exe（auto-man debug，11-19 12:27 构建）早于本计划引擎改动——陈旧观察记录，未重建/未重启（发布不在本次授权范围）。批量回归：不到期（1114%5≠0；回执 2026-11-21T04:20Z <48h）。outcome=pass（delivered）。

stage: goal | plan_id: PLAN-1114 | plan_revision: r1 | round: 3 | from: merge | to: done | status: archived | outcome: pass | worktree: (removed) | commit: 3904aeb0e | next: done

## 10. 待澄清事项

T-01 复核运维所需的外部服务、可携测试依赖与少数旧 Rust 专用 helper 的公开边界。Windows 无法运行的 Linux 验证腿按 1063 延期台账记，列实际 required 输入、平台、状态和 owner。

已在途决议（work 阶段记录，复审核）：
- redis 真服务器验证=外部服务依赖（B-11），RESP2 算法沿 975 共享 Auto；如需真服务器腿需运维输入（服务地址/可用窗口）。
- serde_json=1.0.151 钉版沿 MIX-07 干净工程规则；json 用例依赖本机 cargo registry 已有该版（mix07 同环境先例）。
- 完成口形（sleep/now 旧 `#[rs]` 面）归 M2Build 工程执行器承载（B-10，1113 sdk_pack 已证）；avrs 会话车道不重复该面。
- **F-1114-R3（复审登记）**：image/frame 域外义务的"原 owner 正式接收"跨会话不可得——B-03 归属（980/OS064）已在覆盖矩阵记账；正式签收属 980/OS064 owner 会话在其自身复审中的职责，本计划不代签、不冒充。关闭条件：对应 owner 计划复审时核对 B-03 行。
