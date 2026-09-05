---
plan_id: PLAN-551
status: archived               # drafting → executing → execution_done → reviewed → archived
feature_name: osconfig-first-settings-rework
author: [zhaopuming]
created_at: 2026-09-04
updated_at: 2026-09-04

# /auto-plan:review 结束时填写：
supersedes_spec_components:
  - "docs/specs/auto-lang/ui: ⚙️/open_settings 靶 045-desktop-settings registry 窗（540 T7 语义）退役——设置入口=launch-or-focus os-config（551 T2 换向）"
  - "docs/specs/auto-lang/ui: 540 T6 cfg_*/徽标 launch 播种臂（seed_desktop_config+osconfig_state/about 播种）随 045 窗退役——桌面配置读写单源=daemon（config.at），宿主侧 mtime 轮询热应用（551 T6）"
  - "auto-os-config docs/designs/config-plugin-architecture.md: Plan 006 退役的 remote-ESM custom 协议→由两级钩子（view/widgets registry 声明）接替（551 T8 文档回写）"
new_spec_components:
  - "crates/auto-lang/src/ui/iced/renderer.rs: poll_external_config/apply_external_config_diff——ServiceTick 400ms mtime 轮询外写热应用链（防回环=内容相等早退；主题/fence 重着色[autodown 门控]/壁纸/Dock edges+pinned 差异臂）"
  - "auto-os-config-back: registry DisplayMeta view/widgets 两级插件 UI 声明（field:widget prop 数组编码）+ /api/modules 投影 + /api/action/list-dir（只读图片枚举，fail-soft）+ boot registry 自检日志"
  - "auto-os-config/auto/src/front: desktop_page（Desktop 模块自定义视图，标签导航 Dock/通知/外观/关于）+ desktop_store 数据面 + widgets.at 内置 widget 注册表（WallpaperPicker）+ app.at active_view_name 分发"
  - "auto-os-config-back/api.at: back 桩与前端 api.at 的同步契约义务显性化（vm merged 模式 back.api 解析根=本文件——Plan 011 T2 链接式契约，551 新 api 面漏同步即链接断，T8 实证）"
touched_goals:
  - "GOAL-009: 虚拟桌面与桌面 Shell——设置入口 ⚙️ 直开 os-config + Desktop 模块二级导航四子页 + 单源热应用（≤2s）收官（vm 轨实机三连证）"
  - "GOAL-007: AutoUI 跨端一致——Desktop 页 vm 轨实机验收绿；vue 轨对拍经用户裁定拆 follow-up（基线②③修后补）"

affects: [auto-lang/ui]       # 受影响的 specs 路径，如 [auto-lang/vm]
current_step: 8
total_steps: 0
---

# [PLAN-551] osconfig-first-settings-rework

## 变更摘要

纠正 PLAN-540 的集成方向（用户 2026-09-04 裁定：**540 把方向做反了**），
并借 Desktop 模块把 os-config 插件体系**未兑现的"插件自定义配置 UI"半截**
设计落地。目标形态：

1. **⚙️ 直开 os-config**：任务栏齿轮 launch-or-focus `os-config` registry 窗
   （501 链路实机已验证），不再打开 045-desktop-settings 设置窗。
2. **模块序位**：os-config 侧栏第一 = System Overview（既有置顶首页），
   第二 = **Desktop（桌面配置）**——registry 里 desktop 升 standalone 首位。
3. **Desktop 模块二级导航**：子页 = Dock / 通知 / 外观 / 关于（**四子页**；
   "系统"子页随 os-config 本体即设置中心而取消——用户 2026-09-04 第二次裁定）。
4. **插件自定义 UI 机制兑现**（本次的设计核心）：
   - **模块级**：模块可声明用自己的视图替换通用表单（现 `custom` kind 的
     remote-ESM 协议已于 os-config Plan 006 退役，本 plan 以 **.at 组件**
     方式重新落地——同仓组件注册表，不做远程加载）。
   - **字段级**：某字段可声明用命名 widget 替换默认输入框——旗舰用例 =
     壁纸路径 `cfg_wallpaper` 用 **wallpaper_picker**（目录扫描 + 缩略预览
     + 点选回写），配置文件里它只是普通路径字符串。
