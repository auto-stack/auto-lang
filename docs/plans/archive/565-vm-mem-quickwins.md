---
plan_id: PLAN-565
status: archived                # drafting → executing → execution_done → reviewed → archived
feature_name: vm-mem-quickwins
author: [zhaopuming]
created_at: 2026-09-05
updated_at: 2026-09-07

# /auto-plan:review 结束时填写：
supersedes_spec_components:
  - ".config/test-mem-weights.md: 565 L1 后受触集复测与结论注记（m1 809/m2 806 持平、m3 792/m4 796 落 LG 边界、新增 rerun_consistency 813 行；表头沉淀'峰值≈单次编译高水位，L1 消除重复不降峰值，压缩属 566'结论）"
new_spec_components: []             # 无新 spec 文档组件——mem_profile.rs 与 aavm2_corpus_runner.rs 为代码/测试模块，知识经 ledger reports/reviews 节沉淀
touched_goals:                      # 引用 docs/specs/goals.md 的 GOAL-NNN
  - "goal-016: 构建与测试基础设施——aavm 语料闸门 once-compiled（m2 147s→2.98s、m4 ~315s→秒级）+ mem-profile 归因工具 + 权重表 565 版"
  - "goal-017: 自举 aavm——六道闸门中四道 harness 提速（L1，判据断言原样、语义零改动）"

affects: [auto-lang/vm]       # 优化 VM 测试期内存足迹（P0 归因/L1 编译复用/L3 优化档）
current_step: 6
total_steps: 6
---

# [PLAN-565] VM 内存快赢三件套（P0 归因 + L1 编译复用 + L3 测试优化档）

## 变更摘要

564 家族第二环（564 遏制与度量 → **565 快赢** → tag 锚点 → 566 Value 装箱）。
不触碰 `Value` 内存布局（那是 566 的领地），只做三件低风险高回报的事：

1. **P0 归因**：feature 门控的计数分配器（`mem-profile`），把单次重测试 ~800MB
   峰值精确分解到"编译侧 token/AST/session vs 执行侧 Value 堆"——为 566 的
   装箱范围提供数据裁决，避免盲改。
2. **L1 编译复用**：aavm2 语料闸门 harness 目前**每个语料文件重新编译整条
   ~500KB 自举链**（m2 = 35 文件 × 4.2s = 147s）。改为链只编译一次、语料走
   运行时通道，预期 m2 147s → <30s，同时消除 35 次重复内存爬坡。
3. **L3 测试优化档**：`[profile.test.package]` 对 auto-lang/auto-val 设
   `opt-level=1` 的受控实验，量化 debug 构建（opt-level=0）对内存/时长的
   膨胀占比，数据说话决定采纳与否。

三者均以 564 的 `scripts/measure_test_mem.py` 权重表为前后对比标尺。

## 目标

- G1: `--features mem-profile` 下任一测试进程退出时输出分配报告
      （total/peak/live bytes、分配次数、按尺寸分桶），不带 feature 时零开销。
- G2: aavm2 m2 语料闸门独跑时长 147s → **<30s**（编译复用），35 文件全绿不变。
- G3: L3 实验有明确数据结论（采纳：权重表峰值降幅记录在案；否决：回退并记录原因）。
- G4: 全部效果量化记录进本 plan 与 `.config/test-mem-weights.md`，供 566 裁决。

## 架构方案

```
P0  crates/auto-lang/src/mem_profile.rs（新）
    #[cfg(feature="mem-profile")] #[global_allocator] CountingAlloc
    （包裹 System 分配器；原子计数 total/allocs/frees/peak-live + 尺寸分桶；
     报告经 #[ignore] 专用测试输出。默认构建零成本——无 feature 时 System。）
L1  crates/auto-lang/src/tests/aavm2_corpus_runner.rs（新）+ 四闸门切换
    现状: program = lib_code + main{parse_dump("语料字面量")} → 每文件一程序
    → 每文件全链重编译（tokenize 500KB × 35 次）。
    方案选型（T3 定夺，倾向 a）:
      a) 运行时通道: 程序固定为 lib + main{parse_dump(env/文件读入)}，
         语料经 env（AUTO_CORPUS_INPUT）或固定临时文件注入 → 程序串恒定
         → 编译产物在 helper 内缓存，34/35 次编译被消掉。
      b) VM 模块缓存 API: compile_session 复用 + 仅编译小 main + 链接
         （引擎侧新 API，改动大，a 不可行才走）。
L3  Cargo.toml（workspace 根）
    [profile.test.package.auto-lang] opt-level = 1（+ auto-val）
    实验性质：跑 564 权重表 + cargo t 基线对比后定去留。
```

