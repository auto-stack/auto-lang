---
plan_id: PLAN-1001
status: archived
feature_name: 增量缓存生命周期治理与生成工程调试策略验证
author: [Codex]
created_at: 2026-11-10
updated_at: 2026-11-10
plan_revision: 1
current_step: 8
total_steps: 9
depends_on: [PLAN-910, PLAN-942]
supersedes_spec_components: [docs/specs/auto-test-case/build-storage-lifecycle.md]
new_spec_components: [docs/specs/repository-boundaries/generated-debug-policy-study.md]
touched_goals: [GOAL-003, GOAL-016]
affects: [auto-test-case, auto-generation-core, repository-boundaries]
---

# [PLAN-1001] 增量缓存生命周期治理与生成工程调试策略验证

## 0. 变更摘要

承接910的受管共享target、942的共享workspace研究及2026-11-10空间调查，按两个独立阶段验收：**A先交付增量缓存治理，B再验证生成工程调试策略**。本轮仅起草r1，0/9；不实施、不清缓存、不改profile、不启动构建或定时清理。

阶段A交付可反复执行的“盘点→候选→确认闲置→安全回收→重建后复核”入口；阶段B交付公平对照和采用建议，**不包含生成器/应用默认profile推广**。B收益为零、负数或不确定，均须如实给出结论，不通过制造正收益完成验收。

编号由中央脚本完整分配为1001，`.next-id`升至1002，不截断/复用旧号。已发现auto-musk的`backend/crates/musk/src/relay/plan_control.rs::parse_plan_id`使用`PLAN-([0-9]{1,3})`，可能把本号误认成100。四位编号的发现、读取、路由和交接完成兼容验证前，**不自动派发本计划**；修复属auto-musk独立事项，不夹带实施于本包。文档起草不受阻，执行前置见T-01。

## 1. 目标

1. 将空间分成受保护、归属不明、可治理闲置三类，明确owner、构建上下文、profile/triple及实际缓存路径；可重复预览且默认零写入。
2. 对已确认闲置的增量缓存提供有锁、快照绑定和失败记录的有限回收；保护源码、锁文件、发布件、deps/native build、活跃上下文及持有证据。
3. 以容量水位辅助候选排序/告警，不能为达容量目标越过保护；测出即时释放、完整必需工作集恢复后的净空间和重编代价。
4. A验收后对UI/Web生成工程做调试信息对照，量化空间、构建成本与调试能力，给出维持/建议采用/证据不足的决定。
5. 形成长期只读巡检入口；清理保持显式执行，支持无候选、保护容量超预算、失败重试及用户持有状态。

非目标：关闭全局incremental、降低sccache的30GiB配置、跨worktree共享可写target、集中生成workspace/统一锁、改优化级别/LTO/panic、减少必要测试、推广产品默认调试策略、清理未知/在途计划材料、创建或启用周期任务。

## 2. 架构方案

复用`scripts/test-governance/build_storage.py`的精确路径、owner、进程、lease和reparse拒绝规则，增加受限的增量缓存模式及分配空间统计，避免第二套宽松清理器。当前脚本只认精确Cargo缓存根，**不能直接把incremental子目录传给现有 --apply 当作已支持**。新接口由T-03交付后才可使用。

数据流：受限owner根/真实Cargo配置/retention持有→只读库存及计量→保护集合与候选快照→确认维护窗口、持有Cargo互斥并重核→回收获准增量单元→收据与完整工作集重放→分阶段结论。

治理单元优先采用已确认闲置上下文内的完整incremental根，不按rustc内部session名称或文件mtime猜测过期crate。不把`cargo clean -p`、删除deps或整target伪装成仅清增量。内部布局会变，须核当前工具链及实际target-dir/build-dir；未知布局只报告。

B沿用910共享target语义，只在独立试验缓存中改变调试因素；锁文件、workspace布局及增量治理状态固定，不重复942的集中布局试验。

## 3. 技术栈

Python（现有清理器、unittest、Windows文件身份/分配量及进程检查）、Cargo/rustc（锁、profile、JSON构建消息）、现有生成入口与发布收据。psutil缺失时保守拒绝，不自动安装、不引入常驻服务。Windows锁语义先由T-02实测，不能照搬Unix假设。

## 4. 需求分析与背景调查

### 授权与范围

用户此前授权三处旧target的一次性治理；现要求立正式计划，先治理增量缓存、再验证调试策略。**本轮仅规划**，后续按正式work入口实施；不将立计划解释成即刻清理/配置授权。用户未指定回收百分比、时长或重编预算，不虚构已批准数值。

拟实施代码集中在语言仓清理器与现有测试。候选维护根限显式登记的`D:/autostack/auto-lang/target`、`D:/autostack/auto-lang/experimental/ace/target`、`D:/autostack/auto-man/target`及语言producer具名生成上下文；每根重核owner、配置、lease、持有和维护窗口。Man仅维护已核缓存数据，不改源码/profile；无owner闲置证明则只读。worktree、batch根、其他producer、plan-artifacts默认保护。维护已存在主检出缓存不等于允许在主检出编译/修改代码。

用户明确保留`D:/autostack/plan-artifacts/fix-cache-governance-20261110`四份审计材料，registry状态`cleanup-completed-audit-held-by-user`。本包及长期巡检不得因年龄、容量或上次清理结束释放该持有。执行材料只在`D:/autostack/plan-artifacts/plan-1001/`，开工时才创建、登记owner；本轮不建材料scope。

### 已核事实与收益边界

| 2026-11-10调查快照 | 本包含义 |
|---|---|
| 扫描内编译/缓存相关分配305.258GiB，incremental 99.696GiB | 非全机磁盘总量，也非可回收承诺；执行时更新基线 |
| 语言主target增量11.682GiB、Man 16.418GiB、ACE主target 8.542GiB，合计36.642GiB | 优先调查池，不承诺全部闲置；活跃/持有先剔除 |
| 语言工具主target统计已排除嵌套generated-consumers；具名生成根增量约1.946GiB | 嵌套根单列，不重复相加 |
| 共享生成根PDB约0.648GiB，rlib内调试信息未量化 | B测完整闭包，PDB不是全部收益 |
| 前次三处旧target已释放25,545,295,160B（23.79091GiB） | 排除出本计划收益 |
| sccache约30.054GiB，当前服务缓存命中405/484约83.7% | 容量不改；当前样本非长期命中保证 |

原调查按卷/文件ID去重，不含目录/MFT、未关闭的已删除文件等开销；路径逻辑量中的硬链接重复不能再算收益。快照有时间窗口及并发，不能作为删除前清单。

### 规范、代码与重叠核查

调查HEAD=`003fa8816f3d10d185247dc74cba216a1b3b9d77`；有其他未提交文档，以下按实际内容SHA256绑定，执行时重核，不只信HEAD。

| 来源 | SHA256 / 当前结论 |
|---|---|
| `docs/specs/auto-test-case/build-storage-lifecycle.md` | `191c22f8c59b4e448a1ea95ab7e6ecc0db048429f1a76fc136945f21e82dab47`；精确根清理、材料保护、完整必需工作集稳态 |
| `docs/specs/repository-boundaries/generated-cargo-build-lifecycle.md` | `10cab2b159f9899244acbddf2f7705703b2617a4a5b8260e6a56fd114d0f658b`；910上下文/发布/lease/flags边界 |
| `docs/specs/repository-boundaries/generated-workspace-sharing-study.md` | `f8f70f6298f1b72010bb18882665db64cb8b6fc0c3d36ad7314b383fc2b90603`；942未证明集中workspace额外收益 |
| `scripts/test-governance/build_storage.py` | `cf028980b1a263e4019890a797f77f58a20da3beded6b051b21af70596d95e53a`；只支持精确target，现计量非完整分配量 |
| `scripts/test-groups.toml` | `b2f6f6b9203efe576ac2b0795f218338b05512e1d203f50b1ba7606bdc4f7453`；已有infra-fixed、auto-generation-core、generated-delivery-controls |

相关实现：`build_storage.py::{inspect,refuse_active_processes,foreign_compiler_scope,main}`、`crates/auto-generation-core/src/{generated_build.rs,rust_workspace.rs}`；守卫测试在`scripts/tests/test_fixed_groups.py`。语言根已采用成员行号级/非成员依赖无debug，ACE另有full-debug与runtime专用合同，本包不改。

