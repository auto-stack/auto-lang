---
plan_id: PLAN-466
status: reviewed
feature_name: 测试与构建提速三方案落地（sccache 启用 / cargo t ≤30s / 全量测试门禁收敛到 review）
author: [zcode]
created_at: 2026-08-28T11:42:29+08:00
updated_at: 2026-08-28T13:05:00+08:00

supersedes_spec_components: []
new_spec_components: []
touched_goals: []

current_step: 11
total_steps: 11
---

# [PLAN-466] 测试与构建提速三方案落地

## 变更摘要

AI 多 worktree 并行开发下，每个 worktree 冷编译 + 全量测试已成为主要时间/磁盘瓶颈。本计划落地三个互补方案：

1. **方案A（提速测试）**：把 `cargo t` 从实测 40.7s 压到 ≤30s——瓶颈是单个 12s 慢测试 `str_churn_bounded_large`（1M 次迭代）+ 约 11.6s 的零改动构建检查开销。通过"慢测试参数化降档 + nextest 双 profile"实现日常快、门禁全。
2. **方案B（提速编译）**：启用已建成但从未开启的 sccache 设施（wrapper 脚本已存在于 `scripts/`，`RUSTC_WRAPPER` 从未设置，缓存命中数为 0），使新 worktree 冷构建的外部依赖大头变为缓存命中。
3. **方案C（降低全量频率）**：修改 auto-plan 技能与 AGENTS.md，把每个 plan 生命周期的全量测试从"work 收尾一次 + review 一次"收敛为"review 唯一全量门禁"（多阶段 plan 的阶段 fold 前另跑一次）。

## 目标

- G1: 零改动、热 target 下 `cargo t` 全流程 ≤30s（基线 40.7s），且不降低合入门禁的覆盖强度。
- G2: sccache 启用后，新建 worktree 的冷构建中外部 registry 依赖编译以缓存命中为主（命中率 ≥50% 编译请求）。
- G3: auto-plan-work / auto-plan-review 技能明文规定：执行期只跑 scoped 检查，全量 `cargo tf` 仅出现在 review（及阶段 fold 前），失败打回机制沿用既有路由。
- G4: dev 构建产物瘦身（`debug = "line-tables-only"`），缓解每 worktree ~26G 的 target 膨胀。

## 架构方案

三个方案的"旋钮"分布在不同层面，互不耦合、可独立回退：

| 层面 | 旋钮 | 归属 |
|:---|:---|:---|
| 测试运行时 | `RC_CHURN_ITERS` 环境变量参数化慢测试；nextest `default`(100k)/`full`(1M) 双 profile；别名 `t`(default)、新增 `tf`(full)、`ta` 改挂 full | 仓库 git 内 |
| 编译缓存 | `RUSTC_WRAPPER` → `scripts/sccache-wrap.cmd`（已有脚本）；`SCCACHE_DIR`/`SCCACHE_CACHE_SIZE` | 用户环境变量（setx，非 git） |
| 构建配置 | `[profile.dev] debug = "line-tables-only"`；`jobs` 8→12 | 仓库 git 内 |
| 流程门禁 | auto-plan-work Step 2/6、auto-plan-review Step 2、AGENTS.md 别名表与 Category B | 技能文件（用户目录，非 git）+ AGENTS.md |

关键设计决策：

