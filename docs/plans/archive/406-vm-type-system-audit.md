# Plan 406: VM 类型系统系统性审计

> **状态（2026-08-20 finish-plan 复审）**: ✅ **目标缺陷已全部修复，审计矩阵让位**。本计划的立项动机（038 VM 阻塞）早已消失，其 Phase 2 列出的目标 bug 已由 2026-08-20 审计批次根治：
> - **GET_ELEM List\<bool\>/List\<Value\>::Bool 裸 1/0**（§2 表 bug 1/2）→ audit-A4（`plan-fix/406-getelem-bool` 合并 c1316a2c）：engine.rs 两分支 + native.rs push_value 改 `encode_bool`，e2e 测试 ×4；
> - **JMP_IF_Z/NZ pop_i32 魔数**（§2 表 bug 3）→ audit-B4（`plan-fix/b406-jmpif-bool` 合并 e58fff15）：`nv_truthy` 统一 tag 优先解码（JMP_IF/AND/OR/NOT 五处），真整数 -2147483647 与遗留哨兵不可区分为已注明限制；
> - **EQ 无 is_bool 臂**（§2 表 bug 4）→ 复核结论：bool==bool 走 raw 位比较本已正确，**无需修复**；
> - bug 5/6/7 此前已由 e3e427cc/9b9fec81/958819ab 顺带解决（三批核查时确认）。
> Phase 1 的全量 nanbox 生产者-消费者审计矩阵（docs/audit/vm-type-audit.md）**未产出**——驱动 bug 既已根治，矩阵价值让位，🟢 延期登记债务簿。
> **优先级**: 高 — 阻塞 038 扫雷 VM 版及所有依赖 Obj/bool/float/数组的 VM 示例
> **前置**: Plan 402 §13（038 扫雷 VM 诊断过程中发现）

## 1. 问题背景

在 038 扫雷 VM 版的诊断过程中，剥洋葱式地发现了 **VM nanbox 类型系统的多个不一致 bug**。每个 bug 单独看都是"某个 opcode 的类型处理不对"，但根因是 **VM 的类型处理没有统一规范**——有的 opcode 按 nanbox tag 解码，有的按编译时 ObjectType 解码，有的混用。

### 已发现并修复的 4 个 bug（master `236f558f`）

| # | Bug | 位置 | 表现 |
|---|---|---|---|
| 1 | GET_FIELD 读 ObjectData bool 字段 push `i32(0/1)` 而非 `encode_bool` | engine.rs GET_ELEM→GET_FIELD | `obj.bool_field == false` 比较 i32 vs TAG_BOOL → tag 不同 → EQ 返回 false → 条件永不成立 |
| 2 | CREATE_OBJ Bool 分支 `pop_i32` 读 `encode_bool` payload | engine.rs CREATE_OBJ | `false` 字面量 payload = i32::MIN+1 → `!=0` → true → 所有 bool 字段存成 true |
| 3 | CREATE_OBJ 按 ObjectType 分派字段值弹出 | engine.rs CREATE_OBJ | `infer_object_type` 对 `Dot`/`Ident` 不可靠（常误判 NestedObject）→ `mine: cell.mine` 用 NestedObject 分支 `pop_i32 as usize` → VmRef(垃圾id) |
| 4 | `to_int` 只处理 str receiver，float 误解 | engine.rs CALL_SPEC str.to_int | `(float_expr).to_int()` 用 `decode_string(float_nv)` → 垃圾 |

### 未修复的根因（当前阻塞 038）

| # | Bug | 表现 |
|---|---|---|
| 5 | `math.random()` 不调用 native shim | codegen 把 `math` 模块当占位符 push 0，`math.random()` 对整数 0 调 `random` 方法 → 永远返回垃圾 → 布雷全失败 |
| 6 | LCG `%` 运算或布雷条件仍有问题 | 即使绕开 math.random 用确定性 LCG，布雷仍 `placed=0`，说明 `%` 或 `==` 或 GET_ELEM 链路仍有 bug |
| 7 | VM 进程偶发退出 | 多处修复后 VM 启动正常但运行中偶尔 failed exit code 1（无 panic 日志） |

