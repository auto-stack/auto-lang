---
plan_id: PLAN-578
status: drafting               # drafting → executing → execution_done → reviewed → archived
feature_name: desktop-gallery-apps
author: [ZCode, zhaopuming]
created_at: 2026-09-07
updated_at: 2026-09-07

# /auto-plan:review 结束时填写：
supersedes_spec_components: []
new_spec_components: []
touched_goals: []             # 引用 docs/specs/goals.md 的 GOAL-NNN

affects: [auto-lang/ui, auto-man]   # 受影响的 specs 路径，如 [auto-lang/vm]
current_step: 0
total_steps: 6
---

# [PLAN-578] 桌面画廊上架——ui-gallery / widgets-gallery 注册进虚拟桌面缺省应用面

## 变更摘要

把 `examples/ui-gallery` 与 `examples/widgets-gallery` 两个画廊应用注册进
虚拟桌面的**缺省**应用注册表扫描面。机制沿用 Plan 501 多根聚合的相邻仓
探测惯用法：双轨宿主的 extra 根缺省探测各扩展两个**仓内**画廊根（VM 轨
`apps_dir.parent()/{ui-gallery,widgets-gallery}`；Vue 轨
`<workspace>/examples/{ui-gallery,widgets-gallery}`），外部自含根按
PLAN-552 语义缺省 `desktop_visible=true`（opt-out），无需 pac `desktop:`
字段即入 launcher/图标格/dock 消费面。两 pac.at 补 `title:`/`icon:`
展示字段。VM 轨新增 `shell.apps.scan_galleries` storage 开关（缺省开，
命名对齐既有 `shell.apps.scan_siblings`）。

不动：examples/ui 主根扫描与策展恰等断言（20 id 口径）、Plan 552 boot
两分逻辑、`extra_roots_from`/`desktop_extra_app_roots` 既有签名与 env
覆盖语义（`AUTO_DESKTOP_APPS_EXTRA` 设置时整体替换缺省，保持不变）。

## 目标

1. **ui-gallery 上架**（用户裁定：替代 Plan 552 从桌面清退的十多个教学
   demo——demo 探索职能已收拢进画廊，画廊本身理应在桌面）：
   - Vue desktop-host 轨（其原生形态，`render: "vue"`）：生成注册表含
     `ui-gallery`，可 launch。
   - VM 轨：注册可见（boot 不过滤 render，Plan 463 T7 裁定）；launch
     兼容性以"不崩桌面"为验收口径（占位页兜底属设计内降级）。
2. **widgets-gallery 上架**（用户裁定：首个独立 UI app + AutoUI 文档与
   展示面）：
   - VM 轨（其原生形态，`render: "vm"`，Plan 411 VM 原生窗口）：注册
     可见、可 launch。
   - Vue 轨：因 pac 声明 `render: "vm"` 被 `Some("vue")` 过滤器排除
     ——设计行为（前端目标声明即过滤，Plan 559 双轨对齐语义），本计划
     不绕过。
3. **展示元数据完备**：两画廊 pac.at 补 `title:`（"UI Gallery" /
   "Widgets Gallery"）与 `icon:`（`image` / `layout-grid`——两者均在
   iced 渲染器 lucide 清单内，renderer.rs:5312-5313）。
4. **可关**：VM 轨 `shell.apps.scan_galleries=false` 可整体关闭画廊
   探测（对称 `shell.apps.scan_siblings`）；Vue 轨沿用
   `AUTO_DESKTOP_APPS_EXTRA` 整体覆盖既有语义，不新增开关。

## 架构方案

不改架构——纯扫描面扩展，复用两套既有机制：

- **多根聚合**（Plan 501 `aggregate_scan`：主根优先按 id 去重 + extra
  根 `scan_app_root` 自含模式）；
- **缺省探测**（相邻仓 `../auto-os-config/auto` 惯用法：目录缺席静默
  跳过，宿主在任意检出/worktree 形态下不失效）。

双轨落点：

| 轨道 | 注册表装配点 | 画廊根来源 | 可见性 |
|---|---|---|---|
| VM（iced ui_desktop） | `renderer.rs` boot `aggregate_scan`（~11274） | `host_extra_roots()` + 新 `gallery_extra_roots(apps_dir)` | 552 两分：策展集含画廊（外部根 opt-out 缺省 true） |
| Vue（desktop-host） | `vue.rs generate_desktop_host`（~3650） | `desktop_extra_app_roots(root_dir)` 缺省臂扩展 | render 过滤：ui-gallery 入、widgets-gallery 排除 |

## 需求分析与背景调查
（从 docs/specs/goals.md GOAL-009/010 与代码实勘取材）

- **现状缺口**：虚拟桌面注册表扫描面 = 主根 `examples/ui` + 缺省相邻仓
  `../auto-os-config/auto`（VM 轨 `host_extra_roots`，Vue 轨
  `desktop_extra_app_roots`，Plan 559 W3 双轨对齐）。两画廊位于
  `examples/ui-gallery`、`examples/widgets-gallery`——主根的**兄弟目录**，
  不在任何扫描面内；master 实测策展集恰为 examples/ui 的 20 个
  `desktop: "true"` app，无画廊条目。Plan 549（画廊建设）/552（策展）
  均未规划画廊上架——整理工作的自然缺口，本轮补上。
