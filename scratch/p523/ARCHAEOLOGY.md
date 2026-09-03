# Plan 523 W0 考古记录（确定性 + 逐族发射 + 载体定案）

- 日期：2026-09-03；基线 master `ac54037b9`（Plan 442 归档合并后）
- 工具：`target/release/auto.exe` 于 HEAD 重建（P517-2 纪律；2026-09-03，2m27s）
- 探针脚本：`scratch/p523/w0/determinism_probe.sh`；产物留档：
  - `det/<case>/run1.rs` —— 主 a2r 单文件转译产物（27 件规范产物）
  - `arch/<case>.rs` + `.out` —— b34–b43 产物 + rustc 运行输出（OK 件）
  - `merge/run1.rs` —— merge 转译（剥 use 的 auto/lib 七件套目录模式）产物
  - `probe/*.at` —— H 系根因最小复现语料

## 1. 主 a2r 发射确定性 ✅ 全绿

| 面 | 样本 | 次数 | 结果 |
|---|---|---|---|
| 单文件转译 | b34–b43（现存 9 件，b41 无文件）+ corpus_a2r g01–g18（18 件） | 每件 5 次独立进程 | 27/27 逐字节一致 |
| merge 转译（compile corpus 同机制：目录模式 + 剥 use 七件套） | auto/lib v2 | 3 次独立进程 | 406,784 字节逐字节一致 |

**结论：无非确定位。** 计划风险表「HashMap 迭代序非确定」在现语料面
不成立——rust.rs 中的 HashMap/HashSet 均不参与发射序（发射序由 AST 序
与固定 dep_order 决定）。金样可直落，无需 M4 式规范化前置。

## 2. b34–b43 逐族考古（主 a2r 转译 → rustc --edition 2021 编译运行）

| 语料 | 主 a2r 产物 | rustc | 运行输出 |
|---|---|---|---|
| b34_struct_basic | `arch/b34_struct_basic.rs` | ✅ | 10/20 |
| b35_struct_field_rw | `arch/b35_struct_field_rw.rs` | ❌ H1 | — |
| b36_struct_nested | `arch/b36_struct_nested.rs` | ❌ H2 | — |
| b37_struct_in_fn | `arch/b37_struct_in_fn.rs` | ✅ | 7/14/-2 |
| b38_for_in_arr | `arch/b38_for_in_arr.rs` | ❌ H3 | — |
| b39_str_index | `arch/b39_str_index.rs` | ❌ H4 | — |
| b40_neg | `arch/b40_neg.rs` | ✅ | -7/-3/-10/7 |
| b42_globals | `arch/b42_globals.rs` | ❌ H5 | — |
| b43_global_shadow | `arch/b43_global_shadow.rs` | ❌ H5 | — |

三件绿（b34/b37/b40）即 struct 声明/注解构造/字段读/一元负的主 a2r
发射已成型；六个红全部定位到主 a2r 侧五个洞（H1–H5，见 §3）。

## 3. 主 a2r 洞登记（W1 顺修清单，宿主为规范）

### H1：字段名撞内置方法名 → 误发射方法调用（b35）
- 现象：`c.count`（c 为含 `count` 字段的 struct 实例）发射成 `c.count()`，
  连赋值左端都带括号（`c.count() = c.count() + c.step`）。
- 根因：rust.rs 字段访问位的 `is_rust_method` 名单
  （len/is_empty/capacity/**count**/push/pop）无条件加 `()`；433 A1 已有
  字段优先护栏 `object_has_such_field`，但其查 `local_var_types`——
  **未注解 `let` + struct 构造字面量初始化不登记类型**，护栏落空。
  （注解形态 `var d Counter = ...` 命中护栏，发射正确。）
- 复现：`probe/f1.at`（同函数内注解/非注解对照）。
- 修法（W1）：let/var 初始化为 struct 构造字面量 `Name { ... }` 且 Name
  在已知 struct 表时，登记 `local_var_types[name] = User(td)`（rust.rs
  let 发射位，镜像 433 A1 注释风格）。

### H2：嵌套 place 写不标 mut（b36）
- 现象：`o.first.v = ...`（两级）后 `o` 未发射 `let mut`；一级
  `t.v = 8` 正确发射 `let mut t`。
- 根因：mutation 标记只覆盖一级 place（机制在 scan/W1 复核定位，
  scan_mutated_bindings 仅收 mutating 方法调用；一级命中来自另一路径）。
- 复现：`probe/m1.at`（同函数一级/两级对照）。
- 修法（W1）：mutation 扫描对 Store 赋值目标取 place 根标识符（任意
  深度 Dot 链），与现有 mutating-method 集合并。

