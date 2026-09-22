---
plan_id: PLAN-689
status: reviewed               # drafting → executing → execution_done → reviewed → archived
feature_name: bp-admin-standalone-example
author: []
created_at: 2026-09-22
updated_at: 2026-09-22

# /auto-plan:review 结束时填写：
supersedes_spec_components: []
new_spec_components: []
touched_goals: [GOAL-010, GOAL-011]  # 引用 docs/specs/goals.md 的 GOAL-NNN

affects: [blueprint]            # 受影响的 specs 路径，如 [auto-lang/vm]
current_step: 6
total_steps: 6
---

# [PLAN-689] bp-admin-standalone-example

## 变更摘要

`examples/ui/047-bp-admin`（PLAN-657 L1 组装样板）迁出画廊轨道，升格为
`examples/bp-admin` 独立示例（与 `examples/bp-gate`、`examples/bps-gallery`
同族平铺命名，无编号前缀）；仓内引用面全量收口（测试钉点 / CI 矩阵 /
specs 与债册指针），auto-os 侧画廊再发射按 PLAN-666 先例作为 merge 后
伴随动作。

## 目标

- `git mv examples/ui/047-bp-admin examples/bp-admin`（历史随行）。
- `pac.at` 的 `dep bps` 相对路径按新层级修正（`../../../blueprints` →
  `../../blueprints`）。
- 仓内两处计数/策展断言重基线（scan ≥35→≥34；C 档 want 21→20——
  022-kanban 升格退策展同型）+ `plan657_bp_admin_tests.rs` 路径钉更新。
- CI gen-only 覆盖保留在新路径（build-ui-examples.yml 矩阵改 path 形态
  + paths 过滤扩 `examples/bp-admin/**`）。
- specs（blueprint/project.md L1 样板位置句、goals.md 657 注记）与
  KNOWN-DEBT P657-D1 证据指针同步新路径。

## 非目标

- **不动 047 语料本身**（app.at / api.at / db.at / tests/ 原样随迁，
  含 P657-D1 证据 screenshots）。
- **不做 ui 轨序号连续化**——047 是轨道尾号（当前最大占位），迁出仅
  缩短轨道不产生中段空洞；第二轮序号整治（14 空洞 + 012×2/031×2 重复）
  策略仍待用户裁定，本计划不触碰。
- **不注册 auto-os 桌面入口**——迁出后 examples/ui 策展集（C 档）自然
  退策展（21→20，与 022-kanban 升格独立仓同型）；若日后仍要桌面入口，
  走 auto-os `apps.manifest` extra root（auto-kanban 先例），另案裁定。
- **auto-os 画廊再发射不在本仓 worktree 内执行**——merge 后由 merge
  会话在 auto-os 侧 regen 提交（见 §跨仓伴随动作）。

## 需求分析与背景调查

（2026-09-22 主检出全量勘定；grep 面 `047-bp-admin|bp-admin` 覆盖
rs/at/ts/vue/md/json/toml/yml/py/mjs。）

- **demo 自身**：`pac.at`（name: bp-admin, scene: ui, render: vue,
  api: rust, desktop: "true"，`dep bps { path: "../../../blueprints" }`）
  + `src/back/{api,db}.at` + `src/front/app.at` +
  `tests/walk_vm_657.py`（自包含：skill 用仓根绝对路径、输出
  `__file__` 相对——随迁零改动）。
- **代码钉点（必须改）**：
  - `crates/auto-lang/src/ui/app_registry.rs`
    `scan_examples_ui_curation_set`：want 列表含 `"047-bp-admin"`
    （C 档恰等断言，21 项）；
    同文件 scan 计数断言 `apps.len() >= 35`（PLAN-666 重基线值）。
  - `crates/auto-lang/src/plan657_bp_admin_tests.rs:25`
    `example_047()` 钉 `examples/ui/047-bp-admin`。
  - `.github/workflows/build-ui-examples.yml`：matrix 项
    `working-directory: examples/ui/${{ matrix.example }}` 硬编码前缀
    （PLAN-657 T-04 刚清过四条同型死目录项——本迁移若不改矩阵会复刻
    该型红）；`on.paths` 不含 `examples/bp-admin/**`（改动不触发）。
