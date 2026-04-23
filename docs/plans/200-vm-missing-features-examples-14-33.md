# Plan 200: 补足 AutoVM 缺失特性（Examples 14-33）

> 日期：2026-04-21
> 状态：Phase 1-2 已完成，Phase 3.1 spec 分发已提前完成，剩余 Phase 3.3/.map_err() 闭包待实现
> 范围：使 AutoVM 能够运行 ac-examples 14~33

## 背景

通过分析 examples 14~33，发现 AutoVM 缺失以下能力导致这些示例无法运行。按实现优先级和难度分为四个阶段。

---

## 完成状态总结（2026-04-21）

| Task | 描述 | 状态 | 提交 |
|------|------|------|------|
| 1.1 loop 关键字 | `loop { ... break }` 语法 | ✅ 完成 | `ff3781a3` |
| 1.2 next→continue | `next` 关键字别名 | ✅ 完成 | `ff3781a3` |
| 1.3 if let 语法糖 | `if let Pattern = expr` 反糖 | ✅ 完成 | `a2ba627f` |
| 2.1 元组类型 | `Type::Tuple` + CREATE_TUPLE | ✅ 完成 | `739af4bb` |
| 2.2 字符串 natives | split_once, match_count, replace_first | ✅ 完成 | `a2ba627f` |
| 2.3 范围切片 | `buffer[..pos]` / `SLICE_RANGE` | ✅ 完成 | `101e4440` |
| 2.4 Option .or()/.unwrap_or() | Option 链式操作 | ✅ 完成 | `eaafc805` |
| 2.5 Map 类型化访问器 | 改写示例用 Json.get() + as_*() | ✅ 改写即可 | — |
| 2.6 内联 Map 字面量 | `{key: value}` 语法 | ✅ 完成 | `bd23f2d6` |
| 2.7 Backtick 原始字符串 | `` `raw string` `` | ✅ 完成 | `a2ba627f` |
| 2.8 命名统一 | has→contains, append→push | ✅ 改写即可 | — |
| 3.1 spec 动态分发 | CALL_SPEC opcode | ✅ **提前完成** | `1933641e` |
| 3.2 enumerate() | EnumerateIterator | ✅ 完成 | `e8314053` |
| 3.3 .map_err() 闭包 | native→闭包回调 | ✅ **完成** | Plan 218 |
| 3.4 fs 模块别名 | 已归入 Phase 4 | ⏭️ 低优先 | — |

**剩余工作**：Phase 3.4 fs 模块别名（低优先级，可通过改写示例绕过）。

---

这些特性可以通过重写示例以 VM 已支持的语法来绕过，同时也应考虑在语言层面直接支持。

### Task 1.1：`loop { }` 关键字

**问题**：示例 16 使用 `loop { ... break }`，VM 不支持。
**现有替代**：`for ever { ... break }` 已完全支持。

**语言级修复**（可选，提升语法糖）：

| 文件 | 修改 |
|------|------|
| `token.rs:15` | 添加 `TokenKind::Loop` 枚举变体 |
| `token.rs:398` | 添加 `"loop" => Some(TokenKind::Loop)` |
| `parser.rs:3227` | 添加 `TokenKind::Loop => self.loop_stmt()` |
| `parser.rs` 新方法 | `loop_stmt()` → 生成 `For { iter: Iter::Ever, range: Expr::Nil, body, ... }` |

代码生成无需修改——`Iter::Ever` 分支（`codegen.rs:2143`）已正确处理。

**估计**：~30 行代码，1 小时。

### Task 1.2：`next` 关键字 → `continue`

**问题**：示例 15 使用 `next` 跳过迭代，但 VM 只支持 `continue`。
**修复**：改写示例使用 `continue`。

如需语言级支持：在 `keyword_kind` 中将 `"next"` 映射到 `TokenKind::Continue` 即可。

**估计**：~5 行代码。

### Task 1.3：`if let` 语法糖

**问题**：示例 15 使用 `if let Some((key, value)) = line.split_once(":")`，VM 无 `if let`。
**现有替代**：`is expr { Some(x) { ... } else { ... } }` 已完全支持。

**语言级修复**（推荐实现）：

