---
plan_id: PLAN-768
status: archived
feature_name: navbar-repository-icons
author: [Codex]
created_at: 2026-10-12
updated_at: 2026-10-12
plan_revision: 1
current_step: 3
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
- [x] T-01：lang-768 专用工作树更新 UnifiedNavbar 和 canonical project；AC-01/02。
- [x] T-02（依赖T-01）：构建、既有定向回归和链接/断点/主题/键盘临时实机校验、截图审读；AC-01/02。
- [x] T-03（依赖T-02）：提交、备份部署及线上核验，review/merge/规范投影/归档守卫清理；AC-03。

## 9. 复审记录
stage:new | PLAN-768 r1 | outcome:pass | next:work。现有用户明确指示覆盖范围，无需重复确认。

## 10. 待澄清事项
无。默认全站共用顶栏一并受益，桌面和手机都有入口。

T-01/02：共用顶栏两行内 SVG 外链；本地冻结依赖，构建141.02s EXIT0。12既有导航/搜索/菜单/主题/触控回归 + 1临时实机测试（24次布局访问+四路由SSR+双图标Enter实际弹出）通过。手机/桌面截图人工审读，零布局调整、无新依赖。durable docs/reports/p768-verification.json。

T-03交付验证：源码8f959f3fc，release20261012-8f959f3fc；HTTP/HTTPS源码字节匹配，线上中英×手机/桌面4项全通过，备份保留。执行已交付，review/merge簿记随后完成。

review PLAN-768:r1 | pass | reviewed_commit:8f959f3fc023d621325e3a4146ab07c338f889d9 | dependencies: p768-verification.json | frozen_delta: SD-01/project.md/hash在报告 | next:merge；同会话重建，AC-01/02/03 pass，零新债；docs/reports/p768-review.md。

## 11. spec-sync 与交付收据
PLAN-768:r1 completion_kind:delivered。
- prepared：原review8f959f3fc、SD-01/project blob hash冻结于p768-verification.json；重对齐复审72c20c4bd，delivery3e15e9037，网站树与验证/部署源8f959f3fc零差异。
- landed：同期cd61db302进入master，首个ff-only失败；错误命令链继续导致966118691提前归档/checkout清理，开发分支完整保留。恢复同一checkout后rebase至966118691；range-diff全等（8f959f3fc→72c20c4bd，f6be5b751→5b2170aa1），复审/投影刷新3e15e9037；master正确ff-only 966118691→3e15e9037。主检出网站树、canonical blob及账本bytes重新核对通过。没有重开归档，只完成已有交付证据的缺失落地；后续命令显式检测失败即停止。
- ledger_refreshed：B轨 guarded offline projection，localhost8080不可达；写前逐字节备份，852旧项/未知字段均不动，仅P768-1/P768-R/P768-Report三个新条目；六段读回/引用/hash和撤回三条后原字节全等验证通过。旧投影无version字段，兼容根形状保持；详细前/后sha在p768-deposit.json。
- spec-sync：website/project.md新增共用顶栏源码入口契约，website/plans.md新增768；spec-index.py26projects再生无语义diff。没有新架构/goal。
- deployment：release20261012-8f959f3fc，archive b0a691b8481fea8bc0d30cf2fd8b092db2c8c971b63e0a617e14e395cca1d629；HTTP/HTTPS bytes与保留十张原图hash验证；备份 /home/visus/auto-website-backup-20261012-8f959f3fc保留。12既有回归+1实机测试（24访问）+4线上通过。
- batch_due：768%5!=0、2026-10-12T04:08Z收据<48h；不触发批量；Category A零Rust，无Cargo/docs_gen。
- archived：docs/plans/archive/768-navbar-repository-icons.md；966118691提前归档已由实际落地后的本收据矫正，Plan/报告/账本均已在master Git。
- cleaned：守卫与实际移除后补记。
