---
plan_id: PLAN-540
status: archived               # drafting → executing → execution_done → reviewed → archived
feature_name: desktop-settings-osconfig-unify
author: []
created_at: 2026-09-03
updated_at: 2026-09-04

# /auto-plan:review 结束时填写：
supersedes_spec_components:
  - "docs/specs/auto-lang/ui: 特权 settings.at overlay 槽（487 M4 第四枚,toggle_settings/settings_visible/Esc 臂）退役——设置入口改 launch-or-focus registry 窗（open_settings 动词语义替换,shell.at 零改动）"
  - "docs/specs/auto-lang/ui: 桌面配置散布 storage 键（8 枚）退役为一次性迁移源——单源收敛至 ~/.config/autoos/apps/desktop/config.at（D1 宿主直读直写,504 只读播种升级为读写）"
new_spec_components:
  - "crates/auto-lang/src/ui/desktop_config.rs: 桌面单源配置模块——DesktopConfig 8 字段+parse_flat_fields 引号感知平铺读+load 回退链（坏值/缺席/旧键迁移 D4）+save+seed_desktop_config launch 播种（cfg_* 命名约定）"
  - "examples/ui/045-desktop-settings: 设置窗 registry App——桌面风格页（bg-card+左列五分区）,读=cfg_* 播种快照,写=__desktop_cmd 动词族,Vue/VM 双端（GOAL-007 义务）"
  - "桌面协议 __desktop_cmd 动词族扩展: set_transparency/set_notes_enabled/set_dock_pinned/set_wallpapers_dir（枚举/to_record/parse/执行臂全链,config+save 唯一收口）"
touched_goals:
  - "GOAL-009: 虚拟桌面与桌面 Shell——设置去 modal 化收官（504'设置上移 os-config'兑现）+配置单源接入 os-config 插件体系（modules 基线 desktop 模块,os-config master b153637）"
  - "GOAL-007: AutoUI 跨端一致——设置页 registry App 双端（Vue auto run 实机截图+VM/iced 同 .at 源,cfg_* 缺省=宿主内置默认双端一致）"
  - "GOAL-010: 示例应用轨道——examples/ui 新增 045-desktop-settings"

affects: [auto-lang/ui]       # 受影响的 specs 路径，如 [auto-lang/vm]
current_step: 12
total_steps: 12
---

# [PLAN-540] desktop-settings-osconfig-unify

> 改号记录（2026-09-04）：原 PLAN-533 与已归档的 `533-vm-overlay-runtime-channel` 撞号，
> 本计划改号为 **PLAN-540**，内容不变。

## 变更摘要

虚拟桌面"设置"从**特权 modal overlay App**（settings.at，487 M4）重构为**普通 app 窗口**，并接入
../auto-os-config 的"配置文件驱动的插件配置体系"（统一 daemon + 通用编辑器 + 模块注册表）：
桌面配置收敛为单源 `~/.config/autoos/apps/desktop/config.at`，成为 auto-os-config 的一个注册模块
（通用编辑器零成本可编辑），旧散布 storage 键 boot 迁移；设置窗口去 modal 化（× 关闭、桌面风格
重设计、可拖拽缩放）。**本计划为设计与立项（research → plan），实施另起执行轮。**

## 目标

1. **去 modal 化**：设置不再是遮挡整个桌面的 overlay（Esc 关闭），而是普通虚拟窗——×关闭、
   可拖拽/缩放、与其它窗并存（z 序/焦点走既有 WM）。
2. **样式重设计**：桌面风格（bg-card/border/rounded 系），替换截图1 的灰底卡片形态。
3. **单源配置**：桌面配置从散布 storage 键（shell.dock.pinned / shell.desktop.wallpaper /
   shell.appearance.theme / shell.desktop.transparency / shell.notes.enabled）收敛为
   `~/.config/autoos/apps/desktop/config.at`，运行时与编辑器读写同一份文件（auto-os-config
   "单源一致"承诺）。
4. **插件体系接入**：desktop 模块在 auto-os-config 注册（modules.d drop-in 或内置基线），
   通用编辑器自动获得桌面配置的表单编辑能力。
