# RoadMap to v0.2

v0.2 是 AutoLang 的重要基础版本，包含 **479 个 commit**（2024年9月 - 2025年1月），奠定了编译器和生态系统的核心基础。

## 核心语言特性

### 基础语法
- **表达式和语句**
  - 算术运算符 (优先级解析)
  - 一元和前缀运算符
  - if/else 条件语句
  - for 循环 (带初始化器、带索引)
  - when 语句模式匹配

### 类型系统
- **基础类型**
  - `int`, `uint`, `float`, `bool`
  - `byte`, `i8`, `u8`
  - `str`, `cstr`
  - 数组类型 `[]T` 和指针类型 `*T`

- **变量声明**
  - `var` - 动态类型
  - `let` - 不可变绑定
  - `mut` - 可变绑定

- **类型组合**
  - 类型实例化
  - 简单的类型组合

### OOP 系统
- **单继承** (Plan 021)
  - `is` 关键字实现单继承
  - Rust transpiler 继承支持

- **Spec 多态** (Plan 019)
  - Spec trait 系统实现
  - 成员级委托语法
  - 基于多态的方法调用

- **方法系统**
  - 用户类型方法
  - 方法调用类型推断

### 函数式特性
- **Lambda 匿名函数**
  - 匿名函数语法
  - 自动命名 fn 转换

## 编译器架构

### Transpilers
- **C Transpiler (a2c)**
  - 完整的 C 代码生成
  - stdio.h 实现 (Phase 1-2, 4)
  - 布尔类型支持
  - 结构体初始化类型检查
  - void 参数和类型推断

- **Rust Transpiler (a2r)**
  - Rust 代码生成
  - CLI 集成
  - 结构体、枚举支持
  - F-字符串 (FStr) 支持

- **Python Transpiler (a2p)** (Plan 022)
  - Python 代码生成
  - Phases 1-8 实现
  - 结构体、枚举、方法支持

- **JavaScript Transpiler (a2j)** (Plan 023)
  - JavaScript 代码生成
  - CLI 集成 (Phase 10)
  - Phase 1 核心基础设施
  - 9 个测试用例

### AST 和代码生成
- **Atom 宏系统**
  - `value!`, `atom!` 宏
  - Builder Pattern 实现
  - AST 到 Atom 转换
  - Atom/Node/Array/Obj 构造 DSL

### 错误系统 (Plan 008)
- **miette 集成**
  - 源码片段显示
  - 位置信息附加
  - 结构化错误类型
  - IDE 友好的诊断输出
  - 逐步替换 panic! 调用

### 类型推断
- **基础类型推断** (Plan 010)
  - 表达式类型推断
  - 方法调用类型推断
  - 枚举类型推断

## 值系统

### auto-val 独立化
- **Value 类型系统**
  - 独立的 `auto-val` crate
  - `AutoStr` 替代 `String`
  - `Array` 类型 (`Vec<Value>`)
  - `Obj` 对象类型

- **值操作**
  - 算术运算符
  - 对象和数组方法
  - 类型转换器

## 工具链

### AutoGen
- **代码生成器**
  - 新的 auto-gen CLI
  - AST 代码生成
  - 测试用例支持

### AutoShell
- **交互式 Shell**
  - AutoLang REPL 集成
  - 历史记录
  - Tab 补全 (reedline 集成)
  - miette 错误格式化

### 标准库组织
- **模块化**
  - `io.at` - IO 函数
  - `sys.at` - 系统函数
  - 文件组织重构

## Widget/AURA 基础

### 早期探索
- **Widget 语法**
  - widget/model/view 解析
  - 动态视图 (dyna view)
  - 事件处理器 (on 语句)

### 数据绑定
- **Model 和 View**
  - 状态变量
  - 事件绑定
  - Widget 评估

## 测试

### 测试基础设施
- **AtomWriter 测试**
  - AST 输出格式化
  - 测试用例覆盖

### Transpiler 测试
- **a2c 测试** (100+ 测试用例)
- **a2r 测试** (002-012)
- **a2p 测试**
- **a2j 测试** (9 个测试)

## 项目结构

### Crate 组织
- **auto-val** - 值系统独立化
- **auto-lang** - 核心编译器
- **auto-gen** - 代码生成器
- **auto-man** - 项目管理

