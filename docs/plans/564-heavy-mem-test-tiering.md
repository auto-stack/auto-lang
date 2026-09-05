---
plan_id: PLAN-564
status: drafting               # drafting → executing → execution_done → reviewed → archived
feature_name: heavy-mem-test-tiering
author: [zhaopuming]
created_at: 2026-09-05
updated_at: 2026-09-05

# /auto-plan:review 结束时填写：
supersedes_spec_components: []
new_spec_components: []
touched_goals: []             # 引用 docs/specs/goals.md 的 GOAL-NNN

affects: []                   # 受影响的 specs 路径（测试基建，review 时定）
current_step: 0
total_steps: 7
---

# [PLAN-564] 重内存测试分层与并发限流（aavm2 裸 cargo test 峰值 9.78GB 事件）

## 变更摘要

2026-09-05 14:37 事件：某会话在 lang-532 worktree 跑裸
`cargo test -p auto-lang --lib --features test-vm-files aavm2_`，libtest 单进程
12 线程全并发，实测峰值工作集 **9.78 GB**（PeakCommit 9.93 GB）。本 plan 落实
三层防线：

1. **逐用例内存标记 + 加权限流**：nextest `[test-groups]` + `max-threads`
   （本仓 nextest 0.9.138 已实测支持），重测试进低并发组——重测试占用整个
   组的线程配额，等效"按内存消耗加权占用更多线程数"。
2. **重测试独立成档**：XL 级（单测峰值 ≥800MB）从日常档 default-filter
   排除（先例：Plan 466 1M churn / Plan 532 T3 塔），仅在 tf/t3 全量档运行
   且受组限流。
3. **裸 cargo test 防线**：XL/LG 级测试体内自守门——非 nextest 环境
   （无 `NEXTEST` env）且无显式 opt-in 时秒退 SKIP，防止裸 `cargo test`
   误触发全并发重跑。

配套：per-test 峰值内存测量脚本常驻化（`scripts/measure_test_mem.py`），
权重表作为 nextest overrides 的单一事实来源；全档峰值预算 **≤2 GB**
（理想 ≤1 GB，若最重单测自身超限则单测削内存或接受 2 GB 硬顶）。

## 目标

- G1: 任何测试档（t/tf/tv/tt/tb/ta/t3）运行期全进程树峰值内存 ≤2 GB
  （现状: 裸 cargo test aavm2_ = 9.78 GB）。
- G2: 重内存测试可标识、可测量、可复测——新增重测试时有明确登记路径
  （测量脚本 → 权重表 → nextest 组）。
- G3: 裸 `cargo test`（不经 nextest）不再可能触发重测试全并发——自守门
  秒退并打印指引。
- G4: 日常档（cargo t）不回归：4304 测试 ~46s 基线不动摇（XL 已在
  default-filter 排除，本就不跑；不新增轻测试负担）。

## 架构方案

不动编译器/VM/语言核心，只动**测试基建四层**：

```
层1 测量层   scripts/measure_test_mem.py
             （nextest --jobs=1 逐测试跑 + 轮询子进程 PeakWorkingSet64，
              输出按峰值排序的权重表；复测对比用同一脚本）
层2 数据层   .config/test-mem-weights.md
             （人读机读兼顾：测试名 → 实测峰值 → 档位 XL/LG/MD/LT，
              是层3 overrides 的单一事实来源）
层3 限流层   .config/nextest.toml / nextest-full.toml / nextest-t3.toml
             （[test-groups] mem-xl{max-threads=1} mem-lg{max-threads=2}
              mem-md{max-threads=4}；[[profile.default.overrides]] 按
              test(...) 过滤归属；default-filter 排除 XL）
层4 自守门   crates/auto-lang/src/tests/heavy_gate.rs
             （heavy_gate("名字") -> bool：NEXTEST 或 AUTO_LANG_HEAVY_MEM
              任一存在才真跑，否则 eprintln SKIP 指引后返回）
```

内存预算模型：档峰值 ≈ 轻池并发 × 轻单测峰值 + Σ(组并发上限 × 组内单测
峰值)。轻池单测实测普遍 <50MB，重池按组限流后，约束退化为
`max(最重单测峰值, 2×LG, 4×MD) + 轻池余量 ≤ 2GB`。若最重单测本身 >2GB，
进入 D2 决策点（单测削内存 / 接受例外并记录）。

## 需求分析与背景调查

（取材 docs/specs/overview.md 九模块表 + 本仓测试基建现状，2026-09-05 实测）

