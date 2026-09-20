# PLAN-036 度量报告（T-09）

日期：2026-09-20 · 基线：lang-036 worktree（提交链 b777e6532 → 95c4ff066
→ 89c185571 → 7a2bf7521 → T-08 e2e 提交）· 口径分节随注。

## 1. Boot 时延（AC-01 度量行）

同进程内对拍（shell-pack `boot_latency_metric_row` 测试，debug 档
1 轮均值/进程内口径）：

| 轨 | 面装载耗时 | 说明 |
|---|---|---|
| 解释装载（parse + build_dynamic_component ×2） | **25.7 ms** | in-proc/解释 child 现行路径 |
| 编译装配（Default + RqProjector ×2，mount_face） | **0.8 ms** | shell-pack 编译轨（crates/shell-pack） |

**约 32× 收益**——boot 期 .at 解释装载消除（P030-D1 核销的量化
佐证）。编译轨含 ensure_covered 覆盖门扫描。

## 2. 五面全 outproc 终态 e2e 观测（AC-05 佐证）

`p036_all_faces_outproc_arm`（AUTO_DESKTOP_E2E=1，六腿全绿，
nextest 单测计时）：

| 腿 | 内容 | 结果 |
|---|---|---|
| 0 | 壳 exe attach → 五伪窗（bg/chrome/switcher/notification/dashboard） | PASS |
| 1 | 五面投影推送 → 帧全在案（overlay 懒装） | PASS |
| 2 | parity：本地 RqProjector 渲染 vs child 帧（结构全等 7 行；色彩 token 豁免——见发现） | PASS |
| 3 | launcher 独立 exe attach + 召唤快照帧（双进程拓扑） | PASS |
| 4 | 崩溃隔离：kill launcher exe → 管线回收 + 壳帧不连坐 | PASS |

全链墙钟 **≈ 0.9 s**（含双子进程 spawn/attach/五面帧 + 崩溃恢复，
debug 档）。交互往返：Advance 键盘动词事件快照 → child 帧变 <1 拍
（400ms 帧泵粒度内观测，腿4 断言路径）。

p030 四腿回归 PASS（五面 child 向后兼容——attach 分支按声明数建窗）。

帧留痕：`docs/plans/reports/assets/036/`（五面 DrawList 文本快照）。

## 3. 内存对照（口径注记）

034 量化门口径（release × 缺省 daemon × private KB）要求真机
release 档实测——**本报告未含**：e2e 为 debug 档测试进程（child
RSS 与发布形态不可比）。五面 outproc 的稳态内存行（壳 exe 五表面 +
launcher exe 两进程 private 合计）**归实机 smoke 补测**（smoke-036
脚本随 M7-c/M7-d 批次或复审档执行；034 先例 smoke-034 同型）。

## 4. 发现记录（随复审）

- **跨进程色彩解析差**：parity 腿实测 child 帧与本地渲染的色彩
  token 有 ±3/255 档差异（`text-muted-foreground` 151,163,181 vs
  148,163,183——非单纯 wire 量化舍入，疑主题感知解析的进程初始化
  差）。结构全等准则下豁免；根因细究归复审。
- launcher exe v1 无看门兵（死亡 → 下次召唤重 spawn；壳看门兵链
  不覆盖）——P036 债随注（T-07 落账）。
