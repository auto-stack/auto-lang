---
plan_id: PLAN-758
status: reviewed
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
  [✅ 9490d7e67；余名审计 docs/reports/p758-reference-audit.json（worktree 未提交残留→已随 r2 证据链入库核验）]
- [x] T-02：核验原图、接入主图与功能画廊、指南/规范与来源清单（AC-03）。
  [✅ 9490d7e67 + ee65db613 证据冻结；六图 hash 对照在 p758-autojade-verification.md]
- [x] T-03：v0.5 历程前置、构建、网站与 Rust 注释触面验证、余名审计（AC-01..05）。
  [✅ 9490d7e67/24ddb99e1/3685a86b4；构建 151.40s+原生版 144.04s；63 网站场景通过记录；
  cargo check 过、t 5212/5231（19 红全基线对照）、tu 858 过+1 既有金样红；
  九文件非行注释行与 master 一致；余名审计 p758-reference-audit.json]
- [x] T-04：提交实现与验证材料，准备复审/线性合入所需收据、store-mediated 知识投影及阻塞描述（AC-04）。正式复审、线性合入、投影、归档/清理是后续生命周期操作，逐项记入 §9，不以本勾选声称已完成。
  [✅ 证据全提交（ee65db613/9024f1f95）；Review r2 needs_fix（P758-F4）已修复
  （5188b7822 恢复原生链接）并独立复验（206acb729：四定位场景+section-nav 5/5，
  9.8s）；store 投影阻塞 P758-D1 在案，pending 清单 ledger-projection-pending.json]

## 9. 复审记录
stage: new | revision: 1 | outcome: pass | next: work。既有明确改名/素材授权覆盖本合同；无另需确认的执行项。

stage: new | revision: 2 | outcome: pass | next: work。2026-10-11 用户明确要求“五个月，三大步”放在开头，以版本成果为先；增加 SD-03 / AC-05，不取消原改名与截图任务。T-03 重开页面验证。

### Work r2（2026-10-11）

stage: work | revision: 2 | outcome: ready_for_review。T-01..04 的实现/材料准备完成。T-04 将 work 准备与未来 review/merge 生命周期分开表述，原线性合入及 store-mediated 收尾义务不减少。未声称投影、归档、清理或全部原始字符串清零。证据：docs/reports/p758-autojade-verification.md；八份规范输入/输出与 patch 冻结在 docs/reports/p758-autojade/spec-delta.json。

门禁：最终网站构建 151.40s 成功；63 个网站场景有通过记录，手机浮层最后 4/4 复验成功。cargo check 成功；t 5212/5231 通过，19 红均有基线/隔离对照；tu 858 通过、1 既有金样红、2 ignored。九个源/夹具文件每个非行注释行与 master 一致。未运行 tf 或单独 docs_gen。依赖 auto-down 895f8d0f9355c9f5ec3ce8fca268bdb768395846。

### Review r2 — needs_fix（2026-10-11）

stage: review | revision: 2 | outcome: needs_fix | reviewed_commit: a0a7553dd85ec435a6fcf8c46c2a8351d7d95030 | finding: P758-F4。新增 @click.prevent 的手动 hash 处理会拦截 Ctrl/Meta 修饰键的原生链接行为。浮层已解决高度问题，处理器不必新增；回 work 恢复原生链接处理，重验四个定位场景。原交付范围、AC 与规范增量不减少。

### Work r2 修复回执（2026-10-11，needs_fix → ready for re-review）

stage: work | revision: 2 | outcome: pass | code plan-758-dev @ 7f6861efd（5188b7822 修复 + 206acb729 复验绑定 + 7f6861efd concurrent-baseline-check.json 保全）
| tasks: P758-F4 修复（SectionNav @click.prevent→@click，删除手动 nextTick/hash 写入逻辑，仅保留 current 高亮+mobileOpen 收起；ui-presentation.md 规范行同步）
| evidence: 前会话四场景 4/4（7.4s）+ 原生版构建 144.04s；续跑会话独立复验 5/5（9.8s，四定位场景+section-nav hash 直达/点击/无遮挡，dist 10:59 晚于修复提交 10:57 实证产物新鲜）；`git diff master -- SectionNav.vue` 终态仅 CSS 差异（浮层）；并发基线核对 concurrent-baseline-check.json 入库（754 组件字节一致）
| blockers: P758-D1 store 不可写（投影 pending 清单在案）；无其他
| next: review

