---
plan_id: PLAN-531
status: executing              # drafting → executing → execution_done → reviewed → archived
feature_name: aavm-debt-clearance-batch
author: [zhaopuming]
created_at: 2026-09-03
updated_at: 2026-09-03

# /auto-plan:review 结束时填写：
supersedes_spec_components: []
new_spec_components: []
touched_goals: [GOAL-017]     # 自举（债务清欠,塔顶验证链前置）

affects: [aavm, auto-lang/trans]
current_step: 7
total_steps: 11
---

# [PLAN-531] aavm 债务清偿批：tt 真缺陷 / a2r 模式两洞 / pac target / May 发射

## 变更摘要

打包清偿 523/525 收口挂账的存量债务（P523-1/2/3 + P525-4，及可选项
P525-3 `??`），**为塔顶自举（Plan 532）清障**——其中 P523-2② 转译版
struct 字段表洞直接位于塔顶验证链上：

- **P523-3（核心,含两件真缺陷）**：tt 档（a2r test-trans 金样）28 件
  存量过期——批量 bless + **两件真发射缺陷修复**（pointer/004：cast 后
  方法调用；arc_dyn_spec/008：str as usize——文本金样长期掩盖）+
  tt 纳入复审清单（防再烂）；
- **P523-2（塔顶路径障碍）**：aavm.at a2r 模式入口两洞——①宿主
  `argv.get(1)` 发射 `Vec::get` Option 形态缺自动解包（E0308）；②转译
  版 VM 运行期 struct 字段表（`c.field_idx` 动态查表在转译宿主的语料内
  struct 形态未覆盖,b34 报 `no field x in Point`;VM 解释路径不受影响）；
- **P523-1**：pac 最小工程 rust target 生成缺口（CargoBuilder "no rust
  target found"）——pac.rs 语义补齐（api-example 前端存量红清偿视成本
  裁定归属,见待澄清③）;
- **P525-4**：宿主 May 裸值 return 发射修复（`fn f() ?int { return n*10 }`
  主 a2r 发裸值于 Option<T> fn——E0308;修后补裸值语料）;
- **可选**：P525-3 `??` NullCoalesce 实现（已入 Pratt 表,缺 codegen 臂;
  便宜则顺做）。

**不纳入**（裁定留档）：P525-1 主 a2r merge 后处理 AST 级根治（lib 已
规避,超小计划范围,塔顶后有需要再立）；P525-2 闭包 m4 对拍（已裁定迁位
corpus_a2r,判据面=发射闸+四路）；P525-5 塔顶稳定性（归 Plan 532 W0）。

**与 532 的关系**：532（塔顶自举）**硬前置 = 本计划 archived**——
P523-2② 洞不清,塔顶的转译版验证链不干净。

## 目标

1. tt 档 28 件 bless 后全绿；两件真缺陷修复有回归锚；tt 入复审清单
   （计划复审节+tf 可见性裁定）。
2. `auto/aavm.at` a2r 模式两洞清偿：位置参数形态经主 a2r 转译构建后
   **b07→55 实测**；转译版 struct 语料（b34）运行绿。
3. pac 最小工程 rust target 生成链补齐（冒烟脚本走 pac 正路而非旁路）。
4. 宿主 May 裸值 return 发射修复 + g34 补裸值语料。
5. （可选）`??` 操作符 aavm 四层实现 + 语料。
6. 全程 525 终态保护网不破绿。

### 非目标（Out of Scope）

- 塔顶自举本体（Plan 532）；api-example 全管线前端 strict 红清偿
  （视待澄清③裁定,缺省独立处置）;生成器 yield（P525-3 裁定延后不动）；
  P525-1/P525-2（上文裁定留档）。

## 架构方案

```text
W0 考古                 W1 tt 档清偿            W2 a2r 两洞            W3 pac+May+收尾
──────────            ──────────────          ─────────────          ───────────────
两真缺陷根因 →   批量 bless+修复锚定 →   argv 解包+字段表 →   pac target 链 →
tt 复审可见性 →   tt 入清单            →   塔顶障碍清除      →   May 裸值+(??) → 复审
bless 流程                                        aavm.at 全链实测        归档
```

