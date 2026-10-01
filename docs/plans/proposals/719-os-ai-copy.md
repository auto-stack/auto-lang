# PLAN-719 · OS / AI 网站介绍稿

> 用途：中英文网站文案与页面结构审阅稿。尚未写入网站；不构成新运行时功能或发布承诺。
> 当前能力核对日期：2026-10-01。页面落地后在“当前进展”旁保留该日期，不把阅读日期当作验证日期。

## 页面分工与呈现

- `/os`：解释 AutoOS 是什么、LaOS 与 OS over OS 的关系、组成与运行形态、当前进展、长期方向。
- `/autoos/`：实际桌面体验与项目入口；复用总览中的能力状态，展示现有真实桌面/设置/应用截图。详情解释窗口管理、配置与应用，不再重复一整套愿景。
- `/ai`：解释 AI 在体系中的位置、应用/Agent/服务的关系、已有接入和长期的人本方向。
- `/articles/autoos-history` 与 `/articles/auto-ai-history`：独立的历史/展望长文，正文含项目来源和资料时点，由对应总览提供阅读入口；均有中文镜像。
- `/ui-desktop`：解释 AutoUI 的 Vue 与原生桌面运行方式，更新“即将推出”的过时描述；移动/其他生态只按核实的状态链接出去。
- 中文路由与上述一一对应。首页两张卡片只做摘要；v0.5 页仅对 OS/AI 相关的架构、现状和未来路线作必要同步，不重编其他发布内容。
- UI：小型介绍首屏 → 真实桌面图/架构关系 → 分节说明 → 状态表 → 文档入口。取消无依据的统计数字、无限符号和促销 CTA。理念段落默认可读，细节可折叠；状态必须有文字，不只靠颜色。

## 中文 · OS 总览

### AutoOS：以 Auto 语言为基础的操作系统环境

AutoOS 正在构建一套跨平台的系统环境，包含桌面、应用、配置、命令接口和系统服务。当前可见的形态是运行在宿主系统中的虚拟桌面；长期方向是让这些组件根据设备与系统环境组合起来，逐步形成可以独立部署的操作系统。

[了解当前桌面](/zh/autoos/) · [阅读桌面架构](/zh/docs/design/autoui/virtual-desktop)

深入阅读：[AutoOS：从 Language as OS 到 AI、语言与系统的协作](/zh/articles/autoos-history)。

### 从 Language as OS 到 OS over OS

**Language as OS（LaOS）** 是项目最初的实现理念：用 Auto 组织和实现操作系统的各个部件，让语言及其运行时适配不同的软硬件环境，同时维持对外接口和使用体验的一致性。

这包括 UI 库、合成器、系统应用、配置、启动器、监控、命令 Shell、AI 服务和桌面；内核也是长期设想的一部分，但目前尚未实现自有内核。现阶段的语言运行时与底层系统设施主要建立在 Rust 和既有生态上。未来也可能探索语言底座的自实现原生路径。

**OS over OS** 从 AutoOS 的使用形态描述同一方向：AutoOS 作为虚拟桌面与宿主桌面并行，自带应用在自己的桌面中运行，同时逐步对接宿主应用与系统能力。它是 LaOS 在当前阶段的产品形态，两者承接同一套架构。

### 系统如何组成

| 部分 | 作用 |
|---|---|
| AutoLang 与运行时 | 提供语言、执行、代码生成及系统能力的接入方式 |
| AutoUI 与渲染/合成宿主 | 描述组件和界面，适配浏览器与原生渲染环境 |
| 桌面与窗口管理 | 组织应用窗口、工作区、任务栏、启动入口与通知 |
| 配置与系统服务 | 管理共享设置、资源与应用间协作所需的能力 |
| 系统应用与工作工具 | 承载终端、编辑、监控、看板和知识工作等实际任务 |
| 平台适配层 | 对接宿主窗口系统、进程、文件、设备与既有内核 |

`auto-lang` 是语言与框架的实现根；`auto-os` 是桌面表面、系统应用与产品集成的组织根。配置、Shell、终端、AI 与工作应用分布在各自项目中，由共同的接口和集成规则连接。