在 `parser.rs:4805` 的 `if_stmt()` 中，`if` 后检查 `let` 关键字：

```
if let Pattern = expr { body }
```

反糖为：

```
is expr { Pattern { body } }
```

直接复用 `Is` AST 节点和 codegen（`codegen.rs:2277`），无需新增字节码。

| 文件 | 修改 |
|------|------|
| `parser.rs:4805` | 在 `if` 后检测 `let`，解析 pattern + `=` + expr，构造 `Stmt::Is` |
| `token.rs` | 确认 `TokenKind::Let` 已存在 |

**注意**：元组解构（`Some((k, v))`）依赖 Task 2.1 的元组类型。

**估计**：~50 行代码，2 小时。

---

## Phase 2：中等成本（需要新增 native shim）

### Task 2.1：元组类型 `Type::Tuple(Vec<Type>)`

**问题**：示例 15、17、20 使用元组 `(str, str)`、`(str, bool)`，VM 无元组类型。
**影响**：`if let Some((k, v))` 解构、`List<(str, bool)>` 泛型参数。

**AST 层**：

| 文件 | 修改 |
|------|------|
| `ast/types.rs:24` | `Type` 枚举添加 `Tuple(Vec<Type>)` |
| `ast/types.rs:72` | `unique_name()` 添加 `Tuple(ts) => format!("({})", ts.join(","))` |
| `ast/types.rs:134` | `default_value()` 添加元组分支 |
| `ast/types.rs:190` | `substitute()` 添加元组递归替换 |
| `ast/types.rs:309` | `is_optimized_by_value()` → `false` |
| `ast/types.rs:501` | `Display` 添加格式化 |
| `ast/types.rs:850` | `AtomWriter` 添加序列化 |
| `ast.rs:286` | `Expr` 添加 `Tuple(Vec<Expr>)` 变体 |

**Parser 层**：

| 文件 | 修改 |
|------|------|
| `parser.rs:7935` | `parse_type()` 添加 `LParen` 分支，解析 `(T1, T2, ...)` |
| `parser.rs` | `parse_expr_or_stmt` 中识别 `(expr1, expr2)` 为元组构造 |

**Codegen 层**：

| 文件 | 修改 |
|------|------|
| `vm/codegen.rs` | `Expr::Tuple` → 将每个元素压栈 + `CREATE_TUPLE n` opcode |
| `vm/codegen.rs` | 元组解构 → `GET_TUPLE_FIELD index` opcode |
| `vm/opcode.rs` | 添加 `CREATE_TUPLE`、`GET_TUPLE_FIELD` |
| `vm/engine.rs` | 实现两个新 opcode 的执行逻辑 |

**所有 `match Type` 和 `match Expr` 的位置**（约 15+ 处）需添加 `_ =>` 或新分支。

**估计**：~300 行代码，1-2 天。

### Task 2.2：缺失的字符串 native 函数

**问题**：示例 15、16、19 使用 `split_once`、`match_count`、`replace_first`，均无 native 实现。

**添加位置**（每个函数遵循相同模式）：

| 文件 | 修改 |
|------|------|
| `vm/ffi/stdlib.rs:217` | 添加常量 `NATIVE_STR_SPLIT_ONCE = 1518`、`NATIVE_STR_MATCH_COUNT = 1519`、`NATIVE_STR_REPLACE_FIRST = 1520` |
| `vm/ffi/stdlib.rs` | 添加 3 个 shim 函数 |
| `vm/ffi/stdlib.rs:2842` | 注册 3 个 shim |
| `vm/native_registry.rs` | 注册 3 个名称→ID 映射 |
| `stdlib/auto/str.at` | 添加 3 个 `#[vm] fn` 声明 |

**实现细节**：

```rust
// split_once(s, delimiter) → ?(str, str)
fn shim_str_split_once(task, vm) {
    let (s, delim) = pop_two_strings(task, vm);
    match s.split_once(&delim) {
        Some((before, after)) => { /* push tuple or object {0: before, 1: after} */ }
        None => { task.ram.push_i32(-1); /* None */ }
    }
}

// match_count(s, pattern) → int
fn shim_str_match_count(task, vm) {
    let (s, pat) = pop_two_strings(task, vm);
    task.ram.push_i32(s.matches(&pat).count() as i32);
}

// replace_first(s, from, to) → str
fn shim_str_replace_first(task, vm) {
    let (s, from, to) = pop_three_strings(task, vm);
    let result = s.replacen(&from, &to, 1);
    /* push result string */
}
```