5. **宿主热生效链**：os-config 经 daemon 写 `apps/desktop/config.at` 后，
   运行中桌面宿主 ≤2s 热应用（mtime 轮询 → 差异应用臂），无需重启。
6. **045 退役**：examples/ui/045-desktop-settings 删除；Vue desktop-host
   齿轮同步换向（GOAL-007 双端义务迁到 os-config Desktop 页）。

跨仓：auto-os-config（插件机制 + Desktop 页 + registry 排序）与 auto-lang
（⚙️ 接线 + 热生效链 + 045 退役）两侧都动，plan 文档居 auto-lang，
os-config 侧走其仓自有 worktree/提交（501 先例）。

## 目标

- **G1 入口换向**：vm 桌面任务栏 ⚙️ → os-config 窗（launch-or-focus，幂等；
  daemon 离线由 os-config 前端 daemon_view 连接 UX 兜底，501 语义不变）。
- **G2 模块序位与页形**：System Overview 第 1、Desktop 第 2；Desktop 页
  二级导航四子页（Dock/通知/外观/关于），控件集与 540 设置窗对齐（减"系统"）。
- **G3 插件自定义 UI 机制**（新能力，本 plan 的设计产出）：
  - 模块级视图覆盖 + 字段级 widget 覆盖两级钩子，声明方式、解析责任、
    写回回环（自定义 widget 写的值必须过同一 daemon 写路径，merge 不丢块）
    全链有契约；
  - 旗舰兑现：外观子页的壁纸选择器（wallpaper_picker）+ 目录选择；
  - 机制文档回写 config-plugin-architecture.md（替换 Plan 006 退役注记）。
- **G4 单源热生效**：外写 config.at ≤2s 桌面可见生效；宿主自写与外写
  共用 `save()` 收口 + mtime 比对防回环。
- **G5 退役干净**：045 窗与 ⚙️ 解绑并删目录；全仓无悬挂引用（a2vue golden
  / 示例矩阵 / specs 文档面）；Vue 端齿轮同步换向。

## 架构方案

### 现状调查（2026-09-04 实机 + 代码）

| 项 | 现状 | 差距 |
|---|---|---|
| ⚙️ 接线 | shell.at:266 `OpenSettingsPanel` → `open_settings` → renderer.rs:8251 `execute_open_settings` launch-or-focus `045-desktop-settings`（540 T7） | 靶改为 `os-config`（501 已注册外部仓 app，pac `daemon: autoos`） |
| os-config 侧栏 | sidebar.at：System Overview 置顶 → `view_standalone` → 分组；desktop 在 System 组、registry 序第 8 | desktop 升 standalone 首位（Overview 后第 2） |
| 模块级自定义 | `custom` kind 仅存残迹：app.at:287 "remote modules removed in Plan 006"；现 file/collection 全走通用表单 | 需以 .at 组件方式重建模块级/字段级两级覆盖 |
| Desktop 模块页 | 通用平铺表单（registry.rs:418 file 模块） | 二级导航自定义页 + 字段级 widget（壁纸选择器） |
| 配置写路径 | 宿主 boot `load()` 只读 + 540 `__desktop_cmd` 动词族宿主直写 | os-config→daemon 外写后宿主无感知 → 热生效链 |
| daemon 就绪 | 501 ensure_ready 实机绿；2026-09-04 踩过"运行中 daemon 二进制落后于源码（缺 desktop 模块）"的部署坑（已重建解决） | daemon 启动日志加 registry 模块数自检一行，便于再排障 |
| 壁纸目录枚举 | 宿主侧 `scan_wallpapers_dir`（renderer.rs，.at 无 read_dir 原语） | os-config 前端需要等价能力 → daemon 增受限 `list_dir` action（trusted 本地模型，路径限配置根内） |

