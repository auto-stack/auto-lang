---
plan_id: PLAN-579
status: executing               # drafting → executing → execution_done → reviewed → archived
feature_name: auto-os 立项 Stage A——伞形仓骨架 + 首个真实 app auto-kanban（配置驱动看板，v1 计划板）
author: [zhaopuming, ZCode]
created_at: 2026-09-07
updated_at: 2026-09-07

# /auto-plan:review 结束时填写：
supersedes_spec_components: []
new_spec_components: []
touched_goals: []             # 引用 docs/specs/goals.md 的 GOAL-NNN

affects: []                   # 本计划不改 auto-lang crates/ 代码；specs 沉淀目标为 auto-os 仓自身 ledger
current_step: 8
total_steps: 18
---

# [PLAN-579] auto-os 立项 Stage A——伞形仓骨架 + 首个真实 app「auto-kanban」（v1 计划板）

## 变更摘要

auto-lang 仓的角色重划第一步（用户裁定 2026-09-07）：**auto-lang = 语言/框架根，
auto-os = 产品根（伞形组织根）**。本计划交付 Stage A 两件事，互相验证：

1. **auto-os 伞形仓骨架**（`D:/autostack/auto-os`，新建 git 仓）：README
   （AutoOS 愿景 + Stage A/B/C 三阶段路线）、AGENTS.md（app 仓约定 + 跨仓解析序）、
   auto-plan 四技能范式脚手架（docs/plans/ + .next-id + scripts/new-plan.sh）、
   `.autoos/specs.json` 空 ledger、`apps.manifest`（虚拟伞形清单——**不用 git
   submodule**，机制升级延后到 Stage C 按需决定）。
2. **首个真实 app「auto-kanban」**（`D:/autostack/auto-kanban`，新建 git 仓）：
   **配置驱动的通用只读看板**（用户 2026-09-07 二次裁定：看板不止管计划，
   计划只是首个看板类型，未来以配置文件扩展多类型）。v1 交付：
   `boards.json` 看板注册配置 + 后端看板类型（kind）分发缝 + 泛化 Card 模型 +
   唯一 kind 实现 `lang_plans`（扫描 auto-lang `docs/plans/*.md` +
   `docs/plans/archive/*.md` 的 YAML frontmatter，五态状态机映射看板列）。
   Vue + VM 双端。

**明确不做**（边界）：

- 不做写回——看板是仪表盘不是编辑器（状态翻转归 auto-plan 四技能）；
- 不预建多看板类型机制——v1 只留 seam（boards.json + kind 分发 + 泛化
  Card），第二 kind 的扩展契约/列集自定义等真实需求出现再定稿；
- 不迁移桌面 shell（Stage B，待在途 525/526 折叠后另行立项）；
- 不引入 submodule（Stage C 出现"CI 钉树构建 OS 镜像"类真实需求再议）；
- 不修改 auto-lang `crates/` 任何代码（本计划对 auto-lang 的唯一变更 =
  本计划文件自身）。

**worktree 裁定**：本计划不创建 lang-579 worktree——所有产出落在两个新建
兄弟仓的首提交上（空仓无 master 可污），auto-lang 侧仅本文件在默认检出上
bookkeeping（Plan 529 布局针对"改 auto-lang 代码"的计划，本计划不属此类）。

## 目标

1. `D:/autostack/auto-os` 仓就绪：约定可循（AGENTS.md）、范式可用（取号脚本 +
   空 ledger 可 upsert）、伞形可视（apps.manifest + README Apps 表）。
2. `D:/autostack/auto-kanban` 仓就绪：examples 形态的 AutoUI app（pac.at +
   src/front + src/back + tests + boards.json），`auto run`（Vue）与
   `auto run -r vm`（VM）双端跑通。
3. **配置驱动验证**：boards.json 增删看板条目（同 kind）零代码生效；
   `/api/boards` 如实反映配置（P2-T8）。
4. 看板数据真实性验收：对账真实 auto-lang 检出——计划板各列卡片数之和 ==
   `ls docs/plans/*.md docs/plans/archive/*.md` 文件数（活账对账，C3）。
5. 结构互验：新 app 仓从零在仓外被 auto CLI 完整拉起（gen → run → test），
   证明"examples 归 demo、真实 app 独立仓"的组织形态成立——这是 Stage B
   桌面 shell 搬迁的前置信心。

## 架构方案

### 组织拓扑（Stage A 后）

