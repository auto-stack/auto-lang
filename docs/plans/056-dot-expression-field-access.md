# AutoLang 点表达式和字段访问实现计划

## 目标

实现完整的点表达式（`.`）和字段访问支持，使 AutoLang 能够：
1. 在类型中声明字段（`data []T`, `len int`, `cap int`）
2. 在实例方法中访问字段（`.data`, `.len`, `.cap`）
3. 区分字段访问、静态方法调用和实例方法调用
4. 支持字段读取和赋值操作

## 核心愿景

**用户体验**：用户可以编写纯 AutoLang 的数据结构：
```auto
type Vec<T> {
    data []T    // 底层数据
    len int     // 当前长度
    cap int     // 容量

    fn push(elem T) {
        if .len >= .cap {
            .realloc(.cap * 2)
        }
        .data[.len] = elem
        .len = .len + 1
    }
}

let v = Vec.new()
v.push(42)       // ✅ 实例方法调用
let len = v.len   // ✅ 字段读取
v.cap = 8         // ✅ 字段赋值
```

## 问题分析

### 当前问题清单

根据探索结果，发现以下问题：

#### 1. **AST 结构缺陷** ✅ 已完成
- **文件**: `crates/auto-lang/src/ast.rs:235-280`
- **问题**: 点表达式被表示为 `Expr::Bina(left, Op::Dot, right)`，没有专门的 `Expr::Dot` 类型
- **影响**: 语义不明确，难以区分字段访问和方法调用
- **当前行为**: `obj.field` → `Expr::Bina(Expr::Ident("obj"), Op::Dot, Expr::Ident("field"))`
- **期望行为**: 应该有 `Expr::Dot { object, field }` 类型

#### 2. **字段读取缺失** ✅ 已完成
- **文件**: `crates/auto-lang/src/eval.rs`
- **问题**: 只支持字段赋值，不支持字段读取
- **当前行为**: `obj.field = value` ✅ 工作，`obj.field` ❌ 不工作
- **影响**: 无法在表达式中使用字段值
- **需要**: 添加字段读取的求值逻辑

#### 3. **解析逻辑不完整** ✅ 已完成
- **文件**: `crates/auto-lang/src/parser.rs`
- **问题**: 点表达式当作普通二元表达式处理，没有专门优化
- **当前状态**: `PREC_DOT = infix_prec(17)` 有正确的优先级
- **缺失**: 没有专门的 `parse_dot` 函数
- **影响**: 解析效率低，容易出错

#### 4. **类型成员无运行时映射** ✅ 已完成
- **文件**: `crates/auto-lang/src/ast/types.rs:490-494`
- **问题**: 类型声明中的 `Member` 只是语法结构，没有运行时映射
- **当前行为**:
```auto
type File {
    path str  // ✅ 可以声明
}
// ❌ 但 File.path 无法在运行时访问
```
- **影响**: 类型字段无法使用

#### 5. **方法调用与字段访问混淆** ✅ 已完成
- **文件**: `crates/auto-lang/src/eval.rs:1900-2024`
- **问题**: 点表达式被统一处理，无法区分：
  - 静态方法调用: `List.new()`
  - 实例方法调用: `list.push(1)`
  - 字段读取: `list.len`
  - 字段赋值: `list.cap = 8`
- **影响**: 代码逻辑复杂，容易出错

#### 6. **点表达式求值路径不完整** ✅ 已完成
- **文件**: `crates/auto-lang/src/eval.rs:1117-1266`
- **当前支持**:
  - ✅ 字段赋值: `obj.field = value`
  - ✅ 嵌套字段赋值: `obj.inner.field = value`
  - ✅ 数组索引字段: `arr[0].field = value`
- **缺失支持**:
  - ❌ 字段读取: `let x = obj.field`
  - ❌ 链式字段访问: `obj.field1.field2`
  - ❌ 字段作为方法参数: `say(obj.field)`