### Review r2 聚焦重审 — pass（2026-10-11）

stage: review | revision: 2 | outcome: pass | reviewed_commit: 7f6861efd（plan-758-dev，worktree D:/autostack/.wt/lang-758/auto-lang，除 playwright 输出目录外 clean）| base_commit: 3e6727d75（分支已由前会话 rebase 至 754 落地后基面；a0a7553dd 为 rebase 前旧基面复审锚，等价性归 merge 档 range-diff 证明）| dependency_revisions: auto-down 895f8d0f9355c9f5ec3ce8fca268bdb768395846
spec_inputs: 冻结 reviewed-spec-delta.patch.gz + spec-delta.json 在分支内；F4 修复对 ui-presentation.md 的单行修订（原生链接行为）与实现一致，纳入本重审确认。
acceptance_results: AC-01..05 维持 r2 复审结论（唯一 finding=P758-F4）；F4 修复验证——①代码对勘：SectionNav.vue 终态 vs master 仅 CSS 差异（浮层样式），模板/脚本回原生；②独立复验：四定位场景（390/1440×EN/ZH，含原生点击→URL hash→标题可见断言）+ section-nav hash 直达/点击/无遮挡 5/5（9.8s）；③修复面仅 SectionNav.vue+ui-presentation.md+docs，不触碰 63 场景/构建/cargo 门禁/图片 hash 等冻结证据的有效性；④Rust 面 9 文件（5 .rs+4 .at）过滤注释行后 diff 全空=零可执行改动（重审实测，work r2 的"非行注释行一致"主张独立复核成立）。
findings: 无新增。P758-D1（store 阻塞）与 P758-OBS1（深色图无工具链 manifest）维持登记。独立性限制：修复与重审同会话；按工件重建结论（diff 过滤实测、e2e 独立复跑、dist 新鲜度时间戳实证）。
next: merge

### Review r2 — 最终同步基面与证据重绑定（2026-10-11）

stage: review | plan_id: PLAN-758 | plan_revision: 2 | outcome: pass
reviewed_commit: 6e1d53da58a89c1067d560c9bbbe7ec5bc2a1f1b
base_commit: 53f81a09dcc26716b2a67b2c3f9922c70008315b
dependency_revisions: auto-down 895f8d0f9355c9f5ec3ce8fca268bdb768395846；AutoJade 来源仓只读核验身份 28e67b21414a26aac2414d7e3b457f3121ed418d（不冒充深色图构建身份）。
spec_inputs: docs/reports/p758-autojade/spec-delta.json 八个输入/输出 SHA 全部对当前 canonical 基面/工作树复算一致；reviewed-spec-delta.patch.gz 解压 SHA 042ea5aeb15caea6a8092e4d02f85f185eff16f52ca5fa21796adc35153f2b28，原样等于当前八目标 diff。new_spec_components=[]，touched_goals=[]，功能目标状态未变。
acceptance_results: AC-01 pass（全跟踪 UTF-8 余名 116 项逐项分类，unclassified=[]，可维护产品引用统一；九源文件非行注释内容对当前 master 全等）；AC-02 pass（新专题及同语言旧路由 query/hash replace、首页/发布/总览实际检查）；AC-03 pass（六原图逐字节再对照；P4 排除、SHOT-09 留空符合合同，原图/画廊/键盘/断点主题检查）；AC-04 pass（最终提交 clean，八规范冻结和投影草案有效，既有门禁红实证分类，明确保留 store 阻塞不假归档）；AC-05 pass（简介→journey→亮点→桌面/应用→理念规模，EN/ZH 四顺序/定位场景复跑通过，实际浏览器标题可见）。T-01..04 材料完成，不代表未来 ledger/归档/清理完成。
evidence: 完整构建 124.30s；最终当前基面 63 网站场景中 61 pass，搜索隔离 pass，/playground 既有静态无后端探测的 console 检查隔离 fail（两个宿主组件与 master 全等；未放宽豁免）。check 成功且 406 条预存 lib 警告；t 5213/5231 pass、18 fail、1539 skipped、87.844s（16 基线已复现，state_file/plan502 隔离 2/2 pass）；tu 857 pass、2 fail、2 ignored（既有桌面金样 + env 路径用例隔离 1/1 pass）。全部日志/hash、来源/审计/失败基面检查在 docs/reports/p758-autojade/ 及 p758-autojade-verification.md。未运行 tf 或另外 docs_gen。
rebase_evidence: 3e6727d75..bbe1e3556 → 53f81a09d..6e1d53da5 九提交 range-diff 全等；网站 tree 与实际构建/测试的 5188b7822:website 完全相同，仅后续证据/规范基面冻结更新。更早 754 同步出现的 diff context/debt append 冲突由最终完整重审覆盖，未冒充为全等。
findings: P758-F1..F4 已按证据处理，F4 脚本与模板回原生（对 master 完全相同），最终导航仅浮层 CSS。P758-D1（store writer 不可达）、OBS1/2/3 如实登记，无交付内容遗漏、未批准延期或 workaround。独立性限制：当前实现会话复审，以真实 diff、原始日志、目标 hash 和运行页面重建结论，不声称另派独立 agent。
next: merge；publication-only store 阻塞按 merge 技能保留 reviewed 活跃计划/工作树。


