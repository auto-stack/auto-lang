---
plan_id: PLAN-511
status: drafting               # drafting → executing → execution_done → reviewed → archived
feature_name: aavm-midlang-struct-use-modules
author: [zhaopuming]
created_at: 2026-09-01
updated_at: 2026-09-01

# /auto-plan:review 结束时填写：
supersedes_spec_components: []
new_spec_components: []
touched_goals: [GOAL-017]     # 自举：用 Auto 写 Auto 编译器（aavm）

affects: [aavm]
current_step: 0
total_steps: 22
---

# [PLAN-511] aavm 中阶语言能力：struct 类型定义 / use 模块化 / 中阶补缺

## 变更摘要

aavm v2（`auto/lib` 六文件，Plan 429–434/447/495 收口）目前已是**能完整编译并
执行一个小而完整语言子集**的自举编译器：算术/逻辑、let/var/赋值、if/else-if/else、
while、for-in-range、break/continue、顶层 fn 声明与调用（含递归，corpus fib）、
数组（字面量/下标读写/循环/.len()）、字符串与 f-string、enum + is 模式匹配
（含载荷实例）。判据为 M1–M5 五道 corpus 闸门（与 Rust 宿主逐字符/逐行对拍）
+ 五方对比矩阵，CI（vm-files-ci.yml）常绿。

本计划把语言能力推到**中阶**，三个波次（每波独立折叠，镜像 Plan 447 三部曲
单计划分波合入的先例）：

- **W1 type 基础定义**：`type X { 字段 }` 声明 + `X { a: e1 }` 构造字面量 +
  `p.a` 字段读/写，四层（parser/typeinfo/codegen/engine）收编。
- **W2 中阶补缺**：for-in 数组/表达式迭代、字符串下标、一元负号、下标复合
  赋值、**全局变量**（顶层 let/var 跨 fn 可见，load.global/store.global——
  亦是 W3 模块级状态的地基）。
- **W3 use 模块化**：`use mod` 语句 → 文件解析（file shim）→ 多编译单元
  codegen → 链接器（pub 导出 + `mod#fn` 跨模块符号 + 模块初始化序）→
  引擎多模块执行（`ev_run_files` 入口）。

**明确延后**（下一阶段，与 OOP 同批）：方法/impl/trait、闭包与嵌套 fn、
泛型与 alias/is-as-has 类型操作符、Option/May 错误传播、生成器、并发。

## 目标

1. aavm 可编译并执行 struct 类型定义程序（声明/构造/字段读写），字节码与
   宿主（Rust 参考实现）发射序逐字符一致（M4 判据），运行输出一致（M5）。
2. aavm 补齐控制流/函数残留缺口：for-in 数组、字符串下标、一元负、下标
   复合赋值、全局变量——中阶子集日常可用。
3. aavm 支持 `use` 多文件程序：`use db` 解析同目录 `db.at`，pub fn/pub let
   跨模块调用与可见，模块顶层按依赖序初始化；多文件 corpus 双闸（M4/M5）
   与宿主对拍一致。
4. 全程五道闸门（M1–M5）+ vm-files-ci 不回绿即不合入；新增能力先考古宿主
   发射序、落盘规格再实现（Plan 495 教训：语义分歧先定案谁为规范）。

### 非目标（Out of Scope）

- 不追求与主编译器全特性对齐（沿 project.md 立项边界）；仅扩中阶子集。
- OOP（方法/impl/trait/继承）、闭包/捕获、嵌套 fn、泛型实例化、`?`/May、
  生成器、async/actor：明确不做，登记延后。
- `use.rs`/`use.py`/`use.c` 异构导入：拒绝报错（超出 aavm 域）。
- AA2R（`a2r.at`）对 struct/use 的转译发射：缺省暂缓 + divergence 登记
  （见待澄清①；AA2R 服务自举回路，新语法非其语料必需）。
- aavm VBool parity 债（KNOWN-DEBT P474-旁支）不在本计划内清偿，但**不得
  扩面**（新语料避免裸 bool print 断言形态）。

## 架构方案

现状链路（依赖序，`AUTO_LIB_FILES_V2` 单一事实源）：

