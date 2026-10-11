---
plan_id: PLAN-765
plan_revision: 1
status: archived
feature_name: VM 宿主活跃会话稳定专项（空闲内存残留 + 原生层 exit(1)，承接 musk PLAN-104 定罪）
author: [agent]
created_at: 2026-10-11
updated_at: 2026-10-11
supersedes_spec_components: []
new_spec_components: ["docs/specs/auto-lang/ui/design/vm-host-stability.md"]
touched_goals: [GOAL-007]
affects: [auto-lang, a2r-std]
current_step: 4
total_steps: 4
---

# [PLAN-765] VM 宿主活跃会话稳定专项——空闲内存残留 + 原生层 exit(1)

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

## 9. 复审记录

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
