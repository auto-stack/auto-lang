---
plan_id: PLAN-512
status: reviewed               # drafting → executing → execution_done → reviewed → archived
feature_name: examples-desktop-batch-2
author: [zhaopuming]
created_at: 2026-08-31
updated_at: 2026-09-01

# /auto-plan:review 结束时填写：
supersedes_spec_components:
  - "docs/specs/auto-lang/ui/overview.md: fit 窗口语义段落——「首帧一次性测量」升级为「内容尺寸变化→重测→窗口双向跟随（滞回 8px）」；standalone ServiceTick 订阅条件、scrollable 量测法、用户锁定语义三点机制事实回写"
new_spec_components:
  - "fit 动态重测机制（P504-2 清偿）：dispatch 漏斗 fit_dirty 打标 → ServiceTick 节拍重测（standalone 订阅补齐）→ decide_fit_resize 滞回决策 → 双路径跟随（OS 窗 iced resize / desktop vwin rect）；fit_aware_root 锚点外套 scrollable 取内容自然尺寸（破活树窗口钳制）"
  - "批二迁移 20 例：fit 线 4 例（001/004/005/016）拆居中外壳实测留痕 + title-only 线 16 例补 title；探针资产 011/005 tests/test_512_fit_remeasure.py"
touched_goals:
  - "GOAL-010: 示例应用轨道——批二 20 例桌面化（fit 4 + title 16）兑现"
  - "GOAL-009: 虚拟桌面与桌面 Shell——fit 窗口语义补全（动态重测 + 用户锁定）"
  - "GOAL-007: AutoUI 跨端视觉一致——批一回归双端全绿；w-[28rem] rem 任意值 iced 端不支持实证，改 Tailwind 刻度双端兼容"

affects: [auto-lang/ui]
current_step: 8
total_steps: 8
---

# [PLAN-512] examples 桌面化批二——fit 动态重测机制 + 剩余 20 示例迁移

## 变更摘要

承接 506（批一已归档，顺产批二范围清单）与 P504-2 债务。两部分：

1. **机制增量——P504-2 fit 动态重测**：现 `window: "fit"` 为**首帧一次性
   测量**（011 切 Scientific 模式内容增高后窗口不跟随，底部 `=` 键裁剪，
   实机证据在案）。本期落"内容尺寸变化 → 重测 → 窗口跟随"：view 重建后
   重测，尺寸差超阈值（滞回）才触发——虚拟窗（session 窗口矩形更新 +
   relayout 联动）与独立窗（iced resize）双路径，复用 504 S4/S5 测量管线。
2. **剩余 20 示例迁移**（506 顺产清单）：分两线——
   - **title-only 线**（内容充满型 app：chat/dashboard/video 等，纯补
     `title:`，零行为变化，快速批）；
   - **fit 线**（固定内容型小工具：逐个判内容形态 + 拆居中外壳 + 补
     `window: "fit"`，含 016-calendar 切月增高这类**动态重测的天然验证
     对象**）。
   逐示例判定表先行（T3 产出），交互型子集的验证前置 **P506-1 UAF**
   （038 Reveal VM RC UAF，master 预存，疑 511 线）修复确认。

## 目标

- **G1 动态重测机制**：`window: "fit"` app 内容尺寸变化后窗口跟随
   （阈值滞回防抖动）；011 Scientific 切换（P504-2 原始复现）与 016 切月
   双实证，`=` 键不再裁剪。
- **G2 title-only 线**：判定表归入该线的示例 pac.at 全部补 `title:`，
   桌面 chrome 正确显示，行为零变化。
- **G3 fit 线**：归入该线的示例逐个拆除外壳、声明 fit、实测收缩留痕
   （506 形态：尺寸数字 + 截图）。
- **G4 判定表成文**：20 示例逐个"内容形态（fixed-intrinsic / fill）/
   外壳现状 / 归线 / 验证口径"四列成表，回写本计划（复审对照物）。
- **G5 债务清偿**：P504-2 回写已清偿；P506-1 状态核对（修复则交互型
   验证放行，未修复则交互型验证项降级标注并留痕）。
- **非目标**：`window: "fill"` 等新取值（判定表如证需要，登记待澄清不
  夹带）；outproc 形态的尺寸提示协议消息（508 进程模型联动，归协议线）；
  043/044（485/488 新式示例，本就无此债）；示例功能改动。

