---
plan_id: PLAN-568
status: executing               # drafting → executing → execution_done → reviewed → archived
feature_name: aavm-aa2r-test-tier
author: [zhaopuming]
created_at: 2026-09-05
updated_at: 2026-09-05

# /auto-plan:review 结束时填写：
supersedes_spec_components: []
new_spec_components: []       # 无 module spec 新增：知识沉淀于 AGENTS.md 测试档表（Heavy-Mem Tiering 节邻位）
touched_goals: ["GOAL-016: 构建与测试基础设施——AAVM/AA2R 测试域独立成档（tv 拆耦），反射性 tv 不再触发自举重测试"]

affects: ["crates/auto-lang/Cargo.toml", "crates/auto-lang/src/tests.rs", "crates/auto-lang/src/tests/vm_file_tests.rs", "crates/auto-lang/src/tests/aavm_runner_tests.rs(新)", ".cargo/config.toml", ".github/workflows/vm-files-ci.yml", "AGENTS.md", "docs/plans/KNOWN-DEBT-AND-RISKS.md"]
current_step: 5
total_steps: 8
---

# [PLAN-568] AAVM/AA2R 测试独立成档（cargo tv 拆耦自举重测试）

## 变更摘要

AAVM 自举（auto/lib/*.at，~504KB Auto 版编译器）与 AA2R（a2r.at 发射对齐）
相关测试目前挂在 `test-vm-files` feature 下，随 `cargo tv` 运行。**概念纠偏
（用户裁定，2026-09-05）："改 VM/编译器后的回归测试"与"AAVM 自举展示"不是
同一概念**——AAVM 目前仅用于展示 VM 能力（秀肌肉），无实用面，平时修改
VM/编译器**不需要关心 aavm 是否被改坏**（守护面=CI push/PR + 全量档）；
aavm 测试的触发条件独立：**只有改了 aavm 自己的代码才需要测 aavm，且按
改动位置缩小作用域，不全量跑**。多个 agent 改完编译器后反射性调用
`cargo tv`，每次都被这批三重解释/现场 cargo build 的测试拖住（本机实测
21 测 Σ单测 ≈1163s、20 路并行墙钟 4m24s、单测峰值内存 800MB~1.2GB——
多 agent 并发即资源耗尽）。本 plan 把 AAVM/AA2R 全系测试**按域**拆到独立
feature `test-aavm` + 独立别名 `cargo taa`：

- `cargo tv` 回归**纯 .at 语料 golden 档**（vm_file_tests/cookbook/conformance）
  ——名实相符地对应"改 VM/编译器后的回归"场景，编译期即不含任何 aavm
  代码，agent 再怎么反射性调用也拖不慢系统；
- `cargo taa` = AAVM/AA2R 专属档（full 配置：带 564 组限流、含 XL、排 T3
  塔），**别名不带固定滤串**（同 `cargo t <module>` 习惯用法）：裸
  `cargo taa` = aavm 全集兜底；`cargo taa aavm2_m5` = 追加滤串缩小作用域，
  只跑相关闸门（触发条件与作用域映射见 D6）；
- `cargo ta`/`t3` 追加 `test-aavm` feature（全量门禁语义不变）；
- CI `vm-files-ci.yml` 的 aavm2 六闸门步骤同步换 feature。

与 Plan 564（重内存分层，按**内存**分级限流）互补：564 治"跑的时候别爆"，
本 plan 治"平时根本不该跑"。564 合入后其日常档 default-filter 对 aavm XL 的
排除行变为冗余保险（编译期已不可见），保留不动。

## 目标

- G1: `cargo tv` 不再编译、不再运行任何 AAVM/AA2R 测试（`nextest list` 零命中）。
- G2: AAVM/AA2R 测试可经 `cargo taa` 一键独立运行（名单=现 tv 内 21 测 +
  532 分支增量），全绿，且受 nextest 组限流（树峰值 ≤2GB，564 预算口径）。
- G3: `cargo t` 日常档同步剥离 aavm（含当前**无 feature 门**、每天在跑的
  `aavm2_m1`，实测 31s/次）。
- G4: 全量门禁不缩水：`cargo ta`（全量）与 `t3`（里程碑）feature 集合追加
  `test-aavm`；CI aavm2 六闸门步骤换用新 feature 后覆盖不变。
- G5: 文档口径更新（AGENTS.md 档表 + 改码指引：动 auto/lib / aavm2 →
  `cargo taa`；动编译器 → `cargo tv` 纯语料档即可）。
- G6: aavm 测试**触发条件与作用域**成文（AGENTS.md）：非 aavm 改动零触发；
  aavm 改动按 D6 映射缩小作用域（`cargo taa <滤串>`），review/fold 前全量
  `taa` 兜底；顺带收口全档资源表（测试数/耗时/内存，含 tt/tb/th/ta 补测）。

## 架构方案

不动编译器/VM/语言核心，只动**测试基建五层**：

```
层1 feature   crates/auto-lang/Cargo.toml
              test-aavm = ["test-vm-files"]   （implies：aavm runner 复用
              vm_file_tests 的语料缓存 loader，杜绝半开feature的坏态）
层2 模块门    crates/auto-lang/src/tests.rs
              aavm2_a2r / aavm_at_mode_tests / aavm2_m2..m5 /
              aavm2_repro_242：test-vm-files → test-aavm
              aavm2_m1（原无门，日常档在跑）：加 test-aavm 门
层3 文件搬移  vm_file_tests.rs 内嵌 aavm 内容 → tests/aavm_runner_tests.rs(新)
              （test_aavm v1 runner / test_aavm2 / test_aavm2_compile /
               build_aavm_rust_bin[现场 transpile+shell cargo build] /
               001_smoke·compile_corpus·compile_use_corpus 三测 /
               99_bootstrap v1 ~100 行 ignored 一行测试块）
              vm_file_tests.rs 从此名实相符=纯 .at 语料 golden 套件
层4 别名      .cargo/config.toml
              taa = nextest run -p auto-lang --lib --features test-aavm
                    --config-file .config/nextest-full.toml
              （无固定尾滤串，同 cargo t <module> 习惯用法：裸跑=aavm 全集
               兜底；追加滤串=作用域缩小，如 cargo taa aavm2_m5 只跑 M5；
               full 配置=564 组限流+含 XL+排 T3 塔）
              ta/t3 feature 列表 + test-aavm；tv 字符串不动（语义自动变轻）
层5 守护面    .github/workflows/vm-files-ci.yml aavm2 六闸门步骤
              --features test-vm-files → test-aavm（implies 自带）；
              AGENTS.md 档表 + Category B 指引 + Heavy-Mem 节补测法注
层6 指引层    AGENTS.md 触发条件与作用域映射（D6）+ 全档资源表
              （测试数/耗时/内存，现状散落 plan 文档与 config 注释，
               本次一并收口进档表）
```

**与 532/564 的叠放约束**：564 已在 aavm 测试文件里接线 heavy_gate 守门行、
且 532 分支还有未 fold 的 aavm2 增量测试（static_diff/t3 塔等）→ 本 plan
worktree 从 **plan-564-dev tip** 派生（stacked，564 同款先例），搬移时逐行
保留 heavy_gate 接线；fold 排队 **532 → 564 → 568**。主检出存在 532 未提交
改动（.cargo/config.toml/.config/nextest*/AGENTS.md，564 Q5 在案），本 plan
的计划文件提交须分离暂存（仅收本 plan + .next-id，先例 79f85bbd5）。

