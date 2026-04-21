# Plan 100: a2js → a2ts 移植计划

## Context

### 背景
当前 AutoLang 有两个独立的 JavaScript 生成器：
1. **a2js transpiler** (`trans/javascript.rs`) - 通用 AST → JavaScript
2. **Vue generator** (`ui_gen/vue.rs`) - AURA AST → JavaScript（Vue SFC）

为了支持：
- TypeScript 生态（更好的类型安全、IDE 支持）
- ArkTS（鸿蒙前端，TypeScript 变种）
- 未来可能的 React/Solid 生成

决定将两个生成器都升级到 TypeScript。

### 关键决策
- **默认生成 TypeScript**，可编译到 JavaScript
- **ArkTS 作为 TypeScript 的变体**，调整语法即可
- **共通逻辑抽取复用**，避免重复代码

---

## 当前状态

### a2js (`trans/javascript.rs`)
- ~700 行代码
- 支持表达式、语句、函数、类、枚举等
- 无类型注解

### Vue generator (`ui_gen/vue.rs`)
- ~2400 行代码
- `expr_to_js()` - 表达式转换
- `stmt_to_js()` - 语句转换
- 无类型注解

---

## Phase 1: 创建 TypeScript 通用模块

### 目标
创建共享的 TypeScript 生成逻辑，供 a2ts 和 Vue generator 使用。

### 文件结构
```
crates/auto-lang/src/trans/
├── typescript.rs       # 主生成器（原 javascript.rs）
├── ts_common.rs        # 共享的 TS 生成逻辑
└── javascript.rs       # 保留（可作为 ts 编译后的简化输出）
```

### ts_common.rs 内容
```rust
/// 共享的 TypeScript 生成工具
pub struct TypeScriptCommon;

impl TypeScriptCommon {
    /// 类型注解生成
    pub fn type_annotation(ty: &Type) -> String {
        match ty {
            Type::Int => "number",
            Type::Str(_) => "string",
            Type::Bool => "boolean",
            Type::Array { elem, len } => "...",
            // ...
        }
    }

    /// 通用表达式转 TS（无类型注解）
    pub fn expr_base(expr: &Expr) -> String { ... }

    /// 通用语句转 TS（无类型注解）
    pub fn stmt_base(stmt: &Stmt) -> String { ... }
}
```

---

## Phase 2: 升级 a2js → a2ts

### 文件修改
- `trans/javascript.rs` → `trans/typescript.rs`

### 需要添加的功能

#### 2.1 类型注解
```typescript
// 函数参数和返回类型
function add(a: number, b: number): number {
    return a + b;
}

// 变量类型
let count: number = 0;
const name: string = "hello";
```

#### 2.2 接口生成
```typescript
// AutoLang type → TypeScript interface
interface Person {
    name: string;
    age: number;
}
```

#### 2.3 类型别名
```typescript
type Point = {
    x: number;
    y: number;
}
```

### 实现步骤
1. 复制 `javascript.rs` 为 `typescript.rs`
2. 修改 `fn_decl()` 添加参数和返回类型注解
3. 修改 `store()` 添加变量类型注解
4. 修改 `type_decl()` 生成 `interface` 而非 `class`
5. 添加类型映射函数 `type_to_ts()`
6. 更新测试用例 `.expected.js` → `.expected.ts`

---

## Phase 3: 升级 Vue Generator

### 文件修改
- `ui_gen/vue.rs`

### 需要修改的方法
1. `expr_to_js()` → `expr_to_ts()`（添加类型）
2. `stmt_to_js()` → `stmt_to_ts()`（添加类型）
3. `generate_script()` 生成 TypeScript 代码

### Vue SFC TypeScript 模式
```vue
<script setup lang="ts">
import { ref, computed } from 'vue'
import type { Ref } from 'vue'

const count: Ref<number> = ref(0)

function handleInc(): void {
  count.value += 1
}
</script>
```

