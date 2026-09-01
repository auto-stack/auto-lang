---
plan_id: PLAN-511
status: reviewed               # drafting → executing → execution_done → reviewed → archived
feature_name: aavm-midlang-struct-use-modules
author: [zhaopuming]
created_at: 2026-09-01
updated_at: 2026-09-01

# /auto-plan:review 结束时填写：
supersedes_spec_components:
  - "docs/specs/aavm/project.md: 修改——模块清单 codegen/engine 行 511 增量（struct 四层/全局变量/for-in 双通道/use 多编译单元+链接器/ev_run_files）、新闸门四件（use 多文件 M4/M5+错误通道+自证）、能力矩阵中阶行 ✅ + 延后清单（OOP/闭包/嵌套 fn/泛型/May/生成器）"
  - "docs/specs/aavm/design/divergences.md: 修改——新增「511 W1（struct 类型四层收编）」与「511 W3（use 模块化）」两节：D-AA2R-struct（ignored 腿 30/37 基线+7 件预存转译债）、D-mirror、D-soft-ident、D-stdlib-shadow、D-module-shape"
  - "docs/specs/overview.md: 修改——aavm 行 plans 序列扩至 429–434/447/495/511，注记 struct/全局/补缺/use 模块化"
new_spec_components:
  - "docs/specs/aavm/design/midlang-w0-archaeology.md: 新增——W0 四路源码考古定案（struct=NEW_INSTANCE 族+链式 get.generic.field 实测修正、全局变量读写不对称+fn main 体首重放、for-in 三隐藏槽+迭代器协议 CALL_NAT 112 零迭代、字符串码点下标、D1 下标/字段复合赋值宿主同文本拒绝、use 四形态+D2 环静默跳过+D3 pub 不过滤+错误文本表、D5 L3 聚合方案）+ §5/§7 执行期回写"
touched_goals:                 # 引用 docs/specs/goals.md 的 GOAL-NNN
  - "GOAL-017: 自举：用 Auto 写 Auto 编译器（aavm）——中阶语言能力落地：struct 定义/构造/字段读写、全局变量、for-in 数组、字符串下标、一元负、use 多文件模块化（多编译单元+链接器+初始化序），M1–M5 五闸+多文件双闸+错误通道全绿；L3 Auto 侧单测 13 件成建制（auto test 直跑）"

affects: [aavm]
current_step: 22
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

#### 既有验收用例回归面（429-434/447/495 存量资产清单——叠加不替换）

| 存量验收用例 | 载体 | 新方案中的位置 |
|---|---|---|
| corpus_m1~m4（c/p/t/b 系列 50+ 件，M1-M5 判据语料） | 目录扫描闸门 | L1 tv-aavm2 原样续跑（目录泛型，自动含新件） |
| corpus_a2r（g01–g08，AA2R 判据语料） | aavm2_a2r 闸门 | 同上 |
| M3 里程碑 fib（test_aavm2_m3_milestone_fib） | 显式测试 | 同上（--include-ignored 含） |
| 99_idiom_probe 16 件（447 宿主加固 H1-H6 验收） | vm_file_tests 显式 fn | `cargo tv` + CI 金样层续跑（**非** tv-aavm2 前缀——两条门禁命令都不可省） |
| 99_idiom_probe AA2R 探针冒烟（p01/p02b/p04/p05/p12 经 AA2R rustc 零错） | a2r_probe_smoke（ignored） | tv-aavm2 --include-ignored 续跑 |
| 001_smoke / 002_hello_compile 金样（唯二 .expected.out） | 文件金样 | `cargo tv` + CI 金样层续跑 |
| aavm2_repro_242（RC 回归钉） | 显式测试 | tv-aavm2 前缀匹配续跑 |
| 五方矩阵稳定集（corpus_m4 × ①②③④⑤） | parity/ 手动跑 | **不在 CI**——每折叠点手动必跑 + 验收 5 不回绿破腿判据 |
| vm-files-ci.yml 三层（六闸门/金样+ffi_dual/conformance） | GitHub Actions | 每折叠点要求绿（验收 1） |

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

