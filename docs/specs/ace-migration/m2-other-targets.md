# ace-migration: m2-other-targets — 其余A2X目标与独立转换工具接管

> Status: **977/PY 批已交付**（PLAN-977 T-01 + B-PY-I/V/R，2026-11-16 phase fold；
> lang `9fc1d2a34`）。**GD/SCENE 批未交付**——V 腿需真实 Godot headless 环境（本机
> 缺席=缺证不 NA，批不半途开工）；**REVERSE 批及收口任务属 B**（M2A-08 后）。
> 本文件只记录**已验**接口与边界；未列入的能力不视为已支持。

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
- GD/SCENE/REVERSE 与全包收口（T-05/06/07）未交付，前置如上。
