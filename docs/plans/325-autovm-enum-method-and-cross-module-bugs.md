# Plan 325: AutoVM 基础缺陷修复（enum 方法 + 跨模块字符串 + print）— 高优先级

> **状态（2026-08-01 复核）**：缺陷 1/2/3 **全部已修复**。缺陷 2/3 在 6 周间由后续工作修复；
> **缺陷 1（enum 实例方法）于 2026-08-01 修复**（见 §实施记录）。
> **类型**：Bugfix（基础缺陷，**高优先级**）
> **严重度**：阻断性——比 Plan 316（#[api] panic）更基础，**阻塞所有后端 Auto 代码**（不只是 IO 层）
> **来源**：auto-musk Spec 数据模型层实现（路径 A 后端逻辑先行）实测发现
> **复现 commit**：auto-lang `17118eab`，auto.exe 构建 2026-06-16 16:58

---

## 背景与影响

auto-musk 决定用 AutoVM 脚本运行模式做后端。为绕开 #[api] server panic（Plan 316），我们尝试"后端逻辑层先行"——用 AutoVM 跑纯逻辑代码（不含 HTTP/SSE）。结果在**最基础的后端数据模型测试**上连续撞到 3 个 AutoVM 缺陷。

**核心结论**：AutoVM 当前在"跨模块 + enum + 字符串"这个组合下不稳定，连"跨模块调一个返回字符串的函数"都不可靠。这**不只是 #[api] 那一个点的问题，而是 AutoVM 基础成熟度问题**，阻塞了 auto-musk 的**所有**后端 Auto 代码（逻辑层和 IO 层都受阻）。

---

## 缺陷 1【最基础】：enum 实例方法不被调用

### 现象
enum 上定义的实例方法（`fn name() { is self {...} }`，隐式 self）**调用后方法体根本不执行**，直接返回 None。

### 最小复现（`enum_method_bug.at`）
```auto
pub enum Color {
    Red
    Green

    fn name() str {
        print("  [inside name method]")   // 这行【从不打印】
        return "called-ok"
    }
}

pub fn main() {
    let n = Color.Red.name()
    print(n)   // 输出 None（期望 called-ok）
}
```

### 实际输出
```
calling Color.Red.name():
result:
None
```
方法体内的 `print("inside name method")` **从未执行**，证明方法体未被调用；返回 None 而非 `called-ok`。

### 证据与对照
- stdlib `stdlib/auto/result.at:19-24` 的 `Result<T,E>` 用同样的 `fn is_ok() bool { is self {...} }` 写法。若这些方法同样不工作，则 stdlib 的 Result/May/Cmp 的实例方法在 AutoVM 下全部失效——影响面巨大，建议一并验证。
- `is self` 本身**没问题**：模块级函数 + `is` 匹配 enum 参数能正常工作（见缺陷 1 的绕过验证）。问题专属于"enum 实例方法"这个绑定机制。

### 影响
auto-coder 蓝本（`coder/forge/specs.at`）和 auto-musk 的数据模型，所有 `to_str`/`from_str`/`as_str` 都依赖 enum 方法——全部失效。

---

## 缺陷 2：跨模块（use）字符串返回错乱

### 现象
跨模块（`use specs`）调用返回字符串的函数，返回值损坏（`<invalid string index: 19>`）。

### 最小复现（`cross_module_str_bug.at`）
```auto
// specs.at（被 use 的模块）
pub enum SpecStatus { Empty Proposed Draft /* ... */ }

pub fn status_to_str(s SpecStatus) str {
    is s {
        SpecStatus.Empty -> return "empty"
        SpecStatus.Proposed -> return "proposed"
        SpecStatus.Draft -> return "draft"
    }
    return "draft"
}

// main（use specs）
use specs

pub fn main() {
    let s = status_to_str(SpecStatus.Proposed)
    print(s)   // 期望 proposed
}
```

### 实际输出
```
direct print s2:
<invalid string index: 19>
```
（`<invalid string index: 19>` 是 AutoVM 内部字符串操作错误的泄露）