- **注释/文档指针（顺手收口）**：
  - `docs/specs/blueprint/project.md:136`：L1 组装样板位置句
    "`examples/ui/047-bp-admin` 是 L1 主通道的首次**多包**直连"。
  - `docs/specs/goals.md` GOAL-010/GOAL-011 行的 657 括注（入矩阵史实
    保留，补迁出注）。
  - `docs/plans/KNOWN-DEBT-AND-RISKS.md` P657-D1 证据路径
    `examples/ui/047-bp-admin/tests/screenshots/vm_walkthrough_657.json`；
    P657-D2（013/046/047 vue-tsc 双预存红）与 P657-D3（db.at 注记）
    在新位置语义不变，无需改。
  - `blueprints/navigation/sidebar-shell/reference/default.at` 头注
    "047-bp-admin 首次多包 L1 直连消费"（活跃位置指针，补注新路径）。
  - `crates/auto-man/src/vue.rs:5146` / `lib.rs:7868`：历史注记
    （实测实录/计划锚），不改。
- **auto-os 侧（跨仓，4 命中）**：`ui-gallery/src/front/registry.at`、
  `ui-gallery/src/gallery/AppViewport.vm.at`、
  `ui-gallery/src/gallery/demos/047-bp-admin.at`（均为画廊生成物，
  regen 自愈）+ 一份归档 plan（历史，不动）。
- **无耦合确认**：bps-gallery `registry.at` 的 "047" 命中实为
  `047-bp-compose`（capability-tests，另一物）；aavm/vm 语料 047 为
  无关编号；`plan633_fullstack_embed_tests` 无 047 引用。
- **先例**：022-kanban 升格（C 档退策展 17→16，桌面入口走
  apps.manifest extra root）；PLAN-666 画廊再发射（auto-os `24aa01f`，
  35 demos/22 loadable/30 VM-live 基线）。
- **画廊后端影响**：047 是 back-proxy 七会话之一（2026-09-22 实测），
  迁出后会话 7→6，纯动态扫描零代码改动。

## 详细设计

1. **物理迁移**：`git mv examples/ui/047-bp-admin examples/bp-admin`。
2. **pac.at**：`path: "../../../blueprints"` → `path: "../../blueprints"`
   （新层级 examples/bp-admin → 仓根 blueprints）。
3. **app_registry.rs 双断言**：
   - want 列表摘 `"047-bp-admin"` 行，注释补
     "PLAN-689：047-bp-admin 升格独立示例 examples/bp-admin 迁出
     examples/ui（022-kanban 同型），C 档 21→20"；
   - `>= 35` → `>= 34`，注释补同因（PLAN-666 基线 35 的后继）。
4. **plan657_bp_admin_tests.rs**：`example_047()` 改
   `repo_root().join("examples/bp-admin")`，函数头注补迁出注。
5. **build-ui-examples.yml**：matrix 五项改为仓根相对全路径
   （`examples/ui/001-helloworld` … + `examples/bp-admin`），
   `working-directory: ${{ matrix.example }}`；`on.push.paths` 与
   `on.pull_request.paths` 增 `'examples/bp-admin/**'`。047 项注释保留
   PLAN-657 原意（bps dep-scan + back #[api] 生成 CI 覆盖）并补迁出注。
6. **文档/specs/债册**：§需求分析所列四指针改新路径（goals.md 括注
   形态：`657（047-bp-admin 蓝图组装样板——PLAN-689 迁出为
   examples/bp-admin 独立示例）`）。

## 测试设计

- `cargo t app_registry`——两断言新基线（34/20）转绿，无邻红。
- `cargo t plan657`——回归锚在新路径全绿（四包直连解释器断言不因
  迁移漂移）。
- `cd examples/bp-admin && auto build --gen-only`——`dep bps` 相对
  路径解析 + back #[api] 生成在新位置成立（CI 同型验证本地预跑）。
- 残留扫描：`grep -rn "examples/ui/047" --exclude-dir=target
  --exclude-dir=gen` 仓内零命中（归档 plans 历史陈述除外——归档
  不回写）。
- CI：workflow YAML 语法目测 + matrix 路径与 working-directory 一致性；
  首跑绿归 merge 后 watch（PLAN-676 AC-06 先例）。

## 验收标准

- **AC-01**：`git log --follow examples/bp-admin/pac.at` 跨越迁移
  commit 历史可溯（git mv 非 delete+add）。
- **AC-02**：`cargo t app_registry` 全绿——scan ≥34 与 C 档恰等 20。
- **AC-03**：`cargo t plan657` 全绿（新路径）。
- **AC-04**：`examples/bp-admin` 下 `auto build --gen-only` 成功。
- **AC-05**：仓内非归档引用清零（§残留扫描判据）。
- **AC-06**：build-ui-examples.yml matrix 含 `examples/bp-admin` 且
  paths 过滤覆盖其变更（CI 首跑绿为 merge 后 watch 项，非本计划门禁）。