#### 7. **缺少字段访问测试** ⚠️ 不工作
- **当前**: 有简单的对象字段赋值测试
- **缺失**:
  - 字段读取测试 ✅
  - 类型字段访问测试 ✅
  - 混合字段和方法调用测试
  - 边界情况测试

---

## 实施计划

### 阶段 1：AST 结构修复（0.5 天） ✅ 已完成

#### 1.1 添加专门的 Dot 表达式类型

**文件**: `crates/auto-lang/src/ast.rs`

在 `Expr` 枚举中添加（约 line 235）：

```rust
pub enum Expr {
    // ... 现有变体 ...

    /// Dot expression: object.field or Type.method
    /// Used for both field access and method calls
    Dot {
        object: Box<Expr>,
        field: Name,
    },
}
```

**理由**:
- 明确的语义表示
- 易于区分字段访问和方法调用
- 便于类型检查和优化

#### 1.2 更新解析器以使用新的 Dot 类型

**文件**: `crates/auto-lang/src/parser.rs`

修改点表达式解析（约 line 940-958）：

```rust
fn parse_dot_expr(&mut self, left: Expr) -> AutoResult<Expr> {
    self.expect(TokenKind::Dot)?;
    let field = self.parse_ident()?;

    Ok(Expr::Dot {
        object: Box::new(left),
        field,
    })
}
```

**验证**: `cargo test -p auto-lang test_parser`

---

### 阶段 2：字段读取实现（1 天） ✅ 已完成

#### 2.1 添加字段读取求值逻辑

**文件**: `crates/auto-lang/src/eval.rs`

在 `eval_expr` 函数中添加 `Expr::Dot` 的处理（约 line 1200）：

```rust
Expr::Dot { object, field } => {
    // 求值对象
    let obj_val = self.eval_expr(object)?;

    match obj_val {
        Value::Instance(inst) => {
            // 实例字段访问
            let field_name = field.to_string();

            // 从实例的 fields 中获取字段值
            if let Some(field_value) = inst.fields.get(&field_name) {
                Ok(field_value.clone())
            } else {
                Ok(Value::error(format!(
                    "Field '{}' not found in instance",
                    field_name
                )))
            }
        }
        Value::Type(type_name) => {
            // 类型方法调用: Type.method(...)
            // 返回类型的元信息，供后续调用处理
            Ok(Value::Meta(MetaID::Type(type_name)))
        }
        _ => Ok(Value::error(format!(
            "Cannot access field on non-instance value: {:?}",
            obj_val
        ))),
    }
}
```

**关键点**:
1. 区分实例字段访问 (`obj.field`) 和类型方法访问 (`Type.method`)
2. 从 `inst.fields` 中读取字段值
3. 返回字段值的克隆（避免所有权问题）

#### 2.2 更新赋值语句以支持 Dot 表达式

**文件**: `crates/auto-lang/src/eval.rs`

修改 `eval_assign` 或相关函数（约 line 1117-1200）：

```rust
// 字段赋值: obj.field = value
Expr::Dot { object, field } => {
    let obj_val = self.eval_expr(object)?;
    let right_val = self.eval_expr(right)?;

    if let Value::Instance(inst) = obj_val {
        let field_name = field.to_string();
        inst.fields.set(field_name.as_str(), right_val);
        Ok(right_val)
    } else {
        Ok(Value::error("Cannot assign field on non-instance"))
    }
}
```

**验证**:
```bash
# 创建测试文件
cat > test_field_read.at << 'EOF'
type Point {
    x int
    y int
}

let p = Point.new()
p.x = 10
p.y = 20
say(p.x)
say(p.y)
EOF

cargo run --release -- run test_field_read.at
# 预期输出: 10 20
```

---

### 阶段 3：区分方法和字段访问（1 天） ✅ 已完成

#### 3.1 修改点表达式求值以区分方法调用

**文件**: `crates/auto-lang/src/eval.rs`

在 `eval_call` 函数中添加对 `Expr::Dot` 的处理（约 line 1900）：

