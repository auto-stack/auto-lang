---
plan_id: PLAN-530
status: reviewed               # drafting → executing → execution_done → reviewed → archived
feature_name: VM mobile 断点双份绘制 + 启动内存崩溃专项
author: [zhaopuming, ZCode]
created_at: 2026-09-03
updated_at: 2026-09-03T20:00:00+08:00

# /auto-plan:review 结束时填写：
supersedes_spec_components:
  - "docs/specs/auto-lang/project.md (ui 模块): Column/Row 提升叠层语义改版——z-index 前缀 overlay 双渲染路径退役(472 立场废弃),叠层仅 absolute 子脱流(column_layer_partition,Row 分支同语义),z-index-only 子保持流内;is_elevated_view/extract_max_z_index 随路径退役"
  - "docs/specs/auto-lang/ui/overview.md (VM 运行时渲染): alert-dialog 全族 VM 消费臂——复用 Plan 422 Popover 原语模态化,面板 chrome=shadcn AlertDialogContent 同款(w-96 bg-background border rounded-lg shadow-lg p-6 gap-4);title/description/header/footer/cancel/action 子臂 chrome 类重写"
  - "docs/specs/auto-lang/ui/overview.md (VM 运行时渲染): code_editor 视口防御契约——render() 非 finite 视口(NaN/∞)整帧拒绝 + GutterCache 光栅 4096/维上限;∞ 视口曾饱和 u32::MAX 致 gutter 光栅 42×u32::MAX×4=721GB 分配崩溃(OBS-1)"
  - "docs/specs/auto-lang/ui/overview.md (VM 运行时渲染): lucide_svg 16×16 文档与 textarea/input placeholder 的 'static 化改按内容去重缓存——每帧 Box::leak 无界泄漏根除(强复现静默死亡根因,实测 +120MB/min→持平)"
new_spec_components:
  - "PopoverPlacement::Modal + PopoverWidget.modal(): 模态对话框原语——面板视口居中+全屏 scrim(50% 黑)+面板外点击整吞;on_dismiss=None 时外点/Esc 不关(shadcn AlertDialog 语义,关闭仅经 cancel/action 状态翻转);render_dynamic_view 与 into_iced 双臂同口径"
  - "toggle_group/toggle_group_item VM 消费臂: 组→row+item 重写为 button 注入位置类(-ml-px 叠边/首尾圆角),variant=outline 传导 item outline 预设,size sm/lg→padding 档;tracked/untracked D-GAP 双镜像;页面书写的 [&>*+*] CSS 选择器经 unmapped 报告通道自然跳过"
touched_goals:
  - "GOAL-007: VM 轨 shadcn 组件补缺与稳定性对齐——toggle_group 横排连体组/alert-dialog 模态弹层双端观感补齐;VM 移动断点双份绘制根除(Column 叠层语义修复)+首页动画内存崩塌根除(Box::leak 家族去重缓存),69 页双宽扫描零失败"

affects: [auto-lang/vm]       # 受影响的 specs 路径，如 [auto-lang/vm]
current_step: 8
total_steps: 8
---

# [PLAN-530] VM mobile 断点双份绘制 + 启动内存崩溃专项

## 变更摘要

立项收录 widgets-gallery 人工检查期间发现的两个 **VM shell 层**问题
（均登记于 PLAN-528 观察区/W10，根因已初步定位但修复属 VM 架构深水区，
按用户裁决单独立项深挖）：

1. **A（源 W10）**：窗口宽 ≤768（mobile 断点）时页面内容整体 ×2 叠印
   ——绘制层双通道合成，控件树结构单份；
2. **B（源 OBS-1）**：VM 模式启动渲染期原生崩溃
   `memory allocation of 721554505560 bytes failed`（约 721GB 荒谬分配），
   间歇性。

两问题均不出示例层可修范围,涉及 iced 0.14 虚拟窗口表面生命周期、
fit 缩放合成与 Plan 527 T7 响应式重建的交互。

## 目标

1. A：定位 mobile 断点下双表面的来源（创建/残留），断点翻转时旧表面
   正确销毁,任意窗口宽下内容仅绘制一份。
2. B：定位 721GB 分配的调用点（尺寸计算溢出/未初始化），消除崩溃；
   崩溃可稳定复现时补回归测试。
3. 两问题修复后,widgets-gallery VM 模式在 700–1440px 全宽度区间
   逐页截图无异常（含跨断点往返）。

