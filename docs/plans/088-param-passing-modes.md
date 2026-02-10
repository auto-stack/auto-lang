# Plan 088: 函数参数传递模式实现 - 语义 View，实现 Copy 优化

**主题**: 语义上统一 View，实现上自动 Copy (Semantic View, Implementation Copy)

**目标**: 实现函数参数的智能传递策略，在保持"默认不可变借用"语义的同时，利用自动优化获得最大性能

**优先级**: **高** - Phase 3 泛型方法支持的前提条件

**依赖**: [Plan 024](024-ownership-first-implementation.md) (所有权系统), [param-passing-default.md](../design/param-passing-default.md) (设计文档)

**工作量**: 2-3 周

**设计原则**:
1. **用户侧**: 所有参数默认都是 `view`（不可变引用）
2. **实现侧**: 小对象自动 Copy 优化，大对象引用传递
3. **安全性**: 前端禁止修改 view 参数

---

## 设计文档参考

本计划基于 **[param-passing-default.md](../design/param-passing-default.md)** (ABO-01) 详细设计文档实现。

### 核心策略

> **"语义上统一 View，实现上自动 Copy"**

- **用户视角**: 所有参数都是 `view`（不可变引用）
- **编译器视角**: 自动优化小对象的传递方式
- **结果**: 简洁的语义 + 最优的性能

### 类型分类

**小对象（值传递优化）**:
```auto
int, float, bool, char, byte  // 原生类型
enum (C-style, 无数据)
```

**大对象（引用传递）**:
```auto
string, vector, map  // 堆分配
struct, closure      // V1 暂不优化
```

### 语义映射矩阵

| Auto 类型 | Auto 语义 | VM 实际传递 | Rust 后端 |
|-----------|----------|-------------|-----------|
| `int` | View (ReadOnly) | 值传递（优化） | `fn foo(x: i64)` |
| `bool` | View (ReadOnly) | 值传递（优化） | `fn foo(x: bool)` |
| `string` | View (ReadOnly) | 引用传递 | `fn foo(x: &String)` |
| `struct` | View (ReadOnly) | 引用传递 | `fn foo(x: &MyStruct)` |

---

## 问题背景

当前 AutoVM 的函数参数都是**值传递**（copy），导致方法无法修改调用者的对象：

```auto
type Point {
    x int
    fn set_x(self, new_x int) void {
        self.x = new_x  // ❌ 不生效 - self 是副本
    }
}

let p = Point{x: 10}
p.set_x(100)
say(p.x)  // 输出: 10 (不是 100)
```

**根因**: AutoVM 当前的 `self` 是**值传递**（Copy），不是**引用传递**（View/Mut）

---

## 解决方案

### Phase 1: 类型系统扩展（1-2 天）

#### 1.1 添加 `is_optimized_by_value()` 方法

**文件**: `crates/auto-lang/src/ast/types.rs`

```rust
impl Type {
    /// 判断是否应该进行"值传递优化"
    /// 参考: param-passing-default.md Section 2.1
    pub fn is_optimized_by_value(&self) -> bool {
        match self {
            // 小对象：值传递优化
            Type::Int | Type::Uint | Type::I8 | Type::U8 |
            Type::I64 | Type::U64 | Type::Byte |
            Type::Bool | Type::Char | Type::Float(_) | Type::Double(_) => true,

            // 大对象：引用传递
            Type::Str(_) => false,  // string 引用传递
            Type::Array(..) => false,
            Type::Object | Type::Tag(_) => false,  // struct 引用传递

            // 其他
            Type::Fn | Type::Closure => false,
            Type::Unknown | Type::Nil | Type::Null => false,
            _ => false,
        }
    }

    /// 判断类型是否实现 Copy spec（未来扩展用）
    pub fn is_copy(&self) -> bool {
        // V1: 简化版本，直接使用 is_optimized_by_value
        // 未来: 检查 TypeDecl.spec_impls 是否包含 Copy
        self.is_optimized_by_value()
    }
}
```

