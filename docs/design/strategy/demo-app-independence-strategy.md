# Demo 应用独立化与产品孵化（v0.6 近期工作草案）

> 更新：2026-10-04。用户补充：近期主要改动集中在四大主力 app，其他 Demo 较少，视频有未成功改进。
> 本文据此修订“暂缓所有应用工作”的建议；现有 Demo 保留，独立产品采用新仓、新开发线。
> 2026-10-03 已完成 21 个公开产品仓的源码导入、来源快照与 push；AutoOS v0.6-dev 已接入 21 个新仓及 3 个已有应用 submodule。
> 2026-10-04 AutoOS 主检出已切到 v0.6-dev，24 个应用子模块及 Notes 嵌套依赖完成初始化/核验，VM 桌面实际启动。
> 建仓与源码接线已完成；下文功能、持久化、双端验收及产品演进仍是后续工作，不将首轮启动计为全部完成。
> 本地发布基线为 `v0.6-base-20261003`。主电脑 739/740 的实际内容仍不可见。

## 1. 新的近期工作边界

应用独立化和新产品演进可以成为近期主线，与 Atom/HIR/AC、统一安装核心并行。
沿用已有示例作为新产品的初始快照，不表示要续做 v0.5 的产品收尾。

- examples/ui 中的教学 Demo 保留原样；独立仓库拥有自己的功能路线、提交和发行。
- 四大主力 app（auto-shell、auto-musk、auto-edit、jade-edit）继续使用已有仓库和外部应用接入。
  当前不为它们迁移 submodule 或改变启动方式。
- 新 app 核心和业务代码不回写旧 Demo；依赖共享框架的新能力另外立项，避免顺手改 v0.5。
- 视频播放器/视频站点先保存来源与边界，功能演进优先避开不可见的媒体改动，恢复后再对照导入。
- Git submodule 固定源码组合；系统 package/install 管理运行制品与生命周期。两者共享 app ID 和来源契约。

## 2. 为什么可提前做、哪些步骤仍需验证

复制代码到新仓不会修改源目录，新增远端与独立开发线也不会改变 v0.5。
GitHub 首次发布前可准备精确文件集、来源、依赖清单与命名表，使仓库创建/push 是具体结果的发布。

但“新增 submodule”“替换已有目录”“切换启动源”有不同影响：

| 阶段 | 可交付内容 | 与隐藏 v0.5 工作的关系 |
|---|---|---|
| A 来源快照与独立仓 | 新仓首次导入、GitHub remote、可追溯来源 | 源代码不回写，文本冲突很低；仍需核对导入范围与依赖 |
| B 新增 submodule | AutoOS v0.6-dev 的 .gitmodules、gitlink、manifest 对应项 | 共享少量配置文件，后续按 app 合并；添加并不自动证明已启动独立源码 |
| C 已维护 app 目录转 gitlink | apps/025 等原目录替换为 submodule | 隐藏 v0.5 修改可能产生目录/文件与 modify/delete 冲突；逐 app 导入，不覆盖隐藏更新 |
| D 启动来源切换 | v0.6 专用启动配置、注册表路径验证 | 涉及来源优先级、ID、端口/daemon/原生制品；单独验收 |
| E 新产品功能 | 读书、博客等独立仓的业务演进 | 独立提交，后续通过 gitlink 固定组合，不同步回教学 Demo |

可以先批量完成 A、新增路径的 B 和 E。已有 AutoOS 目录的 C 可以先发布副本，
再逐个评估替换；最后一轮来源对照可安排在完整 v0.5 恢复后，避免漏掉隐藏修改。

## 3. 当前可核查的 28 项来源

来源：[网站目录](../../../website/.vitepress/theme/data/demos.json)及
[截图来源清单](../../reports/p723-capture-catalog.json)。它们确定展示范围，不证明当前系统全量验收通过。
20 项来自 auto-lang、6 项来自 auto-os，另有 auto-term/auto-os-config 两项已有独立仓。
不要将“28 项展示”误读成“需新建 28 个仓库”。

