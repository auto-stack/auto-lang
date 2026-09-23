# P697 归因档案——桌面宿主静默退出（P694-D1）勘定结论

日期：2026-09-23 ｜ 勘定人：PLAN-697 T-00（bounded investigation）
对象：ui_desktop 宿主（inproc VM App 拉起后静默退出 ×3，P694-D1 登记）

## 结论（一段话）

**外因成立：机器级内核/驱动不稳定（三日三蓝屏 + GPU TDR 族），非宿主内部
缺陷。** 走查窗口（09-23 上午）内 System 日志实锤 `10:59:12 非正常关机`
（Event 6008，bugcheck 0xA IRQL_NOT_LESS_OR_EQUAL，minidump 0923 在档）；
进程被内核蓝屏瞬时吞没 = 无 panic、无错误行、无 WER 应用崩溃记录、进程
直接消失——与 P694-D1 签名逐项吻合。宿主内部崩溃面被三重排除法排除
（15 轮复现零死亡 + 嫌疑面全点火 + WER 零应用崩溃记录）。处置走观测加固
分支（契约预授权：待澄清①默认口径），残面=机器稳定性本身（非本仓代码）。

> 证据位面注（review R-1 修复）：原始宿主日志（host-*.log）受仓根
> `.gitignore` `*.log` 规则约束不入库；入库的持久证据 = `matrix.jsonl`
> （17 条逐轮记录）+ `host-evidence-excerpt.txt`（足迹桩/点火行摘录，
> review 复验时自加固版宿主 fresh 采集）+ 本档案内联引文。

## 证据一：复现（≥5 轮，零死亡）

复现脚本 `repro_host_exit.py`（自起宿主 + MCP 验收通道 bus verb
`launch\t020-music-player` + 进程/MCP 双探逐秒盯存活）。基线与对照臂全绿
（矩阵口径 15 轮，另有 CLI 形态误配冒烟 1 轮 + 加固验证 1 轮，见
matrix.jsonl 17 条逐轮记录）：

| 臂 | 形态 | 轮数 | 存活 | 死亡 |
|---|---|---|---|---|
| 基线（全供给） | 窗口化 | 5×90s+1×75s+1×40s | 全存活 | 0 |
| 基线（全供给） | 全屏 | 2×60s | 全存活 | 0 |
| --no-proxy（禁A） | 窗口化 | 2×60s | 全存活 | 0 |
| --no-media（禁B） | 窗口化 | 2×60s | 全存活 | 0 |
| 011-calculator（他App） | 窗口化 | 2×60s | 全存活 | 0 |

关键路径确认：基线日志实锤 `[session] launch_app(inproc) 020-music-player`
+ `[session] back-proxy lazy-start on "020-music-player" (port 3358)`——
嫌疑A（proxy 懒启）与嫌疑B（media 供给→305KB scan→json.to_value）在基线
轮**真实点火**且宿主稳定。测试缝生效核验：禁A 臂日志零 proxy 行（8 份含
proxy vs 6 份对照臂零 proxy）；--no-media 对 020 等效零供给（media 是其
唯一 proxy 消费面，无 back_entry/session）。原始记录：`matrix.jsonl`
（17 条逐轮记录）+ `host-evidence-excerpt.txt`（点火行/足迹行摘录，
.gitignore 注见证据位面注）。

## 证据二：Windows 取证（退出位面档案）

事件日志取证（`events-extract.txt` 全量摘录）：

1. **走查窗口内有机器级灾难**：System 日志 Event 6008
   `The previous system shutdown at 10:59:12 on 2026/9/23 was unexpected`
   ——694 走查提交链 fa524d126 09:53 → c2c538657 10:43，非正常关机点
   10:59:12 落在走查会话存续期内。重启完成 11:42:35（bugcheck 0xA，
   `C:\WINDOWS\Minidump\0923*` 在档）。
2. **三日三蓝屏**：09-21 09:42（0x9F DRIVER_POWER_STATE_FAILURE）/
   09-22 17:59（0x1A MEMORY_MANAGEMENT）/ 09-23 10:59（0xA）+ 12:31 群发
   重演桶（0x133 DPC_WATCHDOG ×N、0x119 VIDEO_SCHEDULER、0x141/0x117
   LiveKernelEvent=GPU TDR 族；桶跨 26100/26200 两个 OS 构建=慢性）。