1. [✅ 已完成] 规格落盘 `docs/specs/aavm/design/midlang-w0-archaeology.md`：struct
   （构造=NEW_INSTANCE 族定案/字段读写 set.generic.field+get.field/get.field 实测
   反汇编贴证/disasm 幽灵 nop 注记/print 形态分裂）、全局变量（var/const 顶层判定/
   读写优先级不对称/fn main 体首重放实测）、for-in 数组（三隐藏槽+const.0+迭代器
   协议 CALL_NAT 112）、字符串下标（码点 i32）、一元负、下标复合赋值（宿主编译
   错误→D1 同文本拒绝）、use 四形态+Display/四级搜索/环=静默跳过（D2）/pub 不过滤
   （D3）/链接器 mod#fn/初始化序/错误文本表；L3 探针实证（auto test+裸 assert_eq
   可用、use auto.lib.* 不可行、聚合方案 D5 定案）。四路源码考古+两份真机反汇编
   （b99 探针，scratch 已删，闸门复绿验证：m4/m5 corpus 2 passed）。

### W1 struct 四层（worktree）

2. [✅ 已完成] **W1 语料先行（红）**：corpus_m1 c05_struct（M1 绿——无新 token）、
   corpus_m2 p25_struct_ctor/p26_struct_field_forms、corpus_m3 t08_struct_fields、
   corpus_m4 b34–b37 共 8 件落盘（worktree 提交）。红清单：M2 p25
   `PARSE-ERROR:6:expected end of statement, got <LBrace>`；M3 t08
   `PARSE-ERROR:1:typeinfo: expected expression, got <Type>`；M4/M5 b34
   `CODEGEN-ERROR:unsupported expr atom: Type`；宿主侧 8 件全部可编译
   （disasm 诊断无 COMPILE ERROR）。AA2R 腿（#[ignore] 件 test_aavm2_compile_corpus）
   在 --include-ignored 下因 b34 红——按待澄清①缺省暂缓，折叠点①以 skip 清单落地。
3. [✅ 已完成] `auto/lib/parser.at`：expr_atom Ident/soft-ident 分支增构造字面量
   拦截（已注册类型名紧随 `{` → ctor_node，`(node (name X) (body (pair (name a)
   e1) ...))` 对齐宿主 Pair-落-body 段形态）+ soft-ident 表达式位兜底
   （is_soft_ident_kind 镜像宿主 Plan 356——`o.tag` 点右值曾是 M2 红点）。
   验证：tv-aavm2 M1 绿 + M2 绿（p25/p26/c05 全过）。
4. [✅ 已完成] `auto/lib/typeinfo.at`：type 声明走查（复用 parse_type_decl 注册入
   p.decls）+ 构造字面量推断（绑定类型名，镜像宿主 var_types 可观察行为）+
   Dot 字段型推断（t_field_ty 从注册 dump 括号配平提取；方法调用消费→unknown）。
   验证：tv-aavm2 M3 绿（t08 过，M2 不回归）。
5. [✅ 已完成] `auto/lib/codegen.at`：cg_type_decl（零发射注册 c.tys）+
   cg_ctor_emit（NEW_INSTANCE 族，考古定案非 create.obj）+ 字段读（首跳
   get.field 名字池非去重追加；链式跳 get.generic.field field=0+幽灵 nop——
   实测修正考古报告）+ 字段赋值两段快照（构造源变量 set.generic.field+3nop；
   嵌套/注解源 load.str+set.field；Dot 复合赋值 D1 同文本拒绝）。验证：
   tv-aavm2 M4 绿（b01–b37 全量）。
6. [✅ 已完成] `auto/lib/engine.at`：set.generic.field/get.field/set.field
   三条分派（VInst 复用既有 arena 载体；ev_field_idx 按名定位；print 形态
   对齐经语料红线规避 print(struct)）。验证：tv-aavm2 M5 绿——W1 五闸
   M1–M5 齐绿（32s 单跑全过）。
7. [✅ 已完成] 折叠点①：AA2R 腿 skip 清单落地（b34–b43 前缀，待澄清①缺省）+
   divergences.md 登记（D-AA2R-struct/D-mirror/D-soft-ident）+ W0 规格实测修正回写
   （body 段/链式读/soft-ident 三处）+ pre-fold 门禁：cargo tf 3338/3338 绿、
   cargo tv 3475/3477（cb_asynchronous_channel/cb_devtools_log_error 两件经
   master 实跑确认为预存,非 W1 回归）+ master 合入 6094d554f + worktree 重同步
   后五闸复绿。

