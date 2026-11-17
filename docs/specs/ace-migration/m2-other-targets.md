# ace-migration: m2-other-targets — 其余A2X目标与独立转换工具接管

> Status: **977/PY 批已交付**（PLAN-977 T-01 + B-PY-I/V/R，2026-11-16 phase fold；
> lang `9fc1d2a34`）；**977/REVERSE 批已交付**（B-REVERSE-I/V，前置 M2A-08
> 收据 = PLAN-985 delivery `95c7fbad8`），含六路径覆盖（V-COVERAGE）与 Linux 移交
> 审计（V-LINUX-PHASE）。**GD/SCENE 批未交付**——V 腿需真实 Godot headless 环境
> （本机缺席=缺证不 NA，批不半途开工）。本文件只记录**已验**接口与边界；未列入的
> 能力不视为已支持。

## 977/PY 已验接口（旧 a2p 公开面 → ACE Python 车道）

| 面 | 接口/事实 | 来源 |
|---|---|---|
| 车道 | Checked HIR → `ace-a2x-python`（AXPY1 Scalar / AXPY2 RecordText / AXPY3 Collections，1042 交付，无第二后端）→ 版本化 runtime → 真实 OS Python 3.14.2（`python -I -B a2x_main.py`）；观察面 = stdout `value:<n>`（bool→0/1）/ `trap:<code>`；短路 trace 走 stderr marks | `experimental/ace/crates/a2x-python` + `a2x-worker` pyrun |
| **路由事实（B-PY-I 冻结）** | 条件循环 `for <cond>`=Scalar；模板 `<T>`=Scalar+`ac.source.instances.scalar/v1`（935 脱糖；caps-blind trim 与 packages binder 拒 `<` 为具名路由事实）；records/方法（`static fn`/隐式 self `.field`）/strings=TextRecord；enum/is-表达式/`??`=Aggregates；**List=PLAN-946 aggregates packages 线**（`PackageA2xTarget::cold_aggregates` + `use std.list.*`）——每构造皆有已准入路由，零车道缺口 | `a2x-worker/tests/cases/p977_py_public_surface.rs` |
| 永久验证（crate 级） | `p977_legacy_public_surface_runs_green_on_python`：9/9 旧语料家族（if/for、func、generics、methods、enum、str、struct、array/List、option/is）真实 Python 执行，独立手算 oracle（≤10s 门） | `cargo t ace-a2x` 组 |
| 永久验证（case 线） | 跟踪语料 `crates/auto-lang/test/cases/ace-m2/py-977`（8 检查：value 14 / generic 42 / 短路 value:7+stderr `mark_b,mark_a` / trap:Overflow / trap:DivByZero / str-record 10 / enum-is 141 / methods 43）经 `ac-a2py/hir-stdlib-run-v1`（`ac_a2py_hir` 适配器 → 预构建 a2x-worker `run-py` → 真实 Python，父侧独立裁判，worker 不见期望）；门 `ace_m2_p977_py_case_line_green` + `ace_m2_p977_py_wrong_oracle_is_red`（stdout 轴 RED、局部化） | `experimental/ace/tests/conformance/tests/host/language_conformance.rs` |
| 观察五轴 | 返回（value/generic）/ 输出（stdout 面）/ 短路（stderr marks 确定性序）/ 异常（trap:Overflow、trap:DivByZero）/ 资源（record-text 显式生命周期值面；零泄漏 trace 证在 1042 `pyready_02`） | 语料 + 1042 |
| 坏输入负控 | 本注册 wrong-oracle RED；准入负控（emit 冒 run / 旧 PythonTrans / AutoVM 回退 / expected 注入 / 缺 Python = 拒）= 1042 `pyready_04_admission_negative_controls` | `ac_a2py_hir` 准入规则 |

## 977/PY 边界（现值开放项）

- 旧 a2p 99 例**字节金样**属旧实现（legacy `trans/python.rs` 测试面不动，`cargo t a2py`）；新面按**家族代表 + 路由台账**承接公开义务——逐例移植语料可持续扩充（每例机械映射到已冻结路由），不以代表例宣称逐例覆盖。
- case 线单文件：List/packages 腿只在 crate 级套件（多模块经 `compile_package_product_aggregates`）。
- 预构建纪律：case 线需 `target/release/a2x-worker.exe`（显式预构建阶段，缺 = 具名 Error 非 PASS，测试内不编译）。
- 平台：Windows x64 + Python 3.14.2 实证；Linux 真 Python runner 取证归 1063（r8 裁定）。
- GD/SCENE 未交付（缺真实 Godot headless 环境）；REVERSE 见下节；全包收口
  （T-05/06/07）状态见计划 §8。

## 977/REVERSE 已验接口（r2a 独立逆向工具 + s2s 源码改写 → ACE 执行）

