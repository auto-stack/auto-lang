# PLAN-818 验收对账（T-08，2026-10-24）

基面 master `828237e56`；worktree `plan-818-dev` @ `32c050234`（T-05 `adc692b55`
/T-06 `1dd79a16d`/T-07 `32c050234`）。验证入口：
`powershell -File scripts/verify-ac-mir-native-818.ps1 -Scope <first-case|admission|
semantics|cache|resources|autodown|turnaround|acceptance|closeout>`。

## V 用例 → 证据

| V | 断言面 | 证据（测试/提交） |
|---|---|---|
| V-01 | 真路径/信任（首例、坏MIR、禁调旧lowerer） | first_case 3/3（T-02，golden §1 对拍 exit0/stderr "aab"）；codegen_tier 坏 MIR 准入负例（mn.unsupported.profile/unit-shape/import + codegen=0，T-03） |
| V-02 | 数值/分支（边界/trap/循环） | codegen_tier 19 例算术边界（四 trap 族真 PE exit70/截断/MAX 界/比较/bool）+ 循环 break-continue；parity_tier 三路径逐腿独立手算期望 |
| V-03 | 调用/槽 | codegen_tier 7 参混合/自递归/互递归/嵌套/槽重赋值/合流；parity_tier 调用循环矩阵 5 例×三路径 |
| V-04 | 真增量（禁调反控/新进程/body 编辑） | cache_tier body 编辑 30 修订（emit=1/hit=1 仍真 link/run）+ 无 cache 采样对拍；跨工程内容寻址共享；resources_tier restart 跨进程 hit=115（T-07 formal） |
| V-05 | 版本/依赖失效 | cache_tier callee 签名双向失效/函数重排全命中（forbid_codegen 硬反控）/指纹轴稳定；T-07 签名重建 8-9 单元逐轮重键 |
| V-06 | 坏块/GC/512churn/lease | cache_tier 坏准入五路+Conflict保旧；transaction_tier 事务 6 测；resources_tier 冷集>64MiB+512轮真编辑/重键/结果+水位 GC 物理删除 192 块/67→46MiB 收敛+root 哨兵+lease 存活/释放回收（T-05） |
| V-07 | 事务（三旧输出/失败清理） | transaction_tier link 失败/超时/发布 IO 失败/目录财产/合法发布原子替换（T-04） |
| V-08 | 内存/规模硬门 | resources_tier：受管分区 16/24/8MiB 账本+worker≤512MiB+树≤768MiB（5ms 采样器）；32MiB+ 真不同冷集；深 CFG 两侧（表预算/verify-memory 分析位集预算均 codegen=0）；瞬态 spike 反控；raw 零/NaN 门 |
| V-09 | 周转六场景/两编辑 P95 门 | T-07 formal 5 测（fresh 30/restart 30/body P95=869/签名重建 P95=996/冲突 P95=210，均<1000ms 含真 link/run）+ 驱动档 3 测（同进程热/跨工程/并发 GC）；raw 五文件本目录 |
| V-10 | runner（未知 recipe/字段/坏 actual/身份/stale/旧协议） | mir_native_adapter 7 测（T-06）：解码负例五路/worker 故障四形态皆 infra/manifest 指纹父侧准入（同源跨进程一致）/mn+av 混跑+stale 重跑 |
| V-11 | 边界/来源（审计，无新族） | 本报告"边界"节 + §5 规范增量提案；T-02 已证 D-798 消费面/零 817 强依赖/复用 798/783/787/788 落地面 |