## 需求分析与背景调查

（取材：KNOWN-DEBT P523-1/2/3、P525-3/4 行、divergences.md
D-a2r-mode-entry（:485）、`scripts/aavm_build_smoke.sh`、tt 档现状）

### 基线（开工时复测留档）

525 reviewed/归档中时点：四路 30/30、tv 3559、tt 28（存量红）、tf 2 红
（schema_drift=523 存量注记+docs_gen=528 面）。

### 逐债要点

| 债 | 现状证据 | 修复位预估 |
|---|---|---|
| P523-3 真缺陷①（004 pointer） | cast 后方法调用发射错（文本金样掩盖） | trans/rust.rs cast 接收者链 |
| P523-3 真缺陷②（008 arc_dyn_spec） | str as usize 缺 | trans/rust.rs 后处理强转位 |
| P523-2① argv.get 解包 | E0308,产品位文本垫片实测通过（临时） | trans/rust.rs Vec::get→索引/解包发射 |
| P523-2② 转译版 struct 字段表 | b34 RUNTIME-ERROR no field x in Point（转译宿主语料内 struct 未入字段表;VM 解释路径绿） | 转译侧 typeinfo 对齐（主 a2r 语料内 type 的字段注册缺失） |
| P523-1 pac target | pac.rs targets 无 rust App/Lib 项 | pac.rs/pac 最小工程 target 生成链 |
| P525-4 May 裸值 return | E0308（语料规避中） | trans/rust.rs return 位 Option 包裹 |
| P525-3 `??` | Pratt 表已有,codegen 缺臂 | codegen.at + 语料 |

### 风险与对策

| 风险 | 对策 |
|---|---|
| 两真缺陷修复波及面（merge 后处理敏感 pass） | 逐缺陷独立提交+金样 diff 评审;525-1 同族敏感性注记参照 |
| bless 掩盖真差异（bless 本身无判别力） | 先修真缺陷再 bless;bless diff 逐件评审（28 件清单留档） |
| 转译版字段表洞根因在宿主深层（525-1 同片区） | W0 根因定位先行;若超小修量级→登记转 532 W0 处置（逃生舱,待澄清②） |
| api-example 前端红范围失控 | 缺省只做 pac target 链,前端红独立处置（待澄清③） |

## 详细设计

### W0 考古（master 纯文档+探针）

1. 两真缺陷最小复现+根因定位（rustc 报文+发射文本对照）;tt 复审可见性
   方案（复审清单条目+tf 别名可见性——`.cargo/config.toml` tt 不在 tf
   档,裁定是否入）;bless 流程（复用 523 `--bless` 基建）。
2. P523-2 两洞根因定位（①：宿主 argv.get 发射对照;②：转译侧 typeinfo
   字段注册路径——以 b34 转译构建运行复现,定位在主 a2r type 发射或
   运行时表桥接）。

### W1 tt 档清偿（worktree）

3. 真缺陷①②修复（各自独立提交+新金样锚）。
4. 28 件批量 bless（diff 逐件清单评审留档）+ tt 入复审清单。

### W2 a2r 模式两洞（worktree 续）

5. argv.get 解包发射修复 → aavm.at 位置参数形态转译构建全链实测
   （b07→55）;垫片撤除。
6. 转译版 struct 字段表修复 → b34 转译构建运行绿（10/20）;若根因超
   量级按待澄清②转 532。

### W3 pac+May+收尾（worktree 续）

7. pac 最小工程 rust target 生成链补齐 → 冒烟脚本切正路（`auto build`
   路径）实测。
8. May 裸值 return 发射修复+g34 补裸值语料;（可选,待澄清①）`??` 四层
   实现+语料。
9. 全量回归（tf+四路+矩阵）+ 折叠合入。
10. 文档回写（KNOWN-DEBT 五债核销/divergences D-a2r-mode-entry 清偿
    注记）+ 复审 → reviewed。
11. merge 沉淀归档。

## 测试设计（TDD）

