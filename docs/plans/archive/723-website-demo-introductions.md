---
plan_id: PLAN-723
status: archived
feature_name: Website 应用介绍与28个Demo真实截图
author: [Codex]
created_at: 2026-10-01
updated_at: 2026-10-01
plan_revision: 2
current_step: 6
total_steps: 6
supersedes_spec_components: [docs/specs/website/project.md, docs/specs/website/design/application-introductions.md, docs/specs/website/design/application-demos.md]
new_spec_components: [docs/specs/website/design/demo-capture-catalog.md]
touched_goals: []
affects: [website]
---

# [PLAN-723] 应用介绍与28个Demo真实截图

## 0. 变更摘要

承接722双语应用介绍，将28个Demo的六类图文及逐项介绍合并进应用概览/apps；补AutoEdit/AutoShell已批准主图。四主应用保留独立详情，Demo不再需要单独阅读页。Musk和Jade截图继续等待已约定的准备条件。v0.5不新增桌面/app在线体验，不实施UI Playground。

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

双语/apps概览按四主应用→六类28 Demo→生态/资料展开。Demo桌面3列/平板2列/手机1列；每条真实缩略图和用途直接可见，三步操作、运行条件、当前范围、截图版本及源码链接收在原生details中，SSR内容完整，键盘可展开。概览仅一个H1；系统应用H2、分类H3、应用H4。每项稳定#demo-<slug>可分享并自动展开，筛选不会阻挡深链。原/apps/demos/及28详情路由只保留兼容跳转与静态兜底链接，目标为当前语言/apps锚点，不再承担独立介绍；四主应用详情继续独立。
稳定ID/来源沿722候选28项。明确系统集成应用与学习示例、video-app vs player、kanban示例vs独立产品。v0.5静态介绍，无新增app/桌面运行按钮；既有画廊为开发资料且说明服务前置。
AutoEdit使用PixPin_2026-10-01_15-14-27.png；Shell使用ash-01；Musk/Jade保持无图文字布局，不展示空框。Capture-only采样不要求28项双端一致性认证，图注明实际Vue/VM，不虚构另一端。

### 规范增量

| delta_id | add/modify/retire | docs/specs target | before/after rule | rationale | acceptance IDs |
|---|---|---|---|---|---|
| SD-01 | modify | docs/specs/website/design/application-introductions.md | 722本轮留空→723已获素材可展示，Musk/Jade仍等待 | 尊重素材授权 | AC-01 |
| SD-02 | modify | docs/specs/website/design/application-demos.md | r1独立目录/详情→/apps内六类28项实图及就地展开，旧路由兼容锚点跳转 | 用户r2页面合并要求 | AC-02..05 |
| SD-03 | add | docs/specs/website/design/demo-capture-catalog.md | 无素材契约→稳定清单、来源与UI捕获质量、发布版本边界；r2更新页面/验证位置 | 真实可追踪 | AC-03..06 |
| SD-04 | modify | docs/specs/website/project.md | 补介绍素材和静态目录现状 | 与交付一致 | AC-01..06 |

## 6. 测试设计

Category A：禁止cargo t/docs_gen；不修改Rust。工作树website npm build；Playwright scoped apps-introduction/demo-catalog/site-ui/spa-routes回归。新SSR条目/唯一ID/图片hash与尺寸/无破图/就地完整介绍和locale链接/分类/键盘放大/无业务API；58旧路由均校验跳转到正确语言/锚点；直接hash和筛选后hash展开；360/390/768/1024/1440×双语×深浅，含详情展开无溢出、截图人工检查。实图来源资格沿r1不变，不重新拍摄。

## 7. 验收标准

