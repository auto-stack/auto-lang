# Unified Enum Migration Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** 将现有的 `enum`（标量枚举）和 `tag`（代数数据类型）统一为一个 `enum` 关键字，支持三种物理形态：标量枚举、同构数据枚举、异构数据枚举。

**Architecture:** 新增 `EnumKind` 枚举区分三种形态（Scalar / Homogeneous / Heterogeneous），扩展 `EnumDecl` 结构体承载所有形态的数据。废弃 `tag` 关键字，现有 `Tag` AST 节点的功能全部合并到 `EnumDecl`。解析器根据 `enum Name` 后跟随的 token 类型（`{` / 类型名 / 底层整数类型）自动判断形态。C/Rust/TS 转译器根据 `EnumKind` 分派到不同的代码生成策略。

**Tech Stack:** Rust, miette (错误报告), AutoLang compiler pipeline (Lexer → Parser → AST → Transpiler)

---

## 现有代码地图

| 组件 | 文件 | 关键行号 | 说明 |
|------|------|---------|------|
| Token | `crates/auto-lang/src/token.rs` | L100 Tag, L115 Enum | 关键字定义 |
| Lexer | `crates/auto-lang/src/lexer.rs` | L347 "tag", L361 "enum" | 关键字查找 |
| AST: EnumDecl | `crates/auto-lang/src/ast/enums.rs` | L1-115 | 标量枚举结构 |
| AST: Tag | `crates/auto-lang/src/ast/tag.rs` | L1-168 | Tag 结构（fields, methods, generics） |
| AST: Type | `crates/auto-lang/src/ast/types.rs` | L31 Tag, L32 Enum | 类型系统 |
| AST: Cover | `crates/auto-lang/src/ast/cover.rs` | L1-217 | Tag 模式匹配 |
| Parser: enum | `crates/auto-lang/src/parser.rs` | L3203-3262 | enum_stmt() |
| Parser: tag | `crates/auto-lang/src/parser.rs` | L6176-6254 | tag_stmt(), tag_field() |
| Parser: tag_cover | `crates/auto-lang/src/parser.rs` | L2546-2566 | Tag.Field 模式 |
| Trans C: enum | `crates/auto-lang/src/trans/c.rs` | L630-649 | C enum 生成 |
| Trans C: tag | `crates/auto-lang/src/trans/c.rs` | L491-548, L392-488 | C tag 生成 |
| Trans Rust: enum | `crates/auto-lang/src/trans/rust.rs` | L2703-2725 | Rust enum 生成 |
| Trans Rust: tag | `crates/auto-lang/src/trans/rust.rs` | L1583-1593, L2785+ | Rust tag 生成 |
| Trans TS | `crates/auto-lang/src/trans/ts_stmt.rs` | L580-630 | TypeScript tag 生成 |
| Infer | `crates/auto-lang/src/infer/stmt.rs` | L92-98 | 类型推断（enum/tag → Void） |
| Tests a2c | `test/a2c/007_enum/`, `test/a2c/014_tag/` 等 15+ 目录 | | C 转译测试 |
| Tests a2r | `test/a2r/007_enum/`, `test/a2r/014_tag/` | | Rust 转译测试 |
| Tests a2ts | `test/a2ts/014_tag/`, `test/a2ts/109_generic_tag/` | | TS 转译测试 |

---

## Phase 1: AST 重构（核心数据结构）

### Task 1: 新增 EnumKind 和扩展 EnumDecl

**Files:**
- Modify: `crates/auto-lang/src/ast/enums.rs`

**Step 1: 在 enums.rs 中新增 EnumKind 枚举和扩展 EnumDecl/EnumItem**

