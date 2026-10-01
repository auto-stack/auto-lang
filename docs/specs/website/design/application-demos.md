# 系统应用与 Demo 的网站展示设计

> PLAN-722 / PLAN-723 · 2026-10-01 · 28项静态图文介绍，不改应用/画廊源码，不做mock，不宣布在线覆盖。

## 1. 选择：分类静态介绍为主，交互画廊为辅

Apps 的系统应用区先回答用途和典型操作；目录采用 VitePress 静态图文卡片与逐项阅读页。28项各有真实运行图，来源、日期和数据性质另记。现有 /ui/demos/ 保留为开发体验入口，但不自动嵌入或作为所有系统应用的产品介绍。

理由：有后台、文件系统、系统状态或外部服务的应用不能靠加载前端证明功能。静态介绍可索引、可分享、移动可读、离线部署无需业务服务；交互页面仅承担经验证的体验范围。

## 2. 页面层次与视觉

/apps（EN/ZH）：四主产品卡片 → 六类28项真实图文 → 生态与资料。DemoCatalog.vue嵌入AppsOverview.vue，单个H1；系统应用H2、分类H3、应用H4。每项用途和图直接可见，原生details收三步操作、条件、范围、截图日期/形态、源码；可键盘展开和放大图片，静态SSR包含完整文字。
稳定#demo-<slug>定位且自动展开；深链清除过滤避免隐藏目标，普通查看仅展开所选卡片。桌面3列/平板2列/手机1列，六类筛选本地执行。四主产品保留独立阅读页，Demo不另开阅读专题。
内容单源website/.vitepress/theme/data/demos.json。原/apps/demos/和28项详情路由及ZH镜像由generate-demo-pages.mjs生成58个兼容跳转：当前语言/apps#system-apps或#demo-<slug>；SSR兜底链接、canonical与noindex、meta refresh和SPA跳转；--check检测漂移。产品入口直接指向概览锚点，不依赖旧路由。

## 3. 三种展示形式与验收门

| 形式 | 展示内容 | 上线条件 |
|---|---|---|
| 实图介绍（默认） | 界面、典型操作、运行条件 | 确认真图/版本/后端/样例数据；文本不借截图承诺未验证能力 |
| 无服务在线体验（选择性） | 真正可在静态网页操作的版本 | 新浏览器上下文下拦截全部业务API，验证关键操作/重置/错误/主题/卸载；不出现无效服务按钮 |
| 演示数据版（后续少量） | 明确的 fixture 操作流程 | UI标明演示数据、无真实服务、重置与持久化范围；fixture与正式数据通路分离并验证 |

依赖后台的应用保留实图、用途与本地运行条件，不先制作28套 mock。优先评估计算器、待办、小型游戏等状态可自足的例子；这是验证优先级，不是已经可静态使用的结论。天气/聊天/数据库/文件/终端/监视器不可用假数据冒充真实服务。不要直接编辑网站 public 中生成的 gallery bundles。

## 4. 数量与来源口径

当前 auto-os gallery registry 有35项，含001–010基础/组件示例及AutoEdit等产品；网站打包的旧 registry ID 与当前源有差异（012-stopwatch/025-dashboard vs 012-clock/025-sys-monitor）。桌面图标精灵槽、mapping 别名、apps.manifest 注册项也不是同一集合。browser预留位不算已完成应用，tetris等别名去重。

下表为用户所说“约28个”的**展示候选清单**，每项源码目录已查存在；它不是28项已集成/完整可运行/无需后台的产品清单。筛选发布目录时去除重复用途、按实际状态计数。四主产品独立介绍，不再用它们补足系统数量。011–031等数字是稳定源码ID，不按行号生成链接。

## 5. 28项候选与验证任务

“核查”列是下一轮验证任务，不是当前环境下已验证的依赖结论。所有候选初始展示形式均为实图介绍；通过第3节门禁才能加入在线按钮。

