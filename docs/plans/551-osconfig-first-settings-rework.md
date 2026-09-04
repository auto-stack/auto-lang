---
plan_id: PLAN-551
status: drafting               # drafting → executing → execution_done → reviewed → archived
feature_name: osconfig-first-settings-rework
author: [zhaopuming]
created_at: 2026-09-04
updated_at: 2026-09-04

# /auto-plan:review 结束时填写：
supersedes_spec_components: []
new_spec_components: []
touched_goals: []             # 引用 docs/specs/goals.md 的 GOAL-NNN

affects: [auto-lang/ui]       # 受影响的 specs 路径，如 [auto-lang/vm]
current_step: 0
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

- [ ] ⚙️ 单击直接打开/聚焦 os-config 窗（不再出现 045 设置窗）。
- [ ] os-config 侧栏：System Overview 第 1、Desktop 第 2。
- [ ] Desktop 页二级导航四子页（Dock/通知/外观/关于），控件集与 540 设置窗
      对齐（壁纸、壁纸目录、主题、透明度、Dock 位置/开关/固定、通知开关）。
- [ ] 壁纸字段经 wallpaper_picker 点选生效（非手输路径），值落 config.at
      与手输同构（纯字符串，无 UI 注解）。
- [ ] 机制契约文档化：`view`/`widgets` 声明、解析责任表回写
      config-plugin-architecture.md（Plan 006 退役注记更新）。
- [ ] os-config 内改主题/壁纸/透明度，运行中桌面 ≤2s 热生效。
- [ ] examples/ui/045-desktop-settings 退役，全仓无悬挂引用。
- [ ] Vue desktop-host 齿轮入口同步换向，Desktop 页双端对拍绿。

## 执行步骤

（原子任务：精确文件路径 + 确切操作 + 验证命令；每步完成后追加 [✅ 已完成] 一行证据）

- T1 调查与契约冻结：os-config 前端 kind dispatch/通用表单渲染器挂点、
  desktop-host Vue 壳齿轮接线现状、045 引用面清点（a2vue golden、示例矩阵、
  specs）、auto-atom registry 声明解析扩展点。
- T2 auto-lang：⚙️ 换向（`SETTINGS_APP_ID` → `OSCONFIG_APP_ID="os-config"`，
  execute_open_settings 改靶 launch-or-focus）。
- T3 os-config：registry 扩展（`view`/`widgets` 字段解析 + desktop 模块
  升 standalone 首位并声明覆盖）+ daemon boot 自检日志（D7）。
- T4 os-config：前端模块级视图覆盖（`view` dispatch + desktop_page 组件
  骨架：二级导航四子页路由）。
- T5 os-config：字段级 widget 机制（通用表单渲染器 widget 命中替换 +
  `widgets/` 注册表）+ wallpaper_picker / dir_picker 实现 + daemon
  `list_dir` action（D6 路径白名单）。
- T6 auto-lang：宿主 config.at mtime 轮询热应用链（差异分发 + 防回环）。
- T7 045 退役（删目录 + golden/矩阵/specs 引用清理）+ Vue desktop-host
  齿轮换向 + 双端对拍。
- T8 双端实机验收（验收通道脚本 + 截图）+ 插件机制文档回写
  config-plugin-architecture.md + spec 沉淀。

## 复审记录

## 待澄清事项

- ~~「系统」子页去留~~ → 已裁定（2026-09-04）：取消，四子页。
- ~~插件自定义 UI 载体~~ → 已裁定方向：借本 plan 兑现两级钩子；声明载体
  v1 落 registry（D1，理由见架构方案），如你倾向坚持 config.at 字段内联
  属性请在确认时说明，T3 前可改道。
- 原设置窗里的即时交互（壁纸预览、主题即点即切）随窗退役后，是否需要
  桌面右键/Dock 等价快捷路径？（默认不做，统一走 os-config）
