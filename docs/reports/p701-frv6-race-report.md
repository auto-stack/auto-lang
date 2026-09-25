# PLAN-701 T-01 F-RV6 根因勘定报告（有界调查件）

> 2026-09-25。下游判据仪器=auto-edit desktop_mcp 矩阵 5 连跑（跨仓只读
> 执行，AUTO_BIN 钉版工具链）。本报告=T-01 产出：复现谱+死亡形态实捕+
> 根因定罪+竞态边界勘定。

## 1. 基线复现谱（旧工具链 v0.4.2-2046-dirty，2026-09-23 构建——下游 014
##    probe 同款钉版）

| run | 结果 | 早崩 | 测试级失败 | 备注 |
|---|---|---|---|---|
| 1 | **早崩** | ✅ | —（T1-T17 全 PASS 后死） | 死亡点=T17.8 后、T8 ActQuit 首拍 |
| 2 | 完成 | ❌ | 1（T12.6 mixed→LF 转换 flake） | 全 117 检查走完含 T8 正常退出 |
| 3 | 完成 | ❌ | 1（同 T12.6 形） | 正常退出 |
| 4 | 完成 | ❌ | 2（T12.6 形 + 1） | 正常退出 |
| 5 | **早崩** | ✅ | —（T6 save E2E 轮询中死） | 死亡点=T6 |

早崩率 2/5——**与下游 1631 时代实录（2/5）精确同率**；死亡点漂移
（T17.8 尾 / T6 中）复现下游「死亡点逐跑不同」形态。

## 2. 死亡形态（run 5 实时捕获——watcher 2s 采样进程表+netstat）

- `10:53:54` 主实例 pid=26852 携 MCP 监听 9247 上线，心跳日志逐 30s
  正常推进至 `10:58:22`（死亡前 5s）。
- **`10:58:27` 两个 auto.exe 进程同秒消失**：pid=26852（主实例，带
  监听）+ pid=3236（无监听 auto.exe——另会话的 `auto build` 进程）。
- 无 panic 痕迹（全实例日志零 `panicked`）、无 WER 1000/1001、无
  fail-fast 输出——**外部 TerminateProcess 形**（进程整体蒸发，非线
  程死非 crash）。
- 矩阵侧 traceback=`src_now()` 的 console 轮询（T6 save E2E，:403）
  收 10061 拒连——**矩阵自身的全部 taskkill 均为 /PID+/T 树内作用**
  （T1-T6 段一个 kill 都还没有），排除矩阵自伤。

## 3. 根因定罪：跨会话 auto.exe 扫膛（/IM sweep 误伤）

**定性**：F-RV6「早崩」非工具链竞态——是**多会话共享主机上的并行自
动化用 `taskkill /IM auto.exe` 清扫「孤儿」时连坐**。证据链：

1. **同秒多杀**（§2）：矩阵树内 kill（/PID+/T）不可能同时命中两个
   无亲缘 auto.exe；/IM 按映像名清扫恰好全杀。
2. **清扫实践在案**：auto-edit `tools/perf/README.md:65` 明文指导
   「复跑前清 auto.exe 孤儿（`taskkill //F /IM auto.exe`）」——
   bench/perf 复跑前按映像名全扫；同仓 `bench.py`/`perf.py` 虽已纪律
   化按 PID 收（「绝不 //IM 连坐矩阵实例」注释在案），README 指导与
   临时会话的 ad-hoc 清理仍走 /IM。
3. **并行会话实活**：本勘定全程另会话在跑（watcher+tasklist 实证：
   cargo×4/rustc×4/node/bash 驱动的 `auto build`、无监听 auto.exe
   每 40-90s 生灭=他会话构建相位；run 1 前机器上已有他会话 python+
   auto.exe 实例）。
4. **死亡点漂移机制**：清扫时机=他会话 bench/perf 复跑节奏，与矩阵
   进度零相关——「死亡点逐跑不同」的直接解释。
5. **无痕机制**：TerminateProcess 无 unwind/无 WER/无 panic 钩子，
   与「渲染层静默死」观察吻合。
6. **下游史吻合**：1631 时代 2/5、2044-dirty 漂移、本会话 2/5——
   跨会话清扫暴露率随机 ±，2/5 与 1/5 都在该分布内；「T10 fresh
   实例独立复现健康」（m1-supply §5）=fresh 短命实例避开清扫窗口。

**附证（勘定过程副产物）**：run 1 主实例日志临终前的 ACTION-CONFIG
双载（26→25 actions）=他会话对同一 auto-edit checkout 跑 T10 同款
改写 app.at 触发本实例 mtime 轮询重载（`check_action_config_changed`
per-tick poll，action_config.rs:630）——多会话同 checkout 干扰的又一
实证面，非临终异常。

## 4. 竞态边界勘定（T-02 记录性件的事实面）

- 工具链内**不存在**可修的早崩缺陷面（mcp_server/iced/app 生命周期
  排除清单见附录 A）；`axum::serve(...).await.unwrap()`
  （mcp_server.rs:574）等线程死候选虽在册，但与本死亡形态无关（无
  panic 痕迹）。
- **清偿面在流程纪律**（下游其仓其计划）：README 的 /IM 清扫指导应
  改按 PID（bench.py 先例注释同款）；多会话并行期矩阵与 bench/perf
  复跑应错峰。下游 T-09 口径评估可据本报告将「早崩重跑条款」改注记
  为「外因条款」。
- **Q-3 分支判定成立**：根因落本包界外（环境/流程面），T-02 按记录
  性件执行——新工具链 5 连跑谱作为基准面交付（§5），不硬凑修复判据。

## 5. 新工具链 5 连跑谱（T-02 判据仪器）

（AUTO_BIN=v0.4.2-2115-g63901ed62——本计划 worktree 构建，含六件供料
+scroll 控制器接线；watcher 全程在线，早崩若现以「单实例孤立死 vs
同秒多杀」二分归因[工具链 vs 扫膛]。结果见计划 §9 复审记录与
baseline_summary。）

## 附录 A：排除清单（勘定过程）

- 矩阵杀毒误伤：全部 taskkill /PID+/T 树内作用（代码级审计）；无 /IM。
- T10 文件干扰假象：多实例日志交织（子实例继承 stdout）+他会话同
  checkout 写 app.at——时序证据不可直接定位死亡窗口。
- MCP 服务器线程 panic：`axum::serve unwrap`/`lock().unwrap()` 候选
  在册，但 panic 必留 stderr 痕迹，全日志零命中。
- 空闲关闭：MCP 服务器无 idle-shutdown（activity gate 仅读侧）。
- 单实例互踢：app 无 single-instance 协议（CreateMutex/FindWindow/
  WM_CLOSE 广播全零）。
- 10061=监听 socket 已关（RST 拒连非超时）→ 阻塞型假死排除。
- 外部系统面：09:32 WER 簇=前夜 BSOD 批量补报（系统未重启）；死亡
  窗口无内核/驱动事件。
