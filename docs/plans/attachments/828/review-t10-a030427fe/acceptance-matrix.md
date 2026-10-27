# PLAN-828 r3 最终合同核销

冻结 a030427fe。下面“通过”指对应功能/断言，不给 T10-H01 格式门或 AC10 合入收尾授通过。原始日志路径见 review.md；R5=相邻 review-c04-r5-a030427fe，经 evidence-reuse.json 全输入/运行物/证据核验。命令见 commands.md。

| AC | 任务 | 实际生产/消费与区分 oracle | 证据 / 结果 |
|---|---|---|---|
| AC01 | T01/03/04/05/06/07 | 单函数历史与真实依赖、未改 HIR/FunctionCode/实际执行payload同对象、恰一安装；各生产窗零未改工作 | R5 contract/b1/b2/r1；本轮三512；通过 |
| AC02 | T01/02/03 | incremental_reparse→project_decl_strict→prepare_candidate_document→resolve_fn/check_body_in；真旧整Program同窗负控红 | R5 contract22/t01r4；本轮scalar差分6/ac-core111；通过 |
| AC03 | T04 | lower_function_unit→verify_mir→ABC/decode/verify_abc/admit，未改全链0、hit原mint | R5 contract/b1/r1；本轮core22、MIR8/ABC15/cache20；通过 |
| AC04 | T05/06 | link_with_maps owner/调用闭包/签名→UnitView；三入口同派发，prepare不发布，失败/foreign/stale/closed拒绝 | 本轮probe组件1/core22/close9；R5 b214/contract22/consumer2/trace1；通过 |
| AC05 | T02/03/04/05/07 | verify_function私有凭证、can4、产物自己的依赖、A07重验/owner/domain/caps/来源门，原完整输入缺body仍拒 | R5 b111/r186；本轮ac-core111/scalar6/core22；通过 |
| AC06 | T06/07 | PausedRun旧链/三入口、实际取消轮询/lastownerWeak、history32/真实GC/HIRrootsbounded | 本轮三512/close9/resources17；R5 t078/b214；通过 |
| AC07 | T08 | 真worker IPC→当前值/身份同次publication→父侧全部校验终点；0门/错值/缺流/whole/copy各红 | R5官方5/5及30raw、judge6/consumer2/identity3/io/handshake；通过 |
| AC08 | T07/08/09 | 完整恢复与兼容、版本映射/后继接缝/SD、健康收尾 | 功能证据通过；T10-H01导致健康partial |
| AC09 | T10 | 独立冻结/依赖/配置/输入绑定逐AC/V/C/SD审核 | 本次审查完成，因T10-H01未授最终PASS |
| AC10 | T11 | reviewed后冻结裸t+tv/consumer/触面、Spec/ledger/归档guard | pending，merge-owned；本轮未运行 |

| V | 对应实际测试入口 / 关键断言 | 本冻结证据 |
|---|---|---|
| V01 | t01_v01_single_body_edit_full_path_incremental、contract_828_hotpath_real_session_edit_is_single_body_window、t05_generations_and_stable_ids_across_revisions；前中后三位置和槽/长度 | R5 session18/contract22/b214/r186、本轮三512；通过 |
| V02 | t02_frozen_load_projection_parity、t02_syntax_error_candidate_rejected_keeps_old、r1 trap/mint/位移矩阵、scalar_projection_ AST+span | R5 session/r1、本轮scalar6；通过 |
| V03 | t03_body_credentials_are_context_bound、t03_body_fact_query_dependency_invalidation、reread/history产物快照、r1 intrinsic每引用/effect | R5 b1/r1/contract；通过（未新增CTE语法） |
| V04 | t04_unit_admission_negatives、b1 A07五门、b2 foreign/unresolved/实际linkErr、full check_module缺body守卫 | R5 b1/b2/contract；本轮组件/core/ac-core；通过 |
| V05 | t06_v05_deterministic_barrier_keeps_inflight_chain_on_its_snapshot、b2 pinned self/mutual recursion、contract三入口 | R5 session/b2/contract/trace；通过，无sleep竞态 |
| V06 | stale/cancel/foreign/close事务、组件linkErr全状态保旧、resume分段预算、trap/fuel/lastowner | R5 contract/b2/t07，本轮close9/组件；通过 |
| V07 | A07/ABC/sidecar重验与治愈、manifest strict+双指纹、真实GC/restart/512/sentinel | R5 mig9/t078/r186；本轮完整512三件；通过 |
| V08 | t08_formal_campaign_v08_dual_gate_30_revisions：正式30新修订，父侧独立期望和当前修订/目标/身份、双线预算 | R5官方5/5及完整raw/report复用；通过 |
| V09 | t08_judge_gate_negatives_each_axis_red、旧pipeline同门红、consumer-copy、缺目录/seq、资源双线越界、受控旧版保持 | R5独立反控；本轮resources17、close9；通过 |