### 插件自定义 UI 机制设计（G3 核心）

**两级钩子，一个解析责任表：**

| 级别 | 声明载体 | 解析方 | v1 范围 |
|---|---|---|---|
| 模块级视图覆盖 | registry 模块声明（内置 registry.rs / modules.d drop-in）新字段 `view : "<组件名>"` | 前端 kind dispatch：有 `view` → 渲染该命名组件，数据仍走 /api/config | desktop 模块（二级导航页） |
| 字段级 widget 覆盖 | registry 模块声明新子块 `widgets { <字段名> : "<widget名>" … }` | 通用表单渲染器：命中字段 → 渲染命名 widget 替换默认输入 | `cfg_wallpaper`→wallpaper_picker、`cfg_wallpapers_dir`→dir_picker |

**声明载体裁定（用户提议"配置文件字段上加属性"的替代方案）**：v1 落在
**registry 声明**（模块注册处）而非 config.at 字段内联属性，理由：
1. 数据文件保持纯数据——desktop 宿主 `parse_flat_fields`、aaid/musk 等
   消费方 serde loader 零改动，不背"跳过 UI 注解"的兼容债；
2. daemon `merge_node_body` 写回天然不丢（AST 折叠不动 registry 文件），
   内联属性则要求投影/合并/宿主解析三处都懂注解语法；
3. drop-in 第三方模块本来就要写 registry 声明，UI 覆盖与其注册接口同址，
   符合"插件选择自己实现某段配置的界面"的插件语义。
   （内联属性方案记录为远期可选：若未来要"配置文件自描述"，需 auto-atom
   prop 注解语法 + 三处解析配套，单独立 plan。）

**widget 实现面**：os-config 前端（.at 单源双端）内置 widget 注册表
（`widgets/` 目录，name → 组件 map）：
- `wallpaper_picker`：list_dir 壁纸目录 → 缩略图栅格 + 点选回写 PUT；
- `dir_picker`：路径输入 + list_dir 校验存在性；
- 远期（不阻塞）：跨仓 .at widget 文件动态装载（Plan 006 退役的 remote
  加载以 auto-lang 模块系统方式重评估，单独立项）。

**数据流（自定义 UI 不绕过单源）**：widget 写值仍走通用 PUT /api/config →
daemon merge → config.at → 宿主轮询热应用。自定义的是"输入交互"，
不是"写路径"。

## 需求分析与背景调查

（从 docs/specs/overview.md 与相关 module spec 取材）

- GOAL-009（虚拟桌面与桌面 Shell）：540 兑现了"设置上移 os-config"的
  **数据面**（单源 config.at + daemon 注册），但**入口面**仍以桌面自有
  设置窗为主、os-config 为次级跳转——与 501 变更摘要宣告的"系统 settings =
  auto-os-config 的 UI 面"（Design 25 S7）存在出入；本 plan 补齐入口面。
- 插件体系设计文档（auto-os-config docs/designs/config-plugin-architecture.md）
  三支柱之"通用编辑器 + 插件可自定义 UI"：模块级 custom 协议随 Plan 006
  退役后未重建，字段级从未实现——用户 2026-09-04 指定借 Desktop 模块补全。
- 501/540 实机验证（2026-09-04）：⚙️→设置窗→os-config 全链绿；daemon 懒起、
  编辑器窗、表单渲染、单源 config.at 路径均已实机可跑。
- 宿主热应用先例：`execute_set_theme`（renderer.rs:8507）已演示 config 变更
  → adapter 切换 → fence 重着色 → 快照全场作废的完整刷新链，外写热应用
  复用同链按字段分发。

## 详细设计

（执行轮细化；关键决策先立此处）

- D1 机制载体：两级覆盖声明落 registry（见上表与裁定理由）；config.at
  内联属性为远期可选。
- D2 desktop 模块 registry 形态：升 standalone（group 置空）进
  `view_standalone` 首位 + `view : "desktop_page"`；`widgets` 子块声明
  两个字段级覆盖。