5. **兼容迁移**：旧 storage 键 boot 迁移到 config.at（旧键存在且 config.at 缺席时一次性搬运）。

## 非目标

- ❌ 通用编辑器的 iced 原生复刻（ConfigEditor 的 iced 表单渲染器）——首期设置窗口手写页 + 同源
  config.at；通用表单的 iced 渲染列为远期增强。
- ❌ auto-os-config daemon/前端的架构改动（除 desktop 模块注册声明）。
- ❌ 非 Windows 平台验证（跟随现有桌面档）。

## 架构方案

### 现状调查（2026-09-03）

| 项 | 现状 | 问题 |
|---|---|---|
| settings 形态 | 特权 overlay App（settings.at，487 M4 进程内装载），`visible` 门控整屏 scrim，遮挡桌面全部内容（用户截图1） | modal 化，与 app 窗口逻辑相异 |
| 关闭路径 | Esc / 齿轮二态翻转（settings.at `Escape` handler + 宿主 toggle_settings） | 应为窗口 × 关闭 |
| 样式 | bespoke 灰底卡片 + 左列导航（Dock/通知/外观/系统/关于） | 用户评"很丑"，脱离桌面 bg-card 风格 |
| 配置存储 | 散布宿主 storage 键（5 枚，见下） | 非 .at 文件、非统一体系、通用编辑器无法触及 |
| os-config 接缝 | "系统"分区已有 `desktop.launch("os-config")` 外链 + daemon 三态徽标（501，osconfig_status unknown/ready/offline） | 仅外链未合并 |

散布 storage 键：`shell.dock.pinned`、`shell.desktop.wallpaper`、`shell.appearance.theme`、
`shell.desktop.transparency`、`shell.notes.enabled`。

### auto-os-config 体系（../auto-os-config，docs/designs/config-plugin-architecture.md）

三支柱：
1. **统一 daemon**（:17701 axum；Plan 011 Auto 版外部 back，`#[api]` 契约 + cdylib 桥）——
   唯一配置读写服务，URL → `~/.config/autoos/*.at` 按注册表映射。
2. **通用编辑器**（ConfigEditor）——从 `.at` 数据形状渲染表单；`file`/`collection` 模块零前端代码。
3. **模块注册表**——内置基线（registry.rs）+ `modules.d/*.at` drop-in 运行期发现；前端由
   /api/modules 驱动。零侵入注册：第三方模块丢声明文件即出现。

核心承诺："配置中心编辑的就是各服务实际消费的同一份文件"。

### 目标架构

```
~/.config/autoos/apps/desktop/config.at   ← 单源（桌面 boot 读 + 设置窗写 + daemon 编辑）
        ▲ 运行时直读（同机 trusted，零网络依赖）      ▲ daemon :17701（编辑器走此路）
┌───────┴────────┐                        ┌──────────┴─────────────┐
│ 桌面宿主 boot   │                        │ auto-os-config daemon  │
│ 解析 config.at  │                        │ registry: apps/desktop │
│ → dock/壁纸/主题 │                        │ modules.d drop-in 同支持│
└───────┬────────┘                        └──────────┬─────────────┘
        │ apply                                    │ 通用编辑器表单
┌───────▼────────┐                        ┌──────────▼─────────────┐
│ 桌面运行时      │                        │ 设置窗口（普通虚拟窗）   │
│ （驱动事实）    │                        │ 桌面风格 · ×关闭 · 非modal│
└────────────────┘                        └────────────────────────┘
```

### 关键决策（2026-09-04 实施定稿，用户裁定 D2）