## 2. 根因分析：为什么会有这么多类型 bug

VM 用 **nanbox（NaN-boxing）** 把所有类型编码进 64 位 u64：

```
Normal f64:  直接存 IEEE 754 bits（零开销）
其他类型:    NANBOX_BASE(0xFFF0_0000_0000_0000) | TAG(4bit) | payload(48bit)
  TAG_F64=0  TAG_I32=1  TAG_STRING=2  TAG_BOOL=3  TAG_NULL=4
  TAG_OBJECT=5  TAG_LIST=6  TAG_F32=7  TAG_I64=8  TAG_U64=9  TAG_BIGINT=A
```

**核心矛盾**：同一个逻辑值，不同 opcode 用不同的编码/解码方式：

| 操作 | bool 的栈编码 | 说明 |
|---|---|---|
| PUSH_BOOL | `encode_bool(b)` = TAG_BOOL | ✅ 正确 |
| CREATE_OBJ 旧 | `pop_i32` 读 → 误解 | ❌ bug 2（已修） |
| GET_FIELD 旧 | `push_i32(0/1)` | ❌ bug 1（已修） |
| GET_ELEM List<Value> Bool | `push_i32(if b {1} else {0})` | ❌ **同 bug 1，未修** |
| EQ 比较 | 按 tag 分派（is_bool arm 缺失，靠 raw u64 == 兜底） | ⚠️ 脆弱 |
| STORE_STATE_FIELD | ？ | 需审计 |
| SET_ELEM | `pop_i32` → `Value::Int` | ❌ bug 3 相关，部分已修 |
| `.to_string()` 对 bool | `pop_i32` → 当数字打印 | ❌ 打印 encode_bool 的 payload |

**每一对 push/pop 如果编码方式不匹配，就是一个潜在 bug。**

## 3. 审计范围

### 3.1 按"值在栈上的生命周期"审计

一个值从**生产者**（push 到栈）到**消费者**（pop 从栈）的完整链路：

```
生产者(opcode push) → [栈] → 消费者(opcode pop)
```

审计每个生产者 push 的编码，和每个消费者期望的解码，确保**配对一致**。

#### 生产者（push 值到栈的 opcode）
- `CONST_I32` / `CONST_F64` / `PUSH_BOOL` / `PUSH_NULL` / `LOAD_STR`
- `LOAD_LOC` / `LOAD_STATE_FIELD` / `LOAD_GLOBAL`
- `CALL_NATIVE`（native shim 的 push_*）
- `GET_FIELD` / `GET_ELEM` / `GET_TUPLE_FIELD`
- 算术 opcodes（ADD/SUB/MUL/DIV → push 结果）
- `EQ` / `NE` / `LT` ... （比较 → push encode_bool）

#### 消费者（pop 值从栈的 opcode）
- `STORE_LOC` / `STORE_STATE_FIELD` / `STORE_GLOBAL`
- `SET_FIELD` / `SET_ELEM`
- `CREATE_OBJ`（pop 字段值）
- `CALL_NATIVE`（native shim 的 pop_*）
- `JMP_IF_FALSE` / `JMP_IF_TRUE`（条件跳转 pop bool）
- 算术 opcodes（pop 操作数）
- `EQ` / `NE` / `LT` ... （pop 比较操作数）

### 3.2 按"类型"审计

对每种类型（bool/int/float/string/object/list），追踪它从生产到消费的完整路径，找出编码不匹配。

**优先级（按 038 暴露的频率）**：
1. **bool** — 已暴露 3 个 bug，可能还有（GET_ELEM List<Value>、JMP_IF_FALSE、STORE_STATE_FIELD）
2. **float/double** — to_int 已暴露，to_string/算术/比较可能也有
3. **object/VmRef** — CREATE_OBJ 已暴露，SET_ELEM/GET_ELEM 部分已暴露
4. **string** — to_string 对非 str 可能有问题
5. **int** — 基数最大，但最稳定（payload 简单）

### 3.3 按"模块函数"审计

`math.random()` 暴露了模块占位符问题（bug 5）。需审计所有标准库模块（math/rand/time/env/fs...）的方法调用是否正确路由到 native shim。

