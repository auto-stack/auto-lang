---
plan_id: PLAN-723
status: drafting
feature_name: Website 应用介绍与28个Demo真实截图
author: [Codex]
created_at: 2026-10-01
updated_at: 2026-10-01
plan_revision: 1
current_step: 0
total_steps: 6
supersedes_spec_components: [docs/specs/website/project.md, docs/specs/website/design/application-introductions.md, docs/specs/website/design/application-demos.md]
new_spec_components: [docs/specs/website/design/demo-capture-catalog.md]
touched_goals: []
affects: [website]
---

# [PLAN-723] 应用介绍与28个Demo真实截图

## 0. 变更摘要

承接722双语应用介绍，将约28个候选Demo制作成六类静态目录和逐项介绍，配真实运行图；补AutoEdit/AutoShell已批准主图。Musk和Jade截图继续等待已约定的准备条件。v0.5不新增桌面/app在线体验，不实施UI Playground。

## 1. 目标

G-01 四主应用介绍带已有合格主图，未就绪素材不冒领；G-02 28候选可逐项了解用途、典型操作、实际状态和运行环境；G-03 每候选至少一张真实运行图，附来源/运行形态与版本记录；G-04 页面静态可读，键盘/窄屏/双主题可用。

## 2. 架构方案

从master创建专用723 worktree，在开发分支合入已复审722作为基础，master不落网站WIP。VitePress双语静态介绍+本地分类筛选+EvidenceImage放大。目录数据稳定ID，介绍及配图同源；候选清单冻结后不以重复、无素材或依赖后台为由悄悄缩减。发布素材是用户授权的网站应用图，不将大批验证对拍截图加入产品仓。

## 3. 技术栈

VitePress/Vue3/TypeScript；Playwright捕获Vue实际画面；必要时AutoUI MCP捕获VM。无应用源码修改、无模拟图、无28套mock。捕获运行实例、端口和日志归本计划，绝不停止别的会话进程。

## 4. 需求分析与背景调查

授权：2026-10-01用户明确要求继续介绍页面，包括28 demo介绍+真实截图；允许本轮页面与拍摄，四主应用素材时机沿用既有指示。无需重复实施批准。
依据：docs/specs/overview.md、website/project.md；设计docs/design/documents/website-interactive-experiences.md（master824bfb6bc）；722 reviewed commit bb8bcefd389589f0522fc0b23951843bb975a8ab和其两个准备Spec/28路径报告。当前master tracked clean；保留无关.tmp-vm文件。
素材初查：011/012/013/015/016/017/019/029及OS系统监视器/Launcher/小游戏等有本地验证图；014/018/021/024/026/027/030/031-image/039等需核实并捕获。现有gallery生成Vue dist可只读复用，部分服务型需后台或改用真实VM。本轮不把接口报错/空容器当合格图。
来源、依赖revision、截图hash/尺寸/日期/操作/数据性质逐项记录；公开内容依据源码README和实际画面，不把未验功能写成已交付。

## 5. 详细设计

新 /apps/demos/ 六类目录，桌面3列/平板2列/手机1列；每条真实缩略图、用途、典型操作与详情链接。逐项介绍可采用28 EN/ZH阅读路由或SSR同等详情方式，单H1、目录、互链与locale正确。典型流程说明初始→操作→结果；每项至少主图，复杂项按已取得素材补步骤图，不假装测试过全部功能。
稳定ID/来源沿722候选28项。明确系统集成应用与学习示例、video-app vs player、kanban示例vs独立产品。v0.5静态介绍，无新增app/桌面运行按钮；既有画廊为开发资料且说明服务前置。
AutoEdit使用PixPin_2026-10-01_15-14-27.png；Shell使用ash-01；Musk/Jade保持无图文字布局，不展示空框。Capture-only采样不要求28项双端一致性认证，图注明实际Vue/VM，不虚构另一端。

### 规范增量

| delta_id | add/modify/retire | docs/specs target | before/after rule | rationale | acceptance IDs |
|---|---|---|---|---|---|
| SD-01 | modify | docs/specs/website/design/application-introductions.md | 722本轮留空→723已获素材可展示，Musk/Jade仍等待 | 尊重素材授权 | AC-01 |
| SD-02 | modify | docs/specs/website/design/application-demos.md | 分类首版正文→28逐项真实图静态介绍 | 完成P0 | AC-02..05 |
| SD-03 | add | docs/specs/website/design/demo-capture-catalog.md | 无素材契约→稳定清单、来源与UI捕获质量、发布版本边界 | 真实可追踪 | AC-03..06 |
| SD-04 | modify | docs/specs/website/project.md | 补介绍素材和静态目录现状 | 与交付一致 | AC-01..06 |

## 6. 测试设计

Category A：禁止cargo t/docs_gen；不修改Rust。工作树website npm build；Playwright scoped apps/demos+原网站/OS/展示/路由回归。新SSR条目/唯一ID/图片hash与尺寸/无破图/详情和locale链接/分类/键盘放大/无业务API；360/390/768/1024/1440×双语×深浅，截图人工检查。每图至少确认非空、应用身份、资源已加载、无遮挡报错；记录真实页面/场景与操作，捕获失败有结构化诊断，不伪图。

## 7. 验收标准

- AC-01：AutoEdit/Shell主图真实、共览与详情合理；Musk/Jade未就绪不插假图，旧Shell指南/v05保持。
- AC-02：冻结的28项皆有双语逐项介绍：用途/典型操作/运行条件/来源/状态；分类/详情链接有效，未冒领集成或全部功能。
- AC-03：28项各至少一张合格真实UI截图；hash/尺寸/来源/运行形态/日期/捕获或复用证据入清单，空白/服务错误图不算通过。
- AC-04：六分类目录、稳定ID、可读卡片、放大/返回焦点与双语互链可用。
- AC-05：v0.5静态可读无业务服务请求，不出现本轮新桌面/app在线运行入口；v0.5.1方向标注准确。
- AC-06：build/触面回归/五宽度双主题/图像视觉检查通过，来源和SD冻结，独立复审逐项重建。

## 8. 执行步骤

- [ ] T-01（AC-02,03）：worktree承接722；核定28来源/README/素材，提交目录与捕获清单。
- [ ] T-02（AC-03）：在受控本地运行环境捕获或复核复用28项图；逐图人工审查/来源记录，不改应用实现。
- [ ] T-03（AC-02,04,05）：双语28介绍与六类图文目录、互链、来源/条件；服务版示例明确。
- [ ] T-04（AC-01）：AutoEdit/Shell主图接入总览/详情，Musk/Jade保持约定，旧Shell功能不回退。
- [ ] T-05（AC-01..06）：构建、适配/链接/API/交互测试和页面视觉走查，准备SD与报告。
- [ ] T-06（AC-06）：提交实现，auto-plan-review逐AC复核（同会话明确独立性限制），记录缺项/债务，不默默缩减28。

## 9. 复审记录

stage: new | PLAN-723:r1 | outcome: pass | next: work
用户已授权介绍和截图；直接实施。捕获某项若遇依赖/环境阻塞，记录证据继续独立内容，未经解决不把AC-03或全计划宣布完成。

## 10. 待澄清事项

无当前必需用户决策。未就绪Musk/Jade素材依此前决定等待，不属于本轮28 demo必需素材；技术阻塞将按逐项证据登记。