Client/Daemon 是其中共享资源与服务的一种组织方式，并非所有组件都必须独立为 Daemon。UI 定义、窗口语义与平台渲染/合成之间也有各自的边界。

### 在不同环境中运行

**宿主中的虚拟桌面**是当前主要形态。它复用宿主的内核、驱动和系统能力，在一个桌面环境中运行 AutoOS 应用，并与宿主桌面共存。宿主应用互操作按平台推进，具体支持取决于窗口系统与接入方式。

**复用现有内核的独立系统**是后续方向。在条件成熟的设备上，AutoOS 可以提供桌面和系统外壳，对接 Linux 内核及必要的系统服务，形成独立发行版；与 OpenHarmony 结合也是长期路线之一。Linux 合成宿主已有早期实现基础，这些独立产品形态尚未完成。

**自有内核的完整系统**属于更长期的研究方向，目前没有自有内核，也没有由它构成的独立 OS。

这些形态共享语言、组件与接口设计，具体平台适配和部署方式仍需分别实现。“一套代码适配不同生态”是持续推进的目标；当前支持按已实现、已验证的环境说明。

### 当前进展

| 领域 | 当前状态 |
|---|---|
| UI 与桌面 | 已有 Vue 与原生桌面运行路径、虚拟窗口、工作区、启动器及桌面表面；各功能仍有具体边界 |
| 应用与配置 | 已有系统应用和独立工具的集成清单；设置中心使用 Auto 源码组织 UI，并支持 Web/桌面形态 |
| Shell 与终端 | AutoShell 提供跨平台结构化命令与脚本接口；AutoTerm 提供终端基础设施。常用命令采用 POSIX 风格接口，具体兼容范围按命令说明 |
| AI 服务 | 已有共享模型服务、客户端与 Agent 应用的接入；应用按需要逐步接入 AI |
| 宿主互操作 | 已有针对具体平台的原生窗口与剪贴板等接入；支持范围按平台和功能核实 |
| 独立系统与内核 | Linux 合成宿主处于早期实现阶段；完整发行版、OpenHarmony 独立系统和自有内核仍属未来方向 |

跨平台一致性覆盖组件、命令、服务接口、通知、启动与通信的设计目标；当前覆盖需要逐项实现和验证。UI 的一致性主要关注布局、交互和主题，平台文本渲染等差异仍会存在。

### 长期方向：AI + Lang + OS

AutoOS 的长期目标是一套面向人的知识管理与工作生活环境。AI 深入参与了 Auto 的开发，也将作为系统服务和应用协作能力继续融入其中；AutoLang 提供可表达、可执行的实现底座；OS 与应用则承载实际的知识、任务和工作过程。

已有编辑器、知识工具、看板、Shell 和 Agent 应用提供了不同入口。让个人知识、任务与应用中的操作形成更连贯的体验，仍是需要继续建设的方向。这一目标不预设所有任务都交给 AI，而是让人能够理解、选择和控制系统提供的能力。

## 中文 · AI 总览

### AI：Auto 体系中的共享能力与工作工具

AI 参与了 Auto 的开发，也正在成为应用可以接入的系统能力。当前建设集中在共享模型服务、Agent 执行与开发工具；长期方向是让 AI 与语言、桌面和应用共同服务于人的知识管理和工作生活。

[了解 AutoMusk](/zh/apps/automusk/) · [了解 AutoShell](/zh/apps/autoshell/) · [阅读 AutoOS 总览](/zh/os)

深入阅读：[Auto 生态中的 AI：从共享模型服务到可追溯的工作过程](/zh/articles/auto-ai-history)。

### 为什么放在系统层

多个应用使用模型时，会遇到相同的问题：连接哪个提供商、如何配置模型与密钥、如何控制并发、如何查看用量。AutoAI 将这些公共能力放到共享服务中，让接入的应用专注于自己的任务。

这种组织方式也使 AI 能力不局限于某一个聊天界面：命令 Shell、开发工具以及后续知识应用可以复用相同基础设施。各应用按自己的任务和工具逐步接入。

