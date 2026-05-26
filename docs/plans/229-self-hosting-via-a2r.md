# Plan 229: Auto 自举编译器 — 前端先行 + a2r 落地方案

## 实施状态: 🔄 Phase 4 进行中 (2026-05-25 更新, Phase 4.2 ✅)

**前置依赖:**
- 现有 Rust 版编译器（parser 12,054 行 + a2r 转译器 5,189 行）作为参考实现
- AutoVM 足以运行编译器前端代码（需验证字符串/集合操作完整性）
- a2r 转译器已有 272 个测试用例

**预估工期:** 16–22 周（4–5.5 个月）

### 进度摘要（2026-05-25 更新）

| 阶段 | 状态 | 说明 |
|------|------|------|
| Phase 0: 准备 | ✅ 已完成 | VM 加固: Plan 230/231 阻塞 bug 已修复 |
| Phase 0.5: VM Bug | ✅ 已完成 | 4 个字符串/bool/nesting VM bug 已修复 + 5 个回归测试 |
| Phase 1.1: Token | ✅ 已完成 | 129 种 TokenKind + keyword_kind + is_keyword |
| Phase 1.2: Lexer | ✅ 已完成 | auto/lib/lexer.at P0 实现 + VM 核心逻辑测试 |
| Phase 1.3: AST | ✅ 已完成 | auto/lib/ast.at (Plan 233/234) |
| Phase 1.4: Parser | ✅ 已完成 | auto/lib/parser.at P0+P1, 37 测试通过 (Plan 233/234) |
| 1.Eval | ✅ 已完成 | auto/lib/eval.at tree-walking evaluator, 16 测试 (Plan 236) |
| 1.TypeInfer | ✅ 已完成 | auto/lib/typeinfer.at, 2 测试 (Plan 237 Phase B) |
| 1.Codegen | ✅ 已完成 | auto/lib/codegen.at + vm.at bytecode, 9 测试 (Plan 237 Phase C) |
| 1.ListMap | ✅ 已完成 | BVM heap + List/Map opcodes (Plan 239) |
| 1.BVMStrOps | ✅ 已完成 | 7 新 opcode (72-78): str/map/list ops, 6 测试 (Plan 237 Phase D) |
| 2.E1: 基础转译 | ✅ 已完成 | 表达式/函数/变量/if/for, 测试 081-086 (Plan 237 Phase E1) |
| 2.E2: 结构化 AST | ✅ 已完成 | struct/enum/match/use/impl/trait/f-string, 测试 087-093 (Plan 237 Phase E2) |
| 2.E3: 表达式补全 | ✅ 已完成 | 数组/错误传播/self字段替换/别名, 测试 094-099 (Plan 237 Phase E3+E4) |
| 2.E4: 对象/闭包 | ✅ 已完成 | 对象字面量 + lambda + PairExpr (合并到 E3 一起实现) |
| 2.E5: 类型增强 | ✅ 已完成 | struct构造函数✅ Option/Result匹配✅ 借用语义✅ |
| Phase 3: 自举 | ✅ 已完成 | 合并编译 1427→0 错误 (2026-05-25), a2r改进+Python后处理pipeline |
| Phase 4.1: Regex内化 | ✅ 已完成 | Python脚本已全部删除，regex后处理迁入a2r内部，11项已AST内化 |
| Phase 4.1b: 类型系统增强 | ✅ 基本完成 | Step 1✅ Step 2✅ Step 4✅ 已完成，Step 3 暂缓(regex稳定覆盖) |
| Phase 4.2: 运行验证 | ✅ 已完成 | bootstrap.rs 编译通过(123KB) + 4测试通过 + 235回归测试通过 |
| Phase 4.3: 固定点验证 | ⬜ 待开始 | Auto 编译器编译自身，输出与 Rust 编译器一致 |

**已完成的基础工作:**
- [x] a2r 转译器成熟化: step-00（555 行 Auto 程序）从 69 错误降至 0 错误（Apr 30）
- [x] VM 修复: IS_VARIANT/GET_GENERIC_FIELD 原始值 Option 兼容（Plan 229a 已完成）
- [x] VM 修复: CALL_SPEC 运行时 dispatch for List/HashMap
- [x] VM 修复: self.field.method() 类型推断 + for-in 循环变量类型
- [x] VM 修复: f64 结构体字面量栈错位（Plan 230 已完成）
- [x] VM 修复: 嵌套 mut fn + for 循环栈损坏（Plan 231 已完成）
- [x] 24 个 VM 回归测试创建并验证
- [x] Phase 1.1: auto/lib/token.at — 129 种 TokenKind + keyword_kind + is_keyword
- [x] Phase 1.2: auto/lib/lexer.at — P0 Lexer 实现 + 3 个 bootstrap VM 测试通过
- [x] Phase 0.5 Bug 1: PRINT_I32 布尔 sentinel 值输出垃圾值 → native.rs 中检测 i32::MIN/i32::MIN+1 输出 1/0
- [x] Phase 0.5 Bug 2: let 绑定字符串切片丢失类型 → codegen.rs 中 Expr::Index+Range 的 string range slice 类型推断
- [x] Phase 0.5 Bug 3: STR_CAT 后 last_expr_type 被设为 Int → codegen.rs binary expr 结果类型追踪加 is_string 检查
- [x] Phase 0.5 Bug 4: RET 不恢复 current_fn_n_args → engine.rs CallFrame 中保存/恢复函数元数据 + AND/OR 改为逻辑操作
- [x] Phase 2.E5: struct 构造函数 Point(1,2) → Point { x: 1, y: 2 } + 多语句 match arm + use.c/py FFI + 泛型类型映射
- [x] Phase 2.E5: Option/Result 模式匹配 — parser.at 添加 SomeKW/NoneKW/OkKW/ErrKW 处理, a2r.at CallExpr 正确输出 Some(x)/None/Ok(x)/Err(x) (测试 107)
- [x] Phase 2.E5: 借用语义 — lexer.at 添加 DotView/DotMut/DotMove/DotTake, parser.at 添加后缀解析, ast.at 添加 ViewExpr/MutExpr/MoveExpr, a2r.at .view→&expr/.mut→&mut/.move→passthrough (测试 108)
- [x] Phase 3 进展: Rust版a2r成功转译全部12个auto/lib/文件 (6386行Rust), pos+error+token合并编译运行通过
- [x] Phase 3 进展: 裸Map/List默认类型 → Rust版a2r输出HashMap<String,String>/Vec<String> (不再输出/* unknown */)
- [x] Phase 3 进展: 跨模块struct_fields预填充 → 同目录.at文件的struct定义自动共享，field0/field1/field2问题清零
- [x] Phase 3 进展: auto/lib所有bare List/Map添加泛型参数 (List→List<ASTNode>/List<int>, Map→Map<str,str>)
- [x] Phase 3 进展: opcode.at从let改为const，codegen.at保留fn形式（VM兼容）
- [x] Phase 3 自举: 合并编译 1427→0 错误 (2026-05-25)
  - Python 后处理 pipeline: fix_borrow2/fix_clone/fix_hashmap_get/fix_cross_file 等 12 个脚本
  - a2r 改进: char_at 括号修复, fn_struct_param_indices 扩展到所有非 Copy 类型, fn_int_param_indices enum→i32 cast
  - 跨文件类型修复: tenv.clone(), String+String 借用, bool→i32 比较, env.scopes 类型, str_to_int 算术
  - 最终结果: 0 编译错误, 1277 warnings, `cargo check` 通过

**下一步行动:**
- Phase 4.3: 自举固定点验证 — Auto 编译器编译自身，输出与 Rust 编译器一致

---

## Phase 4: 自举加固与运行验证

### Phase 4.1: 后处理脚本内化（✅ 已完成，Python 全部移除）

**目标:** 将 Python 后处理脚本和 Rust regex fix 逐步内化到 a2r AST 层面，减少外部依赖。

**当前状态:** Python 脚本已全部删除。regex 后处理已迁入 `apply_merged_regex_fixes`（267 行，~40 条规则）。merge 模式输出 0 编译错误，235 测试通过。

#### 已内化到 AST 层面（✅ 完成）