- AC-01：AutoEdit/Shell主图真实、共览与详情合理；Musk/Jade未就绪不插假图，旧Shell指南/v05保持。
- AC-02：冻结的28项皆在双语应用概览有逐项完整介绍：用途/典型操作/运行条件/来源/状态；不跳到单独阅读页；四主应用详情独立保留，未冒领集成或全部功能。
- AC-03：28项各至少一张合格真实UI截图；hash/尺寸/来源/运行形态/日期/捕获或复用证据入清单，空白/服务错误图不算通过。
- AC-04：概览内六分类、稳定ID、可读卡片、原生详情展开、放大/返回焦点与双语互链可用；原目录及28旧详情路由兼容跳转当前语言/apps锚点，直接深链自动展开且不被筛选隐藏。
- AC-05：v0.5静态可读无业务服务请求，不出现本轮新桌面/app在线运行入口；v0.5.1方向标注准确。
- AC-06：build/触面回归/五宽度双主题/图像视觉检查通过，来源和SD冻结，独立复审逐项重建。

## 8. 执行步骤

- [x] T-01（AC-02,03）：worktree承接722；核定28来源/README/素材，提交目录与捕获清单。
- [x] T-02（AC-03）：在受控本地运行环境捕获或复核复用28项图；逐图人工审查/来源记录，不改应用实现。
- [x] T-03（AC-02,04,05）：将六类28图文和完整介绍合并/apps，就地details、锚点和58兼容旧路由，四主详情独立；同步设计与Spec候选。
- [x] T-04（AC-01）：AutoEdit/Shell主图接入总览/详情，Musk/Jade保持约定，旧Shell功能不回退。
- [x] T-05（AC-01..06）：构建、适配/链接/API/就地交互/兼容跳转测试和页面视觉走查，刷新SD与报告。
- [x] T-06（AC-06）：提交r2实现，auto-plan-review逐AC复核（同会话明确独立性限制），保留r1复审历史，不默默缩减28。

## 9. 复审记录

stage: new | PLAN-723:r1 | outcome: pass | next: work
work-started: D:/autostack/.wt/lang-723/auto-lang (plan-723-dev); master base42fa2e0c0; development branch merged reviewed722 without landing website on master. Capture inventory underway.
用户已授权介绍和截图；直接实施。捕获某项若遇依赖/环境阻塞，记录证据继续独立内容，未经解决不把AC-03或全计划宣布完成。

stage: review | plan_id: PLAN-723 | plan_revision: 1 | outcome: pass（历史；r2页面结构变更使AC-02/04/06及相关回归失效，须重新验证）

- reviewed_commit: 19beb3046fc3d888000514e651071ad4f2748f8c
- base_commit: ce280b0a3f733a3119d32b57f0e7995e86b1f674
- dependency_revisions: reviewed722=bb8bcefd389589f0522fc0b23951843bb975a8ab；各拍摄源采样HEAD/文件hash见p723-capture-catalog.json，未将其冒充二进制构建commit。
- spec_inputs: base上的三个既有Spec及722候选；四个prepared Spec的SHA-256见p723-spec-delta.json，全部匹配；delta.patch SHA-256=0206ee23a5e4afd5e1041cb438a683852a538f94dcbb82a083a69b2ed9e49845。规范只在worktree准备，master ledger未更新。
- acceptance_results: AC-01..06全部pass；T-01..06对应代码、素材与验证矩阵见docs/reports/p723-review.md。
- findings: P723-D1（medium，非静态介绍阻塞）当前AutoTerm本地VM拍摄无有效会话，失败空图拒收，复用已公开日期/形态的2026-09-22原生图；登记KNOWN-DEBT-AND-RISKS.md。无28项遗漏、隐性缩减、mock替代或未经授权的应用源码修补。
- evidence: docs/reports/p723-review.md、p723-verification.md、p723-capture-catalog.json、p723-spec-delta.json、p723-spec-delta.patch（均在docs/reports）；receipt_commit=a35a908a70ffe38abe0fa051f0465cb2270c9435，仅增验证记录。build155.37s，88旧检查+7新检查=95唯一通过，580布局无横向溢出，58页生成核验，PNG原始字节hash/尺寸，git diff --check与wt-guard clean。一次构建未完成时启动sirv造成/zh/ui 404，构建后独立端口完整重跑ZH28详情与冻结来源断言2 passed；未降低断言。
- independence: 同实施会话内按auto-plan-review从已提交diff、断言、实图和冻结材料重建；无独立第二评审者，不将角色名冒充独立性。
- next: merge（尚未执行）。开发分支保留预览供用户查看；未合入master、未归档或移除worktree。

