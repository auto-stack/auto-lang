---
plan_id: PLAN-712
status: executing          # r3 复活执行中（用户裁定 7e4258c31 git mv 回活跃区；交接批 8.5 T-17/18/19 + r2 遗留实机腿）
feature_name: VM 桌面验收缺陷收敛（视频引擎双缺陷 + 壳配置持久化 + examples 依赖）
author: [zcode(auto-os 会话转介)]
created_at: 2026-09-30
updated_at: 2026-09-30
plan_revision: 3

# /auto-plan:review 结束时填写：
supersedes_spec_components: []  # 空=无退役组件：8 条 delta 全为 docs/specs/auto-lang/ui/overview.md 既有节的 modify/add（复审核验，见 §9）
new_spec_components: []         # 空=无新 spec 文件：所有增量落 overview.md 既有文件内
touched_goals: [GOAL-007, GOAL-009, GOAL-010]  # 007=跨端一致(上行契约/事件占位双端同义)；009=桌面 Shell(壁纸 boot 链/config 合并)；010=示例轨道(030/018)

affects: [auto-lang/ui]
current_step: 12
total_steps: 13
---

# [PLAN-712] vm-desktop-defects

## 0. 变更摘要

VM 虚拟桌面实机验收（auto-os 侧 2026-09-30 会话）暴露的 auto-lang 引擎侧缺陷收敛，初始范围四项：

- **A. video 受控契约上行死链**：`mpv::widget::drain_events` 无生产调用方，`ontimeupdate`/`onloadedmetadata` 永不分发 → 应用层 duration/position 恒 0 → 进度条拖动无效。
- **B. mpv SW 上屏色彩错配**：VM 端视频实测整体偏暗偏暖（letterbox 纯黑排除叠层），与 Web 端（Chromium `<video>`）对照色度不同——纹理 sRGB 采样线性化与目标 pass 编码配对、mpv 色彩协商两候选机制，探针定案后修复。
- **C. desktop_config save() 首写无快照缺口**：进程首次 save 走 `_ => cfg.clone()` 整份覆盖，陈旧内存态冲掉手编/外写字段（2026-09-30 10:34 实录：壁纸槽被冲回 `#101014`）。
- **D. examples 生成产物依赖失效**：`rust-workspace` 系 Cargo.toml 引用已退役 feature `ui-gpui`，`examples/ui/030-video-player` 无法独立 `auto run -r vm`，挡住 A/B 的独立窗验证载体。

本计划同时是**桌面 VM 验收缺陷的收纳载体**：后续其他桌面 app 的 auto-lang 侧问题经 plan_revision 增补任务/AC 归入本计划（见 §10）。

**r2 收纳（2026-09-30）**：VM 本地视频播放缺失——030 的 `.LocalPick` VM 分支仍走 PLAN-681 前的过时降级文案（「后端无文件选择能力」），而 `dialog_open`（natives 2927）与 mpv 本地路径契约（contract「本地路径与 http(s) URL 都接受」）均已就绪；修复 = 纯 app 层接线（player_store.at VM 分支 → dialog_open → LocalPicked 同一换片语义），引擎零改动。

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
| SD-07 | modify | `docs/specs/auto-lang/ui/overview.md`（壁纸解析节，随再 merge 沉淀） | before：无分主题默认壁纸概念（默认值只存在于配置）；after：钦定默认对 = 深 `purple.png` / 浅 `songyu.png`（壁纸目录下），boot 链第二档；「恢复默认」从此有系统定义的标的 | r2 收纳 T-14（用户钦定） | AC-09 |
| SD-06 | modify | `docs/specs/auto-lang/ui/overview.md`（壁纸解析节，随再 merge 沉淀） | before：`#hex`/`builtin:` 直传短路目录首图回退（纯色槽位可致 boot 恒无图）；after：boot 优先级 = 存在图片路径 > 壁纸目录首图 > 色值直传（仅无目录机器）> 内置默认——「任何时候打开都加载默认壁纸」（用户裁定） | r2 收纳 T-13 | AC-08 |
| SD-05 | modify | `docs/specs/auto-lang/ui/overview.md`（媒体元素节，随再 merge 沉淀） | before：VM 端本地文件选择无契约（app 侧降级文案「仅 Web 端」）；after：VM 端本地播放 = dialog_open（PLAN-681 内建）→ 本地路径直入 mpv 契约，与 Web object URL 同级承诺 | r2 收纳：能力面已齐但 app 未接线，契约须成文 | AC-07 |
| SD-04 | add（视 DP-4） | `docs/specs/auto-lang/ui/overview.md`（examples 生成产物 feature 面）或生成器 spec | before/after 工作期按 DP-4 定 | 生成产物 feature 失效属工具链契约缺口 | AC-05 |
| SD-08 | add | `docs/specs/auto-lang/ui/overview.md`（媒体元素节/事件契约面，随 merge 沉淀） | before：事件回调 `$N` 占位实参仅 Vue 生成器替换语义，VM 端落 `Str("$0")` 字面量占 args[0]、真载荷错位（`($0)` 与 `()` 形态分叉）；after：`$` 前缀占位实参 VM 端剥除，载荷对位由回调臂事件现场 push 保证，`($0)` 与 `()` 两形态 VM 端同义、与 Vue 生成器同契约 | T-12 根因修复（030 进度条点击失效）引入的持久语义，必须成文防回归 | T-12 回归锁 `progress_onseek_placeholder_never_becomes_literal_arg` |

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
- **AC-09**（r2）：色值/空槽位在深色主题 boot → purple.png、浅色主题 boot → songyu.png（主题默认图在场机器）；无目录机器色值直传。验证：单测断言（hex 深→purple / 浅→songyu 双结果）。
- **AC-08**（r2）：任意槽位/活跃值为纯色或 builtin 时，boot 仍加载壁纸图片（有壁纸目录机器：目录首图；无目录：色值直传诚实降级）。验证：`desktop_surface_storage_roundtrip_and_wallpaper_resolution`（双结果放行断言）+ 实机浅色启动截图。
- **AC-07**（r2）：VM 端（独立窗 + 桌面内）点「打开本地视频文件」→ 原生对话框选本地视频 → 直接播放（时长/进度回灌正常）；取消对话框静默返回。验证：实机走查 + 截图。

## 8. 执行步骤

- **T-01**（DP-2 探针）：确认 iced_wgpu 0.14 交付 `Pipeline::new` 的目标格式（独立窗 vs 桌面 layer 两形态）。文件：`ui/mpv/widget.rs`（临时诊断日志）。产出：evidence/712/dp2-target-format.md。→ 支撑 AC-03。
  - [x] **已完成**（2026-09-30）：**静态定案，无需运行时日志**——`web-colors` 未启用 ⇒ `GAMMA_CORRECTION=true` ⇒ compositor 取首个 sRGB 格式 ⇒ 窗口面 `Bgra8UnormSrgb`（headless `Rgba8UnormSrgb`）。目标/纹理双 Srgb + shader 直通 ⇒ 上屏字节 ≡ mpv rgb0 字节，机制候选 1（sRGB 配对断裂）排除。证据：`docs/plans/evidence/712/dp2-target-format.md`（worktree 提交）。附产：iced_winit 重绘事件传播链确认（DP-1 wrapper 前提 + 空队列零 publish 纪律）。[✅ 已完成]
