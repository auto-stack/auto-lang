# AutoLang 类型推导子系统设计

## 概述

为 AutoLang 设计和实现一套完善的类型推导和类型检查子系统，具备以下特性：
- **混合推导策略**：基础表达式使用局部逐步推导，函数使用简化版 Hindley-Milner
- **静态类型检查**：在编译期捕获类型错误，同时保持运行时类型灵活性
- **类型错误恢复**：推导失败时优雅降级到 `Type::Unknown`
- **友好的错误提示**：使用现有 miette 基础设施提供清晰的诊断信息
- **模块化架构**：与解析器、评估器和转译器清晰分离

## 当前状态分析

### 现有实现

**位置**：`parser.rs:2177` - `infer_type_expr()` 函数

**当前能力**：
```rust
fn infer_type_expr(&mut self, expr: &Expr) -> Type {
    // 字面量：Int, Float, Bool, Str, CStr
    // 二元操作：基本类型传播（取左边类型）
    // 标识符：简单的符号表查找
    // 数组：从第一个元素推导
    // 调用：使用 call.ret（预先计算）
    // 索引：从数组类型提取元素类型
}
```

**主要局限**：
- ❌ 无统一算法（无法求解类型方程）
- ❌ 无函数签名推导
- ❌ 无控制流分析（if/else、循环）
- ❌ 无类型检查（仅推导）
- ❌ 无错误恢复机制
- ❌ 无约束跟踪
- ❌ 仅单态（不支持泛型）

### 类型系统基础

**Type 定义** (`ast/types.rs:8-28`)：
```rust
pub enum Type {
    Byte, Int, Uint, USize, Float, Double, Bool, Char,
    Str(usize), CStr,
    Array(ArrayType), Ptr(PtrType),
    User(TypeDecl), Union(Union),
    Tag(Shared<Tag>), Enum(Shared<EnumDecl>),
    Void, Unknown, CStruct(TypeDecl),
}
```

**变量存储** (`ast/store.rs:15-20`)：
```rust
pub struct Store {
    pub kind: StoreKind,  // Let, Mut, Var, Field, CVar
    pub name: Name,
    pub ty: Type,         // 未指定时为 Type::Unknown
    pub expr: Expr,
}
```

## 设计决策

### 1. 混合推导策略

**选择理由**：完整的 Hindley-Milner 对 AutoLang 的使用场景（自动化脚本、嵌入式系统）来说过于复杂。

**实现方案**：
- **局部表达式**：自底向上的逐步推导（简单、快速）
- **函数**：带约束的简化 HM（支持多态但不支持高阶类型）
- **控制流**：分支类型统一（if/else、循环返回值）

**权衡**：
- ✅ 实现更简单（约 1500 LOC vs 完整 HM 的 5000+ LOC）
- ✅ 错误信息更友好（局部推理）
- ✅ 编译更快（无需全局不动点迭代）
- ❌ 表达能力较弱（不支持高阶类型）
- ❌ 无法推导复杂的相互递归函数

### 2. 泛型支持

**决策**：第一阶段不支持泛型，留待后续阶段。

**原因**：
- 降低初始实现复杂度
- 先建立稳固的类型推导基础
- 泛型需要额外的类型参数化和约束求解机制

### 3. 类型检查与错误处理

**策略**：分三个阶段

**阶段 1：类型推导**（编译期）
- 计算所有表达式的类型
- 生成类型约束
- 产生 `Type` 注解或 `Type::Unknown`

**阶段 2：类型检查**（编译期）
- 验证显式类型注解与推导类型匹配
- 检查运算符有效性
- 验证函数调用签名
- 报告类型错误及诊断信息

**阶段 3：运行时类型**（执行期）
- 保留 `var` 声明的动态类型
- 在安全时允许类型强制转换
- 保持向后兼容性

### 4. 错误恢复策略

**策略**：失败开放，而非封闭。

