---
plan_id: PLAN-575
status: drafting
feature_name: 桌面进程静默退出归因与修复（通知中心二次开合，526 高风险债）
author: [zhaopuming, ZCode]
created_at: 2026-09-06
updated_at: 2026-09-06

# Leave these EMPTY here — /auto-plan:review fills them:
supersedes_spec_components: []
new_spec_components: []
touched_goals: []

current_step: 0
total_steps: 6
---

# [PLAN-575] 桌面进程静默退出归因与修复（通知中心二次开合）

## 变更摘要

KNOWN-DEBT-AND-RISKS 🔴 节唯一开口项（526）：任务栏铃铛二次开合通知中心 →
桌面进程静默退出 code 1（无 panic 输出，复现 2/2）。本计划做**先归因后修
复**的两段式处置：第一段给全部进程退出路径装**退出审计日志**（shim_process_
exit / panic hook，append 时间戳+code+site），配合独占环境自动化复现驱动
（MCP press 铃铛×2 + 进程存活轮询，二进制异名避开并行清扫）跑 N 轮取证据；
第二段按归因结论走分支——产品缺陷则定位修复 + 回归钉死，外部击杀（049 同
族）则 🔴 行降级改写 + 535 D 项销账。本计划不含任何无归因证据的猜测性修复。

## 目标

- **G1 归因闭环**：铃铛二次开合场景在独占环境 + 退出审计下取得确定性证据
  ——退出若发生，审计日志必须指认退出点（site + code）；N 轮零退出则实证
  不可复现（外部击杀假说成立）。
- **G2a（产品缺陷分支）**：定位修复后同场景 N 轮零退出，回归断言入册。
- **G2b（外部击杀分支）**：KNOWN-DEBT 526 行降级改写（结论+049 互链），
  535 D 项销账注记。
- **G3 审计机制零行为变更**：退出审计只追加日志，不改变任何退出路径的语
  义/退出码。

## 架构方案

不引入新机制：退出审计 = `shim_process_exit`（vm/ffi/stdlib.rs，裸
`std::process::exit`）入口处 + 全局 panic hook 追加写一行审计（时间戳/
code/site/尽力 backtrace），日志路径经 env `AUTO_DESKTOP_EXIT_LOG`（缺省
%LOCALAPPDATA%/auto-desktop/exit-audit.log）。复现驱动 = AutoUI MCP 通道
（autoui_find 铃铛 → press ×2 → 进程存活轮询；desktop 流同款 JSON-RPC），
MCP 不可达时回退既有 t5_smoke 原生驱动（tools/native-fixture，473/486 先
例）。二进制以异名副本运行（taskkill 清扫按名匹配，049 定因对策）。

## 技术栈

- 实现面：auto-lang 本仓 `crates/auto-lang/src/vm/ffi/stdlib.rs`（exit 审
  计）+ `crates/auto-lang/src/ui/iced/renderer.rs`（toggle_notification_
  center 双击路径，仅归因/修复期触点）。
- 验证面：node 复现驱动（AutoUI MCP JSON-RPC，vm-smoke 同款协议）+ 实机独
  占环境；auto-lang `cargo tf`（回归门）。

## 需求分析与背景调查

- **来源**：KNOWN-DEBT-AND-RISKS 🔴 节 526 行（唯一开口高危，复现 2/2：
  2026-09-03/04 任务栏铃铛二次开合 → 桌面进程静默退出 code 1，无 panic 无
  栈，RUST_BACKTRACE=full 仍无栈=进程性退出非 panic）；跟踪于
  535-desktop-ux-followups D 项（未清偿）。
- **关键归因备注（535 D 项，2026-09-04）**：本机存在并行会话 taskkill /F
  强杀 ui_desktop 干扰源——强杀退出码恰为 1、无输出，与"静默退出"同
  signature；2/2 复现**不能排除外部击杀**，复核需独占环境。PLAN-049（auto
  -down）已定因同族现象=共享机外部击杀，非产品缺陷。
- **代码锚点**：`stdlib.rs:725 shim_process_exit` = 裸 `std::process::exit
  (code)`（无任何审计）；`renderer.rs:8184` notes_toggle → toggle_
  notification_center 二连击路径；taskkill /F 实测退出码 1（049 侧测量）。
- **边界**：不猜测性修 VM 引擎；不改通知中心功能语义；审计机制必须零行为
  变更（G3）。

## 详细设计

### D1 退出审计（G1/G3 基座）

`exit_audit(code, site)`：append 一行 `{ts, pid, code, site}` 到审计文件
（env 可改路径；写失败静默吞掉——审计绝不引入新退出路径）。挂三点：
①`shim_process_exit`（site="vm_process_exit"）；②全局 `std::panic::set_
hook`（site="panic"，先写审计再调默认 hook——panic 通常有输出，本审计补
"静默"面）；③`main` 正常返回路径（site="main_return", code=0）——用于区
分"走到正常退出"与"被外界终止"（外界 TerminateProcess 三点都不落笔=审计
零记录 ⇒ 外部击杀实锤）。

