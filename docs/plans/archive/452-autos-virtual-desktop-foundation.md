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

1. Design 23（`docs/design/23-autoui-virtual-desktop.md`）从草案转正式，
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