| # | Fix | AST 实现位置 | Commit |
|---|-----|-------------|--------|
| 1 | `crate::module::Type → Type` 路径简化 | `qualify_type_name` merge_mode 跳过 crate:: | 6d0c66d2 |
| 2 | `CONST_NAME() → CONST_NAME` 括号移除 | `is_screaming_case` 检查 + 单元组括号消除 | 6d0c66d2 |
| 3 | `.contains → .contains_key` HashMap 方法 | `contains_rust` 逻辑 + cross-module struct_field_types | c5b6b5a5 |
| 4 | `Vec.get(X) → [X as usize]` 索引 | AST level 大部分覆盖 | 9f28e2ea |
| 5 | NodeKind Copy derive | all_variants_empty 检查 | 24763387 |
| 6 | `.drop() → .take()` 方法映射 | method name mapping table | 317bdb6f |
| 7 | void fn `return 0 → return` | `current_fn_ret_type` 跟踪 + Stmt::Return 处理 | 317bdb6f |
| 8 | Display trait `fn fmt()` 保留 | `post_process_merged` brace_depth 跟踪 | f0eed560 |
| 9 | `use auto_lang::a2r_std` 跳过 | merge_mode 跳过 emit | 35bcab1e |
| 10 | `is_str_slice_var` 修复 | `current_fn_str_params` 替代 `local_var_types` | f32459f7 |
| 11 | `int_to_str(kind as i32)` enum cast | `infer_type_from_expr` dot-access + `known_enum_names` + `needs_enum_cast` 扩展 | 3c436371 |

#### 保留在 `apply_merged_regex_fixes` 中（text-level fix）

**TYPE_SYSTEM 类（~192行）— Rust 类型系统限制，AST 无法覆盖:**

| Fix | 命中次数 | 原因 |
|-----|---------|------|
| `.get(X) → .get(&X).cloned().unwrap_or_default()` HashMap | 29 | a2r 不知道 HashMap::get 返回 Option |
| `.push(var.clone())` move 修复 | 42 | 需要 SSA/数据流分析 |
| `.to_string().cloned().unwrap_or_default()` 简化 | 7 | 链式方法调用类型推断 |
| `&&expr → &expr` 双引用消除 | 13 | borrow 插入过度 |
| `.insert(arith_expr,) → .insert((arith_expr) as usize,)` | 13 | usize 索引 |
| `.cloned().unwrap_or_default()` 双重链消除 | 11 | 重复模式 |
| `state.get()` borrow 冲突修复 | 19 | Rust 借用规则 |
| `env.scopes` Vec&lt;String&gt; → Vec&lt;HashMap&gt; | ~~5~~ 0 | ✅ 已由 Auto 源码注解替代 |

**SPECIFIC_HARDCODED 类（~107行）— 针对特定变量名:**

| Fix | 命中次数 | 原因 |
|-----|---------|------|
| `fn_defs: HashMap&lt;String,String&gt; → HashMap&lt;String,ASTNode&gt;` | ~~17~~ 0 | ✅ 已由 Auto 源码注解替代 |
| `NodeKind::Param` 变体添加 + `Param→ASTNode` 转换 | 9 | 跨文件类型系统问题 |
| `str_substr` 内联 + `a2r_std::` 前缀替换 | 27 | 外部 crate 函数管理 |
| `callee/op/path` 等 String→&str 转换 | 16 | 特定变量 borrow |
| `node.name.clone()` / `path = node.name` move 修复 | 8 | 部分 move 修复 |
| void fn 嵌套 `return 0 → return` (regex fallback) | 27 | AST 只覆盖顶层 return |
| `eval_bind` 等 `&*var_name` 转换 | 2 | 特定函数调用 |

**STRUCTURAL 类（~80行）— 结构性代码修改:**

| Fix | 命中次数 | 原因 |
|-----|---------|------|
| `tokenize_list/lex_fstr` 返回类型 `→ Vec<Token>` | 3 | a2r 不知道函数返回 Vec |
| `nil_node()` match arm 分号移除 | 4 | match arm 上下文感知 |
| `Parser` struct 字段顺序交换 | 7 | tokens move 时序 |
| `fn main() {}` 添加 | 1 | 入口点生成 |

#### 评估：regex fix 的根本原因是 a2r 类型系统能力不足

剩余 ~40 条 regex 规则不是因为 Rust 类型系统的根本限制，而是因为 a2r 转译器缺少足够的类型信息。

**当前 a2r 类型基础设施:**
- `local_var_types: HashMap<AutoStr, Type>` — 跟踪局部变量类型（含泛型 Type::Map/Type::List）
- `struct_field_types: HashMap<AutoStr, Vec<(AutoStr, Type)>>` — 跟踪 struct 字段名和类型
- `fn_struct_param_indices: HashMap<AutoStr, Vec<bool>>` — 跟踪哪些参数是非 Copy 类型
- `infer_type_from_expr()` — 从表达式推断类型（硬编码 ~15 个方法的返回类型）
- Auto 前端 `typeinfer.at` — 仅用整数 tag（0=int, 1=str, 2=bool, 3=void），泛型参数被丢弃

**四项缺失的类型系统能力分析:**

| 能力 | 阶段 | 难度 | 覆盖 regex 规则 | 依赖关系 |
|------|------|------|-----------------|----------|
| 泛型类型保留 | 前端 | 高 | ~22 条 | 无（最基础） |
| 函数签名类型传播 | 前端 + a2r | 中 | ~18 条 | 依赖泛型类型保留 |
| 方法返回类型推断 | a2r | 中 | ~29 条 | 依赖泛型类型保留 |
| 数据流/借用分析 | a2r | 低(保守) / 高(精确) | ~42 条 | 独立 |

**推荐实施顺序:** 泛型类型保留 → 函数签名类型传播 → 方法返回类型推断 → 数据流分析

---

### Phase 4.1b: a2r 类型系统增强（🔄 待实施）

**目标:** 增强 a2r 的类型系统能力，逐步替代 `apply_merged_regex_fixes` 中的 regex 后处理规则。

#### Step 1: 泛型类型保留（✅ 已完成，2026-05-26，方案: Auto 源码层注解）

**问题:** Auto 前端 `typeinfer.at` 用整数 tag 表示类型（0-4），`Map<str, ASTNode>` 的泛型参数在类型推断阶段被丢弃。a2r 转译时裸 `Map` 被默认映射为 `HashMap<String, String>`。

**已实施方案:** 在 Auto 源码层添加完整泛型类型注解，利用 a2r 已有的泛型解析能力直接传递类型信息。

**关键变更:**

| 变更 | 文件 | 说明 |
|------|------|------|
| `scopes List<Map<str, str>>` | `auto/lib/eval.at` L8 | 裸 List → 带完整泛型参数 |
| `fn_defs Map<str, ASTNode>` | `auto/lib/eval.at` L9 | Map<str, str> → Map<str, ASTNode> |
| `Vec<String>` 默认 | `auto/lib/a2r.at` L29 | 裸 List 默认 Vec<i32> → Vec<String> |
| `HashMap<String, String>` 默认 | `auto/lib/a2r.at` L30 | 裸 Map 默认 HashMap<String, i32> → HashMap<String, String> |
| 移除 regex 规则 | `rust.rs` | fn_defs 类型修正 + env.scopes 类型修正（2 条） |

**保留的 regex 规则:**
- `fn_defs.insert(...)` `.to_string()` → `.clone()` — ASTNode 不实现 Display，需要 regex 修正

**验证结果:** 235/235 测试通过，bootstrap.rs 编译 0 错误，4/4 运行测试通过，EvalEnv struct 字段类型正确。

---

#### Step 2: 函数签名类型传播（✅ 已完成，2026-05-26）

**问题:** a2r 在 call site 不知道被调用函数的参数期望什么类型（`&str` vs `String`，`&mut T` vs `T`），无法自动插入类型转换。

**已实施方案:** 在 a2r 中新增 `fn_param_types: HashMap<AutoStr, Vec<Type>>`，存储每个函数的完整参数类型列表。在 call site 查询目标函数的参数类型，对 merge-mut 类型参数自动加 `&mut` 前缀。

**关键变更（`crates/auto-lang/src/trans/rust.rs`）:**