- **事件实测**（本会话 PowerShell 取证）：PID 33692 =
  `auto_lang-95f9106d11fe52ac.exe`（auto-lang lib 测试二进制），父链
  `bash → cargo test -p auto-lang --lib --features test-vm-files aavm2_`，
  lang-532 worktree（Plan 532 aavm2 分支）。12 线程，峰值 WS 9.78GB /
  Commit 9.93GB，事后裁剪回 ~2GB。CPU 15+ 分钟仍在跑。
- **语料不是原因**：`test/vm/aavm2/` 全部 11 文件共 611KB，单文件 33~183
  字节。峰值来自测试体内的解释执行/差分对拍（aavm2 m1-m5 语料闸门 +
  a2r 转译对拍，含自举链 `auto/lib/*.at` 加载与嵌套解释）。
- **两条执行路径语义不同**：
  - 别名（t/tf/tv/tt/tb/ta/th/t3）全走 **nextest**：每测试独立进程，
    但默认 jobs=逻辑核数（本机 20）——重测试全并发同样会爆，只是没有
    单进程叠加效应。nextest 0.9.138 **不支持** profile 级 env /
    全局 [env]（nextest.toml 头注实证），所以限流唯一手段是 test-groups。
  - 裸 `cargo test`（libtest）：单进程多线程，无任何并发-内存控制。
    事件即此路径。`RUST_MIN_STACK=16MB`（仓库级）放大每线程栈但非主因。
- **机制已实测验证**（本会话 probe）：
  `cargo nextest show-config test-groups --config-file <probe.toml>` 输出
  `group: mem-heavy (max threads = 1)`——`[test-groups]` + overrides 过滤
  在本仓版本可用。
- **分层先例**：Plan 466（1M churn 用 default-filter 排除 + full 配置整体
  替换双档）、Plan 532（T3 塔 env 自守门 T3_MILESTONE + 双档 filter 排除
  三重防误触发）。本 plan 是同一哲学在"内存维度"的推广。
- **依赖**：aavm2 测试文件（aavm2_m1~m5/a2r/t3.rs）目前只存在于
  plan-532-dev 分支（532 状态 executing，未 fold）。本 plan 的 aavm2 权重
  测量与 overrides 归属需要这些文件 → worktree 从 plan-532-dev tip 派生
  （stacked；532 先合、564 后合，无冲突）。机制部分（测量脚本/组配置/
  自守门 helper）与 master 无冲突。

## 详细设计

### D1: 测量脚本 `scripts/measure_test_mem.py`

- 输入：测试名过滤串 + feature 集（默认 `test-vm-files`）。
- 实现：`cargo nextest run -p auto-lang --lib --features <F> <filter>
  --jobs=1 --no-fail-fast`，同时后台线程每 200ms 扫
  `Get-CimInstance Win32_Process -Filter "Name LIKE 'auto_lang-%'"` 取
  `PeakWorkingSet64`（nextest 单测单进程，逐个归属：串行 jobs=1 时
  当前活跃进程即当前测试）。输出 Markdown 表：测试名 | 峰值MB | 档位。
- 复测：同命令重跑，与 `.config/test-mem-weights.md` 对比，漂移 >50%
  的行高亮（供 review 档人工复核，不做机判失败——避免脆门禁）。

### D2: 档位与阈值（初值，T3 测量后可调）

| 档位 | 单测峰值 | 组 | max-threads | 运行档 |
|---|---|---|---|---|
| XL | ≥800MB | mem-xl | 1 | 仅 tf/t3（default-filter 排除） |
| LG | 300~800MB | mem-lg | 2 | 全档（含日常） |
| MD | 100~300MB | mem-md | 4 | 全档 |
| LT | <100MB | （默认池） | jobs 默认 | 全档 |

- 若实测最重单测 >2GB：优先单测削内存（如语料分批、drop 中间结构）；
  不可行则该测升格为"塔级"（env 自守门 + 仅 t3，先例 T3），并在
  KNOWN-DEBT 登记。
- 裸 cargo test 防线覆盖面：XL + LG（MD 以下不守门，避免噪音）。

### D3: nextest 配置（三文件同步）

