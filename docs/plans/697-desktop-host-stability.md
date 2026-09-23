---
plan_id: PLAN-697
status: execution_done          # drafting → executing → execution_done → reviewed → archived
feature_name: desktop-host-stability
author: [zcode]
created_at: 2026-09-23
updated_at: 2026-09-23

# /auto-plan:review 结束时填写：
supersedes_spec_components: []
new_spec_components: [SD-01 桌面宿主稳定性契约（inproc VM App 拉起零宿主退出 + 崩溃归因档案）]
touched_goals: []             # 引用 docs/specs/goals.md 的 GOAL-NNN

affects: [docs/specs/auto-lang/ui/overview.md]
current_step: 4
total_steps: 4
---

# [PLAN-697] desktop-host-stability——桌面宿主静默退出勘定与根修

> 来源：PLAN-694 dogfood 走查期观测（P694-D1，medium）。RQ 架构四计划链
> （683/690/693/694）已收官——本计划是 dogfood 可用性的收尾：**宿主会消失的
> 桌面没法日用**。

## 变更摘要

P694-D1 勘定+根修：inproc VM App（020-music-player）拉起后桌面宿主**静默
退出 ×3**（ui_desktop 全屏/窗口化两形态均复现；日志无 panic/错误行，进程
直接消失；时间点在 `launch_app(inproc)` 落地后数十秒内）。走查期间用户实时
操作与合成输入并存，不排除外因（进程被关），但 ×3 时间相关性值得勘定。
嫌疑面：back-proxy lazy-start（port 3359）/ media 能力（native_media/mpv 面）/
inproc 直挂路径。本计划 = 复现归因（cdb 附加取证据）→ 定点根修 → 回归钉。

## 目标

- **G1（复现与归因）**：稳定复现路径或排除法结论——cdb 附加宿主（fastfail/
  access violation/exit 码三判）+ 嫌疑面二分（back-proxy 禁用对照/media 禁用
  对照/他 App 对照）。产出归因档案（reports/p697-stability/）。
- **G2（根修）**：按归因定点修复（若为外因/无法复现 → 排除法证据链 +
  观测加固：宿主退出码留痕行 + 环境自检，残面登记不阻塞）。
- **G3（回归钉）**：最小回归测试或走查脚本入档（复现路径 → 修复后绿）。
- **G4（顺车）**：P683-D6 残面状态核销（694 T-04 复核结论对债册行的刷新，
  预期已过——一行划线）。

**非目标**：不动 RQ remote/desktop 模式链路（694 已验）；不做 launcher 输入
面修复（P694-D2 归 auto-os/shell 线）；不做全 gallery 稳定性普查（G1 复现
结论出来后另定范围）。

**成功标准**：music-player 拉起+运行+关闭全环宿主存活（或外因结论+观测
加固落地）；归因证据链在档。

## 架构方案

勘定驱动（bounded investigation）——三嫌疑面二分：

```
ui_desktop 宿主
  launch_app("020-music-player") inproc
    ├─ 嫌疑A back-proxy lazy-start（back_provision.rs:103
    │   start() — 线程/端口 3359/子进程面）
    ├─ 嫌疑B media 能力（plan.native_media → proxy.add_native_media
    │   — mpv/句柄/驱动面）
    └─ 嫌疑C inproc 直挂本身（解释器在宿主线程组内的 panic→abort 面
        ——日志无 panic 行不排除 abort/fastfail：Windows 错误模式
        silent-exit 家族）
```

- 取证优先：cdb 附加（`-c2` second-chance fastfail + 模块限定断点，
  cdb-windows-forensics-playbook 既有方法论）；进程退出码/ETW 退出原因
  兜底；
- 对照实验矩阵：music-player × {全量, AUTO_NO_BACK_PROXY=1（若无缝则加
  测试缝）, media 禁用} + 他 App（003）对照——×3 复现史说明可观测频率，
  对照跑 ≥5 轮；
