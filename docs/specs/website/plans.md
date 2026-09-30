# website — plans

> 纯表格：`| Plan | 标题 | 状态 | 归档 | 一句话沉淀 |`（scripts/spec-index.py 可解析）

| Plan | 标题 | 状态 | 归档 | 一句话沉淀 |
|---|---|---|---|---|
| 581 | playground-notes-foundation | ✅（reviewed→archived） | archive/ | Notes manifest 管线——scripts/build-playground-notes.mjs 四源确定性采集（vm-golden 460 全带期望输出〔.expected.out 优先/.expected.result 终值回退〕/aavm 158/books 裸 ```auto 围栏 634〔EN 源〕/demo 28 含 4 项目；parity 后置 582）→ public/playground-data/notes.json（gitignore、0.82MB 单文件、byte-identical 幂等）；prepare-content 末段接线三态生成 + --check 计数断言；website build 预存红 core.md 裸 `<prefix>`（P581-D1 上游）；债务 P581-D1..D4 |
| 582 | playground-notes-explorer | ✅（reviewed→archived） | archive/ | /playground（EN/ZH）换 Notes Explorer（52+5 组可浏览，无后端降级卡可浏览态完整）；旧 /playground/ SPA 退役 meta-refresh 重定向；prepare-content 裸尖括号通用转义器（P581-D1 预存红清偿，build 红转绿）；AutoFence 书页围栏 ▶ Run（autorun/收起还原/import 锁+笔记站链接，markdown fence 钩子）；playground-notes e2e 三断言 + playwright.config（webServer preview）；parity 语料入 manifest（51 条五家族）；债务 P582-D1..D4 |
| 713 | autoshell-website-evidence | ✅（reviewed→archived） | archive/ | AutoShell 双语页证据重构（三轮用户纠正修订）——r1 共享 AutoShellLanding.vue 五区组织（日常会话/数据管道/F2 多行脚本/自动化/快速开始）+EvidenceImage 键盘可达放大/Esc 返回+脚本标签方向键 Home/End/复制/样例下载；r2 恢复 ash-01/ash-2 原生截图+F1/F2/F3 形式图示区（原生图/PTY 重绘来源分述）；r3 产品/v0.5 双语主图共用 AutoShellPreview 原生彩色 ls 表格（CSS 视窗放大保留全图，禁模拟输出替身）；SD-01=docs/specs/website/project.md；Category A 零 Cargo |
| 715 | website-ui-refresh | ✅（reviewed→archived） | archive/ | 全站 UI 刷新——navigation.ts 单源双语导航/当前位置/共享 SPA 例外（a2ui 目录 URL 命中专题页为 sirv 既定，SPA 入口 index.html）；VitePress local search 合成热键接线+CodeMirror 捕获守卫；设计令牌+五断点×双语×深浅矩阵（StatCard 长 token overflow-wrap、导航 1200 断点、md 组件块内禁空行/4 空格缩进实证）；HomeDemo 三视图（csv_delimiter 金样三件套，预置/实时/降级诚实契约、同源 /api）；v05 ReleaseLanding 数据驱动重建（产品前置/理念 details/统计后移/launcher 实图）；EvidenceImage/ScreenshotGallery/AppLandingLayout；e2e 64 例；债务=0 新增，pre-existing 注记（/playground hydration、a2ui dir URL） |