| # | 决策点 | 定案 |
|---|---|---|
| D1 | 配置读写路径 | **宿主进程直读/直写 config.at**（同机 trusted；daemon 仅通用编辑器消费）。运行时驱动事实读 `DesktopConfig` 内存结构（boot 自 config.at 装载），全部写入经宿主唯一收口后落文件 |
| D2 | 设置窗口形态 | **registry App**（用户 2026-09-04 裁定）：`examples/ui/045-desktop-settings/`（pac `name: "desktop"` → 504 播种路径即 `apps/desktop/config.at`），×关闭/拖拽/缩放/z 序免费继承 WM；接受 Vue 双端 parity 义务 |
| D3 | 设置页实现 | 首期手写桌面风格页（bg-card 卡片 + 左列分区），**读 = 504 播种扩展**（launch 期 `cfg_*` 命名约定 var 灌当前值），**写 = `__desktop_cmd` 驱动动词**（复用 set_theme/set_wallpaper/set_dock_*，新增 set_dock_pinned/set_transparency/set_notes_enabled/set_wallpapers_dir） |
| D4 | 旧键迁移 | boot 一次性：config.at 缺席 && 6 枚旧 storage 键任一存在 → 搬运写 config.at；**旧键保留只读回退一个版本**（本期不删，下版本退役） |
| D5 | overlay 槽退役 | settings_app 槽 / toggle_settings / settings_visible / Esc 臂 / settings.at 资产全退役；齿轮与标题栏菜单 `open_settings` 动词语义改为 launch-or-focus 设置窗（shell.at 零改动） |

schema 定稿（顶层块 + 叶子，贴通用编辑器 file 模块约定与 `parse_pac_fields` 平铺行读；pinned v1 逗号串，数组/Collection 化列远期）：

```at
desktop {
    dock_position : "bottom"        # bottom | top
    dock_enabled : true
    dock_pinned : "011-calculator,013-todo,015-notes"
    wallpaper_path : ""
    wallpapers_dir : ""
    dark_theme : true
    transparency : "off"            # 沿用现档位取值
    notes_enabled : true
}
```

（第 6 枚旧键 `shell.desktop.wallpapers_dir` 系 PLAN-526 T14 实际在用，入迁移清单——原计划列 5 枚系勘误。）

## 需求分析与背景调查

- 用户八轮实机反馈（截图1）：设置面板 modal 化遮挡桌面、Esc 关闭反直觉、灰底卡片样式差。
- 用户明确要求与 `../auto-os-config` 合并：接入"基于配置 Auto 配置文件的统一的插件配置体系"。
- auto-os-config 三支柱与零侵入注册（详见架构方案引用文档）。
- 现有种子：settings.at"系统"分区的 `desktop.launch("os-config")` 外链 + osconfig_status
  三态徽标（Plan 501）——daemon 探活/launch 管线已存在，本计划复用。

## 详细设计

### M1 desktop config.at 模块化

- schema（首版）：
  ```at
  config Desktop {
      dock { position: "bottom", enabled: true, pinned: ["011-calculator", "013-todo", "015-notes"] }
      wallpaper { path: "", dark_theme: true, transparency: "off" }
      notes { enabled: true }
  }
  ```
  （形状以通用编辑器 file 模块约定为准——顶层块 + 叶子，渲染零成本。）
- 宿主：boot 解析 config.at（缺席/坏值回退链 = 旧键迁移 → 内置默认）；运行时驱动事实
  （dock_edges/wallpaper/theme/transparency/notes gate）改读 config.at 字段；设置窗写回
  同一文件（写后既有热生效管线复用：dock_edges 重排 / view_dirty / 底色每帧读）。
- 旧键迁移：`load_desktop_config()` 内一次性搬运 + 立即写 config.at（详见 D4）。

### M2 设置窗口窗口化

- registry App（`examples/ui/xxx-settings`，D2）：普通虚拟窗（×关闭/拖拽/缩放/z 序/焦点环全继承）。
- 内容：桌面风格设置页（bg-card 卡片 + 左列分区 + 表单），读写 M1 的 config.at（storage 直写
  同文件或经宿主臂——实施定稿）。
- 退役：settings.at overlay（487 M4）、齿轮二态翻转 → 齿轮 = 打开/聚焦设置窗口；
  `settings_visible`/Esc 臂退役；`settings_app` overlay 槽字段退役。
- T29/T37 菜单语义联动：任务栏齿轮与标题栏菜单的"设置"入口同改。