用户已确认以下 21 个新仓名称与合并方案；2026-10-03 已核对组织现有仓库，无同名仓。
新仓与已有 auto-kanban 一样采用 public 可见性；在用户后续授权下已导入并推送源码、固定 submodule 与 manifest。
创建状态见 [建仓清单](../../reports/demo-app-repositories-20261003.json)。
submodule 路径和 manifest ID 在初期尽量保留现有应用身份，仓名可独立命名。

| 当前 ID | 当前来源 | 独立仓候选 / 处理 | 建议批次 |
|---|---|---|---|
| 011-calculator | lang examples/ui | auto-calc | 首批 |
| 012-clock | lang examples/ui | auto-clock | 首批 |
| 013-todo | lang examples/ui | auto-todo | 第二批：后端 |
| 014-weather | lang examples/ui | auto-weather | 第二批 |
| 015-notes | lang examples/ui | auto-notes | 后续：编辑器依赖 |
| 016-calendar | lang examples/ui | auto-calendar | 首批 |
| 017-chat | lang examples/ui | auto-chat | 后续：后端 |
| 018-book-reader | lang examples/ui | auto-reader | 第二批，优先产品孵化 |
| 021-blog-viewer | lang examples/ui | auto-blog，与 023-realworld 合并为一个产品（用户已确认） | 第二批，优先产品孵化 |
| 019-video-app | lang examples/ui | auto-video，与 030-video-player 共用产品仓；社区能力留作远期 | 来源快照保留；社区不列为近期目标 |
| 020-music-player | lang examples/ui | auto-music | 后续：媒体 |
| 029-photo-gallery | lang examples/ui | auto-photos，与 031-image-viewer 合并（用户已确认） | 后续：文件/媒体 |
| 030-video-player | lang examples/ui | auto-video，以播放器为近期产品基础；不新建 auto-video-player | 来源导入先行；媒体演进待对照 |
| 031-image-viewer | lang examples/ui | auto-photos，不新建 auto-image-viewer | 第二批：文件服务 |
| 031-paint | lang examples/ui | auto-paint | 首批 |
| 025-sys-monitor | os apps/025-sys-monitor | auto-monitor；以 OS 产品轨为源 | 既有目录迁移批 |
| 026-database | lang examples/ui | auto-database | 后续：文件/数据库 |
| 027-file-manager | lang examples/ui | auto-explorer | 后续：系统/文件 |
| 028-launcher | os apps/028-launcher | auto-launcher；独立 Windows 通用启动器，亦供 AutoOS 复用 | 产品独立化与 Windows/插件专项 |
| 039-syslog | os apps/039-syslog | AutoOS 内部组件，用户已确认不独立建仓 | 保留在 auto-os |
| auto-term | auto-term/app | 已有仓，已接入 apps/auto-term（入口 app/） | 已初始化，保留固定提交 |
| auto-os-config | auto-os-config | 已有仓，已接入 apps/os-config（入口 auto/） | 已初始化，保留固定提交 |
| 036-tetris | os apps/036-tetris | auto-tetris | 既有目录迁移批 |
| 037-klondike | os apps/037-klondike | auto-solitaire | 既有目录迁移批 |
| 038-minesweeper | os apps/038-minesweeper | auto-minesweeper；以 OS 产品轨为源 | 既有目录迁移批 |
| 022-kanban | lang examples/ui | 沿用已有 auto-kanban 产品仓，教学 Demo 保留，不重复建仓 | 复用批 |
| 023-realworld | lang examples/ui | auto-blog，与 021-blog-viewer 共用产品仓，不新建 auto-realworld | 第二批：文章社区与完整后端 |
| 024-charts | lang examples/ui | auto-charts | 后续：产品定位 |

025/028/038 已在 Plan 666 明确分成教学与 OS 产品两轨；迁移产品仓应取 auto-os 源，
不能拿 examples/ui 副本覆盖已独立演进的系统版本。036/037 等同样以 OS 目录为准。

### 产品合并决策与建议（2026-10-03）

