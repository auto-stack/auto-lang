# PLAN-815 T-05/T-06 R4 修复轮交接（2026-10-22）

修复提交 `8a08e84fe25ec81e164ba50b9f9e970e705505e0`（code），交接稿/
证据随后续 docs 提交（不改代码，二进制身份不变）。审查对象=复审报告
（review.md）十项；逐项 closure 行：finding → invariant → 修复与永久
反例 → 当期结果 → 剩余边界。全部回放产物在 `fix-evidence/`。

## 逐项关闭表

| Finding | 不变量 | 修复与永久反例 | 当期结果 | 剩余边界 |
|---|---|---|---|---|
| F-815-T05-R1-01（非Windows清理） | reader 在声明支持平台可真停/真实结束/有界错误 | Unix `poll(2)` 可用性探测（10ms 刻，与 PeekNamedPipe 同构）替换阻塞 `read_until`；`join_readers` 两阶段=2s 自然收尾窗（正常完成不误分类）+强制停窗；**截止后不再无条件 join**（reader 与 input writer `uninstall` 侧双侧）；`#[cfg(not(windows))]` 真停测试：`sh -c 'sleep 30 & exit 0'` 非 owned 持管道→5s 内真停+`Stopped` 具名；echo 正控 `NaturalEof` | Windows 侧全绿（真停/分类/有界由既有 held-reader oracle 复验）；Unix 编译面入树 | **Unix 真停测试未在本机执行**（本环境无 Linux；不伪称实跑闭合）——待声明支持平台（Linux CI/复审环境）执行；实现与测试均 cfg(unix) 在树 |
| F-815-T05-R1-07（consumer 3/5） | 消费夹具断言与真实能力面一致；负控瞄真实不服务目标 | 描述符断言：build_ui=cancellable+non-interactive、run_ui=cancellable+interactive（正控保留）；零副作用负控 generate→`vm`（真实不服务：vm 仅 run 面服务）+拒绝消息含服务集披露 | `cargo test --offline --locked` 独立 consumer **5/5**（原 3/5） | 无 |
| F-815-T05-R2-02（告警/格式） | 零新增未处理诊断；冻结格式一致 | renderer.rs:18978 `///`→`//`、:18980 死局部删除（18974-78 注释同步改写）；tool_process.rs 长行 rustfmt | 两条新告警清零（行级 grep 复验）；当前树全部净改 `.rs` rustfmt --check 0 diff | merge 冻结时由 gate 统一复核 |
| F-815-T05-R4-01（P1 串行沿用前请求后端） | 每个执行进程只获得本请求上下文 | `ExecutionSpec.env_remove`（设施级新字段，先移除后追加）+`RunIo.vm_session_env`（每请求精确条目）+`TrackedSpawn.without_env`；宿主不再 `set_var` 全局（run_vm 两处根除）；父进程托管形态跳过主题全局应用（子进程 env 同链 compose）；vm-session spawn 恒 `without_env` 两内部键再按请求追加 | 回放 session-context（真实 SDK ABI 后端）：同宿主第 2 请求（external_backend=null + 本地 api.at）呈现 **LOCAL_SECOND_SESSION**（原 R3_EXTERNAL_CONTEXT 污染）；失败后新会话隔离保持 | 无（回放=审查原脚本原样重放） |
| F-815-T05-R4-02（P2 缺DLL仍 completed） | 声明后端的装载失败具名传播、不启依赖会话 | vm child：`prepare_external_backend_from_root` Err→`VM_SESSION_FATAL: <reason>` stderr+专用退出码 3（不跑 entry）；宿主 forwarder 捕获首条 fatal 行→`on_vm_main` 映射 `vm session failed: <reason>`→RunOk failed | 回放 missing-dll：`outcome:failed`、error 含 `cdylib review_backend.dll not found`、presented=False（原 completed/error=None）；成功 DLL 正控保持（serial-external completed） | 无 |
| F-815-T05-R4-03（P2 假输出根） | 回报真实路径与计数、逐 profile 存在且 walk 一致 | `profile_output_dirs`（生成器同源布局常量）：jet=`gen/front/jet`（原 android）、ark=`gen/front/ark`（原 ohos）、vscode=`gen/front/vscode`（原 gen/vscode）、tauri=**双根** `gen/front/vue`+`gen/back/rust`（前后端边界成文）；聚合根=最深公共祖先；永久单测 `generation_layout_roots_match_real_generator_dirs` | 回放 profile-contracts：四 profile `reported_root_exists=True`、files 17/24/25/8 | 无 |
| F-815-T05-R4-04（P2 主pac第二真源） | 新工具生产入口只消费已求值请求 | `VscodeProject::from_workspace_with_request`（name←request.identity、vscode{}←request.raw_nodes）+generate/build/run_vscode_project_with_request；`build_vue_project_with_request`（prepare_vue_sources 消费请求）；jet run 安装显示名请求臂；主 pac 修改/移除不改快照 | 回放 profile-contracts vscode：package.json name/displayName=**request-name**（原 disk-name）；永久串行测试 `generate_ui_vscode_fields_come_from_request_not_disk_pac` | 无 |
| F-815-T05-R4-05（P1 build 子进程无所有权） | 已声明 cancellable 的实际路径统一 owned 树，取消/EOF 真终止 | `proc_host::run_external_status`（托管=owned 树/legacy=冻结 Command 双形态）+`capture_stdout` 捕获排水（adb）；jet wrapper/assembleDebug/installDebug、ark hvigorw、vscode `code` 全接线；vue/vm/rust/tauri 既有托管臂复核在案；附带修复 join 抢停竞态（正常完成误报 teardown） | 回放 build-lifecycle cancel：`child_alive_after_cancel=''`（sleeper 后代真死）、~200ms 终态、describe 响应、EOF exit0、串行下一 build 可用；永久串行测试 `build_ui_cancel_terminates_owned_build_tree`（tasklist PID 级 oracle） | 无 |
| F-815-T05-R4-06（P2 exit7 仍 completed） | 真实失败/缺工具/未真执行=结构化具名 failed | jet/ark：非零退出→具名 Err（含退出码与手动指引）；缺 wrapper/工具→具名 Err（vscode npm 缺失同）；**产物 oracle**：退出 0 但 APK/HAP 缺失→"artifact is missing … did not really run" | 回放 build-lifecycle：exit7→failed 具名带码；exit0 无 APK→failed 产物 oracle；永久串行测试 `build_ui_jet_reports_real_failure_and_missing_artifact` 双例 | 无 |
| F-815-T05-R4-07（P2 Multi 仅首项） | 多目标完整生成+诚实聚合 | `generate_report` 去重全目标循环（声明顺序）；frontends=实际生成集；聚合=去重并集 walk；arkts→ark 规范化（真实 pac `render:"ark"` 声明路径与 wire 路径统一可服务） | 回放 multi-profile：`[vue,jet]` 双根真实（41 文件）、output_root=`gen/front`；永久串行测试 `generate_ui_multi_profile_generates_every_declared_target`（walk=报告计数） | 无 |