```rust
// 处理 object.method(args) 形式的调用
if let Expr::Dot { object, method } = call.name.as_ref() {
    let obj_val = self.eval_expr(object)?;
    let method_name = method.to_string();

    // 区分类型静态方法和实例方法
    match &obj_val {
        Value::Type(type_name) => {
            // 静态方法: List.new()
            self.eval_type_static_call(type_name, &method_name, &call.args)
        }
        Value::Instance(inst) => {
            // 实例方法: list.push(1)
            self.eval_instance_method_call(inst, &method_name, &call.args)
        }
        _ => {
            Ok(Value::error(format!(
                "Cannot call method on non-object value: {:?}",
                obj_val
            )))
        }
    }
} else if let Expr::Ident(func_name) = call.name.as_ref() {
    // 普通函数调用: say(...)
    // ... 现有逻辑 ...
}
```

#### 3.2 添加类型静态方法调用求值

**文件**: `crates/auto-lang/src/eval.rs`

```rust
fn eval_type_static_call(
    &mut self,
    type_name: &str,
    method_name: &str,
    args: &Args,
) -> AutoResult<Value> {
    // 查找类型的静态方法
    let type_decl = self.universe.borrow().lookup_type(type_name);

    if let Some(method) = type_decl.methods.iter()
        .find(|m| m.name == method_name && m.is_static())
    {
        // 调用静态方法
        self.eval_fn_call_with_sig(&method.sig, args)
    } else {
        Ok(Value::error(format!(
            "Static method {}::{} not found",
            type_name, method_name
        )))
    }
}
```

#### 3.3 添加实例方法调用求值

**文件**: `crates/auto-lang/src/eval.rs`

```rust
fn eval_instance_method_call(
    &mut self,
    inst: &Instance,
    method_name: &str,
    args: &Args,
) -> AutoResult<Value> {
    // 通过 VM registry 查找实例方法
    let registry = crate::vm::VM_REGISTRY.lock().unwrap();

    for (_module_name, module) in registry.modules().iter() {
        if let Some(type_entry) = module.types.get(&inst.ty.to_string()) {
            if let Some(method) = type_entry.methods.get(method_name) {
                // 调用 VM 方法
                let mut arg_vals = Vec::new();
                for arg in args.args.iter() {
                    match arg {
                        Arg::Pos(expr) => arg_vals.push(self.eval_expr(expr)?),
                        _ => {}
                    }
                }

                let result = (method)(self.universe.clone(), inst, arg_vals);
                return Ok(result);
            }
        }
    }

    drop(registry);
    Ok(Value::error(format!("Method {} not found", method_name)))
}
```

**验证**:
```bash
# 测试静态方法
cat > test_static_method.at << 'EOF'
let list = List.new()  // 静态方法调用
list.push(1)           // 实例方法调用
EOF

cargo run --release -- run test_static_method.at
```

---

### 阶段 4：类型字段运行时支持（1 天） ✅ 已完成

#### 4.1 修改类型解析以创建字段映射

**文件**: `crates/auto-lang/src/parser.rs`

在解析类型定义时（约 line 2000-2500），确保类型成员被正确解析：

```rust
fn parse_type_decl(&mut self) -> AutoResult<TypeDecl> {
    // ... 现有逻辑 ...

    // 解析类型成员
    while self.is_kind(TokenKind::Ident) && !self.is_kind(TokenKind::RBrace) {
        if let Some(member) = self.parse_type_member()? {
            type_decl.members.push(member);
        }
    }

    // ... 现有逻辑 ...
}
```

#### 4.2 添加类型实例化逻辑

**文件**: `crates/auto-lang/src/eval.rs` 或 `crates/auto-lang/src/vm/`

当创建类型实例时，自动初始化字段：