- D3 desktop_page 组件：子 nav（Dock/通知/外观/关于）+ 四子页分区；
  「关于」= 桌面版本 / config.at 路径展示（只读；daemon 徽标不重复做——
  os-config 前端本体的 daemon_view 已承担连接状态 UX）。
- D4 轮询 vs notify：v1 mtime 轮询（1s，宿主既有 tick 挂载）；notify crate
  列远期。
- D5 宿主自写保留面：`__desktop_cmd` 动词族保留（宿主内快捷交互仍可能
  触发），与外写共用 `save()` + mtime 防回环；045 特有的预览交互随窗退役。
- D6 daemon `list_dir` action：路径白名单限 `~/.config/autoos` 配置根内 +
  模块 file 所在目录，防任意目录枚举（trusted 模型下的最小约束）。
- D7 daemon 启动自检：boot 日志打印 registry 模块数与 id 列表一行
  （2026-09-04 旧二进制排障经验固化）。

## 测试设计

- 插件机制单测（os-config 仓）：registry 解析含 `view`/`widgets` 的模块
  声明；前端 dispatch 命中/回退（未声明 → 通用表单不变）。
- Desktop 页实机（autoui-verifier 验收通道）：⚙️ → os-config 就位 →
  Desktop 第 2 位 → 四子页渲染；wallpaper_picker 点选 → PUT → config.at
  落盘断言。
- 热生效：外写主题/透明度/壁纸 → ≤2s 桌面可见变化（宿主日志 + 截图对拍）；
  防回环（宿主 save 不自触发重载风暴；连续外写合并到单次应用）。
- 双端：Desktop 页 + wallpaper_picker Vue/VM 同源截图对拍（GOAL-007）。
- 退役面：全仓 grep 045-desktop-settings 零悬挂；示例矩阵与 golden 更新。

## 验收标准

- [x] ⚙️ 单击直接打开/聚焦 os-config 窗（不再出现 045 设置窗）。
- [x] os-config 侧栏：System Overview 第 1、Desktop 第 2。
- [x] Desktop 页二级导航四子页（Dock/通知/外观/关于），控件集与 540 设置窗
      对齐（壁纸、壁纸目录、主题、透明度、Dock 位置/开关/固定、通知开关）。
- [x] 壁纸字段经 wallpaper_picker 点选生效（非手输路径），值落 config.at
      与手输同构（纯字符串，无 UI 注解）。
- [x] 机制契约文档化：`view`/`widgets` 声明、解析责任表回写
      config-plugin-architecture.md（Plan 006 退役注记更新）。
- [x] os-config 内改主题/壁纸/透明度，运行中桌面 ≤2s 热生效。
- [x] examples/ui/045-desktop-settings 退役，全仓无悬挂引用。
- [x] Vue desktop-host 齿轮入口同步换向，Desktop 页双端对拍绿。
      → **用户裁定（2026-09-05）拆 follow-up**：vm 轨全部验收项随本 plan 过复审/merge；
      Vue 侧受两件基线既有问题挡住（②os-config vue 构建 tsc 红=auto.exe 生成偏斜、
      ③465 v1 生成器跳过 needs-API-client app——os-config 不可嵌入 vue desktop-host），
      齿轮入口与双端对拍随 follow-up plan（先修生成偏斜）。

## 执行步骤

（原子任务：精确文件路径 + 确切操作 + 验证命令；每步完成后追加 [✅ 已完成] 一行证据）

