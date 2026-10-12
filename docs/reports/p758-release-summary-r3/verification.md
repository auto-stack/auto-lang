# PLAN-758 r3：v0.5 开篇投入与成果复核

日期：2026-10-11。实现：8f73dd56b59af4ab54f6f9d9ef4fcdc23defe470；基面：ada443b9cd753f233317c5045e47cea29f4fc643；website tree：7a61ef8d9a057fdb7ecb0cd0339c41818b25e21c；auto-down：895f8d0f9355c9f5ec3ce8fca268bdb768395846。

用户直接批准开篇最大更新/语言跨向生态宣传句，以及投入 1500 亿 Token、100 万 Rust 行、40 万 Auto 行、1w+ 提交和成果 1/2/3/4/28；“样式”按笔误改为“也是”。2 桌面是 Windows/Linux 原生应用桌面与 Windows/Linux 宿主中的 AutoOS 虚拟桌面；3 平台是 Web/Windows/Linux。双语 AutoOS 专题及发布正文同步该口径；独立发行版/OpenHarmony/自有内核仍为未来路线。数字标注作者 2026-10-11 提供的生态累计口径，不冒充本次独立代码计量或 Linux 实机验收。

| 验收 | 任务 | 证据与结论 |
|---|---|---|
| AC-01/02 | T-01/T-03 | r2 已落地改名、规范/兼容路由证据沿用；r3 未改应用路由/导航/README/旧名例外，实际开篇四应用使用 AutoJade。|
| AC-03 | T-02/T-03 | 六 PNG 与用户源文件逐字节/manifest SHA 再核对全部相同，inherited-assets-check.json；SHOT-09/P4 合同未改；既有画廊切换场景本轮再通过。|
| AC-04 | T-03/T-04 | 构建和目标检查完成，原始日志 gzip+hash 留存；三个当前 canonical 差异冻结，九目标投影草案更新；store 阻塞公开记录，不手改账本。|
| AC-05 | T-03 | 390/1440×EN/ZH 四定位场景通过：固定九章节顺序、导航顺序、时间线在大图前、规模卡在产品后、原生 hash 到达、手机浮层不推动正文且点击后收起。用户本轮明确授权投入摘要置于 hero。|
| AC-06 | T-05 | 数据/模板逐项对勘：4 投入值、5 成果值及名称、双语日期来源均正确；投入共用 releaseStats；后部旧当前数字已替换，日期绑定历史审计未改。五宽度两主题两语言无页面溢出；真实静态页和截图人工审读。|

最终构建：`node node_modules/vitepress/bin/vitepress.js build`，145.37s 成功。
最终检查：`CI=1 AUTO_WEBSITE_TEST_PORT=4225 npx playwright test site-ui.spec.ts os-ai-introduction.spec.ts --grep 'viewports|v05 release page|reduced motion|theme toggle|readable introduction, sources, outline and locale: /(zh/)?autoos/' --workers=2 --reporter=line --output=test-results-p758-r3-verified`，20/20 通过（18 发布/UI 场景 + 2 修改过的 AutoOS 双语专题）。既有测试断言不变，只在已有四定位用例添加 hero 截图记录。

修复过程也保留：初版 16/18，通过布局压缩后 17/18，缩短英文宣传说明后仍 17/18（390 英文 heading y=1296.75，高于 1266）；最终减少手机摘要组间距、来源间距并缩短状态说明后重建/复验全过。没有放宽定位断言、移除数字或以失败当通过。先前三个失败批次的原始日志亦列入 artifact-manifest.json；不拿开发服务页面替代静态产物检查。另一次构建期间整理旧验证输出导致 error-context.md 入口缺失，失败日志保存在 website-build-r3-interrupted-assets.log.gz；移走三批旧验证输出后从干净目录重新构建，最终检查仅使用该成功产物。

Scope/health：r3 不触碰 crates/test/auto 源码，Category A 不跑 cargo t/docs_gen/tf。六图片原字节不变；无新增实现调试输出；git diff --check 无空白错误。VitePress 存在既有未注册 auto 高亮提示，Playwright 的 NO_COLOR/FORCE_COLOR 提示不改变验收。当前修改只涉及发布摘要、两个 AutoOS 平台段落、关联规范/证据及截图记录；其他 main 元数据与临时文件保留。

规范：spec-delta.json 的三目标输入/输出以 CRLF→LF 规范化 SHA 绑定，reviewed-spec-delta.patch.gz 的压缩/解压 SHA 均记录；r2 冻结包保持历史原样。pending 清单保留 P732-1/P722-1 的旧名作为待更新源证据，累计九 canonical 目标，ledger 只读快照与未注册 writer 状态单列。

遗漏/延期/workaround 扫描：无新增交付遗漏或未经批准的延期。已有 P758-D1 仅投影/归档/工作树清理阻塞；暂未拍的其他产品槽沿用原合同，不以本次数字摘要声称已截图完成。同实现会话进行复审，以实际 diff、原始日志、hash 和页面重建结论，不声称独立 agent。