## 需求分析与背景调查

（取材 .autoos/specs.json overview + 本仓测试基建现状，2026-09-05 本机实测）

- **用户痛点**：AGENTS.md 已明示"全量回归放计划最后 review 再测"，但多个
  agent 每个 phase 完都反射性跑 `cargo tv`；AAVM 自举新加的慢测试随之每次
  全量触发，且多 agent 并发时系统资源被耗尽。AAVM 自举在真正切换 Auto 版
  编译器前是"秀肌肉"功能，日常开发用不到。
- **tv 档 aavm 实测清单**（`nextest list -E 'test(aavm) or test(aa2r)'` =
  21 测，逐测计时 nextest 20 路并行，2026-09-05）：
  m5_engine_corpus 263s / m4_codegen_corpus 262s / a2r_is_corpus 180s /
  m2_parser_corpus 172s / m5_use_corpus 53s / m4_use_corpus 53s /
  m3_typeinfo_corpus 47s / **compile_corpus 38s 与 compile_use_corpus 36s
  （测试体内现场 transpile 整个 auto/lib 并 shell 出去 cargo build！）** /
  m1_lexer_corpus 31s / m5_use_errors 21s / milestone_fib 16s /
  001_smoke 6.4s / goldens_check 4.6s / main_dump 4.1s / 其余 6 测 <1s。
  Σ≈1163s，墙钟 4m24s。tv 总量 3593 测（564 复审口径 95.9s 系 XL 已排除+
  LG 限流后的值；master 裸跑即上述 ~4.5 分钟形态）。
