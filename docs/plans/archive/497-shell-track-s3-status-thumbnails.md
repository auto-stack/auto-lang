---
plan_id: PLAN-497
status: archived                 # drafting → executing → execution_done → reviewed → archived
feature_name: shell-track-s3-status-thumbnails
author: [zhaopuming]
created_at: 2026-08-31
updated_at: 2026-08-31

# /auto-plan:review 结束时填写：
supersedes_spec_components:
  - "specs/auto-lang/ui/overview.md: shell-track 段增补 497 S3 落地（时钟/托盘组/每窗口真缩略 + mru_thumbs 通道 + untracked popover 臂修复 + window_thumbnail widget）；switcher/dock 消费面描述 icon 占位→真缩略升级"
new_spec_components:
  - "schema/projection-protocol-v1.md: §2 字段表增 __wm_clock 非门控注入字段（v1.4 内字段扩展小节，497）"
  - "crates/auto-lang/src/ui/iced/snapshot.rs: 快照核心（裁剪式整窗快照——thumbnail_from_screenshot/TTL 缓存/抓取队列；T1 定案产物）"
  - "schema/aura.at + registry: window_thumbnail widget（I4 双端登记族；@/wm/WindowThumbnail vue 映射）"
touched_goals:             # 引用 docs/specs/goals.md 的 GOAL-NNN
  - "GOAL-009: 虚拟桌面与桌面 Shell——桌面特性线收官（S3 Status 栏：时钟/托盘组/每窗口真缩略）"

affects: [auto-lang/ui]
current_step: 8
total_steps: 8
---

# [PLAN-497] shell-track S3——Status 栏（时钟 / 托盘组 / 每窗口真缩略）

## 变更摘要

Design 25 §2 S3（Status 栏：时钟/托盘/每窗口缩略管理）——设计里唯一整体
未动的 shell 表面，也是桌面特性线的**收官件**：

1. **时钟**：dock 区时钟（shell 本地 tick，30s 粒度；不走投影——避免高频
   投影抖动）。
2. **托盘组**：dock 端右侧归组（479 通知铃铛 + 时钟 + 状态图标挂载点容器；
   app 图标注册 API 后置为非目标）。
3. **每窗口真缩略**：**离屏快照 API**（wid → 图像）——设计定位"缩略=离屏
   快照（路线 B lite）"，现状 v1 图标占位（switcher `mru_icons`、dock
   `app-window` 占位）。本期建 UI 侧快照核心（复用 `ui/headless/` 渲染
   地基），登记 `window_thumbnail` widget（Design 25 §4 登记族，I4 双端），
   并接入三处消费者：**switcher 行缩略、dock 条目 hover 预览、pager 分区
   hover 预览**。

## 目标

- **G1 时钟**：dock 右端常驻时钟（HH:MM），本地 tick 驱动，零投影流量。
- **G2 托盘组**：铃铛（479）+ 时钟 + 挂载点容器成组右置；布局在 dock 配置
  （position 顶/底）两态下均正确。
- **G3 快照核心**：`snapshot_window(wid) -> Option<ImageData>`（离屏渲染
  App 视图树 → RGBA/PNG；降采样到缩略尺寸）；新鲜度策略 = 召唤时即时抓取
  + 短 TTL 缓存 + relayout/关闭失效。
- **G4 widget 登记**：`window_thumbnail` 进 `schema/aura.at` +
  WidgetRegistry（I4）；vm 臂 = 快照 image 渲染（无快照回退 icon 占位）；
  vue 臂 = 组件登记 + v1 占位渲染（双端行为差异记待澄清①）。
- **G5 消费者三处**：switcher 行（icon→缩略，开关保留 icon 兜底）；dock
  条目 hover 预览（422 popover 先例）；pager 分区 hover 预览（该区窗口
  缩略小网格，条目≤4 截断）。
- **非目标**：native docked 窗口的 DWM 缩略（待澄清②）；托盘 app 图标
  注册 API（挂载点先行）；S8 shell IME；快照后台定时刷新（召唤式即取即用）。