### W2 中阶补缺（worktree 续）

8. [✅ 已完成] **W2 语料先行（红）**：b38_for_in_arr（数组变量/字面量/返回数组
   调用三通道）/b39_str_index/b40_neg/b42_globals/b43_global_shadow 落盘；b41 按
   D1 不入 corpus（改 L3 错误文本件）。M4 红清单：b38 `for: expected range op`、
   b40 `unsupported expr atom: LParen`（-(x+y) 括号形态暴露真缺口）、b39/b42/b43
   文本差异（b42/b43 见 aavm 把顶层 var 当 wrapper 局部 store.loc——store.global
   缺失；b39 差异待实现时定源）。宿主侧全可编译；b43 宿主语义实证：fn 内
   `let g = 5` 写全局（输出 5/5）。
9. [✅ 已完成] `auto/lib/codegen.at`+`engine.at`：for-in 数组/表达式双通道
   （数组通道三隐藏槽+const.0+槽位复用/hi_idx 高水位；Call 通道迭代器协议
   nat#112 且**镜像宿主零迭代**——裸句柄非迭代器，b38 宿主零输出实证；
   Call 通道不推作用域变量落 fn 层）。顺收缺口①：List 型 fn 参数记 arr
   （`.len()` 接收者跟踪）。验证：b38 M4/M5 绿。
10. [✅ 已完成] `auto/lib/codegen.at`+`engine.at`：字符串下标（get.elem 码点
   既有+str.cat 穿透——宿主字符串性穿透下标、STR_CAT 恒拼接 s[i]+s[j]→
   "6667"）+ 一元负（既有 neg+新增 LParen 原子补 -(x+y) 形态）+ 下标复合
   赋值 D1 同文本拒绝（auto test 探针断言逐字符一致）。验证：b39/b40 绿
   （b41 按 D1 入 L3）。
11. [✅ 已完成] `auto/lib/codegen.at`+`engine.at`：全局变量——global_vars
   （仅顶层 var/const 注册+无 scope 守卫：fn 内同名 let 写全局 b43 怪序镜像）
   + load.global/store.global（u32 名字池索引，赋值带 DUP+POP、声明无）+
   读局部>全局/写全局>局部不对称 + fn main 体首 init token 区间重放 +
   引擎名字键全局区（未命中缺省 0）。验证：b42/b43 绿——五闸 M1–M5 齐绿
   （36s 单跑全过）。
12. [✅ 已完成] 折叠点②：divergence 已随 W1 折叠登记（skip 清单覆盖 b34–b43
   前缀,无需新增）+ pre-fold 门禁 cargo tf 3338/3338 绿（tv 两件 master 预存
   同折叠①结论）+ master 合入 + worktree 重同步。

### W3 use 模块化（worktree 续；测试架构先于实现）

13. [✅ 已完成] **W3 测试架构先行**：corpus_use 六用例（001 定向导入/
    002 全限定 db.fn()/003 非 pub 可见性 D3/004 D2 合法环 A↔B 互递归/
    005 传递依赖+初始化序/006 通配）+ errors 三件（e1 未声明模块/e2 use.rs
    /e3 use.py）落盘;M4 多文件腿 compile_and_link_multi（session.resolve_
    uses+S4 wrapper 约定+池合并镜像+Linker）——**自证绿**：43 件单文件
    corpus 与 S4 compile_and_link 逐字符一致（execute 路径逐顶层语句发
    .line 的 Run-mode 分歧经自证发现并绕开,S4 口径为 aavm 镜像基准）;
    M5 多文件腿 run_with_capture_and_path live oracle + 错误通道（两侧
    错误文本一致判据）;lib.rs remap 四项自 execute 函数体内上提模块级
    pub(crate)（纯代码搬移,五闸回归绿）。aavm 侧红清单:M4/M5/错误通道
    三件均 Undefined symbol: codegen_dump_files / ev_run_files。宿主侧
    9 用例全部可编译/可执行/可报错。
