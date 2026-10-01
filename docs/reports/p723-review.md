# PLAN-723:r1 复审收据

2026-10-01 · stage: review · outcome: **pass** · next: merge（本轮尚未合入master）。

## 基线与独立性

- reviewed_commit: `19beb3046fc3d888000514e651071ad4f2748f8c`，实施与测试配置已提交，复审时tracked clean。
- base_commit: `ce280b0a3f733a3119d32b57f0e7995e86b1f674`；前序722 reviewed_commit: `bb8bcefd389589f0522fc0b23951843bb975a8ab`。worktree `D:/autostack/.wt/lang-723/auto-lang`，branch `plan-723-dev`，由git worktree list确认。
- 合同来自主检出 `docs/plans/723-website-demo-introductions.md`，revision 1。用户明确授权本轮28项介绍与真实截图；没有改变验收标准或缩减集合。
- 按auto-plan-review重建检查；仍在实施会话中复核，没有独立第二会话/评审者。结论来自已提交diff、页面行为、断言、PNG及冻结清单，不把执行勾选当证据。
- 拍摄源码采样HEAD：auto-lang `ae5dcdd7928874dc468acc7880eff0cdb6bec037`；auto-os `73d02b50f535543bd635c6f6f49dad6835efff45`；auto-term `7a0d0e8b718fb260641f0d09ada95a7346771cd3`；auto-os-config `76970a6552ea9517bb132140814966ce91d0eb6c`。各源文件hash见捕获清单。采样HEAD不是二进制构建证明。
- 捕获清单SHA-256: `3580beb07e2febb3124060bfa1d11f1e2df111822ee0ff141752ebc92a7448a4`；722冻结来源清单SHA-256: `2edcf0c13118d1057549a335760994e4976976ef37934b06a3fadc283d2c2a98`。

## 验收映射

| AC → tasks | 代码/材料与复核方法 | 结果 |
|---|---|---|
| AC-01 → T-04,05 | AppsOverview.vue、EN/ZH AutoEdit/Shell阅读页；原始PixPin与发布图hash相同 `fc52cc6cee4818237fae74652f3391c64054061748b818ff68bcfbbc4528fd26`；Shell ash-01 `5c5512cc8eaaadfa810977f1b732b3db89b236952310a921c1bf2915b36f7d0c` 未改；Musk/Jade无图空框；旧ShellLanding/Preview与public/v05相对base零diff，88项触面回归 | pass |
| AC-02 → T-01,03,05 | demos.json的28个唯一id/slug与p722-source-baseline.json的(repo,path)集合完全相同；中英用途、三步操作、条件、范围、来源、图注；generate-demo-pages.mjs --check校验58页，56详情逐项加载/链接/语言互链检查 | pass |
| AC-03 → T-01,02,05 | p723-capture-catalog.json与28张public PNG的SHA-256、尺寸、运行形态、日期/复用依据；重新查看四张全部发布图接触表，核对界面身份/有效资源/非空内容；25新拍、3留存，未将失败空图或mock响应发布 | pass |
| AC-04 → T-03,05 | DemoCatalog.vue六类6/3/6/7/3/3、SSR全部28项、本地筛选、aria-pressed/live计数；EN/ZH键盘Enter放大、Esc关闭回焦；580布局检查中3/2/1列与无横向溢出，手机浅色目录人工检查 | pass |
| AC-05 → T-03,05 | 目录与56详情无iframe/业务API请求；静态源码/图片/相关介绍入口，v0.5.1在线体验明确为后续；未引入本轮桌面/app执行入口，既有开发画廊标注后台前置 | pass |
| AC-06 → T-05,06 | 最终build、88旧检查+7新检查、58页生成核验、56阅读页及20目录矩阵截图、git diff --check、四Spec候选hash、wt-guard clean；逐项AC及缺项扫描如下 | pass |

## 可复现验证与失败归因

