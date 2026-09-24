---
plan_id: PLAN-700
status: archived               # drafting → executing → execution_done → reviewed → archived（终态）
feature_name: unify-vm-files-tier
author: [zcode]
created_at: 2026-09-24
updated_at: 2026-09-24
plan_revision: 1
current_step: 5
total_steps: 5

# /auto-plan:review 结束时填写：
supersedes_spec_components: []
new_spec_components: []
touched_goals: []             # 引用 docs/specs/goals.md 的 GOAL-NNN

affects: []                   # 无 docs/specs 触面（见 5. 规范增量的零影响说明）
---

# [PLAN-700] unify-vm-files-tier——语料档并入全档 feature 集，消除 tv↔tf 跨档重编

## 变更摘要

`test-vm-files` 退役，语料测试族（vm_file_tests/cookbook_vm_tests/conformance_tests，
~148 测）成为 test 构建的常驻组成：

1. `src/tests.rs` 三处 `#[cfg(feature = "test-vm-files")]` 门拆除 → 语料测试
   随 `--lib` 测试构建恒在；
2. `test-vm-files` feature 退役，`test-aavm` 独立化（imply 关系随常驻化失效，
   aavm loader 依赖的 vm_file_tests 模块恒在）；
3. `cargo tv` 从 feature 档降级为**纯 nextest filter**（语料三族筛选），与
   t/tf 同二进制——三档间切换零重编；
4. `cargo t`（日常档）经 nextest default-filter 排除语料族（镜像 1M churn 的
   现有待遇），**tf 全档自动包含语料**——review 门禁从 tf+tv 两次一次收口。

背景痛点（PLAN-698 复审实测）：t/tf↔tv 跨 feature 档切换=lib test 二进制
整链重编（3–20 分钟/次），一次复审周期多次跨档即损失小时级。

## 目标

- **G1**：t/tf/tv 三档共享同一测试二进制，三向切换零重编（fingerprint 实测）。
- **G2**：tf 全档包含语料族（review 单次覆盖 VM 语义触面），语料测试在 tf 下全绿。
- **G3**：日常 `t` 档墙钟不显著回退（语料经 default-filter 排除，镜像 churn 待遇），
  tv 仍可作为"改 VM 后只跑语料"的定向快捷档。

**非目标**：`test-trans`/`test-book` 的同构统一（待本计划稳定后单独评估——
tt 的转译金样执行成本更高，需独立测量）；aavm 档（taa/ta/t3）语义不动；
1M churn 分层不动。

## 架构方案

```
现状:  t/tf (default) ──切换──> tv (default+test-vm-files) = 重编
目标:  t/tf/tv (default，同一二进制) ──只差 nextest filter = 零重编

src/tests.rs   三处 #[cfg(feature="test-vm-files")] 删除（模块常驻）
Cargo.toml     test-vm-files 退役；test-aavm = []（原 imply 失效无害化）
.cargo/config  tv = nextest filter（语料三族）；t 的 nextest.toml
               default-filter 追加语料三族排除；tf 不动（full 天然含）
CI             vm-files-ci.yml 命令改语料名筛（--features 形态随 feature 退役）
AGENTS.md      档位表 tv 行改口径（feature 档→filter 档；tf 含语料）
```

## 需求分析与背景调查

- **授权记录**：用户 2026-09-24「OK」——对本会话提案（"tv 的 feature 集做成和
  t/tf 一样，tf/tv 切换不重编"）的立项批准。
- **实测依据**（PLAN-698 复审，2026-09-24）：tv 5652 测 vs t 5504 vs tf 5505
  ——差集 ~148 = 语料三族（cfg 门后）；tv 首跑冷编 15–23 分钟；同档重编
  3–5 分钟。1M churn 仅 +2 测（tf 5505 vs t 5504）证明 nextest-config 分层
  机制成熟可镜像。
- **语料本体**：`test/vm/{category}/{NNN_name}/{name}.at`（repo 根相对），
  运行时读盘不入二进制；全族执行 ~20s（AGENTS 2026-09-06 实测）。
- **相关决策先例**：2026-09-22 用户裁定"ui-iced 入 default 全档同源"（消除
  ui-iced 跨档重编）——本计划为同一方向在测试分组 flag 上的收尾。

## 详细设计

### D1 cfg 门拆除（tests.rs）

