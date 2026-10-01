# PLAN-718 r1 — 第一阶段合入复核

用户 2026-10-01 要求“最近关于 website 的修改都先提交到主分支”，授权本次已提交实现的阶段落地。原 `lang-718` 工作区仍有 T-05 开发内容；为避免移动、暂存或裹挟该会话的 WIP，本次使用独立合入快照 `D:/autostack/.wt/lang-718-land/auto-lang` / `codex/website-718-land`。它是临时合入工作区，原计划工作区和分支继续保留，PLAN-718 不归档。

## 基线与范围

- 原四个交付提交：a316ec21d / 76cd8d2a8 / c86630f64 / 013b61208（T-01..T-04 的已提交实现）。
- 初次 rebase 到主分支 417aff474 后对应 abc9dae09 / 8a76aeab2 / e3e6699a6 / ece321dd5；`git range-diff` 四项全部 `=`。
- 本次修正的实现基线：`39d8be1035303a8261e52d1abdeb72a6247213e9`。复核基于实际 diff、独立构建和行为检查；原功能实现来自其他执行上下文。本会话修正了两个小问题并自复核，因此这两项不声称外部独立审查。
- book 只读依赖：`d7a71a7fb1fa42ddd26d6cec859715ed98161f7a`。VitePress 安装版 1.6.4；两处 npm ci 在本工作区安装，没有链接其他工作区依赖。
- 主分支既有 canonical Spec 输入：project SHA256 `5a1a542d5eba5c598ce32b9f7b0ceaeadc61e6d88f034a79863d10c0ea5f8d7b`，ui-presentation SHA256 `137265b853eded5b2cfb552a39c9685a49826959790d413712221e70477e08c9`。
- 主仓 13 个 untracked website PNG 一并入库，PNG 签名与非零尺寸检查通过。这些是保留资产；没有将它们换成网站主图或改变原有图片来源说明。

## 阶段验收

| 阶段标准 | 对应计划验收 | 证据与结论 |
|---|---|---|
| PC-01 四个学习入口与八书真实出口 | AC-01/02 的入口部分 | EN/ZH hub 行为用例逐个请求学习卡片链接，HTTP 200 且非 VPNotFound；八书均有入口。pass |
| PC-02 生成所有权与幂等 | AC-03 | 重复 prepare 比较 977 个生成文件（含作者映射/侧栏），零差异；notes --check: vm=471 / aavm=158 / demo=28（4 project）/ book=634 / parity=52。pass |
| PC-03 阅读导航与已测布局 | AC-04/05/06 的本阶段部分 | 五宽度无页面横向溢出；滚动态原生目录不盖顶栏；书页 pager 限于本书，有目录返回与真实 Gitee 源链接；hub 无虚假 editLink/默认 next。pass |
| PC-04 双语操作、源码复制与键盘开合 | AC-07/08 的既有运行器部分 | 复制内容保持源码（Windows CRLF 与渲染 LF 按行尾等价比较）；双语 copy/run/collapse，唯一 aria-controls 和键盘开合；源码与工具栏不叠放。pass |
| PC-05 不引入网站回归 | AC-11/12 的阶段部分 | 64 个既有 e2e 在最终生产构建通过，6 个新增阶段用例随后全绿；32 张阅读矩阵图 landing fails=0 / problem rows=0。pass |

全计划 AC-01..12 **没有据此全部勾选**。首末章节全量审计、全部触控目标/锚点、128 张最终图、按需加载、失败重试等仍由 T-05..T-08 与最终 review 完成。阶段 pass 只绑定上表 PC-01..05，不冒充整计划 reviewed。

## 运行与问题修正

- 最终 `npm run build`：exit 0，154.19s；仅原有 auto 高亮回退与大 chunk 告警，未扩大忽略名单。Category A，不运行 Cargo 或 docs_gen。
- 全集：端口 4207，CI=1，4 workers；其中原有 64 例通过。新增用例初轮误把 edit-link 容器当链接、header 匹配到隐藏原生导航，修正定位器后又发现 Windows clipboard CRLF 与 innerText LF 差异；修正测试的行尾比较，没有改变复制实现或降低源码保真要求。端口 4209，CI=1，1 worker 重跑新增 6 例：6 passed（53.6s）。这两次结果共同覆盖最终全部 70 例；不声称最初单次全集零失败。
- 截图：端口 4208，`p718-shot.cjs`，32 图。人工查看中文移动 docs、英文桌面深色 books，以及双语章节工具栏；未见遮挡或错位。manifest：`docs/reports/p718-website-ui/phase-landing/final-report.json`。
- P718-P1（closed）：顶栏内部 56px 加 1px border，原目录滚动态 top=56 对 navbarBottom=57 重叠 1px；CSS 增加边框偏移，最终 32 行 problem=0。
- P718-P2（closed）：作者源收集误收 `test-results/**/error-context.md`，测试后生成不稳定；排除 test-results/playwright-report，重复生成零差异。
- 未发现删除验收项、隐藏错误、全站 ClientOnly 或新增范围外 workaround。两个已关闭问题不作为债务；T-05..T-08 原有任务保留，不转移至其他计划。

## 本阶段规范增量（冻结）

只落实现有行为，未来能力不写成现状：

- SD-01/02：新增 `docs/specs/website/design/learning-reading.md`，SHA256 `404c4b25775284ad86faacd5c0a437950035ba1c6f2af682412b35ce9866f87b`。
- SD-03/05 阶段部分：`docs/specs/website/design/ui-presentation.md`，SHA256 `b5bbbeee215fb26b856f33ed12fdf2479cd7bd215f3aa3d740fe2bd79580ae8d`。
- SD-04 仅 UI/导航现状：`docs/specs/website/project.md`，SHA256 `ae221cbbadcaac30daef10976c002fcd08b169fbe9717a5bb33cfe6a4064b8cf`；同步 runner 导入仍在，本阶段不宣称静态浏览已轻量化。

supersedes_spec_components=[]；new_spec_components=[docs/specs/website/design/learning-reading.md]；touched_goals=[]（没有真实对应运行时 goal）。后续完整 review 必须以已落地 Spec 为 before 状态核对剩余 delta。

## 合入边界与交接

PLAN-713/715 的网站修改及 PLAN-719 r2 四篇文章草稿已在 master；719 尚未实施。未修改运行中的 718 WIP、其他计划或主检出脏 ledger。

ledger refresh 暂缺可用 store writer（没有 write_spec/update_spec 工具，localhost:8080 不可达）；禁止手写 `.autoos/specs.json`。保持 PLAN-718 active，后续 merge 收口时补此阶段派生索引。原工作区下阶段继续前须吸收 master 中两个修正及阶段测试，核对 rebase 的等价提交，不能将旧父分支再次普通 merge。生产站点未部署；当前 4197 预览仍是前次成功构建。本次成功构建在临时合入工作区，清理后需从主分支重新构建才能更新预览。
