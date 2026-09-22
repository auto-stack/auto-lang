# PLAN-690 T-06 —— remote 模式规模化实测报告（方案 2 第二阶段）

- 日期：2026-09-22
- 环境：Windows 11 实机（本机），debug 档 `auto.exe`（worktree lang-690，
  commit 基线 = plan-690-dev 分支带 IME 桥建）
- 装配：单 rqhost daemon（`auto rqhost --pipe autodesk-rqhost-p690`）+
  N 个 remote 客户端（`auto run -q --render remote`，headless iced 宿主 +
  DisplayList v2 产线帧）
- 观测源：daemon stderr `[rqhost] mem`（既有，每 ~4.5s）与
  `[rqhost] perf`（本计划新增观测行——per-client fps + 当前合成面
  DisplayList 精确 wire 编码字节 + op/text 计数，与 mem 行同拍）

## 1. 帧体积（DisplayList v2 精确 wire 编码）

| App | frame_bytes | ops | texts | 备注 |
|---|---|---|---|---|
| 001-helloworld | 51 | 1 | 1 | 单标题 |
| 004-profile-card | 358 | 7 | 2 | 卡面+位图+按钮组 |
| 003-converter | 412 | 9 | 6 | 双输入+公式联动 |
| 024-charts（tick 400ms） | 627 | 11 | 8 | 图表代表 |
| bps-gallery | 919 | 22 | 10 | 画廊代表（20+ demo 壳） |

**结论**：全部 < 1KB（帧级 wire 载荷，native 原语直落）——远低于 16KiB
shm 槽与内联回退阈值，帧体积面无超限项。

## 2. 帧率

| App | 实测 fps | 预期 | 判读 |
|---|---|---|---|
| 024-charts（400ms tick） | 2.4-2.7 | 2.5 | tick 门控端到端吻合 |
| 静态四 app | 0.0 | 0 | revision 门控稳态（零输入零帧——泵不空转） |

交互帧率：管道环测试（p690_ime_downlink_arrival_and_zero_resize_guard）
键入→帧回传在 50ms 采样粒度内完成（daemon 15ms 泵上限 ≈66fps；交互
帧率受客户端 revision 门控按需产帧，非连续光栅）。120Hz 连续光栅非
remote 模式目标形态（事件驱动产帧），如实记录。

## 3. N 窗内存曲线（daemon 进程）

| 窗数 | private | working_set | 增量 |
|---|---|---|---|
| 0（boot 待命） | ~7.0MB | ~20MB | — |
| 3（001/003/004） | 11.93MB | 40.1MB | ≈1.6MB/窗 |
| 4（+024-charts） | 12.97MB | 41.4MB | ≈1.0MB/窗 |
| 5（+bps-gallery） | 14.06MB | 42.6MB | ≈1.1MB/窗 |

**结论**：daemon 边际成本 ≈1.1-1.6MB/窗（private），5 窗总计 14MB
（P034 的 ≤100MB 门 7× 余量）。App 侧 widget 树在各自进程（headless
宿主），daemon 只持合成面——多窗规模化无内存墙。

## 4. 超限项登记

无——帧体积（<1KB）、帧率（tick 吻合）、内存（1.2MB/窗）三项均低于
既有阈值，无债登记项。子树 diff/脏区优化（计划非目标）在当前数据
尺度下无触发面。

## 5. 观测行样本（原始留痕）

```
[rqhost] perf `App` fps=2.7 frame_bytes=627 ops=11 texts=8   # 024-charts
[rqhost] perf `App` fps=0.0 frame_bytes=919 ops=22 texts=10  # bps-gallery
[rqhost] perf `App` fps=0.0 frame_bytes=412 ops=9 texts=6    # 003-converter
[rqhost] perf `App` fps=0.0 frame_bytes=358 ops=7 texts=2    # 004-profile-card
[rqhost] perf `App` fps=0.0 frame_bytes=51 ops=1 texts=1     # 001-helloworld
[rqhost] mem private=14060KB working_set=42616KB             # 5 窗 daemon
```
