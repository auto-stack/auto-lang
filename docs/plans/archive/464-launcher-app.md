# Plan 464: Launcher App（examples/ui/028-launcher，真注册表 + 真启动）

---
plan_id: PLAN-464
status: archived
author: [zcode]
created_at: 2026-08-28T00:00:00+08:00
updated_at: 2026-08-29T00:00:00+08:00

supersedes_spec_components:
  - "docs/specs/auto-lang/ui/overview.md: 修改——VM 渲染/桌面 shell 现状段：463 预留的 launcher overlay 消费点落地（SummonLauncher 懒挂载/平行字符串列表注入/LaunchApp 上行/__focus_input 聚焦约定）；split_mut 对 windowless 特权 App（shell/launcher）的 update 侧拆借垫片（463 任务栏点击静默丢弃根因修复）；DesktopBus 增 summon 动词"
new_spec_components:
  - "docs/specs/auto-lang/ui/overview.md: 新增组件——examples/ui/028-launcher（palette/grid 双形态启动器；单 App 内聚形态=双端转译实测结论；模糊排序整数权重规则 exact>prefix>词首>子序列+recent 同档折扣=465 对拍规范源，见 SPEC.md）"
  - "docs/specs/auto-lang/ui/overview.md: 新增组件——vue 生成器 bind{} 块全局 keydown 层（__autoBindKeymap，vm keyboard_subscription key_bindings 同源镜像；修复前 bind 块 vue 端静默失效）"
  - "docs/specs/auto-lang/ui/overview.md: 新增测试资产——028 tests/（desktop_mcp.py 24 断言 + vue_verify.mjs 5 组断言，双端验收入口）"
touched_goals:
  - "GOAL-009: 桌面 shell 端到端——launcher overlay 召唤/真注册表模糊搜索/LaunchApp 真启动/最近使用 闭环（M3 收口；463 预留接缝的首个消费方；462 焦点分区消费实测）"

current_step: 7
total_steps: 7
---

> **状态**：✅ reviewed（2026-08-29 /auto-plan:review 独立复审通过，见文末复审记录；
> worktree plan-464-dev 6 commits 待 /auto-plan:merge 折叠）
> **来源**：产品需求「launcher 本身做成独立 UI APP，快捷键查找并启动 examples/ui 应用」
> （Design 24 §1 N3）；**吸收 Plan 441**（028-launcher 原为 mock 注册表 demo，
> 本计划把边界升级为真桌面启动器，处置映射见 §1.1）。
> **架构依据**：Design 23（R1/R8）、Design 24（§2.2 launcher 调研 P1–P6、R10 注册表、
> R11 启动=挂载、R12 召唤热键、I5 三端一份源）；Design 25（§2 S2=launcher 是
> shell 第一表面、§3 命令接缝=`desktop.launch(name)` builtin、I7–I9）。
> **依赖**: 462（overlay 槽/焦点分区）+ 463（`SummonLauncher` 事件与 `LaunchApp` 接缝）；
> **M1 子阶段可先行**（仅依赖 mock 注册表，见任务表）。
> **目录**: `examples/ui/028-launcher/`（编号沿用 441 预订；pac `name:"launcher"`、
> 独立调试端口 `front_port: 4028`）。**基线**: 463 合入后的 master。

## 1. 目标

`examples/ui/028-launcher` 是一个**普通 AutoUI App**（同一份 .at 跑三端，I5），
在桌面 shell 内以 overlay 形态被召唤，提供：

1. **palette 形态**（主）：居中搜索框 + 分组结果列表（最近使用 / 全部应用），
   输入即模糊过滤（P2），键盘流 ↑↓/Enter/Esc（P3）；
2. **网格形态**（辅，对标 Activities/Launchpad，P5）：图标网格，Tab 切换形态；
3. **真注册表**（R10）：消费 463 的 `scan_apps`（.at 侧经接缝读宿主注入的应用清单，
   或独立模式下内嵌 mock 清单——同一状态变量约定，见 §3.2）；
4. **真启动**（R11）：Enter/点击 → `DesktopCommand::LaunchApp(name)` → 桌面内新虚拟窗口，
   launcher 自动隐匿；独立模式（`auto run` 直接跑）降级为 console 提示（441 原 demo
   边界保留为 fallback），保证示例单独可演示、desktop_mcp 可测。

### 1.1 对 Plan 441 的吸收映射