## 从 v0.1 的主要变化

### 新增功能
1. **多 transpiler 支持** - C, Rust, Python, JavaScript
2. **Spec 多态系统** - trait 和委托
3. **错误报告系统** - miette 集成
4. **AutoShell** - 交互式 REPL
5. **Widget/AURA** - 早期 UI 框架探索

### 重大改进
1. **类型推断** - 基础类型推断子系统
2. **Atom 宏系统** - AST 构造 DSL
3. **标准库组织** - 模块化 stdlib
4. **测试覆盖** - 完善的测试用例

## 下一步 (v0.3)

- [ ] 泛型编程系统
- [ ] 多后端 UI 生成
- [ ] 增量编译 (AIE)
- [ ] BigVM 迁移
- [ ] 异步任务系统


# RoadMap to v0.3

v0.3 是 AutoLang 的重要里程碑版本，包含 **1172 个 commit**，跨越以下主要领域：

## 核心语言特性

### 类型系统增强
- **泛型编程系统** (Plan 048, 057, 058, 059, 062, 067, 076)
  - 泛型类型实例化和单态化
  - 泛型 spec 带类型替换
  - 类型别名语法
  - 泛型字段支持
  - Rust transpiler 泛型支持
  - BigVM 泛型类型擦除和特化

- **Optional/Result 类型** (Plan 027, 049, 120)
  - 早期: 统一的 `May<T>` 类型 (包含 Option 和 Result，用 `?T` 表示)
  - 后期: 拆分为 Rust 风格的 `Option<T>` 和 `Result<T>`
  - `?T` → `Option<T>` (值可能不存在)
  - `!T` → `Result<T, E>` (操作可能失败)
  - `??` 和 `?.` 操作符

- **借用检查系统** (Plan 024, 026, 034, 038)
  - 所有权优先实现
  - 属性关键字: `.view`, `.mut`, `.take`
  - 借用表达式: `hold`, `view`, `mut`, `take`
  - `str_slice` 类型

### 数据结构
- **动态集合类型** (Plan 041, 042, 043, 051, 052)
  - `List<T>` 动态数组类型
  - `DStr` 动态字符串
  - `Slice<T>` 切片类型
  - 迭代器系统: `map`, `filter`, `reduce`, `collect`
  - 存储抽象: `List<T, Storage>` (Heap/InlineInt64)

### OOP 系统
- **扩展语句** (Plan 035, 044)
  - `ext` 语句用于多平台类型扩展
  - 私有字段添加

- **方法调用** (Plan 038, 025)
  - VM 方法调用表达式
  - 字符串方法库

### 函数式特性
- **闭包系统** (Plan 060, 061, 071)
  - 闭包语法和变量捕获
  - `#[with(...)]` 语法
  - BigVM 闭包支持

## 编译器架构

### Auto Incremental Engine (AIE)
- **AIE 架构迁移** (Plan 063, 064, 065)
  - 编译时与运行时分离
  - Database 持久化
  - QueryEngine 智能缓存
  - 熔断机制
  - 增量 transpilation

### BigVM 迁移
- **BigVM 实现** (Plan 068, 069, 070, 071, 073, 076, 087)
  - 全局变量支持
  - 迭代器系统
  - 闭包环境加载
  - 泛型类型支持
  - 测试迁移

### 注解系统
- **Rust 风格注解** (Plans)
  - `#[c]`, `#[vm]`, `#[pub]`, `#[primary]`
  - 函数和类型注解

## UI 生成系统

### AURA Widget 系统
- **Widget 库** (Plan 140, 143)
  - 60+ shadcn-vue 组件
  - WidgetRegistry 统一组件查找
  - 组件分类: layout, form, display, navigation, overlay, feedback

### 多后端支持
- **Vue 生成器** (Plan 099, 104, 135)
  - shadcn-vue 组件生成
  - Tailwind CSS 支持
  - 增量编译缓存

- **Jetpack Compose (a2jet)** (Plan 113, 133, 134, 136, 145)
  - Material3 组件映射
  - Form, Layout, List, Navigation 生成
  - 完整项目生成

- **ArkTS/HarmonyOS (a2ark)** (Plan 137, 138, 142)
  - ArkTS 代码生成
  - @Component, @State, @Prop 装饰器
  - Tabs 模式支持
  - 生命周期支持