`vm_file_tests`/`cookbook_vm_tests`/`conformance_tests` 三处
`#[cfg(feature = "test-vm-files")]` 删除，模块随 `#[cfg(test)]` 恒编。
aavm 系（test-aavm 门）不动。

### D2 feature 退役（Cargo.toml）

- `test-vm-files = []` 删除；`test-aavm = ["test-vm-files"]` → `test-aavm = []`
  （imply 的唯一理由是 aavm runner 复用 vm_file_tests 语料缓存 loader——
  常驻化后依赖恒满足）。plan394_future_arch_tests::c1_future_all 等
  非门控测试不受影响。
- 残留引用清零：CI workflow、AGENTS.md、config 注释（T-03）。

### D3 alias 与 nextest 配置

- `tv = "nextest run -p auto-lang --lib -E 'test(vm_file_tests::) or
  test(cookbook_vm_tests::) or test(conformance_tests::)'"`（filter 表达式，
  与 t/tf 同二进制）。
- `.config/nextest.toml` default-filter 追加
  `and not test(vm_file_tests::) and not test(cookbook_vm_tests::)
  and not test(conformance_tests::)`（镜像 churn 行；`::` 后缀锚定模块
  路径避免子串误伤）。
- `.config/nextest-full.toml` 不动 → tf 天然含语料。

### D4 CI 与账面（口径件）

- `vm-files-ci.yml`：`cargo test --features test-vm-files` 形态改语料名筛
  （`cargo test -p auto-lang --lib -- vm_file_tests cookbook_vm_tests
  conformance_tests`），workflow 名与注释同步。
- `AGENTS.md` 档位表：tv 行改"语料筛选档（与 t/tf 同二进制，零重编）"，
  tf 行注"含语料族"；Category B 门禁说明同步（改 VM→tf 已含语料，tv 为
  定向快捷档可选）。
- `.cargo/config.toml` alias 注释（Plan 568 段）更新。

### 规范增量

| delta_id | add/modify/retire | docs/specs/... target | before/after rule | rationale | acceptance IDs |
|---|---|---|---|---|---|
| （无） | — | — | 无 Spec 触面：测试分档契约的权威载体是 AGENTS.md 档位表与 `.cargo/config.toml`/`.config/nextest*.toml`（仓指令与构建配置层），本计划在 T-03 内原地更新；`docs/specs/aavm/project.md:63` 引用的是 `test-aavm` CI 命令行，`test-aavm` 独立化后该行语义不变 | 测试基础设施重组不产生新的模块行为契约 | AC-01..04 |

## 测试设计

- **切换零重编实测（G1）**：同一 worktree 依次 `cargo t` → `cargo tf` →
  `cargo tv` → `cargo t`，第二次起各档输出不得出现 `Compiling auto-lang`
  （fingerprint 稳定判定；本轮全程 CARGO_INCREMENTAL 稳态，非 0 覆盖）。
- **三档计数**：t（语料排除，测试数≈现 5504）、tf（含语料，≈5505+148）、
  tv（≈148，全绿）。
- **tf 总墙钟**：记录含语料后的 tf 执行段增量（预期 +20–60s 量级）。
- **CI 对账**：vm-files-ci.yml 改后语法校验（actionlint 或 dry-run 结构审查）。
- 既有门禁：本计划不改任何运行时代码（tests.rs 仅 cfg 门/模块声明层），
  `cargo t` 日常档全绿 + 语料族在 tf 下全绿即收口口径；不触发 tf 之外的
  重档（无 VM 语义/转译/书改动）。

## 验收标准

- [ ] AC-01 语料族常驻化：`src/tests.rs` 三 cfg 门删除 + Cargo.toml
      `test-vm-files` 退役 + `test-aavm = []`；`cargo check -p auto-lang`
      与 `cargo t plan394` 冒烟绿（非门控测试零涟漪）。
- [ ] AC-02 三档同源：tv 改 filter 形态后与 t/tf 同二进制；实测三向切换
      （t→tf→tv→t）第二次起各档零 `Compiling auto-lang`；t 的
      default-filter 排除语料三族且 tf 包含之（测试数实测：t≈5504 /
      tf≈5650± / tv≈148）。
- [ ] AC-03 口径件同步：vm-files-ci.yml 命令改筛形态；AGENTS.md 档位表
      tv/tf 行与 Category B 说明更新；config 注释更新；全仓
      `grep -rn "test-vm-files"` 残留清零（历史计划文档除外）。