## 架构方案

```
快照链：wid → AppSession 视图树快照(缓存视图) → 离屏渲染(ui/headless 地基)
        → 降采样 RGBA → window_thumbnail(image) / 消费面注入
消费者：switcher.at 行（召唤注入 mru_thumbs 平行列表，同 mru_icons 模式）
        shell.at dock 条目 hover → popover 预览（422 先例）
        shell.at pager 分区 hover → 分区缩略网格
时钟/托盘：shell.at 本地 tick 状态（setInterval 同型宿主定时）+ 右端组容器
```

- **快照取材点**：既有渲染缓存（`cached_rendered`/视图树）而非重演 VM——
  渲染树已每帧维护，离屏栅格化是纯渲染侧动作；实现路径 T1 spike 定案
  （headless 复用 vs iced overlay 离屏 target）。
- **投影协议**：消费者注入走既有平行字符串列表模式（`mru_thumbs` 同
  `mru_icons`），不新增协议动词/字段族——快照数据不走投影（体积大、
  召唤式，由宿主直接注入控件资产）。

## 技术栈

iced 离屏渲染（headless/testbench 地基）+ image 降采样 + 既有 popover/
投影注入管线。零新三方依赖。

## 需求分析与背景调查

（取材 docs/specs/overview.md §ui + 现场核验 2026-08-31）

- **设计依据**：Design 25 §2 S3 行（风险列注明"缩略与 IME 触及深水区"——
  真缩略即本期深水点，故 T1 spike 先行）；§6 挂起注记"缩略管理（S3 真缩略）
  → 挂 386 复活（离屏快照=路线 B lite；v1 图标占位）"——386 全阶段已归档，
  解锁。
- **现状核验**：switcher 消费 `mru_icons` 平行列表（assets/switcher.at:11/30，
  icon 占位）；dock native 条目占位 `"app-window"`（486）；479 铃铛在 dock；
  无时钟；UI 侧无 wid→图 API（480 的快照为协议/v2a 形态侧，非 UI 消费件）。
- **可复用**：`ui/headless/`（离屏渲染地基）；422 `ui/iced/popover.rs`
  （hover 弹层）；472/478 平行列表注入模式；479 铃铛。
- **排程**：494/495/496 在 review（即将释放会话）；本期与三者改动面交叠
  仅 shell.at/switcher.at（496 也动 shell 资产——后合者 rebase，预期 hunk
  级）。**桌面特性线在本计划之后仅剩长线项**（457/S8 与增强型债务）。

## 详细设计

### 1. 快照核心（ui/iced/ 新 snapshot.rs）

- `WindowSnapshot { rgba: Vec<u8>, w, h }`；`snapshot_window(wid)`：
  从渲染缓存取视图 → 离屏栅格化（T1 定路径）→ box 降采样（长边 ≤256）；
- 缓存：`HashMap<AppId, (WindowSnapshot, Instant)>`，TTL 2s + 事件失效
  （relayout/close/dirty）；召唤式调用，无后台定时。
- **T1 spike**：headless 复用 vs overlay 离屏 target 的可行性与成本对比，
  结论回写待澄清③。

### 2. window_thumbnail 登记（I4）

- `schema/aura.at` 新 widget：props `{ wid: string, fallback_icon: string }`
  （backends：iced full / web component）；
- vm 臂：查询快照缓存 → image；miss → 异步触发抓取 + 本帧 fallback icon；
- vue 臂：组件登记 + v1 占位渲染（icon + 边框），双端差异记待澄清①。

### 3. 时钟与托盘组（assets/shell.at）

- dock 右端组：`[挂载点容器][铃铛(既有)][时钟]`；时钟本地 tick（宿主定时
  注入分钟字符串或 .at 本地 interval——执行期按 shell.at 定时先例定）；
- dock position 顶/底两态布局正确（487 set_dock_position 联动验证）。