```rust
/// 三种枚举物理形态
#[derive(Debug, Clone, PartialEq)]
pub enum EnumKind {
    /// 标量枚举：纯整数状态值
    /// `enum Color { Red, Green, Blue }`
    /// `enum HttpCode u16 { OK = 200, NotFound = 404 }`
    Scalar {
        repr_type: Option<Type>,  // 底层整数类型（None = 默认 i32）
    },

    /// 同构数据枚举：所有分支共享同一负载类型
    /// `enum Vertex Point { LeftTop, LeftBottom, RightTop, RightBottom }`
    Homogeneous {
        payload_type: Type,       // 共享的负载类型
    },

    /// 异构数据枚举：各分支拥有独立负载类型（原 tag）
    /// `enum Msg { Quit, Move Point, Write string }`
    Heterogeneous {
        generic_params: Vec<GenericParam>,
        methods: Vec<super::Fn>,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct EnumDecl {
    pub name: AutoStr,
    pub kind: EnumKind,
    pub items: Vec<EnumItem>,
}

#[derive(Debug, Clone)]
pub struct EnumItem {
    pub name: AutoStr,
    // 标量值（Scalar 专用，其他形态为 None）
    pub scalar_value: Option<i32>,
    // 负载类型（Heterogeneous 专用，Homogeneous 继承共享类型）
    pub payload_type: Option<Type>,
}

// 向后兼容的 helper 方法
impl EnumDecl {
    /// 是否为标量枚举
    pub fn is_scalar(&self) -> bool {
        matches!(self.kind, EnumKind::Scalar { .. })
    }

    /// 是否为同构数据枚举
    pub fn is_homogeneous(&self) -> bool {
        matches!(self.kind, EnumKind::Homogeneous { .. })
    }

    /// 是否为异构数据枚举
    pub fn is_heterogeneous(&self) -> bool {
        matches!(self.kind, EnumKind::Heterogeneous { .. })
    }

    /// 获取标量值（仅 Scalar 有效）
    pub fn scalar_value_of(&self, name: &str) -> Option<i32> {
        if !self.is_scalar() { return None; }
        self.items.iter()
            .find(|item| item.name == name)
            .and_then(|item| item.scalar_value)
    }

    /// 获取分支的负载类型（Heterogeneous 专用）
    pub fn payload_type_of(&self, name: &str) -> Option<Type> {
        self.items.iter()
            .find(|item| item.name == name)
            .and_then(|item| item.payload_type.clone())
    }

    /// 获取默认值（标量枚举用）
    pub fn default_value(&self) -> i32 {
        self.items.first()
            .and_then(|item| item.scalar_value)
            .unwrap_or(0)
    }

    pub fn unique_name(&self) -> AutoStr {
        format!("{}", self.name).into()
    }

    pub fn get_item(&self, name: &str) -> Option<&EnumItem> {
        self.items.iter().find(|item| item.name == name)
    }
}
```

**注意**：此步骤会破坏编译。这是预期的 — 后续 Task 会修复所有引用。

**Step 2: 验证编译失败**

Run: `rtk cargo check -p auto-lang 2>&1 | head -30`
Expected: 编译错误，提示 `EnumItem` 缺少 `value` 字段等。

**Step 3: Commit**

```bash
git add crates/auto-lang/src/ast/enums.rs
git commit -m "feat(ast): extend EnumDecl with EnumKind for unified enum (Phase 1, Task 1)"
```

---

### Task 2: 添加向后兼容桥接，修复所有 EnumDecl 引用

**Files:**
- Modify: `crates/auto-lang/src/ast/enums.rs` — 保留旧的 `EnumItem` 兼容性
- Modify: `crates/auto-lang/src/ast/types.rs` — 适配新的 EnumDecl
- Modify: `crates/auto-lang/src/parser.rs` — 临时适配 enum_stmt
- Modify: `crates/auto-lang/src/trans/c.rs` — 临时适配 enum_decl
- Modify: `crates/auto-lang/src/trans/rust.rs` — 临时适配 enum_decl

**Step 1: 为 EnumItem 添加兼容性方法**

在 `EnumItem` 中添加 `value()` helper，使旧代码通过 `item.value()` 访问标量值：

```rust
impl EnumItem {
    /// 获取标量值（向后兼容）
    pub fn value(&self) -> i32 {
        self.scalar_value.unwrap_or(0)
    }
}
```

**Step 2: 修复所有编译错误**

全局搜索 `item.value`（不带括号）、`enum_decl.items`、`EnumItem { name, value }` 等旧模式，逐个适配：

- `parser.rs:3213-3226`: `enum_stmt()` 中构造 `EnumItem` 改为 `EnumItem { name, scalar_value: Some(value), payload_type: None }`，构造 `EnumDecl` 改为 `EnumDecl { name, kind: EnumKind::Scalar { repr_type: None }, items }`
- `c.rs:630-649`: `enum_decl()` 中 `item.value` 改为 `item.value()`
- `rust.rs:2703-2725`: 同上
- `types.rs:81,138`: `unique_name()` 和 `default_value()` 保持不变（已有方法）

