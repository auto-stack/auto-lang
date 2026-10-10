---
plan_id: PLAN-753
status: executing
feature_name: ecosystem-readme-refresh
author: [codex]
created_at: 2026-10-10
updated_at: 2026-10-10

plan_revision: 1
approval_status: approved
approved_revision: 1
approved_at: 2026-10-10
supersedes_spec_components: [docs/specs/website/project.md]
new_spec_components: [docs/specs/website/design/ecosystem-readme.md]
touched_goals: [GOAL-014]

affects: [website, auto-lang]
current_step: 0
total_steps: 5
---

# [PLAN-753] 中英文 README：Auto 生态主入口与当前语言介绍

## 0. 变更摘要

重写 README.md / README.cn.md：由约 1360 行的早期语言手册改为 AutoLang、AutoUI、
AutoOS、AutoAI 与应用生态入口。语言内容重新整理为 docs/language/overview.md 与
overview.cn.md，截图直接引用 website/public/ 内已有原图。
具体结构、开头文案和素材见 [内容方案](proposals/753-ecosystem-readme-content.md)。

## 1. 目标

- 读者快速理解四层生态、应用用途、当前状态和源码/学习入口。
- 中英文完整自然、语义一致，保留 README.cn.md 文件名与双向切换。
- 语言介绍下沉并更新，以实际源码、语料与当前 CLI 为依据，不原样搬运旧语法。
- 共享真实截图，注明来源与捕获日期，不把历史图当未冻结发行候选的验收证明。

范围：本仓双语 README、语言概览、必要学习导航与规范沉淀。兄弟仓仅读。
不改 Rust/UI 实现、正式语法规约或应用功能，不部署网站或冻结/发布发行包。

## 2. 架构方案

文档分层：生态首页 → 语言概览 / 产品专题 / 深入学习资料。
根页预期约 150–220 行，语言概览预期约 250–400 行，按信息完整性调整。
使用 GitHub Markdown / 简单图片 HTML，不引入 VitePress 专用组件。
根页相对引用 website/public/...，点击打开同一原图，无图片副本或图片生成。
官网专题优先引用作者源或已确认的实际公开路由；不以 HTTP 200 判定页面正常。

## 3. 技术栈

Markdown、现有 PNG、既有 VitePress 内容准备与构建。
CLI 帮助和现有 .at 语料核对命令/示例；不增加 Rust 功能或运行时依赖。

## 4. 需求分析与背景调查

### 授权与前提

用户于 2026-10-10 要求调研并更新两份 README，介绍生态、更新下沉语言资料并共享截图。
AGENTS.md §1 L1 明确要求「present for confirmation before executing」。用户于 2026-10-10
回复「OK，继续执行」，确认 revision 1 与完整实施、验证、复审、合入收尾。
未获得兄弟仓写入或部署/发布授权，这些动作也不属于本计划。
主检出已有音乐播放器改动、临时核查文件与发布跟踪文档，全部保留且不纳入本计划提交。
无匹配的在途 README 计划。编号由 scripts/new-plan.sh 分配；外层持有 .allocation.lock
独占句柄并核对最大编号，防止脚本仅靠 .next-id 读写时并发撞号。

### 取证来源

主检出基线：58796d33509325beb511e09cd8995fd7759fc0b6。

| 来源 | 本轮采用内容 |
|---|---|
| docs/specs/overview.md | 全局地图；2026-09-04 快照数量不当作今日统计 |
| docs/specs/website/project.md、design/application-introductions.md | 双语网站、素材共享与四主产品规则 |
| docs/specs/auto-lang/{project.md,frontend/overview.md,interpreter/overview.md,trans/overview.md} | 当前 VM、执行与多目标路径 |
| docs/specs/auto-cli/project.md、crates/auto/src/main.rs / Cargo.toml | CLI 命令、当前依赖与前置条件 |
| docs/releases/v0.5.md | 截至 2026-09-30 的已合入历史，14e444f02，非冻结发行包 |
| docs/plans/v05-release-closeout.md | 2026-10-09 用户发布门禁；既有文件仅读 |
| website/.vitepress/theme/data/{applications,os-ai-introduction,desktop-showcase,release-v05}.ts | 生态、产品与截图 |
| website/apps/{autoedit,autoshell,automusk,jadeedit}/index.md 与 zh 镜像 | 当前产品介绍，优先于旧首页宽泛宣传 |
| docs/tour、docs/script-to-ship、test/cookbook、parity | 实际语法、可运行例子与支持范围 |

