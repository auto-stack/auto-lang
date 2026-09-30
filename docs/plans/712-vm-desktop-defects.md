---
plan_id: PLAN-712
status: drafting               # drafting → executing → execution_done → reviewed → archived
feature_name: VM 桌面验收缺陷收敛（视频引擎双缺陷 + 壳配置持久化 + examples 依赖）
author: [zcode(auto-os 会话转介)]
created_at: 2026-09-30
updated_at: 2026-09-30

# /auto-plan:review 结束时填写：
supersedes_spec_components: []
new_spec_components: []
touched_goals: []             # 引用 docs/specs/goals.md 的 GOAL-NNN

affects: [auto-lang/ui]
current_step: 0
total_steps: 8
---

# [PLAN-712] vm-desktop-defects

## 0. 变更摘要

VM 虚拟桌面实机验收（auto-os 侧 2026-09-30 会话）暴露的 auto-lang 引擎侧缺陷收敛，初始范围四项：

- **A. video 受控契约上行死链**：`mpv::widget::drain_events` 无生产调用方，`ontimeupdate`/`onloadedmetadata` 永不分发 → 应用层 duration/position 恒 0 → 进度条拖动无效。
- **B. mpv SW 上屏色彩错配**：VM 端视频实测整体偏暗偏暖（letterbox 纯黑排除叠层），与 Web 端（Chromium `<video>`）对照色度不同——纹理 sRGB 采样线性化与目标 pass 编码配对、mpv 色彩协商两候选机制，探针定案后修复。
- **C. desktop_config save() 首写无快照缺口**：进程首次 save 走 `_ => cfg.clone()` 整份覆盖，陈旧内存态冲掉手编/外写字段（2026-09-30 10:34 实录：壁纸槽被冲回 `#101014`）。
- **D. examples 生成产物依赖失效**：`rust-workspace` 系 Cargo.toml 引用已退役 feature `ui-gpui`，`examples/ui/030-video-player` 无法独立 `auto run -r vm`，挡住 A/B 的独立窗验证载体。

本计划同时是**桌面 VM 验收缺陷的收纳载体**：后续其他桌面 app 的 auto-lang 侧问题经 plan_revision 增补任务/AC 归入本计划（见 §10）。

## 1. 目标

- **G-1**：video 契约上行双端一致——VM 端 `ontimeupdate`/`onloadedmetadata` 真实分发到 `.at` handler，播放器时长/进度/seek 全链可用。
- **G-2**：VM 端视频上屏色彩与 Web 端对照一致（同源视频双端像素对比无可见偏暗/偏暖），letterbox 保持纯黑。
- **G-3**：config.at 手编/外部写方字段在任意运行实例（含首写 save）之后存活；PLAN-044 T-03 的合并语义补全为无首写例外。
- **G-4**：`examples/ui/030-video-player` 可独立 `auto run -r vm` 启动，作为 A/B 的独立窗验证载体。

**非目标**：Vue/Web 端行为变更；mpv 运行库分发基建（auto-os 侧已落 `D:/autostack/tools/mpv/`）；desktop.sh/ps1 的 mpv-widget 构建门（auto-os 侧已修）；020 音乐播放器等既有媒体面债（auto-os PLAN-037 F-R4 在案）。

**受影响仓/模块**：仅本仓 `crates/auto-lang`（ui/mpv、ui/iced、ui/desktop_config）与 `examples/`。Category A 适用（本计划改 crates，`cargo t` 允许）。

## 2. 架构方案

- **A（上行分发）**：在 iced video 节点外包裹事件泵 wrapper（先例：`iced/seek_area.rs` 的薄包装形态）——`update` 臂调用 `mpv::widget::drain_events(id)`，把 `VideoContractEvent` 按 `.at` 侧 `ontimeupdate`/`onloadedmetadata` 绑定合成应用消息 `shell.publish`。备选：runtime 订阅轮询 + 全局 id 注册表。决策点 **DP-1**（见 §5）。
- **B（色彩）**：探针确认 iced_wgpu 0.14 实际交给 `Pipeline::new(format)` 的目标格式（独立窗 surface vs 桌面 layer 两形态，决策点 **DP-2**），据此修复编码配对（shader 内编码 / 双管线 / 非 Srgb 纹理三选一）；mpv 色彩协商（colormatrix/range）以探针 **DP-3** 定案后显式化。
- **C（配置）**：`DesktopConfig::load()` 尾部把盘上解析结果同步进 `LAST_SNAPSHOT`，使进程首次 save 即走字段级合并；「盘上无文件的 boot 迁移首写」保持调用方整份（语义无外写可保）。
- **D（examples）**：探明 `ui-gpui` 出自生成器模板还是历史生成产物，取小修复（改模板重生成 vs 产物批量替换为现行 feature），决策点 **DP-4**。