| 441 原交付物 | 处置 |
|---|---|
| M1 palette UI + mock 注册表 + 模糊过滤 + 最近使用 | **并入本计划**（mock 注册表 = 独立模式 fallback 清单） |
| M2 command-palette widget 原语化 | **降为可选任务 T6**（本计划先用 app 级实现跑通；沉淀为 widget 的时机由 465 对拍后另议，避免过早抽象） |
| M3 vm 端焦点原语 | **改由 Plan 462 承载**（WM 焦点分区 + AppId 命名空间）；palette 内只做自建焦点索引（441 §3 的「焦点封闭在 palette 内」路线） |
| 键盘导航/IME 风险表 | 继承（IME 中文输入实测列入 T5 验收） |

441 状态头由本计划立项时注记「被 464 吸收」。

## 2. 关键事实

- **清单/端口约定**：`pac.at` 字段与 4028 端口沿用 441 预订（`crates/auto-man/src/pac.rs`）；
  `icon:` 字段由 463 T7 落地（lucide 名，缺省 `app-window`）。
- **接缝前提（463 T1/T4 交付；Design 25 §3 定案）**：launcher 与宿主的双向
  接缝——下行：宿主注入应用清单（`Vec<AppRegistryEntry{name,title,icon,
  category,launchable}>`）与 `SummonLauncher` 事件；上行：**`desktop.launch(name)`
  builtin**（DesktopBus v1 命令形状）。launcher 是 **overlay 槽上的普通 App**，
  也是默认 shell pack（Design 25 §4.1，`widget Desktop`）装载的**第一个表面**
  （S2）——shell-track M1 的投影协议以本表面的接缝消费为第一个实测样本（R8/I9）。
- **模糊搜索的双端一致**：匹配逻辑写在 .at（纯函数状态法），vm 端跑 AutoVM、
  vue 端转 TS——013 等示例的 .at 逻辑双端转译是既有管线；排序权重（精确 > 前缀 >
  词首 > 子序列；近期使用加权）写进 SPEC 并作为 465 对拍项。
- **最近使用持久化**：`storage.get/set`（018/025/041 先例），键名
  `launcher.recent_apps`，上限 5。
- **键盘流前提（462 T6 交付）**：桌面级热键路由（Ctrl+Space 召唤，R12）+
  焦点分区；palette 打开时搜索框自动聚焦，Esc 逐层退出（清词 → 退网格 → 关闭）。
- **测试设施**：desktop_mcp（013 约定：`tests/desktop_mcp.py`）+ vue 端 Playwright
  （`.agents/skills/autoui-verifier/scripts/test_vue_playwright.mjs`，键盘流用
  `press`/`fill`/`screenshot` 步骤）。

## 3. 设计要点

### 3.1 UI 结构（.at 声明）

```
028-launcher/
  pac.at            # name:"launcher" render:"vue"(dev)/"vm"(desktop) front_port:4028 icon:"search"
  src/front/
    app.at          # 根：visible 态门控的 overlay（palette|grid 两形态容器）
    search.at       # 查询状态 + fuzzy 排序纯函数（query, entries, recent) -> ranked
    palette_view.at # 结果列表 + 高亮/选中索引 + 分组（最近/全部）
    grid_view.at    # 图标网格 + 网格焦点索引
  tests/desktop_mcp.py
  SPEC.md
```

overlay 门控 `visible` 状态变量由宿主 `SummonLauncher` 事件置位、Esc/启动后清位；
独立模式下键盘/按钮自管开关（无宿主事件时的 fallback 路径，也是 I3 允许的配置差异）。

### 3.2 双模式数据注入

- **desktop 模式**：宿主经下行接缝写 `apps` 状态变量（真注册表）+ `summon` 事件；
- **独立模式**（`auto run`，vue 调试/palette 开发）：`apps` 回退内置 mock 清单
  （10 条左右，覆盖全部 lucide 展示类目）；`launch()` 无宿主桥时 console 提示。
  两条路径共用同一状态变量名（I3：数据来源差异，非代码分叉）。

### 3.3 键盘流（P3 全集）

打开即聚焦搜索框 → 输入即过滤重排 → `↑↓` 移动选中（列表尾部回绕）→ `Enter` 启动
（发 `LaunchApp` + 写 recent + 关闭）→ `Tab` 切换 palette/grid（网格内方向键移动）→
`Esc` 清词 → 关闭。焦点索引是 app 内自建状态（不依赖全局焦点协议，441 M3 路线），
但**输入框本体聚焦**依赖 462 的 focus 分区命名空间。

## 4. 任务表