```
D:/autostack/                    # 扁平兄弟仓布局（现状维持，无嵌套无 submodule）
├── auto-lang/                   # 语言/框架根（compiler/VM/UI 栈/examples demo）
│   └── docs/plans/*.md          # ← 计划板数据源（frontmatter 只读）
├── auto-os/                     # ★ 新建：产品根 / 伞形组织根
│   ├── README.md                #   愿景 + 三阶段路线 + Apps 表
│   ├── AGENTS.md                #   app 仓约定 + 跨仓解析序 + wt-guard 纪律
│   ├── apps.manifest            #   虚拟伞形清单（JSON）
│   ├── docs/plans/ + .next-id   #   auto-plan 范式脚手架（自 001 起）
│   ├── scripts/new-plan.sh      #   自 auto-lang 拷贝适配
│   └── .autoos/specs.json       #   空 ledger（六节空数组，结构对齐 auto-lang）
└── auto-kanban/                 # ★ 新建：首个真实 app 仓（通用只读看板）
    ├── pac.at                   #   仓根（examples 形态；T6 验证不过则转 auto/ 形态）
    ├── boards.json              #   看板注册配置（id/title/kind/root_env）
    ├── src/front/               #   app.at + boards_store.at + pages/board.at
    ├── src/back/                #   api.at（#[api] 端点）+ boards_registry.at（配置+kind 分发）
    │                            #     + sources/lang_plans.at（首个看板类型适配器）
    └── tests/                   #   playwright 套件 + testdata/ 合成语料仓
```

### 看板模型（配置驱动）

```
boards.json                      后端 boards_registry.at                前端
{ "boards": [                    读配置 → BoardDef 列表                  GET /api/boards → 板列表/切换
  { id, title,                   kind 分发：                              GET /api/boards/:id/cards
    kind, root_env } ] }         "lang_plans" → sources/lang_plans.at    → 泛化 Card[] + meta
                                 （未来新 kind = 新 source 模块 + 配置条目）
```

- **泛化 Card**（与数据源解耦）：`{ id, title, column, badge, current, total,
  updated_at, archived, file }`——计划板的适配映射：id=plan_id、
  current/total=current_step/total_steps、column=五态之一。
- **列语义归 source**：v1 lang_plans 的列集 = drafting / executing /
  execution_done / reviewed（四列）+ archived 折叠区；列集自定义属第二
  kind 的扩展课题，不在 v1。

### 数据流（v1 计划板）

```
[front: board]  ←HTTP←  GET /api/boards, GET /api/boards/:id/cards
[back: boards_registry.at] 读 boards.json → 定位 source 模块（kind 分发）
[sources/lang_plans.at]
    resolve_lang_root():  Env.get(board.root_env) → "../auto-lang"(file.exists) → "D:/autostack/auto-lang"
    fs.read_dir(docs/plans) + fs.read_dir(docs/plans/archive)   # 返回 JSON 文件名数组
    File.read_text(每个 .md) → parse_frontmatter() → status→column 映射 → 泛化 Card[]
```

每请求全量重扫（12+40 个小文件，无需缓存/文件监听；刷新按钮=重 fetch）。

### 状态机 → 看板列映射（lang_plans source）

| frontmatter status | column | 说明 |
|---|---|---|
| drafting / draft（legacy 停泊） | drafting | draft 加 `parked` 徽章 |
| executing | executing | |
| execution_done | execution_done | |
| reviewed | reviewed | |
| archived / 其他未知值 | archived | 折叠区：计数 + 最近 5 张（按 updated_at 降序） |

## 技术栈

- **app**：AutoUI（.at DSL）——Vue 轨（`auto run`，vite :17100 → proxy :17101）
  + VM 轨（`auto run -r vm`，iced）；真 app 端口带取 17xxx（沿 auto-os-config
  17700/17701 先例，与 examples 30NN/80NN 带区分）。
- **后端 .at 能力（均已核实存在，双后端同签名）**：`fs.read_dir(dir) str`（返回
  JSON 文件名数组，stdlib/auto/fs.at:20）、`File.read_text(path) str`（041
  editor_store.at:221 用例）、`file.exists(path) bool`、`Env.get(key) str`、
  str 扩展 `split/trim/find/contains/starts_with/sub/len`、`.to_int()`（018
  reading.at:97 用例）、`json` 模块（解析 read_dir 与 boards.json）。
- **测试**：playwright（沿 022-kanban tests/ 形态）+ autoui-verifier 技能
  （`test_vue_playwright.mjs` / `test_vm_mcp.py`）双端一致性。
- **验证门档（AGENTS.md 分级）**：Category A——本计划不改 auto-lang `crates/`，
  **严禁在 auto-lang 跑 `cargo t`/`docs_gen`**；验证全部落在 app 双端运行 +
  playwright + manifest/boards JSON 解析。

## 需求分析与背景调查
（自 .autoos/specs.json overview + 本会话勘察取材；相关 module：auto-cli
run/gen 仓外可用性、stdlib fs/env/json、widgets/blocks 看板所需基础件、
autoui-skill 双端验证基建）

1. **需求是真实的**：auto-lang 已周期性人工盘点计划状态（archive 内
   plans-status-audit-2026-08.md 等三份审计文档）；当前活跃 12 plan
   （8 drafting / 2 executing / 1 reviewed / 1 legacy draft 停泊）+ 归档 40+，
   手工 git grep 维持。计划板 = 把这个动作产品化。