### H3：无返回类型注解的 fn 不推断（b38）
- 现象：`fn mk() { return [7, 8, 9] }` 发射 `fn mk()`（unit），
  `for w in mk()` rustc E0308。
- 根因：主 a2r 对未注解 fn 零返回推断（`probe/r1/r2/r3.at` 三形态全
  unit）。注解形态（b37 `fn sum(p Point) int`）正常。
- 修法（W1）：fn 发射位对 ret=Unknown 的函数扫描 body 的 `return` 表达式
  与尾表达式，用现成 `infer_type_from_expr` 推型（int/str/bool/数组
  字面量/struct 构造/调用链），得非 unit 型则发射返回类型注解。

### H4：字符串下标直发 `s[i]`（b39）
- 现象：`s[0]` 发射 `s[0]`（String 不可整数下标，E0277）。
- 语义基准：v2 engine（aavm 镜像）GetElem VStr 臂 =
  `chars().nth(j).unwrap_or('\0') as i64`（码点值；b39 期望输出
  65/66/67/133）。
- 修法（W1）：下标读发射位按接收方 str 型分流：
  `(s.chars().nth((i) as usize).unwrap_or('\0') as i64)`。
- 负下标回绕（VM：j<0 → len+j）不在语料面，登记差异不实现。

### H5：全局变量发射依赖 once_cell（b42/b43）
- 现象：顶层 `var count int = 100` 发射
  `static COUNT: Lazy<Mutex<i64>> = ...` + `use once_cell::sync::Lazy;`
  ——独立 rustc 产物无外部 crate，E0433。merge 语料（auto/lib 无全局）
  从不触达，故 compile corpus 未暴露。
- 修法（W1）：字面量可 const 初始化的全局（int/bool/float 字面量 →
  `static X: Mutex<T> = Mutex::new(lit);`，std-only，Rust 1.63+ const；
  str 字面量 → `static X: Mutex<&str> = Mutex::new("lit");`）改为直发
  static Mutex，访问位 `*X.lock().unwrap()` 形态不变；非字面量初始化
  保留 Lazy（登记差异，中阶语料不覆盖）。访问位/复合赋值改写位
  （`{ let __a2r_gv = ...; *X.lock().unwrap() = __a2r_gv; }`）不动。
- 连带：`test/a2r/14_modules/007_shared_var` 金样同步再生成
  （COUNTER 变直发 static Mutex；APP_NAME str 走 &str 静态）。

## 4. 三件套格式定案（cookbook 模板裁剪）

```text
test/vm/aavm2/corpus_a2r/<nnn_name>/
├── <name>.at            # ① 语料源
├── <name>.expected.rs   # ③ 金样：主 a2r 转译产物（bless 再生成）
└── <name>.expected.out  # ② 金样：参考执行输出（bless 再生成）
```

- 命名/布局对齐既有两约定：a2r 金样（per-case dir + `<name>.expected.rs`，
  `test_runner.rs` 同构）+ cookbook（`expected.out`）。**去掉 cookbook 的
  `reference.rs` 运行夹具位**（② 由 Rust 参考 live 承担）；保留扩展口：
  三件之外可挂 `<name>.files/` 等附属（corpus_use 多文件扩展位）。
- 存量 g01–g18 保持平铺单文件（live 对拍不动）；新件一律 per-case dir。
  既有平铺 walker（`extension()=="at"`）不进目录 → **新件落盘不破 master
  保护网**，红证由 W1 扩展 walker + AA2R 实现同 worktree 落地转绿。

## 5. bless 工作流定案

- 校验（默认）：Rust 测试件 live 对拍金样（transpile_rust vs
  expected.rs；run_with_capture vs expected.out），失配写
  `<name>.wrong.rs/.wrong.out`（镜像 a2r 金样 .wrong.rs 先例）。
- 再生（bless）：env `A2R_BLESS=1` 时测试件以 live 输出覆写金样并打印
  `BLESSED <path>`（镜像 cookbook `GENERATE_EXPECTED` 先例）；产物走
  git diff 评审。
- g19–g25 金样在 W1 主 a2r 洞修复**之后**bless（否则锚住带洞输出）。

## 6. runner 载体定案（待澄清② 裁定）

**Rust 测试件 + Python 薄壳（CI/本地双形态）**——按计划缺省倾向：

- Rust 侧：`--features test-vm-files` 下新增四路 runner 测试件
  （path1 VM 内 ev_run / path2 VM 内 ar_run / path3 aavm2_bin 执行 /
  path4 AA2R on 自举 bin 译文；+ 译文回链 rustc 编译运行对拍 + 主 a2r
  同锚 ③ + 逐用例判定表 eprintln）。`#[ignore]`（需 cargo/rustc），
  验收/折叠点 `-- --ignored --nocapture` 跑全量。
