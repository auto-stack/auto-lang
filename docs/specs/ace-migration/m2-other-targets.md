# ace-migration: m2-other-targets — 其余A2X目标与独立转换工具接管

> Status: **977/PY 批已交付**（PLAN-977 T-01 + B-PY-I/V/R，2026-11-16 phase fold；
> lang `9fc1d2a34`）；**977/REVERSE 批已交付**（B-REVERSE-I/V，前置 M2A-08
> 收据 = PLAN-985 delivery `95c7fbad8`）。**GD/SCENE 批未交付**——V 腿需真实
> Godot headless 环境（本机缺席=缺证不 NA，批不半途开工）。本文件只记录
> **已验**接口与边界；未列入的能力不视为已支持。

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
| r2a 受检面（新增） | `transpile_r2a_checked`：产物含任一未支持形态标记即**具名拒绝**（`r2a: unsupported Rust forms (n): dyn-trait@L, macro-call@L …`，稳定键 20 类，见 `r2a_unsupported_forms` 分类表）；扫描只认注释位标记（字符串/字符字面量与 `//` 注释内同形文本不误报），`unimplemented!/todo!/unreachable!` 的显式 panic 翻译不计有损；Rust 语法错误仍是 `Rust parse error:` 诊断 | `cargo t auto-lang-core`（`trans::r2a::tests::p977_*`） |
| s2s 分类（Auto 脚本糖→正常模式改写） | 依赖旧**前端**（`parser`/`ast`），**不**依赖旧执行器与旧目标发射；旧产物面（`lower_source`）逐字节不变（identity/幂等/py 套件 round-trip 既有断言保持） | `trans::auto_s2s` |
| s2s 发射方言（新增） | `EmitDialect::{Legacy, AceCanonical}` + `emit_code_with_map`：`Legacy` 逐字节同 `emit_code`；`AceCanonical` 仅把 fn 签名的历史 `) -> ` 归一为 `) `（ACE `ret-ty` 只认裸 type word；字符串/字符字面量与 `//` 注释内不动） | `trans::emit`；`cargo t s2s`（`p977_s2s_*`） |
| s2s 行级 source map（新增） | `lower_source_with_map` / `lower_source_for_ace` → `(text, Vec<SourceMapEntry>)`；粒度 = 顶层语句起点行（`Code::source_lines`，与 A2C/A2R/A2PY 的 `Sink` 记录同口径），语句内逐行同形行细化为**精确原行**（结构行不参与），被规则改写的行回落语句起点；`source_line_for_output` 做产物行→原源行查询（精确优先、缺口回退最近前驱） | `trans::auto_s2s` / `trans::emit`；`p977_s2s_ace_dialect_is_canonical_and_mapped`、`p977_s2s_map_falls_back_to_statement_start_on_rewrite` |
| ACE 执行（V 证据，非 emit-only） | s2s 规范产物经 `run_ace(AceAV)`（M2 唯一执行器）真实执行：递归 `fib(10)=55` 对**独立手算** oracle；历史方言（带 `->`）在同一门是 `text/source.syntax` **具名拒绝**（规范方言是可执行前提）；绑定期诊断 span 经 source map 回到原源**精确行**（`return add(1)`）；`autovm/vm/evaluator` 引擎名具名拒绝——本链无旧执行器回退 | `crates/auto-lang-ace-host/tests/p977_reverse_s2s.rs`；`cargo t ace-facade` |

## 977/REVERSE 边界（现值开放项）

- **不要求任意语言互逆**：r2a 只映射已支持形态；受检面把未支持形态拒在工具边界
  （宽松历史面仍返回有损注释产物——公开命令不因受检面改变）。有损注释本身不是
  可执行语义，受检面是"不冒充支持"的机制。
- source map 粒度 = 语句起点行 + 同形行细化：被规则改写的那一行回落语句起点
  （不猜近似位置）；嵌套块内的改写行不做逐 token 定位。该口径与 A2C/A2R/A2PY
  的记录粒度一致，不宣称精确 token 级。
- s2s→ACE 只覆盖 ACE 当前 source profile 接受的子集（纯 Auto 标量/记录面）；
  `use.py` 桥面仍归旧脚本管线（Python 目标面见 977/PY），本批不把 py 桥搬进 ACE。
- 平台：Windows x64 实证；Linux 腿按 r8 裁定归 1063。
- GD/SCENE 与全包收口（T-05/06/07）状态见计划 §8；本文件在 T-05 复审通过后随
  T-06 fold 落主，不提前签全包。