**Step 3: 确认编译通过**

Run: `rtk cargo check -p auto-lang`
Expected: 编译成功，0 errors

**Step 4: 确认现有测试通过**

Run: `rtk cargo test -p auto-lang 2>&1 | tail -5`
Expected: 所有测试通过

**Step 5: Commit**

```bash
git add -A
git commit -m "feat(ast): migrate EnumDecl references to new EnumKind (Phase 1, Task 2)"
```

---

## Phase 2: Parser — 统一枚举解析

### Task 3: 重写 enum_stmt() 解析器支持三种形态

**Files:**
- Modify: `crates/auto-lang/src/parser.rs:3203-3262`

**Step 1: 编写标量枚举 + 指定底层类型的测试**

创建测试文件 `crates/auto-lang/test/a2c/200_enum_scalar_u16/enum_scalar_u16.at`：

```auto
enum HttpCode u16 {
    OK = 200
    NotFound = 404
    InternalError = 500
}

fn main() {
    let code HttpCode = HttpCode.OK
    print("code:", code)
}
```

创建测试函数（c.rs 中添加）：

```rust
#[test]
fn test_200_enum_scalar_u16() {
    test_a2c("200_enum_scalar_u16").unwrap();
}
```

Run: `rtk cargo test -p auto-lang test_200_enum_scalar_u16 2>&1 | tail -10`
Expected: FAIL（解析器还不支持 `enum Name u16 { ... }` 语法）

**Step 2: 编写同构数据枚举测试**

创建 `crates/auto-lang/test/a2c/201_enum_homo/enum_homo.at`：

```auto
type Point {
    x int
    y int
}

enum Vertex Point {
    LeftTop
    LeftBottom
    RightTop
    RightBottom
}

fn main() {
    let v Vertex = Vertex.LeftTop
    v.x = 10
    v.y = 20
    print("x:", v.x, "y:", v.y)
}
```

**Step 3: 编写异构数据枚举测试**

创建 `crates/auto-lang/test/a2c/202_enum_hetero/enum_hetero.at`：

```auto
enum Msg {
    Quit
    Move Point
    Write string
}

fn main() {
    let m Msg = Msg.Quit
    is m {
        Msg.Quit => print("quit")
        Msg.Move p => print("move:", p.x)
        Msg.Write s => print("write:", s)
    }
}
```

**Step 4: 重写 enum_stmt() 解析逻辑**

核心解析算法（伪代码）：

```
enum_stmt():
    skip 'enum'
    name = parse_ident()

    // 判断形态
    if cur is LBrace:
        // 无修饰：检查分支内容决定是 Scalar 还是 Heterogeneous
        items = parse_enum_items()
        if all items have no payload_type:
            kind = Scalar { repr_type: None }
        else:
            kind = Heterogeneous { generic_params: [], methods: [] }
    elif cur is ident and lookup_type(cur) is Type:
        // enum Name Type { ... } → Homogeneous
        payload_type = lookup_type(cur.text)
        skip type_name
        items = parse_enum_items_no_payload()  // 分支不带类型
        kind = Homogeneous { payload_type }
    elif cur is ident (int/uint/u8/u16/u32/i8/i16/i32/i64):
        // enum Name u16 { ... } → Scalar with repr type
        repr_type = parse_type()
        items = parse_enum_items()
        kind = Scalar { repr_type: Some(repr_type) }
    else:
        error("expected '{' or type name after enum name")
```

**关键实现细节**：