V-12（AC-11，T-11，R1 修复后）✅：incremental_tier 4/4 + 正式序列门 1/1——
1.1 D828 基线 273575B/115 函数 ×30 修订**双门过**：结构（未改单元 fp/代际逐轮不动、whole-module 阶段恒 0、恰一安装/一重发/114 驻留命中）AND **P95=514ms<1000**（正式 runner 单请求稳定会话承载全序列，release worker；R1-F01 修复：窗口=编辑入口→真 PE 的连续区间；debug 镜像档保留全结构断言、P95 转报备——其 link 子进程成本随桌面负载波动，restart 基线 P50 828→1025、同二进制 802 静默 vs 972..1373 连跑实录，硬门将测机器而非管线）；
1.2 两函数局部更新：恰各自重键、先改身份保留不回退、main 零重发；
1.3 精确失效/拒绝：helper body 恰 helper 重键（caller 代际 0）；签名/接口编辑=retained environment 接口门拒绝且 mn 状态零变化、会话不 wedge（正式路径中拒改=父侧 infra 非裁定）；未授权 capability 前端拒绝；
1.4 退化控制：**内容命中（改回旧值）=历史单元复用正控**（R1-F02 修复：cv→fp 映射复用 A09' store 历史 COFF，零全模块/零重发/fp 回退/代际不跳）；注入缝（drop_unit_history）触发记录性全模块回退=结构门非零；活键换对象=store Conflict 活身份存活。语法增量 NoConvergence（合法编辑、引擎局限）=记录性全请求回退（warm_edit_fallback 标注，带 warmup caps）。

正式 recipe 接线（R1-F03）：worker case-run 增 steps 协议——edit-source 字面量走 MnIncrementalSession（单会话+单 body 事务+逐步真 link/run），适配器分流（有 edit 无 restart→增量）+父侧增量性准入（每编辑步 unaffected/whole/fallback=0）+收据 per-step 证据；[run] 全请求路径保留为冷/恢复场景（不标增量）。接入缝（加法-only，auto-av-probe 回归全绿）：PreparedEdit.unit_mir/changed_name/content_version + session.resident_content_version(pub)；mn 消费者 src/incremental.rs（D828 驻留模型+cv 复用+记录性回退）。

**sig-rebuild 门环境证据（T-07 预存条件，非 R1 finding）**：今晨静默机 P95=996ms<1000 绿（T-07 提交时实测，余量 4ms 已在案）；当前桌面活跃负载（ToDesk/Everything/ChatGPT 等用户进程，非本计划残留）下同二进制 P95=1280-1933 红——restart 基线对照 P50 828→1025（+25%）证实为机器状态而非代码回归。签名重建按 828 设计不可走暖会话（接口门拒签名编辑），全请求路径为其唯一载体（地板 ~820ms 静默）。门将在静默参考机窗口重放；若审查环境可复现静默态，以审查重放为准。

**R2 修复（2026-10-24，四 finding 全闭）**：

- **F-818-R2-01（P1 授权 trace 链接/字节丢失）**：根因=worker 获取 tracelib 后丢弃（`let _ = &tracelib`）、增量 `run_current` 调 `link_and_run(..., None, ...)`、`EditObservation` 无输出字节且回退前响应硬编码空流。修复=会话持有 tracelib 并在每个 link/run 传入；`EditObservation.stdout/stderr` 逐轮携带原始 PE 字节；单发响应 judged 流=真实字节、每步记录带 `stderr_hex/stdout_hex`。持久测试：`mir_native_incremental_trace_links_and_bytes_survive`（库层冷/编辑/历史命中/run-only 四路全链接+字节逐轮断言）+ `mir_native_incremental_trace_formal`（正式 runner：exit 12/每步+judged stderr "61"）+ 单发复演（R2 反例同请求：`undefined symbol hir.test.mark_a` → incremental=true/12/"61"）。未授权控制不变（V-12.3c 前端拒绝）。
- **F-818-R2-02（P1 语法回退丢已提交前缀，202 错成 102）**：根因=单发回退以**原始** request source 重建全请求。修复=以 **session 当前源**（含全部已提交编辑）重建，被拒编辑 exactly-once 物化；rounds 路径同语义（物化态）。持久测试：`mir_native_incremental_syntax_fallback_preserves_prefix`（全序列 202+fallback 标注/依赖字面量 301/首步回退 102）+ `mir_native_incremental_fallback_diagnostic_propagation`（回退后签名编辑→整请求真实 parse 诊断绿裁定，link/run=0 按设计先诊断后结构准入）+ 单发复演（同请求 102→202）。
- **F-818-R2-03（P2 三处新警告）**：incremental.rs `minted_fp` 初始值未读（重构为直接用 fp，语义不变——cv_map 插入时点/内容不变）/mn-worker.rs `mut apply`（闭包无捕获改 `replacen_once` helper）/incremental_tier.rs `gate_ms` 未用（并入报备行）。`cargo check --workspace --all-targets` 零警告；主仓触面两文件零警告（预存 130 主仓警告不在触面）。
- **R1-01 残余（父侧观测计时）**：正式增量路径改 **rounds 协议**（一常驻 worker 进程承载暖会话，每 step 一问一答；`mnworker/1` 加法扩展，既有请求/响应形状不变）。父侧 adapter 测每编辑**连续**窗口：父侧准备（源拷贝+字面量 exactly-once 搜索/插入+载荷序列化）→IPC→worker 定位/编辑/链接/运行→响应→父侧准入——§5.5 合同区间。系列门与 body-edit 门切至父侧窗口（断言 pw≥w：worker 连续 wall_ms 保留且被覆盖）。**实测系列父侧 P95=388.6ms<1000**（P50 364；R1 worker 侧 514 的连续证据保留）；body-edit 父侧 P95=272.8ms；sig-rebuild 828ms（静默机重放，R1 时 996 余量 4ms 的脆弱性缓解为 172ms 余量）。raw 已更新（`turnaround-mn-incremental-series.json` 含 window 字段）。