### 应用、Agent 与服务的关系

| 层次 | 当前职责 |
|---|---|
| 应用与用户界面 | 表达用户任务、呈现结果与进度，提供操作和确认入口 |
| Agent 执行层 | 组织角色、技能、工具调用与工作流 |
| 共享客户端 | 向模型服务发送统一请求、接收响应 |
| `aaid` 模型服务 | 管理已配置提供商的接入、并发、路由与用量记录 |
| 配置服务 | 管理应用、角色、技能和模型配置 |

典型接入路径为“应用 → Agent/客户端 → aaid → 已配置的模型提供商”。工具执行发生在相应应用或 Agent 工具层，模型 Daemon 专注于模型通信与资源协调。

### 已有的应用与接入

- **AutoMusk**：开发工作工具，组织聊天、工具活动以及 Plan、Specs、Wiki 等项目材料。规划、执行、复核与知识沉淀分阶段进行，并保留人的确认环节。其实现结合 Auto 源码、生成代码与 Rust 基础设施。
- **AutoAI CLI 与 AutoShell**：提供终端中的 Agent 与 AI 交互入口。AutoAI CLI 的命令执行优先使用 AutoShell；执行失败与策略拒绝有明确的处理规则。
- **配置中心**：管理角色、技能、应用和模型设置，使相关工具复用同一套配置入口。

各接入点的能力由自身工具与工作流决定，任务执行保留相应的人机协作与确认环节。

### 当前能力与边界

| 领域 | 当前能力与边界 |
|---|---|
| 模型接入 | 对已配置且有适配实现的提供商发送请求；可用模型取决于配置、凭据与提供商服务 |
| 请求协调 | 共享并发管理、模型路由和用量记录；路由遵循配置与候选规则，不保证自动选择“最佳模型” |
| Agent 工具 | 角色、技能、工具调用与工作流已有实现，具体能力由应用的接入和工具配置决定 |
| 系统集成 | 已有开发工具、Shell 与配置入口；跨应用知识协作和更完整的个人工作环境仍在建设 |

模型网关与命令执行、文件操作等工具是不同层次。界面应让人看清任务、状态、输出、验证结果和需要确认的操作。

### 从开发工具走向知识与工作环境

长期的 AI + Lang + OS 将把语言、系统服务和应用作为相互衔接的基础：用语言描述数据与操作，通过服务连接能力，在应用中保留知识、任务和工作成果。

这是长期产品方向。现有工具提供了部分基础，但统一的个人知识体系、跨应用长期协作与更广的工作生活场景，还需要明确的数据、权限和交互设计，并逐步实现。

## English · OS overview

### AutoOS: an operating environment built around Auto

AutoOS is developing a cross-platform system environment spanning the desktop, applications, configuration, command interfaces, and services. Its current visible form is a virtual desktop running within a host system. The longer-term direction is to assemble these components for different devices and system environments, including independently deployable operating systems.

[Explore the current desktop](/autoos/) · [Read the desktop architecture](/docs/design/autoui/virtual-desktop)

Further reading: [AutoOS — From Language as OS to AI, Language, and System Collaboration](/articles/autoos-history).

### From Language as OS to OS over OS

**Language as OS (LaOS)** is the original implementation idea: use Auto to organize and implement system components, adapting the language and runtime to different software and hardware environments while maintaining consistent public interfaces and user experiences.

The scope includes UI libraries, compositors, system applications, configuration, launchers, monitoring, command shells, AI services, and the desktop. A kernel is also part of the long-term vision, but an AutoOS kernel has not been implemented. The current runtime and underlying infrastructure rely substantially on Rust and existing ecosystems. A self-implemented native language foundation may also be explored in the future.

**OS over OS** describes the same direction from the product's current operating model. AutoOS runs alongside the host desktop, with its own applications inside a virtual desktop and platform-specific integration with host applications and services. It is the current product form of LaOS, carrying forward the same architecture.

### How the system is organized

