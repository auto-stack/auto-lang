# Plan 229: Auto 自举编译器 — 前端先行 + a2r 落地方案

## 实施状态: ⏳ Phase 2 (a2r) E1-E5 部分完成 (2026-05-24 更新)

**前置依赖:**
- 现有 Rust 版编译器（parser 12,054 行 + a2r 转译器 5,189 行）作为参考实现
- AutoVM 足以运行编译器前端代码（需验证字符串/集合操作完整性）
- a2r 转译器已有 272 个测试用例

**预估工期:** 16–22 周（4–5.5 个月）

### 进度摘要（2026-05-24 更新）

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
| Phase 3: 自举 | ⏳ 待开始 | 依赖 Phase 2 |

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

**下一步行动:**
- D2: 泛型注册表 (generics.at 类型字符串替换)
- Phase 3 准备: 用 AA2R 转译 auto/lib/ 所有文件，验证输出可编译

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