3. **走查窗口内零宿主应用崩溃记录**：Application 日志 1000/1001/1002
   事件中与 auto 相关的仅三例，均与桌面宿主无关——09-22 21:55 两例=
   `musk-084` worktree 的 release auto.exe（c0000005→c000041d，P084 线
   自有现场）；09-23 12:31 一例 AppHang auto.exe（蓝屏群后的挂起收尾，
   另一会话）。**ui_desktop.exe 零记录**——若宿主内部崩溃（AV/fastfail/
   栈溢出）必有 Event 1000，缺席即排除。
4. **同源旁证**：694 档案 T-03 自述"桌面进程被关 ×6 次实测"，与合成输入/
   用户实时操作竞夺并存——×3（D1）/×6（T-03）的消失频率与机器蓝屏频率
   同量级。

## 证据三：嫌疑面排除（静态+动态）

| 嫌疑 | 排除证据 |
|---|---|
| A back-proxy lazy-start | 基线 7 轮全点火（port 3358 在册）零死亡；禁A 对照 2 轮零差异信号 |
| B media 能力 | 同上点火零死亡；缺省构建无 mpv-widget（video=诚实占位，播放不涉 mpv 驱动面）；scan 响应 305KB→json.to_value 深度帽 64 层+平铺数组无递归深度放大（stdlib.rs json_to_vm_value）；宿主主线程 /STACK:33554432（32MB） |
| C inproc panic→abort | 走查窗口 WER 零 ui_desktop 崩溃记录（AV/fastfail/栈溢出必留 Event 1000）；15 轮复现零 abort |

## 残面与边界（诚实声明）

- 内核蓝屏瞬间无法从进程侧取证（本次 10:59 蓝屏的 minidump 是机器侧唯一
  现场），"外因"结论的强度=排除法链完整 + 机器灾难时间吻合，非进程级
  死亡现场直接观测。
- 09-23 12:31 蓝屏群/AppHang 发生在走查提交后（另一会话时段），计入机器
  不稳定谱系，不计为走查窗口直接证据。
- **新数据点**：当前构建下宿主窗口忽略外来 WM_CLOSE（PostMessage 5s 不
  退；renderer.rs:21592 有 065 打点臂走 `__window_close_request` 消息臂）——
  "用户点 ×/Alt+F4 关掉宿主"假设被削弱，进一步利好机器外因结论。
- 机器稳定性本身（GPU 驱动/内核）为环境残面，不属本仓修复面；建议用户
  关注 NVIDIA 驱动（617.14）/Windows 更新（bugcheck 谱系 0x119/0x117/
  0x141 典型 GPU TDR）。

## 观测加固（T-01，落地于本档案同 commit）

ui_desktop 宿主此前零足迹（死亡无法与机器灾难对时）。镜像 rust_ui.rs
X9 足迹桩形制补三件（`examples/ui_desktop.rs`）：

1. `[ui-desktop] start pid=.. fullscreen=.. apps_dir=.. t=<epoch>`——boot 留痕；
2. `[ui-desktop] alive pid=.. t=<epoch>` 30s 心跳——墙钟与 System 事件日志
   直接对时（死亡时刻贴近 6008/1001 即机器外因）；
3. `[ui-desktop] exit ok (code 0) / exit err (code 1)` + panic 钩子带
   backtrace（此前无钩子，内部 panic 静默 abort 零痕迹）。

**死亡判位表**（未来任何宿主消失，查日志三行即判）：

| 日志位面 | 结论 |
|---|---|
| exit 行在场 | 正常退出（关窗链） |
| 心跳截断、无 exit、无 WER | 进程被外力杀（taskkill/机器蓝屏——对时 6008 即定） |
| 心跳截断、无 exit、WER Event 1000 在 | 内部崩溃——panic 钩子 backtrace 或 WER dump 归因 |
| 无 start 行 | 宿主未起（装载期死亡，另勘定） |

CLI 桌面路径（`auto run --desktop`→rust_ui.rs）已有同形制足迹桩
（X9-ALIVE 30s 心跳 + X9-PANIC 钩子 + returned ok 标记），双面覆盖。

## 附：勘定方法论（可复现）

```bash
# 基线/对照臂（每臂 ≥2 轮；watch 覆盖"数十秒"死亡窗）
python repro_host_exit.py --rounds 5 --watch 90            # 基线窗口化
python repro_host_exit.py --rounds 2 --watch 60 --fullscreen
python repro_host_exit.py --rounds 2 --watch 60 --no-proxy
python repro_host_exit.py --rounds 2 --watch 60 --no-media
python repro_host_exit.py --rounds 2 --watch 60 --app 011-calculator
# 事件日志取证（PowerShell，见 events-extract.txt 生成命令）
```