## 执行步骤

（原子任务：精确文件路径 + 确切操作 + 验证命令；每步完成后追加
[✅ 已完成] 一行证据）

- **T-01 物理迁移 + pac.at 路径**
  `git mv examples/ui/047-bp-admin examples/bp-admin`；编辑
  `examples/bp-admin/pac.at` dep bps path → `../../blueprints`。
  验证：`cd examples/bp-admin && auto build --gen-only`（AC-04）。
  [✅ 已完成] worktree 1d829b935：git mv 五文件 rename 全识别（pac.at 63%
  相似度因路径编辑、其余四文件 100%）；gen-only 绿——29 component(s)、
  TypeScript api client + Rust server 生成、`dep bps` 新相对路径解析成立
  （auto.exe 0.4.2-1893 主检出当日构建）。
- **T-02 app_registry 双断言重基线**
  `crates/auto-lang/src/ui/app_registry.rs` want 摘项（21→20）+
  `>= 35` → `>= 34`，各补 PLAN-689 注。
  验证：`cargo t app_registry`（AC-02）。
  [✅ 已完成] 25/25 passed（含 scan_examples_ui_curation_set 恰等 20 +
  scan_examples_ui_finds_at_least_27_apps ≥34），零邻红。
- **T-03 plan657 回归锚路径**
  `crates/auto-lang/src/plan657_bp_admin_tests.rs` `example_047()`
  → `examples/bp-admin`。
  验证：`cargo t plan657`（AC-03）。
  [✅ 已完成] 3/3 passed（t01 VM 轨全量组装/t02 vue 轨 import+零副本/
  t03 语料面五 reference+palette 零漂移）——四包直连解释器断言不因迁移漂移。