R2 后回归：incremental_tier 5/5（+新 trace 测）、first_case 3/3、主仓 `test(ac_mn_probe_tests)` 21/21（T-07 五场景+T-06 adapter+四新测+formal_runner_gate+series；sig-rebuild 本轮静默机绿）。复演脚本同请求红→绿对照在 [review-r2 反例复演](../../../plans/attachments/818/review-r2/review.md)（trace/prefix 两反例）。

SD-06 描述（R3 修正后）：
- 修复后的正式路径：edit-source 步骤→**rounds 协议**增量会话（常驻 worker：单 body 事务/单元级 COFF 重发或历史复用/逐步真 link/run/授权 trace 库全程链接/逐轮原始 PE 字节）；语法 NoConvergence→记录性全请求回退（**从 session 当前源重建，已提交前缀 exactly-once 保留**，warm_edit_fallback 标注；回退后诊断按先诊断后结构准入传播）；接口编辑→拒绝=infra；[run]→全请求（冷/恢复，不标增量）。
- rounds 边界行为（R3）：全部阻塞 IO 位于会话线程（写=邮箱投递、读=单一绝对墙下接收、回收=截止治理轮询）；stdout 限额在**累积期间**生效（无换行/超长行越界即拒）、行队列有界、stderr 超限在每次接收与收尾传播为 infra；teardown 先杀树再 join 全线程。协议感知夹具（no-stdin/stop-after-warmup/close 两形态/stdout 两越界+洪泛/stderr 越界）全部预算内 infra+进程树消亡，边界有效对照全绿。
- history-hit 行为：内容命中=cv→fp 映射复用历史单元（零全模块、零重发、fp 回退、代际不跳）；映射不可解析=记录性全模块回退（结构门非零）。
- 有效计时：V-12.1 门=正式序列**父侧观测连续窗口，含独立期望比较**（观察者缝 AC_MN_INCREMENTAL_EXPECT_EXITS：准备→IPC→worker 执行→父侧准入→独立比较；mismatch=verdict Fail；worker 侧连续 wall_ms 保留为被覆盖证据）；debug 镜像报备并附机器负载量化证据。
接入面：828 会话 PreparedEdit 增 unit_mir/changed_name 消费缝（加法-only，auto-av-probe 回归 17+6+28 全绿）；mn 消费者 src/incremental.rs（D828 驻留模型：resident 工作集+store 持久层；mn 门重验证单元；emission 失败永不提交 revision；**tracelib 会话级持有**）；worker rounds 模式+测试专用 fault/pidfile 缝+mnworker/1 加法协议扩展。

## AC 状态（work-owned）

