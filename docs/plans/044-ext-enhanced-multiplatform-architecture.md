# AutoLang 标准库重构计划：基于 ext 机制的多平台架构

## 目标

实现 `docs/design/stdlib-organization.md` 中描述的**"接口契约 + 物理补全"**架构，核心创新是扩展 `ext` 以支持**同模块私有成员变量添加**，从而实现平台特定的内存布局，同时保持公共接口兼容性。

**核心改变**：
1. **编译器增强**：扩展 `ext` 以支持同模块私有字段添加
2. **标准库迁移**：重构现有模块以使用新的 ext 能力
3. **Transpiler 更新**：生成合并字段的正确 C 结构体
4. **测试策略**：每个阶段的全面验证

---

## 为什么需要重构？

### 当前问题

1. **代码重复**：`io.c.at` 中重复定义了 `path` 和 `file` 字段
2. **不灵活**：无法为不同平台定义不同的内存布局
3. **维护困难**：需要在多个文件间保持同步
4. **类型不安全**：C 平台和 VM 平台使用相同的底层结构

### 目标方案优势

根据 `docs/design/stdlib-organization.md`：

1. **接口契约阶段**（.at 文件）：
   - 定义跨平台通用的公开字段和方法签名
   - 例如：`File { path str }` - 只定义用户可见的接口

2. **物理补全阶段**（.c.at/.vm.at 文件）：
   - 通过 `ext` 关键字添加平台特定的私有成员
   - 例如：C 平台添加 `_fp *FILE`，VM 平台添加 `_handle uint64`
   - 提供平台特定的实现

3. **编译期缝合**：
   - 收集阶段：扫描所有 `type` 和 `ext` 块
   - 合并布局：`Public Fields + Private Fields` → 最终结构体
   - 方法注入：将所有 `ext` 方法翻译为目标平台函数

---

## 设计示例：auto.io.File

### 主文件 io.at（公共契约）

```auto
type File {
    #[pub]
    path str
}
```

### io.c.at（C 平台补全）

```auto
use c.stdio: FILE, fopen, fclose

ext File {
    // 同模块：可以添加私有成员
    _fp *FILE

    #[pub]
    static fn open(path str) ?File {
        let f = fopen(path.to_cstr(), c"r")
        if f == nil {
            return nil
        }
        // 可以初始化私有字段！
        return File(path: path, _fp: f)
    }

    #[pub]
    fn read_text() str {
        let buf cstr = c"                                        "
        fgets(buf, 40, ._fp)
        buf
    }

    #[pub]
    fn close() {
        fclose(._fp)
    }
}
```

### io.vm.at（VM 平台补全）

```auto
ext File {
    // VM 使用句柄而非指针
    _handle uint64

    #[vm]
    static fn open(path str) File

    #[vm]
    fn read_text() str

    #[vm]
    fn close()
}
```

### 生成的 C 代码

```c
#pragma once
#include <stdio.h>

typedef struct {
    char* path;    // 来自 io.at
    FILE* _fp;     // 来自 io.c.at ext！
} File;

File File_open(char* path);
char* File_read_text(File* self);
void File_close(File* self);
```

**关键优势**：
- ✅ C 平台有 `FILE*` 指针，VM 平台有 `uint64` 句柄
- ✅ 公共接口保持一致（用户只能看到 `path` 字段）
- ✅ 类型安全：不需要在 `void*` 和 `u64` 间转换
- ✅ 零开销抽象：直接使用平台原生类型

---

## 实施阶段

### 阶段 1：编译器基础设施增强 ✅ (已完成)

**目标**：扩展解析器和类型系统以支持带私有字段的 `ext`。

#### 1.1 AST 扩展 ([`crates/auto-lang/src/ast/ext.rs`](crates/auto-lang/src/ast/ext.rs))

**✅ 已完成**：
- 添加了 `fields: Vec<Member>` 字段
- 添加了 `module_path: AutoStr` 字段
- 添加了 `is_same_module: bool` 字段
- 实现了 `Ext::with_fields()` 构造函数
- 更新了所有 trait 实现（Display, AtomWriter, ToNode, PartialEq）

#### 1.2 解析器增强 ([`crates/auto-lang/src/parser.rs:1861-2045`](crates/auto-lang/src/parser.rs#L1861-L2045))

**✅ 已完成**：
- 解析 `name Type` 格式的字段声明（与 type 定义语法一致）
- 支持 `#[...]` 和 `[...]` 注解语法
- 实现条件编译（根据 compile_dest 跳过不兼容的函数）
- 添加友好的错误消息（引导用户使用正确的语法）
- 支持 ext 字段的默认值：`name Type = value`

