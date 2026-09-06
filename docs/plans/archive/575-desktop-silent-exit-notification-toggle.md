---
plan_id: PLAN-575
status: archived
feature_name: 桌面进程静默退出归因与修复（通知中心二次开合，526 高风险债）
author: [zhaopuming, ZCode]
created_at: 2026-09-06
updated_at: 2026-09-06

# Leave these EMPTY here — /auto-plan:review fills them:
supersedes_spec_components: []
new_spec_components:
  - "docs/specs/auto-lang/vm/architecture.md: 新增——stdlib 退出审计三挂点（exit_audit/exit_audit_path/install_exit_audit_panic_hook，挂点=shim_process_exit/全局 panic hook/run_session 正常返回，env AUTO_DESKTOP_EXIT_LOG 缺省 %LOCALAPPDATA%/auto-desktop/exit-audit.log，写失败静默=零行为变更）"
touched_goals:
  - "goal-009: 虚拟桌面与桌面 Shell——526 静默退出债归因收口：退出审计机制常驻 + N=20 不可复现降档（疑外部击杀 049 同族），KNOWN-DEBT 526 行降档🟡 + 535 D 项销账"

current_step: 6
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

- [x] **T1** 退出审计：`stdlib.rs` 增 `exit_audit(code, site)`（env
      `AUTO_DESKTOP_EXIT_LOG`，缺省 %LOCALAPPDATA%/auto-desktop/exit-
      audit.log；写失败静默）+ `shim_process_exit` 挂点 + 全局 panic hook
      （main 装配处）+ main_return 挂点。验证：`cargo test -p auto-lang
      --lib exit_audit`（临时路径注入写读断言）。
      [✅ 已完成] stdlib.rs 增 exit_audit/exit_audit_to/exit_audit_path/
      install_exit_audit_panic_hook + shim_process_exit 挂点①（commit
      41a284f1e）；挂点②③装 renderer.rs run_session（装配管线头部+
      `.run()?` 后正常返回路径）；5 测试过（含子进程探针：退出码 7 不变 +
      审计行 site=vm_process_exit），`cargo test -p auto-lang --lib
      exit_audit` 5 passed。
- [x] **T2** 复现驱动脚本 `notes_toggle_repro.mjs`（desktop MCP：find 铃
      铛 → press×2 → 存活轮询 → 台账行；异名二进制启动；MCP 不可达走
      t5_smoke 原生驱动回退）。验证：实机跑通 ≥5 轮台账。
      [✅ 已完成] scratch/p575/notes_toggle_repro.mjs（零依赖 node，异名
      副本 ui_desktop_p575.exe --fullscreen，JSONL 台账含 toggle 执行证据
      与审计尾抓取）。实测：宿主 dock 铃铛不在组件 VTree（autoui_find
      NOT FOUND，截图+probe 证实），故二次开合走 autoui_desktop bus 注入
      notes_toggle×2——真实铃铛点击的同一 records→NotesToggle→toggle_
      notification_center 路径（stderr 证据：NotificationCenter Init/
      RebuildNotes handler 各轮命中；截图证面板开）。5 轮台账全存活零退出
      （scratch/p575/ledger.jsonl，T2 commit c51218f7b）。
- [x] **T3** 独占环境 N=20 轮归因运行：逐轮台账 + 审计文件留档，产出分支
      判据结论（D3 三分支其一）。验证：台账 + 结论行入计划复审记录。
      [✅ 已完成] N=20 轮（异名副本独占运行，20 个独立进程）：20/20
      存活零退出，逐轮 toggle 执行证据（NotificationCenter Init/
      RebuildNotes handler 行）在册，审计文件零记录（scratch/p575/
      ledger.jsonl + round-*.log，commit 见 plan-575-dev）。D3 判据命中
      **不可复现分支**（第三支）。

### W2 分支收口（按 T3 结论走其一）

