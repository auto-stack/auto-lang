# Website / v0.5 截图拍摄清单

更新：2026-10-10 · PLAN-756。**本次仅修正文案与预留位置，没有拍摄以下新图。**
首页与 v0.5 使用同一组产品主图；专题页使用同一组工程、数据和版本继续展示操作，避免每页重新搭景。页面的 `SHOT-01..12` 对应下表。已批准的桌面、Shell、AutoEdit 专题图继续保留，不能因此视为候选已验收。

## 先准备一次可复现的拍摄环境

1. 发行候选确定后，记录工具链与各应用 **实际构建对应的 commit/tag**。准备匹配的 Rust、Auto CLI、Node/pnpm、CPython 和兄弟依赖；不要只记录拍摄当日 HEAD。
2. 建立专用 `website-demo` 项目/知识库，只用演示代码、公开素材与专用模型账户。标题、代码和任务都用可读的自然内容；排除个人目录、密钥、客户信息和真实私有会话。
3. 原生窗口优先 1920×1200 或足够显示全部工作区的尺寸；Web 视口建议 1440×900，浏览器 100%。双端对照采用相同逻辑内容区、主题、字体大小、数据与滚动位置，原图记录实际像素/DPI，不强拉伸为相同尺寸。
4. 图中应有一条真实操作完成后的结果。排除加载中、空白、阻塞报错、被菜单盖住的主内容。允许真实内置样例，但必须标明；截图不能证明未实际运行的功能。
5. 保存完整 PNG 原图。若制作压缩展示图，另存派生文件，放大/原图仍指向完整版本。不要重绘 UI、拼接不曾同时存在的状态或以普通 Kanban 冒充 Musk Kanban。
6. 每图记录：ID、用途、源仓、源文件/fixture、构建 commit/tag、工具链 commit/tag、二进制路径及 SHA-256、实际运行命令、OS、渲染后端、窗口/视口与 DPI、主题、服务/模型配置范围、逐步操作、拍摄日期、PNG 尺寸及 SHA-256。未知项写“待确认”，不以旧图或预测版本填充。

## 新图空位：逐张怎么拍