**测试**: `cargo test -p auto-lang types`

---

### Phase 2: AST 更新（1 天）

#### 2.1 添加 `ParamMode` 枚举

**文件**: `crates/auto-lang/src/ast/fun.rs`

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ParamMode {
    Copy,  // 显式值传递
    View,  // 不可变引用（默认）
    Mut,   // 可变引用
    Take,  // Move 语义
}

impl Default for ParamMode {
    fn default() -> Self {
        Self::View  // ✅ 默认 View
    }
}

impl fmt::Display for ParamMode {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Copy => write!(f, "copy"),
            Self::View => write!(f, "view"),
            Self::Mut => write!(f, "mut"),
            Self::Take => write!(f, "take"),
        }
    }
}
```

#### 2.2 扩展 `Param` 结构体

**文件**: `crates/auto-lang/src/ast/fun.rs`

```rust
#[derive(Debug, Clone)]
pub struct Param {
    pub name: Name,
    pub ty: Type,
    pub default: Option<Expr>,
    pub mode: ParamMode,  // ✅ 新增字段
}

impl Param {
    pub fn new(name: Name, ty: Type, default: Option<Expr>) -> Self {
        Self {
            name,
            ty,
            default,
            mode: ParamMode::default(),  // ✅ 默认 View
        }
    }

    pub fn with_mode(name: Name, ty: Type, default: Option<Expr>, mode: ParamMode) -> Self {
        Self {
            name,
            ty,
            default,
            mode,
        }
    }
}

impl fmt::Display for Param {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "(param (name {}) (type {}) (mode {})",
            self.name, self.ty, self.mode)?;
        if let Some(default) = &self.default {
            write!(f, " (default {})", default)?;
        }
        write!(f, ")")
    }
}
```

---

### Phase 3: Parser 解析（2-3 天）

#### 3.1 解析参数模式

**文件**: `crates/auto-lang/src/parser.rs`

修改 `fn_params()` 函数：

```rust
pub fn fn_params(&mut self) -> AutoResult<Vec<Param>> {
    let mut params = Vec::new();
    while self.is_kind(TokenKind::Ident) {
        // 1. 检查参数传递模式（可选，默认 View）
        let mut mode = ParamMode::default();  // ✅ 默认 View

        if self.is_kind(TokenKind::Copy) {
            mode = ParamMode::Copy;
            self.next(); // skip 'copy'
        } else if self.is_kind(TokenKind::View) {
            mode = ParamMode::View;
            self.next(); // skip 'view'
        } else if self.is_kind(TokenKind::Mut) {
            mode = ParamMode::Mut;
            self.next(); // skip 'mut'
        } else if self.is_kind(TokenKind::Take) {
            mode = ParamMode::Take;
            self.next(); // skip 'take'
        }

        // 2. param name
        let name = self.cur.text.clone();
        let name_pos = self.cur.pos;
        self.next(); // skip name

        // 3. param type (skip ':' if present)
        let mut ty = Type::Int;
        if self.is_kind(TokenKind::Colon) {
            self.next(); // skip ':'
        }
        if self.is_type_name() {
            ty = self.parse_type()?;
        }

        // 4. default value
        let mut default = None;
        if self.is_kind(TokenKind::Asn) {
            self.next(); // skip =
            let expr = self.parse_expr()?;
            default = Some(expr);
        }

        // 5. 定义参数到作用域
        let var = Store {
            kind: StoreKind::Var,
            name: name.clone(),
            expr: default.clone().unwrap_or(Expr::Nil),
            ty: ty.clone(),
        };
        self.define(name.as_str(), Meta::Store(var.clone()));

        // 6. 注册符号位置
        let loc = SymbolLocation::new(
            name_pos.line.saturating_sub(1),
            name_pos.at,
            name_pos.pos,
        );
        self.scope.borrow_mut().define_symbol_location(name.clone(), loc);

        // ✅ 创建参数（包含 mode）
        params.push(Param { name, ty, default, mode });
        self.sep_params()?;
    }

    // Handle variadic arguments (...)
    // ...

    Ok(params)
}
```

#### 3.2 单元测试

```rust
#[test]
fn test_param_mode_parsing() {
    // 默认 view
    assert_eq!(parse_one("fn foo(x int)"),
        fn_decl("foo", [param("x", INT, View)]));

    // 显式 copy
    assert_eq!(parse_one("fn foo(copy x int)"),
        fn_decl("foo", [param("x", INT, Copy)]));

    // 显式 mut
    assert_eq!(parse_one("fn set_x(mut self Point, new_x int)"),
        fn_decl("set_x", [param("self", Point, Mut), param("new_x", INT, View)]));
}
```

---

### Phase 4: Codegen 编译（3-4 天）

#### 4.1 添加新指令

**文件**: `crates/auto-lang/src/vm/opcode.rs`

```rust
pub enum OpCode {
    // ... existing opcodes ...