- [ ] AC-04 行为守恒：tf 下语料三族全绿（0 fail）；t 档墙钟不劣化
      （对照本计划前基线，容忍 ±10%）；tv 空跑定位正确（在语料族上）。

## 执行步骤

（原子任务：精确文件路径 + 确切操作 + 验证命令；每步完成后追加 [✅ 已完成] 一行证据）

- [x] T-01 常驻化：`crates/auto-lang/src/tests.rs` 删三处
      `#[cfg(feature = "test-vm-files")]`（vm_file_tests/cookbook_vm_tests/
      conformance_tests）；`crates/auto-lang/Cargo.toml` 删 `test-vm-files
      = []`、`test-aavm = ["test-vm-files"]` → `test-aavm = []`。验证：
      `cargo check -p auto-lang` + `cargo t plan394`。[关联 AC-01]
      [✅ 已完成] 2026-09-24 lang-700：check 绿（349 警告全预存）；plan394
      17/17 绿（二进制 6197 列表含语料族=常驻化实证）；附带清扫
      aavm2_t3.rs 冗余内层门+四文件注释措辞（见复审记录勘定①）。
- [x] T-02 档位重组：`.cargo/config.toml` tv alias 改 filter 形态；
      `.config/nextest.toml` default-filter 追加语料三族排除。验证：
      tv 单跑 ≈148 全绿；t 计数不含语料族。[关联 AC-02]
      [✅ 已完成] 2026-09-24 lang-700：执行期勘定③——计划的 -E 内联
      filterset 形态在 Windows 不可行（cargo 对 alias 实参不做引号剥离，
      单/双引号两形态实测均 filterset 解析失败），等价改走 nextest
      profile：`.config/nextest.toml` 增 `[profile.tv]`（语料三族白名单
      default-filter）+ alias `tv = nextest run --profile tv`，语义与
      计划等价且与 t/tf 的 config-file 模式同构。实测 tv 162/162 绿
      3.95s 零 Compiling；t1.log 语料族零出现。
- [x] T-03 口径件：`vm-files-ci.yml` 命令改筛形态+注释；`AGENTS.md`
      档位表 tv/tf 行+Category B 说明；`.cargo/config.toml` Plan 568 段
      注释。验证：全仓 grep 残留清零（docs/plans 历史档除外）。[关联 AC-03]
      [✅ 已完成] 2026-09-24 lang-700：CI 五步骤摘 feature 旗标（aavm2
      步保 test-aavm）+workflow 名/头注重写（yaml.safe_load 过）；AGENTS
      档位表 tv/tf 行+Category B+别名参考+taa 行（implies 残留）四处更
      新+实测数回填；scripts 三件旗标改 test-aavm；project.md:63 一词
      修正；全仓 grep（rs/toml/yml/md/sh/py）worktree 内残留=0
      （docs/plans 历史档豁免）。
- [x] T-04 实测收口：三向切换零重编实测 + 三档计数 + tf 语料全绿 + tf/t
      墙钟记录。验证命令：见测试设计。[关联 AC-02/AC-04]
      [✅ 已完成] 2026-09-24 lang-700（四段实测，scratch/p700/{t1,tf,tv,t2}.log）：
      t(scoped)→tf→tv→t(scoped) 四段 **零 `Compiling auto-lang`**（G1 同二进制
      零重编成立）；计数 t=5521（scoped，+dep_parity 6≈默认档 5527）/tf=5689/
      tv=162（计划估算 ≈5504/≈5650±/≈148 系 P698 期数字，语料族已增长，
      delta 口径吻合）；tf 语料三族 **162/162 零红**（AC-04 主证），tf 红册
      11 条与 t 红册全等=预存（10 条与 P698 红册同名；plan606 test_029 经
      master 检出单跑复现同红——预存定责，非本计划引入；back_provision 本次
      绿=对方红册中环境 flaky）；tv 162/162 绿 3.95s。墙钟：本 worktree 冷态
      t/tf 被 gallery_pages_compile 围栏支配（~800s/次，三连跑同值——**该围
      栏无跨进程缓存**，每 nextest 进程全量重编画廊页；非本计划触面，观察记
      录在案）；语料执行段实测 3.95s（≈计划的 +20-60s 预期下限以下）。
      dep_parity 首段 >30min 零 CPU 卡死（内嵌腿构建排队锁+疑似管道停滞，
      环境/基建行为，计时段以 -E 排除并记录；其余 5543 测正常流过）。