- 用户已确认：021-blog-viewer 与 023-realworld 合并为 auto-blog；保留两份教学 Demo，
  独立产品整合博客阅读、写作、发布和社区能力。首次导入分别记录来源，不直接覆盖两套源码。
- 用户已确认：019-video-app 与 030-video-player 统一归入 auto-video，不新建 auto-video-player。
  近期以视频播放器为产品基础；视频社区留作远期方向，不因共用仓库而立即整合社区代码或后端。
  两份教学 Demo 保留，来源快照分别记录。
- 用户已确认：029-photo-gallery 与 031-image-viewer 合并为 auto-photos，提供快速看图与
  图库管理两种入口，共用查看、缩略图、元数据和基础编辑能力。直接打开图片不强制导入图库。
- 用户已确认：039-syslog 是 AutoOS 内部组件，保留在 auto-os，不新建 auto-system-log。
- 用户明确 Launcher 产品定位：auto-launcher 应独立建仓，可单独安装在 Windows 上，
  作为开始菜单的替代入口；通过插件扩展应用启动、文件快速查找、AI 提问等能力。
  AutoOS 复用产品核心与对应宿主适配，不将仅能在 AutoOS 内启动视为独立产品验收通过。
- 用户已确认系统监视器命名 auto-monitor，资源浏览器命名 auto-explorer。
- 博客、视频、图片合并，系统日志排除，Launcher 独立后，共需新建 21 个仓库。
  28 个教学/展示 Demo 仍保留，独立产品不要求与 Demo 一一对应。

## 4. 已发现的依赖与启动事实

### 4.1 依赖需要随导入处理

- calculator、calendar、paint、book-reader、blog-viewer 等 pac.at 使用 `dep stylekit { path: "../stylekit" }`。
  单独 clone app 后该目录缺席，原样拷贝并不自动形成自包含包。
- notes 的 npm_deps 使用 `link:D:/autostack/auto-down/...`，不是适合 GitHub 用户的发行配置。
- 多个 app 的 `api: rust`、端口和 back 源须成套导入；测试不能仅检查 front 窗口出现。
- 现有 OS 游戏/Launcher 的 desktop_exe 指向生成目录；gitlink 和 .at 本身不会携带已经构建的 exe。

来源快照提交先保存原始 pac/src/assets；依赖、端口、测试路径和产品 README 调整另提交。
stylekit 可使用仓内版本化副本/正式包，或 OS 中同层共享包；具体选型需能满足独立 clone
和子模块检出两种运行方式，不能只在当前开发目录偶然成功。不为此创建 junction/symlink。

导入采用明确的受版本控制文件集，保留许可与来源信息；生成缓存、target、node_modules、
真实用户数据和开发机配置不作为源码发布。另核查运行必需资产是否恰好被忽略，避免漏资产。

### 4.2 submodule 检出与应用发现已有基础

AutoOS 已有 apps/kanban gitlink 和 .gitmodules；apps/ 直接子目录有 pac.at 时可发现。
现有 manifest 只接受 kind=repo/local 的契约；submodule 物理形态并不意味着写 `kind: submodule`。
沿 AutoOS AGENTS §4，manifest 与 gitlink 配套，使用实际支持的 kind。

### 4.3 必须证明实际启动的是独立源码

当前 `aggregate_scan` 主根优先，extra 中同 ID 被跳过；Vue 镜像也是 primary-root 优先。
只增加 apps/011-calculator 时，默认主根仍是 examples/ui，可能继续启动教学版。

当前 VM 宿主已有 `--apps-dir`，可在 v0.6 专用入口中将主根指向产品目录并明确
AUTO_OS_ROOT/AUTO_LANG_ROOT。优先用现有配置接口，不先改公共注册表或 v0.5 默认脚本。
Vue 入口的主根配置要单独核查/验证，不拿 VM 的命令行参数当作 Web 已支持的证明。

每项记录 registry ID、entry/back_root 的实际路径、submodule SHA、窗口/进程形态和
后端地址；界面相同不作为路径切换证据。未初始化 submodule 的行为也要明确。
完整虚拟桌面验证需使用对应 OS/框架工作树，不能静默回退到 D:/autostack 主检出。

