# PLAN-828 第二批 T-05＋T-06 独立复审

2026-10-22；`review_scope: phase`；**needs_fix：2 项 P1、2 项 P2**。第一批既有关闭结论保持；本收据不授整份 reviewed/merge。

## 冻结身份与范围

- Plan r3，executing 0/11；当前主检出 Plan/接口文档冻结为 `plan-input.md`、`interface-input.md`，含 SD-01..06；不以工作树的旧 Plan 为准。
- 审核树 `D:/autostack/.wt/lang-828/auto-lang`，`plan-828-dev@a66d9d9e54e4db1de90edc078ed7d23d161e00d5`，测试期间 clean、HEAD 未变。写收据前一次复核514项一致；**写收据后最终复核检测到并发WIP**，见下述限制，当前树不能称clean。
- 第二批原实施 `168b6b56586acbad4521633a329739a8b2c6f58f`，diff base `6f482efafdfbb712dbc65c503de7153e8c2edf85`。当前冻结树包括此后第一批修复；**运行证据绑定当前 a66，而非追认历史 168 的 PASS**。完整原五文件 diff 见 `second-batch-original.diff`。
- auto-down `c39ac7f9dacdc769fc4a5ad3b98ec896240b28e8`，本 probe 未消费；514 项源码、配置、锁文件、Specs/Design43、合同、技能输入的哈希见 `baseline.json`。最终 `final-state.json` 为 `matches_baseline=false`：只有 session.rs 的输入hash变化，状态另新增未跟踪 T-07 测试；依赖/config/Specs/两个core源码未变。
- reviewer：本 Codex 审核上下文，未实施本批代码，按 `/auto-plan:review` 从代码和独立输入重建结论；无新代理/任务，未编造模型或会话身份。未修改实施源码、canonical Specs 或 ledger。

**最终漂移边界**：并发 `session.rs` 增加 `PreparedEdit::code_weak` 与 `resident_weak` 观测接口，同时新增 `function_incremental_828_t07.rs`，均未提交；原差异和状态见 `concurrent-wip.diff` / `concurrent-wip-status.log`，最终检测失败原样保留在 `final-freeze.log`。未覆盖这些修改、未重置树。测试、私有consumer投影与四项 finding 只绑定测试时 clean 的 a66 冻结，不能给当前WIP授审核结论；本次也不对T-07进行验收。下一冻结提交须包含/处置实际全部修改后复审受影响面。之前一次匹配成功不能替代最后一次失败。

## Findings

### B2-C01 / P1：链接只按名字和签名选中异 owner 的函数

**原规则**：V-03 的同名异 owner / 错上下文必须拒绝或正确扩大集；V-04、C02-04 的受检链接不能仅凭“已准入”接受当前上下文错误的单元。当前单模块支持面无需新增多模块能力。

**输入/入口**：`link-controls.rs::link_foreign_owner_same_name_is_refused`。冷源 helper=7、main=helper()+2；把真实 helper 单元的 owner.module 改为 `different-owner`、常量 7 改为 41，经真实 encode/decode/verify_abc（使用其合法 foreign expected owner）/admit_unit，再交公开 `ProgramSnapshot::link`。这不是伪造 LoadedModule 或绕过 ABC 准入。

**期望/观察**：同名同签名不能冒充本模块 helper，链接应拒绝；实际 link Ok，AV 成功返回 **43**。同 owner、相同常量替换的正控 link/普通/分步恢复均正确返回43；普通冷源返回9。失败不来自常量、签名或编码非法。

**已证实原因**：`core/src/program_snapshot.rs:119–140` 按 `c.name()==target` 选择 callee，仅核结构签名，未核 owner/context。ABC 准入证明单元对其自身 owner 合法，不证明属于该 snapshot 的调用上下文。

**关闭 oracle / 边界**：保持合法同 owner 替换与递归；在真实受检链接/安装 consumer 核对既定 owner/context，foreign 输入安装/派发为0、旧根/来源不变。不得靠测试先过滤、公共自证字符串或扩大到跨模块语言功能关闭。此项是当前原规则缺口，不声称已定位为第二批首次引入的回归。