- **内存实证**（564 权重表，532 分支超集口径）：aavm2 系 12 个 XL
  （802–1232MB）+ 4 个 LG（744–797MB）——并发 tv 的资源耗尽主因。
- **m1 漏门**：tests.rs:71 `mod aavm2_m1;` 无 cfg（其余 aavm2 模块均有
  test-vm-files 门）——日常 `cargo t` 每天在跑它（全 lib 走一遍管线 ×5 语料）。
- **vm_file_tests.rs 名实不符**：文件头自称"reads .at files from test/vm/"
  但内嵌 aavm v1/v2 runner、build_aavm_rust_bin（含 prelude/harness 大字符串）
  与 3 个非 ignore aavm2 测试。
- **CI 依赖**：vm-files-ci.yml（430 复审补网）aavm2 六闸门步骤
  `--features test-vm-files -- test_aavm2 --test-threads=1`——滤串是 fn 名
  子串，模块搬移后仍命中；其余三步（goldens/ffi_dual/conformance/cookbook）
  与 aavm 无关，保持 test-vm-files。
- **564 交互**：564 的 nextest 日常档 XL 名单排除行、overrides 归属滤串均用
  fn 名（子串匹配），模块路径迁移不影响；heavy_gate 接线在 6 个 aavm 测试
  文件 + vm_file_tests.rs 内（本 plan 搬移须携带）。
- **命名占据**：`ta` 已被全量档占用 → 新档名 `taa`（AA=AAVM/AA2R）。
- **仓库范式**：test-trans/test-book/test-vm-files 三个 feature 门是既有
  idiom（Plan 289）；本 plan 是同范式第四门，非新发明。

## 详细设计

### D1: feature 定义（crates/auto-lang/Cargo.toml）

```toml
# Plan 568: AAVM/AA2R 自举测试独立档（implies test-vm-files：runner 复用
# vm_file_tests 语料缓存 loader）。cargo tv 从此不含 aavm；专属档 cargo taa。
test-aavm = ["test-vm-files"]
```

### D2: 模块门翻转（crates/auto-lang/src/tests.rs）

- `aavm2_a2r` / `aavm_at_mode_tests` / `aavm2_m2` / `aavm2_m3` / `aavm2_m4` /
  `aavm2_m5` / `aavm2_repro_242`：`#[cfg(feature = "test-vm-files")]` →
  `#[cfg(feature = "test-aavm")]`（注释行同步 Plan 568）。
- `aavm2_m1`：新增 `#[cfg(feature = "test-aavm")]`（离开日常档——31s/次的
  lexer parity 早警转为 CI+taa 守护，覆盖差登记 KNOWN-DEBT）。
- 新增 `#[cfg(feature = "test-aavm")] mod aavm_runner_tests;`（D3 搬移目标）。

### D3: runner 搬移（vm_file_tests.rs → tests/aavm_runner_tests.rs）

搬移内容（**逐行保留 564 heavy_gate 接线**，532 分支增量文件不在此分支、
fold 后自然带上）：
- runner：`test_aavm`（v1）、`test_aavm2`（v2）、`test_aavm2_compile`；
- `build_aavm_rust_bin()`（prelude/harness 字符串、staging 原子发布逻辑整体）；
- 非 ignore 三测：`test_aavm2_001_smoke` / `test_aavm2_compile_corpus` /
  `test_aavm2_compile_use_corpus`；