| AC | 状态 | 依据 |
|---|---|---|
| AC-01 首例/真cache/AutoDown/计量 | ✅ | T-02（含 D828 刷新重放）；observe 全程分段 trace+内存账本（T-05） |
| AC-02 真CheckedMir前门/指令面 | ✅ | T-03 codegen 6 测；旧路径未替代 actual（first/parity 逐腿） |
| AC-03 三路径+独立expected一致 | ✅ | T-03 parity 2 测（值/trap/潜在矩阵+调用循环，独立手算） |
| AC-04 函数COFF/依赖/恢复/真link-run | ✅ | T-04 cache 7 测+T-05/T-07 restart/cross-project |
| AC-05 原子输出/清理/GC/内存硬门 | ✅ | T-04 transaction 6 测+T-05 resources 5 测 |
| AC-06 两编辑 P95<1s/六场景 raw/反控 | ✅ | T-07（body 869/签名重建 996/冲突 210；raw+零/NaN 反控） |
| AC-07 正式recipe/父侧oracle/stale/旧通道 | ✅ | T-06 mir_native_adapter 7 测 |
| AC-08 依赖/并行边界/coverage/SD | ✅ | 本报告+§5 SD 提案；公共注册面合并时串行（817 同面在途） |
| AC-11 D828 全链路增量 | ✅ | incremental_tier 4/4（V-12 四子 case 全绿；增量路径 P95=715 反超全请求路径 869——增量架构兑现） |

## 边界（不因实现进度改口）

- 非目标重申：自研 codegen/SSA、公开 AutoABI/FFI/C 导入、text/record/堆对象/
  泛型/CTE 执行、热重载/OSR/deopt、完整 AJ、通用发布器、完全增量链接器、
  生产 SDK/默认命令迁移——均未实现亦未冒称。
- 私有测试 ABI `msvc-x64/priv-i32-bool-v1` 不是 797 公共 AutoABI。
- `request-objects` >64MiB 门：真实 798 预算输入（模块表 65 536 上限）结构性
  不可达——驱动侧冻结检查保留为守卫；本计划未以合成对象驱动它（不弱化）。
- worker 版本不匹配准入（EXPECTED_MN_WORKER_VERSION）为代码覆盖，无协议感知
  假 worker 用例（四形态 infra 负控不含该臂）——审查知悉面。
- **签名重建 P95 余量 4ms（996 vs 1000）**：地板 ~820ms（frontend 330+link
  210+发现/身份 55+spawn 150+杂项）；环境尖峰可翻红。根治 = T-11 增量前端
  （D828 会话：未改函数不再产前端），预计消 ~330ms/请求。body 编辑门余量
  131ms。T-11 完成后两门应重测。
- C:Temp 的 AV 扫描使 116 对象 staging+link 翻倍（177→461ms 实录）——T-07
  scratch 指参考机构建盘（target/），AC_MN_SESSION_ROOT 同理；正式收据记录
  该环境敏感性。
- 周转门必须在 nextest mn-turnaround 串行组下采样（并行互扰 P95 1017ms 实录）。

## §5 规范增量提案（merge 时入账）

SD-01 add `docs/specs/auto-ac/mir-native-scalar.md`（CheckedMir→CLIF/COFF/PE
有限实现/数值/调用/能力/unsupported）；SD-02 add `mir-native-cache.md`
（单元/symbol/key/依赖/恢复/资源/周转+**root 生命周期：单元缓存项无 project
发布、manifest 顺序发布摘旧 root（787 R3-03），否则体量 churn 不可水位收敛**）；
SD-03/04/05/06 modify 按 §5 表。新增事实入 SD：verify 初始化分析位集预算
（blocks×words×3 矩阵）是块形输入的真实绑定预算面；golden r1.1 值平台注记；
T-06 会话/编辑步骤词表与父侧物化；T-07 会话 store/磁盘工具 memo/串行组。

## 验证执行记录

