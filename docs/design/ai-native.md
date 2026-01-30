这是一个关于将 **Auto 语言**确立为 **AI 原生中间语言 (AI-Native Intermediate Language)** 的战略设计文档。该文档整合了我们之前的讨论，将 Auto 的定位从单纯的“系统级编程语言”提升为“人机共生的语义锚点”。

---

# Design Document: Auto as an AI-Native Language

**Version**: 1.0
**Status**: Draft
**Target Audience**: Compiler Team, AI Tooling Team, Early Adopters

## 1. 愿景与定位 (Vision & Mission)

在 AI 生成代码日益普及的时代，编程语言的角色正在发生转变。Auto 不仅是为人类工程师设计的工具，更是为 AI 智能体（LLM Agents）设计的 **“思维载体”** 和 **“执行基石”**。

**核心定位**：

> **Auto 是 AI 时代的“意图 IR” (Intent Intermediate Representation)。**

它旨在解决当前 AI 编程的两大痛点：

1. **C/C++ 的语义丢失**：只有底层实现，缺乏高层意图，AI 难以理解和维护。
2. **Python 的约束缺失**：只有模糊意图，缺乏实现约束，AI 容易产生幻觉或生成低效代码。

Auto 通过 **“显式意图 + 强类型约束 + 编译器验证”** 的铁三角，成为连接人类需求与底层机器代码的最佳桥梁。

---

## 2. 核心设计原则 (Core Principles)

### 2.1 意图显式化 (Explicit Intent)

代码结构必须直接反映逻辑流，减少 AI 推理负担。

* **Auto Flow (`|>`)**: 直接映射思维链 (Chain of Thought)。
* **Comptime (`#`)**: 显式分离编译期逻辑与运行时逻辑，消除环境上下文歧义。

### 2.2 约束即 Prompt (Constraints as Prompts)

利用类型系统和契约作为对 AI 的“强提示词 (Hard Prompts)”，迫使 AI 生成符合规范的代码。

* **Design by Contract**: 通过标注表达输入输出约束。
* **Strong Typing**: 消除类型幻觉。

### 2.3 错误即反馈 (Errors as Feedback)

编译器不仅是检查器，更是 AI 的“教练”。

* **Structured Diagnostics**: 提供机器可读的错误信息，支持 AI 自我修正循环。

---

## 3. 语言特性演进 (Feature Evolution)

为了适应 AI-Native 的目标，Auto 语言特性需做以下针对性增强：

### 3.1 契约编程系统 (Contract System via Annotations)

利用 Auto 的标注系统，将逻辑约束标准化，使其成为 AST 的一部分，供 AI 读取和遵守。

* **前置条件 (`pre`)**: 输入参数约束。
* **后置条件 (`post`)**: 返回值承诺。
* **不变式 (`invariant`)**: 状态一致性保证。
* **示例 (`example`)**: Few-shot learning 的内嵌示例。

```auto
fn resize_image(img: Image, width: int, height: int) -> Result<Image>
    #[pre(width > 0 && height > 0)]        // 告诉 AI：别生成负数尺寸的处理逻辑
    #[post(return.width == width)]         // 告诉 AI：确保结果符合预期
    #[example(img, 100, 100 => valid_img)] // 给 AI 的参考用例
{
    // Implementation...
}

```

### 3.2 局部未知与填空 (`??` Typed Holes)

支持不完整的代码编译，允许 AI 进行渐进式生成（In-filling）。

* **语法**: `??`
* **编译器行为**:
* 不报错，而是推导出 `??` 所需的**类型上下文 (Type Context)**。
* LSP/编译器返回：“这里需要一个 `fn(User) -> bool` 类型的表达式”。



```auto
let active_users = all_users 
    |> filter(u => u.is_active)
    |> map(??) // AI 暂停在这里，LSP 提示：需要 map 的映射函数

```

### 3.3 文档即数据 (Docs as Data)

将文档注释 (`///`) 提升为 AST 的一等公民，不再在解析阶段丢弃。

* **目的**: 让 AI 在读取 AIR (Auto IR) 时，能同时获取代码逻辑和自然语言描述（RAG 增强）。
* **实现**: 解析为 `DocAttribute`，随库文件 (`.alib`) 分发。

### 3.4 元指令 (Meta-Instructions)

引入 `#!` 语法，作为给 AI 生成器的“影子指令”，不影响运行时逻辑，但指导生成策略。

```auto
fn sort_large_dataset(data: []int) {
    #! optimize: memory   // 指令：生成省内存的代码
    #! algo: stable       // 指令：必须是稳定排序
    
    // AI 将根据上述指令生成具体的排序实现
}

```

---

## 4. 编译器与工具链架构 (Toolchain Architecture)

### 4.1 多模态错误报告 (Multimodal Diagnostics)

编译器 `stderr` 输出不再仅是文本，而是根据环境自适应。

* **Mode A: Human (Console)**
* 使用 Miette 风格，漂亮的 ASCII Art，高亮错误位置，提供修复建议。


* **Mode B: AI (JSON/Atom)**
* 输出极简的结构化数据，节省 Token，包含错误码、AST 节点路径、预期类型。



```javascript
// Atom 格式示例 (比 JSON 更省 Token)
{err: "TypeMismatch", line: 42, exp: "u32", got: "i32", fix: "as u32"}

```

### 4.2 验证闭环 (Verification Loop)

建立 **"Generate -> Compile -> Test -> Refine"** 的自动化闭环。

1. **AI 生成**: 根据 Prompt 生成 Auto 代码。
2. **编译器检查**: 语法、类型、生命周期检查。
3. **契约验证**: 运行 `#[test]` 和 `#[pre/post]` 检查。
4. **反馈**: 将结构化错误回传给 AI。
5. **修正**: AI 根据反馈生成 Patch。

---

## 5. 生态发展路线图 (Ecosystem Roadmap)

### Phase 1: 基础建设 (Foundation)

* **Parser 升级**: 支持 `#[pre]`, `#[post]`, `??`, `#!` 等新语法。
* **AIR 增强**: 确保 AST/AIR 可以序列化并保留所有元数据（文档、契约）。
* **Diagnostics 改造**: 实现 Atom 格式的错误输出。

### Phase 2: 数据合成 (Synthetic Data Factory)

* **FFI Generator**: 利用 Clang/Bindgen 生成 C/Rust 的 Auto 接口，确保基础库可用。
* **AI Rewrite Lab**:
* 选取 GitHub 高星 C/Rust 算法库。
* 使用 GPT-4/Claude-3.5 配合 Auto Spec，将其**重写**为地道的 Auto 代码。
* 通过编译器自动验证，构建数百万行的高质量训练数据集。



### Phase 3: AI 辅助工具 (AI Tooling)

* **Auto Copilot**: 基于训练数据微调的小模型，专门补全 Auto 代码。
* **Auto Doctor**: 一个基于 AI 的静态分析工具，利用 `require/ensure` 自动查找逻辑 Bug。

---

## 6. 总结 (Conclusion)

通过将 **Auto** 定义为 AI 原生语言，我们不仅创造了一个更好的系统编程工具，更在 **人** 与 **超级智能** 之间建立了一种通用的、严谨的、可验证的沟通协议。

* **对于人类**：它是表达意图的高效速记法（Flow, Comptime）。
* **对于 AI**：它是包含约束的结构化思维链。
* **对于机器**：它是高性能的底层指令。

Auto 语言将成为 AI 时代软件工程的 **"Truth Anchor" (真理锚点)**。