---
plan_id: PLAN-757
status: archived
feature_name: AutoEdit 自仓工作区实拍
author: [Codex]
created_at: 2026-10-10
updated_at: 2026-10-11

# /auto-plan:review 结束时填写：
supersedes_spec_components: [docs/specs/website/design/application-introductions.md, docs/specs/website/design/ui-presentation.md]
new_spec_components: []
touched_goals: []             # 引用 docs/specs/goals.md 的 GOAL-NNN

affects: [website]
plan_revision: 3
current_step: 4
total_steps: 4
---

# [PLAN-757] autoedit-website-real-captures

## 0. 变更摘要
从定型 AutoEdit 源码启动 VM/Iced 深色程序，以其自身仓库为 workspace，拍摄工作台、跨文件搜索、双文件比较，替换 PLAN-756 的 SHOT-01..03。

**r3（2026-10-11 用户裁定）**：第三张（双文件比较）因 VM 比较视图布局缺陷（P757-D1：空白编辑区挤压差异正文）**留空合入**——SHOT-03 维持 PLAN-756 待拍槽并在拍摄合同中记录缺陷与重拍条件；比较布局缺陷按用户指示登记为待修产品债务。两张合格实拍（工作台/跨文件搜索）正常交付。AC-02/AC-03 相应收窄，其余范围、SD-01/02 与授权不变。

## 1. 目标
交付三张真实原图、构建/动作记录、网站双语引用和更新后的逐图拍摄指南。其他九个空位保留。不修改 AutoEdit 产品代码、不部署网站、不承诺整套 v0.5 候选验收。

## 2. 架构方案
独立组 D:/autostack/.wt/lang-757/{auto-lang,auto-down,auto-edit}；lang 分支 plan-757-dev，其余依赖 detached。使用已从干净源码重新编译的 Auto CLI，启动 auto run -r vm --theme dark；不使用 Rust/a2r 画面。工作区优先指向 D:/autostack/auto-edit，若启动器强制注入工程根，采用受支持的等价自仓工作区入口并记录。会话及比较版本隔离。

## 3. 技术栈
Auto CLI → VM/Iced 深色、Windows、真实窗口操作/AutoUI MCP/PNG、VitePress EvidenceImage。

## 4. 需求分析与背景调查
授权：2026-10-10 用户明确允许自行编译、打开 AutoEdit 截图，并优先用它自身仓库作为 workspace；继承网站文案/空位任务的实施与合入范围，无另需确认事项。
修订授权：同日用户明确要求截图用 VM 版深色；指出 Rust 版尚未调试且实际样式有问题。r1 Rust 构建/试运行记录保留为历史，不作交付证据；后台 Rust 编译停止，只关闭本任务实例。r2 继承其他目标和授权。
基面：auto-lang 690e23bd4；auto-edit f9ae2a64361f74ad8565fc5df5c4a9aeb5fc10f2。PATH auto 版本含 dirty，需记录构建/二进制身份，不能当作干净工具链来源。
依据：docs/specs/website/project.md、design/application-introductions.md、design/ui-presentation.md、design/demo-capture-catalog.md、docs/reports/website-v05-capture-guide.md；auto-edit/tools/portable/build_portable.py、specs/auto-edit/pac.at、src/back/fsys.at。
主检出 blueprints 删除与 PLAN-755 WIP 非本任务，禁止纳入/重置；独立依赖副本保证构建不消费这些删除。

## 5. 详细设计
发布原图 website/public/v05/candidate/autoedit-{workspace,search,diff}.png；首页、v0.5、应用总览及专题复用工作台图，专题引用搜索/比较图。记录尺寸/hash、源版本、工具链、构建日志、exe hash、OS、后端、会话、动作、日期。比较使用实际源码 Git 两个版本或隔离副本，不编辑主仓制造修改。