## 需求分析与背景调查

（承接 564 背景调查，2026-09-05 本会话实测/静态取证）

- **单测试足迹实测**：m2_parser_corpus 独跑 147.3s/35 文件/峰值 ≥811MB；
  static_diff 独跑 13min+ 峰值 1050MB；另有一重测 13s 内即达 760MB——
  峰值与语料内容无关，是"编译+执行 500KB 自举链"的固定足迹（~1600× 放大）。
- **静态成因**（已核实）：`Value` 巨型 enum（auto-val/value.rs:147）由
  `Node`（~232B：3×AutoStr+Args+Obj(IndexMap)+Kids(IndexMap)）撑大——此为
  566 领地，本 plan 不动；本 plan 只动"重复编译"（L1）与"debug 膨胀"（L3），
  并用 P0 把两者占比测出来。
- **L1 证据**：`test_m2_corpus_file`（aavm2_m2.rs:33）每文件拼
  `format!("{}\nfn main() {{...}}", lib_code, 语料字面量)` 后 `run_with_capture`
  → 程序串逐文件不同 → 无任何复用可能；`aavm2_lib_source` 每文件重读磁盘。
- **排除项**：`LinearAllocator` 死代码；string_pool 包级非全局；
  `run_with_capture` 每调用新线程瞬态无泄漏；`AutoStr`=EcoString 无问题。
- **依赖**：564 的测量脚本与权重表（对比标尺）；aavm2 测试文件在
  plan-532-dev 分支（同 564，stacked worktree）。

## 详细设计

### D1: P0 计数分配器

- `CountingAlloc`：包裹 `std::alloc::System`；`alloc/dealloc/realloc` 上
  原子累计 `total_bytes/alloc_count/free_count`，维护 `current_live` 与
  `peak_live`（原子 CAS 环）；尺寸分桶（≤64B/≤256B/≤1K/≤4K/≤64K/>64K）。
- 报告输出：libtest 控制进程生命周期，无法拦 exit——采用**专用报告测试**：
  `test_mem_profile_report()`（#[ignore]，仅 mem-profile feature 下编译），
  测试体内 eprintln 报告；归因跑法：
  `cargo test -p auto-lang --lib --features mem-profile,test-vm-files -- --ignored test_mem_profile_report`
  （单测独占进程，计数即该进程全程）。
- 编译侧/执行侧拆分：在 `run_with_capture` 前后各采样一次计数（编译结束/
  执行开始分界），差值归执行侧，首采样归编译侧。
- 线程安全：全部 AtomicU64，无锁。

### D2: L1 编译复用（方案 a 优先）

- 新 helper `aavm2_corpus_runner.rs`：`once_compiled_run(main_tpl, feed)`
  ——首次拼程序（模板含语料注入点：env 或固定临时文件
  `target/tmp/aavm2_corpus_input.at`，T3 按 native 可用性定）并
  `run_with_capture`，程序串与编译产物按程序哈希缓存；后续仅换注入内容。
- **T3 先做现状实测**：同程序串连跑 3 次的耗时对比——确认 VM 无任何
  进程内编译缓存（预期第 2/3 次与第 1 次同阶），坐实 helper 缓存的必要性
  与收益上限。
- m1/m2/m3/m4 语料闸门切换到该 helper；m5/engine（执行塔）不在本期范围。
- 判据不降级：35 文件 AST dump 逐字节一致（既有断言原样保留）。

### D3: L3 实验规程

- 基线：564 权重表（opt-level=0）+ `cargo t` 总时长。
- 实验：workspace `Cargo.toml` 加 `[profile.test.package.auto-lang]` /
  `[profile.test.package.auto-val]` `opt-level = 1`，重测同表。
- 采纳判据：重测试峰值降 ≥25% 或日常档总时长降 ≥30%，且测试编译增量
  可接受（<2× 基线编译时长）；不满足即回退并记录数据。

## 测试设计

- P0：`test_mem_profile_report`（#[ignore]+feature 门控）输出报告行
  `MEMPROFILE total=… peak_live=… buckets=…`；不带 feature 的
  `cargo check -p auto-lang` 零新增符号、`cargo t` 基线不变。
- L1：m1/m2/m3/m4 语料闸门全绿（断言原样）；独跑时长前后对比入 plan；
  新增回归：同程序串连跑两次输出一致（缓存正确性）。
