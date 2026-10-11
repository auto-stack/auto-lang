---
plan_id: PLAN-758
status: execution_done
feature_name: AutoJade 改名与用户提供深色宣传图
author: [Codex]
created_at: 2026-10-10
updated_at: 2026-10-11
plan_revision: 2
current_step: 4
total_steps: 4
supersedes_spec_components: [docs/specs/website/project.md, docs/specs/website/design/application-introductions.md, docs/specs/website/design/ecosystem-readme.md, docs/specs/website/design/ui-presentation.md, docs/specs/auto-lang/ui/overview.md, docs/specs/auto-lang/ui/architecture.md, docs/specs/auto-lang/ui/design/autodown-wikilink.md, docs/specs/auto-lang/ui/plans.md]
new_spec_components: []
touched_goals: []
affects: [website, auto-lang/ui]
---

# [PLAN-758] autojade-rename-and-promo

## 0. 变更摘要
将本仓当前产品引用统一为 AutoJade / auto-jade，并接入用户提供的 AutoJade 深色准备态实拍；按用户 2026-10-11 指示，把 v0.5 的“五个月，三大步”前置，先说明版本进展，再展示产品。

## 1. 目标
更新网站双语首页、v0.5、应用总览/专题、README、当前规范、可维护文档和源码注释中的名称；规范专题 URL 为 /apps/auto-jade/，旧 jadeedit URL 兼容跳转。现有仓库实际地址未改名，不制造不存在的下载/源仓地址。历史冻结证据保留原字节并登记残余理由；derived ledger 只通过应用 store 更新。

## 2. 架构方案
独立 D:/autostack/.wt/lang-758/auto-lang，branch plan-758-dev，主检出仅计划簿记。共享原图元数据及既有 EvidenceImage/ScreenshotGallery，无新运行端点。SHOT-07 工作台共享首页/发布/总览；SHOT-08 用知识关系与快速打开画廊；SHOT-09 恢复仍空位。PLAN-757 的未合入 AutoEdit 两图继续在原工作树保留，不把它的诊断比较图当完成项。

## 3. 技术栈
VitePress/Vue、完整 PNG、Playwright；Rust 只改注释、不改执行语义。

## 4. 需求分析与背景调查
授权：2026-10-10 用户提供 D:/autostack/jade-edit/docs/releases/v0.1.0-rc/promo-dark，并明确要求本仓引用全部改为 auto-jade / AutoJade；对岸产品已改名，仓库名尚未改。延续 website 截图补齐授权，无需再确认这项明确要求。
基面：master 072949d8da69576f899989c5be06a9d2d4c48182；对岸当前 HEAD 28e67b21414a26aac2414d7e3b457f3121ed418d，promo-dark 为用户提供未提交资产。对应 promo/README.md 的旧批来源为 212551ffc0a54ec4353a09e68ed043b16822d466、debug dirty 工具链；不能将其冒充深色文件的精确构建身份。深色图来源单独标注用户提供与未独立核验事项；发布 manifest 是 preparation_only。
权威源：docs/specs/overview.md、website/project.md、website/design/{application-introductions,ecosystem-readme,ui-presentation}.md；对岸 release manifest/trial-guide/promo README。完整检索覆盖 git 跟踪文本、代码注释、旧 URL、证据与账本。主检出已有其他任务 blueprints 删除与临时文件，禁止纳入或重置。
目视结果：P1/P7 主界面与目录，P2 反链，P3 双链，P5 快速打开，P6 建页确认可用；P4 文件名虽为 tags，实际右栏是回收站/查找而非标签面板，不能以文件名冒充标签演示。

## 5. 详细设计
产品显示名 AutoJade，项目标识 auto-jade；website/apps/auto-jade 与 zh 镜像作为规范路径。原路径只用于兼容页、尚未改名的物理仓地址或冻结证据，并生成逐项例外清单。普通历史说明可用新产品名，原始日志、hash 绑定的快照/补丁/manifest 不改写。
从 promo-dark 原样复制六张合格图到 website/public/apps/auto-jade/；第七张保存来源核查状态，不放入标签功能展示。记录源路径、尺寸、完整 SHA-256、用户提供时点、准备态与载体资料的证明边界。不改变图片、不重绘；快开仅说明标题/别名检索，不声称是全文检索。