**注意**：`split_once` 的返回类型 `?(str, str)` 依赖元组（Task 2.1）或可用对象 `{before: str, after: str}` 替代。

**估计**：~100 行代码，半天。

### Task 2.3：范围切片语法 `buffer[..pos]`、`buffer[pos+4..]`

**问题**：示例 16 使用半开放范围切片，VM 不支持。

**AST 层**：

| 文件 | 修改 |
|------|------|
| `ast/range.rs:8` | `Range.start` 改为 `Option<Box<Expr>>`，`Range.end` 改为 `Option<Box<Expr>>` |
| `parser.rs` | 范围解析支持缺失边界（`..pos`、`pos..`、`..`） |

**Codegen 层**：

| 文件 | 修改 |
|------|------|
| `vm/codegen.rs:3611` | `Expr::Index` 中检查 `idx` 是否为 `Range`，若是则编译为 `SLICE_RANGE` |
| `vm/opcode.rs` | 添加 `SLICE_RANGE` opcode（弹出 container + start + end） |
| `vm/engine.rs` | 实现 `SLICE_RANGE`：对字符串创建新子串，对数组创建新数组 |

**字符串切片**可直接复用 `str.substr` native（ID 1503）。

**估计**：~150 行代码，半天。

---

## Phase 3：高成本（需要架构扩展）

### Task 3.1：`spec`/`has` 动态分发（vtable）

**问题**：示例 20 使用 `spec Tool { ... }` + `ext EchoTool has Tool { ... }` + `Map<str, Tool>` 实现多态，VM 端 `SpecDecl` 为空操作。

**当前架构**：
- 所有方法调用在编译期静态解析为 `TypeName.method` 的直接调用
- 无运行时类型标签分发机制
- 无 vtable 或间接调用 opcode

**方案 A：Tag-Based Dispatch（推荐，最小改动）**

利用现有的 heap object 类型标签系统。

1. **编译期**（`Stmt::TypeDecl` 处理时）：
   - 当类型声明 `has Spec`，在全局注册表中记录 `TypeName → SpecName` 关系
   - 为每个 `has` 的 spec 方法生成标准的 `TypeName.method` 函数

2. **运行期**（调用 spec 方法时）：
   - 新增 opcode `CALL_SPEC`（或复用 `CALL_INDIRECT`）
   - 操作码参数：spec 名称 + 方法名称
   - 执行引擎根据对象的类型标签查找 `TypeName.method` 地址并调用

3. **trait object 存储**：
   - `Map<str, Tool>` 中存储对象时，保留对象的类型标签
   - 调用 `tool.execute(input)` 时，通过标签查找具体方法地址

**关键文件**：

| 文件 | 修改 |
|------|------|
| `vm/codegen.rs:1633` | `SpecDecl` 处理：注册 spec → method 签名映射 |
| `vm/codegen.rs:1548` | `TypeDecl` 处理：记录 `has Spec` 关系 |
| `vm/codegen.rs:4519` | 方法调用分发：检测 spec 类型参数，生成 `CALL_SPEC` |
| `vm/opcode.rs` | 添加 `CALL_SPEC` opcode |
| `vm/engine.rs` | 实现运行时 spec 分发逻辑 |
| `vm/loader.rs` | 链接器处理 spec 方法重定位 |

**方案 B：胖指针（Fat Pointer）**

每个 trait object 存储为 `(data_ptr, vtable_ptr)`。成本高，需改动整个 heap object 系统。

**推荐**：先实现方案 A，覆盖示例 20 的用例。完整的动态分发可在后续迭代中增强。

**估计**：~500 行代码，2-3 天。

### Task 3.2：`enumerate()` 迭代器

**问题**：示例 17 使用 `for (i, m) in msgs.iter().enumerate()`。