stage: work | plan_id: PLAN-723 | plan_revision: 2 | outcome: pass | code_commit: 137bc5557855c7757a0f3754ed4466226fa752f3 | task_ids: T-03,T-05,T-06 | evidence: docs/reports/p723-r2-review.md | blockers: none | next: review（已完成，下记收据）

stage: review | plan_id: PLAN-723 | plan_revision: 2 | outcome: pass

- reviewed_commit: 137bc5557855c7757a0f3754ed4466226fa752f3；base_commit: a35a908a70ffe38abe0fa051f0465cb2270c9435；receipt_commit: 62eb4886f9f9b20e930dc681a061db6a4de4bb84（仅记录，无实施变化）。
- dependency_revisions: 722 reviewed bb8bcefd389589f0522fc0b23951843bb975a8ab；沿r1冻结demos.json、28 PNG与p723-capture-catalog.json，素材hash和来源集合再次校验。
- spec_inputs: r2 base中的四准备Spec；p723-r2-spec-delta.json四hash匹配；p723-r2-spec-delta.patch SHA-256=584a552c2c13d79639623c8b354cd2c315af672c8b6bf4fda0f41faedd3e16a5；master canonical/ledger未发布。
- acceptance_results: AC-01..06全部pass；T-01..06代码/行为/材料对应矩阵见docs/reports/p723-r2-review.md。完整正文迁入/apps的SSR及原生details，四主专题独立，58旧路径兼容定位。
- findings: 无新增阻塞；P723-D1素材限制保持明确，28项正文、步骤、条件及状态无遗漏，无mock/伪图/新执行入口；r1结构测试未冒充r2证据。
- evidence: docs/reports/p723-r2-review.md、p723-r2-spec-delta.json、p723-r2-spec-delta.patch；冻结实施最终build177.57s，70 passed(3.2m)；双语就地28项、58旧路由、直接/筛选后hash、图片键盘交互、5宽×双语×主题全部展开无溢出；人工桌面/手机视觉检查；git diff --check与wt-guard clean。
- independence: 在实施会话从已提交代码、断言与实图重新核验，没有第二独立评审者。
- next: merge（未执行）；保留723工作树及4235最新预览供查看。

## 10. 待澄清事项

### PLAN-723:r2 合入检查点（2026-10-01）

stage: merge | plan_id: PLAN-723 | plan_revision: 2 | outcome: blocked（仅账本/归档/清理；网站与canonical已交付）

- prepared: reviewed_commit=137bc5557855c7757a0f3754ed4466226fa752f3，receipt=62eb4886f9f9b20e930dc681a061db6a4de4bb84。最新master142458d21上rebase七项全部range-diff等价；当前实施映射fd2d09c58d763737dbe13090fe8ba8ff84a5c955，review收据映射1cfd65993d3b35107af5a1fec029f1325707211e；规范准备仅增plans/合入记录，未改实施/依赖/测试。完整映射docs/reports/p722-p723-merge.md。
- landed: delivery_commit=02f4b1a90e288ad3dc9d970a435ad29693f9fe43；master先ff-only落722，再ff-only到本交付，实际HEAD相等验证。后继f504b506d62ace4b0f6554934ec7d171c61c89f8仅增加成功收据，网站全等。
- spec-sync: 四r2目标落地且p723-r2-spec-delta.json hash均匹配；website/plans.md补722/723，spec-index.py再生无语义变化；现有介绍目标，不新增Goal。
- integration: master与plan分支website/Spec diff为空，master生成器--check58路由通过；等价重放后的独占4249冒烟3 passed(15.2s)，含冻结28图及中英整页/筛选/就地展开/图片键盘行为；70项最终回归证据代码未变可复用。
- ledger_refreshed: blocked；4248官方musk既有调试程序真实workspace store GET /api/specs=500 failed to load specs；历史账本格式缺project/version，不直接编辑/删除/迁移，不发POST。main与重放树既有.autoos/specs.json未改；临时store已验证PID归属后停止。
- archived/cleaned: pending；保持reviewed和723工作树/分支，4235当前预览保留。账本兼容后仅续做派生投影、读回、归档、guard清理，不重合代码。719之前的相同收尾阻塞保留。
- artifacts: 已验证网站产物在723树，4235可用；main dist/index.html不存在，公开站未部署；未触后台与其他仓产品代码，不重建release。其他会话进程/未跟踪探针不动。
- batch: 722/723非%5、距2026-09-30T16:20Z不足48h；历史720到期已获用户显式跳过(4717753c3)，本轮Category A不跑Rust批量档，last-batch receipt不改。