- [x] T-05 收口：AC 对账 + 复审记录 +（如引债）债册登记。[关联 AC-01..04]
      [✅ 已完成] 2026-09-24：AC-01..04 全对账通过（见复审记录）；无新增债
      入册（gallery 围栏无跨进程缓存为预存观察，留 review 裁量是否立项）。

## 复审记录

- draft 交付（2026-09-24）：stage=new，PLAN-700 rev1。outcome=pass。
  next=work（单仓单 worktree `lang-700`；改动面 5 文件全为测试基建/口径件，
  无运行时代码；T-04 实测为验收主门）。
- work 启动（2026-09-24）：status drafting→executing。worktree
  `D:/autostack/.wt/lang-700/auto-lang`（branch `plan-700-dev`），
  base=master `499b8bdff`；依赖位 auto-down 组内 detached
  `D:/autostack/.wt/lang-700/auto-down`@`3373a5cc6`（主检出同 commit，零改动）。
  执行期勘定①：计划正文的 5 文件面之外，AC-03 grep 清零要求扩展清扫——
  `aavm2_t3.rs:16` 冗余内层 cfg 门（模组门 test-aavm 已等价覆盖，删）、
  `aavm_runner_tests.rs`/`heavy_gate.rs`/`plan024_named_view_tests.rs`/
  `aavm2_a2r.rs` 注释措辞、scripts 三件（aavm4_check.py/
  measure_test_mem.py/aavm_native_gen_check.sh 的 feature 旗标改 test-aavm）、
  `docs/specs/aavm/project.md:63` 命令引用一词修正（test-vm-files→test-aavm；
  计划正文称该行"引用 test-aavm 语义不变"与实况有出入——实为仍挂旧 feature
  名的命令字面量，语义按 Plan 568 意图不变，不构成规则变更、无 specs.json
  条目）；语义均等价或措辞级，不扩权。勘定②：语料模块仅常驻于测试构建
  （`#[cfg(test)]` 面），非测试编译零涟漪（cargo check 实证）。
- work 收口（2026-09-24）：stage=work | plan_id=PLAN-700 | plan_revision=1 |
  outcome=**pass** | code_commit=worktree `plan-700-dev`@`403b9ea4a`
  （15 文件 +82/−66）| task_ids=T-01..T-05 | evidence=scratch/p700/
  {t1,tf,tv,t2}.log 四段零 Compiling + tv 162/162 绿 3.95s + tf 语料零红 +
  grep 残留清零 + yaml.safe_load 过 + plan394 17/17 | blockers=无 | next=review。
  AC 对账：AC-01 ✓（三门+feature 退役+check/plan394 绿）；AC-02 ✓（四段零
  重编；t 排除/tf 包含/tv=162 实测）；AC-03 ✓（CI/AGENTS/config/scripts/
  project.md 同步，残留清零）；AC-04 ✓（tf 语料 162/162；红册 11 全预存
  ——test_029 master 复现定责；tv 空跑定位正确）。勘定③（T-02 实施形态）：
  -E 内联 filterset 于 Windows alias 不可行（引号字面传递，两形态实测皆败），
  等价改 `[profile.tv]` + `--profile tv`（语义不变，与 t/tf config-file 模式
  同构）；勘定④（观察，非本计划触面）：gallery_pages_compile 围栏每
  nextest 进程全量重编画廊页 ~800s、无跨进程缓存（三连跑同值），且
  dep_parity 首段出现过 >30min 零 CPU 停滞（环境基建）——两项留 review
  裁量是否立项清偿。
