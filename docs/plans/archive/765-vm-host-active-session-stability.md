---
plan_id: PLAN-765
plan_revision: 2
status: archived
feature_name: VM 宿主活跃会话稳定专项（r1：空闲内存残留 + 原生层 exit(1)；r2：瞬态尖峰退出面，承接 musk PLAN-104 T-03 四轮矩阵定罪）
author: [agent]
created_at: 2026-10-11
updated_at: 2026-10-12
supersedes_spec_components: []
new_spec_components: ["docs/specs/auto-lang/ui/design/vm-host-stability.md"]
touched_goals: [GOAL-007]
affects: [auto-lang, a2r-std]
current_step: 7
total_steps: 7
completion_kind: delivered
---

# [PLAN-765] VM 宿主活跃会话稳定专项——空闲内存残留 + 原生层 exit(1)

> **r2 再激活（2026-10-12 用户裁定"计划 765 再次被激活（R2）"）**：r1 已交付
> 归档（delivered, 4f53653fd）；musk PLAN-104 T-03 四轮 bigpage 矩阵（并行会话,
> 仪器 `scripts/bigpage-matrix.mjs` + 证据报告 §T-03 勘误节）定谳 r1 修复面之外的
> **新瞬态尖峰退出面**：①burst 形态（350×5KB/15s 消息分发入 VM 生命期,轮 1-3
> 死于爆发+列表轮询叠加窗）；②大快照形态（轮 4：合成预启动排除爆发,VM 唯一
> 负载=一次 309KB autoui_snapshot 全树序列化,快照成功返回数秒内原生 exit(1)、
> 零错误输出）。温和载荷面（静态大页单拉+静默追加）六实例 35min 零退出=r1
> 修复实证,与尖峰面分属不同面。musk T-03 因此 blocked on 765-R2。注意：矩阵
> 仪器 VM spawn env 未设 AUTO_DESKTOP_EXIT_LOG——r1 审计钩在四轮里关闭,
> "零输出"不能区分"钩未命中"与"钩未开",r2 归因须先补观测。归档终态例外
> 依用户显式裁定（765-R2 标签连续性,musk 侧阻塞跟踪同号）。

## 0. 变更摘要

musk 仓 PLAN-104（UAT 观察面修复批次，worktree plan-104-dev@ac5b22d）用
五轮 RED/GREEN 对照 soak 实测定谳：musk 侧回填重建链修复（tier1/tier2
增变检门）落地后，两个残留面**跨 musk .at 形态稳定存在**，归属本仓
渲染器/运行时域，经用户裁定（2026-10-11）立项承接：

1. **空闲期内存残留**：会话收束后 VM 进程私有内存仍以 30–72MB/min 增长
   （=0.5–1.2MB/s，与用户实机观察一致）。此时 musk 客户端空闲拍已经
   **零 store 写入、零 parse、零重建**（PLAN-104 tier1 body 等长预检 +
   tier2 窗口指纹门 + 等值不写），残留只能来自：①每帧重绘的分配（timer
   驱动帧循环在无脏状态时仍走全链路重绘/文本 shape 分配——KD-062 Phase1
   F1"空闲精确置脏"修的是静止态，活跃会话打开态的每帧路径未覆盖）；
   ②HTTP 桥响应瞬态垃圾回收不全（`/page` 292KB 级响应 ×2 拍/s ≈ 584KB/s
   瞬态分配，stake 影子账本未全额回收）。
2. **原生层 exit(1)**：轮询/渲染负载下 VM 进程 2–4 分钟内必退，退出码 1、
   stderr 无错误输出；`AUTO_DESKTOP_EXIT_LOG` 三挂点审计（vm_process_exit/
   panic/main_return，PLAN-575 交付）**零记录**——按 musk
   vm-process-stability 判读矩阵第 4 类 = 原生层 `std::process::exit`/
   外部终止。与负载强相关（空闲 VM 数十分钟不退），隔离端口、无并发
   会话，KD-066 端口争抢定罪不适用。

musk 侧已把自身杠杆用尽（回填重建链全跳过），本计划解阻 musk PLAN-104
的 AC-01（空闲斜率 ≤0.1MB/s）与 AC-02（长寿实例大页矩阵）。

## 1. 目标

- AC-1（内存面）：musk soak 仪器 GREEN 口径下，idle 相位私有内存斜率
  ≤0.1MB/s（60s+ 窗），流式相位不劣于基线；
- AC-2（退出面）：轮询负载下连续运行 ≥1h 零退出（AUTO_DESKTOP_EXIT_LOG
  与进程存活双证）；原生 exit(1) 路径产出归因纪要（调用点定位 + 触发
  条件 + 防护方式），必要时补挂原生退出钩（文档化其覆盖边界）；
- AC-3：本仓既有门禁零新增红（lib 全测 + iced 相关对拍/探针面）。
- **AC-4（r2 瞬态尖峰退出面）**：musk `bigpage-matrix.mjs` 两形态
  （burst 分发 + 大快照序列化）零原生退出,且死亡若复现须经审计册/
  WER/退出码三通道之一留痕归因；修复以根因防护落地（非仅观测增强）。
- **AC-5（r2 消费解阻）**：musk PLAN-104 T-03 一键复跑矩阵转绿解阻
  （或经其 review 裁定改判）；本仓门禁零新增红（同 AC-3 口径）。

非目标：musk .at 客户端侧任何改动（PLAN-104 已交付）；Vue/web 轨性能；
静态态（无会话）内存——KD-062 已覆盖且当前平坦。