| Part | Responsibility |
|---|---|
| AutoLang and runtime | Language, execution, code generation, and access to system capabilities |
| AutoUI and rendering/composition hosts | Component definitions and interfaces for browser and native environments |
| Desktop and window management | Application windows, workspaces, taskbar, launch entry points, and notifications |
| Configuration and system services | Shared settings, resources, and capabilities for application coordination |
| System applications and work tools | Terminal, editing, monitoring, boards, and knowledge work |
| Platform adapters | Host window systems, processes, files, devices, and existing kernels |

`auto-lang` owns the language and framework implementation. `auto-os` organizes the desktop surfaces, system applications, and product integration. Configuration, shell, terminal, AI, and work tools live in their respective projects and connect through shared interfaces and integration rules.

Client/Daemon is one pattern for shared resources and services; it does not require every component to be a separate daemon. UI definitions, window semantics, and platform rendering/composition also have distinct boundaries.

### Operating forms

**A virtual desktop within a host system** is the main current form. It reuses the host kernel, drivers, and system capabilities, runs AutoOS applications in its own desktop, and coexists with the host desktop. Host interoperability is developed per platform, with support depending on the window system and integration method.

**An independent system using an existing kernel** is a future direction. On suitable devices, AutoOS could provide the desktop and system shell, connect to the Linux kernel and required services, and form an independent distribution. OpenHarmony integration is another long-term path. Early Linux composition-host work exists; these complete product forms have not yet been delivered.

**A complete system with its own kernel** is a longer-term research direction. No AutoOS kernel or independent OS based on it is currently available.

These forms share language, component, and interface design. Platform integration and deployment still require separate implementation work. One codebase adapting to different ecosystems is a continuing goal. Current support is described for implemented and verified environments.

### Current progress

| Area | Current state |
|---|---|
| UI and desktop | Vue and native desktop paths, virtual windows, workspaces, launcher, and desktop surfaces exist, with feature-specific limits |
| Applications and configuration | System applications and separate tools have an integration manifest; the settings UI uses Auto sources for Web and desktop forms |
| Shell and terminal | AutoShell provides structured cross-platform commands and scripting; AutoTerm supplies terminal infrastructure. Common commands use POSIX-style interfaces; compatibility is described per command |
| AI services | Shared model services, clients, and Agent application integrations exist; applications integrate AI incrementally as needed |
| Host interoperability | Native-window and clipboard integration exists for specific platforms; support is assessed per platform and feature |
| Independent systems and kernel | Linux composition-host work is at an early stage; complete distributions, an OpenHarmony-based independent system, and an AutoOS kernel remain future directions |

The consistency goal spans components, commands, service interfaces, notifications, launch, and communication. Coverage is implemented and verified incrementally. UI consistency focuses on layout, interaction, and themes; platform text rendering can still differ.

### Longer-term direction: AI + Lang + OS

AutoOS aims to become a human-centered environment for knowledge management, work, and everyday life. AI has participated deeply in Auto's development and is intended to become a shared service and a means of application coordination. AutoLang supplies the expressive and executable foundation, while the OS and applications carry knowledge, tasks, and work processes.

Existing editors, knowledge tools, boards, shells, and Agent applications provide different starting points. Connecting personal knowledge, tasks, and actions across applications remains work to be done. The aim is to help people understand, choose, and control the system's capabilities, without assuming that every task should be delegated to AI.

## English · AI overview

### AI: shared capabilities and work tools in the Auto ecosystem

AI has participated in Auto's development and is becoming a system capability that applications can use. Current work centers on shared model services, Agent execution, and development tools. The longer-term direction brings AI, language, desktop, and applications together for human-centered knowledge management, work, and everyday life.

[Explore AutoMusk](/apps/automusk/) · [Explore AutoShell](/apps/autoshell/) · [Read the AutoOS overview](/os)

Further reading: [AI in the Auto Ecosystem — From Shared Model Services to Traceable Work](/articles/auto-ai-history).

### Why a shared system layer

Applications using models face common questions: which provider to connect to, how to configure models and credentials, how to manage concurrency, and how to inspect usage. AutoAI puts these shared capabilities into a service so integrated applications can focus on their tasks.