- 复审（2026-09-24）：stage=review | plan_id=PLAN-700 | plan_revision=1 |
  reviewed_commit=`403b9ea4a`+R-1 修复 `c615d696a` | base_commit=`499b8bdff` |
  dependency_revisions=auto-down 组内 detached `3373a5cc6`（==属主 master，零
  改动）| spec_inputs=docs/specs/aavm/project.md（一词修正，见下）|
  **独立性声明**：与执行同会话，结论全部从工件重建（直接读 diff + 在被审
  提交上重跑验证），未采信执行摘要。
  **outcome 第一轮=needs_fix（R-1）→ 修复 → 第二轮=pass。**
  - R-1（minor，已修 `c615d696a`）：常驻化使 `cookbook_vm_tests.rs:11` 死常量
    `COOKBOOK_DIR` 首次暴露于 t/tf 日常档警告面（`constant is never used`；
    test_cookbook 直用字面量，base 期即死仅不可见）——违反零新增警告健康线。
    修复=删一行；复验 `cargo check --tests` 零 COOKBOOK_DIR 警告 + tv 162/162 绿。
  - 复审重跑证据（被审提交上独立复现）：cargo check 绿；plan394 17/17 绿
    （首跑失败系与后台 tf 的 target 锁并发互扰，单跑即绿）；`cargo nextest
    list`（默认档）5512 测语料族零出现；tv 162/162 绿 2.4-4.3s（两轮）；
    fresh tf 全量 5689 测零 Compiling、语料三族 162/162 **零红**、红册 12 条
    =已知 11 预存 + `ffi_dual_019`（Plan 619/689 在册环境 flaky 轮换现身，
    零改动非引入）；残留清零（docs/plans 豁免类除外）；CI yaml safe_load 过。
  - 健康检查勘误：fmt 分叉（5 文件）经 base 对照（git show base|rustfmt
    --check）确证 base 期即存在，非本计划新增，不构成发现。
  - AC 对账：AC-01 ✓ / AC-02 ✓ / AC-03 ✓ / AC-04 ✓（证据同上；tf 下语料零
    红为本轮 fresh 复现，非仅沿用执行段日志）。
  - 规范增量附注：T-03 实际触及 `docs/specs/aavm/project.md:62-63` 命令引用
    字面量一词修正（旧语料 feature 名→test-aavm）——属 Plan 568 意图内的事
    实同步、非规则变更；`supersedes/new_spec_components` 维持空，无 specs.json
    条目。
  - 观察项（非阻塞，出计划范围，留裁量）：①gallery_pages_compile 围栏每
    nextest 进程全量重编画廊页 ~800s、无跨进程缓存（四连跑同值）；②
    dep_parity 一次 >30min 零 CPU 停滞；③ffi_dual_019 环境 flaky 在册。
  evidence 摘要（工作树 scratch 将随 merge 清理，此处留可稽核摘录）：
  tf-review.log 语料 PASS 计 162、corpus FAIL 0、Compiling 0；tv 两轮
  162/162（2.419s/4.316s）；plan394 17 passed。| next=**merge**。
  status: execution_done → reviewed。
- merge 收据（2026-09-24）：**PLAN-700:r1 completion_kind=delivered**，
  五 checkpoint 全闭环：
  ① `prepared`——reviewed 基线 `403b9ea4a`+R-1 `c615d696a`（plan-700-dev），
    base `499b8bdff`；零 spec 触面（复审确认，规范增量附注在案）；依赖位
    auto-down 组内 detached `3373a5cc6` 零改动。
  ② `landed`——rebase 2 commits 到 master `0d95cbe61`，range-diff **2/2
    全等**（`403b9ea4a→b9f747e45` / `c615d696a→af95350af`），ff-only 落地
    **delivery_commit=`af95350af`**（master==分支，无 merge commit）；
    master smoke `cargo t plan394` 17/17 绿 exit 0（scratch/p700-master-smoke.log）。
  ③ `ledger_refreshed`——`.autoos/specs.json` P700-1（reviews 复审合并收据）
    外科 append-only（+12 行纯插入，json.loads 验证过）+ `docs/specs/
    auto-lang/vm/plans.md` 700 行（域归属=vm，564/568 测试档先例同域）+
    `python scripts/spec-index.py` 再生（输出零变化，与 698 先例一致）；
    提交 `8416033fa`。
  ④ `archived`——`git mv` 至 `docs/plans/archive/700-unify-vm-files-tier.md`
    + frontmatter `status: archived`（终态）。
  ⑤ `cleaned`——wt-guard 双 worktree clean（auto-lang/auto-down 均"无任何
    reparse point"）→ `git worktree remove` lang-700/auto-lang +
    `git branch -d plan-700-dev`（was af95350af）+ auto-down 属主仓侧移除
    lang-700/auto-down 依赖位 → 组目录 rmdir；`git worktree list` 零 lang-700
    残留。全组清空。

## 待澄清事项

1. 日常 `t` 是否长期排除语料族——本计划取"排除"（镜像 churn，保 daily
   墙钟）；备选"纳入"（daily +~30s 换每日 VM 行为覆盖）如需翻转属一行
   filter 改动，执行期呈报即可不阻塞。