Category A；未修改crates/**或应用源码，未运行Cargo tests/docs_gen。

- `npm run build`：最终155.37s通过。既有Auto高亮回退txt与大chunk提示保留，无新阻塞错误。
- `node scripts/generate-demo-pages.mjs --check`：`Checked 58 bilingual catalog and introduction pages`。
- `python -m py_compile scripts/p723-capture-vm.py`：通过。
- `AUTO_WEBSITE_TEST_PORT=4236 npx playwright test apps-introduction.spec.ts site-ui.spec.ts desktop-showcase.spec.ts os-ai-introduction.spec.ts spa-routes.spec.ts --workers=2`：`88 passed (3.5m)`。
- `AUTO_WEBSITE_TEST_PORT=4237 npx playwright test demo-catalog.spec.ts --workers=1`：6 passed，ZH详情检查因`/zh/ui`临时404失败。该预览在最终build尚未完成时启动，sirv启动时的目录alias不完整；相同URL在4235正常200，最终`.dist/zh/ui/index.html`存在。不是删掉链接或降低断言来通过。
- 待build结束，独立新端口启动：`AUTO_WEBSITE_TEST_PORT=4238 npx playwright test demo-catalog.spec.ts --grep 'frozen|/zh all 28 reading' --workers=1 --output=.p723-runtime/recheck-test-results --reporter=list`：`2 passed (1.4m)`，其中ZH28详情全检查1.4m通过，冻结来源集合加强断言通过。
- 合并证据得到**95个唯一检查通过**：88旧+7新；不是声称一次95项运行全绿。4237的另外6项页面/布局断言在此次复审未变；元数据检查增加的冻结来源集合断言由4238重跑覆盖，失败的ZH详情由4238完整重跑，不复用失败结论。580布局 = 29路由×5宽度×2主题×2语言，未宣称580条独立测试。
- `git diff ce280b0a3f733a3119d32b57f0e7995e86b1f674 HEAD --check`：通过。Git Bash `wt-guard.sh`：clean，无reparse point。预览4235的中文目录及`/zh/ui`最终均200。

运行原始日志位于ignored website/.p723-runtime/{site-build-final,regression-tests,demo-tests-final,demo-recheck}.log；本收据保存命令、结果、失败归因，可在清理后复现。截图与原始捕获文件hash、实际交互动作、媒体文件/服务身份在已提交捕获清单保存。

## 缺项、延后与替代扫描

- 28项没有漏项、合并替代或减少介绍；019门户与030播放器、022示例与独立产品、AutoTerm与AutoShell分开。Musk/Jade截图等待沿用用户约定，属于四主产品素材边界，未占用28项名额。
- Launcher/Config原生已批准素材拍摄时间无法确定，captureDate:null、reusedAt与来源单独记录。AutoTerm留存2026-09-22原生验证图，素材commit `6ff3bad0ff0f780e81e7c9c13af2f0b3f8902040`。复用符合原合同，不冒充新拍。
- **P723-D1（medium、非本轮静态介绍阻塞）**：当前本地auto.exe运行AutoTerm VM出现App.Tick future_all/race无效列表，未产有效会话；失败图拒收，未修应用实现。主检出KNOWN-DEBT-AND-RISKS.md已登记。后续匹配工具链/修复后重拍；此次pass不证明当前VM终端功能或28项双端全功能一致性。
- 天气/聊天/文章/数据库/syslog内置数据都有文字说明；照片为实际扫描公开截图，视频为实际解码WebM测试片段，音乐扫描WAV测试音。测试资源不冒充外部服务或用户文件；未以28套mock替代实际应用。
- 拍摄生成器曾自动建立deps/stylekit junction，收尾只删除已核实的链接entry并复制普通目录；未删除目标/移除worktree。最终guard clean，后续拍摄先物化普通deps。详见p723-verification.md。
- 未发现遗留debug输出、未授权产品源码修补或新运行按钮。现有高亮/构建提示不假称清零；源码采样与二进制来源的不确定性已披露。

## 规范增量核验

SD-01..04对应项目现状、主图资格、六类28项双语静态介绍及素材可追踪契约；阅读四个prepared Spec并逐一核验p723-spec-delta.json全部hash匹配。既有guide、共享SPA语言例外和v0.5.1边界保持；文本描述当前行为及长期素材规则。delta.patch SHA-256: `0206ee23a5e4afd5e1041cb438a683852a538f94dcbb82a083a69b2ed9e49845`（统一零上下文，三修改文件；新增Spec完整文件随提交保存）。Spec输入为base_commit上的canonical文件和722已复审候选，不在复审中发布master canonical或更新ledger。

supersedes_spec_components: docs/specs/website/project.md、docs/specs/website/design/application-introductions.md、docs/specs/website/design/application-demos.md。new_spec_components: docs/specs/website/design/demo-capture-catalog.md。touched_goals=[]：交付现有website介绍范围，不创建顶层Goal。

实施commit通过本次复审；随后本收据提交仅增加验证记录，不扩大已审实现的范围。网站和规范候选仍在plan-723-dev，722基础也未由本轮合入master。预览保留供用户查看。