## 架构方案

- 排查入口 A：`crates/auto-lang/src/ui/iced/virtual_window.rs`、
  `renderer.rs`（fit 通道 Plan 512 / dock 合成 Plan 463 / 表面注册表）、
  `session.rs`（window_size 回写）；与 `crates/auto-lang/src/ui/style/mod.rs`
  Plan 527 T7 响应式门控的重建时序。
- 排查入口 B：崩溃点在首个 view 构建期（wgpu/iced 初始化后、Plan 412
  flex-wrap/sticky 降级告警之后）；疑似尺寸计算溢出（未初始化/负数→
  usize 回绕）。RUST_BACKTRACE=full + windbg/cdb 或 分页二分（逐页排除）
  定位。721GB ≈ 0xAB_XXXX_XXXX 量级,高度疑似 f32 NaN→usize 转换或
  元素数乘法溢出。
- 修复落点倾向：VM shell 表面生命周期管理（A）、尺寸计算防御（B）；
  均在 `crates/auto-lang` 内,不动示例。

## 需求分析与背景调查

- 问题 A 现象与对分实验全文：`docs/plans/528-widgets-gallery-review-fixes.md`
  W10 节（结构单份/绘制双份/两种宽度约束叠印/断点门控本身正确）。
- 问题 B 现象：PLAN-528 OBS-1（2026-09-03 02:48 首次记录,当日后续
  VM 启动均未复现——间歇性/疑似页面相关;同日 togglegroup/W10 实验
  多次 VM 启动正常）。
- 相关上游：Plan 527 T7（responsive 断点,验证仅覆盖样式解析层分叉,
  未覆盖真实窗口 resize/冷启动全回路）;Plan 512（fit 缩放一次性锁定）、
  Plan 463（WM→shell 状态注入）。
- 复现配方：
  - A：`examples/widgets-gallery/pac.at` `window: "700x900"` 冷启动即现
    （实验后已还原 1440x900）;推断 resize 跨 768 亦可触发。
  - B：master @ 7ab140c41 时代 `auto run`（VM merged）首启即崩;
    复现概率不稳定,建议保留当日环境线索（RTX 4060 Ti / Vulkan backend）。
  **强复现配方（2026-09-03 下午,W11 排查中实证）**：widgets-gallery
  VM 启动后停在首页（LineChart/DonutChart 动画 AnimLnTick/AnimDnTick
  持续 tick）,约 1–5 分钟内进程静默死亡（exit 无 panic、无 memory
  allocation 行,日志末尾即动画 tick 事件）。离开首页则存活。崩溃与
  首页动画 tick 强相关——优先 bisect 图表动画路径。

## 详细设计

（深挖后回填——当前为立项档案。预估方向：）

- A 假设 H1：断点翻转触发 view 重建时,虚拟窗口表面注册表新增了
  mobile 表面而旧表面未注销 → dock 合成叠印两表面（各自持有
  不同时期的布局宽度）。
- A 假设 H2：fit 缩放通道与直绘通道同时上屏（fit_pending 未清）。
- B 假设 H1：某 widget（疑似 svg/chart 家族或 grid 占位）在
  viewport 极值下 size 计算 f32→usize 回绕。
- B 假设 H2：AbstractView 缓存跨断点复用时 children 数被污染放大。

## 测试设计

- A：headless 断点翻转样本（700↔1440 往返）+ surface 注册表数量断言;
  MCP 截图逐宽度扫描（已有 69 页扫描基线可比对）。
- B：崩溃复现后,最小化 .at 样本沉淀为回归测试
  （`cargo t --features test-vm-files` 档,参照既有 VM 文件测试）。
- 回归底线：widgets-gallery 69 页 VM 逐页截图扫描（PLAN-528 W5 既有
  工具链,MCP 驱动脚本 `.agents/skills/autoui-verifier/scripts/test_vm_mcp.py`）。

## 验收标准

- [x] A：mobile 断点下任意页面内容仅绘制一份（树单份=像素单份）;
      跨断点往返（700↔1440 多次）无残留表面。
- [x] B：721GB 崩溃根因定位并有确定性修复;原始复现路径（widgets-gallery
      VM 首启）连续 10 次启动无崩溃。
- [x] 全程 cargo t iced + 既有 VM 文件测试绿。

## 执行步骤