14. [✅ 已完成] `auto/lib/parser.at`：parse_use 四形态 + 异构 use.rs/py kind
    Display + pub 前缀跳过；corpus_m2 p27（含 use.rs/py 显示件；{} 花括号
    形态经 :: 不可达——宿主实测,注记规格）。验证：tv-aavm2 M2 绿。
15. [✅ 已完成] 模块解析器（codegen.at cg_use_scan/cg_resolve_into）：
    File.read_text（bin 层 shim 前导保 a2r 可构建）+ 同目录相对解析 + 传递
    依赖递归 + 环静默跳过（D2）+ 按名去重；宿主错误文本三件（Module not
    found/Crate not declared/Python FFI——错误通道逐字符一致）。99_unit
    Auto 侧单测以 auto test 探针形式先行验证（verbose 通道）。
16. [✅ 已完成] `auto/lib/codegen.at`：多编译单元 cg_compile_mod（dep 裸
    形态：Type/Use 消费+顶层 var/const 限名全局+fn jmp-over+尾 HALT 无
    wrapper；主模块 wrapper 形态）+ pub 导出（D3:全 fn 导出）+ `mod.fn`
    限定符号（延迟调用 defers+const.i32 0 接收者占位镜像）+ 导入名作用域
    注入（use mod: item/通配→延迟符号）。验证：corpus_use M4 前置全绿。
17. [✅ 已完成] 链接器 cg_link_modules（缺省 codegen.at 内实现,未超限不
    升第七文件）：拼装（链接序 DFS 后序+main 末位）/jmp·Call 平移/池去重
    合并+remap（LoadStr/GetField/Load·Store.global 按 ins.n——序列化同步
    修正）/符号定址三级（bare 首到+mod.fn+strip 兜底）/mod_bases 初始化
    序锚点。验证：链接层用例（004 环/005 传递+初始化序）绿。
18. [✅ 已完成] `auto/lib/engine.at`：ev_run_files（ev_exec 核心自 ev_run_t
    抽出,全局区 gnames/gvals 共享传递）+ 初始化序（dep 链接序逐模块执行至
    各自 HALT 再 main 入口）；corpus_use 全量接入。验证：tv-aavm2 全绿
    （M4/M5 多文件腿+错误通道+单文件五闸+自证,16/16）。
19. [✅ 已完成] 折叠点③：divergence 登记（D-AA2R-struct 追记 30/37+
    预存债/D-stdlib-shadow/D-module-shape）+ pre-fold 门禁 cargo tf
    3338/3338 绿、cargo tv 仅 2 件 master 预存（同折叠①结论）+ master 合入
    + worktree 重同步。AA2R ignored 腿修复至可构建（W2 遗留的转译阻断
    清偿：File shim/借用拆分/括号优先级塌缩规避）。
20. [✅ 已完成] L3 资产固化：`test/vm/aavm2/99_unit/all_unit.at` 聚合套件
    （scripts/aavm2_unit_cases 四组 13 件：引擎微行为×4/struct 字段×2/
    D1 错误文本×2/模块解析×5 含 D2 合法环+错误通道三文本）+ 生成器
    scripts/gen-aavm2-unit.py（--check 同步校验）。D5 聚合方案（W0 定案）
    回写 midlang-w0-archaeology.md §5。验证：`auto test
    crates/auto-lang/test/vm/aavm2/99_unit` 13/13 绿。
21. [✅ 已完成] 文档回写：project.md（模块清单 511 增量/新闸门四件/能力
    矩阵+延后清单）、auto/lib/README.md（Plan 511 段）、divergences.md
    （W1/W3 两节）、KNOWN-DEBT P511 四条（AA2R 预存转译债/闸门口径偏差/
    b41 处置/占位泄漏镜像）、overview.md aavm 行注记。