### B2-C02 / P1：已准入但未绑定的实际调用可进入 snapshot，并先产生副作用

**原规则**：V-04 的好 payload＋坏 import/context 在准入前 dispatch/install=0；C02-04 安装前拒绝，C02-03 三入口共用派发。受控 Infra 发生在 guest 指令之后不能满足此门。

**输入/入口**：`link_unresolved_called_import_refuses_before_dispatch`。合法 `main=mark_a()+helper()`、trace cap 已授权；只把 main 的真实 `fn:d_helper` import 改成已登记的 `hir.test.mark_b`，effect=Observable，保持 ()→i32 签名。真实 ABC 验证/单元准入成功；调用指令仍引用该 import。

**期望/观察**：未闭合调用在 checked link 被拒，或经合法机制闭合后才允许运行。实际 link Ok，`unit_binding_complete()` 已返回 `import #0 hir.test.mark_b is unbound`，执行却继续：普通和 step(1)→resume(None) 均先留下 **[MarkA]**，再返回 `Infra("callee resolution failed in func #1: Import(0)")`。合法未经篡改的 Trace 正控返回8、绑定完备检查通过。

**已证实原因**：`program_snapshot.rs:140–143` 对非 `fn:` import 存 None；可执行 snapshot 的构造与执行前没有核实际调用闭包，只有运行到 Call 时 resolve_callee 失败。`unit_binding_complete` 的存在不等于生产准入已消费它。

**关闭 oracle / 边界**：实际使用的 import 必须执行前闭合；可拒绝不支持的已登记 intrinsic Call 或按原语义解析，但不误拒已合法 lower 成 Trace 的来源程序。三个入口均不能先执行 Trace 再报绑定错误；已有缺函数/错签名、直接执行 unit 的拒绝保持。优先核真实 call 引用与允许的 metadata，勿用一律拒绝合法 intrinsic metadata 的补丁替代诊断。

### B2-C03 / P2：代码槽重排后，以旧 func 编号选中别的函数的来源图

**原规则**：C02-02 稳定单元/局部坐标，C02-03 三入口来源正确，F-04 不变。`link_with_maps` 明确 maps 按 code slot 配对；旧 unit payload 必须保持不可变。

**输入/入口**：两个控件覆盖同一根因：`link_reordered_units_use_their_own_span_maps` 和 `link_reordered_after_real_edit_keeps_current_trap_span`。后者是真实源码 main/helper/tail，helper 除零前经 session.edit 插入5空格；证明 `reuse_hit:helper=1`，取该 snapshot 原有 code 与真实 span_maps 一起交换 main/helper 槽位再公开 relink。

**期望/观察**：真实源编辑正常 snapshot 的 trap 为 **64..69**；同 code/maps 配对重排后却变为 **33..33**。独立偏移图控制中 helper 应映射 **125..130**，普通和 step→resume 都误用 main 图，得到 **225..230**。合法六种代码排列的值32均通过，值正确不能证明来源正确。

**已证实原因**：`av.rs:160–181` 用 payload 中生产时的 `SpanInfo.func` 索引 snapshot 的 **per-code-slot** maps；UnitView 真实执行槽可变化，raw span.func 不随槽变化。`span_of` 原样返回持久 payload span；身份保持与诊断重定位需要在执行/snapshot view 接缝协调，不能重写共享单元。

**关闭 oracle / 边界**：在实际执行的函数身份/槽与该 snapshot 的图之间正确投影，明确结果 func 轴的既定语义；真实编辑的 start/end 必须64..69，普通/分步/resume、旧 pin 的来源均一致。此例不主张任意伪造 span_map 可被信任；使用的最终反例是生产 session 生成的原图。

### B2-C04 / P2：正数有限 resume 预算被当成累计阈值，恢复永不前进

**原规则**：T-06 / C02-03 的三执行入口语义一致；公开 resume 文档明确“another step budget”。步骤切片和全局 fuel 是两种限额，不能混用。

