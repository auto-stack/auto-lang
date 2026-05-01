# Plan 229: IS_VARIANT/GET_GENERIC_FIELD 原始值 Option 兼容

## 状态: 已完成 (commit: WIP)

## 问题

`vmtest-08-sse-parser.at` 中 `is pos { Some(p) -> ..., None -> {} }` 对 `str.find()` 返回的 i32 值做模式匹配时失败：
- `IS_VARIANT` 只处理 heap object（instance_id >= 4000000），对原始值返回 false
- `GET_GENERIC_FIELD` 对原始值报 "Invalid instance ID"

## 根因

`str.find()` 返回 i32（-1 = 未找到，>=0 = 找到的位置），但 codegen 的 `is` 表达式编译器总是 emit `IS_VARIANT` opcode，假设目标是枚举实例。

## 修复方案（已实施）

### 引擎层面的原始值兼容

**不修改 codegen**（保持 `IS_VARIANT` + `GET_GENERIC_FIELD` 路径不变），改为在 engine.rs 中添加兼容处理：

#### IS_VARIANT (engine.rs ~2374)

当 `instance_id < 1000000`（不是 heap object）时：
- `Option.Some` → `val >= 0`（匹配）
- `Option.None` → `val < 0`（匹配）
- 其他变体 → false

#### GET_GENERIC_FIELD (engine.rs ~2425)

当 `instance_id < 1000000` 时：
- Pop instance_id，push 原始值本身（作为 "field 0"）
- 这样 `Some(p)` 绑定的 `p` 就是原始 i32 值

## 优点

1. 不影响 codegen 逻辑，减少引入 bug 的风险
2. 对已有的枚举实例路径完全透明
3. 自动处理所有原生方法返回 i32 但被 `is` 匹配为 Option 的场景

## 验证

- 22/24 VM 测试通过（vmtest-08 仍失败，但原因已确认为独立 bug）
- 3144 passed, 3 failed 单元测试（与修改前一致）
- vmtest-22-is-primitive.at 通过

## 未解决

vmtest-08 的实际崩溃原因不是 `is` 匹配问题，而是：
- `mut fn push()` 内部调用 `mut fn drain_frames()` 时
- `drain_frames()` 中的 `for frame != None` 循环导致栈帧损坏
- 在函数返回后触发 "Memory Access Out of Bounds" panic
- 这是一个独立的 bug，需要 Plan 231 处理
