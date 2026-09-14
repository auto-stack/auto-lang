---
plan_id: PLAN-620
status: reviewed                # drafting → executing → execution_done → reviewed → archived
feature_name: autoui-icon-table-and-pointer-primitives
author: [zhaopuming]
created_at: 2026-09-13
updated_at: 2026-09-13

# /auto-plan:review 结束时填写：
supersedes_spec_components: []
new_spec_components: []
touched_goals: []

affects: [auto-lang/ui]
current_step: 8                 # T-01..T-04（617 现场落地）+ T-05..T-08（2026-09-14 收尾批，worktree plan-620-dev @ e31be5a80）
total_steps: 8
---

# [PLAN-620] autoui-icon-table-and-pointer-primitives

> **本计划是「事后立项」**：代码已于 2026-09-13 在 PLAN-617 现场按用户指示实施并验证
> （`plan-617-dev` @ `7c13643ba`），但**当时没有独立计划**——这是 PLAN-617 §9.21 明确
> 记录的流程缺口。本文件把那批**平台级**改动收拢为独立审计线，并把尚未完成的收尾项
> （设计注记、漂移门禁、规范增量、独立复审）显式排出来。

## 变更摘要

三件互相咬合的 AutoUI 平台级修复，起因是 030-video-player 在 Vue/VM 两端观感不一致：

1. **lucide 字形表从「手抄 85 条」改为「官方数据生成的全量表 1401 条」**——
   根治债务 P537-D1（VM 端缺名会**静默渲染成空盒**，两端同名不同形）。
2. **`icon` 尺寸口径两端统一**（显式 `w-*`/`h-*` 类 > `size` prop > 默认 20px）——
   `size` 此前是死参数，且 Vue 端硬编码类会静默盖掉用户显式尺寸。
3. **`progress` 新增 `onseek`（双端可拖拽 seek）**——补上「带坐标的按下/拖动」原语：
   VM 侧新 widget `SeekArea`，Web 侧生成指针三包装。

## 目标

- **G1**：VM/iced 端的图标可得集合与 Web 端**由构造成立地一致**，不再依赖「记得去补表」。
- **G2**：`icon` 的尺寸在两端有**单一、可预测的口径**，且显式尺寸不被静默覆盖。
- **G3**：进度类控件在两端都能「点哪跳哪 + 按住拖动」，且该能力成为**平台原语**
  （不靠示例侧拼装，也不靠 `$event` 进 handler 体这类现状限制绕行）。
- **G4**：把上述三件的口径写进规范/设计注记，并留下防漂移门禁。

## 架构方案

### 2.1 图标数据源（G1）

```
lucide 官方数据（本地已装 lucide-vue-next 的 dist/esm/icons/*.js）
        │  scripts/gen-lucide-table.mjs（离线、幂等）
        ▼
crates/auto-lang/src/ui/iced/lucide_generated.rs   ← 生成文件，勿手改
   static LUCIDE_ICONS: &[(&str, &str)]  (按名升序，24×24 内部 markup)
   fn lookup(name) -> Option<&'static str>  (二分查找)
        │
        ▼
renderer.rs::lucide_svg → lucide_fragment(name)   （全量表 + 遗留别名）
```

**为什么是「生成」而不是「引 crate」**：iced 端此前没有任何 lucide 数据源，而 Web 端
用的是 `lucide-vue-next` 整包。引 Rust 侧 lucide crate 会引入新依赖与其版本漂移；
从本地已安装的包生成静态表则**零新依赖**、可离线重跑，且数据与 Web 端同源。

**遗留别名**：`sidebar`（上游已更名 `panel-left`）、`file-icon`（已并入 `file`）
——这两个名字在 lucide-vue-next 里已不存在，保留别名只为不把既有语料打成空盒。

### 2.2 图标尺寸口径（G2）

优先级 **显式 `w-*`/`h-*` 类 > `size` prop > 默认 20px（= `w-5 h-5`）**，两端同实现：

| 端 | 落点 |
|---|---|
| VM | `aura_view_builder::with_icon_size`（由 `convert_image_or_icon` 调用） |
| Web | `ui_gen/vue.rs` 的 `icon` 臂（显式尺寸类在时**不再**注入默认；`size` 转 px 内联样式） |