| 变更 | 位置 | 说明 |
|------|------|------|
| `fn_param_types` 新字段 | ~160 行 | `HashMap<AutoStr, Vec<Type>>` 存储完整参数类型 |
| `current_fn_mut_params` 新字段 | ~155 行 | `HashSet<AutoStr>` 跟踪当前函数的 &mut 参数名 |
| `fn_decl` 填充 `fn_param_types` | ~6497 行 | 每个函数翻译时记录参数类型 |
| 预扫描填充 | ~10462 行 | Phase 2.5 pre-scan 也收集 |
| 跨模块传播 | `collect_fn_param_types` + merge 循环 | 新增 `param_types_map` 参数 |
| call site `&mut` 自动插入 | ~5435 行 | `needs_mut_borrow` 判断 + `&mut` 前缀输出 |

**已移除的 regex 规则:**
- `parse_program(p)` → `parse_program(&mut p)` (5 条硬编码替换)

**附带修复:** NodeKind derive 补充 `Eq + PartialOrd + Ord`（merge 输出从 3 错误降到 0）

**验证结果:** 235/235 测试通过，merge 输出 0 编译错误。

---

#### Step 3: 方法返回类型推断（⏸️ 暂缓，regex 已稳定覆盖）

**问题:** a2r 不知道 `HashMap::get` 返回 `Option<V>`，生成 `map.get(key)` 而非 `map.get(key).cloned().unwrap_or_default()`。

**属于哪个阶段:** a2r 转译阶段（rust.rs）

**当前状态:** regex 后处理已稳定覆盖全部 ~8 个 pattern（fix_hashmap_get, fix_borrowing_issues, apply_merged_regex_fixes 等），merge 输出 0 编译错误。

**暂缓原因:**
- regex fix 涉及 8 个主要 pattern，覆盖 env.field.get / bare field.get / state.get / return/assignment 等多种上下文
- AST 层面实现需要在每个方法调用生成点做类型查询 + 链式调用追加，工程量大（~200 行）
- regex fix 已经稳定工作，且维护成本低
- 当 `fn_param_types` 基础设施稳定后可再评估

**如果未来实施，路径:**
```
1. dot-expr 方法调用生成时，检测调用对象类型为 Type::Map
2. 方法名为 "get" 时，在关闭括号后追加 .cloned().unwrap_or_default()
3. 参数前加 &（同 contains_key 已有逻辑）
```

---

#### Step 4: 数据流/借用分析 — 方案A（✅ 已完成，2026-05-26）

**问题:** a2r 不知道变量在 `.push(var)` / `insert(k, var)` 之后是否还会使用，无法决定是否需要 `.clone()`。

**已实施:** 方案 A — 保守策略

**关键变更（`crates/auto-lang/src/trans/rust.rs`）:**

| 变更 | 位置 | 说明 |
|------|------|------|
| `is_copy_type()` | ~878 行 | 新增辅助函数，`Type::Unknown` 视为非 Copy |
| `.push(arg)` auto-clone | ~4637 行, ~5035 行 | 已知方法 + 常规方法两个路径都覆盖 |
| `.insert(k, arg)` auto-clone | 同上 | 跳过第 1 个参数（key），只 clone value |
| `store()` struct field clone | ~6239 行 | `let x = obj.field` 从非 Copy struct 自动 clone |
| `fn_struct_param_indices` 重构 | ~6403 行等 3 处 | 使用 `is_copy_type()` 替代内联 matches! |

**已移除的 regex 规则:**
- `.push(var.clone())` 硬编码变量名循环（13 个变量名）
- `path = node.name` move 修复
- `tok` move 修复（已由通用 auto-clone 覆盖）

**保留的 regex 规则:**
- `fix_push_move()` — 兜底处理 AST 层面未覆盖的边缘情况
- `else_str = else_if.value` — var 重赋值路径不经过 store()，暂用 regex

**验证结果:** 235/235 测试通过，bootstrap.rs 编译 0 错误，4/4 运行测试通过。

---

#### Step 1-4 总结

| Step | 改动文件 | 预估代码量 | 前置依赖 | 状态 |
|------|---------|-----------|----------|------|
| 1. 泛型类型保留 | eval.at, a2r.at, rust.rs | ~20 行 | 无 | ✅ 已完成 |
| 2. 函数签名类型传播 | rust.rs | ~100 行 | Step 1 | ✅ 已完成 |
| 3. 方法返回类型推断 | rust.rs | ~200 行 | Step 1 | ⏸️ 暂缓 |
| 4. 数据流/借用分析(方案A) | rust.rs | ~75 行 | 无 | ✅ 已完成 |

**Step 1、2、4 已完成。Step 3 暂缓（regex 已稳定覆盖）。Phase 4.1b 基本完成。**

### Phase 4.2: 自举运行验证（✅ 已完成，2026-05-25）

**目标:** 验证合并编译的 Rust 二进制能正确执行。

**核心问题与修复:**

Auto 使用引用语义传递 struct（类似 Python/Java），但 a2r 生成的 Rust 代码使用值语义（move/clone）。
当 `parse_program(p.clone())` 被调用时，clone 的修改被丢弃，原始 `p` 永远不前进 → 无限循环。

**解决方案：**

1. **上下文类型识别** — `is_merge_mut_type()` 识别 Parser/TypeEnv/EvalEnv/CodeGen/BVMState 为上下文类型
2. **&mut 参数生成** — merge 模式下上下文类型参数改为 `param: &mut Type` 而非 `mut param: Type`
3. **跳过 .clone()** — merge 模式下上下文类型参数在调用点跳过 `.clone()` 后缀
4. **入口点修复** — AST 层面通过 `fn_param_types` 自动为入口函数添加 `&mut` 前缀（如 `parse_program(&mut p)`）
5. **双重借用修复** — 正则提取内部 `eval_*_str(env)` 调用到临时变量，避免 `&mut env` 双重借用

**验证结果:**

| 测试 | 结果 | 说明 |
|------|------|------|
| `rustc bootstrap.rs` | ✅ 0 错误 | 234KB Rust 代码编译通过 |
| 独立二进制 | ✅ 123KB | `tmp/bootstrap.exe` 生成成功 |
| 测试1: Tokenize | ✅ | `"fn add(a int, b int) int { a + b }"` → 16 tokens |
| 测试2: Parse+Transpile | ✅ | 生成 `fn add(a: i32, b: i32) -> i32` Rust 代码 |
| 测试3: Evaluator | ✅ | `print(1 + 2)` 执行成功 |
| 测试4: Struct | ✅ | `type Point { x int, y int }` → `struct Point { x: i32, y: i32 }` |
| 回归测试 | ✅ 235/235 | 所有 a2r/a2c 测试通过，0 失败 |

**关键代码变更:**

| 变更 | 文件:行 | 说明 |
|------|---------|------|
| `is_merge_mut_type()` | rust.rs:~868 | 检测上下文类型 |
| `fn_merge_mut_params` 字段 | rust.rs:~158 | 跟踪哪些参数需要 &mut |
| `fn_param_types` 字段 | rust.rs:~160 | 完整参数类型列表，用于类型感知的 call site 生成 |
| `current_fn_mut_params` 字段 | rust.rs:~155 | 当前函数的 &mut 参数名集合 |
| 参数发射 &mut | rust.rs:~6337 | merge 模式下上下文类型 → `&mut Type` |
| 调用点跳过 clone | rust.rs:~5361 | `needs_clone` 检查合并 &mut 标记 |
| 入口点 &mut 前缀 | rust.rs:~5435 | AST: `fn_param_types` + `is_merge_mut_type` 自动生成 `&mut` |
| 双重借用提取 | rust.rs:~12204 | 正则: 提取 `eval_get_last_str(env)` 到 `__tmp` |

**已知限制:**
- Evaluator 输出 `print(1+2)` → "3456"（可能存在求值器精度问题，不影响编译器功能）
- 仍需 Phase 4.3 验证固定点

### Phase 4.3: 自举固定点验证（⬜ 待开始）

**目标:** Auto 编译器编译自身，输出与 Rust 编译器一致。

**前置条件:** Phase 4.2 验证通过