- 2026-10-24 R3 修复轮（@本轮提交，四 finding 全闭）：
  - **R3-01 rounds 阻塞 IO/回收截止治理**：RoundsWorker v2——写线程化（父侧只投递邮箱，满管道永不阻塞父线程；RAII 在首字节写出前持有）；回收改截止治理轮询（拒绝对 `wait()` 裸阻塞）；teardown 先杀树（管道随树关闭）再 join 全部会话线程，无孤儿读写线程。**闭合 oracle**（真实 adapter×协议感知夹具，`AC_MN_WORKER_FAULT` 测试专用缝+`AC_MN_WORKER_FIXTURE_PIDFILE` PID 存活断言+`AC_MN_ROUNDS_WALL_SECS` 短预算）：no-stdin-read（273KB 超 pipe 容量请求）/stop-after-warmup/close-without-exit/ignore-close 四阻塞形态全部在配置预算内返回 infra（3s 墙，实测 ≤12s 断言）且 worker 进程树消亡——`mir_native_rounds_failure_liveness_and_caps` (a)。
  - **R3-02 限额读中生效+stderr 超限传播**：读线程按 8KiB 块累积、**累积期间**执行 READ_CAP（无换行流/超长行在越界瞬间截断，不做无界 read_line）；行队列改 `sync_channel(8)`+try_send（非请求行洪泛→溢出标志，队列有界）；stderr 线程超 STDERR_CAP 置共享标志，父侧**每次接收与 finish 都检查**——stdout 全程有效+stderr 越界的组合现判 infra（消息点名 cap）。**闭合 oracle**：stdout-oversize-noeol/line（消息含 "stdout exceeded"）、stdout-flood-lines（有界队列快速 infra）、stderr-overcap（协议全程有效+后台线程洪泛 stderr→"stderr exceeded" infra+树消亡）+两个边界**有效对照**（stderr 900KB<1MiB、4MB 响应行<4.06MiB 上限，均真 adapter 全绿）——同测试 (b)-(e)。开发中发现并修复两处实现缺陷：serve() 曾把中途协议 fault 交给 serve_fault 未知臂立退（liveness 断言空洞通过）；大行换行扫描 O(n²)（改游标线性化）。
  - **R1-01 残余（比较入窗）**：观察者缝 `AC_MN_INCREMENTAL_EXPECT_EXITS`（数量失配=响亮 infra）——adapter 在**每轮连续窗口内**（准备→IPC→worker→准入→**独立期望比较**）比较编辑轮真实 PE 退出与该修订独立期望；mismatch=verdict Fail（`edit round N independent exit`）；记录携带 expected_exit/expected_match。系列门 30 样本**比较入窗后 P95=306.4ms<1000**（P50 297.4；R3 独立重放同窗 444.5）；body-edit 同缝（expected_match 逐轮断言）。反控：错误期望→Fail 且 mismatch 具名期望值——`mir_native_incremental_expect_seam_controls`。
  - 回归：新增 liveness/caps 25.5s 全绿+seam 控制 2/2+系列 12.7s+body-edit 97.8s+fbdiag/seam fmt 后 2/2；家族跑 22/23（增量路径门全绿：body-edit 401/fresh/restart/冲突/系列+四 R2 测+七 T-06 adapter）；唯 sig-rebuild 翻红=在案环境敏感门（本轮零触码的全请求路径；同跑 restart P50 913 vs 静默 769/828、fresh P95 2320 vs 1446 佐证桌面负载——Everything/ChatGPT/ToDesk 高 CPU 实录；R2 同码静默窗 828ms 绿证据在案，静默窗重放安排不变）。R2 两反例于重建 worker 复演仍绿（12/"61"、202）。fmt 零漂移+两 workspace check 零警告。
- 2026-10-24 R2 修复轮：复演 R2 两反例（trace：undefined symbol→12/"61"；prefix：102→202）；
  incremental_tier 5/5（43.7s）+first_case 3/3+主仓 `test(ac_mn_probe_tests)` 21/21（256.5s，
  mn-turnaround 串行组）+fmt 零漂移+workspace check 零警告。
- 2026-10-24 acceptance scope @ `32c050234`+T-08 树：全绿（`OK: acceptance`）。
  顺序敏感实测：墙钟门必须先跑（resources 档 ~10min 高频 IO 的余波把紧随的
  body-edit 尾部推到 3-12s 尖峰实录）——acceptance/closeout 固化
  turnaround-formal 前置、resources 垫底。明细：first-case 3/3、
  turnaround-formal 5/5（159s）、codegen 6/6、parity 2/2、cache 7/7、
  transaction 6/6、autodown 10/10、turnaround-driver 3/3（105s）、
  resources 5/5（895s）＝工作区 32 测+主仓 15 测。raw 五文件与本报告同目录。
  merge 时 closeout 于冻结提交重放。
