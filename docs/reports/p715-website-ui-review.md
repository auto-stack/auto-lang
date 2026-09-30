# PLAN-715 全站 UI 改进验证报告

- 日期：2026-09-30
- worktree：`D:/autostack/.wt/lang-715/auto-lang`（branch `plan-715-dev`）
- 验证基线：T-01 报告（`p715-website-ui-baseline.md`）+ 本报告
- 最终 commit：见提交历史（T-01..T-07 三笔实现提交）

## 1. 构建与测试结果

| 门禁 | 命令 | 结果 |
|---|---|---|
| 构建 | `npm run build`（website） | **通过**（61.6s，多轮迭代最终态） |
| e2e 全量 | `AUTO_WEBSITE_TEST_PORT=4186 npx playwright test` | **64/64 通过**（新增 site-ui 39 例 + home-demo 12 例 + 既有 spa-routes 9 例 + playground-notes 4 例；site-ui 含 md 缩进代码块回归锁） |
| 依赖安装 | `npm ci`（website）、`npm install`（packages/auto-playground-vue） | 通过 |
| 补丁卫生 | `git diff --check` | 干净 |
| Rust 门禁 | 按 Category A 不适用 | 未跑 cargo（零 Rust 改动，符合计划 §6） |

服务隔离：全部 e2e/截图在 **4186** 端口（`AUTO_WEBSITE_TEST_PORT`，strictPort），
`baseURL` 与 `webServer` 同源；4185 曾被本计划基线截图的残留 preview 占用（Windows 下
shell:true 的 kill 不杀子进程），e2e 复用该旧进程的教训已写入测试脚本注释——后续以
4186 起步、占用即顺延。

## 2. 关键修复实录（实现过程中发现并解决）

1. **md 中 Vue 插槽大块内容会被 markdown 块规则切断**（`apps/autoui/index.md` missing end tag 实证）：
   AppLandingLayout 收敛为 hero 骨架组件，章节内容由页面顶层排版；组件调用压成单一
   HTML 块（无内部空行）。autodown 多行 `<pre>` 内容单行化（`&#10;`），autoui 迁移时
   丢失的 features-section 闭合 div 已补。
2. **导航 1024px 溢出 39px**（e2e 实测）：桌面导航+操作区总宽超出 1024 视口——桌面断点
   从 1024 提到 1200，1024-1200 段走手机菜单（44px 触控）。
2b. **md 插槽提升后的 4 空格缩进被 markdown 当代码块**（e2e 矩阵实测 automusk/autodown
   EN 页溢出 449/4055px，页面尾部出现字面 `<ShowcaseSection` 源码）：六页 `</AppLandingLayout>`
   之后内容统一去前导缩进 + 新增字面源码泄漏回归锁。
2c. **StatCard 长斜杠 token 溢出**（矩阵实测 autodown-en 390px 残留 449px）：
   `parser/links/...linkgraph…` 为 393px 不可断 token——`.stat-desc` 加
   `overflow-wrap: anywhere`，`.landing-page` 加 `break-word` 兜底。
3. **宿主 OS 中文 locale 泄入测试浏览器**：`navigator.language=zh` 触发首页自动跳转，
   `/` 的测试整个跑到 `/zh/` 上——playwright 配置钉住 `locale: 'en-US'`。
4. **4185 端口被残留 preview 占用**：sirv 启动时固化文件清单，重建后新资产 404——杀旧
   进程后恢复；教训已注记。
5. **A2UI SPA 目录 URL 撞 VitePress 专题页（pre-existing）**：`/ui/a2ui/` 优先命中
   `dist/ui/a2ui.html`（sirv extensions 对 dir URL 的解析，master 同样行为——dist 产物
   与 preview 服务器均非本计划改动）。共享 SPA 规范入口改为 `/ui/a2ui/index.html`
   （与导航一致），spa-routes 测试同步更正并注记。
6. **/playground hydration 告警（pre-existing）**：仅该页出现（AutoPlayground 运行时态），
   console 卫生测试如实豁免此一条并注记；本计划新增组件在所有页面无 hydration 告警。
7. **搜索首次打开需构建全站索引**：`.results` 在 minisearch 就绪前带 hidden——测试放宽
   到 20s；合成 Ctrl+K 依赖水合完成的竞态用轮询重试防御。

## 3. 验收标准对照（AC-01..14）

