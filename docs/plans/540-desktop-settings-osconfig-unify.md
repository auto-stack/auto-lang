---
plan_id: PLAN-540
status: drafting               # drafting → executing → execution_done → reviewed → archived
feature_name: desktop-settings-osconfig-unify
author: []
created_at: 2026-09-03
updated_at: 2026-09-04

# /auto-plan:review 结束时填写：
supersedes_spec_components: []
new_spec_components: []
touched_goals: []             # 引用 docs/specs/goals.md 的 GOAL-NNN

affects: []                   # 受影响的 specs 路径，如 [auto-lang/vm]
current_step: 0
total_steps: 0
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

### 关键决策（定案见待澄清→实施时回填）

| # | 决策点 | 倾向 |
|---|---|---|
| D1 | 配置读写路径：进程直读 config.at vs 走 daemon API | **直读同一文件**（同机 trusted；daemon 仅编辑器消费——与体系"运行时代码直接读同一文件"惯例一致：aaid/roles/musk 均直读） |
| D2 | 设置窗口形态：registry App（examples/ui，Vue 双端 parity）vs 特权 .at 窗（进程内 iced） | **registry App**（普通窗行为免费获得：×关闭/拖拽/缩放/z 序；双端一致性受益）。特权形态仅保留引导期兜底 |
| D3 | 设置页实现：手写桌面设置页（读写 config.at）vs iced 复刻通用表单 | **首期手写**（桌面风格页，读写同源文件）；通用表单 iced 复刻列远期 |
| D4 | 旧键迁移 | boot 迁移：config.at 缺席 && 旧键存在 → 搬运写 config.at（一次性，之后 config.at 为准） |
| D5 | overlay 槽退役 | settings_app overlay 槽（487 M4）与齿轮二态翻转退役；齿轮 → 打开设置窗口（已开则聚焦） |

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

（立项评审后细化；骨架：M1 config.at 模块化 → M2 窗口化 → M3 注册 → M4 收尾。
建议 worktree：`git worktree add D:/autostack/.wt/lang-540/auto-lang -b plan-540-dev`。）

## 复审记录

（/auto-plan:review 回填）

## 待澄清事项

1. **D1 直读 vs daemon**：桌面进程直读 config.at（同机 trusted，daemon 仅编辑器用）是否
   为最终形态？若未来跨机远程桌面则需要切 daemon API——接口预留。
2. **D2 registry App 的双端义务**：examples/ui 形态意味着 Vue 端也要能跑设置页（双端 parity
   义务）；若不想背，退特权 .at 窗（进程内 iced only）——需要用户定夺。
3. **config.at schema 形状**：以通用编辑器 file 模块约定（顶层块+叶子）为准，嵌套层级 v1
   只做顶层块——dock.pinned 数组形态与 Collection 模块是否更契合（collection 有 sidecar/CRUD）
   需与 auto-os-config 侧对一次。
4. **迁移触发点**：boot 一次性搬运后旧键是否删除（防回滚双源）——建议保留只读回退一个版本。
5. **与 Plan 530（vm-mobile-paint-crash，他人在办）无依赖**；与 501 os-config 外链的退役衔接
   需要知会 auto-os-config 侧。