- ignored：`test_aavm2_002_hello_compile` + 99_bootstrap v1 一行测试块
  （~100 行，`#[ignore]` 原样）。

新文件头注释注明来源与门语义；vm_file_tests.rs 删除上述内容后仅剩 .at 语料
golden 套件与共享 loader（loader 保持 test-vm-files 门内，taa 经 implies 可见）。
测试 ID 变化：`tests::vm_file_tests::test_aavm2_*` → `tests::aavm_runner_tests::
test_aavm2_*`——CI 滤串/564 权重表滤串均 fn 名子串，不受影响（T6 复核）。

### D4: 别名接线（.cargo/config.toml）

```toml
# Plan 568: AAVM/AA2R 自举档——auto/lib+aavm2/aa2r 全系（implies vm-files；
# full 配置=564 组限流+含 XL+排 T3 塔）。仅当改动触及 aavm 代码时使用
# （触发条件与作用域映射见 AGENTS.md）；cargo tv 已不含 aavm（纯 .at 语料
# golden），反射性 tv 不再触发自举重测试。裸跑=aavm 全集；追加滤串缩小
# 作用域（同 cargo t <module> 习惯用法），如 cargo taa aavm2_m5 只跑 M5。
taa = "nextest run -p auto-lang --lib --features test-aavm --config-file .config/nextest-full.toml"
ta = "nextest run -p auto-lang --lib --features test-aavm,test-trans,test-book --config-file .config/nextest-full.toml"
t3 = "nextest run -p auto-lang --lib --features test-aavm,test-trans,test-book --config-file .config/nextest-t3.toml"
# tv/tt/tb/tf/th 不动（tv 语义自动变轻；tf 本就不含 feature 档，复审清单加 taa）
```

（头注 usage 块同步补 `taa` 行与 `tv` 语义变化说明。设计取舍：别名**不带**
固定尾滤串——若带 `aavm` 尾串，追加 `aavm2_m5` 会变成两者 OR（nextest 语义）
反而放大到全集，作用域缩小就废了；裸跑附带日常面 ~46s，相对 aavm 分钟级
可忽略，换取滤串追加的习惯一致性。）

### D5: CI 与文档

- vm-files-ci.yml 步骤"aavm2 six gates"：`--features test-vm-files` →
  `--features test-aavm`；文件头注释补 Plan 568 拆档说明。其余步骤不动。
- AGENTS.md：
  - 别名参考表加 `taa` 行、`tv` 行语义改注（"改 VM/编译器后的 .at 语料
    golden 回归，不含 aavm"）；
  - Category B 指引补两行："动 auto/lib / aavm2 / AA2R → `cargo taa`
    （aavm 专属档，按 D6 作用域缩小）；动编译器/VM → `cargo tv`（纯语料
    档）——aavm 无实用面，非 aavm 改动**不需要**跑 aavm 测试"；
  - Heavy-Mem Tiering 节补一句 "aavm 系复测命令 `-F test-aavm`"；
  - **全档资源表收口**：别名表扩为"档位 | 场景 | 测试数 | 耗时 | 内存"
    （已知数据先行：t=4304/~46s、tf=3441/77.2s、tv 拆档后待实测、tb 69 测
    ×5-7s；tt/th/ta 耗时 T7 补测后填入；同步修正 `t` 的过时"~3200"口径）。
- KNOWN-DEBT-AND-RISKS.md：登记覆盖差两条（①tv/t 日常面不再含任何 aavm
  闸门，守护移至 CI push/PR + taa + ta；②m1 lexer parity 离开日常档）。

### D6: aavm 测试触发条件与作用域映射（核心新增，成文入 AGENTS.md）

**触发条件（什么时候才需要测 aavm）**——diff 触及以下路径时才触发，其余
（VM/编译器/stdlib/examples/ui 等）**零触发**：

```
auto/lib/*.at            aavm2 主体（Auto 版编译器 + a2r.at）
test/vm/aavm2/**         aavm2 语料与用例（corpus_m1..m4/corpus_use/corpus_a2r/99_unit…）
parity/**                aavm2 生成脚本（gen-aavm2-unit.py 等）
aavm2 专属基建            src/tests/aavm2_*.rs、aavm_runner_tests.rs、
                         lib.rs 的 aavm2_lib_source/AUTO_LIB_FILES*
```