只读兄弟仓：auto-ai=f1c7794cbd29be95985b142d7eeebf512aba3e82；
auto-os=ab6b2660623703e1f4669a1bcd2674dbc0a05512；
auto-musk=05764bb171420317b039d26084da03bc7e255f99。

SHA256（2026-10-10 调研）：README.md=79c529fa6adbfa95abfd082b1229f30f2eee360dc0ea7ac9edcff8f6955f560f；
README.cn.md=06e4edff3e1ae62119927220f021ad9bf56761fc0a1fd7c3a036609b5e992dbc；
v0.5.md=e68520bbe1b5f4da15fc97fb8286316f4e2f2870ca5cbd2d2eba828e566ec9e0；
applications.ts=a6602d43b093bb85b75281613aedc64b75b21a89605282307473f63750912cd3；
os-ai-introduction.ts=82a30a079252edd0a14a1b77570e5b52e570ac75bea0392913190262f1d1854b。

### 发现与取舍

- 旧 README 仍称 GPUI 为框架基础，提供 evaluator 切换，并把枚举/生成器/异步列为计划。
  当前主路径是 AutoVM、Vue/iced 和 Rust 发布；GPUI feature 已移除。
- 中文首页保留大量英文内容；旧 pac.at 与转译命令须实际核对，不保留未验证示例。
- 最新四产品为 AutoEdit/AutoShell/AutoMusk/JadeEdit；AutoDown 是文档与编辑器基础。
  28 Demo 是介绍集合，不是冻结发布数量、统一成熟度或全端验收结果。
- Musk/Jade 当前网站素材合同为文字介绍，不能用旧发布 Musk 图片替代待准备的样例走查。
- docs/language/specification.md 页头为 v0.2 Draft / 2026-04，不称其为最新 v0.5 概览，
  不通过仅改版本号冒充全量审计。本轮写当前概览，不改正式语法规约。
- 默认 auto CLI 带 python / autodown；Cargo 有兄弟 auto-down 路径依赖。旧「仅需
  Rust/Cargo」不是当前默认构建完整说明，实际依赖与准备步骤须明确。
- 2026-10-10 公网检查：autolang.dev 是 Coming soon 占位页；部署指南的
  http://112.74.45.241/v05/ HTTP 200 实际返回首页 H1「Auto: Language for AI & OS」，
  约 108KB HTML，未含 AutoOS。该路由回落不算专题可用；优先仓内真实入口。

## 5. 详细设计

具体内容合同见 proposal。新语言子页覆盖 .at/.as、AutoVM/a2r、基础/函数/集合、
类型/spec/泛型/模式、错误处理、Actor/async/生成器、模块、Rust/Python/C 互操作、
配置/Node/UI 和工具链。示例选真实语料，各执行/转译后端支持单独表达。
既有 prepare-content.js 把 overview.cn.md 映射到 /zh/docs/language/overview；
需要新学习入口时只改 website/content/learning-navigation.mjs 双语作者数据，不改生成页。

### 规范增量

| delta_id | add/modify/retire | docs/specs/... target | before/after rule | rationale | acceptance IDs |
|---|---|---|---|---|---|
| SD-01 | add | docs/specs/website/design/ecosystem-readme.md | 无 README 分层约定 → 双语生态首页、当前语言子页、事实来源与成熟度规则 | 避免再退回旧语法全集 | AC-01..03 |
| SD-02 | modify | docs/specs/website/project.md | 站点素材/学习路由 → 补 README 共享 public 原图与双语概览路由 | 明确资源与导航单源 | AC-04..05 |

## 6. 测试设计

