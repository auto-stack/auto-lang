# Plan 137: Comptime 示例代码库

> **Status**: ✅ Complete
> **Priority**: Medium
> **Dependencies**: Plan 095 (Compile-Time Execution Engine)
> **Related**: CommentTest 机制（待规划）
> **Completed**: 2026-03-20

## 目标

创建一套从简单到高阶的编译期执行示例代码，展示 AutoLang comptime 能力，同时作为测试用例使用。

## 实现状态

### 当前 Comptime 功能支持

| 功能 | 语法 | 状态 | 说明 |
|------|------|------|------|
| `#{ }` 表达式 | `#{ 1 + 2 }` | ✅ 支持 | 编译期表达式求值 |
| `#if` 语句 | `#if cond { }` | ⏸️ 解析支持 | 需要 CTEE 变换 |
| `#for` 语句 | `#for i in 0..n { }` | ⏸️ 解析支持 | 需要 CTEE 变换 |
| `#is` 语句 | `#is val { }` | ⏸️ 解析支持 | 需要 CTEE 变换 |

**注意**: `#if`、`#for`、`#is` 已在 Parser 层面支持，但需要 CTEE (Compile-Time Execution Engine) 在编译时对 AST 进行变换。当前示例使用运行时等价代码作为替代。

## 目录结构

```
test/comptime/
├── 01_basic/                    # Level 1: 基础示例
│   ├── 010_const_eval.at        # 编译期常量表达式求值
│   ├── 020_builtin_constants.at # 编译期算术运算
│   ├── 030_interpolation.at     # 编译期变量引用
│   ├── 040_nested_expr.at       # 编译期嵌套表达式
│   └── 050_boolean_logic.at     # 编译期布尔逻辑
├── 02_intermediate/             # Level 2: 中级示例
│   ├── 010_platform_select.at   # 平台选择 (运行时模拟)
│   ├── 020_loop_unroll.at       # 循环计算 (运行时模拟)
│   ├── 030_pattern_match.at     # 模式匹配
│   ├── 040_hash_if_chain.at     # 条件链
│   └── 050_loop_with_condition.at # 循环与条件组合
└── 03_advanced/                 # Level 3: 高级示例
    ├── 010_factorial.at         # 阶乘计算
    ├── 020_fibonacci.at         # 斐波那契数列
    ├── 030_power_table.at       # 查找表
    ├── 040_bitmask.at           # 位掩码计算
    ├── 050_state_machine.at     # 字符串处理
    └── 060_config_validation.at # 配置验证
```

## 示例列表

### Level 1: 基础示例 (01_basic)

| 文件 | 用例 | 描述 |
|------|------|------|
| 010_const_eval.at | `#{ }` 表达式 | 编译期计算 1+2 |
| 020_builtin_constants.at | 算术运算 | 编译期乘法 2*3 |
| 030_interpolation.at | 变量引用 | 编译期变量求值 |
| 040_nested_expr.at | 嵌套表达式 | 编译期 (1+2)*(3+2) |
| 050_boolean_logic.at | 布尔逻辑 | 编译期布尔运算 |

### Level 2: 中级示例 (02_intermediate)

| 文件 | 用例 | 描述 |
|------|------|------|
| 010_platform_select.at | 条件选择 | 根据条件选择平台 |
| 020_loop_unroll.at | 循环计算 | 循环累加 1+2+3 |
| 030_pattern_match.at | 模式匹配 | 值匹配分支 |
| 040_hash_if_chain.at | 条件链 | 多层条件判断 |
| 050_loop_with_condition.at | 条件循环 | 带条件的循环 |

### Level 3: 高级示例 (03_advanced)

| 文件 | 用例 | 描述 |
|------|------|------|
| 010_factorial.at | 阶乘 | 5! = 120 |
| 020_fibonacci.at | 斐波那契 | F(6) = 8 |
| 030_power_table.at | 查找表 | 预计算平方表 |
| 040_bitmask.at | 位掩码 | 8 位全 1 掩码 |
| 050_state_machine.at | 字符串 | 字符 ASCII 码 |
| 060_config_validation.at | 配置验证 | 条件配置 |

## CommentTest 标记规范

期望标记放在 `fn main()` 函数前：

```auto
// == Comptime Example: 示例名称 ==
//
// 描述：简短说明示例目的

// #[expect_value(期望返回值)]
// #[expect_output("期望输出")]
fn main() {
    // 代码实现
}
```

**断言类型**:
- `// #[expect_value(X)]` - 期望表达式/程序返回值
- `// #[expect_output("...")]` - 期望标准输出
- `// #[expect_error("...")]` - 期望编译错误

## 验证清单

### Phase 1 Complete ✅
- [x] 目录结构创建完成
- [x] 010_const_eval.at 编译通过
- [x] 020_builtin_constants.at 编译通过
- [x] 030_interpolation.at 编译通过
- [x] 040_nested_expr.at 编译通过
- [x] 050_boolean_logic.at 编译通过

### Phase 2 Complete ✅
- [x] 010_platform_select.at 编译通过
- [x] 020_loop_unroll.at 编译通过
- [x] 030_pattern_match.at 编译通过
- [x] 040_hash_if_chain.at 编译通过
- [x] 050_loop_with_condition.at 编译通过

### Phase 3 Complete ✅
- [x] 010_factorial.at 编译通过
- [x] 020_fibonacci.at 编译通过
- [x] 030_power_table.at 编译通过
- [x] 040_bitmask.at 编译通过
- [x] 050_state_machine.at 编译通过
- [x] 060_config_validation.at 编译通过

## 后续工作

1. **CTEE 集成** - 当 CTEE 完成后，将运行时示例转换为编译期版本
2. **CommentTest 机制** - 创建独立计划，实现自动化测试运行器
3. **更多示例** - 根据用户反馈添加更多实用示例
4. **文档整合** - 将示例链接到主文档中

## 参考资料

- [Zig Comptime Documentation](https://ziglang.org/documentation/master/#comptime)
- [Zig Code Samples](https://ziglang.org/zh-CN/learn/samples/)
- [Basic MetaProgramming in Zig](https://www.openmymind.net/Basic-MetaProgramming-in-Zig/)
- [Compile-Time Configuration For Zig Libraries](https://www.openmymind.net/Compile-Time-Configuration-For-Zig-Libraries/)
- [Plan 095: Compile-Time Execution Engine](./095-compile-time-execution-engine.md)