```rust
fn enum_stmt(&mut self) -> AutoResult<Stmt> {
    self.next(); // skip 'enum'
    let name: AutoStr = self.cur.text.clone().into();
    self.next();

    let kind;
    let items;

    if self.is_kind(TokenKind::LBrace) {
        // enum Name { ... } — 延迟判断 Scalar or Heterogeneous
        let parsed = self.parse_enum_body()?;
        kind = parsed.kind;
        items = parsed.items;
    } else if self.is_kind(TokenKind::Ident) {
        let next_text = self.cur.text.to_string();

        // 检查是否为底层整数类型
        if is_integer_type_name(&next_text) {
            // Scalar with repr type: enum Name u16 { ... }
            let repr_type = self.parse_type()?;
            kind = EnumKind::Scalar { repr_type: Some(repr_type) };
            items = self.parse_scalar_items()?;
        } else {
            // 检查是否为已定义的类型（Homogeneous）
            let looked_up = self.lookup_type(&Name::from(&next_text));
            match *looked_up.borrow() {
                Type::User(_) | Type::Tag(_) | Type::Enum(_) => {
                    let payload_type = looked_up.borrow().clone();
                    self.next(); // skip type name
                    kind = EnumKind::Homogeneous { payload_type };
                    items = self.parse_homo_items()?;
                }
                _ => {
                    // 回退：可能是异构枚举的分支名紧跟类型名
                    // 但 enum Name Type { ... } 要求 Type 是已定义类型
                    let span = pos_to_span(self.cur.pos);
                    return Err(SyntaxError::Generic {
                        message: format!("expected '{{' or known type after '{}', got '{}'", name, next_text),
                        span,
                    }.into());
                }
            }
        }
    } else {
        let span = pos_to_span(self.cur.pos);
        return Err(SyntaxError::Generic {
            message: format!("expected '{{' or type after '{}'", name),
            span,
        }.into());
    }

    let enum_decl = EnumDecl { name, kind, items };
    // 注册到符号表
    self.register_enum(&enum_decl);
    Ok(Stmt::EnumDecl(enum_decl))
}

/// 判断标识符是否为整数类型名
fn is_integer_type_name(s: &str) -> bool {
    matches!(s, "int" | "uint" | "u8" | "u16" | "u32" | "u64" | "usize" |
               "i8" | "i16" | "i32" | "i64" | "byte")
}
```

**Step 5: 实现 parse_scalar_items / parse_homo_items / parse_enum_body**

这些方法分别解析三种形态的分支。`parse_enum_body` 最复杂，需要延迟判断：

```rust
struct ParsedEnumBody {
    kind: EnumKind,
    items: Vec<EnumItem>,
}

fn parse_enum_body(&mut self) -> AutoResult<ParsedEnumBody> {
    self.expect(TokenKind::LBrace)?;
    self.skip_empty_lines();

    let mut items = Vec::new();
    let mut has_any_payload = false;

    while !self.is_kind(TokenKind::RBrace) && !self.is_kind(TokenKind::EOF) {
        let item_name: AutoStr = self.cur.text.clone().into();
        self.next();

        let mut payload_type = None;
        let mut scalar_value = None;

        // 检查是否有 = (标量赋值)
        if self.is_kind(TokenKind::Asn) {
            self.next();
            let value = self.parse_ints()?;
            scalar_value = Some(self.get_int_expr(&value) as i32);
        }
        // 检查是否有负载类型（空格分隔的类型名）
        else if self.is_kind(TokenKind::Ident) || self.is_kind(TokenKind::LParen) || self.is_kind(TokenKind::LBrace) {
            // 这是负载类型
            payload_type = Some(self.parse_enum_payload_type()?);
            has_any_payload = true;
        }

        items.push(EnumItem { name: item_name, scalar_value, payload_type });
        self.skip_enum_separator()?;
    }

    self.expect(TokenKind::RBrace)?;

    let kind = if has_any_payload {
        EnumKind::Heterogeneous {
            generic_params: Vec::new(),
            methods: Vec::new(),
        }
    } else {
        EnumKind::Scalar { repr_type: None }
    };

    Ok(ParsedEnumBody { kind, items })
}

/// 解析枚举分支的负载类型
/// 支持：类型名、元组 (T1, T2)、匿名结构体 { field Type }
fn parse_enum_payload_type(&mut self) -> AutoResult<Type> {
    if self.is_kind(TokenKind::LParen) {
        // 元组类型 (T1, T2, ...)
        self.next();
        let mut types = Vec::new();
        if !self.is_kind(TokenKind::RParen) {
            types.push(self.parse_type()?);
            while self.is_kind(TokenKind::Comma) {
                self.next();
                types.push(self.parse_type()?);
            }
        }
        self.expect(TokenKind::RParen)?;
        // 返回元组类型（用 RuntimeArray 或新类型表示）
        // 简化：如果只有一个元素，直接返回该类型
        if types.len() == 1 {
            Ok(types.into_iter().next().unwrap())
        } else {
            // 多元素元组 → 暂时用 Array 类型占位
            Ok(Type::Array(ArrayType {
                elem: Box::new(types.into_iter().next().unwrap()),
                len: 0, // 0 表示元组
            }))
        }
    } else if self.is_kind(TokenKind::LBrace) {
        // 匿名结构体 { field Type, ... }
        self.next();
        let mut members = Vec::new();
        self.skip_empty_lines();
        while !self.is_kind(TokenKind::RBrace) {
            let field_name: Name = self.cur.text.clone().into();
            self.next();
            let field_type = self.parse_type()?;
            members.push(Member::new(field_name, field_type, None));
            if self.is_kind(TokenKind::Comma) {
                self.next();
            }
            self.skip_empty_lines();
        }
        self.expect(TokenKind::RBrace)?;
        // 匿名结构体 → 创建 TypeDecl
        let type_decl = TypeDecl {
            name: "<anonymous>".into(),
            kind: TypeDeclKind::UserType,
            parent: None,
            has: Vec::new(),
            specs: Vec::new(),
            spec_impls: Vec::new(),
            generic_params: Vec::new(),
            members,
            delegations: Vec::new(),
            methods: Vec::new(),
        };
        Ok(Type::User(type_decl))
    } else {
        // 普通类型名
        self.parse_type()
    }
}
```