## 4. 审计方法

### 4.1 静态审计（代码审查）
对每个 opcode 的 push/pop，记录：
- push 的编码方式（encode_i32 / encode_bool / encode_f64 / encode_object / push_i32 / push_nv）
- pop 的解码方式（pop_i32 / pop_nv + tag 分派）
- 是否配对一致

输出：`docs/audit/vm-type-audit.md`（生产者-消费者矩阵）。

### 4.2 动态审计（运行时检查）
在 debug 构建里加一个**栈值类型追踪**：
- 每个 push_nv 记录值的 tag
- 每个 pop_nv 验证 tag 符合期望（可选，debug-only）
- 发现 tag 不匹配时 eprintln 警告

### 4.3 回归测试
为每个类型写最小化的 .at 测试用例：
```autolang
// bool_test.at
widget BoolTest {
    model { var b bool = false }
    view { text "ok" }
    on { .Check -> {
        if .b == false { .b = true }  // GET_FIELD bool + EQ + CREATE_OBJ bool
    } }
}
```
覆盖：bool 字面量、字段读写、比较、Obj 字面量、数组元素、方法返回值。

## 5. 统一修复原则

审计后，确立**一条规则**：

> **所有 opcode 的 push/pop 必须按 nanbox tag 编解码，不依赖编译时类型推断。**
>
> - push：按值的实际 Rust 类型用对应的 encode_*（bool→encode_bool，int→encode_i32，...）
> - pop：先 pop_nv，按 tag 分派解码（is_bool→decode_bool，is_i32→decode_i32，...）
> - 编译时 ObjectType 仅用于无法从 tag 区分的窄化（Byte/Uint/Char 都是 i32 payload）

## 6. 任务分解

### Phase 1: 静态审计 + 矩阵 ⚪
- [ ] 遍历 engine.rs 所有 opcode，记录 push/pop 编码方式
- [ ] 生成生产者-消费者矩阵（`docs/audit/vm-type-audit.md`）
- [ ] 标记所有不配对的地方

### Phase 2: bool 类型修复 ⚪（最高优先级）
- [ ] GET_ELEM List<Value> Bool 分支：push encode_bool 而非 push_i32
- [ ] JMP_IF_FALSE / JMP_IF_TRUE：按 tag 解码 bool 而非 pop_i32
- [ ] STORE_STATE_FIELD：bool 字段存储编码审计
- [ ] `.to_string()` 对 bool：打印 "true"/"false" 而非 encode_bool payload
- [ ] EQ/NE 的 is_bool 分支：显式处理（不靠 raw u64 兜底）

### Phase 3: float/double 类型修复 ⚪
- [ ] to_int float 分支（已部分修复，确认 to_int 路由正确）
- [ ] to_string 对 float
- [ ] 算术 opcode 的 float/int 混合运算审计
- [ ] math.random() 路由到 native shim（模块占位符 bug）

### Phase 4: object/VmRef 类型修复 ⚪
- [ ] SET_ELEM 按 tag 解码（已部分修复）
- [ ] GET_ELEM 对 VmRef 元素的一致性
- [ ] CREATE_OBJ 字段值（已修复，确认无回归）

### Phase 5: 回归测试 + 038 验证 ⚪
- [ ] 每个类型的最小化测试用例
- [ ] 038 扫雷 VM 版完整游戏流程验证
- [ ] 015-notes 回归（确保不破坏已有功能）

## 7. 风险

- **改动范围大**：engine.rs 有 7000+ 行，几百个 opcode。逐个审计耗时。
- **回归风险**：修改 push/pop 编码可能破坏已有的工作用例（015-notes、计算器等）。
- **缓解**：每个修复都跑现有测试 + 038 验证；debug 构建加类型追踪。

## 8. 不改的东西

- nanbox 编码方案本身（NANBOX_BASE / TAG 常量）—— 改这个是核弹级重构
- codegen 的 ObjectType 枚举 —— 保留作为 hint，但不作为唯一解码依据
- 已有的工作用例逻辑 —— 只修 push/pop 编码一致性
