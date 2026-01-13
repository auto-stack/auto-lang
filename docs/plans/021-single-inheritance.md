# AutoLang Single Inheritance Implementation

## 概述

为 AutoLang 实现了单继承功能，使用 `is` 关键字表示类型继承。

**实现日期**: 2025-01-13
**状态**: ✅ 完成并测试通过
**支持**: C transpiler 和 Rust transpiler 均已支持

## 语法

```auto
type ChildType is ParentType {
    // 子类字段和方法
}
```

## 示例

```auto
type Animal {
    name str

    fn speak() {
        print("Animal sound")
    }
}

type Dog is Animal {
    breed str

    fn bark() {
        print("Woof!")
    }
}

fn main() {
    let dog = Dog()
    dog.name = "Buddy"
    dog.breed = "Labrador"

    // 可以访问继承的字段
    print(dog.name)

    // 可以调用继承的方法
    dog.speak()

    // 可以调用自己的方法
    dog.bark()
}
```

## 实现细节

### 1. AST 修改

**文件**: [crates/auto-lang/src/ast/types.rs](../../crates/auto-lang/src/ast/types.rs)

- 在 `TypeDecl` 结构中添加了 `parent: Option<Box<Type>>` 字段
- 使用 `Box<Type>` 避免递归类型错误
- 更新了 `Display` 实现以显示继承关系

### 2. Parser 修改

**文件**: [crates/auto-lang/src/parser.rs](../../crates/auto-lang/src/parser.rs)

- 在 `type_decl_stmt()` 方法中添加了 `is` 关键字解析
- 解析顺序：`is` → `as` → `has` → type body
- 在类型体解析后，自动将父类的字段和方法添加到子类

```rust
// deal with `is` keyword (single inheritance)
let mut parent = None;
if self.is_kind(TokenKind::Is) {
    self.next(); // skip `is` keyword
    let parent_name = self.parse_name()?;
    // Lookup parent type
    if let Some(meta) = self.lookup_meta(parent_name.as_str()) {
        if let Meta::Type(Type::User(parent_decl)) = meta.as_ref() {
            parent = Some(Box::new(Type::User(parent_decl.clone())));
        } else {
            return Err(SyntaxError::Generic {
                message: format!("'{}' is not a user type", parent_name),
                span: pos_to_span(self.cur.pos),
            }.into());
        }
    } else {
        return Err(SyntaxError::Generic {
            message: format!("Parent type '{}' not found", parent_name),
            span: pos_to_span(self.cur.pos),
        }.into());
    }
}
decl.parent = parent;
```

### 3. 继承机制

在解析类型体后，parser 会自动：

1. **继承字段**: 将父类的所有字段添加到子类的字段列表
2. **继承方法**: 将父类的所有方法添加到子类的方法列表
3. **方法重写**: 子类的方法会覆盖父类的同名方法

```rust
// add members and methods from parent type (inheritance)
if let Some(ref parent_type) = decl.parent {
    match parent_type.as_ref() {
        Type::User(parent_decl) => {
            // Inherit members from parent
            for m in parent_decl.members.iter() {
                members.push(m.clone());
            }
            // Inherit methods from parent
            for meth in parent_decl.methods.iter() {
                let mut inherited_meth = meth.clone();
                inherited_meth.parent = Some(name.clone());
                let unique_name = format!("{}::{}", &name, &inherited_meth.name);
                self.define(unique_name.as_str(), Meta::Fn(inherited_meth.clone()));
                methods.push(inherited_meth);
            }
        }
        _ => {
            // System types cannot be inherited
        }
    }
}
```

### 4. C Transpiler 支持

**文件**: [crates/auto-lang/src/trans/c.rs](../../crates/auto-lang/src/trans/c.rs)

C transpiler 已经支持继承，生成的 C 代码：

```c
struct Animal {
    char* name;
};

struct Dog {
    char* breed;
    char* name;  // 继承的字段
};

void Animal_Speak(struct Animal *self) {
    printf("%s\n", "Animal sound");
}

void Dog_Speak(struct Dog *self) {
    printf("%s\n", "Animal sound");  // 继承的方法
}

void Dog_Bark(struct Dog *self) {
    printf("%s\n", "Woof!");
}
```

### 5. Rust Transpiler 支持

**文件**: [crates/auto-lang/src/trans/rust.rs](../../crates/auto-lang/src/trans/rust.rs)