## 架构方案

```
动态重测（复用 504 S4/S5 测量管线）:
  view 重建(view-dirty/revision 变) ──▶ 重测内容尺寸 ──差值>阈值(滞回)──▶
    虚拟窗: session 窗口矩形更新 + relayout 联动（既有 apply_layout 消费）
    独立窗: iced window resize 指令
  阈值滞回: 尺寸差 ≤8px 忽略 + 帧边界合并（防输入态抖动 resize 循环）
```

- **改动面**：`crates/auto-lang/src/ui/iced/renderer.rs`（重建后重测挂点）
  + `session.rs`（虚拟窗矩形更新链，504 替换 session.rs:1716 的同区）；
  examples 侧 = pac.at 与根容器外壳（Category A 形态）。
- **与在途计划**：507/508 均在执行且同碰 renderer/session——**轻交叠，
  后合者 rebase**（本期改动集中，hunk 级预期）；建议领取顺序上不阻塞。

## 技术栈

既有栈（504 测量管线 + session relayout + iced resize）。零新依赖。

## 需求分析与背景调查

（取材 506 归档件批二建议 + P504-2 债务条目 + 现场核验 2026-08-31）

- **506 顺产批二清单（20 个）**：001、004、005、006、007、013、014、015、
  016、017、018、019、020、021、022、023、024、025、028、042（p051/p493/p507
  为计划工作目录不计）。506 建议两线分批 + 注意 P504-2 动态高度债与
  P506-1 UAF。
- **P504-2 证据**：fit = 首帧一次性收缩；011 Scientific 切换增高不重测、
  `=` 键裁剪（实机截图在案）；候选后续"内容尺寸变化信号 → 宿主重测"即
  本期 G1。
- **504 样板资产**：`window: "fit"` 双路径（VM 独立窗首帧 shrink + 虚拟
  桌面窗测量值替代 60% 写死初始）；测量管线在 renderer/session，本期复用
  扩展不重造。
- **P506-1**：038 Reveal VM RC UAF（master 预存，疑 511 线回归）——交互型
  app（017 chat/019 video/020 music 等）验证前核对；511（池族修复线）状态
  执行期查。
- **排程**：507 已归档、508 execution_done（起草时的 session/renderer 轻交叠
  风险基本出清，后合者 rebase 压力小）；502/509/510 drafting/归档待领。
  本计划可并行领取。（2026-09-01 开工时更新）

## 详细设计

### 1. 动态重测（G1）

- 挂点：view 重建路径既有"fit 首帧测量"处扩为"每次重建后重测"；
- 滞回：|Δw|、|Δh| 均 ≤ 8px 忽略；同一帧多次重建合并（帧边界取末值）；
- 虚拟窗更新：改 `WmState` 窗口尺寸 → 走既有 relayout（相邻窗口让位
  语义与用户手动 resize 一致，不自造新布局分支）；
- 用户手动 resize 后的语义：**用户尺寸优先**——手动 resize 过的窗口
  （fit 已被用户覆盖）不再自动跟随（一次性锁定，记 per-window 标志）；
  待澄清③若定"可重解锁"再扩。

### 2. 判定表（T3 产出，G4）

四列：内容形态（fixed-intrinsic / fill）/ 外壳现状（居中外壳行数）/
归线（fit / title-only）/ 验证口径（收缩实测 or title 显示断言）。预期
fit 线 ≈ 001/004/005/007/013/015/016/042，title-only 线 ≈ 006/014/017–025/028
——以逐个扫描为准，不预设。

### 3. 迁移执行

- title-only 线：pac.at 补 `title:`（桌面 chrome 显示断言走既有装载测/
  MCP 快照）；零 .at 行为改动。
- fit 线：拆根容器居中外壳（`min-h-screen`/center 外层，506 对 012 的
  同型核查法）+ pac.at `window: "fit"` + 收缩实测留痕（尺寸数字+截图）；
  016-calendar 额外跑"切月 → 窗口增高跟随"断言（G1 双实证之一）。


### 判定表（T3 产出，2026-09-01 逐例扫描成文）