`.config/nextest.toml`（日常档）追加：
```toml
[test-groups]
mem-xl = { max-threads = 1 }
mem-lg = { max-threads = 2 }
mem-md = { max-threads = 4 }

[[profile.default.overrides]]
filter = "test(<XL 名单，逗号并列>)"
test-group = "mem-xl"
# … mem-lg / mem-md 同构；default-filter 追加 and not test(<XL 名单>)
```
- nextest-full.toml / nextest-t3.toml 自成一体（现状如此），同样带组
  配置但**不排除 XL**（全量语义保留，靠组限流控内存）。
- overrides 名单来自权重表，逐名精确匹配（不用宽前缀，防误伤后续新增
  同前缀轻测试）。

### D4: 自守门 `crates/auto-lang/src/tests/heavy_gate.rs`

```rust
/// 重内存测试守门：仅在 nextest（每测独立进程，受 test-groups 限流）
/// 或显式 opt-in（AUTO_LANG_HEAVY_MEM=1）时真跑。
/// 裸 cargo test（libtest 单进程多线程全并发）下秒退，防 9.78GB 事件复发。
pub(crate) fn heavy_gate(name: &str) -> bool {
    let ok = std::env::var_os("NEXTEST").is_some()
        || std::env::var_os("AUTO_LANG_HEAVY_MEM").is_some();
    if !ok {
        eprintln!("SKIP {name}: heavy-mem test; run via cargo tv/tf (nextest) or AUTO_LANG_HEAVY_MEM=1");
    }
    ok
}
```
- 使用方（XL/LG 测试函数头部）：`if !heavy_gate("test_aavm2_m5_engine_corpus") { return; }`
- `NEXTEST` env 的存在性由 T1 实证（nextest 文档声明子进程注入
  `NEXTEST=1`，T1 用一次性探针确认本机版本行为）。
- 门只加在 plan-532-dev 侧现存的重测试上；master 侧无 XL/LG 存量
  （str_churn_bounded_large 1M 档实测峰值待 T3 一并定级——若达 LG/XL
  同样入组+守门）。

### D5: 执行时序与 worktree

- `git worktree add D:/autostack/.wt/lang-564/auto-lang -b plan-564-dev plan-532-dev`
  （stacked 于 532 tip；532 先 fold 后本分支再合，无冲突。AGENTS.md 默认
  从 master 建，此处偏离的理由：aavm2 测试文件在 532 分支，从 master 建
  则 T3/T5 全部空转）。
- 机制文件（scripts/measure_test_mem.py、heavy_gate.rs、nextest 三配置、
  AGENTS.md 档表）为通用基建，532 合并后自然带出。

## 测试设计

- 机制自测：T1 探针（nextest 下 NEXTEST=1 可见；裸 cargo test 下不可见）。
- 权重表实证：T3 对 aavm2 全套 + str_churn 两档逐测测量，表入库。
- 防线实证（T5/T7）：
  - 裸路径：worktree 内 `cargo test -p auto-lang --lib --features
    test-vm-files aavm2_ -- --test-threads 12`——须秒级完成、输出 SKIP
    指引、进程峰值 <500MB。
  - nextest 路径：`cargo tf aavm2_` 照跑且全绿；同时另终端轮询
    auto_lang-* 进程树峰值 ≤2GB。
- 日常档不回归：`cargo t` 全绿且耗时与 4304/~46s 基线同量级。

## 验收标准

- [ ] A1: `scripts/measure_test_mem.py` 可用，产出 aavm2 全套 + str_churn
      两档的逐测峰值权重表，入库 `.config/test-mem-weights.md`。
- [ ] A2: `.config/nextest.toml`（含 full/t3）三组配置生效：
      `cargo nextest show-config test-groups` 显示组与匹配数非零；
      default-filter 排除 XL 名单。
- [ ] A3: 裸 `cargo test --features test-vm-files aavm2_`（12 线程）秒级
      秒退 + SKIP 指引，进程峰值 <500MB（实测前后对比记入 plan：
      9.78GB → <0.5GB）。
- [ ] A4: `cargo tf aavm2_`（nextest 全量语义）全绿，运行期进程树峰值
      ≤2GB（轮询实测记录）；`cargo t` 基线不回归。
- [ ] A5: XL/LG 单测清单与权重表一致（review 时人工核对 overrides 名单
      与权重表档位列逐条对应）。
- [ ] A6: AGENTS.md 测试档表更新（新增重内存分层说明 + 测量复测方法 +
      新增重测试登记路径）。

## 执行步骤
（原子任务：精确文件路径 + 确切操作 + 验证命令；每步完成后追加 [✅ 已完成] 一行证据）