```text
token.at ─ lexer.at ─ parser.at ─ typeinfo.at ─ codegen.at ─ engine.at
                                    │              │            │
                              typecheck_dump   cg_compile    ev_run(source)→str
                              (推断层)         (I{op,s,n}    (栈式 VM,Val=
                                               序列化直出)    VInt/VStr/VArr/VInst)
```

三波改动均沿既有四层收编模式（Plan 447 ② is-match 同款），不改六文件依赖
拓扑；W3 视体量把链接器做成 engine.at 内新段或第七文件（决策点见步骤 17，
缺省 engine 内段，避免动 `AUTO_LIB_FILES_V2` 与 pac.at 面扩容）。

**关键设计约束（全程生效）**：

- **宿主为规范**：M4 判据是 aavm `codegen_dump` 与 Rust codegen 反汇编逐行
  相等。每个新语法的宿主发射序必须先考古（现成范式：`docs/specs/aavm/design/
  m4-bytecode-format.md`），发现宿主自身怪序（如 set.elem 的 value 展栈底
  quick-fix 栈序）时镜像之并在规格注记，**不改宿主**。
- **`.line` 语义**：新语句路径一律走 `cg_line`（同线去重状态机，Plan 495），
  is 单表达式 arm 体行发射等既有定案不回退。
- **规范化条款最小化**：M4 已有两条规范化（load.str 池内容显示、槽释放组
  排序）。新opcode序列若暴露宿主 HashMap 迭代序等不稳定面，按先例追加
  规范化条款并注记于规格，而非迁就。
- **语料即判据**：能力落地的定义 = corpus 文件进对应闸门目录且全绿。

## 需求分析与背景调查