| # | 候选/用途 | 源码位置（仓/相对目录） | 交互前需核查 |
|---|---|---|---|
| 1 | 计算器 | auto-lang: examples/ui/011-calculator | 运算、错误输入与重置是否纯本地 |
| 2 | 时钟 | auto-lang: examples/ui/012-clock | 宿主时间源、计时与卸载定时器 |
| 3 | 待办 | auto-lang: examples/ui/013-todo | 增删筛选与保存是否调用服务 |
| 4 | 天气 | auto-lang: examples/ui/014-weather | 数据来源、网络与密钥；fixture范围 |
| 5 | 备忘录 | auto-lang: examples/ui/015-notes | 样例种子、编辑/保存与持久化 |
| 6 | 日历 | auto-lang: examples/ui/016-calendar | 日期操作与事件数据源 |
| 7 | 聊天 | auto-lang: examples/ui/017-chat | 会话/消息/模型后台，禁冒充真实回复 |
| 8 | 图书阅读器 | auto-lang: examples/ui/018-book-reader | 书目与正文资源、导航 |
| 9 | 视频应用原型 | auto-lang: examples/ui/019-video-app | 与播放器是否重复；资源/路由/后端 |
| 10 | 音乐播放器 | auto-lang: examples/ui/020-music-player | 真实媒体播放、浏览器权限、资源 |
| 11 | 博客阅读器 | auto-lang: examples/ui/021-blog-viewer | 本地文章与远端加载差别 |
| 12 | 看板示例 | auto-lang: examples/ui/022-kanban | 样例状态/持久化；区别auto-kanban产品 |
| 13 | RealWorld | auto-lang: examples/ui/023-realworld | 登录、文章、路由、服务完整性 |
| 14 | 图表 | auto-lang: examples/ui/024-charts | 预置数据与可操作项；区别图表组件画廊 |
| 15 | 系统监视器 | auto-os: apps/025-sys-monitor | 真监控接口；禁止静态图当实时指标 |
| 16 | 数据库工作室 | auto-lang: examples/ui/026-database | 连接/查询、读写范围与后台 |
| 17 | 文件管理器 | auto-lang: examples/ui/027-file-manager | 宿主文件API、只读展示/实际操作界限 |
| 18 | 启动器 | auto-os: apps/028-launcher | registry和宿主启动协议 |
| 19 | 照片图库 | auto-lang: examples/ui/029-photo-gallery | 样例媒体、目录/上传依赖 |
| 20 | 视频播放器 | auto-lang: examples/ui/030-video-player | 资源、播放/进度、后端与9的差异 |
| 21 | 图片查看器 | auto-lang: examples/ui/031-image-viewer | 资源装载、缩放、宿主文件入口 |
| 22 | 画板 | auto-lang: examples/ui/031-paint | 逐格绘制、历史与Storage保存；不承诺通用导出 |
| 23 | 俄罗斯方块 | auto-os: apps/036-tetris | 键盘/状态/计时器、焦点隔离 |
| 24 | 纸牌接龙 | auto-os: apps/037-klondike | 发牌、拖动、重置与持久化 |
| 25 | 扫雷 | auto-os: apps/038-minesweeper | 开格、标记、输赢、重置 |
| 26 | 系统日志 | auto-os: apps/039-syslog | 日志来源、隐私、服务API |
| 27 | 终端 | auto-term: app | 宿主进程与终端核心，不能静态执行本机命令 |
| 28 | 设置中心 | auto-os-config: . | 设置服务和写入范围，静态页不改真实配置 |

## 6. 逐项素材规范与后续顺序

每项保持稳定id/slug、分类、双语用途/操作/条件/状态/图注、sourceRepo/sourcePath、图片路径/尺寸/运行形态/日期。素材依据 docs/reports/p723-capture-catalog.json 冻结，标准见 [demo capture catalog](demo-capture-catalog.md)。
当前v0.5以真实图和静态介绍交付；v0.5.1安排网页桌面与app体验，UI Playground先验证少量Auto→Vue预览再扩展。此前在线体验的逐项验证原则保留，禁止本次增设桌面/app运行按钮或把开发可挂载标记升级成发布承诺。

## 7. 本轮实现

28项合并在双语/apps内静态图文与就地详情，旧58路由仅兼容锚点跳转，六分类本地筛选；无iframe、无业务API。25项新拍，Launcher/AutoTerm/Config复用已有真图并披露来源/版本；未开展双端全功能一致性验收。构建/SSR/链接/locale/断点/图像hash/键盘测试见 PLAN-723 报告。