- **用户裁定**（2026-09-07）：ui-gallery 替代已下架的十多个教学 demo
  （552 清退的 Tier1/2 教学示例职能已收拢进画廊），理应上架；
  widgets-gallery 是自研的第一个独立 UI app，兼 AutoUI 文档与展示面，
  也有理由上架。
- **PLAN-552 语义适配**：外部自含根 `desktop_visible` 缺省 true
  （opt-out）——画廊以 extra 根形态注册即上架，零 pac 改动（pac
  `desktop:` 显式值仍可覆盖）。策展恰等断言只扫 examples/ui 主根，
  不受影响。
- **Vue 轨无 desktop_visible 过滤**（实勘：vue.rs 零处引用）——其隐式
  过滤即 `ScanOptions.render = Some("vue")`（`generate_desktop_host`
  3639-3642）；widgets-gallery 声明 `render: "vm"` 自然排除。
- **入口探测兼容**（实勘）：两画廊均为 `src/front/app.at`
  （`probe_entry` 兜底形态），`scan_app_root` 直接可用。
- **图标格徽标色**：`badge_color_for` 按 id 哈希 8 色板（Plan 518 G4③），
  零配置面。
- 相关 GOAL：GOAL-009（虚拟桌面与桌面 Shell）、GOAL-010（示例应用轨道）。

## 详细设计

### 1. VM 轨：`crates/auto-lang/src/ui/app_registry.rs`

新公开函数（放 `host_extra_roots` 邻位）：

```rust
/// PLAN-578：仓内画廊根探测（ui-gallery / widgets-gallery）——
/// `apps_dir.parent()` 下按名探测，目录缺席静默跳过；
/// storage `shell.apps.scan_galleries=false` 可关（对称 scan_siblings）。
pub fn gallery_extra_roots(apps_dir: &Path) -> Vec<(String, PathBuf)> {
    if crate::vm::ffi::stdlib::storage_host_read("shell.apps.scan_galleries")
        .as_deref() == Some("false")
    {
        return Vec::new();
    }
    const GALLERIES: [&str; 2] = ["ui-gallery", "widgets-gallery"];
    GALLERIES
        .iter()
        .filter_map(|name| {
            let p = apps_dir.parent()?.join(name);
            p.is_dir().then(|| ((*name).to_string(), p))
        })
        .collect()
}
```

（实现期可微调形态，语义三件套不变：按名探测 `apps_dir` 兄弟目录 /
缺席跳过 / storage 开关。）

### 2. VM 轨接线：`crates/auto-lang/src/ui/iced/renderer.rs`（~11274）

```rust
let mut extra = crate::ui::app_registry::host_extra_roots();
extra.extend(crate::ui::app_registry::gallery_extra_roots(apps_dir));
let full = crate::ui::app_registry::aggregate_scan(apps_dir, &extra, &…);
```

`aggregate_scan` 按 id 去重主根优先，画廊 id 与主根无冲突；boot 计数
eprintln（`{} entries ({} desktop-visible)`）自动反映 +2/+2。

### 3. Vue 轨：`crates/auto-man/src/vue.rs` `desktop_extra_app_roots`（~5507）

缺省臂（非 env 覆盖路径）os-config 探测之后追加：

```rust
for name in ["ui-gallery", "widgets-gallery"] {
    let p = root_dir.join("examples").join(name);
    push_root(p, &mut out);   // 既有闭包：is_dir 才入，id 取目录名
}
```

注意 `push_root` 入的是 `(目录名, 路径)`——id 即 `ui-gallery` /
`widgets-gallery`，与 VM 轨一致。env `AUTO_DESKTOP_APPS_EXTRA` 覆盖
路径**早返回不动**（整体替换语义保持）。`generate_desktop_host` 的
render 过滤随后自然裁决：ui-gallery 入、widgets-gallery 剔除。

### 4. pac 展示字段：两画廊 `pac.at`

- `examples/ui-gallery/pac.at`：增 `title: "UI Gallery"`、`icon: "image"`
- `examples/widgets-gallery/pac.at`：增 `title: "Widgets Gallery"`、
  `icon: "layout-grid"`

条目字段解析链：`title:` → `name:` → 目录名；`icon:` → 回退
`app-window`（`entry_for_dir` 114-123）。

### 5. 已知边界（验收口径，非债）

- VM 轨 launch ui-gallery：前端为 Vue 形态，解释器路径可能降级
  （Plan 463 T7 panic 边界 + 占位页兜底为设计内降级）——验收只要求
  "不崩桌面"；全功能体验走 Vue 轨。
- Vue 轨 widgets-gallery 缺席：render 声明过滤，设计行为。

## 测试设计

全部新增测试紧贴既有形态（app_registry.rs 真实仓库材料门控单测 +
vue.rs 同文件 tests mod）：