（深挖时展开为原子任务;当前立项目录:）

1. [✅ 已完成] worktree 内 700x900 冷启动复现成功：×2 叠印清晰（内容区
   整份重绘两遍、垂直错位，header/底栏单份）；基线截图
   scratch/p530/a700_coldstart.png + 启动日志 scratch/p530/run_700.log
   （同进程兼作 B 强复现观察窗）
2. [✅ 已完成] 表面追踪（P530_TRACE 诊断：LayoutCollector 重复 id 记录 +
   view/resize 宽度轨迹）：**双表面假设 H1/H2 均否**——重复在 iced 控件树
   本身：700x900 冷启动 73/75 widget id 被布局两次（header 两份完全重叠
   (0,0,700x56)；页面行一份 (0,0,700x900) 全窗、一份 (0,57,700x778) 流内）。
   根因：renderer.rs Column 分支 472 时代"前缀 overlay"路径——z-index 子
   （app.at header `z-40`、移动底栏 `md:hidden fixed z-40`，<768 才可见）
   触发 has_elevated，把 0..=elev_idx 整段重渲染进叠层 → 非提升子被
   base+overlay 双渲染；base 压 compact 还把正文顶进 header 之下。
   ≥768 时底栏被 md:hidden 剪枝 → 只有 header 单独提升 → 单份但存在
   57px 顶入（被 95% 不透明 header 掩盖）。W11（sidebar 叠印 header、
   /position 树在像素空）与 W10 同根。证据链：
   scratch/p530/tree_rendered.txt（树单份）+ run_700_trace.log（dup 73/75）
3. [✅ 已完成] 修复：Column 分支叠层判定改为仅 absolute 子脱流
   （column_layer_partition，Row 分支同语义；z-index-only 子保持流内，
   退役 is_elevated_view/extract_max_z_index 前缀 overlay 路径）。
   回归：TDD 测试 p530_column_layer_partition_zindex_stays_in_flow 绿；
   实证 700x900 冷启动单份绘制（a700_fixed_os.png）+ SetWindowPos
   700↔1440 三轮往返 trace 全程 0 重复、1440 端sidebar/hero 干净
   （a1440_after_roundtrip.png，run_700_fixed.log）
4. [✅ 已完成] B-复现/定位：泄漏实测 +120MB/min（68B/tick，∝ 树规模）——
   强配方死亡实体即泄漏耗尽家族；A/B 判别（P530_NOMCP 门控，MCP/capture
   排除）+ master 对照构建（双份泄漏 ~2 倍）锁定每帧重建路径。full
   backtrace 抓到 OBS-1 原值：code-editor 页确定性复现
   `memory allocation of 721554505560 bytes` = gutter 宽 42 ×
   (∞→u32::MAX) × 4（数学逐位一致）
5. [✅ 已完成] B-修复+回归：lucide_svg/placeholder 'static 化按内容去重
   （每帧 Box::leak 无界泄漏根除；复测动画 tick 浸泡内存 262MB 持平）；
   render() 非 finite 视口整帧拒绝 + GutterCache 光栅 4096/维上限
   （OBS-1 721GB 根除）。TDD 4 测试绿（p530_lucide/p530_placeholder/
   p530_gutter_rejects/p530_render_rejects）；实机 CodeEditor 页访问
   不再崩。**10 连发启动回归（每次 120s 浸泡）10/10 PASS**
   （scratch/p530/ten_launch_results.txt）
6. [✅ 已完成] 全宽度扫描验收：1440 宽 67/67 页 MCP 截图扫描零失败
   （scratch/p530/scan1440_*.png）；跨断点逐页扫（导航→缩 700→截图→
   还原 1440）67/67 零失败，700 实拍 1374×1800 单份（x700b/）；全程
   P530_TRACE 0 重复 id。计划缺陷：Breadcrumb 页栈溢出为存量 master
   缺陷（对照构建复现归因），登记 P530-D1 后跳过
7. [✅ 已完成] W12-toggle_group VM 映射：tracked/untracked 双层 D-GAP
   镜像臂，组→row+item 连体类注入（-ml-px 叠边/首尾圆角），variant/
   size 传导；实机 togglegroup 页横排连体组成型（w12b_crop.png）