**策略层级**：
```
1. 尝试从表达式推导类型
2. 尝试从使用上下文推导
3. 尝试与相似类型统一（int/uint、float/double）
4. 降级到 Type::Unknown
```

## 架构设计

### 模块结构

```
crates/auto-lang/src/
├── infer/
│   ├── mod.rs              # 公共 API，模块重导出
│   ├── context.rs          # InferenceContext（类型环境、约束）
│   ├── unification.rs      # 类型统一算法
│   ├── constraints.rs      # TypeConstraint 表示
│   ├── expr.rs             # 表达式类型推导
│   ├── stmt.rs             # 语句类型检查
│   ├── functions.rs        # 函数签名推导
│   └── errors.rs           # 类型相关错误辅助
```

### 核心数据结构

#### InferenceContext

```rust
pub struct InferenceContext {
    /// 类型环境：变量 -> Type
    pub type_env: HashMap<Name, Type>,

    /// 推导期间收集的约束
    pub constraints: Vec<TypeConstraint>,

    /// 用于变量遮蔽的作用域链
    pub scopes: Vec<HashMap<Name, Type>>,

    /// 当前函数返回类型（用于检查返回语句）
    pub current_ret: Option<Type>,

    /// Universe 引用（用于符号查找）
    pub universe: Shared<Universe>,

    /// 错误累加器
    pub errors: Vec<TypeError>,

    /// 警告累加器
    pub warnings: Vec<Warning>,
}
```

#### TypeConstraint

```rust
pub enum TypeConstraint {
    /// 两个类型必须相等
    Equal(Type, Type, SourceSpan),

    /// 类型必须可调用
    Callable(Type, SourceSpan),

    /// 类型必须可索引（数组/字符串）
    Indexable(Type, SourceSpan),

    /// 类型必须是另一个类型的子类型
    Subtype(Type, Type, SourceSpan),
}
```

## 算法设计

### 1. 表达式类型推导

**算法**：自底向上遍历，生成约束。

```rust
fn infer_expr(ctx: &mut InferenceContext, expr: &Expr) -> Type {
    match expr {
        // 字面量：已知类型
        Expr::Int(_) => Type::Int,
        Expr::Float(_, _) => Type::Float,
        Expr::Bool(_) => Type::Bool,

        // 标识符：环境查找
        Expr::Ident(name) => {
            ctx.lookup_type(name)
                .unwrap_or_else(|| Type::Unknown)
        }

        // 二元运算符
        Expr::Bina(lhs, op, rhs) => {
            let lhs_ty = infer_expr(ctx, lhs);
            let rhs_ty = infer_expr(ctx, rhs);

            // 添加相等性约束
            ctx.add_constraint(TypeConstraint::Equal(
                lhs_ty.clone(),
                rhs_ty.clone(),
                expr.span(),
            ));

            // 推导结果类型
            infer_binop_type(ctx, op, lhs_ty, rhs_ty)
        }

        // 数组
        Expr::Array(elems) => {
            if elems.is_empty() {
                Type::Unknown  // 无法推导空数组类型
            } else {
                let elem_ty = infer_expr(ctx, &elems[0]);
                // 检查所有元素类型相同
                for elem in &elems[1..] {
                    let ty = infer_expr(ctx, elem);
                    ctx.add_constraint(TypeConstraint::Equal(
                        elem_ty.clone(),
                        ty,
                        elem.span(),
                    ));
                }
                Type::Array(ArrayType {
                    elem: Box::new(elem_ty),
                    len: elems.len(),
                })
            }
        }

        // If 表达式
        Expr::If(if_expr) => {
            let cond_ty = infer_expr(ctx, &if_expr.cond);
            ctx.add_constraint(TypeConstraint::Equal(
                Type::Bool,
                cond_ty,
                if_expr.cond.span(),
            ));

            let then_ty = infer_expr(ctx, &if_expr.then_branch);
            let else_ty = if let Some(else_branch) = &if_expr.else_branch {
                infer_expr(ctx, else_branch)
            } else {
                Type::Void
            };

            // 统一分支类型
            ctx.unify(then_ty.clone(), else_ty.clone())
                .unwrap_or(Type::Unknown)
        }
    }
}
```