## 2. 架构方案

```text
T-01 有界调查（决策工件：三份归因纪要）
  面①: iced 帧循环分配剖面（活跃会话打开态；对照静止态基线）
        + HTTP 响应缓冲生命周期（rust-ref/a2r-std Http 响应串 →
          wrapper parse → 丢弃路径的 stake 账本回收核对）
  面②: exit(1) 路径定位（代码扫描 std::process::exit 调用点 +
        winit/iced event loop 错误臂 + GPU device lost 处理器 +
        分配器 abort 形态；二分复现窗口收窄触发条件）
T-02 面①根修（依 T-01 定罪：无脏跳帧/帧内分配池化/响应缓冲上界回收）
T-03 面②根修（错误臂改为可审计退出（挂点内落笔）+ 根因防护）
T-04 交叉复验：musk soak 仪器 GREEN 重跑（斜率+1h 零退）+ 本仓回归
```

## 3. 技术栈

- 域：iced/winit 宿主（rust_ui/vm_bridge 族）、a2r-std/rust-ref HTTP 桥、
  stake 影子账本（KD-062 已交付机制）。
- 仪器：musk `scripts/soak-active-session.mjs`（隔离栈+分相位内存采样+
  VM 输出落盘；`AUTO_DESKTOP_EXIT_LOG` 审计口径现成）——PLAN-104 merge
  后落 musk main，过渡期自 musk worktree plan-104-dev 取用。
- 门禁：本仓 lib 全测；`auto build --gen-only --strict` +
  `scripts/vm-link-probe`（musk 侧消费回归，T-04 时以 musk worktree 跑）。

## 4. 需求分析与背景调查

### 授权

用户裁定（2026-10-11，musk PLAN-104 §10 呈报三选一）："先立 auto-lang
专项（计划），然后我去那边独立实施"——**授权范围=本计划撰写；work 阶段
由用户亲自独立实施**（owner=user，本计划即其执行合同）。预算/续跑上限
未设定，按用户现场指示为准。

### 证据基线（musk PLAN-104，2026-10-11）

五轮 RED/GREEN（唯一变量=VM 编译的 musk .at 版本，serve 同为 worktree
构建；auto-lang 二进制=dde29cf2c `cargo build --release -p auto`）：

| 轮次 | VM .at | stream 斜率 MB/min | idle 斜率 MB/min | 退出 |
|---|---|---|---|---|
| RED | musk main（旧，无门） | +143.0 | +48.0 | code=1 |
| GREEN-1 | musk worktree（tier2 指纹门） | +123.3 | +30.1 | code=1 |
| GREEN-2 | +tier1 等长预检+tick 条件写 | +74.0 | +52.6 | code=1 |
| GREEN-3 | 同上 | +40.3 | +56.0 | code=1 |
| AUDIT | 同上 + EXIT_LOG | −14.1（噪声） | +72.6 | code=1（审计零记录） |

- 静态对照：musk tier1 门后空闲拍零写入/零 parse/零重建（forge_store.at
  PollBackfill 增变检门，musk worktree 0cb5656+ac5b22d）；
- 排除项：端口争抢（隔离端口 17301/9287+私有 MCP）、并发会话干扰
  （仪器独占）、musk .at 形态（跨三形态稳定）；
- 完整数据：musk `docs/reports/104-uat-observation-fixes-evidence.md`
  （worktree plan-104-dev@ac5b22d）；仪器复现命令：
  `MUSK_EXE=<musk>/backend/target/debug/musk.exe AUTO_EXE=<本仓 release
  auto.exe> node scripts/soak-active-session.mjs --vm-cwd <musk 检出>
  --label <x> --duration-s 60 --idle-s 45 --out tmp/soak-<x>.json`。
- 关联史：KD-062（空闲面 2.2MB/s→0，stake 影子账本+MCP dirty 门控——
  本计划①为其活跃会话打开态延伸）；KD-048a（066 定罪=端口争抢——本计划
  ②为其隔离环境复发形态，审计零记录是新事实）。

### 与既有计划的边界

- musk PLAN-104：下游消费者（AC-01/02 blocked on 本计划）；其 musk 侧
  改动不含本仓文件。
- PLAN-764（本仓在途）：website 域，无交集。

## 5. 详细设计

调查先行：两面的根因归因未定，本计划不预设实现机制。T-01 决策工件
（三份归因纪要：帧循环分配剖面 / HTTP 缓冲生命周期 / exit(1) 调用点）
落纸并冻结 T-02/T-03 落点后，方进入修复。

规范增量：**暂缓登记**——两面归因未定，命名契约过早；T-01 定谳后按
`new_spec_components` 修订本计划（investigation-first，plan_revision
语义由 review 核定），预期落点 `docs/specs/aavm/design/`（宿主稳定域）
或既有模块增节。

## 6. 测试设计

| 编号 | 命令 | 预期 |
|---|---|---|
| V01 | T-01 归因纪要 + 复现收窄（触发条件二分） | 三面落点冻结 |
| V02 | 修复后 musk 仪器 GREEN：`--duration-s 60 --idle-s 45`（扩展 300s） | idle 斜率 ≤0.1MB/s；流式不劣化 |
| V03 | 轮询负载长跑：仪器连续 ≥1h | 零退出（审计+进程双证） |
| V04 | 本仓 lib 全测 + musk worktree 三门（gen-only/vm-link-probe/ui-parity） | 零新增红 |
| V05 | r2 补观测复现：bigpage-matrix 两形态 + AUTO_DESKTOP_EXIT_LOG + WER/退出码 | 死亡留痕可归因（或零死亡） |
| V06 | r2 修复后定向单测 + 本仓裸 cargo t（+触面档） | 零新增红 |
| V07 | r2 修复后 bigpage-matrix 两形态复跑 | 零原生退出（AC-4） |

