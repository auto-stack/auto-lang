# PLAN-719 r2 · OS / AI 介绍交付与复核

实施日期：2026-10-01。用户明确要求实施 PLAN-719；沿用 r2 的 8 个任务、12 个验收与 4 个 Spec delta。VitePress + Vue 3 保持不变，Category A，零 Cargo/docs_gen、零跨仓实现修改、零公开部署。

## 基线与取材

- 实现树：`D:/autostack/.wt/lang-719/auto-lang`，分支 `plan-719-dev`。从主线 `a19be0e845c767c417a69785ee0fa2c5c4b3b323` ff 同步；继承已完成的 718 阅读/按需加载与 720 更正后的六图。
- 初始实现提交：`cce4df6657a293df537b6cbd5a10bf172b22d875`；最终代码/测试提交：`3c6ea4c9ba753b7bd6ebf0b43dab3bc28174df20`（包括图库标题/锚点恢复及首页测试定位修正）。
- auto-os：`73d02b50f535543bd635c6f6f49dad6835efff45`；auto-ai：`5a50a55844d7aa3523b593f21ba0fb03d18eac48`。
- auto-musk：实施开始 `653b0879fc29b9c13009a508f368f51d831653dd`，复核时 `6e6b6cc847639805c796e0e5a7cc6091b96f4319`；两者的 `docs/specs/modules/plan-flow.md` diff 为空，后续无相关取材变更。
- book（只读构建依赖）：`d7a71a7fb1fa42ddd26d6cec859715ed98161f7a`。
- Spec 输入 SHA256：website/project `994113f25acd22617fb92862204fe14a737077d6caa0e66d35c53c4a55a8ebdc`；website/ui-presentation `07a644d95f92d73ef244fcebabf4a1cd75c4b6ce32cde8825ac25323ec7734c0`；AutoUI overview `f968bb10bcde669cdc4bc4c8059eee964e5301cded7e73e64ae895cfa2b71f4e`（主线后续 UI 补充需在落地时另核对）。

## 主张、来源与状态

| 主张 | 依据 | 落地口径 |
|---|---|---|
| LaOS → OS over OS → AI + Lang + OS 连续 | 用户概述；raw/os 历史设计；virtual-desktop | 实现理念、当前产品视角、长期人本方向，分别说明 |
| 跨生态接口一致 | 用户概述；AutoUI overview 与虚拟桌面正式设计 | 命令/UI/服务/通知/启动/通信是逐项验证目标；非所有平台已完成 |
| 宿主与独立系统 | virtual-desktop；509/auto-cosmic；Harmony 战略设计 | 当前复用宿主；Linux 合成早期；完整发行版/OpenHarmony/自有内核未来 |
| 语言与产品两轴 | auto-os Design 01、apps.manifest、desktop-app-launch Spec、shell 和 desktop.ps1 | auto-lang 框架与 auto-os 产品区分；当前清单主要 VM，不把编译壳当全部应用原生化 |
| Auto/Rust 基础 | 各仓当前实现与历史 Auto 化记录 | Auto 源码、生成代码与 Rust 基础设施组合；native 自实现未来 |
| AI 分层 | auto-ai server/pool/配置及 CLI Specs；auto-musk plan-flow | 应用/Agent 工具、client、aaid/provider 分层，工具执行与模型通信分开 |
| AI 当前与未来 | 当前 Role/模型绑定、命令规则、Plan 流；用户方向 | 共享接入/开发工具已有；个人长期知识和跨应用协作尚需建设 |
| 历史日期/转折 | proposals/719-history-sources.md 的提交/设计/归档计划核验 | 设计、实现、迁移和归档分别判断；历史 e2e 不当本次运行结果 |

`prepare-content` 生成后仍没有 `/docs/os` 或 `/docs/ai` 正文，改为存在的 OS/AI 概括页、当前产品、设计资料与长文入口。修改的八个介绍页不再引用这两个不存在的根页。

## 实现取舍与稿件对账

采用计划允许的等价方案：可索引 Markdown 正文 + 同源双语概要/架构组件。十二页均使用 VitePress 默认阅读布局；没有把长文放进 landing 卡片。概要与页内目录帮助快速浏览，正文默认展开。OS 先呈现真实桌面双主题预览，再在组成章节放架构关系；AI 以应用/Agent/client/aaid/provider 关系示意承接介绍。