前置收据：M2A-08 正式成立（PLAN-985 A 批 phase fold，delivery `95c7fbad8`）；
本批在 M2A-08 之后开工，A 切片证据不重记。旧公开命令面**逐字节保留**。

| 面 | 接口/事实 | 来源 |
|---|---|---|
| r2a 分类（Rust→Auto 独立逆向工具） | 中立 `syn` 解析 + 自有文本发射：**不**消费 Auto 前端/AST，不经旧执行器（AutoVM）也不经旧目标发射器；公开命令 `transpile_r2a_file`（CLI `trans r2a`）行为不变——37 例 golden 语料逐字节、round-trip 断言无 `/* unknown */` | `crates/auto-lang-core/src/trans/r2a.rs` |
| r2a 受检面（新增） | `transpile_r2a_checked`：产物含任一未支持形态标记即**具名拒绝**（`r2a: unsupported Rust forms (n): dyn-trait@L, macro-call@L …`；`r2a_unsupported_forms` 分类表 = **17 个具名键 + 1 个兜底 `lossy-marker`**）；扫描只认注释位标记（字符串/字符字面量与 `//` 注释内同形文本不误报），跨行/未闭合标记 fail-closed，`unimplemented!/todo!/unreachable!` 的显式 panic 翻译不计有损；Rust 语法错误仍是 `Rust parse error:` 诊断 | `cargo t auto-lang-core`（`trans::r2a::tests::p977_*`） |
| s2s 分类（Auto 脚本糖→正常模式改写） | 依赖旧**前端**（`parser`/`ast`），**不**依赖旧执行器与旧目标发射；旧产物面（`lower_source`）逐字节不变（identity/幂等/py 套件 round-trip 既有断言保持） | `trans::auto_s2s` |
| s2s 发射方言（新增） | `EmitDialect::{Legacy, AceCanonical}` + `emit_code_with_map`：`Legacy` 逐字节同 `emit_code`；`AceCanonical` 仅把 fn 签名的历史 `) -> ` 归一为 `) `（ACE `ret-ty` 只认裸 type word；字符串/字符字面量与 `//` 注释内不动） | `trans::emit`；`cargo t s2s`（`p977_s2s_*`） |
| s2s 行级 source map（新增） | `lower_source_with_map` / `lower_source_for_ace` → `(text, Vec<SourceMapEntry>)`；粒度 = 顶层语句起点行（`Code::source_lines`，与 A2C/A2R/A2PY 的 `Sink` 记录同口径），语句内按序做**两级细化**：先精确同行、再 ≥6 字符子串命中（如单行 `if c { return n }` 内的 `return n`），结构行（以 `{` 收尾/`}`/`else` 片段）不参与，未命中者回落语句起点；`source_line_for_output` 做产物行→原源行查询（精确优先、缺口回退最近前驱） | `trans::auto_s2s` / `trans::emit`；`p977_s2s_ace_dialect_is_canonical_and_mapped`、`p977_s2s_map_falls_back_to_statement_start_on_rewrite` |
| ACE 执行（V 证据，非 emit-only） | s2s 规范产物经 `run_ace(AceAV)`（M2 唯一执行器）真实执行：递归 `fib(10)=55` 对**独立手算** oracle；历史方言（带 `->`）在同一门是 `text/source.syntax` **具名拒绝**（规范方言是可执行前提）；绑定期诊断 span 经 source map 回到原源**精确行**（`return add(1)`）；`autovm/vm/evaluator` 引擎名具名拒绝——本链无旧执行器回退 | `crates/auto-lang-ace-host/tests/p977_reverse_s2s.rs`；`cargo t ace-facade` |

## 977/REVERSE 边界（现值开放项）

- **不要求任意语言互逆**：r2a 只映射已支持形态；受检面把未支持形态拒在工具边界
  （宽松历史面仍返回有损注释产物——公开命令不因受检面改变）。有损注释本身不是
  可执行语义，受检面是"不冒充支持"的机制。
- **受检面的可达面=库级**：`transpile_r2a_checked` / `r2a_unsupported_forms` 是
  `auto-lang-core` 的公开库接口；CLI（`trans r2a`）当前仍只走宽松
  `transpile_r2a_file`，命名开关/报错面属 **982/REVERSE 批**（Man 命令批次）的
  接线义务。本条是具名边界，不是"CLI 已支持拒绝"的声明。
- source map 粒度 = 语句起点行 + 两级同行细化（精确 → ≥6 字符子串）：被规则改写
  且不再是任何原行子串的那一行回落语句起点（不猜近似位置）；嵌套块内的改写行不做
  逐 token 定位。该口径与 A2C/A2R/A2PY 的记录粒度一致，不宣称精确 token 级。
