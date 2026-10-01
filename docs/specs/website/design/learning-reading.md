# 学习入口与阅读导航

> PLAN-718 r1 分阶段交付：本页记录 T-01..T-04 已实现的契约；按需加载、失败重试和最终验收仍由在途计划负责。

- 网站继续使用 VitePress 默认阅读布局。`website/content/learning-navigation.mjs` 是学习入口作者数据，生成器与 `LearningHub.vue` 共用；四个中英 docs/books 根页在静态 HTML 中提供说明和真实入口。八本教程按主书和已有语言背景组织。
- 内容与侧栏由 `website/scripts/prepare-content.js` 生成，生成页不直接提交。书籍源解析顺序为 `AUTO_BOOK_ROOT`、相邻 book、主检出相邻 book。学习入口在生成时校验目标；语法、路线图和迁移指南纳入根级读者文档白名单。
- 文档侧栏优先提供 Auto Tour；每本书使用独立路径侧栏，章节顺序由 SUMMARY 或既有章节文件顺序提供。上一页/下一页限于当前书；学习根页关闭无语义的下一页。章节显示本书目录入口。
- 原生目录与阅读控件按当前语言显示。移动目录在统一顶栏下方固定，计算偏移时包含顶栏底边框；锚点额外让出目录栏高度。
- 编辑链接使用生成页到真实作者源的映射：docs/website 对应 auto-stack/auto-lang 的 master，书籍对应 Gitee auto-stack/book 的 master。纯生成入口关闭编辑链接；测试结果、构建缓存和依赖目录不能进入作者映射。
- AutoFence 操作栏置于普通文档流，避免覆盖原始代码和原生复制按钮。运行/收起、复制标题、模块依赖提示随语言切换；锁定示例链接到对应语言的 Playground。复制保留源码，运行按钮通过唯一 aria-controls 关联面板。

本阶段仍保留同步运行器导入；不能据此声称静态页面已停止加载编辑器。T-05..T-08 的按需加载、错误恢复、完整断点与最终截图验收尚未完成。

证据：`docs/reports/p718-website-phase-landing.md`；阶段回归：`website/tests/phase-718-landing.spec.ts`。