### 关键对照
- **同模块内**字符串函数正常：缺陷 1 的绕过测试里，`color_name(Color.Red)`（同文件、模块级函数）正确返回 "red"。
- 问题专属于**跨模块**（`use specs` 后调用 specs 的字符串函数）。

### 影响
任何"模块定义数据模型/字符串转换 + 另一模块 use 它"的结构都不可靠——这是组织后端代码的基础模式。

---

## 缺陷 3：print 字面量重复 + 字符串拼接取错值（跨模块场景）

### 现象（与缺陷 2 同场景）
```auto
use specs
pub fn main() {
    let s1 = section_type_as_str(SectionType.Goals)
    print("direct print s1:")    // 打印了【两次】自己
    print(s1)
    print("got=" + s1)           // 得到 "got=direct print s1:"（取了 print 的字面量而非 s1）
}
```

### 实际输出
```
direct print s1:
direct print s1:        ← 重复
direct print s2:
<invalid string index: 19>
concat:
got=direct print s1:    ← 拼接取错值
```

### 影响
基础 IO（print）+ 字符串拼接在跨模块场景损坏。这让任何带输出的调试/测试都不可信。

---

## 附带观察：类型重复注册警告

每次跨模块 `use specs`，都打印：
```
Warning: Failed to register generic template 'SpecItem': Generic type 'SpecItem' already registered
```
（SpecItem/SpecsSection/SpecsDocument/SpecChange 各一次）

虽非致命，但说明模块加载有重复注册问题（可能是缺陷 2/3 的根因之一——重复注册导致符号/字符串表错乱）。建议修复时一并排查。

---

## 三个缺陷的共性

缺陷 1/2/3 都指向 **AutoVM 的模块系统 + 类型/字符串表管理在跨模块边界不可靠**。可能是同一根因（模块加载/符号注册的内存管理 bug）的不同表现。建议**作为一组排查**，而非三个独立 bug。

### 优先级判断
- **比 Plan 316（#[api] panic）更基础**：316 阻塞 IO 层（HTTP server），本组缺陷阻塞**逻辑层**（连纯数据模型测试都跑不动）。
- 修复本组缺陷是 auto-musk **任何后端工作**的前提（无论逻辑层还是后续 IO 层）。
- 建议与 316 并列为最高优先级，甚至先于 316（因为逻辑层工作量更大、更早需要）。

---

## 修复后请验证（auto-musk 阻塞项）

1. 缺陷 1 的复现脚本：`Color.Red.name()` 返回 `"called-ok"`，方法体 print 执行。
2. stdlib 的 `Result.is_ok()`/`May` 等实例方法是否本来就能工作（若不能，是更广的回归）。
3. 缺陷 2 的复现：跨模块 `status_to_str` 返回正确字符串。
4. 缺陷 3 的复现：print 不重复、拼接取对值。
5. **端到端**：auto-musk 的 `src/back/specs_test.at`（在 auto-musk 仓库）全绿——这是数据模型层的完整测试，覆盖 SectionType/SpecStatus 往返、SpecItem/SpecsDocument 工厂、tags 字段。

---

## auto-musk 侧的相关文件（修复后用于回归）

- `D:\autostack\auto-musk\src\back\specs.at` — Spec 数据模型层（已实现，用模块级函数绕过了缺陷 1，但缺陷 2/3 仍阻塞测试）
- `D:\autostack\auto-musk\src\back\specs_test.at` — 数据模型测试（全绿即本组缺陷已修）

注：auto-musk 已决定，待本组缺陷修复后，specs.at 的模块级函数写法（绕过缺陷 1）可以保留（它本就是合理的函数式风格），或回退为 enum 方法（若缺陷 1 修复且 enum 方法更符合语言惯例）——由 auto-musk 届时决定。

---

## 复现环境

