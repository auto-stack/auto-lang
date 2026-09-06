---
plan_id: PLAN-565
status: drafting               # drafting → executing → execution_done → reviewed → archived
feature_name: vm-mem-quickwins
author: [zhaopuming]
created_at: 2026-09-05
updated_at: 2026-09-05

# /auto-plan:review 结束时填写：
supersedes_spec_components: []
new_spec_components: []
touched_goals: []             # 引用 docs/specs/goals.md 的 GOAL-NNN

affects: [auto-lang/vm]       # 优化 VM 测试期内存足迹（P0 归因/L1 编译复用/L3 优化档）
current_step: 0
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

- [ ] A1: `cargo test -p auto-lang --lib --features mem-profile -- --ignored test_mem_profile_report --nocapture`
      产出含 total/peak_live/分桶的报告；无 feature 时构建与基线零差异。
- [ ] A2: m2 语料闸门独跑 <30s（基线 147s），35 文件断言全绿；
      m1/m3/m4 语料闸门同步切换且全绿。
- [ ] A3: L3 有数据结论（采纳配置入库或回退），对比数字记入本 plan。
- [ ] A4: `.config/test-mem-weights.md` 更新（565 后权重表 vs 564 基线），
      最重测试峰值变化量化。
- [ ] A5: P0 报告给出编译侧/执行侧占比结论，写入执行证据区
      （566 的装箱范围裁决直接引用）。

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
- **T2** 报告测试 + 编译/执行分界采样。
  文件: `crates/auto-lang/src/mem_profile.rs`（report() 辅助）+
  `crates/auto-lang/src/tests/`（挂 #[ignore] 报告测试，测试体内
  run_with_capture 前后采样）。
  操作: 按 D1 输出报告行。
  验证: `cargo test -p auto-lang --lib --features mem-profile -- --ignored test_mem_profile_report --nocapture`
  输出 MEMPROFILE 行。
- **T3** L1 现状重复成本实测 + 通道选型。
  操作: worktree 内一次性探针（不入库）：同程序串 `run_with_capture` 连跑
  3 次记录耗时；查 .at env()/read_file native 可用性定注入通道。
  验证: 三次耗时 + 通道结论记入本 plan 执行证据区。
- **T4** L1 helper 落地 + 四闸门切换。
  文件: `crates/auto-lang/src/tests/aavm2_corpus_runner.rs`（新）+
  `aavm2_m1.rs`/`aavm2_m2.rs`/`aavm2_m3.rs`/`aavm2_m4.rs`（corpus 循环
  改走 helper）+ `tests/mod.rs` 挂模块。
  操作: 按 D2 实现；判据断言原样保留。
  验证: nextest 跑 `aavm2_m2` 全绿；独跑 m2 时长 <30s（对照 A2）。
- **T5** L3 实验。
  文件: workspace 根 `Cargo.toml`（[profile.test.package] 两段）。
  操作: 按 D3 规程测权重表 + cargo t 基线；达标保留/不达标回退。
  验证: 前后峰值与时长对比数据记入本 plan 执行证据区。
- **T6** 权重表更新 + 收口。
  文件: `.config/test-mem-weights.md`、本 plan（执行证据）。
  操作: `python scripts/measure_test_mem.py aavm2_ -F test-vm-files` 重测
  全表更新；编译侧/执行侧占比结论（A5）写入。
  验证: 新旧表 diff 记录在 plan。

## 复审记录

## 待澄清事项

- **Q1（L1 通道）**: .at 侧 env() native 是否存在决定注入通道（env vs 临时
  文件），T3 定；两者都不通则升级方案 b（引擎模块缓存 API，另行评估）。
- **Q2（L3 编译时长）**: opt-level=1 使测试编译变慢，日常档可接受阈值
  <2× 基线编译时长，超出则仅 tv/tf 档生效，T5 数据裁决。
- **Q3（时序）**: stacked 于 plan-532-dev tip（同 564）；若 532 在执行期
  fold，rebase 到 master。