### 4. 消费者

- **switcher**：召唤注入 `mru_thumbs`（平行于 mru_icons；缩略缺失项空串
  → 控件 fallback）；行渲染 icon→thumbnail 升级；
- **dock hover 预览**：条目 hover → popover（422 先例）内 window_thumbnail
  （wid=条目窗口）；
- **pager hover 预览**：分区标签 hover → popover 网格（该区窗口缩略 ≤4，
  超出 "+N"）。

## 测试设计

1. **T1 spike 成文**：快照路径对比结论（待澄清③）。
2. **T2 快照单测**：注入彩色节点视图 → snapshot_window 断言尺寸/中心像素
   色；TTL 过期/失效路径。
3. **T3 装载测**：shell.at 时钟/托盘组渲染；switcher mru_thumbs 注入行
   缩略；dock/pager hover popover 出现（desktop_mcp 五套同型）。
4. **T4 I4 对拍**：window_thumbnail vue 端登记与占位渲染金样（a2vue 体系）。
5. **T5 实机**：switcher 召唤见真缩略；dock hover 预览；pager hover 网格；
   时钟走字；顶/底 dock 两态；后台 App 缩略新鲜度（切换内容后再召唤更新）。

## 验收标准

1. T2–T4 绿；T5 实机清单 PASS 留痕。
2. 缩略缺失路径全程有兜底（icon 占位），无空白/panic。
3. `schema` 三件套绿（新 widget 登记）；`cargo t ui` 不回归；零警告。
4. 投影协议零改动（平行列表注入模式）；非目标项未夹带。

## 执行步骤

（原子任务：精确文件路径 + 确切操作 + 验证命令；每步完成后追加 [✅ 已完成] 一行证据）

1. **T1 spike**：快照路径对比（headless 复用 vs overlay 离屏 target）——
   临时最小 demo 验证一条可 行路径，结论回写待澄清③。
   验证：demo 产出一张真缩略图留痕。
   [✅ 已完成] 定案**裁剪式整窗快照**（三候选对比见待澄清③）：headless
   ❌（no-op 无栅格化）、overlay 离屏 target ❌（iced 0.14 无公开子树
   离屏 API）、`iced::window::screenshot` 整窗 RGBA × `VWinState.rect×
   scale_factor` 裁剪 × box 降采样 ✅。demo = 临时 example
   `examples/t1_thumb_spike.rs`（真 iced 窗口四色块 2×2 → 800ms tick →
   screenshot 800×600 物理 scale_factor=2 → 裁剪 400×300 → 缩略 256×192
   → 四象限色相断言 PASS exit 0）；留痕 `tests/screenshots/
   t1-spike-full.png` + `t1-spike-thumb.png`（10:37，2026-08-31）。
2. **快照核心**：新建 `crates/auto-lang/src/ui/iced/snapshot.rs`
   （snapshot_window + TTL 缓存 + 降采样）+ T2 单测；`ui/iced/mod.rs` 登记。
   验证：`cargo check -p auto-lang && cargo t snapshot`。
   [✅ 已完成] snapshot.rs 落地：`WindowSnapshot` + `thumbnail_from_screenshot`
   （crop_physical×scale_factor + downsample_box 长边≤256，越界 clamp/零尺寸/
   短 RGBA 守卫）+ 进程级 TTL 2s 缓存（snapshot_window/cache_put/invalidate/
   invalidate_all，惰性过期清除）。TDD 红→绿（RED stub 3 failed → 实现
   4 passed）。验证实况：`cargo check -p auto-lang --features ui-iced` 零
   error（模块在 ui-iced gate 下，无 feature 时空集）+ `cargo test -p
   auto-lang --lib --features ui-iced t2_snapshot` 4/4 绿 + `cargo t
   snapshot` 3/3 绿（既有 snapshot 名测试无回归；注：iced 模块测试需
   ui-iced feature，日常 `cargo t` 档不编译该模块——同 renderer.rs 既有
   测试同况）。