### 实现步骤
1. 添加 `lang="ts"` 到 `<script setup>`
2. 修改 `expr_to_js` 为 `expr_to_ts`，添加类型支持
3. 修改 `stmt_to_js` 为 `stmt_to_ts`，添加类型支持
4. 生成带类型的 Vue 响应式变量（`Ref<T>`）
5. 更新测试用例

---

## Phase 4: 抽取共享逻辑（可选优化）

### 目标
减少 `trans/typescript.rs` 和 `ui_gen/vue.rs` 之间的重复代码。

### 方案 A：trait 抽象
```rust
/// TypeScript 表达式生成 trait
pub trait TsExprGen {
    fn expr_to_ts(&self, expr: &Expr) -> String;
}

// 通用 AST 实现
impl TsExprGen for Expr { ... }

// AURA AST 实现
impl TsExprGen for AuraExpr { ... }
```

### 方案 B：共享工具函数
```rust
// ts_common.rs
pub fn binary_op_to_ts(op: &BinOp) -> &'static str { ... }
pub fn unary_op_to_ts(op: &UnaryOp) -> &'static str { ... }
pub fn type_to_ts(ty: &Type) -> String { ... }
```

---

## 文件清单

### 需要创建的文件
- `crates/auto-lang/src/trans/ts_common.rs` - 共享 TS 生成逻辑

### 需要修改的文件
- `crates/auto-lang/src/trans/javascript.rs` → 重命名为 `typescript.rs`
- `crates/auto-lang/src/trans/mod.rs` - 更新模块导出
- `crates/auto-lang/src/ui_gen/vue.rs` - 升级到 TypeScript
- `crates/auto-lang/src/lib.rs` - 更新 API

### 需要更新的测试
- `crates/auto-lang/test/a2j/` → 重命名为 `test/a2ts/`
- `.expected.js` → `.expected.ts`

---

## 验证方法

1. **单元测试**
   ```bash
   cargo test -p auto-lang trans
   cargo test -p auto-lang ui_gen
   ```

2. **集成测试**
   - 运行 a2ts 生成 TypeScript 代码
   - 用 tsc 编译生成的代码验证语法正确
   - 运行 Vue generator 生成 .vue 文件

3. **手动验证**
   - 生成 TypeScript 代码并检查类型注解
   - 在 Vue 项目中测试生成的组件

---

## 优先级

| 阶段 | 优先级 | 依赖 |
|------|--------|------|
| Phase 1: ts_common.rs | P2 | 无 |
| Phase 2: a2js → a2ts | P1 | 无 |
| Phase 3: Vue generator | P1 | Phase 2 |
| Phase 4: 共享逻辑 | P3 | Phase 2, 3 |

建议先完成 Phase 2，再完成 Phase 3，Phase 4 可作为后续优化。

---

## 时间线（预估）

- Phase 2 (a2ts): 2-3 天
- Phase 3 (Vue): 1-2 天
- Phase 4 (优化): 1-2 天（可选）

---

## 状态

- [ ] Phase 1: 创建 ts_common.rs
- [x] Phase 2: 升级 a2js → a2ts ✅ (2026-03-01)
- [x] Phase 3: 升级 Vue generator ✅ (2026-03-01)
- [ ] Phase 4: 抽取共享逻辑

### Phase 2 完成记录

创建了 `trans/typescript.rs`，实现了：
- AutoLang 类型到 TypeScript 类型的映射 (`type_to_ts()`)
- 函数参数和返回值的类型注解
- 变量声明的类型注解
- `interface` 生成（替代 `class`）
- `const enum` 生成
- 测试用例：000_hello, 003_func, 006_struct, 007_enum

### Phase 3 完成记录

更新了 `ui_gen/vue.rs`，实现了：
- 添加 `use_typescript` 标志（默认为 true）
- `<script setup lang="ts">` 生成
- `ref<T>()` 类型注解（如 `ref<number>(0)`）
- `computed<T>()` 类型注解
- 函数返回类型注解（如 `function handler(): void {`）
- `expr_to_ts_type()` 辅助函数用于推断表达式类型