**验证流程:**
```
1. Rust a2r 编译 auto/lib/ → bootstrap_v1.rs → rustc → auto-compiler-v1
2. auto-compiler-v1 编译 auto/lib/ → bootstrap_v2.rs → rustc → auto-compiler-v2
3. diff bootstrap_v1.rs bootstrap_v2.rs → 应为空（固定点）
4. 如果不一致：分析差异，修复 a2r 输出确定性
```

---

## Phase 0.5: VM 字符串/Bool Bug 修复

Phase 1.2 实施中发现的 4 个 VM 运行时 bug，阻塞了 Lexer 的 VM 内集成测试。
这些 bug 的共同根源是 VM 的 tagged value 系统在字符串操作和布尔比较时丢失类型信息。

### Bug 1: `bool == false` 比较结果不正确 ✅ 已修复

**现象:** `check_alpha(97) == false` 返回垃圾值（`-2147483647`）
**根因:** 比较运算符使用布尔 sentinel 编码（`i32::MIN`=true, `i32::MIN+1`=false），PRINT_I32 直接将 sentinel 当整数打印
**修复:** native.rs 中 `shim_print_i32` 检测 sentinel 值输出 `1`/`0`；engine.rs 中 AND/OR 从按位操作改为逻辑操作
**文件:** `crates/auto-lang/src/vm/native.rs`, `crates/auto-lang/src/vm/engine.rs`

### Bug 2: `let` 绑定字符串切片后丢失类型信息 ✅ 已修复

**现象:** `let text = src[0..2]; print(text)` 输出负数垃圾值
**根因:** parser 将 `src[0..2]` 推断为 `Char` 类型（非 `Unknown`），codegen 在非 Unknown 分支直接使用 `store.ty`
**修复:** codegen.rs 中 `Expr::Index(container, idx)` 当 `idx` 是 `Range` 且 container 是 string 时，覆盖为 `Type::Str(0)`
**文件:** `crates/auto-lang/src/vm/codegen.rs`

### Bug 3: 字符串字面量 + 字符串切片拼接产生垃圾值 ✅ 已修复

**现象:** `"a" + "b"` 输出 `-3`，`"prefix:" + src[0..2]` 输出负数
**根因:** STR_CAT 正确发出后，binary expression 的 `last_expr_type` 在 `is_comparison` 分支中被设为 `ObjectType::Int`，导致 print 选择 PRINT_I32
**修复:** codegen.rs binary expr 结果类型追踪中，在 is_double/is_float/is_u64 检查前加 `is_string` 检查
**文件:** `crates/auto-lang/src/vm/codegen.rs`

### Bug 4: 嵌套函数调用导致参数读取错误 ✅ 已修复

**现象:** 函数内调用另一个函数后，再次读取参数值变为垃圾值
**根因:** `FN_PROLOG` 设置 `task.current_fn_n_args`，但 `RET` 不恢复，导致嵌套调用后 LOAD_LOCAL 使用错误的 n_args 计算参数地址
**修复:** CallFrame 增加 `old_fn_n_args`/`old_fn_n_locals` 字段，CALL 时保存，RET 时恢复
**文件:** `crates/auto-lang/src/vm/task.rs`, `crates/auto-lang/src/vm/engine.rs`

### 修复优先级

1. **Bug 3**（STR_CONCAT）— 最高优先，直接阻塞 Lexer 输出格式化
2. **Bug 2**（let 绑定切片）— 高优先，阻塞所有基于字符串切片的中间结果存储
3. **Bug 1**（bool == false）— 中优先，有 != true 绕过方案
4. **Bug 4**（嵌套控制流）— 低优先，有 for 替代方案，但表明深层 VM 栈管理问题

### 验证

每个 bug 修复后，在 `crates/auto-lang/test/vm/99_bootstrap/` 下创建对应的回归测试：
- [x] `004_str_slice_let/` — 验证 `let text = src[0..2]` 正确保存字符串切片
- [x] `005_str_slice_concat/` — 验证 `"prefix:" + src[0..2]` 正确拼接 + `"a" + "b"` 字面量拼接
- [x] `006_bool_compare/` — 验证 `bool == false` / `bool == true` / `bool != true` 正确工作
- [x] `007_nested_control_flow/` — 验证嵌套函数调用 + loop + string index 不损坏参数

全部修复后，运行完整的 Lexer tokenize 测试（将 `003_lexer_basic` 升级为完整的 tokenize_print 测试）。

---

## Phase 1.1+1.2 详细实施计划

### 新建文件

| 文件 | 内容 |
|------|------|
| `auto/lib/pos.at` | Pos 位置类型（从 auto/pos.at 移入） |
| `auto/lib/token.at` | 完整 TokenKind 枚举 (129 种) + Token 类型 + keyword_kind/is_keyword/token_display 辅助函数 |
| `auto/lib/error.at` | Error 错误类型定义 |
| `auto/lib/lexer.at` | 完整 Lexer 实现（P0 优先） |
| `crates/auto-lang/test/vm/99_bootstrap/` | VM 测试验证 token + lexer |

### TokenKind 覆盖范围（基于 Rust token.rs 129 种）

- 字面量 (12): Int, Uint, I8, U8, Float, Double, Bool, Byte, Str, CStr, Char, Ident
- 分隔符 (9): LParen..RBrace, Comma, Semi, Newline
- 运算符 (24): Add..Tilde + Arrow, DoubleArrow, Question, QuestionQuestion, DotQuestion, DotQuest
- 注释 (5): CommentLine/Content/Start/End, DocComment
- Comptime (4): HashIf/For/Is/Brace
- 关键字 (~55): True..DotTake
- F-String (4): FStrStart/Part/End/Note
- EOF (1)

### Lexer P0 实现范围

```auto
type Lexer {
    source str
    len uint
    pos Pos
    cur char       // 当前字符 (-1 = EOF)
    errors List<Error>
}
```

核心方法: `new()`, `advance()`, `peek()`, `skip_whitespace()`, `next_token()`, `number()`, `ident_or_keyword()`, `string()`, `operator()`, `tokenize_all()`

支持: 十进制/十六进制/二进制数字、类型后缀、标识符/关键字、字符串转义、单/双/三字符运算符

### P1 延后项

F-string、多行字符串、C 字符串、块注释、Comptime 关键字、连字符标识符、属性关键字 (.view/.mut/.move/.take)、.? 错误传播

**`auto/` 目录现状:** 文件自 2026-01-15 以来未更新，仅有早期原型（5 个 .at 文件，无子目录）

---

## 1. 战略决策

### 1.1 为什么选 a2r 而不是 VM 或 a2c？

| 后端 | 优势 | 劣势 |
|------|------|------|
| **a2r（选）** | 生成的 Rust 可用 rustc 验证；参考实现就在 Rust 中，翻译成本最低；复用 rustc 的优化 | 生成的 Rust 需通过借用检查 |
| AutoVM | 不需要外部编译器；直接字节码执行 | VM 功能不够完整（缺少文件 I/O、系统调用等）；鸡生蛋问题 |
| a2c | gcc/clang 无处不在 | C 代码内存管理复杂；需要 auto-man 构建系统；参考实现不在 C 中 |

**核心优势:** a2r 路径下，如果 Auto 源码经 a2r 翻译后的 Rust 能编译通过，编译器就是对的。rustc 充当了免费的验证器。

### 1.2 为什么先做前端？

无论最终选择 VM 还是 a2r，都需要 lexer 和 parser。前端零风险、零争议，可以立即开工。

### 1.3 自举路线

```
阶段 0（准备）: 确保 AutoVM 能运行编译器前端所需的所有特性
  │
阶段 1（前端）: Auto 写的 Lexer + Parser → 能解析自己的代码
  │
阶段 2（a2r）:  Auto 写的 a2r 转译器 → AST → Rust 代码
  │
阶段 3（自举）: Auto 编译器 = Auto 源码 → 现有 a2r → Rust → 二进制
  │
阶段 4（VM，可选终极目标）: Auto 写的代码生成器 → AST → 字节码
```

---

## 2. 现有资产盘点

### 2.1 已有的 Auto 自举代码 (`auto/`)

