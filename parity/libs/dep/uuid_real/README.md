# uuid_real — 非白名单 pack 面勘测（PLAN-591 T9）

## Scope

`uuid 1.24.0`（Cargo.lock 既有版本，与主仓一致）经 `dep uuid(version: "1.24.0") +
use.rs` 动态加载。**uuid 不在 BUILTIN_OPAQUE_CRATES 白名单**——类型构造器/方法面
全部走真编译 methods pack（594 五库的类型面走 VM native_catalog，本库是首个
非白名单 pack 面样本，正是 594 命中率报告方法论注记要补的勘测对象）。

## 绿面（TAP，phase p11，AUTO_LANG_PARITY_NET=1 门控）

三轨 3/3 全绿（2026-09-09，PLAN-591 T9 实勘；`AUTO_LANG_PARITY_NET=1
phase p11`）：

| case | 面 | 591 解锁归属 |
|---|---|---|
| parse_nil_is_nil | `Uuid.parse_str` Option 返回（T2 nullable，'p?' 槽）+ `is_nil` | 591 T2 |
| parse_v4_version_num | `parse_str` + `get_version_num` 数值面（let 绑定比较，DIV-DEP-16 修复后） | 430 pack 基线 |
| parse_display_print | `print(v4)` Display 面（VM shim 合成 to_string 路由，591 D2） | 591 D2 |

## 红面/边界（不进 TAP）

- `parse_invalid_is_none`（非法输入 None 断言）：a2r 腿 DIV-DEP-15
  （parse* 启发式投影 Option→Result + 变量名 nil→None 劫持）；VM/oracle
  双轨可用，TAP 缺 a2r 腿故不入。
- `Uuid::new_v4`：随机值，三轨无法逐字节对齐（oracle 只能断形状）——不入语料。
- `as_u128`/`to_u128_pair`：u128 超 i64 槽（classify 显式跳过）。
- `Uuid::nil()` 常量面：DIV-DEP-13 家族（常量接收者），不重测。
- 深模块路径类型（fmt 适配器 Braced/Hyphenated 族）：DIV-DEP-17，
  rustdoc 短名限制 → 生成器误径 → 类型级剔除归因修复后其余面可编译。

## 版本 pin（DIV-DEP-17）

**`=1.24.0` 精确 pin**（dep 与 oracle 同版）。caret 语义（"1.24.0"）会漂移到
最新 1.x（实勘 1.26.0 API 移除 → wrapper 整包编译失败）。改版需清
`~/.auto/sandbox` 相关键并三轨重跑（P592-D1 家族：自由函数 wrapper 缓存键
弱失效）。