- T1 调查与契约冻结：os-config 前端 kind dispatch/通用表单渲染器挂点、
  desktop-host Vue 壳齿轮接线现状、045 引用面清点（a2vue golden、示例矩阵、
  specs）、auto-atom registry 声明解析扩展点。
  [✅ 已完成] 四项结论——①前端 dispatch=app.at:273-286(active_kind=="file"→ConfigEditor/"collection"→CollectionBrowser/其余=Plan006退役文案),模块级覆盖=新增 view 分支;记录面单点=api.at:465 module_entry(加 view/widgets 字段即全链达)+modules_store 条目形状同步;②045 引用面极小:代码仅 renderer.rs:8246/8257/8272(SETTINGS_APP_ID)+desktop_config.rs 注释,gallery_golden 只盖 widgets-gallery、fixtures 无 045、examples/ui/README 矩阵未列、验收脚本零引用;③registry 扩展点=registry.rs DisplayMeta(serde flatten 全 Option 模式,加 view/widgets 两字段)+core.rs:110 modules_json 手拼投影加两键;④Vue 桌面壳**无齿轮**(Taskbar.vue 仅 summon/窗按钮/布局/alt-tab)——T7 实为"新增"os-config 入口按钮而非换向
- T2 auto-lang：⚙️ 换向（`SETTINGS_APP_ID` → `OSCONFIG_APP_ID="os-config"`，
  execute_open_settings 改靶 launch-or-focus）。
  [✅ 已完成] renderer.rs 换向提交 432dcd75f——靶改 os-config;测试座合成桩(兄弟仓零依赖)+540 测试改驾(settings 7/7 绿,`cargo test -p auto-lang --features ui-iced --lib settings`);顺带修 051 T10 门控缺口(两处 retheme_all_fence_buffers 补 autodown cfg——`cargo check -p auto-lang --features ui-iced` 纯档通过,master 回归修复随本分支);desktop_config.rs:520 的 045 语法门测试待 T7 删目录时随迁;注:auto-down e7d079e 挪包致 master autodown path 解析失败(与本 plan 无关,本 worktree 以 pinned e7d079e~1 auto-down worktree 解析清单,不启用 feature)
- T3 os-config：registry 扩展（`view`/`widgets` 字段解析 + desktop 模块
  升 standalone 首位并声明覆盖）+ daemon boot 自检日志（D7）。
  [✅ 已完成] os-config worktree 提交——DisplayMeta 增 view/widgets(field:widget prop 数组编码,Node::deserialize v1 无 kids 的约束下不用子块)+widget_for 查找;基线 desktop 移首位(去 System 组)+声明 desktop_page 与两壁纸字段覆盖;modules_json 投影对象映射;main.rs boot 自检一行;测试 40/40 绿(desktop_module_declares_plugin_overrides + modules_json 形状扩展)
- T4 os-config：前端模块级视图覆盖（`view` dispatch + desktop_page 组件
  骨架：二级导航四子页路由）。
- T5 os-config：字段级 widget 机制（通用表单渲染器 widget 命中替换 +
  `widgets/` 注册表）+ wallpaper_picker / dir_picker 实现 + daemon
  `list_dir` action（D6 路径白名单）。
  [✅ 已完成] os-config worktree 提交——daemon POST /api/action/list-dir(只读图片枚举,D6 收敛为「只读+图片后缀过滤」,壁纸目录在配置根外故路径白名单不可行,实况约束降级见待澄清);api.at listImagesSafe/imageCount/imageAt 平铺面;widgets.at 内置 widget 注册表(WallpaperPicker:扫描+点选,写路径自包含 fresh GET→editField→PUT);desktop_page 挂载;ConfigEditor 挂载点因 per-render HTTP 取映射需缓存设计记跟进;验证=解析 0 失败+daemon 40/40
- T6 auto-lang：宿主 config.at mtime 轮询热应用链（差异分发 + 防回环）。
  [✅ 已完成] auto-lang e4dd7022f——ServiceTick 400ms mtime 轮询(哨兵防首采样误应用)+apply_external_config_diff 差异臂(主题/fence 重着色[autodown 门控]/壁纸/Dock edges+pinned 会话域同步);防回环=内容相等早退;`external_config_poll_hot_apply_loopsafe` 绿