- L3：`cargo t` 全绿 + 基线对比；变更仅 workspace Cargo.toml。

## 验收标准

- [x] A1: `cargo test -p auto-lang --lib --features mem-profile -- --ignored test_mem_profile_report --nocapture`
      产出含 total/peak_live/分桶的报告；无 feature 时构建与基线零差异。
      （T1/T2 证据：报告五行输出；默认构建 168 warnings 与主检出基线逐字一致）
- [x] A2: m2 语料闸门独跑 <30s（基线 147s），35 文件断言全绿；
      m1/m3/m4 语料闸门同步切换且全绿。
      （T4 证据：m2 2.98s/35 文件；m1 5/m3 8/m4 58 文件全绿）
- [x] A3: L3 有数据结论（采纳配置入库或回退），对比数字记入本 plan。
      （T5 证据：否决回退——峰值 0% 降幅/日常 -6.7%/构建 2.29×）
- [x] A4: `.config/test-mem-weights.md` 更新（565 后权重表 vs 564 基线），
      最重测试峰值变化量化。
      （T6 证据：受触五测复测入表，m3/m4 落 LG 边界，最重 static_diff
      1232 未触及沿用；结论=峰值≈单编译高水位，L1 不降峰值）
- [x] A5: P0 报告给出编译侧/执行侧占比结论，写入执行证据区
      （566 的装箱范围裁决直接引用）。
      （T2 证据：编译侧 alloc 7419.7MiB/8.8M 次 vs 执行侧 2.9MiB/5.5万次，
      ~99.96% 编译侧；端到端重跑再烧 7431.0MiB=每次调用全链重编译）

## 执行步骤
（原子任务：精确文件路径 + 确切操作 + 验证命令；每步完成后追加 [✅ 已完成] 一行证据）

- **T1** mem-profile 骨架。
  文件: `crates/auto-lang/Cargo.toml`（feature "mem-profile"）、
  `crates/auto-lang/src/mem_profile.rs`（新）、`crates/auto-lang/src/lib.rs`
  （`#[cfg(feature="mem-profile")] pub mod mem_profile;` + cfg 条件
  `#[global_allocator]`）。
  操作: 按 D1 实现计数分配器与原子统计。
  验证: `cargo check -p auto-lang`（默认零差异）；
  `cargo check -p auto-lang --features mem-profile` 通过。
  [✅ 已完成] worktree commit `feat(vm): PLAN-565 T1 mem-profile 骨架`；
  默认 check 168 warnings 与主检出基线完全一致（零新增符号），
  `--features mem-profile` check 绿。
  附注: worktree 内跨仓依赖 `autodown-core`(path `../../../auto-down/...`)
  按组布局补兄弟 worktree `D:/autostack/.wt/lang-565/auto-down`
  （detached @ a6d5ecf，本 plan 不改它，merge 时一并清理）。
- **T2** 报告测试 + 编译/执行分界采样。
  文件: `crates/auto-lang/src/mem_profile.rs`（report() 辅助）+
  `crates/auto-lang/src/tests/`（挂 #[ignore] 报告测试，测试体内
  run_with_capture 前后采样）。
  操作: 按 D1 输出报告行。
  验证: `cargo test -p auto-lang --lib --features mem-profile -- --ignored test_mem_profile_report --nocapture`
  输出 MEMPROFILE 行。
  [✅ 已完成] worktree commit `feat(vm): PLAN-565 T2`；
  `tests/mem_profile_report_tests.rs`（m2 同形态探针）+ Snapshot/delta/
  report 三辅助。分界实现取 `create_vm_from_source`（编译侧）与
  `spawn_task+run_task_loop`（执行侧，主管线同形态，lib.rs "5. Execute"段）
  ——比 run_with_capture 黑盒前后采样更精确（run_with_capture 单次调用
  无法分出编译/执行；D1 的分界意图按此忠实落地），并保留
  run_with_capture 端到端交叉核对（输出逐字一致断言通过）。
  实测报告（5.26s，含两次全链）：
  - phase0 harness(lib 源拼装): 3.2MiB
  - **phase1 编译侧: alloc 7419.7MiB / 8.8M 次分配, live +1.8MiB**
  - **phase2 执行侧: alloc 仅 2.9MiB / 5.5万次**
  - phase3 端到端重跑(run_with_capture): 又烧 7431.0MiB——单次调用
    全链重编译坐实
  - 进程 total 14857.3MiB, **peak_live 755.2MiB**, 终态 live 1.3MiB(无泄漏)
  - 编译期分桶: <=1K 4.0GiB / <=4K 1.5GiB / <=64K 2.2GiB(中尺寸 AST/串主导)
