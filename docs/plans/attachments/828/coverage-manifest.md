# PLAN-828 T-09 覆盖清单（case → 子case → 真实入口）

日期：2026-10-20 ｜ 基线：`plan-828-dev`（T-08 提交后）｜ 验证命令：
`pwsh scripts/verify-function-incremental-828.ps1 all`

| V-case | 子case | 真实入口（测试名） | oracle | 状态 |
|---|---|---|---|---|
| V-01 仅一函数变化 | 双函数体编辑、结果/身份/计数/安装集 | `t01_v01_single_body_edit_full_path_incremental` | 独立手算期望 23→28；main 三重 Arc 身份；11轴计数；install={helper}；新旧 snapshot 双运行 | ✅ |
| V-02 来源/语法 | 未改函数不重新投影 | `t01_v01`（main 全 0 计数）+ `t02_frozen_load_projection_parity`（投影/解析 AST+span 全等） | 差分对拍（ac-core scalar parser 为 oracle） | ✅（见开放项 1） |
| V-02 语法错误 | 破坏性编辑被拒、旧版可用 | `t02_syntax_error_candidate_rejected_keeps_old` | 旧版 23 可运行、main 全 0 | ✅ |
| V-02 trap span | 位移后 span 映射 | `t07_trap_span_maps_through_line_col` | line_col 随编辑 +11 字节位移 | ✅ |
| V-03 精确依赖 | 只读接口消费者不重编 | `t01_v01`（main 计数 0）+ `t03_signature_change...`（签名变化非快路径） | 计数器 + 接口环境门 | ✅（见开放项 2） |
| V-03 凭证失效 | 错上下文不可复用 | `t03_body_credentials_are_context_bound` | 不同声明上下文→不同修订→凭证不可复用 | ✅ |
| V-04 信任 | 坏候选保留旧版 | `t01_v04_rejected_candidate_keeps_current_version`（类型错误） | 诊断 + 旧版可运行 + 恢复编辑 | ✅ |
| V-04 单元准入反控 | 可执行闭包不可入/签名不匹配链接拒 | `t04_unit_admission_negatives` | admit_unit Err / link Err | ✅ |
| V-04 raw unit | 单元凭证不可直接执行 | auto-av-core `av_unit_kind_*`（798 既有 3 例） | executor 拒绝派发 | ✅（继承） |
| V-05 版本可见性 | 确定性屏障 | `t06_v05_deterministic_barrier_keeps_inflight_chain_on_its_snapshot` | 链暂停→安装→恢复=旧语义；新调用=新语义 | ✅ |
| V-06 失败/迟到 | 乱序提交 | `t06_v06_stale_candidate_never_overwrites_newer_result` | r2 先提交、r1 陈旧被拒、当前结果存活 | ✅ |
| V-06 fuel/trap | 资源耗出不改安装 | 同上（尾部 fuel 段） | Resource 结果 + 快照可再运行 | ✅ |
| V-07 恢复 | 温恢复重验/miss 回退 | `t07_restore_readmits_and_miss_falls_back` | restore_units_admitted=2、miss 回退冷启动仍正确 | ✅ |
| V-07 回收 | 512 修订+有界历史+旧版保活 | `t07_512_revision_soak_bounded_history_pinned_old_snapshot` | 每64修订独立期望；history ≤ 2×CAP；pin 的 rev0 快照全程可用 | ✅ |
| V-08 双门 | 冻结负载30修订 P95+结构 | `t08_v08_formal_p95_and_structural_dual_gate` | 独立期望 + 双门（P95<1000ms 且结构零门） | ✅（见开放项 3） |
| V-09 反控 | 每轴独立可红 | `t08_v09_gate_negatives_redden_independently` | 零阈值/缺样本/伪全量/慢而细粒度 各自必红 | ✅ |

## 兼容证据（T-09 全量定向回归，全部绿）

- 828 面：`function_incremental_828` 17/17；adapter `scalar_projection_` 5/5 + 全套 10 suites。
- 798 面（未动支持面）：mir 8、abc 15、binding 4、cache 20、vertical 3、native_admission 11、native_trace_parity 1、case_binding 5、auto-av-core 21。
- 整模块入口保留证据：`compile_parts_with`/`lower_checked`/`split_units`/`assemble_closure`/Driver 全部原样（上列 798 面即其回归）。

## 开放项（work 已如实登记，不冒充已交付）

1. **V-02 错误恢复 span 全等**：语法错误的诊断 span 映射走拒绝路径（reason 携带），
   未做错误恢复树与 fresh 的逐 span 全等对拍——错误面为拒绝语义，首版无错误恢复消费方。
2. **V-03 body 事实消费者**：scalar profile 无 CTE/inline/布局消费者，"真实body事实查询"
   失效轴的负例不可构造；接口轴（签名）已全覆盖。语言扩展落地时补该轴（后继合同已预留）。
3. **V-08 进程级内存双线**：正式档在进程内会话上计时+结构门通过；798 双线内存预算
   （A≤224MiB/B≤128MiB）对 828 会话路径的归因测量与 AutoDown worker/IPC 正式包装
   归 batch/后续修订（会话进程峰值证据在 batch 档补齐；contract-decision §7 已记口径）。