### D2 复现驱动（G1 证据机）

node 脚本（desktop MCP JSON-RPC，vm-smoke 协议复用）：initialize → find
铃铛 → press → press（二次开合）→ 轮询进程存活 3s → 记一轮；循环 N 轮；
退出即抓审计文件尾 + exit code。二进制异名副本（如 ui_desktop_p575.exe）
避开并行清扫；MCP 不可达 → t5_smoke 原生驱动回退。

### D3 归因判据（分支开关）

- 审计有记录（site=vm_process_exit/panic/main_return）→ **产品缺陷分支
  T4a**：site 直接指认修复面（双击路径的退出触发点），修复后 T5 复跑。
- 审计零记录但进程死亡（exit≠0）→ **外部击杀分支 T4b**：🔴 526 行降级改
  写（归因结论 + 049/D3 互链），535 D 项销账。
- N 轮零退出且进程存活 → 不可复现：降档改 🟡"未能复现（疑外部击杀）"并
  留审计机制常驻，一轮真实复现再启。

## 测试设计

| 门 | 内容 | 命令/方式 |
|---|---|---|
| 单测 | exit_audit 追加写 + panic hook 挂接（临时文件路径注入） | `cargo test -p auto-lang --lib exit_audit` |
| 单测 | shim_process_exit 落审计后退出码不变（子进程探针） | 同上（`std::process::Command` 探针或手验） |
| 实机 | 复现驱动 N=20 轮：进程存活/审计记录逐轮台账 | node 驱动脚本（独占环境） |
| 回归 | auto-lang 全量 tf（唯一红=charts 既有） | `cargo tf --no-fail-fast` |

## 验收标准

1. 退出审计机制落地且零行为变更：三条挂点各出一行审计证据（单测/手验）。
2. 归因结论落档：N=20 轮复现运行的逐轮台账（存活/退出+审计行）+ 分支判据
   命中哪一支，写入计划复审记录。
3. 分支收口：T4a 修复后 20 轮零退出 + 回归断言；或 T4b 登记降级 + 535 D
   销账；或不可复现降档注记。
4. 回归绿：`cargo tf`（唯一红=charts 既有）。

## 执行步骤

### W1 审计与复现基座

- [ ] **T1** 退出审计：`stdlib.rs` 增 `exit_audit(code, site)`（env
      `AUTO_DESKTOP_EXIT_LOG`，缺省 %LOCALAPPDATA%/auto-desktop/exit-
      audit.log；写失败静默）+ `shim_process_exit` 挂点 + 全局 panic hook
      （main 装配处）+ main_return 挂点。验证：`cargo test -p auto-lang
      --lib exit_audit`（临时路径注入写读断言）。
- [ ] **T2** 复现驱动脚本 `notes_toggle_repro.mjs`（desktop MCP：find 铃
      铛 → press×2 → 存活轮询 → 台账行；异名二进制启动；MCP 不可达走
      t5_smoke 原生驱动回退）。验证：实机跑通 ≥5 轮台账。
- [ ] **T3** 独占环境 N=20 轮归因运行：逐轮台账 + 审计文件留档，产出分支
      判据结论（D3 三分支其一）。验证：台账 + 结论行入计划复审记录。

### W2 分支收口（按 T3 结论走其一）

- [ ] **T4a** （产品缺陷分支）按审计 site 定位修复 `renderer.rs`
      notes_toggle 双击路径/涉及面；修复后复跑 20 轮零退出 + 双击回归断言
      入 desktop e2e/驱动脚本。验证：20 轮台账零退出 + 断言过。
- [ ] **T4b** （外部击杀分支）KNOWN-DEBT 526 行降级改写（结论+049 互链）
      + 535 D 项销账注记（或不可复现降档 🟡）。验证：两处账本 diff。
- [ ] **T5** 回归与折回：`cargo tf --no-fail-fast`（唯一红=charts 既有）
      + 折回 auto-lang master + 簿记。验证：计数落复审记录。

## 复审记录

（待 /auto-plan:review 填写）

## 待澄清事项

1. **N=20 轮是否足够**：外部击杀波次不可控，若 20 轮内恰无清扫波次，零退
   出证据力下降——是否接受"20 轮 + 审计机制常驻，真实复现再启"的降档出口
   （计划默认接受）。
2. **二进制异名副本的副作用**：改名副本是否影响桌面进程的自更新/单实例锁
   等按名机制（执行期实测，异常即回退原名 + 错峰运行）。
3. **T4a 若审计指认到 VM 引擎语义面**（如 .at 面真调了 Process.exit）：
   默认只加防护/日志不改语义；行为级变更需用户裁定。