- **Tauri 桌面应用** (Plan 151)
  - Tauri IPC 模式
  - a2r 全局状态支持

### 项目结构
- **工作区和场景** (Plan 129, 130)
  - `pac.at` 工作区配置
  - 多前端支持
  - 统一后端输出

## 标准库

### IO 和文件系统
- **File 类型** (Plan 020, 036)
  - `File.read_all()`, `File.write_lines()`, `File.read_line()`, `File.flush()`
  - C FFI 支持和 CStr 类型
  - 标准库文件组织

### 字符串操作
- **字符串方法** (Plan 025)
  - `split()`, `lines()`, `words()` 等方法

## 工具链

### AutoMan
- **项目管理** (Plan 078, 079, 093)
  - 统一项目生成
  - 多后端构建支持
  - Rust 后端集成

- **模块系统** (Plan 131, 074)
  - `pac` 和 `super` 前缀
  - 多目录搜索

### Shell
- **ASH (Auto Shell)** (Plan 017, 047)
  - 结构化命令输出
  - 管道和值操作
  - 系统命令: `ps`, `sys`
  - 文件操作: `cp`, `mv`, `rm`, `mkdir`

### API 代码生成
- **API 示例** (Plan 132)
  - TypeScript 客户端生成
  - Rust 服务器生成
  - CORS 支持

## 并发和异步

### 任务系统
- **AutoVM Task/Msg** (Plan 069, 121, 127)
  - 异步并发框架
  - 任务生成和消息传递
  - 调度器守护进程

- **微并发** (Plan 126)
  - `.go` 后缀操作符
  - `~T` 异步类型语法

## 开发体验

### 错误系统
- **错误报告** (Plan 008, 009, 120)
  - miette 集成
  - 源码片段显示
  - 类型错误 Option/Result

### IDE 支持
- **语言服务器**
  - LSP 基础架构

## 测试

### 测试迁移
- **BigVM 测试迁移** (Plan 039, 073, 085)
  - 551+ 测试通过
  - Category A-D 迁移

## 文档

### 设计文档
- 152 个计划文档 (Plans 001-151)
- 实现总结和架构文档

## 从 v0.2 的主要变化

### 新增功能
1. **泛型编程系统** - 完整的泛型支持
2. **多后端 UI 生成** - Vue, Jetpack Compose, ArkTS, Tauri
3. **增量编译** - AIE 架构
4. **Widget 库** - 60+ 组件
5. **异步任务系统** - Task/Msg 框架

### 重大改进
1. **BigVM** - 替代 AutoVM 作为默认 VM
2. **借用检查** - 所有权系统
3. **模块系统** - pac/super 前缀
4. **Shell** - 结构化命令
5. **工具链** - AutoMan 统一管理

## 下一步 (v0.4+)

- [ ] Plan 146 ASH SmartCmd 集成完成
- [ ] LSP 完整实现
- [ ] 自举编译器 (auto/)
- [ ] Python/JavaScript transpiler
- [ ] 模式匹配系统
- [ ] async/await 语法


# RoadMap to v0.1

## v0.1 Features

### Auto Lang

- Refs and Pointers
- Basic Memory Management
- Component based OOP
- Specs
- Basic Async/Concurrent Design
- Compile-Time Scripting
- Transpilers
    - C transpiler
    - Python transpiler
    - Rust transpiler

### Auto Lib

- Multi-Lang Design
    - Interpreter Std
    - C Std
    - Rust Std
    - Python Std
- Basic Libs
    - Number/Math
    - String
    - File
    - Net
    - Task

### Auto Man

- Node Specs
- Fluent DSL
- Ninja backend
- direct toolchain support like Makefile
- Binary Deps
- Install/Flashing
- Cross OS App Package Management (like apt/winget)


### Auto Shell

- Integration with Nushell
- Command completion system
- History/Bookmark management
- Graphing/Charting
- Integration with other Auto Tools
- Inline and multi-line editor
- Git integration
- AI Agent like warp

### Auto UI

- DSL based on Auto Lang
- multi-framework support
    - GPUI + GPUI_Components
    - Iced
    - SDL3
    - Harmony UI
- Widget lib
- Editor
- Charting
- Notebook