- **保护网**：525 终态全绿面（四路 30/30+tv 3559+矩阵）+tf 基线红
  （归属注记）不新增。
- **红先行**：两真缺陷（修前 rustc 红/金样 diff）→ 修后绿;P523-2 两洞
  （b07/b34 转译构建运行红→绿）;May 裸值（g34 裸值件红→绿）;`??`
  （新语料红→绿）。
- **命令**：`cargo tt`（tt 档）/ `bash scripts/aavm_build_smoke.sh` /
  四路 runner / 矩阵（P517-2 纪律）。

## 验收标准

1. `cargo tt` 全绿（28 件 bless 后+新锚）;两真缺陷修复有回归金样;tt
   入复审清单（条目留档）。
2. aavm.at a2r 模式：转译构建后位置参数 b07→55 实测;b34→10/20 实测
   （或按待澄清②显式转 532 登记）。
3. pac 最小工程 target 生成链补齐,冒烟脚本走 `auto build` 正路。
4. May 裸值语料绿;（可选项按裁定）。
5. tf/四路/矩阵零新增红;KNOWN-DEBT 核销;无静默丢弃。

## 执行步骤
（原子任务;W0 在 master,实现 in worktree `.worktrees/plan-531-dev`;
单折叠点=步骤 9）

1. [✅ 已完成] W0 考古：两真缺陷根因+tt 可见性方案+bless 流程;两洞根因定位。
   验证：考古注记+复现留档。
   - **W0 考古注记（2026-09-03,master 探针）**：
   - **tt 基线**：`cargo tt --no-fail-fast` 实测 28 红（与台账一致）,清单存
     `scratch/p531/`；`.wrong.rs` 已随跑写入（gitignored）。
   - **真缺陷①根因**（pointer/004,`x as *mut _.clone()`）：`rust.rs` fn call
     实参循环 `dot_arg_owned_param`（:9147,Plan 016 Phase 4 引入）把
     `Expr::Dot` 形实参（含取址伪字段 `x.@`）当"字段读取+所有权形参"附加
     `.clone()`（:9344）——`x.@` 发射 `x as *mut _` 后接 `.clone()` 即
     "cast cannot be followed by a method call"（rustc 实证 E0282+语法错）。
     裸指针 Copy,修复=排除指针伪字段（`@`）。引入提交 06d086abc(2026-08-25)。
   - **真缺陷②根因**（arc_dyn_spec/008,`self.tools[(name) as usize]`）：
     `rust.rs` :7741-7757（Plan 514 W3 引入,8aef313ca 2026-09-02）
     `recv_is_list2` 的 `Expr::Dot` 臂——`matches!(base, Ident("self"))`
     **无条件**把任何 `self.xxx` 接收者当 List 索引化；`self.tools` 为
     Map（HashMap）→ `.get(name)` 被改写 `[name as usize]`,`&str` 键被
     强转 usize（E0605 同族,最小语料 probe_map_get.at 复现）。修复=经
     `struct_field_types` 解析 self 字段真实类型,仅 List/Array 索引化。
     （另注：非 self 局部接收者路径 `r.tools.get(name)` 发射正确。）
   - **tt 复审可见性裁定**：沿用 Plan 507 desktop_protocol 先例——**tf 档
     语义不动**（tf 不加 test-trans）,在 `.cargo/config.toml` tf 注记块
     追加"复审清单须另跑 `cargo tt`"条目（W1 步骤 5 落地）。
   - **bless 流程**：tt 金样 harness 失配即写 `.wrong.rs`（gitignored）；
     bless = 修完真缺陷后重跑 tt,逐件 diff 评审 wrong vs expected 后
     wrong→expected 覆写（28 件清单+逐件 diff 摘要留档本计划）。
   - **P523-2①根因**（argv.get E0308）：双因——(a) `a2r_std::env::args()`
     仍返回**空格拼接 String**（a2r_std.rs :510）,而 VM 参考侧 P524 已定
     `process.args()`=List 契约（`shim_process_args()->Vec<String>`=
     [程序路径]+透传,stdlib.rs :692）;现存 .at 消费方（03_image_scraper
     `list.len/args[1]`）均已按 List 形态消费。(b) `var argv =
     process.args()` 绑定的局部类型 Unknown → `.get(1)` 不走 Auto List
     索引化发射（:6479/:7741 均需 List 类型）,落 Rust `Vec::get` Option
     形态 → 传 `ev_run_files(str)` 位 E0308。修复向=a2r_std 对齐 List
     契约+args() 绑定局部登记 List 类型（索引化自动接管）。
   - **P523-2②定位面**（精确根因待红证复现）：报错点=engine.at :584-586
     GetField 臂 `c.field_idx(tname,fname)` 动态查 `c.tys`;注册点=
     codegen.at `cg_type_decl`（早注册+尾态刷新,:2032-2131）。差异面=
     **ev_run_files（cg_compile_files 多文件路径,含 :3458 tys 跨单元合并）
     vs ev_run（单文件）**——corpus 腿（ev_run 嵌源码形态）绿,位置参数
     形态（--files 文件路径形态）红;同 .at 代码解释路径绿/转译路径红 →
     a2r 对 cg 链某构造发射分歧,复现探针=worktree 内 aavm.at merge
     构建（W0b 落盘,沿 fourpath/build_aavm_rust_bin 基建）。
   - **a2r 模式构建基建**：`transpile_rust_project_merged`（rust.rs :21966,
     无 CLI 暴露）+ vm_file_tests `build_aavm_rust_bin`（剥 use 行→merge→
     prelude/harness→cargo build）+ aavm2_a2r.rs fourpath runner（#[ignore]
     验收档）。aavm.at 入口形态探针沿此骨架改 harness 为 aavm.at 自身
     main+原生 shim（process/IO.read_line/parse_int/a2r_std::value_len）。