判据一律取**已解析出的类集**而不是原始 prop：`icon (…) { style: "…" }` 这种**块式**
样式不在 props 里，只看 props 会重复注入默认类（既有 bug，金样曾把它固化）。

### 2.3 可拖拽进度条（G3）

契约：`progress (value: .v, max: .m, onseek: .H($0))`，**handler 收一个 float =
条内横向比例 0..1**（两端同尺，作者写 `SeekTo(.duration * $0)` 不必知道像素或 max）。

- **Web**：生成器把 `<Progress>` 包进一层 pointer 包装
  （`pointerdown/move/up` + `setPointerCapture`，拖出条外仍跟手）。
- **VM**：新 widget `ui/iced/seek_area.rs`——iced 原生 `mouse_area.on_press` **不带坐标**，
  故由本 widget 自持按下态并在事件现场换算比例；`View::ProgressBar` 增
  `on_seek: Option<PointerMoveHandler<M>>`，aura 层按 mouse-area `onmousemove` 同型装配。
- **门控**：按下即 seek，**按住期间的移动**才继续 seek（悬停不 scrub）；两端同语义。
- **兼容**：未声明 `onseek` 的 `progress` 输出逐字节不变（有测试）。

### 2.4 与既有原语的关系

`PointerArea`（Plan 499，坐标流 + 限频）服务于「画布/图表坐标」；`SeekArea` 服务于
「条内比例」，两者**刻意分开**：把比例语义塞进 PointerArea 会让它的 `coords` 契约变形。
若后续出现第三个「比例型」控件，再考虑抽象。

## 需求分析与背景调查

### 4.1 实勘证据（2026-09-13，`plan-617-dev`）

| # | 事实 | 出处 |
|---|---|---|
| P-1 | VM 图标表是**手写 match**，85 条 | `ui/iced/renderer.rs` `lucide_svg`（改前） |
| P-2 | 表外名字 → `None` → **空盒**，无警告 | 同文件 `AbstractView::Image` 的 lucide miss 分支返回 `container(text(""))` |
| P-3 | 媒体字形**一个都没有**：play / pause / volume-1 / volume-2 / volume-x / volume-off / skip-back / skip-forward / repeat / maximize / gauge / film / captions | 与 `lucide_svg` 逐名核对 |
| P-4 | Web 端**无校验**直接 import lucide-vue-next | `ui_gen/vue.rs` icon 臂 → `kebab_to_pascal` |
| P-5 | 本地 lucide-vue-next v0.312.0 有 **1401** 个图标，数据形如 `createLucideIcon("PlayIcon", [["polygon", {points, key}]])` | `dist/esm/icons/*.js` |
| P-6 | 属性集仅几何（`cx,cy,d,height,points,r,rx,ry,width,x,x1,x2,y,y1,y2`）→ 无需 camelCase 转换 | 生成器实测输出 |
| P-7 | 030 自己的 `pac.at icon: "film"` 不在表内 ⇒ 基线红 `lucide_icon_coverage_manifest_all_hit` 的唯一失败项 | 该测试断言 `["manifest:030-video-player:film"]` |
| P-8 | `size` 在 VM 的 icon 臂**从不被读** | `convert_image_or_icon`（只 `extract_style_with`） |
| P-9 | `size` 在 Web 端被丢弃且**硬编码注入** `w-5 h-5` | `ui_gen/vue.rs` 的 media 默认类表 |
| P-10 | Tailwind `w-N`/`h-N` 在 VM 端映射为 `N×4` px | `ui/style/class.rs` `parse_size_value` |
| P-11 | `progress` 两端只读；`slider` **两端都不可用作拖拽控件**（Web 端注释自认 "value just isn't reactively synced to model state"；VM 端 `aura_view_builder` 零 slider 臂 + `render_support` 标 fallback） | 代码 + 注释 |
| P-12 | 带坐标的双端指针原语**只有** `mouse-area.onmousemove`（`PointerArea`），且**无按键门控**；`mouse_area.on_press` 不带坐标 | `ui/iced/pointer_area.rs`、`renderer.rs` MouseArea 臂 |
| P-13 | 基线红 21 项含 `lucide_icon_coverage_manifest_all_hit`（稳定）+ 2 项并发 flake | PLAN-617 §9.14/§9.16 归因 |

### 4.2 被测试钉住的既有语义（不可默默改）