2. **产品定位（用户二次裁定）**：通用看板 app，配置驱动多看板类型，计划板
   只是 v1 首个类型——命名 auto-kanban（管什么容器就叫什么，容器=看板；
   数据源类型才逐板命名）。
3. **数据源规整可浅解析**：v2 frontmatter 字段固定（plan_id/status/
   feature_name/current_step/total_steps/updated_at），逐行 `key: value` +
   剥 `#` 行内注释即可，不需要完整 YAML parser；唯一 legacy 变体 =
   `status: draft`（parked 注记，见 docs/plans/242 头）。
4. **仓外 app 有先例**：auto-os-config（活仓）以 `auto/pac.at` +
   外部 back 工程 + `daemon:` 键在自身仓内被 auto CLI 拉起，端口 17700/17701；
   本计划 app 更简单（单工程 src/front + src/back，examples 022 同构）。
5. **组织名已预留**：`.autoos/specs.json` 命名、AGENTS.md"四技能为 auto-os 仓
   书写"注记、Design 23 AutoOS 愿景（virtual-desktop.md）均在 Stage A 之前
   存在；`auto-ui` 仓 2026-02 起休眠（Plan 096 时代，已被仓内 UI 栈取代），
   命名上不与本计划冲突（伞形根定名 auto-os）。
6. **submodule 否决依据**（用户接受的三阶段裁定）：现有扁平兄弟仓 + 跨仓
   worktree 组（.wt/down-047 先例）+ 解析序约定已工作；submodule 的
   detached-HEAD/更新仪式/与 wt-guard 的交互风险（2026-09-03 .git 删除事故
   教训）在无"钉树构建"真实需求前不值得引入。
7. **examples 022-kanban 不是复用底座**：022 定位是看板 UI 模式教学示例
   （CRUD/拖拽/store 模式演示），本 app 是配置驱动的只读数据看板（数据源
   适配器架构）；仅复用其结构模式（k1 store + 单集合列过滤渲染），不复用代码。

## 详细设计

### 后端：`src/back/boards_registry.at`（配置 + kind 分发）

```auto
pub type BoardDef = { id: str, title: str, kind: str, root_env: str }

// 读仓根 boards.json（File.read_text + json 解析），失败/为空 → 返回错误信息
pub fn load_boards() List[BoardDef]

// kind 分发缝（v1 唯一 kind）
pub fn load_cards(def BoardDef) (List[Card], Meta):
    if def.kind == "lang_plans": return sources.lang_plans.scan(def.root_env)
    else: 返回错误 Meta（未知看板类型）——错误经 meta.source_root 透传前端空态
```

`boards.json`（v1 单条目）：

```json
{ "boards": [
  { "id": "lang-plans", "title": "计划", "kind": "lang_plans", "root_env": "AUTO_LANG_ROOT" }
] }
```

### 后端：`src/back/sources/lang_plans.at`（首个看板类型适配器）

```auto
// 路径解析序（与 AGENTS.md 跨仓约定一致；env 变量名来自 BoardDef.root_env）
fn resolve_lang_root(env_key str) str:
    1. Env.get(env_key) 非空且 file.exists(join(root, "docs/plans")) → 用之
    2. "../auto-lang"（相对 app 运行目录的兄弟检出）同上校验 → 用之
    3. "D:/autostack/auto-lang" 同上校验 → 用之
    4. 全部落空 → 返回错误串（api 层透传给前端显示"数据源未找到"空态）

// frontmatter 浅解析（纯函数，可对拍）
fn parse_frontmatter(text str) 字段集:
    逐行扫描：首行 == "---" 进入，遇第二个 "---" 退出
    行内: i = line.find(":")（跳过不含 ":" 的行）
          key = line.sub(0, i).trim()
          raw = line.sub(i+1, ...).trim()
          c = raw.find(" #")；c >= 0 → raw = raw.sub(0, c).trim()   // 剥行内注释
          raw 空或以 "[" 开头（YAML 列表，如 author）→ 跳过
    字段: plan_id/status/feature_name/current_step(.to_int())/total_steps/
          updated_at；缺失默认 ""/0

// 扫描 + 映射为泛化 Card
pub fn scan(env_key str) (List[Card], Meta):
    for dir in [join(root,"docs/plans"), join(root,"docs/plans/archive")]:
        names = json 解析 fs.read_dir(dir)（过滤 .md 后缀）
        for name: parse_frontmatter(File.read_text(join(dir,name)))
                  → status→column 映射（见架构方案表）
                  → title 缺省 = 文件名 stem；archived 位 = dir 是 archive
                  → Card { id: plan_id, current: current_step, total: total_steps, ... }
```

