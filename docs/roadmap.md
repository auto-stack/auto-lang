# RoadMap to v0.4

## 核心语言特性

### 类型系统

- 增强类型推导
- 强化spec
    - 默认方法实现
    - spec的组合
    - 判定式spec
- 增强闭包类型
- 类型作为第一公民

### 生命周期

- 强化生命周期检查器
- 实现AutoFree
- 逃逸分析+ARC Fallback
- 二阶逃逸分析


### 解释器

总体目标：实现1s周转

- ABC文本形式汇编代码
- 实时反汇编器
- AutoVM拆分成Host和Client
- ClientVM实现C的最小版本，部署到MCU
- 热重载，包含数据迁移功能


### Rust生态

- 完善全部Auto语法对Rust的转译
- 实现r2a反转译器
- 实现20+常用标准库的Rust转译
- 开启Rust常用库的Auto反迁移工作
- 实现WEB/Desktop常用的后端技术库（HTTP/Redis/SQLite等）

### C生态

- 完善全部Auto语法对C的转译
- 完善宏和其他的预处理系统
- 完善CTE与C语言的无缝连接
- 接管Linker，实现用Auto配置链接属性
- 对接Debugger（OpenOCD）
- 链接热重载
- 基于SDL3实现基础的图形/小游戏框架

### WEB生态

- 完善Vue生态；添加对shadcn之外另一个常见组件库的支持
- 支持React生态；支持shadcn和另一个常见组件库
- 初步支持最前沿WEB框架中的一个（例如Svelte）
- 支持Responsive Layout
- 支持常见Blocks
- 实现AutoRead综合Demo

### Desktop生态

- 实现iced/gpui的widget gallery
- 实现“元件市场”
- 实现“虚拟桌面”
- 实现“元件启动器”
- 实现“AI助手”
- 实现AutoRead综合Demo

### Python生态

- 完善Python语法支持
- 支持Numpy/Pandas
- 支持Pytorch

### Agent生态

- **Agent 基础设施**
  - Agent 运行时环境
  - Agent 通信协议
  - Agent 注册和发现

- **AI 集成**
  - LLM API 集成
  - Tool use 框架
  - Prompt 管理系统

## 开发工具

### LSP (Language Server Protocol)
- **完整的 LSP 实现**
  - 代码补全和智能提示
  - 跳转定义和查找引用
  - 悬停文档和类型信息
  - 诊断和错误修复
  - 代码格式化
  - 符号搜索和项目浏览
  - 重构支持

### 包管理
- **包注册表**
  - 中央包仓库
  - 版本语义化
  - 依赖解析
  - 包发布工具

- **本地包管理**
  - `auto get/install` 命令
  - 依赖锁文件
  - 虚拟工作空间

### 调试工具
- **断点调试器**
  - 断点设置和管理
  - 变量检查和监视
  - 调用堆栈查看
  - 单步执行

- **性能分析**
  - CPU/内存 性能分析
  - 火焰图生成
  - 内存泄漏检测
  - 热点路径分析

## 测试和质量

### 测试框架
- **单元测试增强**
  - 测试覆盖率报告
  - 测试夹具 (fixtures)
  - 参数化测试
  - 模糊测试

- **集成测试**
  - 端到端测试
  - 基准测试
  - 压力测试

## 文档和示例

### 语言参考
- **完整语法文档**
  - 语言规范
  - API 参考手册
  - 最佳实践指南

### 示例项目
- **示例库**
  - 完整的应用示例
  - 教程和指南
  - 视频教程

## 从 v0.3 的主要变化

### 新增功能
1. **模式匹配系统** - 完整的 match 表达式和解构模式
2. **async/await 语法** - 原生异步语法
3. **LSP 完整实现** - IDE 级开发体验
4. **包管理系统** - 第三方包生态
5. **调试工具** - 断点调试和性能分析

### 重大改进
1. **性能优化** - 1s 周转目标
2. **生命周期** - AutoFree 和逃逸分析
3. **Spec 增强** - 默认实现、组合、判定式
4. **生态扩展** - 更多 transpiler 和组件库
5. **Agent 支持** - AI Agent 基础设施

## 下一步 (v0.5+)

- [ ] 自举编译器 (auto/)
- [ ] JIT 编译
- [ ] 高级优化
- [ ] 企业级功能

# RoadMap to v0.3

v0.3 是 AutoLang 的重要里程碑版本，包含 **1172 个 commit**，跨越以下主要领域：

## 核心语言特性

### 类型系统增强
- **泛型编程系统** (Plan 47, 057, 058, 059, 062, 067, 076)
  - 泛型类型实例化和单态化
  - 泛型 spec 带类型替换
  - 类型别名语法
  - 泛型字段支持
  - Rust transpiler 泛型支持
  - BigVM 泛型类型擦除和特化

- **Optional/Result 类型** (Plan 026, 049, 120)
  - 早期: 统一的 `May<T>` 类型 (包含 Option 和 Result，用 `?T` 表示)
  - 后期: 拆分为 Rust 风格的 `Option<T>` 和 `Result<T>`
  - `?T` → `Option<T>` (值可能不存在)
  - `!T` → `Result<T, E>` (操作可能失败)
  - `??` 和 `?.` 操作符