2. [✅ 已完成] W0b 红先行：两真缺陷最小件/两洞复现/May 裸值件落盘（红证）。
   - 红证留档：(a) 两真缺陷=tt 基线 28 红（`scratch/p531/tt_28_reds.txt`）+
     rustc 报文（①"cast cannot be followed by a method call"+E0282;②
     E0605 族 str as usize——最小语料 probe_map_get.at 复现 `self.tools`
     接收者即触发）;(b) May 裸值=probe_may_bare.at 发射 `return n * 10;`
     于 `-> Option<i64>` 位,rustc E0308;(c) 洞①全链红证=aavm_at_mode
     探针（worktree `tests/aavm_at_mode_tests.rs`,#[ignore] 验收锚）实测
     merged 转译 aavm.at（13k 行,含全 lib）构建仅 1 错：
     `ev_run_files(argv.get(1))` E0308 Option<&String> vs &str——与台账
     记录逐字吻合;(d) 洞②**不重现**：argv 文本垫片桥接后 b34 经转译版
     aavm 输出 10/20（转译版 struct 字段表已绿——疑 525 codegen 早注册批
     顺带清偿;步骤7 转为无垫片复证+核销注记）。
   - 探针基建修正留痕：merge 入口须为**目录**（文件入口经 use 发现,
     use 已剥→lib 不入）;原生 shim 形态=ProcessShim const 实例（mod/
     关联函数在 `process.args()` 值位是 E0423/E0599）+IO 静态+
     parse_int 扩展 trait+a2r_std::value_len。
3. [✅ 已完成] 真缺陷①修复（独立提交+锚）。验证：新金样绿。
   - 提交 b9e044e20：`dot_arg_owned_param` 排除 `@` 伪字段
     （rust.rs :9147 形）;pointer/004 tt 件翻转绿;修复发射与金样逐字节
     一致（残余 E0133 unsafe-deref 为裸指针语料固有形,金样同形）。
4. [✅ 已完成] 真缺陷②修复（独立提交+锚）。验证：同上。
   - 提交 978bad5fd：新增 `current_impl_type`（type/ext 方法发射位设置）
     +`recv_is_list_like`（self.<field> 经 struct_field_types 实型解析,
     仅 List/Array 索引化,不可解析回落 .get 方法形）;两处 get 转换位
     （索引化位+实参 as usize 位）统一走该判定。arc_dyn_spec/008 的
     `self.tools.get(name)` 与金样一致（残余 `mut tool` 参数位漂移属
     bless 面）;E0605 str-as-usize 消失（残余 Arc 裸引为该语料金样
     固有形,两形同）。