改 VM/编译器导致 aavm 破坏的兜底 = CI push/PR（vm-files-ci）+ `ta`/`t3`
全量档 + fold 前 `taa` 全量，**不进日常/反射性档**。

**作用域映射（改了 aavm 的什么 → 跑哪个闸门）**——管线结构决定：上游共享
文件级联全链，终段文件与语料可精确缩小。耗时为 2026-09-05 master 实测
（供选择参考；fold/review 前一律全量 `taa` 兜底）：

| 改动位置 | 跑什么 | 实测耗时 |
|---|---|---|
| `corpus_m1/**` | `cargo taa aavm2_m1` | 31s |
| `corpus_m2/**` | `cargo taa aavm2_m2` | 172s |
| `corpus_m3/**` | `cargo taa aavm2_m3` | 47s |
| `corpus_m4/**` | `cargo taa aavm2_m4` | ~315s |
| `corpus_use/**` | `cargo taa aavm2_m4 aavm2_m5` | ~127s（compile_use 腿按需） |
| `corpus_a2r/**` | `cargo taa aavm2_a2r` | ~185s |
| `auto/lib/engine.at`（终段：执行器） | `cargo taa aavm2_m5` | ~350s |
| `auto/lib/a2r.at`（终段：发射器） | `cargo taa aavm2_a2r aavm_at_mode` + compile 腿 | ~260s |
| `auto/lib/{token,lexer,parser,typeinfo,codegen}.at`（上游共享） | 全管线级联，直接裸 `cargo taa` | ~10min 量级 |

（闸门粒度=测试目录级：单语料文件的新增/修改由所属闸门整体覆盖，不做
单文件粒度——闸门本体就是逐文件遍历断言。compile 腿
`compile_corpus`/`compile_use_corpus` 含现场 cargo build，产物按内容 hash
缓存，二次运行为秒级。）

## 测试设计

- 门隔离：`cargo nextest list -p auto-lang --lib --features test-vm-files
  -E 'test(aavm) or test(aa2r)'` 须 0 行；`cargo nextest list -p auto-lang
  --lib`（日常）不含 m1。
- 档完整性与作用域：裸 `cargo nextest list -p auto-lang --lib --features
  test-aavm` 名单含 aavm 全集（≥21 测 + 日常面）；`cargo taa aavm2_m5`
  实跑只出 M5 四测（作用域缩小实证——追加滤串不放大到全集）。
- 运行：`cargo tv` 全绿；裸 `cargo taa` 全绿（墙钟预期 ~10 分钟量级）；
  `cargo t` 基线不回归（564 Q6 预存红在案，非本 plan 归因）。
- CI：yaml 解析校验 + 六闸门滤串 `test_aavm2` 对新名单逐一命中。
- 计时对比：tv 搬移前后墙钟 + tt/tb/th/ta 补测（全档资源表数据源）记入
  本文件与 AGENTS.md。

## 验收标准

- [ ] A1: tv 零 aavm——`nextest list --features test-vm-files -E 'test(aavm)
      or test(aa2r)'` 输出 0 行；`cargo tv` 全绿。
- [ ] A2: 裸 `cargo taa` 全绿（名单含 aavm 全集 21 测 + 日常面），nextest
      full 档组限流生效（`show-config test-groups` 三组非零）。
- [ ] A3: `cargo t` 日常档 m1 消失（计数 -1），基线无新增红。
- [ ] A4: `cargo ta` feature 集编译通过且 list 含 aavm 全集（全量门禁不缩水）。
- [ ] A5: CI vm-files-ci.yml 四步骤 feature/滤串核对无误（yaml 解析过 +
      滤串对名单复核），aavm2 步骤换 test-aavm。
- [ ] A6: AGENTS.md（档表/Category B/Heavy-Mem 节/D6 触发条件与作用域
      映射/全档资源表）与 KNOWN-DEBT 覆盖差两条登记完成。
- [ ] A7: tv 前后墙钟实测对比 + tt/tb/th/ta 补测耗时记入复审记录区与
      AGENTS.md 资源表。