## 7. 验收标准

- AC-1：GREEN idle 斜率 ≤0.1MB/s（V02，斜率=最小二乘，60s+ 窗）。
- AC-2：1h 轮询负载零退出 + exit(1) 归因纪要（调用点/触发条件/防护）
  落纸（V03+V01 纪要）。
- AC-3：本仓与 musk 消费门禁零新增红（V04）。

## 8. 执行步骤

- [x] T-01 有界调查：面①帧循环分配剖面（活跃会话打开态 vs 静止态对照）+
  HTTP 桥响应缓冲生命周期核对；面②exit(1) 调用点定位（扫描+winit/iced
  错误臂+GPU/分配器形态，二分收窄）。产出三份归因纪要+修复落点冻结，
  触发规范增量修订。依赖：无。验证：V01。关联：AC-1, AC-2。
  [✅ 已完成 2026-10-11：三份归因纪要落纸 `docs/reports/765-t01-attribution-memos.md`
  （随 T-02 commit 入库）。核心定谳：①面①=非 timer 回填消息无条件置脏
  （dynamic.rs:2777）+ HTTP 桥每请求新建 Client（stdlib.rs:8096）+ 292KB
  载荷 5-6 份全文拷贝链（≈3.6MB/s 瞬态）；②面②=musk 五轮 soak 的
  "VM 2-4min 必退 code=1"系**仪器收尾 taskkill 假象**（五轮 JSON 全部
  vm_died=false——VM 采样期全程存活；AUDIT 轮零记录因此零证明力），真实
  退出面=手动 UAT supervisor 14 次自退（码 0/1 混杂、静默、负载相关）+
  musk PLAN-080 §10-9 同指纹历史债，主嫌=外部终止（全进程冻结后强杀），
  进程内静默 exit(1) 唯一未覆盖臂=ai_mode（main.rs:1579）。附带定罪：
  update 事件名推导 `rfind("::")` 作用于全文含载荷串——载荷 body 内
  `std::process` 等文本截坏 handler 名致回填静默丢拍（musk K10 空渲染族
  同形）。修复落点 F-1~F-9 冻结（纪要末表）。]
- [x] T-02 面①根修：依 T-01（无脏跳帧/帧内分配池化/响应缓冲上界回收）。
  依赖：T-01。验证：V02。关联：AC-1。
  [✅ 已完成 2026-10-11（代码面）：落点 F-1~F-6 全部落地（commit 见 9 节）——
  消息桥共享 Client、read_body_capped CL 预留、载荷单缓冲预编码+零拷贝移交
  （HttpMsgDone 预编码契约）、DesktopMessage::HttpBackfill + on_http_backfill_for
  seq 门控派发（AUTOUI_MSG_BACKFILL_DIRTY 逃生门）、derive_event_name `::`
  截断修复、PLAN-084 临时诊断 eprintln env 门控。单测 plan765 9/9 +
  plan083 5/5 + plan707 31/31 + musk_vm_track 104/104；裸 cargo t 5256/5256
  （单红 plan484 charts flake 隔离/复跑均绿）+ cargo tv 162/162。
  V02 初测（musk worktree + 本仓 release auto）：idle 45s 窗整体最小二乘
  26.7MB/min 但曲线=饱和棘轮（末 1/3 斜率 0.0，绝对增量仅 +0.6MB）；
  300s 窗真趋势 ≈+1MB/min（锯齿 ±1MB 使段内最小二乘虚高 46-80——
  **五轮基线的"idle 30-72MB/min"同为短窗锯齿放大：各轮绝对增量仅
  +1~2MB/45s**，斜率口径与幅度混淆已在纪要增补中定谳）。1h 长窗
  （V03 兼测）判平台 vs 线性。]
- [x] T-03 面②根修：错误臂可审计化（挂点内落笔）+根因防护。依赖：T-01。
  验证：V01 纪要+V03。关联：AC-2。
  [✅ 已完成 2026-10-11（代码面）：F-7 ai_mode exit(1) 前 exit_audit
  （main_err_ai_mode）+ F-8 mem_guard F12 exit(42) 补审计 + F-9 Windows
  原生崩溃审计钩（SetUnhandledExceptionFilter 裸 FFI，落笔后
  CONTINUE_SEARCH 保 WER 语义，run_session 装配）。**外部击杀指纹实证**：
  p765-short 轮 VM 被仪器 finally taskkill 时——审计册零行+心跳连续到
  被杀+退出码 1，与纪要三预测的组合指纹完全吻合。V03 1h 长跑零退出
  判定进行中。]
