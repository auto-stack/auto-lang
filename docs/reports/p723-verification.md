# PLAN-723 验证材料

- Contract: PLAN-723:r1，主检出docs/plans/723-website-demo-introductions.md；开发worktree D:/autostack/.wt/lang-723/auto-lang，branch plan-723-dev。
- 实施基线：ce280b0a3f733a3119d32b57f0e7995e86b1f674（合入已复审722；master未落网站WIP）。前序reviewed722=bb8bcefd389589f0522fc0b23951843bb975a8ab；master创建基线42fa2e0c0dfc666001f591f69fbe7ce5d688e06e。
- 内容同源：demos.json → generate-demo-pages.mjs →58个双语页面；28候选(repo,path)集合与p722-source-baseline.json完全一致，未换项/删项。六类数量6/3/6/7/3/3。
- 实图：28 PNG，共约8.9MB；25新拍，Launcher/AutoTerm/Config三项复用。捕获源、原始/发布文件SHA-256、尺寸、日期依据、样例数据、源文件hash和动作收据见p723-capture-catalog.json。公开PNG逐个视觉审查，不接受加载/接口错误/空白/断图。
- AutoTerm留存Rust原生图对应2026-09-22素材commit6ff3bad0ff0f780e81e7c9c13af2f0b3f8902040；当前本地VM App.Tick报future_all/race导致空会话，失败图拒收、源码不修补，正文公开旧图形态/日期。登记P723-D1，不冒领新版本全功能/双端验收。
- AutoEdit用户批准PixPin主图与原AutoShell ash-01分别接入概览和EN/ZH专题；Musk/Jade无假图/占位框。AutoShellLanding/Preview与website/public/v05零diff。

## 构建与自动验证

Category A：未改crates/** Rust源码，未运行Cargo tests/docs_gen。拍摄编排中的临时Rust宿主只链接既有auto_lang库提供真实media/photo服务，未新增产品Rust代码。二进制来源commit未证明，hash作为运行身份。

- npm ci --ignore-scripts --no-audit --no-fund：成功。
- npm run build：初次157.24s，最终155.37s通过。既有Auto语法高亮txt兜底及>500KB chunk提示保留，不宣称零历史提示。
- node scripts/generate-demo-pages.mjs --check：58页通过。
- python -m py_compile scripts/p723-capture-vm.py：通过。
- AUTO_WEBSITE_TEST_PORT=4236 npx playwright test apps-introduction.spec.ts site-ui.spec.ts desktop-showcase.spec.ts os-ai-introduction.spec.ts spa-routes.spec.ts --workers=2：88 passed（3.5m）。含原指南脚本交互、OS/AI介绍与搜索、desktop展示、v05、全局导航/主题/SPA入口回归。
- 新目录7项最终全部通过（含新端口完整重跑ZH详情及冻结来源断言），与88项旧检查合计95个唯一检查。覆盖28素材hash/冻结来源集合，EN/ZH六分类与键盘放大回焦、56详情图片/互链/大纲/源码入口、580次目录和详情布局（29路由×5宽度×2主题×2语言）、无业务API请求。启动时build未完成造成的一次sirv alias 404与复跑依据见p723-review.md。

初次23例运行：19通过，4个中文旧介绍断言把新增“查看原图”PNG链接误当必须带/zh/的页面链接。校正为共享PNG资源豁免，页面链接locale断言保持。修正后88回归全部通过，不降低链接/图像资格。

## 视觉与现场整理

28张发布图逐图检查：真实应用身份、有效内容/资源、无阻塞错误，内置天气/文章/消息/表/日志有配文说明；Photo扫描实际公开文件，Video实际HTML视频解码至2s，Paint和Minesweeper用真实控件动作，File Manager地址栏进入专用目录。时钟与日历区分实际时间/样例初始日期。

56阅读页及20个目录宽度/主题/语言矩阵图在ignored .p723-runtime/pages留作当前现场；人工查阅中英目录、手机浅色目录、深色图文详情、主应用概览，截图只做复核，不额外提交几十MB回归图。

旧desktop-showcase测试回写p720验证图与manifest，验证后还原，旧产品/证据不纳入723改动。拍摄进程3364/9367、9368、4231/4232/4233只停止本计划PID，经命令/父进程/监听端口验证；其他会话进程保持。新网站预览4235独占，旧4227预览保持。

拍摄auto gen工具自动建立030的deps/stylekit junction。收尾guard发现，验证链接路径位于本计划捕获目录、目标为普通stylekit副本后仅删除reparse entry，再复制真实文件；未穿透删除目标，未移除worktree。Git Bash wt-guard最终clean，无任何reparse point。后续生成捕获工程须先物化普通deps目录；禁止依赖链接。

## 准备规范增量

四个canonical Spec候选仅在开发分支准备（project/application-introductions/application-demos/demo-capture-catalog），未更新master ledger或宣称merge；p723-spec-delta.json记录四文件SHA-256，p723-spec-delta.patch保留修改差异，新文件完整内容随提交保存。touched_goals=[]：本轮为现有website介绍范围的素材/内容交付，不新建顶层Goal组件。