| 作者稿/旧页面 | 最终位置与变化 |
|---|---|
| 719-os-ai-copy 的 OS/AI 第一节 | `os-ai-introduction.ts`，IntroductionFrame/OSIntroduction/AIIntroduction；入口明示当前资料与历史文章 |
| OS 后续五节 / AI 后续五节 | 双语 os.md / ai.md 原意完整保留，H2 正文 + 状态表，AI 补 OS 总览互链 |
| AutoOS 首屏与老正文 | 概要与完整六图 → 窗口/工作区 → 应用 → 共享设置 → 框架/产品/宿主；保留旧设置及 Kanban 真实图并与 10/1 新序列区分 |
| desktop“即将推出” | 说明 Vue、VM/iced、代码生成与编译/合成路径，状态按功能/平台判断 |
| 首页两卡 | 从共享双语概要取摘要；其他卡片/布局保全 |
| release-v05 | 只改 LaOS、OS/AI 接缝、应用/设置的缺据断言及相关展望；保全其余发布历史、数量快照、资产及展示结构 |
| 四份文章 proposals | 完整复制为正式作者 Markdown；两处公开引用按下节调整，中英文同步，其余叙事/解释保全 |

取消旧总览的“∞ 模型”“4 核心 crates”、假运行输出、百分比数字墙、固定 Pop!_OS/COSMIC 路线和全平台已支持暗示。不删除全局旧 hero 组件，不扩大到跨仓运行时。

720 六图、desktop-showcase.ts、DesktopShowcase.vue 相对基线均零 diff；原 PNG 与新版快捷方式全部保全。保留原图库的键盘放大、原图、尺寸、捕获日与小组件前提。

## 公开来源核验

`website/scripts/p719-check-sources.mjs` 匿名访问实际 GitHub 页面，检测 HTTP 状态与不存在页面；失败做有限重试，一小时内成功收据可复用，逐源保留检查时间。最终 31 个唯一公开引用全部通过，见 `p719-public-sources.json`。

初查 33 引用中两条 HTTP404，未悄悄删除：

- auto-os 的 `docs/specs/shell/desktop-app-launch.md` 本地有现状但未公开。文章相关句改由已经公开的 `desktop-shell-a2r.md` 支撑解释/编译双路径；仍引用真实 RQHost 接入提交。当前 VM 集成事实在概括/产品页按 10/1 本地 apps.manifest 与 Spec 核验说明。公开 apps.manifest 比本地旧，未把它伪称为最新 launch 字段证明。
- auto-musk 的 `12dff96bd3c2d5f7aad8dfe1528c41d689032141` 本地存在但未公开。两语言改用公开 plan-flow Spec，说明计划路径绑定、执行前 Human 确认与后续验证。公开 Spec 仍有旧的“双缺降级提示”，因此网站不引用它声称已具有后来本地提交的路径存在硬失败修正。具体修正证据继续保留于研究收据。

## 验证结果

最终构建 `npm run build` exit0，142.09s。最后一次网站源码变化是图库调用恢复章节标题；之后只有测试定位修正，运行时源码零 diff。固定 dist 后执行 `$env:AUTO_WEBSITE_TEST_PORT='4226'; $env:CI='1'; npx playwright test --workers=1`，**125 passed，5.8m，零重试**，包含新增 16 项介绍用例、718 阅读族、720 图库、site-ui、home-demo、SPA/playground 等。构建与测试均在 719 工作树；预览 strictPort，不复用他人服务。

`node scripts/p719-shot.cjs` 最终 exit0：十二路由 × 五宽度 × 两主题 **120 检查**，390/1440 × 双主题 **48 份 full-page PNG**；最终 manifest 时间 `2026-10-01T07:40:20.473Z`。全部校验实际 pathname、locale、唯一 h1/非404正文、主题、页面横向溢出≤1px、标题在顶栏下方、介绍入口≥44px；所有可见图片解码后取图。人工逐一通过四组 opening 和四组 whole 联系图查看全部48页面，配合双语正文审读；没有裁切、缺图、乱码、被顶栏遮挡的标题或主题白块。旧设置截图在深色页保留原图本身的浅色内容，来源说明不混同网站主题。