- 未声明 `onseek` 的 `progress`、未声明受控 prop 的 `video`：输出**逐字节不变**。
- 未声明尺寸类且无 `size` 的 `icon`：保持既有默认（20px / `w-5 h-5`）。
- `View<M>` 是跨后端协议类型：加字段必须同时处理所有构造/映射点
  （含 `map_msg_with_arc`、`convert_view_messages`、`vnode_converter` 投影）。

## 详细设计

### 5.1 文件清单

```
scripts/gen-lucide-table.mjs                         新：生成器（离线、幂等，247 行）
crates/auto-lang/src/ui/iced/lucide_generated.rs     新：生成产物（1401 条，254 KiB，勿手改）
crates/auto-lang/src/ui/iced/seek_area.rs            新：条内比例指针 widget（223 行）
crates/auto-lang/src/ui/iced/mod.rs                  改：注册 lucide_generated / seek_area
crates/auto-lang/src/ui/iced/renderer.rs             改：lucide_svg 改查全量表 + 别名；progress 臂接 on_seek
crates/auto-lang/src/ui/aura_view_builder.rs         改：with_icon_size；progress_seek_arm；convert_progress 收 events
crates/auto-lang/src/ui/view.rs                      改：ProgressBar 增 on_seek
crates/auto-lang/src/ui/vnode_converter.rs           改：构造点补字段
crates/auto-lang/src/ui_gen/vue.rs                   改：icon 尺寸口径；可拖拽 progress 生成臂 + 脚本块
schema/aura.at                                       改：progress 增 onseek（msg_ref）
docs/components/core.md                              改：docs_gen 重生成
crates/auto-lang/test/a2vue/desktop_surface_asset/expected.vue  改：金样重生成（重复类修正）
```

### 5.2 关键实现约束（踩过的坑，写下来）

1. **生成器必须离线可跑**：数据源是「本地已安装的 lucide-vue-next」，`--src` 可显式
   指定；生成产物头部记录源版本。
2. **`icon` 尺寸判定取已解析类集**，不取 props（块式 `style` 不在 props 里）。
3. **`ProgressBar.on_seek` 复用 `PointerMoveHandler`**（mouse-area onmousemove 同型），
   让消息装配/投影/转换链路零新增分支；只用到第一个实参（比例）。
4. **`SeekArea` 自己记按下态**：iced 的 `CursorMoved` 不携带按键状态，这是
   「悬停不 scrub」的唯一判据；拖动用绝对坐标（`position()`）而非 `position_in()`，
   否则拖出条外会断流。
5. **Web 侧判据取 `e.currentTarget`**：三包装里 `pointermove` 的 target 可能是子元素。

## 测试设计

| 层 | 手段 | 覆盖 |
|---|---|---|
| 生成器自检 | `lucide_generated::tests`（表有序/非空/代表性命名命中/miss 为 None） | G1 |
| 图标覆盖门禁 | `lucide_icon_coverage_manifest_all_hit`（原基线红 → 应转绿） | G1 |
| 生成器单测 | `test_icon_size_precedence`（三档优先级） | G2 |
| 金样 | `test_a2vue_desktop_surface_asset`（块式 style 与默认类不得重复） | G2 回归 |
| 生成器单测 | `test_progress_onseek_wrapper`（三包装 + 悬停门控 + 未声明时零改动） | G3 |
| VM 视图层 | `progress_onseek_builds_seek_handler`（`on_seek: Some` + value/max 归一 0..1） | G3 |
| 场景（Web） | 030 `tests/smoke.spec.ts` T4（点选）/ T4b（20%→80% 拖拽） | G3 端到端 |
| 场景（VM） | `tests/vm-smoke.mjs`（起窗 + 视图树 + 无 mock 残留） | G1/G3 冒烟 |
| 全局回归 | `cargo t`（与基线 19 红逐项对拍） | 全部 |

## 验收标准

- **AC-01 图标可得性与 Web 端一致**：VM 全量表覆盖 lucide 官方集合（当前 pinned 版本
  1401 个），未知名返回 None（空盒）只发生在**真正不存在**的名字上。
  验证：`lucide_generated` 自检 + `lucide_icon_coverage_manifest_all_hit` 绿。
  **证据**：该测试 2026-09-13 由红转绿（失败项 `manifest:030-video-player:film` 消失）。
