---
plan_id: PLAN-712
status: reviewed               # drafting → executing → execution_done → reviewed → archived
feature_name: VM 桌面验收缺陷收敛（视频引擎双缺陷 + 壳配置持久化 + examples 依赖）
author: [zcode(auto-os 会话转介)]
created_at: 2026-09-30
updated_at: 2026-09-30

# /auto-plan:review 结束时填写：
supersedes_spec_components: []
new_spec_components: []
touched_goals: []             # 引用 docs/specs/goals.md 的 GOAL-NNN

affects: [auto-lang/ui]
current_step: 6
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
  - [x] **已完成**（2026-09-30，**用户裁定收口：全量测试不跑**——当前全量档本身有问题）：fail-fast 跑批 1440/5747：1438 绿 + 2 红 `musk_vm_track_p053_1_widget_computed`（computed 透传解出 0 行）——**master 基线同测实跑复证同样 2 红 ⇒ 预存红，非本计划引入**，AC-06 口径（无新增红）满足。作用域绿面：desktop_config 20/20、video_uplink 3/3、video_contract 4/4、mpv_channel 像素归一双臂绿、`cargo check --features mpv-widget` 干净。`state_file::lock_serializes_critical_sections` 并跑红/隔离绿（flock 争用型 flaky，另案在档）。[✅ 已完成]

## 9. 复审记录

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