```rust
pub fn create_type_instance(
    uni: Shared<Universe>,
    type_name: &str,
    field_values: HashMap<String, Value>,
) -> Value {
    let type_decl = uni.borrow().lookup_type(type_name);

    let mut fields = Obj::new();

    // 初始化所有字段为默认值
    for member in &type_decl.members {
        if let Some(value) = field_values.get(&member.name.to_string()) {
            fields.set(member.name.as_str(), value.clone());
        } else if let Some(default_val) = &member.value {
            // 使用声明的默认值
            let evaluated = self.eval_expr(default_val)?;
            fields.set(member.name.as_str(), evaluated);
        } else {
            // 使用类型的默认值
            fields.set(member.name.as_str(), Type::default_value(&member.ty));
        }
    }

    Value::Instance(Instance {
        ty: auto_val::Type::from(type_decl),
        fields,
    })
}
```

**验证**:
```bash
# 测试类型字段
cat > test_type_fields.at << 'EOF'
type Point {
    x int = 0  // 带默认值
    y int = 0
}

let p = Point.new()  // 应该有默认字段
say(p.x)            // 应该输出 0
p.x = 10
say(p.x)            // 应该输出 10
EOF

cargo run --release -- run test_type_fields.at
```

---

### 阶段 5：测试基础设施（0.5 天）

#### 5.1 创建字段访问测试用例

**文件**: `crates/auto-lang/src/tests/field_access_tests.rs`

```rust
#[test]
fn test_field_read() {
    let code = r#"
        type Point {
            x int
            y int
        }

        fn main() {
            let p = Point.new()
            p.x = 10
            p.y = 20
            p.x  // 返回 10
        }
    "#;

    let result = run(code).unwrap();
    assert!(result.contains("10"));
}

#[test]
fn test_field_assign() {
    let code = r#"
        type Point {
            x int
        }

        fn main() {
            let p = Point.new()
            p.x = 42
            p.x
        }
    "#;

    let result = run(code).unwrap();
    assert!(result.contains("42"));
}

#[test]
fn test_nested_field_access() {
    let code = r#"
        type Inner {
            value int
        }

        type Outer {
            inner Inner
        }

        fn main() {
            let o = Outer.new()
            o.inner.value = 10
            o.inner.value
        }
    "#;

    let result = run(code).unwrap();
    assert!(result.contains("10"));
}

#[test]
fn test_method_vs_field() {
    let code = r#"
        type Counter {
            count int

            fn increment() {
                .count = .count + 1
            }

            fn get_count() int {
                .count
            }
        }

        fn main() {
            let c = Counter.new()
            c.count = 5
            c.increment()
            c.get_count()  // 应该返回 6
        }
    "#;

    let result = run(code).unwrap();
    assert!(result.contains("6"));
}
```

#### 5.2 创建 a2c 转译测试

**文件**: `crates/auto-lang/test/a2c/057_field_access/field_access.at`

```auto
type Point {
    x int
    y int

    fn new(x int, y int) Point {
        let p Point
        p.x = x
        p.y = y
        p
    }

    fn get_x() int {
        .x
    }
}

fn main() {
    let p = Point.new(10, 20)
    let x = p.get_x()
}
```

**预期输出** (`field_access.expected.c`):

```c
#include "field_access.h"

typedef struct {
    int x;
    int y;
} Point;

Point Point_new(int x, int y) {
    Point p;
    p.x = x;
    p.y = y;
    return p;
}

int Point_get_x(Point* self) {
    return self->x;
}

int main(void) {
    Point p = Point_new(10, 20);
    int x = Point_get_x(&p);
    return 0;
}
```

**验证**: `cargo test -p auto-lang test_057_field_access`

---

## 关键实施文件

### 必须修改的文件（按优先级）

1. **`crates/auto-lang/src/ast.rs`**
   - 添加 `Expr::Dot` 类型
   - 更新 Display 实现

2. **`crates/auto-lang/src/parser.rs`**
   - 添加 `parse_dot_expr` 函数
   - 修改点表达式解析逻辑

3. **`crates/auto-lang/src/eval.rs`**
   - 添加 `Expr::Dot` 求值逻辑
   - 区分字段访问和方法调用
   - 实现字段读取