### 2. 类型统一

**算法**：Robinson 统一算法，带 occurs check。

```rust
fn unify(ctx: &mut InferenceContext, ty1: Type, ty2: Type) -> Result<Type, TypeError> {
    match (ty1, ty2) {
        // Unknown 类型是通配符
        (Type::Unknown, ty) | (ty, Type::Unknown) => Ok(ty),

        // 基础类型
        (Type::Int, Type::Int) => Ok(Type::Int),
        (Type::Float, Type::Float) => Ok(Type::Float),

        // 数组：统一元素类型和长度
        (Type::Array(arr1), Type::Array(arr2)) => {
            let elem_ty = unify(ctx, *arr1.elem, *arr2.elem)?;
            if arr1.len != arr2.len {
                return Err(TypeError::Mismatch {
                    expected: format!("[{}; {}]", elem_ty, arr1.len),
                    found: format!("[{}; {}]", elem_ty, arr2.len),
                    span: SourceSpan::new(0, 0),
                });
            }
            Ok(Type::Array(ArrayType {
                elem: Box::new(elem_ty),
                len: arr1.len,
            }))
        }

        // 强制转换：int <-> uint, float <-> double
        (Type::Int, Type::Uint) | (Type::Uint, Type::Int) => {
            ctx.warnings.push(Warning::ImplicitTypeConversion {
                from: "int".into(),
                to: "uint".into(),
                span: SourceSpan::new(0, 0),
            });
            Ok(Type::Uint)
        }

        // 类型不匹配
        (ty1, ty2) => Err(TypeError::Mismatch {
            expected: ty1.to_string(),
            found: ty2.to_string(),
            span: SourceSpan::new(0, 0),
        }),
    }
}
```

### 3. 函数签名推导

**算法**：带约束的简化 HM。

```rust
fn infer_function(ctx: &mut InferenceContext, fn_decl: &Fn) -> Result<Type, TypeError> {
    // 为函数创建新作用域
    ctx.push_scope();

    // 1. 推导参数类型（如果未指定）
    let param_tys: Vec<Type> = fn_decl.params.iter()
        .map(|param| {
            if !matches!(param.ty, Type::Unknown) {
                Ok(param.ty.clone())
            } else {
                // 尝试从默认值推导
                if let Some(default) = &param.default {
                    Ok(infer_expr(ctx, default))
                } else {
                    Err(TypeError::InvalidParameter {
                        param: param.name.clone(),
                        span: param.span,
                    })
                }
            }
        })
        .collect::<Result<Vec<_>, _>>()?;

    // 2. 将参数添加到环境
    for (param, ty) in fn_decl.params.iter().zip(param_tys.iter()) {
        ctx.type_env.insert(param.name.clone(), ty.clone());
    }

    // 3. 推导 body 类型
    let body_ty = infer_body(ctx, &fn_decl.body)?;

    // 4. 检查返回类型
    let ret_ty = if !matches!(fn_decl.ret, Type::Unknown) {
        // 显式返回类型：检查 body 是否匹配
        ctx.unify(fn_decl.ret.clone(), body_ty)?;
        fn_decl.ret.clone()
    } else {
        // 从 body 推导返回类型
        body_ty
    };

    // 5. 弹出作用域
    ctx.pop_scope();

    // 6. 返回函数类型
    Ok(Type::Fn(Box::new(FunctionType {
        params: param_tys,
        ret: Box::new(ret_ty),
    })))
}
```

## 与现有代码的集成

### 1. Parser 集成

**位置**：`parser.rs`

**改动**：