1. `gallery_extra_roots_probes_repo_galleries`（app_registry）：以
   `repo_examples_ui()` 为 apps_dir，断言产出恰 `(ui-gallery, …)`、
   `(widgets-gallery, …)` 两根（真实仓材料，与既有
   `scan_examples_ui_finds_at_least_27_apps` 同款门控）。
2. `gallery_extra_roots_scan_galleries_off`：storage 开关注入
   `shell.apps.scan_galleries=false` → 空表。（storage_host_read 注入
   方式参照 extra_roots_from 既有测试 576-592 行形态；若 storage 不可
   注入则把开关判定提为纯函数参数化——实现期定，语义不变。）
3. `scan_gallery_roots_entries_visible`：`scan_app_root` 对两画廊目录
   产出条目——`desktop_visible == true`（外部根 opt-out）、`title` 取
   pac `title:`、`icon` 取 pac `icon:`、`render` 字段原样。
4. `gallery_render_filter_matrix`：widgets-gallery 目录条目在
   `ScanOptions{render: Some("vue")}` 下 None、default（None）下 Some
   ——钉死 Vue 轨排除/VM 轨收录的双轨语义。
5. `desktop_extra_app_roots_default_includes_galleries`（vue.rs tests
   mod）：tmp workspace 造 `examples/ui-gallery` 空目录，断言缺省臂
   含该根；再造 env 覆盖臂断言 galleries 不在（整体替换语义）。
   （env 测试需串行/隔离——参照 vue.rs 既有 env 测试的处理形态。）

策展恰等断言（`scan_examples_ui_curation_set`）不改动：其只扫主根。

## 验收标准

1. VM 桌面 boot：注册表日志相对现状 **+2 entries / +2
   desktop-visible**；launcher/图标格可见 UI Gallery 与 Widgets
   Gallery（图标 `image`/`layout-grid`、标题非裸 id）。
2. VM 桌面实机：launch `widgets-gallery` 可用（VM 原生形态）；launch
   `ui-gallery` 不崩桌面（占位页或可用均可，见 §详细设计 5）。
3. Vue desktop-host 生成：输出含 `extra root: ui-gallery`，注册表含
   ui-gallery 且可 launch；widgets-gallery 不在（render 过滤，设计
   行为）。
4. `shell.apps.scan_galleries=false` 后 VM 轨两画廊消失（单测 +
   可选实机）。
5. 门禁：`cargo check -p auto-lang -p auto-man` 触及文件零新警告；
   `cargo t gallery_extra`、`cargo t scan_gallery`、
   `cargo t desktop_extra` 全绿；`cargo t app_registry` 全绿。

## 执行步骤
（原子任务：精确文件路径 + 确切操作 + 验证命令；每步完成后追加 [✅ 已完成] 一行证据）

- [ ] **T1 pac 展示字段**：`examples/ui-gallery/pac.at` 增
  `title: "UI Gallery"` + `icon: "image"`；`examples/widgets-gallery/pac.at`
  增 `title: "Widgets Gallery"` + `icon: "layout-grid"`。
  验证：`grep -n "title:\|icon:" examples/ui-gallery/pac.at examples/widgets-gallery/pac.at`
- [ ] **T2 app_registry 画廊根探测**：`crates/auto-lang/src/ui/app_registry.rs`
  增 `gallery_extra_roots`（§详细设计 1）+ 测试设计 1/2/3/4 四单测。
  验证：`cargo t gallery_extra && cargo t scan_gallery && cargo t gallery_render_filter`
- [ ] **T3 VM 轨 boot 接线**：`crates/auto-lang/src/ui/iced/renderer.rs`
  ~11274 处 extra 拼接 `gallery_extra_roots(apps_dir)`（§详细设计 2）。
  验证：`cargo check -p auto-lang`（零新警告）
- [ ] **T4 Vue 轨缺省臂扩展**：`crates/auto-man/src/vue.rs`
  `desktop_extra_app_roots` 缺省臂追加两画廊探测（§详细设计 3）+
  测试设计 5 单测。
  验证：`cargo t desktop_extra_app_roots`
- [ ] **T5 局部门禁**：`cargo check -p auto-lang -p auto-man` +
  `cargo t app_registry` + 新增滤串复跑。
  验证：命令全绿、触及文件零新警告
- [ ] **T6 双轨实机冒烟**：VM 轨
  `cargo run -p auto-lang --features ui-iced --example ui_desktop -- --fullscreen`
  ——boot 日志 +2/+2、图标格两画廊可见、launch widgets-gallery 可用、
  launch ui-gallery 不崩；Vue 轨 `examples/desktop-host` 下
  `auto run --desktop`——生成日志含 `extra root: ui-gallery`、可 launch。
  证据：日志摘录 + 截图存 `docs/plans/evidence/578/`（可选用
  autoui-verifier 脚本）。

## 复审记录

## 待澄清事项

1. VM 轨 ui-gallery 解释器路径的实际形态（可用/占位/半可用）以 T6 实机
   结果为准记录——若完全占位且体验不可接受，可选后续给 pac 增按轨可见
   区分（本计划不做，552 语义 desktop_visible 是全局单值）。
2. storage 开关注入测试形态（T2 测试设计 2 的两种落地）实现期定，
   语义不变。