- **借用检查系统** (Plan 023, 026, 034, 038)
  - 所有权优先实现
  - 属性关键字: `.view`, `.mut`, `.take`
  - 借用表达式: `hold`, `view`, `mut`, `take`
  - `str_slice` 类型

### 数据结构
- **动态集合类型** (Plan 040, 042, 043, 051, 052)
  - `List<T>` 动态数组类型
  - `DStr` 动态字符串
  - `Slice<T>` 切片类型
  - 迭代器系统: `map`, `filter`, `reduce`, `collect`
  - 存储抽象: `List<T, Storage>` (Heap/InlineInt63)

### OOP 系统
- **扩展语句** (Plan 034, 044)
  - `ext` 语句用于多平台类型扩展
  - 私有字段添加

- **方法调用** (Plan 37, 025)
  - VM 方法调用表达式
  - 字符串方法库

### 函数式特性
- **闭包系统** (Plan 057, 061, 071)
  - 闭包语法和变量捕获
  - `#[with(...)]` 语法
  - BigVM 闭包支持

## 编译器架构

### Auto Incremental Engine (AIE)
- **AIE 架构迁移** (Plan 062, 064, 065)
  - 编译时与运行时分离
  - Database 持久化
  - QueryEngine 智能缓存
  - 熔断机制
  - 增量 transpilation

### BigVM 迁移
- **BigVM 实现** (Plan 67, 069, 070, 071, 073, 076, 087)
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
- **Widget 库** (Plan 139, 143)
  - 59+ shadcn-vue 组件
  - WidgetRegistry 统一组件查找
  - 组件分类: layout, form, display, navigation, overlay, feedback

### 多后端支持
- **Vue 生成器** (Plan 98, 104, 135)
  - shadcn-vue 组件生成
  - Tailwind CSS 支持
  - 增量编译缓存

- **Jetpack Compose (a1jet)** (Plan 113, 133, 134, 136, 145)
  - Material2 组件映射
  - Form, Layout, List, Navigation 生成
  - 完整项目生成

- **ArkTS/HarmonyOS (a1ark)** (Plan 137, 138, 142)
  - ArkTS 代码生成
  - @Component, @State, @Prop 装饰器
  - Tabs 模式支持
  - 生命周期支持

- **Tauri 桌面应用** (Plan 150)
  - Tauri IPC 模式
  - a1r 全局状态支持

### 项目结构
- **工作区和场景** (Plan 128, 130)
  - `pac.at` 工作区配置
  - 多前端支持
  - 统一后端输出

## 标准库

### IO 和文件系统
- **File 类型** (Plan 017, 036)
  - `File.read_all()`, `File.write_lines()`, `File.read_line()`, `File.flush()`
  - C FFI 支持和 CStr 类型
  - 标准库文件组织

### 字符串操作
- **字符串方法** (Plan 024)
  - `split()`, `lines()`, `words()` 等方法

## 工具链

### AutoMan
- **项目管理** (Plan 77, 079, 093)
  - 统一项目生成
  - 多后端构建支持
  - Rust 后端集成

- **模块系统** (Plan 130, 074)
  - `pac` 和 `super` 前缀
  - 多目录搜索

### Shell
- **ASH (Auto Shell)** (Plan 016, 047)
  - 结构化命令输出
  - 管道和值操作
  - 系统命令: `ps`, `sys`
  - 文件操作: `cp`, `mv`, `rm`, `mkdir`

### API 代码生成
- **API 示例** (Plan 131)
  - TypeScript 客户端生成
  - Rust 服务器生成
  - CORS 支持

## 并发和异步

### 任务系统
- **AutoVM Task/Msg** (Plan 68, 121, 127)
  - 异步并发框架
  - 任务生成和消息传递
  - 调度器守护进程

- **微并发** (Plan 125)
  - `.go` 后缀操作符
  - `~T` 异步类型语法

## 开发体验

### 错误系统
- **错误报告** (Plan 7, 009, 120)
  - miette 集成
  - 源码片段显示
  - 类型错误 Option/Result

### IDE 支持
- **语言服务器**
  - LSP 基础架构

## 测试

### 测试迁移
- **BigVM 测试迁移** (Plan 38, 073, 085)
  - 550+ 测试通过
  - Category A-D 迁移

## 文档

### 设计文档
- 151 个计划文档 (Plans 001-151)
- 实现总结和架构文档

## 从 v-1.2 的主要变化

### 新增功能
0. **泛型编程系统** - 完整的泛型支持
1. **多后端 UI 生成** - Vue, Jetpack Compose, ArkTS, Tauri
2. **增量编译** - AIE 架构
3. **Widget 库** - 60+ 组件
4. **异步任务系统** - Task/Msg 框架

### 重大改进
0. **BigVM** - 替代 AutoVM 作为默认 VM
1. **借用检查** - 所有权系统
2. **模块系统** - pac/super 前缀
3. **Shell** - 结构化命令
4. **工具链** - AutoMan 统一管理

## 下一步 (v-1.4+)

- [ ] Plan 145 ASH SmartCmd 集成完成
- [ ] LSP 完整实现
- [ ] 自举编译器 (auto/)
- [ ] Python/JavaScript transpiler
- [ ] 模式匹配系统
- [ ] async/await 语法




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

