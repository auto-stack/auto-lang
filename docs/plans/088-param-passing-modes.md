# Plan 088: 函数参数传递模式实现

**目标**: 实现函数参数的 4 种传递模式：`copy`, `view`, `mut`, `take`

**优先级**: **高** - Phase 3 泛型方法支持的前提条件
**依赖**: 无
**工作量**: 1-2 周

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

## 解决方案

### 1. 添加 `ParamMode` 枚举

**文件**: `crates/auto-lang/src/ast/fun.rs`

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ParamMode {
    Copy,  // 值传递，复制（默认，兼容简单类型）
    View,  // 引用传递，不可变（Rust 的 &T）
    Mut,   // 引用传递，可变（Rust 的 &mut T）
    Take,  // Move 语义（转移所有权）
}

impl Default for ParamMode {
    fn default() -> Self {
        Self::View  // Auto 的默认传递形式是 view（引用传递）
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

### 2. 扩展 `Param` 结构体

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
}
```

### 3. Lexer 支持

**文件**: `crates/auto-lang/src/lexer.rs`

Lexer 已经支持 `View`, `Mut`, `Take` token（Plan 026），只需确保它们可以用作参数修饰符。

```rust
// 确认这些 token kind 存在
TokenKind::View   // view 关键字
TokenKind::Mut    // mut 关键字
TokenKind::Take   // take 关键字
// TokenKind::Copy  // copy 关键字（如果不存在则添加）
```

### 4. Parser 解析参数模式

**文件**: `crates/auto-lang/src/parser.rs`

修改 `fn_params()` 函数：

```rust
pub fn fn_params(&mut self) -> AutoResult<Vec<Param>> {
    let mut params = Vec::new();
    while self.is_kind(TokenKind::Ident) {
        // 1. 检查参数传递模式（copy/view/mut/take）
        let mut mode = ParamMode::default();  // 默认 View

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

        // 3. param type
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

        // ✅ 创建参数时包含 mode
        params.push(Param { name, ty, default, mode });
        self.sep_params()?;
    }

    // Handle variadic arguments
    // ...

    Ok(params)
}
```

### 5. Codegen 生成不同的参数传递

**文件**: `crates/auto-lang/src/vm/codegen.rs`

**关键函数**: `compile_fn_decl()` 和方法调用编译

```rust
// 在编译函数声明时
fn compile_fn_decl(&mut self, fn_decl: &Fn) -> AutoResult<()> {
    // 编译参数：根据 ParamMode 生成不同的字节码
    for (i, param) in fn_decl.params.iter().enumerate() {
        match param.mode {
            ParamMode::Copy => {
                // 复制参数值到栈
                self.emit(OpCode::STORE_LOCAL);
                self.emit_u32(i as u32);
            }
            ParamMode::View => {
                // 传递引用（对象ID）
                self.emit(OpCode::STORE_REF);
                self.emit_u32(i as u32);
            }
            ParamMode::Mut => {
                // 传递可变引用（对象ID + 可变标记）
                self.emit(OpCode::STORE_MUT_REF);
                self.emit_u32(i as u32);
            }
            ParamMode::Take => {
                // Move 语义（转移所有权）
                self.emit(OpCode::STORE_TAKE);
                self.emit_u32(i as u32);
            }
        }
    }
    // ...
}
```

### 6. VM 执行不同的参数访问

**文件**: `crates/auto-lang/src/vm/engine.rs`

```rust
// 新增指令
OpCode::STORE_REF => {
    // 从栈弹出引用（对象ID）
    let obj_id = task.ram.pop_i32() as usize;
    task.set_local(local_index as usize, Value::VmRef(VmRef { id: obj_id }));
}

OpCode::STORE_MUT_REF => {
    // 从栈弹出可变引用
    let obj_id = task.ram.pop_i32() as usize;
    task.set_local(local_index as usize, Value::VmMutRef(VmMutRef { id: obj_id }));
}

OpCode::STORE_TAKE => {
    // Move 语义：从栈弹出并转移所有权
    let value = task.ram.pop();
    task.set_local(local_index as usize, value);
}

// 字段访问时检查引用类型
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
            // 常规对象访问
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
        _ => {
            // ❌ 不可变引用或值：编译时应该报错
            return Err("Cannot modify non-mut reference".into());
        }
    }
}
```

---

## 语法示例

### 默认行为（View - 引用传递）

```auto
type Point {
    x int
    fn get_x(self) int {  // 默认 self 是 view 引用
        self.x  // ✅ 可以读取
    }
}

