# PLAN-816 r2 / M816-A / T-13 独立阶段复审 R3

2026-10-21。结论 **needs_fix**。R2 判定的三残项（03/05/06）**主面闭合、
01/02/04 无回归**；但本轮发现 **2 项新 finding**（R2 修复引入的边界缺陷，
非旧例复发）。Plan 保持 `executing`，T-02/T-12 重开，current_step=1/14。

## 审查身份与限制（如实声明）

- reviewer=本聊天会话，**与 R2 修复实施同会话**（用户指定执行 R3）；不能
  声称独立上下文。为对冲：结论全部从工件重建——冻结 diff 逐行精读、
  两个既有 replay 重放、18+40 次定向运行、门禁重跑；未采信实施侧 §9/
  候选摘要的任何未验证声明。
- review_scope=`phase`（M816-A/T-13，非整 Plan final）。未创建 agent；
  未修改实施源码；本轮只写本报告与 Plan 簿记。

## 冻结身份

- worktree `D:/autostack/.wt/lang-816/auto-lang`，branch `codex/plan-816-dev`，
  reviewed_commit=`aa2edc8fbc3889f72fcf3dcb4b82887dd28d3bd9`（入场/收场均
  clean）；R2 修复 diff（4372d56d9→aa2edc8fb）5 文件 +1166/−263。
- 祖先链验证：b65d0e963 → … → 4372d56d9 → aa2edc8fb；上游 M815-A
  `e609caf1af` 三 crate 对该链 diff 为空（零漂移）。无 814 消费。

## 指派范围核验（三残项闭合 + 01/02/04 无回归 + 新回归质量）

| 项 | 核验方法 | 结果 |
|---|---|---|
| F-01/02/04 无回归 | R1 replay 重放（F01a 251ms/F01b 262ms/F02a 190ms/F02b 204ms contract_pass；C01/C02/C03 对照）+ 集成 deadline_covers/root_exit/early_spawned/stdin_writer 多轮 | **无回归**（CLOSED 保持） |
| F-03 主面 | R2 replay：close-bad=`Err(StreamViolation(TooLarge{4294967295}))`、close-clean=`Ok(Success)`；fixture close_phase 两测多轮绿 | **主面闭合**；间歇丢帧面见 R3-01 |
| F-05 | fixture 15 测多轮 0 告警 | **CLOSED** |
| F-06 主面 | R2 replay：timeout-held-readers **251ms 返回、线程 4→4 零悬留、非拥有 heartbeat 保持**；abort_stops<1.5s；std::io::pipe 真实管道停止性单测 | **主面闭合**；早期错误路径见 R3-02 |
| 新永久回归质量 | 逐条精读 | 合格：proto-close 确定性（EOF 后才写垃圾，无 sleep）；真实管道 stop 测以 Stopped 断言；Windows writer 停止性为"有界+具名"（同步句柄内核限制，已文档化并双分支断言）——接受 |

实跑门禁（@aa2edc8fb）：crates 10+8+17、fixture 15/15（多轮）；clippy
--no-deps 0、rustdoc 0、fmt+fixture rustfmt 过；verify-deps PASS；777
strict pass（78 边/0 违规/2 grandfathered）；`cargo check -p auto -p
auto-man` exit 0。未整档（时序裁定；tf 归主检出到期批量）。

## Findings（全部 OPEN）

### F-816-R3-01 / P2：stop 语义"先停后排空"，丢弃已到达未读数据

**R2 修复引入的回归类缺陷；T-12（client reader/stderr），AC-07/V-05。**
`crates/auto-tool-client/src/lib.rs` stderr reader 循环以
`if sstop.load(...) { break Ok(()) }` 为**第一**检查（reader_run 的
`if stop ... return` 同构）：线程从 50ms 轮询片醒来时若 stop 已置位，
**OS 管道里已到达的字节被直接放弃**——

- stderr：日志缺字。实机复现一次：高负载全套并行批次中
  `consumer_describes_real_tool_via_client` 与
  `shutdown_bounds_pipe_holding_descendants` 同批红（其后 18 次全套 +
  40 次隔离运行全绿——竞争窗口 ≈ 一个轮询切片，负载相关）。
- stdout：**已到达未读的完整帧或违规帧可被丢弃**——shutdown 先置 stop
  再 drain，若坏帧恰在切片边界外，close-bad 场景可间歇性回到"报
  Success"，击穿 R2-03 修复的保持合同（当前 replay 通过依赖 reader 通常
  先于 stop 解析完垃圾——时序侥幸，非不变量）。

修复要求：stop 语义 = **先排空已可读内容再退出**（drain 至 poll 为假，
有界），且对"预置数据+先行置 stop"补确定性注入单测（数据不得丢）；
不得以加大等待时长替代。该修复同时消除 stderr 缺字与 stdout 丢帧两个
表现。

### F-816-R3-02 / P3：早期错误返回路径不停自有线程

**R2 修复的收尾覆盖缺口；T-02/T-12，AC-02 当期/AC-07。**
违反"**任何**返回路径都不带活的自有线程"（R2-06 原话）：

- `auto-tool-execution/src/lib.rs` `run()`：`wait` 失败（两处）、超时分支
  `child.wait()` 失败、normal 路径 `terminate after root exit` 失败——
  四处经 `?`/直接 return 时 stdout_w/stderr_w/writer_w 未 request_stop。
- `auto-tool-client/src/lib.rs` `shutdown()`：`terminate after root exit`
  与 `wait for host shutdown` 失败在停止标志置位**前**经 `?` 返回；
  `drain_until_done` 的 writer/stderr 错误分支在 stdout reader 停止前
  返回。

均为罕见路径（wait/terminate 系统调用失败），源码证明、未伪称实机
复现（同 R2 对 F-04 的标准）。修复要求：错误返回统一先走收尾
（stop + 有界 join）再传播；可用注入单测钉住"错误路径也不悬线程"的
结构（如 fake wait 失败）。

## AC / Spec / 后续

- AC-07 partial（保持）；AC-08/T-13 本轮 **fail**。B 面按既有门 pending。
- SD-05-A 处置 R2 已确认对齐，本轮无变化；SD-01-A 的"reader 真停/失败
  具名"断言在 R3-01/R3-02 的边界路径上仍被反驳，不能发布。
- 后续：原树 work 一次修齐两项（修复窗口集中在 client reader/stderr
  停止语义 + 两 crate 错误路径统一收尾），补确定性注入回归，提交新 SHA
  后 T-13 R4 按受影响范围复审（01/02/04 与主面闭合证据可按
  未变代码/依赖复用，R3-01/02 相关面必须重验）。