3. **widget 登记**：`schema/aura.at` 增 window_thumbnail +
   `ui_gen/widget/registry.rs` spec；vm 臂（`ui/iced/renderer.rs` 增渲染臂）
   + vue 臂占位。
   验证：`cargo test -p auto-lang --test schema_drift && cargo test -p auto-lang --test docs_gen && cargo t ui`。
   [✅ 已完成] 七表登记：aura.at element（vue: @/wm/WindowThumbnail 映射
   即 vue 占位臂——a2vue 转译读 schema 同源，金样对拍在步骤 7）+
   schema.rs ElementDef + view_builder 两臂（convert_window_thumbnail，
   三表字面统一 window_thumbnail）+ View::WindowThumbnail 变体（vnode/
   snapshot_builder 检视臂随编译器穷举驱动补全）+ renderer 渲染臂（快照
   命中→image::Handle::from_rgba 直绘+Nearest 锐度+border/radius/bg 包裹；
   miss→request_capture（500ms 冷却队列）+ lucide fallback 经 Image 臂
   复用；native "N<slot>" parse 失败天然 fallback=待澄清②）+
   render_support Full + registry WidgetSpec(Display)。围栏实况：
   schema_drift 1/1 ✓（新孤儿零——三表字面对齐策略）；docs_gen 4/4 ✓
   （KITCHEN_SINK_UPDATE/DOCS_GEN_UPDATE 再生成 kitchen-sink.at + core.md；
   docs 覆盖围栏按 mousearea 484 先例入 DOC_EXCLUDE——桌面 shell 专用
   消费面，单 App gallery 无虚拟窗可缩略恒 fallback，不设独立页）；
   cargo t ui 777/777 ✓。
4. **时钟/托盘组**：`crates/auto-lang/assets/shell.at` dock 右端组（挂载点/
   铃铛归组/时钟 tick）。
   验证：`cargo t desktop_mcp`（T3 相关用例）。
   [✅ 已完成] tick 机制定案 = **宿主注入**（.at 无 interval 先例核验）：
   renderer `update_shell_clock` 挂 ServiceTick 400ms 帧泵（分钟变化才写
   `__wm_clock` + view_dirty——同分钟复调零写入零 dirty，稳态零重建；
   chrono Local HH:MM；DesktopState.clock_text 去重缓存）。shell.at 两态
   taskbar 右端追加托盘组尾部 `[挂载点容器(空 row)][时钟 text]`（铃铛
   479/齿轮 487 为组内既有成员；挂载点 v1 占位——app 图标注册 API 非目标）。
   T3 `desktop_mcp_clock_injects_and_dedupes`（真资产 assets/shell.at 直载
   ——478 switcher 先例，同时覆盖托盘组/时钟节点 .at 装载面；命名族
   desktop_mcp_* 使计划验证命令字面成立）。验证实况：`cargo test -p
   auto-lang --lib --features ui-iced desktop_mcp` 2/2 绿（iced gate 同况
   见步骤 2 注）。
5. **switcher 消费**：`crates/auto-lang/assets/switcher.at` 行缩略
   （mru_thumbs 平行注入）+ 宿主注入臂（renderer.rs switcher 召唤段）。
   验证：`cargo t desktop_mcp`。
   [✅ 已完成] switcher.at 行渲染 icon→window_thumbnail 升级
   （w-24 h-14 缩略框；fallback_icon=r.icon 兼容旧视觉）+ `mru_thumbs`
   平行列表声明（"1"/""就绪标记合同面）+ RebuildMru rows 增 thumb 字段。
   宿主臂：summon_switcher 注入 mru_thumbs + miss 窗显式预抓入队；
   **抓取编排闭环** = `service_snapshot_requests`（ServiceTick 排空
   request_capture 队列→记 `snapshot_pending_wids`→一次整窗 screenshot
   （411 零尺寸守卫同款））→ `DesktopEvent::SnapshotShot` 回调臂（按各窗
   VWinState.rect×scale_factor 裁剪入缓存 + switcher 可见置 dirty——
   fallback 行下帧升级真缩略）。缺陷修复：invalidate_all 原不清冷却表
   （T3 并行双实例暴露）——失效后清队列+冷却，允许立即重抓。
   T3 `desktop_mcp_switcher_thumbs_injected_and_requested` 绿。验证实况：
   nextest `desktop_mcp` 2/2 ✓（libtest 同进程双实例共享全局队列会串扰
   ——nextest 每测一进程为仓库日常档；clock/switcher 两测同族）+
   `cargo t ui` 777/777 ✓ + t2_snapshot 4/4 ✓。
