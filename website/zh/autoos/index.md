---
title: AutoOS 虚拟桌面
description: 当前 AutoOS 桌面、应用、设置与宿主集成。
---

<script setup>
import IntroductionFrame from '../../.vitepress/theme/components/IntroductionFrame.vue'
import DesktopShowcase from '../../.vitepress/theme/components/DesktopShowcase.vue'
import EvidenceImage from '../../.vitepress/theme/components/EvidenceImage.vue'
</script>

<IntroductionFrame kind="desktop" />

<DesktopShowcase embedded />

## 窗口、工作区与启动

虚拟桌面是当前 **OS over OS** 的形态：AutoOS 应用在宿主系统中的桌面环境里运行，与宿主桌面并行。内核、驱动与系统服务继续由宿主提供。

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

宿主应用与系统服务按平台接入。共享 UI 定义以布局、交互和主题一致为目标，平台适配与文本渲染仍会有差异。Linux 合成宿主已有早期工作，完整独立发行版、OpenHarmony 集成和自有内核仍是未来方向。

[了解 OS 的组成与运行形态](/zh/os) · [阅读 AutoOS 历史与展望](/zh/articles/autoos-history) · [了解桌面运行路径](/zh/ui-desktop) · [返回 v0.5](/zh/v05/)