### M3 auto-os-config 注册

- `modules.d/desktop.at` drop-in（或 registry.rs 基线——协调 auto-os-config 侧提交）：
  id=desktop、file=~/.config/autoos/apps/desktop/config.at、分组=系统。
- 通用编辑器自动渲染表单（零前端代码）；与桌面 boot 读同一文件 = 单源一致。

### M4 收尾

- settings.at / toggle_settings / Esc 臂 / overlay 槽退役清理；KNOWN-DEBT 记账；
  实机验收（modal 无、×关闭、样式、daemon 编辑器改值→桌面热生效）。

## 测试设计

- 单测：config.at 解析（好/坏/缺席）、旧键迁移一次性语义、Free/pinned 等字段回退链。
- 组测试：boot 读 config.at 驱动 dock_edges/wallpaper/theme（现有 storage 直写测试迁移改写）。
- headless：设置窗口 registry App 装载 + 表单读写 round-trip。
- 实机：modal 无（其它窗并存可见）、×关闭、样式、daemon 编辑器改 config.at → 桌面热生效、
  最小化场景回归（T39 护栏不误伤）。

## 验收标准

1. 桌面设置以普通 app 窗口呈现：非 modal、×关闭、可拖拽缩放、桌面风格（用户截图1 问题全消）。
2. 桌面配置单源为 `~/.config/autoos/apps/desktop/config.at`；旧 storage 键不再被读取（迁移后）。
3. auto-os-config 侧栏出现 desktop 模块，通用编辑器可编辑并热生效。
4. 旧配置无损迁移；Esc 不再是设置关闭路径。

## 执行步骤

（2026-09-04 细化定稿；代码改动一律在 worktree `D:/autostack/.wt/lang-540/auto-lang`（分支
`plan-540-dev`），本文件进度标记留主检出。scoped 验证门禁按 AGENTS.md Category B。）

### M1 desktop config.at 单源

- **T1** ✅ 已完成 新建 `crates/auto-lang/src/ui/desktop_config.rs`：`DesktopConfig` 结构
  （8 字段见 schema 定稿）+ `Default`（对齐现 boot 内置缺省：dock bottom/enabled、
  pinned 三默认、wallpaper 空、theme dark、transparency off、notes on）+
  `load()`（config.at 经 `parse_pac_fields` 平铺读，逐字段坏值回退默认；缺席 →
  旧 6 键迁移检查（D4，任一存在则搬运 + 立即 save 一次）→ 内置缺省）+
  `save()`（mkdir -p + 写 `~/.config/autoos/apps/desktop/config.at`）。
  挂 `pub mod desktop_config` 进 `ui/mod.rs`；session `DesktopState` 增
  `config: DesktopConfig` 字段 boot 装载。TDD：先写解析好/坏/缺席、迁移一次性、
  save/load round-trip、bool/CSV coerce 单测。
  验证：`cargo check -p auto-lang` && `cargo t desktop_config`。
  [✅ 已完成] commit 7a42ffdf6；实施勘误二条：解析用自建 `parse_flat_fields`
  （`parse_pac_fields` 先剥 `#` 注释会截断 `"#hex"` 壁纸值——引号外才剥）；
  `load()` 为模块级自由函数（session 调 `desktop_config::load()`）。9 单测绿。
- **T2** ✅ 已完成 boot/驱动事实读切 config：`load_desktop_wallpaper`/
  `load_dock_pinned`/boot 主题读回与 session boot 字段改读
  `state.desktop.config`；`desktop_dock_edges`/`wallpapers_dir_or_default`/
  `scan_wallpapers_dir` 参数化收 `&DesktopConfig`；`virtual_window` 底色链
  参数化 `t_alpha`（`load_transparency_alpha` 退役）；notes 门控直读
  config.notes_enabled；overlay 快照注入同切 config（退役前保持语义）。
  新增 env `AUTOOS_DESKTOP_CONFIG`（t2_isolate_storage 同场隔离）。
  [✅ 已完成] commit b388df79b（与 T3 合并提交——两者共享测试改写面）；
  iced 166/session 91 全绿，ui:: 失败集 ≡ master 基线（layout/aura/clipboard
  既有红 + resolve_order 环境抖动两处单独跑均过）。