a2r 工作区注意（沿 022 db.at 的 Plan 401 技术约定）：返回位置的 struct 字面量
先 `var result` 零值 + `let` ctor 构造再赋裸标识；借代迭代字段经 `let` ctor
重建触发 clone。

### 后端：`src/back/api.at`

```auto
pub type Card = {              // 泛化模型，与数据源解耦
    id: str,                   // 计划板 = PLAN-NNN
    title: str,                // 计划板 = feature_name，缺省文件名 stem
    column: str,               // 归一列名（source 责任）
    badge: str,                // "" | "parked"
    current: int, total: int,  // 进度（计划板 = current_step/total_steps）
    updated_at: str, archived: bool,
    file: str,                 // 溯源（详情/未来跳转用）
}
pub type Meta = { source_root: str, scanned_at: str, count_active: int, count_archived: int }

#[api(GET /api/boards)]           pub fn boards() []BoardDef
#[api(GET /api/boards/:id/cards)] pub fn cards(id str) CardsResult   // { cards: []Card, meta: Meta }
```

### 前端

- `boards_store.at`（k1 模式）：`BoardsStore { boards List[BoardDef],
  cards List[Card], meta Meta, current BoardDef, load_boards(), load_cards(id) }`。
- `app.at`：路由壳 `routes{ "/" -> board }` + header（"AutoOS Kanban" +
  当前板 title（boards.json 而来）+ 板切换下拉（v1 单项）+ 数据源路径 +
  扫描时间 + 刷新按钮 → load_cards(current.id)）。
- `pages/board.at`：四列横排（drafting/executing/execution_done/reviewed，
  列头 + 计数徽章；`for card in .store.cards` + `if card.column == ...`
  过滤，022 同型）；卡片 = id 徽章 + title（超长截断）+ 进度条（total>0 时
  current/total 文本 + 百分比条，否则"—"）+ updated_at + badge（parked
  灰标）；底部 archived 折叠行：总数 + 最近 5 张缩略（按 updated_at 降序，
  显隐切换）。空态：meta.source_root 为错误串时显示引导文案（设
  root_env 指定的环境变量或将 auto-lang 检出置于兄弟位置）。
- **板切换语义 v1 边界**：切换仅重拉 cards（store 单卡集），不保多板并发
  状态——多板并存交互留第二 kind 到来时一并定稿。

### fixture 合成语料（`tests/testdata/`）

```
boards.test.json               # 测试用 boards.json：两条目（lang-plans 同款 + lang-plans-2 同 kind 同 root）
                                  → 证明配置驱动注册零代码生效（P2-T8）
fake-lang/docs/plans/
  101-fixture-active.md        # executing, current 3/7, 带 #[api] 无关注释行
  102-fixture-draft.md         # drafting, 0/5
  103-fixture-legacy.md        # status: draft（legacy 停泊 → drafting 列 + parked 徽章）
  104-fixture-no-steps.md      # 无 current_step/total_steps 字段（进度"—"）
fake-lang/docs/plans/archive/
  091-fixture-archived.md      # archived
  092-fixture-archived-old.md  # archived, updated_at 更早（验证排序）
```

测试运行以 `AUTO_LANG_ROOT=<repo>/tests/testdata/fake-lang` 注入（验证解析序
第 1 优先级），断言全确定性。boards 配置切换通过环境变量
`AUTO_KANBAN_BOARDS`（boards_registry 读 boards.json 路径可被 env 覆盖，
默认仓根 boards.json）指向 `tests/testdata/boards.test.json`。

### 仓库骨架细节

- `auto-os/.autoos/specs.json`：`{"sections":[{id:reports,items:[]},…六节]}`
  结构对齐 auto-lang 现文件（reports/goals/architecture/designs/tests/reviews），
  保证未来 upsert 脚本零适配。
- `auto-os/scripts/new-plan.sh`：自 auto-lang 拷贝，`sed` 去除 Plan 529 分组
  worktree 提醒段中 lang 专属措辞（或加一行 auto-os 注记），逻辑不动。
- `apps.manifest`：`{ "apps": [ { "id": "kanban", "name": "通用看板",
  "repo": "../auto-kanban", "kind": "repo", "ports": [17100, 17101],
  "status": "active", "added": "2026-09-07" } ] }`——kind 字段为 Stage C
  预留（repo | subtree | submodule）。

## 测试设计