4. **`crates/auto-lang/src/eval.rs`**
   - 确保类型字段在实例化时初始化 
   - [x] 实现 `create_default_instance`
   - [x] 更新 `eval_store`

5. **`crates/auto-lang/src/tests/field_access_tests.rs`** (新建)
   - 字段访问测试用例

6. **`crates/auto-lang/test/a2c/057_field_access/`** (新建)
   - a2c 转译测试

---

## 成功标准

### 阶段 1-2: AST 和字段读取
- ✅ 点表达式使用专门的 `Expr::Dot` 类型
- ✅ 字段读取正常工作: `obj.field`
- ✅ 字段赋值继续工作: `obj.field = value`

### 阶段 3: 方法调用区分
- ✅ 静态方法调用正常: `List.new()`
- ✅ 实例方法调用正常: `list.push(1)`
- ✅ 字段访问不会与方法调用冲突

### 阶段 4: 类型字段支持
- ✅ 类型字段可以在实例化时访问
- ✅ 字段默认值正常工作
- ✅ 嵌套字段访问正常: `obj.inner.field`

### 阶段 5: 测试验证
- ✅ 所有单元测试通过
- ✅ a2c 转译测试通过
- ✅ 实际代码示例可以运行

### 最终验收
- ✅ 用户可以编写带字段的类型
- ✅ 方法中可以访问实例字段: `.field`
- ✅ 点表达式语义清晰明确
- ✅ 向后兼容现有代码

---

## 时间估算

- **阶段 1**: AST 结构修复 - 0.5 天
- **阶段 2**: 字段读取实现 - 1 天
- **阶段 3**: 区分方法调用 - 1 天
- **阶段 4**: 类型字段支持 - 1 天
- **阶段 5**: 测试基础设施 - 0.5 天
- **总计**: 4 天

---

## 风险与缓解

### 技术风险

**风险 1: 破坏现有代码**
- **影响**: 高 - 可能影响 List、File 等现有类型
- **缓解**:
  - 增量实现，每个阶段都运行完整测试
  - 保留旧的 `Expr::Bina(..., Op::Dot, ...)` 兼容层
- **回退**: 回滚到旧的点表达式处理

**风险 2: 性能下降**
- **影响**: 中 - 新的 Dot 类型可能增加解析开销
- **缓解**:
  - 使用 Box<Expr> 避免递归类型大小问题
  - 在热路径上优化字段访问
- **回退**: 内联简单的字段访问

**风险 3: VM 方法调用兼容性**
- **影响**: 中 - 可能破坏现有的 VM 方法注册
- **缓解**:
  - 保持现有 VmMethod 签名不变
  - 只修改调用路径，不修改方法本身
- **回退**: 恢复旧的 eval_call 逻辑

### 运营风险

**风险 4: 延误 Vec 实现**
- **影响**: 低 - Vec 实现依赖字段访问功能
- **缓解**:
  - 优先实现阻塞问题（阶段 1-2）
  - Vec 可以先用 Rust VM 实现，后续迁移
- **回退**: Vec 继续使用当前 List 实现

---

## 后续工作

完成本计划后，可以：

1. **实现纯 AutoLang Vec<T>**
   - 在 `stdlib/auto/vec.at` 中编写纯 AutoLang 实现
   - 使用字段访问: `.data`, `.len`, `.cap`
   - 使用 VM 内存函数: `alloc_array`, `realloc_array`, `free_array`

2. **改进现有 List<T>**
   - 暴露内部字段（如果需要）
   - 优化字段访问性能

3. **支持更多特性**
   - 链式字段访问: `a.b.c.d`
   - 方法链调用: `obj.method1().method2()`
   - 属性访问器: `obj.length` (自动调用 `length()`)

---

## 参考资料

- **现有点表达式处理**: `crates/auto-lang/src/eval.rs:1117-1266`
- **类型成员定义**: `crates/auto-lang/src/ast/types.rs:490-494`
- **VM 方法注册**: `crates/auto-lang/src/vm.rs:15-100`
- **List 实现**: `crates/auto-lang/src/vm/list.rs`