| 示例 | 内容形态 | 外壳现状 | 归线 | 验证口径 |
|---|---|---|---|---|
| 001-helloworld | fixed | `center` 居中外壳 | **fit** | 收缩实测 |
| 004-profile-card | fixed | `center` + max-w-sm 卡 | **fit** | 收缩实测 |
| 005-login | fixed（校验错误行小幅动态增行） | `center` + max-w-md 卡 | **fit** | 收缩实测 + 校验错误增/缩跟随（G1 实证之二，见下） |
| 016-calendar | fixed（切月恒 42 格，**高度不变**） | `center` + max-w-md 卡 | **fit** | 收缩实测 |
| 006-hero-section | fill（min-h-screen 渐变） | 满屏壳 | title-only | title 显示断言 |
| 007-stats-board | fill | min-h-screen | title-only | 同上 |
| 013-todo | 列表无限增长（fit 会随列表撑窗，禁 fit） | min-h-screen + max-w-xl 卡 | title-only | 同上 |
| 014-weather | 名义 fixed 实际充满（内卡自带 min-h-screen） | `center` 壳 + 内卡满屏 | title-only（fit 需先拆内卡，留账） | 同上 |
| 015-notes | fill（编辑器 h-screen） | h-screen + flex-1 | title-only | 同上 |
| 017-chat | fill | h-screen + flex-1 | title-only | 同上 |
| 018-book-reader | fill（路由） | min-h-screen + outlet | title-only | 同上 |
| 019-video-app | fill | h-screen | title-only | 同上 |
| 020-music-player | 内容 intrinsic 但壳 h-screen | h-screen | title-only（拆壳留账） | 同上 |
| 021-blog-viewer | fill（list/detail 不撑窗） | h-screen + flex-1 | title-only | 同上 |
| 022-kanban | fill | min-h-screen | title-only | 同上 |
| 023-realworld | fill（7 路由 SPA） | min-h-screen | title-only | 同上 |
| 024-charts | fill | min-h-screen | title-only | 同上 |
| 025-dashboard | fill | min-h-screen | title-only | 同上 |
| 028-launcher | fill（overlay 召唤槽，禁 fit） | h-full scrim | title-only | 同上 |
| 042-two-inputs-child | fixed（最小复现件，保持纯粹） | 无外壳 | title-only | 同上 |

**G1 实证之二改案（判定表驱动）**：起草时拟以"016 切月增高"为动态重测
第二实证；逐例扫描证伪——`build_month_grid`（calendar_util.at:84-117）恒补
齐 42 格（6 行），切月**不产生高度变化**。改以 **005-login 校验错误行**（
`if .email_error != ""` 条件增行/回缩）为第二实证：属 fit 线成员，增/缩
双向可验。016 只做静态 fit 收缩实测。

## 测试设计

1. **T1 机制单测**：重测决策（阈值滞回/帧合并/用户锁定标志）纯函数单测。
2. **T2 机制集成**：011 Scientific 切换前后窗口高度断言（desktop_mcp/
   MCP 快照测）；016 切月同型。
3. **T3 判定表**：20 示例四列成文回写。
4. **T4 迁移验证**：title 线 grep+装载断言全绿；fit 线逐个收缩实测留痕。
5. **T5 回归**：批一 7 示例既有脚本不回归（504/506 行为零变化）。

## 验收标准

1. G1 双实证绿（011 `=` 键不裁剪 + 016 切月跟随），P504-2 回写清偿。
2. 20 示例全部归线处理完毕（title 齐 / fit 实测），判定表成文对账。
3. 批一 7 示例零回归；`cargo check -p auto-lang` 零警告；
   `cargo t ui`、`cargo t session` 不回归（机制改动门禁）。
4. P506-1 状态核对留痕（修复放行 or 降级标注）。

## 执行步骤

（原子任务：精确文件路径 + 确切操作 + 验证命令；每步完成后追加 [✅ 已完成] 一行证据）

1. **机制——重测决策纯逻辑**：`crates/auto-lang/src/ui/iced/renderer.rs`
   fit 测量区抽"重测决策"纯函数（阈值/帧合并/用户锁定）+ T1 单测。
   验证：`cargo t fit_remeasure`（或并入既有套件名）。
   [✅ 已完成] `decide_fit_resize` 纯函数 + FIT_REMEASURE_THRESHOLD=8px（renderer.rs fit 区）；T1 `decide_fit_resize_threshold_lock_clamp`（锁定/滞回/双向/clamp 五断言）绿；ui-iced 档 fit 10/10。