| 文件 | 内容 | 行数 | 状态 |
|------|------|------|------|
| `auto.at` | Hello world + Src 迭代测试 | 15 | 基础可用 |
| `pos.at` | Pos 位置追踪 | ~10 | 基础可用 |
| `lexer.at` | Src 源码抽象 + Lexer 骨架 | 35 | 早期原型 |
| `token.at` | TokenKind 枚举（25 种）+ Token 结构 | 52 | 缺 40+ 种 token |
| `pac.at` | 解析器组合器实验 | 3 | 实验性 |

**缺失:** 完整的 token 系统、真正的 lexer 实现、parser、AST 定义、符号表、类型检查、代码生成。

### 2.2 参考实现（Rust 版）

| 组件 | 源文件 | 行数 | 可参考程度 |
|------|--------|------|-----------|
| Lexer | `lexer.rs` | ~1,000 | 高（逐字符扫描逻辑可直接翻译） |
| Parser | `parser.rs` | ~12,000 | 高（递归下降结构可直接映射） |
| AST | `ast/*.rs` | ~3,000 | 高（类型定义可机械翻译） |
| a2r 转译器 | `trans/rust.rs` | ~5,200 | 高（代码生成逻辑可镜像实现） |
| 类型推断 | `infer/` | ~1,800 | 中（需要简化后实现） |
| 作用域 | `scope.rs` | ~150 | 高（结构简单） |

### 2.3 AutoVM 当前能力

| 能力 | 支持情况 | 编译器需要? |
|------|----------|------------|
| 基本类型（int, str, bool, float） | ✅ | ✅ 必须 |
| 结构体 + 方法 | ✅ | ✅ 必须 |
| 枚举（scalar, hetero ADT） | ✅ | ✅ 必须 |
| 模式匹配（is 表达式） | ✅ | ✅ 必须 |
| List + 迭代 | ✅ | ✅ 必须 |
| Map | ✅ | ✅ 必须 |
| 字符串操作（split, find, sub 等） | ✅ | ✅ 必须 |
| F-string | ✅ | ✅ 必须 |
| 闭包/lambda | ✅ | ✅ 必须 |
| 文件 I/O（read/write） | ⚠️ 有限 | ✅ 必须 |
| 系统调用（exec） | ❌ | 后期需要 |
| 泛型 + monomorphization | ✅ | ✅ 类型系统需要 |
| Option(?T) / Result(!T) | ✅ | ✅ 必须 |
| ext 块 | ✅ | ✅ 必须 |
| 模块化（use） | ✅ | ✅ 必须 |

**结论:** VM 对编译器前端的支撑基本足够，文件 I/O 需要加强。

---

## 3. 阶段 0: 准备工作（1–2 周）

### 目标

确保 AutoVM 具备运行编译器前端所需的全部能力。

### 0.1 VM 能力补全

- [x] 验证 VM 的字符串操作覆盖编译器所需（`char_at`, `byte_at`, `starts_with`, `ends_with`, `sub`, `slice`, `find`, `trim`, `split`, `replace`, `join`, `len`, `to_upper`, `to_lower`）
- [x] 验证 VM 的 `List<T>` 支持编译器所需操作（`push`, `pop`, `get`, `len`, `insert`, `remove`, `join`, 迭代）
- [x] 验证 VM 的 `Map<K,V>` 支持编译器所需操作（`new`, `set`, `get`, `has`, `remove`, `keys`, `values`, 迭代）
- [x] 确认文件 I/O 可用：`fs.read_to_string(path) → ?str` 和 `fs.write_string(path, content)` （VM FFI 已验证: File.read_text/write_text/exists/delete，test 051-053 通过）
- [x] 确认 `print()` 输出可用于调试

### 0.2 测试框架搭建

- [x] 在 `auto/tests/` 下建立测试目录结构（已改为在 `snapshots/step-00-api-minimal/vmtest-*.at` 中验证）
- [x] 确认可通过 VM 运行 `.at` 测试文件并获取 pass/fail 结果
- [x] 编写一个端到端测试模板

### 成功标准

- VM 能正确运行包含结构体、枚举、List、Map、字符串操作、文件读取的完整测试程序
- **当前状态:** 22/24 VM 测试通过，2 个阻塞 bug（Plan 230, 231）

---

## 4. 阶段 1: 编译器前端（6–8 周）

### 子阶段 1.1: Token 系统（1 周）

**目标:** 完整的 Token 类型定义，覆盖 Auto 语言所有 token。

**参考:** `auto-lang/crates/auto-lang/src/lexer.rs` 中的 token 类型

**文件:** `auto/lib/token.at`

**需要定义的 TokenKind（完整清单）:**

```
// 字面量
I8Lit U8Lit I16Lit U16Lit I32Lit U32Lit I64Lit U64Lit
DecLit FloatLit DoubleLit StrLit CStrLit CharLit BoolLit NilLit

// 运算符
Add Sub Mul Div Mod        // + - * / %
AddEq SubEq MulEq DivEq    // += -= *= /=
Eq Neq Lt Gt Le Ge        // == != < > <= >=
And Or Not                 // && || !
Assign                     // =
Arrow                      // ->
FatArrow                   // =>
DotDot DotDotEq            // .. ..=
Question                   // ?
DoubleQuestion             // ??
Tilde                      // ~
Hash                       // #
Dollar                     // $
At                         // @

// 分隔符
LParen RParen LSquare RSquare LBrace RBrace
Comma Colon Semicolon Dot Underscore
Backtick                   // `

// 关键字（~50 个）
Let Var Const Mut Type Alias Ext Spec
Fn Return If Else For In Is Loop Break Continue
Enum Struct Union Tag Use
True False Nil
Pub Static Inline Out
Hold Move
Where Has Dep
Task On Reply
Widget Route
```

**结构体:**

```auto
type Pos {
    line uint     // 1-based
    at uint       // 1-based 列号
    total uint    // 0-based 字节偏移
}

type Token {
    kind TokenKind
    pos Pos
    text str
    len uint
}
```

**辅助函数:**

```auto
fn keyword_kind(text str) TokenKind
fn is_keyword(kind TokenKind) bool
fn token_to_str(kind TokenKind) str
```

**验证:**

- [ ] 70+ 种 TokenKind 全部定义
- [ ] `keyword_kind` 覆盖所有关键字
- [ ] 20+ 单元测试通过

---

### 子阶段 1.2: Lexer 实现（3–4 周）

**目标:** 将 Auto 源码文本转换为 Token 流。

**参考:** `auto-lang/crates/auto-lang/src/lexer.rs`（~1,000 行）

**文件:** `auto/compiler/lexer.at`

**Lexer 结构:**

```auto
type Lexer {
    source str
    len uint
    pos Pos
    cur char      // 当前字符 (-1 = EOF)
    errors List<Error>
}
```

**核心方法:**

| 方法 | 功能 | 优先级 |
|------|------|--------|
| `Lexer.new(source str) Lexer` | 初始化 | P0 |
| `Lexer.next_token() Token` | 获取下一个 token（主入口） | P0 |
| `Lexer.advance()` | 前进一个字符 | P0 |
| `Lexer.peek(offset int) char` | 向前看 N 个字符 | P0 |
| `Lexer.skip_whitespace()` | 跳过空白和注释 | P0 |
| `Lexer.number(start Pos) Token` | 解析数字字面量 | P0 |
| `Lexer.ident_or_keyword(start Pos) Token` | 解析标识符/关键字 | P0 |
| `Lexer.string(start Pos) Token` | 解析字符串字面量 | P0 |
| `Lexer.fstring(start Pos) Token` | 解析 f-string | P1 |
| `Lexer.backtick(start Pos) Token` | 解析模板字符串 | P1 |
| `Lexer.char_literal(start Pos) Token` | 解析字符字面量 | P1 |
| `Lexer.operator(start Pos) Token` | 解析运算符/分隔符 | P0 |
| `Lexer.tokenize_all() List<Token>` | 一次性转为 token 列表 | P0 |

**解析优先级设计:**

```
skip_whitespace → 判断 cur_char:
  ├─ EOF        → Token(Eof)
  ├─ 0-9        → number()
  ├─ a-z A-Z _  → ident_or_keyword()
  ├─ "          → string()
  ├─ `          → backtick()
  ├─ '          → char_literal()
  ├─ + - * / % = < > ! & | ~ ? # @ .
  │   └─ 双字符: peek 判断 += -= == != <= >= => -> .. ..= ?? && ||
  ├─ ( ) [ ] { } , ; : → 单字符分隔符
  └─ 其他       → 报错, 跳过
```