| # | 任务 | 内容 | 验证 |
|---|---|---|---|
| T1 | 脚手架 + mock M1 | 目录/pac.at/app.at 骨架 + mock 清单 + palette UI（搜索框/列表/高亮）+ fuzzy 排序纯函数 | `cd examples/ui/028-launcher && auto run`：vue 端 palette 可输入可过滤（Playwright 截图） `[✅ 已完成]` commit 6454ec1f9：vue build 一次通过 + Playwright fill"calc"→1/1 过滤、fill"to"→4 结果档位序 correct（tests/shots/t1-*.png）；单组件内聚（025 形态）为双端转译实测结论，见 §9 待澄清 |
| T2 | 键盘流（vue） | ↑↓/Enter/Esc/Tab 全集 + 网格形态 + recent（storage） | `tests/` Playwright 步骤：fill→↓×2→Enter 命中正确项断言 `[✅ 已完成]` commit 287aa4f87：tests/vue_verify.mjs 5 组断言全绿（fill"to"→↓×2→Enter=012-stopwatch、recent 顶置+localStorage 重载持久、Tab→grid 方向键 Enter=015-notes、Esc 清词→关闭、Ctrl+Space 召唤）；vue 端 bind{} 块原本静默失效——生成器补 `__autoBindKeymap` 全局 keydown 层（451 actions 层镜像） |
| T3 | vm 形态 | palette 在 462 焦点分区上的 vm 渲染 + 输入框自动聚焦 + app 内焦点索引 | `auto run --render vm` 实机全键盘流（441 M1 验收的 vm 版） `[✅ 已完成]` commit b85918f02：tests/desktop_mcp.py 24 断言全绿（渲染/visible 门控/过滤/↑↓ 移动/Enter 启动/Esc 逐层/Tab 形态/Ctrl+Space 召唤/grid 焦点/recent）；自动聚焦经 `__focus_input` 状态约定（update_inner 尾部消费→`operation::focus(prompt_input)`，实测 consumed 日志）；隐藏态键盘门控为本任务补强 |
| T4 | 接缝消费 | 消费 463 接缝：`summon` 置位/`apps` 真清单/`LaunchApp` 上行/启动后隐匿；独立模式 fallback | 在 ui_desktop 全屏桌面实机：Ctrl+Space 召唤 → 搜 "calc" → Enter → calculator 虚拟窗出现且 launcher 隐匿 `[✅ 已完成]` commit 1a743571d：实机全流程绿（召唤→真注册表注入→calc 过滤→Enter→calculator 虚拟窗+任务栏新增按钮+隐匿；recent 顶置跨进程持久）。补强：summon 动词入 DesktopBus、split_mut windowless 特权 App 垫片（463 任务栏点击静默丢弃同根因，§5.4 顺延项前置修复）、注入形态改平行字符串列表（Obj 数组 VM handler 字段读失效=B12 同族，探针测试钉死）；中文 IME 抢 Ctrl+Space 实测→Ctrl+Alt+Space fallback 顶上（463 注释预案成立） |
| T5 | 打磨 + IME | 图标/空态/无结果态/长清单滚动；中文 IME 输入实测（462 焦点分区上的组合输入） | 实机清单：中文搜索命中、Esc 逐层退出、重复启动同一 App `[✅ 已完成]` commit 61aca6813：空态/无结果态 T1 已含；重复启动（recent 顶置再启动）实机绿；Esc 逐层退出实机绿（Esc 被焦点输入框/IME Captured 时宿主转发补丁——launcher 键盘子网关 escape_forward，幂等）；IME 组合输入实测：组合串入框、提交文本走 SetQ 过滤、IME 抢 Ctrl+Space 由 Ctrl+Alt+Space fallback 顶上（463 预案成立）；中文标题搜索：注册表标题全拉丁，组合管线本身已证（结构上无障碍）；图标=monogram 瓷贴（lucide 动态名注入未证，T7 记账）；长清单滚动 vm 无 scrollable 映射（T7 记账）；§5.4 顺延矩阵：布局热键/Ctrl+Tab 循环实机绿，任务栏物理点击与空桌面关闭受沙箱点击注入通道所阻（split_mut 根因已修+单测钉死，见待澄清） |
| T6（可选） | command-palette 原语化 | 441 M2 遗产：DSL `command-palette { command(id:,title:,icon:,shortcut:,onclick:) }` + registry 登记 + 本 app 自举 | `cargo t ui_gen` 漂移测试绿 + a2vue 金样 |
| T7 | 收尾 | SPEC.md、examples/ui/README.md 总览表补行（028 落位）、441 归档 | `cargo test -p auto-lang --test docs_gen`（若动语法参考）+ README 核对 `[✅ 已完成]` SPEC.md 落地（接缝/排序规则钉死/键盘流/已知边界 4 项记账）；README 028 行更新为 464 ✅；docs_gen 4/4 绿；441 状态头已于立项时注记吸收、归档随 T6 缓议（见待澄清 4） |