next: 处理旧账本与store格式兼容，再完成缺失收尾检查点；本会话全部网站修改已合入master。

无当前必需用户决策。未就绪Musk/Jade素材依此前决定等待，不属于本轮28 demo必需素材；技术阻塞将按逐项证据登记。

r2 revision handoff：2026-10-01用户查看后明确要求将28 Demo页面合并进应用概览，四主应用独立详情保留。stage:new | PLAN-723:r2 | outcome:pass | next:work；同723 worktree，r2 base=a35a908a70ffe38abe0fa051f0465cb2270c9435。T-01/02/04素材和主图成果保留，T-03/05/06重开；未复用r1页面结构验收。直接请求已授权本修订，无必需决策；touched_goals仍为空，现有网站介绍范围。

r1执行完成（历史，结构测试不覆盖r2）：28项双语用途/操作/条件/边界与58个阅读/目录页面；28项真图逐图审查，25项新拍，Launcher、AutoTerm、Config三项复用并标来源。AutoTerm当前VM拍摄失败未发布，采用注明版本的2026-09-22原生留存图。真实媒体与文件服务使用公开/专用资源，无mock API/替换UI。AutoEdit/Shell批准主图已接入；实施19beb3046fc3d888000514e651071ad4f2748f8c复审pass，构建与95个唯一检查通过。更详证据与准备Spec保留在计划分支，尚未合入master。

r2执行完成：137bc5557855c7757a0f3754ed4466226fa752f3提交概览合并；28项正文仍由原demos.json提供，没有删除或改写素材。就地原生details与稳定深链、58兼容跳转接入；四主产品独立保留。npm run build首次187.22s；最终冻结实现npx vitepress build 177.57s通过，未在构建过程中继续改实现。r2专项+全站UI/SPA触面回归70项通过，逐AC复审pass；现有4235预览已更新，未合入master。

### PLAN-723:r2 收尾完成（2026-10-09）

（上两段"尚未合入master/未合入master"为 2026-10-01 历史执行记录；实际交付当天已按 §10 合入检查点 ff-only 落 master，delivery 02f4b1a90。）

- stage: merge | outcome: pass（收尾补齐；2026-10-01 合入检查点的 blocked 仅剩账本/归档/清理，本节清偿）
- ledger_refreshed: docs/specs/README.md §5 手工回退——designs 新增 P723-1（demo-capture-catalog.md）；application-introductions/application-demos 现行态以 723:r2 并轨后为准（P722-1/-2 内容已注明）；website/project.md 复用更新 P720-1。tests P723-2、reviews P723-3、reports P723-4。spec-index.py 再生无语义变化；JSON 重载校验通过、新 ID 无冲突。
- archived: git mv → docs/plans/archive/723-website-demo-introductions.md，status → archived。
- cleaned: wt-guard clean → git worktree remove D:/autostack/.wt/lang-723/auto-lang + 组目录 rmdir；plan-723-dev（f504b506d，已全含于 master）branch -d 删除。
- artifacts/batch: 沿 2026-10-01 收据（公开站未部署、main 无 dist、无后台触面）；723 非 %5、批量收据 2026-10-08T11:20Z 未到期，不改 .last-batch-regression.json。

## spec-sync 回写记录（2026-10-09 收尾）

- website/plans.md：723 行状态 → ✅（reviewed→archived），链接指向 archive/。
- .autoos/specs.json：P723-1/-2/-3/-4 upsert；P720-1 复用增补。
- docs/specs/INDEX.md：spec-index.py 再生，无语义变化。