- 修复形态按归因分派：A/B = 面内修复（隔离/降级路径）；C = panic 边界
  （inproc 崩溃围栏——017-chat/PLAN-041 同族先例）；外因 = 观测加固
  （退出留痕+自检）。

## 需求分析与背景调查

- **授权记录**：用户 2026-09-23 「计划694已经落地；后续还有计划要规划吗」
  ——盘点后裁定立本计划（RQ 线唯一 medium 残项，dogfood 可用性阻塞）。
- **P694-D1 原文**（债册 :37）：现象/复现形态/嫌疑/诊断入口全在档；
  walkthrough.py/phys.py（p694-dogfood/）= 复现脚本底子（合成输入侧）。
- **代码实勘**：
  - `back_provision.rs:96 ensure_backend`——lazy-start 臂（失败有降级
    打印，静默退出不符合该路径自述）；`add_native_media`/`add_session`
    在同函数；
  - `session.rs launch_app(inproc)`——解释器直挂 + back/media 供给决策树
    （PLAN-037）；
  - 既有先例：PLAN-041 崩溃围栏（017-chat 树/布局失配崩溃族——inproc
    App 崩塌波及宿主的同族问题，P041-D1 统一登记在案）+ cdb 取证方法论
    （memory：-hd/-c2/bu 模块限定）。
- **spec 现状**：ui/overview 无宿主稳定性契约行——SD-01 增补。

## 详细设计

**T-00 归因结论回填（2026-09-23 勘定收口）**：外因结案——机器级内核/驱动
不稳定（三日三蓝屏 0x9F/0x1A/0xA + GPU TDR 族；走查窗口 System Event
6008 = 09-23 10:59:12 非正常关机实锤，minidump 0923 在档）。蓝屏瞬时吞
进程 = 无 panic/无错误行/无 WER 应用崩溃记录，与 P694-D1 观测签名逐项
吻合。嫌疑 A（back-proxy lazy-start）/B（media 供给）/C（inproc panic
边界）经 13 轮复现矩阵（基线窗口化×5+全屏×2+禁A×2+禁B×2+他App×2 全绿
零死亡，A/B 面点火实证）+ WER 零宿主崩溃记录三重排除。修复形态走**观测
加固**分支（契约预授权：待澄清①默认口径）——非围栏/根因层（待澄清②随
外因结论不适用）。

落地件（worktree plan-697-dev 45aef8e04）：

- `examples/ui_desktop.rs`：`[ui-desktop] start/alive(30s)/exit` 足迹桩 +
  panic 钩子带 backtrace（此前宿主零足迹，内部 panic 静默 abort 零痕迹；
  镜像 rust_ui.rs X9 形制——CLI 桌面路径既有同款，双面覆盖）；
- `back_provision.rs` 对照缝：`AUTO_NO_BACK_PROXY=1` / 
  `AUTO_NO_NATIVE_MEDIA=1`（勘定器械，缺省未设生产零影响）；
- 归因档案 `docs/plans/reports/p697-stability/`（attribution.md 主档 +
  events-extract.txt 事件日志摘录 + matrix.jsonl 矩阵记录 + repro/
  walkthrough 两脚本）；
- SD-01 落表 `docs/specs/auto-lang/ui/overview.md`（宿主稳定性契约节：
  零宿主退出底线 + 观测面 + 死亡判位表 + 对照缝 + 勘定方法）。

执行纪要：worktree `D:/autostack/.wt/lang-697/auto-lang`（base
0f6a91b29，分支 plan-697-dev，依赖位 auto-down @3373a5c detached 只读）。
勘定过程两处增量：①宿主本尊勘定=ui_desktop example（非 `auto run
--desktop` CLI 形态——后者为单 App 占位渲染，非 694 走查形态）；②AC-04
"预期销号"勘正=694 T-04 实际结论为环境阻塞未复核（复审裁协议级等价），
债册行按证据刷新为"状态维持+对账注记"而非划线销号。