22. [✅ 已完成·execution_done] 收尾验证（/auto-plan:review 为独立交接,
    本步骤按 auto-plan:work Step 6 口径执行 scoped 复核）:tv-aavm2 16/16
    （3 ignored=AA2R 腿,30/37 基线+7 件预存债见 KNOWN-DEBT P511）+
    `auto test crates/auto-lang/test/vm/aavm2/99_unit` 13/13 + 生成器
    --check 同步 + cargo tf 3345/3345 绿 + 收尾修复（StoreGlobal 定位后
    单次写——clone 双语义问题）。待 /auto-plan:review：验收标准 8 条逐条
    对代码、遗漏/延后扫描、spec-impact 元数据填写。

## 复审记录

**复审人**：ZCode（/auto-plan:review 独立复审）
**复审时间**：2026-09-01
**复审基线**：worktree `.worktrees/plan-511-dev` @ 9e217f812（= b08f354c7 + 复审修补提交，见发现①）；
折叠点①②③及 W1/W2/W3 语料已随 fold 在 master（3ca0fdb2a）。

### 验收标准逐条重验（verify, don't trust——全部本机重跑）

| # | 判据 | 结论 | 证据（本机实测） |
|---|---|---|---|
| 1 | tv-aavm2 全绿 + vm-files-ci 绿 | **pass（带披露豁免）** | 默认档 `-- test_aavm2`：**16 passed / 0 failed / 3 ignored**（3 ignored=AA2R ignored 腿×2+002 金样探针）。`--include-ignored` 下 AA2R 腿 30/37，7 件失败逐一核对=KNOWN-DEBT P511-1 登记的原列表（b13/b14×2/b19/b27/b29/b30），根因实证：b13 源文件头确有 CJK 注释（Unknown token 根因）、b27 源确为 `a.len()` 数组接收者跟踪债——均预存非 511 引入（pre-511 该腿整体转译不可构建，30/37 为改善，无绿→红破腿）。闸门口径收窄（计划 L1 定义含 --include-ignored）已登记于 P511-2，复审裁定：**接受豁免**——AA2R 扩面沿待澄清①缺省（暂缓+divergence），债有主、非静默。vm-files-ci 为 push 后 GitHub Actions 门禁，本环境（gitee 远端、无 gh）不可观测，依赖折叠点执行期簿记（步骤 7/12/19），合并后由 push 验证——登记为观察项 |
| 2 | TDD 过程证据（三波红先行+harness 自证留档） | **pass** | 红先行提交逐一核对：W1 fdf466c22（M2 p25 PARSE-ERROR/M3 t08/M4-M5 b34 红清单入提交文）、W2 9cd32793a（b38/b40/b42/b43 红清单）、W3 2d2dd2f50（自证绿+Undefined symbol 红清单）；`test_aavm2_m4_use_harness_selfcheck`（43 件单文件与 S4 管线逐字符一致）在 16 件绿清单中实跑通过 |
| 3 | ev_run_files 多模块程序 | **pass** | `test_aavm2_m4_use_corpus`/`m5_use_corpus`/`m5_use_errors` 全绿；错误通道 harness 实读（aavm2_m5.rs）：宿主 `run_with_capture_and_path` 错误文本 vs aavm `ev_run_files` stdout 双侧 `assert_eq`；宿主三件错误文本镜像落点核对 codegen.at:2489/2536（Module not found）、:2506（Crate not declared）、:2509（Python FFI）；六用例（定向/全限定/非pub/D2 合法环/传递依赖+初始化序/通配）全过 |
| 4 | 99_unit Auto 侧单测成建制 | **pass** | worktree 构建的 `auto test -d crates/auto-lang/test/vm/aavm2/99_unit`：**13 passed / 0 failed**；`gen-aavm2-unit.py --check` → "up to date" |
| 5 | 五方矩阵稳定集不破腿+新语料矩阵腿显式处置 | **pass** | 稳定集 b01–b33 的 ①②（宿主/aavm run）④⑤（双侧 disasm）腿=corpus M4/M5 闸绿；③ AA2R 腿 30/37 如 #1（预存债实证）；新语料处置=D-AA2R-struct（30/37+7 件债）divergence 登记 ✓ |
| 6 | W0 落盘+实现与规格一致抽查+回写 | **pass** | midlang-w0-archaeology.md 存在；抽查：D2 环=loading 栈命中静默跳过（codegen.at:2486–2539）与 W0 §6 定案一致；错误文本三件与规格一致；divergences.md「511 W1」「511 W3」两节在；project.md/overview.md 回写 diff 实核（模块清单 511 行/新闸门四件/能力矩阵+延后清单/overview aavm 行） |
| 7 | cargo tf 全量绿+零新增警告+无调试打印 | **pass** | **cargo tf 3345/3345 passed**（96 skipped，26.2s summary 实跑）；警告数 159 = master 基线 159，**零新增**；511 全量 diff（auto/lib+src）grep TODO/FIXME/dbg!/eprintln = **0 命中** |
| 8 | 延后项显式登记 | **pass** | OOP/闭包/嵌套 fn/泛型/May/生成器→计划 Out-of-Scope+project.md 延后清单 ✓；AA2R 扩面→待澄清①+P511-1 ✓；b41→D1+P511-3（L3 两件承载，99_unit 内实跑绿）✓；占位泄漏镜像→P511-4（Plan 437 已知泄漏镜像，宿主清偿需同步）✓；VBool 债 P474 预存未扩面 ✓ |