    // Plan 088: Reference passing opcodes
    LOAD_REF,       // 加载不可变引用（对象ID）
    LOAD_MUT_REF,   // 加载可变引用（对象ID + 可变标记）
    STORE_REF,      // 存储不可变引用
    STORE_MUT_REF,  // 存储可变引用
    STORE_TAKE,     // Move 语义（所有权转移）
}
```

#### 4.2 智能参数编译

**文件**: `crates/auto-lang/src/vm/codegen.rs`

**关键**: 实现自动分流优化

```rust
// 在 compile_call() 中编译参数
for (i, arg) in call.args.iter().enumerate() {
    match arg {
        Arg::Pos(expr) => {
            // 获取目标函数的参数信息
            let (param_ty, param_mode) = self.get_param_info(&func_name, i)?;

            match expr {
                Expr::Ident(name) => {
                    let var_index = self.lookup_var(&name.to_string())?;

                    // ✅ 核心优化：自动分流
                    match param_mode {
                        ParamMode::View => {
                            if param_ty.is_optimized_by_value() {
                                // 优化：小对象直接值传递
                                self.emit_load_loc(var_index);
                                eprintln!("DEBUG: Optimizing {} param '{}' as value (copy)", param_ty, name);
                            } else {
                                // 默认：大对象引用传递
                                self.emit_load_ref(var_index);
                                eprintln!("DEBUG: Passing {} param '{}' by reference", param_ty, name);
                            }
                        }
                        ParamMode::Mut => {
                            if param_ty.is_optimized_by_value() {
                                // 可变引用 + Copy 类型：值传递
                                self.emit_load_loc(var_index);
                            } else {
                                // 可变引用 + 大类型：可变引用
                                self.emit_load_mut_ref(var_index);
                            }
                        }
                        ParamMode::Take => {
                            // Move 语义：转移所有权
                            self.emit_load_loc(var_index);
                        }
                        ParamMode::Copy => {
                            // 显式 Copy
                            if param_ty.is_optimized_by_value() {
                                self.emit_load_loc(var_index);
                            } else {
                                // 大对象显式 copy：需要 clone
                                self.emit_clone(var_index);
                            }
                        }
                    }
                }
                _ => {
                    // 常量等直接编译
                    self.compile_expr(expr)?;
                }
            }
        }
    }
}
```

**辅助函数**:

```rust
impl Codegen {
    fn emit_load_ref(&mut self, var_index: usize) {
        match var_index {
            0 => self.emit(OpCode::LOAD_REF_0),
            1 => self.emit(OpCode::LOAD_REF_1),
            2 => self.emit(OpCode::LOAD_REF_2),
            _ => {
                self.emit(OpCode::LOAD_REF);
                self.emit_u32(var_index as u32);
            }
        }
    }