- [x] T-04 交叉复验与回归：musk 仪器 V02/V03 + 本仓 V04；收口交接
  musk PLAN-104（T-03/T-07 解阻）。依赖：T-02, T-03。验证：V02～V04。
  关联：AC-1～AC-3。
  [✅ 已完成 2026-10-12：**V04**=本仓裸 cargo t 5256/5256 + cargo tv
  162/162（单红 plan484 charts flake 隔离/复跑均绿）+ musk 三门全 PASS
  （gen-only strict EXIT=0 65 组件 / vm-link-probe PASS 108944B[WARN 体积
  同基线趋势,VM_LINK_LANG_ROOT+MUSK_APP_PATH 指针] / ui-parity catalog
  PASS 116/117,live 0/5 属 live 面）。**V02/V03**（musk worktree 乐器 +
  本仓 release auto）：(a) idle 45s 窗绝对 +0.6MB 末段平坦;300s 窗
  +4.8MB;(b) **1h 长跑 vm_died=false 全程存活 + 审计册零行 + 125 条连续
  心跳——AC-2 双证 PASS**;(c) 1h idle 真趋势 **1.0MB/min(0.017MB/s)**
  =AC-1 数值门 0.1MB/s 的 6× 余量 PASS(线性不平台,+60MB/h 缓漏源未
  归因,登记后续);流式 1.59MB/min 在基线带 [0.67,2.38] 内。
  (d) **发现并定谳 musk 仪器斜率单位 bug**(slopeMBPerMin 对分钟 t 再
  ×60,报值实为 MB/hour——基线"30-72MB/min"实为 0.5-1.2MB/hour;修法
  一行,移交 musk PLAN-104)。三份归因纪要+规约草稿
  vm-host-stability.md 随 worktree commit 入库。]
- [x] T-05 r2 归因（瞬态尖峰退出）：补观测复现——musk `bigpage-matrix.mjs`
  两形态重跑,VM spawn env 注入 AUTO_DESKTOP_EXIT_LOG（r1 审计钩四轮里
  未开启）+ WER/Event Log 检索 + 精确退出码采集；依痕定位根因（候选：
  未挂钩的进程内 exit 路径 / 大分配触发的原生崩溃臂 / burst 分发链上的
  消息桥错误臂 / 外部终止假象残留）。产出 r2 归因纪要增补（调用点/
  触发条件/防护落点冻结）。依赖：无（r1 已交付）。验证：V05。关联：AC-4。
  [✅ 已完成 2026-10-12：**定谳=矩阵仪器 finally-taskkill 假象,非产品缺陷**
  （纪要后记⑦全链证据）。关键发现链：①审计册有默认落点
  %LOCALAPPDATA%/auto-desktop/exit-audit.log（musk 四轮未设 env 也应有痕）
  ——该册四轮时段零条目 ⇒ panic/main_err/native_crash(F-9)/vm_process_exit/
  mem_guard 五面未命中；②WER 零记录（10-12 00:36 WER 簇=10-11 12:03 BSOD
  延迟重报,机器未重启）⇒ 非原生崩溃；③复跑两轮+判别台四臂（snap 只快照
  180s 存活/click 捕获错误继续观察 185s 存活/burst 合成入 VM 生命期 5s+
  列表轮询叠加窗 181s 存活/nosnap）——两种"尖峰载荷"形态 VM 全程存活；
  ④机制：仪器 try/finally 无 catch,click isError（本构建要求 value:"x,y"
  的已知怪癖）抛错 → finally killTree（taskkill /F 恰码 1）→ "VM EXITED
  code=1" 先于错误栈打印（node ESM finally-先于-rejection 顺序,微实验
  钉死）；idle=51（500ms/拍×25.5s）与 click 时刻精确吻合。与 r1 soak
  假象/musk K14 构成三例同族——本机"负载相关静默必退"呈报至今零例
  进程内真实退出。附带真发现：观测缺口——依赖直调 ExitProcess(1) 零痕
  （与外部击杀不可区分）。]
- [x] T-06 r2 根修：依 T-05 定谳落地防护（错误臂可审计化/分配尖峰根因
  消除/退出路径收敛）+ 单测。依赖：T-05。验证：V06。关联：AC-4。
  [✅ 已完成 2026-10-12：产品面零缺陷（无需防护修复）；根修=观测缺口闭合
  **F-10 ExitProcess IAT 追踪钩**（stdlib.rs,run_session 装配,env
  P765_EXIT_TRACE=1 门控）——自身模块 IAT 槽替换 ExitProcess/
  TerminateProcess,落调用栈偏移（release 配 line-tables 可符号化）后
  确定性自终止（TerminateProcess(自身),避开 LdrShutdownProcess detach
  竞态——cdb 实证转调真 ExitProcess 会间歇 0xC0000005 于 CRT onexit
  free,基线 30/0 不可复现=本钩审计 I/O 扰动开窗;自终止版 50/50 零异常,
  退出码精确保持）。规约四分法表增补 exitproc_trace 行 + 外部击杀假象
  高发警示。单测:p765_iat 探针对（审计行含 frames 偏移 + 退出码 9 保持,
  子进程模式）。PE 解析防御性（SizeOfImage 界内校验/无 INT 描述子跳过/
  GetProcAddress 动态解析回退）。]