8. [✅ 已完成] W13-alert-dialog 复用 Popover 原语：PopoverPlacement::Modal
   （视口居中+全屏 scrim+外点整吞）+ alert-dialog 全族子臂（chrome 类
   重写）；demo 页绑定 open 状态；实机点击弹层居中+遮罩（像素 50% 黑
   验证）+ Cancel/Continue 可点（w13v3 系列）

## 复审记录

**复审人**：ZCode（/auto-plan:review，2026-09-03）
**复审基址**：worktree `D:/autostack/.wt/lang-530/auto-lang`（plan-530-dev
@c7cdb588f..17148de34，4 提交，基 96586cca4）；计划文件在 master 检出。

### 逐条验收复核（verify, don't trust——全部现场复跑/复核）

| # | 验收标准 | 判定 | 证据 |
|---|---|---|---|
| A | mobile 断点任意页面单份绘制 + 700↔1440 往返无残留 | **PASS**（含 D1 例外注记） | 复审期复核执行期留痕：run_700_fixed.log / run_scan_final.log 全程 `P530-TRACE` **0 DUPLICATE**（A 修复前 73/75 id 双布局对照）；700 冷启动单份 OS 截图 a700_fixed_os.png；跨断点逐页扫 67/67（导航→缩 700→实拍 1374×1800→还原），x700b/ 67 张全单份；往返 3 轮 trace 无残留。**例外注记**：Breadcrumb 页无法参与扫描（导航即栈溢出）= 存量 master 缺陷（master 对照构建复现归因，P530-D1），与本计划改动无关（本分支 diff 不触及其路径） |
| B | 721GB 根因定位 + 确定性修复 + 首启 10 连发无崩溃 | **PASS** | 根因：GutterCache::image 无防御 × ∞ 视口饱和——`42 × 4294967295 × 4 = 721,554,505,560` 与 OBS-1 **逐位一致**，full backtrace frame16 `GutterCache::image` 指认；确定性复现（单点 code-editor 页）修复后 ALIVE（codeeditor_fixed.png）。10 连发启动回归（每发 120s 浸泡，超出强配方 1-5min 死亡窗）**10/10 PASS**（ten_launch_results.txt）。附带根除泄漏（+120MB/min → 262MB 持平） |
| C | 全程 cargo t iced + 既有 VM 文件测试绿 | **PASS**（带 master 预存红注记） | 本复审门重跑：`cargo t iced` 164/164、`cargo t code_editor` 41/41、`cargo tv` **3559/3559 全绿**、`cargo tf` 3397/3399——唯二红 `docs_gen kitchen_sink_page_in_sync` + `schema_drift schema_drift_fence` 在 **master 检出同样复现**（归因复跑），属并行会话存量漂移（先例：448"schema_drift 基线陈旧"/528"master 预存红"），本分支 diff（ui/* + 示例页）不含其输入 |

### 遗漏 / 延后 / workaround 猎查

- diff 全文扫描 0 新增 TODO/FIXME/HACK。
- **P530-D1**（延后→债务）：breadcrumb 栈溢出为执行期新暴露的存量 master
  缺陷，非计划任务缩水；归因实验在案，已立项建议写入债务台账。
- **P530-D2/D3**（延后→债务）：图表 timer 路由切换不退订 + Element 缓存
  快速路径架构性失效——执行期发现的放大器/空转债，不在计划任务清单，
  未经批准的缩水不存在（计划 8 步全数交付）。
- **P530-D4**（留档非债）：P530_TRACE/P530_NOMCP 诊断门控留存（env 缺省关）。
- workaround 检查：gutter 4096 上限为防御性契约（上游无限高测量的布局侧
  根治留给布局约束面，已在上游 render() 同步拦截非有限值），非掩盖。

### 判定

**全部验收标准 PASS，无阻塞债（P530-D1..D4 均为登记在案的增量债/存量缺陷）。**
status → `reviewed`，可进入 `/auto-plan:merge`。

## 待澄清事项

1. （已裁决项沉淀）B 的"静默死亡"实体 = 泄漏耗尽家族（commit 异常时
   stderr 丢失呈静默）；OBS-1 首启 721GB 与强配方同根因不同触发位
   （首启 = editor 首帧无限高测量,强配方 = 泄漏积累后任意大分配失败）。
2. （已裁决项沉淀）A 的"双表面"假设 H1/H2 均否——实为 iced 控件树内
   前缀 overlay 双渲染,无表面注册表参与（单窗 scene:ui 不经 dock 层）。