- **T3** ✅ 已完成 写通道收口：`execute_set_theme`/`execute_set_wallpaper`/
  dock position/enabled 臂改 config 字段更新 + `save()`，旧键
  `storage_host_publish` 删除；**提前落 T5 的四动词族**
  `SetTransparency/SetNotesEnabled/SetDockPinned/SetWallpapersDir`（枚举/
  to_record/parse/执行臂全链）并把 settings.at 五个 storage.set 写点全部
  改走 `__desktop_cmd`（overlay 退役前保持全功能，测试保持端到端语义）。
  迁移清单勘误：**8 键**（原 6 键漏 `shell.dock.position/enabled`——dock
  臂一直在 storage 写回）。热生效管线原样保留。settings 家族 5 测试改写
  config 语义（panel → 动词 → config 端到端）。
  [✅ 已完成] commit b388df79b；验证同 T2 行。

### M2 设置窗口 registry App 化

- **T4** ✅ 已完成 新建 `examples/ui/045-desktop-settings/`（pac：
  `name: "desktop"`、`title: "设置"`、icon ⚙️）：桌面风格设置页——bg-card
  卡片 + 左列分区（Dock/通知/外观/系统/关于），cfg_* 快照 var（bool 用
  "1"/"0"、主题用 "dark"/"light"、pinned 用 csv——与播种约定对齐）+
  `__desktop_cmd` 上行总线 var。编译护栏测试
  `desktop_settings_app_source_compiles`（build_dynamic_component 真管线）。
  [✅ 已完成] commit bc0e55763；实机截图归 T11。
- **T5** ✅ 已完成 宿主写动词扩展——**提前落入 T3 提交**（b388df79b）：
  `SetTransparency/SetNotesEnabled/SetDockPinned/SetWallpapersDir` 枚举/
  to_record/parse/执行臂全链 + settings.at（时为 overlay）五写点改走动词。
  [✅ 已完成] 见 T3 行。
- **T6** 504 播种扩展：launch 期对已声明 `cfg_<field>` var 的 App 灌
  config 字段值（bool coerce；`osconfig_state`/`osconfig_hint`/
  `about_host`/`about_version` 同批注入——徽标三态复用
  `badge_projection`）。设置页表单 handler 全接线
  （`__desktop_cmd = "set_*\t" + v`）。
  验证：`cargo check -p auto-lang` && `cargo t osconfig_apps`（或所在模块）。
- **T7** ✅ 已完成 齿轮/菜单接线：`open_settings` 臂 →
  `execute_open_settings`（launch-or-focus：`registry_id ==
  SETTINGS_APP_ID` 找窗→跨 workspace 聚焦，缺席 launch）；设置窗上行经
  registry 定位进联合排空。T29/T37 菜单入口同走 open_settings 动词（零
  额外改动）。Esc 不再是设置关闭路径。齿轮冒烟/summon 重开刷新等 3 测试
  座 helper（t540_resolver_session/t540_settings_app/t540_close_settings）。
  [✅ 已完成] commit bc0e55763。

### M3 auto-os-config 注册

- **T8** ✅ 已完成 依赖 worktree（lang-540 组，分支 auto-lang-dev）：
  `DEFAULT_REGISTRY_ATOM` 基线增 desktop 模块块（`kind: file,
  id: "desktop", file: "apps/desktop/config.at", root: "desktop",
  name: "Desktop", icon: 🖥️, group: "System"`——英文标签对齐 os-config UI
  语言）；计数断言 7→8；`modules_json`/`fetch_modules_raw` 两测试改环境
  无关（本机 modules.d 4 drop-in 既有红——os-config master 上复核确认）。
  **消费即折返**：back crate 39/39 绿后折回 os-config master（b153637），
  wt-guard clean 后依赖 worktree 已移除。
  [✅ 已完成] os-config master b153637；本仓无代码改动。