### 规范增量

| delta_id | add/modify/retire | docs/specs/... target | before/after rule | rationale | acceptance IDs |
|---|---|---|---|---|---|
| SD-01 | add | docs/specs/auto-lang/ui/overview.md | 无稳定性契约 → inproc VM App 拉起零宿主退出 + 崩溃归因档案指针 + 观测面（退出留痕）契约 | G2/G3 沉淀 | AC-01..04 |

## 测试设计

- 复现脚本（walkthrough.py 派生）：拉起→播放面就位→N 秒存活断言→关闭
  回收——修复前红/后绿双态录证；
- 对照矩阵输出（表格入 reports：全量/禁A/禁B/他App × 退出码）；
- 既有门禁零新增红。

## 验收标准

- [x] AC-01 归因档案在档（cdb/退出码/对照矩阵三证其一以上；外因结论
      需排除法证据链完整）。
      [✅ 已完成] `docs/plans/reports/p697-stability/attribution.md`：
      对照矩阵（13 轮）+ Windows 事件日志取证（退出位面档案）双证；排除
      法链完整（复现零死亡+嫌疑面点火实证+WER 零宿主崩溃记录）。
- [x] AC-02 修复后复现脚本绿（或外因结论+观测加固落地：宿主退出留痕行
      + 自检）。
      [✅ 已完成] 外因分支：观测加固落地实机演示——`[ui-desktop] start/
      alive/exit` 三行 + panic 钩子验证（host-1790159470.log 等；exit 行
      走 run_session 返回臂，与 X9 同形制）。复现脚本 repro_host_exit.py
      14 轮全绿在档。
- [x] AC-03 music-player 拉起+运行+关闭全环宿主存活实机录证。
      [✅ 已完成] walkthrough_full_cycle.py：拉起（wid2 落地，media scan
      393 曲目就位=305KB 级 JSON 全链通）→播放态 20s→关闭（窗撤）→
      复拉起（wid3 重建）——宿主全程存活（tasklist 实证）；四截图
      t2-01/03/04/06 + state 转录在档。
- [x] AC-04 P683-D6 债册行状态刷新（694 T-04 结论对齐——预期销号划线）。
      [✅ 已完成（勘正后）] 694 T-04 实际=⛔环境阻塞未复核（非预期中的
      已过）；694 复审裁管道环=协议级等价。债册行刷新为"状态维持大部
      销号+694 对账注记（残面降级可选复核）"——按证据记账非划线，偏差
      已在本计划 §详细设计/复审记录声明。
- [x] AC-05 既有门禁零新增红。
      [✅ 已完成] cargo t 同作用域 --no-fail-fast 全量：5495 跑 5485 绿
      10 红=预存 9（musk×6 双源漂移+desktop_protocol×2 逐名同 693/690
      在册+a2vue 金样×1 PLAN-682 在册）+1 负载敏感 flake
      （test_plan358_d1_for_style_if_msg_on_stress 全量负载下红、隔离
      复跑 5.0s 绿）——零新增红。cargo check --examples 干净；
      back_provision 作用域 4/4 绿。

## 执行步骤

（原子任务：精确文件路径 + 确切操作 + 验证命令；每步完成后追加 [✅ 已完成] 一行证据）

- [x] T-00 勘定（bounded investigation）：复现尝试（≥5 轮）+ cdb 附加
  取证 + 对照矩阵（A/B/他App）。产出归因档案回填 §详细设计。
      [关联 AC-01] 验证：reports/p697-stability/ 在档。
      [✅ 已完成] 14 轮复现（基线窗口化×5+冒烟×2+全屏×2+禁A×2+禁B×2+
      他App×2）零死亡；Windows 取证替代 cdb 附加（走查窗口已闭，cdb 无
      可捕现场——事件日志三查实锤机器级外因 10:59:12 bugcheck，WER 零
      宿主崩溃记录）；对照矩阵四臂 + 测试缝生效核验（8 份含 proxy vs 6
      份对照臂零 proxy）。归因档案 attribution.md 在档；宿主本尊勘定增
      量=ui_desktop example 形态（CLI --desktop 为单 App 占位非走查形态）。
