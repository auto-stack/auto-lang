---
plan_id: PLAN-666
status: archived            # drafting → executing → execution_done → reviewed → archived
feature_name: ui-demo-repatriation
author: [agent]
created_at: 2026-09-20
updated_at: 2026-09-20
completion_kind: delivered

# /auto-plan:review 结束时填写：
supersedes_spec_components:
  - docs/specs/auto-lang/ui/overview.md#资产位置注记（PLAN-590 修订 2026-09-20）  # 旧"三 demo 在 auto-os/apps/"单一表述废止
new_spec_components:
  - docs/specs/auto-lang/ui/overview.md#ui-gallery-双轨事实源契约（PLAN-666）  # SD-01+SD-02（发射器缺席语义并入）
touched_goals: [GOAL-010]

affects: [ui/overview.md, auto-man/overview.md]
current_step: 7
total_steps: 7
---

# [PLAN-666] ui-demo-repatriation

## 0. 变更摘要

把 PLAN-590（Stage B P-5）迁出 auto-lang 的三个 demo——`025-sys-monitor`、
`028-launcher`、`038-minesweeper`——以 auto-os 侧当前快照**复刻回**
`examples/ui/`，恢复编号教学系列的完整性与 ui-gallery 画廊条目；auto-os
侧原件不动、继续独立演化（用户 2026-09-20 裁定：**分道扬镳**——examples/ui
= demo/教学轨，auto-os = 真实 app 轨，两轨无同步义务、仅 README 互链）。

配套做**锚点分家**：PLAN-590 迁出时刻意改指 auto-os 的 auto-lang 测试/文档
语料锚，按"demo 面读 examples/ui 副本、app 面读 auto-os 原件"的边界逐项
回接或维持。auto-os 仓源码零改动，仅一笔画廊再生产物提交（PLAN-658 先例
"os 侧产物 b56afa7"）。

## 1. 目标

1. `examples/ui/` 编号系列补回 025/028/038 三个洞（025-dashboard 空壳与
   038 未跟踪残骸顺带清除）。
2. ui-gallery 画廊恢复三 demo 条目——auto-man 发射链
   （`generate_gallery_host`/`refresh_gallery_registry`）扫 examples/ui
   全量覆写 registry（web/VM 双臂），回源后自动跟随。
3. 锚点分家边界成文并落锚：画廊/docs 语料锚读 `examples/ui` 副本；桌面
   app 面（plan503 launcher 语料、桌面注册表）维持读 auto-os 原件。
4. examples/ui README 恢复三行表项，PLAN-590 迁出注记改写为双向互链。

**非目标**：

- 不重写/简化三个 demo（用户裁定：先快照复刻，"先直接挪过去就行了（反正
  能跑）"；将来有教学简洁度需求另立计划）。
- 不动 045/046"画廊两件"（维持画廊自有现状）。
- 不填 032–044 老洞、不动 036-tetris / 037-klondike（auto-os 原生 app）。
- 不建立两轨同步机制（分道扬镳：复刻即快照，此后各自演化）。

## 2. 架构方案

**拷贝面**（auto-os `apps/{025-sys-monitor,028-launcher,038-minesweeper}`
→ auto-lang `examples/ui/` 同名目录）：

- 仅 git tracked 文件（三 app 合计 30 个 tracked 文件：025×15、028×8、
  038×7），`.auto/`、`.am/`、`tests/test-results/` 等未跟踪缓存/产物零
  带入（tracked 内容如 038 的历史 fit 截图则照带，对账以 tracked 清单为准）。
- 端口沿用快照原值（025=4025/8425、028=4028、038=4038），已核实
  examples/ui 现有 pac 无占用。
- 快照语义：auto-os 侧迁移后已演化 19 提交（sys-monitor 的 PLAN-024
  mini 重设计/栅格条等），复刻的是**当前**形态，此后两轨各自演化。