- **T3** L1 现状重复成本实测 + 通道选型。
  操作: worktree 内一次性探针（不入库）：同程序串 `run_with_capture` 连跑
  3 次记录耗时；查 .at env()/read_file native 可用性定注入通道。
  验证: 三次耗时 + 通道结论记入本 plan 执行证据区。
  [✅ 已完成] 探针（跑毕已删，未入库）实测：
  - 同程序串 3 连跑（m2 同形态探针程序）：**2.374s / 2.173s / 2.110s**
    ——同阶无衰减，VM 无任何进程内编译缓存坐实（T2 内存侧互证：
    同程序重跑再烧 7.43GiB churn）。
  - 通道选型（Q1 裁决）：**文件读入通道**。`File.read_text(path)` 在
    纯 .at 程序可直接调用（native_registry.rs:609
    `to_canonical("File.read_text") → "auto.file.read_text"` = nat#1000，
    P532 W1 G15；aavm lib 自身 engine.at:808/codegen.at:4200 在用）；
    探针验证 dump 与内联字面量通道逐字一致（match=true）。
    env 通道无需启用（文件通道零疑虑已足够）。临时文件取
    `std::env::temp_dir()` 下进程唯一名（避并行测试进程冲突）。
- **T4** L1 helper 落地 + 四闸门切换。
  文件: `crates/auto-lang/src/tests/aavm2_corpus_runner.rs`（新）+
  `aavm2_m1.rs`/`aavm2_m2.rs`/`aavm2_m3.rs`/`aavm2_m4.rs`（corpus 循环
  改走 helper）+ `tests/mod.rs` 挂模块。
  操作: 按 D2 实现；判据断言原样保留。
  验证: nextest 跑 `aavm2_m2` 全绿；独跑 m2 时长 <30s（对照 A2）。
  [✅ 已完成] worktree commit `perf(test-infra): PLAN-565 T4`。
  `aavm2_corpus_runner.rs`（挂 `tests.rs`，cfg test-aavm）：
  `run_corpus_once_compiled(gate, dump_fn, cases)` —— 程序串
  `lib+main{print(dump_fn(File.read_text(临时文件)))}` 闸门内恒定 →
  `create_vm_from_source` 编译一次 → 逐语料换注入文件+清 stdout 缓冲+
  同 VM 重跑 main（spawn_task+run_task_loop，daemon 同款重入）；
  VM !Send → 编译与全部重跑固定同一专用 16MB 栈线程；判据断言
  （含 m4 失败现场 RAW 诊断）原样留在各闸门。
  **实测（Windows，显式 --ignored + AUTO_LANG_HEAVY_MEM=1；闸门带
  cfg_attr(windows,ignore) nextest 不跑 ignored，本地度量走此通道，
  Linux/CI 走 taa 全额受益）**：
  - m2: **147s → 2.98s**（35 文件 AST dump 全绿，A2 达成 <30s）
  - m1: 5 文件全绿；m3: 8 文件全绿；m4: **58 文件全绿**（m4 闸门
    基线 ~315s，本批内 3 闸+回归合计 15.56s）
  - 新增回归 `test_aavm2_corpus_runner_rerun_consistency`（缓存 VM 重入
    3 语料无串台 + 两批跑一致）绿。
  附注（574 语境）: 12 个双重解释器路径测试在 Windows 日常档 ignored
  是 2026-09-06 裁定，本步未改动 ignore 属性——L1 收益落在 Linux/CI
  全量档与显式诊断跑法。