### 规范增量
| ID | 动作 | 目标 | 前后规则 | 理由 | 验收 |
|---|---|---|---|---|---|
| SD-01 | modify | docs/specs/website/design/application-introductions.md | AutoEdit 空位→实拍与来源；其余保留 | 定型产品实拍 | AC-01..04 |
| SD-02 | modify | docs/specs/website/design/ui-presentation.md | AutoEdit 主图共享、功能图真实来源 | 完成对应占位 | AC-03..04 |

## 6. 测试设计
Category A 不运行 Cargo tests/docs_gen；编译是用户要求的步骤。检查 PNG/hash、源码对应、网站构建及双语/移动/深浅引用、放大和原图。

## 7. 验收标准
- AC-01：从定型源码通过 VM 官方入口以深色启动，记录 CLI 构建 commit/完整 hash、应用源码版本与命令；Rust 画面不入库。
- AC-02（r3）：工作台与搜索两图来自真实 AutoEdit VM 深色自仓 workspace 与源码（搜索来自实际文件、结果逐一点击核对）；比较图（原第三张）因 P757-D1 布局缺陷留空——功能已核查（真实差异 +16/-1、保存重比 +17/-1 在案），缺陷在 KNOWN-DEBT 登记，修复后按拍摄合同重拍。
- AC-03（r3）：发布两张完整原始 PNG；双语首页/v0.5/应用总览/专题引用与实际后端/日期一致；SHOT-03 维持带缺陷说明的待拍槽；其他九槽存在。
- AC-04：更新指南与规范，网站构建/资源/放大/移动主题通过，复审记录遗漏及债务。

## 8. 执行步骤
- [x] T-01：独立组、已编译 CLI、VM 深色官方启动与来源记录（AC-01）。
- [x] T-02：工作台、搜索、比较原生实拍与动作证据（AC-02）；比较图仅为诊断，合格发布依赖 T-03。
- [x] T-03：双语三槽与 AutoEdit 总览替图、指南/规范更新（AC-03，r3 范围=两图接入+SHOT-03 留空）。
  [✅ 6722669d8..64f820cfd：两图入 public/v05/candidate/ + 双语专题/总览/首页接线 + 指南 SHOT-03 标"待布局修复后重拍"；rebase master（57 commits，与 758 六文件冲突按并集语义解决，重放伪影修复 2 提交）；构建 164.41s；apps-introduction+v05-copy 21/21（总览 62 图/槽 SHOT-01,07×2img 探针实证定谳）；PNG sha256 与捕获目录逐字节一致]
- [x] T-04：验证、提交、独立复审、合入及按门禁归档（AC-04）。
  [✅ 验证/提交完成（64f820cfd clean）；复审/合入/归档/清理收据见 §9 后续记录]

## 9. 复审记录
stage: new | revision: 1 | outcome: pass | next: work。用户已有授权，无新增产品实现。
stage: new | revision: 2 | outcome: pass | next: work。用户纠正后端为 VM 深色；AC-01/02、T-01 更新。r1 preview 仅诊断：默认 1MB 栈溢出，按生成配置扩栈后有样式和插值问题，用户按 Esc 中止；不存在合格实拍或旧复审可复用。

### r2 证据复核（2026-10-10）