    fn emit_load_mut_ref(&mut self, var_index: usize) {
        match var_index {
            0 => self.emit(OpCode::LOAD_MUT_REF_0),
            1 => self.emit(OpCode::LOAD_MUT_REF_1),
            2 => self.emit(OpCode::LOAD_MUT_REF_2),
            _ => {
                self.emit(OpCode::LOAD_MUT_REF);
                self.emit_u32(var_index as u32);
            }
        }
    }
}
```

---

### Phase 5: VM 执行（3-4 天）

#### 5.1 实现引用类型

**文件**: `crates/auto-lang/src/val/value.rs`

```rust
#[derive(Debug, Clone)]
pub enum Value {
    // ... existing variants ...

    // Plan 088: Reference types
    VmRef(VmRef),       // 不可变引用（对象ID）
    VmMutRef(VmMutRef), // 可变引用（对象ID + 可变标记）
}

#[derive(Debug, Clone)]
pub struct VmRef {
    pub id: usize,  // 引用的对象 ID
}

#[derive(Debug, Clone)]
pub struct VmMutRef {
    pub id: usize,  // 引用的对象 ID（可变）
}
```

#### 5.2 实现指令

**文件**: `crates/auto-lang/src/vm/engine.rs`

```rust
OpCode::LOAD_REF => {
    // 加载不可变引用
    let local_index = task.read_u32() as usize;
    let obj_id = task.get_object_id(local_index);  // 从局部变量获取对象ID
    task.ram.push(Value::VmRef(VmRef { id: obj_id }));
}

OpCode::LOAD_MUT_REF => {
    // 加载可变引用
    let local_index = task.read_u32() as usize;
    let obj_id = task.get_object_id(local_index);
    task.ram.push(Value::VmMutRef(VmMutRef { id: obj_id }));
}

OpCode::STORE_REF => {
    let local_index = task.read_u32() as usize;
    let value = task.ram.pop();
    task.set_local(local_index, value);
}

OpCode::STORE_MUT_REF => {
    let local_index = task.read_u32() as usize;
    let value = task.ram.pop();
    task.set_local(local_index, value);
}

OpCode::STORE_TAKE => {
    // Move 语义：从栈弹出并转移所有权
    let value = task.ram.pop();
    let local_index = task.read_u32() as usize;
    task.set_local(local_index, value);
}
```

#### 5.3 字段访问时的引用处理

```rust
OpCode::GET_FIELD => {
    let field_idx = task.read_u16();
    let obj_value = task.ram.pop();

    match obj_value {
        Value::VmRef(vm_ref) => {
            // 不可变引用：读取字段
            let obj = task.vm.get_object(vm_ref.id)?;
            let field_val = obj.get_field(field_idx)?;
            task.ram.push(field_val);
        }
        Value::VmMutRef(vm_mut_ref) => {
            // 可变引用：可以读取
            let obj = task.vm.get_object(vm_mut_ref.id)?;
            let field_val = obj.get_field(field_idx)?;
            task.ram.push(field_val);
        }
        _ => {
            // 常规对象访问（值类型）
            // 现有逻辑
        }
    }
}

OpCode::SET_FIELD => {
    let field_idx = task.read_u16();
    let new_value = task.ram.pop();
    let obj_value = task.ram.pop();

    match obj_value {
        Value::VmMutRef(vm_mut_ref) => {
            // ✅ 可变引用：允许修改
            let obj = task.vm.get_object_mut(vm_mut_ref.id)?;
            obj.set_field(field_idx, new_value)?;
        }
        Value::VmRef(_) => {
            // ❌ 不可变引用：编译时应该报错
            return Err("Cannot modify through immutable reference".into());
        }
        _ => {
            // 常规对象
            // 现有逻辑
        }
    }
}
```

---

### Phase 6: 类型检查器 - 不可变性保证（2-3 天）

**文件**: `crates/auto-lang/src/typeck/mod.rs` 或创建新文件

```rust
pub struct TypeChecker {
    // 检查 view 参数是否被修改
    fn check_fn_decl(&mut self, fn_decl: &Fn) -> Result<(), Vec<TypeError>> {
        let mut errors = Vec::new();

        // 收集所有 view 参数
        let view_params: HashSet<Name> = fn_decl.params.iter()
            .filter(|p| p.mode == ParamMode::View)
            .map(|p| p.name.clone())
            .collect();

        // 检查函数体
        self.check_body_immutable(&fn_decl.body, &view_params, &mut errors)?;

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }

