# 学习入口与阅读导航

> PLAN-718 r1 交付（合并收据在案）：学习入口、阅读导航、章节衔接、双语控件、按需加载与加载失败恢复的全部现行契约。

- 网站继续使用 VitePress 默认阅读布局。`website/content/learning-navigation.mjs` 是学习入口作者数据，生成器与 `LearningHub.vue` 共用；四个中英 docs/books 根页在静态 HTML 与本地搜索索引（生成器输出的"全部入口"markdown 清单）中提供说明和真实入口。八本教程按主书和已有语言背景组织，书籍卡标题链接书首页、CTA 链接第一章入口。
- 内容与侧栏由 `website/scripts/prepare-content.js` 生成，生成页不直接提交。书籍源解析顺序为 `AUTO_BOOK_ROOT`、相邻 book、主检出相邻 book（候选以 tapl 存在性验证）。学习入口在生成时校验目标（不可解析即构建失败）；语法、路线图和迁移指南纳入根级读者文档白名单。SUMMARY 解析同时接受列表项与 mdbook 裸顶层链接。
- 文档侧栏优先提供 Auto Tour（Start here/开始使用）；每本书使用独立路径侧栏（`/books/<id>/` 最长前缀），章节顺序由 SUMMARY（含裸顶层链接）或既有章节文件顺序提供。上一页/下一页限于当前书，首章无上一页、末章无下一页；学习/教程根页关闭无语义的下一页。章节页显示本书目录入口（ReaderContext）。
- 原生目录与阅读控件按当前语言显示（outline/sidebarMenuLabel/docFooter/returnToTopLabel）。移动目录在统一顶栏下方固定，计算偏移时包含顶栏底边框；锚点 scroll-margin 按"顶栏+目录条"两层实际高度让位（`--site-reading-offset`）。窄屏目录弹层触发按钮触达 ≥44px（弹层根节点垂直 padding 清零，条高保持不变）。
- 编辑链接使用生成页到真实作者源的映射（生成文件 `.vitepress/config/author-source.ts`，VitePress 序列化 config 函数会丢闭包，故 pattern 自包含）：docs/website 对应 auto-stack/auto-lang 的 master，书籍对应 Gitee auto-stack/book 的 master（`.cn.md` 译本感知），website 手工页映射仓内文件。纯生成入口关闭编辑链接；测试结果、构建缓存、依赖目录与 Playwright 产物目录不能进入作者映射。
- AutoFence 操作栏置于普通文档流，避免覆盖原始代码和原生复制按钮。运行/收起、复制标题（围栏渲染按 `env.relativePath` 构建期本地化，locale 级 markdown 配置不参与 SSG）、模块依赖提示随语言切换；锁定示例链接到对应语言的 Playground 且不渲染运行器。复制保留源码，运行按钮通过唯一 aria-controls 关联面板（SSR 稳定 ID）并支持键盘开合，收起时面板内焦点归还开关。
- 运行器（CodeMirror 链）按需加载：主题注册边界以异步组件拆分（CodeView/ScriptShipView/AutoFence/AutoPlayground/NotesExplorer），SSR 正文保留；冷访问首页、docs/books hub、未打开运行器的书页不请求编辑器 chunk、不调用 `/api/run`。失败加载给出本地化说明与重试/收起，原文在等待与失败态保持可读；重试经三个真实入口 shim 换 module-map 键（失败 URL 会被 module map 缓存，同 URL 重试秒拒）。`@codemirror/*` 等依赖经 vite `resolve.dedupe` 保持单实例。

证据：`docs/reports/p718-website-learning-reading.md`（基线+回执）；视觉复核 `docs/reports/p718-website-ui/VISUAL-REVIEW.md`；行为回归 `website/tests/learning-reading.spec.ts`、`website/tests/reader-loading.spec.ts`；第一阶段复核（历史）：`docs/reports/p718-website-phase-landing.md`、`website/tests/phase-718-landing.spec.ts`。