6. **dock/pager hover 预览**：shell.at 条目 hover popover（422 先例）+
   pager 分区网格（≤4+"+N"）。
   验证：`cargo t desktop_mcp`。
   [✅ 已完成] dock App 条目：mouse-area(onmouseenter/leave → HoverWin/
   HoverEnd) 包 popover(open: .dock_hover==w.wid，widget 锚=条目按钮，
   content=window_thumbnail w-48 h-28；placement top)——两态 taskbar 同步；
   native 条目维持占位（待澄清②）。pager 分区：HoverWs/HoverWsEnd 同型，
   popover content=该区窗口缩略网格（w-28 h-16+标题 truncate）。
   **偏差记录**：分区网格 v1 全量显示，无 ≤4+"+N" 截断——.at 无"过滤后
   截断"原语（for+if 过滤无局部计数器，take 不存在），宿主派生平行列
   表需新投影字段违反"协议零改动"；密度复核归待澄清（原判定已注明可调）。
   **缺陷修复**：untracked convert_element 缺 popover 臂——tracked 兜底
   （taskbar 等无专门臂容器）整体 delegate 到 untracked 实现，popover 落
   fallback 容器直通 children（锚/overlay/open 语义全失，T3 实测暴露）；
   补臂（空 debug 上下文，open 属性驱动形态不依赖自管 slot）。T3
   `desktop_mcp_dock_pager_hover_popovers`（View 树断言：基线 4 枚预构建
   缩略叶 + hover 态 open popover ×1 + End 收起）。验证实况：nextest
   `desktop_mcp` 3/3 ✓ + `cargo t ui` 777/777 ✓ + schema_drift ✓。
7. **I4 对拍**：vue 端 window_thumbnail 占位金样（a2vue 体系挂靠）。
   验证：a2vue/vue 套件绿。
   [✅ 已完成] 金样 `test/a2vue/window_thumbnail/`（input.at 含 wid/
   fallback_icon props + style 类）→ SFC 对拍：`@/wm/WindowThumbnail`
   import（schema/aura.at vue: 行同源映射=I4 登记同源）+ class 透传 +
   v-for/key。**现状注记**：wid/fallback_icon props 未透传到 DOM——
   与 465 virtual_window（win prop 同不透）先例一致的 v1 局限，占位
   组件（icon+边框）不需要动态 wid（待澄清①的真缩略 web 路径一并解）。
   验证实况：a2vue 家族 14/14 ✓ + `cargo t ui` 778/778 ✓。