| C | 实际 oracle / 入口 | 证据 / 结果 |
|---|---|---|
| C01-01 | contract单body冷编逐字节相等/生产窗、旧pipeline反控 | R5 contract/t01r；通过 |
| C01-02 | bool/命名实参/短路/for/递归/trace授权、冷编差分 | R5 r186；本轮scalar6/ac-core111；通过 |
| C01-03 | 只读接口caller不重编；每产物已读/未读body事实，intrinsic/owner/domain/caps | R5 b111/r186；通过 |
| C01-04 | frontend错误不毒化env，语法/类型/未初始化/缺返回拒绝、修好后继续 | R5 contract/session；本轮源前端/凭证兼容；通过 |
| C01-05 | 来源全局位移不失效未改凭证；签名/跨声明/不收敛明确拒绝，缺观测及whole负控红 | R5 contract/t01r/r1；通过，词法邻接扫描单列 |
| C02-01 | 跨修订未改实际AV payload同对象/代际/内容+真副本被拒 | R5 consumer2/identity3/contract；通过 |
| C02-02 | b2 heterogeneous tables/named args/allslotorders、共享producer槽不同函数独立映射 | R5 b214/r186；本轮core22；通过 |
| C02-03 | pinned self/mutual recursion、三入口、旧链来源/trace | R5 b2/contract/trace；本轮close；通过 |
| C02-04 | 实际组件link失败全状态保持后好候选可提交；foreign/stale/closed | 本轮probe组件1/close9；R5 contract/b2；通过 |
| C02-05 | 真执行深副本值可对但resident/identity/seq红，正常控制绿 | R5consumer2/judge6/identity3/trace1；通过 |
| C05-01 | cancel消费不安装/兄弟提交/stale/foreign/所有终态守卫 | 本轮close_review_cancel_then_sibling_commit_and_stale_cancel等；通过 |
| C05-02 | close非drop-session：未pin root/HIR/code立即Weak死、外pin保持最后owner释放 | 本轮close_controls对应三Weak控制/三512；通过 |
| C05-03 | historyhit候选处置点、PausedRun完成/取消最后owner、trapspan全等 | 本轮close_adjacent三件/close_review_actual_paused_chain；R5t07/b2；通过 |
| C05-04 | 非零revision/entry保持、vacant普通/分步0/1/10受控Infra、resident空、幂等 | 本轮close_adjacent_vacant_accessors_and_both_execution_entries_are_total；通过 |

任务核销：T01接口决策/竖切、T02真实前端、T03凭证/A07/依赖、T04单函数生成/内容复用、T05单元派发/受检链接、T06事务/三入口/终态、T07存储恢复/GC/预算、T08正式worker/judge均通过。T09现仅格式收尾partial，T10已完成审查但verdict needs_fix，T11未开始；completed count=8/11。所有前批finding闭合不因本次格式项重新打开。