This also allows AI capabilities to extend beyond one chat interface. Command shells, development tools, and future knowledge applications can reuse the infrastructure. Applications integrate the service incrementally according to their tasks and tools.

### Applications, Agents, and services

| Layer | Current responsibility |
|---|---|
| Applications and user interfaces | User tasks, results, progress, operations, and confirmation entry points |
| Agent execution | Roles, skills, tool calls, and workflows |
| Shared client | Normalized requests and responses for the model service |
| `aaid` model service | Configured provider access, concurrency, routing, and usage records |
| Configuration service | Application, role, skill, and model settings |

A typical path is application → Agent/client → aaid → configured model provider. Tool execution belongs to the relevant application or Agent tool layer; the model daemon focuses on model communication and resource coordination.

### Existing applications and integrations

- **AutoMusk** is a development work tool organizing chat, tool activity, and project materials such as Plans, Specs, and Wiki. Planning, execution, review, and knowledge consolidation are separate stages with human confirmation. Its implementation combines Auto sources, generated code, and Rust infrastructure.
- **AutoAI CLI and AutoShell** provide terminal Agent and AI interaction entry points. AutoAI CLI prefers AutoShell for command execution, with explicit handling of failures and policy denials.
- **The configuration center** manages roles, skills, applications, and model settings through a shared configuration entry point.

Capabilities depend on each integration’s tools and workflows, with the corresponding human collaboration and confirmation steps.

### Current capabilities and boundaries

| Area | Capabilities and boundaries |
|---|---|
| Model access | Requests to configured providers with implemented adapters; model availability depends on configuration, credentials, and provider services |
| Request coordination | Shared concurrency management, model routing, and usage records; routing follows configuration and candidate rules, without guaranteeing the “best model” |
| Agent tools | Roles, skills, tool calls, and workflows exist; capabilities depend on application integration and tool configuration |
| System integration | Development tools, shell, and configuration integrations exist; knowledge coordination across applications and a broader personal work environment remain under development |

The model gateway and tools for command execution or file operations are separate layers. Interfaces should make tasks, status, output, verification results, and required confirmation understandable.

### From development tools to a knowledge and work environment

The long-term AI + Lang + OS direction connects language, services, and applications: describe data and operations with the language, connect capabilities through services, and preserve knowledge, tasks, and outcomes in applications.

This remains a long-term product direction. Existing tools provide part of the foundation. A unified personal knowledge system, lasting collaboration across applications, and broader work and everyday-life scenarios still require explicit data, permission, and interaction designs, followed by implementation.

## 配套页面与入口摘要

| 位置 | 中文 | English |
|---|---|---|
| 首页 OS 卡 | AutoOS 的虚拟桌面、系统应用与平台适配；从 LaOS 的实现理念走向独立系统的长期路线。 | The AutoOS virtual desktop, system applications, and platform integration; a long-term path from LaOS to independent systems. |
| 首页 AI 卡 | 共享模型服务与 Agent 工具，连接语言、应用和实际工作；跨应用知识协作仍在建设。 | Shared model services and Agent tools connecting language, applications, and work; knowledge coordination across applications remains under development. |
| AutoOS 桌面页首屏 | 当前桌面：窗口、应用与系统入口。AutoOS 在宿主系统中运行自己的桌面表面与应用，复用 AutoUI 的组件和交互定义。 | The current desktop: windows, applications, and system entry points. AutoOS runs its desktop surfaces and applications within a host system, reusing AutoUI component and interaction definitions. |
| 桌面 UI 页 | AutoUI 的桌面运行路径。相同视图定义可用于 Vue 与原生桌面；运行、生成和平台适配有各自的支持范围。 | AutoUI desktop paths. Shared view definitions support Vue and native desktop environments, with separate coverage for execution, generation, and platform integration. |

配套详情不再使用“100% AutoUI”“所有平台”“零前端代码”“v0.6 首发”“任意模型”等无范围或固定版本承诺。真实截图继续标明来源与形态，架构关系图明确标注为示意，不伪造终端运行记录。
