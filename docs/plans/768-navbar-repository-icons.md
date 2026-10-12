---
plan_id: PLAN-768
status: drafting
feature_name: navbar-repository-icons
author: [Codex]
created_at: 2026-10-12
updated_at: 2026-10-12
plan_revision: 1
current_step: 0
total_steps: 3
supersedes_spec_components: [docs/specs/website/project.md]
new_spec_components: []
touched_goals: []
affects: [website]
---

# [PLAN-768] 顶栏源码仓库图标入口

## 0. 变更摘要
用户要求首页标题栏增加 GitHub/GitCode 两个指定仓库的 icon 链接。

## 1. 目标
共用顶栏双语/深浅/桌面手机均可见的两个源码入口；准确链接、清晰图标、键盘可达，保全既有导航与页面。

## 2. 架构方案
UnifiedNavbar 添加行内 SVG 图标链接，复用 nav-icon-btn；外链不做 locale 拼接、新页打开并 rel=noopener noreferrer。图标由 Simple Icons 的 GitHub/GitCode SVG 路径本地化，不引入依赖/远程图片。

## 3. 技术栈
Vue/VitePress、现有 CSS 与 Playwright。Category A，仅网站；不跑 Cargo/docs_gen。

## 4. 需求分析与背景调查
用户于本轮明确授权两个外链 https://github.com/auto-stack/auto-lang 与 https://gitcode.com/auto-stack/auto-lang；本会话本地提交/部署授权持续覆盖。既有 shared UnifiedNavbar 被全站采用，而默认 VitePress nav 被隐藏；不能只配默认主题 socialLinks。读 docs/specs/overview.md、website/project.md、navigation.ts、UnifiedNavbar.vue、site-ui.spec.ts。导航1200断点，360品牌只剩logo；新增两44px触控入口须不拥挤。仓库页面已通过 web 工具打开；仅按用户指定网址，无第三方写入。

## 5. 详细设计
在顶栏右侧保留 GitHub/GitCode 图标及 aria-label/title（双语仓库/新标签说明），SVG currentColor/aria-hidden。手机也直接看得到；若布局确有拥挤，仅收紧间距或相应断点，既有菜单/搜索/语言/主题仍可达。不改内页内容/源码仓库。

### 规范增量
| delta_id | operation | target | before/after | rationale | acceptance |
|---|---|---|---|---|---|
| SD-01 | modify | docs/specs/website/project.md | 顶栏无外部源码图标 → 双语共用顶栏两指定仓库图标/可达/外链 | 用户指定入口 | AC-01/02 |

## 6. 测试设计
专用工作树冻结依赖安装、AUTO_BOOK_ROOT 指向既有 book 后网站构建；相关 site-ui 导航/手机菜单/搜索/触控回归。临时浏览器校验（不新增镜像实现式永久单测）中英×360/390/768/1024/1200/1440×深浅：两个图标/链接/触控/键盘/无溢出，截图审读手机/桌面。提交后按既有 deploy-website.ps1 目标备份部署，HTTP/HTTPS源页字节与线上两外链核对。

## 7. 验收标准
- AC-01：首页及共用顶栏中英均有可识别 GitHub/GitCode icon，href 精确等于两个指定网址，不拼 /zh；链接新页打开且命名明确，可键盘焦点。
- AC-02：两主题六宽度可见无挤压/溢出，手机入口触控44px；搜索/菜单/语言/主题可用，构建与定向检查全通过。
- AC-03：本地提交/合入，备份部署线上准确外链；review/spec/归档/守卫清理有收据。

## 8. 执行步骤
- [ ] T-01：lang-768 专用工作树更新 UnifiedNavbar 和 canonical project；AC-01/02。
- [ ] T-02（依赖T-01）：构建、既有定向回归和链接/断点/主题/键盘临时实机校验、截图审读；AC-01/02。
- [ ] T-03（依赖T-02）：提交、备份部署及线上核验，review/merge/规范投影/归档守卫清理；AC-03。

## 9. 复审记录
stage:new | PLAN-768 r1 | outcome:pass | next:work。现有用户明确指示覆盖范围，无需重复确认。

## 10. 待澄清事项
无。默认全站共用顶栏一并受益，桌面和手机都有入口。