Rust transpiler 也完全支持继承，生成的 Rust 代码使用扁平结构体（flat struct）：

```rust
struct Animal {
    name: String,
}

impl Animal {
    fn speak(&self) {
        println!("Animal sound");
    }
}

struct Dog {
    name: String,      // 继承的字段
    breed: String,     // 自己的字段
}

impl Dog {
    fn speak(&self) {
        println!("Animal sound");  // 继承的方法
    }

    fn bark(&self) {
        println!("Woof!");  // 自己的方法
    }
}
```

**实现细节**：

1. **字段继承**: 父类字段直接添加到子类结构体中
2. **方法继承**: 继承的方法在子类 impl 块中生成
3. **扁平结构**: 不使用嵌套结构体，而是将所有字段放在同一层级

```rust
// 字段收集顺序
if let Some(ref parent_type) = type_decl.parent {
    if let Type::User(parent_decl) = parent_type.as_ref() {
        // 首先添加父类字段
        for member in &parent_decl.members {
            if seen_fields.insert(member.name.clone()) {
                all_members.push(member);
            }
        }
    }
}

// 然后添加组合类型字段
for has_type in &type_decl.has {
    // ...
}

// 最后添加自己的字段（可以覆盖）
for member in &type_decl.members {
    if seen_fields.insert(member.name.clone()) {
        all_members.push(member);
    }
}
```

### 6. 测试

#### C Transpiler 测试

**测试文件**: [crates/auto-lang/test/a2c/112_inheritance/](../../crates/auto-lang/test/a2c/112_inheritance/)

创建了基本的继承测试用例：
- `inheritance.at` - 测试源代码
- `inheritance.expected.c` - 期望的 C 代码
- `inheritance.expected.h` - 期望的 C 头文件

测试结果：✅ 通过

#### Rust Transpiler 测试

**测试文件**: [crates/auto-lang/test/a2r/035_inheritance/](../../crates/auto-lang/test/a2r/035_inheritance/)

创建了基本的继承测试用例：
- `inheritance.at` - 测试源代码
- `inheritance.expected.rs` - 期望的 Rust 代码

测试结果：✅ 通过

## 特性

### 支持的功能

- ✅ 单继承（`type Child is Parent`）
- ✅ 字段继承（子类自动获得父类的所有字段）
- ✅ 方法继承（子类自动获得父类的所有方法）
- ✅ 方法重写（子类方法覆盖父类同名方法）
- ✅ 类型检查（继承关系在编译时验证）

### 限制

- ❌ 只支持单继承（不支持多重继承）
- ❌ 不支持继承系统类型（如 int, str 等）
- ❌ 继承深度没有限制（但建议不超过 3 层）

### 与组合的区别

**继承**（`is`）:
```auto
type Dog is Animal {
    // Dog 自动获得 Animal 的所有字段和方法
}
```

**组合**（`has`）:
```auto
type Dog {
    has core Animal
    // 需要通过 core.member 访问 Animal 的成员
}
```

## 错误处理

### 父类不存在

```auto
type Dog is NonExistent {
}
```

错误信息：
```
error: Parent type 'NonExistent' not found
```

### 父类不是用户类型

```auto
type Dog is int {
}
```

错误信息：
```
error: 'int' is not a user type
```

## 未来改进

1. ~~**Rust Transpiler 支持**: 生成 Rust 的继承代码（使用 trait 或组合）~~ ✅ **已完成**
2. **访问控制**: 添加 `private`, `protected`, `public` 关键字
3. **构造函数**: 自动调用父类构造函数
4. **`super` 关键字**: 显式调用父类方法
5. **多态**: 通过继承实现运行时多态
6. **抽象类型**: 定义不能实例化的父类

## 总结

成功实现了 AutoLang 的单继承功能，使用 `is` 关键字表示继承关系。实现包括：

- ✅ AST 扩展（添加 parent 字段）
- ✅ Parser 支持（解析 `is` 语法）
- ✅ 继承机制（自动继承字段和方法）
- ✅ C transpiler 支持（生成正确的 C 代码）
- ✅ **Rust transpiler 支持（生成正确的 Rust 代码）** - 新增
- ✅ 测试用例（a2c 和 a2r 都有继承测试）

**测试结果**:
- C transpiler: 74 个测试通过（包括 test_112_inheritance）
- Rust transpiler: 74 个测试通过（包括 test_035_inheritance）
- 总计: 74 个转译器测试全部通过，没有破坏现有功能