| ID / 建议文件 | 页面位置 | 准备与操作 | 必须入镜 / 验收 |
|---|---|---|---|
| **SHOT-01** `autoedit-workspace.png` | 双语首页、v0.5 的 AutoEdit 主图 | 在 `auto-edit/specs/auto-edit` 准备工程，按对应版本运行 `auto run -r vm`。打开 `website-demo`，创建 `main.at`、`README.md`、一个配置文件；打开两个标签，在 `main.at` 做一次小修改并保存。 | 左侧项目树、标签、可读 Auto 源码、状态栏。无未解释报错；既有 `apps/autoedit/overview-dark.png` 可作构图参考，复用时必须保留 2026-10-01 来源，不能改称新候选实拍。 |
| **SHOT-02** `autoedit-search.png` | AutoEdit 专题“编辑、搜索与文件保存” | 延续 SHOT-01 工程，在两个文件放入同一个函数名；执行跨文件搜索，点击第二个结果定位到文件。 | 搜索词、两个真实结果、对应文件的匹配位置；结果与源码对应。不要把普通查找截图标作跨文件搜索。 |
| **SHOT-03** `autoedit-diff.png` | AutoEdit 专题“比较文件、目录与缓冲区” | 用同一小文件制作 before/after 两个版本，包含一处新增、一处修改；打开实际双文件比较，定位到一个差异块。另保存“跳到文件侧编辑→保存→重新比较”的操作记录。 | 两侧路径与正文、增改块、差异导航。不要宣称差异行可原位编辑；单张主图展示比较，后续操作靠真实记录佐证。 |
| **SHOT-04** `musk-workspace.png` | 首页、v0.5、AutoMusk 专题主图 | 按 `auto-musk/README.md` 配置 aaid/模型；生成主 Vue UI：`auto build --gen-only`，在 `gen/front/vue` 安装依赖并构建，回仓库根运行 `musk serve`。从实际启动日志打开地址，不猜端口。选择 `website-demo`，发起“阅读项目并说明一个小改动”，等待至少一次真实工具调用返回。 | 项目名、请求、正常消息及工具活动；不露提供商密钥。不要用旧 v0.1.0 桌面图代替新工作台。若模型服务不可用，保留空位，不摆拍成功结果。 |
| **SHOT-05** `musk-plan-review.png` | AutoMusk 专题 Plan 工作流 | 沿 SHOT-04 同一项目创建一个小 Plan，经过用户确认后执行，再完成对应复审；打开能显示任务与验证结果的视图。 | Plan 名称/状态、具体任务、测试或复审结果；能够对应项目里的真实 Plan 与改动。不要以 Relay 流程结束代替 Plan 已验收/归档；不能展示重启恢复等尚未验收能力。 |
| **SHOT-06** `musk-kanban-canvas.png` | AutoMusk 专题发布候选场景 | **前置：确认 Musk Kanban/Canvas 的实际发布范围及可用入口。** 使用候选中受支持的操作：创建或打开真实任务，选择其对应的可预览产物，启动实际预览，等待内容完成显示。把任务与预览并排放在同一应用工作区。 | 任务与实际预览的关联、预览内容及入口状态；caption 明确预览对象和运行条件。若候选只支持外部打开，分别拍两张并如实写明，不拼成内嵌 Canvas。不可用时继续留空；不复用普通 Kanban 截图。 |
| **SHOT-07** `jadeedit-workspace.png` | 首页、v0.5、JadeEdit 专题主图 | **前置：对应 Milestone 与拍摄准备验收。** 在 `jade-edit` 根按该版本启动 `auto run -r vm`；Web 路径按工程构建说明准备前端及文件后端。使用统一知识库“项目笔记”，含“主页”“Auto 学习”“发布记录”，打开两篇 `.ad` 文档，实际编辑并保存一句话。 | 文档目录、标签、标题、正文，保存后内容可重开；采用 JadeEdit 真界面，不能以 AutoDown/Jade Garden 替代。 |
| **SHOT-08** `jadeedit-links-search.png` | JadeEdit 专题“连接页面” | 延续 SHOT-07，主页链接“Auto 学习”，后者链接“发布记录”；打开“Auto 学习”查看实际反链，再用搜索词定位“发布记录”。 | 链接/反链与真实文件内容对应，搜索词和匹配结果可读；若这些面板不能同时出现，拍 `-links`、`-search` 两张，不人为拼接。 |
| **SHOT-09** `jadeedit-draft-recovery.png` | JadeEdit 专题未保存内容保护 | 使用隔离知识库。在一篇已保存文档里输入可识别的未保存标记，按该版本已验证的检查点/重启流程触发恢复；选择打开恢复副本。 | 原文件与恢复副本可区分，副本含未保存标记；另记录原文件未被覆盖的检查。只说明本版本已验证的恢复路径，不扩写为多文档所有操作均安全。 |
| **SHOT-10** `playground-source-result.png` | v0.5 Playground 段落 | 按网站后端说明在匹配仓库启动 `cargo run -p auto-playground`，确认实际健康状态和 API 配置。打开中文或英文 Playground，选择短小、已支持的循环/函数示例，点击运行，等真实结果。 | 示例名、源码、实际输出、后端连接/运行状态。需要 Debug 展示时，另在真实调试路径拍 `playground-debug.png`，保留断点/步进/变量与执行位置；若没有验证该路径，不拍模拟 Debug 图。 |
| **SHOT-11** `autoui-kanban-web.png` | v0.5 AutoUI Web 对照空位 | 在 `auto-lang/examples/ui/022-kanban` 用冻结源码、保留默认三列九任务（4 todo / 2 doing / 3 done），先按该版本说明执行生成，再运行 Vue 路径（`auto run`）；按实际版本说明准备依赖/后端。通过实际移动按钮把一张任务卡移到下一列，另记录交互结果；重启隔离后端恢复默认种子后拍摄，确认两端连接同一准备状态。 | 三列、相同任务名/排序/计数、实际 Vue 界面；记录交互结果。此处是 022 示例，不是 Musk 产品。 |
| **SHOT-12** `autoui-kanban-vm.png` | v0.5 AutoUI 桌面对照空位 | **与 SHOT-11 相同源码/数据/主题**，在同一工程运行 `auto run -r vm`。执行相同移动操作，恢复到同一截图状态；使完整看板可见。 | 三列内容与 SHOT-11 一致，显示真实原生窗口。保留窗口差异，不裁图伪装像素全等；截图说明与交互验证分开。 |

Web / 原生的准备依赖、参数或界面若在冻结版本变化，以该版本 README/pac.at 和启动日志为准，并更新实际命令记录。上述命令是已有开发入口，不表示本轮已验证候选安装和运行。

## 保留图：冻结后如何重拍或确认复用