- **T-04 CI 矩阵 path 形态化**
  `.github/workflows/build-ui-examples.yml` matrix 全路径化 +
  working-directory 参数化 + paths 增 `examples/bp-admin/**`。
  验证：YAML 目测 + AC-06。
  [✅ 已完成] YAML python.safe_load 解析通过；matrix 五项=仓根全路径
  （含 examples/bp-admin）、working-directory=${{ matrix.example }}、
  push/PR paths 双含 examples/bp-admin/**。**实施期适配**：exe 调用从
  `../../../target/debug/auto`（固定三层回退）改 `$GITHUB_WORKSPACE/
  target/debug/auto` 绝对形态——matrix 项现跨两种深度（examples/ui/X
  两层 / examples/bp-admin 一层），固定 ../ 数无法同时满足（设计条 5
  "path 形态化"的必要推论，授权范围内）。
- **T-05 specs/债册/头注指针**
  blueprint/project.md:136、goals.md GOAL-010/011 括注、
  KNOWN-DEBT P657-D1 证据路径、blueprints/navigation/sidebar-shell/
  reference/default.at 头注。
  验证：§残留扫描（AC-05）。
  [✅ 已完成] 四指针全落新路径（goals.md 双括注保留 657 入矩阵史实+
  补 PLAN-689 迁出注；迁出注按规范只引新路径不引旧路径字面量）；
  残留扫描 `grep -rn "examples/ui/047"` 非归档零命中（归档 plans 与
  本计划自述文件除外）。
- **T-06 门禁收口**
  `cargo t app_registry && cargo t plan657` 复跑 + `cargo check -p
  auto-lang`；AC-01 follow 检查。
  [✅ 已完成] app_registry 25/25 + plan657 3/3 复跑绿（如上）；
  cargo check -p auto-lang Finished 无 error（lib 344 警告=master
  基线存量，两改动文件零命中）；AC-01 `git log --follow
  examples/bp-admin/pac.at` 跨迁移 commit 1d829b935 直溯 PLAN-657
  创建提交 3e190da3a；worktree 终态 clean。

## 跨仓伴随动作（merge 后，不在本 worktree）

- auto-os `ui-gallery` 再发射：`cd auto-os/ui-gallery && auto run`
  regen（35→34 demos、6 sessions、demos/047-bp-admin.at 摘除）后
  跨仓提交——PLAN-666 "画廊再发射 auto-os 24aa01f" 同型。

## 复审记录

- **work 交付记录（2026-09-22）**
  `stage: work | PLAN-689 | r1 | pass | 1d829b935 (plan-689-dev, base
  97614062f) | T-01..T-06 | 六任务全绿：gen-only 29 组件（AC-04）/
  app_registry 25/25 恰等 20+≥34（AC-02）/plan657 3/3（AC-03）/YAML
  解析+矩阵静态断言（AC-06）/残留扫描非归档清零（AC-05）/git log
  --follow 跨迁移直溯 657 创建提交（AC-01）；cargo check 无 error、
  改动文件零警告 | 无阻塞 | 下一步 /auto-plan:review`
  - worktree：`D:/autostack/.wt/lang-689/auto-lang`（plan-689-dev）；
    依赖位：`.wt/lang-689/auto-down` detached @ fba6563（只读，未改动）。
  - **环境事故留档**：auto-down 依赖位首次 worktree add 后 909 文件
    null 字节损坏（含 .git 指针；尺寸正确内容零填充，疑似 NTFS 延迟
    写失败，盘空间充足 218G free 排除盘满）——prune+rmdir 重建后
    `git update-index --really-refresh` 全绿；auto-lang 实现树同法
    校验完好（17 扫描命中均为被跟踪的合法二进制内容）。后续建
    worktree 后建议例行 really-refresh 抽验。
  - CI 首跑绿=merge 后 watch 项（AC-06 附注）；auto-os 画廊再发射
    （35→34）=merge 后伴随动作（§跨仓伴随动作）。

- **review 复审记录（2026-09-22，实施会话内复审——独立性声明：裁定
  自工件重构，六 AC 全部重新复现，不采信 work 摘要）**
  `stage: review | PLAN-689 | r1 | pass | 1d829b9356c3f30447ec709902
  aa8a56df28894c | 97614062f | auto-down detached @ fba6563（只读未改）
  | spec 输入=blueprint/project.md + goals.md（GOAL-010/011）基线版 |
  AC-01..06 全 pass（复现命令与结果如下）| 发现 F-689-1（非阻塞）|
  证据见下 | next=merge`
  - **AC 复现**：AC-01 `git log --follow examples/bp-admin/pac.at` 2
    commit 跨迁移直溯 3e190da3a ✓；AC-02 `cargo t app_registry`
    25/25（含 C 档恰等 20）✓；AC-03 `cargo t plan657` 3/3 ✓；AC-04
    `auto build --gen-only` 29 组件+api client+Rust server ✓；AC-05
    残留扫描非归档 0 命中 ✓；AC-06 YAML safe_load+静态断言（matrix
    全路径含 examples/bp-admin/paths 双含/wd 参数化/exe 绝对形态）
    ✓。worktree 终态 clean，无未提交实现。
  - **全量门禁**：`cargo tf`（nextest-full --no-fail-fast）两轮：
    5450 run，复跑 5442 pass / 8 fail / 112 skip。**8 红零回归全归因**：
    musk×6 + projector_counter×1 = 记忆在案预存（.at 双源漂移，
    修归 auto-os sync 脚本待认领）；a2vue_desktop_surface_asset×1 =
    基线（主检出 97614062f 同代码）隔离复跑同红——PLAN-682 转告的
    e85143621 pin 同步后金样未同步预存红。首轮另见
    ffi_dual_019_dep_layout_invariants 1 红：隔离绿+复跑全量绿+输入
    面（test/ffi_dual fixture×nightly methods_pack）与本 diff 零交集
    ——裁定全量并行下 fixture 现场编译争用型 flaky，非本计划回归。
  - **diff 审查**：12 文件逐 hunk 过——代码面（app_registry 双断言/
    plan657 锚/CI 矩阵+exe 路径/pac.at path）与设计条 1-5 逐条对应；
    $GITHUB_WORKSPACE 适配=设计条 5 的必要推论（矩阵跨深度），授权
    范围内；docs 面（project.md/goals.md/KNOWN-DEBT/头注）与设计条
    6 对应，迁出注统一只引新路径。
  - **F-689-1（非阻塞）**：`blueprints/navigation/sidebar-shell/
    reference/default.at:7` 迁出注使单注释行超宽（~100 字符，原文件
    惯例 ~76 折行）。不影响解析/生成/任何 AC（plan657 t03 绿）；留
    后续顺手修，不构成 needs_fix。
  - **spec 增量裁定**：本计划无新增/废弃规范组件——canonical spec
    改动为指针性更新两处（blueprint/project.md L1 样板位置句、
    goals.md GOAL-010/011 括注），已随 delivery 1d829b935 提交；
    frontmatter 维持 supersedes=[]/new=[]，touched_goals
    [GOAL-010, GOAL-011] 为完整影响面。

## 待澄清事项

（无阻塞项。命名取 `examples/bp-admin`——与 bp-gate/bps-gallery
同族平铺、无编号前缀；如需其他形态请在批准时指出。）