- [ ] A8: 作用域缩小实证——`cargo taa aavm2_m5` 只运行 M5 闸门测试
      （不放大到 aavm 全集）。

## 执行步骤
（原子任务：精确文件路径 + 确切操作 + 验证命令；每步完成后追加 [✅ 已完成] 一行证据）

- **T1** 建 worktree（stacked 于 plan-564-dev tip，564 D5 同款偏离先例）。
  操作: `git worktree add D:/autostack/.wt/lang-568/auto-lang -b plan-568-dev plan-564-dev`
  （532/564 未 fold 前不可改从 master 建——aavm 测试文件与 heavy_gate 接线
  在 564 分支；fold 序 532→564→568）。
  验证: `git -C D:/autostack/.wt/lang-568/auto-lang log --oneline -1` = 564 tip。
  [✅ 已完成] 2026-09-06 worktree D:/autostack/.wt/lang-568/auto-lang 建于
  plan-564-dev tip 3f0d21c72（log 验证一致）；组内补只读 detached 兄弟
  auto-down@1b3e4bc（564 同款先例，autodown-core path 依赖解析用）。
- **T2** feature 定义。
  文件: `crates/auto-lang/Cargo.toml`。操作: 按 D1 在 test-book 行后加
  `test-aavm = ["test-vm-files"]` + 注释。
  验证: `cargo check -p auto-lang --features test-aavm` 零错。
  [✅ 已完成] 2026-09-06 feature `test-aavm = ["test-vm-files"]` 入
  crates/auto-lang/Cargo.toml；check 零错（注：本机 sccache 抖动一次
  STATUS_ACCESS_VIOLATION，后续构建统一 RUSTC_WRAPPER= 绕开）。
- **T3** 模块门翻转（tests.rs）。
  文件: `crates/auto-lang/src/tests.rs`。操作: 按 D2 翻 7 个模块门 + m1 加门
  + 注册 aavm_runner_tests。
  验证: `cargo nextest list -p auto-lang --lib --features test-vm-files
  -E 'test(aavm) or test(aa2r)'` = 0 行；`cargo check -p auto-lang` 零错。
  [✅ 已完成] 2026-09-06 九模块门翻转（m1 原无门入档、m2-m5/a2r/at_mode/
  t3塔/repro）；check 零错；模块级清零（tv 集残 3 = vm_file_tests 内嵌腿，
  T4 搬移后归零——见 T4 证据）。
- **T4** runner 搬移。
  文件: `crates/auto-lang/src/tests/vm_file_tests.rs`（删）、
  `crates/auto-lang/src/tests/aavm_runner_tests.rs`（新）。操作: 按 D3 搬移，
  heavy_gate 接线逐行保留。
  验证: `cargo check -p auto-lang --features test-aavm` 零错；
  `cargo nextest list -p auto-lang --lib --features test-aavm aavm` 名单 ≥21。
  [✅ 已完成] 2026-09-06 vm_file_tests.rs -589 行 → tests/aavm_runner_tests.rs
  （v1 runner/v2 基建/compile 腿/build_aavm_rust_bin/001_smoke 等三测/
  100 行 ignored v1 一行测/build_aavm_rust_bin_pub；564 heavy_gate 三处
  接线随迁）；VmTestData+get_cached_test 升 pub(crate)；aavm2_a2r.rs 调用
  点改指。验证：check 零错；tv 集 aavm=0 行；full 档 aavm 名单 **22 测**
  （含迁入的 aavm_runner_tests 三测；对齐 564 "tf aavm_ 22/22" 口径）。
  > 执行注记（t3 别名协调）: 564 tip 分支上无 t3 别名与 nextest-t3.toml
  > （532 主检出未提交态，564 Q5 在案同因）——本分支不代笔；532→564→568
  > 全部 fold 后须在 master 把 t3 别名 feature 列表改 test-aavm（一行，
  > T7 登记 KNOWN-DEBT 转告 merge 会话）。