## 5. 验收

1. **端到端（N4 联合验收）**：463 全屏桌面内，Ctrl+Space 召唤 → 模糊搜索 → Enter
   启动 examples/ui vm 兼容 App ≥6 个（011/013/024/025/038/041 + 459-dual-app 任取），
   多窗口经 grid/master-stack 排布，launcher 最近使用按启动记录更新。
2. **三端一份源（I5）**：同一 `app.at` 独立 vue（T1/T2 绿）、desktop vm（T3/T4 绿）、
   465 web 形态（由 465 复验）无分叉。
3. desktop_mcp 全键盘流绿；vue Playwright 绿；vm 端 IME 中文搜索实录通过。
4. **463 顺延项补测（463 §9-2 裁决转记，2026-08-28）**：执行期用户前台被并行会话
   占用未实测的实机交互矩阵，随本计划同一桌面流一并跑——任务栏点击（聚焦/关闭/
   布局切换/召唤）、Alt+Tab / Ctrl+Tab 窗口循环实机按键流、Ctrl+Alt+G/L/F 布局
   热键、空桌面态（逐个关闭后任务栏存活不退出）。463 侧已备齐的等价覆盖：会话级
   端到端单测（三 App 真实启动 → 三虚拟窗）、渲染/同步 MCP 截图、Esc 实机（同
   订阅路径）。

## 6. 风险

| 风险 | 缓解 |
|---|---|
| 接缝未合（463 延期）时 M1 阻塞 | T1–T3 只依赖 mock 清单与既有渲染，可完全先行；T4 单独验收 |
| vm 端 text_input 焦点与 overlay 门控组合问题 | 462 T6 已验焦点分区；palette 焦点自建索引隔离复杂度；问题实录回灌 462 |
| fuzzy 双端排序不一致（浮点/字符宽度） | 排序键全用整数权重；SPEC 钉死规则；465 对拍项 |
| lucide 图标缺口 | 缺图标走 414 补图先例；无图标回退 `app-window` |

## 7. 并发边界

- **拥有**：`examples/ui/028-launcher/**`。
- **消费**（不改）：463 接缝 API、462 焦点分区、storage、lucide 图标集。
- 若 T6 立项：增量拥有 `crates/auto-lang/src/ui_gen/widget/registry.rs` 的
  command-palette 项与 `schema/aura.at` 对应声明。

## 8. 关联

- 吸收：441（立项时注记其状态头；T6 完成后 441 可归档）。
- 依赖：462、463。下游：465（launcher 的 vue/tauri 形态复验与对拍）。

## 9. 待澄清事项（执行期记录）

1. **单组件结构偏离 §3.1 文件树（已裁决按实测管线收敛）**：`search.at` 的模糊
   排序与 palette/grid 视图未拆子文件/子组件——执行期盘点证实三处工具链缺口
   使「多文件 + 状态共享」无法双端一份源：①模块级 fn 不进 vue SFC（024 先例
   + 探针复证）；②store 子组件的 vue TS 生成损坏（013/038 实测）；③Obj 数组
   经 `write_state_vec` 注入后 VM handler 字段读失效（B12 同族，探针
   `injected_obj_array_for_field_read` 钉死）。采用 025 形态单 App 内聚
   （§1.1 M2「先用 app 级实现跑通」的延伸），app.at 内按注释分区；465 对拍后
   再议拆分/原语化。
2. **工具链补强（超出原 §7 拥有面，已随 T2/T4 落地）**：①vue 生成器补
   `bind {}` 块全局 keydown 层 `__autoBindKeymap`（此前 bind 块 vue 端静默
   失效，011-calculator 键盘仅 vm 可用）；②`split_mut` 对 windowless 特权
   App（shell/launcher）的静默丢弃修复——463 任务栏点击顺延项 §5.4 的根因；
   ③DesktopBus 增 `summon` 动词。均为既有管线的缺口补齐，非接缝形状变更。
3. **任务栏物理点击 + 空桌面关闭项实机未验**：测试沙箱的鼠标注入通道
   （mouse_event/SendInput/PostMessage）均未达 ui_desktop 窗口（键盘注入正常），
   物理点击无法执行；代码路径已修并以
   `windowless_shell_split_mut_and_bus` 单测钉死。留待真人会话复验
   （463 §5.4 顺延项剩余半边）。
