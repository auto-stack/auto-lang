# PLAN-757 VM 深色截图验证

日期：2026-10-10。实现提交：`b9a91fad7675c6088d05a3736f7dd707c7afbe6b`；基面：`aea687684ae09c49ed1c32be1040b1290e36ffb1`。最终复审绑定的证据/规范提交见主检出 PLAN-757 §9。

## 交付与证据

- [捕获目录](p757-autoedit-capture-catalog.json)记录 CLI 完整身份、依赖、AutoEdit 源码、VM 官方启动、深色模式、PNG 字节 hash、尺寸与动作。两张合格 PNG 为 2560×1600，未裁剪、缩放、拼接或重绘。
- SHOT-01/02 已在双语首页、v0.5、应用总览/专题按用途接入；SHOT-03 仍是文字空位。[第三张完整实拍](media/p757-autoedit-diff-layout.png)仅供诊断，未进入 CAPTURE_IMAGES。
- [逐图指南](website-v05-capture-guide.md)保留全部 12 个稳定 ID，列出已拍场景与其余场景的准备、操作、入镜要求。
- 真实比较及保存重比由[操作记录](p757-autoedit-save-recompare.txt)证明；保存发生在隔离 Git 版本副本，不编辑定型产品源码。
- 原生查找输入焦点问题没有通过 MCP 绕过而获得功能验收；其限制明确登记于来源目录及 P757-D2。

## 网站检查

均在 `D:/autostack/.wt/lang-757/auto-lang/website` 执行。未修改 Rust 源码，遵守 Category A：不运行 Cargo tests 或 docs_gen。CLI 编译是用户授权的截图准备。

| 检查 | 命令 / 条件 | 结果 | 记录 |
|---|---|---|---|
| 网站构建 | `npm run build`；NODE_OPTIONS 8GB | pass，148.93s | [构建日志](p757-capture/website-build.log.gz) |
| 应用、发布页与站内行为 | `npx playwright test apps-introduction.spec.ts site-ui.spec.ts v05-copy.spec.ts home-demo.spec.ts --workers=2`；CI=1，端口 4197 | 70 pass / 1 fail，4.2m；失败是 AutoShell 首次查询索引未完成 | [综合日志](p757-capture/website-tests.log.gz) |
| 新图原图与键盘操作 | `npx playwright test v05-copy.spec.ts --grep 'AutoEdit captures' --workers=1`；独占端口 | 2 pass，39.6s；EN/ZH 原像素、原图 200、Enter 打开、Esc 关闭、焦点返回 | [放大日志](p757-capture/website-capture-zoom.log.gz) |
| 搜索第一次独立复跑 | `npx playwright test site-ui.spec.ts --grep 'AutoShell query' --workers=1 --reporter=line --output=test-results-p757-search`；端口 4197 | fail；输入已为 AutoShell，索引请求未完成 | [日志](p757-capture/website-search-recheck.log.gz)、[保留 trace](p757-capture/website-search-timeout-trace.zip) |
| 搜索最终独立复核 | 同一测试/原超时与断言；端口 4201；输出目录 test-results-p757-search-final | pass，19.6s，命中 AutoShell 专题；未改网站搜索实现 | [日志](p757-capture/website-search-final.log.gz) |

综合检查覆盖 EN/ZH、360/390/768/1024/1440 宽度、深浅主题、正文与链接、导航、主题保持、原有演示行为；两项新增放大检查单独验证原始 2560×1600 图片。新增放大检查曾与综合检查共用默认输出目录，覆盖了首次失败附件；此执行错误已纠正为串行、独立输出，并保存第二次失败的完整 trace。不能将首轮结果表述为全部通过。

本地搜索诊断：索引模块为 7,415,054 bytes，直接 HTTP 读取约 0.09s；Node 导入与 MiniSearch.loadJSON 约 512ms，AutoShell 搜索约 7.9ms，结果包含 `/apps/autoshell/`。独立 Chromium 会话的索引请求从页面开始后 1,266ms 到 14,847ms 完成，随后显示正确链接。数据证明内容存在，不能证明浏览器加载长尾的精确根因；P757-D4 保留该问题，未给测试加时或移除断言。

构建日志中的既有 Auto 高亮 fallback、Browserslist 数据提示，以及测试 NO_COLOR/FORCE_COLOR 提示保留在日志中；没有本轮 Rust 代码修改或新增编译器诊断处理。

## 验收判定

复核在实施会话内完成，没有使用未经授权的另一任务/agent；以提交、原始图、运行状态、Git 两版本、页面测试重新建立证据，而不是信任任务勾选。

| 验收 | 结果 | 依据 |
|---|---|---|
| AC-01 | pass | 官方 VM 深色入口、干净 AutoEdit 源码与新编译 CLI 身份完整；Rust 试运行没有作为宣传图 |
| AC-02 | pass（图三仅诊断） | 三张均为真实 VM 原图；两张合格，第三张展示实际差异；比较/保存重比动作证据及 Git 原始字节保留 |
| AC-03 | partial | 两张完整 PNG 已接入；第三张因布局不合格留空，未满足 r2 三图宣传替换要求 |
| AC-04 | partial | 指南、规范提案、构建、双语与响应式、放大检查完成，搜索复核通过并保留加载长尾；整体交付尚未通过最终复审/合入 |

结论：`needs_fix`，PLAN-757 保持 `executing`。阻塞项 P757-R1-01 / P757-D1 影响 T-03 与依赖的 T-04：修复正式 VM 比较布局后重拍 SHOT-03，或按明确的范围修订重新立约，不能将诊断图或债务登记当成三图验收完成。原生查找输入、同步搜索和网站索引长尾为另行调查项，不在本轮截图任务中修改产品代码。准备的规范增量如实描述两张合格图与第三槽待拍，不在本次复审发布到主检出或 live ledger。