**关键实现**：
```rust
// 字段语法：name Type（与 type 定义一致）
ext File {
    _fp *FILE           // ✅ 正确
    // _fp: *FILE        // ❌ 错误（不带冒号）
}
```

#### 1.3 作用域系统更新

**⏸️ 待实现**：
- 添加 `Universe::current_module()` 方法
- 添加 `Universe::register_type()` 的模块参数
- 在类型注册时跟踪模块路径

#### 1.4 类型系统集成

**⏸️ 待实现**：
- 在 TypeDecl 中添加 `private_members: Vec<Member>` 字段
- 实现 `TypeDecl::merge_ext()` 方法
- 实现 `TypeDecl::all_fields()` 方法

---

### 阶段 2：C Transpiler 增强 ⏸️ (未开始)

**目标**：生成合并类型定义 + ext 字段的 C 结构体。

#### 2.1 类型声明生成 ([`crates/auto-lang/src/trans/c.rs`](crates/auto-lang/src/trans/c.rs))

**需要实现**：
- 修改结构体生成逻辑以使用 `decl.all_fields()`
- 生成合并的 C 结构体（公开字段 + 私有字段）

#### 2.2 模块收集阶段

**需要实现**：
- 创建 `ModuleCollector` 结构体
- 实现多文件加载逻辑（.at + .c.at 或 .vm.at）
- 实现 `merge_extensions()` 方法

#### 2.3 条件编译

**已部分实现**：
- ✅ 文件加载已支持（`get_file_extensions()`）
- ✅ 条件跳过已实现（在 `parse_ext_stmt` 中）

---

### 阶段 3：标准库迁移 ⏸️ (未开始)

**迁移优先级**：

**高优先级**（需要平台分离的复杂类型）：
1. `io.at` - 带平台句柄的文件 I/O
2. `sys.at` - 系统调用（getpid 等）

**中优先级**（简单类型）：
3. `builder.at` - StringBuilder 变体
4. `str.at` - 已使用 ext 添加方法

**低优先级**（已经工作正常）：
5. `math.at` - 纯 Auto 实现，无需修改

#### 3.1 io.at 重构

**当前状态**：
- `io.at` 有完整的类型定义和方法声明
- `io.vm.at` 有 ext 块，但字段为空
- 需要迁移到新架构

**重构后设计**：

**io.at**（公共接口）：
```auto
type File {
    #[pub]
    path str
}
```

**io.c.at**（C 平台实现）：
```auto
use c.stdio: FILE, fopen, fclose, fgets

ext File {
    // 同模块：可以添加私有成员
    _fp *FILE

    #[pub]
    static fn open(path str) ?File {
        let f = fopen(path.to_cstr(), c"r")
        if f == nil {
            return nil
        }
        return File(path: path, _fp: f)
    }

    #[pub]
    fn read_text() ?str {
        let buf cstr = c"                                        "
        if fgets(buf, 40, ._fp) == nil {
            return nil
        }
        buf
    }

    #[pub]
    fn close() {
        fclose(._fp)
    }
}
```

**io.vm.at**（VM 平台实现）：
```auto
ext File {
    // VM 使用句柄而非指针
    _handle uint64

    #[vm]
    static fn open(path str) File

    #[vm]
    fn read_text() str

    #[vm]
    fn close()
}
```

---

### 阶段 4：测试策略 ⏸️ (部分完成)

#### 4.1 单元测试

**✅ 已完成** ([`crates/auto-lang/src/ast/ext.rs`](crates/auto-lang/src/ast/ext.rs))：
- `test_ext_creation` - 基础 ext 创建
- `test_ext_display` - Display 格式化
- `test_ext_equality` - 相等性比较
- `test_ext_with_fields` - 带 ext 字段的创建
- `test_ext_display_with_fields` - 带 ext 字段的 Display
- `test_ext_to_node_with_fields` - 带 ext 字段的 ToNode 转换

#### 4.2 集成测试

**⏸️ 待创建**：
- a2c 测试用例：`114_ext_fields/`
- 验证生成的 C 结构体包含私有字段

#### 4.3 验证测试

**⏸️ 待创建**：
- 内存布局测试
- 跨平台编译测试

---

### 阶段 5：实施时间表

**第 1-2 周：编译器基础设施（阶段 1）** ✅ (部分完成)
- [x] 扩展 `Ext` AST 结构，添加 `fields` 和 `is_same_module`
- [x] 更新解析器以解析 ext 块中的字段声明
- [ ] 在作用域系统中实现模块跟踪
- [ ] 添加同模块检测逻辑（目前设为 true）
- [x] 编写解析器单元测试