overview仓库地图较旧，拆仓/生成事实以以上专页和代码为准。活跃计划未发现本需求的相同合同；978是ACE M2增量/多目标/SDK，不由本包替代；1000等在途构建受保护。本结果不自动充当989联合验收。

Cargo官方资料（2026-11-10核读）：[profiles](https://doc.rust-lang.org/cargo/reference/profiles.html)说明workspace根profile、config/env覆盖及行号级信息不保局部变量；[build cache](https://doc.rust-lang.org/cargo/reference/build-cache.html)说明target/build目录可分离且内部布局会变。安装工具链的实际解析由T-02/T-06验证。

## 5. 详细设计

### A：增量缓存生命周期和容量

**库存。** 每行包含规范owner根/来源、实际target/build-dir、profile/triple、上下文、用途、lease/活动/持有/预留、路径/unique逻辑量、unique分配量、可释放分配上界与拒绝原因。嵌套根唯一归属，跨scope硬链接全局去重，归属字节与可释放字节分开；文件ID/分配量读不出标未知，不拿长度当实测。

**候选/水位。** 状态为protected、unknown、eligible-idle。先剔除前两类，后排序。闲置由owner退役/维护窗口、有效配置和无占用共同证明；mtime只展示。容量policy显式列owner、增量范围、high/low字节值及依据。T-02测工作集/恢复成本后提出初始值；未配置只盘点，不擅设硬上限。保护容量超限、安全候选不足时报告protected-bytes和shortfall，不越保护、不关闭incremental。

**快照/互斥。** 预览manifest绑定时间、工具链、路径/卷/文件ID、owner/持有及树摘要。apply仅消费显式选中的同一快照；在owner确认的维护窗口，按固定序持有当前Cargo真实互斥锁及受管lease协调，再重读retention、进程、路径和候选树。任一身份/持有/树变化即拒绝，不默默更新快照继续删。T-02须证明真实锁位置和阻塞效果，“能打开锁文件”不是互斥证明。

读取真实进程cwd/argv/env，覆盖cargo/rustc/linker/nextest及受管服务；不可观察或归属不明拒绝，保留现有responsefile/flags/drive-relative等边界。Cargo锁只约束参与协议的producer，不能承诺阻止任意直接启动的rustc；不受协调写入者须有明确停写窗口，否则不支持apply。不停其他进程。

递归删除前重新解析绝对路径，验证位于获准owner根的精确incremental单元，祖先/子树无reparse、证据/源码/发布件混入。禁止通过任意owner参数、字符串前缀或手写JSON把未知路径升级为准入；外仓owner须真实workspace/config及显式登记双重证明。校验失败零删除；部分失败非零，分别记已删/未删，不把预计量冒称释放。

**拟新增入口（T-03已交付，最终形态如下）。**

```text
python scripts/test-governance/build_storage.py --incremental --inventory <registered-owner-root> --output <inv.json> [--policy <p>]
python scripts/test-governance/build_storage.py --incremental --preview --policy <policy.json> --output <preview.json>
python scripts/test-governance/build_storage.py --incremental --apply-manifest <preview.json> --policy <policy.json> --idle-confirmed --output <apply.json>
```

T-03 落地要点（commit 67ab372d9）：policy schema=`incremental-cache-policy/v1`（owners[{owner,root,rationale,foreign}]+watermarks[{high_bytes,low_bytes,rationale}]，严格键校验）；manifest schema=`incremental-cache-manifest/v1`（policy_sha256/工具链指纹/每单元树摘要+文件数/水位报告，`zero_deletion:true`）；单元状态 eligible-idle/protected/unknown（unknown=分配量读不出，保护性降级，不拿逻辑量冒充）；apply 收据 schema=`incremental-cache-apply/v1`（deleted[]/failed[]/locks_held/zero_delete，部分失败退出码3，拒绝退出码2）；锁=profile级 `.cargo-lock`(+build/artifact 若在) 非阻塞固定序获取，被持整体拒绝；foreign 根双证明=foreign:true 登记+父级 Cargo.toml+目录名 target；foreign 生成根=经登记 foreign 工作区链+context.json producer 证明（schema/context_id/producer_root 三核）；retention registry 缺失/不可读=拒绝；保护单元宽松记账（lenient accounting）保保护容量诚实；旧 CLI 完全不变。22 个新测试钉 V-01..V-05（含真实 msvcrt 锁竞争拒绝、篡改 manifest 拒绝、水位 shortfall、嵌套/硬链接/unknown/三位形、幂等失效、旧CLI兼容）。

输出与控制夹具显式在本包scope。policy/manifest严格校验，不能附带任意删除路径。T-03回填最终帮助/字段/错误码或等价接口；旧精确target CLI保持兼容。长期周期运行仅只读库存/告警，apply仍需当前快照和显式操作。旧本地target重现时报告配置链，不自动删。此前周一巡检仅呈现建议卡片，未确认启用，不声称已运行。

归档后不复用已关闭的plan-1001材料目录。每次后续维护须有独立run身份；需要落盘时在统一plan-artifacts下登记轻量任务slug、owner及关闭条件，结束后将必要摘要/释放事实记入retention索引并清理该次临时材料。只读巡检可直接输出摘要，不为每次扫描永久堆积原始清单；用户持有及未知状态仍保守保护。

**收益/成本。** 同口径记录清理前、清理后、完整必需工作集恢复后、代表编辑重编后，分别给即时释放、恢复后净差、缓存回长、重编和治理耗时。并发扰动单列，无法归因标不确定。零候选合法，不造空target充正控/收益；无持续净收益时明确“一次性治理/持续收益未证明”。

### A到B的闸门

T-05核实安全控制、重复执行、真实处置或零候选、重建后计量，并独立检查AC-A1..A4通过，才开始B。此阶段检查不替代最终T-08复审，不提前归档/合并A。没有A结论不得并行改debug混合收益。

### B：生成工程调试策略对照，仅验证

**样本。** 复用942的UI `auto-ui/examples/ui/002-counter`、`013-todo`及Web `auto-chat`、`auto-database`，以拆仓后真实路径和owner入口为准。T-06冻结来源提交/hash、完整生成命令、生成ws、锁文本、工具链、triple、features、flags、linker、有效profile覆盖及行为oracle。样本/入口缺失或红具名记录，不静默换小样本。Web主要走release，先核有效debug；已无debug作为零空间边界，不拿UI dev候选与Web release基线比较。

**臂。** A为当前有效策略；B只将生成workspace根dev成员改为`line-tables-only`；C在B上仅增`[profile.dev.package."*"] debug=false`。A−B归因成员debug，B−C归因非workspace成员依赖override（含path依赖，不只第三方库）。若A已含某因素则合并重复臂并说明。各臂固定incremental、锁和workspace形态；release按实际入口另记，不顺改release。

**公平性。** 各臂独立试验target/build-dir，同文件系统，不用A暖B。尽量在同一规范化生成路径串行切换根配置并恢复，所有产物在scope；源码/锁相同，必要路径差异逐项列明。禁止在用户真实ws改profile。sccache容量固定，各臂采用等价隔离命名空间或仅试验禁wrapper，列命中/冷暖；不清全局cache。无改动复建估计波动，收益不大于误差界不称有效。

**测量序列。** 两UI应用加入并完成构建后的必需稳态→无改动复建→同一代表编辑→回退/恢复；Web按真实release入口补行为及有效debug观察。分别报冷编、无改动、编辑重编时间、单元/复用、稳态及峰值。全闭包包括target/build、生成源码/锁/收据、发布exe/PDB、辅助profile/回退产物和临时试验峰值；PDB/rlib是交叉分类，不叠加总量。

**调试/决策。** 验证相同行为、发布发现和新鲜度；选确定生成Rust调用点核文件行号栈，调试器可用时测成员断点/变量及依赖诊断。缺调试器记未验证，不报完好。明确行号级失去变量信息的成本；写出并实测恢复现有完整debug的根配置及override步骤，不假设自定义profile自动恢复依赖override。记录旧新指纹共存峰值及恢复成本。采用建议须空间净差超过误差且必要行为/所声称保留能力成立，否则维持或证据不足。无论结论如何不改产品默认。

### 空间口径与归因

主口径按卷/文件ID去重的文件分配字节，辅报路径/unique逻辑字节、外链、未知项、开始结束及扰动。释放由实际文件/链接状态复核，磁盘空闲量仅交叉检查；未关闭句柄未核实的量标“待实际释放”。

互斥分类至少为incremental/deps/native build/profile其他/生成源码元数据/发布件/试验其他。PDB与rlib为交叉视图；嵌套target、独立build-dir、发布硬链接均去重。sccache不作本包减量分子，但试验增量计入总体临时峰值；材料和残留也列账。

分账：①前次23.79GiB排除；②A即时与恢复后净收益；③B在相同A状态下的A−B/B−C及误差；④迁移/回滚峰值。不同时间/闭包不直接相加，删除试验scope不算持续产品收益，统一锁/workspace集中不算本包收益。

### T-02 回填（2026-11-10 实测，全部只读；原件在 plan-artifacts/plan-1001/t02/）

**基线**（psutil 7.2.2，18.8s；分配量=GetCompressedFileSizeW，与逻辑量相等=卷未压缩；跨scope硬链接0；unknown-alloc 0）：

| 根 / 增量单元 | 分配量 | 文件数 |
|---|---|---|
| auto-lang `target/debug/incremental` | 11.749 GiB | 26,954 |
| ACE `target/debug/incremental` | 8.693 GiB | 40,044 |
| ACE release/runtime-dev/runtime-release/x86_64…/runtime-release | ≈0 | 10 |
| auto-man `target/debug/incremental` | 17.524 GiB | 33,419 |
| 生成根 `target/generated-consumers/f3f98a9e/debug/incremental` | 2.036 GiB | 3,909 |
| **合计** | **≈40.0 GiB** | ≈104k |

较§4调查快照全线上浮（11.682→11.749 / 16.418→17.524 / 8.542→8.693 / 1.946→2.036）＝活根持续写入。**注意**：lang主target基线取于14:40，随后14:44锁探针shell后台化失误在主检出触发了一次全workspace debug构建（sccache暖命中，73s），已写入新增量会话——T-04 apply前必须重测，该73s同时是"全量debug重建（sccache暖）"的真实数据点。基线时进程检查0个构建进程。

**锁/布局（cargo 1.98.0 / Windows 实测）**：锁文件在**profile级**——`target/<profile>/.cargo-lock`（另有 `.cargo-build-lock`、`.cargo-artifact-lock`，均0字节）；6.3s真实构建期间第二个 `cargo build` 打印 `Blocking waiting for file lock on build directory` 并阻塞4.78s至锁释放后 rc=0——真实互斥已证（"能打开锁文件"≠互斥）。**设计修正**：增量apply须持有受影响根的全部profile级锁（debug单元→`target/debug/.cargo-lock`），不存在target根级单一锁。增量内部布局=`incremental/<crate>-<不透明hash>/` 会话目录（无法按crate猜过期，整单元回收设计成立）。`CACHEDIR.TAG` 在target根。环境：`RUSTC_WRAPPER=sccache` 全局生效。

**各owner必需组/入口（T-04重放用）**：lang主=`cargo t infra-fixed`（python scripts套件，T-03门禁）+`cargo t daily`（默认固定日常：nextest -p auto-lang -p auto-lang-core --lib）；man=其 `cargo t` 别名（全workspace库+auto bin，CLAUDE.md）；ACE=`cargo t ace`（T-04时以 `cargo t --list` 复核选择）；生成根f3f98a9e=按context.json经auto-generation-core owner入口重放（字段在baseline.json）。

**水位初值（临时，待T-04恢复成本收敛）**：high=各根2026-11-10基线分配量（lang 11.749 / man 17.524 / ACE 8.693 / gen 2.036 GiB，即"已见稳态上限"）；low暂不设——恢复成本未测前不配置回收下限或硬上限，policy未配置时只盘点（§5既定）。无预设必须释放GiB。

### T-04 回填（2026-11-10 实际回收与恢复计量；原件在 scope t04/）

**Policy/快照/apply**：t04/policy.json（四 owner foreign 登记+T-02 基线水位）；preview=9 单元全 eligible-idle、0 拒绝、0 跨单元硬链接（lang-main 12.789 GiB/30,133 files——含 T-02 事故构建回长；ACE debug 8.693；man 17.524；gen 2.036）；**apply rc=0，9/9 删除 0 失败，39s**，分配量估计 41.042 GiB、可回收上界（保守）38.609 GiB、锁=6 profile 目录三锁文件；**实际释放（disk free 交叉验证）138.6→177.2 GiB = +41.487 GiB**；deps/exe/pdb 完好；retention registry 以主检出副本快照入 worktree（sha f780ae39…一致，gitignored）后通过 fail-closed 读取。

**必需工作集恢复（V-06）**：

| 入口 | 结果 | 耗时 | 扰动注记 |
|---|---|---|---|
| lang `cargo t infra-fixed` | PASS | 4.5s | 无改动复建（全 fresh） |
| lang `cargo t daily` | **HEAD 红（PLAN-901 ACE 泄漏守卫）**：`auto_lang_core --lib` 源内 81 个 ACE 前缀测试（ac_av/abi::/ac_a2r_hir…）触发 legacy 域拒绝 | 56.8s（发现即拒） | **预存源状态红，与本包无关**（HEAD 003fa8816 即如此）；构建/发现本身正常 |
| lang 代表编辑重编（touch auto-lang-core/lib.rs→`cargo test -p auto-lang-core --lib --no-run`） | rc=0，git 干净 | 65.1s（cargo 报 1m04s） | 与 lang-1000 会话 ace-m1 构建并发=时间上界 |
| ACE `cargo t ace`（首跑） | 构建真实执行（增量回长 2.570 GiB）；runner 结果记录不完整（骨架无终局）→ 首跑判定不可判定 | 1092.2s（≈18.2min，含全 workspace 特性重建） | 与 lang-1000 会话并发=上界 |
| ACE `cargo t ace`（安静窗口干净补跑） | **ace-adapters PASS / ace-batch PASS / ace-workspace FAIL（2831/2838，7红68skip，1044s 测试腿）**；7红隔离重放：**2绿**（rt_certify_modes_matrix、r3_f01_concurrent_staging_budget_is_atomic=并发/预算类环境归因）；**5持续红=源内容域**：certify_dry_run_b01+a2r_hir_first_case_green（golden drift in a2x_runtime.rs，CRLF类）+a2r_hir_worker_credential_pin、abi_recipe_binding_exclusivity（unknown ABI leg "z"）、av_source_inventory_denominators_replayable（015-notes/app.at scan-error）——产物由未变源码确定性再生成、git跟踪代码零改动，**与本包缓存回收无关（预存 M1 域红，PLAN-941/1000 领域）** | 1486.0s 全组；隔离 117.4s（7中5红2绿） | 隔离时机器安静 |
| man `cargo t` | **PASS 283/283（4 skip）** | 188.3s | 同上并发注记；man 检出预存未提交 Cargo.lock 改动（非本包） |
| gen ctx f3f98a9e 重放 | **并入 T-06/T-07 冻结入口**（同为 UI 002-counter 样本集），T-04 记 absent，稳态回长届时计量 | — | — |

**稳态（replays 后，机器安静时测）**：四根增量 41.042→**8.629 GiB**（lang 3.826/ACE 2.570/man 2.233/gen 0）；全状态回长 23.57 GiB（含 deps/产物/增量）；**净空间 +15.05 GiB**（138.6→153.7 GiB free，gen 未重放）。含义如实：本次为**一次性治理**，增量随必需工作集使用自然回长（编辑驱动），无证据支持"持续净收益"主张。

**并发扰动与预存问题清单（如实）**：① lang-1000 会话（plan 1000 infra-ace-m1 组）与多次计量并发，全部时间标上界；② daily 在 HEAD 预存红（81 泄漏，PLAN-901/941 领域，候选入 KNOWN-DEBT 由 T-08 定夺）；③ fixed-tests 执行租约在 guard 失败/崩溃路径不清理（两次陈旧租约人工核死后清除；候选债务）；④ 主检出 untracked 杂物（run.txt、hello.case.ad）预存非本包；⑤ ace-workspace 5 持续红（源内容域，见上表）=预存 M1 域，候选入 KNOWN-DEBT 由 T-08 定夺。

### T-06 回填（2026-11-10 冻结；原件 t06/freeze.json sha256前16=05b88d0a06f4819d）

**工具（关键纠偏）**：PATH 上的 auto.exe/auto-ui-tool.exe 为 2026-11-05 旧源构建（sha 54124fd4/ecc96200），晚于其后 auto-man/auto-ui-tools/auto-lang 源变更——首轮探针全部误用旧工具（含 MSYS PATH 的 "D:/" 形式条目不参与查找导致"新工具"探针仍解析旧二进制）。**已按 942 先例在 scope 重建**：auto `6757a12f…`（auto-man@347c29f，"-dirty"=其仓预存 Cargo.lock 改动）、auto-ui-tool `1d455b2e…`（auto-ui@8d289105），CARGO_TARGET_DIR=scope（PATH 二进制未动）；auto.exe 经 ToolClient spawn auto-ui-tool，两 exe 同 PATH 冻结。**013-todo 红定性=工具/源偏斜**：旧工具 38 个 E0428/E0425 → 冻结工具下绿（44s 0 错）——非当前生成器缺陷。

**四样本入口（冻结工具下全绿）**：UI=`auto build -r rust -d .`（cwd=样本目录，env AUTO_LANG_ROOT=主检出+AUTO_GENERATED_TARGET_ROOT=scope目标+RUSTC_WRAPPER=空），002-counter rc=0 174s 冷（member `counter-f2eeae29`）、013-todo rc=0 44s（members `todo-2531442b`+`app-013-todo-2531442b-back`）；Web=`auto service --server rust -d . -B <port>`（+AUTO_WEB_SOURCE_ROOT），auto-chat release 4m14s/就绪 265.7s/HTTP 404 于 `/`（member `auto-chat-0eb07c63-back`）、auto-database release 6.3s 暖/就绪 19.1s（member `auto-database-330495cd-back`）。**cargo profile：Web=release [optimized]**（banner 的 "development" 是服务运行配置）；HTTP oracle 路由已冻（chat=/__auto/health/live+/api/contacts；db=/__auto/health/live+/api/db/columns）。锁哈希（sha16）：002 `b6fea600`、013 `7eda88dd`、chat `9b4cf333`、db `e1f825b8`；生成 ws 根均无 [profile] 节。

**臂机制（经 owner 入口实测）**：①入口每次构建**重写根清单**（手工 [profile] 被抹回）→ 注入走 **scope 试验根的 `.cargo/config.toml`**（cwd 祖先，逐臂串行切换并恢复，用户 ws 零改动）；②**B=逐成员覆盖** `[profile.dev.package."<member>"] debug="line-tables-only"`——仅成员重编（33s/1 单元），JSON 单元证明成员 line-tables-only、依赖 debuginfo=2；③**弃用变体**：`[profile.dev] line-tables + package."*" debug=true` 非中性——显式 debug=true 强制 host-only/build 依赖（cargo 默认延迟关闭）开 debuginfo，级联全闭包重编（205s）；④**C=B+`package."*" debug=false`**（全部非成员含 path 依赖）；override 是否连带关 path 依赖增量不预判——T-07 按 incremental/deps/pdb 分类计量，override 因素按其全部真实效果归因。⑤A=无配置（cargo 默认 dev：成员+依赖 debuginfo=2，host-only 依赖延迟关闭）。

**Web 零差计划**：[profile.dev.*] 不作用于 release；T-07 以 B/C 配置下 release 全 fresh（0 重编）实证零差，A 闭包实测；release PDB 质量（MSVC 最小 PDB+build-script/proc-macro 单元，如成员 pdb 7.8MB/共 143 个/1.6GiB target）描述性记录、不归因 dev 调试策略。**调试器缺口**：cdb/windbg/lldb/gdb/llvm-pdbutil/llvm-symbolizer/dumpbin 全缺、rust-lldb shim 拒 MSVC 工具链→V-08 断点/变量记**未验证**；行号栈 oracle 改用 dbghelp（SymLoadModuleEx+SymGetLineFromAddr64 定址生成 Rust 调用点）。

**计量法**：分配量=GetCompressedFileSizeW+file-ID 去重（同 T-02）；闭包=臂 target 根+生成 ws（源+锁）+发布 exe/pdb；分类 incremental/deps_rlib_rmeta/build_dir/pdb/member_sources_ws/other；sccache=逐臂 RUSTC_WRAPPER 置空（等价冷隔离，全局缓存不动，命中/冷暖列示）；误差=无改动复建+一次重复冷臂估 E_delta；每计时运行前查机器安静度（lang-1000 会话干扰如实注记）。探针 target 全删（旧工具 12.6G+冻结工具 22.6G），scope 余 tools-target（冻结工具运行所需）+src 副本，磁盘 157.7 GiB 空闲。

### T-07 回填（2026-11-10 对照执行；原件 t07/comparison.json）

**UI 闭包（分配字节，file-ID 去重，target+两生成ws；002 冷→暖→编辑→恢复→013 加入，逐臂独立冷 target、wrapper 禁用、冻结工具）**：

| 臂 | TOTAL | incremental | deps(rlib/rmeta) | pdb | 冷编s | 编辑s | 恢复s | 013加入s | 暖复s |
|---|---|---|---|---|---|---|---|---|---|
| A | 12,565,376,130（11.71GiB） | 4.337G | 5,946,610,587 | 1.798G | 208.2 | 14.5 | 12.7 | 27.3 | 3.2 |
| B | 11,912,329,628（11.10GiB） | 3.761G | **5,946,610,587（与A逐字节同）** | 1.723G | 197.0 | 14.1 | 12.5 | 18.7 | 3.3 |
| C | 5,503,934,229（5.13GiB） | 1.834G | 2.814G | 0.421G | 144.0 | 7.0 | 6.8 | 12.1 | 2.7 |
| A2(重复) | 12,565,332,738 | 4.337G | 5,946,616,519 | 1.798G | 215.0* | 14.9 | 12.9 | 31.5* | 3.3 |

**误差与差值**：E_delta=|A−A2|=43,392B（0.00035%，A2 起跑时有 5 个外部构建进程=耗时上界注记）；A−B=**653,046,502B（−5.20%）**＝成员 incremental −576.8MB + pdb −74.7MB + deps ±0；B−C=**6,408,395,399B（−53.8%B/−51.0%A）**＝deps rlib −3.13G + incremental −1.93G + pdb −1.30G。两差值均远超 E_delta（>10⁴倍）。

**机制证据**：三臂增量会话数均 31（**package 覆盖不禁用增量**——此前担忧实证否定）；逐单元 JSON：A 全 debuginfo=2，B 成员=line-tables-only/依赖=2，C 成员=line-tables-only/依赖=0；三臂暖复建均 0 重编。**行号 oracle（dbghelp）**：成员调用点三臂均解析 `002-counter-f2eeae29/src/main.rs:379`；成员 PDB 751→715→95MB（交叉视图，含静态链入依赖）；依赖行号能力 A/B 有、C 无（逐单元证据）；**断点/局部变量=未验证**（机器无任何调试器，按计划如实记缺口）。**发布/新鲜度**：rust-render 发布面=target 顶层 exe+pdb（已入闭包分类），项目目录仅 `.am/` 工具状态；**UI 窗口应用行为=未运行**（用户活动桌面无人值守开窗，环境边界如实记），验证由新鲜度+行号+Web HTTP 承担。

**Web 零差实证**：A=chat release 437 单元冷编/就绪 272.3s、db 暖 24.2s；**B 配置 0 重编**（就绪 6.5s，`/__auto/health/live`→200 `{"state":"ready"}`、`/api/contacts`→200 `[]`）；**C 配置 chat/db 均 0 重编**（6.6s/12.1s；db `/api/db/columns`→405=路由存在方法受限）——`[profile.dev.*]` 对 release 闭包零影响实证。

**迁移/回退补测（T-08 复审发现的缺口，2026-11-10 补）**：既有 A 填充 target 切 C=180.5s/595 单元重编，共存峰 **17.01 GiB**（纯A 11.71+新C变体，旧A rlib/pdb 残留至清理——非增量残留不属本计划 T-03 工具管辖，需 cargo clean 或整 target 治理）；既有 C target 回退完整 debug=239.2s/595 单元，共存 **16.00 GiB**；恢复步骤=移除 config 重建（覆盖表随配置消失，不假设自定义 profile 自动恢复，实测 595 单元全量回 A 指纹）。两次起跑时各有外部构建进程，时间为上界。构建期瞬态峰值未采样（cargo target 单调增长，稳态即主口径），如实记录。

**决定（V-08/AC-B2/B3；2026-11-10 用户裁定修订）**：~~维持现状默认~~ → **采用 C（用户裁定）**。裁定理由：大部分 Rust 代码由 AI 编写，一般无需进入调试模式；配置须保留基本行信息使报错栈仍有用——C 的成员 `line-tables-only` 恰好保留**成员行号**（三臂 oracle 均解析 `main.rs:379` 为证）；**依赖帧无行号/类型信息**为裁定已知的接受代价（逐单元 JSON：C 依赖 debuginfo=0）。实测支撑：C 空间 −51.0%（11.71→5.13 GiB）、冷编 −31%（208→144s）、编辑重编 −52%（14.5→7.0s）、暖复建无差、Web 零差、增量会话不消失（31/31/31）。B（仅成员行号级）作为对照保留记录。**本计划合同不包含实施推广**（§1 非目标"推广产品默认调试策略"仍然成立——本计划交付研究+裁定记录；落地为生成器/配置默认的变更属后续独立计划）。

### 规范增量

| delta_id | add/modify/retire | docs/specs目标 | before/after规则 | 理由 | 验收 |
|---|---|---|---|---|---|
| SD-01 | modify | `docs/specs/auto-test-case/build-storage-lifecycle.md` | 仅精确target→保留旧CLI，增加已验证增量单元、快照/锁、容量不足保护及长期只读入口 | 重复治理不误删 | AC-A1..A4 |
| SD-02 | add | `docs/specs/repository-boundaries/generated-debug-policy-study.md`（新） | 无本次对照→冻结方法/字节/成本/调试/误差及**用户裁定=采用C（成员行号级+依赖无debug；理由：AI 代码为主免调试器、保留成员行号使报错栈有用；依赖帧无行号=已知接受代价）**；实施推广不在本计划，落地属后续独立计划 | 沉淀研究实证+裁定 | AC-B1..B3 |

本轮不改canonical Specs。若执行需要改变910 context/flags/freshness合同或跨仓默认profile，应提修订，不以试验便利默改生产合同。

## 6. 测试设计

以下均拟执行，当前未运行。工作/内检/最终独立复审在同一worktree顺序进行，未指定模型或额外会话。删除与多上下文规则需真实竞争/文件系统证明，不能只凭静态fixture通过。

| V-ID / 所有者 | 不变量/有效无效状态 | 入口及区分性oracle | 边界 |
|---|---|---|---|
| V-01 / work | 默认preview；未知owner/恶意manifest/前缀同名拒绝；合法闲置正控 | 扩充`scripts/tests/test_fixed_groups.py`，`cargo t infra-fixed`；零写入/非零拒绝/只删准入单元 | 临时正控不计真实收益 |
| V-02 / work | 活PID/lease、进程不可读、responsefile、source引用、drive-relative、在途及用户持有均拒绝 | 同组负控，保护前后hash/ID相同，不只比错误文本 | 原严格边界不能回退 |
| V-03 / work | 真Cargo锁竞争；预览后新增持有/换路径/加reparse/改树；删除中途失败 | T-02固定真实竞争探针与A入口；拒绝或精确已删/未删收据，无越界，合法正控成功 | 未证明锁协议不能apply |
| V-04 / work | 嵌套、跨scope硬链接、逻辑/分配不同、未知ID、独立build-dir | 同组夹具及Windows实际计量；去重恒等式、外链不能冒称可释放 | 未知不填0，卷空闲非唯一oracle |
| V-05 / work | 重复运行、容量不足、旧target重现 | 新CLI inventory/preview/apply；第二次无候选或失效拒绝并零额外删除，shortfall明确 | 幂等不意味着失效manifest可重用 |
| V-06 / work | 完整必需构建/代表编辑后的真实稳态和成本 | T-02固定的既有完整组/应用owner入口，保命令/计数/退出码/字节/时间及行为 | 空包不能代表36.6GiB；无候选须说明 |
| V-07 / work | B/C只变debug，源码/锁/flags/features闭包一致 | T-06冻结生成/Cargo命令，有效配置、JSON单元和计量manifest比对 | 路径差异明列；不跨dev/release比 |
| V-08 / work | 发布/新鲜度/行为、行号栈/调试能力及完整debug恢复 | UI既有发布发现/行为探针，Web真实HTTP；确定Rust调用点栈/调试器，恢复后复建 | 缺调试器记缺口；字符串存在不等于可调试 |
| V-09 / review | 每条AC/SD、收益/持有/误差可独立重算，查遗漏和变相延后 | 同树冻结diff/计量/命令/本计划，必要反例重放 | 未执行、环境失败/重放、未验证分列 |

A至少完整`cargo t infra-fixed`。生产Rust零改动不补全仓构建；若实现触及生成核心，追加完整`cargo t auto-generation-core generated-delivery-controls`及受影响UI/Web消费者，先回填触面理由。B即使只有试验也跑真实冻结工作集/行为，不能以单测代空间验证。ACE代码不改；回收其缓存后重放对应必需组，T-02写实组名，不一概编全部ACE。

最终冻结验证复用身份未变证据，新增改动/失败才补必要重放；merge不机械再跑矩阵。周期`cargo t all`若到期，沿主检出单实例机制，不作本计划常规门。用例/执行/条件排除及准备/运行/清理耗时分列。

## 7. 验收标准

| AC-ID | owner_stage | 可观察验收 |
|---|---|---|
| AC-A1 | work | V-01/02/04成立，库存含target/build-dir及保护集合；preview零删除，去重和未知项完整 |
| AC-A2 | work | V-03证明实际互斥/变化拒绝；只删manifest内闲置增量，源码/锁/发布件/deps/持有不变；半失败不冒PASS |
| AC-A3 | work | V-05成立，水位有依据，容量不足不越保护，旧CLI不回退，能反复库存/生成新候选 |
| AC-A4 | work | V-06给即时/恢复后账及工作集/编辑耗时，处置或零候选均有依据；无持续收益如实记，不拿调查池当兑现 |
| AC-AG | review | 独立核AC-A1..A4及负控，绑定revision后才进B，不提前合并A |
| AC-B1 | work | V-07来源/锁/工具链/有效配置及臂差异冻结，无冷暖/路径/workspace/增量状态混淆 |
| AC-B2 | work | V-08行为/声明调试能力/回退有记录，全闭包字节、冷暖编辑成本、波动/共存峰值完整 |
| AC-B3 | work | 分别归因成员debug/依赖override，决定由数支持；产品默认/sccache容量不变 |
| AC-R | review | V-09逐条独立核AC/SD/diff，遗漏/扩面无隐瞒，warning/格式及未解决债务具名 |
| AC-M | merge | SD沉淀、摘要折叠、所有实际输出逐项对账，只清本包可删材料，用户持有不动，guard clean后移树 |

## 8. 执行步骤

T-01已执行（2026-11-10）。计划簿记在主检出；实现/验证在`D:/autostack/.wt/lang-1001/auto-lang`（分支`plan-1001-dev`，基面003fa8816，已建）。不建第二同步worktree或链接。scope已登记`D:/autostack/plan-artifacts/plan-1001/`（retention in-flight）。

| 任务 | owner_stage / 依赖 | 路径、动作与交付 | 验证/AC |
|---|---|---|---|
| T-01 ✅ | work / 外部前置 | 核四位号全链修复及读回证据、本r1实施授权、HEAD/Spec/持有及在途重叠；建唯一worktree和scope登记 | ID往返仍1001、不路由100；全部AC前置 |
| T-02 ✅ | work / T-01 | §4各根只读基线及Windows锁/布局可行性探针；核owner停写窗口、完整必需组/入口、初始水位及误差 | 实际命令/根/配置/预算依据回填§5/6；不明根只报告；A1/A2/A4 |
| T-03 ✅ | work / T-02 | 扩展`build_storage.py`与现有`test_fixed_groups.py`，交付库存/分配计量/受限policy与manifest/互斥/失败收据，保旧CLI | V-01..05、完整infra-fixed，回填最终CLI；A1..A3 |
| T-04 ✅ | work / T-03 | 有候选按维护窗口回收；真实工作集恢复/编辑，保不能处理原因和保护hash；无候选照实报告 | V-06，即时/稳态/耗时分列；A4 |
| T-05 ✅ | review / T-04 | 同树独立阶段A检查，未通过有界修复重核，不进B | V-01..06证据及必要反例；AG |
| T-06 ✅ | work / T-05 | 仅scope内冻结UI/Web来源/完整入口、A/B/C差异及等价缓存；不改生成器生产默认 | V-07；B1；入口红单列 |
| T-07 ✅ | work / T-06 | 执行对照及调试恢复，报告字节/误差/峰值/冷暖编辑成本与决定，备SD-02摘要 | V-08；B2/B3；正文留可核数字，原件在scope |
| T-08 ✅ | review / T-07 | 冻结revision/完整职责组及必要消费者并集，逐AC/SD独立审、遗漏/延后/workaround扫描，债务具名入KNOWN-DEBT | V-09；R；B无正收益可完成研究交付 |
| T-09 | merge / T-08 | 正式沉淀/归档；折叠摘要/输出清单，重核各盘路径与retention，只清本包可删scope，guard后移树/分支 | M；现有外部持有保持，零遗漏才报清完 |

下一步是T-01。T-02若锁/布局证明失败可先交只读库存并记阻塞，不能勾A2、宣告A完成或进入B。容量数由测量收敛，不以时间流逝代实施授权。

## 9. 复审记录

2026-11-10：仅规划自检，**未执行独立review**。核了阶段顺序、对照因素、分配量/硬链接/峰值口径、前次收益排除、用户持有、清理器现有边界及固定组来源；所有V/AC/T待执行，0/9。文档自检不是阶段A验收或产品验证。

stage: work | PLAN-1001 | r1 | blocked（T-01 四位号前置不成立） | worktree 未建 | 分支未建 | base 003fa8816 | code 无 | 依赖 auto-musk main@721ecfa | T-01 | 证据：auto-musk 三处三位截断正则全部在位未修——`backend/crates/musk/src/relay/plan_control.rs:94`（`PLAN-([0-9]{1,3})`）、`backend/crates/musk/src/server.rs:1031`（`/auto-plan:merge` 入口 `\d{1,3}`）、`server.rs:1050`（`@plan/(\d{1,3})`）；git branch -a 无修复分支、worktree list 无修复树、origin/main==main==721ecfa 无未拉取修复（fetch --dry-run 空）；T-01 oracle「ID往返仍1001、不路由100」当前不成立。其余 T-01 核查项通过：HEAD=003fa8816 与 §4 一致；§4 五个 SHA256 绑定逐一重算全部匹配（191c22f8…/10cab2b1…/f8f70f62…/cf028980…/b2f6f6b9…）；用户持有 `D:/autostack/plan-artifacts/fix-cache-governance-20261110` 四份材料在盘（apply/cleanup/owner/preview），保护不变——差异：`.retention/policy.json` 六个 scope 及 holds 均未列该目录，owner.json 内 status 仍为 in-flight，与 §4「registry 状态 cleanup-completed-audit-held-by-user」陈述不符，簿记差异已记录不改保护行为；在途 lang-1000（codex/plan-1000-dev）/lang-955 无同合同重叠，其构建按 §4 受保护；`.next-id`=1002 与编号陈述一致；r1 实施授权=用户 `/goal 完成计划1001` | 阻塞：§10.1 前置未解除，T-02..T-04 依赖 T-01，无可继续的独立工作 | 下一步：用户决策（修 auto-musk 后续跑 / 有界修订放宽前置 / 暂停）

stage: goal | PLAN-1001 | r1 | round 1 | from: drafting | to: work | status: executing | outcome: blocked | worktree: 未建 | commit: 003fa8816 | next: user（三位路由前置决策）

T-01（2026-11-10 完成）：用户裁定解除路径①（授权先以独立 fix 任务修 auto-musk）。**独立 fix 任务 fix-planid-4digit 已落地 auto-musk main**：commit `ccc7629`（ff 合回，721ecfa→ccc7629，guard clean 后 worktree/分支/组目录全清）——实际修复 **5 处**（比 §10.1 已知 3 处多 2 处发现侧）：①`relay/plan_control.rs:94` parse_plan_id `PLAN-([0-9]{1,3})`→`PLAN-([0-9]+)`；②`server.rs:1031` /auto-plan:merge 入口 `\d{1,3}`→`\d+`；③`server.rs:1050` `@plan/(\d{1,3})`→`@plan/(\d+)`；④`plans.rs` PlanFile::from_content seq 由 `filename.get(..3)` 改数字前缀推导；⑤`plans.rs` scan_dir 恰「3位数字+'-'」过滤器改「非空数字前缀+'-'」（1001-*.md 原被整文件跳过）。新增 6 处四位号钉子测试（parse_plan_id/merge指令/@plan提及/bootstrap 端到端/发现+读取×2），超 u32 优雅拒绝；create 的 999 分配阻断（PLAN-096 AC-16）按独立策略不动。**读回证据（落地 main@ccc7629 上执行）**：`cargo test --lib -- plans::tests relay::plan_control::tests parse_plan_merge resolve_chat_mention` = **52 passed / 0 failed**（worktree 内同集 52/52，含全部四位号钉子）——T-01 oracle「ID往返仍1001、不路由100」成立。触及文件零新增 warning（258 个为仓库存量）、零 fmt 偏差（三文件 fmt-diff 数 worktree==main==0）。其余 T-01 核查项见上条 blocked 收据（HEAD/SHA256 5/5、持有、在途、.next-id）。worktree `D:/autostack/.wt/lang-1001/auto-lang`（plan-1001-dev@003fa8816）与 scope `D:/autostack/plan-artifacts/plan-1001/`（retention 已登记 in-flight）建成。current_step=1，下一任务 T-02。

T-02（2026-11-10 完成，全只读）：实测回填已写入 §5「T-02 回填」小节。要点：四根+生成根基线合计≈40.0 GiB/约104k文件（lang 11.749 / man 17.524 / ACE 8.693 / gen 2.036 GiB），跨scope硬链接0、分配=逻辑（卷未压缩）、unknown 0、基线时无构建进程；锁实测=profile级 `target/<profile>/.cargo-lock` 三锁文件，6.3s真实构建下第二构建阻塞4.78s并打印 `Blocking waiting for file lock on build directory`（互斥已证，非"能打开文件"级）；增量布局=`<crate>-<hash>` 会话目录（整单元回收设计成立）；各owner必需组入口已定（lang=infra-fixed+daily / man=cargo t / ACE=cargo t ace / 生成根=context.json owner重放）；水位=临时high基线值、low不设，T-04后收敛。**偏差记录**：锁探针首次shell后台化失误在主检出意外触发全workspace debug构建（73s，sccache暖）——lang主target基线已过期，T-04 apply前重测；该事故同时构成"全量debug重建成本"真实样本。证据（scope t02/，sha256前16）：baseline_inventory.py `5ada03b56d7ff5d2`、baseline.json `71ee38a1ba924954`、lock_probe.sh `2ad127a238f69869`、build2.log `de92bd98d7c3c536`、build1.log `04e72d20e5d538ab`。current_step=2，下一任务 T-03。

T-03（2026-11-10 完成）：实现落 worktree `D:/autostack/.wt/lang-1001/auto-lang` commit `67ab372d9`（2 文件 +989/-2），最终 CLI/schema/错误码已回填 §5。验证：`python scripts/tests/test_fixed_groups.py` = **59/59 OK**（37 存量+22 新增；存量全绿=旧CLI兼容证明）；**完整 `cargo t infra-fixed` PASS**（worktree 内，results 落 target/test-groups/run-*）。V 覆盖：V-01 零写入正控/未登记foreign拒绝/篡改manifest拒绝/policy摘要绑定；V-02 活租约拒绝+死租约stale/证据与reparse拒绝/retention持有阻止+registry缺失fail-closed/无--idle-confirmed拒绝；V-03 真实msvcrt锁竞争整体拒绝零删除/预览后身份变化拒绝/部分失败收据(rc=3,已删/未删分列)/happy path只删单元不动deps与锁文件且锁释放；V-04 嵌套根唯一归属+test-tools剪枝/跨单元硬链接去重与不可回收上界/unknown降级/triple形单元；V-05 已消费manifest不可重放/水位shortfall=保护量-high/零候选合法/旧CLI输出不变。设计增量（超§5原文、经测试钉住）：foreign生成根链式准入（登记工作区+context.json三核）；保护单元lenient记账保shortfall诚实。真实cargo-vs-cargo阻塞已由T-02证明，工具侧同原语（msvcrt区域锁）拒绝路径由单测证明——组合互斥语义成立，T-04真实apply时复核。current_step=3，下一任务 T-04。

T-04（2026-11-10 完成）：实测与分账全量回填 §5「T-04 回填」。核心数字：**apply 9/9 单元 39s 零失败，实际释放 +41.487 GiB**（disk free 138.6→177.2 交叉验证；保守可回收上界 38.609 GiB 另记）；必需工作集恢复后稳态四根增量 8.629 GiB、全状态回长 23.57 GiB、**净空间 +15.05 GiB（gen 根未重放，并入 T-06/07 计量）**；如实结论=**一次性治理**，增量随编辑驱动自然回长，无"持续净收益"证据。工作集重放：infra-fixed PASS 4.5s；man 283/283 PASS 188.3s；代表编辑重编 65.1s（touch 法，git 干净）；daily=HEAD 预存红（81 ACE 前缀泄漏，PLAN-901 守卫正确拒绝，非本包）；ace 补跑 2831/2838、7红经隔离重放定性=2 环境+5 源内容域预存（golden drift/CRLF 类×3、ABI 合同×1、AV 清单 scan-error×1，PLAN-941/1000 领域）；ace 首跑与 lang-1000 会话并发（时间上界注记），两次陈旧 test-groups 租约人工核死后清除。证据（scope t04/）：policy.json、preview.json（9单元快照+树摘要）、apply.json（收据+锁清单）、ntl-daily.json（daily 81泄漏清单）、ace-replay3.log、ace-isolated-replay.log、man-replay.log、edit-rebuild.log、ace-replay.log/ace-replay2.log（首跑碰撞+不可判定记录）。**阶段A work 任务（T-01..T-04）全部完成**；下一任务 T-05（review 阶段A独立检查）。worktree 内实现 commit 67ab372d9；主检出零代码 WIP。候选债务（T-08 复审定夺入 KNOWN-DEBT）：①daily 81 泄漏预存红；②ace-workspace 5 持续源域红；③fixed-tests 租约失败路径不清理；④主检出 untracked 杂物。

stage: work | PLAN-1001 | r1 | pass（阶段A T-01..T-04 完成） | worktree D:/autostack/.wt/lang-1001/auto-lang | plan-1001-dev | base 003fa8816 | code 67ab372d9 | 依赖 auto-musk main@ccc7629 | T-01..T-04 | §5 T-02/T-04 回填+§9 各收据 | 无阻塞 | next_responsible_role: review（T-05 阶段A独立检查）

stage: goal | PLAN-1001 | r1 | round 2 | from: work(T-01前置受阻) | to: work(T-04完成) | status: executing | outcome: pass | worktree: D:/autostack/.wt/lang-1001/auto-lang | commit: 67ab372d9 | next: review（T-05）

收尾提交（2026-11-10）：worktree commit `163753390`（67ab372d9 之后 +17/-1）——normalize_root 剥 Windows verbatim `\\?\` 前缀（真实 f3f98a9e context.json 的 producer_root 为 verbatim 形式，foreign 生成根链证明原被拒；T-04 真实 apply 即运行此修复后的代码）+ verbatim 形式测试钉子。**验证绑定 163753390**：`python scripts/tests/test_fixed_groups.py` = 59/59 OK；完整 `cargo t infra-fixed` PASS。当前 code identity = `163753390`（基 003fa8816）。

stage: review | review_scope: phase-A（T-05） | reviewer: 同会话独立复核（计划§6明文"未指定模型或额外会话"；结论自工件重构，非采信执行者摘要） | PLAN-1001 | r1 | **phase pass** | worktree D:/autostack/.wt/lang-1001/auto-lang | plan-1001-dev | reviewed 163753390553014a9492e739b59587d7a97abee5 | base 003fa8816 | 依赖 auto-musk main@ccc7629 | spec 输入：§4 五处 SHA256 绑定复核一致（T-01）；阶段A零 canonical Spec 改动 | **AC-A1 pass**（V-01 preview 只读正控 tree_state 前后相等+真实 preview 9单元0拒绝；V-02 活租约/证据/reparse/retention 持有拒绝+37 存量进程域测试全绿=responsefile/drive-relative/source 引用不回退；V-04 嵌套唯一归属/跨单元硬链接去重/unknown 降级不填0/triple 形）；**AC-A2 pass**（V-03 msvcrt 锁竞争拒绝零删除+T-02 真实 cargo 阻塞4.78s+真实 apply locks_held 收据；身份变化拒绝；篡改 manifest 拒绝且 victim 完好；部分失败 rc=3 已删/未删分列；真实 apply 只删 9 个 manifest 单元、deps/exe/pdb/源码完好=git 干净）；**AC-A3 pass**（V-05 已消费 manifest 不可重放/零候选合法/水位 shortfall=保护量-high 不可越保护/旧 CLI 37 存量+专项不变/水位必带依据）；**AC-A4 pass**（即时 +41.487 GiB 磁盘空闲交叉验证 vs 估计 41.042/保守上界 38.609 分列；稳态回长 8.629 GiB、净 +15.05 GiB、gen 并入 T-06/07；工作集/编辑耗时全带扰动注记；"一次性治理/无持续净收益"如实；未拿调查池充兑现） | 阶段A无 pending merge 门（终审冻结并集归 T-08/T-09） | 发现（非阻塞观察，不入门禁）：O-1 find_incremental_units profile 推导含死分支（纯外观）；O-2 --inventory 保护行无 lenient 记账（preview 为权威账路）；O-3 活租约测试断言存在性而非全树哈希（同代码路径只读性已由 V-01 tree_state 钉）；**复审新增负控 NC-1..NC-4 全拒**（policy 未知键 / foreign 非 target 名系统目录准入 / manifest schema 不符 / 水位缺依据）；候选债务转 T-08：daily 81 泄漏预存红、ace-workspace 5 持续源域红、fixed-tests 租约失败路径不清理、主检出 untracked 杂物、retention registry 缺用户持有条目（§4 差异） | 证据：复审者重跑 59/59 OK+完整 infra-fixed PASS（绑定 163753390）+NC-1..4 输出 + §5 T-02/T-04 回填数字与 scope t04/ 原件抽核一致（apply.json 9 删除路径=preview 9 单元逐一致） | **next: T-06（work，阶段B冻结入口）**；阶段A未提前合并，B 闸门打开

stage: goal | PLAN-1001 | r1 | round 3 | from: work(收尾提交163753390) | to: review(T-05 phase pass) | status: executing | outcome: pass(阶段A) | worktree: D:/autostack/.wt/lang-1001/auto-lang | commit: 163753390 | next: work（T-06 阶段B冻结）

T-06（2026-11-10 完成）：冻结全记录=t06/freeze.json（sha16 `05b88d0a06f4819d`），回填 §5「T-06 回填」。核心：**工具陈旧纠偏**（PATH 旧工具→scope 内当前源重建 auto `6757a12f`/auto-ui-tool `1d455b2e`，PATH 二进制未动）；**013-todo 红定性=工具/源偏斜**（旧工具 38 错→冻结工具绿）；**四样本入口全绿**（UI=auto build -r rust；Web=auto service --server rust，cargo profile=release）；**臂机制**=scope 级 .cargo/config.toml 串行切换（入口每次重写根清单→manifest 注入不可行；逐成员 B 经 JSON 单元证明只动成员；package."*"=true 钉因 host 依赖 debuginfo 显式化引发级联而弃用）；锁哈希/成员名/HTTP oracle 路由/工具链/调试器缺口（无调试器→V-08 断点项记未验证，dbghelp 行号 oracle 替代）全冻结。生成器生产默认零改动、用户真实 ws 零改动、全局 sccache 零触碰。**过程偏差如实记**：①一次 `taskkill /IM auto.exe` 按映像名误杀风险（影响面未知，已改为仅自身 PID 树清理）；②MSYS PATH "D:/" 形式不参与查找导致首轮"新工具"探针仍用旧二进制（重探修正）；③B 钉法探索中发现 host 依赖 debuginfo 延迟关闭语义（cargo 默认）——已写入冻结记录防复踩。探针 target 已全删（释放 ~35G），scope=t06/{src 副本, tools-target, freeze.json, 各探针日志}。current_step=6，下一任务 T-07。

stage: goal | PLAN-1001 | r1 | round 4 | from: work(T-06 工具纠偏+冻结) | to: work(T-06 完成) | status: executing | outcome: pass | worktree: D:/autostack/.wt/lang-1001/auto-lang | commit: 163753390（阶段B无代码改动，纯 scope 试验） | next: work（T-07 对照执行）

T-07（2026-11-10 完成）：对照全记录=t07/comparison.json，数字与决定回填 §5「T-07 回填」。核心：**A 11.71 / B 11.10 / C 5.13 GiB**（E_delta=43KB=0.00035%；A−B=−5.20%=成员incremental+pdb、deps 逐字节不变；B−C=−51.0%A=deps rlib/incremental/pdb 三类）；三臂增量会话 31/31/31（覆盖不禁增量）；逐单元 JSON 钉死各臂 debuginfo；成员行号 `main.rs:379` 三臂可解析、依赖行号 C 失（设计使然）；成员 PDB 751→715→95MB；断点/局部变量=未验证（无调试器）、UI 窗口行为=未运行（活动桌面环境边界）——两项缺口如实记录；**Web 零差实证**（B/C 配置 0 重编+真实路由 200/405）；**决定=维持现状默认**（B 可选收益有限且安全；C 空间−51% 但剥离产品自有 crate 依赖调试信息=真实能力代价；产品默认不变）。计时扰动如实注记（A2 与外部 5 构建进程并发）。scope 保留 arm targets/web target 供 T-08 复核，T-09 收尾整删。current_step=7，下一任务 T-08（终审）。

stage: goal | PLAN-1001 | r1 | round 5 | from: work(T-07 对照执行) | to: work(T-07 完成) | status: executing | outcome: pass | worktree: D:/autostack/.wt/lang-1001/auto-lang | commit: 163753390 | next: review（T-08 终审）

stage: review | review_scope: final | reviewer: 同会话独立复核（计划§6既定单上下文；自工件/原始 json 重算，不采信执行摘要） | PLAN-1001 | r1 | **pass → reviewed** | worktree D:/autostack/.wt/lang-1001/auto-lang | plan-1001-dev | reviewed 163753390（自T-05后代码身份未变，冻结证据按规则复用） | base 003fa8816 | 依赖 auto-musk main@ccc7629 | spec 输入：SD-01 目标 `docs/specs/auto-test-case/build-storage-lifecycle.md` sha16 `191c22f8c59b4e44` 复核未变；GOAL-003/016 存在性核过 | **AC 全表**：A1/A2/A3/A4/AG pass（T-05 phase 复核，绑定同 commit 163753390）；B1 pass（freeze.json：来源五仓 commit+工具 sha+锁哈希+工具链+成员名+逐单元 JSON；臂间差异仅 debug 因素——A vs B deps 类 5,946,610,587 逐字节相同；同路径同锁同 wrapper 禁用；增量会话 31/31/31）；B2 pass（**复审发现缺口 F-1=迁移/回退/共存峰未测→当轮补测闭**：A→C 180.5s/595 单元/共存 17.01 GiB、C→A 239.2s/595/16.00 GiB、恢复=删配置重建实测；Web 真实 HTTP 路由 200/405；新鲜度三臂暖复 0 重编；E_delta=43,392B 可独立重算）；B3 pass（归因=成员/依赖两因素分离成立；**决定=采用C（2026-11-10 用户裁定，理由与已知代价入 §5/§10/SD-02）**；本计划未改产品默认/未动 sccache——落地属后续独立计划 D-1001-R7）；R=本复审 | findings（复审中）：F-1 迁移/回退缺口（已补测闭）；F-2 构建期瞬态峰值未采样（cargo target 单调增长，稳态为主口径，如实记录）；F-3 UI 窗口行为未运行（活动桌面环境边界）；F-4 无调试器断点/变量未验证（D-1001-R6） | 债务具名：**D-1001-R1..R7 已入 KNOWN-DEBT-AND-RISKS.md**（daily 81 泄漏红/ace 5 源域红/租约清理/registry 持有缺登记/PATH 工具陈旧/无调试器/C 落地待续） | 冻结验证复用：`cargo t infra-fixed` PASS@163753390（身份未变）；59/59 测试文件绿@163753390；完整职责组并集=infra-fixed（生产 Rust 零改动，无消费者组扩展必要，触面=scripts/tests+scripts/test-governance） | evidence：§5 T-02/T-04/T-06/T-07 回填数字与 t02/t04/t06/t07 原始 json/log 逐一致（measure json 重算抽核） | **next: merge（T-09）**；AC-M 归 merge 所有

## 10. 待澄清事项

1. ~~已知开工阻塞：auto-musk三位路由须修复并有四位号发现/读取/交接端到端证据~~ **已解除（2026-11-10）**：用户裁定路径①，独立 fix 任务 fix-planid-4digit 落地 auto-musk main@ccc7629（5 处位点修复+6 测试钉子+读回 52/52，详见 §9 T-01 收据）。
2. T-02收敛：owner停写窗口、实际Cargo锁/build-dir、完整必需工作集、水位和重编成本；未知闲置则保护，无预设必须释放GiB。
3. T-06冻结：拆仓真实UI/Web入口、可用调试器/行为oracle。缺工具/基线红记缺口，阻断必要对照须修订，不换小样本自证。
4. 本轮只立计划；默认调试推广、sccache降容、周期自动删除未授权，且不在合同内。**2026-11-10 用户裁定**：阶段B对照完成后**采用C**（成员 line-tables-only+依赖 debug=false；理由=AI 代码为主一般免调试器、保留基本行信息使报错栈有用、依赖帧无行号为已知代价）——该裁定改变 T-07 决定字段与 SD-02 沉淀内容，**不**扩大本计划实施范围（落地推广仍属后续独立计划）。

T-07 补记（裁定修订，2026-11-10）：t07/comparison.json 的 `decision` 字段以本计划文件记录为准——原 `maintain_current_default: true` 被用户裁定取代为 **采用C**；裁定语义与理由已写入 §5 T-07 回填与 SD-02。

stage: merge | PLAN-1001 | r1 | **pass（delivered）** | prepared: reviewed 163753390+SD-01/SD-02 冻结 | landed: master 尖端=交付提交 **762941d5b**（ff-only 无合并提交；rebase no-op——基 003fa8816 未动；4 文件 +1056/-2） | canonical_verified: SD-01 `docs/specs/auto-test-case/build-storage-lifecycle.md` sha16 `edf50afad7d52f28`（modify，+14 行契约段）；SD-02 `docs/specs/repository-boundaries/generated-debug-policy-study.md` sha16 `974c985eb9307837`（add，37 行，含用户裁定采用 C）——均于主分支实测核验 | 冻结门: `cargo t infra-fixed` PASS + test_fixed_groups 59/59 OK **绑定 762941d5b**（文档后代不失效代码证据，复用+重跑双保险） | archived: `docs/plans/archive/1001-…md` status: archived，簿记提交 **130977239**（计划文件+KNOWN-DEBT D-1001-R1..R7+.next-id 入库；T 计数 8/8 work+review+merge 一致） | cleaned: wt-guard clean ×2 → worktree `D:/autostack/.wt/lang-1001/auto-lang` 移除、分支 plan-1001-dev 删除、组目录已删；**scope `D:/autostack/plan-artifacts/plan-1001/`（72G）折叠后整删**，retention registry 刷新=deleted-at-merge，无具名持有 | 部署检查: 落地=scripts/docs only（治理工具+规范），无生产二进制/Web bundle 消费此变更——无陈旧部署面 | pending（非本计划门）: 批量回归到期（收据 last_covered=995@2026-11-08，48h+其后 941/1001 合并；PLAN-896 裁定不构成每计划全局门，交周期单实例机制） | 落地后续: D-1001-R7（采用 C 的实施计划待立）

stage: goal | PLAN-1001 | r1 | round 7 | from: review(T-08) | to: merge(T-09) | status: **archived** | outcome: pass | worktree: 已移除 | commit: 762941d5b（delivery）/130977239（bookkeeping） | next: done