- **AC-02 尺寸口径单一**：显式尺寸类 > `size` > 默认 20px，两端一致，显式类不被覆盖。
  验证：`test_icon_size_precedence` + 金样对拍。
- **AC-03 进度条可拖拽（双端）**：点选与「按住拖动」都落到元素 `currentTime`；
  悬停不触发 seek。验证：030 T4（点 50% → `duration*0.5 ±5%`）、T4b（20%→80%）。
  **证据**：2026-09-13 e2e 11/11 绿（含新增两项）。
- **AC-04 兼容**：未声明 `onseek` 的 `progress`、未声明受控 prop 的 `video` 输出
  逐字节不变。验证：`test_progress_onseek_wrapper` 的反向断言 + 生成产物 diff。
- **AC-05 零新增回归**：`cargo t` 相对基线零新增红。**证据**：19 failed = 基线数
  （其中 `external_config_poll_hot_apply_loopsafe` 为在案并发 flake，单跑必过）。
- **AC-06 口径成文**（**未达成**）：`docs/design/autoui/` 增一篇「图标数据源与双端
  一致性口径」注记，并在 `docs/design/00-intro.md` 登记。→ T-06
- **AC-07 防漂移门禁**（**未达成**）：生成表与 Web 端包版本之间的一致性检查
  （表头版本 ≠ 本地包版本时报警，或 CI 重跑生成器并对拍）。→ T-07

## 执行步骤

- [x] **T-01 全量字形表**：写 `scripts/gen-lucide-table.mjs`，生成
  `ui/iced/lucide_generated.rs`（1401 条 / 254 KiB），`lucide_svg` 改为查表 + 遗留别名，
  删手抄 match。验证：生成器自检 + `lucide_icon_coverage_manifest_all_hit` 转绿。
- [x] **T-02 图标尺寸口径**：VM `with_icon_size` + Web icon 臂改造（判定取已解析类集），
  加 `test_icon_size_precedence`；重生成金样（修既有重复类）。
- [x] **T-03 可拖拽进度条**：新 `seek_area.rs`；`View::ProgressBar.on_seek`；
  aura `progress_seek_arm`；Web 生成臂 `try_generate_seekable_progress_html` +
  `progress_seek_script_block`；schema 加 `onseek`；重生成 `docs/components/core.md`。
- [x] **T-04 示例与端到端验证**：030 改用真图标 + `progress(onseek:)`；e2e 增 T4/T4b；
  VM 冒烟与截图。验证：11/11 绿 + VM 截图 `tests/screenshots/after_icons_vm.png`。
- [x] **T-05 规范增量（SD）**：在 `docs/specs/auto-lang/ui/overview.md` 落三条：
  ① `icon` 的尺寸口径与「名字即字形」语义；② `progress.onseek` 契约（比例 0..1、
  按下/拖动语义、未声明时不变）；③ 图标数据源与生成器（含重跑方式）。
  现状：只把「P537-D1 已根治」一句回写进了 overview，**契约条目未补**。
- [x] **T-06 设计注记**：新增 `docs/design/autoui/icon-data-source-and-parity.md`
  （数据源、生成器、双端一致性口径、为何不引 crate、遗留别名），登记进 `00-intro.md`。
- [x] **T-07 漂移门禁**：给生成器加「表头版本 ↔ 本地包版本」一致性检查，
  并在 `cargo t` 里加一条对拍或告警。
- [x] **T-08 独立复审**：按 `/auto-plan:review` 逐条对拍 AC（含本文件勾选的复核）。

## 分支与提交归属（**需要处理**）

- 本计划的代码目前**搭在 `plan-617-dev` 上**（`7c13643ba`），与 PLAN-617 的示例改动同分支。
- 边界：**平台部分** = `crates/**`、`schema/aura.at`、`docs/components/core.md`、
  `scripts/gen-lucide-table.mjs`；**示例部分** = `examples/ui/030-video-player/**`。
  两者通过 030 的 `.at` 互相依赖（示例用新图标/新 `progress`），但方向是**单向的**
  ——平台部分不依赖示例。
- 可选处置（复审时定）：
  - **(a) 现状合入**：随 PLAN-617 折入 master，本计划只作审计线（最省事，归属略混）。
  - **(b) 拆分支**：在 master 上开 `plan-620-dev` worktree，把平台部分按上面的边界
    重组为独立提交（示例部分留 617）；代价是要重写 `7c13643ba` 的拆分。
  推荐 **(a)**：耦合点在「示例消费新原语」，拆开会让两个分支都无法单独构建验证。