- **T5** 别名 + CI + 文档。
  文件: `.cargo/config.toml`、`.github/workflows/vm-files-ci.yml`、
  `AGENTS.md`。操作: 按 D4/D5——含 D6 触发条件与作用域映射表成文入
  AGENTS.md、全档资源表骨架（已知数据先行）。
  验证: `cargo taa`（裸跑，全绿）；`cargo taa aavm2_m5` 只出 M5 测；
  `grep -n taa .cargo/config.toml AGENTS.md`；yaml 解析（python
  yaml.safe_load）通过。
  [✅ 已完成] 2026-09-06 分支侧：配置/CI/AGENTS.md 落地并提交；
  `cargo taa aavm2_m5` 实测只跑 M5 四测（250s，4194 skipped）——A8 作用域
  缩小成立；裸 `cargo taa` 2775/3603 时被唯一红 test_charts_gallery_compiles
  （P555-D4 在案存量红）fail-fast 截断，aavm 段已跑部分全绿（compile 腿
  17.3s 证明内容寻址缓存命中、a2r_is_corpus 78s）。
  > **提前落地（2026-09-06 用户裁定）**：因多 agent 反射性 `cargo tv` 持续
  > 被拖，T2-T5 功能改动**剥离 heavy_gate 依赖后提前合入 master**（提交
  > c825e989f，master 验证：tv 集 aavm=0 / 日常档 3425（m1 -1）/ taa 档
  > 21 测 / `cargo taa repro_242` 冒烟 2/2 绿）。532 主检出未提交改动
  > （t3 别名等）经 blob 分离暂存未卷入；工作区 t3 别名 feature 已同步改
  > test-aavm（留 532 会话收口）。**fold 队列更新**：master 已含 568 核心，
  > 532/564 后续 fold 时其 aavm 测试文件改动（heavy_gate 接线/塔模块）须
  > 向 master 的 aavm_runner_tests.rs 新位置移植 reconciling——本 plan
  > worktree 分支保留 gate 版本作参照，fold 会话用。T6/T7 验证与文档回填
  > 改以 master 为准执行。
- **T6** 档验证矩阵。
  操作: ①`cargo tv` 全绿计时（拆档后）；②`cargo nextest list --features
  test-aavm,test-trans,test-book`（ta 集）含 aavm 全集；③`cargo t` 计数-1、
  无新增红（预存红 Q6 在案基线对照）。
  验证: 三项输出记入本文件证据行。
- **T7** 资源表补测 + 覆盖差登记收口。
  文件: `docs/plans/KNOWN-DEBT-AND-RISKS.md`、`AGENTS.md`、本文件。
  操作: 补测 `cargo tt`/`tb`/`th`/`ta` 墙钟（全档资源表缺口数据）填入
  AGENTS.md；登记两条覆盖差；tv 前后墙钟对比写入复审记录区。
  验证: `grep -n "568" docs/plans/KNOWN-DEBT-AND-RISKS.md` 命中；
  AGENTS.md 资源表无"未测"空洞。
- **T8** 收口自检（worktree 内零 warning 增量、无 debug 残留、格式
  `cargo fmt --check` 于触达文件），status → execution_done。

## 复审记录

## 待澄清事项

- **Q1（档名与交互）**: `taa`（AA=AAVM/AA2R）为默认；备选 `tva`/`tg`。
  `ta` 已被全量档占用（用户原话"比如 cargo ta"不可行，取最近邻 taa）。
  交互设计已按用户裁定定型（D4/D6）：**无固定尾滤串**——裸跑=全集兜底，
  追加滤串=作用域缩小（只测改动相关闸门），非 aavm 改动零触发。
- **Q2（m1 去留）**: 默认随迁（用户口径"所有 AAVM 相关测试"）；代价=日常档
  失去 lexer token 流 parity 早警（31s/次），守护转 CI+taa。若要留日常档，
  T3 中 m1 改挂 `any(feature = "test-aavm", feature = "test-vm-files")`
  即可——但那会让 tv 又含 31s 重测，与 G1 抵触，不推荐。
- **Q3（烟雾金丝雀）**: `test_aavm2_001_smoke`（6.4s）是否例外留在 tv 作
  "auto/lib 没烂透"金丝雀？默认**不留**（口径纯粹性优先，tv 编译期零 aavm
  是本 plan 的核心承诺；CI push/PR 仍是快速网）。
