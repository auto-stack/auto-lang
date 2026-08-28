# Plan 452: AutoOS 虚拟桌面奠基——设计收编、裁定同步与 IME/焦点 Spike

> **状态**: ✅ 完成（2026-08-26 归档）。T1–T5 文档批次 + T6 IME/焦点 spike
> 全部完成。spike 报告：`docs/plans/reports/452-ime-spike.md`——六项验证
> 无阻断级结论（②为"受限"：组合中 Tab 被 IME 吞、点击切换组合静默终止），
> 453 立项条件满足。
> **来源**: 2026-08-26 架构讨论——AutoUI 从"每 App 独立窗口"走向跨平台虚拟桌面。
> 讨论产出 Design 23 与程序跟踪文件两份草案，用户过目无异议后立项收编。
> 上游脉络：Plan 365（宿主架构，COMPLETE）→ Plan 386（RenderQueue，PAUSED）
> → 本计划翻转/重定位其中两条裁定。
> **基线**: master fd454efad
> **性质**: 纯文档 + 一次性验证原型（spike 产出结论文档，不进产品代码路径）。

## 1. 目标

把虚拟桌面架构从讨论沉淀为可执行的程序基座：

1. Design 23（`docs/design/autoui/virtual-desktop.md`）从草案转正式，
   作为 453–457 与 386 复活的共同架构依据。
2. 执行 Design 23 §5 同步清单——doc 20 加修订横幅与段落注记、Plan 365
   归档注记、Plan 386 定位改写——消除存量文档与新股定的打架。
3. 程序跟踪文件（`docs/plans/autos-desktop-program.md`）核销首批裁定。
4. 跑 IME/焦点 spike（Design 23 §9 六项清单），在 453 动 renderer.rs 之前
   把最大 UX 风险（Design 23 §8 风险 1）定性。

**非目标**：任何产品代码改动（那是 453+ 的范围）；spike 原型不进 main
代码路径；453 立项本身（本计划收束后条件即满足，立项另起）。

## 2. 任务

| # | 任务 | 状态 |
|---|---|---|
| T1 | Design 23 状态头 草案→正式；§5 同步清单状态核销 | ✅ 2026-08-26 |
| T2 | doc 20：标题区加修订横幅；§6.1 加 R1 注记（chrome 归特权桌面 App）；§9.2 加 R2/R7 注记 | ✅ 2026-08-26 |
| T3 | Plan 365（archive）：Status 下加"Windows 裁定被 Design 23 R2 翻转"英文注记（归档计划不改正文） | ✅ 2026-08-26 |
| T4 | Plan 386：状态头加定位更新；启动条件改挂虚拟桌面（原条件保留为历史）；设计输入"Windows host 不是 compositor"条目加翻转注记 | ✅ 2026-08-26 |
| T5 | 程序跟踪文件：452 状态行更新；裁定登记簿 1–3 号核销 | ✅ 2026-08-26 |
| T6 | IME/焦点 spike：Design 23 §9 六项验证，产出 `docs/plans/reports/452-ime-spike.md` | ✅ 2026-08-26（证据：`reports/assets/452-ime-spike/01–07.jpg`；原型留存 `scratch/ime-spike/`） |

### T6 spike 说明

- 一次性原型（建议放 `scratch/` 或独立 example，验收后不并入产品路径），
  验证项以 Design 23 §9 为准：① iced 0.14 单窗口 IME 基线；② 组合输入中
  切换虚拟窗口的定性；③ 全屏 borderless 下候选框定位；④ 双虚拟窗口焦点
  分区（focus id 体系）；⑤ Web 端嵌套 div composition events；⑥ 宿主级
  输入层降级预案原型。
- 产出是结论文档（每项：通过/受限/阻断 + 证据截图/录屏），不是产品代码。
- 若 ②/③ 定性为"阻断"：Design 23 §8 风险 1 的降级预案（宿主级输入层）
  升级为 454 的硬需求，并回写 Design 23 §8/§9。

## 3. 验收

1. Design 23 状态头为正式；§5 表内 20/365/386 三行 ✅（450 行保持 ⬜ 待 454）。
2. `grep -rn "not a compositor" docs/` 的每处命中都伴随 Design 23 R2 翻转注记。
3. spike 报告覆盖 §9 全部六项，IME 风险有了"通过/受限/阻断"的定性结论。
4. 程序跟踪文件仪表盘可回答"386 现在离启动还差几项"。