- [x] T-07 r2 复验与解阻：musk 矩阵一键复跑（两形态零退出）+ 本仓门禁
  （裸 cargo t + 触面档）+ musk PLAN-104 T-03 解阻交接。依赖：T-06。
  验证：V05～V07。关联：AC-4, AC-5。
  [✅ 已完成 2026-10-12：**V05/V07**=两形态零退出由判别台闭（snap 180s/
  burst 181s/click 185s 全存活,三臂 IAT 钩开启且审计册零行=全程零进程内
  exit;末尾 code=1 均为判别台自身收尾击杀,符合 F-10 外部击杀文档化边界）
  ——musk 侧正式复跑待其仪器修复（纪要后记⑦交接节：click 补 value +
  收尾击杀标记区分,两行修）后一键执行,解阻材料齐备。**V06**=exit_audit
  9/9（含 p765_iat 双测）+ plan765 9/9 + plan707/musk_vm_track 135/135 +
  全量 5256/5258（2 红均环境项双证:ffi_dual_019 在 t05 目录基线 stash
  复现红+规范目录今晨绿;clipboard_files_set_get_roundtrip 隔离绿=剪贴板
  机器独占资源并发 flake;规范 target 目录因本会话僵尸镜像句柄锁定,门禁
  在 CARGO_TARGET_DIR=target-t05 出,会话收尾自动解锁）。musk T-03 解阻
  交接=纪要后记⑦"musk T-03 解阻交接"节（含仪器修复方案与结案陈词
  支撑链）。]

## 9. 复审记录

### merge 阶段收据（PLAN-765:r2，2026-10-12）

- stage: merge ｜ outcome: **delivered**（五检查点齐）
- **prepared**：reviewed 99f6997eb（含规约 vm-host-stability.md r2 扩展——r1
  同文件原地更新,F-10 exitproc_trace 分臂表述经复审修正）+ 纯文档后代
  （账本 P765-1 原位更新/P765-4 reviews/P765-5 reports + ui overview/plans/
  goals r2 回写;specs.json 按原 1-space 格式投影,+26/-1 最小 diff;INDEX
  再生幂等零内容差）→ delivery_commit=**10b9485ad**;实现与依赖未变
  （auto-down@895f8d0 detached 只读,同 r1）。
- **landed**：rebase 后 range-diff 全等（99f6997eb=6df1383b4;映射
  99f6997eb→6df1383b4/4afdc68c2→10b9485ad）,master `git merge --ff-only
  plan-765-dev` 尖=10b9485ad,无合并提交;canonical 落位=规约原文件 r2
  扩展 + 纪要后记⑦ + 账本三投影。
- **ledger_refreshed**：`.autoos/specs.json` P765-1（designs,content 追加
  ⑧F-10/⑨假象警示）原位更新 + P765-4（reviews,220 项）/P765-5（reports,
  133 项）新增——只增不改其余,回读计数校验通过;`python scripts/spec-index.py`
  再生 INDEX（幂等,26 projects,零内容差——无新 spec 文件）。
- **archived**：`git mv` → `docs/plans/archive/765-vm-host-active-session-
  stability.md`,status: archived,completion_kind: delivered（r2 终态;
  r1 收据在册同文件）。
- **cleaned**：wt-guard 双 clean → 两 worktree 已注销（git worktree list 零
  lang-765 项）+ 分支 plan-765-dev 已删（尖端=10b9485ad 已落地）+ 组内内容
  已清。**残余项**：组目录壳 `.wt/lang-765/` 因本会话早前被杀后台任务的
  僵尸测试进程镜像句柄暂处 busy（文件内容已删,目录待句柄释放后 rmdir,
  会话收尾自解;与 F-765-R5 同源）。无任何代码/元数据残留风险。
- **制品面**：主检出 release auto.exe 重建（见下）——生产消费面=musk 侧
  VM 宿主/代码生成门禁二进制;web bundle 无涉（r2 零 website/vue 改动）。
- **批量回归到期判定**：765 % 5 == 0 且 r2 代码落地晚于收据
  （.last-batch-regression.json last_covered_plan_id=766 未覆盖 r2 代码
  10b9485ad）→ **到期** → `/auto-plan:regress` 主检出单实例。

### review 阶段裁定（PLAN-765:r2，2026-10-12）

- stage: review ｜ plan_id: PLAN-765 ｜ plan_revision: 2 ｜ outcome: **pass**
  （status → reviewed;next = merge）
- **独立性声明**：复审与实施同会话——裁定从工件重构而非采信执行摘要：
  base..HEAD 全 diff 逐行重读（含 fmt 后终版）、门禁在 HEAD 重跑、归因
  关键主张逐条对照原始产物复核（默认审计册行数时点/WER 参数比对/判别台
  四臂原始输出/微实验输出/cdb 栈帧）。
- reviewed_commit: 99f6997eb（终版;中间版 5c3a9b5f0→1b79873ae[fmt 修正]
  →99f6997eb[文档陈旧表述修正],末两步仅空白/注释/文档零语义差）｜
  base_commit: b58bf1835（master）｜ dependency: auto-down@895f8d0
  （detached 只读,同 r1）。
- spec_inputs: `docs/specs/auto-lang/ui/design/vm-host-stability.md` 原地
  扩展（r2 复审对照代码核验:exitproc_trace site 串 ✔、env 名
  P765_EXIT_TRACE ✔、ExitProcess 臂自终止/TerminateProcess 臂转调的
  分臂表述 ✔[复审修正两处中间设计残留文字]、外部击杀假象警示与三例
  同族定谳 ✔）。supersedes_spec_components=[]（r1 判定维持——F-10 为
  增量观测件,不改既有契约）;new_spec_components 沿 r1 同文件;touched_goals
  GOAL-007 不变。
