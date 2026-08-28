# Plan 464: Launcher App（examples/ui/028-launcher，真注册表 + 真启动）

> **状态**：已立项 2026-08-28，未开工
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
| T1 | 脚手架 + mock M1 | 目录/pac.at/app.at 骨架 + mock 清单 + palette UI（搜索框/列表/高亮）+ fuzzy 排序纯函数 | `cd examples/ui/028-launcher && auto run`：vue 端 palette 可输入可过滤（Playwright 截图） |
| T2 | 键盘流（vue） | ↑↓/Enter/Esc/Tab 全集 + 网格形态 + recent（storage） | `tests/` Playwright 步骤：fill→↓×2→Enter 命中正确项断言 |
| T3 | vm 形态 | palette 在 462 焦点分区上的 vm 渲染 + 输入框自动聚焦 + app 内焦点索引 | `auto run --render vm` 实机全键盘流（441 M1 验收的 vm 版） |
| T4 | 接缝消费 | 消费 463 接缝：`summon` 置位/`apps` 真清单/`LaunchApp` 上行/启动后隐匿；独立模式 fallback | 在 ui_desktop 全屏桌面实机：Ctrl+Space 召唤 → 搜 "calc" → Enter → calculator 虚拟窗出现且 launcher 隐匿 |
| T5 | 打磨 + IME | 图标/空态/无结果态/长清单滚动；中文 IME 输入实测（462 焦点分区上的组合输入） | 实机清单：中文搜索命中、Esc 逐层退出、重复启动同一 App |
| T6（可选） | command-palette 原语化 | 441 M2 遗产：DSL `command-palette { command(id:,title:,icon:,shortcut:,onclick:) }` + registry 登记 + 本 app 自举 | `cargo t ui_gen` 漂移测试绿 + a2vue 金样 |
| T7 | 收尾 | SPEC.md、examples/ui/README.md 总览表补行（028 落位）、441 归档 | `cargo test -p auto-lang --test docs_gen`（若动语法参考）+ README 核对 |

## 5. 验收

1. **端到端（N4 联合验收）**：463 全屏桌面内，Ctrl+Space 召唤 → 模糊搜索 → Enter
   启动 examples/ui vm 兼容 App ≥6 个（011/013/024/025/038/041 + 459-dual-app 任取），
   多窗口经 grid/master-stack 排布，launcher 最近使用按启动记录更新。
2. **三端一份源（I5）**：同一 `app.at` 独立 vue（T1/T2 绿）、desktop vm（T3/T4 绿）、
   465 web 形态（由 465 复验）无分叉。
3. desktop_mcp 全键盘流绿；vue Playwright 绿；vm 端 IME 中文搜索实录通过。

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
