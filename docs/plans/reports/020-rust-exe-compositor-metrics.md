# PLAN-020 度量报告：rust exe compositor（native 客户端臂）

- 日期：2026-09-15
- 载体：a2r 编译产物 `counter.exe`（002-counter 源，`auto build -r rust`，
  T-05 scratch020 工程本地 rust-workspace）经宿主生产 spawn 链
  （`outproc_native_exe` 发现 → `spawn_exe_child`，queue 档 =
  pac `desktop_render: "queue"` 透传）接入 `DesktopSession` 桌面会话。
- 方法：`K32GetProcessMemoryInfo`（480 先例，零新依赖），Private /
  WorkingSet 双口径；N=1/3/5 阶梯；宿主 = `__test_session` iced 会话
  （ui_desktop 同族装配）。驱动 = `p020_metrics_native_arm`
  （`AUTO_DESKTOP_E2E=1` 实机档，输出行 `AUTO020-METRICS-*`）。
- 口径注记：508 基线为**五个不同 App** 的边际；本轮载体为**同一编译 exe
  五实例**（同 App 边际——native 客户端面尚无第二 queue 覆盖样本，
  036-tetris 等 auto 降级 independent 不入本口径）。两口径的差值解读
  以"每实例边际"为准。

## 结果（2026-09-15 实测，debug 构建）

| stage | children | children Private | children WS | 边际 Private |
|---|---|---|---|---|
| base（宿主） | 0 | 3.55 MiB | 13.3 MiB | — |
| N=1 | 1 | 2.43 MiB | 12.23 MiB | 2.43 MiB/App |
| N=3 | 3 | 7.26 MiB | 36.64 MiB | 2.42 MiB/App |
| N=5 | 5 | 12.11 MiB | 61.03 MiB | 2.42 MiB/App |

- **每 App 边际内存（Private）≈ 2.42 MiB**（N=1→5 线性，拟合稳定）。
- **启动时延**：attach ≈ 24.6–37.5 ms；spawn→首帧合成 ≈ 25.3–39.6 ms
  （508 解释态 outproc 口径 25–250 ms，本臂贴近下界——子进程免 .at
  装载/VM 引导）。
- **交互时延**（协议点击 → 新帧合成回读，n=20）：
  `median = 1.501 ms，p95 = 1.559 ms`。

## 对照 508 基线

| 形态 | 每 App 边际（Private） | 备注 |
|---|---|---|
| inproc（解释态，508 缺省） | 0.86 MiB | 同进程，无协议/进程开销 |
| outproc（解释态，508） | 6.48 MiB | `auto` re-exec + VM 引导 |
| **outproc（native exe，本计划）** | **≈ 2.42 MiB** | 编译产物 + queue 命令帧 |

结论句：编译 exe 作为 compositor 一等客户端的 queue 臂，把 outproc 形态
的每 App 边际从 6.48 MiB 压到 ≈2.42 MiB（≈2.7×），启动时延贴近 508
口径下界；与 inproc（0.86 MiB）的剩余差值 = 进程隔离 + 协议/共享内存
通道的固有税——正是 508 裁定"隔离选项"语义的兑现价。

pixels 臂（independent）边际未单列：T-03 e2e 已证其链路（隐藏窗 + 真
截图），其内存税 ≈ 解释态 pixels 臂 + iced/wgpu 静态链接增量，待 native
覆盖集爬坡后（queue 为目标形态）再补专项口径。

## 复现

```bash
cd D:/autostack/.wt/lang-020/auto-lang   # plan-020-dev（≥ 6780307f7）
AUTO_DESKTOP_E2E=1 cargo t -p auto-lang --features ui-iced p020_metrics_native_arm -- --nocapture
# 载体（缺省寻址，可用 env AUTO_020_NATIVE_EXE / AUTO_020_NATIVE_APP_DIR 覆盖）：
#   target/debug/counter.exe + ../scratch020/002-counter（a2r 产物不入仓）
```