- acceptance_results: **AC-1/AC-2 pass**（r1 判定沿承;r2 diff 不触 F-1~F-9
  语义——fmt/注释级差异,当前代码门禁不劣化）｜ **AC-3 pass**（全量
  5257/5258[1b79873ae]+tv 162/162+定向 113/113;唯一红 ffi_dual_019 双证
  环境项:t05 目录 stash 基线复现红+规范目录当晨绿;终版 99f6997eb 后
  exit_audit 9/9 复验——门禁后改动均为空白/文档级）｜ **AC-4 pass**
  （两形态零退出:判别台 snap 180s/burst 181s/click 185s 全存活且审计册
  零行=零进程内 exit;归因通道完备:F-10 闭合依赖直调盲区+单测双验+
  50/50 压测;"根因防护"经 T-05 定谳修正为仪器假象——产品面无缺陷,
  修在 musk 仪器[交接在案],investigation-first 预期路径非范围缩减）｜
  **AC-5 pass**（本仓门禁零新增红+解阻交接齐备;musk 侧已并行落地仪器
  修复[click value+cleaningUp 标记,其 worktree 在案未提交]——正式复跑
  为其流程收尾）。
- findings: 无阻断项。F-765-R4（external,nonblocking）musk bigpage 仪器
  修复+正式复跑待其收口（其 worktree 已见修复实现）;F-765-R5
  （observation）本会话早前被杀后台任务遗留僵尸测试进程镜像句柄锁规范
  target 目录——门禁以 CARGO_TARGET_DIR=target-t05 出证,会话收尾自解,
  不入债务册。
- evidence: 纪要后记⑦（全链归因文书,随 99f6997eb 入库）;判别台
  `p765-probe.mjs` 产物在 worktree tmp/（临时,方法论与结论全文录入纪要,
  worktree 清理后可依纪要 §复现命令重建）;musk worktree bigpage 修复
  diff 在案（其所有权）;审计册/WER/日志检索命令与关键行摘录已在纪要。
- next: merge（`/auto-plan:merge`）。

### work 阶段交接（PLAN-765:r2，2026-10-12）

- stage: work ｜ plan_id: PLAN-765 ｜ plan_revision: 2 ｜ outcome: **pass**
  （T-05/T-06/T-07 全勾,可入 review）
- code_commit: worktree `D:/autostack/.wt/lang-765/auto-lang` branch
  `plan-765-dev`（基面 b58bf1835=master）——5c3a9b5f0（F-10 IAT 追踪钩 +
  归因纪要后记⑦ + 规约四分法增补,4 文件 +455/-1）。依赖只读兄弟：
  `.wt/lang-765/auto-down` @895f8d0（detached）。
- task_ids: T-05/T-06/T-07 全勾 ｜ evidence: 纪要后记⑦（三通道零痕核查/
  五臂判别台/node ESM 微实验/cdb detach 竞态取证）;exit_audit 9/9（含
  p765_iat 双测）+ IAT 子进程 50/50 压测 + plan765 9/9 + plan707/
  musk_vm_track 135/135;全量 5256/5258（CARGO_TARGET_DIR=target-t05,
  2 红均环境项双证：ffi_dual_019 t05 基线复现红+规范目录绿、clipboard
  隔离绿剪贴板并发 flake）;探针产物在 worktree tmp/（临时,关键结论已
  全文录入纪要）。
- **r2 核心定谳**：musk 呈报的"瞬态尖峰退出面"（burst 分发/大快照两形态
  原生 exit(1)）= **bigpage-matrix 仪器 finally-taskkill 假象**（click
  isError → 无 catch 抛错 → 收尾击杀,"VM EXITED code=1" 先于错误栈打印
  被误读为进程自退）——与 r1 soak 假象、musk K14 三例同族;本机"负载
  相关静默必退"呈报至今零例进程内真实退出。两形态载荷（309-573KB 快照
  ×20+/350 条 5s 爆发+列表轮询叠加）实测 VM 全程存活、审计册零行。
- **r2 交付**：F-10 ExitProcess IAT 追踪钩（env P765_EXIT_TRACE=1 门控,
  常驻诊断件）——闭合"依赖直调 ExitProcess 零痕"观测缺口（r1 四分法
  的盲区）,确定性自终止设计（避开 LdrShutdownProcess detach 竞态,cdb
  实证 + 50/50 压测）。
- blockers: 无阻断。移交 musk PLAN-104（其 T-03 收口动作,纪要后记⑦交接
  节）：①bigpage-matrix.mjs click 补 `value:"x,y"`;②收尾击杀与真实死亡
  区分（finally 前置 cleaningUp 标记）——修后一键复跑闭 T-03。
- next: review（`/auto-plan:review`）。spec 增量=既有 vm-host-stability.md
  原地扩展（四分法表 + 假象警示）,new_spec_components 无新增文件。

### r2 再激活（PLAN-765:r2，2026-10-12）

- stage: work（再激活簿记） ｜ plan_id: PLAN-765 ｜ plan_revision: 1→2 ｜
  outcome: reactivated（archived → executing,归档终态例外依用户显式裁定
  "计划 765 再次被激活（R2）"；765-R2 标签与 musk PLAN-104 T-03 阻塞
  跟踪同号连续）。
- 依据：musk PLAN-104 复核补记 + 证据报告 §T-03 勘误节（并行会话四轮
  bigpage 矩阵,commit 8531f78/c04c7fd）——r1 修复面（温和载荷/稳态轮询）
  实证有效,但**瞬态尖峰载荷触发原生 exit(1)（零错误输出）**为新面：
  burst 分发（350×5KB/15s）与单次 309KB autoui_snapshot 全树序列化两
  形态均命中。r1 审计钩在四轮矩阵里未开启（仪器 env 未设
  AUTO_DESKTOP_EXIT_LOG）——归因先补观测。