## 4. 完成定义

T6 报告归档（如有 Design 23 修订则走登记簿）+ 本计划移入 archive；
届时 453（多 App 会话运行时）立项条件即告满足。

---

## 复审记录（2026-08-28 re-review，/auto-plan:review）

> **结论：复审通过，维持 ✅ 完成归档。** 四项验收逐项重验均 pass，无遗漏/
> 无未登记的延后/workaround；spike 结论经 459/462 两轮产品化落地检验仍然成立。
> 复审人：ZCode（auto-plan-review）；基线 master 76dc48a02。

### 逐项验收复核

| # | 标准 | 判定 | 证据（2026-08-28 现场复核） |
|---|---|---|---|
| 1 | Design 23 状态头为正式；§5 表 20/365/386 ✅、450 ⬜ | ✅ pass | `docs/design/autoui/virtual-desktop.md` 状态头"正式（2026-08-26 Plan 452 T1 收编）"；§5 L99-102 四行状态与验收原文逐字一致 |
| 2 | `grep "not a compositor"` 每处命中伴随 R2 翻转注记 | ✅ pass | 命中 4 处：Design 23:37（裁定本体）；archive/365:6/:79（Status 下带日期的英文 Amendment，逐字核验）；本计划验收原文自身 |
| 3 | spike 报告覆盖 §9 六项且有定性结论 | ✅ pass | `reports/452-ime-spike.md`（140 行）：①通过（preedit 缺陷在案）②受限③④⑤⑥通过，**无阻断**；证据 `reports/assets/452-ime-spike/01–07.jpg` + 原型 `scratch/ime-spike/` 均在 |
| 4 | 仪表盘可回答"386 还差几项" | ✅ pass（附新鲜度修正） | 三项条件均带"当前"值；复审时修正一处过期：R4 接缝项"当前：N/A"→"接缝 v1 已由 462 落地，I1 评审未做" |

### 遗漏 / 延后 / workaround 猎查

- **遗漏**：无。T1–T5 的每项文档动作均在当前 master 复核到实物（Design 23 状态头、
  doc 20 横幅、365 注记、386 定位改写、跟踪文件裁定登记簿 1–4 号）。
- **延后**：一项已追踪的跨计划延续——spike ②的两项受限遗留（组合中失焦 discard、
  preedit 落盘）由 Design 23 §8.1 指派给 454（解析号 462），462 执行时显式登记
  "IME 两项残留转 463"（计划 §4 T6 + 跟踪文件 462 行），非静默延期。**注意**：
  Design 23 §8.1 的指针文本仍写"454 的两项明确任务"，按"提案号不回改"规则不动
  正文，当前归属以跟踪文件为准。
- **workaround**：无。本计划为纯文档 + 一次性原型，spike 代码留存 `scratch/` 未入
  产品路径，符合完成定义。

### 后验（spike 结论的产品化检验，452 复审特有维度）

- spike ④（focus id 分区可行）经 462 实机验证再证成立（真实点击 + Unicode 键盘
  输入落到指定虚拟窗口，`examples/ui_desktop.rs`）。
- spike ①的 preedit 语义缺陷与 ②的两项受限 → 463 前置确认项，追踪链完整
  （452 报告 → Design 23 §8.1 → 462 计划 T6 → 跟踪文件）。

### Spec-impact 元数据（本仓 ledger = Design 文档 + 程序跟踪文件；无 .autoos ledger）

- **supersedes_spec_components**: `docs/design/20-autoui-separation-architecture.md`
  §6.1/§9.2（修改）；`docs/plans/archive/365-*`（Windows 裁定翻转注记）；
  `docs/plans/386-autoui-renderqueue-future-optimization.md`（定位改写）
- **new_spec_components**: `docs/design/autoui/virtual-desktop.md`（正式，R1–R7）；
  `docs/plans/autos-desktop-program.md`（依赖图/仪表盘/裁定登记簿）；
  `docs/plans/reports/452-ime-spike.md`
- **touched_goals**: AutoOS 虚拟桌面程序 M0（Design 23 §6）——453+ 立项条件达成，
  后续 459/462 相继落地印证奠基有效
