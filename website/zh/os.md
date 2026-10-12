---
title: AutoOS：理念、组成与当前进展
description: AutoOS：理念、组成与当前进展
---

<script setup>
import OSIntroduction from '../.vitepress/theme/components/OSIntroduction.vue'
import DesktopShowcase from '../.vitepress/theme/components/DesktopShowcase.vue'
import EvidenceImage from '../.vitepress/theme/components/EvidenceImage.vue'
import IntroductionRelation from '../.vitepress/theme/components/IntroductionRelation.vue'
</script>

<OSIntroduction />

## 当前虚拟桌面

<DesktopShowcase embedded />

## 窗口、工作区与启动

AutoUI 应用既可在 Windows/Linux 原生桌面上直接运行，也可在 Windows/Linux 宿主中的 AutoOS 虚拟桌面里运行。虚拟桌面是当前 **OS over OS** 的形态：AutoOS 应用在宿主系统中的桌面环境里运行，与宿主桌面并行。内核、驱动与系统服务继续由宿主提供。

桌面外壳本身使用 AutoUI，组织虚拟窗口、焦点、拖拽、缩放、工作区、任务栏、启动器与通知。应用窗口与桌面表面共享窗口语义，渲染与宿主接入则有各自的职责。

上面的截图从桌面开始，依次展示 Launcher、小游戏与工作布局。小组件来自已经启动并最小化的应用。静态截图记录捕获时的演示状态，不代表所有平台或应用行为都已完成验证。

## 应用与工作工具

`auto-os` 组织桌面表面与应用集成。启动清单声明当前应用的运行路径，主要采用 VM 执行；编译壳与原生应用生成是另外的路径，有各自的集成要求。

图集中可以看到 AutoEdit、Todo、日期与三个小游戏。独立项目还提供 [AutoShell](/zh/apps/autoshell/)、[AutoMusk](/zh/apps/automusk/) 和看板等工具。各应用的集成程度和成熟度不同，源码示例并不自动等同于完整接入的系统应用。

Kanban 展示了数据驱动的看板及其 Web、桌面形态。下面保留的项目截图记录这些界面，不属于 10 月 1 日的桌面截图序列。

<EvidenceImage src="/v05/kanban-web.png" alt="Kanban 看板的 Web 界面" caption="已有项目截图：Kanban Web 界面。" zoom-label="放大查看" close-label="关闭" original-label="打开原图" />

<EvidenceImage src="/v05/kanban-desktop.png" alt="Kanban 看板的桌面界面" caption="已有项目截图：Kanban 桌面界面。" zoom-label="放大查看" close-label="关闭" original-label="打开原图" />

## 共享设置

`auto-os-config` 使用 Auto 源码组织设置 UI，提供 Web 与桌面形态。共享配置包括应用、角色、技能与模型。通用编辑器从支持的数据形状生成表单，具体模块仍可能需要自身的规则和集成。

<EvidenceImage src="/v05/autoos-config-agents.png" alt="设置中心的 Agent 配置界面" caption="已有项目截图：Agent 设置。" zoom-label="放大查看" close-label="关闭" original-label="打开原图" />

<EvidenceImage src="/v05/autoos-config-skills.png" alt="设置中心的技能配置界面" caption="已有项目截图：技能设置。" zoom-label="放大查看" close-label="关闭" original-label="打开原图" />

## 框架、产品与宿主

`auto-lang` 负责语言、UI 框架、执行与渲染契约；`auto-os` 负责桌面表面、应用组装与产品集成。底层执行与系统设施目前结合 Auto 源码、生成代码、Rust 基础设施与既有生态。

宿主应用与系统服务按平台接入。共享 UI 定义以布局、交互和主题一致为目标，平台适配与文本渲染仍会有差异。Windows/Linux 虚拟桌面使用已有宿主系统；完整独立发行版、OpenHarmony 集成和自有内核仍是未来方向。

## 从 Language as OS 到 OS over OS

**Language as OS（LaOS）** 是项目最初的实现理念：用 Auto 组织和实现操作系统的各个部件，让语言及其运行时适配不同的软硬件环境，同时维持对外接口和使用体验的一致性。

这包括 UI 库、合成器、系统应用、配置、启动器、监控、命令 Shell、AI 服务和桌面；内核也是长期设想的一部分，但目前尚未实现自有内核。现阶段的语言运行时与底层系统设施主要建立在 Rust 和既有生态上。未来也可能探索语言底座的自实现原生路径。

**OS over OS** 从 AutoOS 的使用形态描述同一方向：AutoOS 作为虚拟桌面与宿主桌面并行，自带应用在自己的桌面中运行，同时逐步对接宿主应用与系统能力。它是 LaOS 在当前阶段的产品形态，两者承接同一套架构。

## 系统如何组成

<IntroductionRelation kind="os" />

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

## 在不同环境中运行

**宿主中的虚拟桌面**是当前主要形态。它复用宿主的内核、驱动和系统能力，在一个桌面环境中运行 AutoOS 应用，并与宿主桌面共存。宿主应用互操作按平台推进，具体支持取决于窗口系统与接入方式。

**复用现有内核的独立系统**是后续方向。在条件成熟的设备上，AutoOS 可以提供桌面和系统外壳，对接 Linux 内核及必要的系统服务，形成独立发行版；与 OpenHarmony 结合也是长期路线之一。Linux 合成宿主已有早期实现基础，这些独立产品形态尚未完成。

**自有内核的完整系统**属于更长期的研究方向，目前没有自有内核，也没有由它构成的独立 OS。

这些形态共享语言、组件与接口设计，具体平台适配和部署方式仍需分别实现。“一套代码适配不同生态”是持续推进的目标；当前支持按已实现、已验证的环境说明。

## 当前进展

| 领域 | 当前状态 |
|---|---|
| UI 与桌面 | 已有 Vue 与原生桌面运行路径、虚拟窗口、工作区、启动器及桌面表面；各功能仍有具体边界 |
| 应用与配置 | 已有系统应用和独立工具的集成清单；设置中心使用 Auto 源码组织 UI，并支持 Web/桌面形态 |
| Shell 与终端 | AutoShell 提供跨平台结构化命令与脚本接口；AutoTerm 提供终端基础设施。常用命令采用 POSIX 风格接口，具体兼容范围按命令说明 |
| AI 服务 | 已有共享模型服务、客户端与 Agent 应用的接入；应用按需要逐步接入 AI |
| 宿主互操作 | 已有针对具体平台的原生窗口与剪贴板等接入；支持范围按平台和功能核实 |
| 独立系统与内核 | Linux 合成宿主处于早期实现阶段；完整发行版、OpenHarmony 独立系统和自有内核仍属未来方向 |

跨平台一致性覆盖组件、命令、服务接口、通知、启动与通信的设计目标；当前覆盖需要逐项实现和验证。UI 的一致性主要关注布局、交互和主题，平台文本渲染等差异仍会存在。

## 长期方向：AI + Lang + OS

AutoOS 的长期目标是一套面向人的知识管理与工作生活环境。AI 深入参与了 Auto 的开发，也将作为系统服务和应用协作能力继续融入其中；AutoLang 提供可表达、可执行的实现底座；OS 与应用则承载实际的知识、任务和工作过程。

已有编辑器、知识工具、看板、Shell 和 Agent 应用提供了不同入口。让个人知识、任务与应用中的操作形成更连贯的体验，仍是需要继续建设的方向。这一目标不预设所有任务都交给 AI，而是让人能够理解、选择和控制系统提供的能力。