- s2s→ACE 只覆盖 ACE 当前 source profile 接受的子集（**纯 Auto 标量面**；
  记录/聚合面的 s2s→ACE 未在本批验证），`use.py` 桥面仍归旧脚本管线（Python
  目标面见 977/PY），本批不把 py 桥搬进 ACE。
- 平台：Windows x64 实证（ACE 宿主桥 `auto-lang-ace-host` 当前 `#![cfg(windows)]`）；
  Linux 腿按 r8 裁定归 1063，逐项审计见下文 §977 Linux 移交审计。
- GD/SCENE 与全包收口（T-05/06/07）状态见计划 §8；本文件在 T-05 复审通过后随
  T-06 fold 落主，不提前签全包。

## 977 六路径覆盖（V-COVERAGE / AC-COVERAGE，2026-11-17）

同一公共 Auto 源 `fn main() int { return 2 + 3 * 4 }`（expected 14，手算；字节即
ACE-M1 B01 源，`source_sha256 b7e5f25e…`）逐路径实跑或发射；不变式：每路径都有真实
producer 身份、独立可判 observation（值/字节）与具名负控，不以"后端相等"或 emit
冒充 run。

| 路径 | producer / 车道 | 观察面与期望 | 性质 | 证据（固定组/门） |
|---|---|---|---|---|
| AV/native | AV executor，`ac-av/core-run-v1` | exit = 14 | **run** | `cover-977` 检查 `av-native`；`cargo t ace-host --test-name ace_m2_p977_cover` |
| AC | AC native 后端，`ac/mir-native-scalar-v1` | exit = 14 | **run** | `ac-native`（同上） |
| A2PY | `ace-a2x-python`，`ac-a2py/hir-stdlib-run-v1` | stdout `value:14\n`（hex `76616c75653a31340a`） | **run**（真 OS Python） | `a2py-run`（同上） |
| A2R | `ace-a2x-rust`，`ac-a2r/hir-scalar-emit-v1` | 4 文件字节 + manifest sha（`0c2257b0…`），worker/recipe 身份回显 | **emit lock** | `a2r-emit`；**certify 轴**=真 rustc：`p977_cover_a2r_certify_real_rustc`（actual+手写 reference 双跑，stdout hex 相符，0.65s） |
| A2C | `ace-a2x-c`，`ac-a2c/hir-scalar-emit-v1` | 4 文件字节 + manifest sha（`28559f86…`），worker/recipe 身份回显 | **emit lock** | `a2c-emit`；**certify 轴**=真 MSVC cl：`p977_cover_a2c_certify_real_cl`（1.88s） |
| AV/C | C provider（`use.c`，真 DLL 机器码） | provider 操作值（独立手算） | **run** | `ace-abi`：`av_runs_the_c_provider_through_a_real_dll`（0.55s） |
| AV/Rust | Rust provider（`use.rs`） | UTF-8 字节长 9（`"A中😀\0"`） | **run** | `ace-abi`：`f0_av_calls_real_rust_provider_end_to_end`（0.22s） |
| 原 A2C（legacy） | 旧 C 目标语料（134 例，`#[ignore]` 历史档） | 冻结 goldens（分母不减） | **不计覆盖 PASS**：`cargo t a2c` 选 0（全 `#[ignore]`）；显式 `--ignored` 实跑 117/134，17 例既有红（源与本批未触面一致） | 归属=legacy 退役线（987）；required 的 C 路径由 ACE A2C（emit+certify）承担 |

**负控（每条都可判）**：
- wrong-oracle RED：exit 14→15 → `av-native` 红并命名 exit 轴，sibling `ac-native` 保持绿
  （`ace_m2_p977_cover_wrong_oracle_is_red`）。
- **emit 冒 run 拒绝**：给 a2r emit 腿加 exit 期望 → 具名拒绝（`a2r/a2c emit checks
  declare artifact expectations only …`）；此前 adapter 会静默忽略该期望，本轮补守卫
  （`ace_m2_p977_cover_emit_cannot_masquerade_as_run`）。
- 缺 Python/helper、旧 PythonTrans/AutoVM 回退、expected 注入 = 1042
  `pyready_04_admission_negative_controls`（本批复跑）。
- 错 provider 撤销/销毁 + 引擎回退 = `auto-lang-ace-host` e0（`ace.provider/revoked`、
  `autovm/vm/evaluator` 具名拒绝，`cargo t ace-facade`）。
- NUL/部分 IO/trap/资源 = py-977 trap 轴（Overflow/DivByZero）+ 1042 `pyready_02`
  零泄漏 trace + AV resource 事件腿。

**边界（具名）**：provider 腿的源必须声明 provider（`use.c`/`use.rs`），不可能与纯源
同字节——这是源的形状约束，不是覆盖缺口；A2R/A2C 对本源的"运行"由 certify 轴
（真 rustc/cl + 手写 reference）证明，日常车道只声明发射锁；Linux 全腿
`NOT_RUN → 1063`（见下节）。