（取材：docs/specs/overview.md §aavm、docs/specs/aavm/project.md、
auto/lib/*.at 文件头 Coverage/Missing 注记、m2/m4 闸门 harness、
KNOWN-DEBT-AND-RISKS.md P474-旁支/P495、Plan 447/495 归档）

### 现状能力矩阵（已核实，2026-09-01）

| 能力 | 状态 | 证据 |
|---|---|---|
| 控制流 if/else-if/else、while、for-in-range | ✅ 已具备 | corpus_m4 b04/b05/b06；codegen.at Coverage 13-16 行 |
| break/continue（含嵌套层 patch 回填） | ✅ 已具备 | b32_is_break_continue；CG.brk_js/cont_js 帧 |
| 顶层 fn 声明/调用/递归/参数/返回 | ✅ 已具备 | b07_fib（递归 fib(10)）、b24–b26 |
| enum 声明 + is 模式匹配（含载荷） | ✅ 已具备 | b13/b31；new.instance/construct.instance/get.generic.field 已实现于 engine.at:310-370 |
| 数组字面量/下标读写/循环/.len() | ✅ 已具备 | b27–b30 |
| 字符串/f-string/str.cat | ✅ 已具备 | b08/b21–b23/b33 |
| **type（struct）声明与构造** | ❌ 仅 parser S-expr dump（D38a，且只 members 路径） | parser.at:1506；cg_stmt 无 Type 分支 |
| **use 模块化** | ❌ 完全缺失（单源 ev_run，无解析/无链接） | engine.at:231 `ev_run(source str)`；codegen.at Missing 注记「多模块链接」 |
| 全局变量（顶层 let 跨 fn） | ❌ 缺失 | codegen.at Missing「LOAD_GLOBAL/STORE_GLOBAL」 |
| for-in 数组/表达式 | ❌ 缺失（仅 range） | cg_for 只识别 range 形态；宿主 `for book in list_books()` 可用 |
| 字符串下标 / 一元负 / 下标复合赋值 | ❌ 缺失 | codegen.at Missing 清单 |
| 嵌套 fn / 闭包 | ❌ 明确报错/缺失 | cg_stmt `nested fn unsupported`；**本计划延后** |

### 宿主侧目标语义（已考古）

- struct 声明：`type User { id int \n name string }`（字段=名字+类型，无冒号）；
  构造字面量：`User { id: 1, name: "Alice" }`（examples/api-example/back/db.at:13）。
- 对象字节码：`CREATE_OBJ`(0x2E,"create.obj") + `SET_FIELD`(0x2A,"set.field",
  栈序 value,field_str_idx→void) + `GET_FIELD`(0x2D,"get.field")——**区别于**
  enum 载荷的 NEW_INSTANCE 族(0xB0-0xB3，aavm 已有前三、缺 SET_GENERIC_FIELD)。
  struct 走哪条宿主路径以 W0 考古为准（推断：create.obj 族，但以实测反汇编定案）。
- 全局变量：宿主 codegen 维护 `global_vars` 集合（codegen.rs:198），模块顶层
  let/var 进集合，fn 体内引用编译为 `LOAD_GLOBAL`(0xC5)/`STORE_GLOBAL`(0xC6)，
  操作数=名字池索引。
- use 解析：宿主 `resolve_uses`（compile.rs:444）扫 use 语句 → 模块路径映射
  同目录 `<mod>.at`（`add_source_dir`，compile.rs:288）→ 逐模块编译 → Linker
  拼装。导入形态：`use mod` / `use mod: item1, item2` / `use mod::{a, b}` /
  `use mod: *`（parser.rs:6122 parse_use_items）。跨模块符号 `mod#fn`
  （loader.rs:289 歧义消解注释）。导出面：`pub fn`（db.at:25）等 pub 项。
- 文件读取：AutoVM 内 `auto.file.read_text`（native id 1000，
  native_catalog.rs:907）可从 aavm 代码调用——W3 模块加载的原生地基。
- for-in 数组：宿主支持 `for book in list_books()`（book-reader main.at:110）。

### 风险与对策

| 风险 | 对策 |
|---|---|
| 宿主 struct/全局/多模块发射序含未成文怪序（set.elem 先例） | W0 强制先行考古并落盘规格；发现怪序镜像+注记，禁改宿主 |
| 多模块链接反汇编含重定位/地址平移，M4 对拍面爆炸 | 考古时同步设计第三/四条规范化条款（如符号表序列化排序），先例可循 |
| AA2R 矩阵腿因新语料破绿 | 语料进 M2–M5 四闸 + aa2r 腿按待澄清①决策（缺省暂缓+divergence） |
| file shim 在 `auto build`（AA2R→Rust）路径的可用性 | W3 判据只锁 VM 路径五闸；build 路径可用性作为观察项登记，不设闸 |
| Val/arena 加对象载荷牵动全分派链（P474-旁支 VBool 前车） | W1 优先复用既有 VInst/arena 实例路径；确需新载体时全链一次性收编并跑 tf |

## 详细设计

### W1：type 基础定义（struct）四层收编

1. **parser.at**：type-decl 解析从「S-expr dump 雏形」升为完整 members 解析
   （字段名+字段类型；字段类型支持内建类型、`[]T` 数组型、**已声明用户类型**
   ——前向引用按宿主行为考古定案）；parse_dump 输出保持与宿主
   `ast/types.rs TypeDecl::fmt` 对齐（D38a 先例）。构造字面量
   `X { a: e1, b: e2 }` 与字段访问 `p.a`（含链式 `p.a.b`）进表达式层
   （Pratt 后缀位）。M2 corpus 扩展 p 系列 type-decl/构造/字段访问用例。
2. **typeinfo.at**：type 注册表（name → 字段名/字段类型表）；构造表达式
   推断为类型名、字段访问推断为字段类型；typecheck_dump 输出对齐宿主
   推断层（M3 判据）。
3. **codegen.at**：cg_stmt 增 TypeDecl 分支（登记类型表，零代码发射或对齐
   宿主行为——以 W0 考古为准）；构造字面量发射 create.obj + set.field 序
   （**发射顺序以考古定案**：字段求值序/池引用方式）；字段读 get.field、
   字段写 `p.a = e` 特判（对齐宿主 store 路径）；`.line` 走 cg_line。
4. **engine.at**：create.obj/set.field/get.field 分派（Val 复用 VInst 实例
   载荷，arena 同数组池模式或镜像宿主 object arena——以宿主 engine 语义
   为准）；print(obj) 的显示形态对齐宿主。
5. **corpus**：corpus_m4 增 `b34_struct_basic/b35_struct_field_rw/
   b36_struct_nested/b37_struct_in_fn` 系列（避免裸 bool print，防 P474 扩面）。

### W2：中阶补缺（控制流/函数残留 + 全局变量）

1. **for-in 数组/表达式**：cg_for 扩第二形态——迭代表达式（数组值/返回数组
   的调用），镜像宿主发射（考古：迭代器协议/arr.len+get.elem 循环降级还是
   专用 opcode）；识别用既有的 CGVar.arr 数组性跟踪扩展到表达式层。
2. **字符串下标**：get.elem 的 str→码点分支（codegen.at Missing 注记点位），
   宿主行为考古（码点 vs 字节）。
3. **一元负号**：`neg` opcode 已在枚举（engine 分派考古补齐），Pratt 前缀位
   接入。
4. **下标复合赋值**：`a[i] += e`——既有「下标赋值两段游标快照」模式
   （cg_stmt a[idx]= 分支）复合化。
5. **全局变量**：CG 增 `global_vars` 名集（镜像宿主 codegen.rs:198 语义）；
   顶层 let/var 登记集合，fn 体内引用发射 load.global/store.global
   （操作数=名字池索引）；engine 增全局区（模块级单份）。is-pattern 绑定、
   fn 参数等仍走局部槽（宿主语义考古确认边界）。
6. **corpus**：`b38_for_in_arr/b39_str_index/b40_neg/b41_idx_compound/
   b42_globals/b43_global_shadow` 系列。

### W3：use 模块化

1. **parser.at**：use 语句解析（四形态 + `use.rs/py/c` 拒绝报错）；
   parse_dump 输出对齐宿主 use-decl Display；M2 corpus 扩展。
2. **模块解析器**（parser/codegen 间新段或 parser.at 内函数群）：
   `use mod` → 主文件同目录 `mod.at` 读取（`auto.file.read_text`）；
   递归解析传递依赖；**环检测**（报错对齐宿主 loader 环语义）；
   已加载模块去重缓存。
3. **codegen.at 多编译单元**：per-module CG 实例；`pub fn`/`pub let`/
   `pub var` 进模块导出表（非 pub 不导出——宿主可见性语义考古确认）；
   跨模块调用解析为 `mod#fn` 限定符号（CALL 的 FnEntry 符号表扩展为
   模块限定名）；`use mod: item` 定向导入名注入作用域、`use mod` 全限定、
   通配导入按宿主规则。
4. **链接器**：模块拼装（依赖序）、`mod#fn` 符号重定位、**模块初始化序**
   （被依赖模块顶层语句先执行，主模块最后——宿主执行序考古定案）、
   全局区合并。缺省实现于 engine.at 新段（`ev_link`），若超 ~300 行或
   需独立测试面则升第七文件并同步 `AUTO_LIB_FILES_V2` + pac.at（决策点）。
5. **engine.at**：新入口 `ev_run_files(main_path str) str`（保留 ev_run
   单源兼容）；跨模块 call 与模块全局区衔接 W2。
6. **corpus**：新目录 `corpus_use/`（主文件 + 被依赖 .at 多文件组，
   覆盖：基本导入/pub 可见性/跨模块递归调用/模块顶层初始化序/传递依赖/
   环报错/通配与定向导入）；m4/m5 harness 增多文件变体（Rust 侧镜像
   resolve_uses+Linker 全管线编译后反汇编/执行，对拍判据同单文件）。

### 规格与登记（贯穿）

- W0 考古结论落盘：`docs/specs/aavm/design/` 新章或扩 m4-bytecode-format.md
  （struct 对象/全局变量/多模块链接三节 + for-in 数组等补缺四件）。
- divergences.md 登记新分叉（编号续既有序列）：AA2R 暂缓腿、规范化新条款、
  延后项（闭包/嵌套 fn/OOP/泛型）。
- project.md 模块清单/判据/能力矩阵回写；auto/lib/README.md 同步。

## 测试设计（TDD 三层，测试先行）

> 范式转变（区别于 429-434 的"corpus 随切片长"）：本计划**每波语料/测试架构
> 先行落盘，闸门转红为实现启动条件**；W3 的多文件 harness 在动 lib 代码之前
> 先建成并用宿主双侧自证正确。既有事实：`auto test` CLI 已存在（#[test] 函数
> 在隔离 VM task 执行，test_runner.rs:131；断言 native `auto.assert_eq`）；
> 文件式用例协议（.at + .expected.out/.result/.error）由 vm_file_tests.rs 与
> `auto test` 的 discover_vm_tests 共用 test/vm/ 目录；宿主 use 多文件样板
> test/vm/17_modules/001_use_fn（点路径 `use auto.greet_mod` → auto/ 根）。

### L1 红先行 corpus（差分主判据，cargo tv）

| 层 | 判据 | 命令 |
|---|---|---|
| M1/M2（token/lexer/parser dump） | corpus_m1/m2 扩展后与宿主逐字符一致 | `cargo test -p auto-lang --lib --features test-vm-files -- test_aavm2 --include-ignored`（下称 tv-aavm2） |
| M3（typeinfo 推断层） | corpus_m3 扩展一致 | 同上 |
| M4（codegen 反汇编） | corpus_m4 + corpus_use 双侧逐行相等（含新规范化条款） | 同上 + m4 多文件变体 |
| M5（全管线行为） | corpus_m4/corpus_use 输出与 `run_with_capture` 逐行相等 | 同上 + m5 多文件变体 |
| AA2R（矩阵⑤腿） | 按待澄清①决策（缺省：新语料不入矩阵腿，divergence 登记） | parity/ 下 `cargo run -- --root . --auto-binary ../target/debug/auto.exe aavm` |
| CI | vm-files-ci.yml 全绿 | push 后 GitHub Actions |
| 分级门禁 | Category B（auto/lib .at 为主 + 测试 harness Rust 改动）：局部 `cargo t vm`/`cargo tv`，合入前一次 `cargo tf` | `.cargo/config.toml` 别名 |

TDD 流程（W1/W2 零新架构——M2-M5 闸门均为目录扫描泛型，新 .at 文件自动进闸）：
新能力宿主已支持 → corpus 落盘即刻获得 Rust 侧期望值，aavm 侧遇新语法报
unsupported → **tv 转红 = 测试就位证据** → 四层实现 → 绿。语料先于 lib 改动提交。

### L2 W3 多文件 harness（新测试架构，先于 W3 实现）

- `corpus_use/{NNN_case}/` 目录协议：main.at + 被依赖模块 .at（结构镜像
  17_modules/001_use_fn；模块文件与 main 同目录，`use mod` 相对解析）。
- Rust 侧新 harness 变体（`crates/auto-lang/src/tests/aavm2_m4.rs`/`m5.rs`
  增多文件腿）：镜像宿主 resolve_uses + Linker 全管线编译/执行，与 aavm
  `ev_run_files` 对拍，判据同单文件。
- **错误用例通道**（aavm2 闸门现无 .expected.error 等价物）：harness 增错误
  形态比对（use.rs/use.py 拒绝报错、循环依赖报错、未声明模块报错），断言
  两侧错误信息一致（错误文本以宿主为规范，考古定案后镜像）。
- harness 自证：实现前先用"双侧都走宿主路径"的对照跑绿，证明架构本身正确。

### L3 Auto 侧 fast loop（`auto test`，红先行单元层）

- `test/vm/aavm2/99_unit/*.at`：`#[test]` fn + `auto.assert_eq`，断言
  ev_run/ev_run_files 微行为（引擎单 opcode 语义、解析边界、模块解析环检测）。
- `auto test <dir>` 直跑 VM，无需 cargo 编译——开发内环；同时是"aavm 测试
  也用 Auto 写"的自举方向第一块资产。
- lib 符号引用方案 W0 考古定案：宿主 use 点路径（`use auto.lib.token` 形态
  → auto/lib/ 解析可行性）或 auto/ 下聚合入口；AA2R 路径（auto build）对
  测试文件的容忍策略同步确认。

回归钉：每个 corpus 文件即回归钉（无静态期望、实时对拍，Plan 495 模式）。

## 验收标准

1. tv-aavm2 全绿（M1–M5 含全部新 corpus：struct 系 b34–b37、补缺系 b38–b43、
   corpus_use 多文件组），vm-files-ci 绿。
2. **TDD 过程证据**：三波各自的"语料先行红清单"留档于执行步骤证据
   （步骤 2/8/13）；W3 harness 自证记录（双侧宿主路径对照绿）留档。
3. `ev_run_files` 能编译执行双模块以上程序：跨模块 pub fn 调用、模块顶层
   初始化序、`use mod: item`/通导入形态，输出与宿主全管线一致；错误用例
   通道（use.rs 拒绝/循环依赖/未声明模块）双侧一致。
4. `test/vm/aavm2/99_unit/` Auto 侧单测套件（#[test]+assert_eq）成建制，
   `auto test` 可跑且绿。
5. 五方矩阵既有稳定集（corpus_m4 原有 b01–b33）不回绿破腿；新语料的矩阵
   腿处置有显式决策与 divergence 登记。
6. W0 考古规格文档落盘且实现与规格一致（复审抽查）；divergences.md/
   project.md/README 回写完成。
7. `cargo tf` 全量绿（折叠前）；零新增编译警告；无调试打印残留。
8. 延后项（闭包/嵌套 fn/OOP/泛型/May/生成器）在 KNOWN-DEBT 或 plan
   Out-of-Scope 有显式登记，无静默丢弃。

## 执行步骤
（原子任务：精确文件路径 + 确切操作 + 验证命令；每步完成后追加 [✅ 已完成] 一行证据）

> 约定：所有 auto/lib 改动在 worktree `.worktrees/plan-511-dev` 内进行；
> **TDD 硬约定：每波语料/测试架构先行落盘（闸门转红）再动 lib 实现**；
> 每波（步骤 7/12/18）为折叠点，波内 tv-aavm2 绿即准合入。

### W0 考古先行（master 直接做，纯文档）

1. [ ] 宿主发射序考古并落盘规格：struct 声明/构造/字段读写（create.obj 族
   vs new.instance 族定案）、全局变量、多模块链接（重定位表示/初始化序/
   可见性）、for-in 数组、字符串下标、一元负、下标复合赋值——产出
   `docs/specs/aavm/design/` 规格（新章或扩 m4-bytecode-format.md），
   含最小宿主样本反汇编与待镜像怪序注记。
   验证：文档评审（宿主反汇编片段贴证）；scratch 考古脚本不入库。
   同批考古（L3 前置）：宿主 use 对 auto/lib 子目录点路径解析行为
   （`use auto.lib.token` 可行性）；`auto test` + `auto.assert_eq` 在
   VM 模式对 ev_run 调用型的可用性（探针一件）；use 错误形态文本
   （use.rs 拒绝/循环依赖/未声明模块，错误用例通道的期望基准）。

### W1 struct 四层（worktree）

2. [ ] **W1 语料先行（红）**：corpus_m1/m2（p 系列 type-decl/构造/字段
   访问）、corpus_m3（t 系列）、corpus_m4（b34–b37）全部落盘；跑
   tv-aavm2 确认 M2-M5 对应件**转红**（unsupported 报错即红证），红清单
   记入本步骤证据。验证：tv-aavm2 红（预期内）+ 红件清单。
3. [ ] `auto/lib/parser.at`：type-decl members 完整解析 + 构造字面量/
   字段访问表达式。验证：tv-aavm2 M1/M2 绿。
4. [ ] `auto/lib/typeinfo.at`：type 注册表 + 构造/字段推断。
   验证：tv-aavm2 M3 绿。
5. [ ] `auto/lib/codegen.at`：TypeDecl 分支 + 构造发射（create.obj+set.field
   序，按步骤 1 规格）+ 字段读写。验证：tv-aavm2 M4 绿。
6. [ ] `auto/lib/engine.at`：create.obj/set.field/get.field 分派（VInst 复用
   或新载体）+ print 形态对齐。验证：tv-aavm2 M5 绿。
7. [ ] 折叠点①：AA2R 矩阵腿决策执行（待澄清①）+ divergence 登记 +
   vm-files-ci 绿 + master 合入（`feat(aavm): Plan 511 W1 struct 类型四层收编 (Plan 511)`）。

### W2 中阶补缺（worktree 续）

8. [ ] **W2 语料先行（红）**：b38–b43 corpus 落盘，tv 转红清单记证据。
9. [ ] `auto/lib/codegen.at`+`engine.at`：for-in 数组/表达式迭代（发射序按
   规格；顺收 447 收账缺口①——List 型 fn 参数 `.len()` 接收者跟踪，便宜则
   一并修）。验证：b38 + tv-aavm2 绿。
10. [ ] `auto/lib/codegen.at`：字符串下标 + 一元负 + 下标复合赋值。
    验证：b39–b41 + tv-aavm2 绿。
11. [ ] `auto/lib/codegen.at`+`engine.at`：全局变量（global_vars 集 +
    load.global/store.global + 引擎全局区）。验证：b42/b43 + tv-aavm2 绿。
12. [ ] 折叠点②：divergence 登记 + CI 绿 + master 合入（W2 同款提交格式）。

### W3 use 模块化（worktree 续；测试架构先于实现）

13. [ ] **W3 测试架构先行**：`corpus_use/` 目录协议落地（首批用例双份期望
    生成）；m4/m5 harness 多文件变体（Rust 侧镜像 resolve_uses+Linker）；
    错误用例通道（.expected.error 等价物）——harness 用"双侧宿主路径"对照
    自证绿。验证：harness 自证跑绿 + aavm 侧红清单。
14. [ ] `auto/lib/parser.at`：use 语句四形态解析 + 异构导入拒绝；
    corpus_m2 扩展（含错误形态件）。验证：tv-aavm2 M2 绿。
15. [ ] 模块解析器：`mod.at` 读取（auto.file.read_text）+ 传递依赖 + 环检测
    + 去重缓存。验证：99_unit Auto 侧单测（auto test）先行红→绿。
16. [ ] `auto/lib/codegen.at`：多编译单元 + pub 导出 + `mod#fn` 限定符号 +
    导入名作用域注入。验证：corpus_use 编译期用例（M4 前置）。
17. [ ] 链接器（缺省 engine.at 新段；超限则第七文件 + AUTO_LIB_FILES_V2/
    pac.at 同步——决策点在步 1 规格里预判）：拼装/重定位/初始化序/全局区
    合并。验证：链接层用例 + 99_unit 单测。
18. [ ] `auto/lib/engine.at`：`ev_run_files(main_path)` 入口 + 跨模块 call；
    corpus_use 全量接入多文件闸门。验证：tv-aavm2 全绿（M4/M5 多文件腿）。
19. [ ] 折叠点③：divergence 登记 + CI 绿 + master 合入（W3 同款提交格式）。

### 收尾

20. [ ] L3 资产固化：`test/vm/aavm2/99_unit/` Auto 侧单测套件（#[test]+
    assert_eq，`auto test` 可跑）成建制落盘；W0 定案的 lib 引用方案回写
    文档。验证：`auto test test/vm/aavm2/99_unit` 绿。
21. [ ] 文档回写：project.md（模块清单/判据/能力矩阵）、auto/lib/README.md、
    divergences.md、KNOWN-DEBT 视情；overview.md aavm 行能力注记。
22. [ ] 复审（/auto-plan:review 范式：验收标准逐条对代码、遗漏/延后扫描、
    健康检查、spec-impact 元数据）→ `cargo tf` 全量 → status: reviewed。

## 复审记录

## 待澄清事项

1. **AA2R 矩阵腿策略**（阻塞步骤 6，不阻塞 W1 实现）：新语料（struct/use）
   是否同步扩 `a2r.at` 发射面？缺省=暂缓 + divergences 登记（AA2R 的服务
   面是自举回路语料，struct/use 非必需）；若用户要求矩阵全腿同步绿则 W1/W3
   各 +1 步（AA2R 扩展，参照 idiom-upgrade-prereqs.md W1–W6 模式）。
2. **W1 范围边界**：is 对 struct 的模式匹配（`is User { id: 1 }` 形态）缺省
   **不入**本计划（enum is 已有，struct 模式匹配随 OOP 批评估）；仅做
   构造+字段读写。若要纳入，W1 增 1 步。
3. **W3 跨模块类型共享**（`pub type` 导入）：缺省**不入**（跨模块共享 struct
   随 OOP 批评估，本波只共享 fn/let/var）；语料避免跨模块类型引用。
4. **W3 入口命名与 build 路径**：`ev_run_files` 为 VM 路径判据入口；
   `auto build`（AA2R）路径的 file shim 可用性仅登记观察项不设闸——确认
   接受此边界。