## T-08 交接稿缺口（partial→补齐）

`attachments/815/handoff.md` 重写：真实冻结身份（源 SHA `8a08e84fe…` +
debug 二进制 SHA256 `ab88ccb4…` + 支持×profile 指纹矩阵）；Cancel 终态
双通道（RunOk **与** BuildOk）；owned 树声明精确到逐命令清单（托管/legacy
双形态边界成文）；SD-01..**07** 正文（before/after 规则，含 SD-07 runtime
公开面）。

## 当期验证账

- scoped：tool-execution **31/31**（新增 env_remove 正负控）、协议 **19/19**、
  man 兼容/媒体/请求缝 **6/6**、ui-tools lib **171/175**（4 红全为在案预存：
  plan593/shell_pack/merged_api/canonical_double）、tool_process 默认面
  **21/21**+重串行 **8/8**（4 旧+4 新）、proc_host lib **7/7**（Windows 面；
  Unix 真停测试 cfg 裁剪）、独立 consumer **5/5**。
- 回放：R4 四脚本（session-context/profile-contracts/build-lifecycle/
  multi-profile）全闭 + R1-R3 全部原例保持（protocol 三例、rust 双例、
  held-reader、vm-restart、worker-fatal、input-initialization、vm-context
  真实 SDK ABI 外部呈现）。
