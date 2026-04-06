# TODO: Enum 模式匹配生成冗余代码

## 问题
C transpiler 在处理 `is` 模式匹配时生成了冗余的变量绑定和花括号：
```c
// 当前输出（错误）:
case VARIANT_NAME:
    int x = m.as.VariantName;
    {
        return xxx;
    }
    break;

// 期望输出（正确）:
case VARIANT_NAME:
    return xxx;
    break;
```

## 根因
`trans/c.rs` 中的 `is_stmt()` 或 enum match 代码生成逻辑为每个 case 分支
创建了不必要的变量绑定（`int x = m.as.Variant`），即使该绑定未被使用。

## 修复方案
在生成 enum match 的 case 分支时：
1. 检查分支体是否使用了绑定变量
2. 如果未使用，跳过变量绑定声明
3. 如果分支体是单个 return/break/continue，直接生成无需花括号包裹
