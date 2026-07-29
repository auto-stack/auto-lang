# Plan 376: a2r 类型流分析（让 re-transpile 达到 0 错误）

> **状态**：⏳ 待实施（调研已完成，方案待批准）
> **仓库**：auto-lang
> **前置**：plan 372（系统性 a2r 根因）、plan 373（B1 细节 + post_process 链）
> **目标**：让 `crates/auto-ai-agent/rust/` 的 **re-transpile**（全部 .at → a2r → 组装 → cargo check）达到与手修版同等的 0 错误水平。
> **背景**：plan 373 的 post_process 链 + `has Spec` lowering + `&mut self`/`.await` 自动化
> 把 re-transpile 错误从 343 降到了 ~285，但剩余的 ~285 个错误**无法用正则 post_process 解决**——
> 它们需要 a2r 在生成阶段就知道值的类型上下文。

## 一、问题定义

### 当前 re-transpile 的错误分布（~285 个，2026-07-30 实测）

| 错误码 | 数量 | 根因分类 | 能否用 post_process 正则解决 |
|---|---|---|---|
| E0308 | ~134 | 类型不匹配（杂项） | ❌ 需要知道赋值目标类型、方法返回类型 |
| E0599 | ~38 | 方法找不到 | ⚠️ 部分（PathBuf.as_str 等）；大部分需类型推断 |
| E0658 | ~17 | 多余 `.as_str()` on `&str` | ⚠️ 需知道接收方参数类型 |
| E0425/E0423 | ~19 | 名字找不到（级联） | ❌ 前序错误的级联 |
| E0614 | ~13 | 桥接类型 deref | ⚠️ 需知道桥接类型的 Deref 语义 |
| E0507 | ~12 | move/borrow（`for m in self.field`） | ⚠️ 需知道 `self` 是 `&` 还是 `&mut` |
| E0277 | ~8 | trait 未实现 | ⚠️ `fix_non_ord_derives` 已覆盖大部分 |
| E0609/E0608 | ~15 | Option 上取字段 | ❌ 需知道表达式返回 Option |
| E0382 | ~8 | use after move | ⚠️ 需借用分析 |
| E0252 | ~7 | 合成 `RoleTrait` 冲突 | ✅ `has Spec` 修复已处理（re-transpile 需重新应用） |
| E0195 | ~3 | `#[async_trait]` 缺失 | ⚠️ 部分 `impl` 块未触发检测 |
| 其他 | ~11 | 杂项 | — |

### 核心瓶颈：a2r 缺乏"使用上下文的类型推断"

a2r 当前是一个**单遍（single-pass）生成器**：它遍历 AST，逐表达式输出 Rust 代码，
但**不跟踪值的类型**。这导致：

1. **`HashMap.get(key)` → 返回 `Option<&T>`，但代码当 `T` 用**：a2r 不知道 `.get()` 返回 Option
2. **`self.field = Some(context)` → `context` 是 `&str`，`field` 是 `Option<String>`**：a2r 不知道目标字段类型
3. **`fn foo(name str)` 调用时传 `name: String` → 需要 `&name` 或 `.as_str()`**：a2r 不知道参数是 `&str`
4. **`a2r_std::fs::read_to_string(path)` → 返回 `String`（不是 `Result`）**：a2r 不知道桥接函数的真实签名
5. **`for m in self.messages` → `self` 是 `&self`，需要 `&self.messages`**：a2r 不知道 self 的借用模式

## 二、方案设计

### 方案 A：最小类型流分析（推荐）

**核心思路**：在 a2r 的 `trans()` 主 pass 中，增加一个**轻量类型上下文（Type Context）**，
不试图做完整的 Hindley-Milner 类型推断，而是跟踪**足够的信息来消除上述 5 类错误**。

#### 类型上下文需要跟踪的信息

| 信息 | 来源 | 用途 |
|---|---|---|
| **方法参数类型**（已有 `fn_param_types`） | `fn_decl.params` | 知道 `name: &str` → 调用时传 String 需 `.as_str()` |
| **结构体字段类型**（新增 `struct_field_types`） | `TypeDecl.members` | 知道 `self.context_block: Option<String>` → 赋 `&str` 需 `.to_string()` |
| **方法返回类型**（已有 `fn_ret_types`，plan 373 新增） | `Fn.ret` | 知道 `get()` 返回 Option → 需 unwrap |
| **外部函数签名表**（新增 `known_fn_sigs`） | 硬编码 | 知道 `HashMap.get` → `Option<&T>`、`Vec.push` → `()` |
| **self 借用模式**（已有 `in_trait_impl` 等） | 方法签名 | 知道 `&self` → `for m in self.x` 需改为 `&self.x` |

#### 实施步骤（4 个 pass，每个独立可验证）

**Pass 1：`struct_field_types` 预扫描**（~40 行）
- 在 `trans()` 的 Phase 1 预扫描中，遍历所有 `Stmt::TypeDecl`，收集 `HashMap<AutoStr, Vec<(field_name, Type)>>`
- 键：`"TypeName.field_name"` 和 bare `"field_name"`
- 用途：当赋值 `self.context_block = Some(context)` 时，查 `self` 的类型 → 查字段类型 → `Option<String>` → 对 `&str` 值自动加 `.to_string()`