### 遗漏/延后/workaround 扫描发现

1. **发现（已当场修复）**：步骤 21 提交 fc303dae7 声称含 auto/lib/README.md 回写，实际未入提交
   （工作区遗留未提交修改）。复审以 fixup 提交 9e217f812 补入（内容与步骤 21 声称一致）。
2. **发现（已当场修复）**：worktree 工作区存在 100 个未提交的文件删除（test/comptime、
   test/generic、test/param_passing、test/ui/plan051*、scratch/* 等——HEAD 与 master 均有此
   些文件，判定为 worktree 重同步事故）。已从 HEAD 恢复后跑闸（否则 cargo tv 目录扫描会静默
   漏件、门禁数字虚高）。非计划行为，无代码影响。
3. **披露债（裁定接受）**：P511-2 闸门口径收窄（--include-ignored → 非 ignored 16/16 + tf）。
   AA2R 腿 pre-511 完全不可构建 → 511 修复至 30/37，属净改善；扩面沿待澄清①缺省暂缓且
   divergence 有登记。**不构成静默缩水**。
4. **workaround 扫描**：D3 pub 不过滤/D2 环静默跳过/const.i32 0 接收者占位均为「宿主为规范」
   镜像（各已入 divergence 或 KNOWN-DEBT），非绕行 hack；步骤 13 S4 wrapper 的 Run-mode
   .line 分歧在计划文本留档。未发现未登记的 scope 收缩或 hack。

### 复审裁定

**8/8 验收通过（#1 带披露豁免，已在 P511-2 留账）**，无未登记遗漏/延后/workaround。
status: execution_done → **reviewed**。spec-impact 元数据已按实际 diff 填写（见 frontmatter）。
**准予进入 /auto-plan:merge**（merge 时注意：vm-files-ci 观察项随 push 验证；worktree 内
docs/plans/511 旧拷贝与本文件无冲突——分支在 master..HEAD 范围未触碰该文件）。

## 待澄清事项

0. **W0 考古三项定案（已按「宿主为规范」原则裁定，详见
   midlang-w0-archaeology.md §6，如需改判请指出）**：
   - **D1 下标复合赋值**：宿主对 `a[i] += e` 直接编译错误（"Compound
     assignment requires a variable on left side"），无发射序可镜像——aavm
     同文本拒绝；b41 语料改入 L3 99_unit（断言错误文本），不进 corpus_m4。
     计划 W2 项 4「复合化」作废。
   - **D2 循环依赖**：宿主 loading_stack 命中即静默跳过（Plan 317 合法环），
     不报错——corpus_use「环」用例改为合法环可用（A↔B 互调），错误通道
     不含环。
   - **D3 pub 可见性**：宿主 VM 链路导出不过滤 pub（仅 a2r 转译器有 pub
     语义）——aavm 镜像为全 fn 导出；计划 W3 项 3「非 pub 不导出」作废。
   - **D5 L3 lib 引用**：聚合生成方案（lib 前置拼接 + #[test] 单文件），
     `use auto.lib.*` 不可行（lib 六文件互相依赖无 use 语句 + test 会话
     不播种源目录 + 入口非 pub，探针实证）。

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