**当前状态**：
- ✅ AST 扩展完成
- ✅ 字段解析完成
- ⏸️ 模块跟踪待实现（暂时假设同模块）
- ✅ 单元测试完成

**交付物**：
- ✅ 接受 `ext { name Type }` 语法的增强解析器
- ✅ 验证字段解析的单元测试

---

**第 3 周：Transpiler 更新（阶段 2）**
- [ ] 实现多文件加载的模块收集器
- [ ] 更新 C transpiler 以合并类型 + ext 字段
- [ ] 添加条件文件加载（.c.at vs .vm.at）
- [ ] 生成合并字段的正确 C 结构体
- [ ] 编写 transpiler 测试

---

**第 4 周：标准库迁移（阶段 3）**
- [ ] 重构 io.at + io.c.at + io.vm.at
- [ ] 重构 sys.at + sys.c.at + sys.vm.at
- [ ] 如果需要，重构 builder.at
- [ ] 更新文档

---

**第 5 周：测试和验证（阶段 4）**
- [ ] 编写全面的单元测试
- [ ] 添加 a2c 集成测试
- [ ] 验证内存布局
- [ ] 测试跨平台编译
- [ ] 性能基准测试

---

## 风险分析和缓解

### 风险 1：破坏现有代码 ✅ (已缓解)
**影响**：高
**概率**：中

**缓解措施**：
- ✅ 为没有字段的 ext 保持向后兼容（`Ext::new()` 仍然可用）
- ⏸️ 对旧模式添加弃用警告（待实现）
- ⏸️ 为现有代码提供迁移指南（待创建）

**当前状态**：
- 662 个测试通过，9 个测试失败
- 失败的测试与标准库兼容性相关
- 需要更新标准库文件以使用新语法

---

### 风险 2：模块系统复杂性
**影响**：中
**概率**：高

**缓解措施**：
- ⏸️ 从简单的模块路径检测开始（基于文件）
- ⏸️ 最初使用相对路径
- ⏸️ 稍后添加完整模块系统
- ⏸️ 关于模块边界的清晰文档

**当前状态**：
- 暂时假设所有 ext 都在同模块中（`is_same_module = true`）
- 这允许我们继续其他工作，稍后再实现完整的模块系统

---

### 风险 3：C Transpiler 边缘情况
**影响**：中
**概率**：中

**缓解措施**：
- ⏸️ 增量 transpiler 更新
- ⏸️ 使用复杂嵌套类型进行测试
- ⏸️ 验证生成的 C 代码可以编译
- ⏸️ 使用 clang/gcc 验证输出

---

### 风险 4：性能下降
**影响**：低
**概率**：低

**缓解措施**：
- ⏸️ 对模块收集阶段进行基准测试
- ⏸️ 缓存合并的类型声明
- ⏸️ 优化字段查找
- ⏸️ 实施前后性能分析

---

## 成功标准

### 阶段 1 成功 ✅ (部分达成)
- [x] 解析器接受 `ext { name Type }` 语法
- [x] 同模块 ext 可以添加字段
- [ ] 添加字段时跨模块 ext 被拒绝（待实现模块跟踪）
- [x] 所有解析器测试通过

### 阶段 2 成功
- [ ] C transpiler 生成合并的结构体
- [ ] 模块收集器加载正确的文件
- [ ] 生成的代码无错误编译
- [ ] 所有 transpiler 测试通过

### 阶段 3 成功
- [ ] io.at 使用新模式
- [ ] sys.at 使用新模式
- [ ] 没有代码重复
- [ ] 文档已更新

### 阶段 4 成功
- [ ] 所有现有测试通过
- [ ] 新添加的测试通过
- [ ] 内存布局已验证
- [ ] 性能可接受

---

## 关键实施文件

### 前 5 个最关键文件：