## 3. 技术栈

Rust；iced 0.14 / iced_wgpu 0.14（shader 自定义 Primitive 管线）；libmpv SW render（运行时加载，构建面 mpv-widget 已在 auto-os 侧脚本接通）；既有 `cargo t` 基建；实机验证走 auto-os 桌面（ui_desktop + 030-video-player，媒体根 `E:\Video`）。

## 4. 需求分析与背景调查

**授权记录**：用户 2026-09-30 裁定（auto-os 会话）——「在 auto-lang 建立一个计划；其他桌面 app 的问题（涉及到 auto-lang）也可以归入该计划」。初始范围 = 本日实机验收发现 + 代码实证，无额外预算/自动续跑限制。

**证据清单**（2026-09-30，auto-lang 主检出现场；实证会话 = auto-os ZCode 会话 sess_d65fce0c）：

| # | 事实 | 出处 |
|---|---|---|
| E-1 | `drain_events` 全 src 仅注释与测试引用，无生产调用方 | `grep -rn drain_events crates/auto-lang/src`（命中：`ui/mpv/widget.rs:141` 定义、`ui/iced/renderer.rs:6427` 注释、`ui/view.rs:970-972` 注释） |
| E-2 | 桌面内播放截图：进度 `00:00 / 00:00`、OSD `时长 00:00`，视频画面在播 | 实机截图 `C:\Users\zhaop\.zcode\cli\image-cache\sess_d65fce0c\image-49f2dd78d1e77e8f8b912b3c2c0891d7.png` |
| E-3 | letterbox 实测纯黑 (0,0,0)——排除半透明叠层变暗；内容白区 (251,237,230) 相对 app 铬层白 (251,248,242) 呈暖偏（R 平、G/B 压） | 同截图 PIL 像素探针（auto-os 会话实测） |
| E-4 | 帧纹理 `Rgba8UnormSrgb`（采样线性化）；渲染管线目标格式由 iced 经 `Pipeline::new(format)` 交付；管线 `blend: None` 全量覆写 | `crates/auto-lang/src/ui/mpv/channel.rs:256`、`ui/mpv/widget.rs:403-422`、`ui/mpv/present.rs:117-141` |
| E-5 | `render_sw_frame` 仅传 SW_SIZE/SW_FORMAT(rgb0)/SW_STRIDE/SW_POINTER，无色彩协商参数 | `crates/auto-lang/src/ui/mpv/engine.rs:287-320` |
| E-6 | `save()` 合并臂 `(Some(base), Some(snap))` 之外走 `_ => cfg.clone()` 整份覆盖；`LAST_SNAPSHOT` 仅 save 成功后写入（load 不写） | `crates/auto-lang/src/ui/desktop_config.rs:533-566` |
| E-7 | 实录：旧桌面实例（修复前内存态）退出 save 把壁纸活跃值+深槽冲回 `#101014`（config.at mtime 2026-09-30 10:34:23）；Plan 043 归档在册同款（「多实例写竞态把活跃壁纸冲回 #101014」） | auto-os 会话实测 + `auto-os docs/plans/archive/043-desktop-app-vm-check.md:427,555` |
| E-8 | `examples/rust-workspace/011-calculator/Cargo.toml` 等一族生成产物引用 `ui-gpui`；`cargo` 解析报「does not have that feature」致 `auto run` 失败（现行 feature 集见报错列表：ui/ui-iced/mpv-* 等） | `auto run -r vm` 失败日志（auto-os 会话 `.auto/vm-standalone.log`） |
| E-9 | spec 面：媒体元素节（617 SD-05 沉淀）与 desktop_config 持久化面均在 `docs/specs/auto-lang/ui/overview.md` | `docs/specs/auto-lang/ui/overview.md:1183` 邻域及 617 归档 SD-05 |

**依赖/约束**：A/B 的实机验证依赖 auto-os 侧已就绪的桌面环境（mpv-widget 构建 + AUTO_MPV_LIB + 媒体根）；D 是 A/B 独立窗验证的前置。

## 5. 详细设计

### A. video 上行分发（→ SD-01）