**发射链**（零新机制，纯跟随）：ui-gallery `auto run` → auto-man
`generate_gallery_host` / `refresh_gallery_registry` 扫 examples/ui →
确定性覆写 `ui-gallery/src/demos-registry.ts`（web 臂）、
`src/front/registry.at`（VM 臂）、`src/gallery/demos/*.at` 适配器与
AppViewport（PLAN-658/662 契约）。三 demo 回源后条目自动恢复，档位
（loadable/fullstack/route_stub）以扫描器实测定案。

**锚点分家**（PLAN-590 `cf103b70d`/`5525ccd16` 的选择性反向）：

| 面 | 锚点 | 读向 |
|---|---|---|
| demo 面（回接） | scan 断言 33→36、策展 C 档 17→20（三 pac 均 `desktop: "true"`）、docs_gen 4 测、cmd_docs 脚手架、gallery_golden、plan370/409/412/492_m4 语料 | examples/ui 副本 |
| app 面（维持） | plan503 launcher 桌面语料、`ui/app_registry` 桌面注册表、os_paths 解析器本身 | auto-os 原件 |
| 不动面 | gallery_pages_compile / schema_drift 第 10 源（锚 widgets-gallery，与本批无关） | auto-os |

**跨仓注记**：本计划主导仓 = auto-lang（全部源码/测试/文档改动）；auto-os
仅一笔 ui-gallery 再生产物提交（消息带 PLAN-666 指针），先例 PLAN-658。
auto-os 侧 apps.manifest 不动（三 app 本就未登记为桌面 app）。

## 3. 技术栈