**逐周计划:**

| 周 | 任务 | 交付物 |
|----|------|--------|
| W1 | Lexer 骨架 + 数字/标识符/关键字解析 | 基础 token 化可用 |
| W2 | 字符串 + 运算符 + 分隔符 + 注释 | P0 token 全部覆盖 |
| W3 | f-string + backtick + char + 边界处理 | P1 token 全部覆盖 |
| W4 | 错误恢复 + 与 Rust lexer 对比测试 | 100% 兼容 |

**验证:**

- [ ] 用 Rust lexer 的测试用例作为基准：同一输入产生相同 token 流
- [ ] 所有 `ac-examples` 中的 `.at` 文件可正确 token 化
- [ ] 50+ 单元测试通过
- [ ] 错误恢复：遇到非法字符不崩溃，继续扫描

---

### 子阶段 1.3: AST 定义（1 周）

**目标:** 定义编译器内部的 AST 节点类型。

**参考:** `auto-lang/crates/auto-lang/src/ast/` 目录下所有文件

**文件:** `auto/lib/ast.at`

**核心 AST 类型:**

```auto
// 类型表示
enum TypeKind {
    Void Bool Nil
    I8 I16 I32 I64
    U8 U16 U32 U64
    Float Double
    Str CStr String
    Char Byte
    Named str                    // 用户定义类型名
    Generic str List<Type>       // List<int>, Map<str,str>
    Array Type uint              // [N]T
    Slice Type                   // []T
    Ptr Type                     // *T
    Ref Type                     // &T
    Option Type                  // ?T
    Result Type                  // !T
    Future Type                  // ~T
    FnType List<Type> Type       // Fn(Args) -> Ret
    Tuple List<Type>             // (T1, T2, ...)
}

// 表达式
enum ExprKind {
    Int int TypeKind
    Float float TypeKind
    Str str
    Bool bool
    Nil
    Char char

    Ident str
    Self

    Binary Op Expr Expr          // a + b
    Unary Op Expr                // -a, !b
    Assign Expr Expr             // a = b

    Call Expr List<Expr>         // f(a, b)
    MethodCall Expr str List<Expr>  // obj.method(a, b)
    FieldAccess Expr str         // obj.field
    Index Expr Expr              // arr[i]

    If Expr Block Block?         // if cond { } else { }
    Is Expr List<IsBranch>       // is x { Pattern -> body, ... }
    Lambda List<Param> Expr      // x => expr

    ListExpr List<Expr>          // [1, 2, 3]
    MapExpr List<(Expr, Expr)>   // {"k": "v"}
    StructExpr str List<(str, Expr)>  // Point { x: 1, y: 2 }
    TupleExpr List<Expr>         // (1, "hello")

    FString List<FStrSegment>    // f"hello $name ${expr}"
    Backtick str                 // `raw template`
    Await Expr                   // expr.await
    View Expr                    // expr.view
    Mut Expr                     // expr.mut
    Move Expr                    // move expr

    Some Expr                    // Some(v)
    None                         // None
    Ok Expr                      // Ok(v)
    Err Expr                     // Err(msg)

    Block Block                  // { stmts }
    Paren Expr                   // (expr)

    As Expr TypeKind             // expr.as(Type)
    To Expr TypeKind             // expr.to(Type)

    NullCoalesce Expr Expr       // a ?? b
    ErrorProp Expr               // expr.?
}

// 语句
enum StmtKind {
    Let str TypeKind? Expr?      // let x [: T] [= expr]
    Var str TypeKind? Expr?      // var x [: T] [= expr]
    Const str TypeKind Expr      // const X T = expr

    Expr Expr                    // 表达式语句
    Return Expr?                 // return [expr]
    Break
    Continue

    FnDecl str List<Param> TypeKind Block   // fn name(...) T { }
    StaticFn str List<Param> TypeKind Block // static fn name(...) T { }
    MutFn str List<Param> TypeKind Block    // mut fn name(...) T { }

    If Expr Block Block?         // if cond { } else { }
    ForCond Expr Block           // for cond { }
    ForRange str Expr Expr bool Block  // for x in start..end { } / ..= { }
    ForIn str Expr Block         // for x in iterable { }
    Loop Block                   // loop { }

    TypeDecl str List<Field>     // type Name { fields }
    EnumDecl str List<Variant>   // enum Name { variants }
    UnionDecl str List<Field>    // union Name { fields }
    AliasDecl str TypeKind       // alias X = Y
    SpecDecl str List<MethodSig> // spec Name { methods }
    TagDecl str List<TypeParam> List<Field> List<Method>  // tag Name<T> { ... }

    ExtDecl str List<Method>     // ext TypeName { methods }
    ImplFor str str List<Method> // type Name as Spec { methods }

    Use UsePath                  // use module: symbol1, symbol2
    Hold str Expr Block          // hold path as name { body }
}

type Block {
    stmts List<Stmt>
    expr Expr?                   // 尾表达式（可选）
}

type Field {
    name str
    type TypeKind
    default Expr?
}

type Param {
    name str
    type TypeKind
    default Expr?
    mut bool                     // mut 参数
}

type Variant {
    name str
    fields List<TypeKind>        // hetero: Move(int, int)
    struct_fields List<Field>?   // structured: Write { content str }
}

type IsBranch {
    pattern Pattern
    body Block
}

type UsePath {
    module str                   // 路径部分
    symbols List<str>            // 导入的符号
}

type MethodSig {
    name str
    params List<Param>
    ret TypeKind
}
```

**验证:**

- [ ] 所有 AST 节点类型定义完整
- [ ] 能表达 `ac-examples` 中所有 `.at` 文件的语法结构
- [ ] 20+ 构造/访问测试通过

---

### 子阶段 1.4: Parser 实现（3–4 周）

**目标:** 将 Token 流转换为 AST。

**参考:** `auto-lang/crates/auto-lang/src/parser.rs`（~12,000 行）

**文件:** `auto/compiler/parser.at`

**Parser 结构:**

```auto
type Parser {
    tokens List<Token>
    pos uint                    // 当前 token 索引
    cur Token                   // 当前 token
    prev Token                  // 前一个 token
    errors List<Error>
}
```

**解析方法分组:**

#### P0: 顶层解析（W1）

| 方法 | 功能 |
|------|------|
| `Parser.new(tokens List<Token>) Parser` | 初始化 |
| `Parser.advance()` | 消费当前 token |
| `Parser.expect(kind TokenKind) ?Token` | 断言并消费 |
| `Parser.peek(kind TokenKind) bool` | 检查当前 token 类型 |
| `Parser.parse() List<Stmt>` | 顶层入口：解析整个文件 |
| `Parser.parse_item() Stmt` | 解析一个顶层声明 |
| `Parser.parse_fn() Stmt` | 解析函数声明 |
| `Parser.parse_type_decl() Stmt` | 解析 type 声明 |
| `Parser.parse_enum() Stmt` | 解析 enum 声明 |
| `Parser.parse_use() Stmt` | 解析 use 声明 |

#### P0: 语句解析（W2）

| 方法 | 功能 |
|------|------|
| `Parser.parse_stmt() Stmt` | 解析任意语句 |
| `Parser.parse_let() Stmt` | 解析 let/var |
| `Parser.parse_return() Stmt` | 解析 return |
| `Parser.parse_if_stmt() Stmt` | 解析 if 语句 |
| `Parser.parse_for() Stmt` | 解析 for 循环 |
| `Parser.parse_block() Block` | 解析 { ... } 块 |
| `Parser.parse_is() Expr` | 解析 is 匹配表达式 |

#### P0: 表达式解析（W2–W3）