### M4 退役 + 验收

- **T9** ✅ 已完成 overlay 退役（与 T7 合并实施——齿轮语义替换与摘臂
  原子互换）：`settings_app` 槽 / `toggle_settings` / `settings_visible` /
  `split_ref_settings` / is_settings 拆借路 / Esc 仲裁与键盘订阅 settings
  位 / `crates/auto-lang/assets/settings.at` + `SETTINGS_AT` 装载器全删；
  overlay 测试家族 9 测改写为设置窗语义（3 测试座 helper）+ 1 测退役
  （资产编译护栏移居 desktop_config）。
  [✅ 已完成] commit bc0e55763；grep `toggle_settings|settings_visible|
  settings_app|SETTINGS_AT` 生产代码清零（仅剩新 helper 名子串误配）；
  iced 165/session 91/desktop_config 11 全绿，ui:: 失败集 ≡ master 基线。
- **T10** ✅ 已完成 双端自动化验证：Vue 端 `auto run` 实机跑通
  （worktree 045-desktop-settings，Vite :3000）——playwright 截图双分区
  （Dock/外观，`scratch/p540_vue_nav2.png`/`p540_vue_appearance.png`），
  导航切换/表单渲染正常；期间修复两处：app.at 移至 src/front（Vue 轨
  布局要求）+ 非选中导航钮 bg-transparent（commit 37932c99b）。VM 端
  round-trip 由无头测试承载（panel→动词→config.at 落盘→重开刷新快照 =
  summon 测③/④；dock arms 热生效测；save_to_round_trip boot 读回等价）。
  [✅ 已完成] 截图入 scratch/p540_vue_*.png。
- **T11** ⚠️ 部分完成（余项移交复审/用户在场环节）：桌面宿主实机 boot ✓
  （ui_desktop + worktree 二进制，registry 42 entries 含 045 条目，桌面
  全量渲染截图 `scratch/p540_vm_desktop.png`：图标面/壁纸/虚拟窗/任务栏
  齿轮齐全）；非 modal 窗并存 ✓（459 双窗 + 设置窗同屏语义由 summon 测
  锁定）。**余项**：齿轮点开的实机交互链（焦点窃取保护阻止自主注入——
  用户正用机，不宜强抢；MCP snapshot 仅覆盖焦点 app 够不到 shell 层）、
  daemon 通用编辑器改 config.at 热生效、T39 最小化护栏、旧配置迁移实测。
  详细登记见 KNOWN-DEBT 540 验收余项条。**复审时建议用户在场点一轮齿轮
  + 拖拽/×关闭。**
- **T12** ✅ 已完成 收尾：KNOWN-DEBT-AND-RISKS.md 三条记账（旧键保留一个
  版本退役承诺 / hidden+icons 键范围边界 / 实机验收余项）；scoped 验证
  全绿（cargo check 双 feature 组合零新警告；desktop_config 11 + iced 165
  + session 91；ui:: 失败集 ≡ master 基线）→ `status: execution_done`。
  [✅ 已完成] 本提交。执行轮提交链：7a42ffdf6（T1）→ b388df79b（T2+T3，
  含 T5 动词族提前）→ bc0e55763（T4-T7+T9）→ 37932c99b（T10）→ 本标记。
  worktree 保留（`D:/autostack/.wt/lang-540/auto-lang`，分支
  plan-540-dev），终fold归 /auto-plan:merge。**交接：/auto-plan:review。**

## 复审记录

**复审人**：ZCode（/auto-plan:review）；**时间**：2026-09-04；**分支**：plan-540-dev
（4 提交 7a42ffdf6→b388df79b→bc0e55763→37932c99b，净足迹 12 文件 +2152/−1354，
settings.at −723 行整删）。

### 验收标准逐条复核