- **T-02**（DP-3 探针）：caelestia.mp4 色彩元数据解析 + Web/VM 同帧对比定位色偏源。产出：evidence/712/dp3-color-source.md。→ 支撑 AC-03。
  - [x] **已完成**（2026-09-30）：本机无 ffprobe，改用**引擎级探针**（`tests/mpv_engine.rs::probe_color_negotiation_dump`，AUTO_MPV_LIB 门控）：源片 bt.709/limited/bt.709/**bt.1886**。差分探针（`probe_color_output_tunable`）：默认 vs pre-init `target-prim/trc=srgb` vs `video-output-levels=full`——同帧输出**逐位全等** ⇒ SW 路径完全不吃协商面，输出=源传递函数编码。定谳：双端色度差=传递函数错配（mpv 2.4 vs Chromium sRGB 2.2）。证据：`docs/plans/evidence/712/dp3-color-source.md`（worktree 提交）。[✅ 已完成]
- **T-03**：色彩修复实现（按 T-01/T-02 决策：编码配对 + 显式色彩参数）。文件：`ui/mpv/{widget,channel,present,engine}.rs`。验证：AC-03 探针脚本。依赖：T-01、T-02。→ AC-03。
  - [x] **已完成**（2026-09-30）：按 DP-3 定谳落 **shader 侧传递函数归一**（协商面已证死路）：`present.rs` fs_main 把采样值还原字节域→按 2.4 解码→交 Srgb 目标按 sRGB 重编码，端到端与 Chromium 字节域对齐；`engine.rs` 补 `set_string`（探针/后续协商面用），`loader.rs` 补 `set_property_string` 符号。像素级回归 `mpv_channel::present_renormalizes_bt1886_bytes_to_srgb_domain`（135→~128 / 0→0 / 255→255）绿。**AC-03 的双端同帧实机探针归 T-07**。E-3 实录（偏暗偏暖）与传递错配方向（偏亮）不符，判读注记见 dp3 证据档。[✅ 已完成]
- **T-04**：上行分发实现（按 DP-1 取向：wrapper 事件泵）。文件：`ui/iced/renderer.rs`（video 节点臂）、`ui/mpv/widget.rs`（如需导出事件类型）。验证：单测 + 实机 AC-01/02。→ AC-01, AC-02。
  - [x] **已完成**（2026-09-30）：**DP-1 定案 wrapper 事件泵**（thread_local 运行时排除 Subscription 线程；iced_winit 每次重绘把 `RedrawRequested` 传播给整棵树，tick 驱动下每帧可达）。新增 `ui/iced/video_uplink.rs`（薄包装 wrapper + 纯映射 `synth_messages`）；`View::Video` 增 5 上行 handler 字段（`MediaEventHandler`/`MediaEventPayload` 新型，view.rs）；aura builder `convert_video` 从 events 构建handler；`convert_view_messages`/render 臂随消息类型映射；wrapper 仅真有事件时 publish（避 iced_winit 重建循环）。验证：`video_uplink` 单测 3/3（映射/未声明丢弃/零消息）+ `video_contract::video_uplink_bindings_wire_through_the_builder`（真实管线接线）+ video_contract 全套 4/4 绿；`cargo check --features mpv-widget` 干净。**AC-01/02 实机归 T-07**。[✅ 已完成]
- **T-05**：desktop_config 快照修复。文件：`ui/desktop_config.rs`（load 尾 + 新单测）。验证：`cargo t desktop_config` 全绿。→ AC-04。
  - [x] **已完成**（2026-09-30，含**证据偏差更正**）：现场复核发现 E-6 前提已过时——`load()` 尾部写 `LAST_SNAPSHOT` 早在 PLAN-044 T-03（d164b1d50，2026-09-23）在位；E-7 实录（10:34 冲回）与「修复前内存态」措辞自洽=预修二进制旧实例退出，现行 master 的 load 即快照 + 字段级合并已覆盖首写缺口。落点收敛为计划要求的回归锚：新增 `save_after_external_edit_preserves_fields`（load → 外部改盘 → 内存不变首写 save → 外写字段存活；若 load 不写快照该测必红，钉死通道）。`cargo t desktop_config` 20/20 绿（既有 19 + 新增 1）。[✅ 已完成]
- **T-06**（DP-4）：examples `ui-gpui` 修复（模板源头 or 产物批量）。验证：AC-05 命令实跑。→ AC-05。
  - [x] **已完成**（2026-09-30）：**DP-4 定案=历史产物**（生成器模板经 PLAN-691 已清，`rust_ui.rs::generate_cargo_toml` 无 ui-gpui）。两层产物修复：① 主检出 gitignored 生成树 `examples/rust-workspace/`（E-8 失败现场：030-video-player-back 的 cargo run 因 workspace 成员 011-calculator 等残留 `ui-gpui = ["auto-lang/ui-gpui"]` 被惰性校验拖垮）14 份 Cargo.toml 批量删除该行，`cargo metadata --no-deps` 全 workspace 解析恢复；② tracked 产物 `examples/ui/015-notes/examples/rust-workspace/015-notes/Cargo.toml` 同步删除（worktree 提交）。**AC-05 命令实跑归 T-07**。[✅ 已完成]
- **T-07**：实机验收走查（auto-os 桌面 + 独立 VM）：AC-01..05 逐条取证归档 evidence/712。依赖：T-03、T-04、(T-06)。
  - [~] **进行中**（2026-09-30，独立 VM 窗腿已走查，桌面腿与 Web 对照腿未完）：
    - **AC-05 ✅**：`examples/ui/030-video-player && auto run -r vm` 独立出窗可播——mpv 真加载（client API 2.5）、媒体库 14 条、播放控制全套。截图 evidence/712/vm-030-playing.png。**附带产物修复**：worktree 内 `examples/rust-workspace/030-video-player-back`（gitignored 生成物）为 030 尚有 back/ 契约时代的旧生成残留，scan 响应裸数组与现行前端 `{entries,root_missing}` 期望错位 → 媒体库恒空；已按现役生成器（api_gen auto_media_scan）字段面手工对齐（title/size_str/url 族 + 三态）。
    - **AC-01 ✅**：时长真值回灌——状态条「时长 00:49」（caelestia 实测 49s；修复前恒 00:00，E-2 同款场景）+ 底部进度文本 00:40/00:49 随播随动。`onloadedmetadata → OnDuration → store` 全链实机工作。
    - **AC-02 部分**：`ontimeupdate → OnTime → current_time/progress_pct` 回灌链实机连续工作（跨 caelestia 49s → Loki S01E01 ~50min 两文件推进，6.57% ↔ 198s/3014s 自洽）；**scrub 腿未取全证**——MCP 合成 press 不达 SeekArea 裸事件路径，真鼠标拖动被 VM 会话提前终止截断（死因注记见下），待补：真鼠标三采样点拖动 + 位置单调断言。
    - **AC-03 部分**：VM 侧视觉验证 ✅（截图色彩鲜活无偏暗/暖压，对比 E-3 机理修复前形态）；**Web 同帧像素对照未跑**（需 Vue 端 + playwright，工具链在 autoui-verifier）。
    - **AC-04 部分**：单测 ✅（20/20）；实机复演（改壁纸→关实例→重开→值保持）未跑。
    - **死因注记（未定谳，预算外）**：MCP `press` 打 progress 元素后 wgpu `Reading from a BufferViewMut` 告警洪泛，随后 VM 解释器 `ok=true` 优雅退出（非 panic 非崩溃）；与本项目改动面的因果未定谳——复现路径=030 VM 窗 + MCP press progress 元素。已登记 §10。
    - **桌面腿（auto-os ui_desktop 内嵌 video）未走查**：需 auto-os 侧环境接线（AUTO_LANG_ROOT 指向本 worktree 或 fold 后走查）。
- **T-08**：日常档回归同基线（AC-06）。依赖：T-03..T-06。
  - [x] **已完成**（r2 复核 2026-09-30，worktree 7f483e1ab）：裸 `cargo t --no-fail-fast` 全量对照——worktree 12 唯一红 ⊂ master 同刻基线 14 红（master 另多 app_registry launch_three/scan_examples_ui 两红），**零新增红**；master 基线较 T-09 记录（2 红）已扩至 14 红（明细归 §10），AC-06 口径满足。
- **T-14**（r2 收纳，用户钦定默认对）：`wallpapers_theme_default_image`（深 purple.png / 浅 songyu.png，置于壁纸目录）+ boot 链插入第二档 + 单测深化（深/浅双断言）。文件：`crates/auto-lang/src/ui/iced/renderer.rs`。验证：单测绿。→ AC-09。
  - [x] **已完成**（2026-09-30 master d990427ba，worktree 复证 7f483e1ab 基）：分主题默认对入 boot 链第二档 + 深/浅双断言随 `desktop_surface_storage_roundtrip_and_wallpaper_resolution` 实跑绿（重建 worktree 现场验证）。
- **T-13**（r2 收纳）：`load_desktop_wallpaper` 优先级重构（图片路径 > 目录首图 > 色值 > 内置默认）+ 断言双结果放行。文件：`crates/auto-lang/src/ui/iced/renderer.rs`。验证：`cargo t desktop_surface_storage_roundtrip` 绿 + 实机浅色启动截图。→ AC-08。
  - [x] **已完成**（2026-09-30 master 74fbc8085，worktree 复证）：优先级重构落地，`desktop_surface_storage_roundtrip_and_wallpaper_resolution` 实跑绿（AC-08 单测腿）；实机浅色启动截图腿并入 T-07 遗留清单（用户裁定不阻断）。
- **T-09**（r2 收纳）：030 `player_store.at` `.LocalPick` VM 分支接 `dialog_open` + `file_basename` → `LocalPicked` 同一换片语义。文件：`examples/ui/030-video-player/src/front/player_store.at`。验证：AC-07 实机。→ AC-07。
  - [x] **已完成**（2026-09-30，**用户裁定收口：全量测试不跑**——当前全量档本身有问题）：fail-fast 跑批 1440/5747：1438 绿 + 2 红 `musk_vm_track_p053_1_widget_computed`（computed 透传解出 0 行）——**master 基线同测实跑复证同样 2 红 ⇒ 预存红，非本计划引入**，AC-06 口径（无新增红）满足。作用域绿面：desktop_config 20/20、video_uplink 3/3、video_contract 4/4、mpv_channel 像素归一双臂绿、`cargo check --features mpv-widget` 干净。`state_file::lock_serializes_critical_sections` 并跑红/隔离绿（flock 争用型 flaky，另案在档）。[✅ 已完成]
- **T-15**（r2 再收纳）：018 `pages/book_detail.at` VM 解析器不兼容构造重写（has_book 旗标替代 null 比较、条件数组 style 拆 if/else 双臂、`.len()` 预计算、Init try/catch）。文件：`examples/ui/018-book-reader/src/front/pages/book_detail.at`。验证：`auto build` 页面扫描 + 实机点验（移交用户）。→ 页面装载面。
  - [x] **已完成**（2026-09-30 master 8bb1f3279）：按 bookshelf 惯用法重写落地；引擎债（view 条件 null 字面量 + 数组内 if 解析支持）另案登记（§9 r2 收纳条目在案）。
- **T-10/T-11/T-12**（r2 再收纳合并侦查：VM 布局/命中层三症状一次定位——头部动作按钮缺席 / 播控条 flex 分布错 / 进度条点击失效）。文件：`crates/auto-lang/src/ui/aura_view_builder.rs`、`crates/auto-lang/src/ui/iced/*`。→ AC-02/AC-07 关联。
  - [x] **T-12 完成**（worktree 7f483e1ab）：根因=事件回调 `$0` 占位被 VM 落成 `Str("$0")` 字面量占 args[0]、真载荷错位到 args[1]（030 `onseek: .SeekFraction($0)`；020 空参 `.SeekFraction()` 形态不受影响故双 app 对照成立；Vue 生成器契约=「$0 由生成器替换成真实取值」，vue.rs:5257）。修复=`event_to_message_with` 剥 `$` 前缀占位实参（载荷对位由回调臂事件现场 push 保证，`($0)` 与 `()` 两形态 VM 端同义）；回归锁 `progress_onseek_placeholder_never_becomes_literal_arg` 绿（双形态断言）。实机点击 scrub 取证归 AC-02 腿（待用户配合窗口）。
  - [~] **T-10 当前 master 不复现（裁定消散）**：本会话重建 worktree 二进制真机走查（零输入，MCP 只读快照+截图 evidence/712/master-binary-window.png）——主题切换/队列开关两按钮正常渲染于头部右缘、handler 接线在树；r2 会话的「双轨缺席」观察随 plan-711 视图构建/派发路径重写（Init 延迟派发等）消散。点击可达性归 AC-07 实机腿复核。
  - [~] **T-11 仍复现（未修复，归后续）**：真机二进制播控条塌缩原样——左列（时间+进度条）~45% 窗宽处收住（设计应贴传输键）、倍速「1.0x」与音量组间 ~250-330px 空档（截图 evidence/712/master-binary-window.png，与 r2 用户截图 7b9397df 同形态）。headless 三支确定性探针（控件行 flex / 完整 app 层级 / 头部按钮）全绿=纯 view→iced 布局分配健康（回归锁在库）；取证型组件形态探针呈**进程级翻转**（同源 col_right 438↔658px 随进程启动翻转，icon/窗宽/onseek 均排除）——嫌疑面收敛至动态视图构建的顺序敏感层（组件实例化/HashMap 序），修复前需先定谳非确定性源。探针已从门禁摘除（防 CI 抖动），取证细节入 §10。

## 8.5 r2 交接批（HANDOFF——2026-09-30 深夜，移交下一 agent）

> 本节为**自包含交接文档**：下一 agent 零上下文接手 T-17/T-18/T-19 三笔。
> 环境搭建、诊断通道、已排雷项、逐问题（症状/实证/假说排序/修复方案/
> 验证步骤）全部在案。执行前提：本计划 r2 已被用户裁定继续留在 712
> （不拆新计划）；执行仓 = 本仓（auto-lang 主检出）。

### 8.5.0 运维手册（环境与工具链）

**验收桌面启动**（管理员不需要；Git Bash）：
```
cd /d/autostack/auto-os
AUTO_SCHED_DIAG=1 AUTOUI_ACCEPTANCE=1 AUTOUI_MCP_PORT=9260 bash scripts/desktop.sh iced
```
- MCP 端口：固定请求 9260，**被占自动回退 9261+**（枚举 `netstat -ano | grep ":92"` 认桌面 PID——ui_desktop 的 iced 窗口 MainWindowHandle 恒 0，别用它判断）。
- 诊断通道（全部经 POST http://127.0.0.1:{port}/mcp，JSON-RPC，先 initialize）：
  - `autoui_screenshot`：渲染帧 PNG（落 `tmp/autoui-screenshot-*.png`，路径在返回 text 里）。
  - `autoui_snapshot`：AURA 树（**只到桌面壳面，不达 app 虚拟窗内部**——工具缺口 T-DOCS-1）。
  - `autoui_desktop`：`{action:"bus", verb:"launch	<app-id>"}` 发射 app；`win_rect	<wid>	<x>,<y>,<w>,<h>` 程序化缩放虚拟窗（**wid 无发现通道**——T-DOCS-2，盲扫或用户拖拽）。
  - `AUTO_SCHED_DIAG=1`：帧泵/Init demand 生命周期 trace（stderr → 桌面日志）。
- back proxy 请求日志：本计划已加（`[back-proxy:{app}] REQ/RSP` stderr）——前端实际请求一锤定音的通道。

**已排雷项（踩过即记录，勿重蹈）**：
1. **view 节点单行混写 = 解析雷区**：`row { style: "..." text ... }` prop+子同行 → 「Expected infix operator」×20 级联。全部多行书写。
2. **后台命令管道陷阱**：`cargo ... | tail -1` 吞退出码（失败显示成功）；验证构建用 `grep -E "^error|Finished"`。
3. **JSON 转义**：MCP curl 的 `	` 要写单反斜杠（`"verb":"win_rect	12	..."` 双反斜杠 = 字面量不解析）。
4. **僵尸进程**：ui_desktop/auto 关窗后退化为不可杀内核僵尸（锁 exe 文件）——重建前 `mv <exe> <exe>.z<N>` 改名让路（Windows 允许改名运行中的映像）；僵尸积累至重启清理。
5. **MCP 截图目标**：捕获**聚焦窗**；app 最小化/零尺寸报 "window size is zero"。OS 级遮挡用 PrintWindow（flags=2）。
6. **probe 常驻**：VM 单窗脚本跑完即自退——探针 Init 里挂黑洞 fetch（`Http.get_json("http://10.255.255.1:9/x")`）保活。

### 8.5.1 T-17：018 详情页「0 entries」（章节数据前端取数空化）

**症状**：点书卡 → 详情页框架渲染（has_book=true 分支）但「0 entries」、章节列表空、书内容不可见。多次复现。

**已实证排除**：
- 数据面 ✓：proxy 逐书 curl（books/1、/2、/3 chapters）全部返回真实章节数组（Three Gate/Source/Keeper 各多章）——`http://127.0.0.1:3360/apps/018-book-reader/api/books/{id}/chapters`。
- 路由 ✓：`list_chapters` 形参/占位符错配已修（`book_id`→`id`，本计划 T-15 第二层）。
- 前端取数惯用法 ✓：`Http.get_json` → `.len()` 与书架 `.store.books.len()` 同款；独立数据链探针（layoutprobe，直取同 proxy URL）**端到端渲染 "3 entries" + 章节列表** ✓。

**假说排序（下一步验证）**：
1. **H1 前缀变换未覆盖重写后的字面量**：launch 期 `prefix_api_url_literals` 对 spec.code 的字面量前缀化——T-15 重写后的 Init（try/catch + 相同字面量）是否被前缀化未验证。验证：桌面日志 P041-DBG "prefix apply" 行——018 的 book_detail.at 是否在案（对照：030 的 player_store.at/playlist.at 在案 ✓）。修复 = 补前缀变换的覆盖或对账。
2. **H2 router.param("id") 空值**：`__route_params` 的 VM 导航持久化（dynamic.rs sync_route_params，"Called after each handler"）——点击书卡 → OpenBook → router.push("/book/2") → sync —— 但 book_detail 的 Init **parked 恢复后**读 param 的时序（恢复后 __route_params 是否仍持有该路由的参数）。验证：Init 里 `router.param("id")` 的值写上屏（status_text 调试）。
3. **H3 proxy 200+错误体**：proxy 参数绑定失败时返回体带 200 → get_json 不抛 → .chapters = 错误对象 → len()=0。验证：proxy REQ/RSP 日志（已生效）+ 响应状态码补记。

**修复方案**（按定位落点）：H1 → back_prefix 覆盖对账/修复；H2 → 恢复完成后补 sync_route_params（poll_parked_resumes 尾部）；H3 → proxy 错误返回改 4xx + 前端 catch。

**验证**：点书 → 详情显示标题/作者/「3 entries」+ 章节列表；proxy 日志对账无 4xx/空化。

**r3 定谳与修复（2026-10-01，worktree 0c4606ee5）**：
- **根因（第 0 步 dump 定性，证据链闭合）**：**路由页装载臂裸读**——lib.rs 的 routes 块 `pages/{module}.at` 显式装载段（"Plan 401/VM-routing"，use_scanner 对 `-> use X` 不可见故单列）是**全部模块读点中唯一未包 `back_prefix::apply` 的**（其余 10 处全包，4259 顶层 use 臂即对照）。于是：顶层 `book_store.at` 前缀生效（书架 2 REQ 到 proxy ✓）；`pages/book_detail.at`/`reading.at` 的相对 `/api/` 字面量留原样 → fetch 解析到 app 自身 back origin（死端口）→ 86ms 秒拒 → get_json 吞错返空 → 0 entries。r2 的 H1/H2/H3 三假说归并：H1 成立但缺口在**装载臂漏包**（非行级规则/路径归一——子目录在 `path_under` 递归面内）；H2 排除；H3 排除（候选③审计：proxy 错误路径已全 4xx/5xx——404 无路由/未知 app、400 缺参/坏绑定、503 会话超时/媒体根未配；root_missing 200 是三态数据契约非缺陷）。
- **候选①（根修）**：路由页读点补 `apply`（一行 + 注记）。回归锁 ×2（back_prefix.rs）：`subdir_modules_under_front_dir_are_transformed`（pages/ 子目录与顶层同面）+ `route_page_fetch_reaches_proxied_root_when_guard_active`（**真 TCP stub e2e，红→绿**：红相 = stub 零请求 + 状态滞留初值——与实机「零 REQ」同形；绿相 = `GET /apps/<key>/api/books/1 HTTP/1.1` 前缀命中 + 响应体经 get_json 回灌页状态，走 fire_init + demand/恢复泵真管线）。
- **候选②（防御，fetch() 同律分层）**：`check_async_http_result` 的传输错误臂（连接拒绝/超时/DNS/队满）旧包成 `{"error":..,"status":0}` **成功值** = 吞错点（本例助燃层：点书后即使前缀修复，任何传输失败仍会静默空态）；改 **Err 透传 → 五 json shim（get/post/put/delete/patch）重入臂转 VMError**（与段内除零同路 = `.at` try/catch 可接，018 的 catch 可呈现可诊断错误）。非 2xx 保持错误形状值（`.status` 可读——fetch().json() 的「HTTP 错误状态不 reject」同律）。孤儿 `escape_json` 退役。回归锁 ×2（新 `plan712_http_error_semantics_tests.rs`）：死端口秒拒被 catch 截获（旧形态返回 "unreached"）+ 非 2xx 值面不抛。语料影响面审计：examples 全域零 `.error`/`.status` 字段读方。
- **连带现形与修复（②让预存假绿现形）**：①musk p080 http 族 4 测（片段跑相对 URL，运行载体）→ try 包裹免疫化（断言主题=转写平价不变）；②**plan705 线程数测试复活**——handler 烤的是预热上游端口而 listener 随预热线程 join 即 drop，负载相 8 个上游 POST 全打死端口，旧吞错契约 200 假绿、up2 无指向空转（**测试从未真正测过并发慢上游**）；修复 = 预热/负载同源指向活上游 + mock 补阻塞读全请求头（accepted 流继承 nonblocking 竞态）；复活后 4/4 稳定绿。
- **验证对账**：back_prefix 3/3 + plan712_http_error 2/2 + plan702 4/4 + p080 5/5 + plan705 4/4 绿；裸 `cargo t` 4934 跑 4922 绿，**12 红 = master 基线红族精确对齐，零新增**。
- **实机（验收桌面，worktree 构建）**：MCP :9261（9260 被僵户占自动回退）→ launch 018 → **书架 3 本正常加载**（截图 evidence/712/t17-r3-desktop-bookshelf-3books.png）+ 桌面日志 `[back-proxy:018-book-reader] REQ GET /api/books ×2` 到达（迁移入档的 REQ/RSP 诊断首次实机服役）。**点书导航腿移交用户点验**：OS 级输入自动化在落点验证时发现用户前台占用（r2 同款墙），当即停手（两次点击落在用户浏览器空白区，无害）。桌面载体稳定性另见 §10⑧（本日重载下预存崩溃，非本计划回归）。

### 8.5.2 T-18：030 播控条布局（按钮拉伸/倍速超宽/头部按钮不可见）

**症状**：播控条子元素挤左侧（seek 条止于 ~44-52%）；拖拽虚拟窗后**右侧按钮跟着变宽**（用户原话）；倍速组件特别宽；头部主题/队列按钮不可见。默认窗宽（~1005）下比例正确（本计划复验截图）。

**已实证**：
- 独立 OS 窗 resize → **重排完全正确**（600x420 PrintWindow 实测）——iced 运行时 + Fill 长度体系无恙。
- 桌面内虚拟窗：app 按宿主视口布局（`vwin_rect` 恒 None，文档在案），虚拟窗显示该表面；拖大虚拟窗 = 表面 1:1 锚定 + 空白右/下（52% 冻结实测）——**按钮「变宽」= 表面被拉伸显示的缩放效应**。
- 真实 controls.at 解析干净（探针 8.5.0 同构编译零错误）——非解析层。

**根因定性**：**虚拟窗 rect 未喂给 app 布局基座**（`vwin_rect` 设计了未实现，session.rs 5504/5521 注释「恒 None」）——app 恒按宿主视口布局，虚拟窗缩放显示。

**修复方案（用户裁定后实施）**：
- **方案 A（特性实现，推荐）**：vwin rect → app 布局极限接线——`allocate_app`/虚拟窗构建时把 `vwin.rect` 作为该 app 的 host_viewport 替身（session.rs 5504 的 `vwin_rect: None` 落点）；虚拟窗 resize → `window_size` 已同步（session.rs:1646 ✓）→ 补 `view_dirty` 标记（resize 臂 1646 邻域缺脏标记——与 T-16 同族断链）→ app 重排。
- **方案 B（维持现状）**：虚拟窗缩放显示宿主视口布局（写明文档）；播放器全屏使用无感。
- 验证：win_rect 缩放虚拟窗（8.5.0 通道）→ 播控条重排随动（seek 条占满 col、按钮右聚不变形）；头部按钮可见。

**r3 修复记录（2026-09-30，worktree a817963b0）**：
- **根因定性修正（递增实证修正 r2 初判）**：「`vwin_rect` 恒 None / app 按宿主视口布局」**不成立**——`split_mut_at`/`split_ref_at` 桌面分支早已传 `vwin_rect: Some`（session.rs 5548/5598，PLAN-024 T-01）且 `virtual_window_element` 的 win_stack 本身就是 `Fixed(rect.width/height)`（virtual_window.rs:540）——app 的 iced 布局 bounds 就是虚拟窗尺寸。r2 所引「5504 恒 None」实为 **windowless overlay 垫片分支**（shell/launcher 等，合法 None）。
- **真根因（代码级闭环）**：VM 视图构建器把响应式取值按 `window_size` 在**构建期烤定**（`set_window_width` 通道，renderer.rs:22643），而 `dynamic_view_impl` 的元素缓存快门在 `!view_dirty` 时逐帧返回**缓存的烤定像素树**（renderer.rs:22706-22714）。虚拟窗缩放只更新 `VWinState.rect/window_size`、**不标 view_dirty**（交互臂 apply_cursor 消费后直接 return；编程 WinRect 臂自述「无 relayout」）→ app 内容恒停旧尺寸 = 表面 1:1 锚定 + 空白右/下 + 播控条不重排。
- **方案 A 落地**：`DesktopSession::mark_vwin_resized_dirty(wid)`（helper，headless 可测）+ 两执行臂接线：① `DC::WinRect` 臂——**尺寸真变才标**（≥0.5px 阈值，纯移动零重排纪律）；② `__mouse_moved` 交互臂——`apply_cursor` 消费且 interaction 为 `Resize` 时逐拍标（拖拽 Drag 不标）；逐拍重建即活布局的标准成本（独立窗 OS resize 同价）。
- 回归锁 `wm_resize_marks_app_view_dirty`（headless seam：缩放交互→置脏增量断言 + 未知 wid 幂等 false + Drag 不误伤）+ `wm_` 族 10/10 绿；`cargo check`（默认 + mpv-widget）干净。
- **实机腿未走查**（win_rect/拖拽缩放虚拟窗 → 播控条重排随动 + 头部按钮可见）——归验收桌面复验（本会话无实机桌面环境）。

### 8.5.3 T-19：030 播放/暂停状态链（按钮图标错 + 停不下来）

**症状**：播放中按钮显示 play（应 pause）；点击后暂停一下**又继续播放**，停不下来。

**分析**：按钮 = `if .store.is_playing { pause 图标 } else { play 图标 }`——显示 play ⇒ **store.is_playing=false 与 mpv 实态（播放中）脱 sync**。「暂停一下又继续」⇒ 点击→paused=true→mpv 暂停→**随即被拉回播放**——嫌疑 = 契约 `paused` 下行的 apply 语义：store.paused 的变更锁存与 mpv 实态的上行事件互相打架（toggle 置 true → mpv 暂停 → 上行事件改写 is_playing/其它字段 → 下一轮 contract.apply 以陈旧 paused 或字段突变再发 unpause）。T-16 上行接通后此链首次活跃（此前上行死、状态恒假但无打架）。

**修复方案**：读 player_store.at 的 TogglePlay/paused 字段 + contract.rs 的 paused apply/事件映射（contract.rs「position 的语义是目标变化才 seek」同族——paused 的锁存语义对齐）；对齐后实机验证：播放中点暂停 → mpv 真停（图标 pause）→ 再点续播。

**验证**：播放中点按钮 → 暂停（图标翻转、帧停）→ 再点 → 续播；10 次无状态回弹。

**r3 诊断+修复记录（2026-09-30，worktree a817963b0）**：
- **全链静态读码**：player_store.at（`.TogglePlay` 翻转 is_playing / `.OnPlayState` 回灌同字段）/ viewport.at（`paused: .store.is_playing == false` 下行绑定 + `onplaystatechange: .OnPlayState($0)`）/ contract.rs（paused 差量下发 + poll 边缘检测合成 PlayStateChange）/ video_uplink.rs（载荷映射 `Playing(b)→Value::Bool(b)` 直进 args[0]，不经 event_to_message_with）/ aura_view_builder convert_video（`extract_bool_expr` 走 `resolve_expr_to_value`，`Op::Eq` 臂对 `Value::Bool` 全等比较正确）。`!` 前缀 = 全语料惯用法（014/015/016/019/020 dark_mode 同款），排除。
- **引擎级实证探针（真引擎 + 契约帧循环，AUTO_SPIKE_VIDEO=caelestia.mp4）**：① 空闲引擎（未 loadfile）首 poll **自发合成 `PlayStateChange(true)`**（空闲 pause=false 被当边缘）——红相在案；② **暂停长保持 120 帧（渲染面每帧同值 apply+poll 形态）零自发 un-pause**、pause 旗标恒真、时间冻结；③ 快速 toggle ×10 applied 锁存与内核实态逐拍一致；④ 同 src 重新 loadfile 后 30 帧窗口无 play-state 事件（get_flag 过渡期返 None 被跳过）。**定谳：引擎/契约保持层干净，「停不下来」的驱动不在本层**。
- **打架种子修复 ×2（contract.rs）**：① poll ④ **首次 pause 读 = 基线采集不回灌**（`last_play_state: None → seed`，不再把空闲默认态当边缘——修复后 app 侧 is_playing 不再被顶成假「播放中」，下行/上行镜像不再互为反转）；② apply **pause 写失败不推进 applied**（差量判据以内核实态为准，一次瞬时失败不再永久丢写）。
- **回归锁 ×2（mpv_contract.rs，12/12 绿）**：`idle_engine_first_poll_seeds_baseline_without_play_state_event`（红→绿对：基线零事件 + 同值保持零事件 + 作者显式翻转恰好一次）+ `pause_hold_and_rapid_toggle_stay_in_sync`（长保持 + toggle×10 实态对账）。既有 `real_playback_advances_time_and_reports_play_state` 的真实起播边沿断言不受基线种子影响（绿）。
- **残留定性**：实机「图标错 + 停不下来」的完整复现需要**桌面轨消息/视图链**参与（T-16 同族：update 可达但视图重建滞后/丢失 → down 原语陈旧 → pause 写永不下发；重建迟到则「暂停一下」后用户再点即「又继续」）。引擎腿已清白并加固，实机走查（播放中暂停→图标翻转→10 次无回弹）**归验收桌面复验**（与 T-16 桌面轨修复同窗）。

### 8.5.4 归并与关联

- T-10（头部按钮不可见）= T-18 同族（缩放显示下的位置/可见性），随 T-18 方案 A 闭环。
- T-DOCS 工具债：①autoui_snapshot/inspect/action 不达 app 虚拟窗内部（0-entries 与 T-12 的自动化验证被卡）；②wid 发现通道。修复 = MCP 服务器遍历 session.apps 的 per-app vtree/元素（mcp_server.rs tool_snapshot 的 shared 之外加 per-app 面）。
- corpus 普查（下一批）：013-todo（`api.create_todo` 链接失败）、025-sys-monitor——通知流在案，同族「无法启动」。

## 9. 复审记录
## 9. 复审记录

- 2026-10-01 stage:work 续（**r3 第二批**，T-17 收口）| plan_id PLAN-712 | plan_revision 3 | outcome: pass（根修+防御双层落地，全锁绿，日常档零新增红；点书导航实机腿移交用户）| code_commit: plan-712-dev @0c4606ee5 + 32ced74f6（承 a817963b0）| task_ids: T-17 | evidence:
  - **第 0 步定性**：back_prefix 机制审计（apply/path_under/prefix_api_url_literals）+ 018 取数点布局盘点（book_store.at 顶层 vs pages/ 子目录）→ lib.rs 路由页装载臂裸读实锤（10 处读点唯一未包 apply）。
  - **候选①根修 + e2e 红绿锁**（真 TCP stub，红相=stub 零请求与实机零 REQ 同形）；**候选②传输失败改抛**（五 shim 分层契约 + 非 2xx 值面保留；语料零 `.error/.status` 读方审计）；**候选③审计免改**（proxy 错误面已全 4xx/5xx）。
  - **连带修复**：p080 http 族 4 测 try 免疫化；plan705 线程数测试复活（旧形态上游 POST 打死端口、吞错 200 假绿、up2 空转——从未真正测过并发慢上游；复活后 4/4 稳定绿）。
  - **AC-06 对表**：裸 `cargo t` 4934 跑 4922 绿，12 红 = master 基线红族精确对齐，零新增。
  - **实机**：验收桌面（worktree 构建 MCP :9261）018 书架 3 本 ✓ + proxy REQ ×2 到达 ✓（截图 evidence/712/）；点书导航因用户前台占用停手移交（自动化两次落点经落屏比对确认无害）。
  - **环境登记**：僵户 +1（旧 ui_desktop 20368，taskkill 不可终止）；yoke-derive 0.8.4 镜像缺席 → 锁降级 0.8.2 解锁；桌面壁纸克隆 OOM 新收纳候选（§10⑧，master A/B 同崩证明预存）。
  | blockers: 无（点书复验步骤已写入 §8.5.1，用户随时可做）| next: 用户点书复验 → review（r3 批次 T-17/18/19 全收口后）。

- 2026-09-30 stage:work 续（**r3**，复活后第一批）| plan_id PLAN-712 | plan_revision 3 | outcome: pass（T-19 引擎腿修复+锁、T-18 方案 A 接线+锁；实机腿归验收桌面复验）| code_commit: plan-712-dev @716ab0a6d + a817963b0（承 master 57b9afa60）| task_ids: T-18,T-19 | evidence:
  - **环境**：lang-712 worktree 残壳被僵进程 PID 35340（.tmp-712-app.log 句柄 + CWD 锁定 examples/ui/030-video-player 链）锁死不可清（Stop-Process/taskkill/mv 全败=内核态卡死实锤）→ 按组惯例重建 **`D:/autostack/.wt/lang-712b/auto-lang`**（路径偏离已在案，重启后残壳可清）；auto-down 兄弟 worktree @3373a5c 补建（`../../../auto-down` path 依赖解析）。上一会话遗留在**主检出工作副本**的 back_proxy.rs +7 行诊断 WIP 按 master 零 WIP 红线迁入 plan 分支（716ab0a6d），主检出还原干净（余 .autoos/specs.json/.next-id/716/717 系其他会话在途簿记，未触碰）。
  - **T-19**：全链静态读码（app/viewport/contract/uplink/builder/`!` 惯用法）+ 真引擎探针定谳——保持层干净（120 帧零自发 un-pause + toggle×10 实态一致）；红相=空闲引擎首 poll 自发 `PlayStateChange(true)`。修复：基线采集不回灌 + 写失败不推进 applied（contract.rs）；回归锁 ×2，mpv_contract 12/12（真引擎）。实机走查腿归桌面复验（与 T-16 同窗）。
  - **T-18**：根因定性修正（r2「vwin_rect 恒 None/按宿主布局」不成立——桌面拆借分支早已接线，win_stack 即 Fixed(rect)；真根因=构建期烤定像素 × 缓存快门 × resize 不标脏）。方案 A 落地：`mark_vwin_resized_dirty` + WinRect/交互缩放双臂接线（尺寸真变/Resize 交互才标）；回归锁 `wm_resize_marks_app_view_dirty` + `wm_` 族 10/10。实机腿归桌面复验。
  - **AC-06 对表**：裸 `cargo t --no-fail-fast` 4930 跑 4918 绿，12 红 ⊆ master 基线红族（p053×4+p054×2+plan606/029+desktop_protocol projector_counter+renderer×2+e4；plan707/app_registry/native_gate 本轮未复现=flake 族）——**零新增红**。`plan502_m3_layout_geometry_e2e` 全量跑红、worktree/master **隔离双双绿** = 并行负载序敏感 flake 候选（非本计划引入），登记 §10。
  | blockers: 无 | next: 验收桌面实机复验（T-19 暂停走查 + T-18 win_rect 缩放重排随动 + AC-02 scrub/AC-07 对话框走查）→ T-16 桌面轨 AppTick 路由/T-17 cdylib 重编两笔遗留 → review。

- 2026-09-30 **r2 收纳复开**（archived → executing；用户预授权收纳通道）：新增 SD-05/T-09/AC-07（VM 本地播放接线，纯 app 层，引擎零改动）。实施随修订即落（T-09 体量 = 单 handler 分支）；AC-07 实机取证后随再 merge 回终态。
- 2026-09-30 **r2 再收纳 T-15（图书阅读器打开书详情即占位）**：018 `pages/book_detail.at` 用了 VM 解析器不支持的构造——view 条件 `== null`（null 字面量不在 view 条件语法内，20 连解析错首错「expected LBrace, found null」）+ 条件数组 style，页面被 VM 装载器静默丢弃 → outlet 占位（VM 装载 `if let Ok` 静默吞页 = 可观测性债）；vue 构建同文件硬失败（曾被 VM 静默吞掩盖）。**修复 = app 层按 bookshelf 惯用法重写**（has_book 旗标替代 null 比较、条件数组 style 拆 if/else 双臂、`.len()` 预计算 store 字段、Init try/catch），`auto build` 页面扫描通过。

**根因修正（递增实证修正初判）**：重写后 app 仍「无法启动」——递增二分（最小骨架→全功能逐层加回）定位真凶：**bookshelf 与 book_detail 两页各自定义了同名 `fn chapter_from_progress`，重名定义毒化链接符号表**（「Undefined symbol: chapter_from_progress in module App」）。重写前 book_detail 解析失败被静默丢弃 → 只剩一个定义 → 症状被掩盖；重写后两份定义并存 → 冲突显形。修复 = book_detail 侧唯一化命名（`detail_chapter_from_progress`）。附带实锤两条链接语义：**① try 块体走链接检查**（普通 handler 体解释执行不查——同一 fn 调用在 try 内炸、try 外过）；**② 页面级 fn 不进链接符号表**（装载循环只收 ViewFragment/Widget/Store 三类声明），try 内引用页面 fn 必炸。验收通道复验：唯一名 + 全量页 launch 干净（App Init parked 无链接错）+ Library 渲染 ✓。**引擎债在案**：view 条件 null 字面量 + 数组内 if 表达式的解析器支持（另立引擎计划候选）。实机点验因用户前台占用移交用户（自动化 Alt+F4 误触用户 ZCode 前台窗，已即时停手）。
- 2026-09-30 **r2 T-10/11/12 侦查第一批结果（最小探针二分）**：① 真实 controls.at 结构（含 style 声明）在独立编译下**解析干净**——T-11/T-12 非解析层，定性收敛为**渲染器布局/命中层**（flex-1 col 宽度分配 + SeekArea 命中 bounds）。② 顺带踩出解析器新地雷：**view 节点单行混写**（`button { style: \"...\" text \"P\" {} }` prop+子同行）→ 字符串 prop 被当拼接表达式续读，「Expected infix operator」×20 级联——诊断 UX 债在案（bookshelf 全多行故未暴露）。③ 验收通道缺口：`autoui_inspect`/`autoui_action` 对动态编译 app 组件的元素解析失败（「View not found at path [2,0,1]」）——acceptance 工具族不达 app 层元素，T-12 的合成点击/量测需引擎侧补通道或用户配合。桌面已带 AUTOUI_ACCEPTANCE=1 + MCP :9260 运行（用户实机点验可用 autoui_screenshot 存证）。
- 2026-09-30 **r2 T-15 第二层修复（proxy 实测章节路由 400）**：back `list_chapters(book_id int)` 形参名与路径占位符 `:id` 错配——proxy 按占位符名绑参，「missing param book_id」400 → 前端 catch → 详情页无内容（用户复测报告的症状）。修复 = 形参对齐占位符（`list_chapters(id int)`）；proxy 实测 chapters 返回真实数据 ✓。**corpus 级发现**：桌面通知流在案同族链接失败——013-todo（`api.create_todo`）、025-sys-monitor 等 app 同样「无法启动」（历史通知 id 39/40），归入 T-10/11/12 渲染器批次之后的下一收纳批（app 可开性普查）。
- 2026-09-30 **r2 T-11/T-12 根因实证（终版：布局首帧冻结，resize 零重排）**：常驻化探针（黑洞 fetch 保活）决定性实验——创建 1295x837 → SetWindowPos 900x600 → MCP 渲染帧**逐像素与 resize 前完全一致**（控件锚定原始绝对位置、渲染面不跟随窗口）。结论：**VM 单窗轨布局在首帧后完全冻结，窗口 resize 既不触发重排也不缩放渲染面**。T-10（头部按钮不可见 = 冻结几何在现窗口外）、T-11（seek 条短 = 冻结时 col 宽度）、T-12（点击失效 = 冻结 bounds 与可视错位）三症同根。修复 = VM 窗口运行时的 resize→(渲染面 resize + 重排) 接线（lang 侧，预计小改：定位 VM 窗口尺寸的设置点与 resize 事件消费点）。探针生命周期另债：VM 单窗脚本跑完即自返退出（无订阅 app）、关窗僵尸锁 exe（三例在案）。
- 2026-09-30 **r2 播放器实机复验（验收桌面 1920x1200）**：DesktopBus launch 拉起播放器——播放 ✓、时长/进度真值（00:29→03:11 / 50:19 随动）✓、进度条填充 59% 随动 ✓、色彩鲜活 ✓、默认窗宽（~1005）下布局比例正确 ✓。**待验证的残留** = 虚拟窗 resize 后的重排（用户 1620 窗的 52% 冻结场景）——win_rect 盲扫 wid 12-18 未命中播放器（wid 无发现通道 = 工具缺口），改由用户拖拽一次验证；T-12 点击失效待 resize 修复后复验（虚拟窗几何/命中映射的量测需 autoui_snapshot 达 app 层——工具债同前）。
- 2026-09-30 **r2 T-17 实机点验结果（用户点书 + 全量日志对账）**：用户点第一本书 → **proxy 零新增请求**（书架的 2 条 /api/books 之后无任何 REQ——detail 的 fetch 未达 proxy）；book_detail Init「resumed to completion (86.6529ms)」——fetch 86ms 极速「完成」。**矛盾三方**：未达 proxy（无 REQ）+ 未打 8018（打死端口会 throw 走 Book not found）+ has_book=true（try 未抛）⇒ get_json 的失败语义为**吞错返回空值**（连接拒绝不抛、返回空/成功形状）——待引擎侧确认 VM Http.get_json 的失败返回契约（stdlib http 臂）。**0 entries 的完整成因链（终版）**：前缀变换覆盖判定待查（030 有 P041-DBG 行、018 的非 media 字面量无该行——debug 只对 /api/media/scan 打印，**018 是否被前缀化当前不可见**）→ 相对 URL → 8018 死端口 → get_json 吞错返空 → 空章节 0 entries。修复候选：①前缀变换覆盖对账（book_detail.at 的字面量是否被改写——dump launch 期变换后的 code 即知）；②get_json 失败语义改抛（配合 catch 呈现可诊断错误）；③proxy 错误体状态码改 4xx。
- 2026-09-30 **r2 T-17 收窄（日志实证：018 全 app 零前缀覆盖 + 取数架构疑云）**：诊断日志（desktop-diag3.log）逐行对账——030 播放器有 P041-DBG prefix apply（player_store/playlist ✓ 前缀生效 → 播放器数据面工作）；**018-book-reader 全 app 零 prefix apply 行**。而 018 书架实测加载 3 本书 ✓——取数未走 proxy 前缀却能拿到数据 ⇒ 018 在桌面内的取数架构 = **in-proc cdylib back**（pac back 装载臂，非 proxy 前缀族）⇒ T-15 的 api.at 形参修复**需要重编 cdylib 产物才生效**（产物缓存自旧 api.at）——「0 entries」的直接根因候选。验证/修复：重编 018 back 产物（或清其构建缓存强制重编）→ 点书复测；长修 = cdylib 产物随源码失效重编。附：proxy 的 REQ 日志只会记 proxy 族请求，in-proc cdylib 取数不经 proxy——观测时勿混淆两族。
- 2026-09-30 **r2 T-16 桌面轨定位（AppTick 路由断链）**：验收桌面（T16-DIAG 插桩二进制）实测——独立轨探针 51 条 T16-DIAG（泵臂正常触发）✓；**桌面内嵌轨 0 条**（bookshelf/book_detail 的 park/resume 全程无 T16-DIAG）→ 桌面模式下动态 app 的 `__parked_resume_tick` AppTick 消息**未路由到 renderer.rs:17582 的泵臂**（或路由到无 dirty 传播的另一消费点）。**统一根因候选**：桌面模式与独立模式的动态 app 更新路径分叉——独立轨全链修复后端到端工作 ✓，桌面轨的 resume→dirty→重渲染链在另一条更新路径上缺失同款处理。定位入口：追 `app_tick(app_id, "__parked_resume_tick", 16)` 订阅的消息在 Desktop 模式 update 分派的消费点（renderer.rs 17582 之外），补同款 poll_parked_resumes + is_dirty→view_dirty。T-10/11/12（虚拟窗布局/命中）疑似同路径家族。
- 2026-09-30 **r2 T-16 独立轨复验 ✓ + 数据链端到端贯通**：布局探针（数据链版：直取 proxy chapters 上屏）——独立轨恢复泵实证工作（51 tick，首 tick dirty=true→view_dirty→重渲染），标签自动更新「3 entries | first: Chapter 1: The Source」+ 三章列表渲染 ✓；VM Http.get_json → 数组 → len → f-string/for 渲染全链与 book_detail 同构造逐一验证。**T-16 修复独立轨+桌面轨双确认**。探针多行化过程中再度踩单行混写雷区（自证其易踩性）。
- 2026-09-30 **r2 T-15 复验补充**：验收桌面（MCP :9260）proxy 逐书实测——books/1、/2、/3 的 chapters 路由全部返回真实章节数据（The Gate/The Source/The Keeper 三册各有多章）→ 「0 entries」复现时数据面已排除；前端 `.len()` 惯用法与书架 `.store.books.len()` 同款已证。剩余验证 = 用户实机单击书卡看详情渲染（MCP 元素点击工具不达 app 层动态组件——工具债在案）。
- 2026-09-30 **r2 T-16（新收纳，Loading 卡死根因）**：018 打开后「Loading… (bookshelf)」永久停留，点别处再回来才出——PLAN-711 Init demand 生命周期的**重渲染断链**：bookshelf_Init park（HttpRequest）→ resume tick 恢复完成 → InFlight→Done 翻转需 dispatch_pending_inits 再驱动（帧泵承担）→ Done 后需 view_dirty 触发重渲染。修复尝试#1 = `__parked_resume_tick` 臂补 is_dirty→view_dirty 回填（renderer.rs，已提交）——**实机复验单点不足**（T-16 二进制下阅读器仍卡 Loading；is_dirty 恒假/恢复写入不经 write_state——深层在 PLAN-711 demand 泵与帧泵的相位翻转重渲染链，嫌疑 = InFlight→Done 静默翻转无 epoch/dirty 传播）。AUTO_SCHED_DIAG=1 实机 trace（desktop-diag.log）：帧泵持续空转（queued_init=0 dispatched=0）→ 恢复后无任何待驱动任务、也无 view 重渲染——**脏标记/重渲染断链的确切环**待下一会话以该 trace 为入口定位（嫌疑：__frame_pump 臂的 state.component 指向 shell 而非 app、或 is_dirty 被 view() 消费清零后无重建）。症状与 T-10/T-11/T-12（布局/命中）可能同源不同层。
- 2026-09-30 **r2 再收纳 T-14（用户钦定默认对）**：「默认深色用 purple.png；浅色用 songyu.png」——默认对升级为引擎概念（boot 链第二档），恢复默认从此有系统定义的标的；os-config「恢复默认壁纸」按钮留后续（写臂就绪后属设置面工作）。
- 2026-09-30 **r2 再收纳 T-13（用户裁定）**：新开桌面（浅色）无壁纸——浅色槽位测试残留纯色 #101014 被 boot 如实渲染。T-13 = boot 壁纸优先级重构（图片 > 目录首图 > 色值 > 内置默认），「任何时候打开都加载默认壁纸」从此有引擎保证；单测绿。
- 2026-09-30 **r2 验证期新发现（下一笔收纳候选 T-10）**：712 二进制下播放器头部动作按钮（主题切换 + 队列开关）在 VM 双轨（独立窗 + 桌面内嵌）均不渲染/不可达——712 前实机截图在案有按钮（用户首报截图），712 后 hover 探针无高亮；队列面板因此不可达，T-09 的 UI 路径被该回归挡住。嫌疑面 = 712 渲染器改动（video_uplink wrapper / 契约类型）对非 video 节点布局/icon 的影响，待 lang 侧定位。注：键盘快捷键缺失（pac「全套键盘快捷键」未实现）为同屏发现的语料债。
- 2026-09-30 **r2 用户实机反馈再收纳两笔（T-11/T-12，同属 VM 布局/命中层）**：
  - **T-11 播控条布局分布错**：controls.at 设计意图 = 左 `col flex-1`（时间行 + `w-full` 进度条）+ 传输键 + 音量/倍速组右聚；实渲染进度条仅占左侧 ~2/3、倍速「1.0x」与音量组之间 ~350px 空档——VM 渲染器对该 flex 行结构的宽度分配偏离设计（用户截图 7b9397df；对比：712 前同布局即已如此 = 非本计划回归，属 VM 布局面存量缺陷被可用性提升后显形）。
  - **T-12 进度条点击跳转失效**：时长 53:12 > 0（store 门已过），SeekFraction → seek_target → 契约 position 的 app 侧链路代码完整；断点疑似与 T-11 同源（绘制位置与命中 bounds 错位）或 SeekArea 命中层。实机自动化验证因用户前台占用中止（探针点击落入用户编辑器窗口，输入自动化即刻停止），待用户配合窗口再取证。
  - 处置：T-10/T-11/T-12 合并侦查——VM 渲染器行布局的 flex 宽度分配 + 命中 bounds 一致性，一次定位三处症状。712 二进制下播放器头部动作按钮（主题切换 + 队列开关）在 VM 双轨（独立窗 + 桌面内嵌）均不渲染/不可达——712 前实机截图在案有按钮（用户首报截图），712 后 hover 探针无高亮；队列面板因此不可达，T-09 的 UI 路径被该回归挡住。嫌疑面 = 712 渲染器改动（video_uplink wrapper / 契约类型）对非 video 节点布局/icon 的影响，待 lang 侧定位。注：键盘快捷键缺失（pac「全套键盘快捷键」未实现）为同屏发现的语料债。
- 2026-09-30 stage:new 起草（drafting）。`outcome: pass`——四项初始范围已按实证落档，DP-1..4 为工作期决策点（均含决策工件要求）。`next: work`（执行前按范式开 `.wt/lang-712` 组 worktree）。
- 2026-09-30 stage:work | plan_id PLAN-712 | plan_revision r1（起草态即执行，无修订）| outcome: pass（T-01..T-06 完成，代码面验证全绿；T-07/T-08 进行中）| code_commit: worktree `plan-712-dev`（见提交）| task_ids: T-01,T-02,T-03,T-04,T-05,T-06 | evidence:
  - T-01/T-02 探针定谳 + T-03/T-04 实现 = worktree 提交（evidence/712/{dp2,dp3} 两档 + mpv_engine/mpv_channel/video_contract/video_uplink 四测面全绿）。
  - T-05：E-6 偏差更正（load 快照已在位，PLAN-044），回归锚测试补齐 20/20。
  - T-06：DP-4=历史产物；主检出 14 份 gitignored 生成 Cargo.toml 已批量清理（cargo metadata 解析恢复）+ tracked 015-notes 产物修正。
  - 环境注记：worktree 组补建 auto-down 兄弟（detached @3373a5c，只读依赖约定）。
  - 预存 flaky 在案：`state_file::lock_serializes_critical_sections` 全库并跑时红、隔离复跑绿（flock 争用型，非本计划改动面，T-08 与 master 基线对表裁决）。
  | blockers: 无 | next: T-07 实机走查（AC-01..05，auto-os 桌面 + 独立 VM 窗）→ T-08 日常档对表。
- 2026-09-30 stage:work 续 | T-07 实机走查（独立 VM 窗腿）+ T-08 回归收口：AC-01/05 实锤（时长 00:49 真值回灌 + 独立窗可播，截图 evidence/712/vm-030-playing.png）；AC-02/03/04 部分取证（遗留项与死因注记见 §8/§10）；T-08 经用户裁定不跑全量（全量档现状有问题），以 master 同测对照收口（2 红预存）。当前 step=8/8 全部处置，**plan 维持 `executing`**：T-07 桌面腿 + Web 对照腿、AC-02 scrub 全证、AC-03 双端像素对照未完，完整收口后走 review。`next: work 续行或 review（视用户对遗留项的取舍）`。
- 2026-09-30 **验收裁定（用户）**：「只要通过了本计划自己的用例，就可以 review 通过并 merge」——本计划自有用例（desktop_config 20/20、video_uplink 3/3、video_contract 4/4、mpv_channel 传递归一双臂、mpv_engine 探针）全绿即满足验收；T-07 遗留取证项（桌面腿/Web 对照/scrub 全证/AC-04 实机复演）不再阻断。

## 10. 待澄清事项

- **扩展位（用户裁定，非待澄清）**：本计划为桌面 VM 验收缺陷的收纳载体——后续其他桌面 app 的 auto-lang 侧问题以 plan_revision 增补任务/AC 归入，不另起计划；每次增补记录来源与证据。
- **r3 工作期新登记（2026-09-30，worktree a817963b0）**：
  - ④ **plan502_m3_layout_geometry_e2e 并行 flake 候选**：r3 日常档全量红（0.3s 失败）、worktree 与 master **隔离复跑双双绿**（0.15s）——并行负载序敏感，与 p053_6 同型；非 712 改动面（diagram 布局几何，本批只触 mpv contract/session resize 脏标记）。归 `/auto-plan:regress` 档观察。
  - ⑤ **僵进程残壳（lang-712 组目录）**：PID 35340（auto.exe @ 已剪除 worktree target，14:08 启动）锁 `.tmp-712-app.log` + CWD 锁 examples 空目录链；Stop-Process/taskkill/改名/mv 全败。**重启后删除 `D:/autostack/.wt/lang-712`** 即清；r3 起 plan worktree 在 lang-712b（本登记为 merge 清理时的例外指引）。
  - ⑥ **viewport.at 过时注释**：「`flex-1` 与 `shrink-0` 在 VM 不生效」已不成立（Plan 370 Issue 1 起.flex-1 → width=Fill 映射在位，iced_adapter.rs:1158）——T-11 播控条塌缩的成因不在 flex-1 缺失；该注释待语料清理批顺带更正（防误导后续排查）。
  - ⑦ **yoke-derive 镜像缺席（env，r3）**：worktree 本地 Cargo.lock（gitignored）钉 yoke-derive 0.8.4，aliyun 镜像索引刷新后仅到 0.8.3 → 全构建拒绝解析；`cargo update -p yoke-derive --precise 0.8.2`（对齐主检出既有锁定）解锁。复发处置同款。
  - ⑧ **ui_desktop 壁纸克隆 OOM（新收纳候选，r3 实证 2026-10-01）**：桌面启动后数秒~数分钟死于 `memory allocation of 2660706 bytes failed`——栈定谳 `load_image_bytes`（renderer.rs:6590）在**每次视图重建**裸 clone 壁纸图字节（本机 purple.png 2.55MB 配置在案，深色主题 boot 链②档），分配失败 = 进程提交耗尽形态。**非 712-r3 回归**：master 构建（A/B，desktop-master-ab.log）同签名同尺寸崩（t≈152s），worktree 构建 3/3 崩（11s/45s/273s 不定）；昨日 21:59 的同源桌面（用户 20368）存活 26h——变量是**当日机器负载**（用户前台浏览器 + 僵尸进程群 + 页面文件峰值 60GB 的提交压力史），重载下重建churn的 2.66MB 裸 clone 是 OOM 金丝雀。修法候选：壁纸字节/解码结果缓存一次（挂 config 或 handle 缓存，与 Plan 650 Element 缓存族同源）；登记待用户裁定收纳批次。**本轮 4 份崩溃日志**：desktop-r3.log / desktop-r3b.log / desktop-r3c.log（含 RUST_BACKTRACE 栈）/ desktop-master-ab.log（worktree 组目录，未入库）。
- **r2 工作期新登记（2026-09-30，worktree 7f483e1ab）**：
  - ① **T-11 塌缩非确定性源（修复前置）**：headless 取证探针同源 col_right 438↔658px 随**进程启动**翻转（icon/窗宽/onseek/样式逐项排除），真机塌缩比例（~45%）落翻转域内；嫌疑=动态视图构建顺序敏感层（组件实例化/HashMap 迭代序）。探针已从门禁摘除；定谳入口=进程内两次构建同 view 比对 + builder 侧 HashMap 遍历审计。
  - ② **master 日常档基线 14 红**（较 T-09 时点 2 红扩大）：p053 族 4（含 p053_4/p053_6，并行负载序敏感——隔离可绿）+ p054 族 2 + plan606 029 + desktop_protocol（projector_counter/native_gate）2 + renderer（desktop_bus_inbox/desktop_surface_merge）2 + e4_default_http + plan707_wait_generator + app_registry 2（仅主检出）。归批量回归档（/auto-plan:regress）收口，非本计划范围。
  - ③ **上一会话卡死进程定谳补充**：死因注记现场（030 VM 解释器 ok=true 后宿主进程挂死 5h、taskkill 不可终止=内核态卡死特征）本会话经 Stop-Process 成功终止；`.tmp-712-app.log` 碎片已清，worktree 无遗留锁。
- DP-3 若证实色偏源为 mpv SW 路径不可协商项（如 colormatrix 无法显式设定），回退方案 = shader 侧矩阵修正或接受并成文差异阈值——届时提交用户裁定。
  - **已定谳（T-02 探针）**：SW 路径确不吃任何协商面（三配置同帧逐位全等）→ 已走 shader 侧归一（T-03），无需用户裁定。
- SD-04 的 spec 落点（overview vs 生成器 spec）待 DP-4 定案后随复审固化。
  - **DP-4 已定案**（历史产物，模板已洁）；SD-04 落点待复审随 SD-01..03 一并固化。
- **T-07 遗留死因（预算外登记）**：030 VM 窗内 MCP `press` progress 元素 → wgpu BufferViewMut 告警洪泛 → VM 解释器 `ok=true` 优雅退出。非崩溃、非 panic，因果未定谳；本计划改动面（uplink wrapper 仅在真事件时 publish，与 press 无关）初步排除但未证。复现路径在案，建议后续会话定谳或归入收纳载体。
- **E-2/E-3 证据勘误**：auto-os 会话证据基于"修复前"二进制（E-7 明示"修复前内存态"）；E-3 的偏暗偏暖机理 = web-colors 形态下 sRGB 纹理双重解码（dp3 证据档闭环复算吻合），非传递函数方向问题。
- 2026-09-30 stage:review | plan_id PLAN-712 | plan_revision r1 | outcome: **pass** | code_commit: plan-712-dev cc46d64c1（3 提交：c3aa6d7be/3896b92fe/cc46d64c1）| task_ids: T-01..T-08 | evidence:
  - 用例面（用户验收口径）全绿：desktop_config 20/20（含首写快照回归锚）、video_uplink 3/3、video_contract 4/4（真实管线接线钉）、mpv_channel present_renormalizes 双目标臂、mpv_engine 探针两测；`cargo check -p auto-lang --features mpv-widget` 干净。
  - 清单复核：AC-01/05 实机实锤（截图+state 断言）；AC-02/03/04 按用户裁定以自有用例口径收口；AC-06 master 同测对照无新增红（2 红预存复证）。
  - 遗漏/延后扫描：T-07 遗留取证项经用户裁定不阻断，登记在案（§10）非静默弃；死因注记（MCP press → 会话退出）已登记待后续定谳。
  - 健康：无编译警告新增、无调试残留打印（临时脚本已 gitignore 不入库）。
  | spec delta: SD-01..04 随 merge 沉淀至 docs/specs/auto-lang/ui/overview.md（媒体元素节上行契约/色彩配对规则/config 合并语义/examples 产物 feature 面）| next: merge。
- 2026-09-30 stage:work 续（r2）| plan_id PLAN-712 | plan_revision 2 | outcome: pass（T-12 引擎修复落地 + T-08 零新增红收口 + T-10 复 refute + T-11 塌缩实锤未修归后续）| code_commit: plan-712-dev 7f483e1ab（承 master 472e035a7）| task_ids: T-08,T-10,T-11,T-12,T-13,T-14,T-15 | evidence:
  - r2 簿记对账：T-09/T-13/T-14/T-15 实现在 master 在案（331fd8f5b..8bb1f3279），勾选与证据补齐；`desktop_surface_storage_roundtrip_and_wallpaper_resolution` 在重建 worktree 实跑绿。
  - T-12 根因修复（engine）+ 双形态回归锁；T-11 真机复现截图取证（evidence/712/master-binary-window.png + MCP 快照）+ 三支 headless 布局回归锁在库（`cargo t p712 --features iced-layout-tests` 3/3 绿）；T-10 主检出不复现裁定（0 输入走查）。
  - 环境：lang-712 worktree 重建（上一会话残留进程卡死内核态占路径，手工注册 worktree 元数据绕过，该进程本会话终获终止后碎片清理）；auto-down 兄弟 worktree @3373a5c 补建（路径依赖解析）；030-back 生成树自主检出拷贝（gitignored，workspace members 收窄为 030 一员）。
  - master 日常档基线已扩至 14 红（T-09 记录 2 红）：p053 族 4 + p054 族 2 + plan606/029 + desktop_protocol 2 + renderer 2 + e4 + plan707 + app_registry 2；worktree 12 红 ⊂ master 14 红=本计划零新增（p053_6 在 worktree 隔离红但 stash 对照+master 全量跑均红=并行序敏感，非本计划引入）。
  | blockers: 无（T-11 定谳修复与 AC-02/AC-07 实机点击取证归后续）| next: T-11 修复或 review（视用户取舍）。
- 2026-09-30 **验收裁定（用户，r2）**：「先收口（T11记录下来）」——T-11 保留在案为登记态（§8 [~] + §10 ①：真机塌缩实锤 + 非确定性源定谳入口），不阻断本轮收口；后续按引擎计划/收纳通道再修。r2 自有用例口径全绿（desktop_config 20/20、video_uplink 3/3、video_contract 4/4、mpv_channel 双臂、T-12 回归锁双形态、p712 布局探针 3/3、AC-06 零新增红），与 r1 裁定同口径。
- 2026-09-30 stage:work 收口 | plan_id PLAN-712 | plan_revision 2 | outcome: pass | code_commit: plan-712-dev 7f483e1ab（承 master 472e035a7；簿记 170e5e3b8）| task_ids: T-01..T-15 | evidence: §8 勾选与子证据全对账；开放项=T-11（登记态）+ 实机取证腿（AC-02 scrub/AC-07 走查/AC-08 浅色截图），均经用户裁定不阻断 | blockers: 无 | next: review。
- 2026-09-30 stage:review | plan_id PLAN-712 | plan_revision 2 | outcome: **pass** | reviewed_commit: plan-712-dev 7f483e1ab | base_commit: 472e035a7（master；r1 提交 cc46d64c1..c3aa6d7be 已在 master 祖先）| dependency_revisions: auto-down 3373a5c（detached 只读兄弟 worktree，路径依赖解析）| spec_inputs: docs/specs/auto-lang/ui/overview.md（SD-01..08 冻结于本件 r2 文本）|
  - **独立性声明**：实现会话自审（同会话无独立复审上下文），裁定从工件重建（r1 先例同款）——判据全部来自本基线新鲜复跑与在库工件，不采信执行期摘要。
  - **验收映射**（AC→证据→裁定）：AC-01 pass（r1 实机实锤：OSD 时长 00:49 真值+进度随动，截图 evidence/712/vm-030-playing.png）；AC-02 pass（用户裁定口径=T-12 修复+OnTime 回灌链实机连续跨文件推进；scrub 三采样全证登记归后续）；AC-03 pass（T-03 shader 传递归一+VM 视觉验证；Web 同帧对照登记归后续）；AC-04 pass（desktop_config 20/20 含 `save_after_external_edit_preserves_fields` 回归锚；实机复演登记）；AC-05 pass（r1 `auto run -r vm` 命令实跑出窗可播）；AC-06 pass（全量对照 worktree 12 唯一红 ⊂ master 14 红=零新增，本基线新鲜复跑）；AC-07 partial→登记（T-09 接线在树+MCP 快照在案，对话框实机走查归后续）；AC-08 pass（`desktop_surface_storage_roundtrip_and_wallpaper_resolution` 双结果放行绿；实机浅色截图登记）；AC-09 pass（深 purple/浅 songyu 双断言绿）。
  - **新鲜复跑**（worktree @7f483e1ab）：验收族 23/23 绿（desktop_config 20+video_uplink/video_contract+progress_onseek 双测+present_renormalizes+desktop_surface_storage）+ `cargo tv` 162/162 + p712 布局探针 3/3（iced-layout-tests）。
  - **findings**：F-712-1 T-11 播控条塌缩未修（用户裁定登记态；定谳入口 §10①）；F-712-2 实机取证腿 4 项登记归后续（AC-02 scrub/AC-07 走查/AC-08 截图/AC-04 复演）；F-712-3 master 日常档基线 14 红扩容归 `/auto-plan:regress` 档（§10②）。
  - **知识增量**：SD-01..07 核验——目标路径 `docs/specs/auto-lang/ui/overview.md` 有效，before/after 与实现一致（上行契约双端 0.25s 节流不变/2.4→sRGB 归一规则/config 合并无首写例外/壁纸 boot 优先级四档/分主题默认对/VM 本地播放 dialog_open 契约）；**补 SD-08**（事件回调 `$` 占位实参契约=T-12 修复的持久语义，本复审新增）；supersedes/new 空表理由见 frontmatter 注；touched_goals=[GOAL-007,GOAL-009,GOAL-010]。
  | spec delta: SD-01..08 随 merge 沉淀至 docs/specs/auto-lang/ui/overview.md | next: merge。
- 2026-09-30 **合并收据（PLAN-712:r2）**——五检查点：
  - **prepared** ✓：reviewed 基线 = plan-712-dev @7f483e1ab（rebase 后 e228b7c1a，patch-id `75daac76a4c01ee94c0fbf3d915de8fe24a27e3a` 两点全等=安全重写证明）；依赖 auto-down 3373a5c；canonical 目标 `docs/specs/auto-lang/ui/overview.md`（媒体节 PLAN-712 增补块族）+ `docs/specs/auto-lang/ui/plans.md`；SD-01..04 已由 r1 merge 在位（复核无需重写），r2 增量 SD-05..08 新增 27 行。
  - **landed** ✓：ff-only 零合并提交——master tip 依次 451dc1401（SD-05..08 沉淀，对 e228b7c1a 纯文档 +27 行）→ ff32d7004（ui/plans.md 712 行回写）；当前 master=ff32d7004。
  - **ledger_refreshed** ✓：`.autoos/specs.json` reviews 节追加 `P712-1`（手工回退路径——musk 后端 127.0.0.1:8080 不可达实测，README §112 文档化回退；读回验证通过、六节完整、无重复项）；`scripts/spec-index.py` 再生 INDEX.md 零 diff（project.md 未变）。
  - **archived** ✓：`docs/plans/archive/712-vm-desktop-defects.md` status archived（本件，归档位置 r1 时已就位）。
  - **cleaned** ✓（含残差登记）：wt-guard 双 worktree 预检 clean（零 reparse point）；`plan-712-dev` 分支已删（was ff32d7004，全提交已 landed）；主 worktree 注销 + 27GB 内容剪除；auto-down 兄弟 worktree 注销移除。**残差**：`.tmp-712-app.log`（219KB）+ 空 `examples/ui/030-video-player/` 目录链两路径被内核卡死僵进程 PID 35340（死因注记现场，§10③）锁定不可删——git 元数据/分支/全部内容已清，残渣 284KB 纯 untracked 无 git 无链接，组目录 `D:/autostack/.wt/lang-712` 待重启后删空壳即可。
  - **陈旧产物观察（PLAN-092 先例口径）**：本计划触及 auto-lang UI 引擎（aura_view_builder/iced renderer/mpv present）——release 档二进制与 `gen/front/vue/dist` 未随本次合并重建；桌面实机消费面当前为 r2 验证 worktree debug 二进制形态（重建+重启归生产面会话）。
  - **批量回归到期判定**：不到期——712%5=2 非整除，收据当日新鲜（last_covered=712@7fcf913eb，2026-09-30T08:04:49Z）。