| 层 | 内容 | 判定 |
|---|---|---|
| P1 解析对拍 | lang_plans 对 fixture 六文件输出（列映射/进度默认/parked 徽章/stem 兜底） | 经 /api/boards/lang-plans/cards curl 断言 |
| P2 playwright | T1 四列标题渲染；T2 fixture 卡片计数（1/1/2/0 列 + archived=2）；T3 进度条与"—"空态；T4 parked 徽章；T5 archived 折叠展开；T6 刷新按钮重 fetch；T7 无 console error；T8 **配置驱动**（boards.test.json 两板条目 → /api/boards 返回 2、板切换下拉两项、第二板卡片可拉） | tests/ 套件全绿（AUTO_LANG_ROOT 指向 fixture） |
| P3 双端一致 | autoui-verifier：vue（test_vue_playwright.mjs）+ vm（test_vm_mcp.py）各一轮 | 两端截图/交互断言一致 |
| P4 真实数据对账 | AUTO_LANG_ROOT 不设，跑真实 auto-lang 检出；`ls docs/plans/*.md docs/plans/archive/*.md \| wc -l` == 计划板总卡数 | 数目相等（P4 截图入 evidence） |
| 门档纪律 | auto-lang 侧零 `cargo t`/`docs_gen`（Category A） | 复审核对 |

## 验收标准

- **C1 仓骨架**：auto-os 五件套齐（README/AGENTS.md/apps.manifest/plan 脚手架
  /specs.json）；`python -c "import json;json.load(open('apps.manifest'))"`
  与 specs.json 同令通过；auto-os `scripts/new-plan.sh smoke` 取号 001 成功
  （验证后删除产物、回滚 .next-id）。
- **C2 双端跑通**：auto-kanban 在 Vue 轨（:17100）与 VM 轨（-r vm）均渲染
  fixture 数据；P2 套件 8 测试全绿；P3 双端一致性记录入本文件复审节。
- **C3 活账对账**：真实数据下，计划板全部卡片数（四列 + archived 计数）==
  `ls D:/autostack/auto-lang/docs/plans/*.md D:/autostack/auto-lang/docs/plans/archive/*.md | wc -l`。
- **C4 零侵入**：auto-lang `git diff master` 仅本计划文件（+.next-id）；两新仓
  `git status` clean；grep 无 debug 残留（console.log/print/debugger）。
- **C5 互链**：auto-os README Apps 表含 kanban 行并链 ../auto-kanban；
  auto-kanban README 注明伞形归属与数据源解析序。

## 执行步骤
（原子任务：精确文件路径 + 确切操作 + 验证命令；每步完成后追加 [✅ 已完成] 一行证据）

- [x] **T1 建仓 auto-os 骨架**
  [✅ 已完成] `git init -b main D:/autostack/auto-os`；README（定位/三阶段路线/目录/ Apps 表占位/关联）+ .gitignore；首提交 f05bcec（`git log --oneline` 1 行通过）。
  `git init -b main D:/autostack/auto-os`；写 `README.md`（AutoOS 产品根定位、
  Stage A/B/C 三阶段路线表、目录结构、Apps 表占位、指针到 auto-lang 为框架根）
  与 `.gitignore`（`*.log` `temp/` `scratch/`）；首提交
  `git -C D:/autostack/auto-os commit -m "init: auto-os 伞形仓骨架 (Plan 579)"`。
  验证：`git -C D:/autostack/auto-os log --oneline` ≥1 行。
- [x] **T2 auto-os AGENTS.md 约定**
  [✅ 已完成] AGENTS.md 五节（范式沿用/解析序红线/app 仓约定/伞形清单/Stage B 桌面）；`grep -c AUTO_LANG_ROOT` = 1 通过；已提交。
  写 `D:/autostack/auto-os/AGENTS.md`：auto-plan 四技能范式沿用（路径映射按
  本仓实际）、跨仓解析序（`AUTO_LANG_ROOT` env → 兄弟 `../auto-lang` →
  `D:/autostack/auto-lang` 主检出）、app 仓结构约定（pac.at + src/front +
  src/back + tests + README 五件）、wt-guard 红线引用（worktree 禁 junction、
  移除前必跑 `bash D:/autostack/wt-guard.sh`）、真实 app 端口带 17xxx 约定。
  验证：`grep -c "AUTO_LANG_ROOT" D:/autostack/auto-os/AGENTS.md` ≥1。
- [x] **T3 auto-os auto-plan 脚手架**
  [✅ 已完成] .next-id=001、new-plan.sh 拷贝并适配（master→main、lang-NNN→os-NNN 组目录两处）、specs.json 六节空数组对齐 auto-lang 结构（顶层仅 sections 键）；冒烟 `new-plan.sh smoke` 取号 001 成功后回滚；提交 71f01f8。
  `mkdir -p docs/plans .autoos`；`echo 001 > docs/plans/.next-id`；拷
  `D:/autostack/auto-lang/scripts/new-plan.sh` → `scripts/new-plan.sh` 并适配
  注释（去 lang-529 worktree 提醒的 lang 专属措辞）；写 `.autoos/specs.json`
  六节空数组（结构对齐 auto-lang 同名文件）。提交。
  验证：`bash D:/autostack/auto-os/scripts/new-plan.sh smoke && ls docs/plans/001-smoke.md`
  （成功后 `rm docs/plans/001-smoke.md`，`.next-id` 回滚 001 再一并提交脚手架）。