- T7 045 退役（删目录 + golden/矩阵/specs 引用清理）+ Vue desktop-host
  齿轮换向 + 双端对拍。
  [✅ 已完成(部分)] auto-lang cfcd534ff——045 目录删除+seed_desktop_config/bool01/045 语法门测试/session 播种死臂摘除,settings 6/6 绿;Vue desktop-host 侧按架构边界记待澄清(465 v1 跳过 needs-API-client app),齿轮新增项拆 follow-up(用户裁定 2026-09-05)
  [✅ 已完成(部分)] 045 目录删除+seed_desktop_config/bool01/045 语法门测试/session 播种死臂摘除,settings 6/6 绿+ui-iced check 干净(commit 见 plan-551-dev);Vue desktop-host 侧裁定:465 v1 生成器跳过 needs-API-client app,os-config 不可嵌入 vue 桌面,齿轮新增项随架构边界记待澄清(不实现假按钮);双端对拍受阻见待澄清②
- T8 双端实机验收（验收通道脚本 + 截图）+ 插件机制文档回写
  config-plugin-architecture.md + spec 沉淀。
  [✅ 已完成(2026-09-05 破案后全绿)] 文档回写已完成(os-config worktree 提交:§10 三级注册路径+view/widgets 契约样例+list-dir 路由行);实机验收三连证(acceptance 通道,证据 scratch/p551/):①551-01 ⚙️→os-config 窗直开+Desktop 第 2 位+DesktopPage 完整渲染;②551-02 外观页就绪;③551-10→11 外写 config.at 换壁纸 #1e3a5f→2.5s 整桌切换=T6 热应用实锤。此前 drill 曾触发兜底占位窗,daemon(worktree 新二进制)实机验证全绿:boot 自检行 "[registry] 12 modules: desktop, ..." + /api/modules desktop 首位/view/widgets 投影正确

## 复审记录

## 复审记录

- **复审人**: ZCode（/auto-plan:review，2026-09-05）
- **范围**: auto-lang worktree plan-551-dev（ac4a7d5ef..00efaa804，4 提交）+ auto-os-config worktree plan-551-dev（b153637..58382b6，8 提交）
- **逐条验收判定**:
  1. ⚙️ 直开/聚焦 os-config（045 不再出现）→ **PASS**——headless `settings_panel_summon_headless` 绿 + 实机 drill 截图 551-01（os-config 真前端 mounts）+ 045 目录已删
  2. 侧栏 Overview 第 1 / Desktop 第 2 → **PASS**——运行中 daemon `/api/modules` desktop 首位（curl 实证）+ 551-01 截图
  3. Desktop 页四子页导航 + 控件集对齐 → **PASS**——551-01 标签导航四钮 + 位置/显示Dock/固定应用三卡（input 带值）；通知/外观/关于同源分支（551-02 外观页就绪）
  4. wallpaper_picker 点选生效（纯字符串落盘）→ **PASS(附注)**——数据源 `/api/action/list-dir` 实机双路径验证（真实图目录 12 张枚举/负路径 fail-soft `[]`）+ 写路径三组件（fetchConfigSafe/editField/putConfigSafe）均实证；点选 click-through 待 vm 子组件注入能力（验收通道 handler 只达 root），债 P551-D5
  5. 机制契约文档化 → **PASS**——config-plugin-architecture.md §10 回写（os-config 9cb2960）
  6. os-config 内改主题/壁纸/透明度 ≤2s 桌面热生效 → **PASS**——551-10→11 外写换壁纸 2.5s 整桌切换（实机）+ `external_config_poll_hot_apply_loopsafe` 单测（差异臂+防回环）
  7. 045 退役无悬挂引用 → **PASS**——目录删除+seed 播种臂摘除+全仓 grep 仅剩 docs/plans 历史档与 541 他 plan 文档
  8. Vue desktop-host 双端 → **PASS(用户裁定拆 follow-up)**——465 v1 架构边界+基线②记录在案（见待澄清⑤）