**当前架构**：
- 迭代器存储在 `DashMap<u32, Iterator>` 中
- `Iterator` 枚举有 `List`、`Map`、`Filter` 变体
- `Iterator.next` native 从迭代器取出下一个元素

**实现方案**：

| 文件 | 修改 |
|------|------|
| `vm/engine.rs:50` | 添加 `EnumerateIterator { source_id: u32, index: u32 }` |
| `vm/engine.rs:28` | `Iterator` 枚举添加 `Enumerate(EnumerateIterator)` |
| `vm/native.rs` | 添加 `shim_iter_enumerate`：包装现有迭代器 |
| `vm/native.rs` | 修改 `shim_iter_next`：`Enumerate` 变体同时返回 index + value |
| `vm/native_registry.rs` | 注册 `"Iterator.enumerate"` |

`for i, x in expr.enumerate()` 的 codegen 可复用现有的 `Iter::Indexed` 路径（`codegen.rs:1909`）。

**替代方案**：不实现 `enumerate()` native，而是将示例 17 改写为已有的 `for i, x in list { }` 内建双变量迭代语法。这对 list 迭代完全足够。

**估计**：~100 行代码（native 实现）或 0 行代码（改写示例）。

---

## 实施顺序

```
Phase 1（可立即改写示例绕过）     Phase 2（需新增代码）         Phase 3（架构扩展）
─────────────────────────     ─────────────────────     ────────────────────
1.1 loop 关键字  ──────┐
1.2 next→continue  ────┤       2.2 字符串 natives ────┐
1.3 if let 语法糖 ─────┘       2.3 范围切片语法 ──────┤
                               2.1 元组类型 ──────────┘   3.1 spec 动态分发
                                                           3.2 enumerate()
```

**建议**：
1. **先改写示例**：用 Phase 1 的替代语法重写 14-19，使其在当前 VM 上可运行
2. **再实现 Phase 2**：按 2.2 → 2.3 → 2.1 顺序，先补字符串函数，再补切片，最后补元组
3. **最后 Phase 3**：示例 20 的 trait 多态是最复杂的特性，独立迭代

## 示例可运行性预估

| 示例 | 改写后可运行？ | 需要的 Phase |
|------|-------------|-------------|
| 14 SSE Frame Extract | ✅ | 改写即可（slice→substr, ?str 已支持） |
| 15 SSE Field Parser | ⚠️ | 改写 if let→is，split_once 需 Phase 2.2 或改用 split |
| 16 SSE Parser Full | ⚠️ | 改写 loop→for ever，切片→substr，同 15 |
| 17 Context Compaction | ✅ | 改写 enumerate→for i, x in list，改 if 表达式 |
| 18 Command Safety Check | ✅ | Result 基本支持，改写即可 |
| 19 Exact Match Edit | ⚠️ | match_count/replace_first 需 Phase 2.2 |
| 20 Tool Registry | ❌ | 必须等 Phase 3.1 spec 动态分发 |

## 背景

通过分析 examples 14~33，发现 AutoVM 缺失以下能力导致这些示例无法运行。按实现优先级和难度分为四个阶段。

---

## Examples 21-33 特性分析

### 新增缺失特性总表

Examples 21-33 在 14-20 已分析的缺失特性之外，还暴露了以下新问题：

| # | 特性 | 状态 | 影响示例 | 修复难度 |
|---|------|------|---------|---------|
| A | `match` 表达式 | 已有替代（→ `is`） | 27 | 改写即可 |
| B | `.or()` / `.unwrap_or()` on Option | **缺失** | 23, 24, 25 | 中等 |
| C | `.map_err(\|e\| ...)` + 闭包参数 | **缺失** | 31 | 高 |
| D | `.has(key)` on Map | **缺失**（有 `.contains()`） | 24 | 改写即可 |
| E | `.get_list()` / `.get_map()` / `.get_uint()` | **缺失** | 23 | 中等 |
| F | Backtick 原始字符串 `` `...` `` | **缺失** | 33 | 低 |
| G | `.append()` on List（vs `.push()`） | **缺失**（有 `.push()`） | 30 | 改写即可 |
| H | `fs.read()` / `fs.write()` 模块名 | **缺失**（有 `File.*`） | 27, 28, 33 | 改写/别名 |
| I | JSON 类型化访问器 `.get_str()` / `.get_obj()` | **部分有**（用 `Json.get()` + `Json.as_string()`） | 23, 33 | 改写即可 |
| J | 内联 Map 字面量 `{k: v}` | **缺失** | 21, 24, 25, 29 | 高 |
| K | `List<!str>` Result 泛型参数 | **缺失** | 31 | 高 |
| L | `for (a, b) in expr` 元组解构 | **缺失** | 26, 30, 31 | 中等 |