**输入/入口**：`link_repeated_finite_resume_budget_makes_progress`。普通执行只需 **6** guest steps；execute_program_steps(...,1)，随后至多16次 resume(...,Some(1))，有界重放不使用 sleep 或无限循环。

**期望/观察**：足够多的额外一步切片应完成；实际16次后仍 Paused，随后 resume(None) 才完成，最终累计仍 **6** 步，值9。说明这些有限恢复未增加指令进度。

**已证实原因**：`av.rs:425–430` 每次比较累计 `st.steps >= budget`，resume 原样传入新的切片预算。第一次执行后 steps=1，此后 Some(1) 立即返回。

**关闭 oracle / 边界**：按每段额外预算推进，同时保留跨段累计 hard fuel/frames/slot/trace；Some(0) 合理暂停、连续正预算完成、全局 fuel 仍能耗尽。此缺陷早于本批存在，但属于本次更换执行器后的完整 T-06 原要求；不据既存性质豁免，也不扩展取消时延要求。

## 已通过的关键路径与覆盖表

| 合同行 | 本轮结论及证据 |
|---|---|
| C02-01 实际不可变执行 payload | 已测代表面 PASS：执行器 UnitView 直接 fetch 单元；真实 helper 增槽/增长后未改 main 的 MirFunction 地址与 Arc 保持，new=33/old=32 |
| C02-02 局部表/顺序/绑定 | 局部 const/type/slot、i32/bool、命名实参、6种代码次序值均 PASS；完整行受 B2-C01/C02 的受检上下文缺口及 C03 来源问题阻断 |
| C02-03 三入口/旧链 | 普通及 step→resume(None) 一致；安装后旧链仍32/新33；自递归与互递归旧链3/新4，trace相等，PASS。有限切片和重排来源 FAIL（C04/C03） |
| C02-04 失败原子性 | 实际缺函数/重复名/错签名/缺入口拒绝 PASS；候选 foreign/stale/closed 既有定向断言 PASS；**真正 session.commit linkErr** 的旧根/源码/环境代际/全 records/history/deps/origin/map/canonical/mint 图像与 trap 全等，随后此前 good 候选仍可提交，PASS；owner/调用闭包准入子轴 FAIL（C01/C02） |
| C02-05 实际副本负控 | PASS：复制未改 main 的执行 payload、真实 relink 并执行，值仍33；same_object=false、same_content=true，原身份轴红；非只复制闲置包装 |
| V-03/04 | 当前 owner/调用闭包 FAIL；合法类型/命名实参/递归与直接执行 unit 拒绝 PASS。未重新声称完成整份精确依赖计划 |
| C05-01..04 / F-05b | 已测关闭/取消/迟到/资源释放/非零 revision vacant 与0/1/10步安全入口的原6＋3控件均 PASS；第一批 permanent close/query断言通过，保留范围不扩大到完整store GC/预算 |

**session.commit 真失败分支如何证明**：`generate-session-controls.py` 从冻结 session.rs 仅将开头 `//!` 文档注释改为 `//` 以允许 include；函数体完全原样，wrapper 在私有组件内加入测试。先准备 good 候选，再准备不同长度 bad 候选，把其 code 换成真实准入但含 `fn:missing_target` 的 unit，调用生产 commit。`session-consumer-input.json` 记录生产原件/投影 hash，`session-injections.rs` 保留注入点和全状态图像。该方式是合同允许的组件内真实 consumer 测试，没有改 WT 或新增不可信公共构造口。观察 revision/env generation=0、same_root=true、same_records=true、trap 70..75 全等；good 随后成功。不能把此项写成“原子性全部失败”。

## 执行证据与复现

在原实施树执行定向构建（非 root 整档）：

```powershell
cargo test --locked --manifest-path experimental/ac-av-probe/Cargo.toml --test function_incremental_828_b1 --test function_incremental_828 --test function_incremental_828_contract --test function_incremental_828_r1 --test function_incremental_828_contract_t01r --no-run --message-format=json
```