- Python 薄壳：`scripts/aavm4_check.py <case-dir> [--all] [--bless]` ——
  组装/调用 Rust 测试件并转发环境变量，输出判定表；`--bless` 透传
  `A2R_BLESS=1`。
- path4 形态：aavm2_bin 复用 compile corpus 内容寻址缓存，harness 增加
  ar_run 驱动模式（读文件 → `ar_run(&source, 0)` 打印译文），W3 落地时
  定夺开关形态。

## 7. AA2R 红证清单（W0-2，g19–g25，2026-09-03 实跑）

方法：诊断 worktree 一次性测试件（参考宿主 lib+`ar_run` 驱动，与
`test_aavm2_a2r_is_corpus` 同管线），7/7 MISMATCH：

| 件 | AA2R 现状（vs 主 a2r） |
|---|---|
| g19_struct_decl | `PARSE-ERROR:7: expected end of statement, got <LBrace>`——**构造字面量 `Point { ... }` 表达式不存在**（struct 声明本身 ar_prescan_type/emit 已有） |
| g20_struct_ctor | 同上（ctor 在 fn 返回位同样炸） |
| g21_field_rw | 同上（16 行 ctor） |
| g22_for_in_arr | 三差异：`for n in &vec![1,2,3,4]`（主 a2r 无 `&`）/ `for w in &mk()`（主 a2r 无 `&`）/ `fn mk()` 同缺返回推断（H3 同款） |
| g23_str_index | `s[0]` 直发（与主 a2r 当前同病；H4 修复后 AA2R 需镜像 chars().nth 形态） |
| g24_neg | `println!("{}", -x + y)`——**一元负括号丢失**（主 a2r 发 `-(x + y)`，AA2R 前缀臂优先级/括号缺失） |
| g25_globals | **全局变量零发射**：顶层 `var count int = 100` 无 static 产物，体内 `count` 裸标识符直发（不可编译）；H5 修复后 AA2R 需镜像 std static Mutex 形态 |

## 8. 附加考古发现（计划外，影响 W2/W3 设计）

### 8.1 m5 语料腿 master 既有红（归属定位）
- 现象：`test_aavm2_m5_engine_corpus` 于 HEAD 红——b38 第三循环
  （`for w in mk()`）参考侧输出完整、aavm 侧缺失。
- 二分实跑：基线 `d54ec540d`（517 终态）绿 → `77c4a5306`（057 T2）绿 →
  `e01eeba0b`（**057 T3 for-in Call 源通道泛化**）红。
- 根因链：511 W2 时 v2 engine.at 的 nat#112 镜像了宿主当时的缺陷
  （裸 List 句柄零迭代，`auto/lib/engine.at:688` 注释自证），v2
  codegen.at `cg_for` 对裸 Ident 调用源仍走该迭代器通道；T3 把宿主
  修对后双侧分歧。auto-down 折叠（f333a21ad）亦登记「m4/m5 corpus
  master 同红存量」。
- **修法（并入本计划 W2 红项根因修复）**：`cg_for` 的裸调用特例并回
  通用「数组句柄+索引循环+GET_ELEM」通道（511 W2 已建），一处修复
  同时转绿 m5 腿与 compile corpus b38（摘前缀后）。

### 8.2 v2 codegen 尚不能编译 lib 自身（path4 设计约束）
- 实证：lib+driver 程序喂 aavm2_bin → `CODEGEN-ERROR:.len() receiver is
  not an array`（`codegen.at:1232`，arr_flag 未知的 `.len()` 接收者）。
- 含义：W3 path4（AA2R self-bin）**不可**走「bin 执行 lib 前缀驱动」
  形态（那要求 v2 自举编译）；须由 bin 的 Rust harness 直调
  `ar_run(&source, 0)`（lib 已转译入 bin，仅语料串经 v2 编译）。
  v2 自举编译缺口（`.len()` 未知接收者族）登记为远期债，不在本计划。

## 9. 待澄清事项裁定回填

- ①迁移范围：缺省执行——新件 g19–g25（7）+ 抽验集 b07/b13/b32/b33 +
  b34/b36/b42（中阶代表 3）共 14 件三件套；全量存量迁移不做（W3 复核
  成本后如超限按计划登记分批）。
- ②runner 载体：见 §6（Rust 测试件 + Python 薄壳）。
- ③aavm.at a2r 模式口径：维持「转译产物 exe 直接跑」缺省；524 若先行
  则切位置参数形态（W4 实测时按 524 落地形态联动，本计划不动 CLI）。