    fn check_body_immutable(
        &mut self,
        body: &Body,
        view_params: &HashSet<Name>,
        errors: &mut Vec<TypeError>
    ) -> Result<(), ()> {
        for stmt in &body.stmts {
            match stmt {
                Stmt::Store(store) => {
                    // 检查是否在修改 view 参数
                    if view_params.contains(&store.name) {
                        errors.push(TypeError::CannotModifyViewParam {
                            param: store.name.clone(),
                            span: pos_to_span(store.expr.pos()),
                        });
                    }
                }
                // ... 其他语句检查 ...
            }
        }
        Ok(())
    }
}

#[derive(Error, Debug, Diagnostic)]
#[error("Cannot modify view parameter '{param}'")]
pub struct CannotModifyViewParam {
    pub param: Name,
    #[label("parameter '{param}' is declared as view (immutable)")]
    #[label("consider using 'mut' instead")]
    pub span: SourceSpan,
}
```

---

## 实现阶段

### Phase 1: 类型系统扩展（1-2 天）
- ✅ 添加 `is_optimized_by_value()` 方法
- ✅ 添加 `is_copy()` 方法
- ✅ 10 单元测试

### Phase 2: AST 更新（1 天）
- ✅ `ParamMode` 枚举
- ✅ 扩展 `Param` 结构体
- ✅ `Display` 实现
- ✅ 5 单元测试

### Phase 3: Parser 解析（2-3 天）
- ✅ `fn_params()` 解析参数模式
- ✅ 支持类型内部方法声明
- ✅ 15 单元测试

### Phase 4: Codegen 编译（3-4 天）
- ✅ 添加引用指令（LOAD_REF, STORE_REF, etc.）
- ✅ 智能参数编译（自动分流优化）
- ✅ 20 单元测试

### Phase 5: VM 执行（3-4 天）
- ✅ VmRef/VmMutRef 类型
- ✅ 指令执行
- ✅ 字段访问引用处理
- ✅ 25 单元测试

### Phase 6: 类型检查器（2-3 天）
- ✅ 不可变性检查
- ✅ View 参数修改检测
- ✅ 15 单元测试

### Phase 7: 集成测试（2-3 天）
- ✅ 端到端测试
- ✅ 性能基准测试
- ✅ 15 集成测试

---

## 验证标准

### 功能完整性

| 功能 | 测试 | 预期结果 |
|------|------|---------|
| 默认 View | `fn add(a int, b int)` | `a, b` 是 view，但实际值传递（优化） |
| 大对象引用 | `fn process(view p Point)` | `p` 引用传递 |
| Mut 修改 | `fn set_x(mut self, x int)` | 可以修改原对象 |
| 不可变性检查 | `fn bad(view a int) { a = 2 }` | 编译错误 |
| Take Move | `fn consume(take s str)` | `s` 使用后报错 |

### 性能目标

| 操作 | 未优化 | 优化后 | 提升 |
|------|--------|--------|------|
| `add(int, int)` | 引用传递 | 值传递 | 2-5x |
| `process(view Point)` | 值传递 | 引用传递 | 10-100x |
| `string` 参数 | 值传递 | 引用传递 | 避免大拷贝 |

### 测试覆盖

- **单元测试**: 110 个
- **集成测试**: 15 个
- **性能基准**: 10 个

---

## 语法示例

### 默认 View（自动优化）

```auto
// 小对象：值传递优化
fn add(a int, b int) int {
    // ✅ a, b 实际是值（已优化）
    return a + b
}

let x = 5
let y = add(x, 10)  // ✅ x 仍可用（值传递）
```

### 大对象引用传递

```auto
type Point {
    x int
    y int
    data: [1000]int
}

fn get_x(view p Point) int {
    // ✅ p 是引用（不复制 1000 个 int）
    return p.x
}