- **T1** 实证 nextest 子进程 env 注入。
  文件: `scratch/p564/probe_env.rs`（临时，不入库）——一个打印
  `NEXTEST`/`NEXTEST_TEST_BINARY_PATH` 的 #[test]。
  操作: 在 lang-564 worktree（建立后）用 `cargo nextest run probe_env`
  跑一次、再裸 `cargo test probe_env` 跑一次，记录两侧 env 差异。
  验证: nextest 侧输出含 `NEXTEST=1`；裸侧无。若本机版本不注入，
  改用 `AUTO_LANG_HEAVY_MEM` 双 env 方案并更新 D4。
- **T2** 测量脚本。
  文件: `scripts/measure_test_mem.py`（新建）。
  操作: 按 D1 实现（nextest --jobs=1 串行 + Win32_Process 峰值轮询 +
  Markdown 表输出 + 与既有权重表对比漂移标注）。
  验证: `python scripts/measure_test_mem.py str_churn`（master 存量）
  输出非零峰值表；重复运行结果稳定（±20%）。
- **T3** aavm2 权重测量与定级。
  文件: `.config/test-mem-weights.md`（新建，入库）。
  操作: worktree 内 `python scripts/measure_test_mem.py aavm2_ -F test-vm-files`
  + `python scripts/measure_test_mem.py str_churn`；按 D2 阈值定级填表。
  验证: 表覆盖全部 aavm2_* 测试名（与 `cargo nextest list aavm2_` 名单
  逐一对照无遗漏）；最重单测数值明确（决定 D2 分支走向）。
- **T4** nextest 组配置接线。
  文件: `.config/nextest.toml`、`.config/nextest-full.toml`、
  `.config/nextest-t3.toml`。
  操作: 按 D3 写入 [test-groups] + overrides（名单来自 T3）；
  日常档 default-filter 追加 XL 排除。
  验证: `cargo nextest show-config test-groups --config-file .config/nextest.toml`
  三组匹配数非零；`cargo nextest list -E 'not test(...)'` 确认 XL 被日常档
  排除。
- **T5** 自守门接线。
  文件: `crates/auto-lang/src/tests/heavy_gate.rs`（新建）+ XL/LG 测试
  函数头部接线（aavm2_m1~m5/a2r/vm_file_tests 中达档者 + str_churn_large
  若达档）+ `crates/auto-lang/src/tests/mod.rs` 挂模块。
  操作: 按 D4 实现 heavy_gate 并在名单测试头部加一行守门。
  验证: `cargo check -p auto-lang` 零错；worktree 内裸
  `cargo test -p auto-lang --lib --features test-vm-files aavm2_` 秒级
  完成且输出 SKIP 指引（对照 A3）。
- **T6** 文档与权重表收口。
  文件: `AGENTS.md`（测试档表 + 重内存分层说明）、
  `docs/plans/564-heavy-mem-test-tiering.md`（本文件，记录前后对比数据）。
  操作: 更新档表注释（t/tf/tv/t3 语义变化：XL 仅全量档 + 组限流；
  新增重测试登记路径三步：测量 → 权重表 → overrides/守门）。
  验证: 文档审读 + `grep -n "mem-xl" AGENTS.md .cargo/config.toml` 命中。
- **T7** 复审验证（峰值实测 + 基线回归）。
  操作: ① worktree 内 `cargo tf aavm2_`，另终端轮询 auto_lang-* 进程树
  峰值记入本文件；② `cargo t` 全绿且耗时对照 46s 基线；③ 裸 cargo test
  防线复跑一次。
  验证: 三项实测数据记入"复审记录"前的执行证据区（9.78GB → 实测值，
  须 ≤2GB）。

## 复审记录

## 待澄清事项

- **Q1（D2 分支）**: 最重单测实测峰值若 >2GB——单测削内存（改测试体，
  动 plan-532 语义）还是升格塔级仅 t3？默认取向：升格塔级（不动 532
  在途语义），KNOWN-DEBT 登记。
- **Q2（tv 档口径）**: XL 从日常 default-filter 排除后，532 在途会话的
  日常 aavm2 验证改用 `cargo tf aavm2_`（全量档带限流）。是否额外提供
  `tvh`（vm-heavy）便捷别名？默认：不加，tf+filter 已够用，别名表已长。
- **Q3（ stacked worktree）**: lang-564 从 plan-532-dev tip 派生（偏离
  AGENTS.md 默认从 master 建）。若 532 在 564 执行期间 fold，564 分支
  rebase 到 master 即可（机制文件无冲突）。