### 规范增量
| ID | 动作 | 目标 | 前后规则 | 理由 | 验收 |
|---|---|---|---|---|---|
| SD-01 | modify | docs/specs/website/project.md、design/application-introductions.md、design/ecosystem-readme.md、design/ui-presentation.md | JadeEdit 名称/待图 → AutoJade 规范路径、实际准备态图与恢复待拍 | 用户改名与素材供给 | AC-01..03 |
| SD-02 | modify | docs/specs/auto-lang/ui/{overview,architecture}.md、ui/design/autodown-wikilink.md 与 ui/plans.md 索引 | 消费方旧产品名称 → AutoJade；真实仓路径保留 | 全仓引用一致 | AC-01 |
| SD-03 | modify | docs/specs/website/design/ui-presentation.md | 历程在后部理念区 → hero 简介后先讲版本进展，目录同步；实图和规模统计在版本说明之后 | 用户要求开头先讲清本版本做了什么 | AC-05 |

## 6. 测试设计
网站构建与 EN/ZH 五宽度、深浅主题、新旧路径、原图、画廊键盘/放大。全仓余名逐项分类，所有图片字节/hash 对照。Rust 注释触及 ui_gen：按仓库门禁 cargo check -p auto-lang、cargo t、cargo tu；零行为修改另用删除注释后的内容比对，已有失败单列实证，不扩展到产品修复。无 tf 或单独 docs_gen；日常档自带测试族如实记账。

## 7. 验收标准
- AC-01：当前网站、README、规范、可维护文字及源码注释不再以旧名指代产品；剩余旧字符串仅明确分类的真实仓地址、兼容路由、冻结证据或未可写 derived ledger，逐项有理由。名称以 AutoJade / auto-jade 为准，不影响 Jade Garden。
- AC-02：新双语规范路径可达，旧 jadeedit 路径跳转相同语言新页，导航/首页/v0.5/搜索引用一致；现有物理仓链接继续可用。
- AC-03：六张图与用户源文件 hash 一致，按实际内容命名并说明准备态，P4 不冒充标签图，SHOT-09 保留；原图/键盘画廊及移动深浅主题可用。
- AC-04：构建/触面检查与复审记录完成，改动提交且无他人 WIP；知识增量准确，store 不可写时清楚登记而非手改账本或假归档。
- AC-05：双语 v0.5 页面开头为简介、版本进展、三项亮点，再展开桌面/应用；版本进展先于大图与规模统计，导航顺序与章节一致，原理念详情及数据来源保留。

## 8. 执行步骤
- [x] T-01：独立工作树，全仓引用/路径/文案改名，分类保留例外（AC-01/02）。
- [x] T-02：核验原图、接入主图与功能画廊、指南/规范与来源清单（AC-03）。
- [x] T-03：v0.5 历程前置、构建、网站与 Rust 注释触面验证、余名审计（AC-01..05）。
- [x] T-04：提交实现与验证材料，准备复审/线性合入所需收据、store-mediated 知识投影及阻塞描述（AC-04）。正式复审、线性合入、投影、归档/清理是后续生命周期操作，逐项记入 §9，不以本勾选声称已完成。

## 9. 复审记录
stage: new | revision: 1 | outcome: pass | next: work。既有明确改名/素材授权覆盖本合同；无另需确认的执行项。

stage: new | revision: 2 | outcome: pass | next: work。2026-10-11 用户明确要求“五个月，三大步”放在开头，以版本成果为先；增加 SD-03 / AC-05，不取消原改名与截图任务。T-03 重开页面验证。

## 10. 待澄清事项
无必需决策；深色 PNG 没有自己的完整工具链 manifest，只标用户提供准备态，不伪造对应 promo 旧批身份。store 可用性按先前 P753-D1 记录核查，若仍失败停止 ledger 写入，继续可落地内容。

### Work r2（2026-10-11）

stage: work | revision: 2 | outcome: ready_for_review。T-01..04 的实现/材料准备完成。T-04 将 work 准备与未来 review/merge 生命周期分开表述，原线性合入及 store-mediated 收尾义务不减少。未声称投影、归档、清理或全部原始字符串清零。证据：docs/reports/p758-autojade-verification.md；八份规范输入/输出与 patch 冻结在 docs/reports/p758-autojade/spec-delta.json。

门禁：最终网站构建 151.40s 成功；63 个网站场景有通过记录，手机浮层最后 4/4 复验成功。cargo check 成功；t 5212/5231 通过，19 红均有基线/隔离对照；tu 858 通过、1 既有金样红、2 ignored。九个源/夹具文件每个非行注释行与 master 一致。未运行 tf 或单独 docs_gen。依赖 auto-down 895f8d0f9355c9f5ec3ce8fca268bdb768395846。