1. **[`crates/auto-lang/src/parser.rs:1861-2045`](crates/auto-lang/src/parser.rs#L1861-L2045)** ✅ (已完成)
   - ✅ 修改 `parse_ext_stmt()` 以解析字段声明
   - ⏸️ 添加同模块检测逻辑（暂时假设 true）
   - ✅ 实现注解支持

2. **[`crates/auto-lang/src/ast/ext.rs`](crates/auto-lang/src/ast/ext.rs)** ✅ (已完成)
   - ✅ 添加 `fields: Vec<Member>` 字段到 `Ext` 结构体
   - ✅ 添加 `is_same_module: bool` 标志
   - ✅ 更新 Display/ToNode 实现

3. **[`crates/auto-lang/src/trans/c.rs`](crates/auto-lang/src/trans/c.rs)** ⏸️ (待实现)
   - ⏸️ 实现模块收集器
   - ⏸️ 更新结构体生成以合并字段
   - ⏸️ 添加 .c.at/.vm.at 的条件文件加载

4. **[`stdlib/auto/io.at`](stdlib/auto/io.at)** ⏸️ (待重构)
   - ⏸️ 重构为最小公共接口
   - ⏸️ 删除重复的字段声明
   - ⏸️ 添加方法签名的 ext 块

5. **[`stdlib/auto/io.c.at`](stdlib/auto/io.c.at)** ⏸️ (待重构)
   - ⏸️ 添加带有 `_fp *FILE` 私有字段的 ext 块
   - ⏸️ 使用私有字段实现方法
   - ⏸️ 删除重复的类型定义

### 其他重要文件：

- **[`crates/auto-lang/src/scope.rs`](crates/auto-lang/src/scope.rs)** - 添加模块跟踪
- **[`crates/auto-lang/src/ast/types.rs`](crates/auto-lang/src/ast/types.rs)** - 添加 `private_members` 到 TypeDecl
- **[`stdlib/auto/io.vm.at`](stdlib/auto/io.vm.at)** - VM 特定实现
- **[`stdlib/auto/sys.at`](stdlib/auto/sys.at)** - 系统模块重构
- **[`docs/tutorials/stdlib-organization.md`](docs/tutorials/stdlib-organization.md)** - 更新文档

---

## 示例代码

### 完整文件结构示例

**重构前**（当前损坏状态）：
```
stdlib/auto/
├── io.at         (有 path, file 字段)
├── io.c.at       (重复 path, file 字段！)
└── io.vm.at      (仅有 ext 方法)
```

**重构后**：
```
stdlib/auto/
├── io.at         (公共接口：path 字段 + 方法签名)
├── io.c.at       (带有 _fp 字段 + C 实现的 ext)
└── io.vm.at      (带有 _handle 字段 + VM 实现的 ext)
```

### 构造函数语法示例

**AutoLang 代码**：
```auto
ext File {
    _fp *FILE

    static fn open(path str) File {
        let f = fopen(path, c"r")
        return File(
            path: path,    // 公开字段
            _fp: f         // 私有字段（同模块中允许）
        )
    }
}
```

**生成的 C 代码**：
```c
File File_open(char* path) {
    FILE* f = fopen(path, "r");
    File result;
    result.path = path;
    result._fp = f;
    return result;
}
```

### 跨模块扩展示例

**模块 A**（a.at）：
```auto
type Point {
    x int
    y int
}
```

**模块 B**（b.at）：
```auto
use a: Point

ext Point {
    // OK：添加方法（跨模块）
    fn distance() int {
        return .x * .x + .y * .y
    }

    // 错误：不能添加字段（跨模块）
    // _id int  // 这会被解析器拒绝！
}
```

---

## 总结

这个重构计划实现了**清晰的关注点分离**：

1. **公共接口**（.at 文件）- 用户看到的
2. **平台实现**（.c.at/.vm.at 文件）- 如何工作的

关键创新是**带字段添加的同模块 ext**，实现：
- 平台特定的内存布局
- 类型安全的私有字段访问
- 零开销抽象
- 向后兼容

**当前进度**：
- ✅ **第 1-2 周**：编译器基础设施（约 60% 完成）
  - ✅ AST 扩展完成
  - ✅ 字段解析完成
  - ⏸️ 模块跟踪待实现
  - ✅ 单元测试完成

**下一步**：
1. 修复标准库兼容性问题（9 个失败测试）
2. 实现 TypeDecl 的 merge_ext 功能
3. 实现 C transpiler 的结构体合并
4. 更新标准库文件以使用新 ext 语法

**预估工作量**：已完成约 20% 的总工作量
**风险等级**：中（通过增量方法缓解）
**价值**：高（实现干净的多平台标准库）

---

## 参考资料

- [设计文档：stdlib-organization.md](docs/design/stdlib-organization.md)
- [当前组织：stdlib-file-organization.md](docs/stdlib-file-organization.md)
- [教程：stdlib-organization.md](docs/tutorials/stdlib-organization.md)
- [现有 ext 实现：parser.rs:1861-2045](crates/auto-lang/src/parser.rs#L1861-L2045)

---

## 更新日志

**2025-01-19**: 初始计划创建，提交 commit 8129406
- ✅ 扩展 Ext AST 结构
- ✅ 实现字段解析功能
- ✅ 添加注解支持
- ⏸️ 需要修复标准库兼容性（9 个测试失败）
