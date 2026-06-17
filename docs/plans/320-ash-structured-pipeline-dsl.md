# Plan 320: Ash 结构化管道 DSL（Shell-Level Predicate / Sort / Select）

> **Status**: Draft
> **关系**: 落地 [Plan 309 Task 3.1（结构化数据管道激活）](309-ash-remaining-features-roadmap.md)。
> 参考 [Nushell](https://github.com/nushell/nushell) 的 `where` / `sort-by` / `select` 设计,
> 但语法用 **shell-level DSL**（不经 Auto parser），而非 NuScript 闭包。

---

## 1. 背景与目标

### 1.1 现状

- `ls` 已产出 `Value::Array<Value::Obj>`（文件对象数组）。
- `AtomPipeline` 能在注册命令间传结构化 Atom 数据。
- 外部→外部管道走 OS Pipe（Plan 309 Task 1.1）。
- **缺失**：管道中间的过滤/排序/投影操作（`.size > 10` / `sort .field` / `select .field`）不存在。
- Plan 309 Task 3.1 标记此缺口。

### 1.2 目标

让 Ash 支持 Nushell 式结构化管道:

```auto
ls | .size > 10.mb | sort .modified | select .name .size
```

### 1.3 设计决策：Shell DSL vs Auto 闭包

**管道阶段 `|` 之后分两层**:

| 层 | 语法 | 解析器 | 说明 |
|---|---|---|---|
| **Tier 1: Shell DSL 简写** | `.size > 10.mb` / `sort .modified` / `.name` | pipeline parser | 管道专用 DSL，**不经过 Auto parser**。类似 bash 的 `\| grep`：grep 的参数不是 bash 表达式。 |
| **Tier 2: Auto 闭包**（后续） | `where(it => it.size > 10 && it.type == "file")` | Auto parser | 复杂谓词用 Auto 现有 `param => expr` 闭包语法。 |

**MVP 只做 Tier 1**（Shell DSL）。Tier 2 是后续增强（需 Auto VM 闭包评价）。

**为什么 `.field op value` 不是 Auto 闭包兼容问题**: 它是 shell-level 管道语法（跟 `|` 本身一样），pipeline parser 直接翻译为 `PipelineOp::Filter`，不涉及闭包、不涉及 `it` 关键字、不经过 Auto parser。

---

## 2. Tier 1 语法规范

### 2.1 管道阶段检测规则

`|` 后的第一个 token 决定阶段类型:

| Token 模式 | 阶段类型 | 翻译为 | 例子 |
|---|---|---|---|
| `.field op value` | **谓词**（隐式 filter） | `Filter { field, op, value }` | `.size > 10.mb` |
| `.field`（单独，无 op） | **投影**（隐式 map） | `Map { field }` | `.name` |
| `sort .field [asc\|desc]` | 排序 | `SortBy { field, desc }` | `sort .modified desc` |
| `select .f1 .f2 ...` | 列选择 | `Select { fields }` | `select .name .size` |
| `first N` | 取头 | `Take(N)` | `first 10` |
| `last N` | 取尾 | `SkipBack(N)` | `last 5` |
| `count` | 计数 | `Count` | `count` |
| `command args` | 普通命令 | 正常分发 | `grep foo` |

**检测逻辑**: pipeline parser 在 `|` 后 peek 第一个 token:
- 以 `.` 开头 → 谓词或投影。
- 是已知 DSL 命令名（`sort`/`select`/`first`/`last`/`count`）→ 对应操作。
- 否则 → 普通命令（现有行为）。

### 2.2 谓词语法（`.field op value`）

```
.field op value

field   = 标识符（对象的 key 名），可嵌套: .user.name
op      = >  <  >=  <=  ==  !=  contains  starts-with  ends-with
value   = 数字 | 字符串 | 单位数字 | 布尔
```

**例子**:
```auto
ls | .size > 10.mb                    // size 大于 10MB
ls | .type == "dir"                   // type 等于 "dir"
ls | .name contains "test"            // name 包含 "test"
ls | .modified starts-with "2026-06"  // modified 以 "2026-06" 开头
```

### 2.3 排序语法

```
sort .field [asc|desc]
sort-by .field [asc|desc]       // 别名（Nushell 兼容）
```

### 2.4 投影语法

```auto
ls | .name                         // 投影为 name 字段列表
ls | select .name .size .type     // 选择多列（保留对象结构）
```

- `.name`（单独）→ map：结果变成 `Value::Array<Value::Str>`（纯值列表）。
- `select .name .size` → 保留对象结构，只留指定字段。

### 2.5 单位（数值后缀）

```auto
10.mb    →  10 * 1024 * 1024  = 10,485,760
10.kb    →  10 * 1024         = 10,240
1.gb     →  1 * 1024 * 1024 * 1024 = 1,073,741,824
```

预处理：正则 `(\d+(?:\.\d+)?)\.(kb|mb|gb|tb)` → 展开为乘法。

---

## 3. 数据模型

### 3.1 PipelineOp 枚举

```rust
/// Shell-DSL 管道操作（非命令的阶段）。
pub enum PipelineOp {
    /// .field op value → 过滤
    Filter { field: String, op: CmpOp, value: Value },
    /// sort .field [desc]
    SortBy { field: String, descending: bool },
    /// select .f1 .f2 ... → 保留对象结构，只留指定字段
    Select { fields: Vec<String> },
    /// .field → 投影为纯值列表
    Map { field: String },
    /// first N
    Take(usize),
    /// last N
    SkipBack(usize),
    /// count
    Count,
}

pub enum CmpOp {
    Gt, Lt, Ge, Le, Eq, Neq,
    Contains, StartsWith, EndsWith,
}
```

### 3.2 值比较

`Filter` 需要比较管道元素的 `obj.field` 值与谓词的 `value`:

| 字段值类型 | 谓词值类型 | 比较方式 |
|---|---|---|
| Int / Float | Int / Float | 数值比较（Int↔Float 自动提升） |
| Str | Str | 字符串比较（`==`/`!=`/`contains`/`starts-with`/`ends-with`） |
| Bool | Bool | 布尔相等 |
| 类型不匹配 | | 转换失败 → 元素被过滤掉（不报错，保守处理） |

### 3.3 嵌套字段访问

`.user.name` → 递归取 `obj.user.name`（嵌套对象）。

MVP 支持**一级**字段（`.size`、`.name`）；嵌套（`.user.name`）作为后续增强。

---

## 4. 实现位置

| 模块 | 文件 | 职责 |
|---|---|---|
| **PipelineOp 定义 + apply** | `ash-core/src/pipeline/operators.rs`（新） | 枚举 + `apply(op, &Value) -> Value` |
| **管道阶段解析** | `ash-core/src/parser/pipe_stages.rs`（新） | 从管道文本解析 PipelineOp |
| **单位展开** | `ash-core/src/parser/pipe_stages.rs` | `10.mb` → 数字 |
| **执行集成** | `auto-shell/src/shell.rs::execute_pipeline_with_auto` | PipelineOp 阶段 → operators::apply |
| **测试** | `ash-core` lib tests + `auto-shell/tests/pipeline_dsl.rs`（新） | 单元 + 集成 |

---

## 5. 实现阶段

### Phase 1: PipelineOp 核心 + 谓词 Filter + 单测

1. `operators.rs`: `PipelineOp`/`CmpOp` 枚举 + `apply()` 实现 `Filter`（遍历 Array，取 field，比较，保留）。
2. 值比较逻辑（数值/字符串/布尔/contains/starts-with/ends-with）。
3. `pipe_stages.rs`: 解析 `.field op value` → `PipelineOp::Filter`。
4. 单位展开（`10.mb` → 数字）。
5. 单测: 各种 op、类型、单位。

### Phase 2: SortBy + Select + Map + Take/SkipBack + Count

1. `operators.rs`: `SortBy`/`Select`/`Map`/`Take`/`SkipBack`/`Count` 的 `apply()`。
2. `pipe_stages.rs`: 解析 `sort .field`/`select .f1 .f2`/`.field`/`first N`/`last N`/`count`。
3. 单测。

### Phase 3: 执行集成

1. `shell.rs::execute_pipeline_with_auto`: 管道阶段分发 — PipelineOp 阶段调用 `operators::apply`，命令阶段走现有路径。
2. 前一段产出 `AtomPipeline::Atom(Value::Array)` → PipelineOp 操作。
3. 非结构化输入（`Text`/`ExternalStream`）→ 尝试解析为 Value，或报「需要结构化输入」。
4. 集成测试: `ls | .size > 0 | sort .name | .name`（端到端）。

### Phase 4（后续）: Tier 2 Auto 闭包

1. `where(it => ...)` / `map(it => ...)` — 用 Auto parser + VM 评价闭包。
2. 复杂谓词（多条件、函数调用）。
3. `group-by .field` / `uniq` / `reverse` / `sum .field` / `avg .field`。

---

## 6. 执行集成详细设计

在 `execute_pipeline_with_auto` 的循环中，每个 `cmd` 字符串判断阶段类型:

```rust
for (i, cmd) in commands.iter().enumerate() {
    // 1. 尝试解析为 PipelineOp（Shell DSL）
    if let Some(op) = parse_pipe_stage(cmd) {
        // 取前一段输出（必须是 AtomPipeline::Atom）
        let input = input_pipeline.take().unwrap_or_empty();
        let value = input.into_atom_value(); // Value::Array
        let output = operators::apply(&op, &value);
        input_pipeline = Some(AtomPipeline::Atom(Atom::new(output, ...)));
        continue;
    }
    // 2. 普通命令（现有逻辑）
    // ... registered command / builtin / external ...
}
```

`parse_pipe_stage(cmd)` 返回 `Option<PipelineOp>`:
- `Some(op)` → 是 DSL 阶段。
- `None` → 是普通命令，走现有路径。

---

## 7. 边界与风险

| 风险 | 应对 |
|---|---|
| 非结构化输入（Text/Stream）到 PipelineOp | 尝试 parse 为 Value；失败报「需要结构化输入」 |
| 字段不存在 | `obj.get("field")` 返回 None → 该元素被过滤掉（filter）或值为空（map） |
| 类型不匹配（Str vs Int 比较） | 转换失败 → 保守处理（元素被过滤） |
| 性能（大列表逐元素评价） | 轻量评价器（不走 VM），O(n) per op |
| `.field` 与命令名冲突 | DSL 命令名（sort/select/first/...）是保留的；`.field` 以 `.` 开头不会冲突 |

---

## 8. 验证

- **单测**（ash-core）: `operators::apply` 对各种 PipelineOp + Value 组合。
- **解析测试**: `pipe_stages::parse_pipe_stage` 正确识别各阶段类型。
- **集成测试**（auto-shell）: `ls | .type == "dir" | sort .name | .name` 端到端。
- **手动**: `ash -c 'ls | .size > 1000 | sort .name'`。

---

## 9. 完整示例流

```
输入: ls | .size > 10.mb | sort .modified desc | select .name .size

解析:
  stage 0: command "ls"
  stage 1: PipeOp Filter { field:"size", op:Gt, value:10485760 }
  stage 2: PipeOp SortBy { field:"modified", descending:true }
  stage 3: PipeOp Select { fields:["name","size"] }

执行:
  ls → Value::Array([
    {name:"app", type:"dir", size:0, modified:"2026-03-02"},
    {name:"big.tar", type:"file", size:20000000, modified:"2026-06-15"},
    {name:"small.txt", type:"file", size:500, modified:"2026-06-10"},
  ])
  → Filter size > 10485760 → [
    {name:"big.tar", size:20000000, modified:"2026-06-15"},
  ]
  → SortBy modified desc → (sorted)
  → Select name, size → [
    {name:"big.tar", size:20000000},
  ]
  → 渲染:
    Name      Size
    big.tar   20000000
```