- stage: review | plan_id: PLAN-757 | plan_revision: 2 | outcome: needs_fix
- reviewed_commit: `150aaccdea412c71e6b55f9e1053bb8f304e60c1`
- base_commit: `aea687684ae09c49ed1c32be1040b1290e36ffb1`；工作树：`D:/autostack/.wt/lang-757/auto-lang`，branch `plan-757-dev`，已核对 worktree inventory 与 clean 状态。
- dependency_revisions: auto-edit `f9ae2a64361f74ad8565fc5df5c4a9aeb5fc10f2`；auto-down `895f8d0f9355c9f5ec3ce8fca268bdb768395846`。CLI 从上述 lang 基面重新编译，exe SHA-256 `c1281ad05f03407995308af0c808562dc4012a52b415ed9e3ffb37aa6f806839`；版本、命令、日志及九个应用源文件 hash 在捕获目录中。
- spec_inputs: 两个 supersedes 路径的基面/准备后 SHA-256、AutoEdit M4.4 权威源 hash 冻结在 `docs/reports/p757-spec-delta.json`，该 manifest SHA-256=`4f2bf2a564bda799e3d726a67fa210032b7a1850b3e5817283fb474e11218467`。已按 Git blob 核对提交内证据 hash 与规范 hash。SD-01/02 提案准确反映两张合格实拍和第三槽待拍，未发布到主检出或 live ledger。touched_goals=[]：本轮只补网站真实展示素材，不改变全局 Goals 的功能状态。
- acceptance_results: AC-01 pass → T-01/捕获目录；AC-02 pass（第三图仅诊断）→ T-02/三张原始 2560×1600 PNG、真实 Git 两版本及保存重比记录；AC-03 partial → T-03，仅 SHOT-01/02 发布；AC-04 pass → 指南、规范提案、构建 148.93s、双语响应式与主题、两项原图/键盘放大检查。T-04 的合入归档仍依赖 T-03，不能勾选。
- findings: P757-R1-01（medium，AC-03/T-03/T-04）VM 比较布局上方空白区挤压差异正文，SHOT-03 未合格。P757-D1 登记布局问题，不替代交付；另登记 D2 原生查找输入焦点泄漏、D3 同步搜索阻塞、D4 网站首次搜索索引加载长尾。未删除验收项、未将 MCP 成功外推为原生键盘链通过、未以 Rust 图或历史图补位。
- evidence: `docs/reports/p757-autoedit-verification.md`、`p757-autoedit-capture-catalog.json`、`p757-spec-delta.json`、`p757-autoedit-{workspace,search,diff}-state.txt`、`p757-autoedit-save-recompare.txt`、`media/p757-autoedit-diff-layout.png`、`p757-capture/*.log.gz` 与失败 trace；均在 reviewed_commit 内，避免只依赖临时目录。
- 测试说明：综合面 70 pass / 1 search timeout；新增 EN/ZH 放大检查 2 pass；独立搜索复跑先 fail 后在新端口单 worker 下原断言 19.6s pass。失败和长尾完整保留，无加时/删断言。后续只改文档/证据封装，网站实现和测试、构建依赖均未变化，复用这些已绑定的检查证据，不重复大范围运行。
- 独立性限制：本次复核在实施会话内完成，没有另建未经授权的任务或 agent；重新读取主检出 r2 合同、实际提交 diff/图像/Git 源字节/测试断言，验证来源与规范增量，而非信任完成勾选。未修改 AutoEdit/AutoLang Rust 产品代码、主检出 blueprints WIP 或其他任务。
- next: work。T-01/02 保留完成；T-03/04 保持未完成，current_step=2。正式 VM 比较布局修复后重拍第三图并复验引用，或按明确的范围修订重新立约；在此之前保留空位、执行状态及 worktree，不合入/归档。原生焦点与响应调查需另行处理。

## 10. 待澄清事项
无需要用户决策的项。构建或展示阻塞时保存失败证据，继续独立工作，不冒充成功截图。
SHOT-03 的真实画面不适合宣传，本轮按既有“截图留下空档”方向继续保留；可用的两张已接入本地预览。未获得将问题画面当作合格宣传图或降低 r2 三图要求的授权，不据此标记整体完成。

### Work r3 — 留空合入执行（2026-10-11）

stage: work | plan_id: PLAN-757 | plan_revision: 3 | outcome: pass | code plan-757-dev @ 64f820cfd
tasks: r3 范围修订执行——两图交付 + SHOT-03 留空（P757-D1 缺陷注记）+ rebase 对账
evidence: rebase master 57 commits 零跳过（六文件冲突并集解决：capture-slots 并存 SHOT-01/02/07、AppsOverview 双 ScreenshotSlot、测试断言双特性并集、指南合并 756/757/758 三方更新行、KNOWN-DEBT 折叠分支侧更富诊断）；重放伪影（apps-introduction 断言块被挤）发现并修复（f88e7c90f 后 2 fix 提交）；website build 164.41s；apps-introduction + v05-copy 21/21（唯一推算偏差=总览图数 64→62，探针实证静态 64/水合剔除 2 系统性差异后钉 62）；两 PNG sha256 与 p757-autoedit-capture-catalog.json 逐字节一致（452599c4…/f696c8b6…）
blockers: 无（Category A 零 cargo；store 投影阻塞按 753/756 先例走 §5 回退路径）
next: review