## 977 Linux 移交审计（V-LINUX-PHASE / AC-LINUX-PHASE）

口径（PLAN-977 r8 + 1063）：本批在 Windows 上写代码并跑全部 Windows required 门；
**Linux 真实运行一律 NOT_RUN（owner = 1063）**，不填 PASS/NA、不删分母、不以
Windows/emit-only/交叉编译冒充 Linux 绿。下表逐项给出实现面、完整输入/expected、
当前状态与 1063 去向。

| 项 | 实现/触面 | 完整输入与独立 expected | Linux 状态 | 1063 去向/备注 |
|---|---|---|---|---|
| Python 车道（A2PY） | `experimental/ace/crates/a2x-python`（AXPY1..3 + ports/runtime）＋`a2x-worker run-py` | `test/cases/ace-m2/py-977` 8 检查（值 14/generic 42/短路 marks/trap Overflow/DivByZero/str-record 10/enum-is 141/methods 43）＋crate 探针 9 族；父侧独立裁判 | `deferred-required / NOT_RUN` | 1063：Linux python3 runner、`python -I -B` 路径与実机 Python 版本取证 |
| Python 标准库端口 | `a2x-python/ports.rs` 按操作 ID 绑定 | 同上语料中 std.math/std.text 腿；1042 `pyready_02/04` 资源与准入负控 | `deferred-required / NOT_RUN` | 1063：Linux 操作级逐行重跑（不继承 Windows 绿） |
| r2a 独立工具 + 受检面 | `crates/auto-lang-core/src/trans/r2a.rs` | 37 例 golden 逐字节 + round-trip + `p977_r2a_*`（含拒绝/字面量/未闭合负控） | `deferred-required / NOT_RUN` | 1063：同 corpus 在 Linux 重放（无平台条件代码，但未实测不记 PASS） |
| s2s 改写器 + 发射方言 + source map | `trans/auto_s2s.rs`、`trans/emit.rs` | identity/幂等/py 套件 round-trip + `p977_s2s_*`（map 总性/精确行/字面量保护） | `deferred-required / NOT_RUN` | 1063：`cargo t s2s` 在 Linux 重放 |
| s2s→ACE 执行证据 | `crates/auto-lang-ace-host/tests/p977_reverse_s2s.rs`（crate 现 `#![cfg(windows)]`，ACE abi provider 腿仅 Windows） | `fib(10)=55` 独立手算；历史方言 `text/source.syntax` 具名拒绝；bind 诊断经 map 回原源第 6 行；`autovm/vm/evaluator` 拒绝 | `deferred-required / NOT_RUN` + **平台边界具名** | 1063：Linux 宿主桥（`ace-abi` provider 腿）尚不存在，属平台能力缺口，不是本批遗漏；不得记 NA |
| 六路径覆盖 `cover-977` | case 语料 + conformance 门 + 发射锁 | 同一公共源 expected 14：av-native/ac-native（run）、a2py-run（真 Python）、a2r/a2c 发射字节锁；AV/C、AV/Rust provider 腿绑 ace-abi F0；legacy A2C 绑 `a2c` 组 | `deferred-required / NOT_RUN` | 1063：Linux 上按六路径 + A2C 重跑（含 MSVC→clang/cl 工具链差异，逐腿记 NOT_RUN 原因） |
| 原 A2C 语料（legacy C 目标） | `crates/auto-lang/src/tests/a2c_tests.rs`（134 例，全部 `#[ignore]`；本批未改） | 冻结 goldens；显式 `--ignored` 实跑 Windows 117/134（17 既有红，与本批无关） | `deferred-required / NOT_RUN`（legacy 档；非 required 覆盖路径） | 1063：Linux 上同档重放；required C 路径由 ACE A2C 承担 |

**Windows required 门（本批实跑，见计划 §8/§9 收据）**：`cargo t s2s`、`cargo t
auto-lang-core`（1653/1654，1 基线红具名非本批触面）、`cargo t ace-facade`、
`cargo t ace-adapters`、`cargo t ace-host --test-name ace_m2_p977_cover`（覆盖门 3/3）、
`cargo t ace-a2x --test-name p977_cover_certify`（A2R/A2C 真编译运行 2/2）、
`cargo t ace-a2x --test-name pyready_04`（准入负控）、`cargo t ace-abi --test-name
av_runs_the_c_provider_through_a_real_dll`、`cargo t ace-abi --test-name
f0_av_calls_real_rust_provider_end_to_end`、`cargo t case-a2c-hir-scalar-921`、
`cargo t case-a2r-hir-collections-904`、`cargo t a2c`（选 0=legacy 全 ignore，另记
`--ignored` 117/134 既有红）。以上 Windows 结果不得外推为 Linux 已验证。