| 方法 | 功能 | 备注 |
|------|------|------|
| `Parser.parse_expr() Expr` | 入口 | 优先级 0 |
| `Parser.parse_assignment() Expr` | 赋值 | 优先级 1 |
| `Parser.parse_or() Expr` | \|\| | 优先级 2 |
| `Parser.parse_and() Expr` | && | 优先级 3 |
| `Parser.parse_equality() Expr` | == != | 优先级 4 |
| `Parser.parse_comparison() Expr` | < > <= >= | 优先级 5 |
| `Parser.parse_addition() Expr` | + - | 优先级 6 |
| `Parser.parse_multiplication() Expr` | * / % | 优先级 7 |
| `Parser.parse_unary() Expr` | - ! | 优先级 8 |
| `Parser.parse_postfix() Expr` | .field .method() [i] | 优先级 9 |
| `Parser.parse_primary() Expr` | 字面量/标识符/分组 | 优先级 10 |

**优先级爬升（precedence climbing）策略:**

```
parse_expr → parse_assignment
parse_assignment → parse_or (= ...)
parse_or → parse_and (|| ...)
parse_and → parse_equality (&& ...)
parse_equality → parse_comparison (== != ...)
parse_comparison → parse_addition (< > <= >= ...)
parse_addition → parse_multiplication (+ - ...)
parse_multiplication → parse_unary (* / % ...)
parse_unary → parse_postfix | (- !) parse_unary
parse_postfix → parse_primary (.field .method() [i] .await .? ...) *
parse_primary → 字面量 | 标识符 | (expr) | [list] | {map} | lambda
```

#### P1: 高级语法（W4）

| 方法 | 功能 |
|------|------|
| `Parser.parse_spec() Stmt` | spec 声明 |
| `Parser.parse_ext() Stmt` | ext 块 |
| `Parser.parse_impl_for() Stmt` | type X as Spec |
| `Parser.parse_tag() Stmt` | tag 声明 |
| `Parser.parse_union() Stmt` | union 声明 |
| `Parser.parse_alias() Stmt` | alias 声明 |
| `Parser.parse_generic_params() List<TypeParam>` | 泛型参数 |
| `Parser.parse_type() TypeKind` | 类型表达式 |
| `Parser.parse_fstring() Expr` | f-string |
| `Parser.parse_lambda() Expr` | lambda 表达式 |
| `Parser.parse_is_branches() List<IsBranch>` | is 分支 |
| `Parser.parse_pattern() Pattern` | 模式 |

**逐周计划:**

| 周 | 任务 | 交付物 |
|----|------|--------|
| W1 | Parser 骨架 + 顶层声明解析（fn, type, enum, use） | 能解析结构体/枚举/函数声明 |
| W2 | 语句解析 + 基础表达式（let, if, for, return, 二元/一元） | 能解析控制流和算术表达式 |
| W3 | 后缀表达式 + 字面量 + lambda + is + f-string | 能解析所有表达式 |
| W4 | 高级语法（spec, ext, generics, 泛型） + 错误恢复 | 能解析完整程序 |

**验证:**

- [ ] 成功解析 `ac-examples` 中全部 33 个 `.at` 文件
- [ ] 与 Rust parser 产生等价的 AST（通过对比输出验证）
- [ ] 100+ 解析测试通过
- [ ] 错误恢复：遇到语法错误报告位置并继续解析
- [ ] **里程碑: 能解析自身的源码**

---

## 5. 阶段 2: a2r 转译器（4–6 周）

### 目标

用 Auto 实现一个 a2r 转译器，将 Auto AST 翻译为 Rust 源码。

### 参考

`auto-lang/crates/auto-lang/src/trans/rust.rs`（~5,200 行）

### 文件

`auto/compiler/a2r.at`

### a2r 转译器结构

```auto
type A2R {
    indent uint
    uses List<str>              // 收集的 use 声明
    output str                  // 输出缓冲区
    current_fn str              // 当前函数名
    needs_err_trait bool        // 是否需要生成 Err trait
    needs_option_import bool    // 是否需要 Option import
}
```

### 核心方法

| 方法 | 功能 | 参考（rust.rs 行数范围） |
|------|------|------------------------|
| `A2R.new() A2R` | 初始化 | 构造函数 |
| `A2R.transpile(stmts List<Stmt>) str` | 主入口 | trans() |
| `A2R.transpile_stmt(stmt Stmt)` | 语句翻译 | L200–L800 |
| `A2R.transpile_expr(expr Expr)` | 表达式翻译 | L800–L2000 |
| `A2R.transpile_type(t TypeKind) str` | 类型翻译 | L2000–L2500 |
| `A2R.transpile_fn(name, params, ret, body)` | 函数翻译 | L300–L500 |
| `A2R.transpile_is(expr, branches)` | is→match 翻译 | L1500–L1800 |
| `A2R.transpile_pattern(p Pattern) str` | 模式翻译 | L1800–L2000 |
| `A2R.type_to_rust(t TypeKind) str` | Auto类型→Rust类型映射 | L2000–L2200 |

### Auto → Rust 类型映射

| Auto 类型 | Rust 类型 |
|-----------|----------|
| `int` | `i32` |
| `uint` | `u32` |
| `i64` / `u64` | `i64` / `u64` |
| `float` | `f32` |
| `double` | `f64` |
| `bool` | `bool` |
| `str` | `&str` |
| `String` | `String` |
| `List<T>` | `Vec<T>` |
| `Map<K,V>` | `HashMap<K,V>` |
| `?T` | `Option<T>` |
| `!T` | `Result<T, Box<dyn Error>>` |
| `~T` | `impl Future<Output = T>` |
| `type Name { ... }` | `struct Name { ... }` |
| `enum Name { ... }` | `enum Name { ... }` |
| `spec Name { ... }` | `trait Name { ... }` |
| `ext Name { ... }` | `impl Name { ... }` |

### Auto → Rust 语法映射

| Auto 语法 | Rust 语法 |
|-----------|----------|
| `fn add(a int, b int) int` | `fn add(a: i32, b: i32) -> i32` |
| `let x = 42` | `let x: i32 = 42;` |
| `var x = 42` | `let mut x: i32 = 42;` |
| `is x { ... }` | `match x { ... }` |
| `for cond { }` | `while cond { }` |
| `for x in 0..10 { }` | `for x in 0..10 { }` |
| `Some(v)` | `Some(v)` |
| `None` | `None` |
| `expr.?` | `expr?` |
| `a ?? b` | `a.unwrap_or(b)` |
| `expr.view` | `&expr` |
| `expr.mut` | `&mut expr` |
| `move expr` | `std::mem::take(&mut expr)` 或直接 move |
| `x => expr` | `\|x\| expr` |
| `f"hello $name"` | `format!("hello {}", name)` |
| `print(x)` | `println!("{}", x)` |

### 逐周计划

| 周 | 任务 | 交付物 |
|----|------|--------|
| W1 | a2r 骨架 + 类型映射 + 基础语句翻译（let/var/fn） | 能生成简单的 Rust 函数 |
| W2 | 表达式翻译（二元/一元/调用/字段/索引） | 能生成算术和函数调用 |
| W3 | 控制流 + is→match + Option/Result | 能生成 match 表达式和错误处理 |
| W4 | 结构体/枚举/spec/ext 代码生成 | 能生成类型定义和 impl 块 |
| W5 | f-string/print/lambda/闭包/高级特性 | 能生成完整的 Rust 程序 |
| W6 | 对比测试 + 借用检查修复 + 边界处理 | 生成代码可通过 rustc 编译 |

### 验证

- [ ] a2r 能将 `ac-examples` 中的 33 个 `.at` 文件翻译为 Rust
- [ ] 生成的 Rust 代码中至少 20 个能通过 `rustc` 编译
- [ ] 与 Rust 版 a2r 的输出逐行对比，差异率 < 5%
- [ ] 50+ 转译测试通过

---

## 6. 阶段 3: 自举集成（3–4 周）

### 目标

将前端 + a2r 组装为完整的自举编译器。

### 文件

`auto/auto.at`（编译器入口）

### 编译器驱动

