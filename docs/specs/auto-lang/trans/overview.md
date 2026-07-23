# trans（多目标转译器）

> **Status**: implemented

## 职责

把 Auto AST 转译为目标语言源码：C（a2c）、Rust（a2r）、TypeScript（a2ts）、
Python（a2p）、JavaScript（a2j）、GDScript（a2gd）及 Godot 场景（tscn），
外加逆翻译 r2a（Rust → Auto，基于 `syn` 解析）。

## 现状

统一抽象是 `Trans` trait + `Sink` 输出缓冲（含 source map），各后端实现同一接口。
规模与成熟度（按代码行数，`crates/auto-lang/src/trans/`）：

| 后端 | 文件 | 行数 | 状态 |
|------|------|------|------|
| a2r | `rust.rs` | 13842 | 最复杂：字符串所有权映射 + 逃逸分析 + 20+ 后处理 pass |
| a2c | `c.rs` | 4533 | 成熟，C99 输出，支持单态化泛型、`use c <header>` FFI |
| a2p | `python.rs` | 2702 | 成熟，含 import 收集、PyDep 依赖跟踪 |
| r2a | `r2a.rs` | 2633 | 逆翻译，`syn` 解析 Rust 源码生成 .at |
| a2gd | `gdscript.rs` | 2072 | 完整（plan-290），Tab 缩进、preload 收集 |
| a2ts | `typescript.rs` + `ts_*.rs` | 2274 | 按 ts_types/ts_expr/ts_stmt/ts_runtime 拆分（plan-152） |
| a2j | `javascript.rs` | 814 | 早期后端，a2ts 迁移后处于维护态 |
| tscn | `tscn.rs` | 675 | Godot 场景文件生成（`SceneDecl` → .tscn） |
| 逃逸分析 | `escape/` | — | 为 a2r 提供借用/clone/Rc 分层决策（plan-310） |

测试采用约定式发现（plan-263）：`tests/a2c_tests.at`、`a2r_tests.at`、`a2ts_tests.at`
通过 FFI `Test.run_*_dir` 扫描 `crates/auto-lang/test/a2{ c,r,ts,p,j,gd}/` 下的
`.at` → `.expected.*` 对。当前规模：a2c 144、a2p 23、a2r 23+cookbook、a2ts 16、a2j 10 个用例目录。

## 关键入口

- `crates/auto-lang/src/trans.rs:Trans` — 后端统一 trait（`fn trans(&mut self, ast: Code, sink: &mut Sink)`）
- `crates/auto-lang/src/trans.rs:Sink` / `MultiSink` — 输出缓冲与多文件项目输出
- `crates/auto-lang/src/trans.rs:escape_str` — 字符串字面量转义
- `crates/auto-lang/src/trans/c.rs:CTrans` / `transpile_c` — C 后端
- `crates/auto-lang/src/trans/rust.rs:RustTrans` / `transpile_rust` / `transpile_rust_project_merged` / `RustTrans::post_process` — Rust 后端
- `crates/auto-lang/src/trans/typescript.rs:TypeScriptTrans` — TS 后端（实现分散在 ts_expr/ts_stmt/ts_types/ts_runtime）
- `crates/auto-lang/src/trans/python.rs:PythonTrans` — Python 后端
- `crates/auto-lang/src/trans/javascript.rs:JavaScriptTrans` — JS 后端
- `crates/auto-lang/src/trans/gdscript.rs:GDScriptTrans` — GDScript 后端
- `crates/auto-lang/src/trans/tscn.rs:generate_scene` — Godot .tscn 生成
- `crates/auto-lang/src/trans/r2a.rs:transpile_r2a` — Rust → Auto 逆翻译
- `crates/auto-lang/src/trans/escape/analyzer.rs:EscapeAnalyzer` — 逃逸分析
- `crates/auto-lang/src/lib.rs:trans_c / trans_rust / trans_python / trans_javascript / trans_typescript / trans_gdscript / transpile_r2a_file` — 库级入口
- `crates/auto/src/main.rs:TransTarget` — CLI 子命令枚举

## 使用示例

```bash
auto trans -i hello.at rust -o hello.rs      # 当前推荐入口
auto trans -i hello.at ts / c / python / js / gd / tscn / godot
auto test                                    # 跑 tests/a2*_tests.at 声明的转译测试
```

旧的隐藏命令（`auto rust <file>`、`auto python <file>`、`auto java-script <file>`、`auto r2a <file>`）仍在，
文档 `docs/a2r-transpiler-guide.md` 中的 `auto.exe transpile rust input.at output.rs` 写法已过时。

## 已知坑

- a2r 字符串转换是启发式叠加：`.to_string()` 注入点 + `post_process` 正则清理，偶有误注入/漏注入
  （docs/design/06 §Known Issues，如 `OsStr::to_str()` 被误映射、`list.get(N)` 被改写为下标 clone）。
- `RustTrans`/`CTrans` 处于 Universe → Database 混合期（`db: Option<...>`，plan-066），两套类型信息来源并存。
- a2j 文档（`docs/javascript-transpiler.md`）与实现脱节：文中 `JavaScriptTrans.scope` 字段已不存在，
  行数估计（~660）也过时（实际 814）；以代码为准。
- a2r 测试分两套：`test/a2r/`（23 目录）+ `test/cookbook/`（163 文件，plan-240），由 `tests/a2r_tests.at` 两个 `#[test]` 分别驱动。
- plans/ 下存在两个 355 号 plan（详见 plans.md 备注），引用 355 时须带 slug。

## 蒸馏来源（Phase 1）

- `docs/design/06-code-generation.md`（§Rust Transpiler / §C Transpiler）
- `docs/a2r-transpiler-guide.md`、`docs/a2r-api-documentation.md`
- `docs/javascript-transpiler.md`、`docs/python-transpiler.md`
- `docs/plan-indices/06-transpilers.md` 及各 plan 文件（见 plans.md）
- `crates/auto-lang/src/trans.rs`、`trans/`、`crates/auto-lang/src/lib.rs`、`crates/auto/src/main.rs`
- `tests/a2c_tests.at`、`a2r_tests.at`、`a2ts_tests.at`