let pt = Point{x: 1, y: 2, data: [...]}
let x = get_x(pt)  // ✅ 零拷贝
```

### Mut 可变引用

```auto
type Point {
    x int
}

fn set_x(mut self Point, new_x int) void {
    // ✅ self 是可变引用
    self.x = new_x  // 修改原对象
}

let p = Point{x: 10}
set_x(p, 100)
say(p.x)  // ✅ 输出: 100
```

### 不可变性检查

```auto
fn bad(view a int) int {
    a = 2  // ❌ 编译错误：Parameter 'a' is view-only
    return a
}
```

---

## 向后兼容

**破坏性变更**: 是

从当前的"值传递默认"改为"引用传递默认"是破坏性变更，但提供了清晰的迁移路径：

### 迁移策略

1. **V1（当前实现）**: 所有参数 Copy
   ```auto
   fn add(a int, b int) int { a + b }  // 值传递
   ```

2. **V2（目标实现）**: 默认 View，自动优化
   ```auto
   fn add(a int, b int) int { a + b }  // view 语义，值传递优化
   ```

**兼容性**:
- ✅ **功能兼容**: `a + b` 在两种实现中都能工作
- ✅ **性能提升**: V2 对小对象保持高性能
- ⚠️ **语义变化**: V2 中参数不可修改（编译时检查）

### 迁移步骤

1. **实现 Phase 1-5**（参数传递基础设施）
2. **添加不可变性检查**（Phase 6）
3. **修复现有代码**（如果违反不可变性）
4. **移除旧的纯 Copy 语义**

---

## 关键文件清单

### 新建文件
1. `crates/auto-lang/src/typeck/param_check.rs` (~300 行) - 参数不可变性检查
2. `crates/auto-lang/src/vm/ref_types.rs` (~200 行) - 引用类型定义

### 修改文件
1. `crates/auto-lang/src/ast/types.rs` (+20 行) - `is_optimized_by_value()`
2. `crates/auto-lang/src/ast/fun.rs` (+50 行) - `ParamMode`, `Param` 扩展
3. `crates/auto-lang/src/parser.rs` (+30 行) - 解析参数模式
4. `crates/auto-lang/src/vm/opcode.rs` (+10 行) - 新指令
5. `crates/auto-lang/src/vm/codegen.rs` (+150 行) - 智能参数编译
6. `crates/auto-lang/src/vm/engine.rs` (+200 行) - 引用类型执行
7. `crates/auto-lang/src/val/value.rs` (+20 行) - `VmRef`, `VmMutRef`

### 参考文件
- [param-passing-default.md](../design/param-passing-default.md) - 详细设计文档
- [Plan 024](024-ownership-first-implementation.md) - 所有权系统
- [Plan 026](026-property-keywords.md) - 属性关键字

---

## 成功指标

### 功能完整性
- ✅ 默认 View（引用语义）
- ✅ 小对象自动 Copy 优化
- ✅ 大对象引用传递
- ✅ Mut 可变引用修改对象
- ✅ Take Move 语义
- ✅ 编译时不可变性检查

### 性能目标
- `add(int, int)`: 零额外开销（寄存器传递）
- `process(view Point)`: 避免大对象复制
- `string` 参数: 引用传递，避免拷贝

### 测试覆盖
- 110 单元测试
- 15 集成测试
- 10 性能基准
- 零回归（现有 1250+ 测试）

---

## 相关计划

- **[Plan 024](024-ownership-first-implementation.md)**: 所有权系统基础
- **[Plan 026](026-property-keywords.md)**: 属性关键字（.view, .mut, .take）
- **[Plan 087](087-autovm-generics-type-erasure-specialization.md)**: 泛型方法支持（依赖本计划）
- **[param-passing-default.md](../design/param-passing-default.md)**: 设计文档 ABO-01

---

**时间估算**: 2-3 周

**下一步**: 实现 Phase 1（类型系统扩展），添加 `is_optimized_by_value()` 方法