8. **实机冒烟 + 收尾**：T5 清单留痕；健康检查；状态翻 execution_done。
   验证：`cargo check -p auto-lang && cargo t ui`。
   [✅ 已完成] **T5 实机六项全 PASS**（ui_desktop 真窗口 + CUA 截图/win32
   真鼠标留痕，2026-08-31）：① switcher 召唤见真缩略（Ctrl+Tab → MRU 行
   = Calculator 键盘网格真像素）；② dock 条目 hover 预览（popover 内
   w-48 h-28 真缩略）；③ pager 分区 hover 网格（双窗真缩略+标题）；
   ④ 时钟走字（11:32→11:44→11:51 跨分钟递进）；⑤ 顶/底 dock 两态
   （settings 切 top——时钟/托盘组两态均正确）；⑥ 新鲜度（冷进程 hover
   →fallback→异步抓取→一帧升级真缩略；TTL 2s）。**收尾缺陷修复**：
   SnapshotShot 回调补 shell dirty（hover 面缩略 miss 不升级——实机暴露）
   + invalidate 三点接线（CloseWindow/SetLayout/apply_dock_edges_now，
   G3 的 relayout/关闭失效兑现）。健康检查：新文件 rustfmt 干净+零警告；
   临时 spike demo 已删（结论/留痕在待澄清③+T1 证据行）。事故注记：
   收尾误跑全仓 `cargo fmt -p` 重排 441 文件——已全量 checkout 恢复
   （未 commit 的接线补丁重放，diff 复核 +21 行净增）。验证实况：
   `cargo check -p auto-lang` 0 error + `cargo t ui` 778/778 ✓ +
   desktop_mcp 3/3 + t2_snapshot 4/4 ✓。

## 复审记录

**Reviewer**：zhaopuming（/auto-plan:review 会话，2026-08-31）
**复审基线**：worktree `.worktrees/plan-497-dev` @ merge master 后（f271b35fd +
协议文档复审补丁；master 同步 d1d79d35e 零冲突——496/497 的 shell.at 改动
自动合流，合并后 desktop_mcp 直载真资产 3/3 即验装载完好）。

### 验收标准逐条复验

1. **T2–T4 绿；T5 实机清单 PASS 留痕** — PASS。
   复验命令与结果：`cargo test --lib --features ui-iced t2_snapshot` 4/4；
   nextest `desktop_mcp` 3/3（clock/switcher/dock_pager_hover）；
   `cargo test --lib test_a2vue` 14/14。T5 六项实机截图留痕（执行期
   CUA/win32 真鼠标驱动：switcher 真缩略真像素/dock hover w-48 h-28/pager
   双窗网格/时钟 11:32→11:51 跨分钟/顶底两态/冷缓存 miss→升级链）。
2. **缩略缺失路径全程兜底无空白/panic** — PASS。渲染臂 miss→lucide
   fallback（native "N&lt;slot&gt;" parse 失败天然回退）；crop 零尺寸/越界/
   短 RGBA 守卫回 None（T2-3 覆盖）；缓存条目仅由自洽
   thumbnail_from_screenshot 产出（Handle::from_rgba 尺寸契约无投毒面）。
3. **schema 三件套绿；cargo t ui 不回归；零警告** — PASS。`cargo tf`
   全量 **3316/3316**（含 schema_drift_fence/docs_gen/component_registry +
   1M churn 档）；`cargo t ui` 778/778；新文件（snapshot.rs/两资产/金样）
   rustfmt 干净、零新增警告（仓库既有 200+ 警告非本计划引入）。
4. **投影协议零改动；非目标未夹带** — PASS（一处 review 补丁）。协议
   文档 diff 零改动核实；快照数据不经投影（mru_thumbs 平行列表 +
   像素资产渲染侧直取）。**review 发现**：`__wm_clock` 新注入面未入字段
   表（合同面缺口）——已按 496 §2.1 先例补协议文档行 + 变更记录小节
   （"v1.4 内字段扩展（497）"，非门控字段语义注明；schema_drift 复绿）。
   非目标核实：DWM 缩略/托盘注册 API/S8 IME/后台定时刷新均未夹带。

### 遗漏 / 延后 / workaround 猎查

- **遗漏**：上述 `__wm_clock` 协议文档缺口（已当场补齐，不复存在）。
  其余无——8 步均有对应 diff 落点，无空壳步骤。