```rust
// 用新的推导引擎替换当前的 infer_type_expr
impl Parser {
    pub fn infer_type(&mut self, expr: &Expr) -> Type {
        // 使用新的推导模块
        infer::infer_expr(&mut self.infer_ctx, expr)
    }

    // 在 parse_store() 中，解析表达式后：
    fn parse_store(&mut self, kind: StoreKind) -> AutoResult<Stmt> {
        // ... 现有解析代码 ...

        // 旧代码：ty = self.infer_type_expr(&expr);
        // 新代码：
        if matches!(ty, Type::Unknown) {
            ty = self.infer_type(&expr);
        }

        // 类型检查
        self.type_checker.check_store(&store)?;

        Ok(Stmt::Store(store))
    }
}
```

### 2. Universe 集成

**位置**：`universe.rs`

**改动**：

```rust
impl Universe {
    pub infer_ctx: RefCell<InferenceContext>,

    pub fn new() -> Self {
        Universe {
            // ... 现有字段 ...
            infer_ctx: RefCell::new(InferenceContext::new()),
        }
    }

    // 变量声明现在跟踪推导的类型
    pub fn define_var(&mut self, name: Name, expr: Expr) {
        let ty = self.infer_ctx.borrow_mut().infer_expr(&expr);

        self.define_var_with_type(name, expr, ty);
    }
}
```

## 测试策略

### 单元测试

**文件**：`crates/auto-lang/src/infer/tests.rs`

覆盖范围：
- 字面量类型推导
- 二元运算类型推导
- 数组类型推导
- 类型统一算法
- 函数签名推导
- 错误恢复机制

### 集成测试

**文件**：`test/type-inference/`

测试结构：
```
test/type-inference/
├── 001_literals/       # 字面量类型
├── 002_arrays/         # 数组类型
├── 003_functions/      # 函数类型
├── 004_control_flow/   # 控制流类型
└── 005_errors/         # 类型错误
```

### 回归测试

确保：
- 所有现有测试仍然通过
- 类型推导不破坏现有功能
- 性能影响在可接受范围内

## 性能考虑

### 优化策略

1. **记忆化**：缓存表达式推导结果
2. **惰性统一**：延迟约束求解直到需要时
3. **增量推导**：仅在编辑影响的部分重新推导（IDE 集成）
4. **类型环境共享**：使用 `Rc<Type>` 避免克隆

### 性能目标

- **推导时间**：< 10ms per 1000 行代码
- **内存开销**：< AST 大小的 2 倍
- **编译时间影响**：< 5% 增加

## 分阶段实现计划

### 阶段 1：核心基础设施（第 1 周）

**任务**：
1. 创建 `infer/` 模块结构
2. 实现 `InferenceContext` (context.rs)
3. 实现 `TypeConstraint` (constraints.rs)
4. 添加核心类型的单元测试

**交付物**：
- `crates/auto-lang/src/infer/mod.rs`
- `crates/auto-lang/src/infer/context.rs`
- `crates/auto-lang/src/infer/constraints.rs`

**成功标准**：
- 模块编译无错误
- 单元测试通过
- Context 可以跟踪类型环境

### 阶段 2：表达式推导（第 2 周）

**任务**：
1. 实现所有表达式类型的 `infer_expr()` (expr.rs)
2. 实现基础类型统一 (unification.rs)
3. 添加字面量、二元运算、数组、标识符推导
4. 表达式集成测试

**交付物**：
- `crates/auto-lang/src/infer/expr.rs`
- `crates/auto-lang/src/infer/unification.rs`
- 10+ 表达式测试用例

**成功标准**：
- 所有表达式类型正确推导
- 基础统一算法工作正常
- 测试套件通过

### 阶段 3：类型检查（第 3 周）

**任务**：
1. 实现 `check_stmt()` (stmt.rs)
2. 添加变量声明类型检查
3. 添加赋值类型检查
4. 添加控制流类型检查
5. 类型错误集成测试

**交付物**：
- `crates/auto-lang/src/infer/stmt.rs`
- 类型错误测试套件（20+ 用例）

**成功标准**：
- 类型错误被检测和报告
- 错误信息清晰可操作
- 所有类型检查测试通过

