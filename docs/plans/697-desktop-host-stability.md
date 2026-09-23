---
plan_id: PLAN-697
status: drafting               # drafting → executing → execution_done → reviewed → archived
feature_name: desktop-host-stability
author: [zcode]
created_at: 2026-09-23
updated_at: 2026-09-23

# /auto-plan:review 结束时填写：
supersedes_spec_components: []
new_spec_components: [SD-01 桌面宿主稳定性契约（inproc VM App 拉起零宿主退出 + 崩溃归因档案）]
touched_goals: []             # 引用 docs/specs/goals.md 的 GOAL-NNN

affects: [docs/specs/auto-lang/ui/overview.md]
current_step: 0
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

（T-00 归因结论后回填：修复点位/围栏形制/观测面。）

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

- [ ] AC-01 归因档案在档（cdb/退出码/对照矩阵三证其一以上；外因结论
      需排除法证据链完整）。
- [ ] AC-02 修复后复现脚本绿（或外因结论+观测加固落地：宿主退出留痕行
      + 自检）。
- [ ] AC-03 music-player 拉起+运行+关闭全环宿主存活实机录证。
- [ ] AC-04 P683-D6 债册行状态刷新（694 T-04 结论对齐——预期销号划线）。
- [ ] AC-05 既有门禁零新增红。

## 执行步骤

（原子任务：精确文件路径 + 确切操作 + 验证命令；每步完成后追加 [✅ 已完成] 一行证据）

- [ ] T-00 勘定（bounded investigation）：复现尝试（≥5 轮）+ cdb 附加
  取证 + 对照矩阵（A/B/他App）。产出归因档案回填 §详细设计。
      [关联 AC-01] 验证：reports/p697-stability/ 在档。
- [ ] T-01 按归因定点修复（或外因结论 → 观测加固：退出码留痕行 +
      环境自检）。[关联 AC-02] 验证：复现脚本双态（前红后绿）或加固
      观测行实机演示。
- [ ] T-02 全环走查录证（music-player 拉起/运行/关闭）。[关联 AC-03]
- [ ] T-03 收口：P683-D6 刷新（AC-04）+ SD-01 落表 + 债册对账
      （P694-D1 销号/转登记）+ 复审。[关联 AC-04/05]

## 复审记录

- draft 交付（2026-09-23）：stage=new，PLAN-697 rev1。outcome=pass。
  next=work（T-00 为勘定门——外因/无法复现也是合法结论，走观测加固
  分支，该分支已在本契约内授权）。

## 待澄清事项

1. 「无法稳定复现」的处置口径（观测加固 + 残面登记 vs 挂起等待再发）——
   T-00 结论出来后呈报裁定；默认前者。
2. 若归因为 inproc 崩溃围栏族（P041-D1 同源），修在围栏层还是根因层——
   勘定后按量级裁定（围栏=止血小件，根因=随 P041-D1 统一登记）。