**Step 6: 验证测试**

Run: `rtk cargo test -p auto-lang test_200_enum_scalar_u16 2>&1 | tail -10`
Expected: 解析通过（但转译器可能还不支持，先验证解析成功）

**Step 7: Commit**

```bash
git add -A
git commit -m "feat(parser): unified enum parsing for 3 forms (Phase 2, Task 3)"
```

---

### Task 4: 废弃 tag_stmt()，将 tag 关键字重定向到 enum_stmt()

**Files:**
- Modify: `crates/auto-lang/src/parser.rs` — tag 入口重定向
- Modify: `crates/auto-lang/src/token.rs` — 添加废弃警告

**Step 1: 修改主解析入口，将 `tag` 重定向到 `enum_stmt()`**

在 `parser.rs` 的语句分发处（搜索 `TokenKind::Tag`），改为：

```rust
TokenKind::Tag => {
    // DEPRECATED: tag 关键字重定向到 enum_stmt
    // tag Name { ... } 等价于 enum Name { ... }（异构形态）
    self.enum_stmt()
}
```

同时在 `enum_stmt()` 开头添加兼容：如果当前 token 是 `Tag`，也跳过它。

**Step 2: 在 token.rs 添加 tag 废弃标记（可选，非阻塞）**

在 `keyword_kind` 中对 `"tag"` 添加注释标记 `// DEPRECATED: use enum`。

**Step 3: 验证 tag 测试仍然通过**

Run: `rtk cargo test -p auto-lang -- trans 2>&1 | tail -10`
Expected: 所有现有 tag 测试仍然通过（通过重定向兼容）

**Step 4: Commit**

```bash
git add -A
git commit -m "feat(parser): deprecate tag keyword, redirect to enum (Phase 2, Task 4)"
```

---

### Task 5: 支持异构枚举的方法和泛型参数

**Files:**
- Modify: `crates/auto-lang/src/parser.rs` — 扩展 parse_enum_body

**Step 1: 扩展 parse_enum_body 支持泛型参数和方法**

```rust
// 在 enum_stmt() 中，LBrace 之前检查泛型参数
if self.is_kind(TokenKind::Lt) {
    // enum Name<T> { ... }
    self.next();
    generic_params.push(self.parse_generic_param()?);
    while self.is_kind(TokenKind::Comma) {
        self.next();
        generic_params.push(self.parse_generic_param()?);
    }
    self.expect(TokenKind::Gt)?;
}
```

在 `parse_enum_body` 内部支持 `fn` 关键字：

```rust
while !self.is_kind(TokenKind::RBrace) && !self.is_kind(TokenKind::EOF) {
    if self.is_kind(TokenKind::Fn) {
        // 异构枚举内部方法
        let fn_stmt = self.fn_decl_stmt(&name.into())?;
        if let Stmt::Fn(fn_expr) = fn_stmt {
            methods.push(fn_expr);
        }
    } else {
        // 分支解析
        // ...
    }
}
```

**Step 2: 编写测试**