先前结果不冒充最终验收：首轮14项通过；一轮整站检查发现 AutoOS 嵌入图库丢失 h2，已恢复标题/定位；新增首页冒烟最初使用不存在的 `.features-section`，改为真实卡片入口，不降低行为断言。该轮与重建 dist 重叠，出现后半段服务空响应（95 pass/30 fail），整轮作废。重建结束后独占固定产物完整重跑125项，所有问题均闭环，包括既有 guided-tour 搜索。初轮图全部覆盖为最终图。

健康检查：`git diff --check` 无错误；十二新/改页面的页面异常及 Vue 警告收集为空；保留既有高亮 `auto→txt` 回退、Browserslist 数据过旧和 bundle 大小提示，未引入新的警告来源或依赖升级。`python scripts/spec-index.py` 成功，26 projects，无语义索引变化。测试生成的旧 P720 报告图已恢复，避免改写历史证据；本次图与来源收据单独入库。

| 验收 | 任务 | 实际证据与结果 |
|---|---|---|
| AC-01 | T-01/02/07 | 双语 OS 默认正文及 AutoOS 长文对照批准稿，完整解释三种理念的连续性、部件与接口目标，pass |
| AC-02 | T-01/02/04/07 | 宿主/现有内核独立系统/自有内核与 Rust/native 状态人工对账；相关页与发布字段扫描，未来无既成断言，pass |
| AC-03 | T-01/02/03/07/08 | 两轴、UI/桌面/命令 Shell/终端、AI 五层及配置职责与源 Spec/代码对账，pass |
| AC-04 | T-02/03/04/07/08 | 两语言概要/架构/状态和文章逐段审读；缺据统计/假输出移除，来源与历史边界保留，pass |
| AC-05 | T-04 | 双语 AutoOS/ui-desktop/home/v05 冒烟 + 全站原图库契约；其他发布内容/PNG 零不相关改动，pass |
| AC-06 | T-01/04 | 十二页 CTA 实际 HTTP/正文/locale e2e；不存在的 docs 根页替换；31公开来源，pass |
| AC-07 | T-02/03/05 | 生产 SSR 有 h2/正文；两语言搜索命中 OS 历史与 AI 概括；默认展开，pass |
| AC-08 | T-05 | 120几何 + 48图人工核对 + 720键盘图库/dialog回焦 + 44px介绍入口，pass |
| AC-09 | T-05/06 | 完整125 pass、稳定构建、diff/警告/范围审计；同会话独立证据复核并注明独立性限制，pass |
| AC-10 | T-01/07/08 | 33原始引用逐源核验，2条改为公开等价内容，最终31全部可访问；历史事件与源/现状分别叙述，pass |
| AC-11 | T-04/07/08 | 四文章真实路由、对应总览返回/进入、两文互链/语言保持主题，pass |
| AC-12 | T-05/07/08 | 四文章默认阅读布局、目录锚点/SSR/search；双语语义对账、全断点主题几何及48图，pass |

遗漏/延后/workaround 扫描：无未完成验收，无新增实现债务。修正图库标题和测试定位没有改变 r2 范围；来源替换按原契约完成，不把未发布修正挂到旧公开 Spec 上。正式 ledger/归档是合入后的服务前提，不降低网站 AC。

## 规范增量冻结

| delta | 当前规则 | SHA256 |
|---|---|---|
| SD-01/04 · 新 os-ai-introduction.md | 理念连续、三形态、职责分层、文章与来源/时点/阅读契约 | `93e021ec16ac2fed2e78876eb5aeccba9d563d2ad6ee37a02a0972256ed46fae` |
| SD-02 · website/project.md | 页面职责、同源概要与双语长文入口 | `15003f95fd1b19c62a38500430fe413c060c9bbb8646fce52982463dfdcf275a` |
| SD-03 · ui-presentation.md | 默认可读/SSR/search、实图/关系图、文字状态与48图校验 | `fe0d8d50cf3167217707c02fd6605d3fec468215f06230a296ac8480e7935fef` |

无规范退役；无新增 goal 身份，沿用网站目标。Spec 更新在实施工作树准备，不在 main 写 WIP，不手写 `.autoos/specs.json`。索引服务 8080 当前连接拒绝且会话没有 write/update Spec tools，正式归档须等待 store writer 可用；代码/规范可先经复审合入。