- 现状：`VideoPrimitive::prepare`（渲染路径）把契约事件堆进 `rt.pending`；`drain_events` 是唯一出口但无人调用；view.rs 的 video 节点明示「不携带任何消息」——上行分发面整体缺失（E-1/E-2）。
- 设计方向：video 节点构建时（renderer.rs `AbstractView::Video` 臂，`renderer.rs:6444` 邻域）已知 widget id 与 `.at` 侧事件绑定（`ontimeupdate: .OnTime($0)` / `onloadedmetadata: .OnDuration($0)`，契约注释 `mpv/contract.rs:7-10`）。在 shader widget 外包事件泵 wrapper（seek_area 先例形态）：`update` 臂每次调用 `drain_events(id)`，按绑定把 `TimeUpdate(f64)`/`Duration(f64)` 合成为应用消息 `shell.publish`。节流沿用契约既有 0.25s 门（`contract.rs:119-122`）。
- **DP-1（工作期定案）**：wrapper update 臂逐事件泵（每帧 update 常发，成本 = 一次 HashMap 读）vs iced Subscription 轮询。默认取向：wrapper 臂（无新基建、与事件天然同帧）。
- 验收锚点：时长显示非 00:00；按住拖动进度条位置连续跟随（`SeekTo(.duration * $0)` 生效）。

### B. 上屏色彩（→ SD-02）

- 机制候选（E-3/E-4/E-5）：
  1. **sRGB 配对断裂**：Srgb 纹理采样线性化后，若 `Pipeline::new` 收到的目标格式为非 Srgb（Bgra8Unorm 类），线性值被当 sRGB 显示 → 中间调压暗、整体蒙灰。修复 = 按 DP-2 实测格式配对（shader 内 linear→sRGB 编码 / 双管线按格式选择 / 改用非 Srgb 纹理直通），三选一在工作期定案。
  2. **色彩协商**：mpv SW 输出未显式协商 colormatrix/range（E-5），与 Chromium 启发式的差即「色度不一样」。**DP-3 探针**：解析 caelestia.mp4 色彩元数据 + 双端同帧对比定位色偏源；若属矩阵错配，在 engine 侧显式设 mpv 属性（工作期确认 SW 路径可设面）。
- **DP-2 探针**（bounded，决策工件 = 探针记录）：临时诊断日志打出 `Pipeline::new` 实收 format，独立窗与桌面两形态各跑一次。
- 验收锚点：同源视频双端（web vs VM）截图像素对比——白区通道差 ≤ 阈值（定 8/255）、无系统性暖偏；letterbox 恒纯黑。

### C. desktop_config 首写快照（→ SD-03）

- 现状：`LAST_SNAPSHOT` 仅 save 成功后写入（E-6），进程首写 save 无基线 → 整份覆盖（E-7 实录）。
- 修复：`load()`（`desktop_config.rs:439`）成功读盘后 `*LAST_SNAPSHOT.lock() = Some(parsed)`；「盘上无文件」分支不变（无外写可保，整份首写即迁移语义）。附带核对 `save_to`（测试隔离路径）不受影响。
- 回归锚点：`desktop_config` 作用域既有 19 测全绿 + 新增「load→外部改→save 保外部字段」测。

### D. examples 生成产物 feature 修复（→ SD-04 候选）

- **DP-4 探明**：`ui-gpui` 出自生成器模板（源头修复 + 金样重生成）还是仅历史产物（批量替换为 `ui-iced`）。检索面：`crates/auto-lang/src/ui_gen`（生成器源）+ `examples/rust-workspace/*/Cargo.toml`（产物）+ 617 系 plan 对 ui-gpui 退役的记录。
- 验收锚点：`cd examples/ui/030-video-player && auto run -r vm` 可启动出窗。

### 规范增量

| delta_id | 操作 | 目标 | before/after 规则 | rationale | acceptance |
|---|---|---|---|---|---|
| SD-01 | modify | `docs/specs/auto-lang/ui/overview.md`（媒体元素节） | before：`ontimeupdate`/`onloadedmetadata` 仅 Web 端承诺；after：双端承诺，VM 端经渲染路径事件泵分发、0.25s 节流口径不变 | 上行分发面缺失属平台契约缺口，必须成文 | AC-01, AC-02 |
| SD-02 | modify | `docs/specs/auto-lang/ui/overview.md`（媒体元素节） | before：上屏色彩未规定；after：sRGB 编码配对规则（纹理/目标格式组合矩阵）+ mpv 色彩协商口径 + 双端像素对照验收阈值 | 「可选 + 可降级」同款——色彩行为必须成文否则被误当默认能力 | AC-03 |
| SD-03 | modify | `docs/specs/auto-lang/ui/overview.md`（desktop_config 持久化面） | before：PLAN-044 锁+字段级合并（首写例外整份）；after：load 即快照，合并无首写例外（仅文件缺席首写保留整份） | E-7 实证首写例外即冲回通道 | AC-04 |
| SD-04 | add（视 DP-4） | `docs/specs/auto-lang/ui/overview.md`（examples 生成产物 feature 面）或生成器 spec | before/after 工作期按 DP-4 定 | 生成产物 feature 失效属工具链契约缺口 | AC-05 |