## 复审记录



## 待澄清事项

1. **是否要为「比例型指针原语」抽象**：现在 `SeekArea` 只服务 `progress`。
   若后续 `slider` 也走这条（或做独立 slider 原语），是否合并成一个 `fraction_area`？
   倾向**先不抽象**——第三个消费者出现时再合并。
2. **`slider` 的去向**：本计划没动 `slider`（两端都不可用作拖拽控件，见 P-11）。
   是把它实现掉，还是从 schema 标注「不推荐」并引导到 `progress.onseek`？
3. **图标表体量**：254 KiB 静态表 + 1401 条目是否接受长期留在仓库？
   （替代方案：只保留使用面 + 生成时按需裁剪。倾向保留全量——那是「两端一致」的保证。）
4. **生成器的数据源**：目前依赖本地已装包。是否要在 CI 里固定 lucide 版本
   （例如把必要字段抽成仓库内 JSON）以彻底离线可复现？

## 复审记录（T-08，2026-09-14）

- **独立性说明**：落地代码（T-01..T-04）由 PLAN-617 会话实现，本复审会话非实现方，
  从工件重建裁定；收尾批（T-05..T-07）由本会话实施，已声明该限制并逐条以测试
  与产物实证。
- **分支归属裁定**：采纳本文件推荐方案 (a)「现状合入」——代码已随 PLAN-617 折入
  master（`7c13643ba` ∈ master 祖先），示例与平台经 030 的 `.at` 单向耦合，拆分
  会使两分支均无法独立构建验证。本收尾批（T-05..T-07）在独立 worktree
  `plan-620-dev`（@ `e31be5a80`）完成。
- **AC 逐条对拍**（master 复跑，worktree e31be5a80）：

  | AC | 结果 | 证据 |
  |---|---|---|
  | AC-01 图标可得性一致 | **pass** | `lucide_icon_coverage_manifest_all_hit` PASS（基线红→绿的证据在案 2026-09-13）；`lucide_generated` 自检 9/9（表有序/非空/代表名命中/miss=None） |
  | AC-02 尺寸口径单一 | **pass** | `test_icon_size_precedence` PASS；双端口径成文于 617/619 段与 ⑤ |
  | AC-03 进度条可拖拽 | **pass** | `progress_onseek_builds_seek_handler` + `test_progress_onseek_wrapper` PASS；030 e2e T4/T4b 11/11 现场证据在案（2026-09-13） |
  | AC-04 兼容（未声明逐字节不变） | **pass** | `test_progress_onseek_wrapper` 反向断言 PASS；产物再生 diff 纯增量（表体逐字节不变）实证 |
  | AC-05 零新增回归 | **pass** | PLAN-621 复审档全量对拍在案（22 红 + tv 3694/3696 全部 base 复现）；本批 delta=文档+生成器+产物+测试，scoped 全绿（lucide 9/9） |
  | AC-06 口径成文 | **pass**（本次达成） | `docs/design/autoui/icon-data-source-and-parity.md` 落地并登记 autoui/README；overview ⑦（onseek 契约）/⑧（数据源与生成器）补齐，尺寸口径引用既有段不重复 |
  | AC-07 防漂移门禁 | **pass**（本次达成） | `LUCIDE_SOURCE_VERSION` 常量 + `source_version_matches_installed_package` 三态验证（正路径真比较/伪造 9.9.9 即红指路再生/缺包与 unknown 跳过）；生成器 `--src` 向上探测 package.json 版本 |

- **遗留与移交**：待澄清①（比例型原语抽象）按「第三消费者出现再合并」维持不抽；
  ②（slider 去向）未动，维持现状待后续立项；③（254KiB 全量表）裁定保留（设计
  注记 §4）；④（CI 固定版本）由漂移门禁 + PLAN-621 补位 playbook（锁版本
  `@iconify-json/*`）共同覆盖，CI 化待 CI 面立项。
- `stage: review | plan_id: PLAN-620 | plan_revision: 1 | outcome: pass |
  reviewed_commit: e31be5a80（plan-620-dev） | next: merge+archive`
