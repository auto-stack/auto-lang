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