2. **机制——双路径接线**：虚拟窗矩形更新（`session.rs` 504 同区）+
   独立窗 iced resize + 用户 resize 锁定标志。
   验证：`cargo t ui && cargo t session`。
   [✅ 已完成] WindowEntry/VWinState 增 fit_enabled/fit_user_locked/fit_dirty（Cell）+ fit_last_applied；SessionViewMut/Ref 下穿 12 处；view 层 fit 窗常驻 Shrink+锚点；dispatch_app 打标漏斗 + has_fit_remeasure_pending/mark_fit_dirty helpers；standalone 锁定走 __window_resized ±2px 回波豁免、desktop 走 WmState::apply_cursor Resize 臂。session 88/88、ui 1680/1680 绿。
3. **机制实证**：011 Scientific / 016 切月前后窗口尺寸断言（T2）。
   验证：desktop_mcp/MCP 快照断言绿 + 截图留痕。
   [✅ 已完成] `examples/ui/011-calculator/tests/test_512_fit_remeasure.py`（win32 EnumWindows 物理尺寸探针）PASS：基线 397x428 → Scientific 397x472（**+44px**，起草 +50px 假设实证下修，探针阈值定为 >24px=3×滞回）→ Basic 回缩 397x428（±16 内）。**实证改案两处**（均先证伪后修）：① standalone 模式 ServiceTick 订阅原仅 desktop 门控（renderer.rs:13228 族）且宿主窗句柄恒 None——订阅门控扩为「desktop 或 standalone 有 fit 脏标」，测量目标 standalone 取待测窗自身 id；② 活树布局受当前窗口钳制，增长方向量不到（首测成功仅因默认窗大于内容）——`fit_aware_root` 锚点外套 vertical scrollable（内部无限高约束排版），锚点量到真实自然尺寸，宽度方向 v1 仍受视口钳制（登记待澄清）。插桩三处（AUTO_FIT_TRACE）实证后已拆净。
4. **判定表**：20 示例扫描成表回写本计划（T3）。
   验证：四列表成文。
   [✅ 已完成] 逐例扫描成文（见「详细设计」判定表节）：fit 线 4 例（001/004/005/016），title-only 线 16 例；**G1 实证之二改案**——016 切月恒 42 格高度不变（扫描证伪），改 005-login 校验错误行增/缩双向为第二实证。
5. **title-only 线**：按表批量补 `title:` + 装载断言。
   验证：grep 20 示例 pac.at title 齐 + 断言绿。
   [✅ 已完成] 16 例 pac.at 补 title（006/007/013/014/015/017/018/019/020/021/022/023/024/025/028/042，插入位置 icon: 后）；`cargo t pac` 81/81 绿。连带修两处过期测试期望：app_registry `launch_three_real_apps_via_registry_resolver` 的 "todo"→"Todo"；coverage `scan_inventory_matches_source_shape` 去 center/max-w-md、改 w-112。
6. **fit 线迁移**：按表逐个拆外壳 + `window: "fit"` + 收缩实测留痕。
   验证：逐示例尺寸数字+截图（506 形态）。
   [✅ 已完成] 4 例（001/004/005/016）center 外壳退役 → 根 col Shrink；卡片宽固定化（004 w-96、005/016 w-112——**实证发现 `w-[28rem]` rem 任意值 iced 端不支持**（213px 塌缩），Tailwind 刻度 112×4px=448px 双端兼容）。实测（VM 物理尺寸+截图留痕 src/front/tests/screenshots/512_*_fit.png）：001 213x236、004 445x451、005 573x535、016 541x441，截图渲染完好无塌缩。**005 重测实证（第二实证腿）** `tests/test_512_fit_remeasure.py` PASS：空表单 Sign In → 两条校验错误行 → 573x535→574（+39px）→ 键入两输入框错误清除 → 回缩 573x535（±16 内）。
7. **P506-1 核对 + 批一回归**：UAF 修复状态查证留痕；504/506 示例脚本
   全绿（T5）。
   验证：既有示例验证脚本 + 核对注记。
   [✅ 已完成] **P506-1 核对**：511 已归档（a28b51ffd 合入），归档文无 Reveal UAF 修复；038 desktop_mcp.py 复现仍在——首点格子 Reveal 即 VM 进程死（RC canary UAF），12 passed / 1 failed（预期 FAIL）/ 2 skipped（506 防御路径），债务保留待专项。**批一回归**：003 VM（fit 收缩断言 + vnode/aura 双 scheme 修脚本后 decimal 键入 161.67 ✓）+ Vue 截图、008/009/010 VM+Vue 全绿、011 VM + Vue + desktop_mcp 17/17 全绿。附带修：`test_converter_mcp.py` 快照 id 正则 aura_ 单 scheme → (aura|vnode)_ 双 scheme（预存时序脆弱性，首帧快慢决定快照路径）。