Cargo 返回的精确库/二进制用于审核，`cargo-artifacts.jsonl`、`target-results.json` 保留路径/hash/args/exit；没有用随机同名 rlib。控件从本目录运行：

```powershell
powershell -NoProfile -File ./run-controls.ps1 -Source link-controls.rs
powershell -NoProfile -File ./run-controls.ps1 -Source execution-controls.rs
python ./generate-session-controls.py
powershell -NoProfile -File ./run-controls.ps1 -Source session-consumer-controls.rs -AllLibraries
powershell -NoProfile -File ./run-controls.ps1 -Source close-replay.rs -Filter close_review_
powershell -NoProfile -File ./run-controls.ps1 -Source close-replay.rs -Filter close_adjacent_
powershell -NoProfile -File ./run-targets.ps1
```

`run-controls.ps1` 的 AllLibraries 实际只加该私有 consumer 所需的明确直接依赖，并从 Cargo artifact 设置该进程的 manifest dir；rustc/测试真实失败会向外传播。link 控件当前预期 **exit101**；这是 needs_fix 证据，不能包装成 ALL-PASS。

| 实际选择 | 结果 |
|---|---|
| 新 link 控件 | **3 pass / 5 fail**，8 tests，5失败对应4 findings；exit101 |
| 新 execution 控件 | 4/4（内含六排列、自递归/互递归等子场景） |
| 真 session consumer link 失败控制 | 1/1 |
| r23a close / adjacent 原控件 | 6/6、3/3（其余 include 的测试由 filter 明确未选） |
| permanent b1 / session / contract / t01r / r1 | 11/11、18/18、22/22、4/4、86/86 |
| auto-av-core 包内 lib 定向 | 22/22 |
| 原批五个 Rust 文件 rustfmt / diff --check | 0 / 0 |

session 明确跳过两个第三批512负载；未把筛掉测试计入通过。上述集合有重叠，不加总宣传独立覆盖数。新 consumer 的8个 unused/dead-code warning 来源于审核投影单独编译，不是新增生产告警；Cargo warm 构建无新 warning 输出，但本轮不宣称进行了全仓 clippy/告警清零。

## 审核探针修正记录

- 初版 unknown `host:not-registered` import 被 ABC verifier 正确拒绝，未到链接：改成已登记、合法 cap 的 mark_b 后才成立最终 C02；旧 raw 保留，前者不是产品 finding。
- 初版正常控制和来源探针用了无限循环 resume(Some(1))，因预算问题挂住。核对并终止的仅是本审核拥有的精确临时 exe；此为审核器有界性错误，现改 step→resume(None)，把有限预算问题独立为最多16次的断言。旧 `link-controls-initial-unbounded.log` 保留，最终结果只采有界版。
- 私有 consumer 初次装配引入两版 foldhash、缺 manifest env；改用 Cargo 明确直接库和真实 manifest dir。补测三入口时曾错包装成 Arc<&Snapshot>，格式检查曾漏 probe 子目录；均属审核装配错误，初始日志保留，更正后实际编译/选择/退出可查，未归为产品缺陷。

## 规范增量与下一责任

SD-03/04/05 的方向（不可变单元、真实受检链接、snapshot寿命）与现行 Spec/原合同一致，但**现实现不能支持其完整已实现声明**：owner/未闭合调用/来源/有限resume仍失败。SD-01/02第一批已审子轴保持，SD-06正式worker/IPC及完整delta落文仍后续 T-08/09。影响元数据现有路径/GOAL-002保留，无需以放宽准入或改变预算语义修订 r3。

下一责任为 **work，在同树一批修 B2-C01..04，冻结后复审受影响面**。根因、路径和关闭条件见 [GLM 下一轮任务单](glm-next-round.md)。不启动第三批全面复审来替代这四项关闭。executing 0/11；后续工作、整份最终验收，以及最终 PASS 后 merge-owned 冻结裸 `cargo t`＋VM/compiler触面 `cargo tv`/必要consumer与收尾门仍 pending。本轮未运行整档、docs_gen、merge、归档或清树。