- auto-lang commit `17118eab`，`target/debug/auto.exe`（2026-06-16 16:58 构建）
- auto-musk 的 specs.at / specs_test.at 在 `D:\autostack\auto-musk\src\back\`

---

## 根因确认（2026-08-01 复核）

### 缺陷 2/3 已修复

2026-08-01 用最新 master 实测：缺陷 2（跨模块 `status_to_str` 返回 `proposed` ✅）、缺陷 3（print 不重复、拼接 `got=proposed` ✅）**均已正常**。6 周间的后续工作（模块系统/字符串表相关修复）已消化这两个缺陷。**仅缺陷 1 仍存在。**

### 缺陷 1 根因：codegen 从未编译 enum 方法

**调用链追踪**：
1. **parser 正确解析** enum 方法——`parse_enum_body`（parser.rs:4628-4644）把 `fn name() {...}` 收集到 `methods: Vec<Fn>`，存入 `EnumKind::Heterogeneous { methods }`（enums.rs 的 EnumKind 定义）。
2. **codegen 丢弃了 methods**——`Stmt::EnumDecl` 处理（codegen.rs:2258-2330）只把变体注册进 `GenericRegistry`（line 2292/2315，注释明写「No methods for enum variants」），**完全没有提取 `enum_decl.kind` 里的 methods 并编译**。
3. 对照 `Stmt::TypeDecl`（codegen.rs:2172-2211）：它有 `for method in &type_decl.methods { ... compile_stmt(Fn) }` 把方法编译成独立函数（mangle 为 `TypeName.method`，注入 `self` 参数）注册到 exports。**EnumDecl 缺这段对应逻辑。**
4. 结果：`Color.name` / `Result.is_ok` 从未进 exports → `Color.Red.name()` 调用时无目标 → 静默返回 None（`Color.Red.name()`）或 CALL_SPEC 报错（`Result.Ok.is_ok()`）。

**两种失败模式的差异**：
- `Color.Red.name()` 返回 None：receiver `Color.Red` 被当作 enum 变体值（i32），`.name()` 走某条 fallback 静默失败。
- `Result.Ok(42).is_ok()` 报 `CALL_SPEC: no function 'Result.Ok.is_ok'`：receiver 是带 payload 的变体（heap 对象），CALL_SPEC 分发找不到方法。

---

## 修复方案（2026-08-01）

### 核心改动：EnumDecl 编译 methods（仿 TypeDecl）

在 `Stmt::EnumDecl` 处理末尾（codegen.rs ~2330），从 `enum_decl.kind` 提取 methods，按 TypeDecl 的模式编译为独立函数：

```rust
// Plan 325: 编译 enum 实例方法为独立函数（仿 TypeDecl methods，codegen.rs:2181）
// 方法名 mangle 为 EnumName.method，self 作为第一个参数（持有 enum 值）。
if let crate::ast::EnumKind::Heterogeneous { methods, .. } = &enum_decl.kind {
    for method in methods {
        let mangled_name = format!("{}.{}", enum_name, method.name);
        let mut method_fn = method.clone();
        method_fn.name = crate::ast::Name::from(mangled_name.as_str());
        method_fn.parent = Some(crate::ast::Name::from(enum_name.as_str()));
        // 实例方法注入 self（enum 值：i32 discriminant 或 heap 对象）
        if !method.is_static {
            let has_self = method_fn.params.first()
                .map(|p| p.name.to_string() == "self").unwrap_or(false);
            if !has_self {
                method_fn.params.insert(0, crate::ast::Param {
                    name: crate::ast::Name::from("self"),
                    ty: Type::Unknown,  // enum 值类型（运行时是 i32 或对象 id）
                    default: None,
                    mode: crate::ast::ParamMode::View,
                });
            }
        }
        self.compile_stmt(&Stmt::Fn(method_fn))?;
    }
}
```

### 关键设计点

1. **`self` 的运行时表示**：enum 值在 VM 里是 i32（scalar 变体的 discriminant）或 heap 对象 id（带 payload 的变体）。方法体里 `is self {...}` 用既有 `is` 匹配机制（按值/类型匹配变体），无需新机制。
2. **方法名 mangle**：`Color.name` / `Result.is_ok`，与调用端 `Color.Red.name()` 的解析对齐（codegen 已有 `is_enum_variant` + 方法分发的 fallback 链）。
3. **仅 Heterogeneous 有 methods**：`EnumKind::Scalar` / `Homogeneous` 无 methods 字段（parser 只把方法存进 Heterogeneous）。但为稳健，可对所有 kind 都尝试提取（若 enum 有方法，parser 会把它归为 Heterogeneous）。

### 调用端验证

`Color.Red.name()` 的 codegen 已有 enum 变体识别（`is_enum_variant`，codegen.rs:6288）。编译方法进 exports 后，调用端应能解析到 `Color.name`。若调用端仍走错路径（如把 `.name()` 当 CALL_SPEC 而非 CALL），需补"enum 方法名在 exports 中 → 走 CALL"的判断（类似 TypeDecl 的 `is_user_type_method`）。

### 测试

- file-based：`test/vm/28_enum_methods/`（新增 category）
  - `001_basic/`：`Color.Red.name()` 返回 `called-ok`，方法体 print 执行（缺陷 1 原复现）
  - `002_is_match/`：`is self {...}` 匹配——`Color.Red.is_red()` 返回 true、`Color.Green.is_red()` 返回 false
  - `003_payload_method/`：带 payload 变体的方法（`Result.Ok(42).is_ok()` 返回 true）
  - `004_stdlib_result/`：stdlib `Result.is_ok()`/`is_err()` 实例方法工作（影响面守护）
- 回归：既有 enum 测试（test/vm 里 enum 相关）不回归。

### 风险

| 风险 | 缓解 |
|------|------|
| `is self` 匹配 self 参数在方法上下文不工作 | 002 测试覆盖；`is` 匹配按值工作，self 是普通参数 |
| enum 方法与同名变体冲突（如 `Color.Empty` 既是变体又有方法） | mangle 名 `Color.method` vs 变体 `Color.Empty` 不冲突（不同符号空间） |
| stdlib Result/May 方法修复后行为变化 | 004 守护；这些方法本应工作，修复是恢复而非破坏 |
| 静态 enum 方法（`is_static`）未注入 self | 方案里已判 `!method.is_static`，静态方法不注入 |

---

## 实施记录（2026-08-01，缺陷 1 修复）

### 实际改动

**方法编译**（`codegen.rs` `Stmt::EnumDecl` ~2330）：在变体注册后，从 `EnumKind::Heterogeneous { methods }` 提取方法，按 TypeDecl 模式（codegen.rs:2181）编译为独立函数——方法名 mangle 为 `EnumName.method`，实例方法注入 `self`（持有 enum 值：scalar 是 i32 discriminant，带 payload 是 heap 对象 id）作首参，`compile_stmt(&Stmt::Fn(method_fn))` 注册到 exports。

**调用端分发**（`codegen.rs` func_name 构造 `_ =>` 分支 ~6773）：新增 enum 变体方法识别。receiver 是 `Color.Red`（Dot）或 `Reg.Yes(42)`（Call{Dot}）时，提取 enum 名，若该 enum 存在（enum_values/types/generic_registry）且 `EnumName.method` 在 exports，func_name 用之。解决"infer_object_type 对 enum 变体只推得出 Int"的问题。

### 测试（test/vm/28_enum_methods/）

- `001_basic`：`Color.Red.name()` 返回 `called-ok` ✅（缺陷 1 原复现，方法体执行）
- `002_is_match`：`Light.Red.is_red()` = 1、`Light.Green.is_red()` = 0 ✅（`is self` 匹配）
- `003_payload_method`：`Reg.Yes(42).is_yes()` = 1、`Reg.No.is_yes()` = 0 ✅（带 payload 变体方法）

### 回归

VM non-ignored：21 FAILED = master 基线 21（零新增）。enum 相关既有测试无回归。

### 已知遗留（独立 bug，本计划修复）

修复 enum 方法后暴露一个**既有的 `_` 通配 bug**：`is` 语句里 `_ -> ...` arm 报 `Undefined variable: _`。**已于 2026-08-01 修复**（见下"根因/修复方案/验证"）。stdlib `result.at` 的 `is_ok`/`is_err`（用 `_ -> false`）受益于本修复——但 `Result` 作为泛型 enum + `use auto.result` 导入时方法分发另有问题（`CALL_SPEC: no function 'Result.Ok.is_ok'`），属独立后续项。

#### 根因（2026-08-01 定位）

**词法层**：`_` 没有专门的通配符 token——被词法化为普通 `Ident("_")`（token.rs 无 Underscore/Wildcard 关键字）。

**parser 层**（`is_branch_cond_expr_inner`，parser.rs:3447）：`_` 作为 Ident 走 `lhs_expr()`（line 3469），被解析为普通表达式 `Expr::Ident("_")`。**没有任何地方把 `_` 转成通配符 pattern**（`Cover` 枚举只有 `Tag`，无 `Wildcard` 变体）。`Cover` 的 `TagCover.bindings` 里 `_` 仅用于"忽略该绑定"（parser.rs:6523/3511），但 arm 左侧的裸 `_` 不是 binding。

**codegen 层**（`Stmt::Is` 的 `EqBranch`，codegen.rs:3489）：pattern match 的默认分支把 pattern 当普通表达式 `compile_expr(pattern)` 然后 `EQ` 比较。`Expr::Ident("_")` → `compile_expr` 试图加载变量 `_` → `Undefined variable: _`。

**关键对照**：`Some(_)` / `Ok(_)` 能工作——因为 parser 的 `parse_option_pattern`/`parse_result_pattern` 把 `(_)` 作为 binding 名 `_` 处理，codegen（line 3277）特判 `binding != "_"`。但**裸 `_` arm（不包裹在 Some/Ok 里）没有对应特判**。

#### 修复方案

在 codegen 的 `EqBranch` pattern match 里，特判 `Expr::Ident("_")` 为**总匹配**（wildcard always matches）：不发 `compile_expr` + `EQ`，直接执行 body。这是最小改动——不引入新 AST 节点、不改 parser、不动 `Cover` 枚举。

```rust
// codegen.rs EqBranch 的 pattern match，在默认 _ => 分支前加：
crate::ast::Expr::Ident(name) if name.as_str() == "_" => {
    // 通配符 _ —— 总是匹配，直接执行 body（不发 EQ 比较）
    // （body 编译 + JMP end 在外层统一处理）
}
```

需在两处 `EqBranch` 处理（Stmt::Is codegen.rs:3239 + Expr::If 的 is 分发 codegen.rs:8797）各加一个通配符 arm。

#### 实施（2026-08-01）

在两处 `EqBranch` 的 `_ =>` 默认分支开头，各加 `_` 通配特判：单 pattern 且 `patterns[0] == Expr::Ident("_")` 时，跳过 `compile_expr + EQ`，直接编译 body + `JMP end`（总匹配，仿 ElseBranch）。

**测试**：`test/vm/28_enum_methods/004_wildcard/`——enum 方法 `label()` 里 `Light.Red -> "red"; _ -> "other"`，验证 `Light.Red`→red、`Light.Green`/`Light.Blue`→other（catch-all 生效）。全绿。

**回归**：VM non-ignored 21=21 基线，零新增。既有 `is` 匹配（具名 variant、Some/Ok pattern）不受影响。

#### 验证

- ✅ `is x { _ -> ... }` 不再报 `Undefined variable: _`，总匹配 arm 执行
- ✅ enum 方法 `label()` 里 `Light.Red -> "red"; _ -> "other"` 协同工作（004 测试）
- ⚠️ stdlib `result.at` 的 `is_ok`/`is_err` 仍报 `CALL_SPEC: no function 'Result.Ok.is_ok'`——这是泛型 enum + use 导入的方法分发问题，非 `_` 通配（独立后续项）
- ✅ 回归：既有 `is` 匹配不受影响
