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

V-12（AC-11，T-11）✅：incremental_tier 4/4 @ T-11——
1.1 D828 基线 273575B/115 函数 ×30 修订**双门过**：结构（未改单元 fp/代际逐轮不动、whole-module 阶段恒 0、恰一安装/一重发/114 驻留命中）AND edit→PE P95=715.5ms<1000（P50 528）；
1.2 两函数局部更新：恰各自重键、先改身份保留不回退、main 零重发；
1.3 精确失效/拒绝：helper body 恰 helper 重键（caller 代际 0）；签名/接口编辑=retained environment 接口门拒绝且 mn 状态零变化、会话不 wedge；未授权 capability（无 trace 能力会话内加 mark_a 调用）前端拒绝；
1.4 退化必被抓：content-hit 回退编辑触发记录性 whole-module fallback（fallback_whole_lower=1，结构门非零）；活键下换对象=store 受控 Conflict、活身份存活、后续编辑正确。
接入面：828 会话 PreparedEdit 增 unit_mir/changed_name 消费缝（加法-only，auto-av-probe 回归 17+6+28 全绿）；mn 消费者 src/incremental.rs（D828 驻留模型：resident 工作集+store 持久层；mn 门重验证单元；emission 失败永不提交 revision）。

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

- 2026-10-24 acceptance scope @ `32c050234`+T-08 树：全绿（`OK: acceptance`）。
  顺序敏感实测：墙钟门必须先跑（resources 档 ~10min 高频 IO 的余波把紧随的
  body-edit 尾部推到 3-12s 尖峰实录）——acceptance/closeout 固定
  turnaround-formal 前置、resources 垫底。明细：first-case 3/3、
  turnaround-formal 5/5（159s）、codegen 6/6、parity 2/2、cache 7/7、
  transaction 6/6、autodown 10/10、turnaround-driver 3/3（105s）、
  resources 5/5（895s）＝工作区 32 测+主仓 15 测。raw 五文件与本报告同目录。
  merge 时 closeout 于冻结提交重放。