- **T5** L3 实验。
  文件: workspace 根 `Cargo.toml`（[profile.test.package] 两段）。
  操作: 按 D3 规程测权重表 + cargo t 基线；达标保留/不达标回退。
  验证: 前后峰值与时长对比数据记入本 plan 执行证据区。
  [✅ 已完成] **结论：否决回退**（Cargo.toml 已还原，无代码变更入库；
  worktree 干净）。数据（Windows，lang-565 worktree，2026-09-07）：

  | 指标 | 基线 opt=0 | 实验 opt=1 | 判据 | 结果 |
  |---|---|---|---|---|
  | 重测试峰值（mem-profile 报告测试 peak_live） | 755.2MiB | **755.2MiB（逐字节同）** | 降 ≥25% | ❌ 0% |
  | 日常档总时长（cargo t 暖态 --no-fail-fast） | 47.76s/45.0s | 44.57s | 降 ≥30% | ❌ ~6.7% |
  | 测试构建段（clean -p 后全量墙钟 − 运行段） | ~63s | ~144s | 增量 <2× | ❌ 2.29× |
  | （参考）重测试时长（同报告测试） | 6.16s | 3.73s | — | 快 ~1.65× |

  **信息性结论（供 566 裁决引用）**：①重测试内存峰值与优化档位**无关**
  （alloc 结构决定，非 debug 构建伪影）——566 装箱必须改分配模式本身，
  编译器优化档救不了峰值；②opt=1 对重测试**时长**确有 ~1.65× 收益但
  被构建段 2.29× 膨胀吞没，日常档（海量轻测）无收益；③574 后权重表
  XL/LG 全员（aavm 系）Windows 本地 ignored，重测试峰值改经 T2 报告
  测试（编译侧 755MiB 峰值同源现象）测量，双臂同法可比。
- **T6** 权重表更新 + 收口。
  文件: `.config/test-mem-weights.md`、本 plan（执行证据）。
  操作: `python scripts/measure_test_mem.py aavm2_ -F test-vm-files` 重测
  全表更新；编译侧/执行侧占比结论（A5）写入。
  验证: 新旧表 diff 记录在 plan。
  [✅ 已完成] worktree commit `docs(test-infra): PLAN-565 T6`。
  复测命令按 574 后现实调整：`measure_test_mem.py <test> -F test-aavm
  --ignored`（aavm 系 568 起挂 test-aavm、574 起 Windows 本地 ignored；
  nextest --run-ignored 显式测）。受触集五测逐一复测（WorkingSet 法，
  与 564 同法可比）：**m1 806→809 / m2 813→806 / m3 807→792（XL→LG）/
  m4 815→796（XL→LG）/ 新增 rerun_consistency 813**；未触及行沿用 564
  值并在表头加注（代码未动 + 本地 ignore，CI/Linux 全量）。
  **表头结论（供 566 直接引用）**：单测峰值 ≈ 单次编译高水位
  （~790-815MB），L1 消除的是重复（时长/churn），**不降单编译峰值**；
  峰值本体 = 编译侧 AST/Node 分配结构（P0 归因：编译侧 churn 7.4GiB
  vs 执行侧 2.9MiB），压缩属 Plan 566 Value/Node 领地。m3/m4 落 LG
  边界（±2% 噪声带），nextest 组归属保守维持 XL（xl/lg 并发同为 1）。

## 复审记录

**复审人**：zhaopuming（ZCode 会话，2026-09-07）；**方式**：verify-don't-trust——
worktree `D:/autostack/.wt/lang-565/auto-lang`（branch `plan-565-dev`）内重跑全部验证。

### 门禁

- **cargo tf**（复审唯一全量门）：3468 测 **3467 绿 / 1 红 / 96 skipped**，22.2s。
  唯一红 `ui_gen::vue::tests::test_charts_gallery_compiles` —— **预存红**，已在
  主检出 master（5ed1e96df）复现同红归属（AGENTS.md taa 行"唯一余红=charts_gallery
  预存"在案）。565 默认构建不编译任何 aavm 改动（test-aavm 门控）、mem-profile
  关闭，与该红零交集。
- **cargo taa**（裸跑兜底，AGENTS.md aavm 测试基建触发条件）：3621 测 3620 绿 /
  1 红（同上预存）/ **608 skipped**——574 形态（607）+ 本计划新增
  rerun_consistency 入 Windows ignore 集，符合预期。

### 验收标准逐条（全部 pass）

- **A1 pass**：报告测试复跑 ok（5.03s），MEMPROFILE 五行输出完整
  （total/peak_live/live/allocs/frees/分桶）。无 feature 默认构建：cargo check
  168 warnings 与基线逐字一致（T1 期核），分支 Cargo.toml 增量仅 feature 声明
  4 行（`git diff 基点..HEAD -- Cargo.toml` 复核）。
- **A2 pass**：四闸门+回归 5/5 绿复跑（18.53s）：m1 5/m2 35/m3 8/m4 58 文件
  断言全绿；m2 批内含编译 <30s 达成（独跑 2.98s）。
- **A3 pass**：L3 数据结论完整记录（否决回退：峰值 0% 降幅/日常 -6.7%/构建
  2.29×），分支无 `[profile.test.package]` 残留（grep 复核）。
- **A4 pass**：`.config/test-mem-weights.md` 565 版入分支（commit 3aa546156），
  受触五测逐一实测（WorkingSet 法与 564 同法），新旧值 diff 记录在 T6 证据。
