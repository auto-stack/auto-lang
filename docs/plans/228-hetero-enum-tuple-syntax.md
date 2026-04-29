# Plan 228: Hetero Enum 多参数变体要求括号元组语法

## Context

当前 Auto 的 hetero enum（tagged union）允许多参数变体使用无括号写法：

```auto
enum OutputContentBlock {
    Text str
    ToolUse str str str
}
```

这种写法有几个问题：
1. **歧义**：`ToolUse str str str` 看起来像三个独立的类型声明，不像一个变体携带三个参数
2. **parser 复杂度**：需要 while 循环贪婪地持续解析类型（`parser.rs ~L4014`），遇到 `Ident` 就尝试下一个类型，无法区分"变体名后的类型列表"和"下一个变体声明"
3. **与主流语言不一致**：Rust `ToolUse(String, String, String)`、C# `ToolUse(string, string, string)`、Swift `case toolUse(String, String, String)` 均使用括号

## 变更

多参数（≥2 个）hetero enum 变体**必须**使用括号元组形式，变体名与左括号之间保留一个空格：

```auto
// 新语法（要求）
enum OutputContentBlock {
    Text str                        // 单参数：保持不变
    ToolUse(str, str, str)          // 多参数：必须加括号和逗号
}
```

**单参数变体不受影响**：`Text str` 和 `Text(str)` 均合法，但推荐 `Text str`。

## 涉及修改

### 1. parser.rs — enum variant 解析

**文件**：`crates/auto-lang/src/parser.rs` ~L4007-4025

**当前逻辑**：
```rust
} else if self.is_kind(TokenKind::Ident)
    || self.is_kind(TokenKind::LParen)
{
    // 单参数或多参数：while 循环贪婪解析
    let mut types = vec![self.parse_type()?];
    while self.is_kind(TokenKind::Ident)
        || self.is_kind(TokenKind::LParen)
        || self.is_kind(TokenKind::Question)
        || self.is_kind(TokenKind::Not)
    {
        types.push(self.parse_type()?);
    }
    // ...
}
```

**目标逻辑**：
```rust
} else if self.is_kind(TokenKind::LParen) {
    // 括号形式：ToolUse(str, str, str)
    self.next(); // 消费 '('
    let mut types = vec![];
    loop {
        types.push(self.parse_type()?);
        if self.is_kind(TokenKind::Comma) {
            self.next(); // 消费 ','
        } else {
            break;
        }
    }
    self.expect(TokenKind::RParen)?; // 期望 ')'
    payload_types = types;
    has_any_payload = true;
} else if self.is_kind(TokenKind::Ident)
    || self.is_kind(TokenKind::Question)
    || self.is_kind(TokenKind::Not)
{
    // 单参数形式：Text str、Some ?int、Err !str
    payload_type = Some(self.parse_type()?);
    has_any_payload = true;
}
```

**关键变化**：
- 去掉 while 循环，改为两个互斥分支
- `LParen` → 解析括号内逗号分隔的类型列表（≥1 个）
- `Ident/Question/Not` → 只解析单个类型
- 括号内单个类型 `ToolUse(str)` 也合法，存入 `payload_types`（长度为 1）

### 2. 构造语法

多参数变体的构造也需要使用括号：

```auto
// 当前
let block = OutputContentBlock.ToolUse("id123", "bash", "ls")

// 新语法（不变，构造已经用括号）
let block = OutputContentBlock.ToolUse("id123", "bash", "ls")
```

构造语法无需修改，已经使用括号。

### 3. `is` 模式匹配

```auto
// 当前
is block {
    OutputContentBlock.ToolUse(id name input) -> print(id)
}

// 新语法（不变，匹配已经用括号）
is block {
    OutputContentBlock.ToolUse(id, name, input) -> print(id)
}
```

匹配语法的括号内是否需要逗号取决于现有 parser 行为，可作为后续改进。

### 4. 现有代码适配

需要更新所有使用无括号多参数写法的 `.at` 文件：

| 文件 | 变体 | 修改 |
|------|------|------|
| `step-00-api-minimal/main.at` | `ToolUse str str str` | → `ToolUse(str, str, str)` |
| `step-00-api-minimal/main.at` | `Text str` | 不变（单参数） |
| `step-00-api-minimal/test-regression.at` | `ToolUse str str str` | → `ToolUse(str, str, str)` |
| auto-lang 测试文件中的多参数 enum | 各处 | 统一更新 |

### 5. 文档更新

- `auto-lang-creator` 技能 Gotcha Checklist
- Auto 语法参考文档
- `CStr` → `String` a2r 测试的 expected 文件（如有涉及）

## 迁移策略

为了不破坏现有代码，可分两步：

1. **Phase 1**：parser 同时接受两种写法（无括号发出 deprecation warning）
2. **Phase 2**：移除无括号支持，只接受括号形式

或者直接一步到位（当前多参数 enum 使用量很少，只在 step-00 的几个测试中出现）。

## 验证

```bash
cd d:/autostack/auto-lang

# 确认括号形式可解析
cat <<'EOF' > /tmp/test_enum.at
enum OutputContentBlock {
    Text str
    ToolUse(str, str, str)
}
fn main() {
    let block = OutputContentBlock.ToolUse("id", "bash", "ls")
    is block {
        OutputContentBlock.ToolUse(id, name, input) -> print(id)
    }
}
EOF
cargo run --bin auto -- /tmp/test_enum.at

# 确认无括号形式被拒绝
cat <<'EOF' > /tmp/test_enum_old.at
enum OutputContentBlock {
    ToolUse str str str
}
EOF
cargo run --bin auto -- /tmp/test_enum_old.at
# 预期：parse error

# 回归测试
cargo test -p auto-lang --lib
```