创建 `crates/auto-lang/test/a2c/203_enum_generic/enum_generic.at`：

```auto
enum May<T> {
    Some T
    None
}

fn main() {
    let m May<int> = May.Some(42)
    is m {
        May.Some v => print("value:", v)
        May.None => print("none")
    }
}
```

**Step 3: Commit**

```bash
git add -A
git commit -m "feat(parser): enum with generics and methods (Phase 2, Task 5)"
```

---

## Phase 3: 转译器适配

### Task 6: C 转译器适配

**Files:**
- Modify: `crates/auto-lang/src/trans/c.rs`

**Step 1: 重写 enum_decl() 根据 EnumKind 分派**

```rust
fn enum_decl(&mut self, enum_decl: &EnumDecl, sink: &mut Sink) -> AutoResult<()> {
    match &enum_decl.kind {
        EnumKind::Scalar { repr_type } => {
            self.scalar_enum_to_c(enum_decl, repr_type, sink)
        }
        EnumKind::Homogeneous { payload_type } => {
            self.homo_enum_to_c(enum_decl, payload_type, sink)
        }
        EnumKind::Heterogeneous { generic_params, methods } => {
            // 复用现有 tag 生成逻辑
            self.hetero_enum_to_c(enum_decl, generic_params, methods, sink)
        }
    }
}
```

**Scalar → C:**
```c
// enum HttpCode u16 { OK = 200, NotFound = 404 }
// 生成:
typedef uint16_t HttpCode;
#define HTTPCODE_OK 200
#define HTTPCODE_NOTFOUND 404
```

**Homogeneous → C:**
```c
// enum Vertex Point { LeftTop, LeftBottom }
// 生成:
struct Vertex {
    enum VertexKind tag;
    struct Point payload;
};
enum VertexKind { VERTEX_LEFTTOP, VERTEX_LEFTBOTTOM };
```

**Heterogeneous → C:** 复用现有 `tag_enum()` + `tag_struct()` 逻辑。

**Step 2: 运行 C 转译测试**

Run: `rtk cargo test -p auto-lang -- trans 2>&1 | tail -10`
Expected: 所有现有 enum 和 tag 测试通过 + 新测试通过

**Step 3: Commit**

```bash
git add -A
git commit -m "feat(trans-c): unified enum code generation (Phase 3, Task 6)"
```

---

### Task 7: Rust 转译器适配

**Files:**
- Modify: `crates/auto-lang/src/trans/rust.rs`

**Step 1: 重写 enum_decl() 根据 EnumKind 分派**

```rust
fn enum_decl(&mut self, enum_decl: &EnumDecl, sink: &mut Sink) -> AutoResult<()> {
    match &enum_decl.kind {
        EnumKind::Scalar { repr_type } => {
            // Rust: enum Name { A = 0, B = 1, ... }
            // 或者 type Name = u16; const A: Name = 200;
        }
        EnumKind::Homogeneous { payload_type } => {
            // Rust: enum Name { A(Point), B(Point) }
        }
        EnumKind::Heterogeneous { generic_params, methods } => {
            // 复用现有 tag → Rust enum 逻辑
        }
    }
}
```

**Step 2: 运行 Rust 转译测试**

Run: `rtk cargo test -p auto-lang -- trans 2>&1 | tail -10`
Expected: 所有 a2r 测试通过

**Step 3: Commit**

```bash
git add -A
git commit -m "feat(trans-rust): unified enum code generation (Phase 3, Task 7)"
```

---

### Task 8: TypeScript 转译器适配

**Files:**
- Modify: `crates/auto-lang/src/trans/ts_stmt.rs`

**Step 1: 更新 tag_decl → enum_decl**

TypeScript 转译器需要根据 EnumKind 生成不同代码：

```typescript
// Scalar → TS: type Name = number; const A = 0; const B = 1;
// Homogeneous → TS: type Name = { tag: "A", payload: Point } | ...
// Heterogeneous → TS: 复用现有 tag → discriminated union 逻辑
```

**Step 2: 运行 TS 转译测试**

Run: `rtk cargo test -p auto-lang -- trans 2>&1 | tail -10`

**Step 3: Commit**

```bash
git add -A
git commit -m "feat(trans-ts): unified enum code generation (Phase 3, Task 8)"
```

---

## Phase 4: 模式匹配适配