### 逐示例分析

| 示例 | 能否运行 | 关键阻塞 |
|------|---------|---------|
| **21** Anthropic Request Build | ⚠️ | 内联 Map 字面量 `{k: v}` 不支持，需改用 `Map.new()` + `.insert()` |
| **22** OpenAI Msg Translate | ✅ | 枚举+`is` 解构已支持，改写即可 |
| **23** OpenAI Response Normalize | ⚠️ | `.or()` 缺失；`.get_list()`/`.get_map()` 缺失，需改用 `Json.get()` + `Json.as_*()` |
| **24** Stream State Ingest | ⚠️ | `if let` 需改写→`is`；`.has()` → `.contains()`；内联 Map 字面量 |
| **25** Stream State Finish | ⚠️ | `.or()` 缺失；`if let` 需改写；内联 Map 字面量 |
| **26** OpenAI Buffer Process | ⚠️ | 元组返回+解构 `(List, bool, str)` 需元组支持；范围切片 `[0..n]` |
| **27** JSONL Persistence | ✅ | `use fs` → `File.*` 改写；`match` → `is` 改写；`?` try 操作符已支持 |
| **28** File I/O Basics | ✅ | `use fs` → `File.*` 改写即可 |
| **29** Provider Dispatch | ✅ | 枚举+`is self {}` 分发已支持；`.slice()` 已支持 |
| **30** REPL Loop | ✅ | `.append()` → `.push()` 改写；`+=` 已支持 |
| **31** Tool Exec With Perm | ❌ | `spec Tool` 动态分发；`.map_err()` 闭包；`List<!str>` Result 泛型；`enumerate()` |
| **32** Stream Event Agg | ✅ | 枚举+`is` 已支持；结构体相等已支持；`+=` 已支持 |
| **33** CLI Settings Loader | ⚠️ | Backtick 原始字符串；`env.*` 已支持；`json.parse()` 已支持 |

---

## Phase 2.4：Option 链式操作（`.or()` / `.unwrap_or()`）

**问题**：示例 23、24、25 使用 `.or("default")` 和 `.unwrap_or("default")`，无 native 实现。

**方案**：添加两个 native shim。

| 文件 | 修改 |
|------|------|
| `vm/native.rs` | 添加 `shim_option_or`：检查栈顶是否为 None（-1），若是则替换为默认值 |
| `vm/native.rs` | 添加 `shim_option_unwrap_or`：同上，命名别名 |
| `vm/native_registry.rs` | 注册 `"Option.or"` → 新 ID，`"Option.unwrap_or"` → 同 ID |
| `stdlib/auto/str.at` 或新建 `stdlib/auto/option.at` | 添加 `#[vm] fn or(default T) T` 和 `#[vm] fn unwrap_or(default T) T` |

**实现逻辑**：
```rust
fn shim_option_or(task, vm) {
    let default_val = task.ram.pop_i32();
    let opt_val = task.ram.pop_i32();
    if opt_val == -1 {  // None sentinel
        task.ram.push_i32(default_val);
    } else {
        task.ram.push_i32(opt_val);
    }
}
```

**注意**：Plan 197 的 heap-object-based Option 可能使用不同的 None 编码（非 -1），需适配。

**估计**：~40 行代码，2 小时。

---

## Phase 2.5：Map 类型化访问器（`.get_list()` / `.get_map()` / `.get_uint()`）

**问题**：示例 23 使用 `.get_list("choices")`、`.get_map("message")`、`.get_uint("tokens", 0)` 等。

**现有替代**：`Json.get()` 返回 JsonValue，再用 `Json.as_string()` / `Json.as_int()` 转换。

