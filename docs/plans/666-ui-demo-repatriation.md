---
plan_id: PLAN-666
status: drafting               # drafting → executing → execution_done → reviewed → archived
feature_name: ui-demo-repatriation
author: [agent]
created_at: 2026-09-20
updated_at: 2026-09-20

# /auto-plan:review 结束时填写：
supersedes_spec_components: []
new_spec_components: []
touched_goals: [GOAL-010]

affects: [ui/overview.md, auto-man/overview.md]
current_step: 0
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

- **T-01** 拷贝与清损（5.1 清单；`git -C D:/autostack/auto-os ls-files`
  取源清单 → 复制 → `git add`；清 025 空壳/038 残骸；diff -r 对账）。
  验证：AC-01。依赖：无。
- **T-02** 锚点普查（bounded investigation）：`git show cf103b70d
  5525ccd16` 全量锚点逐项标注 demo 面/app 面/不动，产出决策清单落本节
  下方（决策件；492_m4 第三副本语义、desktop_mcp.py 038 腿在此裁定）。
  验证：清单覆盖两提交全部锚点文件。依赖：T-01。
- **T-03** 锚点回接（按 T-02 清单执行 5.2；scan/C 档计数重基线；docs_gen
  /golden/plan 语料回读 examples/ui）。验证：AC-03 定向集。依赖：T-02。
- **T-04** README 恢复与互链（5.3）。验证：AC-06。依赖：T-01。
- **T-05** 三 demo 冒烟四腿 + 定向门（6 节）。验证：AC-02。依赖：T-01。
- **T-06** auto-os 侧 ui-gallery 再发射 + 产物提交（跨仓单 commit，消息
  `gallery: PLAN-666 产物刷新——025/028/038 回源条目恢复`+3 app README
  反链若做则并入）。验证：AC-04。依赖：T-01/T-03。
- **T-07** 全量档对账（cargo t + tv/tf vs master 基线）+ 规范增量落册
  （SD-01/SD-02 定稿）。验证：AC-03/AC-05。依赖：T-03。

## 9. 复审记录

- 2026-09-20 draft handoff（stage: new, revision 1）：背景调查完成
  （PLAN-590 反向操作面勘定、发射链机制核实、端口/策展/编号核查），
  T-01..T-07 立项，outcome: pass（授权范围内可交付 work），next: work。

## 10. 待澄清事项

- 三 demo 的画廊档位（loadable/fullstack/route_stub）以 T-06 再发射实测
  为准，不阻塞起草（AC-04 已按"档位记录在案"宽口径）。
- plan492_m4"第三副本"语义与 desktop_mcp.py 038 测试腿去留：T-02 普查
  裁定，产出记录进 T-02 决策清单。
- auto-os 三 app README 反链（5.3 可选项）：T-06 时视 README 存在性定，
  缺席则 pac 注记或仅 auto-lang 单向链接（AC-06 按"双向可解析或单向+
  在案说明"验收）。