- [x] **T4 auto-os apps.manifest**
  [✅ 已完成] manifest（kanban/repo/17100-17101/active）JSON 解析 ok；README Apps 表 T1 已同步该行；已提交。
  写 `apps.manifest`（详细设计节 JSON，含 kanban 一行）；README Apps 表同步该行。
  验证：`python -c "import json;json.load(open('D:/autostack/auto-os/apps.manifest'));print('ok')"`。
- [x] **T5 建仓 auto-kanban 骨架**
  [✅ 已完成] `git init -b main` + README（定位/解析序/boards.json/API/测试/伞形反链）+ .gitignore + pac.at（name auto-kanban/scene ui/front_port 17100/back_port 17101）+ boards.json（lang-plans 单条）+ 目录（src/front/pages、src/back/sources、tests）；首提交 668ac9d。
  `git init -b main D:/autostack/auto-kanban`；写 `README.md`（首个真实 app
  定位：配置驱动通用只读看板、v1 计划板、数据源解析序、运行/测试方式、伞形
  归属反链）、`.gitignore`（`node_modules/ gen/ dist/ target/ *.log`）、
  `pac.at`（`name: "auto-kanban"`、`scene: "ui"`、`front_port: 17100`、
  `back_port: 17101`，参 022/028 字段集）、`boards.json`（详细设计节单条目）；
  建目录 `src/front/pages/ src/back/sources/ tests/`。首提交。
  验证：`git -C D:/autostack/auto-kanban log --oneline` ≥1 行。
- [x] **T6 仓外 CLI 可用性验证（关键前置）**
  [✅ 已完成] **仓根 pac.at 形态成立**：`auto gen` 于 D:/autostack/auto-kanban 退出码 0（两次），gen/front/vue + router/index.ts（1 路由）生成——无需转 auto/ 形态分支。auto CLI = auto-lang target/debug/auto v0.4.1。副作用：`.am/` 状态目录入 .gitignore；S001 INFO（text prop schema 注记）INFO 级不阻塞，与 examples 同现象。
  在 `D:/autostack/auto-kanban` 放最小 front（hello 级 `src/front/app.at`）后跑
  `auto gen`。**分支**：成功 → 维持仓根 pac.at 形态；失败 → 迁就 auto-os-config
  先例改 `auto/pac.at` 形态（pac.at 移入 `auto/`，back 路径相应调整），并在本步
  证据行记录实测结论（此为形态适配，非 workaround 债）。
  验证：`auto gen` 退出码 0，产物目录（gen/ 或等价）生成。
- [x] **T7 后端类型与端点骨架**
  [✅ 已完成] api.at 四类型（BoardDef/泛化 Card/Meta/CardsResult，列表字段沿 015 `tags:[]str` 先例）+ 两端点（GET /api/boards、GET /api/boards/:id/cards）；registry/lang_plans 空壳。**结构适配**：后端平铺模块（boards_registry.at + lang_plans.at，031 先例）替代 sources/ 子目录（嵌套 back 目录无先例）；scan 直接返 CardsResult 消元组返回风险。auto gen exit=0 零 error。
  写 `src/back/api.at`：`pub type Card`（泛化）、`Meta`、`BoardDef`、
  `CardsResult`（字段见详细设计）；`#[api(GET /api/boards)] boards()` 与
  `#[api(GET /api/boards/:id/cards)] cards(id)` 骨架（暂返空）。写
  `src/back/boards_registry.at` 与 `src/back/sources/lang_plans.at` 空壳
  （仅模块头注释）。
  验证：`auto gen` 重跑退出码 0（类型/路由代码生成通过）。
- [x] **T8 boards 注册与 kind 分发**
  [✅ 已完成] `auto run --server vm`（VM HTTP 后端，a2r 路径被阻断后的一等公民替代）下 `curl :17101/api/boards` HTTP 200 返回 lang-plans；**配置驱动实测**：boards.json 加第二同 kind 条目即时返回两板（每请求读配置零重启），还原后回到一板；未知 id → meta.source_root 透传 "unknown board id"。VM 上下文三项适配（探针定案）：stdlib 显式 `use auto.*`、裸数组容器（无 List[T]，Undefined variable: List）、动态 JsonValue 导航替代 json.decode[T]（VM codegen 不支持泛型实例化调用）。提交 bf98b1d。
  写 `boards_registry.at`：`load_boards()`（读 boards.json——路径默认仓根、
  可被 `AUTO_KANBAN_BOARDS` env 覆盖；json 解析 + 字段校验）与
  `load_cards(def)`（kind 分发：v1 仅 `lang_plans`，未知 kind 错误透传）；
  api.at 两端点接线。
  验证：`auto run` 后 `curl -s :17101/api/boards` 返回 lang-plans 一条；
  临时改 boards.json 加第二条同 kind 条目 → curl 返回两条 → 还原（P2-T8
  将固化为测试）。