- `.at`/AutoUI：pac.at + src/front（+ 025 的 src/back）。
- auto-man 画廊发射器（`crates/auto-man/src/vue.rs` :4500 起）。
- auto-lang Rust 测试面（crates/ 内断言计数与语料锚——本计划**改
  crates/**，故 cargo t 门适用，无 Category A 限制冲突）。
- Bash/PowerShell 双壳拷贝对账（字节级 diff -r，排除清单制）。

## 4. 需求分析与背景调查

**授权记录**（用户 2026-09-20 会话裁定）：

- 方向：迁移走并产生序号空洞的 app 复刻回 auto-lang，未来分道扬镳——
  examples/ui 目标 demo/教学、不追求功能完善但代码简单；auto-os 项目
  慢慢扩展成真 app。
- 手法：先不重写以免修改，直接挪（"反正能跑"）——快照复刻。
- 范围：025/028/038 三件；045/046 维持画廊自有；036/037 与老洞不掺。
- 预算/自动续跑限制：未设。

**已核实事实**：

- 迁移批 = auto-lang `047743158`（本体 git rm，去向 auto-os `8a91761`，
  字节级对账）+ `cf103b70d`（引用清零与测试面重锚：os_paths 解析器、九处
  画廊语料锚、scan 36→33、C 档 20→17、gallery_apps_dir 兄弟探测、
  deploy-website、README 去向注记）+ `5525ccd16`（门禁修正：ui_gen/rust.rs
  PLAN-533 页级断言、plan492_m4 第三副本、component_registry_test、e2e
  fixture、a2ts 探针、plan503 launcher 语料重锚、plan502 助手替换）。
- 端口 4025/8425、4028、4038 在 examples/ui 现有 pac.at 零占用。
- 三 app pac 均含 `desktop: "true"` → 回迁后策展集 +3。
- `docs/plans/665-bp-sandwich.md` 为并行会话在册占用（未提交），本计划
  取 666 无冲突。
- examples/ui README 现有：PLAN-590 迁出 callout（L14-15）、"编号说明
  （024–040 空洞的来历）"节（含"新示例优先填入空洞"规则与 025 去向行）。
- 老洞来历（README 成文）：021-block-static、026–040 原能力样板已迁
  `examples/capability-tests/`（PLAN-552），与本批无关。

## 5. 详细设计

### 5.1 拷贝清单与清损

- `apps/025-sys-monitor`（15 tracked：src/front×6、src/back×2、tests×5、
  pac+README）→ `examples/ui/025-sys-monitor/`；
- `apps/028-launcher`（8 tracked）→ `examples/ui/028-launcher/`；
- `apps/038-minesweeper`（7 tracked）→ `examples/ui/038-minesweeper/`；
- 清除：`examples/ui/025-dashboard/`（仅剩 `.am` 缓存空壳）、
  `examples/ui/038-minesweeper/` 现存未跟踪残骸（git 已删、磁盘遗留）。
- 对账：`git -C auto-os ls-files` 为源清单；落位后 `diff -r` 全等（预期
  CRLF 规范化容差，沿 PLAN-590 对账口径：字节级或规范化后全等）。

### 5.2 锚点回接清单（demo 面）

以 `git show cf103b70d` / `git show 5525ccd16` 为反向操作依据，逐项：

1. scan 断言 33→36、C 档 17→20（`crates/auto-lang/src/ui_gen/docs_gen.rs`
   、`crates/auto-lang/tests/docs_gen.rs`、`crates/auto/src/cmd_docs.rs`）；
2. docs_gen 4 测 + cmd_docs 脚手架中的语料路径（若有指向三 demo 的
   auto-os 臂，回读 examples/ui）；
3. gallery_golden（`crates/auto-lang/tests/gallery_golden.rs`）预期面；
4. plan370_test_support / plan412_tests / plan492_m4_tests 语料（492_m4
   "第三副本"语义 T-02 普查后定）；
5. README 去向注记反向（见 5.3）。

维持不动：plan503_tests（launcher 桌面料）、plan502_diagram_tests、
gallery_pages_compile/schema_drift（widgets-gallery 锚）、os_paths 解析
器、app_registry。

**T-02 锚点普查决策清单（2026-09-20 实测，覆盖 cf103b70d 全 18 文件 +
5525ccd16 全 7 文件）**——重要勘定：两提交的"九处画廊语料锚"实锚
**widgets-gallery/charts-gallery**（590 批同迁 auto-os 顶层的画廊两件），
并非三 demo；三 demo 在 crates/ 内的真实锚点只有 scan/C 档断言一处。
plan5.2 预设的"docs_gen 4 测/cmd_docs/gallery_golden/plan370/409/412/
492_m4 语料回读 examples/ui"经普查**不成立**（全部 widgets-gallery 面，
维持 auto-os 解析序定位），实际回接面收窄为下表 R 系：

| # | 锚点（文件） | 590 时的改动 | 面裁定 | 666 动作 |
|---|---|---|---|---|
| R-1 | `ui/app_registry.rs` mod tests scan 断言 | ≥34→≥33 | **demo 面** | 回接 ≥36（33+3 复刻回源） |
| R-2 | 同文件 C 档 want 清单+消息 | 删三 id（20→17） | **demo 面** | 恢复三 id（17→20），消息补 PLAN-666 |
| R-3 | `examples/ui/README.md` | 迁出 callout+空洞注记 | **demo 面** | T-04 改写（复刻回源+互链） |
| R-4 | `docs/specs/auto-lang/ui/overview.md` 资产位置注记 | 三 demo 去向 auto-os/apps | **demo 面** | 注记改写（demo 轨回源）+SD-01 落册（T-07） |
| K-1 | `os_paths.rs`+`lib.rs`+app_registry re-export | 新增解析器 | 基建 | 不动（两面共用） |
| K-2 | docs_gen.rs/tests/docs_gen.rs/cmd_docs.rs（4 测+脚手架） | widgets-gallery 解析序 | widgets 面 | 不动（045/046 画廊两件维持 auto-os） |
| K-3 | gallery_golden.rs / schema_drift.rs / gallery_pages_compile_tests.rs | widgets-gallery | widgets 面 | 不动 |
| K-4 | plan370_test_support/plan409/plan412/plan502_diagram | widgets-gallery pages/components | widgets 面 | 不动 |
| K-5 | plan492_m4 三副本（charts-gallery/024-charts/widgets-gallery） | 第三副本改 os_paths 解析 | widgets 面 | 不动（"第三副本"=widgets-gallery，与三 demo 无关，裁定闭合） |
| K-6 | plan503_tests launcher 桌面料 | 重锚 auto-os/apps/028 | **app 面** | 维持 auto-os 原件（桌面注册表语义） |
| K-7 | deploy-website.yml 画廊段 | checkout auto-os | 画廊面 | 不动（ui-gallery 物理在 auto-os） |
| K-8 | auto-man/vue.rs gallery_apps_dir 兄弟探测 | 补 ../auto-lang/examples/ui | 发射机制 | 不动（正是三 demo 回流画廊的通道） |
| K-9 | a2ts 探针 / ui_gen/rust.rs 533 断言 / component_registry_test / fixtures/pkg_app.at 删除 | widgets-gallery/tsc 归位 | widgets 面 | 不动（fixture 维持删除） |
| K-10 | musk_vm_track/session/iced renderer/vm engine 字面量引用 | 非锚（字符串/注册表 id，不读盘） | 非锚 | 零动作 |

**desktop_mcp.py 裁定**：三 demo 的 `tests/desktop_mcp.py` 为 app 自带
手动 MCP 探针脚本，随快照携带，无 auto-lang cargo 门禁引用——零接入
动作（demo 轨语义=能跑即可，不接入框架门禁）。
**render_filter 断言**：三 demo pac 均 `render: "vue"`，vm 过滤腿提示
"（041/024 等）"无需恢复 025（025 当年 vm 腿已被 590 前形态变更覆盖）。

### 5.3 README 与互链

- `examples/ui/README.md`：PLAN-590 callout 改写为"025/028/038 已于
  PLAN-666 复刻回本目录（demo 轨）；真实 app 轨在 auto-os `apps/` 各自
  演化"；表恢复三行；"编号说明"节 025 行补复刻去向。
- auto-os 侧三 app README（如无则 pac 注记不强制）加一行反链
  "demo 教学版：auto-lang examples/ui/0NN"。此项落 auto-os 再生产物提交
  或独立小提交，不扩源码面。

### 5.4 规范增量

| delta_id | add/modify/retire | docs/specs/... target | before/after rule | rationale | acceptance IDs |
|---|---|---|---|---|---|
| SD-01 | modify | docs/specs/ui/overview.md（§ui-gallery 邻近） | before：画廊语料单一源自 examples/ui（590 后三 demo 缺席由 auto-os 臂补）；after：examples/ui 为 demo 面**唯一**事实源（画廊发射/docs 语料/scan 策展读它），auto-os apps 为 app 面事实源（桌面注册表/桌面测试语料读它），两轨分道无同步义务 | 分道扬镳裁定的规范固化，防未来再漂移 | AC-03/AC-05 |
| SD-02 | modify | docs/specs/auto-man/overview.md | before：发射器扫 examples/ui 全量覆写（未记缺席语义）；after：补记"语料缺席即条目缺席"的确定性语义与 demo 面单一源契约（引 SD-01） | 发射链行为契约补全 | AC-04 |

## 6. 测试设计

- **冒烟**：三 demo 在 auto-lang `auto run` 各一腿；038 追加
  `--render vm` 腿（其 pac 支持 vm 模式）。
- **auto-lang 定向门**：docs_gen 4 测、gallery_golden、plan370/412/492_m4
  、scan 断言所在 cargo 目标——零新增红（master 预存红基线对账，沿
  PLAN-590 R3 口径）。
- **全量档**：`cargo t` + tv/tf 按 PLAN-590 先例口径跑一遍对账（tv/tf
  vs master 零新增确定性红）。
- **画廊再发射**：auto-os `ui-gallery` `auto run` → registry 双臂含三
  demo 条目（确定性守卫触发再生成）→ 截图/条目 dump 为证。
- **对账**：拷贝面 diff -r 全等；examples/ui `git status` 无杂物混入。

## 7. 验收标准

- **AC-01** 三目录落位且快照对账全等：`examples/ui/{025-sys-monitor,
  028-launcher,038-minesweeper}` tracked 文件与 auto-os `apps/` 同名
  目录字节一致（CRLF 规范化容差）；025-dashboard 空壳与 038 残骸清除。
  验证：`git ls-files` 清单 + `diff -r` 输出空（容差内）。
- **AC-02** 三 demo 可跑：auto-lang 侧 `auto run` 冒烟绿×3 + 038
  `--render vm` 腿绿。验证：运行日志/首帧证据。
- **AC-03** auto-lang 门禁零新增红：定向集 + 全量档 vs master 基线对账，
  红集全等或零新增且逐条归因。验证：tv/tf 输出对账记录。
- **AC-04** 画廊条目恢复：ui-gallery 再发射后 demos-registry.ts 与
  registry.at 均含三 demo（档位以扫描器定案记录在案）；auto-os 再生产物
  提交在案（消息含 PLAN-666）。验证：registry dump + auto-os commit。
- **AC-05** 锚点分家落定：5.2 清单逐项回接/维持有 diff 与验证记录，
  plan503 等 app 面锚 diff 为零。验证：`git show cf103b70d` 反向对照表。
- **AC-06** README/编号说明更新且互链双向可解析。验证：链接目标存在。

## 8. 执行步骤

- [x] **T-01** 拷贝与清损（5.1 清单；`git -C D:/autostack/auto-os ls-files`
  取源清单 → 复制 → `git add`；清 025 空壳/038 残骸；diff -r 对账）。
  验证：AC-01。依赖：无。
  [✅ 2026-09-20 commit `17bf19706`] 30 tracked 文件落位（025×15/028×8/
  038×7），diff -r 字节级全等（零容差需求）；根 .gitignore 加 038 历史截图
  反选（旧规则吞 png）；worktree 新检出自净无残骸，主检出磁盘残骸（025-
  dashboard/.am、038 的 .am/.auto/examples/src 遗留，全未跟踪无入口 .at）
  **待合并前清理**（已核 git ls-files 空 + find 无 app.at，零扫描面影响）。
- [x] **T-02** 锚点普查（bounded investigation）：`git show cf103b70d
  5525ccd16` 全量锚点逐项标注 demo 面/app 面/不动，产出决策清单落本节
  下方（决策件；492_m4 第三副本语义、desktop_mcp.py 038 腿在此裁定）。
  验证：清单覆盖两提交全部锚点文件。依赖：T-01。
  [✅ 2026-09-20] §5.2 下方 R/K 决策清单落档，覆盖 cf103b70d 18 文件 +
  5525ccd16 7 文件全量 25 项。关键勘定：两提交画廊语料锚实锚 widgets/
  charts-gallery（非三 demo），回接面收窄至 R-1..R-4；492_m4"第三副本"
  =widgets-gallery 经 os_paths 解析（K-5 不动）；desktop_mcp.py=app 自带
  手动探针（快照携带、零门禁接入）。
- [x] **T-03** 锚点回接（按 T-02 清单执行 5.2；scan/C 档计数重基线；docs_gen
  /golden/plan 语料回读 examples/ui）。验证：AC-03 定向集。依赖：T-02。
  [✅ 2026-09-20 commit `390b3d998`] app_registry scan 断言 ≥33→≥35（实测
  定数=master 净盘 32+3；590 注记"36"为 ui 轨整编前旧基数，且 master 现值
  32<33 系整编 043-046 迁出后未重基线的预存红——本改一并转绿）；C 档
  want 17→21（复刻三 id + 补 047-bp-admin want 漏更[Plan 657 上桌未更，
  9c6c27e86 同型预存红]）；render_filter 断言零动作（三 pac 均
  render:vue）；docs_gen/golden/plan 语料锚经 T-02 勘定全为 widgets 面，
  无需回读。app_registry 24/24 绿。
- [x] **T-04** README 恢复与互链（5.3）。验证：AC-06。依赖：T-01。
  [✅ 2026-09-20 commit `c9d2d8237`] PLAN-590 callout 改写分道扬镳注记；
  表恢复 025/028/038 三行（端口/desktop✓/史录链接）；编号说明 025/038
  行补复刻去向；独立应用节+毕业节改写；407 链接修 archive/ 路径。链接
  目标全部存在（ls 核验 archive/541、438、464、441、407、445）。auto-os
  侧反链归 T-06（038 有 README；025/028 仅 SPEC.md → 单向+pac 口径）。
- [x] **T-05** 三 demo 冒烟四腿 + 定向门（6 节）。验证：AC-02。依赖：T-01。
  [✅ 2026-09-20] 四腿：038 vue=HTTP 200（localhost:4038 Vite ready）；
  028 vue=HTTP 200（4028）；025 vue=前端 200（4025）+后端 8425 编译红
  （29 错 E0308/E0425/E0599——与 auto-os 原件同命令**对称同红**，CLI
  codegen 演化致生成后端漂移，预存非复刻引入，债 P666-D1）；038 VM=
  test_vm_mcp.py 绿（AURA 首帧快照"💣 10"、截图 examples/ui/038-
  minesweeper/tests/screenshots/p666_038_vm_initial.png、PLAN-646
  envelope 双查过、干净终止）。定向门：docs_gen 4/4、app_registry
  24/24、plan370/412/492_m4 45 跑 43 过（红=plan370_015 d10+plan492_m4
  c2，master 同命令同红集）、gallery_vue_golden 与 master 同红（590
  在案基线漂移）。**零新增红**。另注：038 `auto build`（vue-tsc 档）红
  =两侧对称预存（生成 useMinesweeperStore.ts 类型错，auto-os 同错）。
- [x] **T-06** auto-os 侧 ui-gallery 再发射 + 产物提交（跨仓单 commit，消息
  `gallery: PLAN-666 产物刷新——025/028/038 回源条目恢复`+3 app README
  反链若做则并入）。验证：AC-04。依赖：T-01/T-03。
  [✅ 2026-09-20 auto-os `24aa01f`] 执行发现与调整：发射器对扫描副本的
  绝对路径敏感面实证（031 fixture 路径烤入形态）→ 干跑（AUTO_GALLERY_APPS
  指 worktree 副本）先勘档位，核实**产物零路径烤入**后正式提交（与合并后
  主检出再发射字节等价）。再发射结果：**35 demos/22 loadable/30 VM-live**；
  档位定案 025=registry-only（back 链不可内嵌，同 017/031 族）、
  028/038=loadable；三 demo 适配器 9 件新增（025 六件族+028+038 两件）。
  发射器演化语义随产物落定：017/031/045/046 旧适配器退役（back 链 skip+
  整编后语料缺席）、019 组件名对齐（吸收在册孤儿再生成 WIP：019 改名/
  031 路径归位——确定性产物，判孤儿 WIP 入册）。038 README 加 demo 轨
  反链（025/028 仅 SPEC.md，按 5.3 单向+README 口径）。widgets-gallery
  kitchen-sink/ui-cache 孤儿 WIP（他族 docs_gen 产物）**不入册留主**。
- [x] **T-07** 全量档对账（cargo t + tv/tf vs master 基线）+ 规范增量落册
  （SD-01/SD-02 定稿）。验证：AC-03/AC-05。依赖：T-03。
  [✅ 2026-09-20] 全量对账（worktree rebase 后 vs master@e2ca18360 同命令）：
  **cargo t** worktree 31 红 ⊆ master 33 红（零新增；master 独有 2=app_
  registry scan/curation 预存红，本计划修复转绿）；**cargo tv** 红集两侧
  全等（mouse_area+autodown_panel_heading 预存）；**cargo tf** worktree 3
  vs master 2，差 1=ffi_dual_019_dep_layout_invariants 隔离复跑双绿=
  并行 flake（tf 顺序性红基线家族，非回归）。**执行期事件**：并行 os-035
  会话在我 worktree 基点后落 master（assets/shell.at pin 同步等），全新
  编译暴露 shell_pack parity/vocabulary 假性新增红——rebase 到
  master@e2ca18360 后全绿（基线倾斜非回归，复验 5/5）。SD-01/SD-02 落册
  `docs/specs/auto-lang/ui/overview.md`（worktree `c2fe8cb51`）：双轨
  事实源契约节+PLAN-590 资产注记修订；**SD-02 目地调整**——auto-man 无
  spec 模块目录，发射器契约沿 PLAN-642/658/662 先例并入 ui/overview.md。

## 9. 复审记录

- 2026-09-20 draft handoff（stage: new, revision 1）：背景调查完成
  （PLAN-590 反向操作面勘定、发射链机制核实、端口/策展/编号核查），
  T-01..T-07 立项，outcome: pass（授权范围内可交付 work），next: work。
- 2026-09-20 work handoff（stage: work, revision 1）：
  `pass | PLAN-666 | r1 | worktree D:/autostack/.wt/lang-666/auto-lang
  plan-666-dev@e2ca18360+4（rebase 后：6c11d2911/972e7beb5/e6589907c/
  c2fe8cb51） | T-01..T-07 全勾 | 证据：T-01 diff -r 字节全等+30 tracked
  对账；T-02 R/K 决策清单 25 文件全覆盖；T-03 app_registry 24/24（两处
  master 预存红转绿：scan ≥33→≥35 实测、C 档 17→21 补 047 want）；
  T-04 链接目标全验；T-05 四腿（038vue/028vue 200、025 前端 200、038 VM
  MCP 绿+截图）+定向门零新增红；T-06 auto-os 24aa01f（35 demos/档位
  定案/038 反链）；T-07 t/tv/tf 对账零新增确定性红+SD-01/SD-02 落册
  c2fe8cb51 | blockers: 无 | next: review`。
  **债候选（复审入册 KNOWN-DEBT）**：P666-D1 025 生成后端 rust 编译红
  （29 错，auto-os 原件对称同红——CLI codegen 演化漂移，两侧皆坏）；
  P666-D2 038 vue-tsc 档红（生成 store 类型错，两侧对称预存）；
  P666-D3 主检出磁盘残骸清理（025-dashboard/.am、038 .am/.auto/examples/
  src 遗留——全未跟踪，merge 时清）；P666-D4 auto-os widgets-gallery
  kitchen-sink/ui-cache 孤儿 WIP 留主未处置（他族 docs_gen 产物）。
  **观察**（不入债）：examples/ui README 表 043/044 行 stale（整编迁出后
  行未删，reorg 遗留）；047-bp-admin 无表行（657 遗留）。

- 2026-09-20 review（stage: review, revision 1）：
  `pass | PLAN-666 | r1 | reviewed_commit=c2fe8cb51
  （plan-666-dev@.wt/lang-666/auto-lang） | base=e2ca18360 | dep:
  auto-os@24aa01f（跨仓产物）+auto-down 只读依赖位 6a9df40 | spec_inputs:
  ui/overview.md@e6589907c 前版 | AC: 01 pass / 02 pass / 03 pass /
  04 pass / 05 pass / 06 pass | findings: R1-F1..F7（全非阻塞，见下） |
  evidence: 见下 | next: merge`。
  **独立性声明**：与实施同会话——结论自工件重建（提交链/diff 面/命令
  复跑），不采信执行摘要；关键复跑项如下。
  - **基线**：diff 面=e2ca18360..c2fe8cb51 恰四域（30 新文件+README+
    app_registry.rs+specs+gitignore 34 files/5164+）；plan503 等 app 面
    锚 diff=0（AC-05 结构证明）；worktree 无未提交实现（仅未跟踪 VM
    截图证据件）。
  - **AC-01 复跑**：diff -r 30 文件——29 全等 + 1 计划分歧（038 README
    =T-06 反链 auto-os 侧 +5 行，5.3 授权；快照基准=反链提交前）；tracked
    15/8/7；主检出残骸与入站 tracked 路径**零碰撞**（comm 空）。
  - **AC-02 复跑**：038 vue 腿复现 HTTP 200（复审当次）；025/028 腿与
    038 VM 腿沿用 T-05 记录（计划内数字+MCP envelope/截图在案）。
  - **AC-03 复用理由**：末提交 c2fe8cb51 仅 docs/specs（--stat 1 file），
    t/tv/tf 均跑于代码等价输入——t: 31⊆33 零新增（master 独有 2=本计划
    转绿的预存红）；tv: 红集全等；tf: 差 1=ffi_dual_019 隔离双绿 flake。
  - **AC-04 复验**：git grep @24aa01f registry.at 三 id 命中（durable）。
  - **AC-06 复验**：五史录链接目标 ls 全存在。
  - **健康检查**：rustfmt 基线 30 hunk=worktree 30 hunk（零新增，文件级
    预存分叉）；无遗留 debug 输出；编译零新增告警。
  - findings：R1-F1（注记）AC-01 单文件计划内分歧如上；R1-F2 P666-D1
  025 后端 codegen 两侧对称坏；R1-F3 P666-D2 038 vue-tsc 对称红；
  R1-F4 P666-D3 主检出残骸 merge 时清（已验零碰撞）；R1-F5 P666-D4
  auto-os kitchen-sink 孤儿 WIP 留主；R1-F6 README 043/044 stale+047
  缺行（域外观察）；R1-F7 gitignore 反选目录级——038 screenshots 目录
  新增 png 显示为未跟踪（化妆级）。**规范增量**：SD-01/SD-02 文本与
  实测行为核对一致（档位/退役/路径敏感面均有实证）；SD-02 目地调整
  （auto-man 无 spec 模块→并入 ui/overview.md）合规先例 642/658/662。

- 2026-09-20 merge（stage: merge, `PLAN-666:r1`）：
  - **prepared**：账本三件套 worktree 提交（ui/plans.md 表行 + specs.json
    外科插入 P666-1[designs]/P666-2[reviews]，1 空格缩进原文风格 +27 行、
    610→612；spec-index.py 再生 INDEX.md；JSON 有效性门过）。
  - **landed**：两跳 rebase 全等链（并行 665 会话两次推进 master：
    06f6c2a01→7862da23c）——range-diff 5/5 `=` 全等，映射 6c11d2911→
    2d00d4386 / 972e7beb5→3d408f08a / e6589907c→5cd485ea0 / c2fe8cb51→
    31addf88f / 3c5249607→ca7205a90（再 rebase 后终值），`git merge
    --ff-only` 零 merge commit，master tip==delivery `ca7205a90`；
    specs.json 与 665 会话 P665-1/2 异段异位自动并（614 items，P014-1
    跨节镜像=master 既有非本次引入）；主检出落地冒烟 app_registry
    24/24；P666-D3 残骸清理执行（025-dashboard 整删、038 的 .am/.auto/
    examples/三历史 png 删、tracked 快照保留，examples/ui git status
    全净）。
  - **ledger_refreshed**：`.autoos/specs.json` P666-1/P666-2（file 指
    canonical spec 与本归档件）；`docs/specs/auto-lang/ui/plans.md` 666
    行；`docs/specs/INDEX.md` 再生——均随 delivery 落 master。
  - **archived**：`docs/plans/archive/666-ui-demo-repatriation.md`，
    status=archived，completion_kind=delivered。
  - **cleaned**：wt-guard 复跑 clean（gen pnpm junction 1082 件按 660
    先例 cmd rmdir 整树清——目标全为树内 .pnpm 自指；smoke 残留
    .am/.auto/dist 一并清）；worktree `lang-666/auto-lang` --force 移除
    （未跟踪 VM 截图证据件随树消亡，数值已录档）+ 分支 plan-666-dev
    删除（was ca7205a90）+ 只读依赖位 auto-down worktree 移除 + 组目录
    `D:/autostack/.wt/lang-666` 删除；worktree list/prune/磁盘三查零
    残留。**五 checkpoint 闭环。**

## 10. 待澄清事项

- ~~三 demo 的画廊档位~~（T-06 定案：025=registry-only、028/038=loadable）。
- ~~plan492_m4"第三副本"语义与 desktop_mcp.py 038 测试腿去留~~（T-02 裁定
  在案：第三副本=widgets-gallery 面 K-5 不动；desktop_mcp.py 快照携带零
  门禁接入）。
- ~~auto-os 三 app README 反链~~（T-06：038 README 反链已入 24aa01f；
  025/028 仅 SPEC.md，auto-lang 单向+在案说明口径，AC-06 宽口径满足）。
- 无阻塞待澄清项。