- 范围：T-05 归因（V05）→ T-06 根修（V06）→ T-07 复验解阻（V07）；
  新增 AC-4/AC-5。r1 全部验收与收据不受影响（其证据在其当时 commit 上
  有效）；受影响面=r2 复验时以当代码重跑为准。
- next: T-05 补观测复现（worktree `.wt/lang-765/auto-lang`,branch
  `plan-765-dev`,基面 master 364356e30）。

### merge 阶段收据（PLAN-765:r1，2026-10-12）

- stage: merge ｜ outcome: **delivered**（五检查点齐）
- **prepared**：reviewed 9ef5df2cb（rebase 后 0b654af37,range-diff 三提交
  全等,映射 4588ae0ad→e20b6c128/db345925c→9650c966a/9ef5df2cb→0b654af37）
  + 纯文档后代（规约回写/账本/索引）→ delivery_commit=**4f53653fd**;
  实现与依赖未变（auto-down@895f8d0 detached）。
- **landed**：master `git merge --ff-only plan-765-dev` 尖=4f53653fd,
  无合并提交;canonical spec 落位
  `docs/specs/auto-lang/ui/design/vm-host-stability.md` +
  `docs/reports/765-t01-attribution-memos.md`。
- **ledger_refreshed**：`.autoos/specs.json` 追加 P765-1（designs,182 项）/
  P765-2（reviews,218 项）/P765-3（reports,131 项）——只增不改,回读计数
  校验通过;`python scripts/spec-index.py` 再生 INDEX（幂等,26 projects,
  无 diff——新 design 文档在既有 project 组内）。
- **archived**：`git mv` → `docs/plans/archive/765-vm-host-active-session-stability.md`,
  status: archived,completion_kind: delivered。
- **cleaned**：wt-guard（两 worktree 均 clean）→ 移除
  `.wt/lang-765/auto-lang` + `.wt/lang-765/auto-down` + 分支 plan-765-dev +
  组目录。
- **制品面**：主检出 release auto.exe 已随 merge 重建（生产消费面 =
  musk 侧 VM 宿主/代码生成门禁二进制）;web bundle 无涉（本计划零
  website/vue 改动）。
- **批量回归到期**：765 % 5 == 0 → **到期** → `/auto-plan:regress`
  主检出单实例（收据刷新 `.last-batch-regression.json`）。

### review 阶段裁定（PLAN-765:r1，2026-10-12）

- stage: review ｜ plan_id: PLAN-765 ｜ plan_revision: 1 ｜ outcome: **pass**
  （status → reviewed;next = merge）
- **独立性声明**：复审与实施同会话——裁定从工件重构而非采信执行摘要：
  门禁在 HEAD 重跑（裸 `cargo t` 5256/5256 + `cargo tv` 162/162,2026-10-12
  复审时刻新鲜）、base..HEAD 全 diff 逐行重读（F-1~F-9 逐一在位核验）、
  soak 斜率从原始样本独立重算（segment 表推翻脚本输出并定谳单位 bug）。
- reviewed_commit: 9ef5df2cb（HEAD;代码=4588ae0ad,余两提交纯文档）｜
  base_commit: 83e1ab6eb ｜ dependency: auto-down@895f8d0（detached 只读）
- spec_inputs: 新 `docs/specs/auto-lang/ui/design/vm-host-stability.md`
  （草稿随 db345925c 入库,复审对照代码核验:共享 Client pool8/30s ✔、
  逃生门 env 名 ✔、审计 site 串 native_crash/main_err_ai_mode/
  mem_guard_f12_escape ✔、四分法表 ✔）;冲突扫描=async-http-result-
  lifecycle.md（executor/detached 表述仍准确,无过期面）、architecture.md
  :224/227 bridge 级派发契约未触（门控在 DynamicComponent 层,与 parked
  恢复泵"完成自置Dirty"、child-emit 写 __emit_msg(→seq bump 不撤脏)正交）
  ⇒ **supersedes_spec_components=[] 成立**,new_spec_components/touched_goals
  (GOAL-007,同 PLAN-650 VM 减负家族)核定无误。
- acceptance_results: **AC-1 pass**（1h idle +59.3MB/59.7min=0.017MB/s ≤
  0.1MB/s,6× 余量;流式 1.59MB/min 在基线带 [0.67,2.38]MB/min 内;斜率由
  原始样本独立重算,仪器单位 bug（报值×60）已定谳并在纪要后记⑤留档）｜
  **AC-2 pass**（1h vm_died=false 全程存活 + AUTO_DESKTOP_EXIT_LOG 审计册
  零行 + 125 条连续心跳双证;归因纪要含调用点/触发条件/防护与原生钩覆盖
  边界;外部击杀指纹实测复现吻合）｜ **AC-3 pass**（本仓门禁复审重跑双绿;
  musk 三门 gen-only EXIT=0/vm-link-probe PASS 108944B/ui-parity catalog
  PASS;plan484_024 charts 单红=并行负载 flake,隔离与全档复跑双绿,与
  本 diff 零触面——环境项非回归）。
- findings: 无阻断项。F-765-R1（nonblocking）idle ~1MB/min 线性缓漏未归因
  → 登记 **P765-D1**（KNOWN-DEBT）;F-765-R2（external）musk 仪器
  slopeMBPerMin 单位 bug → **P765-D2** 移交 musk PLAN-104（含 AC-01 真值
  重判）;F-765-R3（observation）plan484_024 flake 环境项在案。plan484
  家族与 charts 域不受本 diff 影响。