- [x] T-01 按归因定点修复（或外因结论 → 观测加固：退出码留痕行 +
      环境自检）。[关联 AC-02] 验证：复现脚本双态（前红后绿）或加固
      观测行实机演示。
      [✅ 已完成] 外因分支：ui_desktop.rs 足迹桩三件（start/alive 30s/
      exit + panic 钩子 backtrace）实机演示在档（host-1790159470.log：
      start 行 t=1790159470 + alive 行 t=1790159500 实证）；对照缝双
      env 落地并经矩阵臂生效核验。
- [x] T-02 全环走查录证（music-player 拉起/运行/关闭）。[关联 AC-03]
      [✅ 已完成] walkthrough_full_cycle.py 全环：launch（bus verb）→
      窗落地（AutoMusic UI+393 曲目 scan 就位）→播放态观察 20s→close
      wid=2（窗撤、宿主存活）→复 launch（wid3 重建）——宿主 pid 24288
      全程存活；四截图+三份 state/snapshot 转录在档。
- [x] T-03 收口：P683-D6 刷新（AC-04）+ SD-01 落表 + 债册对账
      （P694-D1 销号/转登记）+ 复审。[关联 AC-04/05]
      [✅ 已完成] P694-D1 销号（外因结案证据链在档）+P683-D6 对账注记
      （worktree 同 commit）；SD-01 落 overview.md 宿主稳定性契约节；
      work 复审记录见下（outcome=pass，next=review）。

## 复审记录

- draft 交付（2026-09-23）：stage=new，PLAN-697 rev1。outcome=pass。
  next=work（T-00 为勘定门——外因/无法复现也是合法结论，走观测加固
  分支，该分支已在本契约内授权）。
- work 交付（2026-09-23）：stage=work，PLAN-697 rev1。outcome=**pass**。
  code_commit=plan-697-dev 45aef8e04（base 0f6a91b29；worktree
  D:/autostack/.wt/lang-697/auto-lang；依赖位 auto-down @3373a5c detached）。
  task_ids=T-00..T-03 全履。evidence=归因档案 attribution.md（外因结案：
  机器级 bugcheck 三日三蓝屏+走查窗口 Event 6008 10:59:12 实锤；13 轮
  矩阵零死亡+嫌疑面点火实证+WER 零宿主崩溃三重排除）+观测加固实机演示
  （[ui-desktop] start/alive 行 host-1790159470.log）+全环走查四截图
  （拉起/播放 393 曲/关闭存活/复拉起）+门禁（5495 跑 10 红全预存/ flakes
  隔离绿，check --examples 干净，back_provision 4/4）。blockers=无。
  待澄清① closure=观测加固分支（预授权默认口径），② closure=外因结论
  下不适用（非围栏/根因裁定面）。next=review。
  **勘正声明（对契约的两处偏差，证据驱动）**：①AC-04"预期销号"实际=
  694 T-04 环境阻塞未复核，债册行按证据刷新为状态维持+对账注记（非划
  线）；②"cdb 附加取证"由 Windows 事件日志三查替代（走查窗口已闭无可
  捕现场；事件日志=更强回溯性证据，方法在档案 §勘定方法）。

## 待澄清事项

1. ~~「无法稳定复现」的处置口径~~ **已 closure（T-00）**：外因结案走
   观测加固分支（预授权默认口径）——加固落地 + 残面（机器稳定性，
   GPU 驱动/内核面）在归因档案 §残面登记，不阻塞。
2. ~~围栏层 vs 根因层裁定~~ **已 closure（T-00）**：外因结论下不适用
   （非 inproc 崩溃围栏族，P041-D1 无涉）。