### 阶段 4：函数推导（第 4 周）

**任务**：
1. 实现函数的简化 HM (functions.rs)
2. 添加函数签名推导
3. 添加返回类型检查
4. 处理递归函数
5. 函数集成测试

**交付物**：
- `crates/auto-lang/src/infer/functions.rs`
- 函数推导测试套件（15+ 用例）

**成功标准**：
- 函数签名正确推导
- 返回类型检查通过
- 递归处理无无限循环

### 阶段 5：Parser 集成（第 5 周）

**任务**：
1. 替换 parser 中的 `infer_type_expr()`
2. 向 parser 管道添加类型检查
3. 更新 `Universe` 跟踪推导的类型
4. 添加错误报告集成
5. 端到端测试

**交付物**：
- 更新的 `parser.rs`
- 更新的 `universe.rs`
- 集成测试套件

**成功标准**：
- Parser 使用新推导引擎
- 解析期间报告类型错误
- 所有现有测试仍然通过

### 阶段 6：错误恢复与建议（第 6 周）

**任务**：
1. 实现类型错误恢复
2. 添加类型建议启发式算法
3. 与现有错误基础设施集成
4. 添加建议测试

**交付物**：
- `crates/auto-lang/src/infer/errors.rs`
- 错误恢复测试套件
- 文档

**成功标准**：
- 推导失败不停止编译
- 提供有用的建议
- 错误恢复测试通过

### 阶段 7：文档与示例（第 7 周）

**任务**：
1. 编写模块文档
2. 添加类型系统指南
3. 创建示例程序
4. 添加性能基准测试

**交付物**：
- `docs/type-system.md`
- `docs/type-inference-guide.md`
- 示例程序
- 基准测试结果

**成功标准**：
- 所有模块已文档化
- 用户指南完整
- 示例可编译和运行

## 未来增强（超出第一阶段范围）

### 阶段 8：泛型（未来）

- 泛型类型参数
- 类型构造器
- 泛型函数推导
- 单态化

### 阶段 9：Traits/接口（未来）

- Trait 定义
- Trait 约束
- Trait 实现推导
- 通过 traits 的动态分发

### 阶段 10：IDE 集成（未来）

- LSP 服务器集成
- 类型悬停信息
- 类型的转到定义
- 类型感知的自动补全

## 成功指标

### 定量指标

- **测试覆盖率**：推导模块 > 90%
- **性能**：< 10ms per 1000 LOC 推导时间
- **编译时间影响**：< 5% 增加
- **错误检测**：测试套件中检测 95%+ 的类型错误

### 定性指标

- **错误信息**：所有类型错误提供清晰、可操作的建议
- **代码质量**：清晰、文档完善、模块化设计
- **可维护性**：易于扩展以支持新语言特性
- **用户体验**：对现有 AutoLang 用户的阻力最小

## 关键文件清单

基于此设计，实现时最关键的 5 个文件是：

1. **`d:\autostack\auto-lang\crates\auto-lang\src\infer\unification.rs`**
   - 核心类型统一算法
   - 推导系统的心脏
   - 必须正确处理所有类型组合

2. **`d:\autostack\auto-lang\crates\auto-lang\src\infer\expr.rs`**
   - 表达式类型推导逻辑
   - 处理 20+ 种表达式类型
   - 生成约束和类型

3. **`d:\autostack\auto-lang\crates\auto-lang\src\infer\context.rs`**
   - 类型和约束跟踪的环境
   - 管理作用域和类型绑定
   - 与 Universe 接口

4. **`d:\autostack\auto-lang\crates\auto-lang\src\parser.rs`**（修改）
   - 替换现有的 `infer_type_expr()`
   - 集成新推导引擎
   - 解析后调用类型检查器

5. **`d:\autostack\auto-lang\crates\auto-lang\src\error.rs`**（扩展）
   - 添加新的类型错误变体
   - 扩展错误代码到 E0106-E0150
   - 改进错误建议