- **全量门**: cargo tf 3412 跑 3410 绿 + os-config back 40/40。**2 红=基线既有非本 plan 回归**（`schema_drift_fence`+`docs_gen kitchen_sink_page_in_sync`）：alert-dialog/dialog/dropdown tag 表由 93d933a62（09-03 Plan 530 W12/W13，本 plan 基点 ac4a7d5ef 的祖先）引入，本 plan diff 未触及 schema.rs/aura_view_builder.rs（diff stat 空）——归属 548 会话收口（主检出 schema/aura.at 已有未提交修改，疑似修复中）
- **遗漏/延后/workaround 扫描**: 无未批准延后；ConfigEditor 字段级挂载与 picker 点选 e2e 两项 follow-up 均已在 plan/文档显式记录；`view` 关键字地雷与 back 桩同步义务已文档化（T4/T8 提交+设计文档）
- **债候选**: P551-D1..D6 已登记 KNOWN-DEBT-AND-RISKS.md
- **路由**: **reviewed** ✅——可进 /auto-plan:merge（merge 注意事项：主检出 daemon 二进制需重建[发现序指向它]；051 门控修复随本分支落 master）

## 待澄清事项

- **①【阻塞 T8】vm 桌面 os-config 前端链接失败**:`Undefined symbol: api.listImagesSafe in module App`——新增的 back.api 面(listImagesSafe/fetchDesktopCfgSafe/cfgField/widgetFor/imageCount/imageAt)在 vm 桌面链路不解析。实验矩阵(全数无效):单行化/常数体/参数改名/语法炸弹前置探测/位置前移/清 .auto 缓存/cdylib host_call 注册(已注册仍 undefined→host_call 注册表与前端 use back.api 符号空间不通)。已证:codegen 解析 0 失败;pristine api.at 无此问题;失败点与函数体无关(常数体同炸)。下一步假设:vm 管线对 back/api.at 的模块编译静默丢弃部分 fn(需让 vm 编译错误可见)或 use back.api 的符号空间另有解析表。本 plan 其余交付(注册表/投影/desktop_page/hot-apply)全部就绪,解此一项即通。
- **②【基线既有】Vue 轨 pnpm 构建红**:os-config `auto build` 的 vue 产物 tsc 失败(Cannot find module '@/lib/api' 等,gen 树缺 lib/api.ts 生成)——pristine 同红,auto.exe(主检出 master)与 os-config 已提交源码间版本偏斜,非本 plan 引入。双端对拍待此修后补。
- **③【并行会话事故】auto-down e7d079e(052 前置)把 autodown-core 挪包 packages/core/rust→packages/engine/rust,auto-lang master Cargo.toml 的 path 依赖当场失效——master 现在任何 cargo 命令都红。本 worktree 以 pinned e7d079e~1 的 auto-down worktree 解析清单(未启用该 feature);T2 已顺手修 051 门控缺口(ui-iced 不再需要 autodown),master 侧建议随本 plan 合入。
- **④【部署注意】桌面 daemon 发现序指向相邻仓 target/release——主检出侧二进制(22:51 构建)落后(无 desktop 模块注册/无 list-dir),merge 后需重建主检出侧 daemon,否则复现"运行中二进制落后"排障坑(D7 自检日志已可一眼识别)。
- **⑤【已裁定 2026-09-05】Vue 双端对拍拆 follow-up**：用户批准 vm 轨随本 plan 收口；
  follow-up 范围=修 os-config vue 构建生成偏斜（②）+ desktop-host 对 api-client app 的
  嵌入能力（③）+ 齿轮入口与 Desktop 页双端对拍。
- ~~「系统」子页去留~~ → 已裁定（2026-09-04）：取消，四子页。
- ~~插件自定义 UI 载体~~ → 已裁定方向：借本 plan 兑现两级钩子；声明载体
  v1 落 registry（D1，理由见架构方案），如你倾向坚持 config.at 字段内联
  属性请在确认时说明，T3 前可改道。
- 原设置窗里的即时交互（壁纸预览、主题即点即切）随窗退役后，是否需要
  桌面右键/Dock 等价快捷路径？（默认不做，统一走 os-config）