### Task 9: 更新 Cover/Uncover 模式匹配支持 enum

**Files:**
- Modify: `crates/auto-lang/src/ast/cover.rs` — 扩展 TagCover → EnumCover
- Modify: `crates/auto-lang/src/parser.rs:2546-2566` — 更新 tag_cover 解析

**Step 1: 扩展 Cover 枚举**

```rust
pub enum Cover {
    Tag(TagCover),      // 保留（向后兼容）
    Enum(EnumCover),    // 新增：统一枚举模式
}

pub struct EnumCover {
    pub enum_name: AutoStr,   // 枚举名
    pub variant: AutoStr,     // 分支名
    pub binding: AutoStr,     // 绑定变量名
}
```

**Step 2: 更新解析器**

当遇到 `EnumName.Variant` 或 `.Variant` 模式时，检查 `enum_name` 是否为已注册的枚举（而非仅检查 Tag），如果是则创建 `Cover::Enum`。

**Step 3: Commit**

```bash
git add -A
git commit -m "feat(parser): enum pattern matching with Cover::Enum (Phase 4, Task 9)"
```

---

## Phase 5: 测试与清理

### Task 10: 迁移现有 tag 测试为 enum 语法

**Files:**
- Modify: `crates/auto-lang/test/a2c/014_tag/tag.at` — 改为 enum 语法
- Modify: 所有 `test/a2c/0??_tag*/` 和 `test/a2c/0??_may*/` 测试
- Update `.expected.c` / `.expected.h` 文件

**Step 1: 逐一迁移测试文件**

将 `tag Atom { Int int, Char char }` 改为 `enum Atom { Int int, Char char }`。

更新的 expected 输出不变（C 代码生成逻辑不变，只是入口变化）。

**Step 2: 运行全部测试**

Run: `rtk cargo test -p auto-lang 2>&1 | tail -10`
Expected: 全部通过

**Step 3: Commit**

```bash
git add -A
git commit -m "test: migrate tag tests to unified enum syntax (Phase 5, Task 10)"
```

---

### Task 11: 清理旧 Tag AST（可选，非阻塞）

**Files:**
- Modify: `crates/auto-lang/src/ast/tag.rs` — 标记 deprecated
- Modify: `crates/auto-lang/src/ast/types.rs` — 可选移除 `Type::Tag`
- Modify: `crates/auto-lang/src/ast/mod.rs` — 导出调整

**注意**：此步骤可延后执行。`Type::Tag` 在内部仍可作为 `EnumKind::Heterogeneous` 的别名存在，以减少一次性变更量。

**Step 1: 在 tag.rs 添加 deprecated 标注**

```rust
#[deprecated(note = "Use EnumDecl with EnumKind::Heterogeneous instead")]
pub struct Tag { ... }
```

**Step 2: 验证编译和测试**

Run: `rtk cargo test -p auto-lang 2>&1 | tail -10`

**Step 3: Commit**

```bash
git add -A
git commit -m "chore: mark Tag AST as deprecated (Phase 5, Task 11)"
```

---

## 成功标准

1. **语法统一**：`tag` 关键字废弃（重定向到 `enum`），所有枚举用 `enum` 声明
2. **三种形态**：
   - `enum Color { Red, Green }` — 标量枚举
   - `enum HttpCode u16 { OK = 200 }` — 带底层类型的标量枚举
   - `enum Vertex Point { LeftTop, RightTop }` — 同构数据枚举
   - `enum Msg { Quit, Move Point, Write string }` — 异构数据枚举
3. **向后兼容**：现有 `tag` 语法通过重定向继续工作
4. **全平台支持**：C / Rust / TypeScript 转译器全部适配
5. **测试覆盖**：所有现有测试通过 + 新增形态测试

## 风险与缓解

| 风险 | 缓解策略 |
|------|---------|
| Tag 内部使用广泛（15+ 测试目录） | Phase 2 重定向而非删除，渐进迁移 |
| EnumDecl 结构变更影响面大 | Phase 1 先改结构 + 桥接方法，再逐步适配 |
| 同构枚举的 O(1) 偏移访问需要特殊处理 | Homogeneous 形态在 C 中生成共享 payload struct |
| 泛型 tag (如 `May<T>`) 迁移 | 复用现有泛型参数解析逻辑到 enum |