- 格式/诊断：净改 `.rs` rustfmt --check 全 0 diff；renderer 两条新告警清零；
  ui-tools 告警保持 22 条既有基线。
- 测试时序裁定遵守：未整档 t/tv/tt/tb/tu/taa/tf；merge 门（裸 t+触面档）
  pending 归 merge 冻结后统一执行。

---

# R5 修复轮（2026-10-22，code `c6fdaaaf1`）

R5 四项（R4-05-P1/R4-04-P2/R1-01-P2/R5-01-P2）全部修复；逐项 closure：

| Finding | 修复与永久反例 | 当期结果 |
|---|---|---|
| F-815-T05-R4-05（P1 捕获排水） | `drain_captured_stdout` 重写：reader_stop/cancelled 每刻检查+10s 硬界+1MiB 收集上限+`DrainEnd` 五分类（Eof/Stopped/Truncated/IoError/BoundOverrun）；jet adb 臂取消哨兵收口、非 Eof 部分输出不冒充完整探测；`map_flow` 取消后到达的 Ok 改判 Cancelled。永久反例：`captured_drain_answers_cancel_and_natural_eof`（跨平台，Windows+Linux 双跑） | 回放 adb-held-pipe：持管道取消 **263ms 内 cancelled**（原 14s 无果+释放后假 completed）、`exit_before_holder_release=0`（宿主 EOF 在 holder 释放前干净退出）；自然 EOF 正控 completed 保持 |
| F-815-T05-R4-04（P2 包管理器主配置） | `pkg::TOOL_PM_STATE` 三态单源：请求 raw `pkg:` 字段 > 外部全局配置/自动探测（**跳过磁盘主 pac.at**）> legacy 冻结全链（磁盘 pac > 全局 > 探测）；每操作 guard（tool_ops 三入口）设置/清理、不跨操作缓存。永久反例：`pkg::tests::tool_pm_state_matrix`（legacy 磁盘/请求字段赢磁盘/无字段跳磁盘/guard 恢复四态） | 回放 package-source：wire npm+disk pnpm 冲突例实际执行 **npm**（原 pnpm）；正控 npm/npm 保持 |
| F-815-T05-R1-01（P2 Unix 平台证据） | 无代码变更（R4 修复成立）；**平台实测补齐**：WSL Ubuntu-22.04（rustc 1.97.1，系统 xkbcommon/wayland dev 在）对冻结源码 `--locked` 全量编译+具名测试执行 | `unix_reader_truly_stops`（非 owned 持管道真停/Stopped + 自然 EOF 正控）**PASS**；captured_drain/vm_session 两单测 Linux 面 **4/4**。详 [unix-platform-evidence.md](../fix-evidence/unix-platform-evidence.md)（含环境/命令/边界绑定与 real_tree 预存可移植缺口披露） |
| F-815-T05-R5-01（P2 旧入口语义回归） | jet/ark/vscode build 内层 `strict = RunIo 在装`：非零退出/缺工具/产物 oracle 仅托管（协议）形态具名 Err；legacy 公开门面恢复基线（打印提示后 Ok）。永久反例：`legacy_build_semantics` 新集成档（unhosted jet exit7→Ok、ark 缺 wrapper→Ok） | 回放 legacy-build：**REVIEW_LEGACY_RESULT:Ok(())**（原 Err）；协议路径严格面由既有 tool_process 测试保持 8/8 |

## 当期验证账（R5 轮）

- scoped：proc_host **9/9**（新增捕获排水，Windows+Linux 双跑）、pkg 矩阵 1/1、
  lib **173/177**（4 红全为在案预存）、tool_process 默认 **21/21**+重串行 **8/8**、
  legacy_build_semantics **2/2**（新）、tool-execution **31/31**、协议 **19/19**、
  man **6/6**、consumer **5/5**；净改 .rs rustfmt --check 0 diff。