**方案 A（推荐）**：改写示例使用 `Json.get()` + `Json.as_*()` 链式调用。

**方案 B（语言级）**：为 JsonValue 添加类型化 getter native。

| 文件 | 修改 |
|------|------|
| `vm/ffi/stdlib.rs` | 添加 `shim_json_get_str`、`shim_json_get_list`、`shim_json_get_map`、`shim_json_get_uint` |
| `vm/native_registry.rs` | 注册 `Json.get_str`、`Json.get_list`、`Json.get_map`、`Json.get_uint` |

**估计**：方案 A 零代码；方案 B ~80 行代码，半天。

---

## Phase 2.6：内联 Map 字面量 `{key: value}`

**问题**：示例 21、24、25、29 使用 `{"key": "value", ...}` 构造 Map，VM 不支持。

**当前方式**：`Map.new()` + 多个 `.insert()` 调用。

**方案**：

| 文件 | 修改 |
|------|------|
| `ast.rs` | 添加 `Expr::MapInit(Vec<(Expr, Expr)>)` 变体 |
| `parser.rs:1726` | 在 `{` 后检测 `key: value` 模式（区分 block body 和 map literal） |
| `vm/codegen.rs` | `Expr::MapInit` → 发射 `Map.new()` + N 个 `insert` 调用 |
| `vm/opcode.rs` | 添加 `CREATE_MAP n` opcode（或内联展开为 CALL_NAT 序列） |
| `vm/engine.rs` | 实现批量创建 Map 的 opcode |

**歧义问题**：`{}` 既可以是空 block 也可以是空 Map。需要上下文区分（赋值右侧 = Map，独立语句 = block）。

**估计**：~200 行代码，1 天。

---

## Phase 3.3：`.map_err()` + 闭包参数

**问题**：示例 31 使用 `.map_err(\|e\| f"${e}")` 将错误映射为新类型。

**依赖**：
1. 闭包作为参数传递（`Expr::Closure` 已支持，opcode `CLOSURE` 0x90）
2. `.map_err()` native 不存在

**方案**：

| 文件 | 修改 |
|------|------|
| `vm/native.rs` | 添加 `shim_result_map_err`：弹出闭包地址 + Result 值，若 Err 则调用闭包 |
| `vm/native_registry.rs` | 注册 `"Result.map_err"` |

**实现逻辑**：
```rust
fn shim_result_map_err(task, vm) {
    let closure_addr = task.ram.pop_i32();  // closure address
    let result_val = task.ram.pop_i32();
    if result_val < 0 {  // Err
        task.ram.push_i32(result_val);  // push error value as arg
        // call closure at closure_addr
        task.ram.push_i32(closure_addr);
        // emit CALL_CLOSURE
    } else {
        task.ram.push_i32(result_val);  // Ok, pass through
    }
}
```

**注意**：这需要 VM 在 native shim 内部调用闭包，可能需要新增 opcode `NATIVE_CALL_CLOSURE` 或在 engine 中添加辅助方法。

**估计**：~100 行代码，半天。

---

## Phase 3.4：Backtick 原始字符串字面量

**问题**：示例 33 使用 `` `{"key": "value"}` `` 多行原始字符串。

**方案**：

| 文件 | 修改 |
|------|------|
| `token.rs` | 添加 `TokenKind::RawStr`；lexer 在遇到 `` ` `` 时扫描到下一个 `` ` `` |
| `parser.rs` | `Expr::RawStr` 或复用 `Expr::Str`（值相同，仅语法不同） |

**注意**：autodown 解析器已有 backtick 支持（`autodown/lexer.rs:400`），可参考。

**估计**：~40 行代码，2 小时。

---

## Phase 4：`fs` 模块别名 + 命名统一

**问题**：示例 27、28、33 使用 `use fs`，但 VM 中的模块名是 `File`。

**方案 A（推荐）**：创建 `stdlib/auto/fs.at` 作为 `File` 的别名模块：

```auto
// stdlib/auto/fs.at
#[vm] fn read(path str) str = File.read_text(path)
#[vm] fn write(path str, content str) = File.write_text(path, content)
#[vm] fn append(path str, content str) = File.append_text(path, content)
#[vm] fn exists(path str) bool = File.exists(path)
// ... 等等
```

