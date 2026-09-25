# PLAN-702 T-06 实测证据（2026-09-25）

## 探针腿（已完成，AC-01/AC-02 PASS）

载体：`examples/ui/032-handler-async-probe`（VM 桌面，本分支 release
auto.exe 12:39 构建）。采集脚本：`collect_probe_evidence.py`（本目录），
两轮实测：

### Run A — 65s 慢端点（超出 reqwest 客户端 30s 超时的边界形态）

- `probe_vm_stderr.log`（65s 轮）：`[VM-PARKED] handler_App_Init parked`、
  `resumed to completion` 在案；`timed out in call_fn_by_name` **零条**。
- 等待全程可交互：t+4s bumps=5/ticks=32 → t+30s bumps=31/ticks=143 →
  t+60s bumps=61/ticks=270（每秒一次 Bump 全部即时生效，250ms Tick
  持续渲染）——**AC-01 达成**。
- 结果形态：`{"error":"error sending request...","status":0}` 落账——
  65s > 客户端 30s 超时，worker 错误体以**数据**形态回到 handler
  （.at 可捕获），不再是旧驱动的 VM 级 RuntimeError（E7 的"catch 接不住
  超时"在本形态下结构性消除）。**契约变更注记**：等待上限从"30s 忙等
  硬超时"变为"reqwest 客户端超时以错误体落账"。

### Run B — 20s 慢端点（客户端超时内的干净值回填）

- 最终态 21.5s：`result = {"ok":true,"slow":702}`（服务端 body 原样
  回填），bumps=20（20 次交互全部响应）、ticks=95（持续渲染）——
  **AC-02 达成**。
- 汇总 `probe_ac01_ac02_report.json`；初始快照
  `probe_snapshot_initial.txt`。

## musk PickFolder 腿（机制已证，人在环走查移交）

E7 原始事故的端到端复跑需要：musk serve（后端 rfd 代弹）+ musk front
VM 桌面（auth/daemon 依赖）+ **人持原生对话框 >60s 后选择**——末环节
本质人在环，无法全自动取证。机制面（widget handler → 异步等待 → park →
窗口交互 → 恢复泵续跑 → 落账）已由探针腿在同一驱动路径全链证实。

复跑手册（人工，~5 分钟）：

```sh
# 1. 本分支 release auto.exe（已构建：.wt/lang-702/auto-lang/target/release/auto.exe）
# 2. musk 后端（主检出 musk.exe 即可——PickFolder 端点未变）
cd D:/autostack/auto-musk/backend && target/release/musk.exe serve
# 3. musk front VM 桌面，AUTO_EXE 指向本分支 exe
cd D:/autostack/auto-musk
AUTO_EXE=D:/autostack/.wt/lang-702/auto-lang/target/release/auto.exe \
  <auto.exe> run -r vm   # 按 musk front 启动约定
# 4. workspace 选择器 →「打开文件夹」→ 原生对话框滞留 >60s：
#    断言窗口全程可交互（悬停/点击均有响应、无灰死）；
#    选中文件夹后 workspace 正常注册切换；vm 日志零
#    "timed out in call_fn_by_name"、有 "[VM-PARKED] ... parked/resumed"。
#    自动化变体：对前台对话框发 ESC = 取消路径（handler 恢复、静默不动），
#    同样证 resume 机制。
```