- [ ] **T9 lang_plans 适配器（解析 + 扫描 + 映射）**
  [⛔ 阻断于框架 bug，详见待澄清 #1/#2] 代码已按实测语义完成（raw 提取
  `sub(ci+1, t.len())`——实测 (start,end) 语义与 master stdlib 文档 (start,len)
  注记不符、json 元素 `""+name` 物化、五态映射/parked 徽章/stem 兜底），
  提交 5d01bd3（WIP）。**验证结论**：fixture 体量（6 文件）扫描全通（探针
  repro2：6 卡/4 active）；**真实语料（55 文件）100% 撞 VM heap_rc 体量触发
  型 tombstone panic**（repro1，五种代码形态规避均败——详见
  auto-kanban/docs/vm-bug-repro/README.md 归因）。T9 的"真实数据 PLAN-577
  在列"验证在 VM bug 修复前无法达成；T10-T15 可降级走 fixture 先行
  （选项 B，待用户裁决）。
  写 `sources/lang_plans.at`：`parse_frontmatter()`（详细设计算法：定界扫描/
  key: value/剥 ` #` 注释/`[` 列表跳过/缺省值）+ `resolve_lang_root(env_key)`
  （三级序 + exists 校验 + 错误串）+ `scan()`（read_dir 双目录/json 解析/
  .md 过滤/read_text/状态映射 + parked 徽章 + title=stem 兜底 + archived 位
  + 泛化 Card 装配）；cards 端点接线（meta 返回 source_root/scanned_at/计数）。
  a2r 约定：返回 struct 走 `var result` + `let` ctor。
  验证：`curl -s :17101/api/boards/lang-plans/cards` 返回 JSON 数组且
  PLAN-577 等真实 plan_id 在列、列值符合映射表；
  `AUTO_LANG_ROOT=D:/nonexistent curl` 降级序仍正确（兄弟 → 主检出）。
- [ ] **T10 前端 store**
  写 `src/front/boards_store.at`：`BoardsStore { boards, cards, meta,
  current, load_boards(), load_cards(id) }`，经 `use back.api` 拉两端点
  （k1 模式，参 022 board_store）。
  验证：`auto gen` 通过（front 引 back 类型可解析）。
- [ ] **T11 前端壳与路由**
  写 `src/front/app.at`：header（"AutoOS Kanban" + 当前板 title + 板切换下拉
  （v1 单项）+ 数据源路径 + 扫描时间 + 刷新按钮）+ `routes{ "/" -> board }`；
  占位 board 页。
  验证：`auto run` 打开 :17100 见 header 与空板。
- [ ] **T12 board 列与卡片渲染**
  写 `src/front/pages/board.at`：四列横排（列头 drafting/executing/
  execution_done/reviewed + 计数徽章）；卡片 = id 徽章 + title 截断 +
  进度条（total>0：`current/total` + 百分比；否则"—"）+ updated_at +
  parked 灰标（`for` + `if card.column ==` 过滤，022 同型）。
  验证：浏览器人工目检真实数据四列卡片渲染正确（截图存
  `D:/autostack/auto-kanban/screenshots/t12-board.png`）。
- [ ] **T13 archived 折叠与空态**
  board.at 底部 archived 折叠行（总数 + 最近 5 张按 updated_at 降序 + 显隐
  切换）；meta 错误串空态引导文案。截图 `t13-archived.png`。
  验证：目检折叠展开交互 + 断开数据源（AUTO_LANG_ROOT 指向空目录）见空态。
- [ ] **T14 fixture 合成语料**
  写 `tests/testdata/fake-lang/docs/plans/{101,102,103,104}-fixture-*.md` 与
  `docs/plans/archive/{091,092}-fixture-archived*.md`（详细设计节规格）+
  `tests/testdata/boards.test.json`（两板条目）。
  验证：`AUTO_LANG_ROOT=<abs>/tests/testdata/fake-lang AUTO_KANBAN_BOARDS=<abs>/
  tests/testdata/boards.test.json curl -s :17101/api/boards/lang-plans/cards`
  六条且列映射符合设计表；`curl :17101/api/boards` 两条。
- [ ] **T15 playwright 套件**
  `tests/` 建 playwright（package.json + spec，沿 022 形态）：T1 四列标题；
  T2 计数 1/1/2/0 + archived 2；T3 进度条与 "—"；T4 parked 徽章；T5 折叠展开；
  T6 刷新重 fetch；T7 零 console error；T8 配置驱动（两板条目 → 板列表 2、
  切换下拉两项、第二板 cards 可拉）。启动脚本注入 AUTO_LANG_ROOT=fixture 与
  AUTO_KANBAN_BOARDS=boards.test.json。
  验证：`cd D:/autostack/auto-kanban/tests && npm install && npm test` 全绿。
- [ ] **T16 双端一致性验证**
  autoui-verifier：Vue 轨（fixture 注入，:17100）+ `test_vue_playwright.mjs`；
  VM 轨 `auto run -r vm` + `test_vm_mcp.py`；真实数据各补一轮（P4 对账）。
  验证：双端断言绿；`ls D:/autostack/auto-lang/docs/plans/*.md
  D:/autostack/auto-lang/docs/plans/archive/*.md | wc -l` == 计划板总卡数。
- [ ] **T17 伞形终登记与互链**
  复核 `apps.manifest` ports/status 与实测一致；auto-os README Apps 表与
  auto-kanban README 反链定稿；两仓分别提交
  （`feat(app): auto-kanban 通用看板 v1——计划板只读双端 (Plan 579)` /
  `feat(umbrella): 登记 kanban 首个真实 app (Plan 579)`）。
  验证：C5 互链 grep 通过。
- [ ] **T18 收尾健康检查**
  双新仓 `git status` clean；`grep -rn "console.log\|debugger\|print(" src/`
  零残留；auto-lang `git status` 仅本文件与 .next-id 变更；本文件回填
  各任务 [✅] 证据与 updated_at。
  验证：三条 grep/status 命令输出贴入证据。

## 复审记录

## 待澄清事项

1. **[T8/T9 执行期发现·阻断 T9 真实数据验证] VM heap_rc 字符串池体量触发型 tombstone bug**
   扫描真实语料（55 文件，~15 万池操作）100% 撞 `[RC canary] string tombstone
   access`（engine.rs:1744，Plan 419 heap_rc canary）；6 文件 fixture 幸存。
   已系统排除：struct 字段写/to_int/Card 构造累积/break/fn 返回 struct/模块级
   var（db.at 同型也炸）/sub 语义误用（修正后仍炸）。二进制基线 ecc27c81e
   含 Plan 567 83ae4e9a6 string-concat 修复且其后 vm/ 零变更 → master 现存。
   **最小复现器×5 已入库** auto-kanban/docs/vm-bug-repro/（README 含完整归因
   与修复建议）。**连带三发现**：① sub 语义文档 (start,len) vs 实测 (start,end)
   不符（repro3）；② JsonValue 元素上直调 str 方法返回 None 语义（物化
   `""+name` 规避）；③ 仓外 a2r rust-server 后端生成落 auto-lang 仓内（污染
   框架仓）且 features=["ui",...] 撞 terminal/mod.rs:42 未门控 `pub use iced`
   → E0432 必炸（原待澄清 #1 内容，1 行 cfg 门控 + 落点外移可解，越本计划
   "不改 crates/"红线，移交裁决）。**两个处置选项待用户裁定**：
   A=停 T9，先立 VM 修复微计划（repro1 即最小复现，顺修 ①③），修复后回本
   计划续行（app 后端零改动即可全量跑通）；B=T10-T15 降级走 fixture 先行
   （VM server + 6 文件已实测可用），C3 真实数据对账挂账待 A 落地。
2. ~~app 仓命名~~ **已裁定（用户 2026-09-07）**：`auto-kanban`——定位为配置
   驱动的通用只读看板，计划（lang_plans）是首个看板类型；未来新看板类型 =
   新 source 模块 + boards.json 条目。
2. **第二看板类型扩展契约**（v1 后）：kind 注册机制形态（模块命名/配置 schema
   /列集自定义/多板并存交互）留待第一个真实新类型需求出现时定稿——v1 只留
   seam（boards.json + kind 分发 + 泛化 Card），不预建机制。
3. **pac.at 仓根形态**（T6 分支点）：examples 形态（仓根）优先；若 auto CLI
   仓外仅支持 auto-os-config 的 `auto/` 子目录形态，则按先例适配并在 T6 证据
   行记录——属形态选择，不登记 KNOWN-DEBT。
4. **VM 轨 iced 端口**：VM 模式无 vite proxy，:17100/:17101 直连形态以
   autoui-verifier 实测为准，若 VM 轨端口注入存在 examples 时代已知缺口
   （022 README 端口注记），以 env 覆盖同法处置并在 T16 证据行注记。
5. **Stage B 启动条件**（非本计划）：在途 525/526 折叠 + 541/566/576/577
   落地后另行立项桌面 shell 搬迁（L2：先设计文档后拆 plan）。
6. **auto-ui 拆仓/迁移不并入 Stage B**（用户裁定 2026-09-07）：其讨论
   前置 = ①虚拟桌面建仓结束（Stage B 完成）+ ②新虚拟桌面跑起来 +
   ③能够展示两个 gallery（画廊上架见 PLAN-578；UI 栈/示例/画廊同属
   auto-ui 项目资产的归属叙述见 PLAN-578 待澄清③）。满足后**独立立项**
   讨论，Stage B 设计不裁定 auto-ui 归属。