- 回放：R5 三反例脚本（adb-held-pipe/package-source/legacy-build）全闭 +
  R4 build-lifecycle 三例/profile-contracts 四根保持。
- Unix 平台面：WSL 真实冻结源码编译+具名测试 1+4 全绿（上表）。
- 测试时序裁定遵守：未整档；merge 门 pending。

---

# R6 修复轮（2026-10-22，code `9291a7ee3`）

R6 两项 P2 全部修复；逐项 closure：

| Finding | 修复与永久反例 | 当期结果 |
|---|---|---|
| F-815-T05-R4-05（P2 捕获失败终态吞没） | jet adb 臂（jet.rs 托管臂）：非自然捕获结束（BoundOverrun/Truncated/IoError/无取消 Stopped）→ 先 `tracked.kill()`+`wait_tracked` 有界收尸，再返回具名 Err（含 `DrainEnd` 分类与 teardown 收据）；非零退出/等待失败/spawn 失败同具名 Err（spawn 失败为同因 sibling，此前也吞成"无设备"）；`DrainEnd::is_natural_eof()` 消费者契约化。永久反例：`captured_drain_truncates_at_collection_cap`（真 1.2MiB 写手 0.2s 触上限→Truncated+kill 收尸）+`captured_drain_overruns_hard_bound_when_silent_holder_keeps_pipe`（静默持有者 10.1s 精确在硬界限→BoundOverrun，#[ignore] 重串行档）；自然 EOF 空列表=合法无设备、取消=cancelled 保持（既有测试+回放） | 回放 adb-drain-exits：bound-overrun **10.3s failed**（"probe did not complete (BoundOverrun; owned tree teardown Ok)"，原 10.6s completed/error=None）；truncated **0.37s failed**（"did not complete (Truncated; …)"，原 completed）；非 owned holder 不被越权杀（holder_alive 保持），owned 树收尸有界 |
| F-815-T05-R6-01（P2 PM 矩阵误拒 bun 回退） | oracle 重构为**来源观测**：`resolve_with_source()` 暴露 `PmSource{Request,Global,AutoDetect,DiskLegacy}`（resolve() 同语义委托，产品行为零变化）；矩阵断言来源——legacy=(Bun,DiskLegacy)；请求=(Pnpm,Request)；无字段=外部链来源（值随机器合法变化**包括 bun**）+ 相同外部环境下磁盘 bun/pnpm/移除翻转 (值,来源) 不变（磁盘独立性正反控）；guard 恢复=(Bun,DiskLegacy)；RAII `EnvGuard` 断言失败也恢复 cwd/PATH。`auto_detect` 用 stub PATH 直测：bun-only→**Bun（合法回退正控）**、bun+pnpm→Pnpm（优先级）。不再有任何"值排除"断言 | 回放 pm-fallback-environment：继承环境 exit0 + **合法 bun 回退环境（私有 PATH 仅 bun+空 HOME/USERPROFILE）同测试 exit0**（原 exit101）；真实工具同环境 wire 无 pkg+disk npm→实际执行 **bun**（产品回退正确，非 R4-04 重开） |

## 当期验证账（R6 轮）

- scoped：proc_host+pkg+auto_detect 定向 **11/11**（新增 Truncated/BoundOverrun/矩阵重写/回退优先级）、
  tool_process 默认 **21/21**+重串行 **8/8**、legacy **2/2**、lib **175/179**（4 预存在案）、
  consumer **5/5**；净改 .rs rustfmt 0 diff。
- Linux（WSL Ubuntu-22.04，冻结源码 --locked）：新面 **4/4**
  （tool_pm_state_matrix/auto_detect_fallback/captured_drain_truncates/captured_drain_answers）。
- 回放：R6 两反例脚本全闭（上表）+ 既有面保持。
- 测试时序裁定遵守：未整档；merge 门 pending。