## 6. 测试设计

- **单测（本仓 cargo t，Category A 允许）**：
  - desktop_config：新增 `save_after_external_edit_preserves_fields`（load → 外部改盘 → 内存不变 save → 外部字段存活）；既有 19 测同绿。
  - uplink：契约事件→消息映射单测（构造 pending 事件 + 绑定闭包 → 断言合成消息序列与节流）。
- **探针（决策工件，非门禁）**：DP-2 目标格式记录、DP-3 色彩元数据与双端同帧对比记录，归档至 `docs/plans/evidence/712/`。
- **实机（人工清单 + 截图探针）**：桌面内 030 播放 caelestia.mp4——时长回灌、拖动 scrub、双端色彩对比（PIL 探针复用 auto-os 会话方法）；独立 `auto run -r vm` 同套。
- **日常档**：`cargo t` 日常档同基线（无新增红）。

## 7. 验收标准

- **AC-01**：VM 桌面内播放视频，OSD 时长与底部进度文本显示真实时长（非 00:00）。验证：实机截图 + 文本断言。
- **AC-02**：按住进度条拖动，播放位置连续跟随（≥3 个采样点位置单调变化），松开生效。验证：实机操作 + 截图序列。
- **AC-03**：同源视频 Web 端与 VM 端截图像素对比：内容白区 RGB 通道差 ≤ 8/255，无系统性暖偏；letterbox 恒 (0,0,0)。验证：PIL 探针脚本，证据归档 evidence/712。
- **AC-04**：config.at 手编字段在运行实例 save 后存活：单测 `save_after_external_edit_preserves_fields` 绿 + 实机复演（改壁纸 → 关实例 → 重开 → 值保持）。
- **AC-05**：`cd examples/ui/030-video-player && auto run -r vm` 启动出窗可播。验证：命令实跑。
- **AC-06**：日常档 `cargo t` 与 master 基线全等（无新增红）。验证：日常档跑批记录。

## 8. 执行步骤

- **T-01**（DP-2 探针）：确认 iced_wgpu 0.14 交付 `Pipeline::new` 的目标格式（独立窗 vs 桌面 layer 两形态）。文件：`ui/mpv/widget.rs`（临时诊断日志）。产出：evidence/712/dp2-target-format.md。→ 支撑 AC-03。
- **T-02**（DP-3 探针）：caelestia.mp4 色彩元数据解析 + Web/VM 同帧对比定位色偏源。产出：evidence/712/dp3-color-source.md。→ 支撑 AC-03。
- **T-03**：色彩修复实现（按 T-01/T-02 决策：编码配对 + 显式色彩参数）。文件：`ui/mpv/{widget,channel,present,engine}.rs`。验证：AC-03 探针脚本。依赖：T-01、T-02。→ AC-03。
- **T-04**：上行分发实现（按 DP-1 取向：wrapper 事件泵）。文件：`ui/iced/renderer.rs`（video 节点臂）、`ui/mpv/widget.rs`（如需导出事件类型）。验证：单测 + 实机 AC-01/02。→ AC-01, AC-02。
- **T-05**：desktop_config 快照修复。文件：`ui/desktop_config.rs`（load 尾 + 新单测）。验证：`cargo t desktop_config` 全绿。→ AC-04。
- **T-06**（DP-4）：examples `ui-gpui` 修复（模板源头 or 产物批量）。验证：AC-05 命令实跑。→ AC-05。
- **T-07**：实机验收走查（auto-os 桌面 + 独立 VM）：AC-01..05 逐条取证归档 evidence/712。依赖：T-03、T-04、(T-06)。
- **T-08**：日常档回归同基线（AC-06）。依赖：T-03..T-06。

## 9. 复审记录

- 2026-09-30 stage:new 起草（drafting）。`outcome: pass`——四项初始范围已按实证落档，DP-1..4 为工作期决策点（均含决策工件要求）。`next: work`（执行前按范式开 `.wt/lang-712` 组 worktree）。

## 10. 待澄清事项

- **扩展位（用户裁定，非待澄清）**：本计划为桌面 VM 验收缺陷的收纳载体——后续其他桌面 app 的 auto-lang 侧问题以 plan_revision 增补任务/AC 归入，不另起计划；每次增补记录来源与证据。
- DP-3 若证实色偏源为 mpv SW 路径不可协商项（如 colormatrix 无法显式设定），回退方案 = shader 侧矩阵修正或接受并成文差异阈值——届时提交用户裁定。
- SD-04 的 spec 落点（overview vs 生成器 spec）待 DP-4 定案后随复审固化。