- Category A：不改 Rust、Schema、docs_gen 或正式语法参考，禁止 cargo t / docs_gen。
- git diff --check；提取新增 Markdown/HTML 本地链接、图片与锚点，检查存在性和大小写。
  外链检查正文和仓库实际地址；记录离线、假 200 与不存在的页面，不能当绿。
- worktree 内 website 的 npm run build；书库通过 AUTO_BOOK_ROOT 指向真实 sibling book，
  无 junction/symlink。检查双语新概览生成路由及中文内容，不接受 EN fallback。
- 用已有 Markdown 渲染工具生成 GFM 预览，检查 EN/ZH × 390px/1280px 图片、表格、
  语言切换和跳转。构建产生无关跟踪变更时仅收本轮必要内容。
- worktree 内用既有 auto 二进制运行选定基础语料，记录二进制/源码身份及 stdout；
  核对 auto trans --help。互操作例子给出实际 parity 语料与依赖，不伪造输出。
- 双语逐节人工审校；图片计算原文件 hash，证实直接引用网站同一文件。

## 7. 验收标准

- **AC-01**：双语根页四层生态清晰、职责/入口齐备，长篇语言语法已下沉；GFM 双语审阅。
- **AC-02**：四产品、AutoDown 基础和系统 Demo 区分；不宣称全 Auto/无人值守/自有内核
  或统一全端成熟度；与最新产品/OS/AI Spec 及 release 逐项核对。
- **AC-03**：新双语语言概览覆盖 §5 各项、命令符合当前 CLI，核心例子有可定位源码与
  实跑结果；旧 evaluator/GPUI/笼统 planned 描述移除；不把历史规范伪称最新版。
- **AC-04**：根页图片全部直接引用 website/public 真实原图，alt/说明本地化、日期明确，
  无重复资产、未准备产品图片或假截图；链接/hash 与窄宽屏实际渲染验证。
- **AC-05**：语言互链、文档/源码/产品路径可达；站点生成概览 EN/ZH 真路由与内容，
  网络不可用不当作通过；构建及链接报告。
- **AC-06**：独立复审绑定 revision/代码；规范、账本、归档和 guard-clean 清理有收据，
  主检出原有 WIP 不被纳入本计划提交。

## 8. 执行步骤

- [ ] **T-01**（确认 revision 1 后；AC-01..05）：重新核对基线，仅提交本计划、proposal、
  .next-id；scripts/new-wt-group.sh lang-753 --branch plan-753-dev 建专属双仓组，
  auto-down 仅读依赖；填写最终事实/命令/图片 hash 表。
- [ ] **T-02**（T-01 后；AC-01/02/04）：worktree 重写 README.md、README.cn.md，
  生态/应用矩阵、共享原图、学习与 quick start 入口一致完整。
- [ ] **T-03**（T-01 后；AC-03/05）：新增 docs/language/overview.md、overview.cn.md，
  必要时更新 learning-navigation.mjs；核对当前语料/CLI并记录旧内容去向。
- [ ] **T-04**（T-02/03 后；AC-01..05）：完成 §6 验证，写
  docs/plans/reports/753-readme-verification.md；证据附真实渲染截图与未通过项处理。
- [ ] **T-05**（T-04 后；AC-06）：显式 /auto-plan:review 独立于实施结论逐项复验，
  扫描遗漏/延后/workaround；通过后 /auto-plan:merge 沉淀 SD-01/02、账本与 spec-index、
  reviewed→archived，按到期规则处理批量门禁；guard-clean 后清理双仓组/分支并留 recovery receipt。

本轮不调用子代理。复审是独立步骤，不能用写作自检或勾选任务替代。

## 9. 复审记录

2026-10-10 /auto-plan:new：stage=new；revision=1；outcome=blocked（待 L1 计划确认）。
合同、事实源、图片候选和验证路径已明确，next=确认后 work。
本轮新增 T-01..05、AC-01..06、SD-01..02；仅完成调研与内容草案，不声称 README 已更新。

## 10. 待澄清事项

- 已解决：用户确认 revision 1，授权继续执行。
- 可选：若已有确定的新官网主域名可替换源码优先入口；当前公网状态不阻碍本计划，
  本轮不自动扩展为网站部署或发布。