4. **T6（command-palette 原语化）按计划设计缓议**：§1.1 定案「先用 app 级
   实现跑通，沉淀时机由 465 对拍后另议」；441 归档随之顺延（其状态头已注记
   被 464 吸收）。
5. **lucide 动态图标与长清单滚动**：`icon` 字段全链路携带但 vue lucide 导入
   收集只认静态名，行/瓦片现用 monogram 瓷贴；vm 无 scrollable 映射，29 条
   全量渲染。均记账于 SPEC.md「已知边界」，不阻塞验收。


## 10. 复审记录（/auto-plan:review，2026-08-29）

**复审人**：zcode（/auto-plan:review 独立门禁）。**方法**：验收逐条复验（不信勾选）+
worktree 实diff核对（master..HEAD=6 commits；crates 侧恰 3 文件：renderer.rs +438/
session.rs +163/vue.rs +129）+ 全量门禁 cargo tf 重跑。

### 门禁

| 门禁 | 结果 |
|---|---|
| cargo tf（全量，唯一全量门） | ✅ 3235/3235 passed, 89 skipped（29.9s） |
| cargo check -p auto-lang --features ui-iced | ✅ 0 error |
| scoped：cargo t ui::session ui::iced --features ui-iced | ✅ 67/67 |
| 探针/垫片单测（summon 注入/Obj 数组读/windowless 拆借） | ✅ 3/3 |
| docs_gen | ✅ 4/4 |
| vue 验收 tests/vue_verify.mjs | ✅ 5/5 组 |
| vm 验收 tests/desktop_mcp.py | ✅ 24/24 |

### 验收复验（§5 逐条）

| # | 条目 | 判定 | 证据 |
|---|---|---|---|
| 5.1 | launcher 启动 ≥6 vm 兼容 App + grid/master-stack 排布 + recent 更新 | ✅ | 实机复测（复审现场重跑）：launcher 键盘流逐个启动 011/013/024/025/038/041 全集 + article-feed/calendar/hello-world，9 虚拟窗 Ctrl+Alt+L master-stack 排布，calculator 置 master 即焦点；recent 顶置跨进程持久 |
| 5.2 | 三端一份源（I5） | ✅/部分 | vue（vue_verify 5 组）+ desktop vm（desktop_mcp 24 组 + 实机）双端绿；465 web 形态由 465 复验（本计划出界，计划明示） |
| 5.3 | desktop_mcp 绿 / vue Playwright 绿 / vm IME 中文搜索实录 | ✅/部分 | 前两项全绿；IME 实测=组合串入框+提交文本过滤+IME 抢 Ctrl+Space 由 Ctrl+Alt+Space fallback（463 预案成立）；「中文标题命中」无对象——注册表标题全拉丁（管线结构无障碍，记账 SPEC 已知边界 4） |
| 5.4 | 463 顺延项补测 | 部分 | 布局热键（G/L/F，G 实测）✅；Ctrl+Tab 循环 ✅（焦点迁移+任务栏 z 序更新实机）；任务栏物理点击（聚焦/关闭/召唤）与空桌面逐个关闭——**split_mut 根因已修**（`windowless_shell_split_mut_and_bus` 单测钉死）+ ⊞ 消费链路实机可用（Ctrl+Alt+Space 通路同源），唯物理鼠标点击受测试沙箱注入通道所阻（mouse_event/SendInput/PostMessage 均未达窗口；键盘注入正常），留真人复验 |

### 遗漏/延后/workaround 排查

- **延后（用户已签）**：T6 原语化缓议（§1.1 明示「465 对拍后另议」）、441 归档随 T6、
  465 web 形态复验——均计划内载明，非静默缩减。
- **workaround 类记账（SPEC 已知边界，不阻塞）**：lucide 动态名图标未渲染
  （monogram 瓷贴顶替，icon 字段全链路保留）；vm 长清单无 scrollable 映射
  （全量渲染）。
- **遗漏排查**：任务表 7 项均有对应 diff（app.at 645 行/tests 477 行/SPEC 91 行/
  crates 3 文件 700 行）；无 Done 标记却无 diff 的任务。
- **环境残留清理**：执行期 AUTO_DEBUG_KEYS/AUTO_DEBUG_FOCUS/AUTO_DEBUG_MSGS
  诊断打印均为 env 门控（ASH_DEBUG_FOCUS 仓库先例形态），默认零输出——保留。

### 结论

**reviewed（通过）**。两处「部分」均有根因修复/结构证明背书且已记账（SPEC 已知
边界 + §9 待澄清），无阻塞债。就绪 `/auto-plan:merge`（折叠 + specs.json 六段沉淀
+ 归档）。