### Merge r2 — canonical 已落地，知识投影阻塞（2026-10-11）

stage: merge | plan_id: PLAN-758 | plan_revision: 2 | outcome: blocked | checkpoint_key: PLAN-758:r2
prepared: reviewed 6e1d53da58a89c1067d560c9bbbe7ec5bc2a1f1b；八 canonical Spec 差异、输入/输出、投影草案均精确复算；所有实现/证据已提交，工作树 clean。Git for Windows wt-guard 当前输出 clean（无 reparse point）。
landed: 4858260642157b866abd5636333835700debddb8 --ff-only→ cf33ace7900be5fa1a4cabb06af9fc9e325ee374，主检出 HEAD 在合入后等于 delivery；逐项复查八规范 SHA、六 PNG 原始字节及 website tree 与已构建/测试 tree 同值。当前页面预览 http://127.0.0.1:4218/zh/v05/#journey 使用该精确 tree 的 124.30s 构建，已重启读取新产物，真实浏览器标题可见；不是外网部署。
rebase_mapping: 53f81a09d..6e1d53da5 → 485826064..cf33ace79 九提交 range-diff 全等：0f03afc51→5176c0733、ba1d33e00→1b31a5ff4、258e0e82e→4023a1e8c、3c8eba7d9→468258e8a、c0dd9978e→760249c14、2deb00e8b→7765440fe、5074e29de→140b6e45b、0dba73187→263aa274e、6e1d53da5→cf33ace79。仅主检出计划簿记增量，代码/依赖/规范合同不变。
ledger_refreshed: blocked，workspace D:/autostack/auto-lang/.autoos/specs.json；受影响旧项 P732-1/P722-1 与八规范目标在 ledger-projection-pending.json。没有可达注册 spec writer；此前临时 AutoMusk 服务启动/只读探测被自动审批拒绝（blocked by policy，无进一步原因）。未直接写 ledger；当前 hash 与 prepared 快照一致：a133434a4fe8056609861c7d514772c8a646dea3cc3e8a5cda09321e4fe0c6cd。
archived: pending，不移动计划；status 保持 reviewed，当前 active 路径 docs/plans/758-autojade-rename-and-promo.md。
cleaned: pending，保留 clean 工作树 D:/autostack/.wt/lang-758/auto-lang / plan-758-dev 及组内依赖；不得跳过 projection 先归档/清理。既有其他会话的 PLAN-759 编辑及临时文件未纳入本提交。
next: writer 可用后只续做 store-mediated 投影、读回校验、归档及 guard 后清理。当前请求的页面顺序、产品引用与已验收素材已落地；P758-D1 为 publication-only 收尾阻塞。批量回归按 merge 技能在 cleaned 后判定，不在此工作树运行；同日现存批量收据覆盖至 755（new_reds=[]）。

## 10. 待澄清事项
无必需决策；深色 PNG 没有自己的完整工具链 manifest，只标用户提供准备态，不伪造对应 promo 旧批身份。store 可用性按先前 P753-D1 记录核查，若仍失败停止 ledger 写入，继续可落地内容。