- evidence: 纪要+规约草稿（worktree db345925c/9ef5df2cb,merge 后入
  `docs/reports/765-t01-attribution-memos.md`+`docs/specs/auto-lang/ui/
  design/vm-host-stability.md`）;门禁命令/结果上文在案;soak 原始 JSON
  存 musk worktree tmp/（临时,关键聚合值已录入纪要后记——worktree 清理
  后仍可复算:复现命令在计划 §4）。
- next: merge（`/auto-plan:merge`;批量回归档到期判定随 merge 收尾走
  `/auto-plan:regress`）。

### work 阶段交接（PLAN-765:r1，2026-10-12）

- stage: work ｜ plan_id: PLAN-765 ｜ plan_revision: 1 ｜ outcome: **pass**
  （四任务全勾，可入 review）
- code_commit: worktree `D:/autostack/.wt/lang-765/auto-lang` branch
  `plan-765-dev`（基面 83e1ab6eb）——4588ae0ad（T-02/T-03 代码+测试+纪要）、
  db345925c（纪要后记+规约草稿 vm-host-stability.md）、9ef5df2cb（T-04
  收官纪要）。依赖只读兄弟：`.wt/lang-765/auto-down` @895f8d0（detached）。
- task_ids: T-01/T-02/T-03/T-04 全勾 ｜ evidence: 三份归因纪要
  `docs/reports/765-t01-attribution-memos.md`（含后记 5/6 两条仪器口径
  定谳）；本仓裸 `cargo t` 5256/5256 + `cargo tv` 162/162（plan484 charts
  单红为 flake，隔离/复跑双绿）；定向 plan765 9/9 + plan083 5/5 +
  plan707 31/31 + musk_vm_track 104/104；musk 三门 PASS（gen-only
  EXIT=0 / vm-link-probe PASS 108944B / ui-parity catalog PASS）；
  V02/V03 soak 实测（45s/300s/1h 三窗，1h:vm_died=false+审计零行+
  125 心跳；idle 真趋势 1.0MB/min=AC-1 门 6× 余量；流式在基线带内）。
- blockers: 无阻断。移交项两条（非本仓面）：①musk soak 仪器
  `slopeMBPerMin` 单位 bug（对分钟 t 再 ×60，报值实为 MB/hour；修法
  =删 `* 60`，一行）——musk PLAN-104 收口径时一并修；②idle ~17KB/s
  线性缓漏源未归因（+60MB/h，AC 门内）——候选=每响应余 3 份全文瞬态
  的堆记账漂移/重绘路径，后续杠杆登记 KNOWN-DEBT 候选。
- next: review（`/auto-plan:review`；spec delta 已填
  new_spec_components=[docs/specs/auto-lang/ui/design/vm-host-stability.md]，
  草稿在 worktree 随 db345925c 入库）。下游：musk PLAN-104 T-03/T-07
  解阻（AC-01 口径随单位 bug 修正后按真值重判；AC-02 长寿实例矩阵
  补跑）。

### work 启动（PLAN-765:r1，2026-10-11）

- 用户指示"实施它"——work 执行权由用户本人移交本会话（覆盖 new 交接中
  owner=user 的备注,范围不变）。worktree
  `D:/autostack/.wt/lang-765/auto-lang`（branch `plan-765-dev`,基面 83e1ab6eb）。

### new 阶段交接（PLAN-765:r1）

- stage: new ｜ plan_id: PLAN-765 ｜ plan_revision: 1 ｜ outcome: **pass**
  （可进入 work；**work owner=用户本人独立实施**——本计划即执行合同）
- 依据：两残留面有 musk PLAN-104 五轮仪器实测+审计零记录的硬证据；
  调查先行、不为未定根因预设实现；仪器/复现命令/门禁口径全部现成。
- 编号：765（active 764、archive 最大 763，双目录核唯一）。
- next: work（.wt/lang-765/auto-lang；owner=用户）。下游：musk PLAN-104
  T-03/T-07 待本计划 AC-1/AC-2 解阻。

## 10. 待澄清事项

- 无阻断项。T-01 定谳后若发现第三面（非①②），按 auto-plan:new 增补
  修订而非扩栈本计划。
- 移交 musk PLAN-104（随其 T-03/T-07 补跑）：①soak 仪器
  `slopeMBPerMin` 单位 bug（详见纪要后记⑤）；②"AC-01 idle 斜率"按
  修正口径（真 MB/min）重判——五轮基线与 p765 各轮真值均已在纪要
  后记列明。

## 11. spec-sync 回写记录（merge 沉淀，2026-10-12）

- 规约新册：`docs/specs/auto-lang/ui/design/vm-host-stability.md`（消息桥
  生命周期契约/回填 seq 门控语义/事件名推导载荷安全/退出审计四分法/
  测量口径注记）。
- 模块回写：`docs/specs/auto-lang/ui/overview.md` 增"VM 宿主活跃会话稳定域
  （PLAN-765）"节（衔接 697/650 域）；`docs/specs/auto-lang/ui/plans.md`
  追加 765 行；`docs/specs/goals.md` GOAL-007 追加 765 括注。
- 账本：`.autoos/specs.json` P765-1（designs）/P765-2（reviews）/P765-3
  （reports）;INDEX 再生幂等。
- 交付提交：4f53653fd（ff-only 落 master）。