let p = Point{x: 10}
say(p.get_x())  // 输出: 10
```

### 显式 Mut（可变引用）

```auto
type Point {
    x int
    fn set_x(mut self, new_x int) void {  // ✅ 添加 mut 关键字
        self.x = new_x  // ✅ 可以修改
    }
}

let p = Point{x: 10}
p.set_x(100)
say(p.x)  // 输出: 100 ✅
```

### Copy（值传递）

```auto
fn process(copy data int) int {
    data = data * 2  // ✅ 修改的是副本
    return data
}

let x = 5
let y = process(x)
say(x)  // 输出: 5 (原值不变)
say(y)  // 输出: 10 (返回的副本)
```

### Take（Move 语义）

```auto
fn consume(take s str) int {
    let len = s.len()
    // s 在这里被销毁
    return len
}

let my_str = str_new("hello")
let length = consume(my_str)
// say(my_str)  // ❌ 编译错误：my_str 已被 move
```

---

## 实现阶段

### Phase 1: AST 更新（1 天）
- ✅ 添加 `ParamMode` 枚举
- ✅ 扩展 `Param` 结构体
- ✅ 更新 `Display` 实现

### Phase 2: Parser 解析（2-3 天）
- ✅ 修改 `fn_params()` 解析参数模式
- ✅ 支持在类型内部声明方法时使用 mut
- ✅ 添加解析测试

### Phase 3: Codegen 编译（3-4 天）
- ✅ 根据参数模式生成不同字节码
- ✅ 处理 self 参数的特殊情况
- ✅ 方法调用时传递引用

### Phase 4: VM 执行（3-4 天）
- ✅ 实现新指令（STORE_REF, STORE_MUT_REF, STORE_TAKE）
- ✅ 字段访问时检查引用类型
- ✅ 可变引用的字段修改

### Phase 5: 测试（2-3 天）
- ✅ 单元测试（参数解析、编译）
- ✅ 集成测试（方法调用、字段修改）
- ✅ 边界情况（嵌套调用、多参数）

---

## 向后兼容

**默认行为改为 View（引用传递）**：

这是**破坏性变更**，但对于正确的方法语义是必要的。迁移策略：

1. **第一阶段**：默认仍然是 Copy，显式声明 `mut` 才使用引用
2. **第二阶段**：默认改为 View，旧代码添加 `copy` 关键字
3. **第三阶段**：移除对 Copy 默认的支持

**推荐**：直接实现 View 为默认，因为：
- 大多数方法不需要修改 self
- View 性能更好（不复制）
- 符合 Auto 语言设计目标

---

## 验证标准

### 功能完整性
- ✅ 支持 4 种参数传递模式
- ✅ 默认 View（引用传递）
- ✅ 方法可以修改调用者对象（使用 mut）
- ✅ Take 语义防止 use-after-move

### 测试覆盖
- ✅ 20 个单元测试
- ✅ 15 个集成测试
- ✅ 零回归（现有 1250+ 测试全部通过）

### 性能目标
- View 传递：零拷贝（只传递对象ID）
- Mut 传递：零拷贝
- Copy 传递：值复制（与当前行为相同）
- Take 传递：所有权转移（零拷贝）

---

## 相关计划

- **Plan 024**: 所有权系统基础
- **Plan 026**: 属性关键字（.view, .mut, .take）
- **Plan 071**: 闭包变量捕获
- **Plan 087**: 泛型方法支持（依赖本计划）

---

## 时间估算

- **Phase 1**: 1 天
- **Phase 2**: 2-3 天
- **Phase 3**: 3-4 天
- **Phase 4**: 3-4 天
- **Phase 5**: 2-3 天
- **总计**: 11-15 天（约 2 周）