- **测试默认值保持 1M**（`std::env::var` 缺省 1_000_000）：`cargo test`（非 nextest）与任何未知调用方仍跑全量语义；降档只发生在 nextest `default` profile 显式设 env 时。`force = true` 防止 shell 环境残留值干扰。
- **断言阈值不随 N 缩放**：`live_pool < 1000` 的有界性断言在 100k 迭代下依然有效（泄漏率 ≥1% 即可捕获）；1M 全量档在 review 捕获更细微的泄漏。
- **`ta`（全特性套件）挂 `full` profile**：它是综合门禁，应保持全量语义；`tv/tt/tb/th` 保持 default（快速迭代用）。
- **执行顺序上 sccache（Phase B）先于 dev profile 改动（Phase C）**：后者会改编译旗标触发一次性全量重建，放在 sccache 启用后这次重建本身大部分可走缓存。
- **技能文件在 `C:\Users\zhaop\.zcode\skills\`，不在本仓库 git 内**——Phase D 直接原地修改（无法走 worktree），改前各备份一份到本仓库 `docs/plans/466-skill-backup/` 留档。

## 技术栈

Rust cargo workspace / cargo-nextest 0.9.138（profile env 配置）/ sccache 0.16.0 / git worktree / SKILL.md 技能文件。

## 需求分析与背景调查

- spec ledger 总览 API 返回 `{"error":"failed to build overview"}`，且无 `.autoos/specs.json` 兜底；本计划为构建/流程基础设施，不触碰 `specs/modules/*`。
- **实测基线**（主 checkout，20 逻辑核 / 31.8G RAM，2026-08-28）：
  - `cargo t` 全程 40.7s = 零改动构建检查 ~11.6s（`cargo build -p auto-lang --lib --tests` 实测）+ nextest 测试运行 19.7s（3222 tests）。
  - 慢测试排行：`vm::tests_rc_lifecycle::str_churn_bounded_large` **12.1s**（`crates/auto-lang/src/vm/tests_rc_lifecycle.rs:625`，1M 次字符串拼接，debug 模式 VM）、`aavm2_m1_lexer_corpus` 4.1s、`churn_returns_to_baseline` 2.4s、`test_library_widgets_list_is_self_consistent` 2.2s，其余均 <1.7s。运行段关键路径即 12.1s 那一个。
  - sccache：`scripts/sccache-wrap.{cmd,sh}` 与 `scripts/README-sccache.md` 已存在；sccache 0.16.0 已安装；但 `RUSTC_WRAPPER`（用户级）为空，`sccache --show-stats` 显示 0 次编译请求——设施从未启用。
  - 链接器已是 `rust-lld.exe`（无需再动）；`.cargo/config.toml` 中 `jobs = 8` 未用满 20 核。
  - 根 `Cargo.toml` 无 `[profile]` 段（dev = 全量 debuginfo）。
  - 磁盘：主 target 54G；`.worktree` 26G 几乎全为单个已构建 worktree（plan-455）的 target；C 盘剩 58G（sccache 默认缓存位），D 盘剩 174G。
- **流程现状**：AGENTS.md Category A/B/C 已规定"开发期 scoped、合入前全量"；活跃 plan 的分步验证均为局部命令。全量重复来自技能两处：auto-plan-work Step 6"收尾把整个验收套件再跑一遍" + auto-plan-review "Verify, don't trust, 全部重跑"。另外 auto-plan-work Step 2 允许阶段 fold 先于 review 发生，当前无全量门禁保护。

## 详细设计

### A. 慢测试参数化 + nextest 双 profile

`crates/auto-lang/src/vm/tests_rc_lifecycle.rs` 的 `str_churn_bounded_large`：

```rust
let iters: u64 = std::env::var("RC_CHURN_ITERS")
    .ok()
    .and_then(|s| s.parse().ok())
    .unwrap_or(1_000_000);
// 循环上界 code 中 0..1000000 改为由 iters 格式化生成（format! 拼 code 字符串）
// eprintln 与 assert 消息显示实际 iters
```

新建 `.config/nextest.toml`：

```toml
[profile.default.env]
RC_CHURN_ITERS = { value = "100000", force = true }

[profile.full.env]
RC_CHURN_ITERS = { value = "1000000", force = true }
```

`.cargo/config.toml` 别名：`t`/`tv`/`tt`/`tb`/`th` 不变（default profile），新增 `tf`，`ta` 改挂 `full`。

### B. sccache 启用（用户环境变量 + README 更新）

```
setx RUSTC_WRAPPER "D:\autostack\auto-lang\scripts\sccache-wrap.cmd"
setx SCCACHE_DIR "D:\autostack\.sccache"       # 避开仅剩 58G 的 C 盘
setx SCCACHE_CACHE_SIZE "30G"                   # 默认 10G 偏小
```

注意 setx 只对新进程生效：本计划会话内的验证命令一律显式 `RUSTC_WRAPPER=D:/autostack/auto-lang/scripts/sccache-wrap.cmd cargo ...`。跨 worktree 命中原理：registry 依赖的 `-C metadata` 不含本地路径 → 内容相同即命中；workspace 本地 crate 因路径参与哈希不命中（本就常改，可接受）。

### C. dev profile 瘦身 + 并行度

根 `Cargo.toml` 加：

```toml
[profile.dev]
debug = "line-tables-only"   # 保留行号回溯，大幅缩小 target 与链接开销
```

`.cargo/config.toml` `[build] jobs = 8` → `12`（20 核 32G 下的保守提升；若出现内存压力回退到 8）。

### D. 流程门禁收敛（技能文件 + AGENTS.md）

- auto-plan-work **Step 6**：收尾复验改为"scoped 复验"（`cargo check -p <涉及crate>` + 模块级 `cargo t <module>`/受影响别名）；明令执行期与收尾禁止全量 `cargo t/tf/ta`。
- auto-plan-work **Step 2** fold 段：每次阶段 fold 进默认分支前，在 worktree 内跑一次 `cargo tf`（全量门禁，防回归上 master）。
- auto-plan-review **Step 2** 表格："Test suite passes" 行明确为唯一全量门禁 `cargo tf`（计划涉及 VM 文件/transpiler/book 时追加 `cargo tv/tt/tb`）；失败沿用既有"打回 /auto-plan:work"路由。
- AGENTS.md：Cargo Test Aliases 表补 `cargo tf` 行（full profile，review/fold 门禁专用）；Category B 末句"合入前运行一次 `cargo t`"改为"`cargo tf`"。

## 测试设计

- 计时验证：`time cargo t` 连跑两次取第二次（热 target、零改动），断言 ≤30s；`time cargo tf` 断言 `str_churn_bounded_large` 仍 ~12s 量级（证明 full 档未降规模）。
- 参数化单点验证：`cargo nextest run -p auto-lang --lib str_churn_bounded_large`（default，~1.2s）vs `RC_CHURN_ITERS=1000000` 显式覆盖（~12s）。
- sccache 验证：scratch worktree（detached HEAD）内 `CARGO_INCREMENTAL=0 cargo check -p auto-lang`，`sccache --show-stats` 命中率 ≥50% 编译请求；随后删除 scratch worktree。
- 行为不变验证：`cargo tf` 3222 项全绿；dev profile 改动后测试无新失败。
- 流程验证：`grep` 技能文件确认新门禁文案落位（"cargo tf"、scoped-only 表述）。

## 验收标准

- [x] AC1: 热态 `time cargo t` ≤30s（基线 40.7s），数字记录入本文件。
  ✅ 实测 **11.3~12.0s**（3222 passed；基线 40.7s，↓约 71%）。
- [x] AC2: `cargo tf` 下 `str_churn_bounded_large` 以 1,000,000 迭代运行（测试输出显示 iters=1000000，耗时 ~12s 量级）；`cargo t`（default）下以 100,000 迭代运行（~1.2s）。
  ✅ 机制因 nextest 版本限制调整为"双测试 + default-filter"（T2 证据）：`cargo tf` 下 1M 大档运行（全量并行负载下 17.1s / 单测 11.7s，输出 `str_churn 1000000`）；`cargo t` 下日常档 100k 运行（1.17s，`str_churn 100000`）。
- [x] AC3: sccache 启用（用户级 RUSTC_WRAPPER/SCCACHE_DIR/SCCACHE_CACHE_SIZE 已 setx）；scratch worktree 冷检查缓存命中率 ≥50%；`scripts/README-sccache.md` 同步推荐值。
  ✅ setx 三项完成（RUSTC_WRAPPER 因 .cmd 转发缺陷直连 `sccache.exe`，见 T5）；scratch worktree 实测命中率 **79.27%**（218 hits / 57 misses，≥50%）；README 已改写。
- [x] AC4: `[profile.dev] debug = "line-tables-only"` 与 `jobs = 12` 合入后 `cargo tf` 全绿且无内存异常。
  ✅ `cargo tf` 3223 全绿、无内存异常；worktree target 26G→**5.7G**。
- [x] AC5: auto-plan-work（Step 2 fold 门禁 + Step 6 scoped 化 + 执行期禁全量）、auto-plan-review（唯一全量门禁 `cargo tf`）两份 SKILL.md 已改并有本仓库内备份；AGENTS.md 别名表含 `tf`、Category B 指向 `cargo tf`。
  ✅ 两份已改（grep：work 3 处 / review 1 处+Rules），备份在 `docs/plans/466-skill-backup/`；AGENTS.md 命中 2 处。
- [x] AC6: 上述改动全部完成后 `cargo tf` 3222 项全绿（0 fail）。
  ✅ **3223 passed / 0 fail**（3222 常规 + 1M 大档）。

## 执行步骤

> Phase A–C 为仓库内改动（worktree `.worktrees/plan-466-dev`）；Phase D 的技能文件在用户目录，直接原地改并先备份到 `docs/plans/466-skill-backup/`（该目录走主 checkout 的 plan 记账，不入 worktree）。

### Phase A：慢测试降档 + 双 profile（仓库）

- [x] T1 **参数化 str_churn_bounded_large**
  文件 `crates/auto-lang/src/vm/tests_rc_lifecycle.rs:624-643`：新增读 `RC_CHURN_ITERS`（缺省 1_000_000），循环上界用该值 `format!` 进测试代码串，eprintln/assert 消息显示实际 iters；断言阈值 `live < 1000` 保持不变。
  验证：`cargo nextest run -p auto-lang --lib str_churn_bounded_large` 全绿（~12s，缺省 1M）；`RC_CHURN_ITERS=100000 cargo nextest run -p auto-lang --lib str_churn_bounded_large` 全绿（~1-2s）。
  [✅ 已完成] 缺省 1M 档 12.19s PASS + 显式 100k 档 1.18s PASS；随后按 T2 发现的版本限制调整为"impl + 双测试"拆分（见 T2 证据）。
- [x] T2 **新建 `.config/nextest.toml`**
  内容见详细设计 A（default=100000 / full=1000000，均 `force = true`）。
  验证：`cargo nextest run -p auto-lang --lib str_churn_bounded_large` ~1-2s（default 生效）；`cargo nextest run -p auto-lang --lib --profile full str_churn_bounded_large` ~12s（full 生效）。若 0.9.138 的 env 语法不同，以计时结果为准调整写法。
  [✅ 已完成] nextest 0.9.138 既不支持 profile 级 env 也不支持全局 `[env]`（两次实测均报 "ignoring unknown configuration key"）→ 按预案调整：测试拆 `str_churn_bounded`(100k)/`str_churn_bounded_large`(1M)，`.config/nextest.toml` 用 `default-filter = "not test(str_churn_bounded_large)"` 排除大档，`.config/nextest-full.toml` 空过滤经 `--config-file` 整体替换。`nextest list` 双档确认：default 仅 bounded、full 两档皆有；无配置警告。
- [x] T3 **别名调整 `.cargo/config.toml`**
  新增 `tf = "nextest run -p auto-lang --lib --profile full"`；`ta` 追加 `--profile full`；更新文件头部注释说明 default/full 双档语义。
  验证：`time cargo tf str_churn_bounded_large` ~12s；`time cargo t str_churn_bounded_large` ~1.2s。
  [✅ 已完成] 机制改为 `--config-file .config/nextest-full.toml`（tf/ta 挂载）；`cargo t str_churn_bounded` = 1.17s（仅日常档）；`cargo tf str_churn_bounded` = 1.19s + 11.71s（两档全跑）。
- [x] T4 **计时验收 AC1**
  `time cargo t` 连跑两次取第二次，数字记入本文件验收标准区。
  [✅ 已完成] 热态 `cargo t`：run1 11.98s / run3 11.31s（3222 passed, 90 skipped；基线 40.7s，AC1 ✓ 余量充足）。run2 出现 fail-fast 中止（1631 未跑），run1/run3 全绿 → 判定为 plan 458 已记录的 ffi_dual_* 并行负载偶发（两次全量回背跑触发），与本改动无关。

### Phase B：sccache 启用（用户环境 + 仓库）

- [x] T5 **setx 三项环境变量 + README 更新**
  执行 setx（见详细设计 B）；`scripts/README-sccache.md` 补 SCCACHE_DIR/SCCACHE_CACHE_SIZE 推荐值与"agent worktree 冷构建收益"说明。
  验证：新起 shell `echo $RUSTC_WRAPPER` 非空；`RUSTC_WRAPPER=D:/autostack/auto-lang/scripts/sccache-wrap.cmd cargo build -p auto-lang --lib && sccache --show-stats` 显示 compile requests > 0 且 hits > 0。
  [✅ 已完成] setx 三项成功；但发现 `.cmd` wrapper 经 cmd.exe 二次解析在超长含空格/括号参数上失败（windows-sys 巨型 `--check-cfg` → exit 1）→ 改为 `RUSTC_WRAPPER` 直指 `C:\Users\zhaop\.cargo\bin\sccache.exe`（wrapper 保留为无 sccache 机器的透传兜底），README 已同步改写并记录此坑。直连后冷检查 `-p auto-man` 全树 65s 正常（632 requests，548 misses 冷填充）；抹 auto-man 指纹重编译 → hits=1 > 0 ✓。
- [x] T6 **跨 worktree 命中验证（AC3）**
  `git worktree add --detach .worktree/scratch-sccache HEAD`，其内执行 `RUSTC_WRAPPER=... CARGO_INCREMENTAL=0 cargo check -p auto-lang`，`sccache --show-stats` 记录命中率（期望 ≥50%），`git worktree remove .worktree/scratch-sccache` 清理。
  验证：命中率数字记入本文件；scratch worktree 已删除。
  [✅ 已完成] 先在 plan worktree 预热 check-mode 缓存（~113 新条目），再于全新 target 的 scratch worktree（detached @ 28c74ef9a）清零统计后复测：**339 requests / 218 hits / 57 misses = 79.27% 命中率**（≥50% ✓；未命中为 workspace 本地 crate，路径参与 `-C metadata`，符合设计预期）。scratch 全树检查全程 27.1s；scratch worktree 已删除。

### Phase C：dev 瘦身 + 并行度（仓库）

- [x] T7 **Cargo.toml profile + jobs（AC4）**
  根 `Cargo.toml` 加 `[profile.dev] debug = "line-tables-only"`；`.cargo/config.toml` `[build] jobs = 8` → `12`（注释注明 32G/20 核依据与回退条件）。预期触发一次性全量重建（此时 sccache 已可用）。
  验证：`cargo t` 全绿（默认档），构建过程无内存异常；若不稳回退 `jobs = 8` 并记录。
  [✅ 已完成] 两项落位；经 sccache 全量重建 ~66s + `cargo t` 3222 全绿（全程 1m17.5s），无内存异常；worktree target 实测 **5.7G**（改前同规模 ~26G，↓约 78%）。

### Phase D：流程门禁收敛（用户目录技能 + 仓库 AGENTS.md）

- [x] T8 **备份并修改 auto-plan-work SKILL.md（AC5 前半）**
  备份 `C:\Users\zhaop\.zcode\skills\auto-plan-work\SKILL.md` → `docs/plans/466-skill-backup/auto-plan-work.SKILL.md`（主 checkout）；然后按详细设计 D 修改 Step 2（fold 前 `cargo tf` 门禁）、Step 6（收尾 scoped 化）与 Rules（执行期禁全量）。
  验证：`grep -n "cargo tf" C:\Users\zhaop\.zcode\skills\auto-plan-work\SKILL.md` 命中 Step 2 与 Step 6 两处。
  [✅ 已完成] 备份完成；grep 命中 3 处：L81（Step 2 pre-fold 门禁）、L132（Step 6 收尾 scoped 化+禁全量）、L167（Rules 新规则）。
- [x] T9 **备份并修改 auto-plan-review SKILL.md（AC5 后半）**
  备份至 `docs/plans/466-skill-backup/auto-plan-review.SKILL.md`；Step 2 表格 "Test suite passes" 行改为唯一全量门禁 `cargo tf`（+按需 tv/tt/tb），并在 Rules 强调这是 plan 生命周期第一次也是唯一一次全量（阶段 fold 前那次除外）。
  验证：`grep -n "cargo tf" C:\Users\zhaop\.zcode\skills\auto-plan-review\SKILL.md` 命中。
  [✅ 已完成] 备份完成；grep 命中 L58（Step 2 表格行：唯一全量门禁 cargo tf + 按需 tv/tt/tb）；Rules 另加"full suite runs here, and only here"规则（L119 邻域）。
- [x] T10 **更新 AGENTS.md**
  Cargo Test Aliases 参考表补 `cargo tf` 行；Category B 末句改为"合入前运行一次 `cargo tf`（full profile）"。
  验证：`grep -n "cargo tf" AGENTS.md` 命中 ≥2 处。
  [✅ 已完成] grep 命中 2 处：L44（Category B 合入前 `cargo tf`）、L54（别名表 tf 行）；t/ta 行同步注明降档/全量语义。

### Phase E：终验

- [x] T11 **全量终验（AC6）+ 记录**
  worktree 内 `cargo tf` 全绿（3222 项）；汇总各项计时/命中率数字回填本文件；`cargo check -p auto-lang` 无新警告。
  [✅ 已完成] `cargo tf` **3223 项全绿**（运行 26.6s / 全程 27.7s，含 1M 大档 17.1s、3222+1 项）。首跑因与 `du` 磁盘扫描并行触发 `benchmark_downcast_performance` perf 断言负载误报，单独重跑全绿（既有负载敏感问题，已记入待澄清）。本轮构建全程无引用本次改动文件的新警告。

## 复审记录

- **复审人**: zcode（/auto-plan:review）
- **时间**: 2026-08-28T14:05+08:00
- **方式**: 在执行 worktree `.worktrees/plan-466-dev` 内复跑全部验收；不信任执行期勾选。

### 逐项判定

| AC | 判定 | 复验证据 |
|:---|:---|:---|
| AC1 cargo t ≤30s | ✅ pass | 复测热态 `time cargo t` = **11.1s**（3222 passed；基线 40.7s） |
| AC2 双档规模 | ✅ pass | 运行时输出实证：default 档 `str_churn 100000: live_pool=4, 1.05s`；full 档（`--config-file nextest-full.toml`）`str_churn 1000000: live_pool=4, 10.35s`。机制为"双测试 + default-filter"（T2 记录的版本适配），行为效果与原设计等价 |
| AC3 sccache | ✅ pass | User 环境变量实测：`RUSTC_WRAPPER=C:\Users\zhaop\.cargo\bin\sccache.exe`（直连，.cmd wrapper 缺陷见 T5）、`SCCACHE_DIR=D:\autostack\.sccache`、`SCCACHE_CACHE_SIZE=30G`；跨 worktree 命中率 79.27%（218 hits/57 misses，执行期 scratch 实测）；README 改写在 diff 中 |
| AC4 profile/jobs | ✅ pass | diff 确认 `[profile.dev] debug="line-tables-only"`（根 Cargo.toml +6 行）与 `jobs = 12`；全量绿、无内存异常；worktree target 实测 **5.7G**（改前同规模 ~26G） |
| AC5 技能/AGENTS | ✅ pass | grep 复测：auto-plan-work 3 处、auto-plan-review 1 处（另 Rules 新增"full suite runs here only"）、AGENTS.md 2 处；两份备份在 `docs/plans/466-skill-backup/` |
| AC6 全量门禁 | ✅ pass | `cargo tf` **3223 passed / 0 fail**（32.8s，含 1M 大档） |

### 遗漏 / 延后 / workaround 检查

- **遗漏**: 无 —— 7 个计划内仓库文件全部在 `d12a1a219..HEAD` diff 中（恰好 3 个提交）；git 外项（setx、两份 SKILL.md、备份）逐一实测确认。
- **延后**（均已在计划文本明示，非静默）: `aavm2_m1_lexer_corpus` 不降档（AC1 已达标无需）；Defender 排除需管理员手动（计划外建议项）；技能文件纳管仓库留待未来。
- **Workaround/债务候选**（非阻塞，建议后续 plan 处理）:
  1. `benchmark_downcast_performance`（`tests/perf_benchmark_tests.rs:394`）刀口断言：本机孤立 best-of-5 实测 optimized 13ns vs 阈值 direct×2=14ns，**余量仅 1ns**；并行 agent 负载下两次误报（复审期间实测，孤立复跑即绿）。与 Plan 466 改动无关（同提交 T11 曾全绿）。建议放宽为 3x 或加绝对容差。
  2. `sccache-wrap.cmd` 在重参数编译上不可用（cmd.exe 限制，T5 记录）；已降级为"无 sccache 机器的透传兜底"，README 记录。后续可删除或改 PowerShell 实现。
  3. `ffi_dual_*` 回背跑全量偶发（plan 458 既有记录，本轮 T4 又现一次）。

### 偏差与交接注记

- 执行期两处机制调整均按计划预案进行并记录于任务证据：T2（nextest 无 env 支持 → default-filter + 双测试）；T5（.cmd wrapper 缺陷 → 直连 sccache.exe）。
- **master 已被并行会话推进**（plan-446/462 合入，触及根 Cargo.toml 等）：本分支 probe-merge 实测**无冲突**（Cargo.toml 自动合并成功，已 abort 还原原状）。正式 fold 由 /auto-plan:merge 执行。
- **spec-impact 元数据**: `supersedes_spec_components` / `new_spec_components` / `touched_goals` 保持空——本计划为构建/测试基础设施与流程技能改动，不触碰 `specs/modules/*` 与任何 spec goal。

### 结论

6/6 AC 全部复验通过，无阻塞债务 → **reviewed**，可进入 /auto-plan:merge。

## 待澄清事项

- Windows Defender 排除（仓库目录 + rustc/cargo/sccache 进程）需管理员手动操作，不在本计划内；建议用户自行添加，可进一步压低零改动构建检查的 ~11.6s。
- `aavm2_m1_lexer_corpus`（4.1s）本计划不降档——AC1 达标无需动它；若未来需进一步压缩可同法参数化。
- `jobs = 12` 的内存余量为经验值（32G RAM）；T7 验证若不稳定即回退 8。
- 技能文件无版本控制，备份仅存 `docs/plans/466-skill-backup/`；如需长期纳管可考虑未来把技能迁入仓库。
- **（执行期新增）负载敏感的 perf/双进程测试既有偶发**：`benchmark_downcast_performance` 在与磁盘扫描（du）并行时误报一次；`ffi_dual_*` 在回背跑全量时偶发一次（plan 458 已记录）。本轮均单独重跑转绿、与本改动无关，但建议后续把 perf 断言改为相对阈值或移入专项档，降低全量门禁的假阳性率。
- **（执行期新增）sccache-wrap.cmd 在 Windows 不可用于重参数编译**：cmd.exe 二次解析在超长含空格/括号参数（如 windows-sys 的巨型 `--check-cfg`）上失败（exit 1）。已改为 RUSTC_WRAPPER 直连 sccache.exe；wrapper 保留为无 sccache 机器的透传兜底，README 已记录此坑。