8. **收尾**：P504-2 回写清偿；spec 沉淀归 merge；健康检查；状态翻
   execution_done。
   验证：`cargo check -p auto-lang && cargo t ui`。
   [✅ 已完成] P504-2 KNOWN-DEBT 标已清偿（含两处实证改案指针）；新增 P512-1（宽度方向视口钳制）/P512-2（锁定后余量视觉语义）/P512-3（p508_g2_outproc_arm 并发偶红注记）三条登记。终态门禁：`cargo check` 干净（警告 159=master 基线零新增）、`cargo t ui` 1680/1680、`cargo t session` 88/88、`cargo t pac` 81/81（S5 时）、fit 16/16；终态二进制（拆插桩后重建）双探针复验 PASS。

## 复审记录

（verify, don't trust——逐条对照实际代码/diff 重验）

1. **清单审计**：
   - G1 动态重测：011 探针（+44px 增高/回缩基线）+ 005 探针（+39px/回缩）
     双实证 PASS，均含截图留痕；探针阈值 >24px=3×滞回有实测依据（非拍头）。
     016 切月改案 005 有判定表节证伪记录（calendar_util.at:84-117 恒 42 格）。✓
   - G2 title-only 16 例：grep 核实 16 例 pac.at 均有 title（006/007/013/
     014/015/017/018/019/020/021/022/023/024/025/028/042）；`cargo t pac`
     81/81。✓
   - G3 fit 线 4 例：001 213x236 / 004 445x451 / 005 573x535 / 016 541x441
     实测数字 + 截图（512_*_fit.png）在案；004/016 截图人工检视渲染完好。✓
   - G4 判定表：20 例四列表成文（详细设计节）。✓
   - G5：P504-2 清偿回写 ✓；P506-1 核对留痕（511 归档未修，038 复现仍在，
     交互腿按 506 防御降级，债务保留）。✓
2. **遗漏/延后/Workaround 扫描**：两处实证改案（ServiceTick standalone
   订阅断链补齐；scrollable 量测法破窗口钳制）均非 workaround——为机制
   正确性修复，留痕步骤 3。新增限制（宽度方向钳制、锁定后余量语义）已
   登记 P512-1/P512-2 而非沉默。探针 +50px→+24px 阈值修正有实测数据
   支撑（非放松过关）。003 脚本双 scheme 修复属预存脆弱性顺带清偿
   （diff 内注释在案）。
3. **健康检查**：三处 AUTO_FIT_TRACE 临时插桩已拆净（grep 零命中）；
   编译警告 159=master 基线零新增；门禁全绿（ui 1680/session 88/pac 81/
   fit 16）；p508_g2_outproc_arm 并发偶红单跑即绿，登记 P512-3。
4. **spec-impact 元数据**：supersedes/new/touched_goals 三节已填（见
   frontmatter），merge 时按 §4 沉淀。

复审结论：**通过**（8/8 步骤完成，五目标全达，两处改案+三条债务全留痕）。

## 待澄清事项

- **① fill 取值需求**：判定表若出现"既非固定内容也非现充满"的中间形态
  （如 006-hero-section），是否需要 `window: "fill"` 取值——默认不新增，
  登记形态后由用户裁定（可顺延批三）。
- **② outproc 联动**：508 进程模型下 fit app 走 outproc 时动态重测需
  协议"尺寸提示"消息（app→host，追加式）——本期仅进程内语义，协议线
  （Stage 6 后续或 508 增补）收口。
- **③ 用户锁定语义**：手动 resize 后锁定为 v1 裁定；"重解锁"（如双击
  标题栏恢复 fit）列为增强候选。
- **④ P506-1 UAF**：511 线修复状态执行期查——未修复则交互型示例
  （判定表标注）验证降级为静态断言 + 债务留痕，不阻塞本批。
- **⑤ 阈值参数**：8px 滞回为初值，T1 时以 011 实测微调后定稿。