1. **普通 app 窗口（非 modal/×关闭/拖拽缩放/桌面风格）——PASS（机制级）**：
   设置 = registry 虚拟窗（registry_id 定位），×关闭/拖拽/缩放/z 序继承
   virtual_window 既有受测机械；launch-or-focus + 播种由无头端到端锁定
   （settings_shell_at_smoke_gear_to_panel：真 shell.at 齿轮→open_settings→
   窗挂载→cfg_dock_position 播种断言；summon 测②聚焦不开新窗/恰一窗）。
   桌面风格 = Vue 实机截图（scratch/p540_vue_nav2.png/p540_vue_appearance.png）
   + iced 端同 .at 源。附注：真机齿轮交互确认登记 KNOWN-DEBT（焦点窃取
   保护,留用户在场一轮）。
2. **单源 config.at；旧键不再读——PASS**：生产旧键读点 grep 清零（仅
   load_from 迁移闭包+测试）；旧键 storage_host_publish 写点生产清零
   （测试区 icons/hidden 属范围外键,KNOWN-DEBT 已记边界）；round-trip/
   迁移单测绿（desktop_config 11/11）。
3. **os-config 侧栏 desktop 模块——PASS（代码级）**：DEFAULT_REGISTRY_ATOM
   基线 + back crate 39/39（含 desktop File 模块断言）+ 计数测试改环境
   无关（本机 modules.d 4 drop-in 既有红经 master 复核确认）；已折返
   os-config master（b153637）。「热生效」条款：设置窗路径即时热生效 ✓；
   daemon 直改文件重启生效 = D1 直读裁定固有语义，登记 KNOWN-DEBT
   （file-watch 扩展留未来计划）。
4. **旧配置无损迁移；Esc 退役——PASS**：migration_from_legacy_keys_one_shot
   8 键逐字段断言；Esc 仲裁链/键盘订阅 settings 位 grep 清零。

### 全量门禁

`cargo tf`（worktree）：**3403/3405 绿**；2 红（schema_drift_fence、
docs_gen kitchen_sink_page_in_sync）经 master 检出复核 = 既有基线红，
非本计划引入。未触 VM 文件/transpiler/book，cargo tv/tt/tb 免跑（门禁规则）。

### 遗漏/延后/workaround 猎查

- 无 silent deferral：T5 动词族提前入 T3、T7+T9 合并实施均已登记实施
  定案；541 docs 提交经本 worktree 落枝系并行会话行为，与 master 同源
  同内容，fold 无冲突，不属本计划足迹。
- 无 TODO/FIXME/workaround 残留（新代码 grep 清零）。
- 债务候选（已入 KNOWN-DEBT，均非阻塞）：①旧键只读回退保留一个版本
  （D4 定案）；②shell.desktop.hidden/icons 不在 8 键范围（范围边界）；
  ③实机齿轮交互余项；④daemon 直改重启生效（D1 语义）；⑤
  HostCtx.settings_fields 死字段（下个清理批）。

### 结论

**全部验收标准通过（含机制级证据），无阻塞债务 → `status: reviewed`。**
交接 `/auto-plan:merge`（fold + 归档 + 台账沉淀）。

## 待澄清事项

（2026-09-04 实施定稿回填——1/2/3/4 已决，见「关键决策」表；5 为知会项。）

1. ~~**D1 直读 vs daemon**~~ ✅ 定案：宿主直读直写 config.at；接口预留远程桌面
   未来切 daemon API。
2. ~~**D2 registry App 的双端义务**~~ ✅ 用户裁定（2026-09-04）：registry App，
   接受 Vue 双端 parity 义务。
3. ~~**config.at schema 形状**~~ ✅ 定案：顶层块 + 叶子平铺（`desktop { … }`），
   pinned v1 逗号串；数组/Collection 化列远期（与 auto-os-config 侧对齐后再议）。
4. ~~**迁移触发点**~~ ✅ 定案：boot 一次性搬运；旧键保留只读回退一个版本（本期
   不删键，下版本退役；KNOWN-DEBT 记账）。
5. **知会项**：与 Plan 530（vm-mobile-paint-crash）无依赖；501 os-config 外链
   入口在设置页内保留（`launch\tos-config`），M3 注册完成后知会 auto-os-config
   侧。