**Pass 2：`known_fn_sigs` 外部函数签名表**（~60 行）
- 硬编码常见标准库/外部函数的返回类型：
  ```
  HashMap.get(key) → Option<&V>
  Vec.get(i) → Option<&T>
  Vec.push(v) → ()
  Vec.len() → usize
  HashMap.insert(k, v) → Option<V>
  HashMap.contains_key(k) → bool
  str.len() → usize
  Option.unwrap() → T
  Option.unwrap_or(d) → T
  Option.is_some() → bool
  Option.is_none() → bool
  Result.unwrap() → T
  a2r_std::fs::read_to_string(path) → String  (注意：不是 Result!)
  a2r_std::fs::write(path, content) → bool    (注意：不是 Result!)
  ```
- 用途：在表达式 emit 时，当遇到 `expr.method()` 且 method 在表中，知道返回类型 → 决定是否需要 unwrap / 处理 Option

**Pass 3：赋值时的类型自动转换**（~50 行）
- 在 `Stmt::Expr(Bina(lhs, Asn, rhs))` 的 emit 路径中：
  - 如果 `lhs` 是 `self.field`，查 `struct_field_types` 得到目标类型
  - 如果 `rhs` 类型与目标类型不匹配（`&str` → `String`、`Option<&T>` → `T` 等），自动插入转换
- 规则表：
  | 目标类型 | 源类型 | 自动转换 |
  |---|---|---|
  | `Option<String>` | `&str` / `String` | `Some(x.to_string())` |
  | `String` | `&str` | `x.to_string()` |
  | `&str` | `String` | `x.as_str()` 或 `&x` |
  | `T` | `Option<T>` | `x.unwrap()` / `x.unwrap_or_default()` |
  | `Vec<T>` | `Option<Vec<T>>` | `x.unwrap_or_default()` |

**Pass 4：`for-in-self.field` 借用修正**（~20 行）
- 当方法签名是 `&self`（非 `&mut self`），且 `for m in self.field` 被输出时：
  - 自动改为 `for m in &self.field`（借用，不 move）
- 已有 `fix_borrowing_issues` post_process 覆盖了部分模式，但此处是在生成阶段精确修正

#### 预期效果

| Pass | 预计消除错误 | 覆盖的错误码 |
|---|---|---|
| Pass 1（字段类型 → 自动转换） | ~15 | E0308（`Some(&str)` 赋 `Option<String>`） |
| Pass 2（外部函数签名 → Option 处理） | ~25 | E0308（`HashMap.get` 当值用）、E0599（Option 上方法） |
| Pass 3（赋值类型转换） | ~20 | E0308（String/`&str` 互转） |
| Pass 4（借用修正） | ~12 | E0507（`for m in self.field`） |
| **合计** | **~72** | **约 25% 的 re-transpile 错误** |

加上 plan 373 已有的 post_process 链（已消除 ~58 个），预计 re-transpile 错误
从 343 降到 **~170–200**（比当前 ~285 再降 ~30%）。

### 方案 B：完整类型检查（远期，不在本计划范围）

引入完整的类型系统（类似 rustc 的 MIR 类型检查），在生成前对每个表达式标注类型。
这是正确的长期方向，但工作量大（需要重构 a2r 架构），不适合在 plan 373 的延伸中做。

## 三、验证流程

```bash
cd D:/autostack/auto-lang
cargo build --release --bin auto     # 重编（含类型流分析）
# re-transpile 全部 .at
cd crates/auto-ai-agent/rust
./rebuild.sh                          # plan 373 的组装脚本（或手动）
cargo check 2>&1 | grep -c "^error"  # 看错误数下降
```

目标：每个 Pass 实施后 re-transpile 错误数下降。

## 四、风险评估

| 风险 | 缓解 |
|---|---|
| 类型推断错误导致 regression（正确代码被改坏） | 每个 Pass 只在确认不匹配时才插入转换；用 golden 测试套件验证 |
| `known_fn_sigs` 表维护成本 | 表很小（~15 条），且只覆盖标准库高频函数 |
| `struct_field_types` 跨模块不可见 | 在 `trans()` Phase 1 预扫描时全局收集（已有 `collect_fn_param_types` 模板） |
| 桥接函数签名差异（a2r_std vs std） | `known_fn_sigs` 表明确标注 a2r_std 的差异（`read_to_string → String` 而非 `Result`） |

## 五、不在本计划范围

- **组装层自动化**（lib.rs/config.rs 等 glue 注入）：已实验，非主要瓶颈
- **完整类型系统**（MIR 级别）：方案 B，远期
- **跨方法 mutation 分析**（间接 `&mut self`）：~5 个错误，ROI 低
- **ContentBlock struct-variant 通用化**：当前硬编码 3 个变体，够用