### Review r3 — pass（2026-10-11）

stage: review | plan_id: PLAN-757 | plan_revision: 3 | outcome: pass | reviewed_commit: 64f820cfd（worktree clean，除 playwright 输出目录）| base_commit: a2a4bb244（master @ r3 立约+债务登记后）| dependency_revisions: auto-edit f9ae2a64361f74ad8565fc5df5c4a9aeb5fc10f2（只读）；auto-down 895f8d0f（detached）
spec_inputs: SD-01（application-introductions.md AutoEdit 空位→实拍与来源、第三槽待拍）/SD-02（ui-presentation.md 主图共享、功能图真实来源）——已在分支内落地，与代码/指南对勘一致。
acceptance_results: AC-01 pass（r2 在案：VM 深色官方入口 + CLI/源码身份记录，Rust 画面未入库）；AC-02 pass（r3 口径）——两图自仓真实源码（sha256 与捕获目录逐字节一致：452599c4…/f696c8b6…），比较图功能核查在案（+16/-1、保存重比 +17/-1）而画面因 P757-D1 留空，缺陷已入 KNOWN-DEBT（high，用户指示待修）；AC-03 pass（r3 口径）——两原始 PNG 发布、双语首页/v0.5/总览/专题引用与日期一致（e2e 断言 2026-10-10）、SHOT-03 待拍槽带缺陷说明、九槽保留（captureIds 断言 SHOT-04..12 结构）；AC-04 pass——构建 164.41s + 21/21 定向测试 + 指南/规范更新 + 本复审记录；Category A 零 cargo 合规（分支零 Rust 改动实测）。
findings: 无新增。rebase 并集解与重放伪影修复（2 提交）已复核：断言语义未放宽（总览图数以探针实证钉 62；槽结构/URL/日期断言全部保留），range-diff 非全等之处=冲突解决本身，如实记录不冒充。独立性限制：实施会话内复审，以工件重建（hash 对拍、dist HTML 槽结构、测试重跑、diff 过滤）。
next: merge

### Merge r3 — delivered（2026-10-11）

stage: merge | plan_id: PLAN-757 | plan_revision: 3 | outcome: pass | delivery_commit: bef72060c

| checkpoint | 证据 |
|---|---|
| prepared | reviewed 64f820cfd（r3 pass）；SD-01/02 已随实现提交在分支；依赖只读（auto-edit f9ae2a64/auto-down 895f8d0f）；worktree clean |
| landed | 二次 rebase master（14 commits 纯 docs/plans 实证，网站面零变化）range-diff 6/6 全等（64f820cfd→bef72060c）；主检出 `--ff-only` 落地，master tip=bef72060c；两张 PNG create 入库；master 侧增量纯 docs 无需重建冒烟（分支构建 164.41s 在案） |
| ledger_refreshed | §5 手工回退路径（753/754/755/756 先例）：P757-1/2（designs）+ P757-3（reviews）；website plans.md 757 行；INDEX 再生；读回校验随批 |
| archived | git mv → docs/plans/archive/757-autoedit-website-real-captures.md，status: archived，delivered |
| cleaned | 见下方回填 |

**落地≠部署观察项**：远端网站（112.74.45.241）未重建/部署本批素材（v0.5 收口 R-03/R-04 管辖）；零 Rust 改动，无二进制陈旧项。

**批量回归判定**：收据覆盖至 755（2026-10-11），757 非触发条件，不到期。