- **A5 pass**：报告测试复跑数值与执行期记录一致（compile 7419.7MiB/8.8M 次、
  peak_live 791864140B=755.2MiB 逐字节可复现）；归因结论（~99.96% 编译侧）
  已写入 T2/T6 证据区供 566 引用。

### 发现与处置

- **F1（缺陷，已修）**：`aavm2_corpus_runner.rs` 临时文件路径拼入 .at 字面量
  前**未转义**——.at lexer（Rust 侧 str()）对已识别转义序列 `\n \t \r \0`
  变换、未知透传；当前机器 TEMP 路径段 `\U \A \L \T` 恰全为未知序列故透传
  侥幸通过，但 `TEMP=C:\tmp`/用户名 tom 等环境 `\t` 段首会改写路径致
  File.read_text 读错文件、闸门破裂（CI/Linux 无反斜杠不受影响）。
  同文件家族既有先例（m4 use 腿对 main_path 做 escape_for_at_literal），
  runner 漏做。**处置**：复审补丁 commit `941720ded`（escape_at_path 反斜杠
  全量双写）+ 复验 5/5 绿——转义后所有路径反斜杠统一走 `\\`→`\` 规则，
  路径往返校验反而更全。已闭环，不遗留债。
- **遗漏扫描**：无——分支纯差异（基点 1c6753a92）恰为声明文件集
  （+610/-149，11 文件 + 补丁），全部 T1-T6 任务有对应 commit。
- **延后扫描**：T6"全表重测"落为**受触集实测 + 未触及行沿用 564 值加注**
  ——合理化：未触及测试代码未动、574 后 Windows 本地 ignored（nextest 默认
  不跑，串行全测需 25min+ 且测的是常量），受触集与 564 同法可比。记为
  复审注记，非债。
- **计划文本 vs 实现差异（良性，均已在执行期记录）**：①D1 分界采样落为
  `create_vm_from_source`/`run_task_loop` 精确边界（D1 字面"run_with_capture
  前后"无法分出编译/执行，意图忠实落地）；②T4 文件清单"tests/mod.rs"实为
  `tests.rs`（仓布局）；③A2"nextest 跑 aavm2_m2"实为显式 `--ignored`
  （574 Windows ignore，nextest 默认不选 ignored；Linux/CI 仍走 taa 全额）。

### Merge 注意

- **master 已前移两次**（010 fold 680b2c815 + 546 merge 5ed1e96df），分支基点
  1c6753a92。重叠面：`crates/auto-lang/src/lib.rs`（本侧 +10 行 cfg 块 vs
  010 侧 book/run_app 改动，不同区域，预期 git 自动并或微冲突）；
  `tests.rs` master 侧未动；m1-m4/tests 目录 master 侧仅 a2r_tests/
  book_listing_tests（无重叠）。
- worktree 组内含 auto-down 兄弟（detached @ a6d5ecf，未改动）——merge 清理
  时先 wt-guard 双查再摘。
- 复审后分支 tip：`941720ded`（5 commits）。

**结论**：A1-A5 全 pass、门禁绿（唯一红为 master 预存已归属）、缺陷 F1 已
闭环——**reviewed，可交 /auto-plan:merge**。

## 待澄清事项

- **Q1（L1 通道）**: .at 侧 env() native 是否存在决定注入通道（env vs 临时
  文件），T3 定；两者都不通则升级方案 b（引擎模块缓存 API，另行评估）。
  **已裁决（T3，2026-09-07）**：文件通道——`File.read_text`（nat#1000）纯 .at
  可直接调，探针 dump 逐字一致；方案 b 不需要。
- **Q2（L3 编译时长）**: opt-level=1 使测试编译变慢，日常档可接受阈值
  <2× 基线编译时长，超出则仅 tv/tf 档生效，T5 数据裁决。
  **已裁决（T5，2026-09-07）**：实测构建段 2.29× 超阈值，且峰值零降幅/
  日常档零收益——整体否决回退，不存在"仅 tv/tf 档生效"的中间形态。
- **Q3（时序）**: stacked 于 plan-532-dev tip（同 564）；若 532 在执行期
  fold，rebase 到 master。
  **已裁决（2026-09-07 用户）**：532 已 fold，aavm2 测试文件已在 master
  （实测核实 m1-m5/a2r/t3 齐全）；plan-565-dev 直接从 master tip
  （1c6753a92）开出，非 stacked。