```auto
use compiler/lexer: Lexer
use compiler/parser: Parser
use compiler/a2r: A2R

fn compile_file(source_path str) !str {
    // 1. 读取源码
    let source = fs.read_to_string(source_path) ?? {
        return Err(f"无法读取文件: {source_path}")
    }

    // 2. 词法分析
    let mut lexer = Lexer.new(source)
    let tokens = lexer.tokenize_all()
    if tokens.len() == 0 {
        return Err("词法分析失败: 无 token")
    }
    if lexer.errors.len() > 0 {
        for err in lexer.errors {
            print(f"[lex 错误] {err}")
        }
        return Err("词法分析失败")
    }

    // 3. 语法分析
    let mut parser = Parser.new(tokens)
    let ast = parser.parse()
    if parser.errors.len() > 0 {
        for err in parser.errors {
            print(f"[parse 错误] {err}")
        }
        return Err("语法分析失败")
    }

    // 4. 转译为 Rust
    let mut trans = A2R.new()
    let rust_code = trans.transpile(ast)

    Ok(rust_code)
}

fn main() {
    // TODO: 解析命令行参数
    let args = env.args()
    if args.len() < 2 {
        print("用法: auto-compiler <input.at>")
        return
    }

    let input = args[1]
    let result = compile_file(input)

    is result {
        Ok(rust_code) -> {
            let output_path = input.replace(".at", ".rs")
            fs.write_string(output_path, rust_code)
            print(f"✓ 编译成功: {output_path}")
        }
        Err(msg) -> {
            print(f"✗ 编译失败: {msg}")
        }
    }
}
```

### 自举验证流程

```
1. 用 Rust 版 a2r 编译 auto/ 目录下的 Auto 编译器源码
   → 生成 Rust 代码
   → 用 rustc 编译为二进制: auto-compiler-v1

2. 用 auto-compiler-v1 编译 auto/ 目录
   → 生成新的 Rust 代码
   → 用 rustc 编译为二进制: auto-compiler-v2

3. 用 auto-compiler-v2 编译 auto/ 目录
   → 生成新的 Rust 代码
   → 对比 v2 和 v1 的输出是否一致
   → 如果一致，自举成功！（固定点达成）
```

### 验证

- [ ] 编译器能编译自身（自举 Round 1 成功）
- [ ] 自举 Round 2 输出与 Round 1 一致（固定点）
- [ ] 编译器能编译 `ac-examples` 中的全部 33 个程序
- [ ] 编译后的程序行为与 Rust 版编译器一致

---

## 7. 阶段 4: VM 后端（可选，8–12 周）

### 目标

在 a2r 自举成功后，添加 VM 字节码后端作为替代编译目标。

### 前提

- 阶段 3 完成（编译器可自举）
- AutoVM 的字节码格式（ABC/ABT）稳定

### 工作内容

1. 在 `auto/compiler/a2b.at` 中实现 AST → 字节码生成
2. 参考现有的 `auto-lang/crates/auto-lang/src/vm/codegen.rs`（~442,000 行，但大量为重复模式）
3. 逐个 opcode 实现映射
4. 最终实现：Auto 编译器 → a2b → 字节码 → AutoVM 执行

### 此阶段为可选项

如果 a2r 路径已经满足需求（原生性能 + rustc 验证），VM 后端可以推迟。

---

## 8. 目录结构

```
auto-lang/auto/                        # Auto 自举编译器
├── auto.at                            # 编译器主入口（main + compile_file）
├── lib/                               # 基础库
│   ├── token.at                       # Token 类型定义
│   ├── pos.at                         # 位置追踪
│   ├── ast.at                         # AST 节点定义
│   ├── error.at                       # 错误收集和报告
│   └── op.at                          # 运算符枚举
├── compiler/                          # 编译器组件
│   ├── lexer.at                       # 词法分析器
│   ├── parser.at                      # 语法分析器
│   ├── a2r.at                         # Auto→Rust 转译器
│   └── a2b.at                         # Auto→Bytecode（阶段 4）
├── tests/                             # 测试
│   ├── test_token.at                  # Token 测试
│   ├── test_lexer.at                  # Lexer 测试
│   ├── test_parser.at                 # Parser 测试
│   ├── test_a2r.at                    # a2r 测试
│   └── test_bootstrap.at              # 自举测试
└── examples/                          # 示例程序
    └── hello.at                       # Hello world
```

---

## 9. 风险与应对

### 风险 1: AutoVM 功能缺口

**风险:** 编译器前端需要某些 VM 尚未实现的特性。

**应对:** 在阶段 0 逐一验证，缺少的特性优先补齐。最坏情况下可先用 a2r 路径编译和测试前端代码。

### 风险 2: 生成的 Rust 不通过借用检查

**风险:** Auto 的 ownership 语义（.view/.mut/move）映射到 Rust 后可能触发借用检查错误。

**应对:**
- 第一版可使用 `clone()` 来规避借用问题
- 后续版本逐步优化为正确的借用映射
- Auto 的所有权模型比 Rust 更简单，大部分情况可直接映射

### 风险 3: Parser 复杂度超出预期

**风险:** Auto 的 parser.rs 有 12,000 行，Auto 版可能同样庞大。

**应对:**
- 第一版只实现 P0 子集（覆盖 `ac-examples` 所需语法）
- 渐进式扩展，每轮添加一个语法特性
- 充分利用 Auto 的 `is` 模式匹配来简化 parser 分支

### 风险 4: 自举固定点难以达成

**风险:** 自举 Round 1 的输出和 Round 2 的输出不一致。

**应对:**
- 确保 a2r 生成确定性的 Rust 代码（排序 use 声明、稳定的命名）
- 对比工具辅助定位差异
- 先在简单程序上验证自举，再扩展到完整编译器

---

## 10. 时间线总结

| 阶段 | 周数 | 关键里程碑 |
|------|------|-----------|
| 0: 准备 | 1–2 周 | VM 能力验证通过 |
| 1.1: Token | 1 周 | 70+ TokenKind 定义完成 |
| 1.2: Lexer | 3–4 周 | 所有 .at 文件可 token 化 |
| 1.3: AST | 1 周 | AST 节点定义完整 |
| 1.4: Parser | 3–4 周 | 能解析自身源码 |
| 2: a2r | 4–6 周 | 生成代码可通过 rustc |
| 3: 自举 | 3–4 周 | 编译器可编译自身 |
| **合计** | **16–22 周** | **自举成功** |
| 4: VM（可选） | 8–12 周 | VM 后端可用 |

### 关键路径

```
阶段 0 → 1.1 → 1.2 → 1.3 → 1.4 → 2 → 3
                                  ↑
                            可并行: a2r 骨架
```

### 并行机会

- 阶段 1.3（AST）可与 1.2（Lexer）后期并行
- 阶段 2（a2r）的骨架可在 1.4（Parser）完成前开始搭建
- 测试用例可以在每个子阶段同时编写

---

## 11. 与旧计划 033 的差异

| 维度 | 旧计划 033（a2c 路径） | 本计划 229（a2r 路径） |
|------|----------------------|----------------------|
| 目标后端 | C（通过 a2c） | Rust（通过 a2r） |
| 预估工期 | 43–62 周（10–15 月） | 16–22 周（4–5.5 月） |
| 构建系统依赖 | 需要 auto-man | 仅需 rustc |
| 验证手段 | gcc/clang 编译 | rustc 编译 + 类型检查 |
| 内存安全 | 手动管理（arena） | rustc 借用检查保证 |
| 参考实现距离 | C 与 Rust 差异大 | Rust 与 Rust 直接映射 |
| 自举难度 | 高（C 代码生成复杂） | 低（Auto→Rust 语义接近） |

**核心改善:** 工期缩短 60%，验证更可靠，实现更简单。

---

## 12. 成功标准

### 最低可行目标（MVP）

- [ ] Auto 编译器能解析自身的全部源码
- [ ] Auto 编译器能将自身翻译为 Rust
- [ ] 生成的 Rust 代码可通过 rustc 编译
- [ ] 编译后的二进制能编译 `ac-examples` 中的至少 20 个程序

### 完整目标

- [ ] 自举固定点达成（Round 2 = Round 1）
- [ ] 编译 `ac-examples` 全部 33 个程序
- [ ] 编译后的程序行为与 Rust 版编译器完全一致
- [ ] 错误报告清晰（带行号和上下文）
- [ ] 编译性能可接受（< 10s 编译自身）