- **延后（记债务候选，非 blocker）**：
  - **P497-1** pager 网格 ≤4 截断未实现（v1 全量显示）——计划文本写
    ≤4+"+N"，实现偏差理由：.at 无过滤后截断原语（for+if 无局部计数器、
    无 take），宿主派生平行列表面临新投影字段违反零改动。已在步骤 6
    证据 + 待澄清区如实记录。截断需 `.at` take 原语语言增强，另行计划。
  - **P497-2** a2vue window_thumbnail props（wid/fallback_icon）不透传
    DOM——与 465 virtual_window 先例一致的 v1 局限；占位组件（icon+
    边框）不需要动态 wid；待澄清①的真缩略 web 路径一并解决。
- **workaround**：无。untracked popover 臂补齐是 D-GAP 镜像纪律的架构
  修复（空 debug 上下文形态与 open 属性驱动语义自洽）；SnapshotShot
  单批 pending 模型（一次整窗截图服务一批请求）为设计而非权宜。
- **执行期事故复核**：全仓 `cargo fmt -p` 误重排 441 文件——已全量
  checkout 恢复 + 接线补丁重放；本复审 diff 逐文件核对（19 文件
  +1117/−31，无 fmt 噪音混入）。

### master 侧既有红（非 497，供知会）

`cargo tt`（test-trans 档）a2r_tests 家族失败——**master 同样红**（
d1d79d35e 上 `test_02_types_004_pointer` 断言失败复现，输出含
`.clone()` 差异），属并行会话的 a2r 转译回归，与 497 diff 无交集
（497 不动 transpiler 路径）。建议在对应 plan/P-053 侧跟进。

### 结论

**全部验收 PASS，无 blocker 债务** → `status: reviewed`。

## 待澄清事项

- **① vue 端缩略形态**：v1 占位渲染（icon+边框）。真缩略的 web 端路径
  （transform 缩放的复制子树，同源 store 驱动）为后续增强——I4 要求的是
  登记同源，不要求本期行为对齐。
- **② native docked 窗口缩略**：DWM 缩略（`DwmRegisterThumbnail` 目标=桌面
  HWND、rect=预览框）技术上适配 dock hover 预览，但与 494 真洞透明模式的
  相互作用未验证——native 条目 hover v1 维持 icon 占位，待 494 合入后的
  实机反馈再定。
- **③ 快照路径（T1 回写）**：**定案 = 裁剪式整窗快照（候选 C）**。
  候选 A headless 复用 ❌——`ui/headless/` 为 no-op 渲染器（无窗口/GPU/
  事件循环，仅 View→VTree 转换），产出像素需自写软光栅器，成本不可接受。
  候选 B overlay 离屏 target ❌——iced 0.14 无公开"渲染单 Element 子树到
  buffer"的 API（compositor 不对应用层暴露；`window::screenshot` 是唯一
  公开栅格化通道且为整窗级），成本 = 侵入 iced runtime。
  候选 C ✅——`iced::window::screenshot(id)` 返回整窗 RGBA **物理像素 +
  `scale_factor`**（iced 官方文档注记即支持"widget bounds (logical) →
  crop screenshots"），按 `VWinState.rect × scale_factor` 裁剪窗口区 →
  box 降采样（长边 ≤256）。复用 Plan 285 已验证通道 + 411 零尺寸守卫；
  性能：整窗 RGBA 拷贝+裁剪+降采样为 ~ms 级 CPU 操作，召唤式 + TTL 2s
  缓存可接受。spike 实测（HiDPI scale_factor=2）裁剪数学/降采样/色相
  断言全绿。退路（MCP 截图管道）不再需要。
- **时钟 tick 机制**：.at 本地 interval vs 宿主定时注入，执行期按 shell.at
  既有定时先例（若有）定；无先例则宿主注入（60s 低频无投影压力）。
  **（执行定案：宿主注入——ServiceTick 400ms 帧泵检查、分钟变化才写，
  步骤 4；.at 无 interval 先例核验属实）**
- **pager 网格密度**：≤4 截断为 v1 判定，实机可视性复核后可调。
  **（执行偏差：v1 全量显示无截断——.at 无过滤后截断原语、宿主派生
  违反协议零改动；见步骤 6 证据行；截断需 .at take 原语增强，另行计划）**