## 5. 来源恢复与未来合并

每个首次导入记录 `source_repo + source_commit + source_path + tracked-file manifest/hash`。
首个代码提交只承担源快照，后续提交再做包装与产品功能，让源差异可重建。

主电脑恢复后的源码更新不会自动跨越仓库复制关系，需要单独对照：

- Base：首次导入的 Demo/OS app 快照。
- Upstream：完整 v0.5 同一路径的最终版本。
- Product：独立仓的产品版本。

必要修复按三方差异选择吸收，不把 Upstream 整目录覆盖 Product。可从首次导入提交
建立来源同步分支，提交最终 v0.5 导出，再合入产品开发分支；两边具有明确共同基线。
教学轨无需长期自动同步产品功能；保留来源追溯不等于永久建立两轨强同步义务。

AutoOS v0.6-dev 的新增 gitlink 按应用合并；已有实体目录转 gitlink 的冲突先保全并
导出完整 v0.5 修改，再更新独立仓提交，最后修改 gitlink。GitHub 提交须先 push 成功，
再发布引用它的 AutoOS 提交，保证其他人能够检出固定组合。

## 6. 批次与验收

1. 首批 calculator/clock/calendar/paint：验证导入模板、共享依赖处理、GitHub/子模块链和启动来源。
2. book-reader/blog-viewer/todo：验证 API/back 与数据持久化，再启动有边界的产品功能。
3. 其他普通 Demo：复用已验证导入流程，但每个检查依赖与真实运行结果。
4. AutoOS 已维护的五个独立应用候选：逐项从 OS 产品轨迁出；Launcher 等系统能力独立验收。
   系统日志作为 AutoOS 内部组件保留，不迁出。
5. 媒体、已有独立仓和 kanban 归属项：单独处理，不为了凑“28 个新仓”重复建仓或冒称全绿。

验收包括首次导入 src/assets 的字节对应、独立 clone 可用、固定 SHA 子模块检出、
VM/Web 各自适用运行方式、桌面显示/打开/交互/关闭/再开、真实后端、实际 entry 路径、
唯一注册项、教学目录无改动和四大主力 app 的接入配置无改动。
框架 bug 若阻塞某项，登记该项失败并隔离，不在此工作包继续 v0.5 共享框架修复。

## 7. 新产品演进优先方向

读书与博客可作为第一批真实产品，两者与个人知识服务有直接连接，但无需等 v0.7
完整知识管理系统。以下是产品候选，不是承诺已选定所有功能或完成了竞品研究：

- Reader：真实书籍导入、可靠阅读进度、书签/批注、出处可追溯的摘录、导出；
  AI 辅助阅读与后续知识包可在基础体验成立后接入。
- Blog：真实内容创建/编辑/管理、发布或导出、订阅/阅读入口、摘录；先明确“个人创作工具”
  与“公共社区服务”的版本边界，不将完整社区运营作为首批前置。
- Todo/Calendar：个人活动与目标管理、用户数据可携带、跨应用任务连接。
- Launcher（用户新增明确方向）：独立 Windows 热键启动入口，统一查询/结果/动作交互，
  以插件接入应用、文件快查和 AI 提问；AutoOS 接入另行验证。现有 OS 版本仅作来源基础，
  Windows 宿主能力与插件契约需要单独设计，不能把现有 demo 拷贝当作上述功能已经实现。
  建议首批先做三类插件与独立安装/更新；文件搜索优先评估既有索引服务适配，
  AI/慢查询采用异步返回、取消过时查询，插件失败不阻塞基础启动体验。
  “最快”作为性能目标，验收需固定设备/数据规模，分别测热键唤起、首批结果和长尾延迟。
- 其余应用分别定义主要用户、核心活动、对标对象、差异化假设和一条真实可验收流程。

对标研究使用官方产品资料和实际体验后再形成范围；每个 app 先完成一条实用流程，
而不是28个同时扩功能。新功能主要在独立 app 仓迭代，AutoOS 只按已验收版本更新组合。
