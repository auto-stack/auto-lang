# PLAN-723:r2 应用概览合并复审

2026-10-01 · stage: review · outcome: **pass** · next: merge（未执行）。

## 基线与授权

- reviewed_commit: `137bc5557855c7757a0f3754ed4466226fa752f3`；base_commit: `a35a908a70ffe38abe0fa051f0465cb2270c9435`。工作树D:/autostack/.wt/lang-723/auto-lang，plan-723-dev；复审时tracked clean。
- 用户查看r1后明确要求28 Demo合并进应用概览，四主应用独立详情保留。主检出723合同已修订r2，重开T-03/05/06，保留素材成果和r1复审历史；不复用r1独立阅读页验收作为新结构的通过依据。
- 按auto-plan-review在本实施会话从已提交代码、测试断言、实际画面与冻结Spec重建；没有第二独立评审者。722依赖仍为bb8bcefd389589f0522fc0b23951843bb975a8ab，r1实施19beb3046fc3d888000514e651071ad4f2748f8c已复审。
- 28项demos.json、PNG、捕获清单未改。素材来源/模式/原始文件hash与3项留存图版本继续沿用p723-capture-catalog.json，当前图像资格和冻结来源集合再次检查。

## AC → tasks → evidence

| 验收 | 实施与验证证据 | 结论 |
|---|---|---|
| AC-01 → T-04,05 | 四主产品卡片及4个中英独立阅读页保留；AutoEdit/Shell原图、Musk/Jade素材等待边界不变；主图与guide/v05 PNG无diff；apps-introduction及site-ui复核原生图、指南tabs、复制、下载 | pass |
| AC-02 → T-01,03,05 | AppsOverview直接嵌入DemoCatalog；/apps和/zh/apps的SSR包含28项用途、三步流程、条件、范围、图片与源码；原生details就地展开，不再打开单独Demo阅读页。逐项按Enter展开检查正文与3条流程，56旧正文改为兼容跳转，不丢内容 | pass |
| AC-03 → T-01,02,05 | 元数据测试再次校验28唯一ID/slug、冻结(repo,path)集合、PNG hash/尺寸；全部缩略图加载。r1全部实图人工审查在图像字节/清单不变条件下复用；25新拍/3留存证据不重新冒领日期 | pass |
| AC-04 → T-03,05 | 概览单H1、系统H2/分类H3/条目H4、六类筛选、结果计数、原生details键盘展开、Enter图片放大/Esc回焦、locale切换；58旧目录/单项URL跳转本语言锚点；直接深链和筛选后hash改变自动恢复全部列表并展开目标；卡片#链接测试通过 | pass |
| AC-05 → T-03,05 | 介绍不发业务API、无iframe；无需应用后台即可展示完整内容。v0.5.1与UI Playground后续方向保持，没有本轮桌面/app运行入口 | pass |
| AC-06 → T-05,06 | 冻结实施build177.57s、70项检查全绿、20个新概览布局状态中全部28项展开无溢出、旧主应用五断点/双语/主题回归、真实页面视觉审查、生成58跳转核验、四Spec hash、git diff --check、wt-guard clean | pass |

## 可复现检查与视觉

Category A；无crates/**或应用源码/媒体改动，不运行Cargo tests/docs_gen。

- `npm run build`初次187.22s通过；完成最后布局修订并冻结实现后，`npx vitepress build`最终177.57s通过。后一次构建期间没有代码变更；保留已有Auto高亮txt兜底、Browserslist数据提示和chunk提示，未以新错误忽略方式通过。
- `node website/scripts/generate-demo-pages.mjs --check`：`Checked 58 bilingual compatibility routes; introductions live in /apps`。
- `AUTO_WEBSITE_TEST_PORT=4242 npx playwright test apps-introduction.spec.ts demo-catalog.spec.ts site-ui.spec.ts spa-routes.spec.ts --workers=2 --output=.p723-runtime/r2-test-results --reporter=list`：**70 passed (3.2m)**，新端口在最终build结束后启动，无启动alias缺失或测试失败。
- 28项×双语就地正文/源链接/图像加载；56旧单项及2目录跳转；直达#demo-terminal、筛选隐藏后#demo-settings和点击#demo-calculator均展开定位。SSR兜底链接及noindex存在，兼容页不重复渲染卡片；生成器校验meta refresh、canonical和SPA跳转模板。
- 360/390/768/1024/1440×双语×深浅，展开全部28项后验证无横向溢出，3/2/1列符合断点。人工查看中文桌面目录、打开计算器详细介绍、390手机详细介绍及矩阵图；记录当前页内阅读，没有空白素材或阻塞覆盖。
- `git diff a35a908a70ffe38abe0fa051f0465cb2270c9435 HEAD --check`通过；相对base的website/public、crates、AutoShellLanding/Preview零diff；wt-guard clean，无reparse point。r2日志及截图在ignored .p723-runtime/{site-build-r2-final.log,r2-tests.log,r2-pages/}；本收据保留命令和最终结果，清理后可复现。
- 仅验证并重启本计划旧4235 preview PID6988，命令路径/端口匹配723后才停止；4235更新为最终产物，其他会话进程不动。预览http://127.0.0.1:4235/zh/apps#system-apps。

## 规范、缺项与边界扫描

Spec输入为r2 base上四个已准备Spec；修改后见p723-r2-spec-delta.json，四文件hash重新核对全部匹配。p723-r2-spec-delta.patch SHA-256 `584a552c2c13d79639623c8b354cd2c315af672c8b6bf4fda0f41faedd3e16a5`：以最初ce280b0a3f733a3119d32b57f0e7995e86b1f674为基准冻结三canonical文件的完整修改；新增demo-capture-catalog.md完整文件与hash另存。历史r1 delta和复审收据保留，不改写历史。

SD-01..04将页面位置/就地展开/稳定锚点/兼容跳转描述为现状；相关网站体验设计同步/apps#system-apps与四主产品独立扩展方向。supersedes_spec_components为project.md、design/application-introductions.md、design/application-demos.md；new_spec_components为design/demo-capture-catalog.md，均在docs/specs/website/下。touched_goals=[]：现有网站介绍信息结构调整，没有新增顶层Goal。只在worktree准备，master canonical/ledger未发布。

没有删减候选、操作步骤、运行条件或状态信息；折叠内容仍在SSR和当前页面，可键盘展开。不存在28套mock、重绘图替换、假在线按钮或未经授权的应用修补。P723-D1的当前本地AutoTerm VM拍摄诊断仍属素材来源限制，已明示旧原生图，并非本轮新增或隐藏缺项。四主应用未来扩充详情内容不冒领为当前已取得新截图。

本收据提交只增加复审记录；通过绑定以上实施SHA，未将后续任意实现自动纳入。计划r2可以进入merge，本轮保留worktree和预览，未合入master或归档。