- [x] **T4a** （产品缺陷分支）按审计 site 定位修复 `renderer.rs`
      notes_toggle 双击路径/涉及面；修复后复跑 20 轮零退出 + 双击回归断言
      入 desktop e2e/驱动脚本。验证：20 轮台账零退出 + 断言过。
      [✅ 已完成—分支未命中] T3 审计零记录+零退出，产品缺陷分支不成立，
      T4a 不启动（无归因证据不作猜测性修复——计划变更摘要红线）。
- [x] **T4b** （外部击杀分支）KNOWN-DEBT 526 行降级改写（结论+049 互链）
      + 535 D 项销账注记（或不可复现降档 🟡）。验证：两处账本 diff。
      [✅ 已完成] 不可复现降档出口（575 待澄清①预授权）：KNOWN-DEBT
      526 行自 🔴 移 🟡 改写"未能复现（疑外部击杀，049 同族）"+审计常驻
      复现即启；535 D 项勾销 + ✅销账注记（PLAN-575 结论+台账指针）。
      两处 diff 落 plan-575-dev。
- [x] **T5** 回归与折回：`cargo tf --no-fail-fast`（唯一红=charts 既有）
      + 折回 auto-lang master + 簿记。验证：计数落复审记录。
      [✅ 已完成] `cargo tf --no-fail-fast`：3466 测 3465 过，唯一红=
      ui_gen::vue::tests::test_charts_gallery_compiles（基线已坏 charts
      既有，与本计划无关——6d61adc0b 已在案）；`cargo tv --no-fail-fast`
      （改 vm/ffi 文件追加档）：3607 测 3606 过，唯一红同 charts。
      aavm 触发面零命中（未动 auto/lib/*.at、aavm2、parity）→ 不跑 taa。
      折回：plan-575-dev --no-ff 合入 master（439877bd9），master 已回
      同 worktree。

## 复审记录

### 归因结论（PLAN-575 T3 执行产出，2026-09-06）

**分支判据命中：D3 第三支——不可复现（降档 🟡）。**

- 运行形态：异名副本 ui_desktop_p575.exe `--fullscreen` 独占实例 ×20（20
  个独立 PID）；每轮 bus 注入 notes_toggle×2（与真实铃铛点击同一
  records→DesktopCommand::NotesToggle→toggle_notification_center 路径，
  stderr 逐轮 handler 证据）→ 3s 存活轮询。
- 结果：20/20 进程存活零退出；退出审计文件零记录（若进程内源性退出——
  Process.exit/panic/正常返回——三挂点必落笔）；审计日志本身经 T1 子进程
  探针验证可用。
- 判读：原 2/2"复现"（其中第二例即验收通道 handler 双调，与本次驱动同
  通道）与 535 D 归因备注的并行会话 taskkill /F 强杀（退出码恰 1、无输出
  同 signature）高度吻合——外部击杀假说成立（049 同族），产品缺陷分支
  证据为零。
- 处置：KNOWN-DEBT 526 行 🔴→🟡 降档改写 + 535 D 项销账（T4b 已落）；
  退出审计机制常驻（T1 三挂点入库），一轮真实复现（审计指认 site）即重启。

### 回归计数（T5）

- `cargo tf --no-fail-fast`：3466 run / 3465 passed / 1 failed（唯一红 =
  `ui_gen::vue::tests::test_charts_gallery_compiles`，基线既有，非本计划引入）。
- `cargo tv --no-fail-fast`：3607 run / 3606 passed / 1 failed（同一 charts 既有红）。
- aavm：改动零触发（无 auto/lib/*.at、test/vm/aavm2、parity、aavm2 基建触碰）。

### 复审记录（/auto-plan:review，2026-09-06，ZCode）

**结论：PASS（4/4 验收全过，无阻塞债）→ `status: reviewed`。验证方式=全部复跑，不采信勾选框。**

| # | 验收标准 | 判定 | 复验证据 |
|---|---|---|---|
| 1 | 退出审计落地且零行为变更，三挂点各出审计证据 | **pass** | ①`shim_process_exit`：子进程探针单测——退出码 7 保持不变 + `code=7 site=vm_process_exit` 审计行（`p575_exit_code_survives_probe`）；②panic hook：单测 `panic_hook_writes_audit_line_before_prev_hook`——`code=101 site=panic msg=…` + 既有 hook 链式保留；③main_return：**实机端到端**——bus 注入 shutdown，进程 0.6s 优雅退出 code 0，审计文件恰一行 `pid=25928 code=0 site=main_return`。复跑 `cargo test -p auto-lang --lib exit_audit` 5/5 过 |
| 2 | 归因结论落档：N=20 逐轮台账 + 分支判据 + 复审记录 | **pass** | ledger.jsonl 20 轮/20 独立 PID/verdict 全 alive/逐轮 toggle_evidence（NotificationCenter handler 行）在册（commit d9290f26a）；分支判据=不可复现（第三支），结论已写入本文件复审记录节 |
| 3 | 分支收口（T4b 登记降级 + 535 D 销账） | **pass** | KNOWN-DEBT 🔴 节已无 526 行、🟡 节新增降档行（结论+049 互链+复现即启）；535 D 项 `- [x]` + ✅销账注记（含 PLAN-575 指针）。两 diff 落 becb8d581 |
| 4 | 回归绿：cargo tf 唯一红=charts 既有 | **pass** | 复跑 `cargo tf --no-fail-fast` 3466 run/3465 pass，唯一红 `ui_gen::vue::tests::test_charts_gallery_compiles`；`cargo tv --no-fail-fast` 3607/3606 唯一红同。charts 红预存基线由 571 T9 提交（53259c4da，先于本计划）在案；aavm 触发面零命中 |

**遗漏/延后/Workaround 扫描**（无阻塞发现）：

- **机制偏注一（非阻塞）**：计划 D2 写"find 铃铛 → press×2"，实测铃铛为宿主
  dock 渲染不在组件 VTree（autoui_find NOT FOUND，截图/探针双证），改走
  `autoui_desktop` bus 注入 notes_toggle×2——与真实点击共用
  records→DesktopCommand::NotesToggle→toggle_notification_center 全路径；
  且原 2/2"复现"之第二例本就是验收通道 handler 双调（535 D 原文），通道
  等价。驱动逐轮仍尝试 find 并记录 bell=false，非静默换道。
- **机制偏注二（非阻塞）**：挂点②③装于 `run_session`（I3 唯一装配管线），
  覆盖面为 desktop∪Standalone 超集——审计零行为变更（G3），覆盖面扩大无
  语义影响，已在代码注释与归因结论中写明。
- **证据载体**：逐轮 stderr 原始日志受仓级 `*.log` gitignore 约定不入库，
  台账 ledger.jsonl 的 `toggle_evidence` 字段逐轮引用 handler 行为证——
  台账即证据正本（scratch/p575/ledger.jsonl）。
- **T4a 未执行=分支未命中**（计划 W2"走其一"语义），非延后；无 TODO/
  FIXME/dbg! 新增；无新编译警告。
- **复审卫生修复**：exit_audit 新增代码两处链式调用 rustfmt 规整
  （d8fdf98fa；stdlib.rs 余量 drift 为预存不动），修复后单测复跑 5/5 过。

**债候选**：无新增（审计机制常驻系计划设计本身；降档出口为待澄清①预授权）。

**spec-impact 元数据**：已回填 frontmatter（new_spec_components=vm
architecture 退出审计三挂点；touched_goals=goal-009；supersedes=空）。
交接 `/auto-plan:merge`。

## 待澄清事项

1. **N=20 轮是否足够**：外部击杀波次不可控，若 20 轮内恰无清扫波次，零退
   出证据力下降——是否接受"20 轮 + 审计机制常驻，真实复现再启"的降档出口
   （计划默认接受）。
2. **二进制异名副本的副作用**：改名副本是否影响桌面进程的自更新/单实例锁
   等按名机制（执行期实测，异常即回退原名 + 错峰运行）。
3. **T4a 若审计指认到 VM 引擎语义面**（如 .at 面真调了 Process.exit）：
   默认只加防护/日志不改语义；行为级变更需用户裁定。