以下不是新增空位，当前网站继续显示既有实图。最终发行时逐图决定“新候选重拍”或“保留历史图并明确来源”。

| 当前素材 | 重拍步骤 / 必须说明 |
|---|---|
| `desktop-showcase/01-desktop-light.png` | 在匹配 `auto-os` 根运行 `./scripts/desktop.ps1 -Track iced`（先可 `-DryRun`）。切浅色，启动时钟/Todo/音乐支持的小组件应用并最小化，不展开主窗口；拍全桌面、图标、任务栏、小组件。 |
| `desktop-showcase/02-desktop-dark.png` | 延续相同桌面状态切深色，保持窗口尺寸和小组件数据；拍完整桌面。用作 v0.5 首图，保留完整原图。 |
| `desktop-showcase/03-launcher-light.png` | 浅色桌面打开 Launcher，以 Tab 切图标网格；清空搜索词，拍应用图标、搜索入口与后方桌面。 |
| `desktop-showcase/04-launcher-dark.png` | 在完全相同 Launcher 状态切深色后拍摄。图标数量按候选实际清单，不按网站 28 项介绍数量布置。 |
| `desktop-showcase/05-games-dark.png` | 实际启动俄罗斯方块、纸牌、扫雷，完成至少一次有效操作；拖动三个窗口并排，游戏主体和任务栏可见。不要使用三个不同会话的拼图。 |
| `desktop-showcase/06-productivity-dark.png` | AutoEdit 左半屏打开示例代码；Todo 右上放三个演示任务，Calendar 右下选实际日期；拖动/缩放后拍同时存在的工作布局。 |
| `apps/autoshell/ash-01.png` | 在支持的 AutoShell 原生终端入口，用隔离目录真实运行彩色 `ls`；目录含短文件名、子目录及不同类型文件。命令和完整彩色表格入镜，保留原生滚动/提示符。按 Shell guide 的版本/平台记来源，不能用网站拟真终端替代。 |
| `apps/autoedit/overview-dark.png` | 当前专题/应用总览已批准原图可继续引用；候选重拍按 SHOT-01，保留旧来源，更新图片时记录新文件哈希和实际版本。 |
| `v05/autodown-{desktop,web}.png` | 当前仅为 AutoDown/Jade Garden 历史资料。若重拍，进入 **AutoDown 对应 showcase 工程**，相同 vault/正文分别运行 VM 和 Vue，拍目录、编辑与预览三窗格；明确标为 Jade Garden，不用来填写 JadeEdit 空位。 |
| `v05/kanban-{web,desktop}.png` | AutoOS 历史资料中保留的旧图不能绑定新候选；新对照按 SHOT-11/12 拍摄。若继续展示旧图，保留旧来源并与 Musk Kanban 分开。 |

## 28 项系统应用现有素材

当前介绍图保留，不因本轮文字修改重新拍摄。每个条目的源仓/工程、实际操作、数据性质、后端和原图记录在 [p723 capture catalog](p723-capture-catalog.json) 与 [拍摄契约](../specs/website/design/demo-capture-catalog.md)；网站单源 `website/.vitepress/theme/data/demos.json` 的 `steps`/运行条件用于对应条目的走查。

若冻结候选需要更新某项：按该条目实际 `sourceRepo/sourcePath` 和原捕获运行形态启动 → 重建记录中的 fixture → 完成该条目 `steps` 的首个有效动作 → 保持其最重要操作区可读 → 拍完整真实界面 → 更新该条目来源/版本/动作/hash；不可用则登记缺口，不能借用其他产品图。音乐用真实测试音、视频用真实测试片段、相册用公开测试图、文件管理用隔离目录、监视器注明采样机器，天气/聊天/数据库内置数据注明演示性质。

## 填图与最终验收

- 所有新主图先进入独立 worktree，建议保存到 `website/public/v05/candidate/`；页面用真实 EvidenceImage 替换对应文字槽，配置原始尺寸、说明与放大。首页/发布页复用同一来源。
- Caption 格式：`产品/场景 · Web(Vue) 或 VM(iced) · 平台 · 拍摄日期 · 源版本`；需要服务或使用内置数据时继续注明。构图一致不等于功能一致。
- 审核中英文说明、桌面/手机、深/浅网站主题和关键入口；缩略清晰、无断图、放大和原图可用。未拍的槽不能在发布收据里勾成“展示可复现”。
- 同步 `docs/plans/v05-release-closeout.md` W-02 与冻结表。最终下载、安装与应用验收独立进行。