**方案 B**：改写示例使用 `File.*` 直接调用。

**方案 C**：在 native_registry 中为每个 `File.*` 添加 `fs.*` 别名。

| 文件 | 修改 |
|------|------|
| `stdlib/auto/fs.at` | 新建，声明 `#[vm]` 函数 |
| `vm/native_registry.rs` | 注册 `fs.read`、`fs.write` 等别名指向 File 的 native ID |
| `crates/auto-lang/src/resolver.rs` | 确保 `use fs` 能解析到正确模块 |

**估计**：方案 A ~60 行代码；方案 B 零代码；方案 C ~30 行代码。

---

## 更新后的实施顺序

```
Phase 1（改写即可）              Phase 2（新增 native/code）    Phase 3（架构扩展）
──────────────────     ─────────────────────────────     ────────────────────
1.1 loop 关键字  ──┐    2.2 字符串 natives ──────────┐    3.1 spec 动态分发
1.2 next→continue ─┤    2.3 范围切片语法 ────────────┤    3.2 enumerate()
1.3 if let 语法糖 ─┘    2.1 元组类型 ───────────────┤    3.3 .map_err() 闭包
                        2.4 Option .or()/.unwrap_or() ┤
                        2.5 Map 类型化访问器 ────────┤    Phase 4（模块别名）
                        2.6 内联 Map 字面量 ─────────┘    ────────────────
                        2.7 Backtick 原始字符串 ──────┐    4.1 fs 模块别名
                        2.8 命名统一（has→contains等）┘
```

## 更新后的示例可运行性预估

| 示例 | 改写后可运行？ | 需要的 Phase |
|------|-------------|-------------|
| 14 SSE Frame Extract | ✅ | 改写即可 |
| 15 SSE Field Parser | ⚠️ | Phase 2.2（split_once）或改用 split |
| 16 SSE Parser Full | ⚠️ | Phase 2.2 + 2.3 或改写 |
| 17 Context Compaction | ✅ | 改写 enumerate→for i, x |
| 18 Command Safety Check | ✅ | 改写即可 |
| 19 Exact Match Edit | ⚠️ | Phase 2.2（match_count/replace_first） |
| 20 Tool Registry | ❌ | Phase 3.1 spec 动态分发 |
| 21 Anthropic Request Build | ⚠️ | Phase 2.6（Map 字面量）或改用 Map.new() |
| 22 OpenAI Msg Translate | ✅ | 改写即可 |
| 23 OpenAI Response Normalize | ⚠️ | Phase 2.4（.or()）+ 2.5 或改用 Json.get() |
| 24 Stream State Ingest | ⚠️ | Phase 2.4（.or()）+ 2.6 或改写 |
| 25 Stream State Finish | ⚠️ | 同 24 |
| 26 OpenAI Buffer Process | ⚠️ | Phase 2.1（元组）+ 2.3（切片） |
| 27 JSONL Persistence | ✅ | 改写 fs→File、match→is |
| 28 File I/O Basics | ✅ | 改写 fs→File |
| 29 Provider Dispatch | ✅ | 改写即可 |
| 30 REPL Loop | ✅ | 改写 append→push |
| 31 Tool Exec With Perm | ❌ | Phase 3.1 spec + 3.3 .map_err() + 元组 |
| 32 Stream Event Agg | ✅ | 改写即可 |
| 33 CLI Settings Loader | ⚠️ | Phase 3.4（backtick）或改用普通字符串 |

### 统计

| 状态 | 数量 | 示例 |
|------|------|------|
| ✅ 改写即可运行 | **10** | 14, 17, 18, 22, 27, 28, 29, 30, 32 |
| ⚠️ 需 Phase 2 支持 | **8** | 15, 16, 19, 21, 23, 24, 25, 26, 33 |
| ❌ 需 Phase 3 支持 | **2** | 20, 31 |

## 参考

- Plan 192：VM enum/ext codegen（已实现）
- Plan 194：Monomorphic dispatch（已实现）
- Plan 197：VM ADT + Generic Lists + Option/Result（已实现）