| AC | 证据 |
|---|---|
| AC-01 双语共享导航 | site-ui：zh 首页链接全 /zh、语言切换保路径往返、Logo 回 locale 首页、当前位置 aria-current、SPA 无假路由（shared 标注+index.html 入口） |
| AC-02 搜索 | 三类页面（/docs/、/zh/docs/language、'/'+zh）+ 固定查询 ownership/所有权/AutoShell 命中有效页面；手机可开 |
| AC-03 键盘/状态 | 下拉 aria-expanded/Esc 回焦、菜单 aria-expanded/Esc 收起+焦点返回+导航收起、主题钮 aria-pressed、放大 dialog Enter/Esc/焦点返回 |
| AC-04 文字层级 | 正文 16–17px、行高 1.75（token 化）；文档页 sidebar/阅读宽度/outline 保全（VitePress 默认+scroll-margin 调整）；五断点矩阵无告警 |
| AC-05 无裁切/溢出 | 五断点（360/390/768/1024/1440）× 6 页 `scrollWidth` 断言 + hero 按钮 bbox 在视口内；修复 1024 溢出与 autodown 390 溢出 |
| AC-06 深浅/触控/reduced-motion | 主题切换+切页保持；360px 导航钮 ≥44px；reduced-motion 下内容 opacity>0.9 |
| AC-07 首页三视图 | csv_delimiter Auto/输出/Rust(a2r 金样) 三视图内容断言；classify.ash 无 Rust 标签（诚实）；证据链接指向仓内金样；ash 原生主图（AutoShellPreview）保留 |
| AC-08 运行/转译诚实 | 无初始 API 请求断言；mock 成功/失败/超时(10s)/取消/静态降级全覆盖；同源断言；stdout/stderr 分列；预置输出标注来源不冒充实时 |
| AC-09 发布主图早见 | EN/ZH × 390/1440：hero 图 top < 1.5×视口高（改造前 9824px@390）；hash 直达+点击导航+当前章节高亮+标题不被遮挡 |
| AC-10 理念/统计/内容保全 | stats 后移断言（desktop 先于 stats）；理念 details 默认收起/可展开；TODO 占位清零断言；launcher 实图；kanban 双臂对照；旧内容映射逐项见 baseline §6 |
| AC-11 图片交互 | 画廊 tab 切换换图；放大 Enter 开/Esc 关/焦点返回；桌面 4 帧（dark/light/tour/launcher）+ 对比图/深浅选择器无占位 |
| AC-12 应用共用骨架 | 6 页迁移 AppLandingLayout hero 骨架+共享视觉；ash 专题回归（native 总览、F1/F2/F3、脚本 tab 切换、复制剪贴板、下载链接全断言） |
| AC-13 构建/e2e/隔离 | 见 §1 表；63/63；独占端口同源 |
| AC-14 证据与 SD 对应 | 本报告 + baseline 报告 + `final-*.png`/`final-report.json` + 前后对比（`baseline-*`）；SD-01..06 见 §4 |

## 4. Spec delta 证据映射

| SD | delta | 证据落点 |
|---|---|---|
| SD-01 | project.md 补共享 locale 导航/当前位置/SPA 例外/搜索范围 | navigation.ts 单源实现 + AC-01/02/03 测试；SPA 例外=shared 标注+index.html 入口（a2ui 实证注记） |
| SD-02 | 新 ui-presentation.md：排版/主题/断点/offset/键盘规则 | style.css/landing.css 令牌 + AC-04/05/06 测试 + 五断点×深浅截图矩阵 |
| SD-03 | project.md 补轻量首页演示契约 | home-demo.ts 数据契约（预置/实时/降级）+ AC-07/08 测试 |
| SD-04 | ui-presentation.md 补产品前置/理念展开/内容映射 | ReleaseLanding 结构 + AC-09/10 测试 + baseline §6 映射表 |
| SD-05 | project.md 单页图片规则推广 | EvidenceImage 扩展 + ScreenshotGallery + AC-11/12 测试；ash 原生图契约零改动 |
| SD-06 | ui-presentation.md 补双语/主题/断点矩阵与测试服务隔离 | p715-shot.cjs 矩阵 + AUTO_WEBSITE_TEST_PORT + locale 钉定教训（§2.3） |

## 5. 截图矩阵

- `final-report.json`：24 页 × 390/1440 × 深/浅，96 行整页图 + `heroImgTop`/`hasHScroll` 测量。
- `final-*` 与 `baseline-*` 同名对照（改造前后）。
- 复测结论（对照 baseline 两大问题）：
  - autodown 390 横向溢出 449px → **已修复**（final 全矩阵 96 行 problems=0，含 360px 断点 e2e；heroTop 533@390、386@1440 均 < 1.5×视口）；
  - v05 首图 top 9824px@390 → **已修复**（desktop-hero eager 进首屏，top < 1.5×视口，e2e 断言）。

## 6. 已知边界与遗留（如实登记）

- **pre-existing（非本计划引入，均注记在测试内）**：/playground hydration 告警；`/ui/a2ui/` 目录 URL 命中 VitePress 专题页（sirv 行为）。
- **测试端遗留**：spa-routes 的 gallery 直连在 4 worker 高负载下偶发 networkidle 超时——已用 `test.slow()`（3× 超时窗）硬化，本轮全量通过；后续若再flake 可考虑 SPA 等待改为 title 断言驱动。
- **e2e 未覆盖**：真实后端（3030）联调的 /api/run 真实响应（mock 只证网站反馈；示例真实性由仓内金样另证，符合计划 §6）；4187 端口在部分时段被系统 Hyper-V 段干扰的观察（与 plan707 记录一致）——本计划全程用 4186 无碰撞。
- **代码遗留**：`HomeDemo` 未抽 DemoFrame（HomeDemo 自包含，无第二使用方，抽象无收益——如实登记为简化）；apps.md 目录页保持共享 landing.css 骨架而非 AppLandingLayout（目录页≠单应用骨架，基线报告已注记）。