5. [✅ 已完成] tt 28 件批量 bless（逐件 diff 清单评审）+入复审清单。
   验证：`cargo tt` 全绿。
   - 实际 bless 27 件（28−pointer/004 真缺陷①修复后直接翻绿,金样未动）。
     逐件 diff 评审归类（27 件全量清单与 diff 摘要存
     `scratch/p531/bless_diffs.txt`）:mut 参数漂移 6/`(i) as usize` 括号化
     3/split→map(to_string).collect 物化 6/as u32 字段字面量强转 2/
     算符优先级显式括号 1（514 W2 修复本体）/builder 链拆句 1/
     File::write_text→std::fs 直写 2/a2r_std use 前置 2/fn 指针字段
     .clone() 1/私有 const doc 注释发射 1/str 借用 .as_str() 1——全部为
     433-514 期发射演进的滞后金样,无真缺陷形态。
   - tt 入复审清单：`.cargo/config.toml` tf 注记块增补（沿 507
     desktop_protocol 先例,tf 档语义不动）:"复审清单须另跑 cargo tt"
     （a2r/cookbook 文本金样曾脱离门禁 444/514/523 三期累积）。
   - 提交 dace61b20。
6. [✅ 已完成] argv.get 解包修复+aavm.at 全链实测（b07→55,垫片撤除）。
   - 提交 f0c10894f：`infer_type_from_expr` 增 `process.args()` 臂 →
     `Type::List(StrOwned)`（P524 List 契约;`env.args` 的 a2r_std String
     契约不动）。`var argv = process.args()` 绑定型经 store 推断登记 →
     `.get(1)` 走既有 List 索引化 + `arg_is_str_get` 实参 `.as_str()` 借用
     → `ev_run_files(argv[(1) as usize].as_str())`。
   - 全链实测（aavm_at_mode 探针,无垫片）：merged 转译 aavm.at(全 lib
     13k 行)+原生 shim → cargo build 零 E0308 → `b07_fib.at` 位置参数
     运行输出 **55** ✓;文本垫片路径自然退役（构建直接成功,垫片分支
     不触发）。另 tt 全绿 3746/3746（bless 后复验）。
7. [✅ 已完成] 转译版 struct 字段表修复（b34→10/20 实测;超量级转 532 登记）。
   - **实测不重现（已绿）**：无垫片全链构建后 `b34_struct_basic.at` 经
     转译版 aavm 输出 **10/20** ✓（位置参数→ev_run_files→cg_compile_files
     →GetField 动态查表全链）。判断：525 的 codegen 早注册批
     （codegen.at W2"早注册+尾态刷新"）已顺带清偿本洞,台账登记后未复测。
     债务核销注记按"已由 525 顺带清偿+本计划实测复证"落 KNOWN-DEBT
     （步骤10 文档回写）;不转 532（无遗留工作）。
8. [ ] pac target 链补齐（冒烟切正路）+May 裸值修复+（可选）`??`。
9. [ ] 全量回归+折叠合入 master。
10. [ ] 文档回写+复审 → reviewed。
11. [ ] merge 沉淀归档。

## 复审记录

## 待澄清事项

1. **`??` 可选项**（阻塞步骤 8）：缺省顺做（Pratt 表已有,四层臂+语料
   半天级）;若 W0 发现与 May 语义耦合超预期则显式延后（P525-3 维持）。
2. **P523-2② 逃生舱**（阻塞步骤 7）：根因若在主 a2r type 发射深层
   （525-1 同片区敏感性）且超小修量级 → 登记转 532 W0 处置,本计划
   其余照收。
3. **api-example 前端红归属**（阻塞步骤 8）：缺省独立处置不入本计划
   （UI 线/独立小件）;若 pac target 链修复顺带揭示同根因再裁定。
